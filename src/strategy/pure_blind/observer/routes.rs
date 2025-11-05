use std::sync::Arc;
use std::time::{Duration, Instant};

use super::profile::{PoolAsset, PoolProfile};
use super::snapshot::{PoolSnapshotPayload, RouteSnapshot, RouteSnapshotEntry, SNAPSHOT_VERSION};
use dashmap::{DashMap, mapref::entry::Entry};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct RouteObservation {
    pub steps: Vec<PoolProfile>,
    pub lookup_tables: Vec<Pubkey>,
    pub base_asset: Option<PoolAsset>,
    pub estimated_profit: Option<i128>,
    pub slot: u64,
}

#[derive(Clone, Debug)]
pub struct RouteActivationPolicy {
    min_hits: u64,
    min_estimated_profit: Option<i128>,
    decay_duration: Duration,
}

impl RouteActivationPolicy {
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

    fn should_activate(&self, stats: &RouteStats) -> bool {
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

#[derive(Clone, Debug)]
pub struct RouteProfile {
    pub key: RouteKey,
    pub steps: Arc<Vec<PoolProfile>>,
    pub lookup_tables: Arc<Vec<Pubkey>>,
    pub base_asset: Option<PoolAsset>,
}

impl RouteProfile {
    fn new(observation: &RouteObservation, key: RouteKey) -> Self {
        Self {
            key,
            steps: Arc::new(observation.steps.clone()),
            lookup_tables: Arc::new(observation.lookup_tables.clone()),
            base_asset: observation.base_asset,
        }
    }

    pub fn markets(&self) -> &[Pubkey] {
        &self.key.markets
    }

    pub(crate) fn new_from_parts(
        key: RouteKey,
        steps: Vec<PoolProfile>,
        lookup_tables: Vec<Pubkey>,
        base_asset: Option<PoolAsset>,
    ) -> Self {
        Self {
            key,
            steps: Arc::new(steps),
            lookup_tables: Arc::new(lookup_tables),
            base_asset,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RouteKey {
    markets: Arc<Vec<Pubkey>>,
}

impl RouteKey {
    pub fn new(markets: Vec<Pubkey>) -> Option<Self> {
        if markets.is_empty() || markets.iter().any(|m| m == &Pubkey::default()) {
            return None;
        }
        Some(Self {
            markets: Arc::new(markets),
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct RouteStats {
    pub observations: u64,
    pub first_seen_slot: Option<u64>,
    pub last_seen_slot: Option<u64>,
    pub estimated_profit_total: i128,
}

impl RouteStats {
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

    pub fn snapshot(&self) -> RouteStatsSnapshot {
        RouteStatsSnapshot {
            observations: self.observations,
            first_seen_slot: self.first_seen_slot,
            last_seen_slot: self.last_seen_slot,
            estimated_profit_total: self.estimated_profit_total,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct RouteStatsSnapshot {
    pub observations: u64,
    pub first_seen_slot: Option<u64>,
    pub last_seen_slot: Option<u64>,
    pub estimated_profit_total: i128,
}

#[derive(Clone, Debug)]
struct RouteRecord {
    profile: Arc<RouteProfile>,
    stats: RouteStats,
    activation: ActivationState,
}

#[derive(Clone, Debug)]
struct ActivationState {
    active: bool,
    consecutive_failures: u32,
    last_observed_at: Option<Instant>,
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

    #[allow(dead_code)]
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

#[derive(Clone, Debug)]
pub struct ActiveRoute {
    pub profile: Arc<RouteProfile>,
    pub stats: RouteStatsSnapshot,
    pub score: f64,
}

#[derive(Clone, Debug)]
pub enum RouteCatalogEvent {
    Activated {
        profile: Arc<RouteProfile>,
        stats: RouteStatsSnapshot,
    },
    Deactivated {
        profile: Arc<RouteProfile>,
        _stats: RouteStatsSnapshot,
        reason: RouteDeactivateReason,
    },
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum RouteDeactivateReason {
    Decay,
    Failure,
    Pruned,
}

pub struct RouteCatalog {
    entries: DashMap<RouteKey, RouteRecord>,
    policy: RouteActivationPolicy,
    events: Mutex<Vec<RouteCatalogEvent>>,
    event_tx: broadcast::Sender<RouteCatalogEvent>,
    max_entries: usize,
}

impl RouteCatalog {
    pub fn new(policy: RouteActivationPolicy, capacity: usize, max_entries: usize) -> Self {
        let (event_tx, _) = broadcast::channel(capacity.max(16));
        Self {
            entries: DashMap::new(),
            policy,
            events: Mutex::new(Vec::new()),
            event_tx,
            max_entries,
        }
    }

    pub fn export_snapshot(&self, generated_at: u64) -> RouteSnapshot {
        let mut entries: Vec<RouteSnapshotEntry> = self
            .entries
            .iter()
            .filter(|entry| entry.value().activation.active)
            .map(|entry| {
                let record = entry.value();
                RouteSnapshotEntry::from_profile(&record.profile, record.stats.snapshot())
            })
            .collect();

        if self.max_entries > 0 && entries.len() > self.max_entries {
            entries.sort_by_key(|entry| entry.stats.last_seen_slot.unwrap_or(0));
            entries.truncate(self.max_entries);
        }

        RouteSnapshot {
            version: SNAPSHOT_VERSION,
            generated_at,
            entries,
        }
    }

    pub fn ingest_snapshot(&self, snapshot: RouteSnapshot) {
        if snapshot.version != SNAPSHOT_VERSION {
            return;
        }

        for entry in snapshot.entries {
            let Some(key) = RouteKey::new(entry.markets.clone()) else {
                continue;
            };

            let steps: Vec<PoolProfile> = entry
                .steps
                .into_iter()
                .map(PoolSnapshotPayload::into_profile)
                .collect();

            let profile = Arc::new(RouteProfile::new_from_parts(
                key.clone(),
                steps,
                entry.lookup_tables.clone(),
                entry.base_asset,
            ));

            let mut record = RouteRecord {
                profile,
                stats: RouteStats {
                    observations: entry.stats.observations,
                    first_seen_slot: entry.stats.first_seen_slot,
                    last_seen_slot: entry.stats.last_seen_slot,
                    estimated_profit_total: entry.stats.estimated_profit_total,
                },
                activation: ActivationState {
                    active: true,
                    consecutive_failures: 0,
                    last_observed_at: None,
                },
            };
            if let Some(event) = record.evaluate_activation(&self.policy) {
                self.push_event(event);
            }
            self.entries.insert(key, record);
        }

        self.enforce_capacity();
    }

    pub fn ingest(&self, observation: RouteObservation) {
        if observation.steps.len() < 2 {
            return;
        }

        let markets: Vec<Pubkey> = observation
            .steps
            .iter()
            .filter_map(|step| step.key.pool_address)
            .collect();
        if markets.len() != observation.steps.len() {
            return;
        }

        let Some(key) = RouteKey::new(markets) else {
            return;
        };

        let now = Instant::now();

        let profile = Arc::new(RouteProfile::new(&observation, key.clone()));

        match self.entries.entry(key) {
            Entry::Occupied(mut entry) => {
                let record = entry.get_mut();
                record.profile = Arc::clone(&profile);
                record
                    .stats
                    .record(observation.slot, observation.estimated_profit);
                record.activation.register_observation(now);
                if let Some(event) = record.evaluate_activation(&self.policy) {
                    self.push_event(event);
                }
            }
            Entry::Vacant(entry) => {
                let mut stats = RouteStats::default();
                stats.record(observation.slot, observation.estimated_profit);
                let mut record = RouteRecord {
                    profile,
                    stats,
                    activation: ActivationState::new(now),
                };
                if let Some(event) = record.evaluate_activation(&self.policy) {
                    self.push_event(event);
                }
                entry.insert(record);
            }
        }

        self.enforce_capacity();
    }

    pub fn active_routes(&self) -> Vec<ActiveRoute> {
        let now = Instant::now();
        self.entries
            .iter()
            .filter_map(|entry| {
                let value = entry.value();
                if value.activation.active {
                    let stats = value.stats.snapshot();
                    Some(ActiveRoute {
                        profile: Arc::clone(&value.profile),
                        stats,
                        score: compute_score(&stats, &value.activation, &self.policy, now),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn record_failure(&self, key: &RouteKey, slot: u64) {
        let now = Instant::now();
        if let Some(mut entry) = self.entries.get_mut(key) {
            let value = entry.value_mut();
            value.stats.record(slot, None);
            if value.activation.register_failure(now) {
                let stats = value.stats.snapshot();
                let profile = Arc::clone(&value.profile);
                self.push_event(RouteCatalogEvent::Deactivated {
                    profile,
                    _stats: stats,
                    reason: RouteDeactivateReason::Failure,
                });
            }
        }
    }

    pub fn enforce_decay(&self) {
        let now = Instant::now();
        if self.policy.decay_duration().is_zero() {
            return;
        }

        for mut entry in self.entries.iter_mut() {
            let value = entry.value_mut();
            if value.activation.active && self.policy.is_expired(&value.activation, now) {
                value.activation.active = false;
                let stats = value.stats.snapshot();
                let profile = Arc::clone(&value.profile);
                self.push_event(RouteCatalogEvent::Deactivated {
                    profile,
                    _stats: stats,
                    reason: RouteDeactivateReason::Decay,
                });
            }
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<RouteCatalogEvent> {
        self.event_tx.subscribe()
    }

    fn push_event(&self, event: RouteCatalogEvent) {
        let mut guard = self.events.lock();
        guard.push(event.clone());
        let _ = self.event_tx.send(event);
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
        let mut candidates: Vec<(RouteKey, RouteStatsSnapshot)> = self
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

        for (key, _) in candidates.into_iter().take(remove_count) {
            if let Some((_, record)) = self.entries.remove(&key) {
                let stats = record.stats.snapshot();
                let deactivated = RouteCatalogEvent::Deactivated {
                    profile: record.profile,
                    _stats: stats,
                    reason: RouteDeactivateReason::Pruned,
                };
                self.push_event(deactivated);
            }
        }
    }
}

impl RouteRecord {
    fn evaluate_activation(&mut self, policy: &RouteActivationPolicy) -> Option<RouteCatalogEvent> {
        if policy.should_activate(&self.stats) && !self.activation.active {
            self.activation.active = true;
            return Some(RouteCatalogEvent::Activated {
                profile: Arc::clone(&self.profile),
                stats: self.stats.snapshot(),
            });
        }
        None
    }
}

fn compute_score(
    stats: &RouteStatsSnapshot,
    state: &ActivationState,
    policy: &RouteActivationPolicy,
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
