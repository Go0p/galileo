use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Error as AnyError;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_compute_budget_interface as compute_budget;
use solana_message::VersionedMessage;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::id as associated_token_program_id;
use spl_token::id as spl_token_program_id;
use thiserror::Error;
use tracing::{error, warn};

use crate::api::ultra::OrderResponsePayload;
use crate::cache::AltCache;
use crate::engine::multi_leg::transaction::decoder::{DecodeTxError, decode_base64_transaction};
use crate::engine::multi_leg::transaction::instructions::{
    InstructionBundle, InstructionExtractionError, extract_instructions,
    rewrite_instruction_accounts_map,
};

use super::context::{UltraContext, UltraLookupResolver};
use super::prepared::{UltraLookupState, UltraPreparedSwap};

pub struct UltraAdapter;

impl UltraAdapter {
    pub async fn prepare<'a>(
        params: UltraPreparationParams<'a>,
        context: UltraContext,
    ) -> Result<UltraPreparedSwap, UltraAdapterError> {
        let payload = params.payload;
        let UltraContext {
            expected_signer,
            lookup_resolver,
        } = context;

        let transaction_b64 = payload
            .transaction
            .clone()
            .ok_or(UltraAdapterError::MissingTransaction)?;
        let versioned_tx =
            decode_base64_transaction(&transaction_b64).map_err(UltraAdapterError::Decode)?;
        let lookup_addresses = collect_lookup_addresses(&versioned_tx.message);

        let mut resolved_lookup_tables = Vec::new();
        let (bundle, lookup_state) = match extract_instructions(&versioned_tx.message, None) {
            Ok(bundle) => (Some(bundle), UltraLookupState::Resolved),
            Err(InstructionExtractionError::MissingLookupTables { .. }) => match lookup_resolver {
                UltraLookupResolver::Fetch { rpc, alt_cache } => {
                    let (bundle, tables) = resolve_instructions_with_lookup_retry(
                        &alt_cache,
                        &rpc,
                        &versioned_tx.message,
                        &lookup_addresses,
                    )
                    .await?;
                    resolved_lookup_tables = tables;
                    (Some(bundle), UltraLookupState::Resolved)
                }
                UltraLookupResolver::Deferred => (None, UltraLookupState::Pending),
            },
            Err(err) => return Err(map_instruction_error(err)),
        };

        let mut requested_limit = None;
        let mut requested_price = params.compute_unit_price_hint;
        let mut compute_budget_instructions = Vec::new();
        let mut main_instructions = Vec::new();

        if let Some(InstructionBundle {
            compute_budget_instructions: mut budget,
            other_instructions,
        }) = bundle
        {
            for ix in budget.drain(..) {
                if let Some(limit) = parse_compute_unit_limit(&ix) {
                    requested_limit = Some(limit);
                    continue;
                }
                if let Some(price) = parse_compute_unit_price(&ix) {
                    requested_price = Some(price);
                    continue;
                }
                compute_budget_instructions.push(ix);
            }
            main_instructions = other_instructions;
        }

        let account_rewrites = build_account_rewrites(payload, params.taker_hint, expected_signer);
        if !account_rewrites.is_empty() && matches!(lookup_state, UltraLookupState::Resolved) {
            rewrite_instruction_accounts_map(&mut compute_budget_instructions, &account_rewrites);
            rewrite_instruction_accounts_map(&mut main_instructions, &account_rewrites);
        }

        Ok(UltraPreparedSwap {
            transaction: versioned_tx,
            compute_budget_instructions,
            main_instructions,
            address_lookup_table_addresses: lookup_addresses,
            resolved_lookup_tables,
            requested_compute_unit_limit: requested_limit,
            requested_compute_unit_price_micro_lamports: requested_price,
            prioritization_fee_lamports: payload.prioritization_fee_lamports,
            account_rewrites,
            lookup_state,
        })
    }
}

pub struct UltraPreparationParams<'a> {
    pub payload: &'a OrderResponsePayload,
    pub compute_unit_price_hint: Option<u64>,
    pub taker_hint: Option<Pubkey>,
}

impl<'a> UltraPreparationParams<'a> {
    pub fn new(payload: &'a OrderResponsePayload) -> Self {
        Self {
            payload,
            compute_unit_price_hint: None,
            taker_hint: None,
        }
    }

    pub fn with_compute_unit_price_hint(mut self, value: Option<u64>) -> Self {
        self.compute_unit_price_hint = value;
        self
    }

    pub fn with_taker_hint(mut self, value: Option<Pubkey>) -> Self {
        self.taker_hint = value;
        self
    }
}

#[derive(Debug, Error)]
pub enum UltraAdapterError {
    #[error("Ultra 报价缺少 transaction 字段")]
    MissingTransaction,
    #[error("Ultra 交易解码失败: {0}")]
    Decode(DecodeTxError),
    #[error("Ultra 指令解析失败: {0}")]
    Instruction(InstructionExtractionError),
    #[error("拉取地址查找表失败: {0}")]
    LookupFetch(AnyError),
}

impl From<AnyError> for UltraAdapterError {
    fn from(error: AnyError) -> Self {
        UltraAdapterError::LookupFetch(error)
    }
}

fn collect_lookup_addresses(message: &VersionedMessage) -> Vec<Pubkey> {
    match message {
        VersionedMessage::Legacy(_) => Vec::new(),
        VersionedMessage::V0(v0) => v0
            .address_table_lookups
            .iter()
            .map(|lookup| lookup.account_key)
            .collect(),
    }
}

async fn fetch_lookup_tables(
    alt_cache: &AltCache,
    rpc: &Arc<RpcClient>,
    addresses: &[Pubkey],
) -> Result<Vec<AddressLookupTableAccount>, UltraAdapterError> {
    if addresses.is_empty() {
        return Ok(Vec::new());
    }
    alt_cache
        .fetch_many(rpc, addresses)
        .await
        .map_err(UltraAdapterError::LookupFetch)
}

fn parse_compute_unit_limit(ix: &Instruction) -> Option<u32> {
    if ix.program_id != compute_budget::id() {
        return None;
    }
    let data = ix.data.as_slice();
    if data.first().copied()? == 2 && data.len() >= 5 {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&data[1..5]);
        return Some(u32::from_le_bytes(buf));
    }
    None
}

fn parse_compute_unit_price(ix: &Instruction) -> Option<u64> {
    if ix.program_id != compute_budget::id() {
        return None;
    }
    let data = ix.data.as_slice();
    if data.first().copied()? == 3 && data.len() >= 9 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(&data[1..9]);
        return Some(u64::from_le_bytes(buf));
    }
    None
}

fn map_instruction_error(err: InstructionExtractionError) -> UltraAdapterError {
    UltraAdapterError::Instruction(err)
}

async fn resolve_instructions_with_lookup_retry(
    alt_cache: &AltCache,
    rpc: &Arc<RpcClient>,
    message: &VersionedMessage,
    lookup_addresses: &[Pubkey],
) -> Result<(InstructionBundle, Vec<AddressLookupTableAccount>), UltraAdapterError> {
    let mut tables = fetch_lookup_tables(alt_cache, rpc, lookup_addresses).await?;
    match extract_instructions(message, Some(tables.as_slice())) {
        Ok(bundle) => Ok((bundle, tables)),
        Err(err @ InstructionExtractionError::LookupIndexOutOfBounds { .. }) => {
            warn!(
                target = "engine::ultra",
                lookup_tables = lookup_addresses.len(),
                error = %err,
                "检测到 ALT 缓存过期，触发强制刷新"
            );
            tables = refresh_lookup_tables(alt_cache, rpc, lookup_addresses).await?;
            match extract_instructions(message, Some(tables.as_slice())) {
                Ok(bundle) => Ok((bundle, tables)),
                Err(err @ InstructionExtractionError::LookupIndexOutOfBounds { .. }) => {
                    error!(
                        target = "engine::ultra",
                        lookup_tables = lookup_addresses.len(),
                        error = %err,
                        "ALT 强制刷新后仍出现索引越界"
                    );
                    Err(UltraAdapterError::Instruction(err))
                }
                Err(other) => Err(map_instruction_error(other)),
            }
        }
        Err(other) => Err(map_instruction_error(other)),
    }
}

async fn refresh_lookup_tables(
    alt_cache: &AltCache,
    rpc: &Arc<RpcClient>,
    lookup_addresses: &[Pubkey],
) -> Result<Vec<AddressLookupTableAccount>, UltraAdapterError> {
    alt_cache
        .refresh_many(rpc, lookup_addresses)
        .await
        .map_err(UltraAdapterError::LookupFetch)?;
    fetch_lookup_tables(alt_cache, rpc, lookup_addresses).await
}

fn build_account_rewrites(
    payload: &OrderResponsePayload,
    taker_hint: Option<Pubkey>,
    expected_signer: Pubkey,
) -> Vec<(Pubkey, Pubkey)> {
    let response_taker = payload.taker.or(taker_hint).unwrap_or(expected_signer);
    if response_taker == expected_signer {
        return Vec::new();
    }

    let mut rewrites = Vec::new();
    rewrites.push((response_taker, expected_signer));

    let mut mints = HashSet::new();
    if let Some(mint) = payload.input_mint {
        mints.insert(mint);
    }
    if let Some(mint) = payload.output_mint {
        mints.insert(mint);
    }
    for step in &payload.route_plan {
        mints.insert(step.swap_info.input_mint);
        mints.insert(step.swap_info.output_mint);
    }

    for mint in mints {
        let from = associated_address(&response_taker, &mint);
        let to = associated_address(&expected_signer, &mint);
        if from != to {
            rewrites.push((from, to));
        }
    }

    rewrites
}

fn associated_address(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let associated_program = Pubkey::new_from_array(associated_token_program_id().to_bytes());
    let token_program = Pubkey::new_from_array(spl_token_program_id().to_bytes());
    let (ata, _) = Pubkey::find_program_address(
        &[owner.as_ref(), token_program.as_ref(), mint.as_ref()],
        &associated_program,
    );
    ata
}
