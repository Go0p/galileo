#![allow(dead_code)]

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering as AtomicOrdering},
    },
    time::{Duration, Instant},
};

use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};
use tokio::time::{Instant as TokioInstant, sleep_until};

use crate::engine::multi_leg::types::{
    AggregatorKind as MultiLegAggregatorKind, LegSide as MultiLegLegSide,
};
use crate::monitoring::events;

use super::{IpInventory, IpSlot, IpSlotKind, IpSlotState, IpSource, NetworkError, NetworkResult};

const DEFAULT_UNBOUNDED_PERMITS: usize = usize::MAX >> 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpTaskKind {
    QuoteBuy,
    QuoteSell,
    SwapInstruction,
    LanderSubmit {
        endpoint_hash: u64,
    },
    MultiLegLeg {
        aggregator: MultiLegAggregatorKind,
        side: MultiLegLegSide,
    },
    TitanWsConnect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpLeaseMode {
    Ephemeral,
    SharedLongLived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpLeaseOutcome {
    Success,
    RateLimited,
    Timeout,
    NetworkError,
}

#[derive(Debug, Clone, Copy)]
pub struct CooldownConfig {
    pub rate_limited_start: Duration,
    pub timeout_start: Duration,
}

impl Default for CooldownConfig {
    fn default() -> Self {
        Self {
            rate_limited_start: Duration::from_millis(500),
            timeout_start: Duration::from_millis(250),
        }
    }
}

#[derive(Debug)]
pub struct IpAllocator {
    inner: Arc<IpAllocatorInner>,
}

impl IpAllocator {
    pub fn from_inventory(
        inventory: IpInventory,
        per_ip_inflight_limit: Option<usize>,
        cooldown: CooldownConfig,
    ) -> Self {
        let source = inventory.source();
        let slots = inventory.into_slots();
        let per_ip_limit = per_ip_inflight_limit.unwrap_or(1).max(1);
        let slot_states = slots
            .into_iter()
            .map(|slot| Arc::new(SlotState::new(slot, Some(per_ip_limit))))
            .collect::<Vec<_>>();

        for slot in &slot_states {
            events::ip_inventory(slot.ip(), slot.kind_label());
        }

        Self {
            inner: Arc::new(IpAllocatorInner {
                slots: slot_states,
                rotation: AtomicUsize::new(0),
                cooldown,
                per_ip_inflight_limit: Some(per_ip_limit),
                source,
                lease_counter: AtomicU64::new(1),
            }),
        }
    }

    pub fn total_slots(&self) -> usize {
        self.inner.slots.len()
    }

    pub fn has_multiple_slots(&self) -> bool {
        self.total_slots() > 1
    }

    pub fn per_ip_inflight_limit(&self) -> Option<usize> {
        self.inner.per_ip_inflight_limit
    }

    pub fn source(&self) -> IpSource {
        self.inner.source
    }

    pub fn summary(&self) -> IpAllocatorSummary {
        IpAllocatorSummary {
            total_slots: self.total_slots(),
            per_ip_inflight_limit: self.per_ip_inflight_limit(),
            source: self.source(),
        }
    }

    pub fn slot_ips(&self) -> Vec<std::net::IpAddr> {
        self.inner
            .slots
            .iter()
            .map(|slot| slot.ip())
            .collect::<Vec<_>>()
    }

    pub async fn acquire_excluding(
        &self,
        kind: IpTaskKind,
        mode: IpLeaseMode,
        exclude: Option<std::net::IpAddr>,
    ) -> NetworkResult<IpLease> {
        if exclude.is_none() || !self.has_multiple_slots() {
            return self.acquire(kind, mode).await;
        }

        let excluded_ip = exclude.expect("checked above");
        let total_slots = self.total_slots();
        let max_attempts = total_slots.saturating_mul(2).max(1);
        let mut attempts = 0usize;

        loop {
            let lease = self.acquire(kind, mode).await?;
            let handle = lease.handle();
            let ip = handle.ip();
            drop(handle);

            if ip != excluded_ip || attempts + 1 >= max_attempts {
                return Ok(lease);
            }

            attempts += 1;
            drop(lease);
            tokio::task::yield_now().await;
        }
    }

    pub async fn acquire_handle_excluding(
        &self,
        kind: IpTaskKind,
        mode: IpLeaseMode,
        exclude: Option<std::net::IpAddr>,
    ) -> NetworkResult<(IpLeaseHandle, std::net::IpAddr)> {
        let lease = self.acquire_excluding(kind, mode, exclude).await?;
        let handle = lease.handle();
        let ip = handle.ip();
        drop(lease);
        Ok((handle, ip))
    }

    pub async fn acquire(&self, kind: IpTaskKind, mode: IpLeaseMode) -> NetworkResult<IpLease> {
        if self.inner.slots.is_empty() {
            return Err(NetworkError::NoEligibleIp);
        }

        let total_slots = self.inner.slots.len();

        loop {
            let mut earliest_cooldown: Option<Instant> = None;
            let start_idx = self.inner.rotation.fetch_add(1, AtomicOrdering::Relaxed);
            let now = Instant::now();

            for offset in 0..total_slots {
                let slot_idx = (start_idx + offset) % total_slots;
                let slot = &self.inner.slots[slot_idx];

                if let Some(delay) = slot.cooldown_delay(now) {
                    match earliest_cooldown {
                        Some(existing) if delay >= existing => {}
                        _ => earliest_cooldown = Some(delay),
                    }
                    continue;
                }

                match slot.try_acquire(mode) {
                    Ok(permit) => {
                        slot.record_request();
                        let lease_id = self.inner.next_lease_id();
                        return Ok(IpLease::new(
                            Arc::clone(&self.inner),
                            Arc::clone(slot),
                            permit,
                            kind,
                            mode,
                            lease_id,
                        ));
                    }
                    Err(TryAcquireError::NoPermits) => continue,
                    Err(TryAcquireError::Closed) => continue,
                }
            }

            if let Some(deadline) = earliest_cooldown {
                let now = Instant::now();
                if deadline > now {
                    sleep_until(TokioInstant::from_std(deadline)).await;
                } else {
                    tokio::task::yield_now().await;
                }
            } else {
                tokio::task::yield_now().await;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IpAllocatorSummary {
    pub total_slots: usize,
    pub per_ip_inflight_limit: Option<usize>,
    pub source: IpSource,
}

#[derive(Debug)]
struct IpAllocatorInner {
    slots: Vec<Arc<SlotState>>,
    rotation: AtomicUsize,
    cooldown: CooldownConfig,
    per_ip_inflight_limit: Option<usize>,
    source: IpSource,
    lease_counter: AtomicU64,
}

impl IpAllocatorInner {
    fn next_lease_id(&self) -> u64 {
        self.lease_counter.fetch_add(1, AtomicOrdering::Relaxed)
    }

    fn apply_outcome(&self, slot: &SlotState, outcome: IpLeaseOutcome) {
        match outcome {
            IpLeaseOutcome::Success => slot.clear_cooldown(),
            IpLeaseOutcome::RateLimited => {
                slot.start_cooldown(self.cooldown.rate_limited_start);
                slot.record_rate_limited();
                events::ip_cooldown(slot.ip(), "rate_limited", self.cooldown.rate_limited_start);
            }
            IpLeaseOutcome::Timeout => {
                slot.start_cooldown(self.cooldown.timeout_start);
                slot.record_timeout();
                events::ip_cooldown(slot.ip(), "timeout", self.cooldown.timeout_start);
            }
            IpLeaseOutcome::NetworkError => {
                slot.start_cooldown(self.cooldown.timeout_start);
                slot.record_network_error();
                events::ip_cooldown(slot.ip(), "network_error", self.cooldown.timeout_start);
            }
        }
    }
}

#[derive(Debug)]
struct SlotState {
    slot: Arc<IpSlot>,
    semaphore: Option<Arc<Semaphore>>,
    cooldown_until: Mutex<Option<Instant>>,
}

impl SlotState {
    fn new(slot: Arc<IpSlot>, per_ip_inflight_limit: Option<usize>) -> Self {
        let semaphore = per_ip_inflight_limit
            .map(|limit| Arc::new(Semaphore::new(limit.max(1))))
            .or_else(|| {
                if matches!(slot.kind(), IpSlotKind::Ephemeral) {
                    Some(Arc::new(Semaphore::new(DEFAULT_UNBOUNDED_PERMITS)))
                } else {
                    None
                }
            });

        Self {
            slot,
            semaphore,
            cooldown_until: Mutex::new(None),
        }
    }

    fn ip(&self) -> std::net::IpAddr {
        self.slot.ip()
    }

    fn kind(&self) -> IpSlotKind {
        self.slot.kind()
    }

    fn kind_label(&self) -> &'static str {
        match self.kind() {
            IpSlotKind::Ephemeral => "ephemeral",
            IpSlotKind::LongLived => "long_lived",
        }
    }

    fn cooldown_delay(&self, now: Instant) -> Option<Instant> {
        let mut guard = self.cooldown_until.lock().unwrap();
        if let Some(deadline) = guard.as_ref() {
            if *deadline > now {
                Some(*deadline)
            } else {
                *guard = None;
                None
            }
        } else {
            None
        }
    }

    fn try_acquire(
        &self,
        mode: IpLeaseMode,
    ) -> Result<Option<OwnedSemaphorePermit>, TryAcquireError> {
        match mode {
            IpLeaseMode::Ephemeral => {
                let permit = match &self.semaphore {
                    Some(semaphore) => Some(semaphore.clone().try_acquire_owned()?),
                    None => None,
                };
                self.slot.acquire();
                self.update_inflight_metrics();
                Ok(permit)
            }
            IpLeaseMode::SharedLongLived => {
                self.slot.acquire();
                self.slot.set_state(IpSlotState::LongLived);
                self.update_inflight_metrics();
                Ok(None)
            }
        }
    }

    fn on_release(&self, mode: IpLeaseMode) {
        match mode {
            IpLeaseMode::Ephemeral => {
                self.slot.release();
                self.update_inflight_metrics();
            }
            IpLeaseMode::SharedLongLived => {
                self.slot.release();
                if self.slot.inflight() == 0 {
                    self.slot.set_state(IpSlotState::Idle);
                }
                self.update_inflight_metrics();
            }
        }
    }

    fn start_cooldown(&self, duration: Duration) {
        if duration.is_zero() {
            return;
        }
        let deadline = Instant::now() + duration;
        let mut guard = self.cooldown_until.lock().unwrap();
        *guard = Some(deadline);
        self.slot.set_state(IpSlotState::CoolingDown);
    }

    fn clear_cooldown(&self) {
        let mut guard = self.cooldown_until.lock().unwrap();
        *guard = None;
        if self.slot.inflight() == 0 {
            self.slot.set_state(IpSlotState::Idle);
        }
    }

    fn record_request(&self) {
        self.slot.stats().record_request();
    }

    fn record_rate_limited(&self) {
        self.slot.stats().record_rate_limited();
    }

    fn record_timeout(&self) {
        self.slot.stats().record_timeout();
    }

    fn record_network_error(&self) {
        self.slot.stats().record_network_error();
    }

    fn update_inflight_metrics(&self) {
        events::ip_inflight(self.ip(), self.kind_label(), self.slot.inflight());
    }
}

#[derive(Debug)]
struct IpLeaseInner {
    allocator: Arc<IpAllocatorInner>,
    slot: Arc<SlotState>,
    permit: Mutex<Option<OwnedSemaphorePermit>>,
    outcome_recorded: AtomicBool,
    lease_id: u64,
    kind: IpTaskKind,
    mode: IpLeaseMode,
}

impl IpLeaseInner {
    fn mark_outcome(&self, outcome: IpLeaseOutcome) {
        if !self
            .outcome_recorded
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            self.allocator.apply_outcome(&self.slot, outcome);
        }
    }

    fn release_permit(&self) {
        if let Some(permit) = self.permit.lock().unwrap().take() {
            drop(permit);
        }
        self.slot.on_release(self.mode);
    }
}

impl Drop for IpLeaseInner {
    fn drop(&mut self) {
        if !self
            .outcome_recorded
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            self.allocator
                .apply_outcome(&self.slot, IpLeaseOutcome::Success);
        }
        self.release_permit();
    }
}

#[derive(Debug, Clone)]
pub struct IpLeaseHandle {
    inner: Arc<IpLeaseInner>,
}

impl IpLeaseHandle {
    pub fn id(&self) -> u64 {
        self.inner.lease_id
    }

    pub fn ip(&self) -> std::net::IpAddr {
        self.inner.slot.ip()
    }

    pub fn kind(&self) -> IpTaskKind {
        self.inner.kind
    }

    pub fn mode(&self) -> IpLeaseMode {
        self.inner.mode
    }

    pub fn mark_outcome(&self, outcome: IpLeaseOutcome) {
        self.inner.mark_outcome(outcome);
    }
}

#[derive(Debug, Clone)]
pub struct IpLease {
    handle: IpLeaseHandle,
}

impl IpLease {
    fn new(
        allocator: Arc<IpAllocatorInner>,
        slot: Arc<SlotState>,
        permit: Option<OwnedSemaphorePermit>,
        kind: IpTaskKind,
        mode: IpLeaseMode,
        lease_id: u64,
    ) -> Self {
        let inner = Arc::new(IpLeaseInner {
            allocator,
            slot,
            permit: Mutex::new(permit),
            outcome_recorded: AtomicBool::new(false),
            lease_id,
            kind,
            mode,
        });

        Self {
            handle: IpLeaseHandle { inner },
        }
    }

    pub fn handle(&self) -> IpLeaseHandle {
        self.handle.clone()
    }
}

impl std::ops::Deref for IpLease {
    type Target = IpLeaseHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}
