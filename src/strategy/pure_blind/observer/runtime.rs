#![allow(dead_code)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use solana_sdk::message::VersionedMessage;
use solana_sdk::pubkey::Pubkey;
use tokio::task::JoinHandle;
use tracing::{info, warn};
use yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo;
use yellowstone_grpc_proto::solana::storage::confirmed_block;
use yellowstone_grpc_proto::tonic::metadata::AsciiMetadataValue;

use crate::instructions::jupiter::parser::PROGRAM_ID as JUPITER_PROGRAM_ID;
use crate::network::yellowstone::{YellowstoneTransactionClient, parse_transaction_update};
use crate::strategy::copy::transaction::{
    TokenBalanceEntry, TransactionLoadedAddresses, TransactionTokenBalances,
    decode_versioned_transaction, instructions_from_message,
};

use super::catalog::PoolCatalog;
use super::decoder::{DecodedJupiterStep, DirectionFlag, DirectionHint};
use super::profile::{PoolAsset, PoolKey, PoolObservation, PoolProfile};
use super::routes::{RouteCatalog, RouteObservation};
use crate::instructions::jupiter::decoder::ParsedSwapAccounts;
use crate::instructions::jupiter::types::resolve_swap_discriminant;

#[derive(Clone, Debug)]
pub struct PoolObserverSettings {
    pub endpoint: String,
    pub token: Option<AsciiMetadataValue>,
    pub wallets: Vec<Pubkey>,
}

pub struct PoolObserverHandle {
    tasks: Vec<JoinHandle<()>>,
}

impl PoolObserverHandle {
    pub fn new(tasks: Vec<JoinHandle<()>>) -> Self {
        Self { tasks }
    }

    pub async fn shutdown(self) {
        for task in self.tasks {
            task.abort();
        }
    }
}

pub async fn spawn_pool_observer(
    settings: PoolObserverSettings,
    pool_catalog: Arc<PoolCatalog>,
    route_catalog: Arc<RouteCatalog>,
) -> Result<PoolObserverHandle> {
    if settings.wallets.is_empty() {
        warn!(target: "pure_blind::observer", "配置未提供任何监听钱包，观察器未启动");
        return Ok(PoolObserverHandle::new(Vec::new()));
    }

    let mut tasks = Vec::with_capacity(settings.wallets.len());
    for wallet in &settings.wallets {
        let endpoint = settings.endpoint.clone();
        let token = settings.token.clone();
        let pool_catalog = Arc::clone(&pool_catalog);
        let route_catalog = Arc::clone(&route_catalog);
        let wallet = *wallet;
        let handle = tokio::spawn(async move {
            if let Err(err) =
                run_wallet_observer(endpoint, token, wallet, pool_catalog, route_catalog).await
            {
                warn!(
                    target: "pure_blind::observer",
                    wallet = %wallet,
                    error = %err,
                    "池子观察器任务退出"
                );
            }
        });
        tasks.push(handle);
    }

    Ok(PoolObserverHandle::new(tasks))
}

async fn run_wallet_observer(
    endpoint: String,
    token: Option<AsciiMetadataValue>,
    wallet: Pubkey,
    pool_catalog: Arc<PoolCatalog>,
    route_catalog: Arc<RouteCatalog>,
) -> Result<()> {
    let mut client = YellowstoneTransactionClient::connect(endpoint, token).await?;
    let mut stream = client
        .subscribe_transactions(wallet)
        .await
        .context("订阅 Yellowstone gRPC 失败")?;

    info!(target: "pure_blind::observer", wallet = %wallet, "已连接至 Yellowstone gRPC");

    while let Some(update) = stream.next().await.transpose()? {
        if let Some(tx_info) = parse_transaction_update(&update) {
            if let Err(err) =
                process_transaction(&pool_catalog, &route_catalog, wallet, tx_info).await
            {
                warn!(
                    target: "pure_blind::observer",
                    wallet = %wallet,
                    error = %err,
                    "处理交易时出错"
                );
            }
        }
    }

    Ok(())
}

async fn process_transaction(
    pool_catalog: &Arc<PoolCatalog>,
    route_catalog: &Arc<RouteCatalog>,
    wallet: Pubkey,
    tx_info: SubscribeUpdateTransactionInfo,
) -> Result<()> {
    let confirmed_tx = tx_info
        .transaction
        .ok_or_else(|| anyhow!("Yellowstone 交易缺失 transaction 字段"))?;
    let meta = tx_info
        .meta
        .ok_or_else(|| anyhow!("Yellowstone 交易缺失 meta 字段"))?;

    if meta.err.is_some() {
        return Ok(());
    }

    let observed_slot = tx_info.index as u64;
    let versioned = decode_versioned_transaction(&confirmed_tx)?;
    let loaded_addresses = TransactionLoadedAddresses::try_from(&meta)?;
    let token_balances = TransactionTokenBalances::try_from(&meta)?;

    let (outer_instructions, account_keys) =
        instructions_from_message(&versioned.message, &[], Some(&loaded_addresses))?;

    let balance_map = build_balance_map(&token_balances, &account_keys);
    let lookup_usages = match extract_lookup_usages(&versioned.message, &loaded_addresses) {
        Ok(usages) => usages,
        Err(err) => {
            warn!(
                target: "pure_blind::observer",
                error = %err,
                "解析地址表使用情况失败"
            );
            Vec::new()
        }
    };
    let profit_insight = detect_profit_asset(wallet, &balance_map);

    for (index, instruction) in outer_instructions.iter().enumerate() {
        if instruction.program_id != JUPITER_PROGRAM_ID {
            continue;
        }
        let route = match super::decoder::decode_route_instruction(instruction) {
            Ok(route) => route,
            Err(err) => {
                warn!(target: "pure_blind::observer", error = %err, "解码 Route 指令失败");
                continue;
            }
        };
        let inner_execs = collect_inner_instructions(&meta, &account_keys, index);
        if inner_execs.is_empty() {
            continue;
        }
        let mut inner_iter = inner_execs.into_iter();
        let mut route_profiles: Vec<PoolProfile> = Vec::new();
        let mut route_lookup_tables: HashSet<Pubkey> = HashSet::new();

        for step in route.steps {
            if let Some((parsed, accounts)) = match_step_with_inner(&step, &mut inner_iter) {
                let step_context =
                    resolve_step_context(wallet, &parsed, &accounts, &balance_map, &step);
                let input_asset = step_context.input_asset;
                let output_asset = step_context.output_asset;
                let discriminant = match resolve_swap_discriminant(step.variant.as_str()) {
                    Ok(value) => value,
                    Err(err) => {
                        warn!(
                            target: "pure_blind::observer",
                            error = %err,
                            variant = %step.variant,
                            "解析 swap discriminant 失败"
                        );
                        continue;
                    }
                };
                let key = PoolKey::from_parsed_swap(
                    &parsed,
                    input_asset.map(|asset| asset.mint),
                    output_asset.map(|asset| asset.mint),
                    discriminant,
                );
                let lookup_tables = match_lookup_tables(&accounts, &lookup_usages);
                let observation = PoolObservation {
                    key: key.clone(),
                    swap: &step.swap,
                    swap_variant: &step.variant,
                    swap_payload: &step.payload,
                    remaining_accounts: &accounts,
                    lookup_tables: &lookup_tables,
                    input_index: step.input_index,
                    output_index: step.output_index,
                    slot: observed_slot,
                    estimated_profit: None,
                    input_asset,
                    output_asset,
                };
                pool_catalog.ingest(observation);

                let profile = PoolProfile::new(
                    key.clone(),
                    step.swap.clone(),
                    step.variant.clone(),
                    step.payload.clone(),
                    step.input_index,
                    step.output_index,
                    input_asset,
                    output_asset,
                    Arc::new(lookup_tables.clone()),
                    Arc::new(accounts.clone()),
                );
                for table in &lookup_tables {
                    route_lookup_tables.insert(*table);
                }
                route_profiles.push(profile);
            }
        }

        if route_profiles.len() >= 2 && route_is_closed(&route_profiles) {
            let base_asset = profit_insight.map(|(asset, _)| asset).or_else(|| {
                route_profiles
                    .first()
                    .and_then(|profile| profile.input_asset)
            });
            let estimated_profit = profit_insight.map(|(_, profit)| profit);
            let mut route_tables: Vec<Pubkey> = route_lookup_tables.into_iter().collect();
            route_tables.sort_unstable();
            let observation = RouteObservation {
                steps: route_profiles,
                lookup_tables: route_tables,
                base_asset,
                estimated_profit,
                slot: observed_slot,
            };
            route_catalog.ingest(observation);
        }
    }

    Ok(())
}

fn route_is_closed(steps: &[PoolProfile]) -> bool {
    let Some(first) = steps.first() else {
        return false;
    };
    let Some(last) = steps.last() else {
        return false;
    };
    match (first.input_asset, last.output_asset) {
        (Some(input), Some(output)) => {
            input.mint == output.mint && input.token_program == output.token_program
        }
        _ => false,
    }
}

fn collect_inner_instructions(
    meta: &confirmed_block::TransactionStatusMeta,
    account_keys: &[Pubkey],
    outer_index: usize,
) -> Vec<ResolvedInnerInstruction> {
    let mut resolved = Vec::new();
    for entry in &meta.inner_instructions {
        if entry.index as usize != outer_index {
            continue;
        }
        for compiled in &entry.instructions {
            if let Some(instr) = resolve_compiled_instruction(compiled, account_keys) {
                resolved.push(instr);
            }
        }
    }
    resolved
}

fn resolve_compiled_instruction(
    compiled: &confirmed_block::InnerInstruction,
    account_keys: &[Pubkey],
) -> Option<ResolvedInnerInstruction> {
    let program_index = compiled.program_id_index as usize;
    let program_id = *account_keys.get(program_index)?;
    let mut accounts = Vec::with_capacity(compiled.accounts.len());
    for idx in &compiled.accounts {
        let idx = *idx as usize;
        if let Some(pubkey) = account_keys.get(idx) {
            accounts.push(*pubkey);
        } else {
            return None;
        }
    }
    Some(ResolvedInnerInstruction {
        program_id,
        accounts,
    })
}

struct ResolvedInnerInstruction {
    program_id: Pubkey,
    accounts: Vec<Pubkey>,
}

fn match_step_with_inner(
    step: &DecodedJupiterStep,
    inner_iter: &mut impl Iterator<Item = ResolvedInnerInstruction>,
) -> Option<(ParsedSwapAccounts, Vec<Pubkey>)> {
    while let Some(inner) = inner_iter.next() {
        if let Some(parsed) = step.parse_accounts(&inner.accounts) {
            if parsed.swap_program() == inner.program_id {
                return Some((parsed, inner.accounts));
            }
        }
    }
    None
}

struct StepContext {
    input_asset: Option<PoolAsset>,
    output_asset: Option<PoolAsset>,
}

fn resolve_step_context(
    wallet: Pubkey,
    parsed: &ParsedSwapAccounts,
    accounts: &[Pubkey],
    balances: &HashMap<Pubkey, TokenBalanceEntry>,
    step: &DecodedJupiterStep,
) -> StepContext {
    let (user_input, user_output) = parsed
        .user_accounts()
        .map(|(src, dst)| (Some(src), Some(dst)))
        .unwrap_or((None, None));

    let mut spent_candidate: Option<(i128, PoolAsset)> = None;
    let mut receive_candidate: Option<(i128, PoolAsset)> = None;

    let mut track_delta = |delta: i128, asset: PoolAsset| {
        if delta < 0 {
            if spent_candidate
                .as_ref()
                .map_or(true, |(best, _)| delta < *best)
            {
                spent_candidate = Some((delta, asset));
            }
        } else if delta > 0 {
            if receive_candidate
                .as_ref()
                .map_or(true, |(best, _)| delta > *best)
            {
                receive_candidate = Some((delta, asset));
            }
        }
    };

    if let Some(entry) = user_input.and_then(|pubkey| balances.get(&pubkey)) {
        let asset = PoolAsset::new(entry.mint, entry.token_program.unwrap_or(spl_token::id()));
        let delta = balance_delta(entry);
        track_delta(delta, asset);
    }
    if let Some(entry) = user_output.and_then(|pubkey| balances.get(&pubkey)) {
        let asset = PoolAsset::new(entry.mint, entry.token_program.unwrap_or(spl_token::id()));
        let delta = balance_delta(entry);
        track_delta(delta, asset);
    }

    // 允许解析到多个用户账户：遍历指令账户寻找余额变化
    for account in accounts {
        if Some(*account) == user_input || Some(*account) == user_output {
            continue;
        }
        if let Some(entry) = balances.get(account) {
            if entry.owner != Some(wallet) {
                continue;
            }
            let delta = balance_delta(entry);
            if delta == 0 {
                continue;
            }
            let asset = PoolAsset::new(entry.mint, entry.token_program.unwrap_or(spl_token::id()));
            track_delta(delta, asset);
        }
    }

    let mut spent_asset = spent_candidate.map(|(_, asset)| asset);
    let mut received_asset = receive_candidate.map(|(_, asset)| asset);

    let user_input_asset = user_input
        .and_then(|pubkey| balances.get(&pubkey))
        .map(|entry| PoolAsset::new(entry.mint, entry.token_program.unwrap_or(spl_token::id())));
    let user_output_asset = user_output
        .and_then(|pubkey| balances.get(&pubkey))
        .map(|entry| PoolAsset::new(entry.mint, entry.token_program.unwrap_or(spl_token::id())));

    // 根据 Jupiter payload 的方向字段兜底
    match step.direction_hint {
        DirectionHint::Known(DirectionFlag::Forward) => {
            if spent_asset.is_none() {
                spent_asset = user_input_asset
                    .clone()
                    .or_else(|| user_output_asset.clone());
            }
            if received_asset.is_none() {
                received_asset = user_output_asset
                    .clone()
                    .or_else(|| user_input_asset.clone());
            }
        }
        DirectionHint::Known(DirectionFlag::Reverse) => {
            if spent_asset.is_none() {
                spent_asset = user_output_asset
                    .clone()
                    .or_else(|| user_input_asset.clone());
            }
            if received_asset.is_none() {
                received_asset = user_input_asset
                    .clone()
                    .or_else(|| user_output_asset.clone());
            }
        }
        DirectionHint::Unknown => {}
    }

    if spent_asset.is_none() {
        spent_asset = accounts.iter().find_map(|account| {
            balances.get(account).and_then(|entry| {
                if entry.owner == Some(wallet) {
                    Some(PoolAsset::new(
                        entry.mint,
                        entry.token_program.unwrap_or(spl_token::id()),
                    ))
                } else {
                    None
                }
            })
        });
    }

    if received_asset.is_none() {
        received_asset = accounts.iter().rev().find_map(|account| {
            balances.get(account).and_then(|entry| {
                if entry.owner == Some(wallet) {
                    Some(PoolAsset::new(
                        entry.mint,
                        entry.token_program.unwrap_or(spl_token::id()),
                    ))
                } else {
                    None
                }
            })
        });
    }

    // 再兜底：在账户列表里找任意 owner==wallet 的资产
    StepContext {
        input_asset: spent_asset
            .or(user_input_asset)
            .or(user_output_asset.clone()),
        output_asset: received_asset.or(user_output_asset).or(user_input_asset),
    }
}

fn build_balance_map(
    balances: &TransactionTokenBalances,
    account_keys: &[Pubkey],
) -> HashMap<Pubkey, TokenBalanceEntry> {
    let mut map = HashMap::new();
    for (index, entry) in balances.iter_indexed() {
        if let Some(pubkey) = account_keys.get(*index) {
            map.insert(*pubkey, entry.clone());
        }
    }
    map
}

type ProfitInsight = (PoolAsset, i128);

fn detect_profit_asset(
    wallet: Pubkey,
    balances: &HashMap<Pubkey, TokenBalanceEntry>,
) -> Option<ProfitInsight> {
    let mut best: Option<ProfitInsight> = None;
    for entry in balances.values() {
        if entry.owner != Some(wallet) {
            continue;
        }
        let pre = entry.pre_amount.unwrap_or(0) as i128;
        let post = entry.post_amount.unwrap_or(0) as i128;
        let delta = post - pre;
        if delta <= 0 {
            continue;
        }
        let asset = PoolAsset::new(entry.mint, entry.token_program.unwrap_or(spl_token::id()));
        match &mut best {
            Some((_, existing_delta)) if delta <= *existing_delta => {}
            _ => best = Some((asset, delta)),
        }
    }
    best
}

#[derive(Clone, Debug)]
struct LookupUsage {
    table: Pubkey,
    addresses: Vec<Pubkey>,
}

fn extract_lookup_usages(
    message: &VersionedMessage,
    loaded: &TransactionLoadedAddresses,
) -> Result<Vec<LookupUsage>> {
    let mut usages = Vec::new();
    let Some(lookups) = message.address_table_lookups() else {
        return Ok(usages);
    };

    let mut writable_cursor = 0;
    let mut readonly_cursor = 0;

    for lookup in lookups {
        let writable_count = lookup.writable_indexes.len();
        let readonly_count = lookup.readonly_indexes.len();

        if writable_cursor + writable_count > loaded.writable.len() {
            return Err(anyhow!(
                "ALT writable addresses 不足: 期望 {}, 剩余 {}",
                writable_count,
                loaded.writable.len().saturating_sub(writable_cursor)
            ));
        }
        if readonly_cursor + readonly_count > loaded.readonly.len() {
            return Err(anyhow!(
                "ALT readonly addresses 不足: 期望 {}, 剩余 {}",
                readonly_count,
                loaded.readonly.len().saturating_sub(readonly_cursor)
            ));
        }

        let writable = loaded.writable[writable_cursor..writable_cursor + writable_count].to_vec();
        let readonly = loaded.readonly[readonly_cursor..readonly_cursor + readonly_count].to_vec();

        writable_cursor += writable_count;
        readonly_cursor += readonly_count;

        let mut addresses = Vec::with_capacity(writable.len() + readonly.len());
        addresses.extend(writable);
        addresses.extend(readonly);

        usages.push(LookupUsage {
            table: lookup.account_key,
            addresses,
        });
    }

    Ok(usages)
}

fn match_lookup_tables(accounts: &[Pubkey], usages: &[LookupUsage]) -> Vec<Pubkey> {
    if usages.is_empty() {
        return Vec::new();
    }
    let mut matched = Vec::new();
    for usage in usages {
        if usage
            .addresses
            .iter()
            .any(|address| accounts.contains(address))
        {
            if !matched.contains(&usage.table) {
                matched.push(usage.table);
            }
        }
    }
    matched
}

fn balance_delta(entry: &TokenBalanceEntry) -> i128 {
    let pre = entry.pre_amount.unwrap_or(0) as i128;
    let post = entry.post_amount.unwrap_or(0) as i128;
    post - pre
}

fn gain_delta(entry: &TokenBalanceEntry) -> i128 {
    balance_delta(entry).max(0)
}

fn spend_delta(entry: &TokenBalanceEntry) -> i128 {
    (entry.pre_amount.unwrap_or(0) as i128 - entry.post_amount.unwrap_or(0) as i128).max(0)
}
