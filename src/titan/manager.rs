use std::{str::FromStr, sync::Arc};

use serde_json::json;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use url::Url;

use crate::engine::QuoteConfig;
use crate::strategy::types::TradePair;

use super::client::{QuoteStreamItem, TitanWsClient};
use super::error::TitanError;
use super::types::{
    QuoteSwapStreamResponse, QuoteUpdateParams, SwapMode as TitanSwapMode, SwapParams,
    SwapQuoteRequest, SwapQuotes, TransactionParams,
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
    pub providers: &'a [String],
    pub reverse_slippage_bps: Option<u16>,
    pub update_interval_ms: Option<u64>,
    pub update_num_quotes: Option<u32>,
}

pub struct TitanQuoteStream {
    #[allow(dead_code)]
    client: Arc<TitanWsClient>,
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
    let client = Arc::new(TitanWsClient::connect(config.endpoint.clone()).await?);
    log_server_metadata(client.as_ref(), config.strategy_label).await;
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
        let Subscription {
            base_pair,
            request_pair,
            amount,
            leg,
            request,
        } = sub;

        let client_clone = Arc::clone(&client);
        let mut sender = events_tx.clone();
        let strategy = config.strategy_label.to_string();

        let session = match client_clone.subscribe_swap_quotes(request).await {
            Ok(session) => {
                log_stream_start(&strategy, &request_pair, leg, amount, &session.info);
                session
            }
            Err(err) => {
                error!(
                    target: "titan::manager",
                    strategy = config.strategy_label,
                    input_mint = %request_pair.input_mint,
                    output_mint = %request_pair.output_mint,
                    amount,
                    error = %err,
                    "failed to subscribe Titan quote stream"
                );
                continue;
            }
        };

        let stream_id = session.stream_id;
        let mut stream = session.receiver;
        let client_for_stop = Arc::clone(&client_clone);
        let base_pair_clone = base_pair.clone();
        let request_pair_clone = request_pair.clone();
        tokio::spawn(async move {
            while let Some(item) = stream.recv().await {
                match item {
                    QuoteStreamItem::Update { seq, quotes } => {
                        if dispatch_update(
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
                            stream_id = end.id,
                            error_code = ?end.error_code,
                            error_message = ?end.error_message,
                            "Titan quote stream ended"
                        );
                        break;
                    }
                }
            }
            let _ = client_for_stop.stop_stream(stream_id).await;
        });
    }

    drop(events_tx);
    Ok(TitanQuoteStream {
        client,
        receiver: events_rx,
    })
}

async fn log_server_metadata(client: &TitanWsClient, strategy: &str) {
    match client.get_info().await {
        Ok(info) => match serde_json::to_string(&info) {
            Ok(json) => info!(
                target: "titan::manager",
                strategy,
                server_info = %json,
                "Titan GetInfo response"
            ),
            Err(err) => warn!(
                target: "titan::manager",
                strategy,
                error = %err,
                "failed to serialize Titan GetInfo response"
            ),
        },
        Err(err) => warn!(
            target: "titan::manager",
            strategy,
            error = %err,
            "failed to fetch Titan GetInfo"
        ),
    }

    match client.list_providers().await {
        Ok(providers) => {
            let sanitized: Vec<_> = providers
                .into_iter()
                .map(|provider| {
                    let crate::titan::types::ProviderInfo { id, name, kind, .. } = provider;
                    json!({
                        "id": id,
                        "name": name,
                        "kind": kind,
                    })
                })
                .collect();
            match serde_json::to_string(&sanitized) {
                Ok(json) => info!(
                    target: "titan::manager",
                    strategy,
                    providers = %json,
                    "Titan ListProviders response"
                ),
                Err(err) => warn!(
                    target: "titan::manager",
                    strategy,
                    error = %err,
                    "failed to serialize Titan ListProviders response"
                ),
            }
        }
        Err(err) => warn!(
            target: "titan::manager",
            strategy,
            error = %err,
            "failed to fetch Titan provider list"
        ),
    }
}

struct Subscription {
    base_pair: TradePair,
    request_pair: TradePair,
    amount: u64,
    leg: TitanLeg,
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
                amount,
                leg: TitanLeg::Forward,
                request: forward_request,
            });

            let reverse_pair = pair.reversed();
            let reverse_request = build_request(
                config,
                &reverse_pair,
                output,
                input,
                amount,
                TitanLeg::Reverse,
            );
            subs.push(Subscription {
                base_pair: pair.clone(),
                request_pair: reverse_pair,
                amount,
                leg: TitanLeg::Reverse,
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

    let provider_whitelist = {
        let filtered: Vec<String> = config
            .providers
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .collect();
        if filtered.is_empty() {
            None
        } else {
            Some(filtered)
        }
    };

    let swap_mode = match leg {
        TitanLeg::Forward => TitanSwapMode::ExactIn,
        TitanLeg::Reverse => TitanSwapMode::ExactOut,
    };

    let slippage_bps = match leg {
        TitanLeg::Forward => quote.slippage_bps,
        TitanLeg::Reverse => config.reverse_slippage_bps.unwrap_or(quote.slippage_bps),
    };

    let swap = SwapParams {
        input_mint,
        output_mint,
        amount,
        swap_mode: Some(swap_mode),
        slippage_bps: Some(slippage_bps),
        dexes: dex_whitelist,
        exclude_dexes: None,
        only_direct_routes: Some(quote.only_direct_routes),
        add_size_constraint: None,
        size_constraint: None,
        providers: provider_whitelist,
        accounts_limit_total: accounts_limit,
        accounts_limit_writable: accounts_limit,
    };

    let transaction = TransactionParams {
        user_public_key: config.user_pubkey,
        close_input_token_account: match leg {
            TitanLeg::Forward => Some(false),
            TitanLeg::Reverse => None,
        },
        create_output_token_account: match leg {
            TitanLeg::Forward => Some(false),
            TitanLeg::Reverse => None,
        },
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
        slippage_bps = slippage_bps,
        only_direct = quote.only_direct_routes,
        accounts_limit = ?accounts_limit,
        providers = ?config.providers,
        "prepared Titan subscription"
    );

    let update = match (config.update_interval_ms, config.update_num_quotes) {
        (None, None) => None,
        _ => Some(QuoteUpdateParams {
            interval_ms: config.update_interval_ms,
            num_quotes: config.update_num_quotes,
        }),
    };

    SwapQuoteRequest {
        swap,
        transaction,
        update,
    }
}

async fn dispatch_update(
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
