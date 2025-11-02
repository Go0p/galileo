use std::sync::Arc;
use std::time::Instant;

use crate::engine::assembly::decorators::{
    ComputeBudgetDecorator, FlashloanDecorator, GuardBudgetDecorator, ProfitGuardDecorator,
};
use crate::engine::assembly::{
    AssemblyContext, DecoratorChain, InstructionBundle, attach_lighthouse,
};
use crate::engine::multi_leg::orchestrator::{LegPairDescriptor, LegPairPlan};
use crate::engine::multi_leg::runtime::{PairPlanBatchResult, PairPlanEvaluation, PairPlanRequest};
use crate::engine::multi_leg::types::{
    LegPlan, LegSide as MultiLegSide, QuoteIntent as MultiLegQuoteIntent,
};
use crate::engine::{
    EngineError, EngineResult, MultiLegInstructions, QuoteTask, SwapInstructionsVariant,
    SwapOpportunity,
};
use crate::instructions::compute_budget::{
    COMPUTE_BUDGET_PROGRAM_ID, compute_unit_limit_instruction, compute_unit_price_instruction,
};
use crate::lander::Deadline;
use crate::monitoring::events;
use crate::strategy::types::TradePair;
use crate::strategy::{Strategy, StrategyEvent};
use solana_sdk::instruction::Instruction;
use tracing::{debug, info, warn};

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

impl<S> StrategyEngine<S>
where
    S: Strategy<Event = StrategyEvent>,
{
    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub(super) async fn process_multi_leg_task(
        &mut self,
        ctx: &mut MultiLegEngineContext,
        task: QuoteTask,
    ) -> EngineResult<()> {
        if task.amount == 0 {
            return Ok(());
        }

        let combinations = ctx.combinations();
        if combinations.is_empty() {
            debug!(
                target: "engine::multi_leg",
                input_mint = %task.pair.input_mint,
                output_mint = %task.pair.output_mint,
                "无可用的腿组合，跳过本次任务"
            );
            return Ok(());
        }

        let compute_unit_price = self.settings.sample_compute_unit_price();
        let buy_intent_template = MultiLegQuoteIntent::new(
            task.pair.input_pubkey,
            task.pair.output_pubkey,
            task.amount,
            0,
        );
        let sell_intent_template = MultiLegQuoteIntent::new(
            task.pair.output_pubkey,
            task.pair.input_pubkey,
            task.amount,
            0,
        );
        let tag_value = format!(
            "{}->{}/{}",
            task.pair.input_mint, task.pair.output_mint, task.amount
        );

        let runtime = ctx.runtime();
        let mut requests = Vec::with_capacity(combinations.len());
        for combo in combinations {
            let buy_descriptor = match runtime
                .orchestrator()
                .descriptor(MultiLegSide::Buy, combo.buy_index)
                .cloned()
            {
                Some(descriptor) => descriptor,
                None => continue,
            };
            let sell_descriptor = match runtime
                .orchestrator()
                .descriptor(MultiLegSide::Sell, combo.sell_index)
                .cloned()
            {
                Some(descriptor) => descriptor,
                None => continue,
            };

            let buy_context =
                ctx.build_context(&buy_descriptor, self.identity.pubkey, compute_unit_price);
            let sell_context =
                ctx.build_context(&sell_descriptor, self.identity.pubkey, compute_unit_price);

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
            return Ok(());
        }

        let batch = ctx.runtime().plan_pair_batch_with_profit(requests).await;
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
                    None,
                    reverse_quote.amount_in,
                    reverse_quote.amount_out,
                    None,
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
                None,
                reverse_quote.amount_in,
                reverse_quote.amount_out,
                None,
                profit.gross_profit_lamports,
                candidate.net_profit(),
                threshold,
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
        let mut variant = SwapInstructionsVariant::MultiLeg(bundle);
        let prioritization_fee = variant.prioritization_fee_lamports().unwrap_or_default();
        let mut instruction_bundle =
            InstructionBundle::from_instructions(variant.flatten_instructions());
        instruction_bundle.set_lookup_tables(
            variant.address_lookup_table_addresses().to_vec(),
            variant.resolved_lookup_tables().to_vec(),
        );

        let flashloan_opportunity = SwapOpportunity {
            pair: pair.clone(),
            amount_in: trade_size,
            profit_lamports: gross_profit,
            tip_lamports,
            merged_quote: None,
            ultra_legs: None,
        };

        let jito_tip_budget = self.jito_tip_budget(tip_lamports);

        let mut assembly_ctx = AssemblyContext::new(&self.identity);
        assembly_ctx.base_mint = Some(&pair.input_pubkey);
        assembly_ctx.compute_unit_limit =
            multi_leg_compute_limit(&variant).unwrap_or(FALLBACK_CU_LIMIT);
        assembly_ctx.compute_unit_price = None;
        assembly_ctx.guard_required = BASE_TX_FEE_LAMPORTS;
        assembly_ctx.prioritization_fee = prioritization_fee;
        assembly_ctx.tip_lamports = tip_lamports;
        assembly_ctx.jito_tip_budget = jito_tip_budget;
        assembly_ctx.variant = Some(&mut variant);
        assembly_ctx.opportunity = Some(&flashloan_opportunity);
        assembly_ctx.flashloan_manager = self.flashloan.as_ref();
        attach_lighthouse(&mut assembly_ctx, &mut self.lighthouse);

        let mut decorators = DecoratorChain::new();
        decorators.register(FlashloanDecorator);
        decorators.register(ComputeBudgetDecorator);
        decorators.register(GuardBudgetDecorator);
        decorators.register(ProfitGuardDecorator);

        decorators
            .apply_all(&mut instruction_bundle, &mut assembly_ctx)
            .await?;

        let final_instructions = instruction_bundle.into_flattened();

        if let Some(meta) = &assembly_ctx.flashloan_metadata {
            events::flashloan_applied(
                strategy_name,
                meta.protocol.as_str(),
                &meta.mint,
                meta.borrow_amount,
                meta.inner_instruction_count,
            );
            debug!(
                target: "engine::multi_leg",
                buy_kind = %descriptor.buy.kind,
                sell_kind = %descriptor.sell.kind,
                amount = trade_size,
                borrow_amount = meta.borrow_amount,
                tip_lamports,
                protocol = meta.protocol.as_str(),
                "多腿组合使用闪电贷"
            );
        }

        let prepared = self
            .tx_builder
            .build_with_sequence(&self.identity, &variant, final_instructions, tip_lamports)
            .await?;

        debug!(
            target: "engine::multi_leg",
            strategy = strategy_name,
            slot = prepared.slot,
            blockhash = %prepared.blockhash,
            "多腿交易已构建"
        );

        if self.settings.dry_run {
            info!(
                target: "engine::dry_run",
                strategy = strategy_name,
                slot = prepared.slot,
                blockhash = %prepared.blockhash,
                landers = self.landers.count(),
                "dry-run 模式：多腿交易已构建，跳过落地"
            );
            return Ok(());
        }

        let dispatch_strategy = self.settings.dispatch_strategy;
        let variant_layout = self.landers.variant_layout(dispatch_strategy);
        let plan = Arc::new(self.variant_planner.plan(
            dispatch_strategy,
            &prepared,
            &variant_layout,
        ));

        let deadline = Deadline::from_instant(Instant::now() + self.settings.landing_timeout);
        let tx_signature = plan
            .primary_variant()
            .and_then(|variant| variant.signature());
        let lander_stack = Arc::clone(&self.landers);
        let strategy_label = strategy_name.to_string();
        let tx_signature_for_log = tx_signature.clone();

        tokio::spawn(async move {
            match lander_stack
                .submit_plan(plan.as_ref(), deadline, &strategy_label)
                .await
            {
                Ok(_receipt) => {}
                Err(err) => {
                    let sig = tx_signature_for_log.as_deref().unwrap_or("");
                    warn!(
                        target: "engine::lander",
                        strategy = strategy_label.as_str(),
                        tx_signature = sig,
                        error = %err,
                        "{}",
                        format_args!(
                            "落地失败: 策略={} 签名={} 错误={}",
                            strategy_label,
                            sig,
                            err
                        )
                    );
                }
            }
        });

        Ok(())
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
