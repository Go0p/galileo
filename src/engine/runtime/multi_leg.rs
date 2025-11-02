use std::collections::HashMap;
use std::sync::Arc;

use solana_sdk::pubkey::Pubkey;

use crate::engine::multi_leg::runtime::MultiLegRuntime;
use crate::engine::multi_leg::types::{
    AggregatorKind as MultiLegAggregatorKind, LegBuildContext as MultiLegBuildContext,
    LegDescriptor as MultiLegDescriptor,
};

#[derive(Clone, Copy)]
pub(crate) struct LegCombination {
    pub(crate) buy_index: usize,
    pub(crate) sell_index: usize,
}

#[derive(Default)]
struct LegContextDefaults {
    wrap_and_unwrap_sol: Option<bool>,
    dynamic_compute_unit_limit: Option<bool>,
    compute_unit_limit_multiplier: Option<f64>,
}

pub struct MultiLegEngineContext {
    runtime: Arc<MultiLegRuntime>,
    combinations: Vec<LegCombination>,
    leg_defaults: HashMap<MultiLegAggregatorKind, LegContextDefaults>,
}

impl MultiLegEngineContext {
    pub fn from_runtime(runtime: Arc<MultiLegRuntime>) -> Self {
        let orchestrator = runtime.orchestrator();
        let buy_count = orchestrator.buy_leg_count();
        let sell_count = orchestrator.sell_leg_count();
        let mut combinations = Vec::with_capacity(buy_count.saturating_mul(sell_count));
        for buy_index in 0..buy_count {
            for sell_index in 0..sell_count {
                combinations.push(LegCombination {
                    buy_index,
                    sell_index,
                });
            }
        }
        Self {
            runtime,
            combinations,
            leg_defaults: HashMap::new(),
        }
    }

    pub fn runtime(&self) -> &MultiLegRuntime {
        &self.runtime
    }

    pub fn combinations(&self) -> &[LegCombination] {
        &self.combinations
    }

    pub fn set_wrap_and_unwrap_sol(&mut self, kind: MultiLegAggregatorKind, value: bool) {
        self.leg_defaults
            .entry(kind)
            .or_default()
            .wrap_and_unwrap_sol = Some(value);
    }

    pub fn set_dynamic_compute_unit_limit(&mut self, kind: MultiLegAggregatorKind, value: bool) {
        self.leg_defaults
            .entry(kind)
            .or_default()
            .dynamic_compute_unit_limit = Some(value);
    }

    pub fn set_compute_unit_limit_multiplier(
        &mut self,
        kind: MultiLegAggregatorKind,
        multiplier: f64,
    ) {
        self.leg_defaults
            .entry(kind)
            .or_default()
            .compute_unit_limit_multiplier = Some(multiplier);
    }

    pub fn build_context(
        &self,
        descriptor: &MultiLegDescriptor,
        payer: Pubkey,
        compute_unit_price: Option<u64>,
    ) -> MultiLegBuildContext {
        let mut ctx = MultiLegBuildContext::default();
        ctx.payer = payer;
        ctx.compute_unit_price_micro_lamports = compute_unit_price;
        if let Some(defaults) = self.leg_defaults.get(&descriptor.kind) {
            if let Some(flag) = defaults.wrap_and_unwrap_sol {
                ctx.wrap_and_unwrap_sol = Some(flag);
            }
            if let Some(flag) = defaults.dynamic_compute_unit_limit {
                ctx.dynamic_compute_unit_limit = Some(flag);
            }
            if let Some(multiplier) = defaults.compute_unit_limit_multiplier {
                ctx.compute_unit_limit_multiplier = Some(multiplier);
            }
        }
        ctx
    }
}
