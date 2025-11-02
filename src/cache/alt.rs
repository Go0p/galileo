use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;
use tracing::warn;

use super::{Cache, InMemoryBackend};

const ALT_BATCH_LIMIT: usize = 100;

/// 全局共享的 ALT 缓存，实现统一的批量拉取与解码逻辑。
#[derive(Clone)]
pub struct AltCache {
    inner: Cache<InMemoryBackend<Pubkey, AddressLookupTableAccount>>,
}

impl AltCache {
    pub fn new() -> Self {
        Self {
            inner: Cache::new(InMemoryBackend::default()),
        }
    }

    /// 批量获取地址查找表账户，命中缓存立即返回，未命中则通过 RPC 拉取。
    pub async fn fetch_many(
        &self,
        rpc: &Arc<RpcClient>,
        keys: &[Pubkey],
    ) -> Result<Vec<AddressLookupTableAccount>> {
        let mut result = Vec::new();
        let mut missing = Vec::new();
        for key in keys {
            if let Some(entry) = self.inner.get(key).await {
                result.push((*entry).clone());
            } else {
                missing.push(*key);
            }
        }

        if missing.is_empty() {
            return Ok(result);
        }

        let fetched = self.refresh_many(rpc, &missing).await?;
        result.extend(fetched);
        Ok(result)
    }

    /// 强制刷新一批 ALT，忽略缓存，常用于命中失败或需要确保最新状态的场景。
    pub async fn refresh_many(
        &self,
        rpc: &Arc<RpcClient>,
        keys: &[Pubkey],
    ) -> Result<Vec<AddressLookupTableAccount>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let mut collected = Vec::new();

        for chunk in keys.chunks(ALT_BATCH_LIMIT) {
            match rpc.get_multiple_accounts(chunk).await {
                Ok(accounts) => {
                    self.collect_accounts(chunk, accounts, &mut collected)
                        .await?;
                }
                Err(err) => {
                    warn!(
                        target: "cache::alt",
                        error = %err,
                        count = chunk.len(),
                        "批量拉取 ALT 失败，尝试逐条回退"
                    );
                    for address in chunk {
                        match rpc.get_account(address).await {
                            Ok(account) => {
                                if let Some(table) = self.decode_and_store(address, account).await?
                                {
                                    collected.push(table);
                                }
                            }
                            Err(fetch_err) => {
                                warn!(
                                    target: "cache::alt",
                                    address = %address,
                                    error = %fetch_err,
                                    "逐条拉取 ALT 失败"
                                );
                                self.inner.remove(address).await;
                            }
                        }
                    }
                }
            }
        }

        Ok(collected)
    }

    async fn collect_accounts(
        &self,
        addresses: &[Pubkey],
        accounts: Vec<Option<Account>>,
        collected: &mut Vec<AddressLookupTableAccount>,
    ) -> Result<()> {
        for (address, maybe_account) in addresses.iter().zip(accounts.into_iter()) {
            match maybe_account {
                Some(account) => {
                    if let Some(table) = self.decode_and_store(address, account).await? {
                        collected.push(table);
                    }
                }
                None => {
                    warn!(
                        target: "cache::alt",
                        address = %address,
                        "批量拉取 ALT 返回空账户"
                    );
                    self.inner.remove(address).await;
                }
            }
        }
        Ok(())
    }

    async fn decode_and_store(
        &self,
        address: &Pubkey,
        account: Account,
    ) -> Result<Option<AddressLookupTableAccount>> {
        match deserialize_lookup_table(address, account) {
            Ok(table) => {
                self.inner
                    .insert_arc(table.key, Arc::new(table.clone()), None)
                    .await;
                Ok(Some(table))
            }
            Err(err) => {
                warn!(
                    target: "cache::alt",
                    address = %address,
                    error = %err,
                    "反序列化 ALT 失败，移除缓存"
                );
                self.inner.remove(address).await;
                Ok(None)
            }
        }
    }
}

pub fn deserialize_lookup_table(
    address: &Pubkey,
    account: Account,
) -> Result<AddressLookupTableAccount> {
    AddressLookupTable::deserialize(&account.data)
        .map(|table| AddressLookupTableAccount {
            key: *address,
            addresses: table.addresses.into_owned(),
        })
        .map_err(|err| anyhow!("{err}"))
        .with_context(|| format!("反序列化 ALT 失败: {address}"))
}
