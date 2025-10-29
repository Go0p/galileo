use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Error, Result, anyhow};
use futures::future::{join_all, try_join};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use tokio::sync::{Mutex, OwnedSemaphorePermit, Semaphore};
use tokio::time::sleep;
use tracing::{debug, instrument, warn};

use crate::multi_leg::alt_cache::AltCache;
use crate::multi_leg::orchestrator::{LegPairDescriptor, LegPairPlan, MultiLegOrchestrator};
use crate::multi_leg::transaction::instructions::{
    InstructionExtractionError, extract_instructions,
};
use crate::multi_leg::types::{
    AggregatorKind, LegBuildContext, LegDescriptor, LegPlan, LegSide, QuoteIntent, SignerRewrite,
};
use crate::multi_leg::types::{
    AggregatorKind as MultiLegAggregatorKind, LegSide as MultiLegLegSide,
};
use crate::network::{IpAllocator, IpLeaseHandle, IpLeaseMode, IpTaskKind};

/// 运行时容器：封装 orchestrator、ALT 缓存与 RPC，提供高层计划接口。
pub struct MultiLegRuntime {
    orchestrator: MultiLegOrchestrator,
    alt_cache: AltCache,
    rpc: Arc<RpcClient>,
    concurrency: ConcurrencyPolicy,
    ip_allocator: Arc<IpAllocator>,
}

/// 运行时行为调优项。
#[derive(Debug, Clone)]
pub struct MultiLegRuntimeConfig {
    /// Titan 推流同一时间可并发的腿数上限；0 表示不限制。
    pub titan_stream_limit: usize,
    /// Titan 推流落地的最小间隔（毫秒）。置 0 表示不做防抖。
    pub titan_debounce_ms: u64,
}

impl Default for MultiLegRuntimeConfig {
    fn default() -> Self {
        Self {
            titan_stream_limit: 2,
            titan_debounce_ms: 200,
        }
    }
}

impl MultiLegRuntime {
    pub fn new(
        orchestrator: MultiLegOrchestrator,
        alt_cache: AltCache,
        rpc: Arc<RpcClient>,
        ip_allocator: Arc<IpAllocator>,
    ) -> Self {
        Self::with_config(
            orchestrator,
            alt_cache,
            rpc,
            ip_allocator,
            MultiLegRuntimeConfig::default(),
        )
    }

    pub fn with_config(
        orchestrator: MultiLegOrchestrator,
        alt_cache: AltCache,
        rpc: Arc<RpcClient>,
        ip_allocator: Arc<IpAllocator>,
        config: MultiLegRuntimeConfig,
    ) -> Self {
        Self {
            orchestrator,
            alt_cache,
            rpc,
            concurrency: ConcurrencyPolicy::new(config),
            ip_allocator,
        }
    }

    pub fn orchestrator_mut(&mut self) -> &mut MultiLegOrchestrator {
        &mut self.orchestrator
    }

    pub fn orchestrator(&self) -> &MultiLegOrchestrator {
        &self.orchestrator
    }

    pub fn alt_cache(&self) -> &AltCache {
        &self.alt_cache
    }

    /// 构建指定腿侧的计划，并补齐 ALT。
    #[instrument(skip(self, intent, context), fields(side = ?side))]
    pub async fn plan_side_with_alts(
        &self,
        side: LegSide,
        intent: &QuoteIntent,
        context: &LegBuildContext,
    ) -> Vec<LegPlanResult> {
        let attempts = self.orchestrator.plan_side(side, intent, context).await;

        let mut results = Vec::with_capacity(attempts.len());
        for attempt in attempts {
            match attempt.result {
                Ok(mut plan) => {
                    if let Err(err) = self.populate_leg_plan(&mut plan).await {
                        debug!(
                            target = "multi_leg::runtime",
                            descriptor = ?attempt.descriptor,
                            error = %err,
                            "填充 ALT 失败"
                        );
                        results.push(LegPlanResult::Failed {
                            descriptor: attempt.descriptor,
                            error: err,
                        });
                    } else {
                        results.push(LegPlanResult::Success {
                            descriptor: attempt.descriptor,
                            plan,
                        });
                    }
                }
                Err(error) => {
                    results.push(LegPlanResult::Failed {
                        descriptor: attempt.descriptor,
                        error,
                    });
                }
            }
        }
        results
    }

    /// 构建指定买/卖腿组合，并填充 ALT。
    #[instrument(
        skip(self, buy_intent, sell_intent, buy_context, sell_context),
        fields(buy_index, sell_index)
    )]
    pub async fn plan_pair_with_alts(
        &self,
        buy_index: usize,
        sell_index: usize,
        buy_intent: &QuoteIntent,
        sell_intent: &QuoteIntent,
        buy_context: &LegBuildContext,
        sell_context: &LegBuildContext,
    ) -> Result<LegPairPlan> {
        let mut plan = self
            .plan_pair_raw(
                buy_index,
                sell_index,
                buy_intent,
                sell_intent,
                buy_context,
                sell_context,
            )
            .await?;

        self.populate_pair_plan(&mut plan).await?;

        Ok(plan)
    }

    async fn plan_pair_raw(
        &self,
        buy_index: usize,
        sell_index: usize,
        buy_intent: &QuoteIntent,
        sell_intent: &QuoteIntent,
        buy_context: &LegBuildContext,
        sell_context: &LegBuildContext,
    ) -> Result<LegPairPlan> {
        let buy_descriptor = self
            .orchestrator
            .descriptor(LegSide::Buy, buy_index)
            .cloned()
            .ok_or_else(|| anyhow!("买腿索引 {buy_index} 超出范围"))?;
        let sell_descriptor = self
            .orchestrator
            .descriptor(LegSide::Sell, sell_index)
            .cloned()
            .ok_or_else(|| anyhow!("卖腿索引 {sell_index} 超出范围"))?;

        let buy_handle = self.acquire_leg_handle(&buy_descriptor).await?;
        let sell_handle = self.acquire_leg_handle(&sell_descriptor).await?;

        let _buy_guard = self.concurrency.acquire(&buy_descriptor).await;
        let _sell_guard = self.concurrency.acquire(&sell_descriptor).await;

        let result = self
            .orchestrator
            .plan_pair(
                buy_index,
                sell_index,
                buy_intent,
                sell_intent,
                buy_context,
                sell_context,
                Some(&buy_handle),
                Some(&sell_handle),
            )
            .await;

        drop(buy_handle);
        drop(sell_handle);

        result
    }

    async fn acquire_leg_handle(&self, descriptor: &LegDescriptor) -> Result<IpLeaseHandle> {
        let task_kind = to_ip_task_kind(descriptor);
        let lease = self
            .ip_allocator
            .acquire(task_kind, IpLeaseMode::Ephemeral)
            .await
            .map_err(|err| {
                anyhow!(
                    "获取 {:?} {:?} IP 资源失败: {err}",
                    descriptor.kind,
                    descriptor.side
                )
            })?;
        let handle = lease.handle();
        drop(lease);
        Ok(handle)
    }

    pub async fn populate_pair_plan(&self, plan: &mut LegPairPlan) -> Result<()> {
        try_join(
            self.populate_leg_plan(&mut plan.buy),
            self.populate_leg_plan(&mut plan.sell),
        )
        .await
        .map(|_| ())
    }

    /// 同时对多条腿组合请求进行规划，输出按收益降序排列的计划列表。
    pub async fn plan_pair_batch_with_profit(
        &self,
        requests: Vec<PairPlanRequest>,
    ) -> PairPlanBatchResult {
        if requests.is_empty() {
            return PairPlanBatchResult::default();
        }

        let futures: Vec<_> = requests
            .into_iter()
            .map(|request| {
                let runtime = self;
                async move {
                    let result = runtime
                        .plan_pair_raw(
                            request.buy_index,
                            request.sell_index,
                            &request.buy_intent,
                            &request.sell_intent,
                            &request.buy_context,
                            &request.sell_context,
                        )
                        .await;
                    (request, result)
                }
            })
            .collect();

        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for (request, outcome) in join_all(futures).await {
            match outcome {
                Ok(plan) => {
                    let profit_lamports = calculate_profit(&plan);
                    successes.push(PairPlanEvaluation {
                        descriptor: LegPairDescriptor {
                            buy: plan.buy.descriptor.clone(),
                            sell: plan.sell.descriptor.clone(),
                        },
                        trade_size: request.trade_size(),
                        tag: request.tag,
                        plan,
                        profit_lamports,
                    });
                }
                Err(error) => {
                    warn!(
                        target = "multi_leg::runtime",
                        buy_index = request.buy_index,
                        sell_index = request.sell_index,
                        trade_size = request.trade_size(),
                        error = %error,
                        "双腿计划构建失败"
                    );
                    failures.push(PairPlanFailure {
                        buy_index: request.buy_index,
                        sell_index: request.sell_index,
                        trade_size: request.trade_size(),
                        error,
                    });
                }
            }
        }

        successes.sort_by(|a, b| b.profit_lamports.cmp(&a.profit_lamports));
        PairPlanBatchResult {
            successes,
            failures,
        }
    }

    async fn populate_leg_plan(&self, plan: &mut LegPlan) -> Result<()> {
        let mut unique = Vec::new();
        let mut seen = HashMap::new();
        for key in &plan.address_lookup_table_addresses {
            if !seen.contains_key(key) {
                seen.insert(*key, ());
                unique.push(*key);
            }
        }

        if unique.is_empty() {
            plan.resolved_lookup_tables.clear();
            self.rebuild_plan_instructions(plan)?;
            return Ok(());
        }

        let tables = self.alt_cache.fetch_many(&self.rpc, &unique).await?;
        let mut table_map: HashMap<Pubkey, _> =
            tables.into_iter().map(|table| (table.key, table)).collect();

        plan.resolved_lookup_tables = plan
            .address_lookup_table_addresses
            .iter()
            .filter_map(|key| table_map.remove(key))
            .collect();

        self.rebuild_plan_instructions(plan)?;

        Ok(())
    }

    fn rebuild_plan_instructions(&self, plan: &mut LegPlan) -> Result<()> {
        let needs_rebuild = plan.raw_transaction.is_some()
            && (plan.instructions.is_empty() || plan.compute_budget_instructions.is_empty());

        if needs_rebuild {
            let tx = plan
                .raw_transaction
                .as_ref()
                .expect("guard ensures raw_transaction exists");

            let bundle =
                extract_instructions(&tx.message, Some(plan.resolved_lookup_tables.as_slice()))
                    .map_err(|err| match err {
                        InstructionExtractionError::MissingLookupTables { count } => {
                            anyhow!("地址查找表缺失: 仍需 {count} 个")
                        }
                        InstructionExtractionError::LookupTableNotFound { table } => {
                            anyhow!("地址查找表 {table} 未解析")
                        }
                        InstructionExtractionError::LookupIndexOutOfBounds {
                            table,
                            index,
                            len,
                        } => {
                            anyhow!("地址查找表 {table} 索引 {index} 超出范围 (len = {len})")
                        }
                        InstructionExtractionError::ProgramIndexOutOfBounds { index, total } => {
                            anyhow!("program index {index} 超出账户数量 {total}")
                        }
                        InstructionExtractionError::AccountIndexOutOfBounds { index, total } => {
                            anyhow!("account index {index} 超出账户数量 {total}")
                        }
                    })?;

            plan.compute_budget_instructions = bundle.compute_budget_instructions;
            plan.instructions = bundle.other_instructions;
        }

        if let Some(rewrite) = plan.signer_rewrite {
            rewrite_instruction_accounts(&mut plan.compute_budget_instructions, rewrite);
            rewrite_instruction_accounts(&mut plan.instructions, rewrite);
        }
        if !plan.account_rewrites.is_empty() {
            rewrite_instruction_accounts_map(
                &mut plan.compute_budget_instructions,
                &plan.account_rewrites,
            );
            rewrite_instruction_accounts_map(&mut plan.instructions, &plan.account_rewrites);
        }

        Ok(())
    }
}

fn rewrite_instruction_accounts(instructions: &mut [Instruction], rewrite: SignerRewrite) {
    for ix in instructions {
        for account in &mut ix.accounts {
            if account.pubkey == rewrite.original {
                account.pubkey = rewrite.replacement;
            }
        }
    }
}

fn rewrite_instruction_accounts_map(
    instructions: &mut [Instruction],
    rewrites: &[(Pubkey, Pubkey)],
) {
    for ix in instructions {
        for account in &mut ix.accounts {
            for (from, to) in rewrites {
                if account.pubkey == *from {
                    account.pubkey = *to;
                    break;
                }
            }
        }
    }
}

pub enum LegPlanResult {
    Success {
        descriptor: LegDescriptor,
        plan: LegPlan,
    },
    Failed {
        descriptor: LegDescriptor,
        error: Error,
    },
}

#[derive(Debug, Clone)]
pub struct PairPlanRequest {
    pub buy_index: usize,
    pub sell_index: usize,
    pub buy_intent: QuoteIntent,
    pub sell_intent: QuoteIntent,
    pub buy_context: LegBuildContext,
    pub sell_context: LegBuildContext,
    pub tag: Option<String>,
}

impl PairPlanRequest {
    pub fn trade_size(&self) -> u64 {
        self.buy_intent.amount
    }
}

#[derive(Debug, Default)]
pub struct PairPlanBatchResult {
    pub successes: Vec<PairPlanEvaluation>,
    pub failures: Vec<PairPlanFailure>,
}

impl PairPlanBatchResult {
    pub fn best(&self) -> Option<&PairPlanEvaluation> {
        self.successes.first()
    }
}

#[derive(Debug)]
pub struct PairPlanEvaluation {
    pub descriptor: LegPairDescriptor,
    pub trade_size: u64,
    pub tag: Option<String>,
    pub plan: LegPairPlan,
    pub profit_lamports: i128,
}

#[derive(Debug)]
pub struct PairPlanFailure {
    pub buy_index: usize,
    pub sell_index: usize,
    pub trade_size: u64,
    pub error: Error,
}

struct ConcurrencyPolicy {
    titan: Option<TitanControl>,
}

impl ConcurrencyPolicy {
    fn new(config: MultiLegRuntimeConfig) -> Self {
        let titan = if config.titan_stream_limit == 0 && config.titan_debounce_ms == 0 {
            None
        } else {
            Some(TitanControl {
                semaphore: (config.titan_stream_limit > 0)
                    .then(|| Arc::new(Semaphore::new(config.titan_stream_limit))),
                throttle: (config.titan_debounce_ms > 0)
                    .then(|| TitanThrottle::new(Duration::from_millis(config.titan_debounce_ms))),
            })
        };

        Self { titan }
    }

    async fn acquire(&self, descriptor: &LegDescriptor) -> Option<OwnedSemaphorePermit> {
        if descriptor.kind != AggregatorKind::Titan {
            return None;
        }
        if let Some(control) = &self.titan {
            control.acquire().await
        } else {
            None
        }
    }
}

struct TitanControl {
    semaphore: Option<Arc<Semaphore>>,
    throttle: Option<TitanThrottle>,
}

impl TitanControl {
    async fn acquire(&self) -> Option<OwnedSemaphorePermit> {
        if let Some(throttle) = &self.throttle {
            throttle.wait().await;
        }
        if let Some(semaphore) = &self.semaphore {
            Some(
                semaphore
                    .clone()
                    .acquire_owned()
                    .await
                    .expect("Titan semaphore closed unexpectedly"),
            )
        } else {
            None
        }
    }
}

struct TitanThrottle {
    min_interval: Duration,
    next_allowed: Mutex<Instant>,
}

impl TitanThrottle {
    fn new(min_interval: Duration) -> Self {
        Self {
            min_interval,
            next_allowed: Mutex::new(Instant::now()),
        }
    }

    async fn wait(&self) {
        let mut guard = self.next_allowed.lock().await;
        let now = Instant::now();
        if now < *guard {
            sleep(*guard - now).await;
        }
        *guard = Instant::now() + self.min_interval;
    }
}

fn calculate_profit(plan: &LegPairPlan) -> i128 {
    let buy_cost = plan.buy.quote.amount_in as i128;
    let sell_proceeds = plan.sell.quote.amount_out as i128;
    let fees = plan.buy.prioritization_fee_lamports.unwrap_or(0) as i128
        + plan.sell.prioritization_fee_lamports.unwrap_or(0) as i128;
    sell_proceeds - buy_cost - fees
}

fn to_ip_task_kind(descriptor: &LegDescriptor) -> IpTaskKind {
    let aggregator = match descriptor.kind {
        AggregatorKind::Ultra => MultiLegAggregatorKind::Ultra,
        AggregatorKind::Dflow => MultiLegAggregatorKind::Dflow,
        AggregatorKind::Titan => MultiLegAggregatorKind::Titan,
        AggregatorKind::Kamino => MultiLegAggregatorKind::Kamino,
    };
    let side = match descriptor.side {
        LegSide::Buy => MultiLegLegSide::Buy,
        LegSide::Sell => MultiLegLegSide::Sell,
    };

    IpTaskKind::MultiLegLeg { aggregator, side }
}
