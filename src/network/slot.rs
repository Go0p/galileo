#![allow(dead_code)]

use std::net::IpAddr;
use std::sync::atomic::{AtomicU8, AtomicU64, AtomicUsize, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpSlotKind {
    Ephemeral,
    LongLived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpSlotState {
    Idle = 0,
    Busy = 1,
    CoolingDown = 2,
    LongLived = 3,
}

impl IpSlotState {
    fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Idle,
            1 => Self::Busy,
            2 => Self::CoolingDown,
            3 => Self::LongLived,
            _ => Self::Idle,
        }
    }
}

#[derive(Debug, Default)]
pub struct IpSlotStats {
    total_requests: AtomicU64,
    rate_limited: AtomicU64,
    timeouts: AtomicU64,
    network_errors: AtomicU64,
}

impl IpSlotStats {
    pub fn record_request(&self) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_rate_limited(&self) {
        self.rate_limited.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_timeout(&self) {
        self.timeouts.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_network_error(&self) {
        self.network_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> IpSlotStatsSnapshot {
        IpSlotStatsSnapshot {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            rate_limited: self.rate_limited.load(Ordering::Relaxed),
            timeouts: self.timeouts.load(Ordering::Relaxed),
            network_errors: self.network_errors.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct IpSlotStatsSnapshot {
    pub total_requests: u64,
    pub rate_limited: u64,
    pub timeouts: u64,
    pub network_errors: u64,
}

#[derive(Debug)]
pub struct IpSlot {
    ip: IpAddr,
    kind: IpSlotKind,
    state: AtomicU8,
    inflight: AtomicUsize,
    stats: IpSlotStats,
}

impl IpSlot {
    pub fn new(ip: IpAddr, kind: IpSlotKind) -> Self {
        let initial_state = match kind {
            IpSlotKind::Ephemeral => IpSlotState::Idle as u8,
            IpSlotKind::LongLived => IpSlotState::LongLived as u8,
        };
        Self {
            ip,
            kind,
            state: AtomicU8::new(initial_state),
            inflight: AtomicUsize::new(0),
            stats: IpSlotStats::default(),
        }
    }

    pub fn ip(&self) -> IpAddr {
        self.ip
    }

    pub fn kind(&self) -> IpSlotKind {
        self.kind
    }

    pub fn state(&self) -> IpSlotState {
        IpSlotState::from_u8(self.state.load(Ordering::Acquire))
    }

    pub fn set_state(&self, state: IpSlotState) {
        self.state.store(state as u8, Ordering::Release);
    }

    pub fn inflight(&self) -> usize {
        self.inflight.load(Ordering::Acquire)
    }

    pub fn acquire(&self) {
        self.inflight.fetch_add(1, Ordering::AcqRel);
        if self.kind == IpSlotKind::Ephemeral {
            self.set_state(IpSlotState::Busy);
        }
    }

    pub fn release(&self) {
        let remaining = self.inflight.fetch_sub(1, Ordering::AcqRel) - 1;
        if remaining == 0 && self.kind == IpSlotKind::Ephemeral {
            self.set_state(IpSlotState::Idle);
        }
    }

    pub fn stats(&self) -> &IpSlotStats {
        &self.stats
    }
}
