use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;

use super::account_delta::{AccountDeltaParams, build_account_delta_instruction};
use super::memory_write::{MemoryWriteParams, build_memory_write_instruction};
use super::program::{
    IntegerOperator, LIGHTHOUSE_PROGRAM_ID, LogLevel, TOKEN_ACCOUNT_AMOUNT_OFFSET,
    TOKEN_ACCOUNT_AMOUNT_SIZE,
};

/// Lighthouse token 数量守护指令集合。
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct TokenAmountGuard {
    pub memory_write: Instruction,
    pub assert_delta: Instruction,
    pub memory_bump: u8,
}

/// 针对 token 账户余额，返回 MemoryWrite + AccountDelta 指令组合。
///
/// 该组合可用于保障交易完成后 `target_account` 的金额相对快照至少增长 `min_delta`。
#[allow(dead_code)]
pub fn build_token_amount_guard(
    payer: Pubkey,
    target_account: Pubkey,
    memory_id: u8,
    min_delta: u64,
) -> TokenAmountGuard {
    let (memory, memory_bump) = Pubkey::find_program_address(
        &[b"memory", payer.as_ref(), &[memory_id]],
        &LIGHTHOUSE_PROGRAM_ID,
    );

    let memory_write = build_memory_write_instruction(MemoryWriteParams {
        payer,
        memory,
        memory_id,
        memory_bump,
        source_account: target_account,
        write_offset: 0,
        account_data_offset: TOKEN_ACCOUNT_AMOUNT_OFFSET,
        account_data_length: TOKEN_ACCOUNT_AMOUNT_SIZE,
    });

    let account_delta = build_account_delta_instruction(AccountDeltaParams {
        memory,
        target_account,
        log_level: LogLevel::FailedPlaintextMessage,
        snapshot_offset: 0,
        account_data_offset: u64::from(TOKEN_ACCOUNT_AMOUNT_OFFSET),
        expected_delta: min_delta as i128,
        operator: IntegerOperator::GreaterThanOrEqual,
    });

    TokenAmountGuard {
        memory_write,
        assert_delta: account_delta,
        memory_bump,
    }
}
