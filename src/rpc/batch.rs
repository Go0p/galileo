use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use once_cell::sync::Lazy;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue};
use serde::Deserialize;
use serde_json::Value;
use solana_client::client_error::{ClientError, Result as ClientResult};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_custom_error::{self, NodeUnhealthyErrorData};
use solana_client::rpc_request::{RpcError, RpcRequest, RpcResponseErrorData};
use solana_client::rpc_response::RpcSimulateTransactionResult;

#[derive(Clone)]
pub struct RpcBatchRequest {
    pub request: RpcRequest,
    pub params: Value,
}

impl RpcBatchRequest {
    pub fn new(request: RpcRequest, params: Value) -> Self {
        Self { request, params }
    }
}

#[derive(Debug, Deserialize)]
struct RpcErrorObject {
    code: i64,
    message: String,
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

static HTTP_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("solana-client"),
        HeaderValue::from_static("rust-batch"),
    );
    reqwest::Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .pool_idle_timeout(Duration::from_secs(30))
        .build()
        .expect("build rpc batch client")
});

pub async fn send_batch(
    client: &RpcClient,
    requests: &[RpcBatchRequest],
) -> ClientResult<Vec<Value>> {
    if requests.is_empty() {
        return Ok(Vec::new());
    }

    let mut payload = Vec::with_capacity(requests.len());
    let mut id_to_index = HashMap::with_capacity(requests.len());

    for (index, entry) in requests.iter().enumerate() {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        id_to_index.insert(id, index);
        payload.push(entry.request.build_request_json(id, entry.params.clone()));
    }

    let response = HTTP_CLIENT
        .post(client.url())
        .header(CONTENT_TYPE, "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(ClientError::from)?
        .error_for_status()
        .map_err(ClientError::from)?;

    let raw = response.json::<Value>().await.map_err(ClientError::from)?;
    let first_request = requests[0].request;
    let entries = raw.as_array().ok_or_else(|| {
        ClientError::from(RpcError::ParseError(
            "batch response must be an array".into(),
        ))
        .into_with_request(first_request)
    })?;

    let mut results = vec![Value::Null; requests.len()];

    for entry in entries {
        let id = entry
            .get("id")
            .and_then(|value| value.as_u64())
            .ok_or_else(|| {
                ClientError::from(RpcError::ParseError(
                    "batch response entry missing id".into(),
                ))
                .into_with_request(first_request)
            })?;

        let index = *id_to_index.get(&id).ok_or_else(|| {
            ClientError::from(RpcError::ParseError(
                "batch response id not recognized".into(),
            ))
            .into_with_request(first_request)
        })?;

        if let Some(error) = entry.get("error") {
            return Err(parse_rpc_error(requests[index].request, error));
        }

        if let Some(result) = entry.get("result") {
            results[index] = result.clone();
        } else {
            return Err(ClientError::from(RpcError::ParseError(
                "batch response entry missing result".into(),
            ))
            .into_with_request(requests[index].request));
        }
    }

    Ok(results)
}

fn parse_rpc_error(request: RpcRequest, error: &Value) -> ClientError {
    match serde_json::from_value::<RpcErrorObject>(error.clone()) {
        Ok(rpc_error_object) => {
            let data = match rpc_error_object.code {
                rpc_custom_error::JSON_RPC_SERVER_ERROR_SEND_TRANSACTION_PREFLIGHT_FAILURE => error
                    .get("data")
                    .cloned()
                    .and_then(|value| {
                        serde_json::from_value::<RpcSimulateTransactionResult>(value).ok()
                    })
                    .map_or(
                        RpcResponseErrorData::Empty,
                        RpcResponseErrorData::SendTransactionPreflightFailure,
                    ),
                rpc_custom_error::JSON_RPC_SERVER_ERROR_NODE_UNHEALTHY => error
                    .get("data")
                    .cloned()
                    .and_then(|value| serde_json::from_value::<NodeUnhealthyErrorData>(value).ok())
                    .map_or(RpcResponseErrorData::Empty, |data| {
                        RpcResponseErrorData::NodeUnhealthy {
                            num_slots_behind: data.num_slots_behind,
                        }
                    }),
                _ => RpcResponseErrorData::Empty,
            };

            let rpc_error = RpcError::RpcResponseError {
                code: rpc_error_object.code,
                message: rpc_error_object.message,
                data,
            };
            let client_error: ClientError = rpc_error.into();
            client_error.into_with_request(request)
        }
        Err(err) => ClientError::from(err).into_with_request(request),
    }
}
