use std::fmt;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use futures::TryFutureExt;
use tracing::debug;

use crate::engine::multi_leg::leg::LegProvider;
use crate::engine::multi_leg::types::{
    LegBuildContext, LegDescriptor, LegPlan, LegQuote, LegSide, QuoteIntent,
};
use crate::network::IpLeaseHandle;

#[async_trait]
pub trait DynLegProvider: Send + Sync {
    fn descriptor(&self) -> LegDescriptor;

    async fn quote(
        &self,
        intent: &QuoteIntent,
        lease: Option<&IpLeaseHandle>,
    ) -> Result<LegQuoteHandle>;
}

#[async_trait]
trait DynQuoteState: Send + Sync {
    async fn build_plan(
        &self,
        context: &LegBuildContext,
        lease: Option<&IpLeaseHandle>,
    ) -> Result<LegPlan>;
}

struct LegProviderAdapter<P> {
    inner: Arc<P>,
}

impl<P> LegProviderAdapter<P> {
    fn new(inner: Arc<P>) -> Self {
        Self { inner }
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
        self.inner.descriptor()
    }

    async fn quote(
        &self,
        intent: &QuoteIntent,
        lease: Option<&IpLeaseHandle>,
    ) -> Result<LegQuoteHandle> {
        let raw_quote = self
            .inner
            .quote(intent, lease)
            .await
            .map_err(anyhow::Error::new)?;
        let summary = self.inner.summarize_quote(&raw_quote);
        let descriptor = self.inner.descriptor();
        let state: Arc<dyn DynQuoteState> = Arc::new(ProviderQuoteState {
            provider: Arc::clone(&self.inner),
            quote: raw_quote,
        });
        Ok(LegQuoteHandle::new(descriptor, summary, state))
    }
}

struct ProviderQuoteState<P>
where
    P: LegProvider<Plan = LegPlan>,
{
    provider: Arc<P>,
    quote: P::QuoteResponse,
}

#[async_trait]
impl<P> DynQuoteState for ProviderQuoteState<P>
where
    P: LegProvider<Plan = LegPlan> + Send + Sync + 'static,
    P::QuoteResponse: Send + Sync,
    P::BuildError: std::error::Error + Send + Sync + 'static,
{
    async fn build_plan(
        &self,
        context: &LegBuildContext,
        lease: Option<&IpLeaseHandle>,
    ) -> Result<LegPlan> {
        self.provider
            .build_plan(&self.quote, context, lease)
            .await
            .map_err(anyhow::Error::new)
    }
}

#[derive(Clone)]
pub struct LegQuoteHandle {
    descriptor: LegDescriptor,
    quote: LegQuote,
    state: Arc<dyn DynQuoteState>,
}

impl fmt::Debug for LegQuoteHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LegQuoteHandle")
            .field("descriptor", &self.descriptor)
            .field("quote", &self.quote)
            .finish()
    }
}

impl LegQuoteHandle {
    fn new(descriptor: LegDescriptor, quote: LegQuote, state: Arc<dyn DynQuoteState>) -> Self {
        Self {
            descriptor,
            quote,
            state,
        }
    }

    pub fn descriptor(&self) -> &LegDescriptor {
        &self.descriptor
    }

    pub fn quote(&self) -> &LegQuote {
        &self.quote
    }

    pub async fn build_plan(
        &self,
        context: &LegBuildContext,
        lease: Option<&IpLeaseHandle>,
    ) -> Result<LegPlan> {
        self.state.build_plan(context, lease).await
    }
}

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

    pub fn register_provider<P>(&mut self, provider: Arc<P>)
    where
        P: LegProvider<Plan = LegPlan> + Send + Sync + 'static,
        P::QuoteResponse: Send + Sync,
        P::BuildError: std::error::Error + Send + Sync + 'static,
    {
        let adapter: Arc<dyn DynLegProvider> = Arc::new(LegProviderAdapter::new(provider));
        let descriptor = adapter.descriptor();
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

    pub fn register_owned_provider<P>(&mut self, provider: P)
    where
        P: LegProvider<Plan = LegPlan> + Send + Sync + 'static,
        P::QuoteResponse: Send + Sync,
        P::BuildError: std::error::Error + Send + Sync + 'static,
    {
        self.register_provider(Arc::new(provider));
    }

    pub fn buy_legs(&self) -> Vec<LegDescriptor> {
        self.buy_legs
            .iter()
            .map(|entry| entry.descriptor.clone())
            .collect()
    }

    pub fn buy_leg_count(&self) -> usize {
        self.buy_legs.len()
    }

    pub fn sell_legs(&self) -> Vec<LegDescriptor> {
        self.sell_legs
            .iter()
            .map(|entry| entry.descriptor.clone())
            .collect()
    }

    pub fn sell_leg_count(&self) -> usize {
        self.sell_legs.len()
    }

    pub fn descriptor(&self, side: LegSide, index: usize) -> Option<&LegDescriptor> {
        match side {
            LegSide::Buy => self.buy_legs.get(index),
            LegSide::Sell => self.sell_legs.get(index),
        }
        .map(|entry| &entry.descriptor)
    }

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

    pub async fn quote_pair(
        &self,
        buy_index: usize,
        sell_index: usize,
        buy_intent: &QuoteIntent,
        sell_intent: &QuoteIntent,
        buy_lease: Option<&IpLeaseHandle>,
        sell_lease: Option<&IpLeaseHandle>,
    ) -> Result<LegPairQuote> {
        let buy_entry = self
            .buy_legs
            .get(buy_index)
            .ok_or_else(|| anyhow!("买腿索引 {buy_index} 超出范围"))?;
        let sell_entry = self
            .sell_legs
            .get(sell_index)
            .ok_or_else(|| anyhow!("卖腿索引 {sell_index} 超出范围"))?;

        let buy_quote = buy_entry
            .provider
            .quote(buy_intent, buy_lease)
            .await
            .map_err(|err| anyhow!("买腿报价失败: {err}"))?;

        let sell_amount = buy_quote
            .quote()
            .min_out_amount
            .unwrap_or(buy_quote.quote().amount_out);
        let mut adjusted_sell_intent = sell_intent.clone();
        adjusted_sell_intent.amount = sell_amount;

        let sell_quote = sell_entry
            .provider
            .quote(&adjusted_sell_intent, sell_lease)
            .await
            .map_err(|err| anyhow!("卖腿报价失败: {err}"))?;

        if sell_quote.quote().amount_in != sell_amount {
            debug!(
                target: "multi_leg::orchestrator",
                expected = sell_amount,
                actual = sell_quote.quote().amount_in,
                side = %sell_entry.descriptor.side,
                kind = %sell_entry.descriptor.kind,
                "卖腿实际输入与期望不一致"
            );
        }

        Ok(LegPairQuote {
            buy: buy_quote,
            sell: sell_quote,
        })
    }

    pub async fn build_pair_plan(
        &self,
        pair_quote: &LegPairQuote,
        buy_context: &LegBuildContext,
        sell_context: &LegBuildContext,
        buy_lease: Option<&IpLeaseHandle>,
        sell_lease: Option<&IpLeaseHandle>,
    ) -> Result<LegPairPlan> {
        let buy_future = pair_quote
            .buy
            .build_plan(buy_context, buy_lease)
            .map_err(|err| anyhow!("买腿计划失败: {err}"));
        let sell_future = pair_quote
            .sell
            .build_plan(sell_context, sell_lease)
            .map_err(|err| anyhow!("卖腿计划失败: {err}"));

        let (mut buy_plan, sell_plan) = tokio::try_join!(buy_future, sell_future)?;
        let sell_amount = sell_plan.quote.amount_in;
        buy_plan.quote.min_out_amount = Some(sell_amount);

        Ok(LegPairPlan {
            buy: buy_plan,
            sell: sell_plan,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LegPairQuote {
    pub buy: LegQuoteHandle,
    pub sell: LegQuoteHandle,
}

impl LegPairQuote {
    pub fn estimated_gross_profit(&self) -> i128 {
        self.sell.quote().amount_out as i128 - self.buy.quote().amount_in as i128
    }
}

#[derive(Debug, Clone)]
pub struct LegPairDescriptor {
    pub buy: LegDescriptor,
    pub sell: LegDescriptor,
}

#[derive(Debug)]
pub struct LegPairPlan {
    pub buy: LegPlan,
    pub sell: LegPlan,
}
