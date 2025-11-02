use rand::prelude::IndexedRandom;
use tracing::debug;

use super::aggregator::{AggregatorKind, QuotePayloadVariant, QuoteResponseVariant};
use super::types::{DoubleQuote, SwapOpportunity, UltraSwapLegs};
use crate::monitoring::events;

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
            static_tip_percentage: 0.0,
            random_percentage: vec![0.0],
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
    multi_ip_enabled: bool,
}

impl ProfitEvaluator {
    pub fn new(config: ProfitConfig, multi_ip_enabled: bool) -> Self {
        let tip_calculator = TipCalculator::new(&config.tip, config.max_tip_lamports);
        Self {
            config,
            tip_calculator,
            multi_ip_enabled,
        }
    }

    pub fn min_threshold(&self) -> u64 {
        self.config.min_profit_threshold_lamports
    }

    pub fn evaluate_multi_leg(&self, gross_profit_lamports: i128) -> Option<MultiLegProfit> {
        if gross_profit_lamports <= 0 {
            return None;
        }
        let profit_u64 = gross_profit_lamports.min(i128::from(u64::MAX)).max(0) as u64;
        if profit_u64 < self.config.min_profit_threshold_lamports {
            return None;
        }
        let tip_lamports = self.tip_calculator.calculate(profit_u64);
        Some(MultiLegProfit {
            gross_profit_lamports: profit_u64,
            tip_lamports,
        })
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

        let aggregator_label = format!("{:?}", double_quote.forward.kind());
        let forward_in = double_quote.forward.in_amount();
        let forward_out = double_quote.forward.out_amount();
        let reverse_in = double_quote.reverse.in_amount();
        let reverse_out = double_quote.reverse.out_amount();
        let forward_latency_ms = double_quote.forward_latency_ms();
        let reverse_latency_ms = double_quote.reverse_latency_ms();

        let second_out = reverse_out as u128;

        let profit = second_out.saturating_sub(amount_in as u128);
        let profit_u64 = profit.min(u128::from(u64::MAX)) as u64;
        let threshold = self.config.min_profit_threshold_lamports;
        if profit_u64 < threshold {
            debug!(
                target: "engine::profit",
                profit = profit_u64,
                threshold,
                "收益低于阈值"
            );
            events::profit_shortfall(
                pair.input_mint.as_str(),
                &aggregator_label,
                forward_in,
                forward_out,
                forward_latency_ms,
                reverse_in,
                reverse_out,
                reverse_latency_ms,
                profit_u64,
                threshold,
            );
            return None;
        }

        let tip_lamports = self.tip_calculator.calculate(profit_u64);
        let net_profit = i128::from(profit_u64) - i128::from(tip_lamports);
        events::profit_opportunity(
            pair.input_mint.as_str(),
            &aggregator_label,
            forward_in,
            forward_out,
            forward_latency_ms,
            reverse_in,
            reverse_out,
            reverse_latency_ms,
            profit_u64,
            net_profit,
            threshold,
            self.multi_ip_enabled,
            double_quote.forward_ip,
            double_quote.reverse_ip,
            double_quote.total_latency_ms(),
        );
        let merged = merge_quotes(
            &double_quote.forward,
            &double_quote.reverse,
            amount_in,
            tip_lamports,
        );

        let ultra_legs = match (&double_quote.forward, &double_quote.reverse) {
            (QuoteResponseVariant::Ultra(forward), QuoteResponseVariant::Ultra(reverse)) => {
                let (_, forward_payload) = forward.clone().into_parts();
                let (_, reverse_payload) = reverse.clone().into_parts();
                Some(UltraSwapLegs {
                    forward: forward_payload,
                    reverse: reverse_payload,
                })
            }
            _ => None,
        };

        Some(SwapOpportunity {
            pair: pair.clone(),
            amount_in,
            profit_lamports: profit_u64,
            tip_lamports,
            merged_quote: Some(merged),
            ultra_legs,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MultiLegProfit {
    pub gross_profit_lamports: u64,
    pub tip_lamports: u64,
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
        (AggregatorKind::Dflow, AggregatorKind::Dflow)
        | (AggregatorKind::Ultra, AggregatorKind::Ultra)
        | (AggregatorKind::Kamino, AggregatorKind::Kamino) => {
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
