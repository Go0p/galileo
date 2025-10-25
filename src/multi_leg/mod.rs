//! 多腿组合模块的基础设施。
//!
//! 聚焦腿角色抽象、未签名交易处理等底层能力，为上层策略提供
//! 可复用的组合能力。

#![allow(dead_code)] // TODO: 集成上层逻辑后移除该抑制。

pub mod alt_cache;
pub mod leg;
pub mod orchestrator;
pub mod providers;
pub mod runtime;
pub mod transaction;
pub mod types;

#[allow(unused_imports)]
pub use leg::LegProvider;
#[allow(unused_imports)]
pub use types::{
    AggregatorKind, LegBuildContext, LegDescriptor, LegPlan, LegQuote, LegSide, QuoteIntent,
};
