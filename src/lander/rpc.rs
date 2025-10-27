use std::net::IpAddr;
use std::sync::Arc;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::transaction::VersionedTransaction;
use tracing::info;

use crate::engine::{TxVariant, VariantId};

use super::error::LanderError;
use super::stack::{Deadline, LanderReceipt};

#[derive(Clone)]
pub struct RpcLander {
    client: Arc<RpcClient>,
    config: RpcSendTransactionConfig,
}

impl RpcLander {
    pub fn new(
        client: Arc<RpcClient>,
        skip_preflight: Option<bool>,
        max_retries: Option<usize>,
        min_context_slot: Option<u64>,
    ) -> Self {
        let mut config = RpcSendTransactionConfig::default();
        if let Some(skip) = skip_preflight {
            config.skip_preflight = skip;
        }
        if let Some(retries) = max_retries {
            config.max_retries = Some(retries);
        }
        if let Some(slot) = min_context_slot {
            config.min_context_slot = Some(slot);
        }

        Self { client, config }
    }

    pub async fn submit_variant(
        &self,
        variant: TxVariant,
        deadline: Deadline,
        local_ip: Option<IpAddr>,
    ) -> Result<LanderReceipt, LanderError> {
        if deadline.expired() {
            return Err(LanderError::fatal("deadline expired before rpc submission"));
        }

        self.send_via_rpc(
            variant.id(),
            variant.transaction(),
            variant.slot(),
            &variant.blockhash().to_string(),
            local_ip,
        )
        .await
    }

    async fn send_via_rpc(
        &self,
        variant_id: VariantId,
        tx: &VersionedTransaction,
        slot: u64,
        blockhash: &str,
        local_ip: Option<IpAddr>,
    ) -> Result<LanderReceipt, LanderError> {
        let signature = self
            .client
            .send_transaction_with_config(tx, self.config.clone())
            .await?;
        let endpoint = self.client.url();
        info!(
            target: "lander::rpc",
            signature = %signature,
            slot,
            blockhash,
            skip_preflight = self.config.skip_preflight,
            max_retries = ?self.config.max_retries,
            min_context_slot = ?self.config.min_context_slot,
            "transaction submitted via rpc client"
        );
        Ok(LanderReceipt {
            lander: "rpc",
            endpoint: endpoint.to_string(),
            slot,
            blockhash: blockhash.to_string(),
            signature: Some(signature.to_string()),
            variant_id,
            local_ip,
        })
    }
}
