//! Jupiter 聚合器 API 封装。

pub mod quote;
pub mod swap_instructions;

use std::fmt;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use metrics::{counter, histogram};
use reqwest::StatusCode;
use serde_json::Value;
use thiserror::Error;
use tracing::{debug, trace, warn};

use crate::config::{EngineTimeoutConfig, LoggingConfig, LoggingProfile};
use crate::monitoring::metrics::prometheus_enabled;
use crate::monitoring::{LatencyMetadata, guard_with_level};
use crate::network::{IpBoundClientPool, ReqwestClientFactoryFn};

pub use quote::{QuoteRequest, QuoteResponse, QuoteResponsePayload};
pub use swap_instructions::{SwapInstructionsRequest, SwapInstructionsResponse};

#[derive(Debug, Error)]
pub enum JupiterError {
    #[error("Jupiter API 请求失败: {0}")]
    Http(#[from] reqwest::Error),
    #[error("请求 {endpoint} 超时（{timeout_ms}ms）")]
    Timeout {
        endpoint: String,
        timeout_ms: u64,
        #[source]
        source: reqwest::Error,
    },
    #[error("响应解析失败: {0}")]
    Json(#[from] serde_json::Error),
    #[error("请求 {endpoint} 返回状态 {status}: {body}")]
    ApiStatus {
        endpoint: String,
        status: StatusCode,
        body: String,
    },
    #[error("请求 {endpoint} 被限流，状态 {status}: {body}")]
    RateLimited {
        endpoint: String,
        status: StatusCode,
        body: String,
    },
    #[error("Jupiter 响应结构不符合预期: {0}")]
    Schema(String),
    #[error("IP 绑定客户端构造失败: {0}")]
    ClientPool(String),
}

impl JupiterError {
    pub fn describe(&self) -> String {
        use std::error::Error as _;
        let mut parts = vec![self.to_string()];
        let mut current = self.source();
        while let Some(err) = current {
            let text = err.to_string();
            if parts.last().map(|last| last == &text).unwrap_or(false) {
                current = err.source();
                continue;
            }
            parts.push(text);
            current = err.source();
        }
        parts.join(" | caused by: ")
    }
}

#[derive(Clone)]
pub struct JupiterApiClient {
    quote_url: String,
    swap_url: String,
    client: reqwest::Client,
    quote_timeout: Duration,
    swap_timeout: Duration,
    log_profile: LoggingProfile,
    slow_quote_warn_ms: u64,
    slow_swap_warn_ms: u64,
    client_pool: Option<Arc<IpBoundClientPool<ReqwestClientFactoryFn>>>,
}

impl fmt::Debug for JupiterApiClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JupiterApiClient")
            .field("quote_url", &self.quote_url)
            .field("swap_url", &self.swap_url)
            .field("quote_timeout", &self.quote_timeout)
            .field("swap_timeout", &self.swap_timeout)
            .field("log_profile", &self.log_profile)
            .field("slow_quote_warn_ms", &self.slow_quote_warn_ms)
            .field("slow_swap_warn_ms", &self.slow_swap_warn_ms)
            .field(
                "ip_pool_size",
                &self.client_pool.as_ref().map(|pool| pool.len()),
            )
            .finish()
    }
}

impl JupiterApiClient {
    #[allow(dead_code)]
    pub fn new(
        client: reqwest::Client,
        quote_url: String,
        swap_url: String,
        timeouts: &EngineTimeoutConfig,
        logging: &LoggingConfig,
    ) -> Self {
        Self::with_ip_pool(client, quote_url, swap_url, timeouts, logging, None)
    }

    pub fn with_ip_pool(
        client: reqwest::Client,
        quote_url: String,
        swap_url: String,
        timeouts: &EngineTimeoutConfig,
        logging: &LoggingConfig,
        client_pool: Option<Arc<IpBoundClientPool<ReqwestClientFactoryFn>>>,
    ) -> Self {
        let quote_timeout = Duration::from_millis(timeouts.quote_ms);
        let swap_timeout = Duration::from_millis(timeouts.swap_ms);
        Self {
            quote_url,
            swap_url,
            client,
            quote_timeout,
            swap_timeout,
            log_profile: logging.profile,
            slow_quote_warn_ms: logging.slow_quote_warn_ms,
            slow_swap_warn_ms: logging.slow_swap_warn_ms,
            client_pool,
        }
    }

    fn http_client(&self, ip: Option<IpAddr>) -> Result<reqwest::Client, JupiterError> {
        if let Some(addr) = ip {
            if let Some(pool) = &self.client_pool {
                return pool
                    .get_or_create(addr)
                    .map_err(|err| JupiterError::ClientPool(err.to_string()));
            }
        }
        Ok(self.client.clone())
    }

    pub async fn quote_with_ip(
        &self,
        request: &QuoteRequest,
        local_ip: Option<IpAddr>,
    ) -> Result<QuoteResponse, JupiterError> {
        let url = self.quote_url.clone();
        let metadata = LatencyMetadata::new(
            [
                ("stage".to_string(), "quote".to_string()),
                ("url".to_string(), url.clone()),
            ]
            .into_iter()
            .collect(),
        );
        let level = if self.log_profile.is_verbose() {
            tracing::Level::INFO
        } else {
            tracing::Level::DEBUG
        };
        let guard = guard_with_level("jupiter.quote", level, metadata);
        let started = Instant::now();

        trace!(
            target: "jupiter::quote",
            input_mint = %request.input_mint,
            output_mint = %request.output_mint,
            amount = request.amount,
            local_ip = ?local_ip,
            slippage_bps = ?request.slippage_bps,
            only_direct_routes = ?request.only_direct_routes,
            restrict_intermediate_tokens = ?request.restrict_intermediate_tokens,
            "开始请求 Jupiter 报价"
        );

        let client = self.http_client(local_ip)?;
        let params = request.to_query_params();
        let response = client
            .get(&url)
            .timeout(self.quote_timeout)
            .query(&params)
            .send()
            .await
            .map_err(|err| {
                if err.is_timeout() {
                    let timeout = self.quote_timeout.as_millis() as u64;
                    self.record_quote_metrics("timeout", None, None);
                    warn!(
                        target: "jupiter::quote",
                        endpoint = %url,
                        timeout_ms = timeout,
                        local_ip = ?local_ip,
                        "Jupiter 报价请求超时"
                    );
                    JupiterError::Timeout {
                        endpoint: url.clone(),
                        timeout_ms: timeout,
                        source: err,
                    }
                } else {
                    self.record_quote_metrics("transport_error", None, None);
                    warn!(
                        target: "jupiter::quote",
                        endpoint = %url,
                        error = %err,
                        local_ip = ?local_ip,
                        "Jupiter 报价请求发送失败"
                    );
                    JupiterError::from(err)
                }
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|err| {
            if err.is_timeout() {
                let timeout = self.quote_timeout.as_millis() as u64;
                self.record_quote_metrics("timeout", None, Some(status));
                warn!(
                    target: "jupiter::quote",
                    endpoint = %url,
                    timeout_ms = timeout,
                    local_ip = ?local_ip,
                    "Jupiter 报价读取响应超时"
                );
                JupiterError::Timeout {
                    endpoint: url.clone(),
                    timeout_ms: timeout,
                    source: err,
                }
            } else {
                self.record_quote_metrics("read_error", None, Some(status));
                warn!(
                    target: "jupiter::quote",
                    endpoint = %url,
                    error = %err,
                    local_ip = ?local_ip,
                    "Jupiter 报价读取响应失败"
                );
                JupiterError::from(err)
            }
        })?;

        if status == StatusCode::TOO_MANY_REQUESTS {
            let summary = summarize_error_body(body);
            self.record_quote_metrics("rate_limited", None, Some(status));
            warn!(
                target: "jupiter::quote",
                endpoint = %url,
                status = status.as_u16(),
                body = %summary,
                local_ip = ?local_ip,
                "Jupiter 报价命中限流"
            );
            return Err(JupiterError::RateLimited {
                endpoint: url,
                status,
                body: summary,
            });
        }

        if !status.is_success() {
            let summary = summarize_error_body(body);
            self.record_quote_metrics("http_error", None, Some(status));
            warn!(
                target: "jupiter::quote",
                endpoint = %url,
                status = status.as_u16(),
                body = %summary,
                local_ip = ?local_ip,
                "Jupiter 报价返回非 200 状态"
            );
            return Err(JupiterError::ApiStatus {
                endpoint: url,
                status,
                body: summary,
            });
        }

        let json: Value = serde_json::from_str(&body).map_err(|err| {
            self.record_quote_metrics("decode_error", None, Some(status));
            warn!(
                target: "jupiter::quote",
                endpoint = %url,
                error = %err,
                local_ip = ?local_ip,
                "Jupiter 报价 JSON 解析失败"
            );
            JupiterError::Json(err)
        })?;

        let quote = QuoteResponse::try_from_value(json).map_err(|err| {
            self.record_quote_metrics("schema_error", None, Some(status));
            warn!(
                target: "jupiter::quote",
                endpoint = %url,
                error = %err,
                local_ip = ?local_ip,
                "Jupiter 报价 schema 校验失败"
            );
            JupiterError::Schema(err.to_string())
        })?;

        let elapsed = started.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1_000.0;
        if elapsed_ms > self.slow_quote_warn_ms as f64 {
            debug!(
                target: "jupiter::quote",
                elapsed_ms = format_args!("{elapsed_ms:.3}"),
                threshold_ms = self.slow_quote_warn_ms,
                "Jupiter 报价耗时较长"
            );
        } else {
            debug!(
                target: "jupiter::quote",
                elapsed_ms = format_args!("{elapsed_ms:.3}"),
                in_amount = quote.payload().in_amount,
                out_amount = quote.payload().out_amount,
                "Jupiter 报价完成"
            );
        }

        self.record_quote_metrics("success", Some(elapsed_ms), Some(status));
        guard.finish();
        Ok(quote)
    }

    pub async fn swap_instructions_with_ip(
        &self,
        request: &SwapInstructionsRequest,
        local_ip: Option<IpAddr>,
    ) -> Result<SwapInstructionsResponse, JupiterError> {
        let url = self.swap_url.clone();
        let metadata = LatencyMetadata::new(
            [
                ("stage".to_string(), "swap-instructions".to_string()),
                ("url".to_string(), url.clone()),
            ]
            .into_iter()
            .collect(),
        );
        let level = if self.log_profile.is_verbose() {
            tracing::Level::INFO
        } else {
            tracing::Level::DEBUG
        };
        let guard = guard_with_level("jupiter.swap_instructions", level, metadata);
        let started = Instant::now();

        let mut payload = serde_json::to_value(request)
            .map_err(|err| JupiterError::Schema(format!("序列化 swap 请求失败: {err}")))?;
        prune_nulls(&mut payload);
        trace!(
            target: "jupiter::swap",
            payload = %payload,
            local_ip = ?local_ip,
            "即将请求 Jupiter swap 指令"
        );

        let client = self.http_client(local_ip)?;
        let response = client
            .post(&url)
            .timeout(self.swap_timeout)
            .json(&payload)
            .send()
            .await
            .map_err(|err| {
                if err.is_timeout() {
                    let timeout = self.swap_timeout.as_millis() as u64;
                    self.record_swap_metrics("timeout", None, None);
                    warn!(
                        target: "jupiter::swap",
                        endpoint = %url,
                        timeout_ms = timeout,
                        local_ip = ?local_ip,
                        "Jupiter 指令请求超时"
                    );
                    JupiterError::Timeout {
                        endpoint: url.clone(),
                        timeout_ms: timeout,
                        source: err,
                    }
                } else {
                    self.record_swap_metrics("transport_error", None, None);
                    warn!(
                        target: "jupiter::swap",
                        endpoint = %url,
                        error = %err,
                        local_ip = ?local_ip,
                        "Jupiter 指令请求发送失败"
                    );
                    JupiterError::from(err)
                }
            })?;

        let status = response.status();
        let body = response.text().await.map_err(|err| {
            if err.is_timeout() {
                let timeout = self.swap_timeout.as_millis() as u64;
                self.record_swap_metrics("timeout", None, Some(status));
                warn!(
                    target: "jupiter::swap",
                    endpoint = %url,
                    timeout_ms = timeout,
                    local_ip = ?local_ip,
                    "Jupiter 指令读取响应超时"
                );
                JupiterError::Timeout {
                    endpoint: url.clone(),
                    timeout_ms: timeout,
                    source: err,
                }
            } else {
                self.record_swap_metrics("read_error", None, Some(status));
                warn!(
                    target: "jupiter::swap",
                    endpoint = %url,
                    error = %err,
                    local_ip = ?local_ip,
                    "Jupiter 指令读取响应失败"
                );
                JupiterError::from(err)
            }
        })?;

        if status == StatusCode::TOO_MANY_REQUESTS {
            let summary = summarize_error_body(body);
            self.record_swap_metrics("rate_limited", None, Some(status));
            warn!(
                target: "jupiter::swap",
                endpoint = %url,
                status = status.as_u16(),
                body = %summary,
                local_ip = ?local_ip,
                "Jupiter 指令请求命中限流"
            );
            return Err(JupiterError::RateLimited {
                endpoint: url,
                status,
                body: summary,
            });
        }

        if !status.is_success() {
            let summary = summarize_error_body(body);
            self.record_swap_metrics("http_error", None, Some(status));
            warn!(
                target: "jupiter::swap",
                endpoint = %url,
                status = status.as_u16(),
                body = %summary,
                local_ip = ?local_ip,
                "Jupiter 指令返回非 200 状态"
            );
            return Err(JupiterError::ApiStatus {
                endpoint: url,
                status,
                body: summary,
            });
        }

        let json: Value = serde_json::from_str(&body).map_err(|err| {
            self.record_swap_metrics("decode_error", None, Some(status));
            warn!(
                target: "jupiter::swap",
                endpoint = %url,
                error = %err,
                local_ip = ?local_ip,
                "Jupiter 指令 JSON 解析失败"
            );
            JupiterError::Json(err)
        })?;

        let instructions = SwapInstructionsResponse::try_from(json).map_err(|err| {
            self.record_swap_metrics("schema_error", None, Some(status));
            warn!(
                target: "jupiter::swap",
                endpoint = %url,
                error = %err,
                local_ip = ?local_ip,
                "Jupiter 指令 schema 校验失败"
            );
            JupiterError::Schema(err.to_string())
        })?;

        if let Some(limit) = instructions
            .compute_budget_instructions
            .iter()
            .find_map(swap_instructions::parse_compute_budget_limit)
        {
            debug!(
                target: "jupiter::swap",
                compute_unit_limit = limit,
                "Jupiter 指令返回 ComputeBudget 限制"
            );
        }

        let elapsed = started.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1_000.0;
        if elapsed_ms > self.slow_swap_warn_ms as f64 {
            debug!(
                target: "jupiter::swap",
                elapsed_ms = format_args!("{elapsed_ms:.3}"),
                threshold_ms = self.slow_swap_warn_ms,
                "Jupiter 指令请求耗时较长"
            );
        } else {
            debug!(
                target: "jupiter::swap",
                elapsed_ms = format_args!("{elapsed_ms:.3}"),
                compute_unit_limit = instructions.compute_unit_limit,
                "Jupiter 指令请求完成"
            );
        }

        self.record_swap_metrics("success", Some(elapsed_ms), Some(status));
        guard.finish();
        Ok(instructions)
    }

    fn record_quote_metrics(
        &self,
        status: &'static str,
        elapsed_ms: Option<f64>,
        http_status: Option<StatusCode>,
    ) {
        if !prometheus_enabled() {
            return;
        }
        counter!(
            "galileo_jupiter_quote_total",
            "status" => status,
            "http_status" => http_status
                .map(|code| code.as_u16().to_string())
                .unwrap_or_else(|| "none".to_string())
        )
        .increment(1);
        if let Some(value) = elapsed_ms {
            histogram!(
                "galileo_jupiter_quote_latency_ms",
                "status" => status
            )
            .record(value);
        }
    }

    fn record_swap_metrics(
        &self,
        status: &'static str,
        elapsed_ms: Option<f64>,
        http_status: Option<StatusCode>,
    ) {
        if !prometheus_enabled() {
            return;
        }
        counter!(
            "galileo_jupiter_swap_total",
            "status" => status,
            "http_status" => http_status
                .map(|code| code.as_u16().to_string())
                .unwrap_or_else(|| "none".to_string())
        )
        .increment(1);
        if let Some(value) = elapsed_ms {
            histogram!(
                "galileo_jupiter_swap_latency_ms",
                "status" => status
            )
            .record(value);
        }
    }
}

fn summarize_error_body(body: String) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return "(empty response body)".to_string();
    }
    let mut single_line = trimmed.replace(['\n', '\r'], " ");
    const MAX_LEN: usize = 512;
    if single_line.len() > MAX_LEN {
        single_line.truncate(MAX_LEN);
        single_line.push('…');
    }
    single_line
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
        Value::Array(array) => {
            for item in array.iter_mut() {
                prune_nulls(item);
            }
            array.retain(|item| !item.is_null());
        }
        _ => {}
    }
}
