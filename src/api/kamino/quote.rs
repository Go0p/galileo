use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;

use super::serde_helpers::{parse_base64, parse_lookup_table_accounts, parse_pubkey, parse_u64};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SwapType {
    ExactIn,
    ExactOut,
}

impl SwapType {
    pub fn as_str(self) -> &'static str {
        match self {
            SwapType::ExactIn => "exactIn",
            SwapType::ExactOut => "exactOut",
        }
    }
}

impl Default for SwapType {
    fn default() -> Self {
        Self::ExactIn
    }
}

#[derive(Debug, Clone)]
pub struct QuoteRequest {
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount: u64,
    pub swap_type: SwapType,
    pub max_slippage_bps: u16,
    pub executor: Option<String>,
    pub referrer_pda: Option<String>,
    pub include_setup_ixs: bool,
    pub wrap_and_unwrap_sol: bool,
    pub routes: Vec<String>,
}

impl QuoteRequest {
    pub fn new(token_in: Pubkey, token_out: Pubkey, amount: u64, max_slippage_bps: u16) -> Self {
        Self {
            token_in,
            token_out,
            amount,
            swap_type: SwapType::ExactIn,
            max_slippage_bps,
            executor: None,
            referrer_pda: None,
            include_setup_ixs: true,
            wrap_and_unwrap_sol: false,
            routes: Vec::new(),
        }
    }

    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::with_capacity(24);
        params.push(("tokenIn".to_string(), self.token_in.to_string()));
        params.push(("tokenOut".to_string(), self.token_out.to_string()));
        params.push(("amount".to_string(), self.amount.to_string()));
        params.push(("swapType".to_string(), self.swap_type.as_str().to_string()));
        params.push((
            "maxSlippageBps".to_string(),
            self.max_slippage_bps.to_string(),
        ));
        params.push((
            "includeSetupIxs".to_string(),
            self.include_setup_ixs.to_string(),
        ));
        params.push((
            "wrapAndUnwrapSol".to_string(),
            self.wrap_and_unwrap_sol.to_string(),
        ));
        if let Some(executor) = self.executor.as_ref() {
            if !executor.trim().is_empty() {
                params.push(("executor".to_string(), executor.trim().to_string()));
            }
        }
        if let Some(referrer) = self.referrer_pda.as_ref() {
            if !referrer.trim().is_empty() {
                params.push(("referrerPda".to_string(), referrer.trim().to_string()));
            }
        }
        if self.routes.is_empty() {
            for route in DEFAULT_ROUTER_TYPES.iter().copied() {
                params.push(("routerTypes[]".to_string(), route.to_string()));
            }
        } else {
            for route in &self.routes {
                let trimmed = route.trim();
                if trimmed.is_empty() {
                    continue;
                }
                params.push(("routerTypes[]".to_string(), trimmed.to_string()));
            }
        }

        params.extend_from_slice(&[
            (
                "includeLimoLogs".to_string(),
                DEFAULT_INCLUDE_LIMO_LOGS.to_string(),
            ),
            ("includeRfq".to_string(), DEFAULT_INCLUDE_RFQ.to_string()),
            ("timeoutMs".to_string(), DEFAULT_TIMEOUT_MS.to_string()),
            (
                "atLeastOneNoMoreThanTimeoutMS".to_string(),
                DEFAULT_AT_LEAST_ONE_TIMEOUT_MS.to_string(),
            ),
            (
                "withSimulation".to_string(),
                DEFAULT_WITH_SIMULATION.to_string(),
            ),
            (
                "filterFailedSimulations".to_string(),
                DEFAULT_FILTER_FAILED_SIMULATIONS.to_string(),
            ),
            (
                "requestPriceImpact".to_string(),
                DEFAULT_REQUEST_PRICE_IMPACT.to_string(),
            ),
            (
                "perMinimumQuoteLifetimeSeconds".to_string(),
                DEFAULT_MIN_QUOTE_LIFETIME_SECONDS.to_string(),
            ),
            (
                "assertSwapBalances".to_string(),
                DEFAULT_ASSERT_SWAP_BALANCES.to_string(),
            ),
            (
                "simulateWithMockInputAmount".to_string(),
                DEFAULT_SIMULATE_WITH_MOCK_INPUT_AMOUNT.to_string(),
            ),
        ]);

        params
    }
}

const DEFAULT_ROUTER_TYPES: &[&str] = &[
    "dflow",
    "jupiter",
    "jupiterSelfHosted",
    "jupiterEuropa",
    "per",
    "fluxbeam",
    "raydium",
    "hashflow",
    "okx",
    "clover",
    "zeroEx",
    "spur",
    "lifi",
];
const DEFAULT_INCLUDE_LIMO_LOGS: bool = false;
const DEFAULT_INCLUDE_RFQ: bool = true;
const DEFAULT_TIMEOUT_MS: u64 = 2000;
const DEFAULT_AT_LEAST_ONE_TIMEOUT_MS: u64 = 2000;
const DEFAULT_WITH_SIMULATION: bool = false;
const DEFAULT_FILTER_FAILED_SIMULATIONS: bool = false;
const DEFAULT_REQUEST_PRICE_IMPACT: bool = false;
const DEFAULT_MIN_QUOTE_LIFETIME_SECONDS: u64 = 3;
const DEFAULT_ASSERT_SWAP_BALANCES: bool = false;
const DEFAULT_SIMULATE_WITH_MOCK_INPUT_AMOUNT: bool = false;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponsePayload {
    #[serde(default)]
    pub data: Vec<Route>,
}

#[derive(Clone, Debug)]
pub struct QuoteResponse {
    payload: QuoteResponsePayload,
}

impl QuoteResponse {
    pub fn try_from_value(value: Value) -> Result<Self, serde_json::Error> {
        let payload: QuoteResponsePayload = serde_json::from_value(value)?;
        Ok(Self { payload })
    }

    pub fn strip_lookup_addresses(&mut self) {
        for route in &mut self.payload.data {
            route.strip_lookup_addresses();
        }
    }

    pub fn routes(&self) -> &[Route] {
        &self.payload.data
    }

    pub fn best_route(&self) -> Option<&Route> {
        self.payload.data.first()
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    pub router_type: String,
    #[serde(default)]
    pub response_time_get_quote_ms: u64,
    #[serde(default)]
    pub price_impact_bps: Option<i64>,
    #[serde(default)]
    pub guaranteed_price_impact_bps: Option<i64>,
    #[serde(default)]
    pub price_impact_amount: Option<String>,
    #[serde(default)]
    pub guaranteed_price_impact_amount: Option<String>,
    #[serde(default, deserialize_with = "parse_lookup_table_accounts")]
    pub lookup_table_accounts_bs58: Vec<LookupTableEntry>,
    pub amounts_exact_in: AmountsExactIn,
    pub amounts_exact_out: AmountsExactOut,
    pub instructions: RouteInstructions,
}

impl Route {
    pub fn amount_in(&self) -> u64 {
        self.amounts_exact_in.amount_in
    }

    pub fn amount_out(&self) -> u64 {
        self.amounts_exact_in.amount_out
    }

    pub fn strip_lookup_addresses(&mut self) {
        for entry in &mut self.lookup_table_accounts_bs58 {
            entry.addresses.clear();
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmountsExactIn {
    #[serde(deserialize_with = "parse_u64")]
    pub amount_in: u64,
    #[serde(deserialize_with = "parse_u64")]
    pub amount_out_guaranteed: u64,
    #[serde(deserialize_with = "parse_u64")]
    pub amount_out: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmountsExactOut {
    #[serde(deserialize_with = "parse_u64")]
    pub amount_out: u64,
    #[serde(deserialize_with = "parse_u64")]
    pub amount_in_guaranteed: u64,
    #[serde(deserialize_with = "parse_u64")]
    pub amount_in: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct RouteInstructions {
    #[serde(default)]
    pub create_in_ata_ixs: Vec<RawInstruction>,
    #[serde(default)]
    pub create_out_ata_ixs: Vec<RawInstruction>,
    #[serde(default)]
    pub wrap_sol_ixs: Vec<RawInstruction>,
    #[serde(default)]
    pub limo_logs_start_ixs: Vec<RawInstruction>,
    #[serde(default)]
    pub limo_ledger_start_ixs: Vec<RawInstruction>,
    #[serde(default)]
    pub swap_ixs: Vec<RawInstruction>,
    #[serde(default)]
    pub limo_ledger_end_ixs: Vec<RawInstruction>,
    #[serde(default)]
    pub limo_logs_end_ixs: Vec<RawInstruction>,
    #[serde(default)]
    pub unwrap_sol_ixs: Vec<RawInstruction>,
}

impl RouteInstructions {
    pub fn flatten(&self) -> Vec<Instruction> {
        let mut result = Vec::new();
        self.extend_into(&mut result);
        result
    }

    pub fn extend_into(&self, target: &mut Vec<Instruction>) {
        append(target, &self.create_in_ata_ixs);
        append(target, &self.swap_ixs);
    }

    pub fn append_from(&mut self, other: &RouteInstructions) {
        self.wrap_sol_ixs.extend(other.wrap_sol_ixs.iter().cloned());
        self.swap_ixs.extend(other.swap_ixs.iter().cloned());
        self.unwrap_sol_ixs
            .extend(other.unwrap_sol_ixs.iter().cloned());
    }

    #[allow(dead_code)]
    pub fn create_out_ata_ixs(&self) -> &[RawInstruction] {
        &self.create_out_ata_ixs
    }
}

fn append(target: &mut Vec<Instruction>, source: &[RawInstruction]) {
    for ix in source {
        target.push(ix.to_instruction());
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LookupTableEntry {
    pub key: String,
    #[serde(default)]
    pub addresses: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawInstruction {
    #[serde(deserialize_with = "parse_pubkey")]
    pub program_id: Pubkey,
    #[serde(deserialize_with = "parse_base64")]
    pub data: Vec<u8>,
    pub keys: Vec<RawAccountMeta>,
}

impl RawInstruction {
    pub fn to_instruction(&self) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts: self
                .keys
                .iter()
                .map(|meta| AccountMeta {
                    pubkey: meta.pubkey,
                    is_signer: meta.is_signer,
                    is_writable: meta.is_writable,
                })
                .collect(),
            data: self.data.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawAccountMeta {
    #[serde(deserialize_with = "parse_pubkey")]
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}
