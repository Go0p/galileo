use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use tracing::warn;

use crate::cache::cached_associated_token_address;
use crate::dexes::clmm::RaydiumClmmAdapter;
use crate::dexes::dlmm::MeteoraDlmmAdapter;
use crate::dexes::framework::{SwapAccountAssembler, SwapAccountsContext, SwapFlow};
use crate::dexes::humidifi::HumidiFiAdapter;
use crate::dexes::obric_v2::ObricV2Adapter;
use crate::dexes::saros::SarosAdapter;
use crate::dexes::solfi_v2::SolFiV2Adapter;
use crate::dexes::tessera_v::TesseraVAdapter;
use crate::dexes::whirlpool::WhirlpoolAdapter;
use crate::dexes::zerofi::ZeroFiAdapter;
use crate::engine::aggregator::MultiLegInstructions;
use crate::engine::assembly::decorators::{
    ComputeBudgetDecorator, FlashloanDecorator, GuardBudgetDecorator, ProfitGuardDecorator,
    TipDecorator,
};
use crate::engine::assembly::{
    AssemblyContext, DecoratorChain, InstructionBundle, attach_lighthouse,
};
use crate::engine::types::SwapOpportunity;
use crate::engine::{EngineError, EngineResult, SwapInstructionsVariant};
use crate::instructions::compute_budget::compute_budget_sequence;
use crate::instructions::jupiter::route_v2::{RouteV2Accounts, RouteV2InstructionBuilder};
use crate::instructions::jupiter::swaps::{
    HumidiFiSwap, MeteoraDlmmSwap, MeteoraDlmmSwapV2, ObricSwap, RaydiumClmmSwap,
    RaydiumClmmSwapV2, SarosSwap, SolFiV2Swap, TesseraVSide, TesseraVSwap, WhirlpoolSwap,
    WhirlpoolSwapV2, ZeroFiSwap,
};
use crate::instructions::jupiter::types::{JUPITER_V6_PROGRAM_ID, RoutePlanStepV2};
use crate::lander::Deadline;
use crate::monitoring::events;
use crate::strategy::types::{BlindDex, BlindMarketMeta, BlindOrder, BlindStep, TradePair};
use crate::strategy::{Strategy, StrategyEvent};

use crate::engine::FALLBACK_CU_LIMIT;

use super::BASE_TX_FEE_LAMPORTS;
use super::StrategyEngine;

impl<S> StrategyEngine<S>
where
    S: Strategy<Event = StrategyEvent>,
{
    pub(super) async fn process_blind_batch(&mut self, batch: Vec<BlindOrder>) -> EngineResult<()> {
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
        let sampled_price = self
            .settings
            .sample_compute_unit_price()
            .or_else(|| self.swap_preparer.sample_compute_unit_price());
        let compute_budget_instructions =
            compute_budget_sequence(sampled_price.unwrap_or(0), compute_unit_limit, None)
                .into_vec();

        let lookup_table_accounts = order.lookup_tables.clone();
        let lookup_table_addresses: Vec<Pubkey> = lookup_table_accounts
            .iter()
            .map(|table| table.key)
            .collect();

        let bundle = MultiLegInstructions::new(
            compute_budget_instructions,
            vec![instruction],
            lookup_table_addresses,
            lookup_table_accounts,
            None,
            compute_unit_limit,
        );
        let mut response_variant = SwapInstructionsVariant::MultiLeg(bundle);

        let pair = TradePair::from_pubkeys(source_mint, destination_mint);
        let mut jito_tip_plan = self.landers.draw_jito_tip_plan();
        let mut effective_tip = 0u64;
        if let Some(plan) = jito_tip_plan.as_ref() {
            if plan.lamports > 0 {
                effective_tip = plan.lamports;
            } else {
                jito_tip_plan = None;
            }
        }
        let flashloan_opportunity = SwapOpportunity {
            pair,
            amount_in: order.amount_in,
            profit_lamports: 0,
            tip_lamports: effective_tip,
            merged_quote: None,
            ultra_legs: None,
        };

        let strategy_name = self.strategy.name();

        let mut bundle =
            InstructionBundle::from_instructions(response_variant.flatten_instructions());
        bundle.set_lookup_tables(
            response_variant.address_lookup_table_addresses().to_vec(),
            response_variant.resolved_lookup_tables().to_vec(),
        );
        let mut assembly_ctx = AssemblyContext::new(&self.identity);
        assembly_ctx.base_mint = Some(&source_mint);
        assembly_ctx.compute_unit_limit = compute_unit_limit;
        assembly_ctx.compute_unit_price = sampled_price;
        assembly_ctx.guard_required = BASE_TX_FEE_LAMPORTS;
        assembly_ctx.prioritization_fee = 0;
        assembly_ctx.tip_lamports = effective_tip;
        assembly_ctx.jito_tip_budget = self.jito_tip_budget(effective_tip);
        assembly_ctx.jito_tip_plan = jito_tip_plan.clone();
        assembly_ctx.variant = Some(&mut response_variant);
        assembly_ctx.opportunity = Some(&flashloan_opportunity);
        assembly_ctx.flashloan_manager = self.flashloan.as_ref();
        attach_lighthouse(&mut assembly_ctx, &mut self.lighthouse);

        let mut decorators = DecoratorChain::new();
        decorators.register(FlashloanDecorator);
        decorators.register(ComputeBudgetDecorator);
        decorators.register(TipDecorator);
        decorators.register(GuardBudgetDecorator);
        decorators.register(ProfitGuardDecorator);

        decorators.apply_all(&mut bundle, &mut assembly_ctx).await?;

        if let Some(meta) = &assembly_ctx.flashloan_metadata {
            events::flashloan_applied(
                strategy_name,
                meta.protocol.as_str(),
                &meta.mint,
                meta.borrow_amount,
                meta.inner_instruction_count,
            );
        }

        let final_instructions = bundle.into_flattened();

        let prepared = self
            .tx_builder
            .build_with_sequence(
                &self.identity,
                &response_variant,
                final_instructions,
                effective_tip,
                jito_tip_plan.clone(),
            )
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
                (BlindMarketMeta::Saros(_), BlindDex::Saros) => {
                    SarosSwap::encode().map_err(|err| {
                        EngineError::InvalidConfig(format!("构造 Saros swap 失败: {err}"))
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
                BlindMarketMeta::Saros(meta) => {
                    SarosAdapter::shared().assemble_remaining_accounts(
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
}

fn ensure_route_pair(first: &BlindStep, last: &BlindStep) -> EngineResult<()> {
    if first.input != last.output {
        return Err(EngineError::InvalidConfig(
            "纯盲发路由未形成闭环：首腿输入资产与末腿输出资产不一致".to_string(),
        ));
    }
    Ok(())
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
            .or_insert_with(|| cached_associated_token_address(&self.owner, &mint, &token_program))
    }
}
