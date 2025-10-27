//! Ultra API 客户端，实现 `/order` 与 `/execute` 两个核心接口。
//!
//! 设计目标与现有 Jupiter 模块保持一致：高性能 HTTP 调用、丰富的监控指标以及
//! 详细的日志便于排障。模块只负责网络交互与数据结构定义，不包含任何策略逻辑。

#![allow(dead_code)] // TODO: 一旦 Ultra API 接入引擎流程，请移除该属性以恢复未使用代码检查。

use std::{fmt, net::IpAddr, sync::Arc, time::Duration};

use metrics::{counter, histogram};
use reqwest::StatusCode;
use serde::Serialize;
use serde_json::Value;
use thiserror::Error;
use tracing::{Level, debug, info, trace, warn};
use url::form_urlencoded;

use crate::config::{BotConfig, LoggingConfig, LoggingProfile};
use crate::monitoring::metrics::prometheus_enabled;
use crate::monitoring::{LatencyMetadata, guard_with_level};
use crate::network::{IpBoundClientPool, ReqwestClientFactoryFn};
use reqwest::header::HeaderName;
use reqwest::header::{ACCEPT, ACCEPT_ENCODING, HeaderValue, USER_AGENT};

pub mod execute;
pub mod order;
pub mod serde_helpers;

#[allow(unused_imports)]
pub use execute::{ExecuteRequest, ExecuteResponse, ExecuteStatus, SwapEvent};
#[allow(unused_imports)]
pub use order::{
    OrderRequest, OrderResponse, OrderResponsePayload, RoutePlanStep, Router, SwapInfo, SwapMode,
    UltraPlatformFee,
};

#[derive(Clone)]
pub struct UltraApiClient {
    base_url: String,
    client: reqwest::Client,
    order_timeout: Duration,
    execute_timeout: Duration,
    log_profile: LoggingProfile,
    slow_order_warn_ms: u64,
    slow_execute_warn_ms: u64,
    client_pool: Option<Arc<IpBoundClientPool<ReqwestClientFactoryFn>>>,
}

impl fmt::Debug for UltraApiClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UltraApiClient")
            .field("base_url", &self.base_url)
            .field("order_timeout", &self.order_timeout)
            .field("execute_timeout", &self.execute_timeout)
            .field("log_profile", &self.log_profile)
            .field("slow_order_warn_ms", &self.slow_order_warn_ms)
            .field("slow_execute_warn_ms", &self.slow_execute_warn_ms)
            .field(
                "ip_pool_size",
                &self.client_pool.as_ref().map(|pool| pool.len()),
            )
            .finish()
    }
}

impl UltraApiClient {
    pub fn new(
        client: reqwest::Client,
        base_url: String,
        bot_config: &BotConfig,
        logging: &LoggingConfig,
    ) -> Self {
        Self::with_ip_pool(client, base_url, bot_config, logging, None)
    }

    pub fn with_ip_pool(
        client: reqwest::Client,
        base_url: String,
        bot_config: &BotConfig,
        logging: &LoggingConfig,
        client_pool: Option<Arc<IpBoundClientPool<ReqwestClientFactoryFn>>>,
    ) -> Self {
        let trimmed = base_url.trim();
        let normalized = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            trimmed.to_string()
        } else {
            format!("https://{trimmed}")
        };
        let quote_timeout = Duration::from_millis(bot_config.quote_ms);
        let swap_ms = bot_config.swap_ms.unwrap_or(bot_config.quote_ms);
        let execute_timeout = Duration::from_millis(swap_ms);
        Self {
            base_url: normalized,
            client,
            order_timeout: quote_timeout,
            execute_timeout,
            log_profile: logging.profile,
            slow_order_warn_ms: logging.slow_quote_warn_ms,
            slow_execute_warn_ms: logging.slow_swap_warn_ms,
            client_pool,
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn order(&self, request: &OrderRequest) -> Result<OrderResponse, UltraError> {
        self.order_with_ip(request, None).await
    }

    pub async fn order_with_ip(
        &self,
        request: &OrderRequest,
        local_ip: Option<IpAddr>,
    ) -> Result<OrderResponse, UltraError> {
        let url = self.endpoint("/order");
        let metadata = LatencyMetadata::new(
            [
                ("stage".to_string(), "order".to_string()),
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
        let guard = guard_with_level("ultra.order", latency_level, metadata);

        debug!(
            target: "ultra::order",
            input_mint = %request.input_mint,
            output_mint = %request.output_mint,
            amount = request.amount,
            taker = request.taker.map(|pubkey| pubkey.to_string()),
            referral_account = request.referral_account.map(|pubkey| pubkey.to_string()),
            referral_fee = request.referral_fee,
            exclude_routers = %request.exclude_routers_label(),
            exclude_dexes = request.exclude_dexes.as_deref().unwrap_or("<none>"),
            payer = request.payer.map(|pubkey| pubkey.to_string()),
            "开始请求 Ultra /order"
        );

        let client = self.http_client(local_ip)?;

        let mut http_request = client
            .get(&url)
            .timeout(self.order_timeout)
            .query(request)
            .header(ACCEPT, HeaderValue::from_static("*/*"))
            .header(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br"))
            .header(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Safari/537.36"))
            .header(
                HeaderName::from_static("origin"),
                HeaderValue::from_static("https://jup.ag"),
            )
            .header(
                HeaderName::from_static("referer"),
                HeaderValue::from_static("https://jup.ag/"),
            )
            .header(
                HeaderName::from_static("x-client-platform"),
                HeaderValue::from_static("jupiter.web.swap_page"),
            );
        if !request.extra_query_params.is_empty() {
            http_request = http_request.query(&request.extra_query_params);
        }

        trace!(
            target: "ultra::order",
            request = ?request,
            extra = ?request.extra_query_params,
            "已构造 Ultra /order 请求"
        );

        let serialized_internal = serde_urlencoded::to_string(request)
            .map_err(|err| UltraError::Schema(format!("序列化订单参数失败: {err}")))?;
        let mut query_pairs: Vec<(String, String)> =
            form_urlencoded::parse(serialized_internal.as_bytes())
                .into_owned()
                .collect();
        for (key, value) in &request.extra_query_params {
            query_pairs.push((key.clone(), value.clone()));
        }
        let final_url = reqwest::Url::parse_with_params(
            &url,
            query_pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())),
        )
        .map_err(|err| UltraError::Schema(format!("构造订单 URL 失败: {err}")))?;
        trace!(
            target: "ultra::order",
            url = %final_url,
            "即将发起 Ultra /order 请求"
        );

        let response = http_request.send().await.map_err(|err| {
            self.record_order_metrics("transport_error", None, None);
            UltraError::from(err)
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|err| format!("<body decode failed: {err}>"));
            let body_summary = summarize_error_body(body_text);
            warn!(
                target: "ultra::order",
                status = status.as_u16(),
                endpoint = %url,
                body = %body_summary,
                "Ultra /order 返回非 200 状态"
            );
            self.record_order_metrics("http_error", None, Some(status));
            return Err(UltraError::ApiStatus {
                endpoint: url,
                status,
                body: body_summary,
            });
        }

        let status = response.status();
        let value: Value = response.json().await.map_err(|err| {
            self.record_order_metrics("decode_error", None, Some(status));
            UltraError::from(err)
        })?;
        let order = OrderResponse::try_from_value(value).map_err(|err| {
            self.record_order_metrics("schema_error", None, Some(status));
            UltraError::Schema(format!("解析订单响应失败: {err}"))
        })?;

        let elapsed = guard.finish();
        let elapsed_ms = elapsed.as_secs_f64() * 1_000.0;
        self.record_order_metrics("success", Some(elapsed_ms), Some(status));
        let log_input_mint = order.input_mint.unwrap_or_default();
        let log_output_mint = order.output_mint.unwrap_or_default();
        let log_in_amount = order.in_amount.unwrap_or_default();
        let log_out_amount = order.out_amount.unwrap_or_default();
        let log_swap_mode = order.swap_mode.unwrap_or(SwapMode::ExactIn);
        let log_router = order.router.as_deref().unwrap_or("<none>");
        let log_quote_id = order.quote_id.as_deref().unwrap_or("<none>");

        info!(
            target: "ultra::order",
            input_mint = %log_input_mint,
            output_mint = %log_output_mint,
            in_amount = log_in_amount,
            out_amount = log_out_amount,
            swap_mode = ?log_swap_mode,
            router = %log_router,
            quote_id = log_quote_id,
            elapsed_ms = format_args!("{elapsed_ms:.3}"),
            "Ultra /order 响应成功"
        );
        if elapsed_ms > self.slow_order_warn_ms as f64 {
            warn!(
                target: "ultra::order",
                elapsed_ms = format_args!("{elapsed_ms:.3}"),
                slow_threshold_ms = self.slow_order_warn_ms,
                input_mint = %log_input_mint,
                output_mint = %log_output_mint,
                router = %log_router,
                "Ultra /order 耗时超过告警阈值"
            );
        }

        Ok(order)
    }

    fn http_client(&self, local_ip: Option<IpAddr>) -> Result<reqwest::Client, UltraError> {
        if let Some(ip) = local_ip {
            if let Some(pool) = &self.client_pool {
                return pool
                    .get_or_create(ip)
                    .map_err(|err| UltraError::ClientPool(err.to_string()));
            }
        }
        Ok(self.client.clone())
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn execute(&self, request: &ExecuteRequest) -> Result<ExecuteResponse, UltraError> {
        let url = self.endpoint("/execute");
        let metadata = LatencyMetadata::new(
            [
                ("stage".to_string(), "execute".to_string()),
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
        let guard = guard_with_level("ultra.execute", latency_level, metadata);

        debug!(
            target: "ultra::execute",
            request_id = request.request_id,
            tx_len = request.signed_transaction.len(),
            "开始请求 Ultra /execute"
        );

        let response = self
            .client
            .post(&url)
            .timeout(self.execute_timeout)
            .json(
                &drop_nulls(request)
                    .map_err(|err| UltraError::Schema(format!("序列化执行请求失败: {err}")))?,
            )
            .send()
            .await
            .map_err(|err| {
                self.record_execute_metrics("transport_error", None, None);
                UltraError::from(err)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|err| format!("<body decode failed: {err}>"));
            let body_summary = summarize_error_body(body_text);
            warn!(
                target: "ultra::execute",
                status = status.as_u16(),
                endpoint = %url,
                body = %body_summary,
                "Ultra /execute 返回非 200 状态"
            );
            self.record_execute_metrics("http_error", None, Some(status));
            return Err(UltraError::ApiStatus {
                endpoint: url,
                status,
                body: body_summary,
            });
        }

        let status = response.status();
        let value: Value = response.json().await.map_err(|err| {
            self.record_execute_metrics("decode_error", None, Some(status));
            UltraError::from(err)
        })?;
        let execute: ExecuteResponse = serde_json::from_value(value).map_err(|err| {
            self.record_execute_metrics("schema_error", None, Some(status));
            UltraError::Schema(format!("解析执行响应失败: {err}"))
        })?;

        let elapsed = guard.finish();
        let elapsed_ms = elapsed.as_secs_f64() * 1_000.0;
        self.record_execute_metrics("success", Some(elapsed_ms), Some(status));
        info!(
            target: "ultra::execute",
            request_id = %request.request_id,
            status = ?execute.status,
            signature = execute.signature.as_deref().unwrap_or("<none>"),
            elapsed_ms = format_args!("{elapsed_ms:.3}"),
            "Ultra /execute 响应成功"
        );
        if elapsed_ms > self.slow_execute_warn_ms as f64 {
            warn!(
                target: "ultra::execute",
                elapsed_ms = format_args!("{elapsed_ms:.3}"),
                slow_threshold_ms = self.slow_execute_warn_ms,
                request_id = %request.request_id,
                status = ?execute.status,
                "Ultra /execute 耗时超过告警阈值"
            );
        }

        Ok(execute)
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn record_order_metrics(
        &self,
        result: &str,
        elapsed_ms: Option<f64>,
        status: Option<StatusCode>,
    ) {
        if !prometheus_enabled() {
            return;
        }
        let result = result.to_string();
        let status_label = status
            .map(|code| code.as_u16().to_string())
            .unwrap_or_else(|| "none".to_string());
        counter!(
            "galileo_ultra_order_requests_total",
            "result" => result.clone(),
            "status" => status_label.clone()
        )
        .increment(1);
        if let Some(ms) = elapsed_ms {
            histogram!(
                "galileo_ultra_order_latency_ms",
                "result" => result,
                "status" => status_label
            )
            .record(ms);
        }
    }

    fn record_execute_metrics(
        &self,
        result: &str,
        elapsed_ms: Option<f64>,
        status: Option<StatusCode>,
    ) {
        if !prometheus_enabled() {
            return;
        }
        let result = result.to_string();
        let status_label = status
            .map(|code| code.as_u16().to_string())
            .unwrap_or_else(|| "none".to_string());
        counter!(
            "galileo_ultra_execute_requests_total",
            "result" => result.clone(),
            "status" => status_label.clone()
        )
        .increment(1);
        if let Some(ms) = elapsed_ms {
            histogram!(
                "galileo_ultra_execute_latency_ms",
                "result" => result,
                "status" => status_label
            )
            .record(ms);
        }
    }
}

#[derive(Debug, Error)]
pub enum UltraError {
    #[error("调用 Ultra API 失败: {0}")]
    Http(#[from] reqwest::Error),
    #[error("解析 Ultra 响应体失败: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Ultra API {endpoint} 返回状态 {status}: {body}")]
    ApiStatus {
        endpoint: String,
        status: StatusCode,
        body: String,
    },
    #[error("Ultra API 响应格式不符合预期: {0}")]
    Schema(String),
    #[error("failed to construct IP-bound HTTP client: {0}")]
    ClientPool(String),
}

impl UltraApiClient {
    #[allow(dead_code)]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

fn drop_nulls<T: Serialize>(value: &T) -> Result<Value, serde_json::Error> {
    let mut json = serde_json::to_value(value)?;
    prune_nulls(&mut json);
    Ok(json)
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
