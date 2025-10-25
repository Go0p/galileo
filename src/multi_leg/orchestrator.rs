use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures::future::join_all;
use tracing::debug;

use crate::multi_leg::leg::LegProvider;
use crate::multi_leg::types::{LegBuildContext, LegDescriptor, LegPlan, LegSide, QuoteIntent};

/// 对外暴露的动态腿提供方接口，统一 quote + plan 调用。
#[async_trait]
pub trait DynLegProvider: Send + Sync {
    fn descriptor(&self) -> LegDescriptor;

    async fn plan(&self, intent: &QuoteIntent, context: &LegBuildContext) -> Result<LegPlan>;
}

/// 将任意实现 [`LegProvider`] 的类型适配为 [`DynLegProvider`]。
struct LegProviderAdapter<P> {
    descriptor: LegDescriptor,
    inner: Arc<P>,
}

impl<P> LegProviderAdapter<P> {
    pub fn new(descriptor: LegDescriptor, inner: Arc<P>) -> Self {
        Self { descriptor, inner }
    }
}

#[async_trait]
impl<P> DynLegProvider for LegProviderAdapter<P>
where
    P: LegProvider<Plan = LegPlan> + Send + Sync + 'static,
    P::QuoteResponse: Send + Sync,
    P::BuildError: std::error::Error + Send + Sync + 'static,
{
    fn descriptor(&self) -> LegDescriptor {
        self.descriptor.clone()
    }

    async fn plan(&self, intent: &QuoteIntent, context: &LegBuildContext) -> Result<LegPlan> {
        let quote = self.inner.quote(intent).await.map_err(anyhow::Error::new)?;
        let plan = self
            .inner
            .build_plan(&quote, context)
            .await
            .map_err(anyhow::Error::new)?;
        Ok(plan)
    }
}

/// 统一管理多条腿提供方，并给出组合配对能力。
#[derive(Default)]
pub struct MultiLegOrchestrator {
    buy_legs: Vec<LegEntry>,
    sell_legs: Vec<LegEntry>,
}

struct LegEntry {
    descriptor: LegDescriptor,
    provider: Arc<dyn DynLegProvider>,
}

impl MultiLegOrchestrator {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册具体的腿提供方，自动按买/卖腿归档。
    pub fn register_provider<P>(&mut self, provider: Arc<P>)
    where
        P: LegProvider<Plan = LegPlan> + Send + Sync + 'static,
        P::QuoteResponse: Send + Sync,
        P::BuildError: std::error::Error + Send + Sync + 'static,
    {
        let descriptor = provider.descriptor();
        let adapter: Arc<dyn DynLegProvider> =
            Arc::new(LegProviderAdapter::new(descriptor.clone(), provider));
        let entry = LegEntry {
            descriptor: descriptor.clone(),
            provider: adapter,
        };

        debug!(
            target: "multi_leg::orchestrator",
            side = %descriptor.side,
            kind = %descriptor.kind,
            "注册腿提供方"
        );

        match descriptor.side {
            LegSide::Buy => self.buy_legs.push(entry),
            LegSide::Sell => self.sell_legs.push(entry),
        }
    }

    /// 便捷函数：支持直接传入拥有所有权的 provider。
    pub fn register_owned_provider<P>(&mut self, provider: P)
    where
        P: LegProvider<Plan = LegPlan> + Send + Sync + 'static,
        P::QuoteResponse: Send + Sync,
        P::BuildError: std::error::Error + Send + Sync + 'static,
    {
        self.register_provider(Arc::new(provider));
    }

    /// 当前所有可用的买腿描述。
    pub fn buy_legs(&self) -> Vec<LegDescriptor> {
        self.buy_legs
            .iter()
            .map(|entry| entry.descriptor.clone())
            .collect()
    }

    /// 当前所有可用的卖腿描述。
    pub fn sell_legs(&self) -> Vec<LegDescriptor> {
        self.sell_legs
            .iter()
            .map(|entry| entry.descriptor.clone())
            .collect()
    }

    /// 获取指定侧指定索引的腿描述。
    pub fn descriptor(&self, side: LegSide, index: usize) -> Option<&LegDescriptor> {
        match side {
            LegSide::Buy => self.buy_legs.get(index),
            LegSide::Sell => self.sell_legs.get(index),
        }
        .map(|entry| &entry.descriptor)
    }

    /// 买卖腿笛卡尔积，后续用于收益评估和配对。
    pub fn available_pairs(&self) -> Vec<LegPairDescriptor> {
        let mut pairs = Vec::new();
        for buy in &self.buy_legs {
            for sell in &self.sell_legs {
                pairs.push(LegPairDescriptor {
                    buy: buy.descriptor.clone(),
                    sell: sell.descriptor.clone(),
                });
            }
        }
        pairs
    }

    /// 对指定腿侧整体尝试构建执行计划，返回每条腿的结果。
    pub async fn plan_side(
        &self,
        side: LegSide,
        intent: &QuoteIntent,
        context: &LegBuildContext,
    ) -> Vec<PlanAttempt> {
        let entries = match side {
            LegSide::Buy => &self.buy_legs,
            LegSide::Sell => &self.sell_legs,
        };

        let futures = entries.iter().map(|entry| {
            let descriptor = entry.descriptor.clone();
            let provider = Arc::clone(&entry.provider);
            async move {
                let result = provider.plan(intent, context).await;
                PlanAttempt { descriptor, result }
            }
        });

        join_all(futures).await
    }

    /// 指定买腿与卖腿组合，生成一组计划。
    pub async fn plan_pair(
        &self,
        buy_index: usize,
        sell_index: usize,
        buy_intent: &QuoteIntent,
        sell_intent: &QuoteIntent,
        buy_context: &LegBuildContext,
        sell_context: &LegBuildContext,
    ) -> Result<LegPairPlan> {
        let buy_entry = self
            .buy_legs
            .get(buy_index)
            .ok_or_else(|| anyhow!("买腿索引 {buy_index} 超出范围"))?;
        let sell_entry = self
            .sell_legs
            .get(sell_index)
            .ok_or_else(|| anyhow!("卖腿索引 {sell_index} 超出范围"))?;

        let buy_provider = Arc::clone(&buy_entry.provider);
        let sell_provider = Arc::clone(&sell_entry.provider);

        let mut buy_plan = buy_provider
            .plan(buy_intent, buy_context)
            .await
            .map_err(|err| anyhow!("买腿计划失败: {err}"))?;

        let sell_amount = buy_plan
            .quote
            .min_out_amount
            .unwrap_or(buy_plan.quote.amount_out);
        let mut adjusted_sell_intent = sell_intent.clone();
        adjusted_sell_intent.amount = sell_amount;
        let sell_plan = sell_provider
            .plan(&adjusted_sell_intent, sell_context)
            .await
            .map_err(|err| anyhow!("卖腿计划失败: {err}"))?;

        if sell_plan.quote.amount_in != sell_amount {
            debug!(
                target: "multi_leg::orchestrator",
                expected = sell_amount,
                actual = sell_plan.quote.amount_in,
                side = %sell_entry.descriptor.side,
                kind = %sell_entry.descriptor.kind,
                "卖腿实际输入与期望不一致"
            );
        }

        buy_plan.quote.min_out_amount = Some(sell_amount);

        Ok(LegPairPlan {
            buy: buy_plan,
            sell: sell_plan,
        })
    }
}

/// 描述一条买卖腿组合的基础信息。
#[derive(Debug, Clone)]
pub struct LegPairDescriptor {
    pub buy: LegDescriptor,
    pub sell: LegDescriptor,
}

/// 带有执行结果的腿尝试。
#[derive(Debug)]
pub struct PlanAttempt {
    pub descriptor: LegDescriptor,
    pub result: Result<LegPlan>,
}

/// 买/卖腿计划结果。
#[derive(Debug)]
pub struct LegPairPlan {
    pub buy: LegPlan,
    pub sell: LegPlan,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::multi_leg::types::{AggregatorKind, LegPlan, LegQuote};
    use solana_sdk::instruction::Instruction;
    use std::sync::Arc;
    use tokio::runtime::Runtime;

    struct MockProvider {
        descriptor: LegDescriptor,
        plan: LegPlan,
    }

    #[async_trait]
    impl LegProvider for MockProvider {
        type QuoteResponse = ();
        type BuildError = anyhow::Error;
        type Plan = LegPlan;

        fn descriptor(&self) -> LegDescriptor {
            self.descriptor.clone()
        }

        async fn quote(
            &self,
            _intent: &QuoteIntent,
        ) -> Result<Self::QuoteResponse, Self::BuildError> {
            Ok(())
        }

        async fn build_plan(
            &self,
            _quote: &Self::QuoteResponse,
            _context: &LegBuildContext,
        ) -> Result<Self::Plan, Self::BuildError> {
            Ok(self.plan.clone())
        }
    }

    fn mock_plan(kind: AggregatorKind, side: LegSide) -> MockProvider {
        let descriptor = LegDescriptor::new(kind, side);
        let (amount_in, amount_out) = match side {
            LegSide::Buy => (100, 90),
            LegSide::Sell => (90, 100),
        };
        let mut quote = LegQuote::new(amount_in, amount_out, 50);
        if let LegSide::Buy = side {
            quote.min_out_amount = Some(amount_out);
        }
        let plan = LegPlan {
            descriptor: descriptor.clone(),
            quote,
            instructions: vec![Instruction::new_with_bytes(
                solana_sdk::pubkey::Pubkey::new_unique(),
                &[1, 2, 3],
                vec![],
            )],
            compute_budget_instructions: Vec::new(),
            address_lookup_table_addresses: Vec::new(),
            resolved_lookup_tables: Vec::new(),
            prioritization_fee_lamports: None,
            blockhash: None,
            raw_transaction: None,
        };
        MockProvider { descriptor, plan }
    }

    #[test]
    fn register_and_list_pairs() {
        let mut orchestrator = MultiLegOrchestrator::new();
        orchestrator.register_owned_provider(mock_plan(AggregatorKind::Ultra, LegSide::Buy));
        orchestrator.register_owned_provider(mock_plan(AggregatorKind::Dflow, LegSide::Sell));

        let buys = orchestrator.buy_legs();
        let sells = orchestrator.sell_legs();
        assert_eq!(buys.len(), 1);
        assert_eq!(sells.len(), 1);

        let pairs = orchestrator.available_pairs();
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].buy.kind, AggregatorKind::Ultra);
        assert_eq!(pairs[0].sell.kind, AggregatorKind::Dflow);
    }

    #[test]
    fn plan_pair_returns_leg_plans() {
        let mut orchestrator = MultiLegOrchestrator::new();
        orchestrator.register_owned_provider(mock_plan(AggregatorKind::Ultra, LegSide::Buy));
        orchestrator.register_owned_provider(mock_plan(AggregatorKind::Dflow, LegSide::Sell));

        let runtime = Runtime::new().expect("runtime");
        runtime.block_on(async {
            let buy_intent = QuoteIntent::new(
                solana_sdk::pubkey::Pubkey::new_unique(),
                solana_sdk::pubkey::Pubkey::new_unique(),
                100,
                10,
            );
            let sell_intent = QuoteIntent::new(
                solana_sdk::pubkey::Pubkey::new_unique(),
                solana_sdk::pubkey::Pubkey::new_unique(),
                100,
                10,
            );
            let context = LegBuildContext::default();

            let plan = orchestrator
                .plan_pair(0, 0, &buy_intent, &sell_intent, &context, &context)
                .await
                .expect("plan pair");

            assert_eq!(plan.buy.instructions.len(), 1);
            assert_eq!(plan.sell.instructions.len(), 1);
        });
    }
}
