use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;

use crate::lighthouse::program::{
    IntegerOperator, LIGHTHOUSE_PROGRAM_ID, LogLevel, push_compact_u64,
};

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

#[cfg(test)]
mod tests {
    use super::*;

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
