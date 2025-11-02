pub mod blind;
pub mod common;
pub mod copy;
pub mod pure_blind;

pub use blind::BlindStrategy;
pub use copy::run_copy_strategy;
pub use pure_blind::{PureBlindRouteBuilder, PureBlindStrategy};
pub mod types {
    pub use super::common::types::*;
}

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
