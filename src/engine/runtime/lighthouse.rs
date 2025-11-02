use std::collections::HashSet;

use solana_sdk::pubkey::Pubkey;

use crate::engine::LighthouseSettings;

const MIN_LIGHTHOUSE_MEMORY_SLOTS: usize = 1;
const MAX_LIGHTHOUSE_MEMORY_SLOTS: usize = 128;

pub(crate) struct LighthouseRuntime {
    pub(super) enabled: bool,
    pub(super) guard_mints: HashSet<Pubkey>,
    pub(super) memory_slots: usize,
    pub(super) available_ids: Vec<u8>,
    pub(super) cursor: usize,
}

impl LighthouseRuntime {
    pub(crate) fn new(settings: &LighthouseSettings, ip_capacity_hint: usize) -> Self {
        let guard_mints: HashSet<Pubkey> = settings.profit_guard_mints.iter().copied().collect();
        let enabled = settings.enable && !guard_mints.is_empty();

        let mut available_ids: Vec<u8> = settings.existing_memory_ids.clone();
        available_ids.sort_unstable();
        available_ids.dedup();

        let derived_slots = if !available_ids.is_empty() {
            available_ids.len()
        } else {
            ip_capacity_hint
                .max(MIN_LIGHTHOUSE_MEMORY_SLOTS)
                .min(MAX_LIGHTHOUSE_MEMORY_SLOTS)
        };
        let configured_slots = settings.memory_slots.map(|value| usize::from(value.max(1)));
        let slot_count = configured_slots.unwrap_or(derived_slots);
        let memory_slots = slot_count
            .max(available_ids.len())
            .max(MIN_LIGHTHOUSE_MEMORY_SLOTS)
            .min(MAX_LIGHTHOUSE_MEMORY_SLOTS);

        Self {
            enabled,
            guard_mints,
            memory_slots,
            available_ids,
            cursor: 0,
        }
    }

    pub(crate) fn should_guard(&self, mint: &Pubkey) -> bool {
        self.enabled && self.guard_mints.contains(mint)
    }

    pub(crate) fn next_memory_id(&mut self) -> u8 {
        if !self.enabled {
            return 0;
        }
        if self.available_ids.is_empty() {
            let id = 0u8;
            if self.memory_slots > 1 {
                self.available_ids
                    .reserve(self.memory_slots.saturating_sub(1));
            }
            self.available_ids.push(id);
            return id;
        }

        let idx = if self.available_ids.len() == 1 {
            0
        } else {
            let current = self.cursor % self.available_ids.len();
            self.cursor = (self.cursor + 1) % self.available_ids.len();
            current
        };
        self.available_ids[idx]
    }
}
