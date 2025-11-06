use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Error, Result, anyhow};
use futures::future::try_join;
use futures::{StreamExt, stream::FuturesUnordered};
use parking_lot::Mutex;
use rayon::prelude::*;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::sleep;
use tracing::{debug, warn};

use super::orchestrator::{LegPairDescriptor, LegPairPlan, MultiLegOrchestrator};
use super::transaction::instructions::{InstructionExtractionError, extract_instructions};
use super::types::{
    AggregatorKind, LegBuildContext, LegDescriptor, LegPlan, LegSide, QuoteIntent, SignerRewrite,
};
use super::types::{AggregatorKind as MultiLegAggregatorKind, LegSide as MultiLegLegSide};
use crate::cache::AltCache;
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

    pub fn orchestrator(&self) -> &MultiLegOrchestrator {
        &self.orchestrator
    }

    async fn plan_pair_raw(&self, request: &PairPlanRequest) -> Result<Option<LegPairPlan>> {
        let buy_descriptor = self
            .orchestrator
            .descriptor(LegSide::Buy, request.buy_index)
            .cloned()
            .ok_or_else(|| anyhow!("买腿索引 {} 超出范围", request.buy_index))?;
        let sell_descriptor = self
            .orchestrator
            .descriptor(LegSide::Sell, request.sell_index)
            .cloned()
            .ok_or_else(|| anyhow!("卖腿索引 {} 超出范围", request.sell_index))?;

        let _buy_guard = self.concurrency.acquire(&buy_descriptor).await;
        let _sell_guard = self.concurrency.acquire(&sell_descriptor).await;

        let buy_quote_handle = self.acquire_leg_handle(&buy_descriptor).await?;
        let sell_quote_handle = self.acquire_leg_handle(&sell_descriptor).await?;

        let pair_quote = self
            .orchestrator
            .quote_pair(
                request.buy_index,
                request.sell_index,
                &request.buy_intent,
                &request.sell_intent,
                Some(&buy_quote_handle),
                Some(&sell_quote_handle),
            )
            .await;

        drop(buy_quote_handle);
        drop(sell_quote_handle);

        let pair_quote = pair_quote?;

        let estimated_profit = pair_quote.estimated_gross_profit();
        if estimated_profit <= 0 {
            debug!(
                target: "multi_leg::runtime",
                buy_index = request.buy_index,
                sell_index = request.sell_index,
                trade_size = request.trade_size(),
                buy_kind = %pair_quote.buy.descriptor().kind,
                sell_kind = %pair_quote.sell.descriptor().kind,
                profit = estimated_profit,
                "报价阶段收益不足，跳过 build_plan"
            );
            return Ok(None);
        }

        let buy_plan_handle = self.acquire_leg_handle(&buy_descriptor).await?;
        let sell_plan_handle = self.acquire_leg_handle(&sell_descriptor).await?;

        let plan = self
            .orchestrator
            .build_pair_plan(
                &pair_quote,
                &request.buy_context,
                &request.sell_context,
                Some(&buy_plan_handle),
                Some(&sell_plan_handle),
            )
            .await;

        drop(buy_plan_handle);
        drop(sell_plan_handle);

        plan.map(Some)
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

        let mut successes = Vec::new();
        let mut failures = Vec::new();

        let mut stream = FuturesUnordered::new();
        for request in requests {
            let runtime = self;
            stream.push(async move {
                let result = runtime.plan_pair_raw(&request).await;
                (request, result)
            });
        }

        while let Some((request, outcome)) = stream.next().await {
            match outcome {
                Ok(Some(mut plan)) => match self.populate_pair_plan(&mut plan).await {
                    Ok(()) => successes.push(PairPlanSuccess { request, plan }),
                    Err(error) => {
                        warn!(
                            target = "multi_leg::runtime",
                            buy_index = request.buy_index,
                            sell_index = request.sell_index,
                            trade_size = request.trade_size(),
                            error = %error,
                            "双腿计划填充 ALT 失败"
                        );
                        failures.push(PairPlanFailure {
                            buy_index: request.buy_index,
                            sell_index: request.sell_index,
                            trade_size: request.trade_size(),
                            error,
                        });
                    }
                },
                Ok(None) => {
                    // Already logged at quote stage; nothing else to do.
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

        let mut evaluations = evaluate_pair_plans(successes);
        evaluations.sort_by(|a, b| b.profit_lamports.cmp(&a.profit_lamports));
        PairPlanBatchResult {
            successes: evaluations,
            failures,
        }
    }

    async fn populate_leg_plan(&self, plan: &mut LegPlan) -> Result<()> {
        if plan.address_lookup_table_addresses.is_empty() {
            plan.resolved_lookup_tables.clear();
            self.rebuild_plan_instructions(plan)?;
            return Ok(());
        }

        let tables = self
            .alt_cache
            .fetch_many(&self.rpc, &plan.address_lookup_table_addresses)
            .await?;
        let table_map: HashMap<Pubkey, _> =
            tables.into_iter().map(|table| (table.key, table)).collect();

        plan.resolved_lookup_tables = plan
            .address_lookup_table_addresses
            .iter()
            .filter_map(|key| table_map.get(key).cloned())
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

struct PairPlanSuccess {
    request: PairPlanRequest,
    plan: LegPairPlan,
}

fn evaluate_pair_plans(successes: Vec<PairPlanSuccess>) -> Vec<PairPlanEvaluation> {
    if successes.is_empty() {
        return Vec::new();
    }

    let pool = crate::concurrency::rayon_pool();
    pool.install(|| {
        successes
            .into_par_iter()
            .map(|success| {
                let PairPlanSuccess { request, plan } = success;
                let profit_lamports = calculate_profit(&plan);
                PairPlanEvaluation {
                    descriptor: LegPairDescriptor {
                        buy: plan.buy.descriptor.clone(),
                        sell: plan.sell.descriptor.clone(),
                    },
                    trade_size: request.trade_size(),
                    tag: request.tag,
                    plan,
                    profit_lamports,
                }
            })
            .collect()
    })
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
        let delay = {
            let mut guard = self.next_allowed.lock();
            let now = Instant::now();
            let delay = if now < *guard {
                Some(*guard - now)
            } else {
                None
            };
            *guard = now + self.min_interval;
            delay
        };

        if let Some(duration) = delay {
            sleep(duration).await;
        }
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
        AggregatorKind::Jupiter => MultiLegAggregatorKind::Jupiter,
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
