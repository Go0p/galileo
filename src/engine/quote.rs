use std::convert::TryFrom;
use std::str::FromStr;

use tracing::{debug, warn};

use super::aggregator::QuoteResponseVariant;
use crate::api::dflow::{
    DflowApiClient, DflowError, QuoteRequest as DflowQuoteRequest, SlippageBps, SlippagePreset,
};
use crate::api::jupiter::{JupiterApiClient, QuoteRequest};
use crate::api::ultra::{OrderRequest as UltraOrderRequest, Router as UltraRouter, UltraApiClient};
use crate::config::{DflowQuoteConfig, JupiterQuoteConfig, UltraQuoteConfig};
use crate::strategy::types::TradePair;
use solana_sdk::pubkey::Pubkey;

use super::error::{EngineError, EngineResult};
use super::types::{DoubleQuote, QuoteTask};

#[derive(Debug, Clone)]
pub struct QuoteConfig {
    pub slippage_bps: u16,
    pub only_direct_routes: bool,
    pub restrict_intermediate_tokens: bool,
    pub quote_max_accounts: Option<u32>,
    pub dex_whitelist: Vec<String>,
    pub dex_blacklist: Vec<String>,
}

impl QuoteConfig {}

#[derive(Clone)]
pub enum QuoteBackend {
    Jupiter {
        client: JupiterApiClient,
        defaults: JupiterQuoteConfig,
    },
    Dflow {
        client: DflowApiClient,
        defaults: DflowQuoteConfig,
    },
    Ultra {
        client: UltraApiClient,
        defaults: UltraQuoteDefaults,
    },
    Disabled,
}

#[derive(Clone)]
pub struct UltraQuoteDefaults {
    pub config: UltraQuoteConfig,
    pub taker: Option<Pubkey>,
    pub include_routers: Option<String>,
}

impl UltraQuoteDefaults {
    fn new(config: UltraQuoteConfig) -> Self {
        let taker = config.taker.as_ref().and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                match Pubkey::from_str(trimmed) {
                    Ok(pk) => Some(pk),
                    Err(err) => {
                        warn!(
                            target: "engine::quote",
                            taker = %value,
                            error = %err,
                            "Ultra taker 配置解析失败，忽略该字段"
                        );
                        None
                    }
                }
            }
        });
        let include_routers = if config.include_routers.is_empty() {
            None
        } else {
            Some(
                config
                    .include_routers
                    .iter()
                    .map(|label| label.to_ascii_lowercase())
                    .collect::<Vec<_>>()
                    .join(","),
            )
        };
        Self {
            config,
            taker,
            include_routers,
        }
    }
}

#[derive(Clone)]
pub struct QuoteExecutor {
    backend: QuoteBackend,
}

impl QuoteExecutor {
    pub fn for_jupiter(client: JupiterApiClient, defaults: JupiterQuoteConfig) -> Self {
        Self {
            backend: QuoteBackend::Jupiter { client, defaults },
        }
    }

    pub fn for_dflow(client: DflowApiClient, defaults: DflowQuoteConfig) -> Self {
        Self {
            backend: QuoteBackend::Dflow { client, defaults },
        }
    }

    pub fn for_ultra(client: UltraApiClient, defaults: UltraQuoteConfig) -> Self {
        Self {
            backend: QuoteBackend::Ultra {
                client,
                defaults: UltraQuoteDefaults::new(defaults),
            },
        }
    }

    pub fn disabled() -> Self {
        Self {
            backend: QuoteBackend::Disabled,
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn round_trip(
        &self,
        task: &QuoteTask,
        config: &QuoteConfig,
    ) -> EngineResult<Option<DoubleQuote>> {
        let forward = match self.quote_once(&task.pair, task.amount, config).await? {
            Some(value) => value,
            None => return Ok(None),
        };

        let first_out = forward.out_amount() as u128;
        if first_out == 0 {
            debug!(
                target: "engine::quote",
                input = %task.pair.input_mint,
                output = %task.pair.output_mint,
                amount = task.amount,
                "首腿报价输出为零，跳过"
            );
            return Ok(None);
        }

        let second_amount = match u64::try_from(first_out) {
            Ok(value) => value,
            Err(_) => {
                debug!(
                    target: "engine::quote",
                    amount = first_out,
                    "首腿输出超过 u64，可疑路线，跳过"
                );
                return Ok(None);
            }
        };

        let reverse_pair = task.pair.reversed();
        let reverse = match self
            .quote_once(&reverse_pair, second_amount, config)
            .await?
        {
            Some(value) => value,
            None => return Ok(None),
        };

        if forward.kind() != reverse.kind() {
            debug!(
                target: "engine::quote",
                forward_kind = ?forward.kind(),
                reverse_kind = ?reverse.kind(),
                "前后腿聚合器类型不一致，跳过"
            );
            return Ok(None);
        }

        Ok(Some(DoubleQuote { forward, reverse }))
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn quote_once(
        &self,
        pair: &TradePair,
        amount: u64,
        config: &QuoteConfig,
    ) -> EngineResult<Option<QuoteResponseVariant>> {
        match &self.backend {
            QuoteBackend::Jupiter { client, defaults } => {
                let mut request = QuoteRequest::new(
                    pair.input_pubkey,
                    pair.output_pubkey,
                    amount,
                    config.slippage_bps,
                );
                request.only_direct_routes = Some(config.only_direct_routes);
                request.restrict_intermediate_tokens = Some(config.restrict_intermediate_tokens);
                if let Some(max_accounts) = config.quote_max_accounts {
                    request.max_accounts = Some(max_accounts as usize);
                }

                if !config.dex_whitelist.is_empty() {
                    let dexes = config.dex_whitelist.join(",");
                    request.dexes = Some(dexes);
                }
                if !config.dex_blacklist.is_empty() {
                    let excluded = config.dex_blacklist.join(",");
                    request.excluded_dexes = Some(excluded);
                }

                apply_jupiter_defaults(defaults, &mut request);
                let response = client.quote(&request).await?;
                Ok(Some(QuoteResponseVariant::Jupiter(response)))
            }
            QuoteBackend::Dflow { client, defaults } => {
                let mut request =
                    DflowQuoteRequest::new(pair.input_pubkey, pair.output_pubkey, amount);
                if config.only_direct_routes {
                    request.only_direct_routes = Some(true);
                }
                if !config.dex_whitelist.is_empty() {
                    request.dexes = Some(config.dex_whitelist.join(","));
                }
                if !config.dex_blacklist.is_empty() {
                    request.exclude_dexes = Some(config.dex_blacklist.join(","));
                }
                if let Some(max_route_length) = defaults.max_route_length {
                    request.max_route_length = Some(max_route_length);
                }
                if defaults.use_auto_slippage {
                    request.slippage_bps = Some(SlippageBps::Preset(SlippagePreset::Auto));
                } else {
                    request.slippage_bps = Some(SlippageBps::Fixed(config.slippage_bps));
                }

                match client.quote(&request).await {
                    Ok(response) => Ok(Some(QuoteResponseVariant::Dflow(response))),
                    Err(DflowError::RateLimited { status, body, .. }) => {
                        warn!(
                            target: "engine::quote",
                            status = status.as_u16(),
                            input_mint = %pair.input_mint,
                            output_mint = %pair.output_mint,
                            "DFlow 报价命中限流，跳过本轮: {body}"
                        );
                        Ok(None)
                    }
                    Err(err @ DflowError::ConsecutiveFailureLimit { .. }) => {
                        Err(EngineError::from(err))
                    }
                    Err(err) => {
                        let detail = err.describe();
                        warn!(
                            target: "engine::quote",
                            input_mint = %pair.input_mint,
                            output_mint = %pair.output_mint,
                            amount = amount,
                            error = %detail,
                            "DFlow 报价失败，跳过当前路线"
                        );
                        Ok(None)
                    }
                }
            }
            QuoteBackend::Ultra { client, defaults } => {
                let cfg = &defaults.config;
                let mut request =
                    UltraOrderRequest::new(pair.input_pubkey, pair.output_pubkey, amount);
                request.slippage_bps = config.slippage_bps;
                request.use_wsol = cfg.use_wsol;
                request.taker = defaults.taker;
                if let Some(ref fee_type) = cfg.broadcast_fee_type {
                    let trimmed = fee_type.trim();
                    if !trimmed.is_empty() {
                        request.broadcast_fee_type = Some(trimmed.to_string());
                    }
                } else {
                    request.broadcast_fee_type = Some("exactFee".to_string());
                }
                if let Some(tip) = cfg.jito_tip_lamports {
                    if tip > 0 {
                        request.jito_tip_lamports = Some(tip);
                    }
                }
                if let Some(priority) = cfg.priority_fee_lamports {
                    if priority > 0 {
                        request.priority_fee_lamports = Some(priority);
                    }
                }
                if let Some(ref routers) = defaults.include_routers {
                    request
                        .extra_query_params
                        .insert("routers".to_string(), routers.clone());
                }
                if !cfg.exclude_routers.is_empty() {
                    let mut parsed = Vec::with_capacity(cfg.exclude_routers.len());
                    for label in &cfg.exclude_routers {
                        let router = parse_ultra_router(label)?;
                        parsed.push(router);
                    }
                    request.exclude_routers = parsed;
                }
                if !config.dex_blacklist.is_empty() {
                    request.exclude_dexes = Some(config.dex_blacklist.join(","));
                }
                let response = client.order(&request).await?;
                Ok(Some(QuoteResponseVariant::Ultra(response)))
            }
            QuoteBackend::Disabled => {
                warn!(
                    target: "engine::quote",
                    input_mint = %pair.input_mint,
                    output_mint = %pair.output_mint,
                    amount,
                    "quote backend 已禁用，忽略该报价任务"
                );
                Ok(None)
            }
        }
    }
}

fn apply_jupiter_defaults(defaults: &JupiterQuoteConfig, request: &mut QuoteRequest) {
    if !request.only_direct_routes.unwrap_or(false) && defaults.only_direct_routes {
        request.only_direct_routes = Some(true);
    }

    if request.restrict_intermediate_tokens.unwrap_or(true)
        && !defaults.restrict_intermediate_tokens
    {
        request.restrict_intermediate_tokens = Some(false);
    }
}

fn parse_ultra_router(label: &str) -> EngineResult<UltraRouter> {
    let normalized = label.trim().to_ascii_lowercase();
    let router = match normalized.as_str() {
        "metis" => UltraRouter::metis(),
        "jupiterz" => UltraRouter::jupiterz(),
        "dflow" => UltraRouter::dflow(),
        "okx" => UltraRouter::okx(),
        other => {
            return Err(EngineError::InvalidConfig(format!(
                "未知的 Ultra router: {other}"
            )));
        }
    };
    Ok(router)
}
