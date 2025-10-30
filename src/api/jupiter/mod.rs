use std::{fmt, net::IpAddr, sync::Arc, time::Duration};

use metrics::{counter, histogram};
use reqwest::StatusCode;
use serde::Serialize;
use serde_json::Value;
use tracing::{Level, debug, info, trace, warn};
use url::form_urlencoded;

use crate::config::{BotConfig, LoggingConfig, LoggingProfile};
use crate::jupiter::error::JupiterError;
use crate::monitoring::metrics::prometheus_enabled;
use crate::monitoring::{LatencyMetadata, guard_with_level};
use crate::network::{IpBoundClientPool, ReqwestClientFactoryFn};

pub mod quote;
pub mod serde_helpers;
pub mod swap_instructions;

pub use quote::{QuoteRequest, QuoteResponse, QuoteResponsePayload};
pub use swap_instructions::{
    ComputeUnitPriceMicroLamports, SwapInstructionsRequest, SwapInstructionsResponse,
};

#[derive(Clone)]
pub struct JupiterApiClient {
    base_url: String,
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
            .field("base_url", &self.base_url)
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
        let quote_timeout = Duration::from_millis(bot_config.quote_ms);
        let swap_ms = bot_config.swap_ms.unwrap_or(bot_config.quote_ms);
        let swap_timeout = Duration::from_millis(swap_ms);
        Self {
            base_url,
            client,
            quote_timeout,
            swap_timeout,
            log_profile: logging.profile,
            slow_quote_warn_ms: logging.slow_quote_warn_ms,
            slow_swap_warn_ms: logging.slow_swap_warn_ms,
            client_pool,
        }
    }

    pub async fn quote_with_ip(
        &self,
        request: &QuoteRequest,
        local_ip: Option<IpAddr>,
    ) -> Result<QuoteResponse, JupiterError> {
        let url = self.endpoint("/quote");
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
        let guard = guard_with_level("jupiter.quote", latency_level, metadata);

        debug!(
            target: "jupiter::quote",
            input_mint = %request.input_mint,
            output_mint = %request.output_mint,
            amount = request.amount,
            slippage_bps = request.slippage_bps,
            only_direct_routes = request.only_direct_routes.unwrap_or(false),
            restrict_intermediate_tokens = request.restrict_intermediate_tokens.unwrap_or(false),
            "开始请求 Jupiter 报价"
        );

        let client = self.http_client(local_ip)?;

        let mut http_request = client.get(&url).timeout(self.quote_timeout).query(request);
        if !request.extra_query_params.is_empty() {
            http_request = http_request.query(&request.extra_query_params);
        }

        trace!(
            target: "jupiter::quote",
            request = ?request,
            extra = ?request.extra_query_params,
            "已构造 Jupiter 报价请求"
        );

        let serialized_internal = serde_urlencoded::to_string(request)
            .map_err(|err| JupiterError::Schema(format!("序列化报价参数失败: {err}")))?;
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
        .map_err(|err| JupiterError::Schema(format!("构造报价 URL 失败: {err}")))?;
        trace!(
            target: "jupiter::quote",
            url = %final_url,
            "即将发起 Jupiter 报价请求"
        );

        let response = http_request.send().await.map_err(|err| {
            self.record_quote_metrics("transport_error", None, None);
            JupiterError::from(err)
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|err| format!("<body decode failed: {err}>"));
            let body_summary = summarize_error_body(body_text);
            warn!(
                target: "jupiter::quote",
                status = status.as_u16(),
                endpoint = %url,
                body = %body_summary,
                "报价请求返回非 200 状态"
            );
            self.record_quote_metrics("http_error", None, Some(status));
            return Err(JupiterError::ApiStatus {
                endpoint: url,
                status,
                body: body_summary,
            });
        }

        let status = response.status();
        let value: Value = response.json().await.map_err(|err| {
            self.record_quote_metrics("decode_error", None, Some(status));
            JupiterError::from(err)
        })?;
        let quote = QuoteResponse::try_from_value(value).map_err(|err| {
            self.record_quote_metrics("schema_error", None, Some(status));
            JupiterError::Schema(format!("解析报价响应失败: {err}"))
        })?;

        let elapsed = guard.finish();
        let elapsed_ms = elapsed.as_secs_f64() * 1_000.0;
        self.record_quote_metrics("success", Some(elapsed_ms), Some(status));
        if quote.time_taken > 0.0 && self.log_profile.is_verbose() {
            info!(
                target: "latency",
                operation = "jupiter.quote.api",
                elapsed_ms = format_args!("{elapsed_ms:.3}"),
                api_time_ms = format_args!("{:.3}", quote.time_taken * 1_000.0),
                "Jupiter 报价耗时对比"
            );
        }
        debug!(
            target: "jupiter::quote",
            input_mint = %quote.input_mint,
            output_mint = %quote.output_mint,
            in_amount = quote.in_amount,
            out_amount = quote.out_amount,
            other_amount_threshold = quote.other_amount_threshold,
            route_len = quote.route_plan.len(),
            elapsed_ms = format_args!("{elapsed_ms:.3}"),
            api_time_ms = format_args!("{:.3}", quote.time_taken * 1_000.0),
            "报价响应成功"
        );
        if elapsed_ms > self.slow_quote_warn_ms as f64 {
            warn!(
                target: "jupiter::quote",
                elapsed_ms = format_args!("{elapsed_ms:.3}"),
                slow_threshold_ms = self.slow_quote_warn_ms,
                input_mint = %quote.input_mint,
                output_mint = %quote.output_mint,
                route_len = quote.route_plan.len(),
                "报价耗时超过告警阈值"
            );
        }

        Ok(quote)
    }

    fn http_client(&self, local_ip: Option<IpAddr>) -> Result<reqwest::Client, JupiterError> {
        if let Some(ip) = local_ip {
            if let Some(pool) = &self.client_pool {
                return pool
                    .get_or_create(ip)
                    .map_err(|err| JupiterError::ClientPool(err.to_string()));
            }
        }
        Ok(self.client.clone())
    }

    pub async fn swap_instructions(
        &self,
        request: &SwapInstructionsRequest,
        local_ip: Option<IpAddr>,
    ) -> Result<SwapInstructionsResponse, JupiterError> {
        let url = self.endpoint("/swap-instructions");
        let metadata = LatencyMetadata::new(
            [
                ("stage".to_string(), "swap_instructions".to_string()),
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
        let guard = guard_with_level("jupiter.swap_instructions", latency_level, metadata);

        let wrap_and_unwrap_sol = request.wrap_and_unwrap_sol;
        let use_shared = request.use_shared_accounts.unwrap_or(false);
        let shared_overridden = request.use_shared_accounts.is_some();
        let skip_user_accounts = request.skip_user_accounts_rpc_calls;
        let allow_optimized = request.allow_optimized_wrapped_sol_token_account;
        let quote_in_amount = request.quote_response.in_amount;
        let quote_out_amount = request.quote_response.out_amount;
        let route_steps = request.quote_response.route_plan.len();
        debug!(
            target: "jupiter::swap_instructions",
            user = %request.user_public_key,
            wrap_and_unwrap_sol = wrap_and_unwrap_sol,
            skip_user_accounts,
            use_shared_accounts = use_shared,
            shared_accounts_overridden = shared_overridden,
            allow_optimized_wrapped_sol = allow_optimized,
            quote_in_amount,
            quote_out_amount,
            route_steps,
            "开始请求 Jupiter Swap 指令"
        );

        let body_json = drop_nulls(request)
            .and_then(|val| serde_json::to_string_pretty(&val))
            .map_err(|err| JupiterError::Schema(format!("序列化 Swap 指令请求失败: {err}")))?;
        trace!(
            target: "jupiter::swap_instructions",
            url = %url,
            request = %body_json,
            "即将发起 Jupiter Swap 指令请求"
        );

        let client = self.http_client(local_ip)?;

        let response = client
            .post(&url)
            .timeout(self.swap_timeout)
            .json(request)
            .send()
            .await
            .map_err(|err| {
                self.record_swap_metrics("transport_error", None, None);
                JupiterError::from(err)
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response
                .text()
                .await
                .unwrap_or_else(|err| format!("<body decode failed: {err}>"));
            let body_summary = summarize_error_body(body_text);
            warn!(
                target: "jupiter::swap_instructions",
                status = status.as_u16(),
                endpoint = %url,
                body = %body_summary,
                "Swap 指令请求返回非 200 状态"
            );
            self.record_swap_metrics("http_error", None, Some(status));
            return Err(JupiterError::ApiStatus {
                endpoint: url,
                status,
                body: body_summary,
            });
        }

        let status = response.status();
        let value: Value = response.json().await.map_err(|err| {
            self.record_swap_metrics("decode_error", None, Some(status));
            JupiterError::from(err)
        })?;
        let instructions = SwapInstructionsResponse::try_from(value).map_err(|err| {
            self.record_swap_metrics("schema_error", None, Some(status));
            JupiterError::Schema(format!("解析 Swap 指令响应失败: {err}"))
        })?;

        let elapsed = guard.finish();
        let elapsed_ms = elapsed.as_secs_f64() * 1_000.0;
        self.record_swap_metrics("success", Some(elapsed_ms), Some(status));

        debug!(
            target: "jupiter::swap_instructions",
            elapsed_ms = format_args!("{elapsed_ms:.3}"),
            compute_unit_limit = instructions.compute_unit_limit,
            prioritization_fee_lamports = instructions.prioritization_fee_lamports,
            setup_ix = instructions.setup_instructions.len(),
            other_ix = instructions.other_instructions.len(),
            "Swap 指令响应成功"
        );
        if elapsed_ms > self.slow_swap_warn_ms as f64 {
            warn!(
                target: "jupiter::swap_instructions",
                elapsed_ms = format_args!("{elapsed_ms:.3}"),
                slow_threshold_ms = self.slow_swap_warn_ms,
                compute_unit_limit = instructions.compute_unit_limit,
                prioritization_fee_lamports = instructions.prioritization_fee_lamports,
                setup_ix = instructions.setup_instructions.len(),
                other_ix = instructions.other_instructions.len(),
                "Swap 指令耗时超过告警阈值"
            );
        }

        Ok(instructions)
    }

    fn record_quote_metrics(
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
            "galileo_jupiter_quote_requests_total",
            "result" => result.clone(),
            "status" => status_label.clone()
        )
        .increment(1);
        if let Some(ms) = elapsed_ms {
            histogram!(
                "galileo_jupiter_quote_latency_ms",
                "result" => result,
                "status" => status_label
            )
            .record(ms);
        }
    }

    fn record_swap_metrics(
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
            "galileo_jupiter_swap_requests_total",
            "result" => result.clone(),
            "status" => status_label.clone()
        )
        .increment(1);
        if let Some(ms) = elapsed_ms {
            histogram!(
                "galileo_jupiter_swap_latency_ms",
                "result" => result,
                "status" => status_label
            )
            .record(ms);
        }
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
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
