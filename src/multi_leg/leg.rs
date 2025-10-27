use std::fmt::Debug;

use async_trait::async_trait;

use crate::multi_leg::types::{LegBuildContext, LegDescriptor, QuoteIntent};
use crate::network::IpLeaseHandle;

/// 套利腿提供方需要实现的通用接口。
///
/// 该 trait 旨在抽象 Ultra / DFlow / Titan 等聚合器在“腿”语义下
/// 的最小能力集，后续策略层可以基于这些接口进行灵活组合。
#[async_trait]
pub trait LegProvider: Send + Sync + Debug {
    type QuoteResponse: Send + Sync + Debug;
    type BuildError: std::error::Error + Send + Sync + 'static;
    type Plan: Send + Sync + Debug;

    /// 返回当前腿的描述信息。
    fn descriptor(&self) -> LegDescriptor;

    /// 发起报价请求。
    async fn quote(
        &self,
        intent: &QuoteIntent,
        lease: Option<&IpLeaseHandle>,
    ) -> Result<Self::QuoteResponse, Self::BuildError>;

    /// 根据报价构建执行计划，例如未签名交易或指令序列。
    async fn build_plan(
        &self,
        quote: &Self::QuoteResponse,
        context: &LegBuildContext,
        lease: Option<&IpLeaseHandle>,
    ) -> Result<Self::Plan, Self::BuildError>;
}
