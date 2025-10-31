use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;

/// Lighthouse 主网程序 ID。
pub const LIGHTHOUSE_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    4, 223, 173, 121, 98, 255, 177, 221, 146, 93, 10, 159, 181, 230, 208, 12, 230, 25, 91, 168,
    187, 58, 145, 253, 7, 239, 152, 96, 197, 233, 123, 184,
]);

/// SPL Token 账户中 `amount` 字段的偏移（单位：字节）。
pub const TOKEN_ACCOUNT_AMOUNT_OFFSET: u16 = 64;
/// SPL Token 账户中 `amount` 字段长度（单位：字节）。
pub const TOKEN_ACCOUNT_AMOUNT_SIZE: u16 = 8;

const SYSTEM_PROGRAM_ID: Pubkey = Pubkey::new_from_array([0u8; 32]);

/// Lighthouse 日志等级枚举。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum LogLevel {
    Silent = 0,
    PlaintextMessage = 1,
    EncodedMessage = 2,
    EncodedNoop = 3,
    FailedPlaintextMessage = 4,
    FailedEncodedMessage = 5,
    FailedEncodedNoop = 6,
}

/// Lighthouse 整数比较运算符。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum IntegerOperator {
    Equal = 0,
    NotEqual = 1,
    GreaterThan = 2,
    LessThan = 3,
    GreaterThanOrEqual = 4,
    LessThanOrEqual = 5,
    Contains = 6,
    DoesNotContain = 7,
}

/// MemoryWrite 指令参数。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryWriteParams {
    pub payer: Pubkey,
    pub memory: Pubkey,
    pub memory_id: u8,
    pub memory_bump: u8,
    pub source_account: Pubkey,
    pub write_offset: u64,
    pub account_data_offset: u16,
    pub account_data_length: u16,
}

/// AccountDelta(Data) 指令参数。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AccountDeltaParams {
    pub memory: Pubkey,
    pub target_account: Pubkey,
    pub log_level: LogLevel,
    pub snapshot_offset: u64,
    pub account_data_offset: u64,
    pub expected_delta: i128,
    pub operator: IntegerOperator,
}

/// 构造 MemoryWrite 指令（对应 Lighthouse `memory_write`）。
pub fn build_memory_write_instruction(params: MemoryWriteParams) -> Instruction {
    let MemoryWriteParams {
        payer,
        memory,
        memory_id,
        memory_bump,
        source_account,
        write_offset,
        account_data_offset,
        account_data_length,
    } = params;

    let mut data = Vec::with_capacity(16);
    data.push(0); // instruction discriminator for MemoryWrite
    data.push(memory_id);
    data.push(memory_bump);
    push_compact_u64(&mut data, write_offset);
    data.push(0); // WriteType::AccountData variant index
    data.extend_from_slice(&account_data_offset.to_le_bytes());
    data.extend_from_slice(&account_data_length.to_le_bytes());

    let accounts = vec![
        AccountMeta::new_readonly(LIGHTHOUSE_PROGRAM_ID, false),
        AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        AccountMeta::new(payer, true),
        AccountMeta::new(memory, false),
        AccountMeta::new_readonly(source_account, false),
    ];

    Instruction {
        program_id: LIGHTHOUSE_PROGRAM_ID,
        accounts,
        data,
    }
}

/// 构造 AccountDelta(Data) 指令（对应 Lighthouse `assert_account_delta`）。
pub fn build_account_delta_instruction(params: AccountDeltaParams) -> Instruction {
    let AccountDeltaParams {
        memory,
        target_account,
        log_level,
        snapshot_offset,
        account_data_offset,
        expected_delta,
        operator,
    } = params;

    let mut data = Vec::with_capacity(32);
    data.push(4); // instruction discriminator for AssertAccountDelta
    data.push(log_level as u8);
    data.push(1); // AccountDeltaAssertion::Data variant index
    push_compact_u64(&mut data, snapshot_offset);
    push_compact_u64(&mut data, account_data_offset);
    data.push(6); // DataValueDeltaAssertion::U64 variant index
    data.extend_from_slice(&expected_delta.to_le_bytes());
    data.push(operator as u8);

    let accounts = vec![
        AccountMeta::new_readonly(memory, false),
        AccountMeta::new_readonly(target_account, false),
    ];

    Instruction {
        program_id: LIGHTHOUSE_PROGRAM_ID,
        accounts,
        data,
    }
}

/// 根据 payer 与 memory_id 推导 Lighthouse memory PDA。
pub fn find_memory_pda(payer: &Pubkey, memory_id: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[b"memory", payer.as_ref(), &[memory_id]],
        &LIGHTHOUSE_PROGRAM_ID,
    )
}

/// Lighthouse token 数量守护指令集合。
#[derive(Clone, Debug)]
pub struct TokenAmountGuard {
    pub memory_write: Instruction,
    pub assert_delta: Instruction,
    pub memory_bump: u8,
}

/// 针对 token 账户余额，返回 MemoryWrite + AccountDelta 指令组合。
///
/// 该组合可用于保障交易完成后 `target_account` 的金额相对快照至少增长 `min_delta`。
pub fn build_token_amount_guard(
    payer: Pubkey,
    target_account: Pubkey,
    memory_id: u8,
    min_delta: u64,
) -> TokenAmountGuard {
    let (memory, memory_bump) = find_memory_pda(&payer, memory_id);

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

fn push_compact_u64(buffer: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buffer.push(byte);
        if value == 0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_write_encoding_matches_expected_layout() {
        let params = MemoryWriteParams {
            payer: Pubkey::new_unique(),
            memory: Pubkey::new_unique(),
            memory_id: 7,
            memory_bump: 255,
            source_account: Pubkey::new_unique(),
            write_offset: 0,
            account_data_offset: 64,
            account_data_length: 8,
        };
        let ix = build_memory_write_instruction(params);
        assert_eq!(ix.program_id, LIGHTHOUSE_PROGRAM_ID);
        assert_eq!(
            ix.data,
            vec![0, 7, 255, 0, 0, 64, 0, 8, 0],
            "unexpected memory_write encoding"
        );
    }

    #[test]
    fn account_delta_encoding_matches_expected_layout() {
        let params = AccountDeltaParams {
            memory: Pubkey::new_unique(),
            target_account: Pubkey::new_unique(),
            log_level: LogLevel::FailedPlaintextMessage,
            snapshot_offset: 0,
            account_data_offset: 64,
            expected_delta: 34_657,
            operator: IntegerOperator::GreaterThanOrEqual,
        };
        let ix = build_account_delta_instruction(params);
        let mut expected = vec![4, 4, 1, 0, 64, 6];
        expected.extend_from_slice(&34_657i128.to_le_bytes());
        expected.push(IntegerOperator::GreaterThanOrEqual as u8);
        assert_eq!(ix.data, expected, "unexpected account_delta encoding");
    }
}
