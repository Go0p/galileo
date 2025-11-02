use async_trait::async_trait;

use crate::engine::EngineResult;
use crate::instructions::compute_budget::{COMPUTE_BUDGET_PROGRAM_ID, compute_budget_sequence};

use super::{AssemblyContext, InstructionDecorator};
use crate::engine::assembly::bundle::InstructionBundle;

pub struct ComputeBudgetDecorator;

#[async_trait]
impl InstructionDecorator for ComputeBudgetDecorator {
    async fn apply(
        &self,
        bundle: &mut InstructionBundle,
        context: &mut AssemblyContext<'_>,
    ) -> EngineResult<()> {
        let target_limit = context.compute_unit_limit;
        let target_price = context.compute_unit_price.unwrap_or(0);

        bundle.set_compute_unit_limit(target_limit);
        bundle.set_compute_unit_price(target_price);

        if let Some(variant) = context.variant.as_deref_mut() {
            match variant {
                crate::engine::SwapInstructionsVariant::Jupiter(response) => {
                    response.compute_unit_limit = target_limit;
                    let preserved: Vec<_> = response
                        .compute_budget_instructions
                        .drain(..)
                        .filter(is_preserved_compute_budget)
                        .collect();
                    let seq = compute_budget_sequence(target_price, target_limit, None);
                    let mut rebuilt = seq.into_vec();
                    rebuilt.extend(preserved);
                    response.compute_budget_instructions = rebuilt;
                }
                crate::engine::SwapInstructionsVariant::MultiLeg(multi_leg) => {
                    multi_leg.compute_unit_limit = target_limit;
                    let preserved: Vec<_> = multi_leg
                        .compute_budget_instructions
                        .drain(..)
                        .filter(is_preserved_compute_budget)
                        .collect();
                    let seq = compute_budget_sequence(target_price, target_limit, None);
                    let mut rebuilt = seq.into_vec();
                    rebuilt.extend(preserved);
                    multi_leg.compute_budget_instructions = rebuilt;
                }
                _ => {}
            }
        }

        Ok(())
    }
}

fn is_preserved_compute_budget(ix: &solana_sdk::instruction::Instruction) -> bool {
    if ix.program_id != COMPUTE_BUDGET_PROGRAM_ID {
        return true;
    }
    match ix.data.first().copied() {
        Some(2) | Some(3) => false,
        _ => true,
    }
}
