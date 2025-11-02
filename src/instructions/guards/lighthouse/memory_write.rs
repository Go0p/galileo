use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;

use super::program::{LIGHTHOUSE_PROGRAM_ID, SYSTEM_PROGRAM_ID, push_compact_u64};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::guards::lighthouse::program::LIGHTHOUSE_PROGRAM_ID;

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
}
