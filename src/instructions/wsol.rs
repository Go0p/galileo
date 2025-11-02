#![allow(dead_code)]

use std::sync::Arc;

use dashmap::DashMap;
use once_cell::sync::Lazy;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;

use crate::cache::cached_associated_token_address;

pub const WSOL_MINT: Pubkey = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");
const SPL_TOKEN_PROGRAM: Pubkey = spl_token::ID;
const ASSOCIATED_TOKEN_PROGRAM: Pubkey = spl_associated_token_account::ID;
const SYSTEM_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("11111111111111111111111111111111");

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct WrapKey {
    owner: Pubkey,
    lamports: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct UnwrapKey(Pubkey);

static WRAP_CACHE: Lazy<DashMap<WrapKey, Arc<Vec<Instruction>>>> = Lazy::new(DashMap::new);
static UNWRAP_CACHE: Lazy<DashMap<UnwrapKey, Arc<Vec<Instruction>>>> = Lazy::new(DashMap::new);

#[derive(Debug, Clone, Copy)]
pub struct WrapSignature {
    pub owner: Pubkey,
    pub amount: u64,
    pub consumed: usize,
}

/// 返回缓存的包裹 SOL 指令序列（创建 ATA + 转账 + sync）。
pub fn wrap_sequence(owner: &Pubkey, lamports: u64) -> Arc<Vec<Instruction>> {
    let key = WrapKey {
        owner: *owner,
        lamports,
    };
    WRAP_CACHE
        .entry(key)
        .or_insert_with(|| Arc::new(build_wrap_sequence(*owner, lamports)))
        .clone()
}

/// 返回缓存的解包 SOL 指令序列（关闭 WSOL ATA）。
pub fn unwrap_sequence(owner: &Pubkey) -> Arc<Vec<Instruction>> {
    let key = UnwrapKey(*owner);
    UNWRAP_CACHE
        .entry(key)
        .or_insert_with(|| Arc::new(build_unwrap_sequence(*owner)))
        .clone()
}

/// 若指令前缀匹配包裹 SOL 序列，返回拥有者、金额与消耗的指令数。
pub fn detect_wrap_sequence(instructions: &[Instruction]) -> Option<WrapSignature> {
    if instructions.len() < 2 {
        return None;
    }

    let create = instructions.first()?;
    if create.program_id != ASSOCIATED_TOKEN_PROGRAM {
        return None;
    }
    if create.accounts.len() < 6 {
        return None;
    }

    let payer = create.accounts[0].pubkey;
    let ata = create.accounts[1].pubkey;
    let owner = create.accounts[2].pubkey;
    let mint = create.accounts[3].pubkey;
    let token_program = create.accounts[5].pubkey;

    if mint != WSOL_MINT || token_program != SPL_TOKEN_PROGRAM || payer != owner {
        return None;
    }
    let expected_ata = cached_associated_token_address(&owner, &WSOL_MINT, &SPL_TOKEN_PROGRAM);
    if ata != expected_ata {
        return None;
    }

    let transfer = instructions.get(1)?;
    if transfer.program_id != SYSTEM_PROGRAM_ID {
        return None;
    }
    if transfer.accounts.len() < 2 {
        return None;
    }
    if transfer.accounts[0].pubkey != payer || transfer.accounts[1].pubkey != ata {
        return None;
    }
    let amount = decode_transfer_lamports(transfer)?;

    let mut consumed = 2;
    if let Some(sync) = instructions.get(2) {
        if is_sync_native_instruction(sync, ata) {
            consumed = 3;
        }
    }

    Some(WrapSignature {
        owner,
        amount,
        consumed,
    })
}

/// 判断指令是否为关闭 WSOL 账户，返回账户拥有者。
pub fn detect_close_instruction(instruction: &Instruction) -> Option<Pubkey> {
    if instruction.program_id != SPL_TOKEN_PROGRAM {
        return None;
    }
    if instruction.data.first().copied() != Some(9) {
        return None;
    }
    if instruction.accounts.len() < 3 {
        return None;
    }

    let ata = instruction.accounts[0].pubkey;
    let destination = instruction.accounts[1].pubkey;
    let authority = instruction.accounts[2].pubkey;
    if destination != authority {
        return None;
    }

    let expected_ata = cached_associated_token_address(&authority, &WSOL_MINT, &SPL_TOKEN_PROGRAM);
    if ata != expected_ata {
        return None;
    }

    Some(authority)
}

fn build_wrap_sequence(owner: Pubkey, lamports: u64) -> Vec<Instruction> {
    let ata = cached_associated_token_address(&owner, &WSOL_MINT, &SPL_TOKEN_PROGRAM);
    let mut instructions = Vec::with_capacity(3);
    instructions.push(build_create_wsol_instruction(owner, ata));
    instructions.push(build_transfer_instruction(owner, ata, lamports));
    instructions.push(
        spl_token::instruction::sync_native(&SPL_TOKEN_PROGRAM, &ata)
            .expect("WSOL sync_native instruction"),
    );
    instructions
}

fn build_unwrap_sequence(owner: Pubkey) -> Vec<Instruction> {
    let ata = cached_associated_token_address(&owner, &WSOL_MINT, &SPL_TOKEN_PROGRAM);
    vec![
        spl_token::instruction::close_account(&SPL_TOKEN_PROGRAM, &ata, &owner, &owner, &[])
            .expect("WSOL close_account instruction"),
    ]
}

fn build_create_wsol_instruction(owner: Pubkey, ata: Pubkey) -> Instruction {
    Instruction {
        program_id: ASSOCIATED_TOKEN_PROGRAM,
        accounts: vec![
            AccountMeta::new(owner, true),
            AccountMeta::new(ata, false),
            AccountMeta::new_readonly(owner, false),
            AccountMeta::new_readonly(WSOL_MINT, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(SPL_TOKEN_PROGRAM, false),
        ],
        data: vec![1],
    }
}

fn build_transfer_instruction(owner: Pubkey, ata: Pubkey, lamports: u64) -> Instruction {
    let mut data = Vec::with_capacity(12);
    data.extend_from_slice(&2u32.to_le_bytes());
    data.extend_from_slice(&lamports.to_le_bytes());
    Instruction {
        program_id: SYSTEM_PROGRAM_ID,
        accounts: vec![AccountMeta::new(owner, true), AccountMeta::new(ata, false)],
        data,
    }
}

fn decode_transfer_lamports(transfer: &Instruction) -> Option<u64> {
    if transfer.data.len() != 12 {
        return None;
    }
    let mut discriminant = [0u8; 4];
    discriminant.copy_from_slice(&transfer.data[..4]);
    if u32::from_le_bytes(discriminant) != 2 {
        return None;
    }
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&transfer.data[4..12]);
    Some(u64::from_le_bytes(bytes))
}

fn is_sync_native_instruction(instruction: &Instruction, ata: Pubkey) -> bool {
    instruction.program_id == SPL_TOKEN_PROGRAM
        && instruction
            .accounts
            .first()
            .map(|meta| meta.pubkey == ata && !meta.is_signer && meta.is_writable)
            .unwrap_or(false)
        && instruction.data.first().copied() == Some(17)
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn detect_wrap_sequence_matches_cached_sequence() {
        let owner = Pubkey::new_unique();
        let lamports = 1_000_000;
        let cached = wrap_sequence(&owner, lamports);
        let signature = detect_wrap_sequence(&cached).expect("wrap signature");
        assert_eq!(signature.owner, owner);
        assert_eq!(signature.amount, lamports);
        assert_eq!(signature.consumed, cached.len());
    }

    #[test]
    fn detect_close_instruction_matches_cached_sequence() {
        let owner = Pubkey::new_unique();
        let cached = unwrap_sequence(&owner);
        assert_eq!(cached.len(), 1);
        let instruction = &cached[0];
        let detected = detect_close_instruction(instruction).expect("owner detected");
        assert_eq!(detected, owner);
    }
}
