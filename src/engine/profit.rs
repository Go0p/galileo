use rand::prelude::IndexedRandom;
use serde_json::Value;
use tracing::debug;

use super::error::EngineResult;
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
        let second_out = double_quote.reverse.out_amount as u128;

        let profit = second_out.saturating_sub(amount_in as u128);
        let profit_u64 = profit.min(u128::from(u64::MAX)) as u64;
        if profit_u64 <= self.config.min_profit_threshold_lamports {
            debug!(
                target: "engine::profit",
                profit = profit_u64,
                threshold = self.config.min_profit_threshold_lamports,
                "收益低于阈值"
            );
            return None;
        }

        let tip_lamports = self.tip_calculator.calculate(profit_u64);
        let merged = match merge_quotes(
            &double_quote.forward.raw,
            &double_quote.reverse.raw,
            amount_in,
            tip_lamports,
        ) {
            Ok(value) => value,
            Err(err) => {
                debug!(
                    target: "engine::profit",
                    error = %err,
                    "合并 quote 失败"
                );
                return None;
            }
        };

        Some(SwapOpportunity {
            pair: pair.clone(),
            amount_in,
            profit_lamports: profit_u64,
            tip_lamports,
            merged_quote: merged,
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
    first: &Value,
    second: &Value,
    original_amount: u64,
    tip_lamports: u64,
) -> EngineResult<Value> {
    let mut merged = first.clone();
    let total_out = (original_amount as u128)
        .saturating_add(tip_lamports as u128)
        .min(u128::from(u64::MAX)) as u64;

    if let Some(obj) = merged.as_object_mut() {
        obj.insert(
            "outputMint".to_string(),
            Value::String(
                second
                    .get("outputMint")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
            ),
        );
        obj.insert("priceImpactPct".to_string(), Value::String("0".into()));
        obj.insert(
            "outAmount".to_string(),
            Value::String(total_out.to_string()),
        );
        obj.insert(
            "otherAmountThreshold".to_string(),
            Value::String(total_out.to_string()),
        );
        if let Some(route_plan) = obj.get_mut("routePlan") {
            if let Some(route_array) = route_plan.as_array_mut() {
                if let Some(second_plan) = second.get("routePlan").and_then(|v| v.as_array()) {
                    route_array.extend(second_plan.iter().cloned());
                }
            }
        }
    }

    Ok(merged)
}
