//! DFlow 聚合器 API 定义，与 Jupiter 模块保持相同的分层结构，方便后续实现客户端逻辑。

pub mod quote;
pub mod serde_helpers;
pub mod swap_instructions;

mod headers;

use std::{
    error::Error as StdError,
    fmt,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use metrics::{counter, histogram};
use reqwest::{
    StatusCode, Url,
    header::{HOST, HeaderValue},
};
use serde_json::Value;
use thiserror::Error;
use tokio::time::sleep;
use tracing::{debug, info, trace, warn};
use url::form_urlencoded;

use crate::config::{BotConfig, LoggingConfig, LoggingProfile};
use crate::monitoring::metrics::prometheus_enabled;
use crate::monitoring::{LatencyMetadata, guard_with_level};

use self::headers::build_header_map;

#[allow(unused_imports)]
pub use quote::{
    PlatformFee, PlatformFeeMode, QuoteRequest, QuoteResponse, QuoteResponsePayload, RoutePlanLeg,
    RoutePlanLegWithData, RoutePlanStep, SlippageBps, SlippagePreset,
};
#[allow(unused_imports)]
pub use swap_instructions::{
    BlockhashWithMetadata, ComputeUnitPriceMicroLamports, CreateFeeAccount,
    DestinationAssociatedTokenAccount, DestinationTokenAccount, DestinationTokenAccountViaOwner,
    PositiveSlippageConfig, PrioritizationFeeLamports, PrioritizationFeeLamportsConfig,
    PrioritizationFeePreset, PrioritizationType, PriorityLevel, PriorityLevelWithMaxLamports,
    SwapInstructionsRequest, SwapInstructionsResponse,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DflowStage {
    Quote,
    Swap,
}

impl DflowStage {
    fn label(self) -> &'static str {
        match self {
            DflowStage::Quote => "quote",
            DflowStage::Swap => "swap",
        }
    }
}

impl fmt::Display for DflowStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

#[derive(Debug, Error)]
pub enum DflowError {
    #[error("failed to call DFlow API: {0}")]
    Http(#[from] reqwest::Error),
    #[error("failed to parse response body: {0}")]
    Json(#[from] serde_json::Error),
    #[error("API request to {endpoint} failed with status {status}: {body}")]
    ApiStatus {
        endpoint: String,
        status: StatusCode,
        body: String,
    },
    #[error("rate limited when calling {endpoint}: status {status}, body: {body}")]
    RateLimited {
        endpoint: String,
        status: StatusCode,
        body: String,
    },
    #[error("unexpected response schema: {0}")]
    Schema(String),
    #[error("failed to generate x-client headers: {0}")]
    Header(String),
    #[error("{stage} 接口连续失败 {count} 次，超过容忍上限 {limit}: {last_error}")]
    ConsecutiveFailureLimit {
        stage: DflowStage,
        count: usize,
        limit: usize,
        last_error: String,
    },
}

impl DflowError {
    pub fn describe(&self) -> String {
        let mut parts = vec![self.to_string()];
        let mut current = StdError::source(self);
        while let Some(source) = current {
            let text = source.to_string();
            if parts.last().map(|last| last == &text).unwrap_or(false) {
                current = source.source();
                continue;
            }
            parts.push(text);
            current = source.source();
        }
        parts.join(" | caused by: ")
    }
}

#[derive(Clone)]
pub struct DflowApiClient {
    quote_base_url: String,
    swap_base_url: String,
    client: reqwest::Client,
    quote_timeout: Duration,
    swap_timeout: Duration,
    log_profile: LoggingProfile,
    slow_quote_warn_ms: u64,
    slow_swap_warn_ms: u64,
    max_consecutive_failures: Option<usize>,
    wait_on_429: Duration,
    quote_failures: Arc<AtomicUsize>,
    swap_failures: Arc<AtomicUsize>,
}

impl fmt::Debug for DflowApiClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DflowApiClient")
            .field("quote_base_url", &self.quote_base_url)
            .field("swap_base_url", &self.swap_base_url)
            .field("quote_timeout", &self.quote_timeout)
            .field("swap_timeout", &self.swap_timeout)
            .field("log_profile", &self.log_profile)
            .field("slow_quote_warn_ms", &self.slow_quote_warn_ms)
            .field("slow_swap_warn_ms", &self.slow_swap_warn_ms)
            .field("max_consecutive_failures", &self.max_consecutive_failures)
            .field("wait_on_429_ms", &self.wait_on_429.as_millis())
            .finish()
    }
}

impl DflowApiClient {
    pub fn new(
        client: reqwest::Client,
        quote_base_url: String,
        swap_base_url: String,
        bot_config: &BotConfig,
        logging: &LoggingConfig,
        max_consecutive_failures: Option<usize>,
        wait_on_429: Duration,
    ) -> Self {
        let quote_timeout = Duration::from_millis(bot_config.quote_ms);
        let swap_ms = bot_config.swap_ms.unwrap_or(bot_config.quote_ms);
        let swap_timeout = Duration::from_millis(swap_ms);
        Self {
            quote_base_url,
            swap_base_url,
            client,
            quote_timeout,
            swap_timeout,
            log_profile: logging.profile,
            slow_quote_warn_ms: logging.slow_quote_warn_ms,
            slow_swap_warn_ms: logging.slow_swap_warn_ms,
            max_consecutive_failures,
            wait_on_429,
            quote_failures: Arc::new(AtomicUsize::new(0)),
            swap_failures: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn endpoint(base: &str, path: &str) -> String {
        format!(
            "{}/{}",
            base.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn quote_endpoint(&self, path: &str) -> String {
        Self::endpoint(&self.quote_base_url, path)
    }

    fn swap_endpoint(&self, path: &str) -> String {
        Self::endpoint(&self.swap_base_url, path)
    }

    fn stage_counter(&self, stage: DflowStage) -> &AtomicUsize {
        match stage {
            DflowStage::Quote => self.quote_failures.as_ref(),
            DflowStage::Swap => self.swap_failures.as_ref(),
        }
    }

    fn register_failure(&self, stage: DflowStage, error: &DflowError) -> Result<(), DflowError> {
        let counter = self.stage_counter(stage);
        let current = counter.fetch_add(1, Ordering::SeqCst) + 1;
        if let Some(limit) = self.max_consecutive_failures {
            if limit != 0 && current >= limit {
                match stage {
                    DflowStage::Quote => {
                        self.record_quote_metrics("limit_exceeded", None, None);
                    }
                    DflowStage::Swap => {
                        self.record_swap_metrics("limit_exceeded", None, None);
                    }
                }
                warn!(
                    target: "dflow::tolerance",
                    stage = stage.label(),
                    consecutive = current,
                    limit,
                    "DFlow {stage} 接口连续失败达到上限"
                );
                return Err(DflowError::ConsecutiveFailureLimit {
                    stage,
                    count: current,
                    limit,
                    last_error: error.to_string(),
                });
            }
        }
        Ok(())
    }

    fn reset_failures(&self, stage: DflowStage) {
        self.stage_counter(stage).store(0, Ordering::SeqCst);
    }

    pub async fn quote(&self, request: &QuoteRequest) -> Result<QuoteResponse, DflowError> {
        let path = "/quote";
        let url = self.quote_endpoint(path);
        let metadata = LatencyMetadata::new(
            [
                ("stage".to_string(), "quote".to_string()),
                ("url".to_string(), url.clone()),
            ]
            .into_iter()
            .collect(),
        );
        let latency_level = if self.log_profile.is_verbose() {
            tracing::Level::INFO
        } else {
            tracing::Level::DEBUG
        };
        let guard = guard_with_level("dflow.quote", latency_level, metadata);

        debug!(
            target: "dflow::quote",
            input_mint = %request.input_mint,
            output_mint = %request.output_mint,
            amount = request.amount,
            slippage = ?request.slippage_bps,
            only_direct_routes = request.only_direct_routes.unwrap_or(false),
            "开始请求 DFlow 报价"
        );

        let serialized_internal = serde_urlencoded::to_string(request)
            .map_err(|err| DflowError::Schema(format!("序列化报价参数失败: {err}")))?;
        let query_pairs: Vec<(String, String)> =
            form_urlencoded::parse(serialized_internal.as_bytes())
                .into_owned()
                .collect();
        let final_url = reqwest::Url::parse_with_params(
            &url,
            query_pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        )
        .map_err(|err| DflowError::Schema(format!("构造报价 URL 失败: {err}")))?;
        let mut path_with_query = final_url.path().to_string();
        if let Some(query) = final_url.query() {
            path_with_query.push('?');
            path_with_query.push_str(query);
        }
        let host_header_value = host_header_from_url(&final_url)
            .map(|host| {
                HeaderValue::from_str(&host)
                    .map_err(|err| DflowError::Header(format!("构造 Host 头失败: {err}")))
            })
            .transpose()?;
        let mut header_map = build_header_map(&path_with_query, "")
            .map_err(|err| DflowError::Header(err.to_string()))?;
        if let Some(value) = host_header_value.as_ref() {
            header_map.insert(HOST, value.clone());
        }

        trace!(
            target: "dflow::quote",
            url = %final_url,
            "即将发起 DFlow 报价请求"
        );

        let response = match self
            .client
            .get(&url)
            .timeout(self.quote_timeout)
            .query(request)
            .headers(header_map)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(err) => {
                let error = DflowError::from(err);
                let detail = error.describe();
                warn!(
                    target: "dflow::quote",
                    url = %final_url,
                    error = %detail,
                    "报价请求发送失败"
                );
                self.record_quote_metrics("transport_error", None, None);
                if let Err(limit_err) = self.register_failure(DflowStage::Quote, &error) {
                    return Err(limit_err);
                }
                return Err(error);
            }
        };

        let status = response.status();
        if !status.is_success() {
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|err| format!("<body decode failed: {err}>"));
            let body_summary = summarize_error_body(body_text);
            if status == StatusCode::TOO_MANY_REQUESTS {
                let log_endpoint = url.clone();
                let log_body = body_summary.clone();
                self.record_quote_metrics("rate_limited", None, Some(status));
                let error = DflowError::RateLimited {
                    endpoint: url,
                    status,
                    body: body_summary,
                };
                let detail = error.describe();
                warn!(
                    target: "dflow::quote",
                    status = status.as_u16(),
                    endpoint = %log_endpoint,
                    body = %log_body,
                    wait_ms = self.wait_on_429.as_millis(),
                    error = %detail,
                    "报价请求命中 DFlow 限流，将放弃当前套利"
                );
                if !self.wait_on_429.is_zero() {
                    sleep(self.wait_on_429).await;
                }
                if let Err(limit_err) = self.register_failure(DflowStage::Quote, &error) {
                    return Err(limit_err);
                }
                return Err(error);
            } else {
                let log_endpoint = url.clone();
                let log_body = body_summary.clone();
                self.record_quote_metrics("http_error", None, Some(status));
                let error = DflowError::ApiStatus {
                    endpoint: url,
                    status,
                    body: body_summary,
                };
                let detail = error.describe();
                warn!(
                    target: "dflow::quote",
                    status = status.as_u16(),
                    endpoint = %log_endpoint,
                    body = %log_body,
                    error = %detail,
                    "报价请求返回非 200 状态，将放弃当前套利"
                );
                if let Err(limit_err) = self.register_failure(DflowStage::Quote, &error) {
                    return Err(limit_err);
                }
                return Err(error);
            }
        }

        let value: Value = match response.json().await {
            Ok(val) => val,
            Err(err) => {
                self.record_quote_metrics("decode_error", None, Some(status));
                let error = DflowError::from(err);
                let detail = error.describe();
                warn!(
                    target: "dflow::quote",
                    endpoint = %url,
                    status = status.as_u16(),
                    error = %detail,
                    "报价响应解析失败"
                );
                if let Err(limit_err) = self.register_failure(DflowStage::Quote, &error) {
                    return Err(limit_err);
                }
                return Err(error);
            }
        };

        let quote = match QuoteResponse::try_from_value(value) {
            Ok(parsed) => parsed,
            Err(err) => {
                self.record_quote_metrics("schema_error", None, Some(status));
                let error = DflowError::Schema(format!("解析报价响应失败: {err}"));
                let detail = error.describe();
                warn!(
                    target: "dflow::quote",
                    endpoint = %url,
                    status = status.as_u16(),
                    error = %detail,
                    "报价响应 schema 校验失败"
                );
                if let Err(limit_err) = self.register_failure(DflowStage::Quote, &error) {
                    return Err(limit_err);
                }
                return Err(error);
            }
        };

        self.reset_failures(DflowStage::Quote);

        let elapsed = guard.finish();
        let elapsed_ms = elapsed.as_secs_f64() * 1_000.0;
        self.record_quote_metrics("success", Some(elapsed_ms), Some(status));
        info!(
            target: "dflow::quote",
            input_mint = %quote.payload().input_mint,
            output_mint = %quote.payload().output_mint,
            in_amount = quote.payload().in_amount,
            out_amount = quote.payload().out_amount,
            route_len = quote.payload().route_plan.len(),
            elapsed_ms = format_args!("{elapsed_ms:.3}"),
            "报价响应成功"
        );
        if elapsed_ms > self.slow_quote_warn_ms as f64 {
            warn!(
                target: "dflow::quote",
                elapsed_ms = format_args!("{elapsed_ms:.3}"),
                slow_threshold_ms = self.slow_quote_warn_ms,
                input_mint = %quote.payload().input_mint,
                output_mint = %quote.payload().output_mint,
                route_len = quote.payload().route_plan.len(),
                "报价耗时超过告警阈值"
            );
        }

        Ok(quote)
    }

    pub async fn swap_instructions(
        &self,
        request: &SwapInstructionsRequest,
    ) -> Result<SwapInstructionsResponse, DflowError> {
        let path = "/swap-instructions";
        let url = self.swap_endpoint(path);
        let metadata = LatencyMetadata::new(
            [
                ("stage".to_string(), "swap-instructions".to_string()),
                ("url".to_string(), url.clone()),
            ]
            .into_iter()
            .collect(),
        );
        let latency_level = if self.log_profile.is_verbose() {
            tracing::Level::INFO
        } else {
            tracing::Level::DEBUG
        };
        let guard = guard_with_level("dflow.swap_instructions", latency_level, metadata);

        let parsed_url = Url::parse(&url)
            .map_err(|err| DflowError::Schema(format!("解析指令 URL 失败: {err}")))?;
        let host_header_value = host_header_from_url(&parsed_url)
            .map(|host| {
                HeaderValue::from_str(&host)
                    .map_err(|err| DflowError::Header(format!("构造 Host 头失败: {err}")))
            })
            .transpose()?;
        let mut body = serde_json::to_value(request)
            .map_err(|err| DflowError::Schema(format!("序列化指令参数失败: {err}")))?;
        prune_nulls(&mut body);
        trace!(
            target: "dflow::swap_instructions",
            payload = %body,
            "即将发起 DFlow 指令请求"
        );

        let body_json = serde_json::to_string(&body)
            .map_err(|err| DflowError::Schema(format!("序列化指令 JSON 失败: {err}")))?;
        let mut header_map = build_header_map(path, &body_json)
            .map_err(|err| DflowError::Header(err.to_string()))?;
        if let Some(value) = host_header_value.as_ref() {
            header_map.insert(HOST, value.clone());
        }

        let response = match self
            .client
            .post(&url)
            .timeout(self.swap_timeout)
            .json(&body)
            .headers(header_map)
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(err) => {
                let error = DflowError::from(err);
                let detail = error.describe();
                warn!(
                    target: "dflow::swap_instructions",
                    url = %url,
                    error = %detail,
                    "指令请求发送失败"
                );
                self.record_swap_metrics("transport_error", None, None);
                if let Err(limit_err) = self.register_failure(DflowStage::Swap, &error) {
                    return Err(limit_err);
                }
                return Err(error);
            }
        };

        let status = response.status();
        if !status.is_success() {
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|err| format!("<body decode failed: {err}>"));
            let body_summary = summarize_error_body(body_text);
            if status == StatusCode::TOO_MANY_REQUESTS {
                let log_endpoint = url.clone();
                let log_body = body_summary.clone();
                self.record_swap_metrics("rate_limited", None, Some(status));
                let error = DflowError::RateLimited {
                    endpoint: url,
                    status,
                    body: body_summary,
                };
                let detail = error.describe();
                warn!(
                    target: "dflow::swap_instructions",
                    status = status.as_u16(),
                    endpoint = %log_endpoint,
                    body = %log_body,
                    wait_ms = self.wait_on_429.as_millis(),
                    error = %detail,
                    "指令请求命中 DFlow 限流，将放弃当前套利"
                );
                if !self.wait_on_429.is_zero() {
                    sleep(self.wait_on_429).await;
                }
                if let Err(limit_err) = self.register_failure(DflowStage::Swap, &error) {
                    return Err(limit_err);
                }
                return Err(error);
            } else {
                let log_endpoint = url.clone();
                let log_body = body_summary.clone();
                self.record_swap_metrics("http_error", None, Some(status));
                let error = DflowError::ApiStatus {
                    endpoint: url,
                    status,
                    body: body_summary,
                };
                let detail = error.describe();
                warn!(
                    target: "dflow::swap_instructions",
                    status = status.as_u16(),
                    endpoint = %log_endpoint,
                    body = %log_body,
                    error = %detail,
                    "指令请求返回非 200 状态，将放弃当前套利"
                );
                if let Err(limit_err) = self.register_failure(DflowStage::Swap, &error) {
                    return Err(limit_err);
                }
                return Err(error);
            }
        }

        let value: Value = match response.json().await {
            Ok(val) => val,
            Err(err) => {
                self.record_swap_metrics("decode_error", None, Some(status));
                let error = DflowError::from(err);
                let detail = error.describe();
                warn!(
                    target: "dflow::swap_instructions",
                    endpoint = %url,
                    status = status.as_u16(),
                    error = %detail,
                    "指令响应解析失败"
                );
                if let Err(limit_err) = self.register_failure(DflowStage::Swap, &error) {
                    return Err(limit_err);
                }
                return Err(error);
            }
        };
        let instructions = match SwapInstructionsResponse::try_from(value) {
            Ok(parsed) => parsed,
            Err(err) => {
                self.record_swap_metrics("schema_error", None, Some(status));
                let error = DflowError::Schema(format!("解析指令响应失败: {err}"));
                let detail = error.describe();
                warn!(
                    target: "dflow::swap_instructions",
                    endpoint = %url,
                    status = status.as_u16(),
                    error = %detail,
                    "指令响应 schema 校验失败"
                );
                if let Err(limit_err) = self.register_failure(DflowStage::Swap, &error) {
                    return Err(limit_err);
                }
                return Err(error);
            }
        };

        self.reset_failures(DflowStage::Swap);

        let elapsed = guard.finish();
        let elapsed_ms = elapsed.as_secs_f64() * 1_000.0;
        self.record_swap_metrics("success", Some(elapsed_ms), Some(status));
        info!(
            target: "dflow::swap_instructions",
            compute_unit_limit = instructions.compute_unit_limit,
            prioritization_fee = instructions
                .prioritization_fee_lamports
                .unwrap_or_default(),
            elapsed_ms = format_args!("{elapsed_ms:.3}"),
            "指令响应成功"
        );
        if elapsed_ms > self.slow_swap_warn_ms as f64 {
            warn!(
                target: "dflow::swap_instructions",
                elapsed_ms = format_args!("{elapsed_ms:.3}"),
                slow_threshold_ms = self.slow_swap_warn_ms,
                "指令请求耗时超过告警阈值"
            );
        }

        Ok(instructions)
    }

    fn record_quote_metrics(
        &self,
        outcome: &str,
        elapsed_ms: Option<f64>,
        status: Option<StatusCode>,
    ) {
        if prometheus_enabled() {
            counter!(
                "galileo_dflow_quote_total",
                "outcome" => outcome.to_string(),
            )
            .increment(1);
            if let Some(value) = elapsed_ms {
                histogram!("galileo_dflow_quote_latency_ms").record(value);
            }
            if let Some(code) = status {
                debug!(
                    target: "dflow::metrics",
                    status = code.as_u16(),
                    "DFlow quote status"
                );
            }
        }
    }

    fn record_swap_metrics(
        &self,
        outcome: &str,
        elapsed_ms: Option<f64>,
        status: Option<StatusCode>,
    ) {
        if prometheus_enabled() {
            counter!(
                "galileo_dflow_swap_total",
                "outcome" => outcome.to_string(),
            )
            .increment(1);
            if let Some(value) = elapsed_ms {
                histogram!("galileo_dflow_swap_latency_ms").record(value);
            }
            if let Some(code) = status {
                debug!(
                    target: "dflow::metrics",
                    status = code.as_u16(),
                    "DFlow swap status"
                );
            }
        }
    }
}

fn host_header_from_url(url: &Url) -> Option<String> {
    let host = url.host_str()?;
    let mut value = host.to_string();
    if let Some(port) = url.port() {
        value.push(':');
        value.push_str(&port.to_string());
    }
    Some(value)
}

fn summarize_error_body(body: String) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        "(empty response body)".to_string()
    } else {
        let mut single_line = trimmed.replace(['\n', '\r'], " ");
        const MAX_LEN: usize = 512;
        if single_line.len() > MAX_LEN {
            single_line.truncate(MAX_LEN);
            single_line.push_str("…");
        }
        single_line
    }
}

fn prune_nulls(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if let Some(entry) = map.get_mut(&key) {
                    prune_nulls(entry);
                    if entry.is_null() {
                        map.remove(&key);
                    }
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                prune_nulls(item);
            }
            arr.retain(|item| !item.is_null());
        }
        _ => {}
    }
}
