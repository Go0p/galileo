use crate::engine::types::JitoTipPlan;
use crate::lander::LanderVariant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanderKind {
    Rpc,
    Staked,
    Jito,
}

impl LanderKind {
    pub fn label(self) -> &'static str {
        match self {
            LanderKind::Rpc => "rpc",
            LanderKind::Staked => "staked",
            LanderKind::Jito => "jito",
        }
    }
}

#[derive(Debug, Clone)]
pub enum TipStrategy {
    UseOpportunity,
    Jito {
        plan: Option<JitoTipPlan>,
        label: &'static str,
    },
}

impl TipStrategy {
    pub fn label(&self) -> &'static str {
        match self {
            TipStrategy::UseOpportunity => "opportunity",
            TipStrategy::Jito { label, .. } => label,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardBudgetKind {
    BasePlusTip,
    BasePlusPrioritizationFee,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeUnitPriceStrategy {
    Disabled,
    Fixed(u64),
}

impl ComputeUnitPriceStrategy {
    pub fn value(self) -> Option<u64> {
        match self {
            ComputeUnitPriceStrategy::Disabled => None,
            ComputeUnitPriceStrategy::Fixed(value) => Some(value),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LandingProfile {
    pub lander_kind: LanderKind,
    pub tip_strategy: TipStrategy,
    pub guard_budget: GuardBudgetKind,
    pub compute_unit_strategy: ComputeUnitPriceStrategy,
}

impl LandingProfile {
    pub fn new(
        lander_kind: LanderKind,
        tip_strategy: TipStrategy,
        guard_budget: GuardBudgetKind,
        compute_unit_strategy: ComputeUnitPriceStrategy,
    ) -> Self {
        Self {
            lander_kind,
            tip_strategy,
            guard_budget,
            compute_unit_strategy,
        }
    }
}

#[derive(Default)]
pub struct LandingProfileBuilder;

impl LandingProfileBuilder {
    pub fn new() -> Self {
        Self
    }

    pub fn build_for_variant(
        &self,
        variant: &LanderVariant,
        sampled_compute_unit_price: Option<u64>,
    ) -> LandingProfile {
        match variant {
            LanderVariant::Jito(_) => {
                let tip_plan = variant.draw_tip_plan();
                let label = variant.tip_strategy_label().unwrap_or("stream");
                LandingProfile::new(
                    LanderKind::Jito,
                    TipStrategy::Jito {
                        plan: tip_plan,
                        label,
                    },
                    GuardBudgetKind::BasePlusTip,
                    ComputeUnitPriceStrategy::Fixed(0),
                )
            }
            LanderVariant::Rpc(_) => LandingProfile::new(
                LanderKind::Rpc,
                TipStrategy::UseOpportunity,
                GuardBudgetKind::BasePlusPrioritizationFee,
                sampled_compute_unit_price
                    .filter(|value| *value > 0)
                    .map(ComputeUnitPriceStrategy::Fixed)
                    .unwrap_or(ComputeUnitPriceStrategy::Disabled),
            ),
            LanderVariant::Staked(_) => LandingProfile::new(
                LanderKind::Staked,
                TipStrategy::UseOpportunity,
                GuardBudgetKind::BasePlusPrioritizationFee,
                sampled_compute_unit_price
                    .filter(|value| *value > 0)
                    .map(ComputeUnitPriceStrategy::Fixed)
                    .unwrap_or(ComputeUnitPriceStrategy::Disabled),
            ),
        }
    }
}
