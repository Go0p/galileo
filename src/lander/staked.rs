use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use bincode::{config::standard, serde::encode_to_vec};
use reqwest::Client;
use serde_json::json;
use tracing::warn;

use crate::engine::{TxVariant, VariantId};

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

    pub fn endpoints_len(&self) -> usize {
        self.endpoints.len()
    }

    pub fn endpoint_list(&self) -> Vec<String> {
        self.endpoints.clone()
    }

    pub async fn submit_variant(
        &self,
        variant: TxVariant,
        deadline: Deadline,
        endpoint: Option<&str>,
    ) -> Result<LanderReceipt, LanderError> {
        if deadline.expired() {
            return Err(LanderError::fatal(
                "deadline expired before staked submission",
            ));
        }

        let encoded = BASE64.encode(encode_to_vec(variant.transaction(), standard())?);

        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [encoded, {"encoding": "base64"}],
        });

        let slot = variant.slot();
        let blockhash = variant.blockhash().to_string();
        let variant_id = variant.id();

        match endpoint {
            Some(target) => {
                self.send_once(target, &payload, slot, &blockhash, variant_id)
                    .await
            }
            None => {
                let mut last_err: Option<LanderError> = None;
                for endpoint in &self.endpoints {
                    if endpoint.trim().is_empty() {
                        continue;
                    }

                    match self
                        .send_once(endpoint, &payload, slot, &blockhash, variant_id)
                        .await
                    {
                        Ok(receipt) => return Ok(receipt),
                        Err(err) => {
                            last_err = Some(err);
                        }
                    }
                }

                Err(last_err.unwrap_or_else(|| {
                    LanderError::fatal("all staked endpoints failed sendTransaction")
                }))
            }
        }
    }

    async fn send_once(
        &self,
        endpoint: &str,
        payload: &serde_json::Value,
        slot: u64,
        blockhash: &str,
        variant_id: VariantId,
    ) -> Result<LanderReceipt, LanderError> {
        let response = self
            .client
            .post(endpoint)
            .json(payload)
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
            return Err(LanderError::fatal("staked endpoint rejected request"));
        }

        let value: serde_json::Value = response.json().await.map_err(LanderError::Network)?;
        if let Some(error) = value.get("error") {
            warn!(
                target: "lander::staked",
                endpoint,
                error = %error,
                "sendTransaction returned error"
            );
            return Err(LanderError::fatal("staked endpoint returned error payload"));
        }

        let signature = value
            .get("result")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(LanderReceipt {
            lander: "staked",
            endpoint: endpoint.to_string(),
            slot,
            blockhash: blockhash.to_string(),
            signature,
            variant_id,
        })
    }
}
