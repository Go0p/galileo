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
    pub process_delay: Duration,
}

#[derive(Debug, Clone)]
pub struct ReadyAmounts {
    pub amounts: Vec<u64>,
    pub process_delay: Duration,
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

    pub fn take_amounts_if_ready(&mut self, mint: &Pubkey) -> Option<ReadyAmounts> {
        if let Some(schedule) = self.trade_profiles.get_mut(mint) {
            let now = Instant::now();
            let process_delay = schedule.process_delay();
            if schedule.ready(now) {
                if let Some(amount) = schedule.next_amount() {
                    schedule.mark_dispatched(now);
                    let delta = schedule.time_until_ready(now);
                    self.update_next_ready(delta);
                    return Some(ReadyAmounts {
                        amounts: vec![amount],
                        process_delay,
                    });
                }

                let delta = schedule.time_until_ready(now);
                self.update_next_ready(delta);
                debug!(
                    target: "engine::context",
                    base_mint = %mint,
                    "base mint 未配置有效的交易规模，跳过"
                );
            } else {
                let delta = schedule.time_until_ready(now);
                self.update_next_ready(delta);
                debug!(
                    target: "engine::context",
                    base_mint = %mint,
                    "base mint 尚未到达下一次调度时间，跳过"
                );
            }
        } else {
            debug!(
                target: "engine::context",
                base_mint = %mint,
                "未找到该 base mint 对应的交易规模，跳过报价调度"
            );
        }
        None
    }

    pub fn push_quote_tasks(&mut self, pair: &TradePair, ready: ReadyAmounts) {
        let ReadyAmounts {
            amounts,
            process_delay,
        } = ready;

        for amount in amounts {
            let batch_id = *self.next_batch_id;
            *self.next_batch_id = self.next_batch_id.wrapping_add(1).max(1);
            self.pending.push(QuoteBatchPlan {
                batch_id,
                pair: pair.clone(),
                amount,
                process_delay,
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
