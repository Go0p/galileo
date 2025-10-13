use crate::engine::{Action, StrategyContext};

use super::{Strategy, StrategyEvent};

pub struct BlindStrategy;

impl BlindStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Strategy for BlindStrategy {
    type Event = StrategyEvent;

    fn name(&self) -> &'static str {
        "blind"
    }

    fn on_market_event(&mut self, event: &Self::Event, mut ctx: StrategyContext<'_>) -> Action {
        match event {
            StrategyEvent::Tick(tick) => {
                let _started_at = tick.at;
                for pair in ctx.trade_pairs() {
                    ctx.schedule_pair_all_amounts(pair);
                }
                ctx.into_action()
            }
        }
    }
}
