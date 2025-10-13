use rand::seq::SliceRandom;

use crate::engine::{Action, StrategyContext};

use super::{Strategy, StrategyEvent};

pub struct SpamStrategy {
    rng: rand::rngs::ThreadRng,
}

impl SpamStrategy {
    pub fn new() -> Self {
        Self { rng: rand::rng() }
    }
}

impl Strategy for SpamStrategy {
    type Event = StrategyEvent;

    fn name(&self) -> &'static str {
        "spam"
    }

    fn on_market_event(&mut self, event: &Self::Event, mut ctx: StrategyContext<'_>) -> Action {
        match event {
            StrategyEvent::Tick(tick) => {
                let _started_at = tick.at;
                let mut indices: Vec<usize> = (0..ctx.trade_pairs().len()).collect();
                indices.shuffle(&mut self.rng);
                for index in indices {
                    if let Some(pair) = ctx.trade_pairs().get(index) {
                        ctx.schedule_pair_all_amounts(pair);
                    }
                }
                ctx.into_action()
            }
        }
    }
}
