mod error;
mod marginfi;

use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{Duration, Instant};

use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::RpcError;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use tokio::sync::Mutex;

use crate::api::SwapInstructionsResponse;
use crate::config::FlashloanMarginfiConfig;
use crate::engine::{EngineError, EngineIdentity, SwapOpportunity};
pub use error::{FlashloanError, FlashloanResult};
pub use marginfi::{
    MarginfiAccountEnsure, MarginfiFlashloan, build_initialize_instruction,
    ensure_marginfi_account, find_marginfi_account_by_authority,
    marginfi_account_matches_authority,
};

const BALANCE_CACHE_TTL: Duration = Duration::from_millis(500);
const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const SPL_TOKEN_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub(super) fn compute_associated_token_address(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[owner.as_ref(), SPL_TOKEN_PROGRAM_ID.as_ref(), mint.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    )
    .0
}

pub struct FlashloanManager {
    rpc: Arc<RpcClient>,
    enabled: bool,
    prefer_wallet_balance: bool,
    configured_marginfi: Option<Pubkey>,
    marginfi: Option<MarginfiFlashloan>,
    balance_cache: Mutex<HashMap<Pubkey, BalanceCacheEntry>>,
}

#[derive(Debug)]
struct BalanceCacheEntry {
    amount: u64,
    fetched_at: Instant,
}

#[derive(Debug, Clone)]
pub struct FlashloanPreparation {
    pub account: Pubkey,
    pub created: bool,
}

impl FlashloanManager {
    pub fn new(
        cfg: &FlashloanMarginfiConfig,
        rpc: Arc<RpcClient>,
        marginfi_account: Option<Pubkey>,
    ) -> Self {
        Self {
            rpc,
            enabled: cfg.enable,
            prefer_wallet_balance: cfg.prefer_wallet_balance,
            configured_marginfi: marginfi_account,
            marginfi: marginfi_account.map(MarginfiFlashloan::new),
            balance_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn try_into_enabled(self) -> Option<Self> {
        if self.enabled { Some(self) } else { None }
    }

    pub fn adopt_preparation(&mut self, prep: FlashloanPreparation) {
        if self.enabled {
            self.marginfi = Some(MarginfiFlashloan::new(prep.account));
        }
    }

    pub async fn prepare(
        &mut self,
        identity: &EngineIdentity,
    ) -> FlashloanResult<Option<FlashloanPreparation>> {
        if !self.enabled {
            return Ok(None);
        }
        if let Some(existing) = &self.marginfi {
            return Ok(Some(FlashloanPreparation {
                account: existing.account(),
                created: false,
            }));
        }
        let MarginfiAccountEnsure { account, created } =
            ensure_marginfi_account(&self.rpc, identity, self.configured_marginfi).await?;
        self.marginfi = Some(MarginfiFlashloan::new(account));
        Ok(Some(FlashloanPreparation { account, created }))
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn assemble(
        &self,
        identity: &EngineIdentity,
        opportunity: &SwapOpportunity,
        response: &SwapInstructionsResponse,
    ) -> FlashloanResult<FlashloanOutcome> {
        let mut flattened = response.flatten_instructions();
        if flattened.is_empty() {
            return Ok(FlashloanOutcome {
                instructions: flattened,
                metadata: None,
            });
        }

        let prefix_len = response
            .compute_budget_instructions
            .len()
            .min(flattened.len());
        let body = flattened.split_off(prefix_len);
        let prefix = flattened;

        let Some(marginfi) = &self.marginfi else {
            return Ok(FlashloanOutcome {
                instructions: combine(prefix, body),
                metadata: None,
            });
        };

        let base_mint = Pubkey::from_str(&opportunity.pair.input_mint).map_err(|err| {
            FlashloanError::InvalidConfigDetail(format!(
                "解析 base mint 失败 {}: {err}",
                opportunity.pair.input_mint
            ))
        })?;

        if self.prefer_wallet_balance {
            let wallet_balance = self.wallet_balance(&identity.pubkey, &base_mint).await?;
            if opportunity.amount_in <= wallet_balance {
                return Ok(FlashloanOutcome {
                    instructions: combine(prefix, body),
                    metadata: None,
                });
            }
        }

        marginfi.wrap(identity, &base_mint, prefix, body, opportunity.amount_in)
    }

    async fn wallet_balance(&self, owner: &Pubkey, mint: &Pubkey) -> FlashloanResult<u64> {
        let ata = compute_associated_token_address(owner, mint);
        {
            let cache = self.balance_cache.lock().await;
            if let Some(entry) = cache.get(&ata) {
                if entry.fetched_at.elapsed() < BALANCE_CACHE_TTL {
                    return Ok(entry.amount);
                }
            }
        }

        let amount = match self.rpc.get_token_account_balance(&ata).await {
            Ok(balance) => balance.amount.parse::<u64>().unwrap_or(0),
            Err(err) => {
                if is_account_not_found(&err) {
                    0
                } else {
                    return Err(FlashloanError::Rpc(err));
                }
            }
        };

        let mut cache = self.balance_cache.lock().await;
        cache.insert(
            ata,
            BalanceCacheEntry {
                amount,
                fetched_at: Instant::now(),
            },
        );

        Ok(amount)
    }
}

#[derive(Debug, Clone)]
pub struct FlashloanOutcome {
    pub instructions: Vec<Instruction>,
    pub metadata: Option<FlashloanMetadata>,
}

#[derive(Debug, Clone)]
pub struct FlashloanMetadata {
    pub protocol: FlashloanProtocol,
    pub mint: Pubkey,
    pub borrow_amount: u64,
    pub inner_instruction_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashloanProtocol {
    Marginfi,
}

impl FlashloanProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlashloanProtocol::Marginfi => "marginfi",
        }
    }
}

impl From<FlashloanError> for EngineError {
    fn from(value: FlashloanError) -> Self {
        match value {
            FlashloanError::InvalidConfig(msg) => EngineError::InvalidConfig(msg.to_string()),
            FlashloanError::InvalidConfigDetail(msg) => EngineError::InvalidConfig(msg),
            FlashloanError::UnsupportedAsset(msg) => EngineError::InvalidConfig(msg),
            FlashloanError::Rpc(err) => EngineError::Rpc(err),
        }
    }
}

impl fmt::Debug for FlashloanManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FlashloanManager")
            .field("enabled", &self.enabled)
            .field("prefer_wallet_balance", &self.prefer_wallet_balance)
            .field("configured_marginfi", &self.configured_marginfi)
            .field("has_marginfi", &self.marginfi.is_some())
            .finish()
    }
}

fn combine(mut prefix: Vec<Instruction>, mut body: Vec<Instruction>) -> Vec<Instruction> {
    prefix.append(&mut body);
    prefix
}

fn is_account_not_found(err: &ClientError) -> bool {
    match err.kind() {
        ClientErrorKind::RpcError(RpcError::RpcResponseError { message, .. }) => {
            is_account_missing_message(message)
        }
        ClientErrorKind::RpcError(RpcError::ForUser(message)) => {
            is_account_missing_message(message)
        }
        _ => false,
    }
}

fn is_account_missing_message(message: &str) -> bool {
    let normalized = message.to_ascii_lowercase();
    normalized.contains("could not find account") || normalized.contains("account does not exist")
}

#[cfg(test)]
mod tests;
