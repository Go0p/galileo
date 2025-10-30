use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use dashmap::DashMap;
use parking_lot::Mutex;
use serde_json::json;
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcTokenAccountsFilter};
use solana_client::rpc_request::RpcRequest;
use solana_client::rpc_response::{Response as RpcResponse, RpcKeyedAccount};
use solana_sdk::pubkey::Pubkey;
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;
use tracing::{debug, warn};

use crate::monitoring::events;

#[derive(Clone, Debug)]
pub struct CachedTokenAccount {
    pub account: Pubkey,
    pub token_program: Pubkey,
    pub balance: Option<u64>,
}

pub struct WalletStateManager {
    owner: Pubkey,
    rpc_client: Arc<RpcClient>,
    accounts: DashMap<Pubkey, CachedTokenAccount>,
    refresh_handle: Mutex<Option<JoinHandle<()>>>,
}

impl WalletStateManager {
    pub async fn new(
        rpc_client: Arc<RpcClient>,
        owner: Pubkey,
        refresh_interval: Option<Duration>,
    ) -> Result<Arc<Self>> {
        let manager = Arc::new(Self {
            owner,
            rpc_client,
            accounts: DashMap::new(),
            refresh_handle: Mutex::new(None),
        });

        manager.refresh_once().await?;

        if let Some(interval) = refresh_interval {
            manager.start_refresh_loop(interval);
        }

        Ok(manager)
    }

    fn start_refresh_loop(self: &Arc<Self>, period: Duration) {
        let manager = Arc::clone(self);
        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(period);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);
            ticker.tick().await; // consume immediate tick
            loop {
                ticker.tick().await;
                if let Err(err) = manager.refresh_once().await {
                    events::copy_wallet_refresh_error(&manager.owner);
                    warn!(
                        target: "wallet::state",
                        wallet = %manager.owner,
                        error = %err,
                        "wallet refresh failed"
                    );
                }
            }
        });
        *self.refresh_handle.lock() = Some(handle);
    }

    pub fn get_account(&self, mint: &Pubkey) -> Option<CachedTokenAccount> {
        self.accounts.get(mint).map(|entry| entry.clone())
    }

    async fn refresh_once(&self) -> Result<usize> {
        let accounts = Self::fetch_accounts(&self.rpc_client, &self.owner).await?;
        self.accounts.clear();
        for (mint, account) in accounts {
            self.accounts.insert(mint, account);
        }
        let count = self.accounts.len();
        events::copy_wallet_refresh_success(&self.owner, count);
        Ok(count)
    }

    async fn fetch_accounts(
        rpc_client: &Arc<RpcClient>,
        owner: &Pubkey,
    ) -> Result<HashMap<Pubkey, CachedTokenAccount>> {
        let mut cached = HashMap::new();
        let program_ids = [
            spl_token::id().to_string(),
            spl_token_2022::id().to_string(),
        ];
        for program_id in program_ids {
            let filter = RpcTokenAccountsFilter::ProgramId(program_id);
            let config = RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::JsonParsed),
                commitment: Some(rpc_client.commitment()),
                data_slice: None,
                min_context_slot: None,
            };
            let params = json!([owner.to_string(), filter, config]);
            let response_accounts: RpcResponse<Vec<RpcKeyedAccount>> = rpc_client
                .send(RpcRequest::GetTokenAccountsByOwner, params)
                .await
                .map_err(|err| anyhow!("获取 Token Accounts 失败: {err}"))?;

            for keyed in response_accounts.value {
                let account_pubkey = Pubkey::from_str(&keyed.pubkey).map_err(|err| anyhow!(err))?;
                if let UiAccountData::Json(parsed) = &keyed.account.data {
                    let owner_str = parsed
                        .parsed
                        .get("info")
                        .and_then(|info| info.get("owner"))
                        .and_then(|owner| owner.as_str());
                    if owner_str.map(Pubkey::from_str).transpose()? != Some(*owner) {
                        continue;
                    }
                    if let Some(mint_str) = parsed
                        .parsed
                        .get("info")
                        .and_then(|info| info.get("mint"))
                        .and_then(|mint| mint.as_str())
                    {
                        let mint = Pubkey::from_str(mint_str).map_err(|err| anyhow!(err))?;
                        let token_program = Pubkey::from_str(keyed.account.owner.as_str())
                            .map_err(|err| anyhow!(err))?;
                        let balance = parsed
                            .parsed
                            .get("info")
                            .and_then(|info| info.get("tokenAmount"))
                            .and_then(|ta| ta.get("amount"))
                            .and_then(|amount| amount.as_str())
                            .and_then(|amount| amount.parse::<u64>().ok());

                        cached
                            .entry(mint)
                            .and_modify(|entry: &mut CachedTokenAccount| {
                                entry.account = account_pubkey;
                                entry.token_program = token_program;
                                if balance.is_some() {
                                    entry.balance = balance;
                                }
                            })
                            .or_insert(CachedTokenAccount {
                                account: account_pubkey,
                                token_program,
                                balance,
                            });
                    }
                } else {
                    debug!(
                        target: "wallet::state",
                        account = %account_pubkey,
                        "unexpected account data format, skipping"
                    );
                }
            }
        }

        Ok(cached)
    }
}

impl Drop for WalletStateManager {
    fn drop(&mut self) {
        if let Some(handle) = self.refresh_handle.lock().take() {
            handle.abort();
        }
    }
}
