use std::collections::{HashMap, HashSet};
use std::str::FromStr;

use anyhow::{Result, bail};
use rand::seq::SliceRandom;
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{account::Account, message::AddressLookupTableAccount, pubkey::Pubkey};

use crate::config;
use crate::dexes::{
    clmm::{RAYDIUM_CLMM_PROGRAM_ID, RaydiumClmmAdapter},
    dlmm::{METEORA_DLMM_PROGRAM_ID, MeteoraDlmmAdapter},
    framework::{DexMarketMeta, DexMetaProvider, SwapFlow},
    humidifi::{HUMIDIFI_PROGRAM_ID, HumidiFiAdapter},
    obric_v2::{OBRIC_V2_PROGRAM_ID, ObricV2Adapter},
    solfi_v2::{SOLFI_V2_PROGRAM_ID, SolFiV2Adapter},
    tessera_v::{TESSERA_V_PROGRAM_ID, TesseraVAdapter},
    whirlpool::{ORCA_WHIRLPOOL_PROGRAM_ID, WhirlpoolAdapter},
    zerofi::{ZEROFI_PROGRAM_ID, ZeroFiAdapter},
};
use crate::engine::{Action, EngineError, EngineResult, StrategyContext, StrategyDecision};
use crate::monitoring::events;
use crate::pure_blind::market_cache::{MarketCacheHandle, MarketRecord};

use super::types::{
    BlindAsset, BlindDex, BlindMarketMeta, BlindOrder, BlindRoutePlan, BlindStep, RouteSource,
};
use super::{Strategy, StrategyEvent};
use tracing::{info, warn};

/// 纯盲发路由构建器：按配置解析盲发市场并生成双向路由。
pub struct PureBlindRouteBuilder<'a> {
    config: &'a config::PureBlindStrategyConfig,
    rpc_client: &'a RpcClient,
    market_cache: &'a MarketCacheHandle,
}

const LOOKUP_TABLE_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("AddressLookupTab1e1111111111111111111111111");

const REQUIRED_MARKETS_PER_PAIR: usize = 2;
const MAX_CANDIDATES_PER_PAIR: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RouteType {
    TwoHop,
    ThreeHop,
}

struct BaseMintEntry<'a> {
    config: &'a config::PureBlindBaseMintConfig,
    mint: Pubkey,
    route_type: RouteType,
    min_profit: u64,
}

struct IntermediateEntry {
    mint: Pubkey,
    text: String,
}

struct AutoRouteSpec<'a> {
    base: &'a BaseMintEntry<'a>,
    intermediate: &'a IntermediateEntry,
    pair_key: PairKey,
}

#[derive(Clone)]
struct AutoMarketCandidate {
    market: Pubkey,
    lookup_table: Option<Pubkey>,
    meta: ResolvedMarketMeta,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct PairKey {
    a: Pubkey,
    b: Pubkey,
}

impl PairKey {
    fn new(x: Pubkey, y: Pubkey) -> Self {
        if x <= y {
            Self { a: x, b: y }
        } else {
            Self { a: y, b: x }
        }
    }
}

impl<'a> PureBlindRouteBuilder<'a> {
    pub fn new(
        config: &'a config::PureBlindStrategyConfig,
        rpc_client: &'a RpcClient,
        market_cache: &'a MarketCacheHandle,
    ) -> Self {
        Self {
            config,
            rpc_client,
            market_cache,
        }
    }

    pub async fn build(&self) -> EngineResult<Vec<BlindRoutePlan>> {
        let base_entries = self.parse_base_entries()?;
        let base_mints: Vec<Pubkey> = base_entries.iter().map(|entry| entry.mint).collect();

        let (manual_market_set, mut dedup_route_keys) = self.collect_manual_route_sets();

        let mut plans = self.build_manual_routes(&base_mints).await?;

        let auto_routes = self
            .build_auto_routes(
                &base_entries,
                &base_mints,
                &manual_market_set,
                &mut dedup_route_keys,
            )
            .await?;

        plans.extend(auto_routes);

        if plans.is_empty() {
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

    fn collect_manual_route_sets(&self) -> (HashSet<Pubkey>, HashSet<String>) {
        let mut markets: HashSet<Pubkey> = HashSet::new();
        let mut route_keys: HashSet<String> = HashSet::new();

        for route in &self.config.overrides {
            let mut per_route: Vec<Pubkey> = Vec::with_capacity(route.legs.len());
            for leg in &route.legs {
                let trimmed = leg.market.trim();
                if trimmed.is_empty() {
                    continue;
                }
                if let Ok(market) = Pubkey::from_str(trimmed) {
                    markets.insert(market);
                    per_route.push(market);
                }
            }
            if !per_route.is_empty() {
                route_keys.insert(Self::canonical_market_key(&per_route));
            }
        }

        (markets, route_keys)
    }

    fn canonical_market_key(markets: &[Pubkey]) -> String {
        let mut values: Vec<String> = markets.iter().map(|m| m.to_string()).collect();
        values.sort();
        values.join("|")
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

    fn parse_base_entries(&self) -> EngineResult<Vec<BaseMintEntry<'_>>> {
        let mut entries = Vec::with_capacity(self.config.assets.base_mints.len());

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

            let route_type = match base
                .route_type
                .as_deref()
                .map(|value| value.trim().to_ascii_lowercase())
            {
                Some(value) if value == "3hop" => RouteType::ThreeHop,
                Some(value) if value.is_empty() || value == "2hop" => RouteType::TwoHop,
                None => RouteType::TwoHop,
                Some(other) => {
                    return Err(EngineError::InvalidConfig(format!(
                        "pure_blind_strategy.assets.base_mints[{idx}] route_type `{other}` 无效，仅支持 2hop / 3hop"
                    )));
                }
            };

            let min_profit = base.min_profit.unwrap_or(1).max(1);

            entries.push(BaseMintEntry {
                config: base,
                mint,
                route_type,
                min_profit,
            });
        }

        if entries.is_empty() {
            return Err(EngineError::InvalidConfig(
                "纯盲发模式需要至少一个有效的 pure_blind_strategy.assets.base_mints 配置".into(),
            ));
        }

        Ok(entries)
    }

    fn parse_intermediate_entries(&self) -> EngineResult<Vec<IntermediateEntry>> {
        let blacklist: HashSet<String> = self
            .config
            .assets
            .blacklist_mints
            .iter()
            .map(|mint| mint.trim().to_ascii_lowercase())
            .filter(|mint| !mint.is_empty())
            .collect();

        let mut entries = Vec::with_capacity(self.config.assets.intermediates.len());

        for (idx, value) in self.config.assets.intermediates.iter().enumerate() {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                continue;
            }
            if blacklist.contains(&trimmed.to_ascii_lowercase()) {
                continue;
            }

            let mint = Pubkey::from_str(trimmed).map_err(|err| {
                EngineError::InvalidConfig(format!(
                    "pure_blind_strategy.assets.intermediates[{idx}] mint `{trimmed}` 解析失败: {err}"
                ))
            })?;

            entries.push(IntermediateEntry {
                mint,
                text: trimmed.to_string(),
            });
        }

        if entries.is_empty() {
            return Err(EngineError::InvalidConfig(
                "pure_blind_strategy.assets.intermediates 不能为空".into(),
            ));
        }

        Ok(entries)
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    async fn build_auto_routes(
        &self,
        base_entries: &[BaseMintEntry<'_>],
        base_mints: &[Pubkey],
        manual_markets: &HashSet<Pubkey>,
        dedup_route_keys: &mut HashSet<String>,
    ) -> EngineResult<Vec<BlindRoutePlan>> {
        let intermediates = self.parse_intermediate_entries()?;
        let two_hop_bases: Vec<&BaseMintEntry<'_>> = base_entries
            .iter()
            .filter(|entry| matches!(entry.route_type, RouteType::TwoHop))
            .collect();
        let three_hop_bases: Vec<&BaseMintEntry<'_>> = base_entries
            .iter()
            .filter(|entry| matches!(entry.route_type, RouteType::ThreeHop))
            .collect();

        if two_hop_bases.is_empty() && three_hop_bases.is_empty() {
            return Ok(Vec::new());
        }

        // 如果只有 3hop 请求，这里不要提前返回，后面仍需依赖 intermediates

        let mut specs = Vec::new();
        for base in &two_hop_bases {
            for intermediate in &intermediates {
                if intermediate.mint == base.mint {
                    continue;
                }
                specs.push(AutoRouteSpec {
                    base,
                    intermediate,
                    pair_key: PairKey::new(base.mint, intermediate.mint),
                });
            }
        }

        let mut required_pairs: HashSet<PairKey> = HashSet::new();
        for spec in &specs {
            required_pairs.insert(spec.pair_key);
        }
        for base in &three_hop_bases {
            for intermediate in &intermediates {
                if intermediate.mint == base.mint {
                    continue;
                }
                required_pairs.insert(PairKey::new(base.mint, intermediate.mint));
            }
        }
        if !three_hop_bases.is_empty() {
            for idx in 0..intermediates.len() {
                for jdx in (idx + 1)..intermediates.len() {
                    let first = &intermediates[idx];
                    let second = &intermediates[jdx];
                    if first.mint == second.mint {
                        continue;
                    }
                    required_pairs.insert(PairKey::new(first.mint, second.mint));
                }
            }
        }

        let snapshot = match self.market_cache.try_snapshot() {
            Some(records) => records,
            None => {
                warn!(
                    target: "strategy::pure_blind",
                    "市场缓存尚未就绪，自动路由生成已跳过"
                );
                return Ok(Vec::new());
            }
        };

        let mut required_tokens: HashSet<Pubkey> = HashSet::new();
        for base in &two_hop_bases {
            required_tokens.insert(base.mint);
        }
        for base in &three_hop_bases {
            required_tokens.insert(base.mint);
        }
        for intermediate in &intermediates {
            required_tokens.insert(intermediate.mint);
        }

        let mut pair_candidates: HashMap<PairKey, Vec<AutoMarketCandidate>> = HashMap::new();
        let mut meta_cache: HashMap<Pubkey, ResolvedMarketMeta> = HashMap::new();

        const MARKET_BATCH: usize = 50;
        let mut candidate_batch: Vec<MarketRecord> = Vec::with_capacity(MARKET_BATCH);

        for record in snapshot {
            if Self::all_pairs_satisfied(&pair_candidates, &required_pairs) {
                break;
            }

            if manual_markets.contains(&record.market) {
                continue;
            }

            if let Some(group) = record.routing_group {
                if group > 3 {
                    continue;
                }
            }

            if !record.token_mints.is_empty()
                && !record
                    .token_mints
                    .iter()
                    .any(|mint| required_tokens.contains(mint))
            {
                continue;
            }

            candidate_batch.push(record);

            if candidate_batch.len() >= MARKET_BATCH {
                self.ingest_market_batch(
                    &candidate_batch,
                    &mut meta_cache,
                    &mut pair_candidates,
                    &required_pairs,
                )
                .await?;
                candidate_batch.clear();
            }
        }

        if !candidate_batch.is_empty()
            && !Self::all_pairs_satisfied(&pair_candidates, &required_pairs)
        {
            self.ingest_market_batch(
                &candidate_batch,
                &mut meta_cache,
                &mut pair_candidates,
                &required_pairs,
            )
            .await?;
        }

        let mut routes = Vec::new();

        for spec in specs {
            let Some(candidates) = pair_candidates.get(&spec.pair_key) else {
                warn!(
                    target: "strategy::pure_blind",
                    base = %spec.base.config.mint,
                    intermediate = %spec.intermediate.text,
                    "未找到满足条件的两腿市场，自动路线跳过"
                );
                continue;
            };

            if candidates.len() < REQUIRED_MARKETS_PER_PAIR {
                warn!(
                    target: "strategy::pure_blind",
                    base = %spec.base.config.mint,
                    intermediate = %spec.intermediate.text,
                    available = candidates.len(),
                    "候选市场数量不足以构造两腿闭环"
                );
                continue;
            }

            let mut built = false;

            'search: for i in 0..candidates.len() {
                for j in (i + 1)..candidates.len() {
                    let first = candidates[i].clone();
                    let second = candidates[j].clone();

                    let permutations = [vec![first.clone(), second.clone()], vec![second, first]];

                    for combo in permutations {
                        let markets: Vec<Pubkey> = combo.iter().map(|item| item.market).collect();
                        let route_key = Self::canonical_market_key(&markets);
                        if dedup_route_keys.contains(&route_key) {
                            continue;
                        }

                        let mut lookup_tables: HashSet<Pubkey> = HashSet::new();
                        let resolved: Vec<ResolvedMarketMeta> = combo
                            .iter()
                            .map(|candidate| {
                                if let Some(table) = candidate.lookup_table {
                                    lookup_tables.insert(table);
                                }
                                candidate.meta.clone()
                            })
                            .collect();

                        let suffix: String =
                            route_key.chars().filter(|ch| *ch != '|').take(16).collect();
                        let label = format!(
                            "auto_{}_{}_{}",
                            spec.base.config.mint.trim(),
                            spec.intermediate.text,
                            suffix
                        );

                        let lookup_tables = self
                            .fetch_lookup_tables(
                                &lookup_tables.into_iter().collect::<Vec<_>>(),
                                &label,
                            )
                            .await?;

                        match self.assemble_route_plan(
                            label,
                            resolved,
                            lookup_tables,
                            base_mints,
                            Some(spec.base.mint),
                            RouteSource::Auto,
                            Some(spec.base.min_profit),
                        ) {
                            Ok(plan) => {
                                dedup_route_keys.insert(route_key);
                                info!(
                                    target: "strategy::pure_blind",
                                    route = plan.label(),
                                    legs = plan.forward.len(),
                                    source = plan.source().as_str(),
                                    "新增自动纯盲发闭环"
                                );
                                routes.push(plan);
                                built = true;
                                break 'search;
                            }
                            Err(err) => {
                                warn!(
                                    target: "strategy::pure_blind",
                                    base = %spec.base.config.mint,
                                    intermediate = %spec.intermediate.text,
                                    markets = %markets
                                        .iter()
                                        .map(|m| m.to_string())
                                        .collect::<Vec<_>>()
                                        .join(","),
                                    error = %err,
                                    "自动闭环构建失败，尝试下一个组合"
                                );
                            }
                        }
                    }
                }
            }

            if !built {
                warn!(
                    target: "strategy::pure_blind",
                    base = %spec.base.config.mint,
                    intermediate = %spec.intermediate.text,
                    "未找到符合要求的闭环组合，已跳过"
                );
            }
        }

        if !three_hop_bases.is_empty() && intermediates.len() < 2 {
            warn!(
                target: "strategy::pure_blind",
                "纯盲配置请求 3hop 路线，但 intermediates 不足 2 个，已跳过"
            );
        }

        for base in &three_hop_bases {
            if intermediates.len() < 2 {
                continue;
            }

            for idx in 0..intermediates.len() {
                let first = &intermediates[idx];
                if first.mint == base.mint {
                    continue;
                }
                for jdx in (idx + 1)..intermediates.len() {
                    let second = &intermediates[jdx];
                    if second.mint == base.mint {
                        continue;
                    }

                    let orders = [(first, second), (second, first)];
                    for (primary, secondary) in orders {
                        if primary.mint == secondary.mint {
                            continue;
                        }
                        if let Some(plan) = self
                            .build_three_hop_route(
                                base,
                                primary,
                                secondary,
                                &pair_candidates,
                                base_mints,
                                dedup_route_keys,
                            )
                            .await?
                        {
                            routes.push(plan);
                            break;
                        }
                    }
                }
            }
        }

        Ok(routes)
    }

    async fn build_three_hop_route(
        &self,
        base: &BaseMintEntry<'_>,
        first: &IntermediateEntry,
        second: &IntermediateEntry,
        pair_candidates: &HashMap<PairKey, Vec<AutoMarketCandidate>>,
        base_mints: &[Pubkey],
        dedup_route_keys: &mut HashSet<String>,
    ) -> EngineResult<Option<BlindRoutePlan>> {
        let pair_first = PairKey::new(base.mint, first.mint);
        let pair_mid = PairKey::new(first.mint, second.mint);
        let pair_last = PairKey::new(second.mint, base.mint);

        let Some(first_candidates) = pair_candidates.get(&pair_first) else {
            warn!(
                target: "strategy::pure_blind",
                base = %base.config.mint,
                mid1 = %first.text,
                "三跳路线缺少 base↔mid1 市场"
            );
            return Ok(None);
        };

        let Some(mid_candidates) = pair_candidates.get(&pair_mid) else {
            warn!(
                target: "strategy::pure_blind",
                mid1 = %first.text,
                mid2 = %second.text,
                "三跳路线缺少 mid1↔mid2 市场"
            );
            return Ok(None);
        };

        let Some(last_candidates) = pair_candidates.get(&pair_last) else {
            warn!(
                target: "strategy::pure_blind",
                mid2 = %second.text,
                base = %base.config.mint,
                "三跳路线缺少 mid2↔base 市场"
            );
            return Ok(None);
        };

        for cand_first in first_candidates.iter().take(MAX_CANDIDATES_PER_PAIR) {
            for cand_mid in mid_candidates.iter().take(MAX_CANDIDATES_PER_PAIR) {
                for cand_last in last_candidates.iter().take(MAX_CANDIDATES_PER_PAIR) {
                    if cand_first.market == cand_mid.market
                        || cand_first.market == cand_last.market
                        || cand_mid.market == cand_last.market
                    {
                        continue;
                    }

                    let combo = [cand_first.clone(), cand_mid.clone(), cand_last.clone()];
                    let markets: Vec<Pubkey> = combo.iter().map(|item| item.market).collect();
                    let route_key = Self::canonical_market_key(&markets);
                    if dedup_route_keys.contains(&route_key) {
                        continue;
                    }

                    let mut lookup_tables: HashSet<Pubkey> = HashSet::new();
                    let resolved: Vec<ResolvedMarketMeta> = combo
                        .iter()
                        .map(|candidate| {
                            if let Some(table) = candidate.lookup_table {
                                lookup_tables.insert(table);
                            }
                            candidate.meta.clone()
                        })
                        .collect();

                    let suffix: String =
                        route_key.chars().filter(|ch| *ch != '|').take(16).collect();
                    let label = format!(
                        "auto3_{}_{}_{}_{}",
                        base.config.mint.trim(),
                        first.text,
                        second.text,
                        suffix
                    );

                    let lookup_tables = self
                        .fetch_lookup_tables(&lookup_tables.into_iter().collect::<Vec<_>>(), &label)
                        .await?;

                    match self.assemble_route_plan(
                        label,
                        resolved,
                        lookup_tables,
                        base_mints,
                        Some(base.mint),
                        RouteSource::Auto,
                        Some(base.min_profit),
                    ) {
                        Ok(plan) => {
                            dedup_route_keys.insert(route_key);
                            info!(
                                target: "strategy::pure_blind",
                                route = plan.label(),
                                legs = plan.forward.len(),
                                source = plan.source().as_str(),
                                "新增自动三跳纯盲发闭环"
                            );
                            return Ok(Some(plan));
                        }
                        Err(err) => {
                            warn!(
                                target: "strategy::pure_blind",
                                base = %base.config.mint,
                                mid1 = %first.text,
                                mid2 = %second.text,
                                markets = %markets
                                    .iter()
                                    .map(|m| m.to_string())
                                    .collect::<Vec<_>>()
                                    .join(","),
                                error = %err,
                                "三跳闭环校验失败，尝试下一个组合"
                            );
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    fn all_pairs_satisfied(
        candidates: &HashMap<PairKey, Vec<AutoMarketCandidate>>,
        required: &HashSet<PairKey>,
    ) -> bool {
        required.iter().all(|key| {
            candidates
                .get(key)
                .map(|values| values.len() >= REQUIRED_MARKETS_PER_PAIR)
                .unwrap_or(false)
        })
    }

    async fn ingest_market_batch(
        &self,
        records: &[MarketRecord],
        cache: &mut HashMap<Pubkey, ResolvedMarketMeta>,
        pair_candidates: &mut HashMap<PairKey, Vec<AutoMarketCandidate>>,
        required_pairs: &HashSet<PairKey>,
    ) -> EngineResult<()> {
        if records.is_empty() {
            return Ok(());
        }

        let mut missing: Vec<Pubkey> = Vec::new();
        for record in records {
            if !cache.contains_key(&record.market) {
                missing.push(record.market);
            }
        }

        const ACCOUNT_BATCH_LIMIT: usize = 100;
        if !missing.is_empty() {
            for chunk in missing.chunks(ACCOUNT_BATCH_LIMIT) {
                match self.rpc_client.get_multiple_accounts(chunk).await {
                    Ok(accounts) => {
                        for (market, maybe_account) in
                            chunk.iter().copied().zip(accounts.into_iter())
                        {
                            let Some(account) = maybe_account else {
                                warn!(
                                    target: "strategy::pure_blind",
                                    market = %market,
                                    "无法获取市场账户，跳过"
                                );
                                continue;
                            };

                            match self.resolve_market_meta(market, &account).await {
                                Ok(meta) => {
                                    cache.insert(market, meta);
                                }
                                Err(err) => {
                                    warn!(
                                        target: "strategy::pure_blind",
                                        market = %market,
                                        error = %err,
                                        "解析市场元数据失败，跳过自动路由候选"
                                    );
                                }
                            }
                        }
                    }
                    Err(err) => {
                        warn!(
                            target: "strategy::pure_blind",
                            error = %err,
                            chunk = chunk.len(),
                            "批量获取市场账户失败，逐个回退"
                        );
                        for market in chunk.iter().copied() {
                            match self.rpc_client.get_account(&market).await {
                                Ok(account) => {
                                    match self.resolve_market_meta(market, &account).await {
                                        Ok(meta) => {
                                            cache.insert(market, meta);
                                        }
                                        Err(err) => {
                                            warn!(
                                                target: "strategy::pure_blind",
                                                market = %market,
                                                error = %err,
                                                "解析市场元数据失败，跳过自动路由候选"
                                            );
                                        }
                                    }
                                }
                                Err(err) => {
                                    warn!(
                                        target: "strategy::pure_blind",
                                        market = %market,
                                        error = %err,
                                        "获取市场账户失败，跳过"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        for record in records {
            let market = record.market;
            let Some(meta) = cache.get(&market) else {
                continue;
            };

            let pair_key = PairKey::new(meta.base_asset.mint, meta.quote_asset.mint);
            if !required_pairs.contains(&pair_key) {
                continue;
            }

            let entry = pair_candidates.entry(pair_key).or_default();
            if entry.iter().any(|candidate| candidate.market == market) {
                continue;
            }
            if entry.len() >= MAX_CANDIDATES_PER_PAIR {
                continue;
            }

            entry.push(AutoMarketCandidate {
                market,
                lookup_table: record.lookup_table,
                meta: meta.clone(),
            });
        }

        Ok(())
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
}

impl PureBlindStrategy {
    pub fn new(routes: Vec<BlindRoutePlan>) -> Result<Self> {
        if routes.is_empty() {
            bail!("纯盲发模式需要至少一个盲发路由");
        }

        Ok(Self { routes })
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
                let mut batch: Vec<BlindOrder> = Vec::new();

                for route in &self.routes {
                    if let Some(first_step) = route.forward.first() {
                        if let Some(ready) = ctx.take_amounts_if_ready(&first_step.input.mint) {
                            for &amount in &ready.amounts {
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

                            if !ready.amounts.is_empty() {
                                let count = ready.amounts.len();
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
                }

                if batch.is_empty() {
                    return ctx.into_decision();
                }

                let mut rng = rand::rng();
                batch.shuffle(&mut rng);

                StrategyDecision {
                    action: Action::DispatchBlind(batch),
                    next_ready_in: ctx.next_ready_delay(),
                }
            }
        }
    }
}
