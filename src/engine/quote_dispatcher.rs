use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::{StreamExt, stream::FuturesUnordered};
use reqwest::StatusCode;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::sleep;
use tracing::{trace, warn};

use crate::engine::EngineResult;
use crate::engine::error::EngineError;
use crate::engine::quote::QuoteExecutor;
use crate::engine::types::{DoubleQuote, QuoteTask};
use crate::monitoring::events;
use crate::network::{IpAllocator, IpLeaseMode, IpLeaseOutcome, IpTaskKind};

use super::context::QuoteBatchPlan;
use super::quote::QuoteConfig;

#[derive(Clone)]
pub struct QuoteDispatcher {
    ip_allocator: Arc<IpAllocator>,
    parallelism_override: Option<u16>,
    batch_interval: Duration,
}

pub struct QuoteDispatchOutcome {
    pub batch: QuoteBatchPlan,
    pub quote: Option<DoubleQuote>,
    pub local_ip: Option<IpAddr>,
}

impl QuoteDispatcher {
    pub fn new(
        ip_allocator: Arc<IpAllocator>,
        parallelism_override: Option<u16>,
        batch_interval: Duration,
    ) -> Self {
        Self {
            ip_allocator,
            parallelism_override,
            batch_interval,
        }
    }

    pub fn plan(&self, batches: Vec<QuoteBatchPlan>) -> Vec<QuoteBatchPlan> {
        let planned = batches.len();
        let parallelism = self.effective_parallelism(planned);
        trace!(
            target: "engine::dispatcher",
            planned,
            parallelism,
            total_slots = self.ip_allocator.total_slots(),
            per_ip_limit = ?self.ip_allocator.per_ip_inflight_limit(),
            "QuoteDispatcher 生成批次计划"
        );
        batches
    }

    pub async fn dispatch(
        &self,
        batches: Vec<QuoteBatchPlan>,
        quote_executor: QuoteExecutor,
        quote_config: QuoteConfig,
        strategy_label: Arc<String>,
    ) -> EngineResult<Vec<QuoteDispatchOutcome>> {
        if batches.is_empty() {
            return Ok(Vec::new());
        }

        let planned = batches.len();
        let parallelism = self.effective_parallelism(planned);
        let semaphore = if parallelism >= planned {
            None
        } else {
            Some(Arc::new(Semaphore::new(parallelism)))
        };

        let mut futures: FuturesUnordered<
            Pin<
                Box<
                    dyn Future<
                            Output = (
                                QuoteBatchPlan,
                                Option<IpAddr>,
                                Result<Option<DoubleQuote>, EngineError>,
                            ),
                        > + Send,
                >,
            >,
        > = FuturesUnordered::new();

        for (index, batch) in batches.into_iter().enumerate() {
            let semaphore = semaphore.clone();
            let dispatcher = self.clone();
            let executor = quote_executor.clone();
            let config = quote_config.clone();
            let strategy = Arc::clone(&strategy_label);

            futures.push(Box::pin(async move {
                let delay = dispatcher.compute_dispatch_delay(index, batch.process_delay);
                if !delay.is_zero() {
                    sleep(delay).await;
                }

                let permit = match semaphore {
                    Some(ref sem) => match acquire_permit(sem).await {
                        Ok(permit) => Some(permit),
                        Err(err) => return (batch, None, Err(err)),
                    },
                    None => None,
                };

                let lease = dispatcher
                    .ip_allocator
                    .acquire(IpTaskKind::QuoteBuy, IpLeaseMode::Ephemeral)
                    .await;
                let lease = match lease {
                    Ok(lease) => lease,
                    Err(err) => {
                        if let Some(permit) = permit {
                            drop(permit);
                        }
                        return (batch, None, Err(EngineError::NetworkResource(err)));
                    }
                };
                let lease_handle = lease.handle();
                drop(lease);
                let local_ip = Some(lease_handle.ip());

                let task = QuoteTask::new(batch.pair.clone(), batch.amount);
                events::quote_start(strategy.as_str(), &task, Some(batch.batch_id), local_ip);
                let started = Instant::now();

                let result = executor.round_trip(&task, &config, &lease_handle).await;

                match &result {
                    Ok(Some(_)) => {
                        events::quote_end(
                            strategy.as_str(),
                            &task,
                            true,
                            started.elapsed(),
                            Some(batch.batch_id),
                            local_ip,
                        );
                    }
                    Ok(None) => {
                        events::quote_end(
                            strategy.as_str(),
                            &task,
                            false,
                            started.elapsed(),
                            Some(batch.batch_id),
                            local_ip,
                        );
                    }
                    Err(err) => {
                        events::quote_end(
                            strategy.as_str(),
                            &task,
                            false,
                            started.elapsed(),
                            Some(batch.batch_id),
                            local_ip,
                        );
                        if let Some(outcome) = classify_ip_outcome(err) {
                            lease_handle.mark_outcome(outcome);
                        }
                    }
                }

                if let Some(permit) = permit {
                    drop(permit);
                }

                drop(lease_handle);

                (batch, local_ip, result)
            }));
        }

        let mut outcomes = Vec::new();
        let mut first_error: Option<EngineError> = None;

        while let Some((batch, local_ip, result)) = futures.next().await {
            match result {
                Ok(quote) => outcomes.push(QuoteDispatchOutcome {
                    batch,
                    quote,
                    local_ip,
                }),
                Err(err) => {
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }
            }
        }

        outcomes.sort_by_key(|outcome| outcome.batch.batch_id);

        if let Some(err) = first_error {
            return Err(err);
        }

        Ok(outcomes)
    }

    fn effective_parallelism(&self, planned: usize) -> usize {
        if planned <= 1 {
            return planned.max(1);
        }

        if let Some(value) = self.parallelism_override {
            return usize::from(value.max(1)).min(planned);
        }

        let slots = self.ip_allocator.total_slots().max(1);
        let per_ip = self
            .ip_allocator
            .per_ip_inflight_limit()
            .unwrap_or(1)
            .max(1);
        let computed = slots.saturating_mul(per_ip);
        computed.max(1).min(planned)
    }

    fn compute_dispatch_delay(&self, index: usize, process_delay: Duration) -> Duration {
        let base_delay = if self.batch_interval.is_zero() || index == 0 {
            Duration::ZERO
        } else {
            // 使用 f32 保持简单，batch 数量有限时不会溢出
            self.batch_interval.mul_f32(index as f32)
        };

        match base_delay.checked_add(process_delay) {
            Some(total) => total,
            None => {
                warn!(
                    target: "engine::dispatcher",
                    batch_index = index,
                    base_delay = ?base_delay,
                    process_delay = ?process_delay,
                    "批次调度延迟溢出，使用 1 秒退避"
                );
                Duration::from_secs(1)
            }
        }
    }
}

async fn acquire_permit(semaphore: &Arc<Semaphore>) -> EngineResult<OwnedSemaphorePermit> {
    semaphore.clone().acquire_owned().await.map_err(|_| {
        EngineError::InvalidConfig("QuoteDispatcher semaphore unexpectedly closed".into())
    })
}

pub(crate) fn classify_ip_outcome(err: &EngineError) -> Option<IpLeaseOutcome> {
    match err {
        EngineError::Jupiter(inner) => classify_jupiter(inner),
        EngineError::Dflow(inner) => classify_dflow(inner),
        EngineError::Kamino(inner) => classify_kamino(inner),
        EngineError::Ultra(inner) => classify_ultra(inner),
        EngineError::Network(inner) => classify_reqwest(inner),
        EngineError::Rpc(_) => Some(IpLeaseOutcome::NetworkError),
        EngineError::NetworkResource(_) => Some(IpLeaseOutcome::NetworkError),
        _ => None,
    }
}

pub(crate) fn classify_jupiter(
    err: &crate::jupiter::error::JupiterError,
) -> Option<IpLeaseOutcome> {
    use crate::jupiter::error::JupiterError;
    match err {
        JupiterError::ApiStatus { status, .. } | JupiterError::DownloadStatus { status, .. } => {
            map_status(status)
        }
        JupiterError::Http(inner) | JupiterError::DownloadFailed { source: inner, .. } => {
            classify_reqwest(inner)
        }
        JupiterError::HealthCheck(_) => Some(IpLeaseOutcome::NetworkError),
        _ => None,
    }
}

pub(crate) fn classify_dflow(err: &crate::api::dflow::DflowError) -> Option<IpLeaseOutcome> {
    use crate::api::dflow::DflowError;
    match err {
        DflowError::RateLimited { .. } => Some(IpLeaseOutcome::RateLimited),
        DflowError::ApiStatus { status, .. } => map_status(status),
        DflowError::Http(inner) => classify_reqwest(inner),
        _ => None,
    }
}

pub(crate) fn classify_kamino(err: &crate::api::kamino::KaminoError) -> Option<IpLeaseOutcome> {
    use crate::api::kamino::KaminoError;
    match err {
        KaminoError::RateLimited { .. } => Some(IpLeaseOutcome::RateLimited),
        KaminoError::ApiStatus { status, .. } => map_status(status),
        KaminoError::Http(inner) => classify_reqwest(inner),
        _ => None,
    }
}

pub(crate) fn classify_ultra(err: &crate::api::ultra::UltraError) -> Option<IpLeaseOutcome> {
    use crate::api::ultra::UltraError;
    match err {
        UltraError::ApiStatus { status, .. } => map_status(status),
        UltraError::Http(inner) => classify_reqwest(inner),
        _ => None,
    }
}

fn classify_reqwest(err: &reqwest::Error) -> Option<IpLeaseOutcome> {
    if err.is_timeout() {
        return Some(IpLeaseOutcome::Timeout);
    }
    if let Some(status) = err.status() {
        return map_status(&status);
    }
    if err.is_connect() || err.is_request() {
        return Some(IpLeaseOutcome::NetworkError);
    }
    None
}

fn map_status(status: &StatusCode) -> Option<IpLeaseOutcome> {
    if *status == StatusCode::TOO_MANY_REQUESTS {
        return Some(IpLeaseOutcome::RateLimited);
    }
    if *status == StatusCode::REQUEST_TIMEOUT || *status == StatusCode::GATEWAY_TIMEOUT {
        return Some(IpLeaseOutcome::Timeout);
    }
    if status.is_server_error() {
        return Some(IpLeaseOutcome::NetworkError);
    }
    None
}
