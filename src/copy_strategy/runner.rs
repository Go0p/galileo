use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use futures::StreamExt;
use solana_client::nonblocking::rpc_client::RpcClient;
use tracing::warn;

use crate::config::{CopyStrategyConfig, LanderSettings};
use crate::engine::{ComputeUnitPriceMode, DispatchStrategy, EngineIdentity, TransactionBuilder};
use crate::network::IpAllocator;

use super::wallet::CopyWalletRunner;

pub struct CopyStrategyRunner {
    pub(crate) config: CopyStrategyConfig,
    pub(crate) rpc_client: Arc<RpcClient>,
    pub(crate) tx_builder: TransactionBuilder,
    pub(crate) identity: EngineIdentity,
    pub(crate) ip_allocator: Arc<IpAllocator>,
    pub(crate) compute_unit_price_mode: Option<ComputeUnitPriceMode>,
    pub(crate) lander_factory: crate::lander::LanderFactory,
    pub(crate) lander_settings: LanderSettings,
    pub(crate) landing_timeout: Duration,
    pub(crate) dispatch_strategy: DispatchStrategy,
    pub(crate) dry_run: bool,
}

impl CopyStrategyRunner {
    pub async fn run(self) -> Result<()> {
        if self.config.wallets.is_empty() {
            warn!(target: "strategy", "copy_strategy.wallets 为空，直接退出");
            return Ok(());
        }

        let mut tasks = futures::stream::FuturesUnordered::new();

        for wallet in self.config.wallets.clone() {
            let runner = CopyWalletRunner::new(
                wallet,
                self.config.copy_dispatch.clone(),
                self.rpc_client.clone(),
                self.tx_builder.clone(),
                self.identity.clone(),
                self.ip_allocator.clone(),
                self.compute_unit_price_mode.clone(),
                self.lander_factory.clone(),
                self.lander_settings.clone(),
                self.landing_timeout,
                self.dispatch_strategy,
                self.dry_run,
            )
            .await
            .map_err(|err| anyhow!("初始化 copy wallet runner 失败: {err}"))?;

            tasks.push(tokio::spawn(runner.run()));
        }

        while let Some(result) = tasks.next().await {
            match result {
                Ok(Ok(())) => {}
                Ok(Err(err)) => return Err(err),
                Err(err) => {
                    if err.is_cancelled() {
                        return Err(anyhow!("copy wallet task cancelled"));
                    } else {
                        return Err(anyhow!("copy wallet task panicked: {err:?}"));
                    }
                }
            }
        }

        Ok(())
    }
}
