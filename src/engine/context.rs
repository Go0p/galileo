use std::collections::BTreeMap;
use std::time::Duration;

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
        }
    }

    pub fn trade_pairs(&self) -> &'a [TradePair] {
        self.pairs
    }

    pub fn take_amounts(&mut self, mint: &Pubkey) -> Option<Vec<u64>> {
        let schedule = match self.trade_profiles.get(mint) {
            Some(schedule) => schedule,
            None => {
                debug!(
                    target: "engine::context",
                    base_mint = %mint,
                    "未找到该 base mint 对应的交易规模，跳过报价调度"
                );
                return None;
            }
        };

        if schedule.is_empty() {
            debug!(
                target: "engine::context",
                base_mint = %mint,
                "未配置有效的交易规模，跳过"
            );
            return None;
        }

        Some(schedule.clone_amounts())
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

    pub fn into_decision(self) -> StrategyDecision {
        let action = if self.pending.is_empty() {
            Action::Idle
        } else {
            Action::Quote(self.pending)
        };
        StrategyDecision {
            action,
            next_ready_in: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StrategyDecision {
    pub action: Action,
    pub next_ready_in: Option<Duration>,
}
