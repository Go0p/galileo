#![allow(dead_code)] // TODO: 集成 Ultra API 后删除该抑制，确保未使用代码及时清理。

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use super::serde_helpers::{field_as_string, option_field_as_string};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteRequest {
    pub signed_transaction: String,
    pub request_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum ExecuteStatus {
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteResponse {
    pub status: ExecuteStatus,
    #[serde(default)]
    pub signature: Option<String>,
    #[serde(default, with = "option_field_as_string")]
    pub slot: Option<u64>,
    #[serde(default)]
    pub error: Option<String>,
    pub code: i64,
    #[serde(default, with = "option_field_as_string")]
    pub total_input_amount: Option<u64>,
    #[serde(default, with = "option_field_as_string")]
    pub total_output_amount: Option<u64>,
    #[serde(default, with = "option_field_as_string")]
    pub input_amount_result: Option<u64>,
    #[serde(default, with = "option_field_as_string")]
    pub output_amount_result: Option<u64>,
    #[serde(default)]
    pub swap_events: Vec<SwapEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapEvent {
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub input_amount: u64,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub output_amount: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use solana_sdk::pubkey;

    #[test]
    fn deserialize_execute_response() {
        let value = json!({
            "status": "Success",
            "signature": "5N8n7nJXG1",
            "slot": "123456",
            "error": null,
            "code": 200,
            "totalInputAmount": "1000",
            "totalOutputAmount": "950",
            "inputAmountResult": "1000",
            "outputAmountResult": "950",
            "swapEvents": [
                {
                    "inputMint": "So11111111111111111111111111111111111111112",
                    "inputAmount": "1000",
                    "outputMint": "Es9vMFrzaCERz7GosCNtvyWxirBM8dEjFCP4F9juF2Nv",
                    "outputAmount": "950"
                }
            ]
        });
        let response: ExecuteResponse = serde_json::from_value(value).expect("parse");
        assert_eq!(response.status, ExecuteStatus::Success);
        assert_eq!(response.total_output_amount, Some(950));
        assert_eq!(response.swap_events.len(), 1);
        let event = &response.swap_events[0];
        assert_eq!(
            event.input_mint,
            pubkey!("So11111111111111111111111111111111111111112")
        );
    }
}
