use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use solana_sdk::pubkey::Pubkey;
use tracing::debug;

use crate::strategy::types::TradePair;

use super::MintSchedule;

#[derive(Debug, Clone)]
pub struct QuoteBatchPlan {
    pub batch_id: u64,
    pub pair: TradePair,
    pub amount: u64,
}

#[derive(Debug, Clone)]
pub enum Action {
    Idle,
    Quote(Vec<QuoteBatchPlan>),
    DispatchBlind(Vec<crate::strategy::types::BlindOrder>),
}

pub struct StrategyResources<'a> {
    pub pairs: &'a [TradePair],
    pub trade_profiles: &'a mut BTreeMap<Pubkey, MintSchedule>,
    pub next_batch_id: &'a mut u64,
}

pub struct StrategyContext<'a> {
    pairs: &'a [TradePair],
    trade_profiles: &'a mut BTreeMap<Pubkey, MintSchedule>,
    next_batch_id: &'a mut u64,
    pending: Vec<QuoteBatchPlan>,
    next_ready_delta: Option<Duration>,
}

impl<'a> StrategyContext<'a> {
    pub fn new(resources: StrategyResources<'a>) -> Self {
        let StrategyResources {
            pairs,
            trade_profiles,
            next_batch_id,
        } = resources;
        Self {
            pairs,
            trade_profiles,
            next_batch_id,
            pending: Vec::new(),
            next_ready_delta: None,
        }
    }

    pub fn trade_pairs(&self) -> &'a [TradePair] {
        self.pairs
    }

    pub fn take_amounts_if_ready(&mut self, mint: &Pubkey) -> Option<Vec<u64>> {
        enum LogReason {
            NotReady,
            MissingAmounts,
            EmptyAfterReady,
        }

        let (batch, delta, reason) = if let Some(schedule) = self.trade_profiles.get_mut(mint) {
            let now = Instant::now();
            let was_ready = schedule.is_ready(now);
            let has_amounts = schedule.has_amounts();
            let batch = schedule.take_ready_batch(now);
            let delta = schedule.time_until_ready(now);
            let reason = if batch.is_some() {
                None
            } else if !has_amounts {
                Some(LogReason::MissingAmounts)
            } else if was_ready {
                Some(LogReason::EmptyAfterReady)
            } else {
                Some(LogReason::NotReady)
            };
            (batch, delta, reason)
        } else {
            debug!(
                target: "engine::context",
                base_mint = %mint,
                "未找到该 base mint 对应的交易规模，跳过报价调度"
            );
            return None;
        };

        self.update_next_ready(delta);

        if let Some(amounts) = batch {
            return Some(amounts);
        }

        if let Some(reason) = reason {
            match reason {
                LogReason::NotReady => debug!(
                    target: "engine::context",
                    base_mint = %mint,
                    "base mint 尚未到达下一次调度时间，跳过"
                ),
                LogReason::MissingAmounts => debug!(
                    target: "engine::context",
                    base_mint = %mint,
                    "未配置有效的交易规模，跳过"
                ),
                LogReason::EmptyAfterReady => debug!(
                    target: "engine::context",
                    base_mint = %mint,
                    "base mint 未生成有效交易规模，跳过"
                ),
            }
        }

        None
    }

    pub fn push_quote_tasks(&mut self, pair: &TradePair, amounts: Vec<u64>) {
        for amount in amounts {
            let batch_id = *self.next_batch_id;
            *self.next_batch_id = self.next_batch_id.wrapping_add(1).max(1);
            self.pending.push(QuoteBatchPlan {
                batch_id,
                pair: pair.clone(),
                amount,
            });
        }
    }

    pub fn next_ready_delay(&self) -> Option<Duration> {
        self.next_ready_delta
    }

    pub fn into_decision(self) -> StrategyDecision {
        let action = if self.pending.is_empty() {
            Action::Idle
        } else {
            Action::Quote(self.pending)
        };
        StrategyDecision {
            action,
            next_ready_in: self.next_ready_delta,
        }
    }

    fn update_next_ready(&mut self, delta: Duration) {
        if delta.is_zero() {
            return;
        };
        let should_update = match self.next_ready_delta {
            Some(current) => delta < current,
            None => true,
        };
        if should_update {
            self.next_ready_delta = Some(delta);
        }
    }
}

#[derive(Debug, Clone)]
pub struct StrategyDecision {
    pub action: Action,
    pub next_ready_in: Option<Duration>,
}
