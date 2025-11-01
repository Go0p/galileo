mod aggregator;
mod builder;
mod context;
mod error;
mod identity;
mod planner;
mod precheck;
mod profit;
mod quote;
mod quote_dispatcher;
mod scheduler;
mod swap_preparer;
mod types;
pub mod ultra;

pub use aggregator::{MultiLegInstructions, SwapInstructionsVariant};
pub use builder::{BuilderConfig, TransactionBuilder};
pub use context::{Action, QuoteBatchPlan, StrategyContext, StrategyDecision, StrategyResources};
pub use error::{EngineError, EngineResult};
pub use identity::EngineIdentity;
pub use planner::{DispatchPlan, DispatchStrategy, TxVariant, TxVariantPlanner, VariantId};
pub use precheck::AccountPrechecker;
pub use profit::{ProfitConfig, ProfitEvaluator, TipConfig};
pub use quote::{QuoteConfig, QuoteExecutor};
pub use quote_dispatcher::QuoteDispatcher;
pub use scheduler::Scheduler;
pub use swap_preparer::{ComputeUnitPriceMode, SwapPreparer};
pub use types::{ExecutionPlan, QuoteTask, StrategyTick, SwapOpportunity, TradeProfile};

use self::types::DoubleQuote;

use std::collections::{BTreeMap, HashMap, HashSet};
use std::mem;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::Value;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use tracing::{debug, error, info, trace, warn};

use spl_associated_token_account::get_associated_token_address;

use crate::api::dflow::DflowError;
use crate::api::jupiter::SwapInstructionsResponse;
use crate::dexes::clmm::RaydiumClmmAdapter;
use crate::dexes::dlmm::MeteoraDlmmAdapter;
use crate::dexes::framework::{SwapAccountAssembler, SwapAccountsContext, SwapFlow};
use crate::dexes::humidifi::HumidiFiAdapter;
use crate::dexes::obric_v2::ObricV2Adapter;
use crate::dexes::solfi_v2::SolFiV2Adapter;
use crate::dexes::tessera_v::TesseraVAdapter;
use crate::dexes::whirlpool::WhirlpoolAdapter;
use crate::dexes::zerofi::ZeroFiAdapter;
use crate::flashloan::FlashloanOutcome;
use crate::flashloan::marginfi::MarginfiFlashloanManager;
use crate::lander::{Deadline, LanderStack};
use crate::lighthouse::build_token_amount_guard;
use crate::monitoring::events;
use crate::multi_leg::orchestrator::{LegPairDescriptor, LegPairPlan};
use crate::multi_leg::runtime::{MultiLegRuntime, PairPlanBatchResult, PairPlanRequest};
use crate::multi_leg::types::{
    AggregatorKind as MultiLegAggregatorKind, LegBuildContext as MultiLegBuildContext,
    LegDescriptor as MultiLegDescriptor, LegPlan, LegSide as MultiLegSide,
    QuoteIntent as MultiLegQuoteIntent,
};
use crate::network::{IpAllocator, IpLeaseMode, IpTaskKind};
use crate::strategy::types::{BlindDex, BlindMarketMeta, BlindOrder, BlindStep, TradePair};
use crate::strategy::{Strategy, StrategyEvent};
use crate::txs::jupiter::route_v2::{RouteV2Accounts, RouteV2InstructionBuilder};
use crate::txs::jupiter::swaps::{
    HumidiFiSwap, MeteoraDlmmSwap, MeteoraDlmmSwapV2, ObricSwap, RaydiumClmmSwap,
    RaydiumClmmSwapV2, SolFiV2Swap, TesseraVSide, TesseraVSwap, WhirlpoolSwap, WhirlpoolSwapV2,
    ZeroFiSwap,
};
use crate::txs::jupiter::types::{JUPITER_V6_PROGRAM_ID, RoutePlanStepV2};

const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
pub const FALLBACK_CU_LIMIT: u32 = 230_000;
const COMPUTE_BUDGET_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ComputeBudget111111111111111111111111111111");
const BASE_TX_FEE_LAMPORTS: u64 = 5_000;

struct LegCombination {
    buy_index: usize,
    sell_index: usize,
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
        let buy_count = runtime.orchestrator().buy_legs().len();
        let sell_count = runtime.orchestrator().sell_legs().len();
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

    fn runtime(&self) -> &MultiLegRuntime {
        &self.runtime
    }

    fn combinations(&self) -> &[LegCombination] {
        &self.combinations
    }

    fn build_context(
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

struct MultiLegExecution {
    #[allow(dead_code)]
    descriptor: LegPairDescriptor,
    pair: TradePair,
    trade_size: u64,
    plan: LegPairPlan,
    gross_profit: u64,
    tip_lamports: u64,
    #[allow(dead_code)]
    tag: Option<String>,
}

impl MultiLegExecution {
    fn net_profit(&self) -> i128 {
        self.gross_profit as i128 - self.tip_lamports as i128
    }
}

pub(super) struct MintSchedule {
    amounts: Vec<u64>,
    process_delay: Duration,
    next_ready: Instant,
}

impl MintSchedule {
    fn from_profile(profile: TradeProfile) -> Self {
        Self {
            amounts: profile.amounts,
            process_delay: profile.process_delay,
            next_ready: Instant::now(),
        }
    }

    fn take_ready_batch(&mut self, now: Instant) -> Option<Vec<u64>> {
        if self.amounts.is_empty() || now < self.next_ready {
            return None;
        }
        self.next_ready = now + self.process_delay;
        Some(self.amounts.clone())
    }

    fn has_amounts(&self) -> bool {
        !self.amounts.is_empty()
    }

    fn is_ready(&self, now: Instant) -> bool {
        now >= self.next_ready
    }

    fn time_until_ready(&self, now: Instant) -> Duration {
        self.next_ready
            .checked_duration_since(now)
            .unwrap_or(Duration::ZERO)
    }
}

#[derive(Clone, Default)]
pub struct LighthouseSettings {
    pub enable: bool,
    pub profit_guard_mints: Vec<Pubkey>,
    pub memory_slots: Option<u8>,
    pub existing_memory_ids: Vec<u8>,
}

#[derive(Clone)]
pub struct EngineSettings {
    pub landing_timeout: Duration,
    pub quote: QuoteConfig,
    pub dry_run: bool,
    pub dispatch_strategy: DispatchStrategy,
    pub cu_multiplier: f64,
    compute_unit_price_mode: Option<ComputeUnitPriceMode>,
    pub quote_parallelism: Option<u16>,
    pub quote_batch_interval: Duration,
    pub lighthouse: LighthouseSettings,
}

impl EngineSettings {
    pub fn new(quote: QuoteConfig) -> Self {
        Self {
            landing_timeout: Duration::from_secs(2),
            quote,
            dry_run: false,
            dispatch_strategy: DispatchStrategy::default(),
            cu_multiplier: 1.0,
            compute_unit_price_mode: None,
            quote_parallelism: None,
            quote_batch_interval: Duration::ZERO,
            lighthouse: LighthouseSettings::default(),
        }
    }

    pub fn with_landing_timeout(mut self, timeout: Duration) -> Self {
        self.landing_timeout = timeout;
        self
    }

    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    pub fn with_dispatch_strategy(mut self, strategy: DispatchStrategy) -> Self {
        self.dispatch_strategy = strategy;
        self
    }

    pub fn with_cu_multiplier(mut self, multiplier: f64) -> Self {
        self.cu_multiplier = multiplier;
        self
    }

    pub fn with_compute_unit_price_mode(mut self, mode: Option<ComputeUnitPriceMode>) -> Self {
        self.compute_unit_price_mode = mode;
        self
    }

    pub fn with_lighthouse(mut self, lighthouse: LighthouseSettings) -> Self {
        self.lighthouse = lighthouse;
        self
    }

    pub fn with_quote_parallelism(mut self, parallelism: Option<u16>) -> Self {
        self.quote_parallelism = parallelism;
        self
    }

    pub fn with_quote_batch_interval(mut self, interval: Duration) -> Self {
        self.quote_batch_interval = interval;
        self
    }

    pub fn sample_compute_unit_price(&self) -> Option<u64> {
        self.compute_unit_price_mode
            .as_ref()
            .map(|mode| mode.sample())
            .filter(|price| *price > 0)
    }
}

struct LighthouseRuntime {
    enabled: bool,
    guard_mints: HashSet<Pubkey>,
    memory_slots: usize,
    available_ids: Vec<u8>,
    cursor: usize,
}

const MIN_LIGHTHOUSE_MEMORY_SLOTS: usize = 1;
const MAX_LIGHTHOUSE_MEMORY_SLOTS: usize = 128;

impl LighthouseRuntime {
    fn new(settings: &LighthouseSettings, ip_capacity_hint: usize) -> Self {
        let guard_mints: HashSet<Pubkey> = settings.profit_guard_mints.iter().copied().collect();
        let enabled = settings.enable && !guard_mints.is_empty();

        let mut available_ids: Vec<u8> = settings.existing_memory_ids.clone();
        available_ids.sort_unstable();
        available_ids.dedup();

        let derived_slots = if !available_ids.is_empty() {
            available_ids.len()
        } else {
            ip_capacity_hint
                .max(MIN_LIGHTHOUSE_MEMORY_SLOTS)
                .min(MAX_LIGHTHOUSE_MEMORY_SLOTS)
        };
        let configured_slots = settings
            .memory_slots
            .map(|value| usize::from(value.max(1)));
        let slot_count = configured_slots.unwrap_or(derived_slots);
        let memory_slots = slot_count
            .max(available_ids.len())
            .max(MIN_LIGHTHOUSE_MEMORY_SLOTS)
            .min(MAX_LIGHTHOUSE_MEMORY_SLOTS);

        Self {
            enabled,
            guard_mints,
            memory_slots,
            available_ids,
            cursor: 0,
        }
    }

    fn should_guard(&self, mint: &Pubkey) -> bool {
        self.enabled && self.guard_mints.contains(mint)
    }

    fn next_memory_id(&mut self) -> u8 {
        if !self.enabled {
            return 0;
        }
        if self.available_ids.is_empty() {
            let id = 0u8;
            if self.memory_slots > 1 {
                self.available_ids.reserve(self.memory_slots.saturating_sub(1));
            }
            self.available_ids.push(id);
            return id;
        }

        let idx = if self.available_ids.len() == 1 {
            0
        } else {
            let current = self.cursor % self.available_ids.len();
            self.cursor = (self.cursor + 1) % self.available_ids.len();
            current
        };
        self.available_ids[idx]
    }

}

pub struct StrategyEngine<S>
where
    S: Strategy,
{
    strategy: S,
    landers: Arc<LanderStack>,
    identity: EngineIdentity,
    ip_allocator: Arc<IpAllocator>,
    quote_dispatcher: QuoteDispatcher,
    quote_executor: QuoteExecutor,
    profit_evaluator: ProfitEvaluator,
    swap_preparer: SwapPreparer,
    tx_builder: TransactionBuilder,
    scheduler: Scheduler,
    flashloan: Option<MarginfiFlashloanManager>,
    settings: EngineSettings,
    trade_pairs: Vec<TradePair>,
    trade_profiles: BTreeMap<Pubkey, MintSchedule>,
    variant_planner: TxVariantPlanner,
    next_batch_id: u64,
    multi_leg: Option<MultiLegEngineContext>,
    lighthouse: LighthouseRuntime,
}

impl<S> StrategyEngine<S>
where
    S: Strategy<Event = StrategyEvent>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        strategy: S,
        landers: Arc<LanderStack>,
        identity: EngineIdentity,
        ip_allocator: Arc<IpAllocator>,
        quote_executor: QuoteExecutor,
        profit_evaluator: ProfitEvaluator,
        swap_preparer: SwapPreparer,
        tx_builder: TransactionBuilder,
        scheduler: Scheduler,
        flashloan: Option<MarginfiFlashloanManager>,
        settings: EngineSettings,
        trade_pairs: Vec<TradePair>,
        trade_profiles: BTreeMap<Pubkey, TradeProfile>,
        multi_leg: Option<MultiLegEngineContext>,
    ) -> Self {
        let quote_dispatcher = QuoteDispatcher::new(
            Arc::clone(&ip_allocator),
            settings.quote_parallelism,
            settings.quote_batch_interval,
        );
        let allocator_summary = ip_allocator.summary();
        let per_ip_capacity = allocator_summary
            .per_ip_inflight_limit
            .unwrap_or(1)
            .max(1);
        let ip_capacity_hint = allocator_summary
            .total_slots
            .max(1)
            .saturating_mul(per_ip_capacity);
        let trade_profiles = trade_profiles
            .into_iter()
            .map(|(mint, profile)| (mint, MintSchedule::from_profile(profile)))
            .collect();
        let lighthouse_runtime = LighthouseRuntime::new(&settings.lighthouse, ip_capacity_hint);

        Self {
            strategy,
            landers,
            identity,
            ip_allocator,
            quote_dispatcher,
            quote_executor,
            profit_evaluator,
            swap_preparer,
            tx_builder,
            scheduler,
            flashloan,
            settings,
            trade_pairs,
            trade_profiles,
            variant_planner: TxVariantPlanner::new(),
            next_batch_id: 1,
            multi_leg,
            lighthouse: lighthouse_runtime,
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn run(mut self) -> EngineResult<()> {
        if self.landers.is_empty() {
            return Err(EngineError::InvalidConfig("未配置可用的落地器".into()));
        }

        info!(
            target: "engine",
            strategy = self.strategy.name(),
            "策略引擎启动"
        );

        debug!(
            target: "engine::network",
            total_slots = self.ip_allocator.total_slots(),
            per_ip_limit = ?self.ip_allocator.per_ip_inflight_limit(),
            source = ?self.ip_allocator.source(),
            "IP 资源池就绪"
        );

        self.run_jupiter().await
    }

    async fn run_jupiter(&mut self) -> EngineResult<()> {
        loop {
            let next_wait = self.process_strategy_tick().await?;
            self.scheduler.wait(next_wait).await;
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn handle_action(&mut self, action: Action) -> EngineResult<()> {
        match action {
            Action::Idle => Ok(()),
            Action::Quote(batches) => self.run_quote_batches(batches).await,
            Action::DispatchBlind(batch) => self.process_blind_batch(batch).await,
        }
    }

    async fn process_blind_batch(
        &mut self,
        batch: Vec<crate::strategy::types::BlindOrder>,
    ) -> EngineResult<()> {
        if batch.is_empty() {
            return Ok(());
        }

        let mut last_error: Option<EngineError> = None;
        for order in batch {
            if let Err(err) = self.execute_blind_order(&order).await {
                warn!(
                    target: "engine::blind",
                    error = %err,
                    "执行盲发交易失败"
                );
                last_error = Some(err);
            }
        }

        if let Some(err) = last_error {
            Err(err)
        } else {
            Ok(())
        }
    }

    async fn execute_blind_order(&mut self, order: &BlindOrder) -> EngineResult<()> {
        if order.steps.is_empty() {
            return Err(EngineError::InvalidConfig("盲发订单缺少步骤".to_string()));
        }

        let first_step = order
            .steps
            .first()
            .ok_or_else(|| EngineError::InvalidConfig("盲发订单缺少首个步骤".to_string()))?;
        let last_step = order
            .steps
            .last()
            .ok_or_else(|| EngineError::InvalidConfig("盲发订单缺少末尾步骤".to_string()))?;

        ensure_route_pair(first_step, last_step)?;

        let mut ata_resolver = AtaResolver::new(self.identity.pubkey);

        let (route_plan, remaining_accounts) = self.build_route_plan(order, &mut ata_resolver)?;

        let source_mint = first_step.input.mint;
        let source_program = first_step.input.token_program;
        let destination_mint = last_step.output.mint;
        let destination_program = last_step.output.token_program;
        let source_account = ata_resolver.get(source_mint, source_program);
        let destination_account = ata_resolver.get(destination_mint, destination_program);

        let mut accounts = RouteV2Accounts::with_defaults(
            self.identity.pubkey,
            source_account,
            destination_account,
            source_mint,
            destination_mint,
            source_program,
            destination_program,
        );
        accounts.destination_token_account = Some(JUPITER_V6_PROGRAM_ID);
        accounts.remaining_accounts = remaining_accounts;

        let quoted_out_amount = order.amount_in.saturating_add(order.min_profit.max(1));

        let instruction = RouteV2InstructionBuilder {
            accounts,
            route_plan,
            in_amount: order.amount_in,
            quoted_out_amount,
            slippage_bps: 0,
            platform_fee_bps: 0,
            positive_slippage_bps: 0,
        }
        .build()
        .map_err(|err| EngineError::Transaction(err.into()))?;

        let compute_unit_limit = self.estimate_cu_limit(order);
        let mut compute_budget_instructions =
            vec![compute_unit_limit_instruction(compute_unit_limit)];
        let sampled_price = self
            .settings
            .sample_compute_unit_price()
            .or_else(|| self.swap_preparer.sample_compute_unit_price());
        if let Some(price) = sampled_price {
            compute_budget_instructions.push(compute_unit_price_instruction(price));
        }

        let lookup_table_accounts = order.lookup_tables.clone();
        let lookup_table_addresses: Vec<Pubkey> = lookup_table_accounts
            .iter()
            .map(|table| table.key)
            .collect();

        let response = SwapInstructionsResponse {
            raw: Value::Null,
            token_ledger_instruction: None,
            compute_budget_instructions,
            setup_instructions: Vec::new(),
            swap_instruction: instruction,
            cleanup_instruction: None,
            other_instructions: Vec::new(),
            address_lookup_table_addresses: lookup_table_addresses,
            resolved_lookup_tables: lookup_table_accounts,
            prioritization_fee_lamports: 0,
            compute_unit_limit,
            prioritization_type: None,
            dynamic_slippage_report: None,
            simulation_error: None,
        };
        let mut response_variant = SwapInstructionsVariant::Jupiter(response);

        let pair = TradePair::from_pubkeys(source_mint, destination_mint);
        let flashloan_opportunity = SwapOpportunity {
            pair,
            amount_in: order.amount_in,
            profit_lamports: 0,
            tip_lamports: 0,
            merged_quote: None,
            ultra_legs: None,
        };

        let FlashloanOutcome {
            instructions: mut final_instructions,
            metadata: flashloan_meta,
        } = match &self.flashloan {
            Some(manager) => manager
                .assemble(&self.identity, &flashloan_opportunity, &response_variant)
                .await
                .map_err(EngineError::from)?,
            None => FlashloanOutcome {
                instructions: response_variant.flatten_instructions(),
                metadata: None,
            },
        };

        if flashloan_meta.is_some() {
            if let Some(manager) = &self.flashloan {
                let overhead = manager.compute_unit_overhead();
                if overhead > 0 {
                    let new_limit = compute_unit_limit.saturating_add(overhead);
                    if !override_compute_unit_limit(&mut final_instructions, new_limit) {
                        final_instructions.insert(0, compute_unit_limit_instruction(new_limit));
                    }
                    if let SwapInstructionsVariant::Jupiter(resp) = &mut response_variant {
                        if let Some(ix) = resp.compute_budget_instructions.iter_mut().find(|ix| {
                            ix.program_id == COMPUTE_BUDGET_PROGRAM_ID
                                && ix.data.first() == Some(&2)
                        }) {
                            *ix = compute_unit_limit_instruction(new_limit);
                        } else {
                            resp.compute_budget_instructions
                                .insert(0, compute_unit_limit_instruction(new_limit));
                        }
                        resp.compute_unit_limit = new_limit;
                    }
                }
            }
        }

        let strategy_name = self.strategy.name();

        if let Some(meta) = &flashloan_meta {
            events::flashloan_applied(
                strategy_name,
                meta.protocol.as_str(),
                &meta.mint,
                meta.borrow_amount,
                meta.inner_instruction_count,
            );
        }

        let guard_required = BASE_TX_FEE_LAMPORTS;
        self.apply_profit_guard(&source_mint, guard_required, &mut final_instructions)?;

        let prepared = self
            .tx_builder
            .build_with_sequence(&self.identity, &response_variant, final_instructions, 0)
            .await?;

        let dispatch_strategy = self.settings.dispatch_strategy;
        let variant_layout = self.landers.variant_layout(dispatch_strategy);
        let plan = Arc::new(self.variant_planner.plan(
            dispatch_strategy,
            &prepared,
            &variant_layout,
        ));

        let deadline = Deadline::from_instant(Instant::now() + self.settings.landing_timeout);

        match self
            .landers
            .submit_plan(&plan, deadline, strategy_name)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                let tx_signature = plan
                    .primary_variant()
                    .and_then(|variant| variant.signature());
                warn!(
                    target: "engine::blind",
                    signature = tx_signature.as_deref().unwrap_or(""),
                    error = %err,
                    "盲发落地失败"
                );
                Err(EngineError::Landing(err.to_string()))
            }
        }
    }

    fn estimate_cu_limit(&self, order: &BlindOrder) -> u32 {
        let subtotal: f64 = order
            .steps
            .iter()
            .map(|step| step.dex.default_cu_budget() as f64)
            .sum();
        let base = if subtotal > 0.0 {
            subtotal
        } else {
            FALLBACK_CU_LIMIT as f64
        };
        let multiplier = if self.settings.cu_multiplier <= 0.0 {
            1.0
        } else {
            self.settings.cu_multiplier
        };
        (base * multiplier).round() as u32
    }

    fn jito_tip_budget(&self, _base_tip: u64) -> u64 {
        if !self.landers.has_jito() {
            return 0;
        }
        // 当前仅支持固定 tip 策略，其余类型（range/stream）暂不计入守护
        self.landers.fixed_jito_tip().unwrap_or(0)
    }

    fn build_route_plan(
        &self,
        order: &BlindOrder,
        resolver: &mut AtaResolver,
    ) -> EngineResult<(Vec<RoutePlanStepV2>, Vec<AccountMeta>)> {
        if order.steps.is_empty() {
            return Err(EngineError::InvalidConfig("盲发订单缺少步骤".to_string()));
        }

        let mut route_plan = Vec::with_capacity(order.steps.len());
        let mut remaining_accounts = Vec::with_capacity(order.steps.len() * 12);
        let mut slot_assets = Vec::with_capacity(order.steps.len() + 1);
        slot_assets.push(
            order
                .steps
                .first()
                .map(|step| step.input.clone())
                .ok_or_else(|| EngineError::InvalidConfig("盲发订单缺少首个步骤".to_string()))?,
        );

        for step in &order.steps {
            let input_slot = slot_assets
                .iter()
                .position(|asset| asset == &step.input)
                .ok_or_else(|| {
                    EngineError::InvalidConfig(format!(
                        "盲发路线输入资产 {} 缺少生产者",
                        step.input.mint
                    ))
                })?;

            let output_slot = match slot_assets.iter().position(|asset| asset == &step.output) {
                Some(idx) => idx,
                None => {
                    slot_assets.push(step.output.clone());
                    slot_assets.len() - 1
                }
            };

            let user_base = resolver.get(step.base.mint, step.base.token_program);
            let user_quote = resolver.get(step.quote.mint, step.quote.token_program);

            let encoded_swap = match (&step.meta, step.dex) {
                (BlindMarketMeta::SolFiV2(_), BlindDex::SolFiV2) => {
                    let swap = SolFiV2Swap {
                        is_quote_to_base: matches!(step.flow, SwapFlow::QuoteToBase),
                    };
                    swap.encode().map_err(|err| {
                        EngineError::InvalidConfig(format!("构造 SolFiV2 swap 失败: {err}"))
                    })?
                }
                (BlindMarketMeta::ZeroFi(_), BlindDex::ZeroFi) => {
                    ZeroFiSwap::encode().map_err(|err| {
                        EngineError::InvalidConfig(format!("构造 ZeroFi swap 失败: {err}"))
                    })?
                }
                (BlindMarketMeta::HumidiFi(meta), BlindDex::HumidiFi) => {
                    let swap_id = meta.next_swap_id().map_err(|err| {
                        EngineError::InvalidConfig(format!("生成 HumidiFi swap_id 失败: {err}"))
                    })?;
                    let swap = HumidiFiSwap {
                        swap_id,
                        is_base_to_quote: matches!(step.flow, SwapFlow::BaseToQuote),
                    };
                    swap.encode().map_err(|err| {
                        EngineError::InvalidConfig(format!("构造 HumidiFi swap 失败: {err}"))
                    })?
                }
                (BlindMarketMeta::TesseraV(_), BlindDex::TesseraV) => {
                    let swap = TesseraVSwap {
                        side: match step.flow {
                            SwapFlow::QuoteToBase => TesseraVSide::Bid,
                            SwapFlow::BaseToQuote => TesseraVSide::Ask,
                        },
                    };
                    swap.encode().map_err(|err| {
                        EngineError::InvalidConfig(format!("构造 TesseraV swap 失败: {err}"))
                    })?
                }
                (BlindMarketMeta::ObricV2(_), BlindDex::ObricV2) => {
                    let swap = ObricSwap {
                        x_to_y: matches!(step.flow, SwapFlow::BaseToQuote),
                    };
                    swap.encode().map_err(|err| {
                        EngineError::InvalidConfig(format!("构造 ObricV2 swap 失败: {err}"))
                    })?
                }
                (BlindMarketMeta::RaydiumClmm(meta), BlindDex::RaydiumClmm) => {
                    if meta.uses_token_2022() {
                        RaydiumClmmSwapV2::encode().map_err(|err| {
                            EngineError::InvalidConfig(format!(
                                "构造 RaydiumClmmV2 swap 失败: {err}"
                            ))
                        })?
                    } else {
                        RaydiumClmmSwap::encode().map_err(|err| {
                            EngineError::InvalidConfig(format!("构造 RaydiumClmm swap 失败: {err}"))
                        })?
                    }
                }
                (BlindMarketMeta::MeteoraDlmm(meta), BlindDex::MeteoraDlmm) => {
                    if meta.uses_token_2022() {
                        MeteoraDlmmSwapV2::encode_default().map_err(|err| {
                            EngineError::InvalidConfig(format!(
                                "构造 MeteoraDlmmSwapV2 失败: {err}"
                            ))
                        })?
                    } else {
                        MeteoraDlmmSwap::encode().map_err(|err| {
                            EngineError::InvalidConfig(format!("构造 MeteoraDlmm swap 失败: {err}"))
                        })?
                    }
                }
                (BlindMarketMeta::Whirlpool(meta), BlindDex::Whirlpool) => {
                    if meta.uses_token_2022() {
                        let swap = WhirlpoolSwapV2 {
                            a_to_b: matches!(step.flow, SwapFlow::BaseToQuote),
                            remaining_accounts: None,
                        };
                        swap.encode().map_err(|err| {
                            EngineError::InvalidConfig(format!("构造 WhirlpoolSwapV2 失败: {err}"))
                        })?
                    } else {
                        let swap = WhirlpoolSwap {
                            a_to_b: matches!(step.flow, SwapFlow::BaseToQuote),
                        };
                        swap.encode().map_err(|err| {
                            EngineError::InvalidConfig(format!("构造 Whirlpool swap 失败: {err}"))
                        })?
                    }
                }
                _ => {
                    return Err(EngineError::InvalidConfig(
                        "纯盲发暂未支持该 DEX".to_string(),
                    ));
                }
            };

            route_plan.push(RoutePlanStepV2 {
                swap: encoded_swap,
                bps: 10_000,
                input_index: u8::try_from(input_slot)
                    .map_err(|_| EngineError::InvalidConfig("盲发路线槽位数量超过限制".into()))?,
                output_index: u8::try_from(output_slot)
                    .map_err(|_| EngineError::InvalidConfig("盲发路线槽位数量超过限制".into()))?,
            });

            let ctx = SwapAccountsContext {
                market: step.market,
                payer: self.identity.pubkey,
                user_base,
                user_quote,
                flow: step.flow,
            };

            match &step.meta {
                BlindMarketMeta::SolFiV2(meta) => {
                    SolFiV2Adapter::shared().assemble_remaining_accounts(
                        meta.as_ref(),
                        ctx,
                        &mut remaining_accounts,
                    );
                }
                BlindMarketMeta::ZeroFi(meta) => {
                    ZeroFiAdapter::shared().assemble_remaining_accounts(
                        meta.as_ref(),
                        ctx,
                        &mut remaining_accounts,
                    );
                }
                BlindMarketMeta::HumidiFi(meta) => {
                    HumidiFiAdapter::shared().assemble_remaining_accounts(
                        meta.as_ref(),
                        ctx,
                        &mut remaining_accounts,
                    );
                }
                BlindMarketMeta::TesseraV(meta) => {
                    TesseraVAdapter::shared().assemble_remaining_accounts(
                        meta.as_ref(),
                        ctx,
                        &mut remaining_accounts,
                    );
                }
                BlindMarketMeta::ObricV2(meta) => {
                    ObricV2Adapter::shared().assemble_remaining_accounts(
                        meta.as_ref(),
                        ctx,
                        &mut remaining_accounts,
                    );
                }
                BlindMarketMeta::RaydiumClmm(meta) => {
                    RaydiumClmmAdapter::shared().assemble_remaining_accounts(
                        meta.as_ref(),
                        ctx,
                        &mut remaining_accounts,
                    );
                }
                BlindMarketMeta::MeteoraDlmm(meta) => {
                    MeteoraDlmmAdapter::shared().assemble_remaining_accounts(
                        meta.as_ref(),
                        ctx,
                        &mut remaining_accounts,
                    );
                }
                BlindMarketMeta::Whirlpool(meta) => {
                    WhirlpoolAdapter::shared().assemble_remaining_accounts(
                        meta.as_ref(),
                        ctx,
                        &mut remaining_accounts,
                    );
                }
            }
        }

        Ok((route_plan, remaining_accounts))
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn process_quote_batch_legacy(&mut self, batch: QuoteBatchPlan) -> EngineResult<()> {
        let QuoteBatchPlan {
            batch_id,
            pair,
            amount,
        } = batch;

        trace!(
            target: "engine::quote",
            batch_id,
            amount,
            "处理报价批次（legacy）"
        );

        let task = QuoteTask::new(pair, amount);
        self.process_task(task, Some(batch_id)).await
    }

    async fn process_quote_outcome(
        &mut self,
        batch: QuoteBatchPlan,
        quote: Option<DoubleQuote>,
        forward_ip: Option<IpAddr>,
        reverse_ip: Option<IpAddr>,
        forward_duration: Option<Duration>,
        reverse_duration: Option<Duration>,
    ) -> EngineResult<()> {
        let QuoteBatchPlan {
            batch_id,
            pair,
            amount,
        } = batch;

        trace!(
            target: "engine::quote",
            batch_id,
            amount,
            has_quote = quote.is_some(),
            forward_ip = ?forward_ip,
            reverse_ip = ?reverse_ip,
            "处理并发报价结果",
        );

        let Some(double_quote) = quote else {
            return Ok(());
        };

        let task = QuoteTask::new(pair.clone(), amount);
        let forward_out = double_quote.forward.out_amount();
        let reverse_out = double_quote.reverse.out_amount();
        let aggregator = format!("{:?}", double_quote.forward.kind());
        events::quote_round_trip(
            self.strategy.name(),
            &task,
            aggregator.as_str(),
            forward_out,
            reverse_out,
            Some(batch_id),
            forward_ip,
        );

        let Some(opportunity) = self
            .profit_evaluator
            .evaluate(task.amount, &double_quote, &pair)
        else {
            return Ok(());
        };

        self.log_opportunity_discovery(
            &task,
            &opportunity,
            forward_duration,
            reverse_duration,
            forward_ip,
            reverse_ip,
        );
        events::profit_detected(self.strategy.name(), &opportunity);

        let plan = ExecutionPlan::with_deadline(opportunity, self.settings.landing_timeout);
        self.execute_plan(plan).await
    }

    async fn run_quote_batches(&mut self, batches: Vec<QuoteBatchPlan>) -> EngineResult<()> {
        if batches.is_empty() {
            return Ok(());
        }

        let planned_batches = self.quote_dispatcher.plan(batches);

        if self.multi_leg.is_some() {
            for batch in planned_batches {
                self.process_quote_batch_legacy(batch).await?;
            }
            return Ok(());
        }

        let strategy_label = Arc::new(self.strategy.name().to_string());
        let outcomes = self
            .quote_dispatcher
            .dispatch(
                planned_batches,
                self.quote_executor.clone(),
                self.settings.quote.clone(),
                Arc::clone(&strategy_label),
            )
            .await?;

        for outcome in outcomes {
            self.process_quote_outcome(
                outcome.batch,
                outcome.quote,
                outcome.forward_ip,
                outcome.reverse_ip,
                outcome.forward_duration,
                outcome.reverse_duration,
            )
            .await?;
        }

        Ok(())
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn process_task(&mut self, task: QuoteTask, batch_id: Option<u64>) -> EngineResult<()> {
        if let Some(mut context) = self.multi_leg.take() {
            let result = self.process_multi_leg_task(&mut context, task).await;
            self.multi_leg = Some(context);
            return result;
        }

        let (forward_handle_raw, forward_ip_addr) = self
            .ip_allocator
            .acquire_handle_excluding(IpTaskKind::QuoteBuy, IpLeaseMode::Ephemeral, None)
            .await
            .map_err(EngineError::NetworkResource)?;
        let forward_ip = Some(forward_ip_addr);
        let mut forward_handle = Some(forward_handle_raw);
        let strategy_name = self.strategy.name();
        events::quote_start(strategy_name, &task, batch_id, forward_ip);
        let quote_started = Instant::now();
        let forward_start = Instant::now();
        let forward_result = self
            .quote_executor
            .quote_once(
                &task.pair,
                task.amount,
                &self.settings.quote,
                forward_handle
                    .as_ref()
                    .expect("forward handle available for first quote"),
            )
            .await;
        let forward_duration = forward_start.elapsed();

        let forward_quote = match forward_result {
            Err(err) => {
                events::quote_end(
                    strategy_name,
                    &task,
                    false,
                    quote_started.elapsed(),
                    batch_id,
                    forward_ip,
                );
                if let Some(handle) = forward_handle.take() {
                    if let Some(outcome) = quote_dispatcher::classify_ip_outcome(&err) {
                        handle.mark_outcome(outcome);
                    }
                }
                return Err(err);
            }
            Ok(None) => {
                events::quote_end(
                    strategy_name,
                    &task,
                    false,
                    quote_started.elapsed(),
                    batch_id,
                    forward_ip,
                );
                let _ = forward_handle.take();
                return Ok(());
            }
            Ok(Some(value)) => value,
        };

        let Some(second_amount) = crate::engine::quote::second_leg_amount(&task, &forward_quote)
        else {
            events::quote_end(
                strategy_name,
                &task,
                false,
                quote_started.elapsed(),
                batch_id,
                forward_ip,
            );
            let _ = forward_handle.take();
            return Ok(());
        };

        let _ = forward_handle.take();

        let (reverse_handle_raw, reverse_ip_addr) = match self
            .ip_allocator
            .acquire_handle_excluding(IpTaskKind::QuoteSell, IpLeaseMode::Ephemeral, forward_ip)
            .await
        {
            Ok(value) => value,
            Err(err) => {
                events::quote_end(
                    strategy_name,
                    &task,
                    false,
                    quote_started.elapsed(),
                    batch_id,
                    forward_ip,
                );
                return Err(EngineError::NetworkResource(err));
            }
        };
        let mut reverse_handle = Some(reverse_handle_raw);
        let reverse_ip = Some(reverse_ip_addr);

        let reverse_pair = task.pair.reversed();
        let reverse_start = Instant::now();
        let reverse_result = self
            .quote_executor
            .quote_once(
                &reverse_pair,
                second_amount,
                &self.settings.quote,
                reverse_handle
                    .as_ref()
                    .expect("reverse handle available for second quote"),
            )
            .await;
        let reverse_duration = Some(reverse_start.elapsed());

        let reverse_quote = match reverse_result {
            Err(err) => {
                events::quote_end(
                    strategy_name,
                    &task,
                    false,
                    quote_started.elapsed(),
                    batch_id,
                    forward_ip,
                );
                if let Some(handle) = reverse_handle.take() {
                    if let Some(outcome) = quote_dispatcher::classify_ip_outcome(&err) {
                        handle.mark_outcome(outcome);
                    }
                }
                return Err(err);
            }
            Ok(None) => {
                events::quote_end(
                    strategy_name,
                    &task,
                    false,
                    quote_started.elapsed(),
                    batch_id,
                    forward_ip,
                );
                let _ = reverse_handle.take();
                return Ok(());
            }
            Ok(Some(value)) => value,
        };

        let _ = reverse_handle.take();

        if !crate::engine::quote::aggregator_kinds_match(&task, &forward_quote, &reverse_quote) {
            events::quote_end(
                strategy_name,
                &task,
                false,
                quote_started.elapsed(),
                batch_id,
                forward_ip,
            );
            return Ok(());
        }

        let double_quote = DoubleQuote {
            forward: forward_quote,
            reverse: reverse_quote,
            forward_latency: Some(forward_duration),
            reverse_latency: reverse_duration,
        };

        events::quote_end(
            strategy_name,
            &task,
            true,
            quote_started.elapsed(),
            batch_id,
            forward_ip,
        );

        let forward_out = double_quote.forward.out_amount();
        let reverse_out = double_quote.reverse.out_amount();
        let aggregator = format!("{:?}", double_quote.forward.kind());
        events::quote_round_trip(
            strategy_name,
            &task,
            aggregator.as_str(),
            forward_out,
            reverse_out,
            batch_id,
            forward_ip,
        );

        let opportunity =
            match self
                .profit_evaluator
                .evaluate(task.amount, &double_quote, &task.pair)
            {
                Some(value) => value,
                None => return Ok(()),
            };

        self.log_opportunity_discovery(
            &task,
            &opportunity,
            Some(forward_duration),
            reverse_duration,
            forward_ip,
            reverse_ip,
        );
        events::profit_detected(strategy_name, &opportunity);

        let plan = ExecutionPlan::with_deadline(opportunity, self.settings.landing_timeout);
        self.execute_plan(plan).await
    }

    fn log_opportunity_discovery(
        &self,
        task: &QuoteTask,
        opportunity: &SwapOpportunity,
        forward_duration: Option<Duration>,
        reverse_duration: Option<Duration>,
        forward_ip: Option<IpAddr>,
        reverse_ip: Option<IpAddr>,
    ) {
        let forward_ms = forward_duration
            .map(|d| format!("{:.3}", d.as_secs_f64() * 1_000.0))
            .unwrap_or_else(|| "-".to_string());
        let reverse_ms = reverse_duration
            .map(|d| format!("{:.3}", d.as_secs_f64() * 1_000.0))
            .unwrap_or_else(|| "-".to_string());
        let ip_summary = match (forward_ip, reverse_ip) {
            (Some(f), Some(r)) if f == r => f.to_string(),
            (Some(f), Some(r)) => format!("{},{}", f, r),
            (Some(f), None) => f.to_string(),
            (None, Some(r)) => r.to_string(),
            _ => "-".to_string(),
        };

        debug!(
            target: "engine::opportunity",
            "本次机会 base_mint={} amount_in={} forward_ms={} reverse_ms={} profit={} net_profit={} ip={}",
            task.pair.input_mint,
            opportunity.amount_in,
            forward_ms,
            reverse_ms,
            opportunity.profit_lamports,
            opportunity.net_profit(),
            ip_summary,
        );
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn process_multi_leg_task(
        &mut self,
        ctx: &mut MultiLegEngineContext,
        task: QuoteTask,
    ) -> EngineResult<()> {
        if task.amount == 0 {
            return Ok(());
        }

        let combinations = ctx.combinations();
        if combinations.is_empty() {
            debug!(
                target: "engine::multi_leg",
                input_mint = %task.pair.input_mint,
                output_mint = %task.pair.output_mint,
                "无可用的腿组合，跳过本次任务"
            );
            return Ok(());
        }

        let compute_unit_price = self.settings.sample_compute_unit_price();
        let buy_intent_template = MultiLegQuoteIntent::new(
            task.pair.input_pubkey,
            task.pair.output_pubkey,
            task.amount,
            0,
        );
        let sell_intent_template = MultiLegQuoteIntent::new(
            task.pair.output_pubkey,
            task.pair.input_pubkey,
            task.amount,
            0,
        );
        let tag_value = format!(
            "{}->{}/{}",
            task.pair.input_mint, task.pair.output_mint, task.amount
        );

        let runtime = ctx.runtime();
        let mut requests = Vec::with_capacity(combinations.len());
        for combo in combinations {
            let buy_descriptor = match runtime
                .orchestrator()
                .descriptor(MultiLegSide::Buy, combo.buy_index)
                .cloned()
            {
                Some(descriptor) => descriptor,
                None => continue,
            };
            let sell_descriptor = match runtime
                .orchestrator()
                .descriptor(MultiLegSide::Sell, combo.sell_index)
                .cloned()
            {
                Some(descriptor) => descriptor,
                None => continue,
            };

            let buy_context =
                ctx.build_context(&buy_descriptor, self.identity.pubkey, compute_unit_price);
            let sell_context =
                ctx.build_context(&sell_descriptor, self.identity.pubkey, compute_unit_price);

            requests.push(PairPlanRequest {
                buy_index: combo.buy_index,
                sell_index: combo.sell_index,
                buy_intent: buy_intent_template.clone(),
                sell_intent: sell_intent_template.clone(),
                buy_context,
                sell_context,
                tag: Some(tag_value.clone()),
            });
        }

        if requests.is_empty() {
            return Ok(());
        }

        let batch = ctx.runtime().plan_pair_batch_with_profit(requests).await;

        let PairPlanBatchResult {
            successes,
            failures,
        } = batch;

        for failure in failures {
            warn!(
                target: "engine::multi_leg",
                input_mint = %task.pair.input_mint,
                output_mint = %task.pair.output_mint,
                amount = task.amount,
                buy_index = failure.buy_index,
                sell_index = failure.sell_index,
                error = %failure.error,
                "多腿腿组合规划失败"
            );
        }

        let mut candidates: Vec<MultiLegExecution> = Vec::new();

        for evaluation in successes.into_iter() {
            if evaluation.profit_lamports <= 0 {
                debug!(
                    target: "engine::multi_leg",
                    input_mint = %task.pair.input_mint,
                    output_mint = %task.pair.output_mint,
                    amount = task.amount,
                    profit = evaluation.profit_lamports,
                    "多腿收益非正值，丢弃"
                );
                continue;
            }

            let aggregator_label = format!(
                "multi_leg:{}->{}",
                evaluation.descriptor.buy.kind, evaluation.descriptor.sell.kind
            );
            let forward_quote = evaluation.plan.buy.quote.clone();
            let reverse_quote = evaluation.plan.sell.quote.clone();
            let estimated_profit = evaluation.profit_lamports.min(i128::from(u64::MAX)) as u64;
            let threshold = self.profit_evaluator.min_threshold();
            let Some(profit) = self
                .profit_evaluator
                .evaluate_multi_leg(evaluation.profit_lamports)
            else {
                debug!(
                    target: "engine::multi_leg",
                    input_mint = %task.pair.input_mint,
                    output_mint = %task.pair.output_mint,
                    amount = task.amount,
                    profit = evaluation.profit_lamports,
                    "多腿收益低于阈值，丢弃"
                );
                events::profit_shortfall(
                    task.pair.input_mint.as_str(),
                    aggregator_label.as_str(),
                    forward_quote.amount_in,
                    forward_quote.amount_out,
                    None,
                    reverse_quote.amount_in,
                    reverse_quote.amount_out,
                    None,
                    estimated_profit,
                    threshold,
                );
                continue;
            };

            let candidate = MultiLegExecution {
                descriptor: evaluation.descriptor,
                pair: task.pair.clone(),
                trade_size: evaluation.trade_size,
                plan: evaluation.plan,
                gross_profit: profit.gross_profit_lamports,
                tip_lamports: profit.tip_lamports,
                tag: evaluation.tag,
            };

            if candidate.net_profit() <= 0 {
                debug!(
                    target: "engine::multi_leg",
                    input_mint = %candidate.pair.input_mint,
                    output_mint = %candidate.pair.output_mint,
                    amount = candidate.trade_size,
                    net_profit = candidate.net_profit(),
                    "多腿净收益不满足条件，丢弃"
                );
                continue;
            }

            events::profit_opportunity(
                candidate.pair.input_mint.as_str(),
                aggregator_label.as_str(),
                forward_quote.amount_in,
                forward_quote.amount_out,
                None,
                reverse_quote.amount_in,
                reverse_quote.amount_out,
                None,
                profit.gross_profit_lamports,
                candidate.net_profit(),
                threshold,
            );

            candidates.push(candidate);
        }

        if candidates.is_empty() {
            return Ok(());
        }

        let mut any_executed = false;
        let mut last_error: Option<EngineError> = None;

        for mut candidate in candidates {
            if let Err(err) = runtime.populate_pair_plan(&mut candidate.plan).await {
                warn!(
                    target: "engine::multi_leg",
                    input_mint = %candidate.pair.input_mint,
                    output_mint = %candidate.pair.output_mint,
                    amount = candidate.trade_size,
                    error = %err,
                    "多腿 ALT 填充失败，跳过该组合"
                );
                continue;
            }
            match self.execute_multi_leg(candidate).await {
                Ok(_) => {
                    any_executed = true;
                }
                Err(err) => {
                    warn!(
                        target: "engine::multi_leg",
                        error = %err,
                        "多腿落地失败，继续尝试下一个组合"
                    );
                    last_error = Some(err);
                }
            }
        }

        if any_executed || last_error.is_none() {
            Ok(())
        } else {
            Err(last_error.expect("checked above"))
        }
    }

    async fn execute_multi_leg(&mut self, execution: MultiLegExecution) -> EngineResult<()> {
        let strategy_name = self.strategy.name();

        let MultiLegExecution {
            descriptor: _,
            pair,
            trade_size,
            plan,
            gross_profit,
            tip_lamports,
            tag: _,
        } = execution;

        let mut bundle = assemble_multi_leg_instructions(&plan);
        let mut final_instructions = bundle.flatten_instructions();

        if let Some(manager) = &self.flashloan {
            let flashloan_opportunity = SwapOpportunity {
                pair: pair.clone(),
                amount_in: trade_size,
                profit_lamports: gross_profit,
                tip_lamports,
                merged_quote: None,
                ultra_legs: None,
            };
            let flashloan_variant = SwapInstructionsVariant::MultiLeg(bundle.clone());
            let outcome = manager
                .assemble(&self.identity, &flashloan_opportunity, &flashloan_variant)
                .await
                .map_err(EngineError::from)?;
            final_instructions = outcome.instructions;
            if let Some(meta) = outcome.metadata {
                let overhead = manager.compute_unit_overhead();
                if overhead > 0 {
                    let new_limit = bundle.compute_unit_limit.saturating_add(overhead);
                    if !override_compute_unit_limit(&mut final_instructions, new_limit) {
                        final_instructions.insert(0, compute_unit_limit_instruction(new_limit));
                    }
                    if let Some(ix) = bundle.compute_budget_instructions.iter_mut().find(|ix| {
                        ix.program_id == COMPUTE_BUDGET_PROGRAM_ID && ix.data.first() == Some(&2)
                    }) {
                        *ix = compute_unit_limit_instruction(new_limit);
                    } else {
                        bundle
                            .compute_budget_instructions
                            .insert(0, compute_unit_limit_instruction(new_limit));
                    }
                    bundle.compute_unit_limit = new_limit;
                }

                events::flashloan_applied(
                    strategy_name,
                    meta.protocol.as_str(),
                    &meta.mint,
                    meta.borrow_amount,
                    meta.inner_instruction_count,
                );
            }
        }

        let prioritization_fee = bundle
            .prioritization_fee_lamports
            .as_ref()
            .copied()
            .unwrap_or_default();
        let jito_tip_budget = self.jito_tip_budget(tip_lamports);
        // 优先费 + 固定 Jito 小费（若配置）+ 基础网络费 = 必须覆盖的支出
        let guard_required = BASE_TX_FEE_LAMPORTS
            .saturating_add(prioritization_fee)
            .saturating_add(jito_tip_budget);
        self.apply_profit_guard(&pair.input_pubkey, guard_required, &mut final_instructions)?;

        let variant = SwapInstructionsVariant::MultiLeg(bundle);

        let prepared = self
            .tx_builder
            .build_with_sequence(&self.identity, &variant, final_instructions, tip_lamports)
            .await?;

        debug!(
            target: "engine::multi_leg",
            strategy = strategy_name,
            slot = prepared.slot,
            blockhash = %prepared.blockhash,
            "多腿交易已构建"
        );

        if self.settings.dry_run {
            info!(
                target: "engine::dry_run",
                strategy = strategy_name,
                slot = prepared.slot,
                blockhash = %prepared.blockhash,
                landers = self.landers.count(),
                "dry-run 模式：多腿交易已构建，跳过落地"
            );
            return Ok(());
        }

        let dispatch_strategy = self.settings.dispatch_strategy;
        let variant_layout = self.landers.variant_layout(dispatch_strategy);
        let plan = Arc::new(self.variant_planner.plan(
            dispatch_strategy,
            &prepared,
            &variant_layout,
        ));

        let deadline = Deadline::from_instant(Instant::now() + self.settings.landing_timeout);
        let tx_signature = plan
            .primary_variant()
            .and_then(|variant| variant.signature());
        let lander_stack = Arc::clone(&self.landers);
        let strategy_label = strategy_name.to_string();
        let tx_signature_for_log = tx_signature.clone();

        tokio::spawn(async move {
            match lander_stack
                .submit_plan(plan.as_ref(), deadline, &strategy_label)
                .await
            {
                Ok(_receipt) => {}
                Err(err) => {
                    let sig = tx_signature_for_log.as_deref().unwrap_or("");
                    warn!(
                        target: "engine::lander",
                        strategy = strategy_label.as_str(),
                        tx_signature = sig,
                        error = %err,
                        "lander submission failed"
                    );
                }
            }
        });

        Ok(())
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn execute_plan(&mut self, plan: ExecutionPlan) -> EngineResult<()> {
        let ExecutionPlan {
            opportunity,
            deadline,
        } = plan;
        let strategy_name = self.strategy.name();
        let swap_lease = match self
            .ip_allocator
            .acquire(IpTaskKind::SwapInstruction, IpLeaseMode::Ephemeral)
            .await
        {
            Ok(lease) => lease,
            Err(err) => return Err(EngineError::NetworkResource(err)),
        };
        let swap_handle = swap_lease.handle();
        let swap_ip = Some(swap_handle.ip());
        drop(swap_lease);

        let swap_variant = match self
            .swap_preparer
            .prepare(&opportunity, &self.identity, &swap_handle)
            .await
        {
            Ok(value) => {
                drop(swap_handle);
                value
            }
            Err(err) => {
                if let Some(outcome) = crate::engine::quote_dispatcher::classify_ip_outcome(&err) {
                    swap_handle.mark_outcome(outcome);
                }
                drop(swap_handle);

                if let EngineError::Dflow(dflow_err @ DflowError::ApiStatus { status, body, .. }) =
                    &err
                {
                    if status.as_u16() == 500 && body.contains("failed_to_compute_swap") {
                        let detail = dflow_err.describe();
                        warn!(
                            target: "engine::swap",
                            status = status.as_u16(),
                            error = %detail,
                            "DFlow swap 指令生成失败，跳过当前机会。Error: {body}"
                        );
                        return Ok(());
                    }
                }
                if let EngineError::Dflow(
                    dflow_err @ DflowError::RateLimited { status, body, .. },
                ) = &err
                {
                    let detail = dflow_err.describe();
                    warn!(
                        target: "engine::swap",
                        status = status.as_u16(),
                        input_mint = %opportunity.pair.input_mint,
                        output_mint = %opportunity.pair.output_mint,
                        error = %detail,
                        "DFlow 指令命中限流，放弃当前机会: {body}"
                    );
                    return Ok(());
                }
                if let EngineError::Dflow(other) = &err {
                    let detail = other.describe();
                    warn!(
                        target: "engine::swap",
                        input_mint = %opportunity.pair.input_mint,
                        output_mint = %opportunity.pair.output_mint,
                        error = %detail,
                        "DFlow 指令失败，跳过当前机会"
                    );
                    return Ok(());
                }
                if let EngineError::InvalidConfig(message) = &err {
                    if message.starts_with("Ultra 指令解析失败") {
                        error!(
                            target: "engine::swap",
                            input_mint = %opportunity.pair.input_mint,
                            output_mint = %opportunity.pair.output_mint,
                            amount_in = opportunity.amount_in,
                            error = %message,
                            "Ultra 指令解析失败，跳过当前机会"
                        );
                        return Ok(());
                    }
                }
                return Err(err);
            }
        };
        let mut compute_unit_limit = swap_variant.compute_unit_limit();
        let prioritization_fee = swap_variant
            .prioritization_fee_lamports()
            .unwrap_or_default();
        let (mut final_instructions, flashloan_meta, flashloan_overhead) = match &self.flashloan {
            Some(manager) => {
                let outcome = manager
                    .assemble(&self.identity, &opportunity, &swap_variant)
                    .await
                    .map_err(EngineError::from)?;
                let overhead = outcome
                    .metadata
                    .as_ref()
                    .map(|_| manager.compute_unit_overhead());
                (outcome.instructions, outcome.metadata, overhead)
            }
            None => (
                swap_variant.flatten_instructions(),
                None,
                Option::<u32>::None,
            ),
        };

        if let Some(overhead) = flashloan_overhead {
            if overhead > 0 {
                let new_limit = compute_unit_limit.saturating_add(overhead);
                if !override_compute_unit_limit(&mut final_instructions, new_limit) {
                    final_instructions.insert(0, compute_unit_limit_instruction(new_limit));
                }
                compute_unit_limit = new_limit;
            }
        }

        events::swap_fetched(
            strategy_name,
            &opportunity,
            compute_unit_limit,
            prioritization_fee,
            swap_ip,
        );

        if let Some(meta) = &flashloan_meta {
            events::flashloan_applied(
                strategy_name,
                meta.protocol.as_str(),
                &meta.mint,
                meta.borrow_amount,
                meta.inner_instruction_count,
            );
        }

        let jito_tip_budget = self.jito_tip_budget(opportunity.tip_lamports);
        // 优先费 + 固定 Jito 小费（若配置）+ 基础网络费 = 必须覆盖的支出
        let guard_required = BASE_TX_FEE_LAMPORTS
            .saturating_add(prioritization_fee)
            .saturating_add(jito_tip_budget);
        self.apply_profit_guard(
            &opportunity.pair.input_pubkey,
            guard_required,
            &mut final_instructions,
        )?;

        let prepared = self
            .tx_builder
            .build_with_sequence(
                &self.identity,
                &swap_variant,
                final_instructions,
                opportunity.tip_lamports,
            )
            .await?;
        events::transaction_built(
            strategy_name,
            &opportunity,
            prepared.slot,
            &prepared.blockhash.to_string(),
            prepared.last_valid_block_height,
            swap_ip,
        );

        if self.settings.dry_run {
            info!(
                target: "engine::dry_run",
                strategy = strategy_name,
                slot = prepared.slot,
                blockhash = %prepared.blockhash,
                landers = self.landers.count(),
                "dry-run 模式：交易已构建，跳过落地"
            );
            return Ok(());
        }

        let dispatch_strategy = self.settings.dispatch_strategy;
        let variant_layout = self.landers.variant_layout(dispatch_strategy);
        let plan = Arc::new(self.variant_planner.plan(
            dispatch_strategy,
            &prepared,
            &variant_layout,
        ));

        let deadline = Deadline::from_instant(deadline);
        let tx_signature = plan
            .primary_variant()
            .and_then(|variant| variant.signature());
        let lander_stack = Arc::clone(&self.landers);
        let strategy_label = strategy_name.to_string();
        let tx_signature_for_log = tx_signature.clone();

        tokio::spawn(async move {
            match lander_stack
                .submit_plan(plan.as_ref(), deadline, &strategy_label)
                .await
            {
                Ok(_receipt) => {}
                Err(err) => {
                    let sig = tx_signature_for_log.as_deref().unwrap_or("");
                    warn!(
                        target: "engine::lander",
                        strategy = strategy_label.as_str(),
                        tx_signature = sig,
                        error = %err,
                        "{}",
                        format_args!(
                            "落地失败: 策略={} 签名={} 错误={}",
                            strategy_label,
                            sig,
                            err
                        )
                    );
                }
            }
        });

        Ok(())
    }

    async fn process_strategy_tick(&mut self) -> EngineResult<Duration> {
        let tick = StrategyTick::now();
        trace!(target: "engine::tick", started_at = ?tick.at);
        let event = StrategyEvent::Tick(tick);
        let resources = StrategyResources {
            pairs: &self.trade_pairs,
            trade_profiles: &mut self.trade_profiles,
            next_batch_id: &mut self.next_batch_id,
        };
        let ctx = StrategyContext::new(resources);
        let decision = self.strategy.on_market_event(&event, ctx);
        let StrategyDecision {
            action,
            next_ready_in,
        } = decision;
        let next_wait = next_ready_in.unwrap_or_else(|| self.earliest_schedule_delay());
        if let Err(err) = self.handle_action(action).await {
            error!(
                target: "engine",
                error = %err,
                "策略 tick 执行失败，将继续运行"
            );
        }
        Ok(next_wait)
    }

    fn apply_profit_guard(
        &mut self,
        base_mint: &Pubkey,
        required_delta: u64,
        instructions: &mut Vec<Instruction>,
    ) -> EngineResult<()> {
        if required_delta == 0 || !self.lighthouse.should_guard(base_mint) {
            return Ok(());
        }

        let token_account = get_associated_token_address(&self.identity.pubkey, base_mint);
        let memory_id = self.lighthouse.next_memory_id();
        let guard = build_token_amount_guard(
            self.identity.pubkey,
            token_account,
            memory_id,
            required_delta,
        );

        let insert_pos = instructions
            .iter()
            .take_while(|ix| ix.program_id == COMPUTE_BUDGET_PROGRAM_ID)
            .count();
        instructions.insert(insert_pos, guard.memory_write);
        instructions.push(guard.assert_delta);

        Ok(())
    }
}

impl<S> StrategyEngine<S>
where
    S: Strategy<Event = StrategyEvent>,
{
    fn earliest_schedule_delay(&self) -> Duration {
        let now = Instant::now();
        self.trade_profiles
            .values()
            .map(|schedule| schedule.time_until_ready(now))
            .min()
            .unwrap_or(Duration::ZERO)
    }
}

fn ensure_route_pair(first: &BlindStep, last: &BlindStep) -> EngineResult<()> {
    if first.input != last.output {
        return Err(EngineError::InvalidConfig(
            "纯盲发路由未形成闭环：首腿输入资产与末腿输出资产不一致".to_string(),
        ));
    }
    Ok(())
}

fn derive_associated_token_address(
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Pubkey {
    let seeds: [&[u8]; 3] = [owner.as_ref(), token_program.as_ref(), mint.as_ref()];
    Pubkey::find_program_address(&seeds, &ASSOCIATED_TOKEN_PROGRAM_ID).0
}

struct AtaResolver {
    owner: Pubkey,
    cache: HashMap<(Pubkey, Pubkey), Pubkey>,
}

impl AtaResolver {
    fn new(owner: Pubkey) -> Self {
        Self {
            owner,
            cache: HashMap::new(),
        }
    }

    fn get(&mut self, mint: Pubkey, token_program: Pubkey) -> Pubkey {
        *self
            .cache
            .entry((mint, token_program))
            .or_insert_with(|| derive_associated_token_address(&self.owner, &mint, &token_program))
    }
}

fn compute_unit_limit_instruction(limit: u32) -> Instruction {
    let mut data = Vec::with_capacity(1 + mem::size_of::<u32>());
    data.push(2);
    data.extend_from_slice(&limit.to_le_bytes());
    Instruction {
        program_id: COMPUTE_BUDGET_PROGRAM_ID,
        accounts: Vec::new(),
        data,
    }
}

fn compute_unit_price_instruction(price_micro_lamports: u64) -> Instruction {
    let mut data = Vec::with_capacity(1 + mem::size_of::<u64>());
    data.push(3);
    data.extend_from_slice(&price_micro_lamports.to_le_bytes());
    Instruction {
        program_id: COMPUTE_BUDGET_PROGRAM_ID,
        accounts: Vec::new(),
        data,
    }
}

fn override_compute_unit_limit(instructions: &mut [Instruction], new_limit: u32) -> bool {
    for ix in instructions.iter_mut() {
        if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID && ix.data.first() == Some(&2) {
            *ix = compute_unit_limit_instruction(new_limit);
            return true;
        }
    }
    false
}

fn assemble_multi_leg_instructions(plan: &LegPairPlan) -> MultiLegInstructions {
    let (buy_cb, buy_limit) = extract_plan_compute_requirements(&plan.buy);
    let (sell_cb, sell_limit) = extract_plan_compute_requirements(&plan.sell);

    let mut compute_budget_instructions = Vec::new();
    compute_budget_instructions.extend(buy_cb);
    compute_budget_instructions.extend(sell_cb);

    let mut main_instructions = Vec::new();
    main_instructions.extend(plan.buy.instructions.iter().cloned());
    main_instructions.extend(plan.sell.instructions.iter().cloned());

    let mut address_lookup_table_addresses = Vec::new();
    address_lookup_table_addresses.extend(plan.buy.address_lookup_table_addresses.iter().cloned());
    address_lookup_table_addresses.extend(plan.sell.address_lookup_table_addresses.iter().cloned());

    let mut resolved_lookup_tables = Vec::new();
    resolved_lookup_tables.extend(plan.buy.resolved_lookup_tables.iter().cloned());
    resolved_lookup_tables.extend(plan.sell.resolved_lookup_tables.iter().cloned());

    let fee_sum = plan
        .buy
        .prioritization_fee_lamports
        .unwrap_or(0)
        .saturating_add(plan.sell.prioritization_fee_lamports.unwrap_or(0));
    let prioritization_fee = if fee_sum > 0 { Some(fee_sum) } else { None };

    let mut merged_limit = buy_limit
        .unwrap_or(0)
        .saturating_add(sell_limit.unwrap_or(0));
    let merged_price = plan
        .buy
        .requested_compute_unit_price_micro_lamports
        .or(plan.sell.requested_compute_unit_price_micro_lamports);

    if merged_limit == 0 {
        merged_limit =
            extract_compute_unit_limit(&compute_budget_instructions).unwrap_or(FALLBACK_CU_LIMIT);
    }

    compute_budget_instructions.retain(|ix| {
        if ix.program_id != COMPUTE_BUDGET_PROGRAM_ID {
            return true;
        }
        match ix.data.first().copied() {
            Some(2) | Some(3) => false,
            _ => true,
        }
    });

    let mut final_compute_budget = Vec::new();
    final_compute_budget.push(compute_unit_limit_instruction(merged_limit));
    if let Some(price) = merged_price {
        if price > 0 {
            final_compute_budget.push(compute_unit_price_instruction(price));
        }
    }
    final_compute_budget.extend(compute_budget_instructions);

    let mut bundle = MultiLegInstructions::new(
        final_compute_budget,
        main_instructions,
        address_lookup_table_addresses,
        resolved_lookup_tables,
        prioritization_fee,
        merged_limit,
    );
    bundle.dedup_lookup_tables();
    bundle
}

fn extract_compute_unit_limit(instructions: &[Instruction]) -> Option<u32> {
    for ix in instructions {
        if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID && ix.data.first() == Some(&2) {
            if ix.data.len() >= 5 {
                let mut buf = [0u8; 4];
                buf.copy_from_slice(&ix.data[1..5]);
                return Some(u32::from_le_bytes(buf));
            }
        }
    }
    None
}

fn extract_plan_compute_requirements(plan: &LegPlan) -> (Vec<Instruction>, Option<u32>) {
    let mut residual = Vec::new();
    let mut limit = plan.requested_compute_unit_limit;

    for ix in &plan.compute_budget_instructions {
        if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID {
            match ix.data.first().copied() {
                Some(2) => {
                    if limit.is_none() && ix.data.len() >= 5 {
                        let mut buf = [0u8; 4];
                        buf.copy_from_slice(&ix.data[1..5]);
                        limit = Some(u32::from_le_bytes(buf));
                    }
                    continue;
                }
                _ => {}
            }
        }
        residual.push(ix.clone());
    }

    if limit.is_none() {
        limit = plan.requested_compute_unit_limit;
    }

    (residual, limit)
}
