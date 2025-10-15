use crate::api::serde_helpers::field_as_string;
use crate::api::transaction_config::TransactionConfig;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapInstructionsRequest {
    #[serde(rename = "quoteResponse")]
    pub quote_response: Value,
    #[serde(with = "field_as_string")]
    pub user_public_key: Pubkey,
    #[serde(flatten)]
    pub config: TransactionConfig,
}

impl SwapInstructionsRequest {
    pub fn new(quote_response: Value, user_public_key: Pubkey) -> Self {
        Self {
            quote_response,
            user_public_key,
            config: TransactionConfig::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PrioritizationType {
    #[serde(rename_all = "camelCase")]
    Jito { lamports: u64 },
    #[serde(rename_all = "camelCase")]
    ComputeBudget {
        micro_lamports: u64,
        estimated_micro_lamports: Option<u64>,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicSlippageReport {
    pub slippage_bps: u16,
    pub other_amount: Option<u64>,
    /// Signed to convey positive and negative slippage.
    pub simulated_incurred_slippage_bps: Option<i16>,
    pub amplification_ratio: Option<Decimal>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UiSimulationError {
    pub error_code: String,
    pub error: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SwapInstructionsResponse {
    pub raw: Value,
    pub token_ledger_instruction: Option<Instruction>,
    pub compute_budget_instructions: Vec<Instruction>,
    pub setup_instructions: Vec<Instruction>,
    /// Instruction performing the action of swapping.
    pub swap_instruction: Instruction,
    pub cleanup_instruction: Option<Instruction>,
    /// Other instructions that should be included in the transaction.
    /// Now, it should only have the Jito tip instruction.
    pub other_instructions: Vec<Instruction>,
    pub address_lookup_table_addresses: Vec<Pubkey>,
    pub prioritization_fee_lamports: u64,
    pub compute_unit_limit: u32,
    pub prioritization_type: Option<PrioritizationType>,
    pub dynamic_slippage_report: Option<DynamicSlippageReport>,
    pub simulation_error: Option<UiSimulationError>,
}

// Duplicate for deserialization.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct SwapInstructionsResponseInternal {
    token_ledger_instruction: Option<InstructionInternal>,
    compute_budget_instructions: Vec<InstructionInternal>,
    setup_instructions: Vec<InstructionInternal>,
    /// Instruction performing the action of swapping.
    swap_instruction: InstructionInternal,
    cleanup_instruction: Option<InstructionInternal>,
    /// Other instructions that should be included in the transaction.
    /// Now, it should only have the Jito tip instruction.
    other_instructions: Vec<InstructionInternal>,
    address_lookup_table_addresses: Vec<PubkeyInternal>,
    prioritization_fee_lamports: u64,
    compute_unit_limit: u32,
    prioritization_type: Option<PrioritizationType>,
    dynamic_slippage_report: Option<DynamicSlippageReport>,
    simulation_error: Option<UiSimulationError>,
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

impl From<AccountMetaInternal> for AccountMeta {
    fn from(value: AccountMetaInternal) -> Self {
        Self {
            pubkey: value.pubkey,
            is_signer: value.is_signer,
            is_writable: value.is_writable,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct PubkeyInternal(#[serde(with = "field_as_string")] Pubkey);

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
            token_ledger_instruction: value.token_ledger_instruction.map(Into::into),
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
            cleanup_instruction: value.cleanup_instruction.map(Into::into),
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
            prioritization_fee_lamports: value.prioritization_fee_lamports,
            compute_unit_limit: value.compute_unit_limit,
            prioritization_type: value.prioritization_type,
            dynamic_slippage_report: value.dynamic_slippage_report,
            simulation_error: value.simulation_error,
        }
    }

    pub fn flatten_instructions(&self) -> Vec<Instruction> {
        let mut capacity = self.compute_budget_instructions.len()
            + self.setup_instructions.len()
            + self.other_instructions.len()
            + 2; // swap + cleanup or token ledger
        if self.token_ledger_instruction.is_some() {
            capacity += 1;
        }
        if self.cleanup_instruction.is_some() {
            capacity += 1;
        }

        let mut instructions = Vec::with_capacity(capacity);
        instructions.extend(self.compute_budget_instructions.iter().cloned());
        if let Some(ledger) = &self.token_ledger_instruction {
            instructions.push(ledger.clone());
        }
        instructions.extend(self.setup_instructions.iter().cloned());
        instructions.push(self.swap_instruction.clone());
        instructions.extend(self.other_instructions.iter().cloned());
        if let Some(cleanup) = &self.cleanup_instruction {
            instructions.push(cleanup.clone());
        }
        instructions
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

impl TryFrom<Value> for SwapInstructionsResponse {
    type Error = serde_json::Error;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let raw = value.clone();
        let internal: SwapInstructionsResponseInternal = serde_json::from_value(value)?;
        Ok(SwapInstructionsResponse::from_internal(raw, internal))
    }
}
