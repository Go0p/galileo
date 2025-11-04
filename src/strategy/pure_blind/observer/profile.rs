#![allow(dead_code)]

use std::hash::{Hash, Hasher};
use std::sync::Arc;

use serde_json::Value;
use solana_sdk::pubkey::Pubkey;

use crate::instructions::jupiter::decoder::ParsedSwapAccounts;
use crate::instructions::jupiter::types::EncodedSwap;

#[derive(Debug, Clone)]
pub struct PoolProfile {
    pub key: PoolKey,
    pub swap: EncodedSwap,
    pub swap_variant: String,
    pub swap_payload: Value,
    pub input_index: u8,
    pub output_index: u8,
    pub remaining_accounts: Arc<Vec<Pubkey>>,
}

impl PoolProfile {
    pub fn new(
        key: PoolKey,
        swap: EncodedSwap,
        swap_variant: String,
        swap_payload: Value,
        input_index: u8,
        output_index: u8,
        remaining_accounts: Arc<Vec<Pubkey>>,
    ) -> Self {
        Self {
            key,
            swap,
            swap_variant,
            swap_payload,
            input_index,
            output_index,
            remaining_accounts,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    pub observations: u64,
    pub first_seen_slot: Option<u64>,
    pub last_seen_slot: Option<u64>,
    pub estimated_profit_total: i128,
}

impl PoolStats {
    pub fn record(&mut self, slot: u64, estimated_profit: Option<i128>) {
        self.observations = self.observations.saturating_add(1);
        if self.first_seen_slot.is_none() {
            self.first_seen_slot = Some(slot);
        }
        self.last_seen_slot = Some(slot);
        if let Some(profit) = estimated_profit {
            self.estimated_profit_total = self.estimated_profit_total.saturating_add(profit);
        }
    }

    pub fn snapshot(&self) -> PoolStatsSnapshot {
        PoolStatsSnapshot {
            observations: self.observations,
            first_seen_slot: self.first_seen_slot,
            last_seen_slot: self.last_seen_slot,
            estimated_profit_total: self.estimated_profit_total,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PoolStatsSnapshot {
    pub observations: u64,
    pub first_seen_slot: Option<u64>,
    pub last_seen_slot: Option<u64>,
    pub estimated_profit_total: i128,
}

#[derive(Debug, Clone)]
pub struct PoolObservation<'a> {
    pub key: PoolKey,
    pub swap: &'a EncodedSwap,
    pub swap_variant: &'a str,
    pub swap_payload: &'a Value,
    pub remaining_accounts: &'a [Pubkey],
    pub input_index: u8,
    pub output_index: u8,
    pub slot: u64,
    pub estimated_profit: Option<i128>,
}

#[derive(Debug, Clone, Eq)]
pub struct PoolKey {
    pub dex_label: &'static str,
    pub dex_program: Option<Pubkey>,
    pub pool_address: Option<Pubkey>,
    pub input_mint: Option<Pubkey>,
    pub output_mint: Option<Pubkey>,
    pub swap_discriminant: u8,
}

impl PoolKey {
    pub fn new(
        dex_label: &'static str,
        dex_program: Option<Pubkey>,
        pool_address: Option<Pubkey>,
        input_mint: Option<Pubkey>,
        output_mint: Option<Pubkey>,
        swap_discriminant: u8,
    ) -> Self {
        Self {
            dex_label,
            dex_program,
            pool_address,
            input_mint,
            output_mint,
            swap_discriminant,
        }
    }

    pub fn from_parsed_swap(
        parsed: &ParsedSwapAccounts,
        input_mint: Option<Pubkey>,
        output_mint: Option<Pubkey>,
        swap_discriminant: u8,
    ) -> Self {
        Self::new(
            parsed.dex_label(),
            Some(parsed.swap_program()),
            Some(parsed.pool_state()),
            input_mint,
            output_mint,
            swap_discriminant,
        )
    }
}

impl PartialEq for PoolKey {
    fn eq(&self, other: &Self) -> bool {
        self.dex_label == other.dex_label
            && self.dex_program == other.dex_program
            && self.pool_address == other.pool_address
            && self.input_mint == other.input_mint
            && self.output_mint == other.output_mint
            && self.swap_discriminant == other.swap_discriminant
    }
}

impl Hash for PoolKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.dex_label.hash(state);
        self.dex_program.hash(state);
        self.pool_address.hash(state);
        self.input_mint.hash(state);
        self.output_mint.hash(state);
        self.swap_discriminant.hash(state);
    }
}
