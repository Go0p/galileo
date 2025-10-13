use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use bincode::{config::standard, serde::encode_to_vec};
use reqwest::Client;
use serde_json::json;
use tracing::warn;

use crate::engine::PreparedTransaction;

use super::error::LanderError;
use super::stack::{Deadline, LanderReceipt};

#[derive(Clone)]
pub struct StakedLander {
    endpoints: Vec<String>,
    client: Client,
}

impl StakedLander {
    pub fn new(endpoints: Vec<String>, client: Client) -> Self {
        Self { endpoints, client }
    }

    pub async fn submit(
        &self,
        prepared: &PreparedTransaction,
        deadline: Deadline,
    ) -> Result<LanderReceipt, LanderError> {
        if deadline.expired() {
            return Err(LanderError::fatal(
                "deadline expired before staked submission",
            ));
        }

        let encoded = BASE64.encode(encode_to_vec(&prepared.transaction, standard())?);

        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [encoded, {"encoding": "base64"}],
        });

        for endpoint in &self.endpoints {
            if endpoint.trim().is_empty() {
                continue;
            }

            let response = self
                .client
                .post(endpoint)
                .json(&payload)
                .send()
                .await
                .map_err(LanderError::Network)?;

            if !response.status().is_success() {
                warn!(
                    target: "lander::staked",
                    endpoint,
                    status = %response.status(),
                    "sendTransaction returned non-success status"
                );
                continue;
            }

            let value: serde_json::Value = response.json().await.map_err(LanderError::Network)?;
            if let Some(error) = value.get("error") {
                warn!(
                    target: "lander::staked",
                    endpoint,
                    error = %error,
                    "sendTransaction returned error"
                );
                continue;
            }

            let signature = value
                .get("result")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            return Ok(LanderReceipt {
                lander: "staked",
                endpoint: endpoint.clone(),
                slot: prepared.slot,
                blockhash: prepared.blockhash.to_string(),
                signature,
            });
        }

        Err(LanderError::fatal(
            "all staked endpoints failed sendTransaction",
        ))
    }
}
