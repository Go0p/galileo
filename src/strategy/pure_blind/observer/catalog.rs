#![allow(dead_code)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use dashmap::DashMap;
use parking_lot::Mutex;
use solana_sdk::pubkey::Pubkey;
use tokio::sync::broadcast;

use super::profile::{PoolKey, PoolObservation, PoolProfile, PoolStats, PoolStatsSnapshot};

pub struct PoolCatalog {
    entries: DashMap<PoolKey, PoolRecord>,
    policy: PoolActivationPolicy,
    events: Mutex<Vec<PoolCatalogEvent>>,
    event_tx: broadcast::Sender<PoolCatalogEvent>,
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
}

impl PoolCatalog {
    pub fn new(policy: PoolActivationPolicy, event_capacity: usize) -> Self {
        let capacity = event_capacity.max(16);
        let (event_tx, _) = broadcast::channel(capacity);
        Self {
            entries: DashMap::new(),
            policy,
            events: Mutex::new(Vec::new()),
            event_tx,
        }
    }
}

impl Default for PoolCatalog {
    fn default() -> Self {
        Self::new(PoolActivationPolicy::default(), 1024)
    }
}

impl PoolCatalog {
    pub fn ingest(&self, observation: PoolObservation<'_>) -> PoolCatalogUpdate {
        use dashmap::mapref::entry::Entry;
        let now = Instant::now();

        match self.entries.entry(observation.key.clone()) {
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
        }
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
        if self.profile.remaining_accounts.is_empty() && !observation.remaining_accounts.is_empty()
        {
            let mut_profile = Arc::make_mut(&mut self.profile);
            *mut_profile = PoolProfile::new(
                observation.key.clone(),
                observation.swap.clone(),
                observation.swap_variant.to_string(),
                observation.swap_payload.clone(),
                observation.input_index,
                observation.output_index,
                Arc::new(observation.remaining_accounts.to_vec()),
            );
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
