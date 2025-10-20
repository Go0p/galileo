use anyhow::{Result, bail};
use rand::seq::SliceRandom;

use crate::engine::{Action, StrategyContext};

use super::types::{BlindOrder, BlindRoutePlan};
use super::{Strategy, StrategyEvent};

/// 纯盲发策略：不依赖报价，直接构造 route_v2 指令。
pub struct PureBlindStrategy {
    routes: Vec<BlindRoutePlan>,
}

impl PureBlindStrategy {
    pub fn new(routes: Vec<BlindRoutePlan>) -> Result<Self> {
        if routes.is_empty() {
            bail!("纯盲发模式需要至少一个盲发路由");
        }

        Ok(Self { routes })
    }
}

impl Strategy for PureBlindStrategy {
    type Event = StrategyEvent;

    fn name(&self) -> &'static str {
        "pure_blind"
    }

    fn on_market_event(&mut self, event: &Self::Event, ctx: StrategyContext<'_>) -> Action {
        match event {
            StrategyEvent::Tick(_) => {
                let trade_amounts = ctx.trade_amounts();
                if trade_amounts.is_empty() {
                    return Action::Idle;
                }

                let mut batch = Vec::with_capacity(self.routes.len() * trade_amounts.len() * 2);

                for route in &self.routes {
                    for &amount in trade_amounts {
                        batch.push(BlindOrder {
                            amount_in: amount,
                            steps: route.forward.clone(),
                        });
                        batch.push(BlindOrder {
                            amount_in: amount,
                            steps: route.reverse.clone(),
                        });
                    }
                }

                if batch.is_empty() {
                    return Action::Idle;
                }

                let mut rng = rand::rng();
                batch.shuffle(&mut rng);

                Action::DispatchBlind(batch)
            }
        }
    }
}
