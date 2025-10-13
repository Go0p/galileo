use std::time::{Duration, Instant};

use serde_json::Value;
use tracing::{debug, info};

use crate::config::BotConfig;
use crate::jupiter::error::JupiterError;
use crate::metrics::{LatencyMetadata, guard_with_metadata};

pub mod quote;
pub mod route_plan_with_metadata;
pub mod serde_helpers;
pub mod swap_instructions;
pub mod transaction_config;

pub use quote::{QuoteRequest, QuoteResponse};
pub use swap_instructions::{SwapInstructionsRequest, SwapInstructionsResponse};
pub use transaction_config::ComputeUnitPriceMicroLamports;

#[derive(Clone, Debug)]
pub struct JupiterApiClient {
    base_url: String,
    client: reqwest::Client,
    request_timeout: Duration,
}

impl JupiterApiClient {
    pub fn new(client: reqwest::Client, base_url: String, config: &BotConfig) -> Self {
        Self {
            base_url,
            client,
            request_timeout: Duration::from_millis(config.request_timeout_ms),
        }
    }

    pub async fn quote(&self, request: &QuoteRequest) -> Result<QuoteResponse, JupiterError> {
        let url = self.endpoint("/swap/v1/quote");
        let metadata = LatencyMetadata::new(
            [
                ("stage".to_string(), "quote".to_string()),
                ("url".to_string(), url.clone()),
            ]
            .into_iter()
            .collect(),
        );
        let guard = guard_with_metadata("jupiter.quote", metadata);
        let start = Instant::now();

        let prepared = request.to_internal();

        let mut http_request = self
            .client
            .get(&url)
            .timeout(self.request_timeout)
            .query(&prepared.internal);
        if let Some(extra) = &prepared.quote_args {
            http_request = http_request.query(extra);
        }
        if !prepared.extra.is_empty() {
            http_request = http_request.query(&prepared.extra);
        }

        let response = http_request.send().await?;

        if !response.status().is_success() {
            return Err(JupiterError::ApiStatus {
                endpoint: url,
                status: response.status(),
            });
        }

        let value: Value = response.json().await?;
        let quote = QuoteResponse::try_from_value(value)
            .map_err(|err| JupiterError::Schema(format!("解析报价响应失败: {err}")))?;

        guard.finish();

        let elapsed_ms = start.elapsed().as_micros() as f64 / 1_000.0;
        if quote.time_taken > 0.0 {
            debug!(
                target: "latency",
                elapsed_ms,
                api_time = quote.time_taken * 1_000.0,
                "对比 Jupiter 报价耗时"
            );
        } else {
            info!(target: "latency", elapsed_ms, "记录到报价耗时");
        }
        info!(
            target: "jupiter::quote",
            input_mint = %quote.input_mint,
            output_mint = %quote.output_mint,
            in_amount = quote.in_amount,
            out_amount = quote.out_amount,
            other_amount_threshold = quote.other_amount_threshold,
            elapsed_ms,
            "报价请求完成"
        );

        Ok(quote)
    }

    pub async fn swap_instructions(
        &self,
        request: &SwapInstructionsRequest,
    ) -> Result<SwapInstructionsResponse, JupiterError> {
        let url = self.endpoint("/swap/v1/swap-instructions");
        let metadata = LatencyMetadata::new(
            [
                ("stage".to_string(), "swap_instructions".to_string()),
                ("url".to_string(), url.clone()),
            ]
            .into_iter()
            .collect(),
        );
        let guard = guard_with_metadata("jupiter.swap_instructions", metadata);
        let start = Instant::now();

        let response = self
            .client
            .post(&url)
            .timeout(self.request_timeout)
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(JupiterError::ApiStatus {
                endpoint: url,
                status: response.status(),
            });
        }

        let value: Value = response.json().await?;
        let instructions = SwapInstructionsResponse::try_from(value)
            .map_err(|err| JupiterError::Schema(format!("解析 Swap 指令响应失败: {err}")))?;

        guard.finish();
        let elapsed_ms = start.elapsed().as_micros() as f64 / 1_000.0;

        info!(
            target: "jupiter::swap_instructions",
            elapsed_ms,
            compute_unit_limit = instructions.compute_unit_limit,
            prioritization_fee_lamports = instructions.prioritization_fee_lamports,
            setup_ix = instructions.setup_instructions.len(),
            other_ix = instructions.other_instructions.len(),
            "已获取 Swap 指令响应"
        );

        Ok(instructions)
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}
