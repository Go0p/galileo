use async_trait::async_trait;
use tracing::debug;

use crate::api::dflow::{
    ComputeUnitPriceMicroLamports as DflowComputeUnitPriceMicroLamports, DflowApiClient,
    DflowError, QuoteRequest as DflowQuoteRequest, QuoteResponse as DflowQuoteResponse,
    SlippageBps, SlippagePreset, SwapInstructionsRequest as DflowSwapInstructionsRequest,
    SwapInstructionsResponse as DflowSwapInstructionsResponse,
};
use crate::config::{DflowQuoteConfig, DflowSwapConfig};
use crate::multi_leg::leg::LegProvider;
use crate::multi_leg::types::{
    AggregatorKind, LegBuildContext, LegDescriptor, LegPlan, LegQuote, LegSide, QuoteIntent,
};

/// DFlow 腿提供方，实现报价与指令获取的封装。
#[derive(Clone, Debug)]
pub struct DflowLegProvider {
    client: DflowApiClient,
    descriptor: LegDescriptor,
    quote_defaults: DflowQuoteConfig,
    swap_defaults: DflowSwapConfig,
    dex_whitelist: Vec<String>,
    dex_blacklist: Vec<String>,
}

impl DflowLegProvider {
    pub fn new(
        client: DflowApiClient,
        quote_defaults: DflowQuoteConfig,
        swap_defaults: DflowSwapConfig,
        side: LegSide,
        dex_whitelist: Vec<String>,
        dex_blacklist: Vec<String>,
    ) -> Self {
        Self {
            client,
            descriptor: LegDescriptor::new(AggregatorKind::Dflow, side),
            quote_defaults,
            swap_defaults,
            dex_whitelist,
            dex_blacklist,
        }
    }

    fn build_quote_request(&self, intent: &QuoteIntent) -> DflowQuoteRequest {
        let mut request =
            DflowQuoteRequest::new(intent.input_mint, intent.output_mint, intent.amount);
        if self.quote_defaults.use_auto_slippage {
            request.slippage_bps = Some(SlippageBps::Preset(SlippagePreset::Auto));
        } else {
            request.slippage_bps = Some(SlippageBps::Fixed(intent.slippage_bps));
        }
        if self.quote_defaults.only_direct_routes {
            request.only_direct_routes = Some(true);
        }
        if let Some(max_route_length) = self.quote_defaults.max_route_length {
            request.max_route_length = Some(max_route_length);
        }
        if !self.dex_whitelist.is_empty() {
            request.dexes = Some(self.dex_whitelist.join(","));
        }
        if !self.dex_blacklist.is_empty() {
            request.exclude_dexes = Some(self.dex_blacklist.join(","));
        }
        request
    }

    fn build_swap_request(
        &self,
        quote: &DflowQuoteResponse,
        context: &LegBuildContext,
    ) -> DflowSwapInstructionsRequest {
        let mut request =
            DflowSwapInstructionsRequest::from_payload(quote.payload().clone(), context.payer);

        // wrap_and_unwrap_sol 默认为配置，允许上下文覆盖。
        request.wrap_and_unwrap_sol = context
            .wrap_and_unwrap_sol
            .unwrap_or(self.swap_defaults.wrap_and_unwrap_sol);
        request.dynamic_compute_unit_limit = Some(
            context
                .dynamic_compute_unit_limit
                .unwrap_or(self.swap_defaults.dynamic_compute_unit_limit),
        );

        if let Some(fee_account) = context.fee_account {
            request.fee_account = Some(fee_account);
        }
        if let Some(sponsor) = context.sponsor {
            request.sponsor = Some(sponsor);
        }
        if let Some(price) = context.compute_unit_price_micro_lamports {
            request.compute_unit_price_micro_lamports =
                Some(DflowComputeUnitPriceMicroLamports(price));
        }

        request
    }

    fn into_plan(
        &self,
        quote: &DflowQuoteResponse,
        swap: DflowSwapInstructionsResponse,
    ) -> LegPlan {
        let payload = quote.payload();
        let mut quote_meta =
            LegQuote::new(payload.in_amount, payload.out_amount, payload.slippage_bps);
        quote_meta.min_out_amount = Some(payload.other_amount_threshold);
        quote_meta.request_id = payload.request_id.clone();
        quote_meta.context_slot = Some(payload.context_slot);
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
            blockhash: Some(swap.blockhash_with_metadata.blockhash),
            raw_transaction: None,
            signer_rewrite: None,
            account_rewrites: Vec::new(),
            requested_compute_unit_limit: None,
            requested_compute_unit_price_micro_lamports: None,
            requested_tip_lamports: None,
        }
    }
}

#[async_trait]
impl LegProvider for DflowLegProvider {
    type QuoteResponse = DflowQuoteResponse;
    type BuildError = DflowError;
    type Plan = LegPlan;

    fn descriptor(&self) -> LegDescriptor {
        self.descriptor.clone()
    }

    async fn quote(&self, intent: &QuoteIntent) -> Result<Self::QuoteResponse, Self::BuildError> {
        let request = self.build_quote_request(intent);
        debug!(
            target: "multi_leg::dflow",
            input = %intent.input_mint,
            output = %intent.output_mint,
            amount = intent.amount,
            "开始请求 DFlow 报价"
        );
        self.client.quote(&request).await
    }

    async fn build_plan(
        &self,
        quote: &Self::QuoteResponse,
        context: &LegBuildContext,
    ) -> Result<Self::Plan, Self::BuildError> {
        let request = self.build_swap_request(quote, context);
        let response = self.client.swap_instructions(&request).await?;
        let mut plan = self.into_plan(quote, response);
        if plan.requested_compute_unit_price_micro_lamports.is_none() {
            plan.requested_compute_unit_price_micro_lamports =
                context.compute_unit_price_micro_lamports;
        }
        Ok(plan)
    }
}
