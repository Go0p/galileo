#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
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
use super::profile::{PoolKey, PoolObservation};
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
    catalog: Arc<PoolCatalog>,
) -> Result<PoolObserverHandle> {
    if settings.wallets.is_empty() {
        warn!(target: "pure_blind::observer", "配置未提供任何监听钱包，观察器未启动");
        return Ok(PoolObserverHandle::new(Vec::new()));
    }

    let mut tasks = Vec::with_capacity(settings.wallets.len());
    for wallet in &settings.wallets {
        let endpoint = settings.endpoint.clone();
        let token = settings.token.clone();
        let catalog = Arc::clone(&catalog);
        let wallet = *wallet;
        let handle = tokio::spawn(async move {
            if let Err(err) = run_wallet_observer(endpoint, token, wallet, catalog).await {
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
    catalog: Arc<PoolCatalog>,
) -> Result<()> {
    let mut client = YellowstoneTransactionClient::connect(endpoint, token).await?;
    let mut stream = client
        .subscribe_transactions(wallet)
        .await
        .context("订阅 Yellowstone gRPC 失败")?;

    info!(target: "pure_blind::observer", wallet = %wallet, "已连接至 Yellowstone gRPC");

    while let Some(update) = stream.next().await.transpose()? {
        if let Some(tx_info) = parse_transaction_update(&update) {
            if let Err(err) = process_transaction(&catalog, wallet, tx_info).await {
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
    catalog: &Arc<PoolCatalog>,
    _wallet: Pubkey,
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
        for step in route.steps {
            if let Some((parsed, accounts)) = match_step_with_inner(&step, &mut inner_iter) {
                let (input_mint, output_mint) = resolve_mints(&parsed, &balance_map, &step);
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
                let key = PoolKey::from_parsed_swap(&parsed, input_mint, output_mint, discriminant);
                let observation = PoolObservation {
                    key,
                    swap: &step.swap,
                    swap_variant: &step.variant,
                    swap_payload: &step.payload,
                    remaining_accounts: &accounts,
                    input_index: step.input_index,
                    output_index: step.output_index,
                    slot: observed_slot,
                    estimated_profit: None,
                };
                catalog.ingest(observation);
            }
        }
    }

    Ok(())
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

fn resolve_mints(
    parsed: &ParsedSwapAccounts,
    balances: &HashMap<Pubkey, TokenBalanceEntry>,
    step: &DecodedJupiterStep,
) -> (Option<Pubkey>, Option<Pubkey>) {
    let (maybe_input_account, maybe_output_account) = parsed
        .user_accounts()
        .map(|(a, b)| (Some(a), Some(b)))
        .unwrap_or((None, None));
    let input_mint =
        maybe_input_account.and_then(|account| balances.get(&account).map(|entry| entry.mint));
    let output_mint =
        maybe_output_account.and_then(|account| balances.get(&account).map(|entry| entry.mint));

    match step.direction_hint {
        DirectionHint::Known(DirectionFlag::Forward) => (input_mint, output_mint),
        DirectionHint::Known(DirectionFlag::Reverse) => (output_mint, input_mint),
        DirectionHint::Unknown => (None, None),
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
