use async_trait::async_trait;
use tracing::debug;

use crate::api::jupiter::{
    JupiterApiClient, JupiterError, QuoteRequest as JupiterQuoteRequest,
    QuoteResponse as JupiterQuoteResponse, SwapInstructionsRequest as JupiterSwapRequest,
    SwapInstructionsResponse as JupiterSwapResponse,
};
use crate::config::{JupiterQuoteConfig, JupiterSwapConfig};
use crate::engine::COMPUTE_BUDGET_PROGRAM_ID;
use crate::engine::multi_leg::leg::LegProvider;
use crate::engine::multi_leg::types::{
    AggregatorKind, LegBuildContext, LegDescriptor, LegPlan, LegQuote, LegSide, QuoteIntent,
};
use crate::network::{IpLeaseHandle, IpLeaseOutcome};

/// Jupiter 聚合器的多腿提供方实现。
#[derive(Clone, Debug)]
pub struct JupiterLegProvider {
    client: JupiterApiClient,
    descriptor: LegDescriptor,
    quote_config: JupiterQuoteConfig,
    swap_config: JupiterSwapConfig,
}

impl JupiterLegProvider {
    pub fn new(
        client: JupiterApiClient,
        side: LegSide,
        quote_config: JupiterQuoteConfig,
        swap_config: JupiterSwapConfig,
    ) -> Self {
        Self {
            descriptor: LegDescriptor::new(AggregatorKind::Jupiter, side),
            client,
            quote_config,
            swap_config,
        }
    }

    fn build_quote_request(&self, intent: &QuoteIntent) -> JupiterQuoteRequest {
        let mut request =
            JupiterQuoteRequest::new(intent.input_mint, intent.output_mint, intent.amount);
        request.slippage_bps = Some(intent.slippage_bps);
        if self.quote_config.only_direct_routes {
            request.only_direct_routes = Some(true);
        }
        if self.quote_config.restrict_intermediate_tokens {
            request.restrict_intermediate_tokens = Some(true);
        }
        request
    }

    fn build_swap_request(
        &self,
        quote: &JupiterQuoteResponse,
        context: &LegBuildContext,
    ) -> JupiterSwapRequest {
        let mut request = JupiterSwapRequest::from_quote(quote.payload().clone(), context.payer);
        request.wrap_and_unwrap_sol = context
            .wrap_and_unwrap_sol
            .unwrap_or(self.swap_config.wrap_and_unwrap_sol);
        request.skip_user_accounts_rpc_calls = Some(self.swap_config.skip_user_accounts_rpc_calls);
        request.dynamic_compute_unit_limit = Some(
            context
                .dynamic_compute_unit_limit
                .unwrap_or(self.swap_config.dynamic_compute_unit_limit),
        );
        if let Some(fee_account) = context.fee_account {
            request.fee_account = Some(fee_account);
        }
        if let Some(cu_price) = context.compute_unit_price_micro_lamports {
            if cu_price > 0 {
                request.compute_unit_price_micro_lamports = Some(cu_price);
            }
        }
        if let Some(flag) = self.swap_config.use_shared_accounts {
            request.use_shared_accounts = Some(flag);
        }
        request
    }

    fn into_plan(
        &self,
        quote: &JupiterQuoteResponse,
        mut swap: JupiterSwapResponse,
        multiplier: Option<f64>,
        context: &LegBuildContext,
    ) -> LegPlan {
        let adjusted_limit = apply_cu_limit_multiplier(swap.compute_unit_limit, multiplier);
        if adjusted_limit != swap.compute_unit_limit {
            rewrite_limit_instructions(&mut swap.compute_budget_instructions, adjusted_limit);
            swap.compute_unit_limit = adjusted_limit;
        }

        let quote_meta = self.summarize_quote(quote);

        let mut instructions = Vec::new();
        instructions.extend(swap.setup_instructions.clone());
        instructions.push(swap.swap_instruction.clone());
        instructions.extend(swap.cleanup_instructions.clone());
        instructions.extend(swap.other_instructions.clone());

        LegPlan {
            descriptor: self.descriptor.clone(),
            quote: quote_meta,
            instructions,
            compute_budget_instructions: swap.compute_budget_instructions.clone(),
            address_lookup_table_addresses: swap.address_lookup_table_addresses.clone(),
            resolved_lookup_tables: Vec::new(),
            prioritization_fee_lamports: swap.prioritization_fee_lamports,
            blockhash: swap.blockhash.as_ref().map(|meta| meta.blockhash),
            raw_transaction: None,
            signer_rewrite: None,
            account_rewrites: Vec::new(),
            requested_compute_unit_limit: Some(swap.compute_unit_limit),
            requested_compute_unit_price_micro_lamports: context.compute_unit_price_micro_lamports,
            requested_tip_lamports: swap.prioritization_fee_lamports,
        }
    }
}

#[async_trait]
impl LegProvider for JupiterLegProvider {
    type QuoteResponse = JupiterQuoteResponse;
    type BuildError = JupiterError;
    type Plan = LegPlan;

    fn descriptor(&self) -> LegDescriptor {
        self.descriptor.clone()
    }

    fn summarize_quote(&self, quote: &Self::QuoteResponse) -> LegQuote {
        let payload = quote.payload();
        let mut quote_meta =
            LegQuote::new(payload.in_amount, payload.out_amount, payload.slippage_bps);
        quote_meta.min_out_amount = Some(payload.other_amount_threshold);
        quote_meta.context_slot = payload.context_slot;
        quote_meta.provider = Some("jupiter".to_string());
        quote_meta
    }

    async fn quote(
        &self,
        intent: &QuoteIntent,
        lease: Option<&IpLeaseHandle>,
    ) -> Result<Self::QuoteResponse, Self::BuildError> {
        let request = self.build_quote_request(intent);
        debug!(
            target: "multi_leg::jupiter",
            input = %intent.input_mint,
            output = %intent.output_mint,
            amount = intent.amount,
            "开始请求 Jupiter 报价"
        );
        let result = self
            .client
            .quote_with_ip(&request, lease.map(|handle| handle.ip()))
            .await;
        if let Err(err) = &result {
            if let Some(handle) = lease {
                if let Some(outcome) = classify_jupiter_error(err) {
                    handle.mark_outcome(outcome);
                }
            }
        }
        result
    }

    async fn build_plan(
        &self,
        quote: &Self::QuoteResponse,
        context: &LegBuildContext,
        lease: Option<&IpLeaseHandle>,
    ) -> Result<Self::Plan, Self::BuildError> {
        let request = self.build_swap_request(quote, context);
        let response = match self
            .client
            .swap_instructions_with_ip(&request, lease.map(|handle| handle.ip()))
            .await
        {
            Ok(resp) => resp,
            Err(err) => {
                if let Some(handle) = lease {
                    if let Some(outcome) = classify_jupiter_error(&err) {
                        handle.mark_outcome(outcome);
                    }
                }
                return Err(err);
            }
        };
        let plan = self.into_plan(
            quote,
            response,
            context.compute_unit_limit_multiplier,
            context,
        );
        Ok(plan)
    }
}

fn classify_jupiter_error(err: &JupiterError) -> Option<IpLeaseOutcome> {
    match err {
        JupiterError::RateLimited { .. } => Some(IpLeaseOutcome::RateLimited),
        JupiterError::ApiStatus { status, .. } => map_status(status),
        JupiterError::Timeout { .. } => Some(IpLeaseOutcome::Timeout),
        JupiterError::Http(inner) => classify_reqwest(inner),
        _ => None,
    }
}

fn map_status(status: &reqwest::StatusCode) -> Option<IpLeaseOutcome> {
    use reqwest::StatusCode;
    if *status == StatusCode::TOO_MANY_REQUESTS {
        return Some(IpLeaseOutcome::RateLimited);
    }
    if *status == StatusCode::REQUEST_TIMEOUT || *status == StatusCode::GATEWAY_TIMEOUT {
        return Some(IpLeaseOutcome::Timeout);
    }
    if status.is_server_error() {
        return Some(IpLeaseOutcome::NetworkError);
    }
    None
}

fn classify_reqwest(err: &reqwest::Error) -> Option<IpLeaseOutcome> {
    if err.is_timeout() {
        return Some(IpLeaseOutcome::Timeout);
    }
    if let Some(status) = err.status() {
        if let Some(mapped) = map_status(&status) {
            return Some(mapped);
        }
    }
    if err.is_connect() || err.is_request() {
        return Some(IpLeaseOutcome::NetworkError);
    }
    None
}

fn apply_cu_limit_multiplier(base: u32, multiplier: Option<f64>) -> u32 {
    let base_limit = base.max(1);
    let sanitized = multiplier.and_then(sanitize_multiplier).unwrap_or(1.0);
    let mut scaled = (base_limit as f64) * sanitized;
    if !scaled.is_finite() {
        return base_limit;
    }
    if scaled < 1.0 {
        scaled = 1.0;
    }
    if scaled > u32::MAX as f64 {
        scaled = u32::MAX as f64;
    }
    scaled.round() as u32
}

fn sanitize_multiplier(value: f64) -> Option<f64> {
    if value.is_finite() && value > 0.0 {
        Some(value)
    } else {
        None
    }
}

fn rewrite_limit_instructions(
    instructions: &mut [solana_sdk::instruction::Instruction],
    limit: u32,
) {
    for ix in instructions {
        if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID && ix.data.first() == Some(&2) {
            if ix.data.len() >= 5 {
                ix.data[1..5].copy_from_slice(&limit.to_le_bytes());
            }
        }
    }
}
