use std::collections::HashMap;
use std::ops::Deref;

use anyhow::Error;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;

use crate::api::route_plan_with_metadata::RoutePlanWithMetadata;
use crate::api::serde_helpers::field_as_string;

#[derive(Serialize, Debug, Clone, Default)]
pub struct ComputeUnitScore {
    pub max_penalty_bps: Option<f64>,
}

#[derive(Serialize, Deserialize, Default, PartialEq, Clone, Debug)]
pub enum SwapMode {
    #[default]
    ExactIn,
    ExactOut,
}

#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    /// The amount to swap, have to factor in the token decimals.
    #[serde(with = "field_as_string")]
    pub amount: u64,
    /// (ExactIn or ExactOut) Defaults to ExactIn.
    /// ExactOut is for supporting use cases where you need an exact token amount, like payments.
    /// In this case the slippage is on the input token.
    pub swap_mode: Option<SwapMode>,
    /// Allowed slippage in basis points.
    pub slippage_bps: u16,
    /// By setting this to true, our API will suggest smart slippage info that you can use.
    pub auto_slippage: Option<bool>,
    /// The max amount of slippage in basis points that you are willing to accept for auto slippage.
    pub max_auto_slippage_bps: Option<u16>,
    pub compute_auto_slippage: bool,
    /// The max amount of USD value that you are willing to accept for auto slippage.
    pub auto_slippage_collision_usd_value: Option<u32>,
    /// Quote with a greater amount to find the route to minimize slippage.
    pub minimize_slippage: Option<bool>,
    /// Platform fee in basis points.
    pub platform_fee_bps: Option<u8>,
    pub dexes: Option<Dexes>,
    pub excluded_dexes: Option<Dexes>,
    /// Quote only direct routes.
    pub only_direct_routes: Option<bool>,
    /// Quote fit into legacy transaction.
    pub as_legacy_transaction: Option<bool>,
    /// Restrict intermediate tokens to a top token set that has stable liquidity.
    pub restrict_intermediate_tokens: Option<bool>,
    /// Find a route given a maximum number of accounts involved.
    pub max_accounts: Option<usize>,
    /// Quote type to be used for routing, switches the algorithm.
    pub quote_type: Option<String>,
    /// Extra args which are quote type specific to allow controlling settings from the top level.
    pub quote_args: Option<HashMap<String, String>>,
    /// Enable only full liquid markets as intermediate tokens.
    pub prefer_liquid_dexes: Option<bool>,
    /// Use the compute unit score to pick a route.
    pub compute_unit_score: Option<ComputeUnitScore>,
    /// Routing constraints.
    pub routing_constraints: Option<String>,
    /// Token category based intermediates token.
    pub token_category_based_intermediate_tokens: Option<bool>,
    #[serde(skip)]
    pub extra_query_params: HashMap<String, String>,
}

impl QuoteRequest {
    pub fn new(input_mint: Pubkey, output_mint: Pubkey, amount: u64, slippage_bps: u16) -> Self {
        Self {
            input_mint,
            output_mint,
            amount,
            slippage_bps,
            swap_mode: None,
            compute_auto_slippage: false,
            ..Self::default()
        }
    }

    pub fn to_internal(&self) -> PreparedQuoteRequest {
        let internal = InternalQuoteRequest {
            input_mint: self.input_mint,
            output_mint: self.output_mint,
            amount: self.amount,
            swap_mode: self.swap_mode.clone(),
            slippage_bps: self.slippage_bps,
            auto_slippage: self.auto_slippage,
            max_auto_slippage_bps: self.max_auto_slippage_bps,
            compute_auto_slippage: self.compute_auto_slippage,
            auto_slippage_collision_usd_value: self.auto_slippage_collision_usd_value,
            minimize_slippage: self.minimize_slippage,
            platform_fee_bps: self.platform_fee_bps,
            dexes: self.dexes.clone(),
            excluded_dexes: self.excluded_dexes.clone(),
            only_direct_routes: self.only_direct_routes,
            as_legacy_transaction: self.as_legacy_transaction,
            restrict_intermediate_tokens: self.restrict_intermediate_tokens,
            max_accounts: self.max_accounts,
            quote_type: self.quote_type.clone(),
            prefer_liquid_dexes: self.prefer_liquid_dexes,
        };
        PreparedQuoteRequest {
            internal,
            quote_args: self.quote_args.clone(),
            extra: self.extra_query_params.clone(),
        }
    }
}

// Essentially the same as QuoteRequest, but without the extra args as we pass the extra args separately.
#[derive(Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InternalQuoteRequest {
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    /// The amount to swap, have to factor in the token decimals.
    #[serde(with = "field_as_string")]
    pub amount: u64,
    /// (ExactIn or ExactOut) Defaults to ExactIn.
    /// ExactOut is for supporting use cases where you need an exact token amount, like payments.
    /// In this case the slippage is on the input token.
    pub swap_mode: Option<SwapMode>,
    /// Allowed slippage in basis points.
    pub slippage_bps: u16,
    /// By setting this to true, our API will suggest smart slippage info that you can use.
    pub auto_slippage: Option<bool>,
    /// The max amount of slippage in basis points that you are willing to accept for auto slippage.
    pub max_auto_slippage_bps: Option<u16>,
    pub compute_auto_slippage: bool,
    /// The max amount of USD value that you are willing to accept for auto slippage.
    pub auto_slippage_collision_usd_value: Option<u32>,
    /// Quote with a greater amount to find the route to minimize slippage.
    pub minimize_slippage: Option<bool>,
    /// Platform fee in basis points.
    pub platform_fee_bps: Option<u8>,
    pub dexes: Option<Dexes>,
    pub excluded_dexes: Option<Dexes>,
    /// Quote only direct routes.
    pub only_direct_routes: Option<bool>,
    /// Quote fit into legacy transaction.
    pub as_legacy_transaction: Option<bool>,
    /// Restrict intermediate tokens to a top token set that has stable liquidity.
    pub restrict_intermediate_tokens: Option<bool>,
    /// Find a route given a maximum number of accounts involved.
    pub max_accounts: Option<usize>,
    /// Quote type to be used for routing, switches the algorithm.
    pub quote_type: Option<String>,
    /// Enable only full liquid markets as intermediate tokens.
    pub prefer_liquid_dexes: Option<bool>,
}

impl From<QuoteRequest> for InternalQuoteRequest {
    fn from(request: QuoteRequest) -> Self {
        Self {
            input_mint: request.input_mint,
            output_mint: request.output_mint,
            amount: request.amount,
            swap_mode: request.swap_mode,
            slippage_bps: request.slippage_bps,
            auto_slippage: request.auto_slippage,
            max_auto_slippage_bps: request.max_auto_slippage_bps,
            compute_auto_slippage: request.compute_auto_slippage,
            auto_slippage_collision_usd_value: request.auto_slippage_collision_usd_value,
            minimize_slippage: request.minimize_slippage,
            platform_fee_bps: request.platform_fee_bps,
            dexes: request.dexes,
            excluded_dexes: request.excluded_dexes,
            only_direct_routes: request.only_direct_routes,
            as_legacy_transaction: request.as_legacy_transaction,
            restrict_intermediate_tokens: request.restrict_intermediate_tokens,
            max_accounts: request.max_accounts,
            quote_type: request.quote_type,
            prefer_liquid_dexes: request.prefer_liquid_dexes,
        }
    }
}

/// Comma delimited list of dex labels.
pub type Dexes = String;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlatformFee {
    #[serde(with = "field_as_string")]
    pub amount: u64,
    pub fee_bps: u8,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponseData {
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub in_amount: u64,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub out_amount: u64,
    /// Not used by build transaction.
    #[serde(with = "field_as_string")]
    pub other_amount_threshold: u64,
    pub swap_mode: SwapMode,
    pub slippage_bps: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computed_auto_slippage: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses_quote_minimizing_slippage: Option<bool>,
    pub platform_fee: Option<PlatformFee>,
    pub price_impact_pct: Decimal,
    pub route_plan: RoutePlanWithMetadata,
    #[serde(default)]
    pub context_slot: u64,
    #[serde(default)]
    pub time_taken: f64,
}

#[derive(Clone, Debug)]
pub struct QuoteResponse {
    pub raw: Value,
    data: QuoteResponseData,
}

#[derive(Clone, Debug)]
pub struct PreparedQuoteRequest {
    pub internal: InternalQuoteRequest,
    pub quote_args: Option<HashMap<String, String>>,
    pub extra: HashMap<String, String>,
}

impl QuoteResponse {
    pub fn try_from_value(value: Value) -> Result<Self, Error> {
        let data: QuoteResponseData = serde_json::from_value(value.clone())?;
        Ok(Self { raw: value, data })
    }
}

impl Deref for QuoteResponse {
    type Target = QuoteResponseData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
