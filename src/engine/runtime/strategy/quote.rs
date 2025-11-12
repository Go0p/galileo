use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tracing::{debug, info, trace};

use crate::engine::EngineResult;
use crate::engine::context::QuoteBatchPlan;
use crate::engine::quote_dispatcher;
use crate::engine::types::{DoubleQuote, QuoteTask, SwapOpportunity};
use crate::monitoring::events;
use crate::monitoring::format::short_mint_str;
use crate::strategy::{Strategy, StrategyEvent};

use super::StrategyEngine;
use super::multi_leg::{MultiLegBatchHandler, MultiLegDispatchResult};

#[derive(Default)]
struct BatchStats {
    total_groups: u64,
    successful_groups: u64,
    forward_latency_total: Duration,
    forward_latency_count: u64,
    reverse_latency_total: Duration,
    reverse_latency_count: u64,
    opportunities: BTreeMap<String, OpportunityStats>,
    executed_trades: u64,
    attempted_trades: u64,
}

#[derive(Default)]
struct OpportunityStats {
    total_profit: i128,
    count: u64,
}

pub(super) struct OpportunityExecution {
    base_mint: String,
    net_profit: i128,
    attempted_execution: bool,
    executed: bool,
}

impl BatchStats {
    fn new(total_groups: u64) -> Self {
        Self {
            total_groups,
            ..Self::default()
        }
    }

    fn record_group_outcome(
        &mut self,
        successful: bool,
        forward_duration: Option<Duration>,
        reverse_duration: Option<Duration>,
    ) {
        if successful {
            self.successful_groups = self.successful_groups.saturating_add(1);
        }
        if let Some(duration) = forward_duration {
            self.forward_latency_total += duration;
            self.forward_latency_count = self.forward_latency_count.saturating_add(1);
        }
        if let Some(duration) = reverse_duration {
            self.reverse_latency_total += duration;
            self.reverse_latency_count = self.reverse_latency_count.saturating_add(1);
        }
    }

    fn record_opportunity(&mut self, base_mint: &str, net_profit: i128) {
        let entry = self
            .opportunities
            .entry(base_mint.to_string())
            .or_insert_with(OpportunityStats::default);
        entry.total_profit += net_profit;
        entry.count = entry.count.saturating_add(1);
    }

    fn record_execution(&mut self, executed: bool) {
        self.attempted_trades = self.attempted_trades.saturating_add(1);
        if executed {
            self.executed_trades = self.executed_trades.saturating_add(1);
        }
    }

    fn summary_line(&self) -> String {
        let opportunity_counts = self.format_opportunity_counts();
        let latency = self.format_latencies();
        let profit = self.format_opportunity_profit();
        let quote_groups = format!("{}/{}", self.successful_groups, self.total_groups);
        let execution = format!("{}/{}", self.executed_trades, self.attempted_trades);

        format!(
            "机会数: {} | 平均延迟: {} | 平均利润: {} | Quote组: {} | 成功发送数: {}",
            opportunity_counts, latency, profit, quote_groups, execution
        )
    }

    fn format_opportunity_counts(&self) -> String {
        if self.opportunities.is_empty() {
            return "-".to_string();
        }
        let mut parts = Vec::with_capacity(self.opportunities.len());
        for (mint, stats) in &self.opportunities {
            let symbol = short_mint_str(mint);
            parts.push(format!("{}/{}", stats.count, symbol.as_ref()));
        }
        parts.join(",")
    }

    fn format_opportunity_profit(&self) -> String {
        if self.opportunities.is_empty() {
            return "-".to_string();
        }
        let mut parts = Vec::with_capacity(self.opportunities.len());
        for (mint, stats) in &self.opportunities {
            let avg = if stats.count == 0 {
                0
            } else {
                (stats.total_profit / stats.count as i128) as i64
            };
            let symbol = short_mint_str(mint);
            parts.push(format!("{}/{}", avg, symbol.as_ref()));
        }
        parts.join(",")
    }

    fn format_latencies(&self) -> String {
        let forward = if self.forward_latency_count == 0 {
            "-".to_string()
        } else {
            let avg = self.forward_latency_total.as_secs_f64() * 1_000.0
                / self.forward_latency_count as f64;
            format!("{:.0}ms", avg.round())
        };
        let reverse = if self.reverse_latency_count == 0 {
            "-".to_string()
        } else {
            let avg = self.reverse_latency_total.as_secs_f64() * 1_000.0
                / self.reverse_latency_count as f64;
            format!("{:.0}ms", avg.round())
        };

        format!("{}/{}", forward, reverse)
    }
}

#[cfg(test)]
mod tests {
    use super::BatchStats;
    use std::time::Duration;

    #[test]
    fn summary_line_formats_expected_content() {
        let mut stats = BatchStats::new(2);

        stats.record_group_outcome(
            true,
            Some(Duration::from_millis(80)),
            Some(Duration::from_millis(120)),
        );
        stats.record_group_outcome(false, None, None);

        stats.record_opportunity("WSOL", 1_500);
        stats.record_opportunity("USDC", 500);
        stats.record_opportunity("USDC", 1_500);

        stats.record_execution(true);
        stats.record_execution(false);

        assert_eq!(
            stats.summary_line(),
            "机会数: 2/USDC,1/WSOL | 平均延迟: 80ms/120ms | 平均利润: 1000/USDC,1500/WSOL | Quote组: 1/2 | 成功发送数: 1/2"
        );
    }

    #[test]
    fn summary_line_handles_empty_stats() {
        let stats = BatchStats::new(0);
        assert_eq!(
            stats.summary_line(),
            "机会数: - | 平均延迟: -/- | 平均利润: - | Quote组: 0/0 | 成功发送数: 0/0"
        );
    }
}

impl<S> StrategyEngine<S>
where
    S: Strategy<Event = StrategyEvent>,
{
    pub(super) async fn run_quote_batches(
        &mut self,
        batches: Vec<QuoteBatchPlan>,
    ) -> EngineResult<Duration> {
        if batches.is_empty() {
            return Ok(Duration::ZERO);
        }

        let planned_batches = self.quote_dispatcher.plan(batches);
        let total_groups = planned_batches.len() as u64;
        let mut max_cycle_cooldown = Duration::ZERO;
        for batch in &planned_batches {
            let timings = self
                .settings
                .quote_cadence
                .timings_for_base_mint(&batch.pair.input_mint);
            if timings.cycle_cooldown > max_cycle_cooldown {
                max_cycle_cooldown = timings.cycle_cooldown;
            }
        }

        let should_summarize = self.settings.console_summary.enable && self.multi_leg.is_none();
        let mut batch_stats = if should_summarize {
            Some(BatchStats::new(total_groups))
        } else {
            None
        };

        if let Some(context) = self.multi_leg.clone() {
            let compute_unit_price = if self.landers.has_jito() && !self.landers.has_non_jito() {
                None
            } else {
                self.settings.sample_compute_unit_price()
            };
            let handler = Arc::new(MultiLegBatchHandler::new(
                (*context.as_ref()).clone(),
                self.identity.pubkey,
                compute_unit_price,
                self.settings.quote.dex_whitelist.clone(),
                self.settings.quote.dex_blacklist.clone(),
                self.titan_event_rx.is_some(),
            ));
            let outcomes: Vec<MultiLegDispatchResult> = self
                .quote_dispatcher
                .dispatch_custom(planned_batches, handler)
                .await?;
            for outcome in outcomes {
                let task = QuoteTask::new(outcome.batch.pair.clone(), outcome.batch.amount);
                self.handle_multi_leg_batch(context.as_ref(), task, outcome.result)
                    .await?;
            }
            return Ok(max_cycle_cooldown);
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
            let quote_present = outcome.quote.is_some();
            if let Some(stats) = batch_stats.as_mut() {
                stats.record_group_outcome(
                    quote_present,
                    outcome.forward_duration,
                    outcome.reverse_duration,
                );
            }

            let quote_dispatcher::QuoteDispatchOutcome {
                batch,
                quote,
                forward_ip,
                reverse_ip,
                forward_duration,
                reverse_duration,
            } = outcome;

            let opportunity = self
                .process_quote_outcome(
                    batch,
                    quote,
                    forward_ip,
                    reverse_ip,
                    forward_duration,
                    reverse_duration,
                )
                .await?;

            if let (Some(stats), Some(op)) = (batch_stats.as_mut(), opportunity) {
                stats.record_opportunity(&op.base_mint, op.net_profit);
                if op.attempted_execution {
                    stats.record_execution(op.executed);
                }
            }
        }

        if let Some(stats) = batch_stats {
            let line = stats.summary_line();
            info!(target: "engine::summary", "{line}");
        }

        Ok(max_cycle_cooldown)
    }

    pub(super) async fn process_quote_outcome(
        &mut self,
        batch: QuoteBatchPlan,
        quote: Option<DoubleQuote>,
        forward_ip: Option<std::net::IpAddr>,
        reverse_ip: Option<std::net::IpAddr>,
        forward_duration: Option<Duration>,
        reverse_duration: Option<Duration>,
    ) -> EngineResult<Option<OpportunityExecution>> {
        let QuoteBatchPlan {
            batch_id,
            pair,
            amount,
            preferred_ip: _,
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
            return Ok(None);
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
            return Ok(None);
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

        let net_profit = opportunity.net_profit();
        let base_mint = pair.input_mint.clone();
        let deadline = Instant::now() + self.settings.landing_timeout;

        match self.execute_plan(opportunity, deadline).await {
            Ok(()) => Ok(Some(OpportunityExecution {
                base_mint,
                net_profit,
                attempted_execution: true,
                executed: true,
            })),
            Err(err) => Err(err),
        }
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
