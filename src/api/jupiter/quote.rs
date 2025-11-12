use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;

use crate::api::dflow::serde_helpers::decimal_from_string;
use crate::api::serde_helpers::field_as_string;

/// Jupiter 支持的 swap 模式。
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SwapMode {
    #[serde(rename = "ExactIn", alias = "exactIn")]
    ExactIn,
    #[serde(rename = "ExactOut", alias = "exactOut")]
    ExactOut,
}

impl SwapMode {
    pub fn as_str(self) -> &'static str {
        match self {
            SwapMode::ExactIn => "ExactIn",
            SwapMode::ExactOut => "ExactOut",
        }
    }
}

impl Default for SwapMode {
    fn default() -> Self {
        Self::ExactIn
    }
}

/// `/quote` 请求体，使用查询字符串传参。
#[derive(Debug, Clone)]
pub struct QuoteRequest {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount: u64,
    pub swap_mode: SwapMode,
    pub slippage_bps: Option<u16>,
    pub only_direct_routes: Option<bool>,
    pub restrict_intermediate_tokens: Option<bool>,
    pub dexes: Option<String>,
    pub exclude_dexes: Option<String>,
    pub as_legacy_transaction: Option<bool>,
    pub platform_fee_bps: Option<u16>,
    pub max_accounts: Option<u16>,
}

impl QuoteRequest {
    pub fn new(input_mint: Pubkey, output_mint: Pubkey, amount: u64) -> Self {
        Self {
            input_mint,
            output_mint,
            amount,
            swap_mode: SwapMode::ExactIn,
            slippage_bps: None,
            only_direct_routes: None,
            restrict_intermediate_tokens: None,
            dexes: None,
            exclude_dexes: None,
            as_legacy_transaction: None,
            platform_fee_bps: None,
            max_accounts: None,
        }
    }

    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::with_capacity(16);
        params.push(("inputMint".to_string(), self.input_mint.to_string()));
        params.push(("outputMint".to_string(), self.output_mint.to_string()));
        params.push(("amount".to_string(), self.amount.to_string()));
        params.push(("swapMode".to_string(), self.swap_mode.as_str().to_string()));
        if let Some(value) = self.slippage_bps {
            params.push(("slippageBps".to_string(), value.to_string()));
        }
        if let Some(value) = self.only_direct_routes {
            params.push(("onlyDirectRoutes".to_string(), value.to_string()));
        }
        if let Some(value) = self.restrict_intermediate_tokens {
            params.push(("restrictIntermediateTokens".to_string(), value.to_string()));
        }
        if let Some(value) = self.dexes.as_ref() {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                params.push(("dexes".to_string(), trimmed.to_string()));
            }
        }
        if let Some(value) = self.exclude_dexes.as_ref() {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                params.push(("excludeDexes".to_string(), trimmed.to_string()));
            }
        }
        if let Some(value) = self.as_legacy_transaction {
            params.push(("asLegacyTransaction".to_string(), value.to_string()));
        }
        if let Some(value) = self.platform_fee_bps {
            params.push(("platformFeeBps".to_string(), value.to_string()));
        }
        if let Some(value) = self.max_accounts {
            params.push(("maxAccounts".to_string(), value.to_string()));
        }
        params
    }
}

/// `/quote` 响应体。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponsePayload {
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub in_amount: u64,
    #[serde(with = "field_as_string")]
    pub out_amount: u64,
    #[serde(with = "field_as_string")]
    pub other_amount_threshold: u64,
    pub swap_mode: SwapMode,
    pub slippage_bps: u16,
    #[serde(with = "decimal_from_string")]
    pub price_impact_pct: Decimal,
    #[serde(default)]
    pub context_slot: Option<u64>,
    #[serde(default)]
    pub time_taken: Option<f64>,
    #[serde(default)]
    pub route_plan: Vec<Value>,
    #[serde(default)]
    pub market_infos: Vec<Value>,
    #[serde(default)]
    pub platform_fee: Option<Value>,
}

#[derive(Clone, Debug)]
pub struct QuoteResponse {
    data: QuoteResponsePayload,
}

impl QuoteResponse {
    pub fn try_from_value(value: Value) -> Result<Self, serde_json::Error> {
        let data: QuoteResponsePayload = serde_json::from_value(value)?;
        Ok(Self { data })
    }

    pub fn payload(&self) -> &QuoteResponsePayload {
        &self.data
    }

    pub fn into_payload(self) -> QuoteResponsePayload {
        self.data
    }
}
