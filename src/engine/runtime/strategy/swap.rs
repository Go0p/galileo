use std::sync::Arc;
use std::time::Instant;

use tokio::task;
use tracing::{debug, error, info, trace, warn};

use crate::api::dflow::DflowError;
use crate::engine::assembly::decorators::{
    ComputeBudgetDecorator, FlashloanDecorator, GuardBudgetDecorator, ProfitGuardDecorator,
};
use crate::engine::assembly::{
    AssemblyContext, DecoratorChain, InstructionBundle, attach_lighthouse,
};
use crate::engine::quote_dispatcher;
use crate::engine::types::ExecutionPlan;
use crate::engine::{EngineError, EngineResult};
use crate::lander::Deadline;
use crate::monitoring::events;
use crate::network::{IpLeaseMode, IpTaskKind};
use crate::strategy::{Strategy, StrategyEvent};

use super::{BASE_TX_FEE_LAMPORTS, StrategyEngine};

impl<S> StrategyEngine<S>
where
    S: Strategy<Event = StrategyEvent>,
{
    pub(super) async fn execute_plan(&mut self, plan: ExecutionPlan) -> EngineResult<()> {
        let ExecutionPlan {
            opportunity,
            deadline,
        } = plan;
        let strategy_name = self.strategy.name();

        trace!(
            target: "engine::swap",
            base_mint = %opportunity.pair.input_mint,
            quote_mint = %opportunity.pair.output_mint,
            amount = opportunity.amount_in,
            profit = opportunity.profit_lamports,
            tip = opportunity.tip_lamports,
            "执行套利计划"
        );

        if opportunity.profit_lamports <= 0 {
            debug!(
                target: "engine::swap",
                "机会利润为非正值，放弃执行",
            );
            return Ok(());
        }

        if Instant::now() > deadline {
            debug!(
                target: "engine::swap",
                "机会已超过执行截止时间，放弃",
            );
            return Ok(());
        }

        let swap_lease = match self
            .ip_allocator
            .acquire(IpTaskKind::SwapInstruction, IpLeaseMode::Ephemeral)
            .await
        {
            Ok(lease) => lease,
            Err(err) => return Err(EngineError::NetworkResource(err)),
        };
        let swap_handle = swap_lease.handle();
        let swap_ip = Some(swap_handle.ip());
        drop(swap_lease);

        let mut swap_variant = match self
            .swap_preparer
            .prepare(&opportunity, &self.identity, &swap_handle)
            .await
        {
            Ok(variant) => {
                drop(swap_handle);
                variant
            }
            Err(err) => {
                if let Some(outcome) = quote_dispatcher::classify_ip_outcome(&err) {
                    swap_handle.mark_outcome(outcome);
                }
                drop(swap_handle);

                if let EngineError::Dflow(dflow_err @ DflowError::ApiStatus { status, body, .. }) =
                    &err
                {
                    if status.as_u16() == 500 && body.contains("failed_to_compute_swap") {
                        let detail = dflow_err.describe();
                        warn!(
                            target: "engine::swap",
                            status = status.as_u16(),
                            error = %detail,
                            "DFlow swap 指令生成失败，跳过当前机会。Error: {body}"
                        );
                        return Ok(());
                    }
                }
                if let EngineError::Dflow(
                    dflow_err @ DflowError::RateLimited { status, body, .. },
                ) = &err
                {
                    let detail = dflow_err.describe();
                    warn!(
                        target: "engine::swap",
                        status = status.as_u16(),
                        input_mint = %opportunity.pair.input_mint,
                        output_mint = %opportunity.pair.output_mint,
                        error = %detail,
                        "DFlow 指令命中限流，放弃当前机会: {body}"
                    );
                    return Ok(());
                }
                if let EngineError::Dflow(other) = &err {
                    let detail = other.describe();
                    warn!(
                        target: "engine::swap",
                        input_mint = %opportunity.pair.input_mint,
                        output_mint = %opportunity.pair.output_mint,
                        error = %detail,
                        "DFlow 指令失败，跳过当前机会"
                    );
                    return Ok(());
                }
                if let EngineError::InvalidConfig(message) = &err {
                    if message.starts_with("Ultra 指令解析失败") {
                        error!(
                            target: "engine::swap",
                            input_mint = %opportunity.pair.input_mint,
                            output_mint = %opportunity.pair.output_mint,
                            amount_in = opportunity.amount_in,
                            error = %message,
                            "Ultra 指令解析失败，跳过当前机会"
                        );
                        return Ok(());
                    }
                }
                return Err(err);
            }
        };

        let mut instruction_bundle =
            InstructionBundle::from_instructions(swap_variant.flatten_instructions());
        instruction_bundle.set_lookup_tables(
            swap_variant.address_lookup_table_addresses().to_vec(),
            swap_variant.resolved_lookup_tables().to_vec(),
        );

        let mut assembly_ctx = AssemblyContext::new(&self.identity);
        assembly_ctx.base_mint = Some(&opportunity.pair.input_pubkey);
        assembly_ctx.compute_unit_limit = swap_variant.compute_unit_limit();
        assembly_ctx.compute_unit_price = None;
        assembly_ctx.guard_required = BASE_TX_FEE_LAMPORTS;
        assembly_ctx.prioritization_fee = swap_variant
            .prioritization_fee_lamports()
            .unwrap_or_default();
        assembly_ctx.tip_lamports = opportunity.tip_lamports;
        assembly_ctx.jito_tip_budget = self.jito_tip_budget(opportunity.tip_lamports);
        assembly_ctx.variant = Some(&mut swap_variant);
        assembly_ctx.opportunity = Some(&opportunity);
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

        let compute_unit_limit = assembly_ctx.compute_unit_limit;
        let prioritization_fee = assembly_ctx.prioritization_fee;
        let final_instructions = instruction_bundle.into_flattened();

        events::swap_fetched(
            strategy_name,
            &opportunity,
            compute_unit_limit,
            prioritization_fee,
            swap_ip,
        );

        if let Some(meta) = &assembly_ctx.flashloan_metadata {
            events::flashloan_applied(
                strategy_name,
                meta.protocol.as_str(),
                &meta.mint,
                meta.borrow_amount,
                meta.inner_instruction_count,
            );
        }

        let prepared = self
            .tx_builder
            .build_with_sequence(
                &self.identity,
                &swap_variant,
                final_instructions,
                opportunity.tip_lamports,
            )
            .await?;
        events::transaction_built(
            strategy_name,
            &opportunity,
            prepared.slot,
            &prepared.blockhash.to_string(),
            prepared.last_valid_block_height,
            swap_ip,
        );

        if self.settings.dry_run {
            info!(
                target: "engine::dry_run",
                strategy = strategy_name,
                slot = prepared.slot,
                blockhash = %prepared.blockhash,
                landers = self.landers.count(),
                "dry-run 模式：交易已构建，跳过落地"
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

        let deadline = Deadline::from_instant(deadline);
        let tx_signature = plan
            .primary_variant()
            .and_then(|variant| variant.signature());
        let lander_stack = Arc::clone(&self.landers);
        let strategy_label = strategy_name.to_string();
        let tx_signature_for_log = tx_signature.clone();

        task::spawn(async move {
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
                        "落地失败: 策略={} 签名={} 错误={}",
                        strategy_label,
                        sig,
                        err
                    );
                }
            }
        });

        Ok(())
    }
}
