use std::sync::Arc;

use anyhow::Context;
use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{account::Account, message::AddressLookupTableAccount, pubkey::Pubkey};
use tracing::{instrument, warn};

use crate::cache::{Cache, FetchOutcome, InMemoryBackend};

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

    pub async fn get(&self, key: &Pubkey) -> Option<AddressLookupTableAccount> {
        self.inner.get(key).await.map(|entry| (*entry).clone())
    }

    pub async fn insert(&self, account: AddressLookupTableAccount) {
        self.inner.insert(account.key, account, None).await;
    }

    #[instrument(skip(self, rpc, keys), fields(len = keys.len()))]
    pub async fn fetch_many(
        &self,
        rpc: &Arc<RpcClient>,
        keys: &[Pubkey],
    ) -> anyhow::Result<Vec<AddressLookupTableAccount>> {
        let mut missing = Vec::new();
        let mut result = Vec::new();

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

    pub async fn refresh_many(
        &self,
        rpc: &Arc<RpcClient>,
        keys: &[Pubkey],
    ) -> anyhow::Result<Vec<AddressLookupTableAccount>> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let batched = rpc.get_multiple_accounts(keys).await?;
        let mut fetched = Vec::new();

        for (address, account) in keys.iter().zip(batched.into_iter()) {
            match account {
                Some(account) => match deserialize_lookup_table(address, account) {
                    Ok(table) => {
                        self.inner
                            .insert_arc(table.key, Arc::new(table.clone()), None)
                            .await;
                        fetched.push(table);
                    }
                    Err(_) => {
                        self.inner.remove(address).await;
                    }
                },
                None => {
                    warn!(
                        target = "multi_leg::alt_cache",
                        address = %address,
                        "ALT 账户不存在"
                    );
                    self.inner.remove(address).await;
                }
            }
        }

        Ok(fetched)
    }

    pub async fn load_or_fetch(
        &self,
        rpc: &Arc<RpcClient>,
        key: Pubkey,
    ) -> anyhow::Result<AddressLookupTableAccount> {
        let arc = self
            .inner
            .load_or_fetch(key, |address| {
                let rpc = Arc::clone(rpc);
                let address = *address;
                async move {
                    let account = rpc
                        .get_account(&address)
                        .await
                        .with_context(|| format!("获取 ALT 账户失败: {}", address))?;
                    let table = deserialize_lookup_table(&address, account)?;
                    Ok(FetchOutcome::new(table, None))
                }
            })
            .await?;
        Ok((*arc).clone())
    }
}

impl Default for AltCache {
    fn default() -> Self {
        Self::new()
    }
}

fn deserialize_lookup_table(
    address: &Pubkey,
    account: Account,
) -> anyhow::Result<AddressLookupTableAccount> {
    AddressLookupTable::deserialize(&account.data)
        .map(|table| AddressLookupTableAccount {
            key: *address,
            addresses: table.addresses.into_owned(),
        })
        .map_err(|err| {
            warn!(
                target = "multi_leg::alt_cache",
                address = %address,
                error = %err,
                "反序列化 ALT 失败"
            );
            err.into()
        })
}
