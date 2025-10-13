use crate::strategy::types::TradePair;

use super::types::QuoteTask;

#[derive(Debug)]
pub enum Action {
    Idle,
    Quote(Vec<QuoteTask>),
}

pub struct StrategyResources<'a> {
    pub pairs: &'a [TradePair],
    pub trade_amounts: &'a [u64],
}

pub struct StrategyContext<'a> {
    resources: StrategyResources<'a>,
    pending: Vec<QuoteTask>,
}

impl<'a> StrategyContext<'a> {
    pub fn new(resources: StrategyResources<'a>) -> Self {
        Self {
            resources,
            pending: Vec::new(),
        }
    }

    pub fn trade_pairs(&self) -> &'a [TradePair] {
        self.resources.pairs
    }

    pub fn schedule_pair_all_amounts(&mut self, pair: &TradePair) {
        for &amount in self.resources.trade_amounts {
            self.pending.push(QuoteTask::new(pair.clone(), amount));
        }
    }

    pub fn into_action(mut self) -> Action {
        if self.pending.is_empty() {
            Action::Idle
        } else {
            Action::Quote(std::mem::take(&mut self.pending))
        }
    }
}
