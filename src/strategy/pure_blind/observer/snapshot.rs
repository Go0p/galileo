use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

use super::profile::{PoolAsset, PoolKey, PoolProfile, PoolStatsSnapshot};
use super::routes::{RouteProfile, RouteStatsSnapshot};
use crate::instructions::jupiter::types::EncodedSwap;
use serde_json::Value;
use std::sync::Arc;

pub const SNAPSHOT_VERSION: u16 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSnapshot {
    pub version: u16,
    pub generated_at: u64,
    pub entries: Vec<PoolSnapshotEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSnapshotEntry {
    pub payload: PoolSnapshotPayload,
    pub stats: PoolStatsSnapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolSnapshotPayload {
    pub dex_label: String,
    pub dex_program: Option<Pubkey>,
    pub pool_address: Option<Pubkey>,
    pub input_mint: Option<Pubkey>,
    pub output_mint: Option<Pubkey>,
    pub swap_discriminant: u8,
    pub swap: EncodedSwap,
    pub swap_variant: String,
    pub swap_payload: Value,
    pub input_index: u8,
    pub output_index: u8,
    pub input_asset: Option<PoolAsset>,
    pub output_asset: Option<PoolAsset>,
    pub lookup_tables: Vec<Pubkey>,
    pub remaining_accounts: Vec<Pubkey>,
}

impl PoolSnapshotPayload {
    pub fn from_profile(profile: &PoolProfile) -> Self {
        Self {
            dex_label: profile.key.dex_label.to_string(),
            dex_program: profile.key.dex_program,
            pool_address: profile.key.pool_address,
            input_mint: profile.key.input_mint,
            output_mint: profile.key.output_mint,
            swap_discriminant: profile.key.swap_discriminant,
            swap: profile.swap.clone(),
            swap_variant: profile.swap_variant.clone(),
            swap_payload: profile.swap_payload.clone(),
            input_index: profile.input_index,
            output_index: profile.output_index,
            input_asset: profile.input_asset,
            output_asset: profile.output_asset,
            lookup_tables: profile.lookup_tables.as_ref().clone(),
            remaining_accounts: profile.remaining_accounts.as_ref().clone(),
        }
    }

    pub fn into_profile(self) -> PoolProfile {
        let key = PoolKey::from_snapshot(
            &self.dex_label,
            self.dex_program,
            self.pool_address,
            self.input_mint,
            self.output_mint,
            self.swap_discriminant,
        );
        PoolProfile::new(
            key,
            self.swap,
            self.swap_variant,
            self.swap_payload,
            self.input_index,
            self.output_index,
            self.input_asset,
            self.output_asset,
            Arc::new(self.lookup_tables),
            Arc::new(self.remaining_accounts),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSnapshot {
    pub version: u16,
    pub generated_at: u64,
    pub entries: Vec<RouteSnapshotEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSnapshotEntry {
    pub markets: Vec<Pubkey>,
    pub steps: Vec<PoolSnapshotPayload>,
    pub lookup_tables: Vec<Pubkey>,
    pub base_asset: Option<PoolAsset>,
    pub stats: RouteStatsSnapshot,
}

impl RouteSnapshotEntry {
    pub fn from_profile(profile: &RouteProfile, stats: RouteStatsSnapshot) -> Self {
        let steps = profile
            .steps
            .iter()
            .map(PoolSnapshotPayload::from_profile)
            .collect();
        Self {
            markets: profile.markets().to_vec(),
            steps,
            lookup_tables: profile.lookup_tables.as_ref().clone(),
            base_asset: profile.base_asset,
            stats,
        }
    }

    #[allow(dead_code)]
    pub fn to_route_profile(&self) -> RouteProfile {
        let steps: Vec<PoolProfile> = self
            .steps
            .iter()
            .cloned()
            .map(PoolSnapshotPayload::into_profile)
            .collect();
        let key =
            crate::strategy::pure_blind::observer::routes::RouteKey::new(self.markets.clone())
                .expect("route snapshot markets must be valid");
        RouteProfile::new_from_parts(key, steps, self.lookup_tables.clone(), self.base_asset)
    }
}
