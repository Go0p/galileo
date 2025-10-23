use super::quote::QuoteResponsePayload;
use super::serde_helpers::{field_as_string, option_field_as_string};
pub use crate::api::jupiter::swap_instructions::{PrioritizationType, PriorityLevel};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::{
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    message::AddressLookupTableAccount,
    pubkey::Pubkey,
};

/// `computeUnitPriceMicroLamports` 的包装类型。
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(transparent)]
pub struct ComputeUnitPriceMicroLamports(pub u64);

/// `createFeeAccount` 配置。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateFeeAccount {
    #[serde(with = "field_as_string")]
    pub referral_account: Pubkey,
}

/// `destinationTokenAccount` 变体：使用 owner 的关联账户。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DestinationAssociatedTokenAccount {
    #[serde(with = "field_as_string")]
    pub owner: Pubkey,
}

/// `destinationTokenAccount` 的对象形式。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DestinationTokenAccountViaOwner {
    pub associated_token_account: DestinationAssociatedTokenAccount,
}

/// 指定正滑点返还配置。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PositiveSlippageConfig {
    pub limit_pct: u32,
    #[serde(with = "option_field_as_string")]
    pub fee_account: Option<Pubkey>,
}

/// prioritizationFeeLamports 预设。
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PrioritizationFeePreset {
    Auto,
    Disabled,
}

/// priorityLevelWithMaxLamports。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PriorityLevelWithMaxLamports {
    pub priority_level: PriorityLevel,
    pub max_lamports: u64,
}

/// prioritizationFeeLamports 的对象形式。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct PrioritizationFeeLamportsConfig {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub priority_level_with_max_lamports: Option<PriorityLevelWithMaxLamports>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub auto_multiplier: Option<u32>,
}

/// prioritizationFeeLamports 支持的多种结构。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum PrioritizationFeeLamports {
    Fixed(u64),
    Preset(PrioritizationFeePreset),
    Config(PrioritizationFeeLamportsConfig),
}

/// destinationTokenAccount 允许字符串或对象两类。
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum DestinationTokenAccount {
    ViaOwner(DestinationTokenAccountViaOwner),
    Address(#[serde(with = "field_as_string")] Pubkey),
}

/// `/swap-instructions` 请求体。
#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapInstructionsRequest {
    #[serde(rename = "quoteResponse")]
    pub quote_response: QuoteResponsePayload,
    #[serde(with = "field_as_string")]
    pub user_public_key: Pubkey,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compute_unit_price_micro_lamports: Option<ComputeUnitPriceMicroLamports>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub create_fee_account: Option<CreateFeeAccount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_token_account: Option<DestinationTokenAccount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_compute_unit_limit: Option<bool>,
    #[serde(with = "option_field_as_string")]
    pub fee_account: Option<Pubkey>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_jito_sandwich_mitigation_account: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positive_slippage: Option<PositiveSlippageConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_input_ata: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prioritization_fee_lamports: Option<PrioritizationFeeLamports>,
    #[serde(with = "option_field_as_string")]
    pub sponsor: Option<Pubkey>,
    pub wrap_and_unwrap_sol: bool,
}

impl SwapInstructionsRequest {
    pub fn from_payload(quote_response: QuoteResponsePayload, user_public_key: Pubkey) -> Self {
        Self {
            quote_response,
            user_public_key,
            compute_unit_price_micro_lamports: None,
            create_fee_account: None,
            destination_token_account: None,
            dynamic_compute_unit_limit: None,
            fee_account: None,
            include_jito_sandwich_mitigation_account: None,
            positive_slippage: None,
            preserve_input_ata: None,
            prioritization_fee_lamports: None,
            sponsor: None,
            wrap_and_unwrap_sol: true,
        }
    }
}

/// 响应中的 blockhash 元数据。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BlockhashWithMetadata {
    #[serde(with = "field_as_string")]
    pub blockhash: Hash,
    pub last_valid_block_height: u64,
}

/// `/swap-instructions` 响应体，解析为 Solana 指令集合。
#[derive(Debug, Clone)]
pub struct SwapInstructionsResponse {
    pub raw: Value,
    pub compute_budget_instructions: Vec<Instruction>,
    pub setup_instructions: Vec<Instruction>,
    pub swap_instruction: Instruction,
    pub cleanup_instructions: Vec<Instruction>,
    pub other_instructions: Vec<Instruction>,
    pub address_lookup_table_addresses: Vec<Pubkey>,
    pub resolved_lookup_tables: Vec<AddressLookupTableAccount>,
    pub blockhash_with_metadata: BlockhashWithMetadata,
    pub prioritization_fee_lamports: Option<u64>,
    pub compute_unit_limit: u32,
    pub prioritization_type: Option<PrioritizationType>,
}

#[derive(Deserialize)]
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
    address_lookup_table_addresses: Vec<PubkeyInternal>,
    blockhash_with_metadata: BlockhashWithMetadata,
    prioritization_fee_lamports: Option<u64>,
    compute_unit_limit: u32,
    prioritization_type: Option<PrioritizationType>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct InstructionInternal {
    #[serde(with = "field_as_string")]
    pub program_id: Pubkey,
    pub accounts: Vec<AccountMetaInternal>,
    #[serde(with = "base64_serde")]
    pub data: Vec<u8>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct AccountMetaInternal {
    #[serde(with = "field_as_string")]
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PubkeyInternal(#[serde(with = "field_as_string")] Pubkey);

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

impl SwapInstructionsResponse {
    fn from_internal(raw: Value, value: SwapInstructionsResponseInternal) -> Self {
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
                .map(|p| p.0)
                .collect(),
            resolved_lookup_tables: Vec::new(),
            blockhash_with_metadata: value.blockhash_with_metadata,
            prioritization_fee_lamports: value.prioritization_fee_lamports,
            compute_unit_limit: value.compute_unit_limit,
            prioritization_type: value.prioritization_type,
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
}

impl TryFrom<Value> for SwapInstructionsResponse {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let raw = value.clone();
        let internal: SwapInstructionsResponseInternal = serde_json::from_value(value)?;
        Ok(Self::from_internal(raw, internal))
    }
}

#[allow(dead_code)]
mod base64_serde {
    use base64::{Engine, engine::general_purpose::STANDARD};
    use serde::{Deserialize, Deserializer, Serializer, de};

    pub fn serialize<S: Serializer>(value: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error> {
        let encoded = STANDARD.encode(value);
        serializer.serialize_str(&encoded)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        STANDARD
            .decode(raw)
            .map_err(|err| de::Error::custom(format!("base64 decoding error: {err:?}")))
    }
}
