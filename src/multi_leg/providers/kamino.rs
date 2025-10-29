use std::str::FromStr;

use async_trait::async_trait;
use reqwest::{Error as ReqwestError, StatusCode};
use solana_compute_budget_interface as compute_budget;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use thiserror::Error;
use tracing::debug;

use crate::api::kamino::{KaminoApiClient, KaminoError, QuoteRequest, QuoteResponse, Route};
use crate::config::KaminoQuoteConfig;
use crate::engine::FALLBACK_CU_LIMIT;
use crate::multi_leg::leg::LegProvider;
use crate::multi_leg::types::{
    AggregatorKind, LegBuildContext, LegDescriptor, LegPlan, LegQuote, LegSide, QuoteIntent,
};
use crate::network::{IpLeaseHandle, IpLeaseOutcome};

/// Kamino 聚合器的多腿适配器。
#[derive(Clone, Debug)]
pub struct KaminoLegProvider {
    client: KaminoApiClient,
    descriptor: LegDescriptor,
    quote_config: KaminoQuoteConfig,
}

impl KaminoLegProvider {
    pub fn new(client: KaminoApiClient, side: LegSide, quote_config: KaminoQuoteConfig) -> Self {
        Self {
            client,
            descriptor: LegDescriptor::new(AggregatorKind::Kamino, side),
            quote_config,
        }
    }

    fn build_quote_request(&self, intent: &QuoteIntent) -> (QuoteRequest, u16) {
        let slippage_bps = if self.quote_config.max_slippage_bps > 0 {
            self.quote_config.max_slippage_bps
        } else {
            intent.slippage_bps
        };
        let mut request = QuoteRequest::new(
            intent.input_mint,
            intent.output_mint,
            intent.amount,
            slippage_bps,
        );
        request.include_setup_ixs = self.quote_config.include_setup_ixs;
        request.wrap_and_unwrap_sol = self.quote_config.wrap_and_unwrap_sol;

        if let Some(executor) = trim_non_empty(&self.quote_config.executor) {
            request.executor = Some(executor.to_string());
        }
        if let Some(referrer) = trim_non_empty(&self.quote_config.referrer_pda) {
            request.referrer_pda = Some(referrer.to_string());
        }
        if !self.quote_config.routes.is_empty() {
            request.routes = self.quote_config.routes.clone();
        }

        (request, slippage_bps)
    }

    fn into_plan(&self, route: &Route, slippage_bps: u16, context: &LegBuildContext) -> LegPlan {
        let mut compute_budget_instructions = Vec::new();
        let mut main_instructions = Vec::new();
        let mut compute_unit_limit: Option<u32> = None;
        let mut compute_unit_price: Option<u64> = None;

        for instruction in route.instructions.flatten() {
            if instruction.program_id == compute_budget::id() {
                if let Some(parsed) = parse_compute_budget(&instruction) {
                    match parsed {
                        ParsedComputeBudget::Limit(limit) => compute_unit_limit = Some(limit),
                        ParsedComputeBudget::Price(price) => compute_unit_price = Some(price),
                        ParsedComputeBudget::Other => {}
                    }
                }
                compute_budget_instructions.push(instruction);
            } else {
                main_instructions.push(instruction);
            }
        }

        let mut lookup_table_addresses = Vec::new();
        for entry in &route.lookup_table_accounts_bs58 {
            let trimmed = entry.trim();
            if trimmed.is_empty() {
                continue;
            }
            match Pubkey::from_str(trimmed) {
                Ok(pubkey) => lookup_table_addresses.push(pubkey),
                Err(err) => {
                    debug!(
                        target: "multi_leg::kamino",
                        address = trimmed,
                        error = %err,
                        "解析 Kamino lookup table 地址失败，忽略"
                    );
                }
            }
        }

        let limit = compute_unit_limit.unwrap_or(FALLBACK_CU_LIMIT);
        if compute_unit_limit.is_none() {
            compute_budget_instructions
                .insert(0, ComputeBudgetInstruction::set_compute_unit_limit(limit));
        }

        let mut compute_price = compute_unit_price;

        if let Some(override_price) = context.compute_unit_price_micro_lamports {
            compute_budget_instructions.retain(|ix| {
                !matches!(
                    parse_compute_budget(ix),
                    Some(ParsedComputeBudget::Price(_))
                )
            });
            if override_price > 0 {
                compute_budget_instructions.push(ComputeBudgetInstruction::set_compute_unit_price(
                    override_price,
                ));
                compute_price = Some(override_price);
            } else {
                compute_price = None;
            }
        }

        let prioritization_fee_lamports = compute_price.map(|price| {
            let fee = (price as u128)
                .saturating_mul(limit as u128)
                .checked_div(1_000_000u128)
                .unwrap_or(0);
            fee.min(u64::MAX as u128) as u64
        });

        let mut quote = LegQuote::new(route.amount_in(), route.amount_out(), slippage_bps);
        quote.min_out_amount = Some(route.amounts_exact_in.amount_out_guaranteed);
        quote.provider = Some(route.router_type.clone());

        LegPlan {
            descriptor: self.descriptor.clone(),
            quote,
            instructions: main_instructions,
            compute_budget_instructions,
            address_lookup_table_addresses: lookup_table_addresses,
            resolved_lookup_tables: Vec::new(),
            prioritization_fee_lamports,
            blockhash: None,
            raw_transaction: None,
            signer_rewrite: None,
            account_rewrites: Vec::new(),
            requested_compute_unit_limit: Some(limit),
            requested_compute_unit_price_micro_lamports: compute_price,
            requested_tip_lamports: prioritization_fee_lamports,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KaminoLegQuote {
    response: QuoteResponse,
    slippage_bps: u16,
}

impl KaminoLegQuote {
    fn new(response: QuoteResponse, slippage_bps: u16) -> Self {
        Self {
            response,
            slippage_bps,
        }
    }

    fn best_route(&self) -> Option<&Route> {
        self.response.best_route()
    }
}

#[derive(Debug, Error)]
pub enum KaminoLegError {
    #[error("Kamino API 请求失败: {0}")]
    Api(#[from] KaminoError),
    #[error("Kamino 返回空路线")]
    EmptyRoute,
}

#[async_trait]
impl LegProvider for KaminoLegProvider {
    type QuoteResponse = KaminoLegQuote;
    type BuildError = KaminoLegError;
    type Plan = LegPlan;

    fn descriptor(&self) -> LegDescriptor {
        self.descriptor.clone()
    }

    async fn quote(
        &self,
        intent: &QuoteIntent,
        lease: Option<&IpLeaseHandle>,
    ) -> Result<Self::QuoteResponse, Self::BuildError> {
        let (request, slippage_bps) = self.build_quote_request(intent);
        let result = self
            .client
            .quote_with_ip(&request, lease.map(|handle| handle.ip()))
            .await;

        match result {
            Ok(response) => {
                if response.best_route().is_none() {
                    Err(KaminoLegError::EmptyRoute)
                } else {
                    Ok(KaminoLegQuote::new(response, slippage_bps))
                }
            }
            Err(err) => {
                if let Some(handle) = lease {
                    if let Some(outcome) = classify_error(&err) {
                        handle.mark_outcome(outcome);
                    }
                }
                Err(KaminoLegError::Api(err))
            }
        }
    }

    async fn build_plan(
        &self,
        quote: &Self::QuoteResponse,
        context: &LegBuildContext,
        _lease: Option<&IpLeaseHandle>,
    ) -> Result<Self::Plan, Self::BuildError> {
        let route = quote.best_route().ok_or(KaminoLegError::EmptyRoute)?;
        Ok(self.into_plan(route, quote.slippage_bps, context))
    }
}

fn classify_error(err: &KaminoError) -> Option<IpLeaseOutcome> {
    match err {
        KaminoError::RateLimited { .. } => Some(IpLeaseOutcome::RateLimited),
        KaminoError::ApiStatus { status, .. } => map_status(status),
        KaminoError::Http(inner) => classify_reqwest(inner),
        KaminoError::ClientPool(_) => Some(IpLeaseOutcome::NetworkError),
        KaminoError::Json(_) | KaminoError::Schema(_) => Some(IpLeaseOutcome::NetworkError),
    }
}

fn classify_reqwest(err: &ReqwestError) -> Option<IpLeaseOutcome> {
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

fn map_status(status: &StatusCode) -> Option<IpLeaseOutcome> {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParsedComputeBudget {
    Limit(u32),
    Price(u64),
    Other,
}

fn parse_compute_budget(ix: &Instruction) -> Option<ParsedComputeBudget> {
    if ix.program_id != compute_budget::id() {
        return None;
    }
    let data = ix.data.as_slice();
    if data.is_empty() {
        return Some(ParsedComputeBudget::Other);
    }
    match data[0] {
        2 if data.len() >= 5 => {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[1..5]);
            Some(ParsedComputeBudget::Limit(u32::from_le_bytes(bytes)))
        }
        3 if data.len() >= 9 => {
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data[1..9]);
            Some(ParsedComputeBudget::Price(u64::from_le_bytes(bytes)))
        }
        _ => Some(ParsedComputeBudget::Other),
    }
}

fn trim_non_empty(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}
