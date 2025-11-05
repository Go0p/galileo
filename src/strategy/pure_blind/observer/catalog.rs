#![allow(dead_code)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::monitoring::events;
use dashmap::DashMap;
use parking_lot::Mutex;
use solana_sdk::pubkey::Pubkey;
use tokio::sync::broadcast;

use super::profile::{PoolKey, PoolObservation, PoolProfile, PoolStats, PoolStatsSnapshot};
use super::snapshot::{PoolSnapshot, PoolSnapshotEntry, PoolSnapshotPayload, SNAPSHOT_VERSION};

pub struct PoolCatalog {
    entries: DashMap<PoolKey, PoolRecord>,
    policy: PoolActivationPolicy,
    events: Mutex<Vec<PoolCatalogEvent>>,
    event_tx: broadcast::Sender<PoolCatalogEvent>,
    max_entries: usize,
}

#[derive(Clone, Debug)]
pub struct PoolCatalogUpdate {
    pub is_new: bool,
    pub stats: PoolStatsSnapshot,
}

#[derive(Clone, Debug)]
pub struct ActivePool {
    pub profile: Arc<PoolProfile>,
    pub stats: PoolStatsSnapshot,
    pub score: f64,
}

#[derive(Clone, Debug)]
struct PoolRecord {
    profile: Arc<PoolProfile>,
    stats: PoolStats,
    activation: ActivationState,
}

#[derive(Clone, Debug)]
struct ActivationState {
    active: bool,
    consecutive_failures: u32,
    last_observed_at: Option<Instant>,
}

#[derive(Clone, Debug)]
pub struct PoolActivationPolicy {
    min_hits: u64,
    min_estimated_profit: Option<i128>,
    decay_duration: Duration,
}

impl PoolActivationPolicy {
    pub fn new(
        min_hits: u64,
        min_estimated_profit: Option<i128>,
        decay_duration: Duration,
    ) -> Self {
        Self {
            min_hits,
            min_estimated_profit,
            decay_duration,
        }
    }

    fn should_activate(&self, stats: &PoolStats) -> bool {
        if stats.observations < self.min_hits {
            return false;
        }
        if let Some(min_profit) = self.min_estimated_profit {
            if stats.estimated_profit_total < min_profit {
                return false;
            }
        }
        true
    }

    fn is_expired(&self, state: &ActivationState, now: Instant) -> bool {
        if self.decay_duration.is_zero() {
            return false;
        }
        match state.last_observed_at {
            Some(last) => now.duration_since(last) >= self.decay_duration,
            None => false,
        }
    }

    fn decay_duration(&self) -> Duration {
        self.decay_duration
    }
}

impl Default for PoolActivationPolicy {
    fn default() -> Self {
        Self {
            min_hits: 1,
            min_estimated_profit: None,
            decay_duration: Duration::from_secs(60),
        }
    }
}

#[derive(Clone, Debug)]
pub enum PoolCatalogEvent {
    Activated {
        profile: Arc<PoolProfile>,
        stats: PoolStatsSnapshot,
    },
    Deactivated {
        profile: Arc<PoolProfile>,
        stats: PoolStatsSnapshot,
        reason: DeactivateReason,
    },
}

#[derive(Clone, Debug)]
pub enum DeactivateReason {
    Decay,
    Failure,
    Pruned,
}

impl PoolCatalog {
    pub fn new(policy: PoolActivationPolicy, event_capacity: usize, max_entries: usize) -> Self {
        let capacity = event_capacity.max(16);
        let (event_tx, _) = broadcast::channel(capacity);
        Self {
            entries: DashMap::new(),
            policy,
            events: Mutex::new(Vec::new()),
            event_tx,
            max_entries,
        }
    }
}

impl Default for PoolCatalog {
    fn default() -> Self {
        Self::new(PoolActivationPolicy::default(), 1024, 0)
    }
}

impl PoolCatalog {
    pub fn ingest(&self, observation: PoolObservation<'_>) -> PoolCatalogUpdate {
        use dashmap::mapref::entry::Entry;
        let now = Instant::now();

        let update = match self.entries.entry(observation.key.clone()) {
            Entry::Occupied(mut entry) => {
                let record = entry.get_mut();
                record.update(observation, now);
                if let Some(event) = record.evaluate_activation(&self.policy) {
                    self.push_event(event);
                }
                PoolCatalogUpdate {
                    is_new: false,
                    stats: record.stats.snapshot(),
                }
            }
            Entry::Vacant(entry) => {
                let mut record = PoolRecord::from_observation(observation, now);
                if let Some(event) = record.evaluate_activation(&self.policy) {
                    self.push_event(event);
                }
                let stats = record.stats.snapshot();
                entry.insert(record);
                PoolCatalogUpdate {
                    is_new: true,
                    stats,
                }
            }
        };
        self.enforce_capacity();
        update
    }

    pub fn get_profile(&self, key: &PoolKey) -> Option<Arc<PoolProfile>> {
        self.entries
            .get(key)
            .map(|entry| Arc::clone(&entry.profile))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn iter_profiles(&self) -> Vec<Arc<PoolProfile>> {
        self.entries
            .iter()
            .map(|entry| Arc::clone(&entry.profile))
            .collect()
    }

    pub fn record_failure(&self, key: &PoolKey, slot: u64) {
        let now = Instant::now();
        if let Some(mut entry) = self.entries.get_mut(key) {
            entry.stats.record(slot, None);
            if entry.activation.register_failure(now) {
                let stats = entry.stats.snapshot();
                let profile = Arc::clone(&entry.profile);
                self.push_event(PoolCatalogEvent::Deactivated {
                    profile,
                    stats,
                    reason: DeactivateReason::Failure,
                });
            }
        }
    }

    pub fn remove(&self, key: &PoolKey) -> Option<Arc<PoolProfile>> {
        self.entries.remove(key).map(|(_, record)| record.profile)
    }

    pub fn enforce_decay(&self) {
        let now = Instant::now();
        if self.policy.decay_duration().is_zero() {
            return;
        }

        for mut entry in self.entries.iter_mut() {
            if entry.activation.active && self.policy.is_expired(&entry.activation, now) {
                entry.activation.active = false;
                let stats = entry.stats.snapshot();
                let profile = Arc::clone(&entry.profile);
                self.push_event(PoolCatalogEvent::Deactivated {
                    profile,
                    stats,
                    reason: DeactivateReason::Decay,
                });
            }
        }
    }

    pub fn drain_events(&self) -> Vec<PoolCatalogEvent> {
        let mut guard = self.events.lock();
        if guard.is_empty() {
            Vec::new()
        } else {
            guard.drain(..).collect()
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<PoolCatalogEvent> {
        self.event_tx.subscribe()
    }

    pub fn active_profiles(&self) -> Vec<Arc<PoolProfile>> {
        self.entries
            .iter()
            .filter_map(|entry| {
                if entry.activation.active {
                    Some(Arc::clone(&entry.profile))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn active_pools(&self) -> Vec<ActivePool> {
        let now = Instant::now();
        self.entries
            .iter()
            .filter_map(|entry| {
                if entry.activation.active {
                    let stats = entry.stats.snapshot();
                    let score = compute_score(&stats, &entry.activation, &self.policy, now);
                    Some(ActivePool {
                        profile: Arc::clone(&entry.profile),
                        stats,
                        score,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn push_event(&self, event: PoolCatalogEvent) {
        let mut guard = self.events.lock();
        guard.push(event.clone());
        let _ = self.event_tx.send(event);
    }

    pub fn export_snapshot(&self, generated_at: u64) -> PoolSnapshot {
        let mut entries: Vec<PoolSnapshotEntry> = self
            .entries
            .iter()
            .filter(|entry| entry.value().activation.active)
            .map(|entry| {
                let value = entry.value();
                PoolSnapshotEntry {
                    payload: PoolSnapshotPayload::from_profile(&value.profile),
                    stats: value.stats.snapshot(),
                }
            })
            .collect();

        if self.max_entries > 0 && entries.len() > self.max_entries {
            entries.sort_by(|a, b| {
                let slot_a = a.stats.last_seen_slot.unwrap_or(0);
                let slot_b = b.stats.last_seen_slot.unwrap_or(0);
                slot_a
                    .cmp(&slot_b)
                    .then(a.stats.observations.cmp(&b.stats.observations))
            });
            entries.truncate(self.max_entries);
        }

        PoolSnapshot {
            version: SNAPSHOT_VERSION,
            generated_at,
            entries,
        }
    }

    pub fn ingest_snapshot(&self, snapshot: PoolSnapshot) {
        if snapshot.version != SNAPSHOT_VERSION {
            return;
        }

        let now = Instant::now();
        for entry in snapshot.entries {
            let PoolSnapshotEntry {
                payload,
                stats: snapshot_stats,
            } = entry;
            let profile_inner = payload.into_profile();
            let key = profile_inner.key.clone();
            let profile = Arc::new(profile_inner);
            let mut record_stats = PoolStats::default();
            record_stats.observations = snapshot_stats.observations;
            record_stats.first_seen_slot = snapshot_stats.first_seen_slot;
            record_stats.last_seen_slot = snapshot_stats.last_seen_slot;
            record_stats.estimated_profit_total = snapshot_stats.estimated_profit_total;

            let is_active = self.policy.should_activate(&record_stats);
            let record = PoolRecord {
                profile,
                stats: record_stats,
                activation: ActivationState {
                    active: is_active,
                    consecutive_failures: 0,
                    last_observed_at: Some(now),
                },
            };
            if is_active {
                self.push_event(PoolCatalogEvent::Activated {
                    profile: Arc::clone(&record.profile),
                    stats: record.stats.snapshot(),
                });
            }
            self.entries.insert(key, record);
        }

        self.enforce_capacity();
    }

    fn enforce_capacity(&self) {
        if self.max_entries == 0 {
            return;
        }
        let current = self.entries.len();
        if current <= self.max_entries {
            return;
        }

        let remove_count = current - self.max_entries;
        let mut candidates: Vec<(PoolKey, PoolStatsSnapshot)> = self
            .entries
            .iter()
            .map(|entry| {
                let stats = entry.value().stats.snapshot();
                (entry.key().clone(), stats)
            })
            .collect();

        candidates.sort_by(|a, b| {
            let slot_a = a.1.last_seen_slot.unwrap_or(0);
            let slot_b = b.1.last_seen_slot.unwrap_or(0);
            slot_a
                .cmp(&slot_b)
                .then(a.1.observations.cmp(&b.1.observations))
        });

        let mut pruned = 0;
        for (key, _) in candidates.into_iter().take(remove_count) {
            if let Some((_, record)) = self.entries.remove(&key) {
                let stats = record.stats.snapshot();
                let event = PoolCatalogEvent::Deactivated {
                    profile: record.profile,
                    stats,
                    reason: DeactivateReason::Pruned,
                };
                self.push_event(event);
                pruned += 1;
            }
        }
        if pruned > 0 {
            events::pure_blind_cache_pruned("pool", pruned);
        }
    }
}

impl PoolRecord {
    fn from_observation(observation: PoolObservation<'_>, now: Instant) -> Self {
        let profile = PoolProfile::new(
            observation.key.clone(),
            observation.swap.clone(),
            observation.swap_variant.to_string(),
            observation.swap_payload.clone(),
            observation.input_index,
            observation.output_index,
            observation.input_asset,
            observation.output_asset,
            Arc::new(observation.lookup_tables.to_vec()),
            Arc::new(observation.remaining_accounts.to_vec()),
        );
        let mut stats = PoolStats::default();
        stats.record(observation.slot, observation.estimated_profit);
        let activation = ActivationState::new(now);

        Self {
            profile: Arc::new(profile),
            stats,
            activation,
        }
    }

    fn update(&mut self, observation: PoolObservation<'_>, now: Instant) {
        let mut_profile = Arc::make_mut(&mut self.profile);

        if mut_profile.remaining_accounts.is_empty() && !observation.remaining_accounts.is_empty() {
            *Arc::make_mut(&mut mut_profile.remaining_accounts) =
                observation.remaining_accounts.to_vec();
        }

        if let Some(asset) = observation.input_asset {
            if mut_profile.input_asset.is_none() {
                mut_profile.input_asset = Some(asset);
            }
        }
        if let Some(asset) = observation.output_asset {
            if mut_profile.output_asset.is_none() {
                mut_profile.output_asset = Some(asset);
            }
        }

        if !observation.lookup_tables.is_empty() {
            let tables = Arc::make_mut(&mut mut_profile.lookup_tables);
            for table in observation.lookup_tables {
                if !tables.contains(table) {
                    tables.push(*table);
                }
            }
        }

        self.stats
            .record(observation.slot, observation.estimated_profit);
        self.activation.register_observation(now);
    }

    fn evaluate_activation(&mut self, policy: &PoolActivationPolicy) -> Option<PoolCatalogEvent> {
        if policy.should_activate(&self.stats) && !self.activation.active {
            self.activation.active = true;
            return Some(PoolCatalogEvent::Activated {
                profile: Arc::clone(&self.profile),
                stats: self.stats.snapshot(),
            });
        }
        None
    }
}

#[allow(dead_code)]
pub fn default_pool_key(discriminant: u8) -> PoolKey {
    PoolKey::new("Unknown", None, None, None, None, discriminant)
}

#[allow(dead_code)]
pub fn pool_key_with_program(
    program: Pubkey,
    pool: Option<Pubkey>,
    input_mint: Option<Pubkey>,
    output_mint: Option<Pubkey>,
    discriminant: u8,
) -> PoolKey {
    PoolKey::new(
        "Custom",
        Some(program),
        pool,
        input_mint,
        output_mint,
        discriminant,
    )
}

impl ActivationState {
    fn new(now: Instant) -> Self {
        Self {
            active: false,
            consecutive_failures: 0,
            last_observed_at: Some(now),
        }
    }

    fn register_observation(&mut self, now: Instant) {
        self.last_observed_at = Some(now);
        self.consecutive_failures = 0;
    }

    fn register_failure(&mut self, now: Instant) -> bool {
        self.last_observed_at = Some(now);
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        if self.active && self.consecutive_failures > 0 {
            self.active = false;
            true
        } else {
            false
        }
    }
}

fn compute_score(
    stats: &PoolStatsSnapshot,
    state: &ActivationState,
    policy: &PoolActivationPolicy,
    now: Instant,
) -> f64 {
    let base = stats.observations.max(1) as f64;
    if policy.decay_duration().is_zero() {
        return base;
    }
    let decay_seconds = policy.decay_duration().as_secs_f64();
    if decay_seconds <= f64::EPSILON {
        return base;
    }

    match state.last_observed_at {
        Some(last) => {
            let elapsed = now.duration_since(last).as_secs_f64();
            if elapsed >= decay_seconds {
                0.0
            } else {
                let ratio = 1.0 - (elapsed / decay_seconds);
                base * ratio.max(0.0)
            }
        }
        None => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::jupiter::types::EncodedSwap;
    use crate::strategy::pure_blind::observer::PoolAsset;
    use serde_json::Value;

    fn sample_pool_profile() -> PoolProfile {
        let pool_address = Some(Pubkey::new_unique());
        let input_mint = Some(Pubkey::new_unique());
        let output_mint = Some(Pubkey::new_unique());
        let token_program = Pubkey::new_unique();
        let key = PoolKey::new(
            "TestDex",
            Some(Pubkey::new_unique()),
            pool_address,
            input_mint,
            output_mint,
            1,
        );
        PoolProfile::new(
            key,
            EncodedSwap::simple(1),
            "test".to_string(),
            Value::Null,
            0,
            1,
            Some(PoolAsset::new(input_mint.unwrap(), token_program)),
            Some(PoolAsset::new(output_mint.unwrap(), token_program)),
            Arc::new(Vec::new()),
            Arc::new(Vec::new()),
        )
    }

    #[test]
    fn ingest_snapshot_emits_activation_event() {
        let catalog = PoolCatalog::new(PoolActivationPolicy::default(), 16, 8);
        let profile = sample_pool_profile();
        let snapshot = PoolSnapshot {
            version: SNAPSHOT_VERSION,
            generated_at: 1,
            entries: vec![PoolSnapshotEntry {
                payload: PoolSnapshotPayload::from_profile(&profile),
                stats: PoolStatsSnapshot {
                    observations: 3,
                    first_seen_slot: Some(1),
                    last_seen_slot: Some(2),
                    estimated_profit_total: 42,
                },
            }],
        };

        let mut receiver = catalog.subscribe();
        catalog.ingest_snapshot(snapshot);

        match receiver.try_recv() {
            Ok(PoolCatalogEvent::Activated { .. }) => {}
            other => panic!("expected activation event, got {other:?}"),
        }

        assert_eq!(catalog.active_pools().len(), 1);
    }
}
