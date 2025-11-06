use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use futures::future::{AbortHandle, Abortable, Aborted};
use futures::{SinkExt, StreamExt};
use rand::seq::IndexedRandom;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use tokio::runtime::Handle;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use tracing::{debug, info, warn};

use crate::config::{LanderJitoConfig, TipStrategyKind, TipStreamLevel};

pub(crate) const TIP_STREAM_URL: &str = "wss://bundles.jito.wtf/api/v1/bundles/tip_stream";
pub(crate) const TIP_STREAM_RECONNECT_DELAY: Duration = Duration::from_secs(1);
pub(crate) const MIN_JITO_TIP_LAMPORTS: u64 = 1_000;

#[derive(Clone)]
pub(crate) struct TipSelector {
    strategy: TipStrategyKind,
    base_tip: u64,
    range_tips: Vec<u64>,
    stream: Option<TipStream>,
    api: Option<TipApi>,
}

impl TipSelector {
    pub fn from_config(config: &LanderJitoConfig) -> Self {
        let strategy = config.tip_strategy;

        let range_tips = config
            .range_tips
            .iter()
            .copied()
            .filter(|value| *value > 0)
            .collect();

        let base_tip = match strategy {
            TipStrategyKind::Fixed | TipStrategyKind::Range => config
                .fixed_tip
                .filter(|value| *value > 0)
                .unwrap_or(MIN_JITO_TIP_LAMPORTS),
            TipStrategyKind::Stream | TipStrategyKind::Api => config
                .fixed_tip
                .filter(|value| *value > 0)
                .unwrap_or(MIN_JITO_TIP_LAMPORTS),
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

        let api = if matches!(strategy, TipStrategyKind::Api) {
            let url = config
                .tip_floor_api
                .as_ref()
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .map(|value| value.to_string());
            match url {
                Some(endpoint) => {
                    let refresh_ms = config.tip_floor_refresh_ms.unwrap_or(5_000);
                    Some(TipApi::spawn(
                        endpoint,
                        Duration::from_millis(refresh_ms.max(200)),
                        MIN_JITO_TIP_LAMPORTS,
                    ))
                }
                None => {
                    warn!(
                        target: "lander::jito::tip",
                        "tip_strategy=api 但未配置 tip_floor_api，回退为 {} lamports",
                        MIN_JITO_TIP_LAMPORTS
                    );
                    None
                }
            }
        } else {
            None
        };

        Self {
            strategy,
            base_tip,
            range_tips,
            stream,
            api,
        }
    }

    pub fn strategy_kind(&self) -> TipStrategyKind {
        self.strategy
    }

    pub fn select_tip(&self) -> Option<u64> {
        match self.strategy {
            TipStrategyKind::Fixed => Some(self.base_tip),
            TipStrategyKind::Range => self.pick_range_tip().or_else(|| {
                warn!(
                    target: "lander::jito::tip",
                    "range 策略未配置有效的随机列表，退回 {} lamports",
                    self.base_tip
                );
                Some(self.base_tip)
            }),
            TipStrategyKind::Stream => {
                if let Some(stream) = &self.stream {
                    if let Some(value) = stream.latest() {
                        return Some(value.max(MIN_JITO_TIP_LAMPORTS));
                    }
                }
                warn!(
                    target: "lander::jito::tip",
                    "tip stream 未取到有效数值，回退为 {} lamports",
                    MIN_JITO_TIP_LAMPORTS
                );
                Some(MIN_JITO_TIP_LAMPORTS)
            }
            TipStrategyKind::Api => {
                if let Some(api) = &self.api {
                    if let Some(value) = api.latest() {
                        return Some(value.max(MIN_JITO_TIP_LAMPORTS));
                    }
                    warn!(
                        target: "lander::jito::tip",
                        "tip api 尚未产出有效值，沿用最后一次拉取或回退为 {} lamports",
                        MIN_JITO_TIP_LAMPORTS
                    );
                } else {
                    warn!(
                        target: "lander::jito::tip",
                        "tip api 未正确初始化，回退为 {} lamports",
                        MIN_JITO_TIP_LAMPORTS
                    );
                }
                Some(self.base_tip.max(MIN_JITO_TIP_LAMPORTS))
            }
        }
    }

    fn pick_range_tip(&self) -> Option<u64> {
        if self.range_tips.is_empty() {
            return None;
        }
        let mut rng = rand::rng();
        Some(*self.range_tips.as_slice().choose(&mut rng)?)
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
                shared_clone.run().await;
            };

            let abortable = Abortable::new(future, abort_registration);
            handle.spawn(async move {
                if let Err(Aborted) = abortable.await {
                    debug!(target: "lander::jito::tip", "tip stream 任务被显式中止");
                }
            });

            TipStreamTask {
                abort: Some(abort_handle),
            }
        } else {
            warn!(
                target: "lander::jito::tip",
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

struct TipApi {
    shared: Arc<TipApiShared>,
    task: Arc<TipApiTask>,
}

impl TipApi {
    fn spawn(endpoint: String, refresh: Duration, min_tip: u64) -> Self {
        let shared = Arc::new(TipApiShared::new(endpoint, refresh, min_tip));
        let task = if let Ok(handle) = Handle::try_current() {
            let shared_clone = shared.clone();
            let (abort_handle, abort_registration) = AbortHandle::new_pair();

            let future = async move {
                TipApiShared::run(shared_clone).await;
            };

            let abortable = Abortable::new(future, abort_registration);
            handle.spawn(async move {
                if let Err(Aborted) = abortable.await {
                    debug!(target: "lander::jito::tip", "tip api 轮询任务被显式中止");
                }
            });

            TipApiTask {
                abort: Some(abort_handle),
            }
        } else {
            warn!(
                target: "lander::jito::tip",
                "tip api 未检测到 Tokio runtime，轮询功能未启动"
            );
            TipApiTask { abort: None }
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

impl Clone for TipApi {
    fn clone(&self) -> Self {
        Self {
            shared: self.shared.clone(),
            task: self.task.clone(),
        }
    }
}

impl Drop for TipApi {
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

struct TipApiTask {
    abort: Option<AbortHandle>,
}

impl TipApiTask {
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
                    info!(target: "lander::jito::tip", "jito tip stream 已连接");
                    while let Some(message) = ws.next().await {
                        match message {
                            Ok(Message::Text(text)) => {
                                if let Err(err) = self.apply_payload(&text) {
                                    warn!(
                                        target: "lander::jito::tip",
                                        error = %err,
                                        "解析 tip stream 文本消息失败"
                                    );
                                }
                            }
                            Ok(Message::Binary(binary)) => match std::str::from_utf8(&binary) {
                                Ok(text) => {
                                    if let Err(err) = self.apply_payload(text) {
                                        warn!(
                                            target: "lander::jito::tip",
                                            error = %err,
                                            "解析 tip stream 二进制消息失败"
                                        );
                                    }
                                }
                                Err(err) => {
                                    warn!(
                                        target: "lander::jito::tip",
                                        error = %err,
                                        "tip stream 二进制消息无法转为 utf8"
                                    );
                                }
                            },
                            Ok(Message::Ping(payload)) => {
                                if let Err(err) = ws.send(Message::Pong(payload)).await {
                                    warn!(
                                        target: "lander::jito::tip",
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
                                    target: "lander::jito::tip",
                                    ?frame,
                                    "tip stream 收到 close 帧"
                                );
                                break;
                            }
                            Err(err) => {
                                warn!(
                                    target: "lander::jito::tip",
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
                        target: "lander::jito::tip",
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
    #[serde(rename = "ema50")]
    ema50: Option<f64>,
    #[serde(rename = "tip_floor_lamports")]
    tip_floor_lamports: Option<f64>,
}

struct TipStreamValues {
    p25: Option<u64>,
    p50: Option<u64>,
    p75: Option<u64>,
    p95: Option<u64>,
    p99: Option<u64>,
    ema50: Option<u64>,
    floor: Option<u64>,
}

impl TipStreamValues {
    fn from_entry(entry: TipStreamEntry) -> Self {
        let TipStreamEntry {
            landed_tips_25th_percentile,
            landed_tips_50th_percentile,
            landed_tips_75th_percentile,
            landed_tips_95th_percentile,
            landed_tips_99th_percentile,
            ema50,
            tip_floor_lamports,
        } = entry;

        Self {
            p25: landed_tips_25th_percentile.and_then(parse_float_tip),
            p50: landed_tips_50th_percentile.and_then(parse_float_tip),
            p75: landed_tips_75th_percentile.and_then(parse_float_tip),
            p95: landed_tips_95th_percentile.and_then(parse_float_tip),
            p99: landed_tips_99th_percentile.and_then(parse_float_tip),
            ema50: ema50.and_then(parse_float_tip),
            floor: tip_floor_lamports.and_then(parse_float_tip),
        }
    }

    fn value(&self, level: TipStreamLevel, cap: Option<u64>) -> Option<u64> {
        let value = match level {
            TipStreamLevel::Percentile25 => self.p25,
            TipStreamLevel::Percentile50 => self.p50,
            TipStreamLevel::Percentile75 => self.p75,
            TipStreamLevel::Percentile95 => self.p95,
            TipStreamLevel::Percentile99 => self.p99,
            TipStreamLevel::Ema50 => self.ema50,
        }
        .or(self.floor);

        match (value, cap) {
            (Some(raw), Some(limit)) => Some(raw.min(limit)),
            (Some(raw), None) => Some(raw),
            (None, Some(limit)) => Some(limit),
            (None, None) => None,
        }
    }
}

struct TipApiShared {
    latest: AtomicU64,
    client: Client,
    endpoint: String,
    refresh: Duration,
    min_tip: u64,
}

impl TipApiShared {
    fn new(endpoint: String, refresh: Duration, min_tip: u64) -> Self {
        Self {
            latest: AtomicU64::new(0),
            client: Client::new(),
            endpoint,
            refresh,
            min_tip,
        }
    }

    fn latest(&self) -> Option<u64> {
        let value = self.latest.load(Ordering::Relaxed);
        if value > 0 { Some(value) } else { None }
    }

    async fn run(self: Arc<Self>) {
        loop {
            match self.fetch_once().await {
                Ok(Some(value)) => {
                    self.latest
                        .store(value.max(self.min_tip), Ordering::Relaxed);
                }
                Ok(None) => {
                    debug!(
                        target: "lander::jito::tip",
                        endpoint = %self.endpoint,
                        "tip api 返回空数据，使用旧值"
                    );
                }
                Err(err) => {
                    warn!(
                        target: "lander::jito::tip",
                        endpoint = %self.endpoint,
                        error = %err,
                        "tip api 拉取失败，将在 {:?} 后重试",
                        self.refresh
                    );
                }
            }

            sleep(self.refresh).await;
        }
    }

    async fn fetch_once(&self) -> Result<Option<u64>, reqwest::Error> {
        let response = self
            .client
            .get(&self.endpoint)
            .header("accept", "application/json")
            .send()
            .await?;
        if !response.status().is_success() {
            warn!(
                target: "lander::jito::tip",
                endpoint = %self.endpoint,
                status = %response.status(),
                "tip api 返回非成功状态码"
            );
            return Ok(None);
        }

        let text = response.text().await?;
        if text.trim().is_empty() {
            return Ok(None);
        }

        let value = match serde_json::from_str::<Value>(&text) {
            Ok(json) => extract_tip_lamports(&json),
            Err(_) => parse_tip_from_str(&text),
        };

        Ok(value)
    }
}

fn extract_tip_lamports(value: &Value) -> Option<u64> {
    match value {
        Value::Null | Value::Bool(_) => None,
        Value::Number(number) => {
            if number.is_u64() {
                number.as_u64()
            } else if let Some(float) = number.as_f64() {
                parse_float_tip(float)
            } else {
                None
            }
        }
        Value::String(text) => parse_tip_from_str(text),
        Value::Array(entries) => entries.iter().find_map(extract_tip_lamports),
        Value::Object(map) => {
            const CANDIDATE_KEYS: &[&str] = &[
                "tip_floor_lamports",
                "tip_floor",
                "tipFloorLamports",
                "tipFloor",
                "tip_floor_sol",
                "tipFloorSol",
                "value",
                "floor",
                "current_tip_floor",
                "currentTipFloor",
            ];

            for key in CANDIDATE_KEYS {
                if let Some(entry) = map.get(*key) {
                    if let Some(value) = extract_tip_lamports(entry) {
                        return Some(value);
                    }
                }
            }

            map.values().find_map(extract_tip_lamports)
        }
    }
}

fn parse_tip_from_str(text: &str) -> Option<u64> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(int_value) = trimmed.parse::<u64>() {
        return Some(int_value);
    }

    if let Ok(float_value) = trimmed.parse::<f64>() {
        return parse_float_tip(float_value);
    }

    None
}

fn parse_float_tip(value: f64) -> Option<u64> {
    if !value.is_finite() || value <= 0.0 {
        return None;
    }

    if (value.fract().abs() < 1e-9) && value > 0.0 {
        return Some(value.round() as u64);
    }

    sol_to_lamports(value)
}

fn sol_to_lamports(value: f64) -> Option<u64> {
    let lamports = (value * 1_000_000_000.0).round();
    if lamports <= 0.0 {
        None
    } else if lamports >= u64::MAX as f64 {
        Some(u64::MAX)
    } else {
        Some(lamports as u64)
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
        config.fixed_tip = Some(10_000);
        config.tip_strategy = TipStrategyKind::Stream;
        let selector = TipSelector::from_config(&config);

        let tip = selector.select_tip();
        assert_eq!(tip, Some(MIN_JITO_TIP_LAMPORTS));
    }

    #[test]
    fn parse_tip_from_string_handles_numeric() {
        assert_eq!(parse_tip_from_str("12345"), Some(12_345));
        assert_eq!(parse_tip_from_str("1.5"), sol_to_lamports(1.5));
    }
}
