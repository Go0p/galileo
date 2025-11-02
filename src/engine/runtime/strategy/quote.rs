use std::sync::Arc;
use std::time::{Duration, Instant};

use tracing::{debug, trace};

use crate::engine::context::QuoteBatchPlan;
use crate::engine::quote::{aggregator_kinds_match, second_leg_amount};
use crate::engine::quote_dispatcher;
use crate::engine::types::{DoubleQuote, SwapOpportunity};
use crate::engine::{EngineError, EngineResult};
use crate::monitoring::events;
use crate::network::{IpLeaseMode, IpTaskKind};
use crate::strategy::{Strategy, StrategyEvent};

use super::StrategyEngine;

impl<S> StrategyEngine<S>
where
    S: Strategy<Event = StrategyEvent>,
{
    pub(super) async fn run_quote_batches(
        &mut self,
        batches: Vec<QuoteBatchPlan>,
    ) -> EngineResult<()> {
        if batches.is_empty() {
            return Ok(());
        }

        let planned_batches = self.quote_dispatcher.plan(batches);

        if self.multi_leg.is_some() {
            for batch in planned_batches {
                self.process_quote_batch_legacy(batch).await?;
            }
            return Ok(());
        }

        let strategy_label = Arc::new(self.strategy.name().to_string());
        let outcomes = self
            .quote_dispatcher
            .dispatch(
                planned_batches,
                self.quote_executor.clone(),
                self.settings.quote.clone(),
                Arc::clone(&strategy_label),
            )
            .await?;

        for outcome in outcomes {
            self.process_quote_outcome(
                outcome.batch,
                outcome.quote,
                outcome.forward_ip,
                outcome.reverse_ip,
                outcome.forward_duration,
                outcome.reverse_duration,
            )
            .await?;
        }

        Ok(())
    }

    pub(super) async fn process_quote_batch_legacy(
        &mut self,
        batch: QuoteBatchPlan,
    ) -> EngineResult<()> {
        let QuoteBatchPlan {
            batch_id,
            pair,
            amount,
        } = batch;

        trace!(
            target: "engine::quote",
            batch_id,
            amount,
            "处理报价批次（legacy）"
        );

        let task = crate::engine::types::QuoteTask::new(pair, amount);
        self.process_task(task, Some(batch_id)).await
    }

    pub(super) async fn process_task(
        &mut self,
        task: crate::engine::types::QuoteTask,
        batch_id: Option<u64>,
    ) -> EngineResult<()> {
        if let Some(mut context) = self.multi_leg.take() {
            let result = self.process_multi_leg_task(&mut context, task).await;
            self.multi_leg = Some(context);
            return result;
        }

        let (forward_handle_raw, forward_ip_addr) = self
            .ip_allocator
            .acquire_handle_excluding(IpTaskKind::QuoteBuy, IpLeaseMode::Ephemeral, None)
            .await
            .map_err(EngineError::NetworkResource)?;
        let forward_ip = Some(forward_ip_addr);
        let mut forward_handle = Some(forward_handle_raw);
        let strategy_name = self.strategy.name();
        events::quote_start(strategy_name, &task, batch_id, forward_ip);
        let quote_started = Instant::now();
        let forward_start = Instant::now();
        let forward_result = self
            .quote_executor
            .quote_once(
                &task.pair,
                task.amount,
                &self.settings.quote,
                forward_handle
                    .as_ref()
                    .expect("forward handle available for first quote"),
            )
            .await;
        let forward_duration = forward_start.elapsed();

        let forward_quote = match forward_result {
            Err(err) => {
                events::quote_end(
                    strategy_name,
                    &task,
                    false,
                    quote_started.elapsed(),
                    batch_id,
                    forward_ip,
                );
                if let Some(handle) = forward_handle.take() {
                    if let Some(outcome) = quote_dispatcher::classify_ip_outcome(&err) {
                        handle.mark_outcome(outcome);
                    }
                }
                return Err(err);
            }
            Ok(None) => {
                events::quote_end(
                    strategy_name,
                    &task,
                    false,
                    quote_started.elapsed(),
                    batch_id,
                    forward_ip,
                );
                let _ = forward_handle.take();
                return Ok(());
            }
            Ok(Some(value)) => value,
        };

        let Some(second_amount) = second_leg_amount(&task, &forward_quote) else {
            events::quote_end(
                strategy_name,
                &task,
                false,
                quote_started.elapsed(),
                batch_id,
                forward_ip,
            );
            let _ = forward_handle.take();
            return Ok(());
        };

        let _ = forward_handle.take();

        let (reverse_handle_raw, reverse_ip_addr) = match self
            .ip_allocator
            .acquire_handle_excluding(IpTaskKind::QuoteSell, IpLeaseMode::Ephemeral, forward_ip)
            .await
        {
            Ok(value) => value,
            Err(err) => {
                events::quote_end(
                    strategy_name,
                    &task,
                    false,
                    quote_started.elapsed(),
                    batch_id,
                    forward_ip,
                );
                return Err(EngineError::NetworkResource(err));
            }
        };
        let mut reverse_handle = Some(reverse_handle_raw);
        let reverse_ip = Some(reverse_ip_addr);

        let reverse_pair = task.pair.reversed();
        let reverse_start = Instant::now();
        let reverse_result = self
            .quote_executor
            .quote_once(
                &reverse_pair,
                second_amount,
                &self.settings.quote,
                reverse_handle
                    .as_ref()
                    .expect("reverse handle available for second quote"),
            )
            .await;
        let reverse_duration = Some(reverse_start.elapsed());

        let reverse_quote = match reverse_result {
            Err(err) => {
                events::quote_end(
                    strategy_name,
                    &task,
                    false,
                    quote_started.elapsed(),
                    batch_id,
                    forward_ip,
                );
                if let Some(handle) = reverse_handle.take() {
                    if let Some(outcome) = quote_dispatcher::classify_ip_outcome(&err) {
                        handle.mark_outcome(outcome);
                    }
                }
                return Err(err);
            }
            Ok(None) => {
                events::quote_end(
                    strategy_name,
                    &task,
                    false,
                    quote_started.elapsed(),
                    batch_id,
                    forward_ip,
                );
                let _ = reverse_handle.take();
                return Ok(());
            }
            Ok(Some(value)) => value,
        };

        let _ = reverse_handle.take();

        if !aggregator_kinds_match(&task, &forward_quote, &reverse_quote) {
            events::quote_end(
                strategy_name,
                &task,
                false,
                quote_started.elapsed(),
                batch_id,
                forward_ip,
            );
            return Ok(());
        }

        let double_quote = DoubleQuote {
            forward: forward_quote,
            reverse: reverse_quote,
            forward_latency: Some(forward_duration),
            reverse_latency: reverse_duration,
        };
        events::quote_end(
            strategy_name,
            &task,
            true,
            quote_started.elapsed(),
            batch_id,
            forward_ip,
        );

        self.process_quote_outcome(
            QuoteBatchPlan {
                batch_id: batch_id.unwrap_or_else(|| self.next_batch_id.wrapping_sub(1)),
                pair: task.pair.clone(),
                amount: task.amount,
            },
            Some(double_quote),
            forward_ip,
            reverse_ip,
            Some(forward_duration),
            reverse_duration,
        )
        .await
    }

    pub(super) async fn process_quote_outcome(
        &mut self,
        batch: QuoteBatchPlan,
        quote: Option<DoubleQuote>,
        forward_ip: Option<std::net::IpAddr>,
        reverse_ip: Option<std::net::IpAddr>,
        forward_duration: Option<Duration>,
        reverse_duration: Option<Duration>,
    ) -> EngineResult<()> {
        let QuoteBatchPlan {
            batch_id,
            pair,
            amount,
        } = batch;

        trace!(
            target: "engine::quote",
            batch_id,
            amount,
            has_quote = quote.is_some(),
            forward_ip = ?forward_ip,
            reverse_ip = ?reverse_ip,
            "处理并发报价结果",
        );

        let Some(double_quote) = quote else {
            return Ok(());
        };

        let task = crate::engine::types::QuoteTask::new(pair.clone(), amount);
        let forward_out = double_quote.forward.out_amount();
        let reverse_out = double_quote.reverse.out_amount();
        let aggregator = format!("{:?}", double_quote.forward.kind());
        events::quote_round_trip(
            self.strategy.name(),
            &task,
            aggregator.as_str(),
            forward_out,
            reverse_out,
            Some(batch_id),
            forward_ip,
        );

        let Some(opportunity) = self
            .profit_evaluator
            .evaluate(task.amount, &double_quote, &pair)
        else {
            return Ok(());
        };

        self.log_opportunity_discovery(
            &task,
            &opportunity,
            forward_duration,
            reverse_duration,
            forward_ip,
            reverse_ip,
        );
        events::profit_detected(self.strategy.name(), &opportunity);

        let plan = crate::engine::types::ExecutionPlan::with_deadline(
            opportunity,
            self.settings.landing_timeout,
        );
        self.execute_plan(plan).await
    }

    fn log_opportunity_discovery(
        &self,
        task: &crate::engine::types::QuoteTask,
        opportunity: &SwapOpportunity,
        forward_duration: Option<Duration>,
        reverse_duration: Option<Duration>,
        forward_ip: Option<std::net::IpAddr>,
        reverse_ip: Option<std::net::IpAddr>,
    ) {
        let forward_ms = forward_duration
            .map(|d| format!("{:.3}", d.as_secs_f64() * 1_000.0))
            .unwrap_or_else(|| "-".to_string());
        let reverse_ms = reverse_duration
            .map(|d| format!("{:.3}", d.as_secs_f64() * 1_000.0))
            .unwrap_or_else(|| "-".to_string());
        let ip_summary = match (forward_ip, reverse_ip) {
            (Some(f), Some(r)) if f == r => f.to_string(),
            (Some(f), Some(r)) => format!("{},{}", f, r),
            (Some(f), None) => f.to_string(),
            (None, Some(r)) => r.to_string(),
            _ => "-".to_string(),
        };

        debug!(
            target: "engine::opportunity",
            "本次机会 base_mint={} amount_in={} forward_ms={} reverse_ms={} profit={} net_profit={} ip={}",
            task.pair.input_mint,
            opportunity.amount_in,
            forward_ms,
            reverse_ms,
            opportunity.profit_lamports,
            opportunity.net_profit(),
            ip_summary,
        );
    }
}
