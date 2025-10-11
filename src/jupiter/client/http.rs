use std::time::Instant;

use tracing::{debug, info};

use super::types::{QuoteRequest, QuoteResponse, SwapRequest, SwapResponse};
use crate::config::HttpConfig;
use crate::jupiter::error::JupiterError;
use crate::metrics::{LatencyMetadata, guard_with_metadata};

#[derive(Clone, Debug)]
pub struct JupiterApiClient {
    base_url: String,
    client: reqwest::Client,
    request_timeout: std::time::Duration,
}

impl JupiterApiClient {
    pub fn new(client: reqwest::Client, config: &HttpConfig) -> Self {
        Self {
            base_url: config.base_url.clone(),
            client,
            request_timeout: std::time::Duration::from_millis(config.request_timeout_ms),
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

        let response = self
            .client
            .get(&url)
            .timeout(self.request_timeout)
            .query(&request.to_query_params())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(JupiterError::ApiStatus {
                endpoint: url,
                status: response.status(),
            });
        }

        let value: serde_json::Value = response.json().await?;
        let quote = QuoteResponse::try_from_value(value)
            .map_err(|err| JupiterError::Schema(format!("invalid quote response: {err}")))?;

        guard.finish();

        let elapsed_ms = start.elapsed().as_micros() as f64 / 1_000.0;
        if let Some(api_time) = quote.time_taken {
            debug!(
                target: "latency",
                elapsed_ms,
                api_time = api_time * 1_000.0,
                "quote latency comparison"
            );
        } else {
            info!(
                target: "latency",
                elapsed_ms,
                "quote latency recorded"
            );
        }

        Ok(quote)
    }

    pub async fn swap(&self, request: &SwapRequest) -> Result<SwapResponse, JupiterError> {
        let url = self.endpoint("/swap/v1/swap");
        let metadata = LatencyMetadata::new(
            [
                ("stage".to_string(), "swap".to_string()),
                ("url".to_string(), url.clone()),
            ]
            .into_iter()
            .collect(),
        );
        let guard = guard_with_metadata("jupiter.swap", metadata);
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

        let value: serde_json::Value = response.json().await?;
        let swap = SwapResponse::try_from_value(value)
            .map_err(|err| JupiterError::Schema(format!("invalid swap response: {err}")))?;

        guard.finish();
        let elapsed_ms = start.elapsed().as_micros() as f64 / 1_000.0;

        info!(
            target: "latency",
            elapsed_ms,
            "swap latency recorded"
        );

        Ok(swap)
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}
