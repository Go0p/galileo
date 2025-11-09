mod blind;
mod multi_leg;
mod quote;
mod swap;

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use solana_sdk::pubkey::Pubkey;
use tracing::{debug, error, info, trace};

use crate::engine::context::{Action, StrategyContext, StrategyDecision, StrategyResources};
use crate::engine::planner::{DispatchStrategy, TxVariantPlanner};
use crate::engine::plugins::flashloan::MarginfiFlashloanManager;
use crate::engine::runtime::{LighthouseRuntime, multi_leg::MultiLegEngineContext};
use crate::engine::titan::subscription::{TitanSubscriptionPlan, TitanSubscriptionPlanner};
use crate::engine::{
    ComputeUnitPriceMode, EngineError, EngineIdentity, EngineResult, ProfitEvaluator, QuoteCadence,
    QuoteConfig, QuoteDispatcher, QuoteExecutor, Scheduler, StrategyTick, SwapPreparer,
    TradeProfile, TransactionBuilder,
};
use crate::lander::LanderStack;
use crate::network::IpAllocator;
use crate::strategy::types::TradePair;
use crate::strategy::{Strategy, StrategyEvent};

pub(super) const BASE_TX_FEE_LAMPORTS: u64 = 5_000;

#[derive(Clone)]
pub(crate) struct MintSchedule {
    amounts: Vec<u64>,
}

impl MintSchedule {
    pub(crate) fn from_profile(profile: TradeProfile) -> Self {
        Self {
            amounts: profile.amounts,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.amounts.is_empty()
    }

    pub(crate) fn clone_amounts(&self) -> Vec<u64> {
        self.amounts.clone()
    }
}

#[derive(Clone, Default)]
pub struct LighthouseSettings {
    pub enable: bool,
    pub profit_guard_mints: Vec<Pubkey>,
    pub memory_slots: Option<u8>,
    pub existing_memory_ids: Vec<u8>,
    pub sol_price_feed: Option<SolPriceFeedSettings>,
}

#[derive(Clone)]
pub struct SolPriceFeedSettings {
    pub url: String,
    pub refresh: Duration,
    pub guard_padding: u64,
}

impl Default for SolPriceFeedSettings {
    fn default() -> Self {
        Self {
            url: String::new(),
            refresh: Duration::from_secs(1),
            guard_padding: 200,
        }
    }
}

#[derive(Clone, Default)]
pub struct ConsoleSummarySettings {
    pub enable: bool,
}

#[derive(Clone)]
pub struct EngineSettings {
    pub landing_timeout: Duration,
    pub quote: QuoteConfig,
    pub dry_run: bool,
    pub dispatch_strategy: DispatchStrategy,
    pub cu_multiplier: f64,
    compute_unit_price_mode: Option<ComputeUnitPriceMode>,
    pub quote_cadence: QuoteCadence,
    pub lighthouse: LighthouseSettings,
    pub console_summary: ConsoleSummarySettings,
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
            quote_cadence: QuoteCadence::default(),
            lighthouse: LighthouseSettings::default(),
            console_summary: ConsoleSummarySettings::default(),
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

    pub fn with_console_summary(mut self, console_summary: ConsoleSummarySettings) -> Self {
        self.console_summary = console_summary;
        self
    }

    pub fn with_quote_cadence(mut self, cadence: QuoteCadence) -> Self {
        self.quote_cadence = cadence;
        self
    }

    pub fn sample_compute_unit_price(&self) -> Option<u64> {
        self.compute_unit_price_mode
            .as_ref()
            .map(|mode| mode.sample())
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
    multi_leg: Option<Arc<MultiLegEngineContext>>,
    lighthouse: LighthouseRuntime,
    titan_plan: Option<TitanSubscriptionPlan>,
    titan_bootstrapped: bool,
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
        let quote_dispatcher =
            QuoteDispatcher::new(Arc::clone(&ip_allocator), settings.quote_cadence.clone());
        let allocator_summary = ip_allocator.summary();
        let per_ip_capacity = allocator_summary.per_ip_inflight_limit.unwrap_or(1).max(1);
        let ip_capacity_hint = allocator_summary
            .total_slots
            .max(1)
            .saturating_mul(per_ip_capacity);
        let trade_profiles: BTreeMap<Pubkey, MintSchedule> = trade_profiles
            .into_iter()
            .map(|(mint, profile)| (mint, MintSchedule::from_profile(profile)))
            .collect();
        let titan_plan = multi_leg.as_ref().and_then(|_| {
            let ips = ip_allocator.slot_ips();
            if ips.is_empty() {
                return None;
            }
            let plan = TitanSubscriptionPlanner::build_plan(&trade_pairs, &trade_profiles, &ips);
            if plan.is_empty() {
                None
            } else {
                debug!(
                    target: "engine::titan",
                    ip_slots = ips.len(),
                    assignments = plan.len(),
                    "Titan 订阅计划已生成"
                );
                Some(plan)
            }
        });
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
            multi_leg: multi_leg.map(Arc::new),
            lighthouse: lighthouse_runtime,
            titan_plan,
            titan_bootstrapped: false,
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn run(mut self) -> EngineResult<()> {
        if self.landers.is_empty() {
            return Err(EngineError::InvalidConfig("未配置可用的落地器".into()));
        }

        self.bootstrap_titan_streams().await?;

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

    async fn bootstrap_titan_streams(&mut self) -> EngineResult<()> {
        if self.titan_bootstrapped {
            return Ok(());
        }
        let Some(plan) = self.titan_plan.as_ref() else {
            self.titan_bootstrapped = true;
            return Ok(());
        };
        let Some(ctx) = self.multi_leg.as_ref() else {
            self.titan_bootstrapped = true;
            return Ok(());
        };
        let Some(source) = ctx.titan_source() else {
            self.titan_bootstrapped = true;
            return Ok(());
        };

        info!(
            target: "engine::titan",
            assignments = plan.len(),
            "预热 Titan 推流订阅"
        );

        source
            .bootstrap_plan(plan)
            .await
            .map_err(|err| EngineError::Internal(format!("Titan 推流预热失败: {err}")))?;
        self.titan_bootstrapped = true;
        Ok(())
    }

    async fn run_jupiter(&mut self) -> EngineResult<()> {
        loop {
            let next_wait = self.process_strategy_tick().await?;
            self.scheduler.wait(next_wait).await;
        }
    }

    async fn handle_action(&mut self, action: Action) -> EngineResult<Option<Duration>> {
        match action {
            Action::Idle => Ok(None),
            Action::Quote(batches) => self.run_quote_batches(batches).await.map(Some),
            Action::DispatchBlind(batch) => {
                self.process_blind_batch(batch).await?;
                Ok(None)
            }
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
            titan_plan: self.titan_plan.as_ref(),
        };
        let ctx = StrategyContext::new(resources);
        let StrategyDecision {
            action,
            next_ready_in,
        } = self.strategy.on_market_event(&event, ctx);
        let strategy_wait = next_ready_in.unwrap_or(Duration::ZERO);
        let cadence_wait = match self.handle_action(action).await {
            Ok(delay) => delay.unwrap_or(Duration::ZERO),
            Err(err) => {
                error!(
                    target: "engine",
                    error = %err,
                    "策略 tick 执行失败，将继续运行"
                );
                Duration::ZERO
            }
        };

        Ok(strategy_wait.max(cadence_wait))
    }
}
