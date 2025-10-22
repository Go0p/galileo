use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use bincode::config::standard;
use bincode::serde::encode_to_vec;
use once_cell::sync::Lazy;
use rand::seq::IndexedRandom;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use solana_system_interface::instruction as system_instruction;
use tokio::sync::Mutex;
use tracing::{debug, warn};
use url::Url;

use crate::config::{LanderJitoConfig, LanderJitoUuidConfig, TipFloorLevel, TipStrategyKind};
use crate::engine::{TxVariant, VariantId};

use super::error::LanderError;
use super::stack::{Deadline, LanderReceipt};

const JSONRPC_VERSION: &str = "2.0";
const DEFAULT_REQUEST_ID: &str = "galileo-jito";
const TIP_FLOOR_URL: &str = "https://bundles.jito.wtf/api/v1/bundles/tip_floor";
const TIP_FLOOR_CACHE_TTL: Duration = Duration::from_secs(3);

static TIP_WALLETS: Lazy<Vec<Pubkey>> = Lazy::new(|| {
    [
        "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
        "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
        "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
        "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
        "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
        "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
        "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
        "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    ]
    .iter()
    .filter_map(|value| Pubkey::from_str(value).ok())
    .collect()
});

#[derive(Clone)]
pub struct JitoLander {
    endpoints: Vec<String>,
    client: Client,
    tip_selector: TipSelector,
    uuid_pool: Option<Arc<Mutex<UuidPool>>>,
}

impl JitoLander {
    pub fn new(config: &LanderJitoConfig, client: Client) -> Self {
        let endpoints = config
            .endpoints
            .iter()
            .map(|endpoint| endpoint.trim().to_string())
            .filter(|endpoint| !endpoint.is_empty())
            .collect();

        let tip_selector = TipSelector::from_config(config);
        let uuid_pool = UuidPool::new(&config.uuid_config).map(|pool| Arc::new(Mutex::new(pool)));

        Self {
            endpoints,
            client,
            tip_selector,
            uuid_pool,
        }
    }

    pub fn endpoints(&self) -> usize {
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
                "deadline expired before jito submission",
            ));
        }

        let configured_tip = self.tip_selector.select_tip(&self.client).await?;
        let base_tip = if variant.tip_lamports() > 0 {
            Some(variant.tip_lamports())
        } else {
            None
        };
        let tip_lamports = variant
            .tip_override()
            .map(|override_tip| override_tip.lamports())
            .or(base_tip)
            .or(configured_tip)
            .unwrap_or(0);

        let mut bundle = Vec::new();
        if tip_lamports > 0 {
            if let Some(recipient) = variant
                .tip_override()
                .and_then(|override_tip| override_tip.recipient())
                .or_else(|| tip_wallet_for_variant(variant.id()))
                .or_else(random_tip_wallet)
            {
                let tip_tx = build_tip_transaction(&variant, recipient, tip_lamports);
                let tip_encoded = encode_transaction(&tip_tx)?;
                bundle.push(tip_encoded);
                debug!(
                    target: "lander::jito",
                    tip_lamports,
                    recipient = %recipient,
                    "added tip transaction to bundle"
                );
            } else {
                warn!(
                    target: "lander::jito",
                    tip_lamports,
                    "tip wallet list empty, skipping tip transaction"
                );
            }
        }

        let main_encoded = encode_transaction(variant.transaction())?;
        bundle.push(main_encoded);

        let bundle_value = Value::Array(
            bundle
                .iter()
                .map(|tx| Value::String(tx.clone()))
                .collect::<Vec<_>>(),
        );

        let target_endpoints: Vec<&str> = match endpoint {
            Some(target) if !target.trim().is_empty() => vec![target],
            _ => self
                .endpoints
                .iter()
                .map(|endpoint| endpoint.as_str())
                .collect(),
        };

        for endpoint in target_endpoints {
            if endpoint.trim().is_empty() {
                continue;
            }

            let ticket = if let Some(pool) = &self.uuid_pool {
                let mut guard = pool.lock().await;
                let ticket = guard.next_ticket();
                drop(guard);
                if let Some(ticket) = &ticket {
                    if ticket.forced {
                        warn!(
                            target: "lander::jito",
                            uuid = ticket.uuid.as_str(),
                            rate_limit = ?ticket.rate_limit,
                            "uuid rate limit exhausted, forcing bundle submission"
                        );
                    }
                }
                ticket
            } else {
                None
            };

            let endpoint_url = match prepare_endpoint_url(endpoint, ticket.as_ref()) {
                Some(url) => url,
                None => {
                    warn!(
                        target: "lander::jito",
                        endpoint,
                        "failed to parse endpoint url"
                    );
                    continue;
                }
            };

            let (request_id, options_value) = match &ticket {
                Some(ticket) => (ticket.bundle_id.clone(), Some(ticket.options_value())),
                None => (DEFAULT_REQUEST_ID.to_string(), None),
            };

            let mut params = vec![bundle_value.clone()];
            if let Some(options) = &options_value {
                params.push(options.clone());
            }

            let payload = json!({
                "jsonrpc": JSONRPC_VERSION,
                "id": request_id,
                "method": "sendBundle",
                "params": params,
            });

            let response = self
                .client
                .post(endpoint_url.clone())
                .json(&payload)
                .send()
                .await
                .map_err(LanderError::Network)?;

            if !response.status().is_success() {
                warn!(
                    target: "lander::jito",
                    endpoint = endpoint_url.as_str(),
                    status = %response.status(),
                    "bundle submission returned non-success status"
                );
                continue;
            }

            let value: serde_json::Value = response.json().await.map_err(LanderError::Network)?;
            if let Some(error) = value.get("error") {
                warn!(
                    target: "lander::jito",
                    endpoint = endpoint_url.as_str(),
                    error = %error,
                    "bundle submission returned error"
                );
                continue;
            }

            let bundle_id = value
                .get("result")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| ticket.as_ref().map(|t| t.bundle_id.clone()));

            let endpoint_string = endpoint_url.to_string();
            return Ok(LanderReceipt {
                lander: "jito",
                endpoint: endpoint_string,
                slot: variant.slot(),
                blockhash: variant.blockhash().to_string(),
                signature: bundle_id,
                variant_id: variant.id(),
            });
        }

        Err(LanderError::fatal("all jito endpoints failed submission"))
    }
}

fn encode_transaction<T: serde::Serialize>(tx: &T) -> Result<String, LanderError> {
    let bytes = encode_to_vec(tx, standard())?;
    Ok(bs58::encode(bytes).into_string())
}

fn random_tip_wallet() -> Option<Pubkey> {
    if TIP_WALLETS.is_empty() {
        return None;
    }
    let mut rng = rand::rng();
    TIP_WALLETS.as_slice().choose(&mut rng).copied()
}

fn tip_wallet_for_variant(variant_id: VariantId) -> Option<Pubkey> {
    if TIP_WALLETS.is_empty() {
        return None;
    }
    let index = (variant_id as usize) % TIP_WALLETS.len();
    TIP_WALLETS.get(index).copied()
}

fn prepare_endpoint_url(endpoint: &str, ticket: Option<&UuidTicket>) -> Option<Url> {
    let trimmed = endpoint.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut url = Url::parse(trimmed).ok()?;
    if let Some(ticket) = ticket {
        url.query_pairs_mut()
            .append_pair("uuid", ticket.uuid.as_str());
    }
    Some(url)
}

fn build_tip_transaction(variant: &TxVariant, recipient: Pubkey, lamports: u64) -> Transaction {
    let signer = variant.signer();
    let payer = signer.pubkey();
    let instruction = system_instruction::transfer(&payer, &recipient, lamports);
    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&payer));
    transaction.sign(&[signer.as_ref()], variant.blockhash());
    transaction
}

#[derive(Clone)]
struct TipSelector {
    strategies: Vec<TipStrategyKind>,
    fixed_tip: Option<u64>,
    range_tips: Vec<u64>,
    floor_level: Option<TipFloorLevel>,
    max_floor_tip: Option<u64>,
    floor_fetcher: Option<Arc<TipFloorFetcher>>,
}

impl TipSelector {
    fn from_config(config: &LanderJitoConfig) -> Self {
        let strategies = if config.tip_strategies.is_empty() {
            vec![TipStrategyKind::Fixed]
        } else {
            config.tip_strategies.clone()
        };

        let range_tips = config
            .range_tips
            .iter()
            .copied()
            .filter(|value| *value > 0)
            .collect();

        let floor_fetcher = if strategies
            .iter()
            .any(|strategy| matches!(strategy, TipStrategyKind::Floor))
        {
            Some(Arc::new(TipFloorFetcher::new(TIP_FLOOR_CACHE_TTL)))
        } else {
            None
        };

        Self {
            strategies,
            fixed_tip: config.fixed_tip.filter(|value| *value > 0),
            range_tips,
            floor_level: config.floor_tip_level,
            max_floor_tip: config.max_floor_tip_lamports,
            floor_fetcher,
        }
    }

    async fn select_tip(&self, client: &Client) -> Result<Option<u64>, LanderError> {
        for strategy in &self.strategies {
            match strategy {
                TipStrategyKind::Fixed => {
                    if let Some(value) = self.fixed_tip {
                        if value > 0 {
                            return Ok(Some(value));
                        }
                    }
                }
                TipStrategyKind::Range => {
                    if let Some(value) = self.pick_range_tip() {
                        if value > 0 {
                            return Ok(Some(value));
                        }
                    }
                }
                TipStrategyKind::Floor => {
                    let level = match self.floor_level {
                        Some(level) => level,
                        None => {
                            warn!(
                                target: "lander::jito",
                                "tip strategy floor configured but floor_tip_level missing"
                            );
                            continue;
                        }
                    };

                    if let Some(fetcher) = &self.floor_fetcher {
                        if let Some(value) =
                            fetcher.fetch(client, level, self.max_floor_tip).await?
                        {
                            if value > 0 {
                                return Ok(Some(value));
                            }
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn pick_range_tip(&self) -> Option<u64> {
        if self.range_tips.is_empty() {
            return None;
        }
        let mut rng = rand::rng();
        self.range_tips.as_slice().choose(&mut rng).copied()
    }
}

#[derive(Clone)]
struct TipFloorFetcher {
    cache: Arc<Mutex<Option<CachedFloors>>>,
    ttl: Duration,
}

impl TipFloorFetcher {
    fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(Mutex::new(None)),
            ttl,
        }
    }

    async fn fetch(
        &self,
        client: &Client,
        level: TipFloorLevel,
        max_tip: Option<u64>,
    ) -> Result<Option<u64>, LanderError> {
        if let Some(value) = {
            let guard = self.cache.lock().await;
            guard.as_ref().and_then(|cached| {
                if cached.fetched_at.elapsed() <= self.ttl {
                    cached.values.value(level, max_tip)
                } else {
                    None
                }
            })
        } {
            return Ok(Some(value));
        }

        let values = Self::download(client).await?;
        let result = values.value(level, max_tip);

        let mut guard = self.cache.lock().await;
        *guard = Some(CachedFloors {
            fetched_at: Instant::now(),
            values,
        });

        Ok(result)
    }

    async fn download(client: &Client) -> Result<TipFloorValues, LanderError> {
        let response = client
            .get(TIP_FLOOR_URL)
            .send()
            .await
            .map_err(LanderError::Network)?;
        if !response.status().is_success() {
            return Err(LanderError::fatal(format!(
                "tip floor endpoint returned non-success status: {}",
                response.status()
            )));
        }

        let mut payload: Vec<TipFloorApiEntry> =
            response.json().await.map_err(LanderError::Network)?;
        let entry = payload
            .drain(..)
            .next()
            .ok_or_else(|| LanderError::fatal("tip floor response empty"))?;
        Ok(TipFloorValues::from_entry(entry))
    }
}

#[derive(Clone)]
struct CachedFloors {
    fetched_at: Instant,
    values: TipFloorValues,
}

#[derive(Clone)]
struct TipFloorValues {
    lamports: HashMap<TipFloorLevel, u64>,
}

impl TipFloorValues {
    fn from_entry(entry: TipFloorApiEntry) -> Self {
        let mut lamports = HashMap::new();
        if let Some(value) = entry.landed_tips_25th_percentile.and_then(sol_to_lamports) {
            lamports.insert(TipFloorLevel::Percentile25, value);
        }
        if let Some(value) = entry.landed_tips_50th_percentile.and_then(sol_to_lamports) {
            lamports.insert(TipFloorLevel::Percentile50, value);
        }
        if let Some(value) = entry.landed_tips_75th_percentile.and_then(sol_to_lamports) {
            lamports.insert(TipFloorLevel::Percentile75, value);
        }
        if let Some(value) = entry.landed_tips_95th_percentile.and_then(sol_to_lamports) {
            lamports.insert(TipFloorLevel::Percentile95, value);
        }
        if let Some(value) = entry.landed_tips_99th_percentile.and_then(sol_to_lamports) {
            lamports.insert(TipFloorLevel::Percentile99, value);
        }
        if let Some(value) = entry
            .ema_landed_tips_50th_percentile
            .and_then(sol_to_lamports)
        {
            lamports.insert(TipFloorLevel::Ema50, value);
        }

        Self { lamports }
    }

    fn value(&self, level: TipFloorLevel, cap: Option<u64>) -> Option<u64> {
        self.lamports
            .get(&level)
            .copied()
            .map(|value| cap.map_or(value, |cap| value.min(cap)))
    }
}

#[derive(Debug, Deserialize)]
struct TipFloorApiEntry {
    #[serde(rename = "landed_tips_25th_percentile")]
    landed_tips_25th_percentile: Option<f64>,
    #[serde(rename = "landed_tips_50th_percentile")]
    landed_tips_50th_percentile: Option<f64>,
    #[serde(rename = "landed_tips_75th_percentile")]
    landed_tips_75th_percentile: Option<f64>,
    #[serde(rename = "landed_tips_95th_percentile")]
    landed_tips_95th_percentile: Option<f64>,
    #[serde(rename = "landed_tips_99th_percentile")]
    landed_tips_99th_percentile: Option<f64>,
    #[serde(rename = "ema_landed_tips_50th_percentile")]
    ema_landed_tips_50th_percentile: Option<f64>,
}

fn sol_to_lamports(value: f64) -> Option<u64> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }
    let lamports = (value * 1_000_000_000.0).round();
    if lamports <= 0.0 {
        None
    } else if lamports >= u64::MAX as f64 {
        Some(u64::MAX)
    } else {
        Some(lamports as u64)
    }
}

#[derive(Clone)]
struct UuidPool {
    entries: Vec<UuidEntry>,
    cursor: usize,
}

impl UuidPool {
    fn new(configs: &[LanderJitoUuidConfig]) -> Option<Self> {
        let mut entries = Vec::new();
        for cfg in configs {
            let uuid = cfg.uuid.trim();
            if uuid.is_empty() {
                continue;
            }
            let limiter = cfg.rate_limit.and_then(|limit| {
                if limit == 0 {
                    None
                } else {
                    Some(RateLimiter::new(limit))
                }
            });
            entries.push(UuidEntry {
                uuid: uuid.to_string(),
                limiter,
                sequence: 0,
            });
        }

        if entries.is_empty() {
            None
        } else {
            Some(Self { entries, cursor: 0 })
        }
    }

    fn next_ticket(&mut self) -> Option<UuidTicket> {
        if self.entries.is_empty() {
            return None;
        }

        let len = self.entries.len();
        for _ in 0..len {
            let entry = &mut self.entries[self.cursor];
            self.cursor = (self.cursor + 1) % len;
            if let Some(ticket) = entry.try_next() {
                return Some(ticket);
            }
        }

        let entry = &mut self.entries[self.cursor];
        self.cursor = (self.cursor + 1) % len;
        Some(entry.force_next())
    }
}

#[derive(Clone)]
struct UuidEntry {
    uuid: String,
    limiter: Option<RateLimiter>,
    sequence: u64,
}

impl UuidEntry {
    fn try_next(&mut self) -> Option<UuidTicket> {
        if let Some(limiter) = &mut self.limiter {
            if !limiter.try_acquire() {
                return None;
            }
        }

        self.sequence = self.sequence.wrapping_add(1);
        Some(UuidTicket {
            uuid: self.uuid.clone(),
            bundle_id: format!("{}-{}", self.uuid, self.sequence),
            forced: false,
            rate_limit: self.limiter.as_ref().map(|limiter| limiter.capacity),
        })
    }

    fn force_next(&mut self) -> UuidTicket {
        if let Some(limiter) = &mut self.limiter {
            limiter.force();
        }
        self.sequence = self.sequence.wrapping_add(1);
        UuidTicket {
            uuid: self.uuid.clone(),
            bundle_id: format!("{}-{}", self.uuid, self.sequence),
            forced: true,
            rate_limit: self.limiter.as_ref().map(|limiter| limiter.capacity),
        }
    }
}

#[derive(Clone)]
struct RateLimiter {
    capacity: u64,
    rate: f64,
    tokens: f64,
    last: Instant,
}

impl RateLimiter {
    fn new(limit: u64) -> Self {
        let now = Instant::now();
        Self {
            capacity: limit,
            rate: limit as f64,
            tokens: limit as f64,
            last: now,
        }
    }

    fn try_acquire(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn force(&mut self) {
        self.refill();
        self.tokens = 0.0;
        self.last = Instant::now();
    }

    fn refill(&mut self) {
        let elapsed = self.last.elapsed().as_secs_f64();
        if elapsed <= 0.0 {
            return;
        }
        self.tokens = (self.tokens + elapsed * self.rate).min(self.capacity as f64);
        self.last = Instant::now();
    }
}

#[derive(Clone)]
struct UuidTicket {
    uuid: String,
    bundle_id: String,
    forced: bool,
    rate_limit: Option<u64>,
}

impl UuidTicket {
    fn options_value(&self) -> Value {
        json!({
            "bundleId": self.bundle_id,
            "uuid": self.uuid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;

    #[tokio::test]
    async fn tip_selector_prefers_fixed_strategy() {
        let mut config = LanderJitoConfig::default();
        config.fixed_tip = Some(3_000);
        config.tip_strategies = vec![TipStrategyKind::Fixed];
        let selector = TipSelector::from_config(&config);

        let tip = selector
            .select_tip(&Client::new())
            .await
            .expect("selector returns result");
        assert_eq!(tip, Some(3_000));
    }

    #[tokio::test]
    async fn tip_selector_handles_range_strategy() {
        let mut config = LanderJitoConfig::default();
        config.range_tips = vec![1_500];
        config.tip_strategies = vec![TipStrategyKind::Range];
        let selector = TipSelector::from_config(&config);

        let tip = selector
            .select_tip(&Client::new())
            .await
            .expect("selector returns result");
        assert_eq!(tip, Some(1_500));
    }

    #[test]
    fn uuid_pool_enforces_rate_limit() {
        let configs = vec![LanderJitoUuidConfig {
            uuid: "uuid-1".to_string(),
            rate_limit: Some(1),
        }];
        let mut pool = UuidPool::new(&configs).expect("pool created");

        let first = pool.next_ticket().expect("first ticket");
        assert!(!first.forced);

        let second = pool.next_ticket().expect("second ticket");
        assert!(second.forced);
        assert_eq!(second.uuid, "uuid-1");
    }

    #[test]
    fn converts_sol_to_lamports() {
        assert_eq!(sol_to_lamports(0.0), None);
        assert_eq!(sol_to_lamports(-0.0001), None);
        assert_eq!(sol_to_lamports(0.000000001), Some(1));
        assert_eq!(sol_to_lamports(0.000005), Some(5_000));
    }

    #[test]
    fn prepare_endpoint_appends_uuid() {
        let ticket = UuidTicket {
            uuid: "uuid-123".to_string(),
            bundle_id: "uuid-123-1".to_string(),
            forced: false,
            rate_limit: Some(5),
        };
        let url = prepare_endpoint_url("https://example.com/api/v1", Some(&ticket))
            .expect("endpoint url");
        let query: Vec<(String, String)> = url.query_pairs().into_owned().collect();
        assert!(query.contains(&("uuid".to_string(), "uuid-123".to_string())));
    }

    #[test]
    fn prepare_endpoint_preserves_existing_query() {
        let ticket = UuidTicket {
            uuid: "uuid-xyz".to_string(),
            bundle_id: "uuid-xyz-7".to_string(),
            forced: false,
            rate_limit: Some(5),
        };
        let url = prepare_endpoint_url("https://example.com/api/v1?region=ny", Some(&ticket))
            .expect("endpoint url");
        let mut query: Vec<(String, String)> = url.query_pairs().into_owned().collect();
        query.sort();
        assert_eq!(
            query,
            vec![
                ("region".to_string(), "ny".to_string()),
                ("uuid".to_string(), "uuid-xyz".to_string())
            ]
        );
    }
}
