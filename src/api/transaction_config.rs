use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use solana_account_decoder::UiAccount;
use solana_sdk::pubkey::Pubkey;

use crate::api::serde_helpers::option_field_as_string;

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum ComputeUnitPriceMicroLamports {
    MicroLamports(u64),
    #[serde(deserialize_with = "auto")]
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
pub enum PrioritizationFeeLamports {
    AutoMultiplier(u32),
    JitoTipLamports(u64),
    #[serde(rename_all = "camelCase")]
    PriorityLevelWithMaxLamports {
        priority_level: PriorityLevel,
        max_lamports: u64,
        #[serde(default)]
        global: bool,
    },
    #[default]
    #[serde(untagged, deserialize_with = "auto")]
    Auto,
    #[serde(untagged)]
    Lamports(u64),
    #[serde(untagged, deserialize_with = "disabled")]
    Disabled,
}

impl Serialize for PrioritizationFeeLamports {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
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

fn auto<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: Deserializer<'de>,
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
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    enum Helper {
        #[serde(rename = "disabled")]
        Variant,
    }

    Helper::deserialize(deserializer)?;
    Ok(())
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DynamicSlippageSettings {
    pub min_bps: Option<u16>,
    pub max_bps: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct TransactionConfig {
    /// Wrap and unwrap SOL. Will be ignored if `destination_token_account` is set because the `destination_token_account` may belong to a different user that we have no authority to close.
    pub wrap_and_unwrap_sol: bool,
    /// Allow optimized WSOL token account by using transfer, assign with seed, allocate with seed then initialize account 3 instead of the expensive associated token account process.
    pub allow_optimized_wrapped_sol_token_account: bool,
    /// Fee token account for the output token, derived using the Jupiter referral seeds.
    #[serde(with = "option_field_as_string")]
    pub fee_account: Option<Pubkey>,
    /// Public key of the token account that will be used to receive the token out of the swap. If not provided, the user's ATA will be used.
    #[serde(with = "option_field_as_string")]
    pub destination_token_account: Option<Pubkey>,
    /// Add a readonly, non signer tracking account that isn't used by Jupiter.
    #[serde(with = "option_field_as_string")]
    pub tracking_account: Option<Pubkey>,
    /// Compute unit price to prioritize the transaction, the additional fee will be compute unit consumed * computeUnitPriceMicroLamports.
    pub compute_unit_price_micro_lamports: Option<ComputeUnitPriceMicroLamports>,
    /// Prioritization fee lamports paid for the transaction in addition to the signatures fee.
    /// Mutually exclusive with `compute_unit_price_micro_lamports`.
    pub prioritization_fee_lamports: Option<PrioritizationFeeLamports>,
    /// When enabled, it will do a swap simulation to get the compute unit used and set it in ComputeBudget's compute unit limit.
    pub dynamic_compute_unit_limit: bool,
    /// Request a legacy transaction rather than the default versioned transaction.
    pub as_legacy_transaction: bool,
    /// Enables the usage of shared program accounts. That means no intermediate token accounts or open orders accounts need to be created.
    pub use_shared_accounts: Option<bool>,
    /// Useful when the instruction before the swap has a transfer that increases the input token amount.
    pub use_token_ledger: bool,
    /// Skip RPC calls and assume the user account do not exist.
    pub skip_user_accounts_rpc_calls: bool,
    /// Providing keyed ui accounts allow loading AMMs that are not in the market cache.
    pub keyed_ui_accounts: Option<Vec<KeyedUiAccount>>,
    /// The program authority ID.
    pub program_authority_id: Option<u8>,
    /// Dynamic slippage configuration.
    pub dynamic_slippage: Option<DynamicSlippageSettings>,
    /// Slots to expiry of the blockhash.
    pub blockhash_slots_to_expiry: Option<u8>,
    /// Requests a correct last valid block height.
    pub correct_last_valid_block_height: bool,
}

impl Default for TransactionConfig {
    fn default() -> Self {
        Self {
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

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct KeyedUiAccount {
    pub pubkey: String,
    #[serde(flatten)]
    pub ui_account: UiAccount,
    /// Additional data an Amm requires, Amm dependent and decoded in the Amm implementation.
    pub params: Option<Value>,
}
