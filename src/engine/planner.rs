use std::fmt;

use std::sync::Arc;

use serde::de::{Error as DeError, Visitor};
use serde::{Deserialize, Deserializer};
use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::compiled_instruction::CompiledInstruction;
use solana_sdk::message::{AddressLookupTableAccount, VersionedMessage};
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::VersionedTransaction;
use tracing::warn;

use super::COMPUTE_BUDGET_PROGRAM_ID;
use super::builder::PreparedTransaction;
use super::types::JitoTipPlan;

pub type VariantId = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DispatchStrategy {
    AllAtOnce,
    OneByOne,
}

impl DispatchStrategy {
    pub fn as_str(self) -> &'static str {
        match self {
            DispatchStrategy::AllAtOnce => "all_at_once",
            DispatchStrategy::OneByOne => "one_by_one",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "allatonce" | "all_at_once" | "all-at-once" => Some(DispatchStrategy::AllAtOnce),
            "onebyone" | "one_by_one" | "one-by-one" => Some(DispatchStrategy::OneByOne),
            _ => None,
        }
    }
}

impl Default for DispatchStrategy {
    fn default() -> Self {
        DispatchStrategy::AllAtOnce
    }
}

impl fmt::Display for DispatchStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for DispatchStrategy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StrategyVisitor;

        impl<'de> Visitor<'de> for StrategyVisitor {
            type Value = DispatchStrategy;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("one of: AllAtOnce, OneByOne")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                DispatchStrategy::from_str(value)
                    .ok_or_else(|| DeError::unknown_variant(value, &["AllAtOnce", "OneByOne"]))
            }
        }

        deserializer.deserialize_str(StrategyVisitor)
    }
}

#[derive(Clone, Debug)]
pub struct TxVariant {
    id: VariantId,
    transaction: VersionedTransaction,
    blockhash: Hash,
    slot: u64,
    last_valid_block_height: Option<u64>,
    signer: Arc<Keypair>,
    base_tip_lamports: u64,
    instructions: Vec<Instruction>,
    lookup_accounts: Vec<AddressLookupTableAccount>,
    jito_tip_plan: Option<JitoTipPlan>,
    tip_strategy_label: &'static str,
    compute_unit_price_strategy_label: &'static str,
    prioritization_fee_lamports: u64,
    compute_unit_price_micro_lamports: Option<u64>,
}

impl TxVariant {
    pub fn new(
        id: VariantId,
        transaction: VersionedTransaction,
        blockhash: Hash,
        slot: u64,
        last_valid_block_height: Option<u64>,
        signer: Arc<Keypair>,
        base_tip_lamports: u64,
        instructions: Vec<Instruction>,
        lookup_accounts: Vec<AddressLookupTableAccount>,
        jito_tip_plan: Option<JitoTipPlan>,
        tip_strategy_label: &'static str,
        compute_unit_price_strategy_label: &'static str,
        prioritization_fee_lamports: u64,
        compute_unit_price_micro_lamports: Option<u64>,
    ) -> Self {
        Self {
            id,
            transaction,
            blockhash,
            slot,
            last_valid_block_height,
            signer,
            base_tip_lamports,
            instructions,
            lookup_accounts,
            jito_tip_plan,
            tip_strategy_label,
            compute_unit_price_strategy_label,
            prioritization_fee_lamports,
            compute_unit_price_micro_lamports,
        }
    }

    pub fn id(&self) -> VariantId {
        self.id
    }

    pub fn transaction(&self) -> &VersionedTransaction {
        &self.transaction
    }

    pub fn blockhash(&self) -> Hash {
        self.blockhash
    }

    pub fn slot(&self) -> u64 {
        self.slot
    }

    pub fn last_valid_block_height(&self) -> Option<u64> {
        self.last_valid_block_height
    }

    pub fn signer(&self) -> Arc<Keypair> {
        self.signer.clone()
    }

    pub fn tip_lamports(&self) -> u64 {
        self.base_tip_lamports
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn lookup_accounts(&self) -> &[AddressLookupTableAccount] {
        &self.lookup_accounts
    }

    pub fn signature(&self) -> Option<String> {
        self.transaction
            .signatures
            .get(0)
            .map(|sig| sig.to_string())
    }

    pub fn jito_tip_plan(&self) -> Option<&JitoTipPlan> {
        self.jito_tip_plan.as_ref()
    }

    pub fn tip_strategy_label(&self) -> &'static str {
        self.tip_strategy_label
    }

    pub fn compute_unit_price_strategy_label(&self) -> &'static str {
        self.compute_unit_price_strategy_label
    }

    pub fn prioritization_fee_lamports(&self) -> u64 {
        self.prioritization_fee_lamports
    }

    pub fn compute_unit_price_micro_lamports(&self) -> Option<u64> {
        self.compute_unit_price_micro_lamports
    }
}

#[derive(Clone, Debug)]
pub struct DispatchPlan {
    strategy: DispatchStrategy,
    lander_variants: Vec<Vec<TxVariant>>,
}

impl DispatchPlan {
    pub fn new(strategy: DispatchStrategy, lander_variants: Vec<Vec<TxVariant>>) -> Self {
        Self {
            strategy,
            lander_variants,
        }
    }

    pub fn strategy(&self) -> DispatchStrategy {
        self.strategy
    }

    pub fn variants_for_lander(&self, index: usize) -> &[TxVariant] {
        self.lander_variants
            .get(index)
            .map(|variants| variants.as_slice())
            .unwrap_or(&[])
    }

    pub fn is_empty(&self) -> bool {
        self.lander_variants
            .iter()
            .all(|variants| variants.is_empty())
    }

    pub fn primary_variant(&self) -> Option<&TxVariant> {
        self.lander_variants
            .iter()
            .flat_map(|variants| variants.iter())
            .next()
    }
}

#[derive(Default)]
pub struct TxVariantPlanner;

impl TxVariantPlanner {
    pub fn new() -> Self {
        Self
    }

    pub fn plan(
        &self,
        strategy: DispatchStrategy,
        prepared: &[PreparedTransaction],
        layout: &[usize],
    ) -> DispatchPlan {
        let mut lander_variants = Vec::with_capacity(layout.len());
        let mut next_id: VariantId = 0;

        for (lander_idx, &count) in layout.iter().enumerate() {
            let needed = count.max(1);
            let mut variants = Vec::with_capacity(needed);

            let prepared_entry = prepared
                .get(lander_idx)
                .or_else(|| prepared.last())
                .expect("prepared transactions missing for variant planning");

            for variant_index in 0..needed {
                let mut variant_tx = prepared_entry.transaction.clone();
                let mut variant_instructions = prepared_entry.instructions.clone();

                if variant_index > 0
                    && (matches!(strategy, DispatchStrategy::OneByOne) || needed > 1)
                {
                    let bump = variant_index as u32;
                    apply_variation(
                        &mut variant_tx,
                        &mut variant_instructions,
                        &prepared_entry.signer,
                        bump,
                        prepared_entry.blockhash,
                    );
                }

                let variant = TxVariant::new(
                    next_id,
                    variant_tx,
                    prepared_entry.blockhash,
                    prepared_entry.slot,
                    prepared_entry.last_valid_block_height,
                    prepared_entry.signer.clone(),
                    prepared_entry.tip_lamports,
                    variant_instructions,
                    prepared_entry.lookup_accounts.clone(),
                    prepared_entry.jito_tip_plan.clone(),
                    prepared_entry.tip_strategy_label,
                    prepared_entry.compute_unit_price_strategy_label,
                    prepared_entry.prioritization_fee_lamports,
                    prepared_entry.compute_unit_price_micro_lamports,
                );
                variants.push(variant);
                next_id = next_id.saturating_add(1);
            }
            lander_variants.push(variants);
        }

        DispatchPlan::new(strategy, lander_variants)
    }
}

fn apply_variation(
    tx: &mut VersionedTransaction,
    instructions: &mut [Instruction],
    signer: &Arc<Keypair>,
    bump: u32,
    blockhash: Hash,
) -> bool {
    if bump == 0 {
        return false;
    }

    let Some(index) = adjust_instruction_list(instructions, bump) else {
        return false;
    };

    match &mut tx.message {
        VersionedMessage::Legacy(message) => {
            adjust_compiled_at(&mut message.instructions, index, bump);
        }
        VersionedMessage::V0(message) => {
            adjust_compiled_at(&mut message.instructions, index, bump);
        }
    }

    if let Err(err) = resign_variant(tx, signer, blockhash) {
        warn!(
            target: "engine::planner",
            error = %err,
            "重新签名 one_by_one 变体失败"
        );
    }

    true
}

fn adjust_instruction_list(instructions: &mut [Instruction], bump: u32) -> Option<usize> {
    for (idx, instruction) in instructions.iter_mut().enumerate() {
        if instruction.program_id != COMPUTE_BUDGET_PROGRAM_ID {
            continue;
        }
        if adjust_budget_payload(instruction.data.as_mut_slice(), bump) {
            return Some(idx);
        }
    }
    None
}

fn adjust_compiled_at(instructions: &mut [CompiledInstruction], idx: usize, bump: u32) -> bool {
    if let Some(instruction) = instructions.get_mut(idx) {
        adjust_budget_payload(instruction.data.as_mut_slice(), bump)
    } else {
        false
    }
}

fn adjust_budget_payload(data: &mut [u8], bump: u32) -> bool {
    if data.first() != Some(&2) {
        return false;
    }
    if data.len() < 5 {
        return false;
    }
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&data[1..5]);
    let current = u32::from_le_bytes(buf);
    let updated = current.saturating_add(bump.max(1));
    data[1..5].copy_from_slice(&updated.to_le_bytes());
    true
}

fn resign_variant(
    tx: &mut VersionedTransaction,
    signer: &Arc<Keypair>,
    blockhash: Hash,
) -> Result<(), Box<dyn std::error::Error>> {
    let signer_ref: &dyn Signer = signer.as_ref();
    let resigned = VersionedTransaction::try_new(tx.message.clone(), &[signer_ref])?;
    // ensure blockhash matches expected value; VersionedMessage already carries it.
    if *resigned.message.recent_blockhash() != blockhash {
        return Err("blockhash mismatch after resign".into());
    }
    *tx = resigned;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::assembly::decorators::GuardStrategy;
    use solana_sdk::message::{Message, VersionedMessage};
    use solana_sdk::signature::Signer;

    fn build_prepared() -> PreparedTransaction {
        let signer = Arc::new(Keypair::new());
        let payer = signer.pubkey();
        let message = Message::new(&[], Some(&payer));
        let versioned = VersionedMessage::Legacy(message);
        let transaction = VersionedTransaction::try_new(versioned, &[signer.as_ref()]).unwrap();

        PreparedTransaction {
            transaction,
            blockhash: Hash::default(),
            slot: 0,
            last_valid_block_height: None,
            signer,
            tip_lamports: 5,
            prioritization_fee_lamports: 77,
            guard_lamports: 42,
            guard_strategy: GuardStrategy::BasePlusTip,
            compute_unit_price_micro_lamports: Some(123),
            tip_strategy_label: "opportunity",
            compute_unit_price_strategy_label: "fixed",
            instructions: Vec::new(),
            lookup_accounts: Vec::new(),
            jito_tip_plan: None,
        }
    }

    #[test]
    fn planner_all_at_once_creates_single_variant() {
        let planner = TxVariantPlanner::new();
        let prepared = build_prepared();
        let plan = planner.plan(DispatchStrategy::AllAtOnce, &[prepared.clone()], &[1]);
        assert_eq!(plan.strategy(), DispatchStrategy::AllAtOnce);
        assert_eq!(plan.variants_for_lander(0).len(), 1);
        assert_eq!(plan.primary_variant().unwrap().id(), 0);
        let variant = plan.primary_variant().unwrap();
        assert_eq!(variant.tip_lamports(), 5);
        assert_eq!(variant.tip_strategy_label(), "opportunity");
        assert_eq!(variant.prioritization_fee_lamports(), 77);
        assert_eq!(variant.compute_unit_price_micro_lamports(), Some(123));
        assert_eq!(variant.compute_unit_price_strategy_label(), "fixed");
    }

    #[test]
    fn planner_one_by_one_respects_budget() {
        let planner = TxVariantPlanner::new();
        let prepared = build_prepared();
        let plan = planner.plan(
            DispatchStrategy::OneByOne,
            &[prepared.clone(), prepared],
            &[2, 1],
        );
        assert_eq!(plan.strategy(), DispatchStrategy::OneByOne);
        let first_group = plan.variants_for_lander(0);
        assert_eq!(first_group.len(), 2);
        let second_group = plan.variants_for_lander(1);
        assert_eq!(second_group.len(), 1);

        let groups = [plan.variants_for_lander(0), plan.variants_for_lander(1)];

        let mut expected: VariantId = 0;
        for variants in groups.iter() {
            for variant in *variants {
                assert_eq!(variant.id(), expected);
                expected = expected.saturating_add(1);
            }
        }
    }
}
