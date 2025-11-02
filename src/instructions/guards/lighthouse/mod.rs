pub mod account_delta;
pub mod guard;
pub mod memory_write;
pub mod program;

pub use guard::TokenAmountGuard;

/// 兼容旧接口：直接暴露构建函数，供装配流水线与其他模块使用。
pub fn token_amount_guard(
    payer: solana_sdk::pubkey::Pubkey,
    target_account: solana_sdk::pubkey::Pubkey,
    memory_id: u8,
    min_delta: u64,
) -> TokenAmountGuard {
    guard::build_token_amount_guard(payer, target_account, memory_id, min_delta)
}
