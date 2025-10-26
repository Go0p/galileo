use solana_sdk::instruction::Instruction;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;

/// 查找表解析状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UltraLookupState {
    Resolved,
    Pending,
}

/// Ultra 报价解码后的中间产物，保留原始交易以及指令元数据。
#[derive(Debug, Clone)]
pub struct UltraPreparedSwap {
    pub transaction: VersionedTransaction,
    pub compute_budget_instructions: Vec<Instruction>,
    pub main_instructions: Vec<Instruction>,
    pub address_lookup_table_addresses: Vec<Pubkey>,
    pub resolved_lookup_tables: Vec<AddressLookupTableAccount>,
    pub requested_compute_unit_limit: Option<u32>,
    pub requested_compute_unit_price_micro_lamports: Option<u64>,
    pub prioritization_fee_lamports: Option<u64>,
    pub account_rewrites: Vec<(Pubkey, Pubkey)>,
    pub lookup_state: UltraLookupState,
}

impl UltraPreparedSwap {
    pub fn transaction(&self) -> &VersionedTransaction {
        &self.transaction
    }

    pub fn lookup_state(&self) -> UltraLookupState {
        self.lookup_state
    }

    pub fn account_rewrites(&self) -> &[(Pubkey, Pubkey)] {
        &self.account_rewrites
    }

    pub fn compute_budget_instructions(&self) -> &[Instruction] {
        &self.compute_budget_instructions
    }

    pub fn main_instructions(&self) -> &[Instruction] {
        &self.main_instructions
    }

    pub fn address_lookup_table_addresses(&self) -> &[Pubkey] {
        &self.address_lookup_table_addresses
    }

    pub fn resolved_lookup_tables(&self) -> &[AddressLookupTableAccount] {
        &self.resolved_lookup_tables
    }

    pub fn requested_compute_unit_limit(&self) -> Option<u32> {
        self.requested_compute_unit_limit
    }

    pub fn requested_compute_unit_price_micro_lamports(&self) -> Option<u64> {
        self.requested_compute_unit_price_micro_lamports
    }

    pub fn prioritization_fee_lamports(&self) -> Option<u64> {
        self.prioritization_fee_lamports
    }
}

#[derive(Debug, Clone)]
pub struct UltraFinalizedSwap {
    pub compute_budget_instructions: Vec<Instruction>,
    pub main_instructions: Vec<Instruction>,
    pub address_lookup_table_addresses: Vec<Pubkey>,
    pub resolved_lookup_tables: Vec<AddressLookupTableAccount>,
    pub prioritization_fee_lamports: Option<u64>,
    pub compute_unit_limit: u32,
}
