use crate::engine::{StrategyContext, StrategyDecision};

use crate::strategy::{Strategy, StrategyEvent};
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

    fn on_market_event(
        &mut self,
        event: &Self::Event,
        mut ctx: StrategyContext<'_>,
    ) -> StrategyDecision {
        match event {
            StrategyEvent::Tick(tick) => {
                let _started_at = tick.at;
                let pairs = ctx.trade_pairs();
                if pairs.is_empty() {
                    return ctx.into_decision();
                }

                let total = pairs.len();
                if self.next_pair_index >= total {
                    self.next_pair_index = 0;
                }
                let start = self.next_pair_index;
                for offset in 0..total {
                    let idx = (start + offset) % total;
                    let pair = &pairs[idx];
                    if let Some(amounts) = ctx.take_amounts(&pair.input_pubkey) {
                        if !amounts.is_empty() {
                            ctx.push_quote_tasks(pair, amounts);
                        }
                    }
                }

                self.next_pair_index = (start + 1) % total;

                ctx.into_decision()
            }
        }
    }
}
