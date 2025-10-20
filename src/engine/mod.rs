mod builder;
mod context;
mod error;
mod identity;
mod precheck;
mod profit;
mod quote;
mod scheduler;
mod swap;
mod types;

pub use builder::{BuilderConfig, PreparedTransaction, TransactionBuilder};
pub use context::{Action, StrategyContext, StrategyResources};
pub use error::{EngineError, EngineResult};
pub use identity::EngineIdentity;
pub use precheck::AccountPrechecker;
pub use profit::{ProfitConfig, ProfitEvaluator, TipConfig};
pub use quote::{QuoteConfig, QuoteExecutor};
pub use scheduler::Scheduler;
pub use swap::{ComputeUnitPriceMode, SwapInstructionFetcher};
pub use types::{ExecutionPlan, QuoteTask, StrategyTick, SwapOpportunity};

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde_json::Value;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use tracing::{debug, info, trace, warn};

use crate::api::SwapInstructionsResponse;
use crate::flashloan::{FlashloanManager, FlashloanOutcome};
use crate::lander::{Deadline, LanderStack};
use crate::monitoring::events;
use crate::strategy::types::{BlindDex, BlindOrder, BlindStep, BlindSwapDirection, TradePair};
use crate::txs::jupiter::route_v2::{RouteV2Accounts, RouteV2InstructionBuilder};
use crate::txs::jupiter::types::{EncodedSwap, RoutePlanStepV2};

const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
use crate::strategy::{Strategy, StrategyEvent};

#[derive(Clone)]
pub struct EngineSettings {
    pub landing_timeout: Duration,
    pub quote: QuoteConfig,
    pub dry_run: bool,
}

impl EngineSettings {
    pub fn new(quote: QuoteConfig) -> Self {
        Self {
            landing_timeout: Duration::from_secs(2),
            quote,
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
    flashloan: Option<FlashloanManager>,
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
        flashloan: Option<FlashloanManager>,
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
            flashloan,
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

        info!(
            target: "engine",
            strategy = self.strategy.name(),
            "策略引擎启动"
        );

        self.run_jupiter().await
    }

    async fn run_jupiter(&mut self) -> EngineResult<()> {
        loop {
            self.process_strategy_tick().await?;
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

        ensure_solfi_pair(first_step, last_step)?;

        let mut ata_resolver = AtaResolver::new(self.identity.pubkey);

        let (route_plan, remaining_accounts) = self.build_route_plan(order, &mut ata_resolver)?;

        let (source_mint, source_program) = resolve_step_input(first_step);
        let (destination_mint, destination_program) = resolve_step_output(last_step);
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
        accounts.remaining_accounts = remaining_accounts;

        let quoted_out_amount = order.amount_in.saturating_add(1);

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

        let response = SwapInstructionsResponse {
            raw: Value::Null,
            token_ledger_instruction: None,
            compute_budget_instructions: Vec::new(),
            setup_instructions: Vec::new(),
            swap_instruction: instruction,
            cleanup_instruction: None,
            other_instructions: Vec::new(),
            address_lookup_table_addresses: Vec::new(),
            prioritization_fee_lamports: 0,
            compute_unit_limit: 0,
            prioritization_type: None,
            dynamic_slippage_report: None,
            simulation_error: None,
        };

        let prepared = self.tx_builder.build(&self.identity, &response, 0).await?;

        let deadline = Deadline::from_instant(Instant::now() + self.settings.landing_timeout);

        let strategy_name = self.strategy.name();
        match self
            .landers
            .submit(&prepared, deadline, strategy_name)
            .await
        {
            Ok(_) => Ok(()),
            Err(err) => {
                let tx_signature = prepared
                    .transaction
                    .signatures
                    .get(0)
                    .map(|sig| sig.to_string());
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

    fn build_route_plan(
        &self,
        order: &BlindOrder,
        resolver: &mut AtaResolver,
    ) -> EngineResult<(Vec<RoutePlanStepV2>, Vec<AccountMeta>)> {
        if order.steps.is_empty() {
            return Err(EngineError::InvalidConfig("盲发订单缺少步骤".to_string()));
        }

        let base_mint = order.steps[0].meta.base_mint;
        let quote_mint = order.steps[0].meta.quote_mint;

        let mut route_plan = Vec::with_capacity(order.steps.len());
        let mut remaining_accounts = Vec::with_capacity(order.steps.len() * 12);

        for (idx, step) in order.steps.iter().enumerate() {
            if step.dex != BlindDex::SolFiV2 {
                return Err(EngineError::InvalidConfig(
                    "纯盲发目前仅支持 SolFiV2".to_string(),
                ));
            }

            if step.meta.base_mint != base_mint || step.meta.quote_mint != quote_mint {
                return Err(EngineError::InvalidConfig(
                    "纯盲发路由中的 SolFiV2 市场配置不一致".to_string(),
                ));
            }

            let encoded_swap = EncodedSwap::from_name(
                "SolFiV2",
                &matches!(step.direction, BlindSwapDirection::QuoteToBase),
            )
            .map_err(|err| EngineError::InvalidConfig(format!("构造 SolFiV2 swap 失败: {err}")))?;

            route_plan.push(RoutePlanStepV2 {
                swap: encoded_swap,
                bps: 10_000,
                input_index: if idx == 0 { 0 } else { 1 },
                output_index: if idx == 0 { 1 } else { 0 },
            });

            let user_base = resolver.get(step.meta.base_mint, step.meta.base_token_program);
            let user_quote = resolver.get(step.meta.quote_mint, step.meta.quote_token_program);

            remaining_accounts.extend(build_solfi_remaining_accounts(step, user_base, user_quote));
        }

        Ok((route_plan, remaining_accounts))
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
            .fetch(&opportunity, &self.identity)
            .await?;
        events::swap_fetched(
            strategy_name,
            &opportunity,
            instructions.compute_unit_limit,
            instructions.prioritization_fee_lamports,
        );

        let FlashloanOutcome {
            instructions: final_instructions,
            metadata: flashloan_meta,
        } = match &self.flashloan {
            Some(manager) => manager
                .assemble(&self.identity, &opportunity, &instructions)
                .await
                .map_err(EngineError::from)?,
            None => FlashloanOutcome {
                instructions: instructions.flatten_instructions(),
                metadata: None,
            },
        };

        if let Some(meta) = &flashloan_meta {
            events::flashloan_applied(
                strategy_name,
                meta.protocol.as_str(),
                &meta.mint,
                meta.borrow_amount,
                meta.inner_instruction_count,
            );
        }

        let prepared = self
            .tx_builder
            .build_with_sequence(
                &self.identity,
                &instructions,
                final_instructions,
                opportunity.tip_lamports,
            )
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

    async fn process_strategy_tick(&mut self) -> EngineResult<()> {
        let tick = StrategyTick::now();
        trace!(target: "engine::tick", started_at = ?tick.at);
        let event = StrategyEvent::Tick(tick);
        let resources = StrategyResources {
            pairs: &self.trade_pairs,
            trade_amounts: &self.trade_amounts,
        };
        let ctx = StrategyContext::new(resources);
        let action = self.strategy.on_market_event(&event, ctx);
        self.handle_action(action).await
    }
}

fn ensure_solfi_pair(first: &BlindStep, last: &BlindStep) -> EngineResult<()> {
    if first.dex != BlindDex::SolFiV2 || last.dex != BlindDex::SolFiV2 {
        return Err(EngineError::InvalidConfig(
            "纯盲发目前仅支持 SolFiV2".to_string(),
        ));
    }
    if first.meta.base_mint != last.meta.base_mint || first.meta.quote_mint != last.meta.quote_mint
    {
        return Err(EngineError::InvalidConfig(
            "纯盲发路由 base/quote mint 不一致".to_string(),
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

fn build_solfi_remaining_accounts(
    step: &BlindStep,
    user_base: Pubkey,
    user_quote: Pubkey,
) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new(step.market, false),
        AccountMeta::new_readonly(step.meta.oracle, false),
        AccountMeta::new_readonly(step.meta.config, false),
        AccountMeta::new(step.meta.base_vault, false),
        AccountMeta::new(step.meta.quote_vault, false),
        AccountMeta::new(user_base, false),
        AccountMeta::new(user_quote, false),
        AccountMeta::new_readonly(step.meta.base_mint, false),
        AccountMeta::new_readonly(step.meta.quote_mint, false),
        AccountMeta::new_readonly(step.meta.base_token_program, false),
        AccountMeta::new_readonly(step.meta.quote_token_program, false),
        AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
    ]
}

fn resolve_step_input(step: &BlindStep) -> (Pubkey, Pubkey) {
    match step.direction {
        BlindSwapDirection::QuoteToBase => (step.meta.quote_mint, step.meta.quote_token_program),
        BlindSwapDirection::BaseToQuote => (step.meta.base_mint, step.meta.base_token_program),
    }
}

fn resolve_step_output(step: &BlindStep) -> (Pubkey, Pubkey) {
    match step.direction {
        BlindSwapDirection::QuoteToBase => (step.meta.base_mint, step.meta.base_token_program),
        BlindSwapDirection::BaseToQuote => (step.meta.quote_mint, step.meta.quote_token_program),
    }
}
