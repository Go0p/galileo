use crate::engine::EngineResult;
use crate::instructions::guards::lighthouse::token_amount_guard;
use async_trait::async_trait;

use super::{AssemblyContext, InstructionDecorator};
use crate::engine::assembly::bundle::InstructionBundle;

pub struct ProfitGuardDecorator;

#[async_trait]
impl InstructionDecorator for ProfitGuardDecorator {
    async fn apply(
        &self,
        bundle: &mut InstructionBundle,
        context: &mut AssemblyContext<'_>,
    ) -> EngineResult<()> {
        let guard_required = context.guard_required;
        if guard_required == 0 {
            return Ok(());
        }

        let Some(lighthouse) = context.lighthouse.as_deref_mut() else {
            return Ok(());
        };

        let Some(base_mint) = context.base_mint else {
            return Ok(());
        };

        if !lighthouse.should_guard(base_mint) {
            return Ok(());
        }

        let payer = context.identity.pubkey;
        let token_account =
            spl_associated_token_account::get_associated_token_address(&payer, base_mint);
        let memory_id = lighthouse.next_memory_id();
        let guard = token_amount_guard(payer, token_account, memory_id, guard_required);
        bundle.insert_profit_guard(guard);
        Ok(())
    }
}
