pub mod blind_strategy;
pub mod copy_strategy;
pub mod types;

pub use blind_strategy::BlindStrategy;
pub use copy_strategy::{
    CopyStrategy, CopySwapParams, compute_associated_token_address, usdc_mint, wsol_mint,
};

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
