use std::mem;

use once_cell::sync::Lazy;
use smallvec::SmallVec;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;

pub const COMPUTE_BUDGET_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ComputeBudget111111111111111111111111111111");

/// 缓存键，用于复用 compute budget 指令组合。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ComputeBudgetKey {
    data_size_limit: u32,
    unit_price: u64,
    unit_limit: u32,
    include_data_limit: bool,
}

static CACHE: Lazy<dashmap::DashMap<ComputeBudgetKey, SmallVec<[Instruction; 3]>>> =
    Lazy::new(dashmap::DashMap::new);

pub fn compute_unit_limit_instruction(limit: u32) -> Instruction {
    let mut data = Vec::with_capacity(1 + mem::size_of::<u32>());
    data.push(2);
    data.extend_from_slice(&limit.to_le_bytes());
    Instruction {
        program_id: COMPUTE_BUDGET_PROGRAM_ID,
        accounts: Vec::new(),
        data,
    }
}

pub fn compute_unit_price_instruction(price_micro_lamports: u64) -> Instruction {
    let mut data = Vec::with_capacity(1 + mem::size_of::<u64>());
    data.push(3);
    data.extend_from_slice(&price_micro_lamports.to_le_bytes());
    Instruction {
        program_id: COMPUTE_BUDGET_PROGRAM_ID,
        accounts: Vec::new(),
        data,
    }
}

pub fn set_loaded_accounts_data_size_limit(limit: u32) -> Instruction {
    let mut data = Vec::with_capacity(1 + mem::size_of::<u32>());
    data.push(4);
    data.extend_from_slice(&limit.to_le_bytes());
    Instruction {
        program_id: COMPUTE_BUDGET_PROGRAM_ID,
        accounts: Vec::new(),
        data,
    }
}

/// 根据参数生成一组 compute budget 指令，并做缓存。
pub fn compute_budget_sequence(
    unit_price: u64,
    unit_limit: u32,
    data_size_limit: Option<u32>,
) -> SmallVec<[Instruction; 3]> {
    let include_data_limit = data_size_limit.map(|value| value > 0).unwrap_or(false);
    let key = ComputeBudgetKey {
        data_size_limit: data_size_limit.unwrap_or_default(),
        unit_price,
        unit_limit,
        include_data_limit,
    };

    if let Some(cached) = CACHE.get(&key) {
        return cached.clone();
    }

    let mut seq = SmallVec::<[Instruction; 3]>::new();
    if include_data_limit {
        seq.push(set_loaded_accounts_data_size_limit(
            data_size_limit.unwrap_or_default(),
        ));
    }

    if unit_price > 0 {
        seq.push(compute_unit_price_instruction(unit_price));
    }
    if unit_limit > 0 {
        seq.push(compute_unit_limit_instruction(unit_limit));
    }

    CACHE.insert(key, seq.clone());
    seq
}

pub fn is_compute_budget(ix: &Instruction) -> bool {
    ix.program_id == COMPUTE_BUDGET_PROGRAM_ID
}
