use std::time::Instant;

use crate::engine::context::QuoteBatchPlan;
use crate::engine::landing::ExecutionPlan;
use crate::engine::multi_leg::orchestrator::{LegPairDescriptor, LegPairPlan};
use crate::engine::multi_leg::runtime::{PairPlanBatchResult, PairPlanEvaluation, PairPlanRequest};
use crate::engine::multi_leg::types::{
    LegPlan, LegSide as MultiLegSide, QuoteIntent as MultiLegQuoteIntent,
};
use crate::engine::quote_dispatcher::DispatchTaskHandler;
use crate::engine::{
    EngineError, EngineResult, MultiLegInstructions, QuoteTask, SwapInstructionsVariant,
    SwapOpportunity,
};
use crate::instructions::compute_budget::{
    COMPUTE_BUDGET_PROGRAM_ID, compute_unit_limit_instruction, compute_unit_price_instruction,
};
use crate::monitoring::events;
use crate::strategy::types::TradePair;
use crate::strategy::{Strategy, StrategyEvent};
use async_trait::async_trait;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use tracing::{debug, warn};

use super::{BASE_TX_FEE_LAMPORTS, MultiLegEngineContext, StrategyEngine};
use crate::engine::FALLBACK_CU_LIMIT;

pub(super) struct MultiLegExecution {
    descriptor: LegPairDescriptor,
    pair: TradePair,
    trade_size: u64,
    plan: LegPairPlan,
    gross_profit: u64,
    tip_lamports: u64,
    tag: Option<String>,
}

impl MultiLegExecution {
    fn net_profit(&self) -> i128 {
        self.gross_profit as i128 - self.tip_lamports as i128
    }
}

pub(super) struct MultiLegDispatchResult {
    pub batch: QuoteBatchPlan,
    pub result: PairPlanBatchResult,
}

pub(super) struct MultiLegBatchHandler {
    context: MultiLegEngineContext,
    payer: Pubkey,
    compute_unit_price: Option<u64>,
    dex_whitelist: Vec<String>,
    dex_blacklist: Vec<String>,
}

impl MultiLegBatchHandler {
    pub(super) fn new(
        context: MultiLegEngineContext,
        payer: Pubkey,
        compute_unit_price: Option<u64>,
        dex_whitelist: Vec<String>,
        dex_blacklist: Vec<String>,
    ) -> Self {
        Self {
            context,
            payer,
            compute_unit_price,
            dex_whitelist,
            dex_blacklist,
        }
    }
}

#[async_trait]
impl DispatchTaskHandler<MultiLegDispatchResult> for MultiLegBatchHandler {
    async fn run(&self, batch: QuoteBatchPlan) -> EngineResult<MultiLegDispatchResult> {
        let combinations = self.context.combinations();
        if combinations.is_empty() {
            return Ok(MultiLegDispatchResult {
                batch,
                result: PairPlanBatchResult::default(),
            });
        }

        let mut buy_intent_template = MultiLegQuoteIntent::new(
            batch.pair.input_pubkey,
            batch.pair.output_pubkey,
            batch.amount,
            0,
        );
        let mut sell_intent_template = MultiLegQuoteIntent::new(
            batch.pair.output_pubkey,
            batch.pair.input_pubkey,
            batch.amount,
            0,
        );
        if !self.dex_whitelist.is_empty() {
            buy_intent_template.dex_whitelist = self.dex_whitelist.clone();
            sell_intent_template.dex_whitelist = self.dex_whitelist.clone();
        }
        if !self.dex_blacklist.is_empty() {
            buy_intent_template.dex_blacklist = self.dex_blacklist.clone();
            sell_intent_template.dex_blacklist = self.dex_blacklist.clone();
        }
        let tag_value = format!(
            "{}->{}/{}",
            batch.pair.input_mint, batch.pair.output_mint, batch.amount
        );

        let orchestrator = self.context.runtime().orchestrator();
        let mut requests = Vec::with_capacity(combinations.len());
        for combo in combinations {
            let Some(buy_descriptor) = orchestrator
                .descriptor(MultiLegSide::Buy, combo.buy_index)
                .cloned()
            else {
                continue;
            };
            let Some(sell_descriptor) = orchestrator
                .descriptor(MultiLegSide::Sell, combo.sell_index)
                .cloned()
            else {
                continue;
            };

            let buy_context =
                self.context
                    .build_context(&buy_descriptor, self.payer, self.compute_unit_price);
            let sell_context =
                self.context
                    .build_context(&sell_descriptor, self.payer, self.compute_unit_price);

            requests.push(PairPlanRequest {
                buy_index: combo.buy_index,
                sell_index: combo.sell_index,
                buy_intent: buy_intent_template.clone(),
                sell_intent: sell_intent_template.clone(),
                buy_context,
                sell_context,
                tag: Some(tag_value.clone()),
            });
        }

        if requests.is_empty() {
            return Ok(MultiLegDispatchResult {
                batch,
                result: PairPlanBatchResult::default(),
            });
        }

        let result = self
            .context
            .runtime()
            .plan_pair_batch_with_profit(requests)
            .await;

        Ok(MultiLegDispatchResult { batch, result })
    }
}

impl<S> StrategyEngine<S>
where
    S: Strategy<Event = StrategyEvent>,
{
    pub(super) async fn handle_multi_leg_batch(
        &mut self,
        ctx: &MultiLegEngineContext,
        task: QuoteTask,
        batch: PairPlanBatchResult,
    ) -> EngineResult<()> {
        let runtime = ctx.runtime();
        let best_profit = batch.best().map(|evaluation| evaluation.profit_lamports);

        let PairPlanBatchResult {
            successes,
            failures,
        } = batch;

        if let Some(profit) = best_profit {
            debug!(
                target: "engine::multi_leg",
                input_mint = %task.pair.input_mint,
                output_mint = %task.pair.output_mint,
                amount = task.amount,
                best_profit = profit,
                "多腿批次最佳收益"
            );
        }

        for failure in failures {
            warn!(
                target: "engine::multi_leg",
                input_mint = %task.pair.input_mint,
                output_mint = %task.pair.output_mint,
                amount = task.amount,
                trade_size = failure.trade_size,
                buy_index = failure.buy_index,
                sell_index = failure.sell_index,
                error = %failure.error,
                "多腿腿组合规划失败"
            );
        }

        let mut candidates: Vec<MultiLegExecution> = Vec::new();

        for evaluation in successes.into_iter() {
            let PairPlanEvaluation {
                descriptor,
                trade_size,
                tag,
                plan,
                profit_lamports,
            } = evaluation;

            if profit_lamports <= 0 {
                debug!(
                    target: "engine::multi_leg",
                    input_mint = %task.pair.input_mint,
                    output_mint = %task.pair.output_mint,
                    amount = task.amount,
                    profit = profit_lamports,
                    "多腿收益非正值，丢弃"
                );
                continue;
            }

            let aggregator_label = format!(
                "multi_leg:{}->{}",
                descriptor.buy.kind, descriptor.sell.kind
            );
            let forward_quote = plan.buy.quote.clone();
            let reverse_quote = plan.sell.quote.clone();
            let forward_latency_ms = forward_quote.latency_ms;
            let reverse_latency_ms = reverse_quote.latency_ms;
            let total_latency_ms = match (forward_latency_ms, reverse_latency_ms) {
                (Some(forward), Some(reverse)) => Some(forward + reverse),
                (Some(forward), None) => Some(forward),
                (None, Some(reverse)) => Some(reverse),
                (None, None) => None,
            };
            let combined_slippage_bps =
                u32::from(forward_quote.slippage_bps) + u32::from(reverse_quote.slippage_bps);
            let estimated_profit = profit_lamports.min(i128::from(u64::MAX)) as u64;
            let threshold = self.profit_evaluator.min_threshold();
            let Some(profit) = self.profit_evaluator.evaluate_multi_leg(profit_lamports) else {
                debug!(
                    target: "engine::multi_leg",
                    input_mint = %task.pair.input_mint,
                    output_mint = %task.pair.output_mint,
                    amount = trade_size,
                    profit = profit_lamports,
                    "多腿收益低于阈值，丢弃"
                );
                events::profit_shortfall(
                    task.pair.input_mint.as_str(),
                    aggregator_label.as_str(),
                    forward_quote.amount_in,
                    forward_quote.amount_out,
                    forward_latency_ms,
                    reverse_quote.amount_in,
                    reverse_quote.amount_out,
                    reverse_latency_ms,
                    estimated_profit,
                    threshold,
                );
                continue;
            };

            let requested_tip = plan
                .buy
                .requested_tip_lamports
                .unwrap_or(0)
                .max(plan.sell.requested_tip_lamports.unwrap_or(0));
            let tip_lamports = profit.tip_lamports.max(requested_tip);
            if requested_tip > profit.tip_lamports {
                debug!(
                    target: "engine::multi_leg",
                    input_mint = %task.pair.input_mint,
                    output_mint = %task.pair.output_mint,
                    amount = trade_size,
                    requested_tip,
                    calculated_tip = profit.tip_lamports,
                    "聚合器请求更高 tip，已按腿要求提升"
                );
            }

            debug!(
                target: "engine::multi_leg",
                input_mint = %task.pair.input_mint,
                output_mint = %task.pair.output_mint,
                amount = trade_size,
                gross_profit = profit_lamports,
                tip_lamports,
                slippage_bps = combined_slippage_bps,
                "多腿候选收益评估"
            );

            let candidate = MultiLegExecution {
                descriptor,
                pair: task.pair.clone(),
                trade_size,
                plan,
                gross_profit: profit.gross_profit_lamports,
                tip_lamports,
                tag,
            };

            if candidate.net_profit() <= 0 {
                debug!(
                    target: "engine::multi_leg",
                    input_mint = %candidate.pair.input_mint,
                    output_mint = %candidate.pair.output_mint,
                    amount = candidate.trade_size,
                    net_profit = candidate.net_profit(),
                    "多腿净收益不满足条件，丢弃"
                );
                continue;
            }

            events::profit_opportunity(
                candidate.pair.input_mint.as_str(),
                aggregator_label.as_str(),
                forward_quote.amount_in,
                forward_quote.amount_out,
                forward_latency_ms,
                reverse_quote.amount_in,
                reverse_quote.amount_out,
                reverse_latency_ms,
                profit.gross_profit_lamports,
                candidate.net_profit(),
                threshold,
                false,
                None,
                None,
                total_latency_ms,
            );

            candidates.push(candidate);
        }

        if candidates.is_empty() {
            return Ok(());
        }

        let mut any_executed = false;
        let mut last_error: Option<EngineError> = None;

        for mut candidate in candidates {
            if let Err(err) = runtime.populate_pair_plan(&mut candidate.plan).await {
                warn!(
                    target: "engine::multi_leg",
                    input_mint = %candidate.pair.input_mint,
                    output_mint = %candidate.pair.output_mint,
                    amount = candidate.trade_size,
                    error = %err,
                    "多腿 ALT 填充失败，跳过该组合"
                );
                continue;
            }
            match self.execute_multi_leg(candidate).await {
                Ok(_) => {
                    any_executed = true;
                }
                Err(err) => {
                    warn!(
                        target: "engine::multi_leg",
                        error = %err,
                        "多腿落地失败，继续尝试下一个组合"
                    );
                    last_error = Some(err);
                }
            }
        }

        if any_executed || last_error.is_none() {
            Ok(())
        } else {
            Err(last_error.expect("checked above"))
        }
    }

    async fn execute_multi_leg(&mut self, execution: MultiLegExecution) -> EngineResult<()> {
        let strategy_name = self.strategy.name();

        let MultiLegExecution {
            descriptor,
            pair,
            trade_size,
            mut plan,
            gross_profit,
            tip_lamports,
            tag,
        } = execution;

        if let Some(label) = tag.as_deref() {
            debug!(
                target: "engine::multi_leg",
                buy_kind = %descriptor.buy.kind,
                sell_kind = %descriptor.sell.kind,
                amount = trade_size,
                tag = label,
                "开始执行多腿组合"
            );
        } else {
            debug!(
                target: "engine::multi_leg",
                buy_kind = %descriptor.buy.kind,
                sell_kind = %descriptor.sell.kind,
                amount = trade_size,
                "开始执行多腿组合"
            );
        }

        let bundle = assemble_multi_leg_instructions(&mut plan);
        let variant = SwapInstructionsVariant::MultiLeg(bundle);
        let prioritization_fee = variant.prioritization_fee_lamports().unwrap_or_default();
        let compute_unit_limit = multi_leg_compute_limit(&variant).unwrap_or(FALLBACK_CU_LIMIT);

        let opportunity = SwapOpportunity {
            pair: pair.clone(),
            amount_in: trade_size,
            profit_lamports: gross_profit,
            tip_lamports,
            merged_quote: None,
            ultra_legs: None,
        };

        events::swap_fetched(
            strategy_name,
            &opportunity,
            compute_unit_limit,
            prioritization_fee,
            None,
        );

        let deadline = Instant::now() + self.settings.landing_timeout;
        let execution_plan = ExecutionPlan::new(
            opportunity,
            variant,
            pair.input_pubkey,
            tip_lamports,
            BASE_TX_FEE_LAMPORTS,
            compute_unit_limit,
            prioritization_fee,
            deadline,
        );

        self.dispatch_execution_plan(execution_plan, None).await
    }
}

fn multi_leg_compute_limit(variant: &SwapInstructionsVariant) -> Option<u32> {
    match variant {
        SwapInstructionsVariant::MultiLeg(bundle) => Some(bundle.compute_unit_limit),
        _ => None,
    }
}

fn assemble_multi_leg_instructions(plan: &mut LegPairPlan) -> MultiLegInstructions {
    if plan.buy.blockhash.is_some() || plan.sell.blockhash.is_some() {
        debug!(
            target: "engine::multi_leg",
            buy_blockhash = ?plan.buy.blockhash,
            sell_blockhash = ?plan.sell.blockhash,
            "腿计划附带 blockhash，将在组合阶段重新获取"
        );
    }

    let (buy_cb, buy_limit) = extract_plan_compute_requirements(&mut plan.buy);
    let (sell_cb, sell_limit) = extract_plan_compute_requirements(&mut plan.sell);

    let mut compute_budget_instructions = buy_cb;
    compute_budget_instructions.extend(sell_cb);

    let mut main_instructions = std::mem::take(&mut plan.buy.instructions);
    main_instructions.extend(std::mem::take(&mut plan.sell.instructions));

    let mut address_lookup_table_addresses =
        std::mem::take(&mut plan.buy.address_lookup_table_addresses);
    address_lookup_table_addresses.extend(std::mem::take(
        &mut plan.sell.address_lookup_table_addresses,
    ));

    let mut resolved_lookup_tables = std::mem::take(&mut plan.buy.resolved_lookup_tables);
    resolved_lookup_tables.extend(std::mem::take(&mut plan.sell.resolved_lookup_tables));

    let fee_sum = plan
        .buy
        .prioritization_fee_lamports
        .unwrap_or(0)
        .saturating_add(plan.sell.prioritization_fee_lamports.unwrap_or(0));
    let prioritization_fee = if fee_sum > 0 { Some(fee_sum) } else { None };

    let mut merged_limit = buy_limit
        .unwrap_or(0)
        .saturating_add(sell_limit.unwrap_or(0));
    let merged_price = plan
        .buy
        .requested_compute_unit_price_micro_lamports
        .or(plan.sell.requested_compute_unit_price_micro_lamports);

    if merged_limit == 0 {
        merged_limit =
            extract_compute_unit_limit(&compute_budget_instructions).unwrap_or(FALLBACK_CU_LIMIT);
    }

    compute_budget_instructions.retain(|ix| {
        if ix.program_id != COMPUTE_BUDGET_PROGRAM_ID {
            return true;
        }
        match ix.data.first().copied() {
            Some(2) | Some(3) => false,
            _ => true,
        }
    });

    let mut final_compute_budget = Vec::new();
    final_compute_budget.push(compute_unit_limit_instruction(merged_limit));
    if let Some(price) = merged_price {
        if price > 0 {
            final_compute_budget.push(compute_unit_price_instruction(price));
        }
    }
    final_compute_budget.extend(compute_budget_instructions);

    let mut bundle = MultiLegInstructions::new(
        final_compute_budget,
        main_instructions,
        address_lookup_table_addresses,
        resolved_lookup_tables,
        prioritization_fee,
        merged_limit,
    );
    bundle.dedup_lookup_tables();
    bundle
}

fn extract_compute_unit_limit(instructions: &[Instruction]) -> Option<u32> {
    for ix in instructions {
        if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID && ix.data.first() == Some(&2) {
            if ix.data.len() >= 5 {
                let mut buf = [0u8; 4];
                buf.copy_from_slice(&ix.data[1..5]);
                return Some(u32::from_le_bytes(buf));
            }
        }
    }
    None
}

fn extract_plan_compute_requirements(plan: &mut LegPlan) -> (Vec<Instruction>, Option<u32>) {
    let mut residual = Vec::new();
    let mut limit = plan.requested_compute_unit_limit;

    for ix in std::mem::take(&mut plan.compute_budget_instructions) {
        if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID {
            match ix.data.first().copied() {
                Some(2) => {
                    if limit.is_none() && ix.data.len() >= 5 {
                        let mut buf = [0u8; 4];
                        buf.copy_from_slice(&ix.data[1..5]);
                        limit = Some(u32::from_le_bytes(buf));
                    }
                    continue;
                }
                _ => {}
            }
        }
        residual.push(ix);
    }

    if limit.is_none() {
        limit = plan.requested_compute_unit_limit;
    }

    (residual, limit)
}
