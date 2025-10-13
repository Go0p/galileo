pub mod blind;
pub mod config;
pub mod spam;
pub mod types;

use crate::engine::{Action, StrategyContext, StrategyTick};

#[derive(Debug)]
pub enum StrategyEvent {
    Tick(StrategyTick),
}

pub trait Strategy {
    type Event;

    fn name(&self) -> &'static str;

    fn on_market_event(&mut self, event: &Self::Event, ctx: StrategyContext<'_>) -> Action;
}
