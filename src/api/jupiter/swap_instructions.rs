use std::collections::HashMap;
use std::str::FromStr;

use serde::de::Error as SerdeDeError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::message::{AddressLookupTableAccount, VersionedMessage};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::VersionedTransaction;
use thiserror::Error;

use super::quote::QuoteResponsePayload;
use crate::api::serde_helpers::{field_as_string, hash_as_string_or_bytes};
use crate::engine::multi_leg::transaction::decoder::decode_base64_transaction;
use crate::engine::multi_leg::transaction::instructions::{
    InstructionBundle, InstructionExtractionError, extract_instructions,
};

/// `/swap-instructions` 请求。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapInstructionsRequest {
    #[serde(rename = "quoteResponse")]
    pub quote_response: QuoteResponsePayload,
    #[serde(with = "field_as_string")]
    pub user_public_key: Pubkey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_token_account: Option<Pubkey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_account: Option<Pubkey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_unit_price_micro_lamports: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prioritization_fee_lamports: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_user_accounts_rpc_calls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_compute_unit_limit: Option<bool>,
    pub wrap_and_unwrap_sol: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_shared_accounts: Option<bool>,
}

impl SwapInstructionsRequest {
    pub fn from_quote(response: QuoteResponsePayload, user: Pubkey) -> Self {
        Self {
            quote_response: response,
            user_public_key: user,
            destination_token_account: None,
            fee_account: None,
            compute_unit_price_micro_lamports: None,
            prioritization_fee_lamports: None,
            skip_user_accounts_rpc_calls: None,
            dynamic_compute_unit_limit: None,
            wrap_and_unwrap_sol: true,
            use_shared_accounts: None,
        }
    }
}

/// 响应中的 blockhash 信息。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BlockhashMetadata {
    #[serde(with = "hash_as_string_or_bytes")]
    pub blockhash: Hash,
    pub last_valid_block_height: u64,
}

/// `/swap-instructions` 响应。
#[derive(Debug, Clone)]
pub struct SwapInstructionsResponse {
    #[allow(dead_code)]
    pub raw: Value,
    pub compute_budget_instructions: Vec<Instruction>,
    pub setup_instructions: Vec<Instruction>,
    pub swap_instruction: Instruction,
    pub cleanup_instructions: Vec<Instruction>,
    pub other_instructions: Vec<Instruction>,
    pub address_lookup_table_addresses: Vec<Pubkey>,
    pub prioritization_fee_lamports: Option<u64>,
    pub compute_unit_limit: u32,
    #[allow(dead_code)]
    pub blockhash: Option<BlockhashMetadata>,
    pub ordered_instructions: Option<Vec<Instruction>>,
    pub decoded_transaction: Option<VersionedTransaction>,
}

impl SwapInstructionsResponse {
    pub fn try_from(value: Value) -> Result<Self, serde_json::Error> {
        if value.get("swapTransaction").is_some() {
            return Self::from_swap_transaction(value);
        }
        let internal: SwapInstructionsResponseInternal = serde_json::from_value(value.clone())?;
        Ok(Self::from_internal(value, internal))
    }

    fn from_internal(raw: Value, value: SwapInstructionsResponseInternal) -> Self {
        let blockhash = value
            .blockhash_with_expiry_block_height
            .or_else(|| value.blockhash_with_metadata.clone())
            .or_else(|| {
                value.recent_blockhash.map(|hash| BlockhashMetadata {
                    blockhash: hash,
                    last_valid_block_height: value.last_valid_block_height.unwrap_or_default(),
                })
            });

        Self {
            raw,
            compute_budget_instructions: value
                .compute_budget_instructions
                .into_iter()
                .map(Into::into)
                .collect(),
            setup_instructions: value
                .setup_instructions
                .into_iter()
                .map(Into::into)
                .collect(),
            swap_instruction: value.swap_instruction.into(),
            cleanup_instructions: value
                .cleanup_instructions
                .into_iter()
                .map(Into::into)
                .collect(),
            other_instructions: value
                .other_instructions
                .into_iter()
                .map(Into::into)
                .collect(),
            address_lookup_table_addresses: value
                .address_lookup_table_addresses
                .into_iter()
                .map(|wrapper| wrapper.0)
                .collect(),
            prioritization_fee_lamports: value.prioritization_fee_lamports,
            compute_unit_limit: value
                .compute_unit_limit
                .unwrap_or(DEFAULT_COMPUTE_UNIT_LIMIT),
            blockhash,
            ordered_instructions: None,
            decoded_transaction: None,
        }
    }

    fn from_swap_transaction(raw: Value) -> Result<Self, serde_json::Error> {
        let response: SwapTransactionEnvelope = serde_json::from_value(raw.clone())?;
        let transaction = decode_base64_transaction(&response.swap_transaction)
            .map_err(|err| SerdeDeError::custom(format!("swapTransaction 解码失败: {err}")))?;
        let lookup_accounts = build_lookup_accounts(&response.addresses_by_lookup_table_address)?;
        let lookup_slice = (!lookup_accounts.is_empty()).then_some(lookup_accounts.as_slice());

        let decode_result = match extract_instructions(&transaction.message, lookup_slice) {
            Ok(bundle) => Some(bundle),
            Err(InstructionExtractionError::MissingLookupTables { .. }) => None,
            Err(err) => {
                return Err(SerdeDeError::custom(format!(
                    "swapTransaction 指令解析失败: {err}"
                )));
            }
        };

        let address_lookup_table_addresses = collect_lookup_addresses(&transaction.message);
        let blockhash = transaction.message.recent_blockhash();
        let blockhash_meta = response
            .last_valid_block_height
            .map(|height| BlockhashMetadata {
                blockhash: *blockhash,
                last_valid_block_height: height,
            });

        let mut instance = Self {
            raw,
            compute_budget_instructions: Vec::new(),
            setup_instructions: Vec::new(),
            swap_instruction: Instruction {
                program_id: Pubkey::default(),
                accounts: Vec::new(),
                data: Vec::new(),
            },
            cleanup_instructions: Vec::new(),
            other_instructions: Vec::new(),
            address_lookup_table_addresses,
            prioritization_fee_lamports: response.prioritization_fee_lamports,
            compute_unit_limit: response
                .compute_unit_limit
                .unwrap_or(DEFAULT_COMPUTE_UNIT_LIMIT),
            blockhash: blockhash_meta,
            ordered_instructions: None,
            decoded_transaction: Some(transaction),
        };

        if let Some(bundle) = decode_result {
            instance
                .install_bundle(bundle)
                .map_err(|err| SerdeDeError::custom(err.to_string()))?;
            if let Some(limit) = instance
                .compute_budget_instructions
                .iter()
                .find_map(parse_compute_budget_limit)
            {
                instance.compute_unit_limit = limit;
            }
        }

        Ok(instance)
    }

    pub fn flatten_instructions(&self) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        instructions.extend(self.compute_budget_instructions.iter().cloned());
        if let Some(ordered) = &self.ordered_instructions {
            instructions.extend(ordered.iter().cloned());
        } else {
            instructions.extend(self.setup_instructions.iter().cloned());
            instructions.push(self.swap_instruction.clone());
            instructions.extend(self.other_instructions.iter().cloned());
            instructions.extend(self.cleanup_instructions.iter().cloned());
        }
        instructions
    }

    pub fn main_instructions(&self) -> Vec<Instruction> {
        if let Some(ordered) = &self.ordered_instructions {
            return ordered.clone();
        }
        let mut instructions = Vec::new();
        instructions.extend(self.setup_instructions.iter().cloned());
        instructions.push(self.swap_instruction.clone());
        instructions.extend(self.cleanup_instructions.iter().cloned());
        instructions.extend(self.other_instructions.iter().cloned());
        instructions
    }

    pub fn adjust_compute_unit_limit(&mut self, multiplier: f64) -> u32 {
        let sanitized = if multiplier.is_finite() && multiplier > 0.0 {
            multiplier
        } else {
            1.0
        };
        let base_limit = self.compute_unit_limit.max(1);
        let mut scaled = (base_limit as f64) * sanitized;
        if !scaled.is_finite() {
            scaled = base_limit as f64;
        }
        if scaled < 1.0 {
            scaled = 1.0;
        }
        if scaled > u32::MAX as f64 {
            scaled = u32::MAX as f64;
        }
        let scaled_limit = scaled.round() as u32;
        if scaled_limit == self.compute_unit_limit {
            return self.compute_unit_limit;
        }
        self.compute_unit_limit = scaled_limit.max(1);

        let mut replaced = false;
        for ix in self.compute_budget_instructions.iter_mut() {
            if parse_compute_budget_limit(ix).is_some() {
                *ix = ComputeBudgetInstruction::set_compute_unit_limit(self.compute_unit_limit);
                replaced = true;
            }
        }

        if !replaced {
            self.compute_budget_instructions.insert(
                0,
                ComputeBudgetInstruction::set_compute_unit_limit(self.compute_unit_limit),
            );
        }

        self.compute_unit_limit
    }

    #[allow(dead_code)]
    pub fn compute_budget_overrides(&self) -> Vec<Instruction> {
        self.compute_budget_instructions.iter().cloned().collect()
    }

    pub fn needs_lookup_resolution(&self) -> bool {
        self.ordered_instructions.is_none() && self.decoded_transaction.is_some()
    }

    pub fn resolve_with_lookup_accounts(
        &mut self,
        tables: &[AddressLookupTableAccount],
    ) -> Result<(), SwapDecodeError> {
        if self.ordered_instructions.is_some() {
            return Ok(());
        }
        let transaction = match &self.decoded_transaction {
            Some(tx) => tx,
            None => return Ok(()),
        };
        let bundle = extract_instructions(&transaction.message, Some(tables))
            .map_err(SwapDecodeError::Instruction)?;
        self.install_bundle(bundle)
    }

    fn install_bundle(&mut self, bundle: InstructionBundle) -> Result<(), SwapDecodeError> {
        let InstructionBundle {
            compute_budget_instructions,
            other_instructions,
        } = bundle;

        if other_instructions.is_empty() {
            return Err(SwapDecodeError::MissingMainInstruction);
        }

        self.compute_budget_instructions = compute_budget_instructions;
        let ordered = other_instructions;
        let mut staged = ordered.clone();
        let swap_instruction = staged
            .pop()
            .ok_or(SwapDecodeError::MissingMainInstruction)?;

        self.setup_instructions = staged;
        self.swap_instruction = swap_instruction;
        self.cleanup_instructions.clear();
        self.other_instructions.clear();
        self.ordered_instructions = Some(ordered);

        Ok(())
    }
}

const DEFAULT_COMPUTE_UNIT_LIMIT: u32 = 200_000;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwapInstructionsResponseInternal {
    #[serde(default)]
    compute_budget_instructions: Vec<InstructionInternal>,
    #[serde(default)]
    setup_instructions: Vec<InstructionInternal>,
    swap_instruction: InstructionInternal,
    #[serde(default)]
    cleanup_instructions: Vec<InstructionInternal>,
    #[serde(default)]
    other_instructions: Vec<InstructionInternal>,
    #[serde(default)]
    address_lookup_table_addresses: Vec<PubkeyWrapper>,
    #[serde(default)]
    prioritization_fee_lamports: Option<u64>,
    #[serde(default)]
    compute_unit_limit: Option<u32>,
    #[serde(default)]
    blockhash_with_metadata: Option<BlockhashMetadata>,
    #[serde(default)]
    blockhash_with_expiry_block_height: Option<BlockhashMetadata>,
    #[serde(default)]
    recent_blockhash: Option<Hash>,
    #[serde(default)]
    last_valid_block_height: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstructionInternal {
    #[serde(with = "field_as_string")]
    program_id: Pubkey,
    accounts: Vec<AccountMetaInternal>,
    #[serde(with = "base64_serde")]
    data: Vec<u8>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AccountMetaInternal {
    #[serde(with = "field_as_string")]
    pubkey: Pubkey,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PubkeyWrapper(#[serde(with = "field_as_string")] Pubkey);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SwapTransactionEnvelope {
    #[serde(rename = "swapTransaction")]
    swap_transaction: String,
    #[serde(default)]
    last_valid_block_height: Option<u64>,
    #[serde(default)]
    prioritization_fee_lamports: Option<u64>,
    #[serde(default)]
    compute_unit_limit: Option<u32>,
    #[serde(default)]
    addresses_by_lookup_table_address: Option<HashMap<String, Vec<String>>>,
}

#[derive(Debug, Error)]
pub enum SwapDecodeError {
    #[error("swapTransaction 指令解析失败: {0}")]
    Instruction(#[from] InstructionExtractionError),
    #[error("swapTransaction 未返回可执行指令")]
    MissingMainInstruction,
}

impl From<AccountMetaInternal> for AccountMeta {
    fn from(value: AccountMetaInternal) -> Self {
        Self {
            pubkey: value.pubkey,
            is_signer: value.is_signer,
            is_writable: value.is_writable,
        }
    }
}

impl From<InstructionInternal> for Instruction {
    fn from(value: InstructionInternal) -> Self {
        Self {
            program_id: value.program_id,
            accounts: value.accounts.into_iter().map(Into::into).collect(),
            data: value.data,
        }
    }
}

/// Jupiter 返回的 compute budget 指令均来自 ComputeBudget 程序。
pub fn parse_compute_budget_limit(instruction: &Instruction) -> Option<u32> {
    if instruction.program_id != solana_compute_budget_interface::id() {
        return None;
    }
    if instruction.data.first().copied() != Some(2) || instruction.data.len() < 5 {
        return None;
    }
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&instruction.data[1..5]);
    Some(u32::from_le_bytes(buf))
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

fn build_lookup_accounts(
    entries: &Option<HashMap<String, Vec<String>>>,
) -> Result<Vec<AddressLookupTableAccount>, serde_json::Error> {
    let mut accounts = Vec::new();
    if let Some(map) = entries {
        for (table_key, addresses) in map {
            let key = Pubkey::from_str(table_key).map_err(|err| {
                SerdeDeError::custom(format!("地址查找表 {table_key} 公钥解析失败: {err}"))
            })?;
            let mut resolved = Vec::with_capacity(addresses.len());
            for entry in addresses {
                let addr = Pubkey::from_str(entry).map_err(|err| {
                    SerdeDeError::custom(format!(
                        "地址查找表 {table_key} 中地址 {entry} 解析失败: {err}"
                    ))
                })?;
                resolved.push(addr);
            }
            accounts.push(AddressLookupTableAccount {
                key,
                addresses: resolved,
            });
        }
    }
    Ok(accounts)
}

#[allow(dead_code)]
mod base64_serde {
    use base64::{Engine, engine::general_purpose};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(value: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
        let encoded = general_purpose::STANDARD.encode(value);
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let raw = String::deserialize(deserializer)?;
        general_purpose::STANDARD
            .decode(raw.as_bytes())
            .map_err(|err| serde::de::Error::custom(format!("base64 decode error: {err}")))
    }
}
