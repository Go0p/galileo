use async_trait::async_trait;

use crate::engine::EngineResult;

use super::{AssemblyContext, InstructionDecorator};
use crate::engine::assembly::bundle::InstructionBundle;

pub struct FlashloanDecorator;

#[async_trait]
impl InstructionDecorator for FlashloanDecorator {
    async fn apply(
        &self,
        bundle: &mut InstructionBundle,
        context: &mut AssemblyContext<'_>,
    ) -> EngineResult<()> {
        let Some(manager) = context.flashloan_manager else {
            return Ok(());
        };
        let Some(opportunity) = context.opportunity else {
            return Ok(());
        };
        let Some(variant) = context.variant.as_deref() else {
            return Ok(());
        };

        let outcome = manager
            .assemble(context.identity, opportunity, variant)
            .await?;

        if let Some(metadata) = outcome.metadata {
            context.flashloan_metadata = Some(metadata);
            let overhead = manager.compute_unit_overhead();
            if overhead > 0 {
                context.compute_unit_limit = context.compute_unit_limit.saturating_add(overhead);
            }
        }

        bundle.replace_instructions(outcome.instructions);

        Ok(())
    }
}
