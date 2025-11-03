use std::net::IpAddr;
use std::time::{Duration, Instant};

use super::aggregator::{QuotePayloadVariant, QuoteResponseVariant};
use crate::api::ultra::order::OrderResponsePayload;
use crate::strategy::types::TradePair;
use solana_sdk::pubkey::Pubkey;

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
    pub forward_latency: Option<Duration>,
    pub reverse_latency: Option<Duration>,
    pub forward_ip: Option<IpAddr>,
    pub reverse_ip: Option<IpAddr>,
}

impl DoubleQuote {
    pub fn forward_latency_ms(&self) -> Option<f64> {
        self.forward_latency
            .map(|duration| duration.as_secs_f64() * 1_000.0)
    }

    pub fn reverse_latency_ms(&self) -> Option<f64> {
        self.reverse_latency
            .map(|duration| duration.as_secs_f64() * 1_000.0)
    }

    pub fn total_latency_ms(&self) -> Option<f64> {
        match (self.forward_latency, self.reverse_latency) {
            (Some(forward), Some(reverse)) => Some((forward + reverse).as_secs_f64() * 1_000.0),
            (Some(forward), None) => Some(forward.as_secs_f64() * 1_000.0),
            (None, Some(reverse)) => Some(reverse.as_secs_f64() * 1_000.0),
            (None, None) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TradeProfile {
    pub amounts: Vec<u64>,
}

#[derive(Debug, Clone)]
pub struct SwapOpportunity {
    pub pair: TradePair,
    pub amount_in: u64,
    pub profit_lamports: u64,
    pub tip_lamports: u64,
    pub merged_quote: Option<QuotePayloadVariant>,
    pub ultra_legs: Option<UltraSwapLegs>,
}

impl SwapOpportunity {
    pub fn net_profit(&self) -> i128 {
        self.profit_lamports as i128 - self.tip_lamports as i128
    }
}

#[derive(Debug, Clone)]
pub struct UltraSwapLegs {
    pub forward: OrderResponsePayload,
    pub reverse: OrderResponsePayload,
}

#[derive(Debug, Clone)]
pub struct JitoTipPlan {
    pub lamports: u64,
    pub recipient: Pubkey,
}

impl JitoTipPlan {
    pub fn new(lamports: u64, recipient: Pubkey) -> Self {
        Self {
            lamports,
            recipient,
        }
    }
}
