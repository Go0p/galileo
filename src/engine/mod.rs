mod builder;
mod context;
mod error;
mod identity;
mod profit;
mod quote;
mod scheduler;
mod swap;
mod types;

pub use builder::{BuilderConfig, PreparedTransaction, TransactionBuilder};
pub use context::{Action, StrategyContext, StrategyResources};
pub use error::{EngineError, EngineResult};
pub use identity::EngineIdentity;
pub use profit::{ProfitConfig, ProfitEvaluator, TipConfig};
pub use quote::{QuoteConfig, QuoteExecutor};
pub use scheduler::Scheduler;
pub use swap::SwapInstructionFetcher;
pub use types::{ExecutionPlan, QuoteTask, StrategyTick, SwapOpportunity};

use std::time::{Duration, Instant};

use tracing::{debug, info, trace, warn};

use crate::lander::{Deadline, LanderStack};
use crate::monitoring::events;
use crate::strategy::types::TradePair;
use crate::strategy::{Strategy, StrategyEvent};

#[derive(Clone)]
pub struct EngineSettings {
    pub landing_timeout: Duration,
    pub quote: QuoteConfig,
    pub compute_unit_price_override: Option<u64>,
    pub dry_run: bool,
}

impl EngineSettings {
    pub fn new(quote: QuoteConfig) -> Self {
        Self {
            landing_timeout: Duration::from_secs(2),
            quote,
            compute_unit_price_override: None,
            dry_run: false,
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
}

pub struct StrategyEngine<S>
where
    S: Strategy,
{
    strategy: S,
    landers: LanderStack,
    identity: EngineIdentity,
    quote_executor: QuoteExecutor,
    profit_evaluator: ProfitEvaluator,
    swap_fetcher: SwapInstructionFetcher,
    tx_builder: TransactionBuilder,
    scheduler: Scheduler,
    settings: EngineSettings,
    trade_pairs: Vec<TradePair>,
    trade_amounts: Vec<u64>,
}

impl<S> StrategyEngine<S>
where
    S: Strategy<Event = StrategyEvent>,
{
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        strategy: S,
        landers: LanderStack,
        identity: EngineIdentity,
        quote_executor: QuoteExecutor,
        profit_evaluator: ProfitEvaluator,
        swap_fetcher: SwapInstructionFetcher,
        tx_builder: TransactionBuilder,
        scheduler: Scheduler,
        settings: EngineSettings,
        trade_pairs: Vec<TradePair>,
        trade_amounts: Vec<u64>,
    ) -> Self {
        Self {
            strategy,
            landers,
            identity,
            quote_executor,
            profit_evaluator,
            swap_fetcher,
            tx_builder,
            scheduler,
            settings,
            trade_pairs,
            trade_amounts,
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn run(mut self) -> EngineResult<()> {
        if self.landers.is_empty() {
            return Err(EngineError::InvalidConfig("未配置可用的落地器".into()));
        }

        info!(target: "engine", strategy = self.strategy.name(), "策略引擎启动");
        loop {
            let tick = StrategyTick::now();
            trace!(target: "engine::tick", started_at = ?tick.at);
            let event = StrategyEvent::Tick(tick);
            let resources = StrategyResources {
                pairs: &self.trade_pairs,
                trade_amounts: &self.trade_amounts,
            };
            let ctx = StrategyContext::new(resources);
            let action = self.strategy.on_market_event(&event, ctx);
            self.handle_action(action).await?;
            self.scheduler.wait().await;
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn handle_action(&mut self, action: Action) -> EngineResult<()> {
        match action {
            Action::Idle => Ok(()),
            Action::Quote(tasks) => {
                for task in tasks {
                    self.process_task(task).await?;
                }
                Ok(())
            }
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn process_task(&mut self, task: QuoteTask) -> EngineResult<()> {
        let strategy_name = self.strategy.name();
        events::quote_start(strategy_name, &task);
        let quote_started = Instant::now();
        let double_quote = match self
            .quote_executor
            .round_trip(&task, &self.settings.quote)
            .await?
        {
            Some(value) => value,
            None => {
                events::quote_end(strategy_name, &task, false, quote_started.elapsed());
                return Ok(());
            }
        };
        events::quote_end(strategy_name, &task, true, quote_started.elapsed());

        let opportunity =
            match self
                .profit_evaluator
                .evaluate(task.amount, &double_quote, &task.pair)
            {
                Some(value) => value,
                None => return Ok(()),
            };
        events::profit_detected(strategy_name, &opportunity);

        let plan = ExecutionPlan::with_deadline(opportunity, self.settings.landing_timeout);
        self.execute_plan(plan).await
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn execute_plan(&mut self, plan: ExecutionPlan) -> EngineResult<()> {
        let ExecutionPlan {
            opportunity,
            deadline,
        } = plan;
        let strategy_name = self.strategy.name();
        info!(
            target: "engine::opportunity",
            input_mint = %opportunity.pair.input_mint,
            output_mint = %opportunity.pair.output_mint,
            amount_in = opportunity.amount_in,
            profit = opportunity.profit_lamports,
            tip = opportunity.tip_lamports,
            net_profit = opportunity.net_profit(),
            "检测到套利机会"
        );

        let instructions = self
            .swap_fetcher
            .fetch(
                &opportunity,
                &self.identity,
                self.settings.compute_unit_price_override,
            )
            .await?;
        events::swap_fetched(
            strategy_name,
            &opportunity,
            instructions.compute_unit_limit,
            instructions.prioritization_fee_lamports,
        );

        let prepared = self
            .tx_builder
            .build(&self.identity, &instructions, opportunity.tip_lamports)
            .await?;
        events::transaction_built(
            strategy_name,
            &opportunity,
            prepared.slot,
            &prepared.blockhash.to_string(),
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

        let deadline = Deadline::from_instant(deadline);

        let tx_signature = prepared
            .transaction
            .signatures
            .get(0)
            .map(|sig| sig.to_string());

        match self
            .landers
            .submit(&prepared, deadline, strategy_name)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                debug!(
                    target: "engine::lander",
                    error = %err,
                    tx_signature = tx_signature.as_deref().unwrap_or(""),
                    "lander submission detail"
                );
                warn!(
                    target: "engine::lander",
                    tx_signature = tx_signature.as_deref().unwrap_or(""),
                    "落地失败"
                );
                let message = tx_signature
                    .map(|sig| format!("交易 {sig} 落地失败"))
                    .unwrap_or_else(|| "交易落地失败".to_string());
                Err(EngineError::Landing(message))
            }
        }
    }
}
