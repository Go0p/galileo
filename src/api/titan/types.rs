use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

pub type RequestId = u32;
pub type StreamId = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientRequest {
    pub id: RequestId,
    pub data: RequestData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestData {
    #[serde(rename = "GetInfo")]
    GetInfo(GetInfoRequest),
    #[serde(rename = "NewSwapQuoteStream")]
    NewSwapQuoteStream(SwapQuoteRequest),
    #[serde(rename = "StopStream")]
    StopStream(StopStreamRequest),
    #[serde(rename = "GetVenues")]
    GetVenues(GetVenuesRequest),
    #[serde(rename = "ListProviders")]
    ListProviders(ListProvidersRequest),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct GetInfoRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapQuoteRequest {
    pub swap: SwapParams,
    pub transaction: TransactionParams,
    #[serde(default)]
    pub update: Option<QuoteUpdateParams>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapParams {
    #[serde(with = "pubkey_bytes")]
    pub input_mint: Pubkey,
    #[serde(with = "pubkey_bytes")]
    pub output_mint: Pubkey,
    pub amount: u64,
    #[serde(default)]
    pub swap_mode: Option<SwapMode>,
    #[serde(default)]
    pub slippage_bps: Option<u16>,
    #[serde(default)]
    pub dexes: Option<Vec<String>>,
    #[serde(default)]
    pub exclude_dexes: Option<Vec<String>>,
    #[serde(default)]
    pub only_direct_routes: Option<bool>,
    #[serde(default)]
    pub add_size_constraint: Option<bool>,
    #[serde(default)]
    pub size_constraint: Option<u32>,
    #[serde(default)]
    pub providers: Option<Vec<String>>,
    #[serde(default)]
    pub accounts_limit_total: Option<u16>,
    #[serde(default)]
    pub accounts_limit_writable: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionParams {
    #[serde(with = "pubkey_bytes")]
    pub user_public_key: Pubkey,
    #[serde(default)]
    pub close_input_token_account: Option<bool>,
    #[serde(default)]
    pub create_output_token_account: Option<bool>,
    #[serde(default, with = "option_pubkey_bytes")]
    pub fee_account: Option<Pubkey>,
    #[serde(default)]
    pub fee_bps: Option<u16>,
    #[serde(default)]
    pub fee_from_input_mint: Option<bool>,
    #[serde(default, with = "option_pubkey_bytes")]
    pub output_account: Option<Pubkey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteUpdateParams {
    #[serde(default)]
    pub interval_ms: Option<u64>,
    #[serde(default)]
    pub num_quotes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopStreamRequest {
    pub id: StreamId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVenuesRequest {
    #[serde(default)]
    pub include_program_ids: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListProvidersRequest {
    #[serde(default)]
    pub include_icons: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    #[serde(rename = "Response")]
    Response(ResponseSuccess),
    #[serde(rename = "Error")]
    Error(ResponseError),
    #[serde(rename = "StreamData")]
    StreamData(StreamData),
    #[serde(rename = "StreamEnd")]
    StreamEnd(StreamEnd),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseSuccess {
    pub request_id: RequestId,
    pub data: ResponseData,
    #[serde(default)]
    pub stream: Option<StreamStart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseData {
    #[serde(rename = "GetInfo")]
    GetInfo(ServerInfo),
    #[serde(rename = "NewSwapQuoteStream")]
    NewSwapQuoteStream(QuoteSwapStreamResponse),
    #[serde(rename = "StreamStopped")]
    StreamStopped(StopStreamResponse),
    #[serde(rename = "GetVenues")]
    GetVenues(VenueInfo),
    #[serde(rename = "ListProviders")]
    ListProviders(Vec<ProviderInfo>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseError {
    pub request_id: RequestId,
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamStart {
    pub id: StreamId,
    pub data_type: StreamDataType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamDataType {
    #[serde(rename = "SwapQuotes")]
    SwapQuotes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamData {
    pub id: StreamId,
    pub seq: u64,
    pub payload: StreamDataPayload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamDataPayload {
    #[serde(rename = "SwapQuotes")]
    SwapQuotes(SwapQuotes),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamEnd {
    pub id: StreamId,
    #[serde(default)]
    pub error_code: Option<i32>,
    #[serde(default)]
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundedValueWithDefault<T> {
    pub min: T,
    pub max: T,
    pub default: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteUpdateSettings {
    pub interval_ms: BoundedValueWithDefault<u64>,
    pub num_quotes: BoundedValueWithDefault<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapSettings {
    pub slippage_bps: BoundedValueWithDefault<u16>,
    pub only_direct_routes: bool,
    pub add_size_constraint: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionSettings {
    pub close_input_token_account: bool,
    pub create_output_token_account: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionSettings {
    pub concurrent_streams: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSettings {
    pub quote_update: QuoteUpdateSettings,
    pub swap: SwapSettings,
    pub transaction: TransactionSettings,
    pub connection: ConnectionSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    pub protocol_version: VersionInfo,
    pub settings: ServerSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteSwapStreamResponse {
    pub interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StopStreamResponse {
    pub id: StreamId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VenueInfo {
    pub labels: Vec<String>,
    #[serde(default, with = "option_vec_pubkey_bytes")]
    pub program_ids: Option<Vec<Pubkey>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderKind {
    #[serde(rename = "DexAggregator")]
    DexAggregator,
    #[serde(rename = "RFQ")]
    Rfq,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub kind: ProviderKind,
    #[serde(default)]
    pub icon_uri48: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutePlanStep {
    #[serde(with = "pubkey_bytes")]
    pub amm_key: Pubkey,
    pub label: String,
    #[serde(with = "pubkey_bytes")]
    pub input_mint: Pubkey,
    #[serde(with = "pubkey_bytes")]
    pub output_mint: Pubkey,
    pub in_amount: u64,
    pub out_amount: u64,
    pub alloc_ppb: u32,
    #[serde(default, with = "option_pubkey_bytes")]
    pub fee_mint: Option<Pubkey>,
    #[serde(default)]
    pub fee_amount: Option<u64>,
    #[serde(default)]
    pub context_slot: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformFee {
    pub amount: u64,
    #[serde(rename = "feeBps")]
    pub fee_bps: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapRoute {
    pub in_amount: u64,
    pub out_amount: u64,
    pub slippage_bps: u16,
    #[serde(default)]
    pub platform_fee: Option<PlatformFee>,
    pub steps: Vec<RoutePlanStep>,
    pub instructions: Vec<Instruction>,
    #[serde(with = "pubkey_vec")]
    pub address_lookup_tables: Vec<Pubkey>,
    #[serde(default)]
    pub context_slot: Option<u64>,
    #[serde(default)]
    pub time_taken_ns: Option<u64>,
    #[serde(default)]
    pub expires_at_ms: Option<u64>,
    #[serde(default)]
    pub expires_after_slot: Option<u64>,
    #[serde(default)]
    pub compute_units: Option<u64>,
    #[serde(default)]
    pub compute_units_safe: Option<u64>,
    #[serde(default)]
    pub transaction: Option<Vec<u8>>,
    #[serde(default, rename = "referenceId")]
    pub reference_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapQuotes {
    pub id: String,
    #[serde(with = "pubkey_bytes")]
    pub input_mint: Pubkey,
    #[serde(with = "pubkey_bytes")]
    pub output_mint: Pubkey,
    pub swap_mode: SwapMode,
    pub amount: u64,
    pub quotes: HashMap<String, SwapRoute>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SwapMode {
    #[serde(rename = "ExactIn")]
    ExactIn,
    #[serde(rename = "ExactOut")]
    ExactOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountMeta {
    #[serde(rename = "p", with = "pubkey_bytes")]
    pub pubkey: Pubkey,
    #[serde(rename = "s")]
    pub signer: bool,
    #[serde(rename = "w")]
    pub writable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    #[serde(rename = "p", with = "pubkey_bytes")]
    pub program_id: Pubkey,
    #[serde(rename = "a")]
    pub accounts: Vec<AccountMeta>,
    #[serde(rename = "d")]
    pub data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn client_request_roundtrip() {
        let request = ClientRequest {
            id: 7,
            data: RequestData::NewSwapQuoteStream(SwapQuoteRequest {
                swap: SwapParams {
                    input_mint: Pubkey::new_unique(),
                    output_mint: Pubkey::new_unique(),
                    amount: 42,
                    swap_mode: Some(SwapMode::ExactIn),
                    slippage_bps: Some(50),
                    dexes: Some(vec!["ExampleDex".to_string()]),
                    exclude_dexes: None,
                    only_direct_routes: Some(false),
                    add_size_constraint: Some(true),
                    size_constraint: Some(900),
                    providers: None,
                    accounts_limit_total: None,
                    accounts_limit_writable: None,
                },
                transaction: TransactionParams {
                    user_public_key: Pubkey::new_unique(),
                    close_input_token_account: Some(false),
                    create_output_token_account: Some(true),
                    fee_account: None,
                    fee_bps: Some(5),
                    fee_from_input_mint: Some(false),
                    output_account: None,
                },
                update: Some(QuoteUpdateParams {
                    interval_ms: Some(1_000),
                    num_quotes: Some(4),
                }),
            }),
        };

        let expected_key = match &request.data {
            RequestData::NewSwapQuoteStream(inner) => inner.transaction.user_public_key,
            _ => unreachable!(),
        };
        let encoded = rmp_serde::to_vec_named(&request).expect("encode request");
        let decoded: ClientRequest = rmp_serde::from_slice(&encoded).expect("decode request");
        assert_eq!(decoded.id, request.id);
        match decoded.data {
            RequestData::NewSwapQuoteStream(inner) => {
                assert_eq!(inner.swap.amount, 42);
                assert_eq!(inner.transaction.user_public_key, expected_key);
            }
            other => panic!("unexpected variant: {:?}", other),
        }
    }

    #[test]
    fn server_response_roundtrip() {
        let stream_start = StreamStart {
            id: 1,
            data_type: StreamDataType::SwapQuotes,
        };
        let response = ServerMessage::Response(ResponseSuccess {
            request_id: 7,
            data: ResponseData::NewSwapQuoteStream(QuoteSwapStreamResponse { interval_ms: 800 }),
            stream: Some(stream_start),
        });

        let encoded = rmp_serde::to_vec_named(&response).expect("encode response");
        let decoded: ServerMessage = rmp_serde::from_slice(&encoded).expect("decode response");
        match decoded {
            ServerMessage::Response(success) => {
                assert_eq!(success.request_id, 7);
                assert_eq!(success.stream.unwrap().id, 1);
            }
            other => panic!("unexpected message: {:?}", other),
        }
    }
}

/// Serde helper module for encoding/decoding `Pubkey` as 32 bytes.
mod pubkey_bytes {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(value: &Pubkey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(value.as_ref())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: &[u8] = Deserialize::deserialize(deserializer)?;
        Pubkey::try_from(bytes).map_err(serde::de::Error::custom)
    }
}

mod option_pubkey_bytes {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<Pubkey>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(key) => serializer.serialize_some(key.as_ref()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Pubkey>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let option: Option<&[u8]> = Option::deserialize(deserializer)?;
        option
            .map(Pubkey::try_from)
            .transpose()
            .map_err(serde::de::Error::custom)
    }
}

mod option_vec_pubkey_bytes {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<Vec<Pubkey>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(vec) => {
                let bytes: Vec<&[u8]> = vec.iter().map(|pk| pk.as_ref()).collect();
                serializer.serialize_some(&bytes)
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<Pubkey>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let option: Option<Vec<Vec<u8>>> = Option::deserialize(deserializer)?;
        option
            .map(|entries| {
                entries
                    .into_iter()
                    .map(|bytes| Pubkey::try_from(bytes.as_slice()))
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()
            .map_err(serde::de::Error::custom)
    }
}

mod pubkey_vec {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(value: &Vec<Pubkey>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes: Vec<&[u8]> = value.iter().map(|pk| pk.as_ref()).collect();
        bytes.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Pubkey>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let entries: Vec<Vec<u8>> = Vec::deserialize(deserializer)?;
        entries
            .into_iter()
            .map(|bytes| Pubkey::try_from(bytes.as_slice()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(serde::de::Error::custom)
    }
}
