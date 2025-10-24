use std::fmt;

use std::sync::Arc;

use serde::de::{Error as DeError, Visitor};
use serde::{Deserialize, Deserializer};
use solana_sdk::hash::Hash;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::VersionedTransaction;

use super::builder::PreparedTransaction;

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
pub struct TipOverride {
    lamports: u64,
    recipient: Option<Pubkey>,
}

impl TipOverride {
    #[allow(dead_code)]
    pub fn new(lamports: u64, recipient: Option<Pubkey>) -> Self {
        Self {
            lamports,
            recipient,
        }
    }

    pub fn lamports(&self) -> u64 {
        self.lamports
    }

    pub fn recipient(&self) -> Option<Pubkey> {
        self.recipient
    }
}

#[derive(Clone, Debug)]
pub struct TxVariant {
    id: VariantId,
    transaction: VersionedTransaction,
    blockhash: Hash,
    slot: u64,
    signer: Arc<Keypair>,
    base_tip_lamports: u64,
    tip_override: Option<TipOverride>,
    instructions: Vec<Instruction>,
    lookup_accounts: Vec<AddressLookupTableAccount>,
}

impl TxVariant {
    pub fn new(
        id: VariantId,
        transaction: VersionedTransaction,
        blockhash: Hash,
        slot: u64,
        signer: Arc<Keypair>,
        base_tip_lamports: u64,
        instructions: Vec<Instruction>,
        lookup_accounts: Vec<AddressLookupTableAccount>,
    ) -> Self {
        Self {
            id,
            transaction,
            blockhash,
            slot,
            signer,
            base_tip_lamports,
            tip_override: None,
            instructions,
            lookup_accounts,
        }
    }

    #[allow(dead_code)]
    pub fn with_tip_override(mut self, tip_override: TipOverride) -> Self {
        self.tip_override = Some(tip_override);
        self
    }

    pub fn id(&self) -> VariantId {
        self.id
    }

    pub fn transaction(&self) -> &VersionedTransaction {
        &self.transaction
    }

    #[allow(dead_code)]
    pub fn into_transaction(self) -> VersionedTransaction {
        self.transaction
    }

    pub fn blockhash(&self) -> Hash {
        self.blockhash
    }

    pub fn slot(&self) -> u64 {
        self.slot
    }

    pub fn signer(&self) -> Arc<Keypair> {
        self.signer.clone()
    }

    pub fn tip_lamports(&self) -> u64 {
        self.base_tip_lamports
    }

    pub fn tip_override(&self) -> Option<&TipOverride> {
        self.tip_override.as_ref()
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
        prepared: &PreparedTransaction,
        layout: &[usize],
    ) -> DispatchPlan {
        let mut lander_variants = Vec::with_capacity(layout.len());
        let mut next_id: VariantId = 0;

        for &count in layout {
            let needed = count.max(1);
            let mut variants = Vec::with_capacity(needed);
            for _ in 0..needed {
                let variant = TxVariant::new(
                    next_id,
                    prepared.transaction.clone(),
                    prepared.blockhash,
                    prepared.slot,
                    prepared.signer.clone(),
                    prepared.tip_lamports,
                    prepared.instructions.clone(),
                    prepared.lookup_accounts.clone(),
                );
                variants.push(variant);
                next_id = next_id.saturating_add(1);
            }
            lander_variants.push(variants);
        }

        DispatchPlan::new(strategy, lander_variants)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            signer,
            tip_lamports: 0,
            instructions: Vec::new(),
            lookup_accounts: Vec::new(),
        }
    }

    #[test]
    fn planner_all_at_once_creates_single_variant() {
        let planner = TxVariantPlanner::new();
        let prepared = build_prepared();
        let plan = planner.plan(DispatchStrategy::AllAtOnce, &prepared, &[1]);
        assert_eq!(plan.strategy(), DispatchStrategy::AllAtOnce);
        assert_eq!(plan.variants_for_lander(0).len(), 1);
        assert_eq!(plan.primary_variant().unwrap().id(), 0);
    }

    #[test]
    fn planner_one_by_one_respects_budget() {
        let planner = TxVariantPlanner::new();
        let prepared = build_prepared();
        let plan = planner.plan(DispatchStrategy::OneByOne, &prepared, &[2, 1]);
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
