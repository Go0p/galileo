use std::collections::{BTreeSet, HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use rand::Rng;
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_message::compiled_instruction::CompiledInstruction;
use solana_message::legacy::Message as LegacyMessage;
use solana_program::pubkey::Pubkey as ProgramPubkey;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::message::VersionedMessage;
use solana_sdk::message::v0::{Message as V0Message, MessageAddressTableLookup};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_sdk::transaction::VersionedTransaction;
use tracing::warn;
use yellowstone_grpc_proto::solana::storage::confirmed_block;

use crate::engine::ComputeUnitPriceMode;
use crate::jupiter_parser::PROGRAM_ID as PARSER_PROGRAM_ID;
use crate::jupiter_parser::{RouteKind, RouteV2Accounts};
use crate::txs::jupiter::types::JUPITER_V6_PROGRAM_ID;

use super::constants::{ASSOCIATED_TOKEN_PROGRAM_ID, COMPUTE_BUDGET_PROGRAM_ID, SYSTEM_PROGRAM_ID};

#[derive(Clone, Debug)]
pub(crate) struct RouteContext {
    pub authority: Pubkey,
}

impl RouteContext {
    pub fn from_instruction(instruction: &Instruction) -> Option<Self> {
        debug_assert_eq!(
            JUPITER_V6_PROGRAM_ID, PARSER_PROGRAM_ID,
            "Jupiter program id mismatch"
        );
        if instruction.program_id != JUPITER_V6_PROGRAM_ID {
            return None;
        }
        match crate::jupiter_parser::classify(&instruction.data) {
            RouteKind::RouteV2 | RouteKind::SharedRouteV2 | RouteKind::ExactRouteV2 => {
                let parsed = RouteV2Accounts::parse(instruction)?;
                Some(Self {
                    authority: parsed.user_transfer_authority,
                })
            }
            RouteKind::Route => {
                let accounts = &instruction.accounts;
                if accounts.len() < 9 {
                    return None;
                }
                Some(Self {
                    authority: accounts[1].pubkey,
                })
            }
            RouteKind::Other => None,
        }
    }
}

fn read_u32(input: &mut &[u8]) -> Option<u32> {
    if input.len() < 4 {
        None
    } else {
        let (value, rest) = input.split_at(4);
        *input = rest;
        Some(u32::from_le_bytes(value.try_into().ok()?))
    }
}

fn read_u64_at(data: &[u8], offset: usize) -> Option<u64> {
    let bytes = data.get(offset..offset + 8)?;
    Some(u64::from_le_bytes(bytes.try_into().ok()?))
}

fn write_u64_at(data: &mut [u8], offset: usize, value: u64) -> Result<()> {
    let target = data
        .get_mut(offset..offset + 8)
        .ok_or_else(|| anyhow!("Route data 长度不足，无法写入 in_amount"))?;
    target.copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn route_legacy_in_amount_offset(data: &[u8]) -> Option<usize> {
    let mut rest = data.get(8..)?;
    let len = read_u32(&mut rest)? as usize;
    if len != 0 {
        return None;
    }
    Some(data.len().saturating_sub(rest.len()))
}

fn route_legacy_quoted_out_offset(data: &[u8]) -> Option<usize> {
    let mut rest = data.get(8..)?;
    let len = read_u32(&mut rest)? as usize;
    if len != 0 {
        return None;
    }
    let in_amount_offset = data.len().saturating_sub(rest.len());
    Some(in_amount_offset + 8)
}

pub(crate) fn read_route_in_amount(kind: RouteKind, data: &[u8]) -> Option<u64> {
    match kind {
        RouteKind::RouteV2 => read_u64_at(data, 8),
        RouteKind::SharedRouteV2 => read_u64_at(data, 9),
        RouteKind::Route => {
            let offset = route_legacy_in_amount_offset(data)?;
            read_u64_at(data, offset)
        }
        _ => None,
    }
}

pub(crate) fn update_route_in_amount(kind: RouteKind, data: &mut [u8], value: u64) -> Result<()> {
    match kind {
        RouteKind::RouteV2 => write_u64_at(data, 8, value),
        RouteKind::SharedRouteV2 => write_u64_at(data, 9, value),
        RouteKind::Route => {
            let offset = route_legacy_in_amount_offset(data).ok_or_else(|| {
                anyhow!("Route 指令 route_plan 非空或数据无效，无法写入 in_amount")
            })?;
            write_u64_at(data, offset, value)
        }
        _ => Err(anyhow!("当前 route 指令类型不支持 in_amount 调整")),
    }
}

pub(crate) fn read_route_quoted_out_amount(kind: RouteKind, data: &[u8]) -> Option<u64> {
    match kind {
        RouteKind::RouteV2 => read_u64_at(data, 16),
        RouteKind::SharedRouteV2 => read_u64_at(data, 17),
        RouteKind::Route => {
            let offset = route_legacy_quoted_out_offset(data)?;
            read_u64_at(data, offset)
        }
        _ => None,
    }
}

pub(crate) fn update_route_quoted_out_amount(
    kind: RouteKind,
    data: &mut [u8],
    value: u64,
) -> Result<()> {
    match kind {
        RouteKind::RouteV2 => write_u64_at(data, 16, value),
        RouteKind::SharedRouteV2 => write_u64_at(data, 17, value),
        RouteKind::Route => {
            let offset = route_legacy_quoted_out_offset(data).ok_or_else(|| {
                anyhow!("Route 指令 route_plan 非空或数据无效，无法写入 quoted_out_amount")
            })?;
            write_u64_at(data, offset, value)
        }
        _ => Err(anyhow!("当前 route 指令类型不支持 quoted_out_amount 调整")),
    }
}

pub(crate) fn filter_transaction(
    transaction: &VersionedTransaction,
    include: &HashSet<Pubkey>,
    exclude: &HashSet<Pubkey>,
    loaded_addresses: Option<&TransactionLoadedAddresses>,
) -> bool {
    let mut account_keys: Vec<Pubkey> = transaction.message.static_account_keys().to_vec();
    if let Some(addresses) = loaded_addresses {
        account_keys.extend(addresses.writable.iter().copied());
        account_keys.extend(addresses.readonly.iter().copied());
    }

    let mut include_matched = include.is_empty();
    for compiled in transaction.message.instructions() {
        let program_index = compiled.program_id_index as usize;
        if let Some(program_id) = account_keys.get(program_index) {
            if exclude.contains(program_id) {
                return false;
            }
            if include.is_empty() {
                include_matched = true;
            } else if include.contains(program_id) {
                include_matched = true;
            }
        }
    }

    include_matched
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TransactionLoadedAddresses {
    pub writable: Vec<Pubkey>,
    pub readonly: Vec<Pubkey>,
}

impl TryFrom<&confirmed_block::TransactionStatusMeta> for TransactionLoadedAddresses {
    type Error = anyhow::Error;

    fn try_from(meta: &confirmed_block::TransactionStatusMeta) -> Result<Self> {
        let writable = meta
            .loaded_writable_addresses
            .iter()
            .map(|bytes| Pubkey::try_from(bytes.as_slice()))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|err| anyhow!("解析 writable loaded address 失败: {err}"))?;
        let readonly = meta
            .loaded_readonly_addresses
            .iter()
            .map(|bytes| Pubkey::try_from(bytes.as_slice()))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|err| anyhow!("解析 readonly loaded address 失败: {err}"))?;

        Ok(Self { writable, readonly })
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct TransactionTokenBalances {
    by_index: HashMap<usize, TokenBalanceEntry>,
}

impl TransactionTokenBalances {
    pub fn entries(&self) -> impl Iterator<Item = &TokenBalanceEntry> {
        self.by_index.values()
    }
}

impl TryFrom<&confirmed_block::TransactionStatusMeta> for TransactionTokenBalances {
    type Error = anyhow::Error;

    fn try_from(meta: &confirmed_block::TransactionStatusMeta) -> Result<Self> {
        let mut balances = HashMap::new();
        for balance in &meta.pre_token_balances {
            if let Some(entry) = TokenBalanceEntry::from_balance(balance, BalanceSnapshot::Pre) {
                balances.entry(entry.account_index).or_insert(entry);
            }
        }
        for balance in &meta.post_token_balances {
            if let Some(entry) = TokenBalanceEntry::from_balance(balance, BalanceSnapshot::Post) {
                balances
                    .entry(entry.account_index)
                    .and_modify(|existing| {
                        existing.update_from_balance(balance, BalanceSnapshot::Post);
                    })
                    .or_insert(entry);
            }
        }

        Ok(Self { by_index: balances })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TokenBalanceEntry {
    pub account_index: usize,
    pub mint: Pubkey,
    pub token_program: Option<Pubkey>,
    pub owner: Option<Pubkey>,
    pub pre_amount: Option<u64>,
    pub post_amount: Option<u64>,
}

enum BalanceSnapshot {
    Pre,
    Post,
}

impl TokenBalanceEntry {
    fn from_balance(
        balance: &confirmed_block::TokenBalance,
        snapshot: BalanceSnapshot,
    ) -> Option<Self> {
        if balance.mint.is_empty() {
            return None;
        }
        let mint = Pubkey::from_str(balance.mint.as_str()).ok()?;
        let token_program = if balance.program_id.is_empty() {
            None
        } else {
            Pubkey::from_str(balance.program_id.as_str()).ok()
        };
        let owner = if balance.owner.is_empty() {
            None
        } else {
            Pubkey::from_str(balance.owner.as_str()).ok()
        };

        let mut entry = Self {
            account_index: balance.account_index as usize,
            mint,
            token_program,
            owner,
            pre_amount: None,
            post_amount: None,
        };
        entry.update_amount(balance, snapshot);
        Some(entry)
    }

    fn update_from_balance(
        &mut self,
        balance: &confirmed_block::TokenBalance,
        snapshot: BalanceSnapshot,
    ) {
        if self.token_program.is_none() && !balance.program_id.is_empty() {
            if let Ok(program) = Pubkey::from_str(balance.program_id.as_str()) {
                self.token_program = Some(program);
            }
        }
        if self.owner.is_none() && !balance.owner.is_empty() {
            if let Ok(owner) = Pubkey::from_str(balance.owner.as_str()) {
                self.owner = Some(owner);
            }
        }
        self.update_amount(balance, snapshot);
    }

    fn update_amount(
        &mut self,
        balance: &confirmed_block::TokenBalance,
        snapshot: BalanceSnapshot,
    ) {
        if let Some(ui_amount) = balance.ui_token_amount.as_ref() {
            if let Ok(amount) = ui_amount.amount.parse::<u64>() {
                match snapshot {
                    BalanceSnapshot::Pre => self.pre_amount = Some(amount),
                    BalanceSnapshot::Post => self.post_amount = Some(amount),
                }
            }
        }
    }
}

pub(crate) fn decode_versioned_transaction(
    tx: &confirmed_block::Transaction,
) -> Result<VersionedTransaction> {
    let message = tx
        .message
        .as_ref()
        .ok_or_else(|| anyhow!("交易缺少 message"))?;

    let signatures = tx
        .signatures
        .iter()
        .map(|bytes| Signature::try_from(bytes.as_slice()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|err| anyhow!("签名解析失败: {err}"))?;

    let message = if message.versioned {
        VersionedMessage::V0(decode_v0_message(message)?)
    } else {
        let legacy = decode_legacy_message(message)?;
        VersionedMessage::Legacy(legacy)
    };

    Ok(VersionedTransaction {
        signatures,
        message,
    })
}

pub(crate) fn decode_legacy_message(message: &confirmed_block::Message) -> Result<LegacyMessage> {
    use solana_message::MessageHeader;

    let header_proto = message
        .header
        .as_ref()
        .ok_or_else(|| anyhow!("legacy message 缺少 header"))?;
    let header = MessageHeader {
        num_required_signatures: header_proto.num_required_signatures as u8,
        num_readonly_signed_accounts: header_proto.num_readonly_signed_accounts as u8,
        num_readonly_unsigned_accounts: header_proto.num_readonly_unsigned_accounts as u8,
    };

    let account_keys = message
        .account_keys
        .iter()
        .map(|bytes| Pubkey::try_from(bytes.as_slice()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|err| anyhow!("account key 解析失败: {err}"))?;

    let recent_blockhash = {
        let bytes: [u8; 32] = message
            .recent_blockhash
            .as_slice()
            .try_into()
            .map_err(|_| anyhow!("recent blockhash 长度错误"))?;
        Hash::new_from_array(bytes)
    };

    let instructions = message
        .instructions
        .iter()
        .map(|ix| CompiledInstruction {
            program_id_index: ix.program_id_index as u8,
            accounts: ix.accounts.clone(),
            data: ix.data.clone(),
        })
        .collect();

    Ok(LegacyMessage {
        header,
        account_keys,
        recent_blockhash,
        instructions,
    })
}

pub(crate) fn decode_v0_message(message: &confirmed_block::Message) -> Result<V0Message> {
    use solana_message::MessageHeader;

    let header_proto = message
        .header
        .as_ref()
        .ok_or_else(|| anyhow!("v0 message 缺少 header"))?;
    let header = MessageHeader {
        num_required_signatures: header_proto.num_required_signatures as u8,
        num_readonly_signed_accounts: header_proto.num_readonly_signed_accounts as u8,
        num_readonly_unsigned_accounts: header_proto.num_readonly_unsigned_accounts as u8,
    };

    let account_keys = message
        .account_keys
        .iter()
        .map(|bytes| Pubkey::try_from(bytes.as_slice()))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|err| anyhow!("account key 解析失败: {err}"))?;

    let recent_blockhash = {
        let bytes: [u8; 32] = message
            .recent_blockhash
            .as_slice()
            .try_into()
            .map_err(|_| anyhow!("recent blockhash 长度错误"))?;
        Hash::new_from_array(bytes)
    };

    let instructions = message
        .instructions
        .iter()
        .map(|ix| CompiledInstruction {
            program_id_index: ix.program_id_index as u8,
            accounts: ix.accounts.clone(),
            data: ix.data.clone(),
        })
        .collect();

    let mut lookups = Vec::with_capacity(message.address_table_lookups.len());
    for lookup in &message.address_table_lookups {
        let account_key = Pubkey::try_from(lookup.account_key.as_slice())
            .map_err(|err| anyhow!("lookup account 解析失败: {err}"))?;
        lookups.push(MessageAddressTableLookup {
            account_key,
            writable_indexes: lookup.writable_indexes.clone(),
            readonly_indexes: lookup.readonly_indexes.clone(),
        });
    }

    Ok(V0Message {
        header,
        account_keys,
        recent_blockhash,
        instructions,
        address_table_lookups: lookups,
    })
}

pub(crate) async fn resolve_lookup_accounts(
    rpc: &Arc<RpcClient>,
    message: &VersionedMessage,
) -> Result<Vec<solana_sdk::message::AddressLookupTableAccount>> {
    let Some(lookups) = message.address_table_lookups() else {
        return Ok(Vec::new());
    };
    if lookups.is_empty() {
        return Ok(Vec::new());
    }

    let keys: Vec<Pubkey> = lookups.iter().map(|lookup| lookup.account_key).collect();
    let accounts = rpc
        .get_multiple_accounts(&keys)
        .await
        .map_err(|err| anyhow!("获取 ALT 失败: {err}"))?;

    let mut resolved = Vec::with_capacity(keys.len());
    for (key, maybe_account) in keys.iter().zip(accounts.into_iter()) {
        if let Some(account) = maybe_account {
            if let Some(table) = deserialize_lookup_table(key, account) {
                resolved.push(table);
            }
        }
    }

    Ok(resolved)
}

fn deserialize_lookup_table(
    key: &Pubkey,
    account: solana_sdk::account::Account,
) -> Option<solana_sdk::message::AddressLookupTableAccount> {
    match AddressLookupTable::deserialize(&account.data) {
        Ok(table) => Some(solana_sdk::message::AddressLookupTableAccount {
            key: *key,
            addresses: table.addresses.into_owned(),
        }),
        Err(err) => {
            warn!(
                target: "strategy::copy",
                address = %key,
                error = %err,
                "反序列化 ALT 失败"
            );
            None
        }
    }
}

pub(crate) fn instructions_from_message(
    message: &VersionedMessage,
    lookups: &[solana_sdk::message::AddressLookupTableAccount],
    loaded_addresses: Option<&TransactionLoadedAddresses>,
) -> Result<(Vec<Instruction>, Vec<Pubkey>)> {
    let account_keys = collect_account_keys(message, lookups, loaded_addresses);
    let static_len = message.static_account_keys().len();
    let (loaded_writable_len, loaded_readonly_len) = loaded_addresses
        .map(|addresses| (addresses.writable.len(), addresses.readonly.len()))
        .unwrap_or((0, 0));
    let loaded_readonly_start = static_len + loaded_writable_len;
    let total_len = static_len + loaded_writable_len + loaded_readonly_len;

    let mut instructions = Vec::with_capacity(message.instructions().len());
    for compiled in message.instructions() {
        let program_index = compiled.program_id_index as usize;
        let program_id = *account_keys.get(program_index).ok_or_else(|| {
            anyhow!(
                "编译指令 program index {program_index} 超出 account_keys 范围 {}",
                account_keys.len()
            )
        })?;

        let mut metas = Vec::with_capacity(compiled.accounts.len());
        for &index in &compiled.accounts {
            let idx = index as usize;
            let pubkey = *account_keys
                .get(idx)
                .ok_or_else(|| anyhow!("account index {idx} 越界"))?;
            let is_signer = if idx < static_len {
                message.is_signer(idx)
            } else {
                false
            };
            let is_writable = if idx < static_len {
                message.is_maybe_writable(idx, None)
            } else if idx < total_len {
                idx < loaded_readonly_start
            } else {
                false
            };
            metas.push(AccountMeta {
                pubkey,
                is_signer,
                is_writable,
            });
        }

        instructions.push(Instruction {
            program_id,
            accounts: metas,
            data: compiled.data.clone(),
        });
    }

    Ok((instructions, account_keys))
}

fn collect_account_keys(
    message: &VersionedMessage,
    lookups: &[solana_sdk::message::AddressLookupTableAccount],
    loaded_addresses: Option<&TransactionLoadedAddresses>,
) -> Vec<Pubkey> {
    let mut account_keys: Vec<Pubkey> = message.static_account_keys().to_vec();
    if let Some(addresses) = loaded_addresses {
        account_keys.extend(addresses.writable.iter().copied());
        account_keys.extend(addresses.readonly.iter().copied());
        return account_keys;
    }
    if let Some(table_lookups) = message.address_table_lookups() {
        let mut lookup_map: HashMap<Pubkey, &solana_sdk::message::AddressLookupTableAccount> =
            HashMap::with_capacity(lookups.len());
        for account in lookups {
            lookup_map.insert(account.key, account);
        }

        for lookup in table_lookups {
            if let Some(account) = lookup_map.get(&lookup.account_key) {
                for idx in &lookup.writable_indexes {
                    let index = *idx as usize;
                    if let Some(address) = account.addresses.get(index) {
                        account_keys.push(*address);
                    } else {
                        warn!(
                            target: "strategy::copy",
                            lookup = %lookup.account_key,
                            index,
                            "ALT writable index 越界"
                        );
                    }
                }
                for idx in &lookup.readonly_indexes {
                    let index = *idx as usize;
                    if let Some(address) = account.addresses.get(index) {
                        account_keys.push(*address);
                    } else {
                        warn!(
                            target: "strategy::copy",
                            lookup = %lookup.account_key,
                            index,
                            "ALT readonly index 越界"
                        );
                    }
                }
            } else {
                warn!(
                    target: "strategy::copy",
                    lookup = %lookup.account_key,
                    "未找到 ALT 账户，指令可能解析失败"
                );
            }
        }
    }

    account_keys
}

pub(crate) fn derive_associated_token_address(
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Result<Pubkey> {
    if mint == &Pubkey::default() {
        return Err(anyhow!("mint 未初始化"));
    }
    if token_program == &Pubkey::default() {
        return Err(anyhow!("token program 未初始化"));
    }
    let owner_prog = to_program_pubkey(owner);
    let mint_prog = to_program_pubkey(mint);
    let token_prog = to_program_pubkey(token_program);
    let program_id = to_program_pubkey(&ASSOCIATED_TOKEN_PROGRAM_ID);
    let (ata, _) = ProgramPubkey::find_program_address(
        &[owner_prog.as_ref(), token_prog.as_ref(), mint_prog.as_ref()],
        &program_id,
    );
    Ok(from_program_pubkey(ata))
}

pub(crate) fn build_create_ata_instruction(
    payer: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Result<Instruction> {
    let ata = derive_associated_token_address(owner, mint, token_program)?;
    Ok(Instruction {
        program_id: ASSOCIATED_TOKEN_PROGRAM_ID,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(ata, false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(*mint, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            AccountMeta::new_readonly(*token_program, false),
        ],
        data: vec![1],
    })
}

pub(crate) fn apply_replacements(instructions: &mut [Instruction], map: &HashMap<Pubkey, Pubkey>) {
    for instruction in instructions {
        for account in &mut instruction.accounts {
            if let Some(replacement) = map.get(&account.pubkey) {
                account.pubkey = *replacement;
            }
        }
    }
}

pub(crate) fn split_compute_budget(
    instructions: &[Instruction],
    price_mode: Option<&ComputeUnitPriceMode>,
) -> (Vec<Instruction>, Vec<Instruction>) {
    let mut compute_budget = Vec::new();
    let mut main = Vec::new();

    for ix in instructions {
        if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID {
            if is_compute_unit_limit(ix) {
                compute_budget.push(ix.clone());
            }
        } else {
            main.push(ix.clone());
        }
    }

    if let Some(price) = price_mode.and_then(sample_compute_unit_price) {
        if price > 0 {
            let mut buf = [0u8; 9];
            buf[0] = 3;
            buf[1..9].copy_from_slice(&price.to_le_bytes());
            compute_budget.push(Instruction {
                program_id: COMPUTE_BUDGET_PROGRAM_ID,
                accounts: Vec::new(),
                data: buf.to_vec(),
            });
        }
    }

    (compute_budget, main)
}

pub(crate) fn extract_compute_unit_limit(instructions: &[Instruction]) -> Option<u32> {
    for ix in instructions {
        if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID && ix.data.first() == Some(&2) {
            if ix.data.len() >= 5 {
                let mut buf = [0u8; 4];
                buf.copy_from_slice(&ix.data[1..5]);
                return Some(u32::from_le_bytes(buf));
            }
        }
    }
    None
}

fn is_compute_unit_limit(ix: &Instruction) -> bool {
    ix.data.first() == Some(&2)
}

pub(crate) fn lookup_addresses(message: &VersionedMessage) -> Vec<Pubkey> {
    message
        .address_table_lookups()
        .map(|lookups| lookups.iter().map(|lookup| lookup.account_key).collect())
        .unwrap_or_default()
}

pub(crate) fn message_required_signatures(message: &VersionedMessage) -> usize {
    match message {
        VersionedMessage::Legacy(msg) => msg.header.num_required_signatures as usize,
        VersionedMessage::V0(msg) => msg.header.num_required_signatures as usize,
    }
}

pub(crate) fn collect_instruction_signers(instructions: &[Instruction]) -> Vec<Pubkey> {
    let mut set = BTreeSet::new();
    for ix in instructions {
        for account in &ix.accounts {
            if account.is_signer {
                set.insert(account.pubkey);
            }
        }
    }
    set.into_iter().collect()
}

fn sample_compute_unit_price(mode: &ComputeUnitPriceMode) -> Option<u64> {
    match mode {
        ComputeUnitPriceMode::Fixed(price) => Some(*price),
        ComputeUnitPriceMode::Random { min, max } => {
            let low = (*min).min(*max);
            let high = (*min).max(*max);
            if low == high {
                Some(low)
            } else if low == 0 && high == 0 {
                None
            } else {
                let mut rng = rand::rng();
                Some(rng.random_range(low..=high))
            }
        }
    }
}

fn to_program_pubkey(pk: &Pubkey) -> ProgramPubkey {
    ProgramPubkey::new_from_array(pk.to_bytes())
}

fn from_program_pubkey(pk: ProgramPubkey) -> Pubkey {
    Pubkey::try_from(pk.as_ref()).expect("pubkey length")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pk(byte: u8) -> Pubkey {
        Pubkey::new_from_array([byte; 32])
    }

    #[test]
    fn instructions_retain_loaded_account_writability() {
        let signer = pk(21);
        let program = pk(22);
        let loaded_writable = pk(23);
        let loaded_readonly = pk(24);

        let header = solana_message::MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 1,
        };

        let message = VersionedMessage::V0(V0Message {
            header,
            account_keys: vec![signer, program],
            recent_blockhash: Hash::default(),
            instructions: vec![CompiledInstruction {
                program_id_index: 1,
                accounts: vec![0, 2, 3],
                data: Vec::new(),
            }],
            address_table_lookups: Vec::new(),
        });

        let loaded_addresses = TransactionLoadedAddresses {
            writable: vec![loaded_writable],
            readonly: vec![loaded_readonly],
        };

        let (instructions, account_keys) =
            instructions_from_message(&message, &[], Some(&loaded_addresses))
                .expect("instructions available");

        assert_eq!(
            account_keys,
            vec![signer, program, loaded_writable, loaded_readonly]
        );

        let accounts = &instructions[0].accounts;
        assert_eq!(accounts[0].pubkey, signer);
        assert!(accounts[0].is_signer);
        assert!(accounts[0].is_writable);

        assert_eq!(accounts[1].pubkey, loaded_writable);
        assert!(!accounts[1].is_signer);
        assert!(accounts[1].is_writable);

        assert_eq!(accounts[2].pubkey, loaded_readonly);
        assert!(!accounts[2].is_signer);
        assert!(!accounts[2].is_writable);
    }

    fn build_route_accounts() -> Vec<AccountMeta> {
        let authority = pk(31);
        let source = pk(32);
        let destination = pk(33);
        let source_mint = pk(34);
        let destination_mint = pk(35);
        let token_program = pk(36);
        vec![
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new(source, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(source_mint, false),
            AccountMeta::new_readonly(destination_mint, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new_readonly(token_program, false),
            AccountMeta::new(destination, false),
            AccountMeta::new_readonly(pk(37), false),
            AccountMeta::new_readonly(pk(38), false),
        ]
    }

    fn route_v2_data(in_amount: u64, quoted_out: u64, slippage: u16) -> Vec<u8> {
        let mut data = crate::jupiter_parser::discriminator::ROUTE_V2.to_vec();
        data.extend_from_slice(&in_amount.to_le_bytes());
        data.extend_from_slice(&quoted_out.to_le_bytes());
        data.extend_from_slice(&slippage.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data
    }

    fn shared_route_v2_data(id: u8, in_amount: u64, quoted_out: u64, slippage: u16) -> Vec<u8> {
        let mut data = crate::jupiter_parser::discriminator::SHARED_ROUTE_V2.to_vec();
        data.push(id);
        data.extend_from_slice(&in_amount.to_le_bytes());
        data.extend_from_slice(&quoted_out.to_le_bytes());
        data.extend_from_slice(&slippage.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data
    }

    fn route_legacy_data(in_amount: u64, quoted_out: u64, slippage: u16) -> Vec<u8> {
        let mut data = crate::jupiter_parser::discriminator::ROUTE.to_vec();
        data.extend_from_slice(&0u32.to_le_bytes()); // empty route_plan
        data.extend_from_slice(&in_amount.to_le_bytes());
        data.extend_from_slice(&quoted_out.to_le_bytes());
        data.extend_from_slice(&slippage.to_le_bytes());
        data.push(0); // platform_fee_bps
        data
    }

    #[test]
    fn route_context_extracts_authority() {
        let accounts = build_route_accounts();
        let instruction = Instruction {
            program_id: JUPITER_V6_PROGRAM_ID,
            accounts,
            data: route_v2_data(1, 1, 0),
        };
        let ctx = RouteContext::from_instruction(&instruction).expect("route context");
        assert_eq!(ctx.authority, pk(31));
    }

    #[test]
    fn route_v2_in_amount_can_be_adjusted() {
        let mut data = route_v2_data(100, 120, 0);
        assert_eq!(read_route_in_amount(RouteKind::RouteV2, &data), Some(100));
        update_route_in_amount(RouteKind::RouteV2, &mut data, 42).expect("update succeeds");
        assert_eq!(read_route_in_amount(RouteKind::RouteV2, &data), Some(42));
    }

    #[test]
    fn shared_route_v2_in_amount_can_be_adjusted() {
        let mut data = shared_route_v2_data(3, 150, 180, 0);
        assert_eq!(
            read_route_in_amount(RouteKind::SharedRouteV2, &data),
            Some(150)
        );
        update_route_in_amount(RouteKind::SharedRouteV2, &mut data, 75).expect("update succeeds");
        assert_eq!(
            read_route_in_amount(RouteKind::SharedRouteV2, &data),
            Some(75)
        );
    }

    #[test]
    fn route_legacy_in_amount_can_be_adjusted() {
        let mut data = route_legacy_data(500, 600, 0);
        assert_eq!(read_route_in_amount(RouteKind::Route, &data), Some(500));
        update_route_in_amount(RouteKind::Route, &mut data, 250).expect("update succeeds");
        assert_eq!(read_route_in_amount(RouteKind::Route, &data), Some(250));
    }
}
