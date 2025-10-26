#![allow(dead_code)] // TODO: 集成 Ultra API 后删除该抑制，确保未使用代码及时清理。

use std::collections::HashMap;
use std::fmt;
use std::ops::Deref;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;

use super::serde_helpers::{field_as_string, option_field_as_string};

pub use crate::api::jupiter::quote::RoutePlanStep;
pub use crate::api::jupiter::quote::SwapInfo;
pub use crate::api::jupiter::quote::SwapMode;

#[derive(Debug, Clone, Default)]
pub struct RouterLabels(String);

impl RouterLabels {
    pub fn new(labels: String) -> Self {
        Self(labels)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RouterLabels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RouterParam(&'static str);

impl RouterParam {
    pub const METIS: Self = Self("metis");
    pub const JUPITERZ: Self = Self("jupiterz");
    pub const DFLOW: Self = Self("dflow");
    pub const OKX: Self = Self("okx");

    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

impl fmt::Display for RouterParam {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Router(pub RouterParam);

impl Router {
    pub fn metis() -> Self {
        Self(RouterParam::METIS)
    }

    pub fn jupiterz() -> Self {
        Self(RouterParam::JUPITERZ)
    }

    pub fn dflow() -> Self {
        Self(RouterParam::DFLOW)
    }

    pub fn okx() -> Self {
        Self(RouterParam::OKX)
    }

    pub fn as_str(&self) -> &'static str {
        self.0.as_str()
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrderRequest {
    #[serde(with = "field_as_string")]
    pub input_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub output_mint: Pubkey,
    #[serde(with = "field_as_string")]
    pub amount: u64,
    #[serde(default = "default_swap_mode")]
    pub swap_mode: SwapMode,
    #[serde(default = "default_slippage_bps")]
    pub slippage_bps: u16,
    #[serde(default = "default_use_wsol")]
    pub use_wsol: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub broadcast_fee_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jito_tip_lamports: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority_fee_lamports: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_dexes: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "option_field_as_string"
    )]
    pub taker: Option<Pubkey>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "option_field_as_string"
    )]
    pub referral_account: Option<Pubkey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referral_fee: Option<u8>,
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "serialize_router_list"
    )]
    pub exclude_routers: Vec<Router>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        with = "option_field_as_string"
    )]
    pub payer: Option<Pubkey>,
    #[serde(skip)]
    pub extra_query_params: HashMap<String, String>,
}

impl OrderRequest {
    pub fn new(input_mint: Pubkey, output_mint: Pubkey, amount: u64) -> Self {
        Self {
            input_mint,
            output_mint,
            amount,
            swap_mode: SwapMode::ExactIn,
            slippage_bps: default_slippage_bps(),
            use_wsol: default_use_wsol(),
            broadcast_fee_type: None,
            jito_tip_lamports: None,
            priority_fee_lamports: None,
            taker: None,
            referral_account: None,
            referral_fee: None,
            exclude_routers: Vec::new(),
            exclude_dexes: None,
            payer: None,
            extra_query_params: HashMap::new(),
        }
    }

    pub fn exclude_routers_label(&self) -> RouterLabels {
        if self.exclude_routers.is_empty() {
            return RouterLabels::new("<none>".to_string());
        }
        RouterLabels::new(
            self.exclude_routers
                .iter()
                .map(|router| router.as_str())
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

fn default_swap_mode() -> SwapMode {
    SwapMode::ExactIn
}

const fn default_slippage_bps() -> u16 {
    0
}

const fn default_use_wsol() -> bool {
    false
}

fn serialize_router_list<S>(routers: &[Router], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let joined = routers
        .iter()
        .map(|router| router.as_str())
        .collect::<Vec<_>>()
        .join(",");
    serializer.serialize_str(&joined)
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OrderResponsePayload {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default, with = "option_field_as_string")]
    pub input_mint: Option<Pubkey>,
    #[serde(default, with = "option_field_as_string")]
    pub output_mint: Option<Pubkey>,
    #[serde(default, with = "option_field_as_string")]
    pub in_amount: Option<u64>,
    #[serde(default, with = "option_field_as_string")]
    pub out_amount: Option<u64>,
    #[serde(default, with = "option_field_as_string")]
    pub other_amount_threshold: Option<u64>,
    #[serde(default)]
    pub swap_mode: Option<SwapMode>,
    #[serde(default)]
    pub slippage_bps: Option<u16>,
    #[serde(default)]
    pub in_usd_value: Decimal,
    #[serde(default)]
    pub out_usd_value: Decimal,
    #[serde(default)]
    pub price_impact: Decimal,
    #[serde(default)]
    pub swap_usd_value: Decimal,
    #[serde(default)]
    pub price_impact_pct: Option<String>,
    #[serde(default)]
    pub route_plan: Vec<RoutePlanStep>,
    #[serde(default, with = "option_field_as_string")]
    pub fee_mint: Option<Pubkey>,
    #[serde(default)]
    pub fee_bps: Option<u16>,
    #[serde(default)]
    pub signature_fee_lamports: Option<u64>,
    #[serde(default)]
    pub prioritization_fee_lamports: Option<u64>,
    #[serde(default)]
    pub rent_fee_lamports: Option<u64>,
    #[serde(default)]
    pub swap_type: Option<String>,
    #[serde(default)]
    pub router: Option<String>,
    #[serde(default)]
    pub transaction: Option<String>,
    #[serde(default)]
    pub gasless: Option<bool>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub total_time: Option<u64>,
    #[serde(default, with = "option_field_as_string")]
    pub taker: Option<Pubkey>,
    #[serde(default)]
    pub quote_id: Option<String>,
    #[serde(default, with = "option_field_as_string")]
    pub maker: Option<Pubkey>,
    #[serde(default)]
    pub expire_at: Option<String>,
    #[serde(default)]
    pub platform_fee: Option<UltraPlatformFee>,
    #[serde(default)]
    pub error_code: Option<i64>,
    #[serde(default)]
    pub error_message: Option<String>,
}

#[derive(Clone, Debug)]
pub struct OrderResponse {
    pub raw: Value,
    data: OrderResponsePayload,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct UltraPlatformFee {
    #[serde(default, with = "option_field_as_string")]
    pub amount: Option<u64>,
    #[serde(default)]
    pub fee_bps: Option<u16>,
}

impl OrderResponse {
    pub fn try_from_value(value: Value) -> Result<Self, serde_json::Error> {
        let data: OrderResponsePayload = serde_json::from_value(value.clone())?;
        Ok(Self { raw: value, data })
    }

    pub fn into_parts(self) -> (Value, OrderResponsePayload) {
        (self.raw, self.data)
    }
}

impl Deref for OrderResponse {
    type Target = OrderResponsePayload;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl fmt::Display for Router {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use solana_sdk::pubkey;

    #[test]
    fn serialize_order_request_exclude_routers() {
        let mut req = OrderRequest::new(
            pubkey!("So11111111111111111111111111111111111111112"),
            pubkey!("Es9vMFrzaCERz7GosCNtvyWxirBM8dEjFCP4F9juF2Nv"),
            1_234_567,
        );
        req.exclude_routers
            .extend([Router::metis(), Router::dflow(), Router::jupiterz()]);

        let encoded = serde_urlencoded::to_string(&req).expect("serialize");
        assert!(encoded.contains("excludeRouters=metis%2Cdflow%2Cjupiterz"));
    }

    #[test]
    fn deserialize_order_response_payload() {
        let payload = json!({
            "mode": "ExactIn",
            "inputMint": "So11111111111111111111111111111111111111112",
            "outputMint": "Es9vMFrzaCERz7gosCNtvyWxirBM8dEjFCP4F9juF2Nv",
            "inAmount": "1000",
            "outAmount": "950",
            "otherAmountThreshold": "900",
            "swapMode": "ExactIn",
            "slippageBps": 100,
            "inUsdValue": 10.5,
            "outUsdValue": 9.9,
            "priceImpact": 0.01,
            "swapUsdValue": 10.0,
            "priceImpactPct": "1%",
            "routePlan": [],
            "feeMint": null,
            "feeBps": 50,
            "signatureFeeLamports": 5000,
            "prioritizationFeeLamports": 2000,
            "rentFeeLamports": 1000,
            "swapType": "aggregator",
            "router": "aggregator",
            "transaction": "base64",
            "gasless": false,
            "requestId": "req-1",
            "totalTime": 12,
            "taker": "11111111111111111111111111111111",
            "quoteId": "quote-1",
            "maker": null,
            "expireAt": null,
            "platformFee": null,
            "errorCode": null,
            "errorMessage": null
        });
        let response = OrderResponse::try_from_value(payload).expect("parse");
        assert_eq!(response.swap_mode, Some(SwapMode::ExactIn));
        assert_eq!(response.swap_type.as_deref(), Some("aggregator"));
        assert_eq!(response.router.as_deref(), Some("aggregator"));
    }

    #[test]
    fn deserialize_order_response_without_in_amount() {
        let payload = json!({
            "inputMint": "So11111111111111111111111111111111111111112",
            "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "routePlan": [{
                "swapInfo": {
                    "ammKey": "11111111111111111111111111111111",
                    "label": "unit",
                    "inputMint": "So11111111111111111111111111111111111111112",
                    "outputMint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
                    "inAmount": "1",
                    "outAmount": "2",
                    "feeAmount": "0",
                    "feeMint": "11111111111111111111111111111111"
                },
                "percent": 100
            }],
            "swapMode": "ExactIn"
        });
        let response = OrderResponse::try_from_value(payload).expect("parse");
        assert_eq!(response.route_plan.len(), 1);
        assert_eq!(response.route_plan[0].swap_info.in_amount, 1);
    }
}
