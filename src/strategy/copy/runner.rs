use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use futures::StreamExt;
use solana_client::nonblocking::rpc_client::RpcClient;
use tracing::warn;

use crate::config::{CopyStrategyConfig, LanderSettings};
use crate::engine::{
    ComputeUnitPriceMode, DispatchStrategy, EngineIdentity, LighthouseRuntime, LighthouseSettings,
    TransactionBuilder,
};
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
    pub(crate) wallet_refresh_interval: Option<Duration>,
    pub(crate) lighthouse_settings: LighthouseSettings,
}

impl CopyStrategyRunner {
    pub async fn run(self) -> Result<()> {
        if self.config.wallets.is_empty() {
            warn!(target: "strategy", "copy_strategy.wallets 为空，直接退出");
            return Ok(());
        }

        let Self {
            config,
            rpc_client,
            tx_builder,
            identity,
            ip_allocator,
            compute_unit_price_mode,
            lander_factory,
            lander_settings,
            landing_timeout,
            dispatch_strategy,
            dry_run,
            wallet_refresh_interval,
            lighthouse_settings,
        } = self;

        let allocator_summary = ip_allocator.summary();
        let per_ip_capacity = allocator_summary.per_ip_inflight_limit.unwrap_or(1).max(1);
        let ip_capacity_hint = allocator_summary
            .total_slots
            .max(1)
            .saturating_mul(per_ip_capacity);
        let lighthouse_runtime = LighthouseRuntime::new(&lighthouse_settings, ip_capacity_hint);
        let lighthouse = Arc::new(tokio::sync::Mutex::new(lighthouse_runtime));

        let mut tasks = futures::stream::FuturesUnordered::new();
        let wallets = config.wallets.clone();
        let copy_dispatch = config.copy_dispatch.clone();

        for wallet in wallets {
            let runner = CopyWalletRunner::new(
                wallet,
                copy_dispatch.clone(),
                rpc_client.clone(),
                tx_builder.clone(),
                identity.clone(),
                ip_allocator.clone(),
                compute_unit_price_mode.clone(),
                lander_factory.clone(),
                lander_settings.clone(),
                landing_timeout,
                dispatch_strategy,
                wallet_refresh_interval,
                dry_run,
                Arc::clone(&lighthouse),
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
