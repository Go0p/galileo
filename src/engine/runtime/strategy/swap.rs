use std::sync::Arc;
use std::time::Instant;

use tokio::task;
use tracing::{debug, error, info, trace, warn};

use crate::api::dflow::DflowError;
use crate::engine::assembly::decorators::GuardStrategy;
use crate::engine::landing::assembler::{
    DefaultLandingAssembler, LandingAssembler, LandingAssemblyContext, TipComputationKind,
};
use crate::engine::landing::{ExecutionPlan, LandingProfileBuilder};
use crate::engine::quote_dispatcher;
use crate::engine::types::SwapOpportunity;
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
    pub(super) async fn execute_plan(
        &mut self,
        opportunity: SwapOpportunity,
        deadline: Instant,
    ) -> EngineResult<()> {
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

        let swap_variant = match self
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

        let compute_unit_limit = swap_variant.compute_unit_limit();
        let prioritization_fee = swap_variant
            .prioritization_fee_lamports()
            .unwrap_or_default();

        events::swap_fetched(
            strategy_name,
            &opportunity,
            compute_unit_limit,
            prioritization_fee,
            swap_ip,
        );

        let base_mint = opportunity.pair.input_pubkey;
        let base_tip_lamports = opportunity.tip_lamports;

        let execution_plan = ExecutionPlan::new(
            opportunity,
            swap_variant,
            base_mint,
            base_tip_lamports,
            BASE_TX_FEE_LAMPORTS,
            compute_unit_limit,
            prioritization_fee,
            deadline,
        );

        self.dispatch_execution_plan(execution_plan, swap_ip).await
    }
}

impl<S> StrategyEngine<S>
where
    S: Strategy<Event = StrategyEvent>,
{
    pub(super) async fn dispatch_execution_plan(
        &mut self,
        execution_plan: ExecutionPlan,
        swap_ip: Option<std::net::IpAddr>,
    ) -> EngineResult<()> {
        let strategy_name = self.strategy.name();

        let variants = self.landers.variants();
        if variants.is_empty() {
            return Err(EngineError::Landing("no lander configured".into()));
        }

        let builder = LandingProfileBuilder::new();
        let sampled_compute_unit_price = self.settings.sample_compute_unit_price();
        let mut profiles = Vec::with_capacity(variants.len());
        for variant in variants {
            profiles.push(builder.build_for_variant(variant, sampled_compute_unit_price));
        }

        let assembler = DefaultLandingAssembler::new();
        let mut entries = Vec::with_capacity(profiles.len());
        for profile in profiles {
            let mut context = LandingAssemblyContext::new(
                &self.identity,
                &self.tx_builder,
                self.flashloan.as_ref(),
                &mut self.lighthouse,
            );
            let entry = assembler
                .assemble_landing(&mut context, &profile, &execution_plan)
                .await
                .map_err(|err| EngineError::Landing(err.to_string()))?;
            entries.push(entry);
        }

        if entries.is_empty() {
            return Err(EngineError::Landing("no landing entries".into()));
        }

        for entry in &entries {
            let guard_label = match entry.guard.kind {
                GuardStrategy::BasePlusTip => "base_plus_tip",
                GuardStrategy::BasePlusPrioritizationFee => "base_plus_fee",
                GuardStrategy::BasePlusTipAndPrioritizationFee => "base_plus_tip_fee",
            };
            let tip_kind_label = match entry.tip.kind {
                TipComputationKind::Opportunity => "opportunity",
                TipComputationKind::JitoPlan => "jito_plan",
            };

            debug!(
                target: "engine::landing_plan",
                strategy = strategy_name,
                lander = entry.profile.lander_kind.label(),
                tip_strategy = entry.profile.tip_strategy.label(),
                tip_kind = tip_kind_label,
                tip_lamports = entry.tip.lamports,
                guard_strategy = guard_label,
                guard_lamports = entry.guard.lamports,
                prioritization_fee = entry.prioritization_fee_lamports,
                base_prioritization_fee = execution_plan.prioritization_fee_lamports,
                cu_price_micro = entry
                    .prepared
                    .compute_unit_price_micro_lamports
                    .unwrap_or(0),
                "落地器计划已构建"
            );

            events::transaction_built(
                strategy_name,
                &execution_plan.opportunity,
                entry.prepared.slot,
                &entry.prepared.blockhash.to_string(),
                swap_ip,
            );
            if let Some(meta) = &entry.flashloan_metadata {
                events::flashloan_applied(
                    strategy_name,
                    meta.protocol.as_str(),
                    &meta.mint,
                    meta.borrow_amount,
                    meta.inner_instruction_count,
                );
            }
        }

        if self.settings.dry_run {
            info!(
                target: "engine::dry_run",
                strategy = strategy_name,
                slot = entries.first().map(|e| e.prepared.slot).unwrap_or_default(),
                blockhash = entries
                    .first()
                    .map(|e| e.prepared.blockhash.to_string())
                    .unwrap_or_else(|| "-".into()),
                landers = self.landers.count(),
                "dry-run 模式：交易将提交至覆盖的 RPC 端点"
            );
        }

        let prepared: Vec<_> = entries.iter().map(|entry| entry.prepared.clone()).collect();

        let dispatch_strategy = self.settings.dispatch_strategy;
        let variant_layout = self.landers.variant_layout(dispatch_strategy);
        let plan = Arc::new(self.variant_planner.plan(
            dispatch_strategy,
            &prepared,
            &variant_layout,
        ));

        let deadline = Deadline::from_instant(execution_plan.deadline);
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
