use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct QuoteRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub slippage_bps: u16,
    pub only_direct_routes: bool,
    pub restrict_intermediate_tokens: bool,
    pub extra: BTreeMap<String, String>,
}

impl QuoteRequest {
    pub fn new(
        input_mint: impl Into<String>,
        output_mint: impl Into<String>,
        amount: u64,
        slippage_bps: u16,
    ) -> Self {
        Self {
            input_mint: input_mint.into(),
            output_mint: output_mint.into(),
            amount,
            slippage_bps,
            only_direct_routes: false,
            restrict_intermediate_tokens: true,
            extra: BTreeMap::new(),
        }
    }

    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("inputMint".to_string(), self.input_mint.clone()),
            ("outputMint".to_string(), self.output_mint.clone()),
            ("amount".to_string(), self.amount.to_string()),
            ("slippageBps".to_string(), self.slippage_bps.to_string()),
            (
                "onlyDirectRoutes".to_string(),
                self.only_direct_routes.to_string(),
            ),
            (
                "restrictIntermediateTokens".to_string(),
                self.restrict_intermediate_tokens.to_string(),
            ),
        ];
        for (key, value) in &self.extra {
            params.push((key.clone(), value.clone()));
        }
        params
    }
}

#[derive(Debug, Clone)]
pub struct QuoteResponse {
    pub raw: Value,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub other_amount_threshold: Option<String>,
    pub time_taken: Option<f64>,
}

impl QuoteResponse {
    pub fn try_from_value(value: Value) -> Result<Self, String> {
        let raw = value;
        let input_mint = raw
            .get("inputMint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "inputMint missing".to_string())?
            .to_string();
        let output_mint = raw
            .get("outputMint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "outputMint missing".to_string())?
            .to_string();
        let in_amount = raw
            .get("inAmount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "inAmount missing".to_string())?
            .to_string();
        let out_amount = raw
            .get("outAmount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "outAmount missing".to_string())?
            .to_string();

        let other_amount_threshold = raw
            .get("otherAmountThreshold")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let time_taken = raw.get("timeTaken").and_then(|v| v.as_f64());

        Ok(Self {
            raw,
            input_mint,
            output_mint,
            in_amount,
            out_amount,
            other_amount_threshold,
            time_taken,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SwapRequest {
    #[serde(rename = "quoteResponse")]
    pub quote_response: Value,
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
    #[serde(rename = "wrapAndUnwrapSol", skip_serializing_if = "Option::is_none")]
    pub wrap_and_unwrap_sol: Option<bool>,
    #[serde(rename = "useSharedAccounts", skip_serializing_if = "Option::is_none")]
    pub use_shared_accounts: Option<bool>,
    #[serde(rename = "feeAccount", skip_serializing_if = "Option::is_none")]
    pub fee_account: Option<String>,
    #[serde(
        rename = "computeUnitPriceMicroLamports",
        skip_serializing_if = "Option::is_none"
    )]
    pub compute_unit_price_micro_lamports: Option<u64>,
}

impl SwapRequest {
    pub fn new(quote_response: Value, user_public_key: impl Into<String>) -> Self {
        Self {
            quote_response,
            user_public_key: user_public_key.into(),
            wrap_and_unwrap_sol: None,
            use_shared_accounts: None,
            fee_account: None,
            compute_unit_price_micro_lamports: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwapResponse {
    pub raw: Value,
    pub swap_transaction: String,
    pub last_valid_block_height: Option<u64>,
    pub priority_fee_micro_lamports: Option<u64>,
}

impl SwapResponse {
    pub fn try_from_value(value: Value) -> Result<Self, String> {
        let raw = value;
        let swap_transaction = raw
            .get("swapTransaction")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "swapTransaction missing".to_string())?
            .to_string();

        let last_valid_block_height = raw.get("lastValidBlockHeight").and_then(|v| v.as_u64());
        let priority_fee_micro_lamports = raw
            .get("prioritizationFeeLamports")
            .and_then(|v| v.as_u64())
            .or_else(|| {
                raw.get("prioritizationFeeMicroLamports")
                    .and_then(|v| v.as_u64())
            });

        Ok(Self {
            raw,
            swap_transaction,
            last_valid_block_height,
            priority_fee_micro_lamports,
        })
    }
}
