use std::time::{Duration, Instant};

use super::aggregator::{QuotePayloadVariant, QuoteResponseVariant};
use crate::strategy::types::TradePair;

#[derive(Debug, Clone)]
pub struct QuoteTask {
    pub pair: TradePair,
    pub amount: u64,
}

impl QuoteTask {
    pub fn new(pair: TradePair, amount: u64) -> Self {
        Self { pair, amount }
    }
}

#[derive(Debug)]
pub struct StrategyTick {
    pub at: Instant,
}

impl StrategyTick {
    pub fn now() -> Self {
        Self { at: Instant::now() }
    }
}

#[derive(Debug, Clone)]
pub struct DoubleQuote {
    pub forward: QuoteResponseVariant,
    pub reverse: QuoteResponseVariant,
}

#[derive(Debug, Clone)]
pub struct TradeProfile {
    pub amounts: Vec<u64>,
    pub process_delay: Duration,
}

#[derive(Debug, Clone)]
pub struct SwapOpportunity {
    pub pair: TradePair,
    pub amount_in: u64,
    pub profit_lamports: u64,
    pub tip_lamports: u64,
    pub merged_quote: Option<QuotePayloadVariant>,
}

impl SwapOpportunity {
    pub fn net_profit(&self) -> i128 {
        self.profit_lamports as i128 - self.tip_lamports as i128
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub opportunity: SwapOpportunity,
    pub deadline: Instant,
}

impl ExecutionPlan {
    pub fn with_deadline(opportunity: SwapOpportunity, timeout: Duration) -> Self {
        let deadline = Instant::now() + timeout;
        Self {
            opportunity,
            deadline,
        }
    }
}
