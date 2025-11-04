use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use solana_sdk::pubkey::Pubkey;
use tracing::warn;

use crate::cache::AltCache;
use crate::cli::context::{
    DryRunMode, override_proxy_selection, resolve_global_http_proxy, resolve_instruction_memo,
    resolve_proxy_profile, resolve_rpc_client,
};
use crate::cli::strategy::StrategyBackend;
use crate::config::launch::resources::{
    build_http_client_pool, build_http_client_with_options, build_ip_allocator,
    build_rpc_client_pool,
};
use crate::config::{AppConfig, LanderSettings, StrategyToggle};
use crate::engine::{
    ComputeUnitPriceMode, EngineIdentity, LighthouseSettings, SolPriceFeedSettings,
    TransactionBuilder,
};
use crate::lander::LanderFactory;

use super::runner::CopyStrategyRunner;

pub async fn run_copy_strategy(
    config: &AppConfig,
    _backend: &StrategyBackend<'_>,
    dry_run: &DryRunMode,
) -> Result<()> {
    let copy_config = &config.galileo.copy_strategy;
    if !config
        .galileo
        .bot
        .strategy_enabled(StrategyToggle::CopyStrategy)
    {
        warn!(target: "strategy", "复制策略未启用，直接退出");
        return Ok(());
    }
    let dry_run_enabled = dry_run.is_enabled();

    let resolved_rpc = resolve_rpc_client(&config.galileo.global, dry_run.rpc_override())?;
    let rpc_client = resolved_rpc.client.clone();
    let identity = EngineIdentity::from_private_key(&config.galileo.private_key)
        .map_err(|err| anyhow!(err))?;

    let enable_yellowstone = !dry_run_enabled && config.galileo.bot.get_block_hash_by_grpc;
    let builder_config = crate::engine::BuilderConfig::new(resolve_instruction_memo(
        &config.galileo.global.instruction,
    ))
    .with_yellowstone(
        config.galileo.global.yellowstone_grpc_url.clone(),
        config.galileo.global.yellowstone_grpc_token.clone(),
        enable_yellowstone,
    );

    let ip_allocator = build_ip_allocator(&config.galileo.bot.network)?;
    let global_proxy = if dry_run_enabled {
        None
    } else {
        resolve_global_http_proxy(&config.galileo.global)
    };
    let rpc_client_pool =
        build_rpc_client_pool(resolved_rpc.endpoints.clone(), global_proxy.clone());

    let lander_proxy = if dry_run_enabled {
        None
    } else {
        resolve_proxy_profile(&config.galileo.global, "lander")
    };
    let effective_proxy =
        override_proxy_selection(None, lander_proxy.clone(), global_proxy.clone());
    let submission_client =
        build_http_client_with_options(effective_proxy.as_ref(), false, None, None)?;
    let submission_client_pool = build_http_client_pool(effective_proxy.clone(), false, None);

    let tx_builder = TransactionBuilder::new(
        rpc_client.clone(),
        builder_config,
        Arc::clone(&ip_allocator),
        Some(rpc_client_pool),
        AltCache::new(),
        dry_run_enabled,
    );

    let compute_unit_price_mode = derive_compute_unit_price_mode(&config.lander.lander);
    let lander_factory = LanderFactory::new(
        rpc_client.clone(),
        submission_client.clone(),
        Some(Arc::clone(&submission_client_pool)),
        dry_run_enabled,
    );

    let landing_timeout = resolve_landing_timeout(&config.galileo.engine.time_out);
    let dispatch_strategy = config.lander.lander.sending_strategy;
    let wallet_refresh_interval = if config.galileo.bot.auto_refresh_wallet_minute == 0 {
        None
    } else {
        Some(Duration::from_secs(
            config
                .galileo
                .bot
                .auto_refresh_wallet_minute
                .saturating_mul(60),
        ))
    };
    let lighthouse_settings = build_lighthouse_settings(&config.galileo.bot.light_house)?;

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
        dry_run: dry_run_enabled,
        wallet_refresh_interval,
        lighthouse_settings,
    };

    runner.run().await
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

fn resolve_landing_timeout(timeouts: &crate::config::EngineTimeoutConfig) -> Duration {
    let ms = timeouts.landing_ms.max(1);
    Duration::from_millis(ms)
}

fn build_lighthouse_settings(
    cfg: &crate::config::LightHouseBotConfig,
) -> Result<LighthouseSettings> {
    if !cfg.enable {
        return Ok(LighthouseSettings::default());
    }

    let mut mints = Vec::with_capacity(cfg.profit_guard_mints.len());
    for mint_text in &cfg.profit_guard_mints {
        let trimmed = mint_text.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mint = Pubkey::from_str(trimmed).map_err(|err| {
            anyhow!(
                "light_house.profit_guard_mints 中的 mint `{}` 无效: {err}",
                trimmed
            )
        })?;
        mints.push(mint);
    }

    if mints.is_empty() {
        return Ok(LighthouseSettings::default());
    }

    let memory_slots = cfg
        .memory_slots
        .and_then(|value| if value == 0 { None } else { Some(value) });

    let sol_price_feed = cfg.sol_price_feed.as_ref().and_then(|feed| {
        let url = feed.url.trim();
        if url.is_empty() {
            None
        } else {
            Some(SolPriceFeedSettings {
                url: url.to_string(),
                refresh: Duration::from_millis(feed.refresh_ms.max(1)),
                guard_padding: feed.guard_padding,
            })
        }
    });

    Ok(LighthouseSettings {
        enable: true,
        profit_guard_mints: mints,
        memory_slots,
        existing_memory_ids: Vec::new(),
        sol_price_feed,
    })
}
