use async_trait::async_trait;

use crate::engine::EngineResult;

use super::{AssemblyContext, InstructionDecorator};
use crate::engine::assembly::bundle::InstructionBundle;

/// 累加 guard 所需预算（基础费 + 优先费 + Jito tip）。
pub struct GuardBudgetDecorator;

#[async_trait]
impl InstructionDecorator for GuardBudgetDecorator {
    async fn apply(
        &self,
        _bundle: &mut InstructionBundle,
        context: &mut AssemblyContext<'_>,
    ) -> EngineResult<()> {
        let updated = match context.guard_strategy {
            super::GuardStrategy::BasePlusTip => context
                .guard_required
                .saturating_add(context.jito_tip_budget),
            super::GuardStrategy::BasePlusPrioritizationFee => context
                .guard_required
                .saturating_add(context.prioritization_fee),
            super::GuardStrategy::BasePlusTipAndPrioritizationFee => context
                .guard_required
                .saturating_add(context.prioritization_fee)
                .saturating_add(context.jito_tip_budget),
        };
        context.guard_required = updated;
        Ok(())
    }
}
