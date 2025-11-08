use std::collections::VecDeque;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tracing::trace;

use super::context::QuoteBatchPlan;
use super::quote::{QuoteConfig, aggregator_kinds_match, second_leg_amount};
use crate::engine::quote::QuoteExecutor;
use crate::engine::quote_cadence::QuoteCadence;
use crate::engine::types::{DoubleQuote, QuoteTask};
use crate::engine::{EngineError, EngineResult};
use crate::monitoring::events;
use crate::network::{IpAllocator, IpLeaseMode, IpLeaseOutcome, IpTaskKind};

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

#[async_trait]
pub trait DispatchTaskHandler<R>: Send + Sync {
    async fn run(&self, batch: QuoteBatchPlan) -> EngineResult<R>;
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
        let slot_limit = self.resolve_slot_limit(&batches);
        let default_timings = self.cadence.default_timings();
        trace!(
            target: "engine::dispatcher",
            planned,
            slot_limit,
            inter_delay_ms = default_timings.inter_batch_delay.as_millis(),
            cycle_cooldown_ms = default_timings.cycle_cooldown.as_millis(),
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

        let slot_limit = self.resolve_slot_limit(&batches);
        let queue = Arc::new(Mutex::new(VecDeque::from(batches)));
        let outcomes = Arc::new(Mutex::new(Vec::new()));
        let first_error = Arc::new(Mutex::new(None));
        let cadence = self.cadence.clone();
        let dispatcher = self.clone();

        let mut join_set = JoinSet::new();
        for _ in 0..slot_limit {
            let queue = Arc::clone(&queue);
            let outcomes = Arc::clone(&outcomes);
            let first_error = Arc::clone(&first_error);
            let dispatcher = dispatcher.clone();
            let cadence = cadence.clone();
            let executor = quote_executor.clone();
            let config = quote_config.clone();
            let strategy = Arc::clone(&strategy_label);

            join_set.spawn(async move {
                loop {
                    let batch = {
                        let mut guard = queue.lock().await;
                        guard.pop_front()
                    };

                    let Some(batch) = batch else {
                        break;
                    };

                    let timings = cadence.timings_for_base_mint(&batch.pair.input_mint);
                    let result = dispatcher
                        .execute_single_batch(
                            batch.clone(),
                            executor.clone(),
                            config.clone(),
                            Arc::clone(&strategy),
                        )
                        .await;

                    match result {
                        Ok(outcome) => {
                            let mut guard = outcomes.lock().await;
                            guard.push((outcome.batch.batch_id, outcome));
                        }
                        Err(err) => {
                            let mut guard = first_error.lock().await;
                            if guard.is_none() {
                                *guard = Some(err);
                            }
                        }
                    }

                    if timings.inter_batch_delay > Duration::ZERO {
                        sleep(timings.inter_batch_delay).await;
                    }
                }
            });
        }

        while let Some(res) = join_set.join_next().await {
            if let Err(join_err) = res {
                let mut guard = first_error.lock().await;
                if guard.is_none() {
                    *guard = Some(EngineError::Internal(join_err.to_string()));
                }
            }
        }

        let mut collected = outcomes.lock().await;
        collected.sort_by_key(|(batch_id, _)| *batch_id);
        let ordered = collected
            .drain(..)
            .map(|(_, outcome)| outcome)
            .collect::<Vec<_>>();

        if let Some(err) = first_error.lock().await.take() {
            return Err(err);
        }

        Ok(ordered)
    }

    pub async fn dispatch_custom<R>(
        &self,
        batches: Vec<QuoteBatchPlan>,
        handler: Arc<dyn DispatchTaskHandler<R>>,
    ) -> EngineResult<Vec<R>>
    where
        R: Send + 'static,
    {
        if batches.is_empty() {
            return Ok(Vec::new());
        }

        let slot_limit = self.resolve_slot_limit(&batches);
        let queue = Arc::new(Mutex::new(VecDeque::from(batches)));
        let outcomes = Arc::new(Mutex::new(Vec::new()));
        let first_error = Arc::new(Mutex::new(None));
        let cadence = self.cadence.clone();

        let mut join_set = JoinSet::new();
        for _ in 0..slot_limit {
            let queue = Arc::clone(&queue);
            let outcomes = Arc::clone(&outcomes);
            let first_error = Arc::clone(&first_error);
            let handler = Arc::clone(&handler);
            let cadence = cadence.clone();

            join_set.spawn(async move {
                loop {
                    let batch = {
                        let mut guard = queue.lock().await;
                        guard.pop_front()
                    };

                    let Some(batch) = batch else {
                        break;
                    };

                    let timings = cadence.timings_for_base_mint(&batch.pair.input_mint);
                    match handler.run(batch.clone()).await {
                        Ok(value) => {
                            let mut guard = outcomes.lock().await;
                            guard.push((batch.batch_id, value));
                        }
                        Err(err) => {
                            let mut guard = first_error.lock().await;
                            if guard.is_none() {
                                *guard = Some(err);
                            }
                        }
                    }

                    if timings.inter_batch_delay > Duration::ZERO {
                        sleep(timings.inter_batch_delay).await;
                    }
                }
            });
        }

        while let Some(res) = join_set.join_next().await {
            if let Err(join_err) = res {
                let mut guard = first_error.lock().await;
                if guard.is_none() {
                    *guard = Some(EngineError::Internal(join_err.to_string()));
                }
            }
        }

        let mut collected = outcomes.lock().await;
        collected.sort_by_key(|(batch_id, _)| *batch_id);
        let ordered = collected
            .drain(..)
            .map(|(_, value)| value)
            .collect::<Vec<_>>();

        if let Some(err) = first_error.lock().await.take() {
            return Err(err);
        }

        Ok(ordered)
    }

    fn resolve_slot_limit(&self, batches: &[QuoteBatchPlan]) -> usize {
        if batches.is_empty() {
            return 1;
        }

        let default_slots = self
            .cadence
            .default_timings()
            .max_concurrent_slots
            .unwrap_or(1)
            .max(1);

        let mut limit = default_slots as usize;
        for batch in batches {
            let timings = self.cadence.timings_for_base_mint(&batch.pair.input_mint);
            if let Some(value) = timings.max_concurrent_slots {
                limit = limit.min(value.max(1) as usize);
            }
        }

        limit = limit.max(1);
        limit.min(batches.len())
    }

    async fn execute_single_batch(
        &self,
        batch: QuoteBatchPlan,
        quote_executor: QuoteExecutor,
        quote_config: QuoteConfig,
        strategy_label: Arc<String>,
    ) -> EngineResult<QuoteDispatchOutcome> {
        let (forward_handle_raw, forward_ip_addr) = match batch.preferred_ip {
            Some(ip) => match self
                .ip_allocator
                .acquire_handle_specific(IpTaskKind::QuoteBuy, IpLeaseMode::Ephemeral, ip)
                .await
            {
                Ok(handle) => (handle, ip),
                Err(err) => {
                    trace!(
                        target: "engine::dispatcher",
                        preferred_ip = %ip,
                        error = %err,
                        "preferred IP unavailable, falling back"
                    );
                    self.ip_allocator
                        .acquire_handle_excluding(
                            IpTaskKind::QuoteBuy,
                            IpLeaseMode::Ephemeral,
                            Some(ip),
                        )
                        .await
                        .map_err(EngineError::NetworkResource)?
                }
            },
            None => self
                .ip_allocator
                .acquire_handle_excluding(IpTaskKind::QuoteBuy, IpLeaseMode::Ephemeral, None)
                .await
                .map_err(EngineError::NetworkResource)?,
        };
        let forward_ip = Some(forward_ip_addr);
        let mut forward_handle = Some(forward_handle_raw);

        let task = QuoteTask::new(batch.pair.clone(), batch.amount);
        events::quote_start(
            strategy_label.as_str(),
            &task,
            Some(batch.batch_id),
            forward_ip,
        );
        let started = Instant::now();

        let forward_start = Instant::now();
        let forward_result = quote_executor
            .quote_once(
                &task.pair,
                task.amount,
                &quote_config,
                forward_handle
                    .as_ref()
                    .expect("forward handle available for initial quote"),
            )
            .await;
        let forward_duration = Some(forward_start.elapsed());

        let mut reverse_ip: Option<IpAddr> = None;
        let mut reverse_duration: Option<Duration> = None;

        let result = match forward_result {
            Err(err) => Err(err),
            Ok(None) => Ok(None),
            Ok(Some(forward_quote)) => match second_leg_amount(&task, &forward_quote) {
                None => {
                    let _ = forward_handle.take();
                    Ok(None)
                }
                Some(second_amount) => {
                    let _ = forward_handle.take();
                    let (reverse_handle_raw, ip) = self
                        .ip_allocator
                        .acquire_handle_excluding(
                            IpTaskKind::QuoteSell,
                            IpLeaseMode::Ephemeral,
                            forward_ip,
                        )
                        .await
                        .map_err(EngineError::NetworkResource)?;
                    reverse_ip = Some(ip);
                    let mut reverse_handle = Some(reverse_handle_raw);

                    let reverse_pair = task.pair.reversed();
                    let reverse_start = Instant::now();
                    let reverse_result = quote_executor
                        .quote_once(
                            &reverse_pair,
                            second_amount,
                            &quote_config,
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
                            if aggregator_kinds_match(&task, &forward_quote, &reverse_quote) {
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
            },
        };

        match &result {
            Ok(Some(_)) => {
                events::quote_end(
                    strategy_label.as_str(),
                    &task,
                    true,
                    started.elapsed(),
                    Some(batch.batch_id),
                    forward_ip,
                );
            }
            Ok(None) => {
                events::quote_end(
                    strategy_label.as_str(),
                    &task,
                    false,
                    started.elapsed(),
                    Some(batch.batch_id),
                    forward_ip,
                );
            }
            Err(err) => {
                events::quote_end(
                    strategy_label.as_str(),
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

        match result {
            Ok(quote) => Ok(QuoteDispatchOutcome {
                batch,
                quote,
                forward_ip,
                reverse_ip,
                forward_duration,
                reverse_duration,
            }),
            Err(err) => Err(err),
        }
    }
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

pub(crate) fn classify_dflow(err: &crate::api::dflow::DflowError) -> Option<IpLeaseOutcome> {
    use crate::api::dflow::DflowError;
    match err {
        DflowError::RateLimited { .. } => Some(IpLeaseOutcome::RateLimited),
        DflowError::ApiStatus { status, .. } => map_status(status),
        DflowError::Http(inner) => classify_reqwest(inner),
        _ => None,
    }
}

pub(crate) fn classify_jupiter(err: &crate::api::jupiter::JupiterError) -> Option<IpLeaseOutcome> {
    use crate::api::jupiter::JupiterError;
    match err {
        JupiterError::RateLimited { .. } => Some(IpLeaseOutcome::RateLimited),
        JupiterError::ApiStatus { status, .. } => map_status(status),
        JupiterError::Timeout { .. } => Some(IpLeaseOutcome::Timeout),
        JupiterError::Http(inner) => classify_reqwest(inner),
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
        if let Some(mapped) = map_status(&status) {
            return Some(mapped);
        }
    }
    if err.is_connect() || err.is_request() {
        return Some(IpLeaseOutcome::NetworkError);
    }
    None
}

fn map_status(status: &reqwest::StatusCode) -> Option<IpLeaseOutcome> {
    use reqwest::StatusCode;
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
