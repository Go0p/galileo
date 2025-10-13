use bincode::config::standard;
use bincode::serde::encode_to_vec;
use reqwest::Client;
use serde_json::json;
use tracing::warn;

use crate::engine::PreparedTransaction;

use super::error::LanderError;
use super::stack::{Deadline, LanderReceipt};

#[derive(Clone)]
pub struct JitoLander {
    endpoints: Vec<String>,
    client: Client,
}

impl JitoLander {
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
                "deadline expired before jito submission",
            ));
        }

        let transaction_bytes = encode_to_vec(&prepared.transaction, standard())?;
        let encoded = bs58::encode(transaction_bytes).into_string();
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [[encoded]],
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
                    target: "lander::jito",
                    endpoint,
                    status = %response.status(),
                    "bundle submission returned non-success status"
                );
                continue;
            }

            let value: serde_json::Value = response.json().await.map_err(LanderError::Network)?;
            if let Some(error) = value.get("error") {
                warn!(
                    target: "lander::jito",
                    endpoint,
                    error = %error,
                    "bundle submission returned error"
                );
                continue;
            }

            let bundle_id = value
                .get("result")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            return Ok(LanderReceipt {
                lander: "jito",
                endpoint: endpoint.clone(),
                slot: prepared.slot,
                blockhash: prepared.blockhash.to_string(),
                signature: bundle_id,
            });
        }

        Err(LanderError::fatal("all jito endpoints failed submission"))
    }
}
