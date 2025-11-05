use super::quote::QuoteResponsePayload;
use crate::api::serde_helpers::{field_as_string, hash_as_string_or_bytes};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::hash::Hash;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;

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
}

impl SwapInstructionsResponse {
    pub fn try_from(value: Value) -> Result<Self, serde_json::Error> {
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
        }
    }

    pub fn flatten_instructions(&self) -> Vec<Instruction> {
        let mut instructions = Vec::new();
        instructions.extend(self.compute_budget_instructions.iter().cloned());
        instructions.extend(self.setup_instructions.iter().cloned());
        instructions.push(self.swap_instruction.clone());
        instructions.extend(self.other_instructions.iter().cloned());
        instructions.extend(self.cleanup_instructions.iter().cloned());
        instructions
    }

    #[allow(dead_code)]
    pub fn compute_budget_overrides(&self) -> Vec<Instruction> {
        self.compute_budget_instructions.iter().cloned().collect()
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
