use std::collections::HashMap;
use std::ops::Deref;

use anyhow::Error;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;

use crate::api::serde_helpers::field_as_string;

/// Jupiter 支持的报价模式。
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapMode {
    #[serde(rename = "ExactIn", alias = "exactIn")]
    ExactIn,
    #[serde(rename = "ExactOut", alias = "exactOut")]
    ExactOut,
}

impl Default for SwapMode {
    fn default() -> Self {
        Self::ExactIn
    }
}

/// `/quote` 请求参数，保持与文档一致，仅保留目前用得上的字段。
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub amount: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub swap_mode: Option<SwapMode>,
    pub slippage_bps: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dexes: Option<Dexes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_dexes: Option<Dexes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_direct_routes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restrict_intermediate_tokens: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub as_legacy_transaction: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_accounts: Option<usize>,
    #[serde(skip)]
    pub extra_query_params: HashMap<String, String>,
}

impl QuoteRequest {
    pub fn new(input_mint: Pubkey, output_mint: Pubkey, amount: u64, slippage_bps: u16) -> Self {
        Self {
            input_mint,
            output_mint,
            amount,
            swap_mode: None,
            slippage_bps,
            dexes: None,
            excluded_dexes: None,
            only_direct_routes: None,
            restrict_intermediate_tokens: None,
            as_legacy_transaction: None,
            max_accounts: None,
            extra_query_params: HashMap::new(),
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

/// 单条路由步骤。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlanStep {
    pub swap_info: SwapInfo,
    pub percent: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bps: Option<u16>,
}

/// 单条 AMM 交换信息。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SwapInfo {
    #[serde(with = "field_as_string")]
    pub amm_key: Pubkey,
    pub label: String,
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub in_amount: u64,
    #[serde(with = "field_as_string")]
    pub out_amount: u64,
    #[serde(with = "field_as_string")]
    pub fee_amount: u64,
    #[serde(with = "field_as_string")]
    pub fee_mint: Pubkey,
}

/// `/quote` 响应体，仅覆盖我们实际需要的字段。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponsePayload {
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub in_amount: u64,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub out_amount: u64,
    #[serde(with = "field_as_string")]
    pub other_amount_threshold: u64,
    pub swap_mode: SwapMode,
    pub slippage_bps: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_fee: Option<PlatformFee>,
    pub price_impact_pct: Decimal,
    pub route_plan: Vec<RoutePlanStep>,
    #[serde(default)]
    pub context_slot: u64,
    #[serde(default)]
    pub time_taken: f64,
}

impl QuoteResponsePayload {
    pub fn try_from_value(value: Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}

#[derive(Clone, Debug)]
pub struct QuoteResponse {
    pub raw: Value,
    data: QuoteResponsePayload,
}

impl QuoteResponse {
    pub fn try_from_value(value: Value) -> Result<Self, Error> {
        let data: QuoteResponsePayload = serde_json::from_value(value.clone())?;
        Ok(Self { raw: value, data })
    }
}

impl Deref for QuoteResponse {
    type Target = QuoteResponsePayload;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
