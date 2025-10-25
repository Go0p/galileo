use std::collections::HashMap;
use std::sync::Arc;

use solana_address_lookup_table_interface::state::AddressLookupTable;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{account::Account, message::AddressLookupTableAccount, pubkey::Pubkey};
use tokio::sync::RwLock;
use tracing::{instrument, warn};

/// ALT 缓存：用于在 orchestrator/TransactionBuilder 阶段快速复用 lookup 内容。
#[derive(Clone, Default)]
pub struct AltCache {
    inner: Arc<RwLock<HashMap<Pubkey, AddressLookupTableAccount>>>,
}

impl AltCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, key: &Pubkey) -> Option<AddressLookupTableAccount> {
        self.inner.read().await.get(key).cloned()
    }

    pub async fn insert(&self, account: AddressLookupTableAccount) {
        self.inner.write().await.insert(account.key, account);
    }

    /// 从缓存或 RPC 加载一批 ALT，返回已解析好的账户列表。
    #[instrument(skip(self, rpc, keys), fields(len = keys.len()))]
    pub async fn fetch_many(
        &self,
        rpc: &Arc<RpcClient>,
        keys: &[Pubkey],
    ) -> anyhow::Result<Vec<AddressLookupTableAccount>> {
        let mut missing = Vec::new();
        let mut result = Vec::new();

        {
            let guard = self.inner.read().await;
            for key in keys {
                if let Some(account) = guard.get(key) {
                    result.push(account.clone());
                } else {
                    missing.push(*key);
                }
            }
        }

        if missing.is_empty() {
            return Ok(result);
        }

        let batched = rpc.get_multiple_accounts(&missing).await?;
        for (address, account) in missing.into_iter().zip(batched.into_iter()) {
            match account {
                Some(account) => {
                    if let Some(table) = deserialize_lookup_table(&address, account) {
                        self.insert(table.clone()).await;
                        result.push(table);
                    }
                }
                None => {
                    warn!(
                        target = "multi_leg::alt_cache",
                        address = %address,
                        "ALT 账户不存在"
                    );
                }
            }
        }

        Ok(result)
    }
}

fn deserialize_lookup_table(
    address: &Pubkey,
    account: Account,
) -> Option<AddressLookupTableAccount> {
    match AddressLookupTable::deserialize(&account.data) {
        Ok(table) => Some(AddressLookupTableAccount {
            key: *address,
            addresses: table.addresses.into_owned(),
        }),
        Err(err) => {
            warn!(
                target = "multi_leg::alt_cache",
                address = %address,
                error = %err,
                "反序列化 ALT 失败"
            );
            None
        }
    }
}
