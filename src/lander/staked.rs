use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use bincode::{config::standard, serde::encode_to_vec};
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use reqwest::Client;
use serde_json::{Value, json, map::Map};
use std::net::IpAddr;
use std::sync::Arc;
use tracing::{info, warn};

use crate::engine::{TxVariant, VariantId};
use crate::network::{IpBoundClientPool, ReqwestClientFactoryFn};

use super::error::LanderError;
use super::stack::{Deadline, LanderReceipt};

#[derive(Clone)]
pub struct StakedLander {
    endpoints: Vec<String>,
    client: Client,
    skip_preflight: Option<bool>,
    max_retries: Option<usize>,
    min_context_slot: Option<u64>,
    client_pool: Option<Arc<IpBoundClientPool<ReqwestClientFactoryFn>>>,
    enable_simulation: bool,
}

impl StakedLander {
    #[allow(dead_code)]
    pub fn new(
        endpoints: Vec<String>,
        client: Client,
        skip_preflight: Option<bool>,
        max_retries: Option<usize>,
        min_context_slot: Option<u64>,
        enable_simulation: bool,
    ) -> Self {
        Self {
            endpoints,
            client,
            skip_preflight,
            max_retries,
            min_context_slot,
            client_pool: None,
            enable_simulation,
        }
    }

    pub fn with_ip_pool(
        endpoints: Vec<String>,
        client: Client,
        skip_preflight: Option<bool>,
        max_retries: Option<usize>,
        min_context_slot: Option<u64>,
        client_pool: Option<Arc<IpBoundClientPool<ReqwestClientFactoryFn>>>,
        enable_simulation: bool,
    ) -> Self {
        Self {
            endpoints,
            client,
            skip_preflight,
            max_retries,
            min_context_slot,
            client_pool,
            enable_simulation,
        }
    }

    pub fn endpoints_len(&self) -> usize {
        self.endpoints.len()
    }

    pub fn endpoint_list(&self) -> Vec<String> {
        self.endpoints.clone()
    }

    fn http_client(&self, local_ip: Option<IpAddr>) -> Result<Client, LanderError> {
        if let Some(ip) = local_ip {
            if let Some(pool) = &self.client_pool {
                return pool
                    .get_or_create(ip)
                    .map_err(|err| LanderError::fatal(format!("构建绑定 IP 的客户端失败: {err}")));
            }
        }
        Ok(self.client.clone())
    }

    pub async fn submit_variant(
        &self,
        variant: TxVariant,
        deadline: Deadline,
        endpoint: Option<&str>,
        local_ip: Option<IpAddr>,
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
        let client = self.http_client(local_ip)?;

        if self.enable_simulation {
            return self
                .simulate_variant(
                    &client,
                    endpoint,
                    encoded.clone(),
                    slot,
                    &blockhash,
                    variant_id,
                    local_ip,
                )
                .await;
        }

        if let Some(target) = endpoint {
            return self
                .send_once(
                    &client, target, &payload, slot, &blockhash, variant_id, local_ip,
                )
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
            let client = client.clone();

            futures.push(async move {
                lander
                    .send_once(
                        &client,
                        &endpoint_clone,
                        &payload_clone,
                        slot,
                        blockhash_clone.as_str(),
                        variant_id,
                        local_ip,
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

    async fn simulate_variant(
        &self,
        client: &Client,
        endpoint: Option<&str>,
        encoded: String,
        slot: u64,
        blockhash: &str,
        variant_id: VariantId,
        local_ip: Option<IpAddr>,
    ) -> Result<LanderReceipt, LanderError> {
        let target_list: Vec<String> = if let Some(single) = endpoint {
            vec![single.to_string()]
        } else {
            self.endpoints
                .clone()
                .into_iter()
                .filter(|value| !value.trim().is_empty())
                .collect()
        };

        if target_list.is_empty() {
            return Err(LanderError::fatal(
                "no staked endpoints configured for simulation",
            ));
        }

        let mut last_err: Option<LanderError> = None;
        for target in target_list {
            let mut config = Map::with_capacity(4);
            config.insert("encoding".to_string(), json!("base64"));
            config.insert("sigVerify".to_string(), json!(true));
            if let Some(slot) = self.min_context_slot {
                if slot > 0 {
                    config.insert("minContextSlot".to_string(), json!(slot));
                }
            }
            let simulate_payload = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "simulateTransaction",
                "params": [
                    encoded,
                    Value::Object(config),
                ],
            });

            match client.post(&target).json(&simulate_payload).send().await {
                Ok(response) => match response.json::<Value>().await {
                    Ok(value) => {
                        let err = value.get("error").cloned().unwrap_or(Value::Null);
                        let logs = value
                            .pointer("/result/value/logs")
                            .cloned()
                            .unwrap_or(Value::Null);
                        info!(
                            target: "lander::staked",
                            endpoint = %target,
                            error = ?err,
                            logs = ?logs,
                            "simulateTransaction completed"
                        );
                        return Ok(LanderReceipt {
                            lander: "staked",
                            endpoint: target,
                            slot,
                            blockhash: blockhash.to_string(),
                            signature: None,
                            variant_id,
                            local_ip,
                        });
                    }
                    Err(err) => {
                        last_err = Some(LanderError::Network(err));
                    }
                },
                Err(err) => {
                    last_err = Some(LanderError::Network(err));
                }
            }
        }

        Err(last_err.unwrap_or_else(|| {
            LanderError::fatal("all staked endpoints failed simulateTransaction")
        }))
    }

    async fn send_once(
        &self,
        client: &Client,
        endpoint: &str,
        payload: &serde_json::Value,
        slot: u64,
        blockhash: &str,
        variant_id: VariantId,
        local_ip: Option<IpAddr>,
    ) -> Result<LanderReceipt, LanderError> {
        let response = client
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
            local_ip,
        })
    }
}
