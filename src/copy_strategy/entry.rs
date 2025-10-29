use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use tracing::warn;

use crate::cli::context::{
    resolve_global_http_proxy, resolve_instruction_memo, resolve_rpc_client,
};
use crate::cli::strategy::{
    StrategyBackend, StrategyMode, build_http_client_pool, build_ip_allocator,
    build_rpc_client_pool,
};
use crate::config::{AppConfig, IntermediumConfig, LanderSettings};
use crate::engine::{ComputeUnitPriceMode, EngineIdentity, TransactionBuilder};
use crate::lander::LanderFactory;
use solana_sdk::pubkey::Pubkey;

use super::runner::CopyStrategyRunner;

pub async fn run_copy_strategy(
    config: &AppConfig,
    _backend: &StrategyBackend<'_>,
    mode: StrategyMode,
) -> Result<()> {
    let copy_config = &config.galileo.copy_strategy;
    if !copy_config.enable {
        warn!(target: "strategy", "复制策略未启用，直接退出");
        return Ok(());
    }

    let dry_run = matches!(mode, StrategyMode::DryRun) || config.galileo.bot.dry_run;

    let resolved_rpc = resolve_rpc_client(&config.galileo.global)?;
    let rpc_client = resolved_rpc.client.clone();
    let identity =
        EngineIdentity::from_wallet(&config.galileo.global.wallet).map_err(|err| anyhow!(err))?;

    let builder_config = crate::engine::BuilderConfig::new(resolve_instruction_memo(
        &config.galileo.global.instruction,
    ))
    .with_yellowstone(
        config.galileo.global.yellowstone_grpc_url.clone(),
        config.galileo.global.yellowstone_grpc_token.clone(),
        config.galileo.bot.get_block_hash_by_grpc,
    );

    let ip_allocator = build_ip_allocator(&config.galileo.bot.network)?;
    let global_proxy = resolve_global_http_proxy(&config.galileo.global);
    let rpc_client_pool =
        build_rpc_client_pool(resolved_rpc.endpoints.clone(), global_proxy.clone());

    let mut submission_builder = reqwest::Client::builder();
    if let Some(proxy_url) = global_proxy.as_ref() {
        let proxy = reqwest::Proxy::all(proxy_url.as_str())
            .map_err(|err| anyhow!("global.proxy 地址无效 {proxy_url}: {err}"))?;
        submission_builder = submission_builder
            .proxy(proxy)
            .danger_accept_invalid_certs(true);
    }
    let submission_client = submission_builder.build()?;
    let submission_client_pool = build_http_client_pool(None, global_proxy.clone(), false, None);

    let tx_builder = TransactionBuilder::new(
        rpc_client.clone(),
        builder_config,
        Arc::clone(&ip_allocator),
        Some(rpc_client_pool),
    );

    let intermediate_mints = Arc::new(parse_intermediate_mints(&config.galileo.intermedium)?);

    let compute_unit_price_mode = derive_compute_unit_price_mode(&config.lander.lander);
    let lander_factory = LanderFactory::new(
        rpc_client.clone(),
        submission_client.clone(),
        Some(Arc::clone(&submission_client_pool)),
    );

    let landing_timeout = resolve_landing_timeout(&config.galileo.bot);
    let dispatch_strategy = config.lander.lander.sending_strategy;

    let runner = CopyStrategyRunner {
        config: copy_config.clone(),
        rpc_client,
        tx_builder,
        identity,
        ip_allocator,
        compute_unit_price_mode,
        lander_factory,
        lander_settings: config.lander.lander.clone(),
        landing_timeout,
        dispatch_strategy,
        dry_run,
        intermediate_mints,
    };

    runner.run().await
}

fn parse_intermediate_mints(config: &IntermediumConfig) -> Result<HashSet<Pubkey>> {
    let mut set = HashSet::new();
    for mint in &config.mints {
        let trimmed = mint.trim();
        if trimmed.is_empty() {
            continue;
        }
        let pubkey = Pubkey::from_str(trimmed)
            .map_err(|err| anyhow!("intermedium.mints 中的 mint `{trimmed}` 解析失败: {err}"))?;
        set.insert(pubkey);
    }
    Ok(set)
}

fn derive_compute_unit_price_mode(settings: &LanderSettings) -> Option<ComputeUnitPriceMode> {
    let strategy = settings
        .compute_unit_price_strategy
        .trim()
        .to_ascii_lowercase();
    match strategy.as_str() {
        "" | "none" => None,
        "fixed" => settings
            .fixed_compute_unit_price
            .map(ComputeUnitPriceMode::Fixed),
        "random" => {
            if settings.random_compute_unit_price_range.len() >= 2 {
                let min = settings.random_compute_unit_price_range[0];
                let max = settings.random_compute_unit_price_range[1];
                Some(ComputeUnitPriceMode::Random { min, max })
            } else {
                warn!(
                    target: "strategy::copy",
                    "random compute unit price 需要提供上下限，忽略配置"
                );
                None
            }
        }
        other => {
            warn!(
                target: "strategy::copy",
                strategy = other,
                "未知的 compute_unit_price_strategy"
            );
            None
        }
    }
}

fn resolve_landing_timeout(bot: &crate::config::BotConfig) -> Duration {
    let ms = bot.landing_ms.unwrap_or(2_000).max(1);
    Duration::from_millis(ms as u64)
}
