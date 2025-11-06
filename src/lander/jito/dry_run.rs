use std::net::IpAddr;
use std::sync::Arc;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_sdk::transaction::VersionedTransaction;
use tracing::info;

use crate::config::LanderSettings;
use crate::engine::VariantId;
use crate::lander::error::LanderError;
use crate::lander::stack::{Deadline, LanderReceipt};

#[derive(Clone)]
pub(crate) struct DryRunFallback {
    client: Arc<RpcClient>,
    config: RpcSendTransactionConfig,
}

impl DryRunFallback {
    pub fn new(client: Arc<RpcClient>, settings: &LanderSettings) -> Self {
        let mut config = RpcSendTransactionConfig::default();
        if let Some(skip) = settings.skip_preflight {
            config.skip_preflight = skip;
        }
        if let Some(retries) = settings.max_retries {
            config.max_retries = Some(retries);
        }
        if let Some(slot) = settings.min_context_slot {
            config.min_context_slot = Some(slot);
        }
        Self { client, config }
    }

    pub async fn submit_transactions(
        &self,
        variant_id: VariantId,
        slot: u64,
        blockhash: &str,
        txs: &[VersionedTransaction],
        deadline: Deadline,
        endpoint_hint: Option<&str>,
        local_ip: Option<IpAddr>,
    ) -> Result<LanderReceipt, LanderError> {
        if deadline.expired() {
            return Err(LanderError::fatal(
                "deadline expired before dry-run jito submission",
            ));
        }

        if txs.is_empty() {
            return Err(LanderError::fatal(
                "dry-run bundle submission missing transactions",
            ));
        }

        let mut signatures = Vec::with_capacity(txs.len());
        for tx in txs {
            if deadline.expired() {
                return Err(LanderError::fatal(
                    "deadline expired during dry-run bundle submission",
                ));
            }
            let signature = self
                .client
                .send_transaction_with_config(tx, self.config.clone())
                .await?;
            signatures.push(signature.to_string());
        }

        let endpoint = endpoint_hint
            .map(|hint| hint.to_string())
            .unwrap_or_else(|| self.client.url().to_string());
        let joined_signatures = signatures.join(",");
        info!(
            target: "lander::jito::dry_run",
            endpoint = %endpoint,
            signatures = %joined_signatures,
            slot,
            blockhash,
            skip_preflight = self.config.skip_preflight,
            max_retries = ?self.config.max_retries,
            min_context_slot = ?self.config.min_context_slot,
            tx_count = txs.len(),
            "dry-run 模式：bundle 逐笔通过 RPC 提交"
        );

        Ok(LanderReceipt {
            lander: "jito",
            endpoint,
            slot,
            blockhash: blockhash.to_string(),
            signature: Some(joined_signatures),
            variant_id,
            local_ip,
        })
    }
}
