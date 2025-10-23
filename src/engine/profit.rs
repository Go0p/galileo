use rand::prelude::IndexedRandom;
use tracing::debug;

use super::aggregator::{AggregatorKind, QuotePayloadVariant, QuoteResponseVariant};
use super::types::{DoubleQuote, SwapOpportunity};

#[derive(Debug, Clone)]
pub struct TipConfig {
    pub enable_random: bool,
    pub static_tip_percentage: f64,
    pub random_percentage: Vec<f64>,
}

impl Default for TipConfig {
    fn default() -> Self {
        Self {
            enable_random: false,
            static_tip_percentage: 0.5,
            random_percentage: vec![0.5],
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProfitConfig {
    pub min_profit_threshold_lamports: u64,
    pub max_tip_lamports: u64,
    pub tip: TipConfig,
}

#[derive(Debug, Clone)]
pub struct ProfitEvaluator {
    config: ProfitConfig,
    tip_calculator: TipCalculator,
}

impl ProfitEvaluator {
    pub fn new(config: ProfitConfig) -> Self {
        let tip_calculator = TipCalculator::new(&config.tip, config.max_tip_lamports);
        Self {
            config,
            tip_calculator,
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub fn evaluate(
        &self,
        amount_in: u64,
        double_quote: &DoubleQuote,
        pair: &crate::strategy::types::TradePair,
    ) -> Option<SwapOpportunity> {
        if double_quote.forward.kind() != double_quote.reverse.kind() {
            debug!(
                target: "engine::profit",
                forward = ?double_quote.forward.kind(),
                reverse = ?double_quote.reverse.kind(),
                "前后腿聚合器类型不一致，跳过"
            );
            return None;
        }

        let second_out = double_quote.reverse.out_amount() as u128;

        let profit = second_out.saturating_sub(amount_in as u128);
        let profit_u64 = profit.min(u128::from(u64::MAX)) as u64;
        if profit_u64 < self.config.min_profit_threshold_lamports {
            debug!(
                target: "engine::profit",
                profit = profit_u64,
                threshold = self.config.min_profit_threshold_lamports,
                "收益低于阈值"
            );
            return None;
        }

        let tip_lamports = self.tip_calculator.calculate(profit_u64);
        let merged = merge_quotes(
            &double_quote.forward,
            &double_quote.reverse,
            amount_in,
            tip_lamports,
        );

        Some(SwapOpportunity {
            pair: pair.clone(),
            amount_in,
            profit_lamports: profit_u64,
            tip_lamports,
            merged_quote: Some(merged),
        })
    }
}

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

fn merge_quotes(
    forward: &QuoteResponseVariant,
    reverse: &QuoteResponseVariant,
    original_amount: u64,
    tip_lamports: u64,
) -> QuotePayloadVariant {
    match (forward.kind(), reverse.kind()) {
        (AggregatorKind::Jupiter, AggregatorKind::Jupiter)
        | (AggregatorKind::Dflow, AggregatorKind::Dflow) => {
            let mut merged = forward.clone_payload();
            let reverse_payload = reverse.clone_payload();

            let total_out = (original_amount as u128)
                .saturating_add(tip_lamports as u128)
                .min(u128::from(u64::MAX)) as u64;

            merged.set_output_mint(reverse_payload.output_mint());
            merged.set_out_amount(total_out);
            merged.set_price_impact_zero();

            let max_slot = merged.context_slot().max(reverse_payload.context_slot());
            merged.set_context_slot(max_slot);

            let max_time = merged.time_taken().max(reverse_payload.time_taken());
            merged.set_time_taken(max_time);

            if reverse_payload.route_len() > 0 {
                merged.extend_route(&reverse_payload);
            }

            merged
        }
        _ => {
            panic!("不支持混合聚合器的报价合并");
        }
    }
}
