use std::collections::HashMap;

use solana_compute_budget_interface as compute_budget;
use solana_message::{
    VersionedMessage, compiled_instruction::CompiledInstruction, v0::MessageAddressTableLookup,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::{AddressLookupTableAccount, MessageHeader},
    pubkey::Pubkey,
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct InstructionBundle {
    pub compute_budget_instructions: Vec<Instruction>,
    pub other_instructions: Vec<Instruction>,
}

pub fn rewrite_instruction_accounts_map(
    instructions: &mut [Instruction],
    rewrites: &[(Pubkey, Pubkey)],
) {
    if rewrites.is_empty() {
        return;
    }
    for ix in instructions {
        for account in &mut ix.accounts {
            for (from, to) in rewrites {
                if account.pubkey == *from {
                    account.pubkey = *to;
                    break;
                }
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum InstructionExtractionError {
    #[error("需要 {count} 个地址查找表，但尚未解析")]
    MissingLookupTables { count: usize },
    #[error("找不到地址查找表 {table}")]
    LookupTableNotFound { table: Pubkey },
    #[error("地址查找表 {table} 索引 {index} 超出范围 (len = {len})")]
    LookupIndexOutOfBounds {
        table: Pubkey,
        index: u8,
        len: usize,
    },
    #[error("指令 program index {index} 超出账户数量 {total}")]
    ProgramIndexOutOfBounds { index: usize, total: usize },
    #[error("指令 account index {index} 超出账户数量 {total}")]
    AccountIndexOutOfBounds { index: usize, total: usize },
}

#[derive(Debug, Clone)]
struct AccountKeyInfo {
    pubkey: Pubkey,
    is_signer: bool,
    is_writable: bool,
}

pub fn extract_instructions(
    message: &VersionedMessage,
    resolved_tables: Option<&[AddressLookupTableAccount]>,
) -> Result<InstructionBundle, InstructionExtractionError> {
    let (account_keys, uses_lookup_tables) = build_account_keys(message, resolved_tables)?;
    let instructions = convert_instructions(message, &account_keys)?;
    let (compute_budget_instructions, other_instructions): (Vec<_>, Vec<_>) = instructions
        .into_iter()
        .partition(|ix| ix.program_id == compute_budget::id());

    // 当含有查找表但未解析时，提前返回 MissingLookupTables。
    if uses_lookup_tables && resolved_tables.is_none() {
        return Err(InstructionExtractionError::MissingLookupTables {
            count: lookup_table_count(message),
        });
    }

    Ok(InstructionBundle {
        compute_budget_instructions,
        other_instructions,
    })
}

fn build_account_keys(
    message: &VersionedMessage,
    resolved_tables: Option<&[AddressLookupTableAccount]>,
) -> Result<(Vec<AccountKeyInfo>, bool), InstructionExtractionError> {
    match message {
        VersionedMessage::Legacy(legacy) => {
            let header = &legacy.header;
            let total = legacy.account_keys.len();
            let mut result = Vec::with_capacity(total);
            for (idx, pubkey) in legacy.account_keys.iter().enumerate() {
                result.push(AccountKeyInfo {
                    pubkey: *pubkey,
                    is_signer: is_signer(idx, header),
                    is_writable: is_writable(idx, header, total),
                });
            }
            Ok((result, false))
        }
        VersionedMessage::V0(v0) => {
            let header = &v0.header;
            let static_total = v0.account_keys.len();
            let mut infos = Vec::with_capacity(static_total);
            for (idx, pubkey) in v0.account_keys.iter().enumerate() {
                infos.push(AccountKeyInfo {
                    pubkey: *pubkey,
                    is_signer: is_signer(idx, header),
                    is_writable: is_writable(idx, header, static_total),
                });
            }

            if v0.address_table_lookups.is_empty() {
                return Ok((infos, false));
            }

            let tables =
                resolved_tables.ok_or(InstructionExtractionError::MissingLookupTables {
                    count: v0.address_table_lookups.len(),
                })?;
            let mut table_map: HashMap<Pubkey, &AddressLookupTableAccount> =
                tables.iter().map(|table| (table.key, table)).collect();

            append_lookup_accounts(&mut infos, &v0.address_table_lookups, &mut table_map, true)?;
            append_lookup_accounts(&mut infos, &v0.address_table_lookups, &mut table_map, false)?;

            Ok((infos, true))
        }
    }
}

fn append_lookup_accounts(
    infos: &mut Vec<AccountKeyInfo>,
    lookups: &[MessageAddressTableLookup],
    table_map: &mut HashMap<Pubkey, &AddressLookupTableAccount>,
    writable: bool,
) -> Result<(), InstructionExtractionError> {
    for lookup in lookups {
        let table = table_map.get(&lookup.account_key).copied().ok_or(
            InstructionExtractionError::LookupTableNotFound {
                table: lookup.account_key,
            },
        )?;
        let indexes = if writable {
            &lookup.writable_indexes
        } else {
            &lookup.readonly_indexes
        };
        for index in indexes {
            let address = table.addresses.get(*index as usize).ok_or(
                InstructionExtractionError::LookupIndexOutOfBounds {
                    table: lookup.account_key,
                    index: *index,
                    len: table.addresses.len(),
                },
            )?;
            infos.push(AccountKeyInfo {
                pubkey: *address,
                is_signer: false,
                is_writable: writable,
            });
        }
    }
    Ok(())
}

fn convert_instructions(
    message: &VersionedMessage,
    account_keys: &[AccountKeyInfo],
) -> Result<Vec<Instruction>, InstructionExtractionError> {
    let compiled = match message {
        VersionedMessage::Legacy(legacy) => &legacy.instructions,
        VersionedMessage::V0(v0) => &v0.instructions,
    };

    compiled
        .iter()
        .map(|ix| convert_single_instruction(ix, account_keys))
        .collect()
}

fn convert_single_instruction(
    ix: &CompiledInstruction,
    account_keys: &[AccountKeyInfo],
) -> Result<Instruction, InstructionExtractionError> {
    let program_index = ix.program_id_index as usize;
    let program = account_keys.get(program_index).ok_or(
        InstructionExtractionError::ProgramIndexOutOfBounds {
            index: program_index,
            total: account_keys.len(),
        },
    )?;

    let mut accounts = Vec::with_capacity(ix.accounts.len());
    for account_index in &ix.accounts {
        let idx = *account_index as usize;
        let key_info =
            account_keys
                .get(idx)
                .ok_or(InstructionExtractionError::AccountIndexOutOfBounds {
                    index: idx,
                    total: account_keys.len(),
                })?;
        accounts.push(AccountMeta {
            pubkey: key_info.pubkey,
            is_signer: key_info.is_signer,
            is_writable: key_info.is_writable,
        });
    }

    Ok(Instruction {
        program_id: program.pubkey,
        accounts,
        data: ix.data.clone(),
    })
}

fn lookup_table_count(message: &VersionedMessage) -> usize {
    match message {
        VersionedMessage::Legacy(_) => 0,
        VersionedMessage::V0(v0) => v0.address_table_lookups.len(),
    }
}

fn is_signer(index: usize, header: &MessageHeader) -> bool {
    index < header.num_required_signatures as usize
}

fn is_writable(index: usize, header: &MessageHeader, total_keys: usize) -> bool {
    let num_required_signatures = header.num_required_signatures as usize;
    let writable_signed =
        num_required_signatures.saturating_sub(header.num_readonly_signed_accounts as usize);
    if index < num_required_signatures {
        return index < writable_signed;
    }

    let num_unsigned = total_keys.saturating_sub(num_required_signatures);
    let writable_unsigned =
        num_unsigned.saturating_sub(header.num_readonly_unsigned_accounts as usize);
    let unsigned_index = index.saturating_sub(num_required_signatures);
    unsigned_index < writable_unsigned
}
