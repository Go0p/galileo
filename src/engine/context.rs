use std::collections::BTreeMap;
use std::time::{Duration, Instant};

use solana_sdk::pubkey::Pubkey;
use tracing::debug;

use crate::strategy::types::TradePair;

use super::{MintSchedule, QuoteTask};

#[derive(Debug, Clone)]
pub enum Action {
    Idle,
    Quote(Vec<QuoteTask>),
    DispatchBlind(Vec<crate::strategy::types::BlindOrder>),
}

pub struct StrategyResources<'a> {
    pub pairs: &'a [TradePair],
    pub trade_profiles: &'a mut BTreeMap<Pubkey, MintSchedule>,
}

pub struct StrategyContext<'a> {
    pairs: &'a [TradePair],
    trade_profiles: &'a mut BTreeMap<Pubkey, MintSchedule>,
    pending: Vec<QuoteTask>,
    next_ready_delta: Option<Duration>,
}

impl<'a> StrategyContext<'a> {
    pub fn new(resources: StrategyResources<'a>) -> Self {
        let StrategyResources {
            pairs,
            trade_profiles,
        } = resources;
        Self {
            pairs,
            trade_profiles,
            pending: Vec::new(),
            next_ready_delta: None,
        }
    }

    pub fn trade_pairs(&self) -> &'a [TradePair] {
        self.pairs
    }

    pub fn take_amounts_if_ready(&mut self, mint: &Pubkey) -> Option<Vec<u64>> {
        if let Some(schedule) = self.trade_profiles.get_mut(mint) {
            let now = Instant::now();
            let (maybe_amount, delta, message) = {
                if schedule.ready(now) {
                    if let Some(amount) = schedule.next_amount() {
                        schedule.mark_dispatched(now);
                        (Some(amount), schedule.time_until_ready(now), None)
                    } else {
                        (
                            None,
                            schedule.time_until_ready(now),
                            Some("base mint 未配置有效的交易规模，跳过"),
                        )
                    }
                } else {
                    (
                        None,
                        schedule.time_until_ready(now),
                        Some("base mint 尚未到达下一次调度时间，跳过"),
                    )
                }
            };

            self.update_next_ready(delta);

            if let Some(amount) = maybe_amount {
                return Some(vec![amount]);
            }

            if let Some(msg) = message {
                debug!(target: "engine::context", base_mint = %mint, "{msg}");
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

    pub fn push_quote_tasks(&mut self, pair: &TradePair, amounts: &[u64]) {
        for &amount in amounts {
            self.pending.push(QuoteTask::new(pair.clone(), amount));
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
