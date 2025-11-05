use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Result, anyhow, bail};
use rand::seq::SliceRandom;
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{account::Account, message::AddressLookupTableAccount, pubkey::Pubkey};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::config;
use crate::dexes::{
    clmm::{RAYDIUM_CLMM_PROGRAM_ID, RaydiumClmmAdapter},
    dlmm::{METEORA_DLMM_PROGRAM_ID, MeteoraDlmmAdapter},
    framework::{DexMarketMeta, DexMetaProvider, SwapFlow},
    humidifi::{HUMIDIFI_PROGRAM_ID, HumidiFiAdapter},
    obric_v2::{OBRIC_V2_PROGRAM_ID, ObricV2Adapter},
    saros::{SAROS_PROGRAM_ID, SarosAdapter},
    solfi_v2::{SOLFI_V2_PROGRAM_ID, SolFiV2Adapter},
    tessera_v::{TESSERA_V_PROGRAM_ID, TesseraVAdapter},
    whirlpool::{ORCA_WHIRLPOOL_PROGRAM_ID, WhirlpoolAdapter},
    zerofi::{ZEROFI_PROGRAM_ID, ZeroFiAdapter},
};
use crate::engine::{Action, EngineError, EngineResult, StrategyContext, StrategyDecision};
use crate::monitoring::events;
use crate::strategy::pure_blind::dynamic::DynamicRouteUpdate;
use crate::strategy::pure_blind::observer::{
    PoolCatalog, RouteCatalog, RouteKey, RouteProfile, RouteStatsSnapshot,
};

use crate::strategy::types::{
    BlindAsset, BlindDex, BlindMarketMeta, BlindOrder, BlindRoutePlan, BlindStep, RouteSource,
};
use crate::strategy::{Strategy, StrategyEvent};

/// 纯盲发路由构建器：按配置解析盲发市场并生成双向路由。
const LOOKUP_TABLE_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("AddressLookupTab1e1111111111111111111111111");

pub struct PureBlindRouteBuilder<'a> {
    config: &'a config::PureBlindStrategyConfig,
    rpc_client: &'a RpcClient,
}

impl<'a> PureBlindRouteBuilder<'a> {
    pub fn new(config: &'a config::PureBlindStrategyConfig, rpc_client: &'a RpcClient) -> Self {
        Self { config, rpc_client }
    }

    pub async fn build(&self) -> EngineResult<Vec<BlindRoutePlan>> {
        let base_mints = self.parse_base_mints()?;

        let plans = self.build_manual_routes(&base_mints).await?;

        if plans.is_empty() {
            if self
                .config
                .observer
                .as_ref()
                .map_or(false, |observer| observer.enable)
            {
                tracing::info!(
                    target: "strategy::pure_blind",
                    "未配置静态盲发路由，将等待观察器注入动态路线"
                );
                return Ok(plans);
            }

            return Err(EngineError::InvalidConfig(
                "纯盲发模式未生成任何可用路由，请检查 pure_blind_strategy 配置".into(),
            ));
        }

        Ok(plans)
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn build_manual_routes(
        &self,
        base_mints: &[Pubkey],
    ) -> EngineResult<Vec<BlindRoutePlan>> {
        let mut plans = Vec::with_capacity(self.config.overrides.len());
        for route in &self.config.overrides {
            plans.push(self.build_manual_plan(route, base_mints).await?);
        }
        Ok(plans)
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn build_manual_plan(
        &self,
        route: &config::PureBlindOverrideConfig,
        base_mints: &[Pubkey],
    ) -> EngineResult<BlindRoutePlan> {
        let route_label = route.name.as_deref().unwrap_or("<pure_route>");

        if route.legs.len() < 2 {
            return Err(EngineError::InvalidConfig(format!(
                "pure_blind_strategy.overrides `{route_label}` 至少需要 2 条腿"
            )));
        }

        let mut markets = Vec::with_capacity(route.legs.len());
        for (idx, leg) in route.legs.iter().enumerate() {
            let market = Pubkey::from_str(leg.market.trim()).map_err(|err| {
                EngineError::InvalidConfig(format!(
                    "pure_blind_strategy.overrides `{route_label}` 第 {idx} 条腿 market `{}` 解析失败: {err}",
                    leg.market
                ))
            })?;
            markets.push(market);
        }

        let resolved = self.load_market_metas(&markets, route_label).await?;
        let lookup_tables = self
            .fetch_lookup_tables_from_strings(&route.lookup_tables, route_label)
            .await?;

        self.assemble_route_plan(
            route_label.to_string(),
            resolved,
            lookup_tables,
            base_mints,
            None,
            RouteSource::Manual,
            None,
        )
    }

    async fn load_market_metas(
        &self,
        markets: &[Pubkey],
        route_label: &str,
    ) -> EngineResult<Vec<ResolvedMarketMeta>> {
        let accounts = self
            .rpc_client
            .get_multiple_accounts(markets)
            .await
            .map_err(EngineError::Rpc)?;

        let mut resolved = Vec::with_capacity(markets.len());
        for (idx, market) in markets.iter().enumerate() {
            let account = accounts
                .get(idx)
                .and_then(|acc| acc.as_ref())
                .ok_or_else(|| {
                    EngineError::InvalidConfig(format!(
                        "pure_blind_strategy.overrides `{route_label}` 第 {idx} 条腿 market `{market}` 不存在"
                    ))
                })?;

            let meta = self.resolve_market_meta(*market, account).await?;
            resolved.push(meta);
        }

        Ok(resolved)
    }

    async fn fetch_lookup_tables_from_strings(
        &self,
        values: &[String],
        route_label: &str,
    ) -> EngineResult<Vec<AddressLookupTableAccount>> {
        if values.is_empty() {
            return Ok(Vec::new());
        }

        let mut pubkeys = Vec::with_capacity(values.len());
        for (idx, value) in values.iter().enumerate() {
            let parsed = Pubkey::from_str(value.trim()).map_err(|err| {
                EngineError::InvalidConfig(format!(
                    "pure_blind_strategy.overrides `{route_label}` 第 {idx} 个 lookup table `{value}` 解析失败: {err}"
                ))
            })?;
            pubkeys.push(parsed);
        }

        self.fetch_lookup_tables(&pubkeys, route_label).await
    }

    async fn fetch_lookup_tables(
        &self,
        tables: &[Pubkey],
        route_label: &str,
    ) -> EngineResult<Vec<AddressLookupTableAccount>> {
        if tables.is_empty() {
            return Ok(Vec::new());
        }

        let accounts = self
            .rpc_client
            .get_multiple_accounts(tables)
            .await
            .map_err(EngineError::Rpc)?;

        let mut resolved = Vec::with_capacity(tables.len());
        for (idx, maybe_account) in accounts.into_iter().enumerate() {
            let address = tables[idx];
            let account = maybe_account.ok_or_else(|| {
                EngineError::InvalidConfig(format!(
                    "pure_blind_strategy.overrides `{route_label}` lookup table `{}` 不存在",
                    address
                ))
            })?;

            if account.owner != LOOKUP_TABLE_PROGRAM_ID {
                return Err(EngineError::InvalidConfig(format!(
                    "pure_blind_strategy.overrides `{route_label}` lookup table `{}` 的程序并非 ALT",
                    address
                )));
            }

            let table = AddressLookupTable::deserialize(&account.data).map_err(|err| {
                EngineError::InvalidConfig(format!(
                    "pure_blind_strategy.overrides `{route_label}` lookup table `{}` 解析失败: {err}",
                    address
                ))
            })?;

            if table.meta.deactivation_slot != u64::MAX {
                return Err(EngineError::InvalidConfig(format!(
                    "pure_blind_strategy.overrides `{route_label}` lookup table `{}` 已失效",
                    address
                )));
            }

            resolved.push(AddressLookupTableAccount {
                key: address,
                addresses: table.addresses.into_owned(),
            });
        }

        Ok(resolved)
    }

    fn assemble_route_plan(
        &self,
        label: String,
        resolved: Vec<ResolvedMarketMeta>,
        lookup_tables: Vec<AddressLookupTableAccount>,
        base_mints: &[Pubkey],
        preferred_base: Option<Pubkey>,
        source: RouteSource,
        explicit_min_profit: Option<u64>,
    ) -> EngineResult<BlindRoutePlan> {
        let forward = Self::build_closed_loop(&resolved)
            .ok_or_else(|| {
                EngineError::InvalidConfig(format!(
                    "纯盲发路由 `{label}` 无法推导闭环资产流，请检查市场顺序"
                ))
            })
            .and_then(|steps| {
                Self::align_with_base_mints(steps, base_mints, preferred_base, &label)
            })?;
        let reverse = Self::build_reverse_steps(&forward);

        let inferred_base = forward.first().map(|step| step.input.mint);
        let min_profit = explicit_min_profit
            .or_else(|| preferred_base.and_then(|mint| self.base_min_profit(&mint)))
            .or_else(|| inferred_base.and_then(|mint| self.base_min_profit(&mint)))
            .unwrap_or(1)
            .max(1);

        let plan = BlindRoutePlan {
            forward,
            reverse,
            lookup_tables,
            label: label.clone(),
            source,
            min_profit,
        };

        events::pure_blind_route_registered(
            plan.label(),
            plan.forward.len(),
            plan.source().as_str(),
        );

        Ok(plan)
    }

    fn base_min_profit(&self, mint: &Pubkey) -> Option<u64> {
        self.config
            .assets
            .base_mints
            .iter()
            .filter_map(|base| {
                let trimmed = base.mint.trim();
                if trimmed.is_empty() {
                    return None;
                }
                Pubkey::from_str(trimmed)
                    .ok()
                    .filter(|parsed| parsed == mint)
                    .map(|_| base.min_profit.unwrap_or(1).max(1))
            })
            .next()
    }

    fn parse_base_mints(&self) -> EngineResult<Vec<Pubkey>> {
        let mut mints = Vec::with_capacity(self.config.assets.base_mints.len());

        for (idx, base) in self.config.assets.base_mints.iter().enumerate() {
            let mint_text = base.mint.trim();
            if mint_text.is_empty() {
                continue;
            }

            let mint = Pubkey::from_str(mint_text).map_err(|err| {
                EngineError::InvalidConfig(format!(
                    "pure_blind_strategy.assets.base_mints[{idx}] mint `{mint_text}` 解析失败: {err}"
                ))
            })?;

            if let Some(route_type) = base.route_type.as_deref().map(|value| value.trim()) {
                if !route_type.is_empty() && route_type != "2hop" && route_type != "3hop" {
                    return Err(EngineError::InvalidConfig(format!(
                        "pure_blind_strategy.assets.base_mints[{idx}] route_type `{route_type}` 无效，仅支持 2hop / 3hop"
                    )));
                }
            }

            mints.push(mint);
        }

        if mints.is_empty() {
            return Err(EngineError::InvalidConfig(
                "纯盲发模式需要至少一个有效的 pure_blind_strategy.assets.base_mints 配置".into(),
            ));
        }

        Ok(mints)
    }

    fn build_closed_loop(resolved: &[ResolvedMarketMeta]) -> Option<Vec<BlindStep>> {
        if resolved.len() < 2 {
            return None;
        }

        Self::try_orientation(resolved, SwapFlow::BaseToQuote)
            .or_else(|| Self::try_orientation(resolved, SwapFlow::QuoteToBase))
    }

    fn try_orientation(
        resolved: &[ResolvedMarketMeta],
        first_flow: SwapFlow,
    ) -> Option<Vec<BlindStep>> {
        let mut steps = Vec::with_capacity(resolved.len());

        let first = resolved.first()?;
        let (first_input, first_output) = Self::assets_for(first, first_flow);
        let mut current_asset = first_input.clone();
        let origin_asset = current_asset.clone();

        steps.push(BlindStep {
            dex: first.dex,
            market: first.market,
            base: first.base_asset.clone(),
            quote: first.quote_asset.clone(),
            input: first_input.clone(),
            output: first_output.clone(),
            meta: first.meta.clone(),
            flow: first_flow,
        });

        current_asset = first_output;

        for meta in resolved.iter().skip(1) {
            let flow = if current_asset == meta.base_asset {
                SwapFlow::BaseToQuote
            } else if current_asset == meta.quote_asset {
                SwapFlow::QuoteToBase
            } else {
                return None;
            };

            let (input_asset, output_asset) = Self::assets_for(meta, flow);
            if input_asset != current_asset {
                return None;
            }

            steps.push(BlindStep {
                dex: meta.dex,
                market: meta.market,
                base: meta.base_asset.clone(),
                quote: meta.quote_asset.clone(),
                input: input_asset.clone(),
                output: output_asset.clone(),
                meta: meta.meta.clone(),
                flow,
            });

            current_asset = output_asset;
        }

        if current_asset != origin_asset {
            return None;
        }

        Some(steps)
    }

    fn build_reverse_steps(forward: &[BlindStep]) -> Vec<BlindStep> {
        forward
            .iter()
            .rev()
            .map(|step| BlindStep {
                dex: step.dex,
                market: step.market,
                base: step.base.clone(),
                quote: step.quote.clone(),
                input: step.output.clone(),
                output: step.input.clone(),
                meta: step.meta.clone(),
                flow: match step.flow {
                    SwapFlow::BaseToQuote => SwapFlow::QuoteToBase,
                    SwapFlow::QuoteToBase => SwapFlow::BaseToQuote,
                },
            })
            .collect()
    }

    fn assets_for(meta: &ResolvedMarketMeta, flow: SwapFlow) -> (BlindAsset, BlindAsset) {
        match flow {
            SwapFlow::BaseToQuote => (meta.base_asset.clone(), meta.quote_asset.clone()),
            SwapFlow::QuoteToBase => (meta.quote_asset.clone(), meta.base_asset.clone()),
        }
    }

    async fn resolve_market_meta(
        &self,
        market: Pubkey,
        account: &Account,
    ) -> EngineResult<ResolvedMarketMeta> {
        if account.owner == ZEROFI_PROGRAM_ID {
            let adapter = ZeroFiAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("ZeroFi 市场 {market} 解码失败: {err}"))
                })?;
            return Ok(ResolvedMarketMeta {
                dex: BlindDex::ZeroFi,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::ZeroFi(meta),
            });
        }

        if account.owner == SAROS_PROGRAM_ID {
            let adapter = SarosAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("Saros 市场 {market} 解码失败: {err}"))
                })?;
            return Ok(ResolvedMarketMeta {
                dex: BlindDex::Saros,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::Saros(meta),
            });
        }

        if account.owner == SOLFI_V2_PROGRAM_ID {
            let adapter = SolFiV2Adapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("SolFiV2 市场 {market} 解码失败: {err}"))
                })?;
            return Ok(ResolvedMarketMeta {
                dex: BlindDex::SolFiV2,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::SolFiV2(meta),
            });
        }

        if account.owner == TESSERA_V_PROGRAM_ID {
            let adapter = TesseraVAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("TesseraV 市场 {market} 解码失败: {err}"))
                })?;
            return Ok(ResolvedMarketMeta {
                dex: BlindDex::TesseraV,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::TesseraV(meta),
            });
        }

        if account.owner == HUMIDIFI_PROGRAM_ID {
            let adapter = HumidiFiAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("HumidiFi 市场 {market} 解码失败: {err}"))
                })?;

            return Ok(ResolvedMarketMeta {
                dex: BlindDex::HumidiFi,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::HumidiFi(meta),
            });
        }

        if account.owner == OBRIC_V2_PROGRAM_ID {
            let adapter = ObricV2Adapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("ObricV2 市场 {market} 解码失败: {err}"))
                })?;

            return Ok(ResolvedMarketMeta {
                dex: BlindDex::ObricV2,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::ObricV2(meta),
            });
        }

        if account.owner == RAYDIUM_CLMM_PROGRAM_ID {
            let adapter = RaydiumClmmAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("RaydiumClmm 市场 {market} 解码失败: {err}"))
                })?;

            return Ok(ResolvedMarketMeta {
                dex: BlindDex::RaydiumClmm,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::RaydiumClmm(meta),
            });
        }

        if account.owner == METEORA_DLMM_PROGRAM_ID {
            let adapter = MeteoraDlmmAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("MeteoraDlmm 市场 {market} 解码失败: {err}"))
                })?;

            return Ok(ResolvedMarketMeta {
                dex: BlindDex::MeteoraDlmm,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::MeteoraDlmm(meta),
            });
        }

        if account.owner == ORCA_WHIRLPOOL_PROGRAM_ID {
            let adapter = WhirlpoolAdapter::shared();
            let meta = adapter
                .fetch_market_meta(self.rpc_client, market, account)
                .await
                .map_err(|err| {
                    EngineError::InvalidConfig(format!("Whirlpool 市场 {market} 解码失败: {err}"))
                })?;

            return Ok(ResolvedMarketMeta {
                dex: BlindDex::Whirlpool,
                market,
                base_asset: BlindAsset::new(meta.base_mint(), meta.base_token_program()),
                quote_asset: BlindAsset::new(meta.quote_mint(), meta.quote_token_program()),
                meta: BlindMarketMeta::Whirlpool(meta),
            });
        }

        Err(EngineError::InvalidConfig(format!(
            "纯盲发暂不支持程序 {}",
            account.owner
        )))
    }

    fn align_with_base_mints(
        steps: Vec<BlindStep>,
        base_mints: &[Pubkey],
        preferred_base: Option<Pubkey>,
        route_label: &str,
    ) -> EngineResult<Vec<BlindStep>> {
        if steps.is_empty() {
            return Err(EngineError::InvalidConfig(format!(
                "pure_blind_strategy.overrides `{route_label}` 未生成任何盲发步骤"
            )));
        }

        if let Some(preferred) = preferred_base {
            if let Some(idx) = steps.iter().position(|step| step.input.mint == preferred) {
                if idx == 0 {
                    return Ok(steps);
                }
                let mut rotated = Vec::with_capacity(steps.len());
                rotated.extend(steps[idx..].iter().cloned());
                rotated.extend(steps[..idx].iter().cloned());
                return Ok(rotated);
            }
        }

        let position = steps.iter().position(|step| {
            base_mints
                .iter()
                .any(|candidate| step.input.mint == *candidate)
        });

        let Some(idx) = position else {
            let allowed = base_mints
                .iter()
                .map(|mint| mint.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            let available = steps
                .iter()
                .map(|step| step.input.mint.to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(EngineError::InvalidConfig(format!(
                "pure_blind_strategy.overrides `{route_label}` 无法匹配 pure_blind_strategy.assets.base_mints，允许: [{allowed}]，可用闭环顺序: [{available}]"
            )));
        };

        if idx == 0 {
            return Ok(steps);
        }

        let mut rotated = Vec::with_capacity(steps.len());
        rotated.extend(steps[idx..].iter().cloned());
        rotated.extend(steps[..idx].iter().cloned());
        Ok(rotated)
    }
}

#[derive(Clone)]
struct ResolvedMarketMeta {
    dex: BlindDex,
    market: Pubkey,
    base_asset: BlindAsset,
    quote_asset: BlindAsset,
    meta: BlindMarketMeta,
}

/// 纯盲发策略：不依赖报价，直接构造 route_v2 指令。
pub struct PureBlindStrategy {
    routes: Vec<BlindRoutePlan>,
    _pool_catalog: Arc<PoolCatalog>,
    route_catalog: Arc<RouteCatalog>,
    dynamic_rx: UnboundedReceiver<DynamicRouteUpdate>,
    dynamic_routes: HashMap<RouteKey, DynamicRoute>,
    base_min_profit: HashMap<Pubkey, u64>,
}

impl PureBlindStrategy {
    pub fn new(
        routes: Vec<BlindRoutePlan>,
        config: &config::PureBlindStrategyConfig,
        pool_catalog: Arc<PoolCatalog>,
        route_catalog: Arc<RouteCatalog>,
        dynamic_rx: UnboundedReceiver<DynamicRouteUpdate>,
    ) -> Result<Self> {
        if routes.is_empty() && config.observer.as_ref().map_or(true, |cfg| !cfg.enable) {
            bail!("纯盲发模式需要至少一个盲发路由或启用观测器");
        }

        let base_min_profit = build_min_profit_map(&config.assets.base_mints)?;

        Ok(Self {
            routes,
            _pool_catalog: pool_catalog,
            route_catalog,
            dynamic_rx,
            dynamic_routes: HashMap::new(),
            base_min_profit,
        })
    }
}

impl Strategy for PureBlindStrategy {
    type Event = StrategyEvent;

    fn name(&self) -> &'static str {
        "pure_blind"
    }

    fn on_market_event(
        &mut self,
        event: &Self::Event,
        mut ctx: StrategyContext<'_>,
    ) -> StrategyDecision {
        match event {
            StrategyEvent::Tick(_) => {
                self.poll_dynamic_updates();
                self.refresh_dynamic_scores();

                let mut batch: Vec<BlindOrder> = Vec::new();

                for route in &self.routes {
                    if let Some(first_step) = route.forward.first() {
                        if let Some(amounts) = ctx.take_amounts(&first_step.input.mint) {
                            if amounts.is_empty() {
                                continue;
                            }
                            for &amount in &amounts {
                                let min_profit = route.min_profit();
                                batch.push(BlindOrder {
                                    amount_in: amount,
                                    steps: route.forward.clone(),
                                    lookup_tables: route.lookup_tables.clone(),
                                    min_profit,
                                });
                                batch.push(BlindOrder {
                                    amount_in: amount,
                                    steps: route.reverse.clone(),
                                    lookup_tables: route.lookup_tables.clone(),
                                    min_profit,
                                });
                            }

                            let count = amounts.len();
                            events::pure_blind_orders_prepared(
                                route.label(),
                                "forward",
                                route.source().as_str(),
                                count,
                            );
                            events::pure_blind_orders_prepared(
                                route.label(),
                                "reverse",
                                route.source().as_str(),
                                count,
                            );
                        }
                    }
                }

                let mut dynamic_entries: Vec<&DynamicRoute> =
                    self.dynamic_routes.values().collect();
                dynamic_entries
                    .sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));

                for route in dynamic_entries {
                    if route.steps.is_empty() {
                        continue;
                    }
                    let Some(first_step) = route.steps.first() else {
                        continue;
                    };
                    if let Some(amounts) = ctx.take_amounts(&first_step.input.mint) {
                        if amounts.is_empty() {
                            continue;
                        }
                        let reverse_steps = reverse_steps(&route.steps);
                        for &amount in &amounts {
                            batch.push(BlindOrder {
                                amount_in: amount,
                                steps: route.steps.clone(),
                                lookup_tables: route.lookup_tables.clone(),
                                min_profit: route.min_profit,
                            });
                            if !reverse_steps.is_empty() {
                                batch.push(BlindOrder {
                                    amount_in: amount,
                                    steps: reverse_steps.clone(),
                                    lookup_tables: route.lookup_tables.clone(),
                                    min_profit: route.min_profit,
                                });
                            }
                        }

                        let route_label = route
                            .profile
                            .markets()
                            .iter()
                            .map(|market| market.to_string())
                            .collect::<Vec<_>>()
                            .join("->");
                        let source_label = "dynamic";
                        let count = amounts.len();
                        events::pure_blind_orders_prepared(
                            &route_label,
                            "forward",
                            source_label,
                            count,
                        );
                        if !reverse_steps.is_empty() {
                            events::pure_blind_orders_prepared(
                                &route_label,
                                "reverse",
                                source_label,
                                count,
                            );
                        }
                    }
                }

                if batch.is_empty() {
                    return ctx.into_decision();
                }

                let mut rng = rand::rng();
                batch.shuffle(&mut rng);

                StrategyDecision {
                    action: Action::DispatchBlind(batch),
                    next_ready_in: None,
                }
            }
        }
    }
}

fn build_min_profit_map(bases: &[config::PureBlindBaseMintConfig]) -> Result<HashMap<Pubkey, u64>> {
    let mut map = HashMap::new();
    for (idx, base) in bases.iter().enumerate() {
        let mint_text = base.mint.trim();
        if mint_text.is_empty() {
            continue;
        }
        let mint = Pubkey::from_str(mint_text).map_err(|err| {
            anyhow!(
                "pure_blind_strategy.assets.base_mints[{idx}] mint `{mint_text}` 解析失败: {err}"
            )
        })?;
        let min_profit = base.min_profit.unwrap_or(1).max(1);
        map.insert(mint, min_profit);
    }
    Ok(map)
}

fn reverse_steps(steps: &[BlindStep]) -> Vec<BlindStep> {
    steps
        .iter()
        .rev()
        .map(|step| BlindStep {
            dex: step.dex,
            market: step.market,
            base: step.base.clone(),
            quote: step.quote.clone(),
            input: step.output.clone(),
            output: step.input.clone(),
            meta: step.meta.clone(),
            flow: match step.flow {
                SwapFlow::BaseToQuote => SwapFlow::QuoteToBase,
                SwapFlow::QuoteToBase => SwapFlow::BaseToQuote,
            },
        })
        .collect()
}

struct DynamicRoute {
    profile: Arc<RouteProfile>,
    stats: RouteStatsSnapshot,
    steps: Vec<BlindStep>,
    lookup_tables: Vec<AddressLookupTableAccount>,
    min_profit: u64,
    score: f64,
}

impl PureBlindStrategy {
    fn poll_dynamic_updates(&mut self) {
        while let Ok(update) = self.dynamic_rx.try_recv() {
            match update {
                DynamicRouteUpdate::Activated {
                    profile,
                    stats,
                    steps,
                    lookup_tables,
                } => {
                    if steps.is_empty() {
                        continue;
                    }
                    let min_profit = profile
                        .base_asset
                        .and_then(|asset| self.base_min_profit.get(&asset.mint).copied())
                        .or_else(|| {
                            steps.first().and_then(|step| {
                                self.base_min_profit.get(&step.input.mint).copied()
                            })
                        })
                        .unwrap_or(1);
                    let key = profile.key.clone();
                    let route = DynamicRoute {
                        profile,
                        stats,
                        steps,
                        lookup_tables,
                        min_profit,
                        score: 0.0,
                    };
                    self.dynamic_routes.insert(key, route);
                }
                DynamicRouteUpdate::Retired { key, .. } => {
                    self.dynamic_routes.remove(&key);
                }
            }
        }
    }

    fn refresh_dynamic_scores(&mut self) {
        if self.dynamic_routes.is_empty() {
            return;
        }

        let active = self.route_catalog.active_routes();
        for entry in active {
            if let Some(route) = self.dynamic_routes.get_mut(&entry.profile.key) {
                route.score = entry.score;
                route.stats = entry.stats;
            }
        }
    }
}
