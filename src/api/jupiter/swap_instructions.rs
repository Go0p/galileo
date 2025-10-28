use super::quote::QuoteResponsePayload;
use super::serde_helpers::{field_as_string, option_field_as_string};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_account_decoder::UiAccount;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::AddressLookupTableAccount,
    pubkey::Pubkey,
};

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwapInstructionsRequest {
    #[serde(rename = "quoteResponse")]
    pub quote_response: QuoteResponsePayload,
    #[serde(with = "field_as_string")]
    pub user_public_key: Pubkey,
    pub wrap_and_unwrap_sol: bool,
    pub allow_optimized_wrapped_sol_token_account: bool,
    #[serde(with = "option_field_as_string")]
    pub fee_account: Option<Pubkey>,
    #[serde(with = "option_field_as_string")]
    pub destination_token_account: Option<Pubkey>,
    #[serde(with = "option_field_as_string")]
    pub tracking_account: Option<Pubkey>,
    pub compute_unit_price_micro_lamports: Option<ComputeUnitPriceMicroLamports>,
    pub prioritization_fee_lamports: Option<PrioritizationFeeLamports>,
    pub dynamic_compute_unit_limit: bool,
    pub as_legacy_transaction: bool,
    pub use_shared_accounts: Option<bool>,
    pub use_token_ledger: bool,
    pub skip_user_accounts_rpc_calls: bool,
    pub keyed_ui_accounts: Option<Vec<KeyedUiAccount>>,
    pub program_authority_id: Option<u8>,
    pub dynamic_slippage: Option<DynamicSlippageSettings>,
    pub blockhash_slots_to_expiry: Option<u8>,
    pub correct_last_valid_block_height: bool,
}

impl SwapInstructionsRequest {
    pub fn from_payload(quote_response: QuoteResponsePayload, user_public_key: Pubkey) -> Self {
        Self {
            quote_response,
            user_public_key,
            wrap_and_unwrap_sol: true,
            allow_optimized_wrapped_sol_token_account: false,
            fee_account: None,
            destination_token_account: None,
            tracking_account: None,
            compute_unit_price_micro_lamports: None,
            prioritization_fee_lamports: None,
            dynamic_compute_unit_limit: false,
            as_legacy_transaction: false,
            use_shared_accounts: None,
            use_token_ledger: false,
            skip_user_accounts_rpc_calls: false,
            keyed_ui_accounts: None,
            program_authority_id: None,
            dynamic_slippage: None,
            blockhash_slots_to_expiry: None,
            correct_last_valid_block_height: false,
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
    pub resolved_lookup_tables: Vec<AddressLookupTableAccount>,
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
            resolved_lookup_tables: Vec::new(),
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
/// 计算单价设置：
/// - `MicroLamports`：直接指定微 lamports；
/// - `Auto`：使用 Jupiter 侧的自动估算（JSON 字段写 `"auto"`）。
pub enum ComputeUnitPriceMicroLamports {
    /// 固定的 compute unit price（单位：微 lamports）。
    MicroLamports(u64),
    #[serde(deserialize_with = "auto")]
    /// 让 Jupiter 自动推算 compute unit price。
    Auto,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum PriorityLevel {
    Medium,
    High,
    VeryHigh,
}

#[derive(Deserialize, Debug, PartialEq, Copy, Clone, Default)]
#[serde(rename_all = "camelCase")]
/// 优先费/Tip 配置入口，对应多种 JSON 结构：
/// - `AutoMultiplier`：`{"autoMultiplier": <u32>}`；
/// - `JitoTipLamports`：`{"jitoTipLamports": <u64>}`；
/// - `PriorityLevelWithMaxLamports`：`{"priorityLevelWithMaxLamports": {...}}`；
/// - `Auto`：`"auto"`；
/// - `Lamports`：直接填整数 lamports；
/// - `Disabled`：`"disabled"`（仅 `/swap` 可用）。
pub enum PrioritizationFeeLamports {
    /// 在 Jupiter 估算基础上乘以倍率。
    AutoMultiplier(u32),
    /// 固定 Jito tip（由用户钱包支付）。
    JitoTipLamports(u64),
    #[serde(rename_all = "camelCase")]
    PriorityLevelWithMaxLamports {
        /// 预设优先级（medium/high/veryHigh）。
        priority_level: PriorityLevel,
        /// 估算优先费的最大上限。
        max_lamports: u64,
        #[serde(default)]
        /// 是否按全局费市估算；默认为局部（相关写入账户）。
        global: bool,
    },
    #[default]
    #[serde(untagged, deserialize_with = "auto")]
    /// 让 Jupiter 自动决定优先费。
    Auto,
    #[serde(untagged)]
    /// 直接填入固定的额外 lamports。
    Lamports(u64),
    #[serde(untagged, deserialize_with = "disabled")]
    /// 显式关闭优先费（仅在 `/swap` 请求可用）。
    Disabled,
}

impl Serialize for PrioritizationFeeLamports {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct AutoMultiplier {
            auto_multiplier: u32,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct PriorityLevelWrapper<'a> {
            priority_level_with_max_lamports: PriorityLevelWithMaxLamports<'a>,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct PriorityLevelWithMaxLamports<'a> {
            priority_level: &'a PriorityLevel,
            max_lamports: &'a u64,
            global: &'a bool,
        }

        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct JitoTipLamports {
            jito_tip_lamports: u64,
        }

        match self {
            Self::AutoMultiplier(auto_multiplier) => AutoMultiplier {
                auto_multiplier: *auto_multiplier,
            }
            .serialize(serializer),
            Self::JitoTipLamports(lamports) => JitoTipLamports {
                jito_tip_lamports: *lamports,
            }
            .serialize(serializer),
            Self::Auto => serializer.serialize_str("auto"),
            Self::Lamports(lamports) => serializer.serialize_u64(*lamports),
            Self::Disabled => serializer.serialize_str("disabled"),
            Self::PriorityLevelWithMaxLamports {
                priority_level,
                max_lamports,
                global,
            } => PriorityLevelWrapper {
                priority_level_with_max_lamports: PriorityLevelWithMaxLamports {
                    priority_level,
                    max_lamports,
                    global,
                },
            }
            .serialize(serializer),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicSlippageSettings {
    pub min_bps: Option<u16>,
    pub max_bps: Option<u16>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct KeyedUiAccount {
    pub pubkey: String,
    #[serde(flatten)]
    pub ui_account: UiAccount,
    pub params: Option<Value>,
}

fn auto<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    enum Helper {
        #[serde(rename = "auto")]
        Variant,
    }

    Helper::deserialize(deserializer)?;
    Ok(())
}

fn disabled<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    enum Helper {
        #[serde(rename = "disabled")]
        Variant,
    }

    Helper::deserialize(deserializer)?;
    Ok(())
}
