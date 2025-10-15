use std::str::FromStr;

use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use url::Url;

use crate::engine::QuoteConfig;
use crate::strategy::types::TradePair;

use super::client::{QuoteStreamItem, TitanWsClient};
use super::error::TitanError;
use super::types::{
    QuoteSwapStreamResponse, SwapMode as TitanSwapMode, SwapParams, SwapQuoteRequest, SwapQuotes,
    TransactionParams,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TitanLeg {
    Forward,
    Reverse,
}

#[derive(Clone, Debug)]
pub struct TitanQuoteSignal {
    pub base_pair: TradePair,
    pub leg: TitanLeg,
    pub amount: u64,
    pub seq: u64,
    pub quotes: SwapQuotes,
}

pub struct TitanStreamConfig<'a> {
    pub endpoint: Url,
    pub user_pubkey: solana_sdk::pubkey::Pubkey,
    pub trade_pairs: &'a [TradePair],
    pub trade_amounts: &'a [u64],
    pub quote: &'a QuoteConfig,
    pub strategy_label: &'a str,
}

pub struct TitanQuoteStream {
    #[allow(dead_code)]
    client: TitanWsClient,
    receiver: mpsc::Receiver<TitanQuoteSignal>,
}

impl TitanQuoteStream {
    pub async fn recv(&mut self) -> Option<TitanQuoteSignal> {
        self.receiver.recv().await
    }
}

pub async fn spawn_quote_streams(
    config: TitanStreamConfig<'_>,
) -> Result<TitanQuoteStream, TitanError> {
    let client = TitanWsClient::connect(config.endpoint.clone()).await?;
    let subscriptions = prepare_subscriptions(&config)?;

    if subscriptions.is_empty() {
        warn!(
            target: "titan::manager",
            strategy = config.strategy_label,
            "no valid Titan subscriptions were prepared; stream updates disabled"
        );
        let (_tx, receiver) = mpsc::channel(1);
        return Ok(TitanQuoteStream { client, receiver });
    }

    let (events_tx, events_rx) = mpsc::channel(512);

    for sub in subscriptions {
        let base_pair = sub.base_pair.clone();
        let request_pair = sub.request_pair.clone();
        let leg = sub.leg;
        let amount = sub.amount;

        let mut stream = match client.subscribe_swap_quotes(sub.request).await {
            Ok((info, stream)) => {
                log_stream_start(config.strategy_label, &request_pair, leg, amount, &info);
                stream
            }
            Err(err) => {
                error!(
                    target: "titan::manager",
                    strategy = config.strategy_label,
                    input_mint = %request_pair.input_mint,
                    output_mint = %request_pair.output_mint,
                    amount,
                    leg = ?leg,
                    error = %err,
                    "failed to subscribe Titan quote stream"
                );
                continue;
            }
        };

        let mut sender = events_tx.clone();
        let strategy = config.strategy_label.to_string();
        let base_pair_clone = base_pair.clone();
        let request_pair_clone = request_pair.clone();
        tokio::spawn(async move {
            while let Some(item) = stream.recv().await {
                match item {
                    QuoteStreamItem::Update { seq, quotes } => {
                        if forward_update(
                            &mut sender,
                            &strategy,
                            &base_pair_clone,
                            leg,
                            amount,
                            seq,
                            quotes,
                        )
                        .await
                        .is_err()
                        {
                            debug!(
                                target: "titan::manager",
                                strategy = strategy,
                                input_mint = %request_pair_clone.input_mint,
                                output_mint = %request_pair_clone.output_mint,
                                amount,
                                leg = ?leg,
                                "dropping Titan stream update (receiver closed)"
                            );
                            break;
                        }
                    }
                    QuoteStreamItem::End(end) => {
                        warn!(
                            target: "titan::manager",
                            strategy = strategy,
                            input_mint = %request_pair_clone.input_mint,
                            output_mint = %request_pair_clone.output_mint,
                            amount,
                            leg = ?leg,
                            stream_id = end.id,
                            error_code = ?end.error_code,
                            error_message = ?end.error_message,
                            "Titan quote stream ended"
                        );
                        break;
                    }
                }
            }
        });
    }

    drop(events_tx);
    Ok(TitanQuoteStream {
        client,
        receiver: events_rx,
    })
}

struct Subscription {
    base_pair: TradePair,
    request_pair: TradePair,
    leg: TitanLeg,
    amount: u64,
    request: SwapQuoteRequest,
}

fn prepare_subscriptions(config: &TitanStreamConfig<'_>) -> Result<Vec<Subscription>, TitanError> {
    let mut subs = Vec::new();

    for pair in config.trade_pairs {
        let input = match solana_sdk::pubkey::Pubkey::from_str(&pair.input_mint) {
            Ok(value) => value,
            Err(err) => {
                warn!(
                    target: "titan::manager",
                    strategy = config.strategy_label,
                    input_mint = %pair.input_mint,
                    "skip Titan subscription: invalid input mint ({err})"
                );
                continue;
            }
        };
        let output = match solana_sdk::pubkey::Pubkey::from_str(&pair.output_mint) {
            Ok(value) => value,
            Err(err) => {
                warn!(
                    target: "titan::manager",
                    strategy = config.strategy_label,
                    output_mint = %pair.output_mint,
                    "skip Titan subscription: invalid output mint ({err})"
                );
                continue;
            }
        };

        for &amount in config.trade_amounts {
            if amount == 0 {
                debug!(
                    target: "titan::manager",
                    strategy = config.strategy_label,
                    input_mint = %pair.input_mint,
                    output_mint = %pair.output_mint,
                    "skip Titan subscription: amount is zero"
                );
                continue;
            }

            let forward_request =
                build_request(config, pair, input, output, amount, TitanLeg::Forward);
            subs.push(Subscription {
                base_pair: pair.clone(),
                request_pair: pair.clone(),
                leg: TitanLeg::Forward,
                amount,
                request: forward_request,
            });

            let reverse_pair = pair.reversed();
            let reverse_input = match solana_sdk::pubkey::Pubkey::from_str(&reverse_pair.input_mint)
            {
                Ok(value) => value,
                Err(err) => {
                    warn!(
                        target: "titan::manager",
                        strategy = config.strategy_label,
                        input_mint = %reverse_pair.input_mint,
                        "skip Titan reverse subscription: invalid input mint ({err})"
                    );
                    continue;
                }
            };
            let reverse_output =
                match solana_sdk::pubkey::Pubkey::from_str(&reverse_pair.output_mint) {
                    Ok(value) => value,
                    Err(err) => {
                        warn!(
                            target: "titan::manager",
                            strategy = config.strategy_label,
                            output_mint = %reverse_pair.output_mint,
                            "skip Titan reverse subscription: invalid output mint ({err})"
                        );
                        continue;
                    }
                };
            let reverse_request = build_request(
                config,
                &reverse_pair,
                reverse_input,
                reverse_output,
                amount,
                TitanLeg::Reverse,
            );
            subs.push(Subscription {
                base_pair: pair.clone(),
                request_pair: reverse_pair,
                leg: TitanLeg::Reverse,
                amount,
                request: reverse_request,
            });
        }
    }

    Ok(subs)
}

fn build_request(
    config: &TitanStreamConfig<'_>,
    display_pair: &TradePair,
    input_mint: solana_sdk::pubkey::Pubkey,
    output_mint: solana_sdk::pubkey::Pubkey,
    amount: u64,
    leg: TitanLeg,
) -> SwapQuoteRequest {
    let quote = config.quote;
    let accounts_limit = quote.quote_max_accounts.and_then(|value| {
        if value == 0 {
            None
        } else if value > u16::MAX as u32 {
            Some(u16::MAX)
        } else {
            Some(value as u16)
        }
    });

    let dex_whitelist = if quote.dex_whitelist.is_empty() {
        None
    } else {
        Some(quote.dex_whitelist.clone())
    };

    let swap_mode = match leg {
        TitanLeg::Forward => TitanSwapMode::ExactIn,
        TitanLeg::Reverse => TitanSwapMode::ExactOut,
    };

    let swap = SwapParams {
        input_mint,
        output_mint,
        amount,
        swap_mode: Some(swap_mode),
        slippage_bps: Some(quote.slippage_bps),
        dexes: dex_whitelist,
        exclude_dexes: None,
        only_direct_routes: Some(quote.only_direct_routes),
        add_size_constraint: None,
        size_constraint: None,
        providers: None,
        accounts_limit_total: accounts_limit,
        accounts_limit_writable: accounts_limit,
    };

    let transaction = TransactionParams {
        user_public_key: config.user_pubkey,
        close_input_token_account: None,
        create_output_token_account: None,
        fee_account: None,
        fee_bps: None,
        fee_from_input_mint: None,
        output_account: None,
    };

    debug!(
        target: "titan::manager",
        strategy = config.strategy_label,
        input_mint = %display_pair.input_mint,
        output_mint = %display_pair.output_mint,
        amount,
        leg = ?leg,
        slippage_bps = quote.slippage_bps,
        only_direct = quote.only_direct_routes,
        accounts_limit = ?accounts_limit,
        "prepared Titan subscription"
    );

    SwapQuoteRequest {
        swap,
        transaction,
        update: None,
    }
}

async fn forward_update(
    sender: &mut mpsc::Sender<TitanQuoteSignal>,
    strategy: &str,
    base_pair: &TradePair,
    leg: TitanLeg,
    amount: u64,
    seq: u64,
    quotes: SwapQuotes,
) -> Result<(), ()> {
    let signal = TitanQuoteSignal {
        base_pair: base_pair.clone(),
        leg,
        amount,
        seq,
        quotes,
    };

    if sender.send(signal).await.is_err() {
        warn!(
            target: "titan::manager",
            strategy,
            input_mint = %base_pair.input_mint,
            output_mint = %base_pair.output_mint,
            amount,
            leg = ?leg,
            "Titan event channel closed; stopping stream forwarding"
        );
        return Err(());
    }

    Ok(())
}

fn log_stream_start(
    strategy: &str,
    pair: &TradePair,
    leg: TitanLeg,
    amount: u64,
    info: &QuoteSwapStreamResponse,
) {
    info!(
        target: "titan::manager",
        strategy,
        input_mint = %pair.input_mint,
        output_mint = %pair.output_mint,
        amount,
        leg = ?leg,
        interval_ms = info.interval_ms,
        "Titan quote stream subscribed"
    );
}
