use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::anyhow;
use bs58;
use reqwest::Client;
use serde_json::json;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    hash::Hash,
    instruction::Instruction,
    message::{AddressLookupTableAccount, VersionedMessage, v0::Message as V0Message},
    pubkey::Pubkey,
    transaction::VersionedTransaction,
};
use tracing::{debug, info, warn};

use solana_address_lookup_table_interface::state::AddressLookupTable;

use crate::api::SwapInstructionsResponse;
use crate::config::{BlindConfig, InstructionConfig, LanderSettings, SpamConfig};

use super::engine::StrategyIdentity;
use super::error::{StrategyError, StrategyResult};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum StrategyMode {
    Spam,
    Blind,
}

#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    pub memo: Option<String>,
    pub enable_simulation: bool,
    pub lander_settings: LanderSettings,
    pub spam_config: SpamConfig,
    pub blind_config: BlindConfig,
}

#[derive(Debug, Clone)]
pub struct BundleSubmission {
    pub lander: String,
    pub endpoint: String,
    pub bundle_id: Option<String>,
    pub slot: u64,
    pub blockhash: Hash,
}

pub struct ExecutionServices {
    rpc: Arc<RpcClient>,
    http: Client,
    config: ExecutionConfig,
}

impl ExecutionServices {
    pub fn new(
        rpc: Arc<RpcClient>,
        http: Client,
        instruction: &InstructionConfig,
        lander_settings: &LanderSettings,
        spam_config: &SpamConfig,
        blind_config: &BlindConfig,
    ) -> StrategyResult<Self> {
        Ok(Self {
            rpc,
            http,
            config: ExecutionConfig {
                memo: resolve_memo(instruction),
                enable_simulation: false,
                lander_settings: lander_settings.clone(),
                spam_config: spam_config.clone(),
                blind_config: blind_config.clone(),
            },
        })
    }

    pub fn with_simulation(mut self, enable: bool) -> Self {
        self.config.enable_simulation = enable;
        self
    }

    pub fn memo(&self) -> Option<&str> {
        self.config.memo.as_deref()
    }

    pub fn allowed_landers(&self, mode: StrategyMode) -> Vec<String> {
        let candidates = match mode {
            StrategyMode::Spam => &self.config.spam_config.enable_landers,
            StrategyMode::Blind => &self.config.blind_config.enable_landers,
        };
        if candidates.is_empty() {
            return vec!["rpc".to_string()];
        }
        candidates.clone()
    }

    pub fn compute_unit_price_override(&self, mode: StrategyMode) -> Option<u64> {
        match mode {
            StrategyMode::Spam => {
                let price = self.config.spam_config.compute_unit_price_micro_lamports;
                (price > 0).then_some(price)
            }
            StrategyMode::Blind => None,
        }
    }

    pub fn spam_max_retries(&self) -> u32 {
        self.config.spam_config.max_retries
    }

    pub(super) async fn prepare_transaction(
        &self,
        identity: &StrategyIdentity,
        instructions: &SwapInstructionsResponse,
        tip_lamports: u64,
        mode: StrategyMode,
    ) -> StrategyResult<PreparedTransaction> {
        let lookup_accounts = self
            .load_lookup_tables(&instructions.address_lookup_table_addresses)
            .await?;
        let blockhash = self
            .rpc
            .get_latest_blockhash()
            .await
            .map_err(StrategyError::Rpc)?;
        let slot = self.rpc.get_slot().await.map_err(StrategyError::Rpc)?;

        let mut ix = Vec::<Instruction>::new();
        ix.extend(instructions.compute_budget_instructions.clone());
        if let Some(token_ledger) = instructions.token_ledger_instruction.clone() {
            ix.push(token_ledger);
        }
        ix.extend(instructions.setup_instructions.clone());
        ix.push(instructions.swap_instruction.clone());
        ix.extend(instructions.other_instructions.clone());
        if let Some(cleanup) = instructions.cleanup_instruction.clone() {
            ix.push(cleanup);
        }

        if let Some(memo) = self.memo() {
            ix.push(build_memo_instruction(memo));
        }

        if tip_lamports > 0 {
            debug!(
                target: "strategy::execution",
                mode = ?mode,
                tip_lamports,
                "tip instruction not yet configured, skipping"
            );
        }

        let message = compile_message(&identity.pubkey, &ix, &lookup_accounts, blockhash)?;
        let versioned = VersionedMessage::V0(message);
        let tx = VersionedTransaction::try_new(versioned, &[identity.signer.as_ref()])
            .map_err(|err| StrategyError::Transaction(anyhow!(err)))?;

        Ok(PreparedTransaction {
            transaction: tx,
            blockhash,
            slot,
        })
    }

    pub async fn submit(
        &self,
        prepared: PreparedTransaction,
        allowed_landers: &[String],
    ) -> StrategyResult<BundleSubmission> {
        let mut last_err = None;
        for lander in allowed_landers {
            match lander.as_str() {
                "jito" => match self.send_jito(&prepared).await {
                    Ok(resp) => return Ok(resp),
                    Err(err) => {
                        warn!(target: "strategy::bundle", lander, error = %err, "jito submission failed");
                        last_err = Some(err);
                    }
                },
                "rpc" | "staked" => match self.send_via_rpc(&prepared, lander).await {
                    Ok(resp) => return Ok(resp),
                    Err(err) => {
                        warn!(
                            target: "strategy::bundle",
                            lander,
                            error = %err,
                            "rpc submission failed"
                        );
                        last_err = Some(err);
                    }
                },
                other => {
                    debug!(
                        target: "strategy::bundle",
                        lander = other,
                        "no sender implementation, skipping"
                    );
                }
            }
        }
        Err(last_err.unwrap_or_else(|| {
            StrategyError::Bundle("no available lander endpoints provided".into())
        }))
    }

    async fn load_lookup_tables(
        &self,
        addresses: &[Pubkey],
    ) -> StrategyResult<Vec<AddressLookupTableAccount>> {
        let mut tables = Vec::new();
        for address in addresses {
            match self.rpc.get_account(address).await {
                Ok(account) => match AddressLookupTable::deserialize(&account.data) {
                    Ok(table) => {
                        tables.push(AddressLookupTableAccount {
                            key: *address,
                            addresses: table.addresses.into_owned(),
                        });
                    }
                    Err(err) => {
                        warn!(
                            target: "strategy::execution",
                            address = %address,
                            error = %err,
                            "failed to deserialize lookup table"
                        );
                    }
                },
                Err(err) => {
                    warn!(
                        target: "strategy::execution",
                        address = %address,
                        error = %err,
                        "failed to fetch lookup table account"
                    );
                }
            }
        }
        Ok(tables)
    }

    async fn send_jito(&self, prepared: &PreparedTransaction) -> StrategyResult<BundleSubmission> {
        let settings = match &self.config.lander_settings.jito {
            Some(value) if !value.endpoints.is_empty() => value,
            _ => {
                return Err(StrategyError::Bundle(
                    "jito endpoints not configured in lander.yaml".into(),
                ));
            }
        };

        let transaction_bytes =
            bincode::serde::encode_to_vec(&prepared.transaction, bincode::config::standard())
                .map_err(|err| StrategyError::Transaction(err.into()))?;
        let encoded = bs58::encode(transaction_bytes).into_string();
        let payload = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "sendBundle",
            "params": [[encoded]]
        });

        for endpoint in &settings.endpoints {
            if endpoint.trim().is_empty() {
                continue;
            }
            let response = self
                .http
                .post(endpoint)
                .timeout(Duration::from_secs(2))
                .json(&payload)
                .send()
                .await
                .map_err(StrategyError::Network)?;
            if !response.status().is_success() {
                warn!(
                    target: "strategy::bundle",
                    endpoint,
                    status = %response.status(),
                    "bundle submission status failure"
                );
                continue;
            }
            let value: serde_json::Value = response.json().await.map_err(StrategyError::Network)?;
            if let Some(error) = value.get("error") {
                warn!(
                    target: "strategy::bundle",
                    endpoint,
                    error = %error,
                    "bundle submission returned error"
                );
                continue;
            }
            let bundle_id = value
                .get("result")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            info!(
                target: "strategy::bundle",
                endpoint,
                bundle_id = bundle_id.as_deref().unwrap_or(""),
                slot = prepared.slot,
                "bundle submitted to jito"
            );
            return Ok(BundleSubmission {
                lander: "jito".to_string(),
                endpoint: endpoint.clone(),
                bundle_id,
                slot: prepared.slot,
                blockhash: prepared.blockhash,
            });
        }

        Err(StrategyError::Bundle(
            "all jito endpoints failed submission".into(),
        ))
    }

    async fn send_via_rpc(
        &self,
        prepared: &PreparedTransaction,
        lander_name: &str,
    ) -> StrategyResult<BundleSubmission> {
        let signature = self
            .rpc
            .send_transaction(&prepared.transaction)
            .await
            .map_err(StrategyError::Rpc)?;
        info!(
            target: "strategy::bundle",
            lander = lander_name,
            slot = prepared.slot,
            blockhash = %prepared.blockhash,
            signature = %signature,
            "transaction submitted via rpc"
        );
        Ok(BundleSubmission {
            lander: lander_name.to_string(),
            endpoint: self.rpc.url(),
            bundle_id: Some(signature.to_string()),
            slot: prepared.slot,
            blockhash: prepared.blockhash,
        })
    }
}

#[derive(Clone)]
pub struct PreparedTransaction {
    pub transaction: VersionedTransaction,
    pub blockhash: Hash,
    pub slot: u64,
}

fn compile_message(
    payer: &Pubkey,
    instructions: &[Instruction],
    tables: &[AddressLookupTableAccount],
    blockhash: Hash,
) -> StrategyResult<V0Message> {
    V0Message::try_compile(payer, instructions, tables, blockhash)
        .map_err(|err| StrategyError::Transaction(anyhow!(err)))
}

fn build_memo_instruction(text: &str) -> Instruction {
    let program_id = Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")
        .unwrap_or_else(|_| Pubkey::new_unique());
    Instruction {
        program_id,
        accounts: vec![],
        data: text.as_bytes().to_vec(),
    }
}

fn resolve_memo(instruction: &InstructionConfig) -> Option<String> {
    let memo = instruction.memo.trim();
    if memo.is_empty() {
        None
    } else {
        Some(memo.to_string())
    }
}
