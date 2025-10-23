use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;

use super::serde_helpers::{decimal_from_string, field_as_string};

/// DFlow 支持的 slippage 参数。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SlippagePreset {
    Auto,
}

/// slippageBps 字段允许填写整数或 "auto"。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum SlippageBps {
    Fixed(u16),
    Preset(SlippagePreset),
}

/// 平台收费模式。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PlatformFeeMode {
    OutputMint,
    InputMint,
}

/// `/quote` 请求参数。
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
    pub slippage_bps: Option<SlippageBps>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dexes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_dexes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_fee_bps: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform_fee_mode: Option<PlatformFeeMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsored_swap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_swap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_direct_routes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_route_length: Option<u8>,
}

impl QuoteRequest {
    pub fn new(input_mint: Pubkey, output_mint: Pubkey, amount: u64) -> Self {
        Self {
            input_mint,
            output_mint,
            amount,
            slippage_bps: None,
            dexes: None,
            exclude_dexes: None,
            platform_fee_bps: None,
            platform_fee_mode: None,
            sponsored_swap: None,
            destination_swap: None,
            only_direct_routes: None,
            max_route_length: None,
        }
    }
}

/// routePlan 的通用字段。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlanLeg {
    #[serde(with = "field_as_string")]
    pub in_amount: u64,
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    pub input_mint_decimals: u8,
    pub market_key: String,
    #[serde(with = "field_as_string")]
    pub out_amount: u64,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    pub output_mint_decimals: u8,
    pub venue: String,
}

/// routePlan 中包含 data 字段的变体。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlanLegWithData {
    pub data: String,
    #[serde(flatten)]
    pub leg: RoutePlanLeg,
}

/// routePlan 在没有 data 字段时的形式。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlanLegWithoutData {
    #[serde(flatten)]
    pub leg: RoutePlanLeg,
}

/// routePlan 支持两种结构，采用 untagged 枚举进行兼容。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum RoutePlanStep {
    WithData(RoutePlanLegWithData),
    WithoutData(RoutePlanLegWithoutData),
}

impl RoutePlanStep {
    pub fn leg(&self) -> &RoutePlanLeg {
        match self {
            RoutePlanStep::WithData(step) => &step.leg,
            RoutePlanStep::WithoutData(step) => &step.leg,
        }
    }
}

/// 平台费用信息。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PlatformFee {
    #[serde(with = "field_as_string")]
    pub amount: u64,
    #[serde(with = "field_as_string")]
    pub fee_account: Pubkey,
    pub fee_bps: u16,
    #[serde(with = "field_as_string")]
    pub segmenter_fee_amount: u64,
    pub segmenter_fee_pct: u32,
}

/// `/quote` 响应体。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponsePayload {
    pub context_slot: u64,
    #[serde(with = "field_as_string")]
    pub in_amount: u64,
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub min_out_amount: u64,
    #[serde(with = "field_as_string")]
    pub other_amount_threshold: u64,
    #[serde(with = "field_as_string")]
    pub out_amount: u64,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    #[serde(with = "decimal_from_string")]
    pub price_impact_pct: Decimal,
    pub route_plan: Vec<RoutePlanStep>,
    pub slippage_bps: u16,
    #[serde(default)]
    pub out_transfer_fee: Option<String>,
    #[serde(default)]
    pub platform_fee: Option<PlatformFee>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub simulated_compute_units: Option<u64>,
}

impl QuoteResponsePayload {
    pub fn try_from_value(value: Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}

/// 保留原始 JSON 的响应封装。
#[derive(Clone, Debug)]
pub struct QuoteResponse {
    pub raw: Value,
    data: QuoteResponsePayload,
}

impl QuoteResponse {
    pub fn try_from_value(value: Value) -> Result<Self, serde_json::Error> {
        let data: QuoteResponsePayload = serde_json::from_value(value.clone())?;
        Ok(Self { raw: value, data })
    }

    pub fn payload(&self) -> &QuoteResponsePayload {
        &self.data
    }
}
