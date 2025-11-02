mod blind;
mod multi_leg;
mod quote;
mod swap;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use solana_sdk::pubkey::Pubkey;
use tracing::{debug, error, info, trace};

use crate::engine::context::{Action, StrategyContext, StrategyDecision, StrategyResources};
use crate::engine::planner::{DispatchStrategy, TxVariantPlanner};
use crate::engine::plugins::flashloan::MarginfiFlashloanManager;
use crate::engine::runtime::{LighthouseRuntime, multi_leg::MultiLegEngineContext};
use crate::engine::{
    ComputeUnitPriceMode, EngineError, EngineIdentity, EngineResult, ProfitEvaluator, QuoteConfig,
    QuoteDispatcher, QuoteExecutor, Scheduler, StrategyTick, SwapPreparer, TradeProfile,
    TransactionBuilder,
};
use crate::lander::LanderStack;
use crate::network::IpAllocator;
use crate::strategy::types::TradePair;
use crate::strategy::{Strategy, StrategyEvent};

pub(super) const BASE_TX_FEE_LAMPORTS: u64 = 5_000;

#[derive(Clone)]
pub(crate) struct MintSchedule {
    amounts: Vec<u64>,
    process_delay: Duration,
    next_ready: Instant,
}

impl MintSchedule {
    pub(crate) fn from_profile(profile: TradeProfile) -> Self {
        Self {
            amounts: profile.amounts,
            process_delay: profile.process_delay,
            next_ready: Instant::now(),
        }
    }

    pub(crate) fn take_ready_batch(&mut self, now: Instant) -> Option<Vec<u64>> {
        if self.amounts.is_empty() || now < self.next_ready {
            return None;
        }
        self.next_ready = now + self.process_delay;
        Some(self.amounts.clone())
    }

    pub(crate) fn has_amounts(&self) -> bool {
        !self.amounts.is_empty()
    }

    pub(crate) fn is_ready(&self, now: Instant) -> bool {
        now >= self.next_ready
    }

    pub(crate) fn time_until_ready(&self, now: Instant) -> Duration {
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
        let per_ip_capacity = allocator_summary.per_ip_inflight_limit.unwrap_or(1).max(1);
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

    async fn handle_action(&mut self, action: Action) -> EngineResult<()> {
        match action {
            Action::Idle => Ok(()),
            Action::Quote(batches) => self.run_quote_batches(batches).await,
            Action::DispatchBlind(batch) => self.process_blind_batch(batch).await,
        }
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
        let StrategyDecision {
            action,
            next_ready_in,
        } = self.strategy.on_market_event(&event, ctx);
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

    fn earliest_schedule_delay(&self) -> Duration {
        let now = Instant::now();
        self.trade_profiles
            .values()
            .map(|schedule| schedule.time_until_ready(now))
            .min()
            .unwrap_or(Duration::ZERO)
    }

    fn jito_tip_budget(&self, base_tip: u64) -> u64 {
        if !self.landers.has_jito() {
            return 0;
        }
        let fixed_tip = self.landers.fixed_jito_tip().unwrap_or(0);
        fixed_tip.saturating_add(base_tip)
    }
}
