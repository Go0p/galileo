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
pub(crate) struct CachedTokenAccount {
    pub account: Pubkey,
    pub token_program: Pubkey,
}

#[derive(Clone, Debug)]
pub(crate) struct RouteContext {
    pub authority: Pubkey,
    pub source_ata: Pubkey,
    pub destination_ata: Pubkey,
    pub source_mint: Pubkey,
    pub destination_mint: Pubkey,
    pub source_token_program: Pubkey,
    pub destination_token_program: Pubkey,
    pub params: RouteParams,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RouteParams {
    pub route_id: Option<u8>,
    pub in_amount: Option<u64>,
    pub out_amount: Option<u64>,
    pub quoted_in_amount: Option<u64>,
    pub quoted_out_amount: Option<u64>,
    pub slippage_bps: Option<u16>,
    pub platform_fee_bps: Option<u16>,
    pub positive_slippage_bps: Option<u16>,
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
        let kind = crate::jupiter_parser::classify(&instruction.data);
        let params = RouteParams::parse(kind, &instruction.data);
        match kind {
            RouteKind::RouteV2 | RouteKind::SharedRouteV2 | RouteKind::ExactRouteV2 => {
                let parsed = RouteV2Accounts::parse(instruction)?;
                Some(Self {
                    authority: parsed.user_transfer_authority,
                    source_ata: parsed.user_source_token_account,
                    destination_ata: parsed.user_destination_token_account,
                    source_mint: parsed.source_mint,
                    destination_mint: parsed.destination_mint,
                    source_token_program: parsed.source_token_program,
                    destination_token_program: parsed.destination_token_program,
                    params,
                })
            }
            RouteKind::Route => {
                let accounts = &instruction.accounts;
                if accounts.len() < 9 {
                    return None;
                }
                Some(Self {
                    authority: accounts[1].pubkey,
                    source_ata: accounts[2].pubkey,
                    destination_ata: accounts[3].pubkey,
                    source_mint: Pubkey::default(),
                    destination_mint: accounts[5].pubkey,
                    source_token_program: accounts[0].pubkey,
                    destination_token_program: accounts[0].pubkey,
                    params,
                })
            }
            RouteKind::Other => None,
        }
    }

    pub fn populate_from_balances(
        &mut self,
        account_keys: &[Pubkey],
        balances: Option<&TransactionTokenBalances>,
    ) -> Result<()> {
        if self.source_mint != Pubkey::default()
            && self.destination_mint != Pubkey::default()
            && self.source_token_program != Pubkey::default()
            && self.destination_token_program != Pubkey::default()
        {
            return Ok(());
        }

        let Some(balances) = balances else {
            return Err(anyhow!("Yellowstone meta 缺少 token balance 数据"));
        };

        if self.source_mint == Pubkey::default() {
            let source_index = find_account_index(account_keys, &self.source_ata)
                .ok_or_else(|| anyhow!("Route source ATA 未出现在账户列表中"))?;
            let balance = balances
                .get(source_index)
                .ok_or_else(|| anyhow!("未在 Yellowstone token balances 中找到 source ATA"))?;
            self.source_mint = balance.mint;
            if let Some(program) = balance.token_program {
                if self.source_token_program == Pubkey::default()
                    || self.source_token_program != program
                {
                    self.source_token_program = program;
                }
            }
        }

        if let Some(dest_index) = find_account_index(account_keys, &self.destination_ata) {
            if let Some(balance) = balances.get(dest_index) {
                if self.destination_mint == Pubkey::default() {
                    self.destination_mint = balance.mint;
                }
                if let Some(program) = balance.token_program {
                    if self.destination_token_program == Pubkey::default()
                        || self.destination_token_program != program
                    {
                        self.destination_token_program = program;
                    }
                }
            }
        }

        if self.source_token_program == Pubkey::default()
            || self.destination_token_program == Pubkey::default()
        {
            return Err(anyhow!("缺少 token program 信息"));
        }
        if self.source_mint == Pubkey::default() {
            return Err(anyhow!("缺少 source mint"));
        }
        if self.destination_mint == Pubkey::default() {
            return Err(anyhow!("缺少 destination mint"));
        }
        Ok(())
    }
}

impl RouteParams {
    fn parse(kind: RouteKind, data: &[u8]) -> Self {
        match kind {
            RouteKind::RouteV2 => parse_route_v2(data).unwrap_or_default(),
            RouteKind::ExactRouteV2 => parse_exact_route_v2(data).unwrap_or_default(),
            RouteKind::SharedRouteV2 => parse_shared_route_v2(data).unwrap_or_default(),
            RouteKind::Route => parse_route_legacy(data).unwrap_or_default(),
            RouteKind::Other => RouteParams::default(),
        }
    }
}

fn parse_route_v2(data: &[u8]) -> Option<RouteParams> {
    let mut rest = data.get(8..)?;
    let in_amount = read_u64(&mut rest)?;
    let quoted_out_amount = read_u64(&mut rest)?;
    let slippage_bps = read_u16(&mut rest)?;
    let platform_fee_bps = read_u16(&mut rest)?;
    let positive_slippage_bps = read_u16(&mut rest)?;
    Some(RouteParams {
        in_amount: Some(in_amount),
        quoted_out_amount: Some(quoted_out_amount),
        slippage_bps: Some(slippage_bps),
        platform_fee_bps: Some(platform_fee_bps),
        positive_slippage_bps: Some(positive_slippage_bps),
        ..RouteParams::default()
    })
}

fn parse_exact_route_v2(data: &[u8]) -> Option<RouteParams> {
    let mut rest = data.get(8..)?;
    let out_amount = read_u64(&mut rest)?;
    let quoted_in_amount = read_u64(&mut rest)?;
    let slippage_bps = read_u16(&mut rest)?;
    let platform_fee_bps = read_u16(&mut rest)?;
    let positive_slippage_bps = read_u16(&mut rest)?;
    Some(RouteParams {
        out_amount: Some(out_amount),
        quoted_in_amount: Some(quoted_in_amount),
        slippage_bps: Some(slippage_bps),
        platform_fee_bps: Some(platform_fee_bps),
        positive_slippage_bps: Some(positive_slippage_bps),
        ..RouteParams::default()
    })
}

fn parse_shared_route_v2(data: &[u8]) -> Option<RouteParams> {
    let mut rest = data.get(8..)?;
    let route_id = read_u8(&mut rest)?;
    let first_amount = read_u64(&mut rest)?;
    let second_amount = read_u64(&mut rest)?;
    let slippage_bps = read_u16(&mut rest)?;
    let platform_fee_bps = read_u16(&mut rest)?;
    let positive_slippage_bps = read_u16(&mut rest)?;
    Some(RouteParams {
        route_id: Some(route_id),
        in_amount: Some(first_amount),
        quoted_out_amount: Some(second_amount),
        slippage_bps: Some(slippage_bps),
        platform_fee_bps: Some(platform_fee_bps),
        positive_slippage_bps: Some(positive_slippage_bps),
        ..RouteParams::default()
    })
}

fn parse_route_legacy(data: &[u8]) -> Option<RouteParams> {
    let mut rest = data.get(8..)?;
    // Skip route_plan vec<RoutePlanStep>. We only handle empty plans; otherwise parsing is complex.
    let len = read_u32(&mut rest)? as usize;
    if len != 0 {
        return Some(RouteParams::default());
    }
    let in_amount = read_u64(&mut rest)?;
    let quoted_out_amount = read_u64(&mut rest)?;
    let slippage_bps = read_u16(&mut rest)?;
    let platform_fee_bps = read_u8(&mut rest)? as u16;
    Some(RouteParams {
        in_amount: Some(in_amount),
        quoted_out_amount: Some(quoted_out_amount),
        slippage_bps: Some(slippage_bps),
        platform_fee_bps: Some(platform_fee_bps),
        ..RouteParams::default()
    })
}

fn read_u8(input: &mut &[u8]) -> Option<u8> {
    if input.len() < 1 {
        None
    } else {
        let (value, rest) = input.split_at(1);
        *input = rest;
        Some(value[0])
    }
}

fn read_u16(input: &mut &[u8]) -> Option<u16> {
    if input.len() < 2 {
        None
    } else {
        let (value, rest) = input.split_at(2);
        *input = rest;
        Some(u16::from_le_bytes(value.try_into().ok()?))
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

fn read_u64(input: &mut &[u8]) -> Option<u64> {
    if input.len() < 8 {
        None
    } else {
        let (value, rest) = input.split_at(8);
        *input = rest;
        Some(u64::from_le_bytes(value.try_into().ok()?))
    }
}

pub(crate) fn find_account_index(account_keys: &[Pubkey], target: &Pubkey) -> Option<usize> {
    account_keys.iter().position(|key| key == target)
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
    pub fn get(&self, index: usize) -> Option<&TokenBalanceEntry> {
        self.by_index.get(&index)
    }

    pub fn entries(&self) -> impl Iterator<Item = &TokenBalanceEntry> {
        self.by_index.values()
    }
}

impl TryFrom<&confirmed_block::TransactionStatusMeta> for TransactionTokenBalances {
    type Error = anyhow::Error;

    fn try_from(meta: &confirmed_block::TransactionStatusMeta) -> Result<Self> {
        let mut balances = HashMap::new();
        for balance in &meta.pre_token_balances {
            if let Some(entry) = TokenBalanceEntry::parse(balance) {
                balances.entry(entry.account_index).or_insert(entry);
            }
        }
        for balance in &meta.post_token_balances {
            if let Some(entry) = TokenBalanceEntry::parse(balance) {
                balances.insert(entry.account_index, entry);
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
}

impl TokenBalanceEntry {
    fn parse(balance: &confirmed_block::TokenBalance) -> Option<Self> {
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

        Some(Self {
            account_index: balance.account_index as usize,
            mint,
            token_program,
            owner,
        })
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
    fn populate_route_context_uses_grpc_balances() {
        let authority = pk(1);
        let source_ata = pk(2);
        let destination_ata = pk(3);
        let destination_mint = pk(4);
        let source_mint = pk(5);
        let token_program = pk(6);

        let mut route = RouteContext {
            authority,
            source_ata,
            destination_ata,
            source_mint: Pubkey::default(),
            destination_mint,
            source_token_program: token_program,
            destination_token_program: token_program,
            params: RouteParams::default(),
        };

        let account_keys = vec![
            token_program,
            authority,
            source_ata,
            destination_ata,
            destination_mint,
        ];

        let mut meta = confirmed_block::TransactionStatusMeta::default();
        meta.pre_token_balances.push(confirmed_block::TokenBalance {
            account_index: 2,
            mint: source_mint.to_string(),
            ui_token_amount: None,
            owner: authority.to_string(),
            program_id: token_program.to_string(),
        });

        let balances = TransactionTokenBalances::try_from(&meta).expect("token balances");
        route
            .populate_from_balances(&account_keys, Some(&balances))
            .expect("populate route context");

        assert_eq!(route.source_mint, source_mint);
        assert_eq!(route.destination_mint, destination_mint);
        assert_eq!(route.source_token_program, token_program);
        assert_eq!(route.destination_token_program, token_program);
    }

    #[test]
    fn populate_route_context_errors_without_balances() {
        let authority = pk(9);
        let source_ata = pk(10);
        let destination_ata = pk(11);
        let token_program = pk(12);
        let mut route = RouteContext {
            authority,
            source_ata,
            destination_ata,
            source_mint: Pubkey::default(),
            destination_mint: pk(13),
            source_token_program: token_program,
            destination_token_program: token_program,
            params: RouteParams::default(),
        };
        let account_keys = vec![token_program, authority, source_ata, destination_ata];
        let result = route.populate_from_balances(&account_keys, None);
        assert!(result.is_err());
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

    fn exact_route_v2_data(out_amount: u64, quoted_in: u64, slippage: u16) -> Vec<u8> {
        let mut data = crate::jupiter_parser::discriminator::EXACT_ROUTE_V2.to_vec();
        data.extend_from_slice(&out_amount.to_le_bytes());
        data.extend_from_slice(&quoted_in.to_le_bytes());
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

    #[test]
    fn route_v2_params_are_parsed() {
        let data = route_v2_data(100, 120, 50);
        let instruction = Instruction {
            program_id: JUPITER_V6_PROGRAM_ID,
            accounts: build_route_accounts(),
            data,
        };
        let ctx = RouteContext::from_instruction(&instruction).expect("route context");
        assert_eq!(ctx.params.in_amount, Some(100));
        assert_eq!(ctx.params.quoted_out_amount, Some(120));
        assert_eq!(ctx.params.slippage_bps, Some(50));
    }

    #[test]
    fn exact_route_v2_params_are_parsed() {
        let data = exact_route_v2_data(200, 180, 25);
        let instruction = Instruction {
            program_id: JUPITER_V6_PROGRAM_ID,
            accounts: build_route_accounts(),
            data,
        };
        let ctx = RouteContext::from_instruction(&instruction).expect("route context");
        assert_eq!(ctx.params.out_amount, Some(200));
        assert_eq!(ctx.params.quoted_in_amount, Some(180));
        assert_eq!(ctx.params.slippage_bps, Some(25));
    }

    #[test]
    fn shared_route_v2_params_are_parsed() {
        let data = shared_route_v2_data(7, 300, 330, 15);
        let instruction = Instruction {
            program_id: JUPITER_V6_PROGRAM_ID,
            accounts: build_route_accounts(),
            data,
        };
        let ctx = RouteContext::from_instruction(&instruction).expect("route context");
        assert_eq!(ctx.params.route_id, Some(7));
        assert_eq!(ctx.params.in_amount, Some(300));
        assert_eq!(ctx.params.quoted_out_amount, Some(330));
        assert_eq!(ctx.params.slippage_bps, Some(15));
    }
}
