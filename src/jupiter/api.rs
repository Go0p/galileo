use std::collections::BTreeMap;
use std::str::FromStr;
use std::time::{Duration, Instant};

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use serde_json::Value;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use tracing::{debug, info};

use crate::config::BotConfig;
use crate::jupiter::error::JupiterError;
use crate::metrics::{LatencyMetadata, guard_with_metadata};

#[derive(Clone, Debug)]
pub struct JupiterApiClient {
    base_url: String,
    client: reqwest::Client,
    request_timeout: Duration,
}

impl JupiterApiClient {
    pub fn new(client: reqwest::Client, config: &BotConfig) -> Self {
        Self {
            base_url: config.jupiter_api_url.clone(),
            client,
            request_timeout: Duration::from_millis(config.request_timeout_ms),
        }
    }

    pub async fn quote(&self, request: &QuoteRequest) -> Result<QuoteResponse, JupiterError> {
        let url = self.endpoint("/swap/v1/quote");
        let metadata = LatencyMetadata::new(
            [
                ("stage".to_string(), "quote".to_string()),
                ("url".to_string(), url.clone()),
            ]
            .into_iter()
            .collect(),
        );
        let guard = guard_with_metadata("jupiter.quote", metadata);
        let start = Instant::now();

        let response = self
            .client
            .get(&url)
            .timeout(self.request_timeout)
            .query(&request.to_query_params())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(JupiterError::ApiStatus {
                endpoint: url,
                status: response.status(),
            });
        }

        let value: Value = response.json().await?;
        let quote = QuoteResponse::try_from_value(value)
            .map_err(|err| JupiterError::Schema(format!("invalid quote response: {err}")))?;

        guard.finish();

        let elapsed_ms = start.elapsed().as_micros() as f64 / 1_000.0;
        if let Some(api_time) = quote.time_taken {
            debug!(
                target: "latency",
                elapsed_ms,
                api_time = api_time * 1_000.0,
                "quote latency comparison"
            );
        } else {
            info!(target: "latency", elapsed_ms, "quote latency recorded");
        }
        info!(
            target: "jupiter::quote",
            input_mint = %quote.input_mint,
            output_mint = %quote.output_mint,
            in_amount = %quote.in_amount,
            out_amount = %quote.out_amount,
            other_amount_threshold = ?quote.other_amount_threshold,
            elapsed_ms,
            "quote completed"
        );

        Ok(quote)
    }

    #[allow(dead_code)]
    pub async fn swap_instructions(
        &self,
        request: &SwapRequest,
    ) -> Result<SwapInstructionsResponse, JupiterError> {
        let url = self.endpoint("/swap/v1/swap-instructions");
        let metadata = LatencyMetadata::new(
            [
                ("stage".to_string(), "swap_instructions".to_string()),
                ("url".to_string(), url.clone()),
            ]
            .into_iter()
            .collect(),
        );
        let guard = guard_with_metadata("jupiter.swap_instructions", metadata);
        let start = Instant::now();

        let response = self
            .client
            .post(&url)
            .timeout(self.request_timeout)
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(JupiterError::ApiStatus {
                endpoint: url,
                status: response.status(),
            });
        }

        let value: Value = response.json().await?;
        let instructions = SwapInstructionsResponse::try_from_value(value)
            .map_err(|err| JupiterError::Schema(format!("invalid swap-instructions response: {err}")))?;

        guard.finish();
        let elapsed_ms = start.elapsed().as_micros() as f64 / 1_000.0;

        info!(
            target: "jupiter::swap_instructions",
            elapsed_ms,
            compute_unit_limit = ?instructions.compute_unit_limit,
            prioritization_fee_lamports = ?instructions.prioritization_fee_lamports,
            setup_ix = instructions.setup_instructions.len(),
            other_ix = instructions.other_instructions.len(),
            "swap-instructions response received"
        );

        Ok(instructions)
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }
}

#[derive(Debug, Clone)]
pub struct QuoteRequest {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: u64,
    pub slippage_bps: u16,
    pub only_direct_routes: bool,
    pub restrict_intermediate_tokens: bool,
    pub extra: BTreeMap<String, String>,
}

impl QuoteRequest {
    pub fn new(
        input_mint: impl Into<String>,
        output_mint: impl Into<String>,
        amount: u64,
        slippage_bps: u16,
    ) -> Self {
        Self {
            input_mint: input_mint.into(),
            output_mint: output_mint.into(),
            amount,
            slippage_bps,
            only_direct_routes: false,
            restrict_intermediate_tokens: true,
            extra: BTreeMap::new(),
        }
    }

    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("inputMint".to_string(), self.input_mint.clone()),
            ("outputMint".to_string(), self.output_mint.clone()),
            ("amount".to_string(), self.amount.to_string()),
            ("slippageBps".to_string(), self.slippage_bps.to_string()),
            (
                "onlyDirectRoutes".to_string(),
                self.only_direct_routes.to_string(),
            ),
            (
                "restrictIntermediateTokens".to_string(),
                self.restrict_intermediate_tokens.to_string(),
            ),
        ];
        for (key, value) in &self.extra {
            params.push((key.clone(), value.clone()));
        }
        params
    }
}

#[derive(Debug, Clone)]
pub struct QuoteResponse {
    pub raw: Value,
    pub input_mint: String,
    pub output_mint: String,
    pub in_amount: String,
    pub out_amount: String,
    pub other_amount_threshold: Option<String>,
    pub time_taken: Option<f64>,
}

impl QuoteResponse {
    pub fn try_from_value(value: Value) -> Result<Self, String> {
        let raw = value;
        let input_mint = raw
            .get("inputMint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "inputMint missing".to_string())?
            .to_string();
        let output_mint = raw
            .get("outputMint")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "outputMint missing".to_string())?
            .to_string();
        let in_amount = raw
            .get("inAmount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "inAmount missing".to_string())?
            .to_string();
        let out_amount = raw
            .get("outAmount")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "outAmount missing".to_string())?
            .to_string();

        let other_amount_threshold = raw
            .get("otherAmountThreshold")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let time_taken = raw.get("timeTaken").and_then(|v| v.as_f64());

        Ok(Self {
            raw,
            input_mint,
            output_mint,
            in_amount,
            out_amount,
            other_amount_threshold,
            time_taken,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SwapRequest {
    #[serde(rename = "quoteResponse")]
    pub quote_response: Value,
    #[serde(rename = "userPublicKey")]
    pub user_public_key: String,
    #[serde(rename = "wrapAndUnwrapSol", skip_serializing_if = "Option::is_none")]
    pub wrap_and_unwrap_sol: Option<bool>,
    #[serde(rename = "useSharedAccounts", skip_serializing_if = "Option::is_none")]
    pub use_shared_accounts: Option<bool>,
    #[serde(rename = "feeAccount", skip_serializing_if = "Option::is_none")]
    pub fee_account: Option<String>,
    #[serde(
        rename = "computeUnitPriceMicroLamports",
        skip_serializing_if = "Option::is_none"
    )]
    pub compute_unit_price_micro_lamports: Option<u64>,
    #[serde(
        rename = "skipUserAccountsRpcCalls",
        skip_serializing_if = "Option::is_none"
    )]
    pub skip_user_accounts_rpc_calls: Option<bool>,
}

impl SwapRequest {
    pub fn new(quote_response: Value, user_public_key: impl Into<String>) -> Self {
        Self {
            quote_response,
            user_public_key: user_public_key.into(),
            wrap_and_unwrap_sol: None,
            use_shared_accounts: None,
            fee_account: None,
            compute_unit_price_micro_lamports: None,
            skip_user_accounts_rpc_calls: None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SwapInstructionsResponse {
    pub raw: Value,
    pub token_ledger_instruction: Option<Instruction>,
    pub compute_budget_instructions: Vec<Instruction>,
    pub setup_instructions: Vec<Instruction>,
    pub swap_instruction: Instruction,
    pub cleanup_instruction: Option<Instruction>,
    pub other_instructions: Vec<Instruction>,
    pub address_lookup_table_addresses: Vec<Pubkey>,
    pub prioritization_fee_lamports: Option<u64>,
    pub compute_unit_limit: Option<u32>,
    pub prioritization_type: Option<Value>,
    pub dynamic_slippage_report: Option<Value>,
    pub simulation_error: Option<Value>,
}

#[allow(dead_code)]
impl SwapInstructionsResponse {
    pub fn try_from_value(value: Value) -> Result<Self, String> {
        let raw = value;
        let token_ledger_instruction =
            parse_instruction_opt(raw.get("tokenLedgerInstruction")).map_err(|err| err.to_string())?;
        let compute_budget_instructions =
            parse_instruction_array(raw.get("computeBudgetInstructions")).map_err(|err| err.to_string())?;
        let setup_instructions =
            parse_instruction_array(raw.get("setupInstructions")).map_err(|err| err.to_string())?;
        let swap_instruction = parse_instruction(
            raw.get("swapInstruction")
                .ok_or_else(|| "swapInstruction missing".to_string())?,
        )
        .map_err(|err| err.to_string())?;
        let cleanup_instruction =
            parse_instruction_opt(raw.get("cleanupInstruction")).map_err(|err| err.to_string())?;
        let other_instructions =
            parse_instruction_array(raw.get("otherInstructions")).map_err(|err| err.to_string())?;

        let address_lookup_table_addresses = parse_pubkey_array(
            raw.get("addressLookupTableAddresses")
                .ok_or_else(|| "addressLookupTableAddresses missing".to_string())?,
        )
        .map_err(|err| err.to_string())?;

        let prioritization_fee_lamports = raw.get("prioritizationFeeLamports").and_then(|v| v.as_u64());
        let compute_unit_limit = raw
            .get("computeUnitLimit")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        let prioritization_type = raw.get("prioritizationType").cloned();
        let dynamic_slippage_report = raw.get("dynamicSlippageReport").cloned();
        let simulation_error = raw.get("simulationError").cloned();

        Ok(Self {
            raw,
            token_ledger_instruction,
            compute_budget_instructions,
            setup_instructions,
            swap_instruction,
            cleanup_instruction,
            other_instructions,
            address_lookup_table_addresses,
            prioritization_fee_lamports,
            compute_unit_limit,
            prioritization_type,
            dynamic_slippage_report,
            simulation_error,
        })
    }
}

#[allow(dead_code)]
fn parse_instruction_opt(node: Option<&Value>) -> Result<Option<Instruction>, JupiterError> {
    match node {
        None | Some(Value::Null) => Ok(None),
        Some(value) => parse_instruction(value).map(Some),
    }
}

#[allow(dead_code)]
fn parse_instruction_array(node: Option<&Value>) -> Result<Vec<Instruction>, JupiterError> {
    let Some(array) = node else {
        return Ok(Vec::new());
    };
    let Some(items) = array.as_array() else {
        return Err(JupiterError::Schema("expected instruction array".into()));
    };
    items.iter().map(parse_instruction).collect()
}

#[allow(dead_code)]
fn parse_instruction(value: &Value) -> Result<Instruction, JupiterError> {
    let program_id = parse_pubkey(
        value
            .get("programId")
            .ok_or_else(|| JupiterError::Schema("instruction.programId missing".into()))?,
    )?;
    let accounts_value = value
        .get("accounts")
        .ok_or_else(|| JupiterError::Schema("instruction.accounts missing".into()))?;
    let accounts_array = accounts_value
        .as_array()
        .ok_or_else(|| JupiterError::Schema("instruction.accounts should be array".into()))?;
    let mut accounts = Vec::with_capacity(accounts_array.len());
    for account in accounts_array {
        accounts.push(parse_account_meta(account)?);
    }
    let data_str = value
        .get("data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| JupiterError::Schema("instruction.data missing".into()))?;
    let data = STANDARD
        .decode(data_str)
        .map_err(|err| JupiterError::Schema(format!("failed to decode instruction data: {err}")))?;
    Ok(Instruction {
        program_id,
        accounts,
        data,
    })
}

#[allow(dead_code)]
fn parse_account_meta(value: &Value) -> Result<AccountMeta, JupiterError> {
    let pubkey = parse_pubkey(
        value
            .get("pubkey")
            .ok_or_else(|| JupiterError::Schema("account.pubkey missing".into()))?,
    )?;
    let is_signer = value
        .get("isSigner")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| JupiterError::Schema("account.isSigner missing".into()))?;
    let is_writable = value
        .get("isWritable")
        .and_then(|v| v.as_bool())
        .ok_or_else(|| JupiterError::Schema("account.isWritable missing".into()))?;
    Ok(AccountMeta {
        pubkey,
        is_signer,
        is_writable,
    })
}

#[allow(dead_code)]
fn parse_pubkey_array(value: &Value) -> Result<Vec<Pubkey>, JupiterError> {
    let arr = value
        .as_array()
        .ok_or_else(|| JupiterError::Schema("expected array of pubkeys".into()))?;
    let mut result = Vec::with_capacity(arr.len());
    for item in arr {
        result.push(parse_pubkey(item)?);
    }
    Ok(result)
}

#[allow(dead_code)]
fn parse_pubkey(value: &Value) -> Result<Pubkey, JupiterError> {
    let s = value
        .as_str()
        .ok_or_else(|| JupiterError::Schema("expected string pubkey".into()))?;
    Pubkey::from_str(s).map_err(|err| JupiterError::Schema(format!("invalid pubkey {s}: {err}")))
}
