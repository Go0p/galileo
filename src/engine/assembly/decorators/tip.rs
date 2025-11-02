use async_trait::async_trait;
use solana_system_interface::instruction as system_instruction;

use crate::engine::EngineResult;

use super::{AssemblyContext, InstructionDecorator};
use crate::engine::assembly::bundle::InstructionBundle;

/// 将 Jito tip 转账指令插入到主指令之后、ProfitGuard 之前。
pub struct TipDecorator;

#[async_trait]
impl InstructionDecorator for TipDecorator {
    async fn apply(
        &self,
        bundle: &mut InstructionBundle,
        context: &mut AssemblyContext<'_>,
    ) -> EngineResult<()> {
        let Some(plan) = context.jito_tip_plan.as_ref() else {
            return Ok(());
        };

        if plan.lamports == 0 {
            return Ok(());
        }

        let payer = context.identity.pubkey;
        let transfer = system_instruction::transfer(&payer, &plan.recipient, plan.lamports);
        bundle.post.push(transfer);
        Ok(())
    }
}
