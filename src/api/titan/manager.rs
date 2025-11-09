use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use solana_sdk::pubkey::Pubkey;
use tokio::sync::mpsc;
use tracing::{debug, error, warn};
use url::Url;

use crate::monitoring::short_mint_str;
use crate::strategy::types::TradePair;

use super::client::{QuoteStreamItem, TitanWsClient};
use super::error::TitanError;
use super::types::{
    QuoteSwapStreamResponse, QuoteUpdateParams, StreamId, SwapMode, SwapParams, SwapQuoteRequest,
    SwapQuotes, TransactionParams,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TitanLeg {
    Forward,
    Reverse,
}

#[derive(Clone, Debug)]
pub struct TitanSubscriptionConfig {
    pub ws_url: Url,
    pub ws_proxy: Option<Url>,
    pub user_pubkey: Pubkey,
    pub providers: Vec<String>,
    pub dexes: Vec<String>,
    pub exclude_dexes: Vec<String>,
    pub only_direct_routes: Option<bool>,
    pub update_interval_ms: Option<u64>,
    pub update_num_quotes: Option<u32>,
    pub close_input_token_account: bool,
    pub create_output_token_account: Option<bool>,
    pub idle_resubscribe_timeout: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct TitanQuoteUpdate {
    pub seq: u64,
    pub quotes: SwapQuotes,
}

pub struct TitanQuoteStream {
    #[allow(dead_code)]
    client: Arc<TitanWsClient>,
    pub info: QuoteSwapStreamResponse,
    pub stream_id: StreamId,
    receiver: mpsc::Receiver<TitanQuoteUpdate>,
}

impl TitanQuoteStream {
    pub async fn recv(&mut self) -> Option<TitanQuoteUpdate> {
        self.receiver.recv().await
    }
}

pub async fn subscribe_quote_stream(
    config: TitanSubscriptionConfig,
    pair: &TradePair,
    leg: TitanLeg,
    amount: u64,
    local_ip: Option<IpAddr>,
    jwt: &str,
) -> Result<TitanQuoteStream, TitanError> {
    if amount == 0 {
        return Err(TitanError::Protocol(
            "Titan subscription amount cannot be zero".into(),
        ));
    }

    let endpoint = build_authenticated_endpoint(&config, jwt)?;
    let client = Arc::new(
        TitanWsClient::connect_with_options(endpoint, config.ws_proxy.clone(), local_ip).await?,
    );

    let request = build_subscription_request(&config, pair, leg, amount);
    let session = client.subscribe_swap_quotes(request).await.map_err(|err| {
        error!(
            target: "titan::manager",
            input_mint = %pair.input_mint,
            output_mint = %pair.output_mint,
            amount,
            leg = ?leg,
            "failed to subscribe Titan stream: {err}"
        );
        err
    })?;

    let base_display = short_mint_str(pair.input_mint.as_str());
    let quote_display = short_mint_str(pair.output_mint.as_str());
    debug!(
        target: "titan::manager",
        base_mint = %base_display,
        quote_mint = %quote_display,
        amount,
        leg = ?leg,
        interval_ms = session.info.interval_ms,
        providers = ?config.providers,
        "Titan quote stream established"
    );

    let stream_id = session.stream_id;
    let info = session.info.clone();
    let (update_tx, update_rx) = mpsc::channel(128);
    let client_for_stop = Arc::clone(&client);
    let mut receiver = session.receiver;

    tokio::spawn(async move {
        let mut update_tx = Some(update_tx);
        while let Some(item) = receiver.recv().await {
            match item {
                QuoteStreamItem::Update { seq, quotes } => {
                    if let Some(sender) = update_tx.as_mut() {
                        if sender.send(TitanQuoteUpdate { seq, quotes }).await.is_err() {
                            debug!(
                                target: "titan::manager",
                                stream_id,
                                seq,
                                "Titan update receiver dropped"
                            );
                            update_tx = None;
                            break;
                        }
                    } else {
                        break;
                    }
                }
                QuoteStreamItem::End(end) => {
                    if let Some(code) = end.error_code {
                        warn!(
                            target: "titan::manager",
                            stream_id,
                            code,
                            error = ?end.error_message,
                            "Titan stream ended with error"
                        );
                    } else {
                        debug!(target: "titan::manager", stream_id, "Titan stream ended");
                    }
                    break;
                }
            }
        }

        if let Err(err) = client_for_stop.stop_stream(stream_id).await {
            debug!(
                target: "titan::manager",
                stream_id,
                "failed to stop Titan stream cleanly: {err}"
            );
        }
    });

    Ok(TitanQuoteStream {
        client,
        info,
        stream_id,
        receiver: update_rx,
    })
}

fn build_authenticated_endpoint(
    config: &TitanSubscriptionConfig,
    jwt: &str,
) -> Result<Url, TitanError> {
    if jwt.trim().is_empty() {
        return Err(TitanError::MissingAuthToken);
    }

    let mut endpoint = config.ws_url.clone();
    endpoint.set_query(None);
    endpoint.query_pairs_mut().append_pair("auth", jwt);
    Ok(endpoint)
}

fn build_subscription_request(
    config: &TitanSubscriptionConfig,
    pair: &TradePair,
    leg: TitanLeg,
    amount: u64,
) -> SwapQuoteRequest {
    let (input_mint, output_mint) = match leg {
        TitanLeg::Forward => (pair.input_pubkey, pair.output_pubkey),
        TitanLeg::Reverse => (pair.output_pubkey, pair.input_pubkey),
    };

    let providers = if config.providers.is_empty() {
        None
    } else {
        Some(config.providers.clone())
    };
    let dexes = if config.dexes.is_empty() {
        None
    } else {
        Some(config.dexes.clone())
    };
    let exclude_dexes = if config.exclude_dexes.is_empty() {
        None
    } else {
        Some(config.exclude_dexes.clone())
    };
    let only_direct_routes = config.only_direct_routes.unwrap_or(false);

    let update = match (config.update_interval_ms, config.update_num_quotes) {
        (None, None) => None,
        _ => Some(QuoteUpdateParams {
            interval_ms: config.update_interval_ms,
            num_quotes: config.update_num_quotes,
        }),
    };

    let slippage_bps = Some(0);

    let swap = SwapParams {
        input_mint,
        output_mint,
        amount,
        swap_mode: Some(match leg {
            TitanLeg::Forward => SwapMode::ExactIn,
            TitanLeg::Reverse => SwapMode::ExactOut,
        }),
        slippage_bps,
        dexes,
        exclude_dexes,
        only_direct_routes: Some(only_direct_routes),
        add_size_constraint: None,
        size_constraint: None,
        providers,
        accounts_limit_total: None,
        accounts_limit_writable: None,
    };

    let transaction = TransactionParams {
        user_public_key: config.user_pubkey,
        close_input_token_account: Some(config.close_input_token_account),
        create_output_token_account: config.create_output_token_account,
        fee_account: None,
        fee_bps: None,
        fee_from_input_mint: None,
        output_account: None,
    };

    SwapQuoteRequest {
        swap,
        transaction,
        update,
    }
}
