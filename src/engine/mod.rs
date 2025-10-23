mod aggregator;
mod builder;
mod context;
mod error;
mod identity;
mod planner;
mod precheck;
mod profit;
mod quote;
mod scheduler;
mod swap;
mod types;

pub use aggregator::SwapInstructionsVariant;
pub use builder::{BuilderConfig, TransactionBuilder};
pub use context::{Action, StrategyContext, StrategyResources};
pub use error::{EngineError, EngineResult};
pub use identity::EngineIdentity;
pub use planner::{DispatchPlan, DispatchStrategy, TxVariant, TxVariantPlanner, VariantId};
pub use precheck::AccountPrechecker;
pub use profit::{ProfitConfig, ProfitEvaluator, TipConfig};
pub use quote::{QuoteConfig, QuoteExecutor};
pub use scheduler::Scheduler;
pub use swap::{ComputeUnitPriceMode, SwapInstructionFetcher};
pub use types::{ExecutionPlan, QuoteTask, StrategyTick, SwapOpportunity};

use std::collections::HashMap;
use std::mem;
use std::time::{Duration, Instant};

use serde_json::Value;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use tracing::{debug, info, trace, warn};

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
use crate::monitoring::events;
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
pub const BLIND_COMPUTE_UNIT_LIMIT: u32 = 230_000;
const COMPUTE_BUDGET_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ComputeBudget111111111111111111111111111111");

#[derive(Clone)]
pub struct EngineSettings {
    pub landing_timeout: Duration,
    pub quote: QuoteConfig,
    pub dry_run: bool,
    pub dispatch_strategy: DispatchStrategy,
}

impl EngineSettings {
    pub fn new(quote: QuoteConfig) -> Self {
        Self {
            landing_timeout: Duration::from_secs(2),
            quote,
            dry_run: false,
            dispatch_strategy: DispatchStrategy::default(),
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
    flashloan: Option<MarginfiFlashloanManager>,
    settings: EngineSettings,
    trade_pairs: Vec<TradePair>,
    trade_amounts: Vec<u64>,
    variant_planner: TxVariantPlanner,
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
        flashloan: Option<MarginfiFlashloanManager>,
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
            variant_planner: TxVariantPlanner::new(),
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

        let mut compute_budget_instructions =
            vec![compute_unit_limit_instruction(BLIND_COMPUTE_UNIT_LIMIT)];
        if let Some(price) = self.swap_fetcher.sample_compute_unit_price() {
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
            compute_unit_limit: BLIND_COMPUTE_UNIT_LIMIT,
            prioritization_type: None,
            dynamic_slippage_report: None,
            simulation_error: None,
        };
        let response_variant = SwapInstructionsVariant::Jupiter(response);

        let pair = TradePair::from_pubkeys(source_mint, destination_mint);
        let flashloan_opportunity = SwapOpportunity {
            pair,
            amount_in: order.amount_in,
            profit_lamports: 0,
            tip_lamports: 0,
            merged_quote: None,
        };

        let FlashloanOutcome {
            instructions: final_instructions,
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

        let prepared = self
            .tx_builder
            .build_with_sequence(&self.identity, &response_variant, final_instructions, 0)
            .await?;

        let dispatch_strategy = self.settings.dispatch_strategy;
        let variant_budget = self.landers.plan_capacity(dispatch_strategy);
        let plan = self
            .variant_planner
            .plan(dispatch_strategy, &prepared, variant_budget);

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

        let instructions = match self.swap_fetcher.fetch(&opportunity, &self.identity).await {
            Ok(value) => value,
            Err(err) => {
                if let EngineError::Dflow(DflowError::ApiStatus { status, body, .. }) = &err {
                    if status.as_u16() == 500 && body.contains("failed_to_compute_swap") {
                        warn!(
                            target: "engine::swap",
                            status = status.as_u16(),
                            "DFlow swap 指令生成失败，跳过当前机会。Error: {body}"
                        );
                        return Ok(());
                    }
                }
                if let EngineError::Dflow(DflowError::RateLimited { status, body, .. }) = &err {
                    warn!(
                        target: "engine::swap",
                        status = status.as_u16(),
                        input_mint = %opportunity.pair.input_mint,
                        output_mint = %opportunity.pair.output_mint,
                        "DFlow 指令命中限流，放弃当前机会: {body}"
                    );
                    return Ok(());
                }
                return Err(err);
            }
        };
        let compute_unit_limit = instructions.compute_unit_limit();
        let prioritization_fee = instructions
            .prioritization_fee_lamports()
            .unwrap_or_default();
        events::swap_fetched(
            strategy_name,
            &opportunity,
            compute_unit_limit,
            prioritization_fee,
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

        let dispatch_strategy = self.settings.dispatch_strategy;
        let variant_budget = self.landers.plan_capacity(dispatch_strategy);
        let plan = self
            .variant_planner
            .plan(dispatch_strategy, &prepared, variant_budget);

        let deadline = Deadline::from_instant(deadline);
        let tx_signature = plan
            .primary_variant()
            .and_then(|variant| variant.signature());

        match self
            .landers
            .submit_plan(&plan, deadline, strategy_name)
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
