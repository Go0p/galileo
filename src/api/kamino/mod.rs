//! Kamino 聚合器 API 封装，复用现有 Jupiter/DFlow 模式的客户端结构。

pub mod quote;
pub mod serde_helpers;

use std::fmt;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use metrics::{counter, histogram};
use reqwest::{StatusCode, header};
use serde_json::Value;
use thiserror::Error;
use tracing::{Level, debug, trace, warn};

use crate::config::{BotConfig, LoggingConfig, LoggingProfile};
use crate::monitoring::metrics::prometheus_enabled;
use crate::monitoring::{LatencyMetadata, guard_with_level};
use crate::network::{IpBoundClientPool, ReqwestClientFactoryFn};

pub use quote::{QuoteRequest, QuoteResponse, Route};

#[derive(Debug, Error)]
pub enum KaminoError {
    #[error("failed to call Kamino API: {0}")]
    Http(#[from] reqwest::Error),
    #[error("request to {endpoint} timed out after {timeout_ms} ms")]
    Timeout {
        endpoint: String,
        timeout_ms: u64,
        #[source]
        source: reqwest::Error,
    },
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
    #[error("failed to construct IP-bound HTTP client: {0}")]
    ClientPool(String),
}

fn apply_kamino_headers(builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
    builder
        .header(header::ACCEPT, "*/*")
        .header(header::ACCEPT_LANGUAGE, "zh-CN,zh;q=0.9")
        .header(header::ORIGIN, "https://kamino.com")
        .header(header::REFERER, "https://kamino.com/")
        .header(header::HOST, "api.kamino.finance")
        .header(
            header::USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36",
        )
        .header("Sec-Ch-Ua", r#""Google Chrome";v="141", "Not?A_Brand";v="8", "Chromium";v="141""#)
        .header("Sec-Ch-Ua-Mobile", "?0")
        .header("Sec-Ch-Ua-Platform", r#""Windows""#)
        .header("Sec-Fetch-Dest", "empty")
        .header("Sec-Fetch-Mode", "cors")
        .header("Sec-Fetch-Site", "cross-site")
        .header("Priority", "u=1, i")
}

#[derive(Clone)]
pub struct KaminoApiClient {
    quote_base_url: String,
    swap_base_url: String,
    client: reqwest::Client,
    quote_timeout: Duration,
    swap_timeout: Duration,
    log_profile: LoggingProfile,
    slow_quote_warn_ms: u64,
    slow_swap_warn_ms: u64,
    client_pool: Option<Arc<IpBoundClientPool<ReqwestClientFactoryFn>>>,
}

impl fmt::Debug for KaminoApiClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KaminoApiClient")
            .field("quote_base_url", &self.quote_base_url)
            .field("swap_base_url", &self.swap_base_url)
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

impl KaminoApiClient {
    #[allow(dead_code)]
    pub fn new(
        client: reqwest::Client,
        quote_base_url: String,
        swap_base_url: String,
        bot_config: &BotConfig,
        logging: &LoggingConfig,
    ) -> Self {
        Self::with_ip_pool(
            client,
            quote_base_url,
            swap_base_url,
            bot_config,
            logging,
            None,
        )
    }

    pub fn with_ip_pool(
        client: reqwest::Client,
        quote_base_url: String,
        swap_base_url: String,
        bot_config: &BotConfig,
        logging: &LoggingConfig,
        client_pool: Option<Arc<IpBoundClientPool<ReqwestClientFactoryFn>>>,
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
            client_pool,
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

    fn http_client(&self, local_ip: Option<IpAddr>) -> Result<reqwest::Client, KaminoError> {
        if let Some(ip) = local_ip {
            if let Some(pool) = &self.client_pool {
                return pool
                    .get_or_create(ip)
                    .map_err(|err| KaminoError::ClientPool(err.to_string()));
            }
        }
        Ok(self.client.clone())
    }

    pub async fn quote_with_ip(
        &self,
        request: &QuoteRequest,
        local_ip: Option<IpAddr>,
    ) -> Result<QuoteResponse, KaminoError> {
        let path = "/kswap/all-routes";
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
            Level::INFO
        } else {
            Level::DEBUG
        };
        let guard = guard_with_level("kamino.quote", latency_level, metadata);

        debug!(
            target: "kamino::quote",
            token_in = %request.token_in,
            token_out = %request.token_out,
            amount = request.amount,
            slippage_bps = request.max_slippage_bps,
            include_setup_ixs = request.include_setup_ixs,
            wrap_and_unwrap_sol = request.wrap_and_unwrap_sol,
            routes = ?request.routes,
            "准备请求 Kamino 报价"
        );

        let query_params = request.to_query_params();
        trace!(
            target: "kamino::quote",
            params = ?query_params,
            "Kamino 报价查询参数"
        );

        let client = self.http_client(local_ip)?;
        let started = Instant::now();
        let response = apply_kamino_headers(
            client
                .get(&url)
                .timeout(self.quote_timeout)
                .query(&query_params),
        )
        .send()
        .await
        .map_err(|err| {
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            if err.is_timeout() {
                let timeout_ms = self.quote_timeout.as_millis() as u64;
                self.record_quote_metrics("timeout", None, Some(elapsed_ms));
                warn!(
                    target: "kamino::quote",
                    endpoint = %url,
                    elapsed_ms = format_args!("{elapsed_ms:.3}"),
                    timeout_ms,
                    "Kamino 报价请求超时"
                );
                KaminoError::Timeout {
                    endpoint: url.clone(),
                    timeout_ms,
                    source: err,
                }
            } else {
                self.record_quote_metrics("transport_error", None, Some(elapsed_ms));
                KaminoError::Http(err)
            }
        })?;

        let status = response.status();
        let body = response.text().await.map_err(|err| {
            let elapsed_ms = started.elapsed().as_secs_f64() * 1000.0;
            if err.is_timeout() {
                let timeout_ms = self.quote_timeout.as_millis() as u64;
                self.record_quote_metrics("timeout", None, Some(elapsed_ms));
                warn!(
                    target: "kamino::quote",
                    endpoint = %url,
                    elapsed_ms = format_args!("{elapsed_ms:.3}"),
                    timeout_ms,
                    "Kamino 报价读取响应超时"
                );
                KaminoError::Timeout {
                    endpoint: url.clone(),
                    timeout_ms,
                    source: err,
                }
            } else {
                self.record_quote_metrics("read_error", None, Some(elapsed_ms));
                KaminoError::Http(err)
            }
        })?;

        if status == StatusCode::TOO_MANY_REQUESTS {
            self.record_quote_metrics("rate_limited", None, None);
            return Err(KaminoError::RateLimited {
                endpoint: url,
                status,
                body,
            });
        }

        if !status.is_success() {
            self.record_quote_metrics("http_error", None, None);
            return Err(KaminoError::ApiStatus {
                endpoint: url,
                status,
                body,
            });
        }

        let elapsed = started.elapsed().as_secs_f64() * 1000.0;

        let payload: Value = serde_json::from_str(&body).map_err(|err| {
            self.record_quote_metrics("decode_error", None, Some(elapsed));
            err
        })?;
        let quote = QuoteResponse::try_from_value(payload).map_err(|err| {
            self.record_quote_metrics("schema_error", None, Some(elapsed));
            KaminoError::Json(err)
        })?;

        let router = quote.best_route().map(|route| route.router_type.as_str());
        self.record_quote_metrics("ok", router, Some(elapsed));

        trace!(
            target: "kamino::quote",
            elapsed_ms = format_args!("{elapsed:.3}"),
            router = quote
                .best_route()
                .map(|route| route.router_type.as_str()),
            routes = quote.routes().len(),
            "Kamino 报价完成"
        );

        guard.finish();
        Ok(quote)
    }

    fn record_quote_metrics(
        &self,
        status: &'static str,
        _router: Option<&str>,
        elapsed_ms: Option<f64>,
    ) {
        if !prometheus_enabled() {
            return;
        }
        counter!("kamino_quote_total", "status" => status).increment(1);

        if let Some(value) = elapsed_ms {
            histogram!("kamino_quote_latency_ms", "status" => status).record(value);
        }
    }

    #[allow(dead_code)]
    pub async fn swap_instructions_with_ip(
        &self,
        _request: &Value,
        _local_ip: Option<IpAddr>,
    ) -> Result<Value, KaminoError> {
        let _url = self.swap_endpoint("/kswap/all-routes");
        Err(KaminoError::Schema(
            "Kamino 暂未提供独立的 swap 指令接口".to_string(),
        ))
    }
}
