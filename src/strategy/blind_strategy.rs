use crate::engine::{Action, StrategyContext};

use super::{Strategy, StrategyEvent};
pub struct BlindStrategy {
    next_pair_index: usize,
}

impl BlindStrategy {
    pub fn new() -> Self {
        Self { next_pair_index: 0 }
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
                let pairs = ctx.trade_pairs();
                if pairs.is_empty() {
                    return Action::Idle;
                }

                let idx = if self.next_pair_index >= pairs.len() {
                    self.next_pair_index = 0;
                    0
                } else {
                    self.next_pair_index
                };

                if let Some(pair) = pairs.get(idx) {
                    if let Some(amounts) = ctx.take_amounts_if_ready(&pair.input_pubkey) {
                        ctx.push_quote_tasks(pair, &amounts);
                    }
                    self.next_pair_index = (idx + 1) % pairs.len();
                }

                ctx.into_action()
            }
        }
    }
}
