pub mod blind_strategy;
pub mod pure_blind_strategy;
pub mod types;

pub use blind_strategy::BlindStrategy;
pub use pure_blind_strategy::{PureBlindRouteBuilder, PureBlindStrategy};

use crate::engine::{StrategyContext, StrategyDecision, StrategyTick};

#[derive(Debug)]
pub enum StrategyEvent {
    Tick(StrategyTick),
}

pub trait Strategy {
    type Event;

    fn name(&self) -> &'static str;

    fn on_market_event(
        &mut self,
        event: &Self::Event,
        ctx: StrategyContext<'_>,
    ) -> StrategyDecision;
}
