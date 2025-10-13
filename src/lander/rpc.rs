use std::sync::Arc;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::transaction::VersionedTransaction;
use tracing::info;

use crate::engine::PreparedTransaction;

use super::error::LanderError;
use super::stack::{Deadline, LanderReceipt};

#[derive(Clone)]
pub struct RpcLander {
    client: Arc<RpcClient>,
}

impl RpcLander {
    pub fn new(client: Arc<RpcClient>) -> Self {
        Self { client }
    }

    pub async fn submit(
        &self,
        prepared: &PreparedTransaction,
        deadline: Deadline,
    ) -> Result<LanderReceipt, LanderError> {
        if deadline.expired() {
            return Err(LanderError::fatal("deadline expired before rpc submission"));
        }

        self.send_via_rpc(
            &prepared.transaction,
            prepared.slot,
            &prepared.blockhash.to_string(),
        )
        .await
    }

    async fn send_via_rpc(
        &self,
        tx: &VersionedTransaction,
        slot: u64,
        blockhash: &str,
    ) -> Result<LanderReceipt, LanderError> {
        let signature = self.client.send_transaction(tx).await?;
        let endpoint = self.client.url();
        info!(
            target: "lander::rpc",
            signature = %signature,
            slot,
            blockhash,
            "transaction submitted via rpc client"
        );
        Ok(LanderReceipt {
            lander: "rpc",
            endpoint: endpoint.to_string(),
            slot,
            blockhash: blockhash.to_string(),
            signature: Some(signature.to_string()),
        })
    }
}
