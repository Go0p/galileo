use std::str::FromStr;

use tracing::warn;

use super::error::{EngineError, EngineResult};
use super::identity::EngineIdentity;
use super::types::SwapOpportunity;
use crate::api::{ComputeUnitPriceMicroLamports, JupiterApiClient, SwapInstructionsRequest};
use crate::config::RequestParamsConfig;

#[derive(Clone)]
pub struct SwapInstructionFetcher {
    client: JupiterApiClient,
    request_defaults: RequestParamsConfig,
}

impl SwapInstructionFetcher {
    pub fn new(client: JupiterApiClient, request_defaults: RequestParamsConfig) -> Self {
        Self {
            client,
            request_defaults,
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn fetch(
        &self,
        opportunity: &SwapOpportunity,
        identity: &EngineIdentity,
        compute_unit_price_override: Option<u64>,
    ) -> EngineResult<crate::api::SwapInstructionsResponse> {
        let mut request =
            SwapInstructionsRequest::new(opportunity.merged_quote.clone(), identity.pubkey);

        let wrap_and_unwrap = self
            .request_defaults
            .wrap_and_unwrap_sol
            .unwrap_or_else(|| identity.wrap_and_unwrap_sol());
        request.config.wrap_and_unwrap_sol = wrap_and_unwrap;
        request.config.use_shared_accounts = Some(identity.use_shared_accounts());
        request.config.skip_user_accounts_rpc_calls = identity.skip_user_accounts_rpc_calls();
        if let Some(fee) = identity.fee_account() {
            match solana_sdk::pubkey::Pubkey::from_str(fee) {
                Ok(pubkey) => request.config.fee_account = Some(pubkey),
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
        if let Some(price) = compute_unit_price_override.or(identity.compute_unit_price_override())
        {
            request.config.compute_unit_price_micro_lamports =
                Some(ComputeUnitPriceMicroLamports::MicroLamports(price));
        }

        self.client
            .swap_instructions(&request)
            .await
            .map_err(EngineError::from)
    }
}
