use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

use solana_client::client_error::{ClientError, ClientErrorKind};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::RpcError;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use tokio::sync::Mutex;

use crate::config::FlashloanMarginfiConfig;
use crate::engine::{EngineIdentity, SwapInstructionsVariant, SwapOpportunity};
use crate::flashloan::{FlashloanError, FlashloanOutcome, FlashloanResult};

use super::account::{MarginfiAccountEnsure, ensure_marginfi_account};
use super::compute_associated_token_address;
use super::instructions::MarginfiFlashloan;

const BALANCE_STATIC_TTL: Duration = Duration::from_secs(60 * 60 * 24); // effectively static

#[derive(Debug, Clone, Default)]
pub struct MarginfiAccountRegistry {
    default: Option<Pubkey>,
}

impl MarginfiAccountRegistry {
    pub fn new(default: Option<Pubkey>) -> Self {
        Self { default }
    }

    pub fn default(&self) -> Option<Pubkey> {
        self.default
    }

    pub fn configured_default(&self) -> Option<Pubkey> {
        self.default
    }
}

#[derive(Debug)]
struct BalanceCacheEntry {
    amount: u64,
    fetched_at: Instant,
    ttl: Duration,
}

#[derive(Debug, Clone)]
pub struct MarginfiFlashloanPreparation {
    pub account: Pubkey,
    pub created: bool,
}

pub struct MarginfiFlashloanManager {
    rpc: Arc<RpcClient>,
    enabled: bool,
    prefer_wallet_balance: bool,
    configured_default: Option<Pubkey>,
    fallback_marginfi: Option<MarginfiFlashloan>,
    balance_cache: Mutex<HashMap<Pubkey, BalanceCacheEntry>>,
    compute_unit_overhead: u32,
}

impl MarginfiFlashloanManager {
    pub fn new(
        cfg: &FlashloanMarginfiConfig,
        rpc: Arc<RpcClient>,
        accounts: MarginfiAccountRegistry,
    ) -> Self {
        let configured_default = accounts.configured_default();
        let fallback_marginfi = configured_default.map(MarginfiFlashloan::new);
        Self {
            rpc,
            enabled: cfg.enable,
            prefer_wallet_balance: cfg.prefer_wallet_balance,
            configured_default,
            fallback_marginfi,
            balance_cache: Mutex::new(HashMap::new()),
            compute_unit_overhead: cfg.compute_unit_overhead,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn try_into_enabled(self) -> Option<Self> {
        if self.enabled { Some(self) } else { None }
    }

    pub fn compute_unit_overhead(&self) -> u32 {
        self.compute_unit_overhead
    }

    pub fn adopt_preparation(&mut self, prep: MarginfiFlashloanPreparation) {
        if self.enabled {
            self.fallback_marginfi = Some(MarginfiFlashloan::new(prep.account));
            self.configured_default = Some(prep.account);
        }
    }

    pub async fn prepare(
        &mut self,
        identity: &EngineIdentity,
    ) -> FlashloanResult<Option<MarginfiFlashloanPreparation>> {
        if !self.enabled {
            return Ok(None);
        }

        if let Some(existing) = &self.fallback_marginfi {
            return Ok(Some(MarginfiFlashloanPreparation {
                account: existing.account(),
                created: false,
            }));
        }
        let MarginfiAccountEnsure { account, created } =
            ensure_marginfi_account(&self.rpc, identity, self.configured_default).await?;
        self.fallback_marginfi = Some(MarginfiFlashloan::new(account));
        Ok(Some(MarginfiFlashloanPreparation { account, created }))
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn assemble(
        &self,
        identity: &EngineIdentity,
        opportunity: &SwapOpportunity,
        response: &SwapInstructionsVariant,
    ) -> FlashloanResult<FlashloanOutcome> {
        let mut flattened = response.flatten_instructions();
        if flattened.is_empty() {
            return Ok(FlashloanOutcome {
                instructions: flattened,
                metadata: None,
            });
        }

        let prefix_len = response
            .compute_budget_instructions()
            .len()
            .min(flattened.len());
        let body = flattened.split_off(prefix_len);
        let prefix = flattened;

        let base_mint = opportunity.pair.input_pubkey;

        let marginfi = if let Some(entry) = &self.fallback_marginfi {
            entry
        } else {
            return Ok(FlashloanOutcome {
                instructions: combine(prefix, body),
                metadata: None,
            });
        };

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
                if entry.fetched_at.elapsed() < entry.ttl {
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
                ttl: BALANCE_STATIC_TTL,
            },
        );

        Ok(amount)
    }
}

impl fmt::Debug for MarginfiFlashloanManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MarginfiFlashloanManager")
            .field("enabled", &self.enabled)
            .field("prefer_wallet_balance", &self.prefer_wallet_balance)
            .field("configured_default", &self.configured_default)
            .field("fallback_marginfi", &self.fallback_marginfi.is_some())
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
