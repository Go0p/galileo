use std::convert::TryFrom;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use futures::{StreamExt, stream::FuturesUnordered};
use reqwest::StatusCode;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::sleep;
use tracing::trace;

use super::quote_cadence::CadenceTimings;
use crate::config::QuoteParallelism;
use crate::engine::error::EngineError;
use crate::engine::quote::QuoteExecutor;
use crate::engine::types::{DoubleQuote, QuoteTask};
use crate::engine::{EngineResult, QuoteCadence};
use crate::monitoring::events;
use crate::network::{IpAllocator, IpLeaseMode, IpLeaseOutcome, IpTaskKind};

use super::context::QuoteBatchPlan;
use super::quote::QuoteConfig;

#[derive(Clone)]
pub struct QuoteDispatcher {
    ip_allocator: Arc<IpAllocator>,
    cadence: QuoteCadence,
}

pub struct QuoteDispatchOutcome {
    pub batch: QuoteBatchPlan,
    pub quote: Option<DoubleQuote>,
    pub forward_ip: Option<IpAddr>,
    pub reverse_ip: Option<IpAddr>,
    pub forward_duration: Option<Duration>,
    pub reverse_duration: Option<Duration>,
}

impl QuoteDispatcher {
    pub fn new(ip_allocator: Arc<IpAllocator>, cadence: QuoteCadence) -> Self {
        Self {
            ip_allocator,
            cadence,
        }
    }

    pub fn plan(&self, batches: Vec<QuoteBatchPlan>) -> Vec<QuoteBatchPlan> {
        let planned = batches.len();
        let parallelism = self.effective_parallelism(planned, &batches);
        let timings = self.cadence.default_timings();
        let override_parallelism = self.parallelism_override(&batches);
        trace!(
            target: "engine::dispatcher",
            planned,
            parallelism,
            total_slots = self.ip_allocator.total_slots(),
            per_ip_limit = ?self.ip_allocator.per_ip_inflight_limit(),
            spacing_ms = timings.intra_group_spacing.as_millis(),
            wave_cooldown_ms = timings.wave_cooldown.as_millis(),
            override_parallelism,
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
        let parallelism = self.effective_parallelism(planned, &batches);
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
                                Option<IpAddr>,
                                Option<Duration>,
                                Option<Duration>,
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
            let delay = dispatcher.compute_dispatch_delay(&batch, index);

            futures.push(Box::pin(async move {
                if !delay.is_zero() {
                    sleep(delay).await;
                }

                let permit = match semaphore {
                    Some(ref sem) => match acquire_permit(sem).await {
                        Ok(permit) => Some(permit),
                        Err(err) => return (batch, None, None, None, None, Err(err)),
                    },
                    None => None,
                };

                let forward = dispatcher
                    .ip_allocator
                    .acquire_handle_excluding(IpTaskKind::QuoteBuy, IpLeaseMode::Ephemeral, None)
                    .await;
                let (forward_handle_raw, forward_ip_addr) = match forward {
                    Ok(value) => value,
                    Err(err) => {
                        if let Some(permit) = permit {
                            drop(permit);
                        }
                        return (
                            batch,
                            None,
                            None,
                            None,
                            None,
                            Err(EngineError::NetworkResource(err)),
                        );
                    }
                };
                let forward_ip = Some(forward_ip_addr);
                let mut forward_handle = Some(forward_handle_raw);
                let task = QuoteTask::new(batch.pair.clone(), batch.amount);
                events::quote_start(strategy.as_str(), &task, Some(batch.batch_id), forward_ip);
                let started = Instant::now();

                let forward_start = Instant::now();
                let forward_result = executor
                    .quote_once(
                        &task.pair,
                        task.amount,
                        &config,
                        forward_handle
                            .as_ref()
                            .expect("forward handle available for initial quote"),
                    )
                    .await;
                let forward_duration = Some(forward_start.elapsed());
                let mut reverse_ip: Option<IpAddr> = None;
                let mut reverse_duration: Option<Duration> = None;

                let result: Result<Option<DoubleQuote>, EngineError> = match forward_result {
                    Err(err) => Err(err),
                    Ok(None) => Ok(None),
                    Ok(Some(forward_quote)) => {
                        match crate::engine::quote::second_leg_amount(&task, &forward_quote) {
                            None => {
                                let _ = forward_handle.take();
                                Ok(None)
                            }
                            Some(second_amount) => {
                                let _ = forward_handle.take();

                                let (reverse_handle_raw, ip) = match dispatcher
                                    .ip_allocator
                                    .acquire_handle_excluding(
                                        IpTaskKind::QuoteSell,
                                        IpLeaseMode::Ephemeral,
                                        forward_ip,
                                    )
                                    .await
                                {
                                    Ok(value) => value,
                                    Err(err) => {
                                        if let Some(permit) = permit {
                                            drop(permit);
                                        }
                                        return (
                                            batch,
                                            forward_ip,
                                            None,
                                            forward_duration,
                                            None,
                                            Err(EngineError::NetworkResource(err)),
                                        );
                                    }
                                };
                                reverse_ip = Some(ip);
                                let mut reverse_handle = Some(reverse_handle_raw);

                                let reverse_pair = task.pair.reversed();
                                let reverse_start = Instant::now();
                                let reverse_result = executor
                                    .quote_once(
                                        &reverse_pair,
                                        second_amount,
                                        &config,
                                        reverse_handle
                                            .as_ref()
                                            .expect("reverse handle available for double quote"),
                                    )
                                    .await;
                                reverse_duration = Some(reverse_start.elapsed());

                                match reverse_result {
                                    Err(err) => {
                                        if let Some(handle) = reverse_handle.take() {
                                            if let Some(outcome) = classify_ip_outcome(&err) {
                                                handle.mark_outcome(outcome);
                                            }
                                        }
                                        Err(err)
                                    }
                                    Ok(None) => {
                                        let _ = reverse_handle.take();
                                        Ok(None)
                                    }
                                    Ok(Some(reverse_quote)) => {
                                        let _ = reverse_handle.take();
                                        if crate::engine::quote::aggregator_kinds_match(
                                            &task,
                                            &forward_quote,
                                            &reverse_quote,
                                        ) {
                                            Ok(Some(DoubleQuote {
                                                forward: forward_quote,
                                                reverse: reverse_quote,
                                                forward_latency: forward_duration,
                                                reverse_latency: reverse_duration,
                                                forward_ip,
                                                reverse_ip,
                                            }))
                                        } else {
                                            Ok(None)
                                        }
                                    }
                                }
                            }
                        }
                    }
                };

                match &result {
                    Ok(Some(_)) => {
                        events::quote_end(
                            strategy.as_str(),
                            &task,
                            true,
                            started.elapsed(),
                            Some(batch.batch_id),
                            forward_ip,
                        );
                    }
                    Ok(None) => {
                        events::quote_end(
                            strategy.as_str(),
                            &task,
                            false,
                            started.elapsed(),
                            Some(batch.batch_id),
                            forward_ip,
                        );
                    }
                    Err(err) => {
                        events::quote_end(
                            strategy.as_str(),
                            &task,
                            false,
                            started.elapsed(),
                            Some(batch.batch_id),
                            forward_ip,
                        );
                        if let Some(handle) = forward_handle.take() {
                            if let Some(outcome) = classify_ip_outcome(err) {
                                handle.mark_outcome(outcome);
                            }
                        }
                    }
                }

                let _ = forward_handle.take();

                if let Some(permit) = permit {
                    drop(permit);
                }

                (
                    batch,
                    forward_ip,
                    reverse_ip,
                    forward_duration,
                    reverse_duration,
                    result,
                )
            }));
        }

        let mut outcomes = Vec::new();
        let mut first_error: Option<EngineError> = None;

        while let Some((
            batch,
            forward_ip,
            reverse_ip,
            forward_duration,
            reverse_duration,
            result,
        )) = futures.next().await
        {
            match result {
                Ok(quote) => outcomes.push(QuoteDispatchOutcome {
                    batch,
                    quote,
                    forward_ip,
                    reverse_ip,
                    forward_duration,
                    reverse_duration,
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

    fn effective_parallelism(&self, planned: usize, batches: &[QuoteBatchPlan]) -> usize {
        if planned <= 1 {
            return planned.max(1);
        }

        if let Some(value) = self.parallelism_override(batches) {
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

    fn compute_dispatch_delay(&self, batch: &QuoteBatchPlan, index: usize) -> Duration {
        let timings = self.timings_for_batch(batch);
        let spacing = timings.intra_group_spacing;
        if spacing.is_zero() {
            return Duration::ZERO;
        }

        let multiplier = u32::try_from(index).unwrap_or(u32::MAX);
        spacing.saturating_mul(multiplier)
    }

    fn parallelism_override(&self, batches: &[QuoteBatchPlan]) -> Option<u16> {
        let mut min_value: Option<u16> = None;
        for batch in batches {
            let timings = self.timings_for_batch(batch);
            if let QuoteParallelism::Fixed(value) = timings.group_parallelism {
                let value = value.max(1);
                min_value = Some(match min_value {
                    Some(current) => current.min(value),
                    None => value,
                });
            }
        }
        min_value
    }

    fn timings_for_batch(&self, batch: &QuoteBatchPlan) -> CadenceTimings {
        self.cadence.timings_for_base_mint(&batch.pair.input_mint)
    }
}

async fn acquire_permit(semaphore: &Arc<Semaphore>) -> EngineResult<OwnedSemaphorePermit> {
    semaphore.clone().acquire_owned().await.map_err(|_| {
        EngineError::InvalidConfig("QuoteDispatcher semaphore unexpectedly closed".into())
    })
}

pub(crate) fn classify_ip_outcome(err: &EngineError) -> Option<IpLeaseOutcome> {
    match err {
        EngineError::Dflow(inner) => classify_dflow(inner),
        EngineError::Kamino(inner) => classify_kamino(inner),
        EngineError::Ultra(inner) => classify_ultra(inner),
        EngineError::Network(inner) => classify_reqwest(inner),
        EngineError::Rpc(_) => Some(IpLeaseOutcome::NetworkError),
        EngineError::NetworkResource(_) => Some(IpLeaseOutcome::NetworkError),
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
        KaminoError::Timeout { .. } => Some(IpLeaseOutcome::NetworkError),
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
