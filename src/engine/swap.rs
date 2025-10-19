use std::str::FromStr;

use tracing::warn;

use super::error::{EngineError, EngineResult};
use super::identity::EngineIdentity;
use super::types::SwapOpportunity;
use crate::api::{ComputeUnitPriceMicroLamports, JupiterApiClient, SwapInstructionsRequest};
use crate::config::JupiterSwapConfig;
use rand::Rng;

#[derive(Clone, Debug)]
pub enum ComputeUnitPriceMode {
    Fixed(u64),
    Random { min: u64, max: u64 },
}

impl ComputeUnitPriceMode {
    fn sample(&self) -> u64 {
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
pub struct SwapInstructionFetcher {
    client: JupiterApiClient,
    request_defaults: JupiterSwapConfig,
    compute_unit_price: Option<ComputeUnitPriceMode>,
}

impl SwapInstructionFetcher {
    pub fn new(
        client: JupiterApiClient,
        request_defaults: JupiterSwapConfig,
        compute_unit_price: Option<ComputeUnitPriceMode>,
    ) -> Self {
        Self {
            client,
            request_defaults,
            compute_unit_price,
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn fetch(
        &self,
        opportunity: &SwapOpportunity,
        identity: &EngineIdentity,
    ) -> EngineResult<crate::api::SwapInstructionsResponse> {
        let mut request =
            SwapInstructionsRequest::new(opportunity.merged_quote.clone(), identity.pubkey);

        request.wrap_and_unwrap_sol = self.request_defaults.wrap_and_unwrap_sol;
        request.dynamic_compute_unit_limit = self.request_defaults.dynamic_compute_unit_limit;
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
            request.compute_unit_price_micro_lamports =
                Some(ComputeUnitPriceMicroLamports::MicroLamports(price));
        }

        self.client
            .swap_instructions(&request)
            .await
            .map_err(EngineError::from)
    }
}
