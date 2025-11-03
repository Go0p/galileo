use std::time::Instant;

use solana_sdk::pubkey::Pubkey;

use crate::engine::aggregator::SwapInstructionsVariant;
use crate::engine::types::SwapOpportunity;

/// ExecutionPlan 表示策略层产出的「交易意图」。
/// 它包含构建落地交易所需的指令变体与利润上下文，但不携带任一落地器特有的细节。
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub opportunity: SwapOpportunity,
    pub swap_variant: SwapInstructionsVariant,
    pub base_mint: Pubkey,
    pub base_tip_lamports: u64,
    pub base_guard_lamports: u64,
    pub compute_unit_limit: u32,
    pub prioritization_fee_lamports: u64,
    pub deadline: Instant,
}

impl ExecutionPlan {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        opportunity: SwapOpportunity,
        swap_variant: SwapInstructionsVariant,
        base_mint: Pubkey,
        base_tip_lamports: u64,
        base_guard_lamports: u64,
        compute_unit_limit: u32,
        prioritization_fee_lamports: u64,
        deadline: Instant,
    ) -> Self {
        Self {
            opportunity,
            swap_variant,
            base_mint,
            base_tip_lamports,
            base_guard_lamports,
            compute_unit_limit,
            prioritization_fee_lamports,
            deadline,
        }
    }
}
