mod aggregator;
pub mod assembly;
mod builder;
mod context;
mod error;
mod identity;
pub mod landing;
pub mod multi_leg;
mod planner;
pub mod plugins;
mod precheck;
mod profit;
mod quote;
mod quote_cadence;
mod quote_dispatcher;
mod runtime;
mod scheduler;
mod swap_preparer;
mod types;
pub mod ultra;

pub use crate::instructions::compute_budget::COMPUTE_BUDGET_PROGRAM_ID;
pub use aggregator::{MultiLegInstructions, SwapInstructionsVariant};
pub use builder::{BuilderConfig, TransactionBuilder};
pub use context::{Action, StrategyContext, StrategyDecision};
pub use error::{EngineError, EngineResult};
pub use identity::EngineIdentity;
pub use planner::{DispatchPlan, DispatchStrategy, TxVariant, TxVariantPlanner, VariantId};
pub use precheck::AccountPrechecker;
pub use profit::{ProfitConfig, ProfitEvaluator, TipConfig};
pub use quote::{QuoteConfig, QuoteExecutor};
pub use quote_cadence::QuoteCadence;
pub use quote_dispatcher::QuoteDispatcher;
pub(crate) use runtime::LighthouseRuntime;
pub use runtime::MultiLegEngineContext;
pub(crate) use runtime::strategy::MintSchedule;
pub use runtime::strategy::{
    ConsoleSummarySettings, EngineSettings, LighthouseSettings, SolPriceFeedSettings,
    StrategyEngine,
};
pub use scheduler::Scheduler;
pub use swap_preparer::{ComputeUnitPriceMode, SwapPreparer};
#[allow(unused_imports)]
pub use types::{JitoTipPlan, QuoteTask, StrategyTick, SwapOpportunity, TradeProfile};

pub const FALLBACK_CU_LIMIT: u32 = 230_000;
