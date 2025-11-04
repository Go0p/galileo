use std::mem;

use solana_sdk::instruction::Instruction;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;

use crate::instructions::compute_budget::{COMPUTE_BUDGET_PROGRAM_ID, compute_budget_sequence};
use crate::instructions::guards::lighthouse::TokenAmountGuard;

/// 描述交易指令不同阶段的组合。
#[derive(Debug, Clone, Default)]
pub struct InstructionBundle {
    pub compute_budget: Vec<Instruction>,
    pub pre: Vec<Instruction>,
    pub main: Vec<Instruction>,
    pub post: Vec<Instruction>,
    pub lookup_addresses: Vec<Pubkey>,
    pub resolved_lookups: Vec<AddressLookupTableAccount>,
    compute_unit_limit: Option<u32>,
    compute_unit_price: Option<u64>,
    extra_compute_budget: Vec<Instruction>,
    #[allow(clippy::struct_field_names)]
    compute_budget_dirty: bool,
}

impl InstructionBundle {
    /// 确保 bundle 中存在最新的 compute unit limit 指令。
    pub fn set_compute_unit_limit(&mut self, new_limit: u32) {
        let new_value = (new_limit > 0).then_some(new_limit);
        if self.compute_unit_limit != new_value {
            self.compute_unit_limit = new_value;
            self.compute_budget_dirty = true;
        }
    }

    pub fn set_compute_unit_price(&mut self, price: u64) {
        let new_value = (price > 0).then_some(price);
        if self.compute_unit_price != new_value {
            self.compute_unit_price = new_value;
            self.compute_budget_dirty = true;
        }
    }

    /// 将利润守护指令插入到预备/收尾阶段。
    pub fn insert_profit_guard(&mut self, guard: TokenAmountGuard) {
        self.pre.insert(0, guard.memory_write);
        self.post.push(guard.assert_delta);
    }

    /// 替换当前 bundle 的指令序列，同时保持现有的 ALT 元数据。
    pub fn replace_instructions(&mut self, instructions: Vec<Instruction>) {
        let mut updated = InstructionBundle::from_instructions(instructions);
        updated.lookup_addresses = mem::take(&mut self.lookup_addresses);
        updated.resolved_lookups = mem::take(&mut self.resolved_lookups);
        *self = updated;
    }

    pub fn set_lookup_tables(
        &mut self,
        lookup_addresses: Vec<Pubkey>,
        resolved_lookups: Vec<AddressLookupTableAccount>,
    ) {
        self.lookup_addresses = lookup_addresses;
        self.resolved_lookups = resolved_lookups;
    }

    /// 扁平化所有指令，按照 compute → pre → main → post 的顺序输出。
    #[cfg(test)]
    pub fn flatten(&mut self) -> Vec<Instruction> {
        self.ensure_compute_budget();
        let total_len = self
            .compute_budget
            .len()
            .saturating_add(self.pre.len())
            .saturating_add(self.main.len())
            .saturating_add(self.post.len());
        let mut combined = Vec::with_capacity(total_len);
        combined.extend(self.compute_budget.iter().cloned());
        combined.extend(self.pre.iter().cloned());
        combined.extend(self.main.iter().cloned());
        combined.extend(self.post.iter().cloned());
        combined
    }

    /// 消耗当前 bundle，直接拼接内部指令，避免额外拷贝。
    pub fn into_flattened(self) -> Vec<Instruction> {
        let mut owned = self;
        owned.ensure_compute_budget();
        let total_len = owned
            .compute_budget
            .len()
            .saturating_add(owned.pre.len())
            .saturating_add(owned.main.len())
            .saturating_add(owned.post.len());
        let InstructionBundle {
            mut compute_budget,
            mut pre,
            mut main,
            mut post,
            ..
        } = owned;
        let mut combined = Vec::with_capacity(total_len);
        combined.append(&mut compute_budget);
        combined.append(&mut pre);
        combined.append(&mut main);
        combined.append(&mut post);
        combined
    }

    /// 根据扁平化后的指令构建 bundle，自动识别前缀的 compute budget 指令与相关配置。
    pub fn from_instructions(mut instructions: Vec<Instruction>) -> Self {
        let compute_count = instructions
            .iter()
            .take_while(|ix| crate::instructions::compute_budget::is_compute_budget(ix))
            .count();

        let mut compute_instructions: Vec<Instruction> =
            instructions.drain(..compute_count).collect();
        let mut compute_unit_limit = None;
        let mut compute_unit_price = None;
        let mut extra_compute_budget = Vec::new();

        for ix in compute_instructions.drain(..) {
            if ix.program_id != COMPUTE_BUDGET_PROGRAM_ID {
                extra_compute_budget.push(ix);
                continue;
            }
            match ix.data.first().copied() {
                Some(2) if ix.data.len() >= 5 => {
                    let mut bytes = [0u8; 4];
                    bytes.copy_from_slice(&ix.data[1..5]);
                    compute_unit_limit = Some(u32::from_le_bytes(bytes));
                }
                Some(3) if ix.data.len() >= 9 => {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&ix.data[1..9]);
                    compute_unit_price = Some(u64::from_le_bytes(bytes));
                }
                _ => extra_compute_budget.push(ix),
            }
        }

        Self {
            compute_budget: Vec::new(),
            pre: Vec::new(),
            main: instructions,
            post: Vec::new(),
            lookup_addresses: Vec::new(),
            resolved_lookups: Vec::new(),
            compute_unit_limit,
            compute_unit_price,
            extra_compute_budget,
            compute_budget_dirty: true,
        }
    }
}

impl InstructionBundle {
    fn ensure_compute_budget(&mut self) {
        if !self.compute_budget_dirty {
            return;
        }
        self.compute_budget.clear();

        let price = self.compute_unit_price.unwrap_or(0);
        let limit = self.compute_unit_limit.unwrap_or(0);
        let sequence = compute_budget_sequence(price, limit, None);
        self.compute_budget.extend(sequence.into_iter());
        self.compute_budget
            .extend(self.extra_compute_budget.iter().cloned());
        self.compute_budget_dirty = false;
    }
}
