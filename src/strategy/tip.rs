use rand::prelude::IndexedRandom;

use super::config::TipConfig;

#[derive(Debug, Clone)]
pub struct TipCalculator {
    config: TipConfig,
    max_tip: u64,
}

impl TipCalculator {
    pub fn new(config: &TipConfig, max_tip: u64) -> Self {
        Self {
            config: config.clone(),
            max_tip,
        }
    }

    pub fn calculate(&self, profit_lamports: u64) -> u64 {
        if profit_lamports == 0 {
            return 0;
        }

        let ratio = if self.config.enable_random && !self.config.random_percentage.is_empty() {
            let mut rng = rand::rng();
            *self
                .config
                .random_percentage
                .choose(&mut rng)
                .unwrap_or(&self.config.static_tip_percentage)
        } else {
            self.config.static_tip_percentage
        };

        let ratio = ratio.clamp(0.0, 1.0);
        let calculated = (profit_lamports as f64 * ratio).round() as u64;
        if self.max_tip == 0 {
            calculated
        } else {
            calculated.min(self.max_tip)
        }
    }
}
