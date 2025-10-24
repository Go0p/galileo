use std::collections::BTreeMap;
use std::time::Instant;

use solana_sdk::pubkey::Pubkey;
use tracing::debug;

use crate::strategy::types::TradePair;

use super::{MintSchedule, QuoteTask};

#[derive(Debug)]
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
        }
    }

    pub fn trade_pairs(&self) -> &'a [TradePair] {
        self.pairs
    }

    pub fn take_amounts_if_ready(&mut self, mint: &Pubkey) -> Option<Vec<u64>> {
        if let Some(schedule) = self.trade_profiles.get_mut(mint) {
            let now = Instant::now();
            if schedule.ready(now) {
                if let Some(amount) = schedule.next_amount() {
                    schedule.mark_dispatched(now);
                    return Some(vec![amount]);
                }
                debug!(
                    target: "engine::context",
                    base_mint = %mint,
                    "base mint 未配置有效的交易规模，跳过"
                );
            } else {
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

    pub fn push_quote_tasks(&mut self, pair: &TradePair, amounts: &[u64]) {
        for &amount in amounts {
            self.pending.push(QuoteTask::new(pair.clone(), amount));
        }
    }

    pub fn into_action(self) -> Action {
        if self.pending.is_empty() {
            Action::Idle
        } else {
            Action::Quote(self.pending)
        }
    }
}
