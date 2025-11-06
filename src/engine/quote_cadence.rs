use std::collections::BTreeMap;
use std::time::Duration;

use crate::config::{QuoteCadenceConfig, QuoteCadenceTimings};

#[derive(Debug, Clone, Copy)]
pub struct CadenceTimings {
    pub max_concurrent_slots: Option<u16>,
    pub inter_batch_delay: Duration,
    pub cycle_cooldown: Duration,
}

impl CadenceTimings {
    fn from_config(config: &QuoteCadenceTimings) -> Self {
        Self {
            max_concurrent_slots: config.max_concurrent_slots,
            inter_batch_delay: duration_from_ms(config.inter_batch_delay_ms),
            cycle_cooldown: duration_from_ms(config.cycle_cooldown_ms),
        }
    }
}

#[derive(Debug, Clone)]
pub struct QuoteCadence {
    default: CadenceTimings,
    per_base_mint: BTreeMap<String, CadenceTimings>,
    per_label: BTreeMap<String, CadenceTimings>,
}

impl QuoteCadence {
    pub fn from_config(config: &QuoteCadenceConfig) -> Self {
        let default = CadenceTimings::from_config(&config.default);
        let per_base_mint = config
            .per_base_mint
            .iter()
            .map(|(mint, timings)| (normalize_key(mint), CadenceTimings::from_config(timings)))
            .collect();
        let per_label = config
            .per_label
            .iter()
            .map(|(label, timings)| (normalize_key(label), CadenceTimings::from_config(timings)))
            .collect();

        Self {
            default,
            per_base_mint,
            per_label,
        }
    }

    pub fn default_timings(&self) -> CadenceTimings {
        self.default
    }

    pub fn timings_for_base_mint(&self, base_mint: &str) -> CadenceTimings {
        let key = normalize_key(base_mint);
        self.per_base_mint
            .get(&key)
            .copied()
            .unwrap_or(self.default)
    }

    #[allow(dead_code)]
    pub fn timings_for_label(&self, label: &str) -> Option<CadenceTimings> {
        let key = normalize_key(label);
        self.per_label.get(&key).copied()
    }
}

impl Default for QuoteCadence {
    fn default() -> Self {
        Self::from_config(&QuoteCadenceConfig::default())
    }
}

const fn duration_from_ms(ms: Option<u64>) -> Duration {
    match ms {
        Some(value) => Duration::from_millis(value),
        None => Duration::ZERO,
    }
}

fn normalize_key(value: &str) -> String {
    value.trim().to_string()
}
