use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use bincode::{config::standard, serde::encode_to_vec};
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use reqwest::Client;
use serde_json::{Value, json, map::Map};
use tracing::warn;

use crate::engine::{TxVariant, VariantId};

use super::error::LanderError;
use super::stack::{Deadline, LanderReceipt};

#[derive(Clone)]
pub struct StakedLander {
    endpoints: Vec<String>,
    client: Client,
    skip_preflight: Option<bool>,
    max_retries: Option<usize>,
    min_context_slot: Option<u64>,
}

impl StakedLander {
    pub fn new(
        endpoints: Vec<String>,
        client: Client,
        skip_preflight: Option<bool>,
        max_retries: Option<usize>,
        min_context_slot: Option<u64>,
    ) -> Self {
        Self {
            endpoints,
            client,
            skip_preflight,
            max_retries,
            min_context_slot,
        }
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

        let mut config = Map::with_capacity(4);
        config.insert("encoding".to_string(), json!("base64"));
        if let Some(skip) = self.skip_preflight {
            config.insert("skipPreflight".to_string(), json!(skip));
        }
        if let Some(retries) = self.max_retries {
            config.insert("maxRetries".to_string(), json!(retries));
        }
        if let Some(slot) = self.min_context_slot {
            if slot > 0 {
                config.insert("minContextSlot".to_string(), json!(slot));
            }
        }

        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendTransaction",
            "params": [
                encoded,
                Value::Object(config),
            ],
        });

        let slot = variant.slot();
        let blockhash = variant.blockhash().to_string();
        let variant_id = variant.id();

        if let Some(target) = endpoint {
            return self
                .send_once(target, &payload, slot, &blockhash, variant_id)
                .await;
        }

        if self.endpoints.is_empty() {
            return Err(LanderError::fatal("no staked endpoints configured"));
        }

        let mut futures = FuturesUnordered::new();
        for endpoint in self.endpoints.clone() {
            if endpoint.trim().is_empty() {
                continue;
            }

            let lander = self.clone();
            let payload_clone = payload.clone();
            let endpoint_clone = endpoint.clone();
            let blockhash_clone = blockhash.clone();

            futures.push(async move {
                lander
                    .send_once(
                        &endpoint_clone,
                        &payload_clone,
                        slot,
                        blockhash_clone.as_str(),
                        variant_id,
                    )
                    .await
                    .map(|receipt| (endpoint_clone, receipt))
            });
        }

        let mut last_err: Option<LanderError> = None;
        while let Some(result) = futures.next().await {
            match result {
                Ok((_endpoint, receipt)) => return Ok(receipt),
                Err(err) => {
                    warn!(
                        target: "lander::staked",
                        error = %err,
                        "sendTransaction failed"
                    );
                    last_err = Some(err);
                }
            }
        }

        Err(last_err
            .unwrap_or_else(|| LanderError::fatal("all staked endpoints failed sendTransaction")))
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
