use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use futures::future::{AbortHandle, Abortable, Aborted};
use futures::stream::FuturesUnordered;
use futures::{SinkExt, StreamExt};

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use bincode::config::standard;
use bincode::serde::encode_to_vec;
use once_cell::sync::Lazy;
use rand::seq::IndexedRandom;
use reqwest::Client;
use serde::Deserialize;
use serde_json::{Value, json};
use solana_sdk::instruction::Instruction;
use solana_sdk::message::VersionedMessage;
use solana_sdk::message::v0::Message as V0Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::VersionedTransaction;
use solana_system_interface::instruction as system_instruction;
use tokio::runtime::Handle;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{debug, warn};
use url::Url;

use crate::config::{LanderJitoConfig, LanderJitoUuidConfig, TipStrategyKind, TipStreamLevel};
use crate::engine::{TxVariant, VariantId};

use super::error::LanderError;
use super::stack::{Deadline, LanderReceipt};

const JSONRPC_VERSION: &str = "2.0";
const TIP_STREAM_URL: &str = "wss://bundles.jito.wtf/api/v1/bundles/tip_stream";
const TIP_STREAM_RECONNECT_DELAY: Duration = Duration::from_secs(1);
const COMPUTE_BUDGET_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ComputeBudget111111111111111111111111111111");

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

        let configured_tip = self.tip_selector.select_tip();
        let base_tip = (variant.tip_lamports() > 0).then(|| variant.tip_lamports());
        let tip_lamports = variant
            .tip_override()
            .map(|override_tip| override_tip.lamports())
            .or(configured_tip)
            .or(base_tip)
            .unwrap_or(0);

        let tip_recipient = if tip_lamports > 0 {
            variant
                .tip_override()
                .and_then(|override_tip| override_tip.recipient())
                .or_else(|| tip_wallet_for_variant(variant.id()))
                .or_else(random_tip_wallet)
        } else {
            None
        };

        let final_tx = match (tip_lamports, tip_recipient) {
            (lamports, Some(recipient)) if lamports > 0 => {
                match build_jito_transaction(&variant, Some((recipient, lamports))) {
                    Ok(tx) => {
                        debug!(
                            target: "lander::jito",
                            tip_lamports = lamports,
                            recipient = %recipient,
                            "tip 指令已合并进主交易"
                        );
                        tx
                    }
                    Err(err) => {
                        warn!(
                            target: "lander::jito",
                            tip_lamports = lamports,
                            recipient = %recipient,
                            error = %err,
                            "构建包含 tip 的交易失败，回退为无 tip 交易"
                        );
                        build_jito_transaction(&variant, None).unwrap_or_else(|fallback_err| {
                            warn!(
                                target: "lander::jito",
                                error = %fallback_err,
                                "重构无 tip 交易失败，使用原始交易"
                            );
                            variant.transaction().clone()
                        })
                    }
                }
            }
            (lamports, None) if lamports > 0 => {
                warn!(
                    target: "lander::jito",
                    tip_lamports = lamports,
                    "tip wallet list empty, skipping tip transaction"
                );
                build_jito_transaction(&variant, None).unwrap_or_else(|err| {
                    warn!(
                        target: "lander::jito",
                        error = %err,
                        "重构无 tip 交易失败，使用原始交易"
                    );
                    variant.transaction().clone()
                })
            }
            _ => build_jito_transaction(&variant, None).unwrap_or_else(|err| {
                warn!(
                    target: "lander::jito",
                    error = %err,
                    "重构无 tip 交易失败，使用原始交易"
                );
                variant.transaction().clone()
            }),
        };

        let main_encoded = encode_transaction(&final_tx)?;
        let bundle_value = Value::Array(vec![Value::String(main_encoded)]);

        let targets: Vec<String> = match endpoint {
            Some(target) => {
                let trimmed = target.trim();
                if trimmed.is_empty() {
                    Vec::new()
                } else {
                    vec![trimmed.to_string()]
                }
            }
            None => self.endpoint_list(),
        };

        if targets.is_empty() {
            return Err(LanderError::fatal("no jito endpoints available"));
        }

        let tickets = if let Some(pool) = &self.uuid_pool {
            let mut guard = pool.lock().await;
            targets.iter().map(|_| guard.next_ticket()).collect()
        } else {
            vec![None; targets.len()]
        };

        let mut requests = Vec::new();
        for (endpoint, ticket) in targets.into_iter().zip(tickets.into_iter()) {
            if endpoint.trim().is_empty() {
                continue;
            }

            if let Some(ticket) = ticket.as_ref() {
                if ticket.forced {
                    warn!(
                        target: "lander::jito",
                        uuid = ticket.uuid.as_str(),
                        rate_limit = ?ticket.rate_limit,
                        "uuid rate limit exhausted, forcing bundle submission"
                    );
                }
            }

            let endpoint_url = match prepare_endpoint_url(&endpoint, ticket.as_ref()) {
                Some(url) => url,
                None => {
                    warn!(
                        target: "lander::jito",
                        endpoint = endpoint.as_str(),
                        "failed to parse endpoint url"
                    );
                    continue;
                }
            };

            let options_value = ticket.as_ref().map(|t| t.options_value());

            requests.push((endpoint_url, ticket, options_value));
        }

        if requests.is_empty() {
            return Err(LanderError::fatal("no valid jito endpoints configured"));
        }

        let slot = variant.slot();
        let blockhash = variant.blockhash().to_string();
        let variant_id = variant.id();

        let mut futures = FuturesUnordered::new();
        for (endpoint_url, ticket, options_value) in requests {
            let mut params = vec![bundle_value.clone()];

            let mut options = options_value.unwrap_or_else(|| json!({}));
            if !options.is_object() {
                options = json!({});
            }
            if let Value::Object(ref mut map) = options {
                map.insert("encoding".to_string(), Value::String("base64".to_string()));
            }
            params.push(options);

            let payload = json!({
                "jsonrpc": JSONRPC_VERSION,
                "id": 1,
                "method": "sendBundle",
                "params": params,
            });

            let client = self.client.clone();
            futures.push(async move {
                let response = client
                    .post(endpoint_url.clone())
                    .json(&payload)
                    .send()
                    .await
                    .map_err(LanderError::Network);

                (endpoint_url, ticket, response)
            });
        }

        while let Some((endpoint_url, ticket, response_result)) = futures.next().await {
            let response = match response_result {
                Ok(resp) => resp,
                Err(err) => {
                    warn!(
                        target: "lander::jito",
                        endpoint = endpoint_url.as_str(),
                        error = %err,
                        "bundle submission network error"
                    );
                    continue;
                }
            };

            if !response.status().is_success() {
                warn!(
                    target: "lander::jito",
                    endpoint = endpoint_url.as_str(),
                    status = %response.status(),
                    "bundle submission returned non-success status"
                );
                continue;
            }

            let value: serde_json::Value = match response.json().await.map_err(LanderError::Network)
            {
                Ok(val) => val,
                Err(err) => {
                    warn!(
                        target: "lander::jito",
                        endpoint = endpoint_url.as_str(),
                        error = %err,
                        "bundle submission decode error"
                    );
                    continue;
                }
            };
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

            return Ok(LanderReceipt {
                lander: "jito",
                endpoint: endpoint_url.to_string(),
                slot,
                blockhash: blockhash.clone(),
                signature: bundle_id,
                variant_id,
            });
        }

        Err(LanderError::fatal("all jito endpoints failed submission"))
    }
}

fn encode_transaction<T: serde::Serialize>(tx: &T) -> Result<String, LanderError> {
    let bytes = encode_to_vec(tx, standard())?;
    Ok(BASE64_STANDARD.encode(bytes))
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

fn build_jito_transaction(
    variant: &TxVariant,
    tip: Option<(Pubkey, u64)>,
) -> Result<VersionedTransaction, LanderError> {
    let signer = variant.signer();
    let payer = signer.pubkey();
    let mut instructions = strip_compute_unit_price(variant.instructions().to_vec());
    if let Some((recipient, lamports)) = tip {
        instructions.push(system_instruction::transfer(&payer, &recipient, lamports));
    }

    let message = V0Message::try_compile(
        &payer,
        &instructions,
        variant.lookup_accounts(),
        variant.blockhash(),
    )
    .map_err(|err| LanderError::fatal(format!("构建 Jito 交易消息失败: {err:#}")))?;
    let versioned = VersionedMessage::V0(message);
    VersionedTransaction::try_new(versioned, &[signer.as_ref()])
        .map_err(|err| LanderError::fatal(format!("签名 Jito 交易失败: {err:#}")))
}

fn strip_compute_unit_price(mut instructions: Vec<Instruction>) -> Vec<Instruction> {
    instructions
        .retain(|ix| !(ix.program_id == COMPUTE_BUDGET_PROGRAM_ID && ix.data.first() == Some(&3)));
    instructions
}

const MIN_JITO_TIP_LAMPORTS: u64 = 1_000;

#[derive(Clone)]
struct TipSelector {
    strategy: TipStrategyKind,
    fixed_tip: u64,
    range_tips: Vec<u64>,
    stream: Option<TipStream>,
}

impl TipSelector {
    fn from_config(config: &LanderJitoConfig) -> Self {
        let strategy = config.tip_strategy;

        let range_tips = config
            .range_tips
            .iter()
            .copied()
            .filter(|value| *value > 0)
            .collect();

        let fixed_tip = match strategy {
            TipStrategyKind::Fixed | TipStrategyKind::Range => config
                .fixed_tip
                .filter(|value| *value > 0)
                .unwrap_or(MIN_JITO_TIP_LAMPORTS),
            TipStrategyKind::Stream => MIN_JITO_TIP_LAMPORTS,
        };

        let stream = if matches!(strategy, TipStrategyKind::Stream) {
            let level = config
                .stream_tip_level
                .unwrap_or(TipStreamLevel::Percentile50);
            let cap = config.max_stream_tip_lamports.filter(|value| *value > 0);
            Some(TipStream::spawn(level, cap, Some(MIN_JITO_TIP_LAMPORTS)))
        } else {
            None
        };

        Self {
            strategy,
            fixed_tip,
            range_tips,
            stream,
        }
    }

    fn select_tip(&self) -> Option<u64> {
        match self.strategy {
            TipStrategyKind::Fixed => Some(self.fixed_tip),
            TipStrategyKind::Range => self.pick_range_tip().or_else(|| {
                warn!(
                    target: "lander::jito",
                    "range 策略未配置有效的随机列表，退回 {} lamports",
                    self.fixed_tip
                );
                Some(self.fixed_tip)
            }),
            TipStrategyKind::Stream => {
                if let Some(stream) = &self.stream {
                    if let Some(value) = stream.latest() {
                        return Some(value);
                    }
                }
                warn!(
                    target: "lander::jito",
                    "tip stream 未取到有效数值，回退为 {} lamports",
                    MIN_JITO_TIP_LAMPORTS
                );
                Some(MIN_JITO_TIP_LAMPORTS)
            }
        }
    }

    fn pick_range_tip(&self) -> Option<u64> {
        if self.range_tips.is_empty() {
            return None;
        }
        let mut rng = rand::rng();
        self.range_tips.as_slice().choose(&mut rng).copied()
    }
}

struct TipStream {
    shared: Arc<TipStreamShared>,
    task: Arc<TipStreamTask>,
}

impl TipStream {
    fn spawn(level: TipStreamLevel, max_tip: Option<u64>, fallback: Option<u64>) -> Self {
        let shared = Arc::new(TipStreamShared::new(level, max_tip, fallback));
        let task = if let Ok(handle) = Handle::try_current() {
            let shared_clone = shared.clone();
            let (abort_handle, abort_registration) = AbortHandle::new_pair();

            let future = async move {
                TipStreamShared::run(shared_clone).await;
            };

            let abortable = Abortable::new(future, abort_registration);
            handle.spawn(async move {
                if let Err(Aborted) = abortable.await {
                    debug!(target: "lander::jito", "tip stream 任务被显式中止");
                }
            });

            TipStreamTask {
                abort: Some(abort_handle),
            }
        } else {
            warn!(
                target: "lander::jito",
                "tip stream 未检测到 Tokio runtime，WebSocket 功能未启动"
            );
            TipStreamTask { abort: None }
        };

        Self {
            shared,
            task: Arc::new(task),
        }
    }

    fn latest(&self) -> Option<u64> {
        self.shared.latest()
    }
}

impl Clone for TipStream {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
            task: self.task.clone(),
        }
    }
}

impl Drop for TipStream {
    fn drop(&mut self) {
        if Arc::strong_count(&self.task) == 1 {
            self.task.abort();
        }
    }
}

struct TipStreamTask {
    abort: Option<AbortHandle>,
}

impl TipStreamTask {
    fn abort(&self) {
        if let Some(handle) = &self.abort {
            handle.abort();
        }
    }
}

struct TipStreamShared {
    latest: AtomicU64,
    level: TipStreamLevel,
    max_tip: Option<u64>,
    fallback: Option<u64>,
}

impl TipStreamShared {
    fn new(level: TipStreamLevel, max_tip: Option<u64>, fallback: Option<u64>) -> Self {
        let initial = fallback.unwrap_or(0);
        Self {
            latest: AtomicU64::new(initial),
            level,
            max_tip,
            fallback,
        }
    }

    fn latest(&self) -> Option<u64> {
        let value = self.latest.load(Ordering::Relaxed);
        if value > 0 { Some(value) } else { None }
    }

    async fn run(self: Arc<Self>) {
        loop {
            match connect_async(TIP_STREAM_URL).await {
                Ok((mut ws, _response)) => {
                    debug!(target: "lander::jito", "tip stream 已连接");
                    while let Some(message) = ws.next().await {
                        match message {
                            Ok(Message::Text(text)) => {
                                if let Err(err) = self.apply_payload(&text) {
                                    warn!(
                                        target: "lander::jito",
                                        error = %err,
                                        "解析 tip stream 文本消息失败"
                                    );
                                }
                            }
                            Ok(Message::Binary(binary)) => match std::str::from_utf8(&binary) {
                                Ok(text) => {
                                    if let Err(err) = self.apply_payload(text) {
                                        warn!(
                                            target: "lander::jito",
                                            error = %err,
                                            "解析 tip stream 二进制消息失败"
                                        );
                                    }
                                }
                                Err(err) => {
                                    warn!(
                                        target: "lander::jito",
                                        error = %err,
                                        "tip stream 二进制消息无法转为 utf8"
                                    );
                                }
                            },
                            Ok(Message::Ping(payload)) => {
                                if let Err(err) = ws.send(Message::Pong(payload)).await {
                                    warn!(
                                        target: "lander::jito",
                                        error = %err,
                                        "tip stream 响应 pong 失败"
                                    );
                                    break;
                                }
                            }
                            Ok(Message::Pong(_)) => {}
                            Ok(Message::Frame(_)) => {}
                            Ok(Message::Close(frame)) => {
                                debug!(
                                    target: "lander::jito",
                                    ?frame,
                                    "tip stream 收到 close 帧"
                                );
                                break;
                            }
                            Err(err) => {
                                warn!(
                                    target: "lander::jito",
                                    error = %err,
                                    "tip stream 读取消息失败"
                                );
                                break;
                            }
                        }
                    }
                }
                Err(err) => {
                    warn!(
                        target: "lander::jito",
                        error = %err,
                        "tip stream 连接失败，将在 {:?} 后重试",
                        TIP_STREAM_RECONNECT_DELAY
                    );
                }
            }

            if let Some(fallback) = self.fallback {
                if fallback > 0 && self.latest.load(Ordering::Relaxed) == 0 {
                    self.latest.store(fallback, Ordering::Relaxed);
                }
            }

            sleep(TIP_STREAM_RECONNECT_DELAY).await;
        }
    }

    fn apply_payload(&self, payload: &str) -> Result<(), serde_json::Error> {
        let envelope: TipStreamEnvelope = serde_json::from_str(payload)?;
        let entry = match envelope {
            TipStreamEnvelope::Single(entry) => entry,
            TipStreamEnvelope::Array(mut entries) => entries.pop().unwrap_or_default(),
        };
        let values = TipStreamValues::from_entry(entry);
        if let Some(value) = values.value(self.level, self.max_tip) {
            self.latest.store(value, Ordering::Relaxed);
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum TipStreamEnvelope {
    Single(TipStreamEntry),
    Array(Vec<TipStreamEntry>),
}

#[derive(Debug, Deserialize, Default, Clone)]
struct TipStreamEntry {
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

#[derive(Clone)]
struct TipStreamValues {
    lamports: HashMap<TipStreamLevel, u64>,
}

impl TipStreamValues {
    fn from_entry(entry: TipStreamEntry) -> Self {
        let mut lamports = HashMap::new();
        if let Some(value) = entry.landed_tips_25th_percentile.and_then(sol_to_lamports) {
            lamports.insert(TipStreamLevel::Percentile25, value);
        }
        if let Some(value) = entry.landed_tips_50th_percentile.and_then(sol_to_lamports) {
            lamports.insert(TipStreamLevel::Percentile50, value);
        }
        if let Some(value) = entry.landed_tips_75th_percentile.and_then(sol_to_lamports) {
            lamports.insert(TipStreamLevel::Percentile75, value);
        }
        if let Some(value) = entry.landed_tips_95th_percentile.and_then(sol_to_lamports) {
            lamports.insert(TipStreamLevel::Percentile95, value);
        }
        if let Some(value) = entry.landed_tips_99th_percentile.and_then(sol_to_lamports) {
            lamports.insert(TipStreamLevel::Percentile99, value);
        }
        if let Some(value) = entry
            .ema_landed_tips_50th_percentile
            .and_then(sol_to_lamports)
        {
            lamports.insert(TipStreamLevel::Ema50, value);
        }

        Self { lamports }
    }

    fn value(&self, level: TipStreamLevel, cap: Option<u64>) -> Option<u64> {
        self.lamports
            .get(&level)
            .copied()
            .map(|value| cap.map_or(value, |cap| value.min(cap)))
    }
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

    #[test]
    fn tip_selector_prefers_fixed_strategy() {
        let mut config = LanderJitoConfig::default();
        config.fixed_tip = Some(3_000);
        config.tip_strategy = TipStrategyKind::Fixed;
        let selector = TipSelector::from_config(&config);

        let tip = selector.select_tip();
        assert_eq!(tip, Some(3_000));
    }

    #[test]
    fn tip_selector_handles_range_strategy() {
        let mut config = LanderJitoConfig::default();
        config.range_tips = vec![1_500];
        config.tip_strategy = TipStrategyKind::Range;
        let selector = TipSelector::from_config(&config);

        let tip = selector.select_tip();
        assert_eq!(tip, Some(1_500));
    }

    #[test]
    fn tip_selector_stream_fallbacks_to_min_tip() {
        let mut config = LanderJitoConfig::default();
        config.fixed_tip = Some(10_000); // should be ignored for stream fallback
        config.tip_strategy = TipStrategyKind::Stream;
        let selector = TipSelector::from_config(&config);

        let tip = selector.select_tip();
        assert_eq!(tip, Some(MIN_JITO_TIP_LAMPORTS));
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
