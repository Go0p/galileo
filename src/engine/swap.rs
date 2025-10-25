use std::str::FromStr;

use tracing::warn;

use super::aggregator::{QuotePayloadVariant, SwapInstructionsVariant};
use super::error::{EngineError, EngineResult};
use super::identity::EngineIdentity;
use super::types::SwapOpportunity;
use crate::api::dflow::{
    ComputeUnitPriceMicroLamports as DflowComputeUnitPriceMicroLamports, DflowApiClient,
    SwapInstructionsRequest as DflowSwapInstructionsRequest,
};
use crate::api::jupiter::{
    ComputeUnitPriceMicroLamports, JupiterApiClient, SwapInstructionsRequest,
};
use crate::config::{DflowSwapConfig, JupiterSwapConfig};
use rand::Rng;

#[derive(Clone, Debug)]
pub enum ComputeUnitPriceMode {
    Fixed(u64),
    Random { min: u64, max: u64 },
}

impl ComputeUnitPriceMode {
    pub fn sample(&self) -> u64 {
        match self {
            ComputeUnitPriceMode::Fixed(value) => *value,
            ComputeUnitPriceMode::Random { min, max } => {
                let (low, high) = if min <= max {
                    (*min, *max)
                } else {
                    (*max, *min)
                };
                if low == high {
                    low
                } else {
                    let mut rng = rand::rng();
                    rng.random_range(low..=high)
                }
            }
        }
    }
}

#[derive(Clone)]
pub enum SwapBackend {
    Jupiter {
        client: JupiterApiClient,
        defaults: JupiterSwapConfig,
    },
    Dflow {
        client: DflowApiClient,
        defaults: DflowSwapConfig,
    },
    Disabled,
}

#[derive(Clone)]
pub struct SwapInstructionFetcher {
    backend: SwapBackend,
    compute_unit_price: Option<ComputeUnitPriceMode>,
}

impl SwapInstructionFetcher {
    pub fn for_jupiter(
        client: JupiterApiClient,
        request_defaults: JupiterSwapConfig,
        compute_unit_price: Option<ComputeUnitPriceMode>,
    ) -> Self {
        Self {
            backend: SwapBackend::Jupiter {
                client,
                defaults: request_defaults,
            },
            compute_unit_price,
        }
    }

    pub fn for_dflow(
        client: DflowApiClient,
        request_defaults: DflowSwapConfig,
        compute_unit_price: Option<ComputeUnitPriceMode>,
    ) -> Self {
        Self {
            backend: SwapBackend::Dflow {
                client,
                defaults: request_defaults,
            },
            compute_unit_price,
        }
    }

    pub fn disabled() -> Self {
        Self {
            backend: SwapBackend::Disabled,
            compute_unit_price: None,
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn fetch(
        &self,
        opportunity: &SwapOpportunity,
        identity: &EngineIdentity,
    ) -> EngineResult<SwapInstructionsVariant> {
        let payload = opportunity
            .merged_quote
            .clone()
            .ok_or_else(|| EngineError::InvalidConfig("套利机会缺少报价数据".into()))?;

        match (&self.backend, payload) {
            (SwapBackend::Jupiter { client, defaults }, QuotePayloadVariant::Jupiter(inner)) => {
                let mut request = SwapInstructionsRequest::from_payload(inner, identity.pubkey);

                request.wrap_and_unwrap_sol = defaults.wrap_and_unwrap_sol;
                request.dynamic_compute_unit_limit = defaults.dynamic_compute_unit_limit;
                request.use_shared_accounts = Some(identity.use_shared_accounts());
                request.skip_user_accounts_rpc_calls = identity.skip_user_accounts_rpc_calls();
                if let Some(fee) = identity.fee_account() {
                    match solana_sdk::pubkey::Pubkey::from_str(fee) {
                        Ok(pubkey) => request.fee_account = Some(pubkey),
                        Err(err) => {
                            warn!(
                                target: "engine::swap",
                                fee_account = fee,
                                error = %err,
                                "手续费账户解析失败，忽略配置"
                            );
                        }
                    }
                }
                if let Some(strategy) = &self.compute_unit_price {
                    let price = strategy.sample();
                    if price > 0 {
                        request.compute_unit_price_micro_lamports =
                            Some(ComputeUnitPriceMicroLamports::MicroLamports(price));
                    }
                }

                client
                    .swap_instructions(&request)
                    .await
                    .map(SwapInstructionsVariant::Jupiter)
                    .map_err(EngineError::from)
            }
            (SwapBackend::Dflow { client, defaults }, QuotePayloadVariant::Dflow(inner)) => {
                let mut request =
                    DflowSwapInstructionsRequest::from_payload(inner, identity.pubkey);
                request.wrap_and_unwrap_sol = defaults.wrap_and_unwrap_sol;
                request.dynamic_compute_unit_limit = Some(defaults.dynamic_compute_unit_limit);
                if let Some(fee) = identity.fee_account() {
                    match solana_sdk::pubkey::Pubkey::from_str(fee) {
                        Ok(pubkey) => request.fee_account = Some(pubkey),
                        Err(err) => {
                            warn!(
                                target: "engine::swap",
                                fee_account = fee,
                                error = %err,
                                "手续费账户解析失败，忽略配置"
                            );
                        }
                    }
                }
                if let Some(strategy) = &self.compute_unit_price {
                    let price = strategy.sample();
                    if price > 0 {
                        request.compute_unit_price_micro_lamports =
                            Some(DflowComputeUnitPriceMicroLamports(price));
                    }
                }

                client
                    .swap_instructions(&request)
                    .await
                    .map(SwapInstructionsVariant::Dflow)
                    .map_err(EngineError::from)
            }
            (SwapBackend::Jupiter { .. }, QuotePayloadVariant::Dflow(_))
            | (SwapBackend::Dflow { .. }, QuotePayloadVariant::Jupiter(_)) => Err(
                EngineError::InvalidConfig("套利机会聚合器类型与落地器不匹配".into()),
            ),
            (SwapBackend::Disabled, _) => Err(EngineError::InvalidConfig(
                "swap backend 已禁用，无法构造指令".into(),
            )),
        }
    }

    pub fn sample_compute_unit_price(&self) -> Option<u64> {
        self.compute_unit_price
            .as_ref()
            .map(|mode| mode.sample())
            .filter(|price| *price > 0)
    }
}
