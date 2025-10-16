use std::{str::FromStr, sync::Arc, time::Duration};

use serde_json::json;
use tokio::{sync::mpsc, time::timeout};
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

#[derive(Clone)]
struct ReverseRequestContext {
    user_pubkey: solana_sdk::pubkey::Pubkey,
    dex_whitelist: Option<Vec<String>>,
    providers: Vec<String>,
    only_direct_routes: bool,
    accounts_limit: Option<u16>,
    forward_slippage_bps: u16,
    reverse_slippage_bps: Option<u16>,
    update_interval_ms: Option<u64>,
    update_num_quotes: Option<u32>,
}

impl ReverseRequestContext {
    fn from_stream_config(config: &TitanStreamConfig<'_>) -> Self {
        let accounts_limit = config.quote.quote_max_accounts.and_then(|value| {
            if value == 0 {
                None
            } else if value > u16::MAX as u32 {
                Some(u16::MAX)
            } else {
                Some(value as u16)
            }
        });

        let dex_whitelist = if config.quote.dex_whitelist.is_empty() {
            None
        } else {
            Some(config.quote.dex_whitelist.clone())
        };

        let providers = config
            .providers
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .collect();

        Self {
            user_pubkey: config.user_pubkey,
            dex_whitelist,
            providers,
            only_direct_routes: config.quote.only_direct_routes,
            accounts_limit,
            forward_slippage_bps: config.quote.slippage_bps,
            reverse_slippage_bps: config.reverse_slippage_bps,
            update_interval_ms: config.update_interval_ms,
            update_num_quotes: config.update_num_quotes,
        }
    }

    fn effective_slippage(&self) -> u16 {
        self.reverse_slippage_bps
            .unwrap_or(self.forward_slippage_bps)
    }
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
    let reverse_context = Arc::new(ReverseRequestContext::from_stream_config(&config));
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
            request,
            ..
        } = sub;

        let client_clone = Arc::clone(&client);
        let reverse_context = Arc::clone(&reverse_context);
        let mut forward_sender = events_tx.clone();
        let reverse_sender = events_tx.clone();
        let strategy = config.strategy_label.to_string();

        let session = match client_clone.subscribe_swap_quotes(request).await {
            Ok(session) => {
                log_stream_start(
                    &strategy,
                    &request_pair,
                    TitanLeg::Forward,
                    amount,
                    &session.info,
                );
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
                        let reverse_amount = select_forward_out_amount(&quotes);
                        if forward_update(
                            &mut forward_sender,
                            &strategy,
                            &base_pair_clone,
                            TitanLeg::Forward,
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

                        if let Some(reverse_amount) = reverse_amount {
                            if let Some(reverse_signal) = request_reverse_once(
                                Arc::clone(&client_clone),
                                Arc::clone(&reverse_context),
                                &base_pair_clone,
                                amount,
                                reverse_amount,
                                &strategy,
                            )
                            .await
                            {
                                if reverse_sender.send(reverse_signal).await.is_err() {
                                    debug!(
                                        target: "titan::manager",
                                        strategy = strategy,
                                        input_mint = %request_pair_clone.input_mint,
                                        output_mint = %request_pair_clone.output_mint,
                                        amount,
                                        "dropping Titan reverse update (receiver closed)"
                                    );
                                    break;
                                }
                            }
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
                request: forward_request,
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
        close_input_token_account: Some(false),
        create_output_token_account: Some(false),
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

fn select_forward_out_amount(quotes: &SwapQuotes) -> Option<u64> {
    quotes
        .quotes
        .values()
        .max_by_key(|route| route.out_amount)
        .map(|route| route.out_amount)
}

async fn request_reverse_once(
    client: Arc<TitanWsClient>,
    context: Arc<ReverseRequestContext>,
    base_pair: &TradePair,
    base_amount: u64,
    reverse_amount: u64,
    strategy: &str,
) -> Option<TitanQuoteSignal> {
    if reverse_amount == 0 {
        return None;
    }

    let reverse_pair = base_pair.reversed();
    let input_mint = match solana_sdk::pubkey::Pubkey::from_str(&reverse_pair.input_mint) {
        Ok(value) => value,
        Err(err) => {
            debug!(
                target: "titan::manager",
                strategy,
                input_mint = %reverse_pair.input_mint,
                "skip reverse quote: invalid input mint ({err})"
            );
            return None;
        }
    };
    let output_mint = match solana_sdk::pubkey::Pubkey::from_str(&reverse_pair.output_mint) {
        Ok(value) => value,
        Err(err) => {
            debug!(
                target: "titan::manager",
                strategy,
                output_mint = %reverse_pair.output_mint,
                "skip reverse quote: invalid output mint ({err})"
            );
            return None;
        }
    };

    let swap = SwapParams {
        input_mint,
        output_mint,
        amount: base_amount,
        swap_mode: Some(TitanSwapMode::ExactOut),
        slippage_bps: Some(context.effective_slippage()),
        dexes: context.dex_whitelist.clone(),
        exclude_dexes: None,
        only_direct_routes: Some(context.only_direct_routes),
        add_size_constraint: None,
        size_constraint: None,
        providers: if context.providers.is_empty() {
            None
        } else {
            Some(context.providers.clone())
        },
        accounts_limit_total: context.accounts_limit,
        accounts_limit_writable: context.accounts_limit,
    };

    let transaction = TransactionParams {
        user_public_key: context.user_pubkey,
        close_input_token_account: None,
        create_output_token_account: None,
        fee_account: None,
        fee_bps: None,
        fee_from_input_mint: None,
        output_account: None,
    };

    let update = match (context.update_interval_ms, context.update_num_quotes) {
        (None, None) => None,
        _ => Some(QuoteUpdateParams {
            interval_ms: context.update_interval_ms,
            num_quotes: context.update_num_quotes,
        }),
    };

    let request = SwapQuoteRequest {
        swap,
        transaction,
        update,
    };

    let session = match client.subscribe_swap_quotes(request).await {
        Ok(session) => session,
        Err(err) => {
            debug!(
                target: "titan::manager",
                strategy,
                error = %err,
                "failed to open reverse Titan stream"
            );
            return None;
        }
    };

    let stream_id = session.stream_id;
    let mut receiver = session.receiver;

    let interval_hint_ms = context.update_interval_ms.unwrap_or(800);
    let timeout_ms = interval_hint_ms.max(800).saturating_add(400);
    let timeout_duration = Duration::from_millis(timeout_ms.min(10_000));

    let item = match timeout(timeout_duration, receiver.recv()).await {
        Ok(Some(item)) => item,
        Ok(None) => {
            debug!(
                target: "titan::manager",
                strategy,
                "reverse Titan stream closed without data"
            );
            let _ = client.stop_stream(stream_id).await;
            return None;
        }
        Err(_) => {
            debug!(
                target: "titan::manager",
                strategy,
                "reverse Titan stream timed out without data"
            );
            let _ = client.stop_stream(stream_id).await;
            return None;
        }
    };

    let result = match item {
        QuoteStreamItem::Update { seq, quotes } => Some(TitanQuoteSignal {
            base_pair: base_pair.clone(),
            leg: TitanLeg::Reverse,
            amount: base_amount,
            seq,
            quotes,
        }),
        QuoteStreamItem::End(_) => None,
    };

    let _ = client.stop_stream(stream_id).await;
    result
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
