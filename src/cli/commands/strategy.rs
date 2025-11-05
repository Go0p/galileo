use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

use crate::api::dflow::DflowApiClient;
use crate::api::jupiter::JupiterApiClient;
use crate::api::kamino::KaminoApiClient;
use crate::api::titan::TitanSubscriptionConfig;
use crate::api::ultra::UltraApiClient;
use crate::cache::AltCache;
use crate::cli::context::{
    DryRunMode, override_proxy_selection, resolve_global_http_proxy, resolve_instruction_memo,
    resolve_proxy_profile, resolve_rpc_client,
};
use crate::config;
use crate::config::launch::resources::{
    build_http_client_pool, build_http_client_with_options, build_ip_allocator,
    build_rpc_client_pool,
};
use crate::config::{
    AppConfig, EngineLegBackend, FlashloanProduct, IntermediumConfig, LegRole, StrategyToggle,
};
use crate::engine::multi_leg::{
    orchestrator::MultiLegOrchestrator,
    providers::{
        dflow::DflowLegProvider,
        kamino::KaminoLegProvider,
        titan::{TitanLegProvider, TitanWsQuoteSource},
        ultra::UltraLegProvider,
    },
    runtime::{MultiLegRuntime, MultiLegRuntimeConfig},
    types::{AggregatorKind as MultiLegAggregatorKind, LegSide},
};
use crate::engine::plugins::flashloan::{MarginfiAccountRegistry, MarginfiFlashloanManager};
use crate::engine::{
    AccountPrechecker, BuilderConfig, ComputeUnitPriceMode, ConsoleSummarySettings, EngineError,
    EngineIdentity, EngineResult, EngineSettings, LighthouseSettings, MultiLegEngineContext,
    ProfitConfig, ProfitEvaluator, QuoteCadence, QuoteConfig, QuoteExecutor, Scheduler,
    SolPriceFeedSettings, StrategyEngine, SwapPreparer, TipConfig, TradeProfile,
    TransactionBuilder,
};
use crate::lander::LanderFactory;
use crate::monitoring::events;
use crate::network::IpAllocator;
use crate::strategy::pure_blind::cache::PureBlindCacheManager;
use crate::strategy::pure_blind::dynamic::spawn_dynamic_worker;
use crate::strategy::pure_blind::observer::{
    PoolActivationPolicy, PoolCatalog, PoolObserverSettings, RouteActivationPolicy, RouteCatalog,
    spawn_pool_observer,
};
use crate::strategy::run_copy_strategy;
use crate::strategy::{
    BlindStrategy, PureBlindRouteBuilder, PureBlindStrategy, Strategy, StrategyEvent,
};
use rand::Rng as _;
use solana_sdk::pubkey::Pubkey;
use url::Url;
use yellowstone_grpc_proto::tonic::metadata::AsciiMetadataValue;

/// 控制策略以正式模式还是 dry-run 模式运行。
pub enum StrategyMode {
    Live,
    DryRun,
}

pub enum StrategyBackend<'a> {
    Jupiter {
        api_client: &'a JupiterApiClient,
    },
    Dflow {
        api_client: &'a DflowApiClient,
    },
    Kamino {
        api_client: &'a KaminoApiClient,
        rpc_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    },
    Ultra {
        api_client: &'a UltraApiClient,
        rpc_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    },
    None,
}

/// 主策略入口，按配置在盲发与 copy 之间切换。
pub async fn run_strategy(
    config: &AppConfig,
    backend: &StrategyBackend<'_>,
    mode: StrategyMode,
) -> Result<()> {
    let dry_run_mode = DryRunMode::from_sources(
        matches!(mode, StrategyMode::DryRun),
        &config.galileo.bot.dry_run,
    )?;

    let blind_enabled = config
        .galileo
        .bot
        .strategy_enabled(StrategyToggle::BlindStrategy);
    let pure_enabled = config
        .galileo
        .bot
        .strategy_enabled(StrategyToggle::PureBlindStrategy);
    let copy_enabled = config
        .galileo
        .bot
        .strategy_enabled(StrategyToggle::CopyStrategy);

    if copy_enabled {
        if blind_enabled || pure_enabled {
            warn!(
                target: "strategy",
                "copy_strategy 已启用，其余策略配置将被忽略"
            );
        }
        return run_copy_strategy(config, backend, &dry_run_mode).await;
    }

    match config.galileo.engine.backend {
        crate::config::EngineBackend::MultiLegs => {
            if !blind_enabled {
                return Err(anyhow!(
                    "multi-legs 模式暂需在 bot.strategies.enabled 中启用 blind_strategy 以提供 trade pair 配置"
                ));
            }
            run_blind_engine(config, backend, &dry_run_mode, true).await
        }
        crate::config::EngineBackend::None => {
            if blind_enabled {
                return Err(anyhow!(
                    "engine.backend=none 仅支持纯盲发或 copy 策略，请从 bot.strategies.enabled 中移除 blind_strategy"
                ));
            }
            if !pure_enabled {
                warn!(
                    target: "strategy",
                    "纯盲发策略未启用，且 copy_strategy 也为关闭状态，直接退出"
                );
                return Ok(());
            }
            run_pure_blind_engine(config, backend, &dry_run_mode).await
        }
        _ => match (blind_enabled, pure_enabled) {
            (false, false) => {
                warn!(target: "strategy", "盲发策略未启用，直接退出");
                Ok(())
            }
            (true, true) => Err(anyhow!(
                "bot.strategies.enabled 中 blind_strategy 与 pure_blind_strategy 不能同时启用"
            )),
            (true, false) => run_blind_engine(config, backend, &dry_run_mode, false).await,
            (false, true) => run_pure_blind_engine(config, backend, &dry_run_mode).await,
        },
    }
}

async fn run_blind_engine(
    config: &AppConfig,
    backend: &StrategyBackend<'_>,
    dry_run: &DryRunMode,
    multi_leg_mode: bool,
) -> Result<()> {
    let blind_config = &config.galileo.blind_strategy;
    let dry_run_enabled = dry_run.is_enabled();

    if !config
        .galileo
        .bot
        .strategy_enabled(StrategyToggle::BlindStrategy)
    {
        warn!(target: "strategy", "盲发策略未启用，直接退出");
        return Ok(());
    }

    let compute_unit_price_mode = derive_compute_unit_price_mode(&config.lander.lander);

    let resolved_rpc = resolve_rpc_client(&config.galileo.global, dry_run.rpc_override())?;
    let rpc_client = resolved_rpc.client.clone();
    let rpc_endpoints = resolved_rpc.endpoints.clone();
    let mut identity = EngineIdentity::from_private_key(&config.galileo.private_key)
        .map_err(|err| anyhow!(err))?;

    let ip_allocator = build_ip_allocator(&config.galileo.bot.network)?;
    let alt_cache = AltCache::new();

    let multi_leg_runtime = if multi_leg_mode {
        let runtime = Arc::new(build_multi_leg_runtime(
            config,
            &identity,
            rpc_client.clone(),
            Arc::clone(&ip_allocator),
            alt_cache.clone(),
        )?);
        log_multi_leg_init(runtime.as_ref(), dry_run_enabled);
        Some(runtime)
    } else {
        None
    };

    let (quote_executor, swap_preparer, quote_defaults_tuple) = prepare_swap_components(
        config,
        backend,
        &mut identity,
        &compute_unit_price_mode,
        alt_cache.clone(),
    )
    .await?;

    let (only_direct_default, _) = quote_defaults_tuple;
    let allocator_summary = ip_allocator.summary();
    let per_ip_capacity = allocator_summary.per_ip_inflight_limit.unwrap_or(1).max(1);
    let ip_capacity_hint = allocator_summary
        .total_slots
        .max(1)
        .saturating_mul(per_ip_capacity);
    let trade_pairs = build_blind_trade_pairs(blind_config, &config.galileo.intermedium)?;
    let trade_profiles = build_blind_trade_profiles(blind_config, ip_capacity_hint)?;
    let multi_leg_context = multi_leg_runtime.as_ref().map(|runtime| {
        let mut ctx = MultiLegEngineContext::from_runtime(Arc::clone(runtime));
        let dflow_defaults = &config.galileo.engine.dflow.swap_config;
        ctx.set_wrap_and_unwrap_sol(
            MultiLegAggregatorKind::Dflow,
            dflow_defaults.wrap_and_unwrap_sol,
        );
        ctx.set_dynamic_compute_unit_limit(
            MultiLegAggregatorKind::Dflow,
            dflow_defaults.dynamic_compute_unit_limit,
        );
        ctx.set_compute_unit_limit_multiplier(
            MultiLegAggregatorKind::Dflow,
            dflow_defaults.cu_limit_multiplier,
        );
        let kamino_defaults = &config.galileo.engine.kamino.quote_config;
        ctx.set_wrap_and_unwrap_sol(
            MultiLegAggregatorKind::Kamino,
            kamino_defaults.wrap_and_unwrap_sol,
        );
        ctx
    });
    let profit_config = build_blind_profit_config(
        blind_config,
        &config.lander.lander,
        &compute_unit_price_mode,
    );
    let quote_config = build_blind_quote_config(blind_config, only_direct_default);
    tracing::info!(
        target: "engine::config",
        dex_whitelist = ?quote_config.dex_whitelist,
        "盲发策略 DEX 白名单"
    );
    let landing_timeout = resolve_landing_timeout(&config.galileo.engine.time_out);

    let enable_yellowstone = !dry_run_enabled && config.galileo.bot.get_block_hash_by_grpc;
    let builder_config =
        BuilderConfig::new(resolve_instruction_memo(&config.galileo.global.instruction))
            .with_yellowstone(
                config.galileo.global.yellowstone_grpc_url.clone(),
                config.galileo.global.yellowstone_grpc_token.clone(),
                enable_yellowstone,
            );
    let global_proxy = if dry_run_enabled {
        None
    } else {
        resolve_global_http_proxy(&config.galileo.global)
    };
    let rpc_client_pool = build_rpc_client_pool(rpc_endpoints.clone(), global_proxy.clone());

    let lander_proxy = if dry_run_enabled {
        None
    } else {
        resolve_proxy_profile(&config.galileo.global, "lander")
    };
    let effective_lander_proxy =
        override_proxy_selection(None, lander_proxy.clone(), global_proxy.clone());
    let submission_client =
        build_http_client_with_options(effective_lander_proxy.as_ref(), false, None, None)?;
    let submission_client_pool =
        build_http_client_pool(effective_lander_proxy.clone(), false, None);
    let tx_builder = TransactionBuilder::new(
        rpc_client.clone(),
        builder_config,
        Arc::clone(&ip_allocator),
        Some(rpc_client_pool),
        alt_cache.clone(),
        dry_run_enabled,
    );

    let marginfi_cfg = &config.galileo.flashloan.marginfi;
    let marginfi_accounts = parse_marginfi_accounts(marginfi_cfg)?;
    let flashloan_enabled = config
        .galileo
        .bot
        .flashloan_enabled(FlashloanProduct::Marginfi);
    let prefer_wallet_balance = config.galileo.bot.flashloan.prefer_wallet_balance;

    let prechecker = AccountPrechecker::new(rpc_client.clone(), marginfi_accounts.clone());
    let (summary, flashloan_precheck) = prechecker
        .ensure_accounts(&identity, &trade_pairs, flashloan_enabled)
        .await
        .map_err(|err| anyhow!(err))?;
    let skipped = summary.total_mints.saturating_sub(summary.processed_mints);
    events::accounts_precheck(
        "blind",
        summary.total_mints,
        summary.created_accounts,
        skipped,
    );

    if identity.skip_user_accounts_rpc_calls() {
        info!(
            target: "strategy",
            "skip_user_accounts_rpc_calls 仅作用于 swap-instructions 请求，账户预检查仍已执行"
        );
    }

    if let Some(prep) = &flashloan_precheck {
        events::flashloan_account_precheck("blind", &prep.account, prep.created);
    }
    let mut recorded_accounts: BTreeSet<Pubkey> = BTreeSet::new();
    if let Some(prep) = &flashloan_precheck {
        recorded_accounts.insert(prep.account);
    }
    if flashloan_precheck.is_none() {
        if let Some(account) = marginfi_accounts.default() {
            if recorded_accounts.insert(account) {
                events::flashloan_account_precheck("blind", &account, false);
            }
        }
    }

    let mut flashloan_manager = MarginfiFlashloanManager::new(
        marginfi_cfg,
        flashloan_enabled,
        prefer_wallet_balance,
        rpc_client.clone(),
        marginfi_accounts.clone(),
    );
    let mut flashloan_precheck = flashloan_precheck;
    if let Some(prep) = flashloan_precheck.clone() {
        flashloan_manager.adopt_preparation(prep);
    } else if flashloan_manager.is_enabled() {
        flashloan_precheck = flashloan_manager
            .prepare(&identity)
            .await
            .map_err(|err| anyhow!(err))?;
        if let Some(prep) = &flashloan_precheck {
            events::flashloan_account_precheck("blind", &prep.account, prep.created);
        }
    }
    let flashloan = flashloan_manager.try_into_enabled();

    let lander_factory = LanderFactory::new(
        rpc_client.clone(),
        submission_client.clone(),
        Some(Arc::clone(&submission_client_pool)),
        dry_run_enabled,
        config.galileo.bot.enable_simulation,
    );
    let default_landers = ["rpc"];

    let requested_landers: Vec<String> = if dry_run_enabled {
        if blind_config.enable_landers.is_empty() {
            vec!["rpc".to_string()]
        } else {
            blind_config.enable_landers.clone()
        }
    } else {
        blind_config.enable_landers.clone()
    };

    let lander_stack = lander_factory
        .build_stack(
            &config.lander.lander,
            &requested_landers,
            &default_landers,
            0,
            Arc::clone(&ip_allocator),
        )
        .map_err(|err| anyhow!(err))?;
    let lander_stack = Arc::new(lander_stack);

    let quote_cadence = resolve_quote_cadence(&config.galileo.engine, backend);

    let mut lighthouse_settings = parse_lighthouse_settings(&config.galileo.bot.light_house)?;
    if lighthouse_settings.enable {
        let mut existing_memory_ids = prechecker
            .detect_lighthouse_memory_accounts(&identity)
            .await
            .map_err(|err| anyhow!(err))?;
        existing_memory_ids.sort_unstable();
        existing_memory_ids.dedup();
        if !existing_memory_ids.is_empty() {
            info!(
                target: "strategy::lighthouse",
                count = existing_memory_ids.len(),
                "检测到 {} 个已有 Lighthouse memory 账户，将优先复用",
                existing_memory_ids.len()
            );
        }
        lighthouse_settings.existing_memory_ids = existing_memory_ids;
    }

    let console_summary_settings = ConsoleSummarySettings {
        enable: config.galileo.engine.enable_console_summary,
    };

    let engine_settings = EngineSettings::new(quote_config)
        .with_quote_cadence(quote_cadence)
        .with_dispatch_strategy(config.lander.lander.sending_strategy)
        .with_landing_timeout(landing_timeout)
        .with_dry_run(dry_run_enabled)
        .with_cu_multiplier(1.0)
        .with_compute_unit_price_mode(compute_unit_price_mode.clone())
        .with_lighthouse(lighthouse_settings)
        .with_console_summary(console_summary_settings);

    let strategy_engine = StrategyEngine::new(
        BlindStrategy::new(),
        lander_stack.clone(),
        identity,
        ip_allocator,
        quote_executor,
        ProfitEvaluator::new(profit_config, config.galileo.bot.network.enable_multiple_ip),
        swap_preparer,
        tx_builder,
        Scheduler::new(),
        flashloan,
        engine_settings,
        trade_pairs,
        trade_profiles,
        multi_leg_context,
    );
    drive_engine(strategy_engine)
        .await
        .map_err(|err| anyhow!(err))?;

    Ok(())
}

fn build_multi_leg_runtime(
    config: &AppConfig,
    identity: &EngineIdentity,
    rpc_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    ip_allocator: Arc<IpAllocator>,
    alt_cache: AltCache,
) -> Result<MultiLegRuntime> {
    let mut orchestrator = MultiLegOrchestrator::new();

    register_ultra_leg(&mut orchestrator, config, identity)?;
    register_dflow_leg(&mut orchestrator, config)?;
    register_kamino_leg(
        &mut orchestrator,
        config,
        Arc::clone(&rpc_client),
        alt_cache.clone(),
    )?;
    register_titan_leg(&mut orchestrator, config)?;

    if orchestrator.buy_legs().is_empty() {
        return Err(anyhow!("multi-legs 模式至少需要一个 buy 腿"));
    }
    if orchestrator.sell_legs().is_empty() {
        return Err(anyhow!("multi-legs 模式至少需要一个 sell 腿"));
    }

    let runtime_cfg = MultiLegRuntimeConfig::default();

    Ok(MultiLegRuntime::with_config(
        orchestrator,
        alt_cache,
        rpc_client,
        ip_allocator,
        runtime_cfg,
    ))
}

fn log_multi_leg_init(runtime: &MultiLegRuntime, dry_run: bool) {
    let buy = runtime.orchestrator().buy_legs();
    let sell = runtime.orchestrator().sell_legs();
    let pairs = runtime.orchestrator().available_pairs();
    info!(
        target: "multi_leg::init",
        buy_legs = buy.len(),
        sell_legs = sell.len(),
        pair_count = pairs.len(),
        dry_run,
        "Multi-leg runtime 已初始化"
    );

    for descriptor in buy.iter() {
        info!(
            target: "multi_leg::init",
            side = %descriptor.side,
            aggregator = %descriptor.kind,
            "可用买腿"
        );
    }
    for descriptor in sell.iter() {
        info!(
            target: "multi_leg::init",
            side = %descriptor.side,
            aggregator = %descriptor.kind,
            "可用卖腿"
        );
    }
    for pair in pairs {
        debug!(
            target: "multi_leg::init",
            buy = %pair.buy.kind,
            sell = %pair.sell.kind,
            "腿组合已接入"
        );
    }
}

fn register_ultra_leg(
    orchestrator: &mut MultiLegOrchestrator,
    config: &AppConfig,
    identity: &EngineIdentity,
) -> Result<()> {
    let ultra_cfg = &config.galileo.engine.ultra;
    let leg = ultra_cfg
        .leg
        .ok_or_else(|| anyhow!("ultra.leg 必须在 multi-legs 模式下配置"))?;

    let leg_allowed = match leg {
        LegRole::Buy => config
            .galileo
            .bot
            .engines
            .buy_leg_enabled(EngineLegBackend::Ultra),
        LegRole::Sell => config
            .galileo
            .bot
            .engines
            .sell_leg_enabled(EngineLegBackend::Ultra),
    };
    if !leg_allowed {
        return Ok(());
    }
    let api_base = ultra_cfg
        .api_quote_base
        .as_ref()
        .ok_or_else(|| anyhow!("ultra.api_quote_base 未配置"))?
        .trim()
        .to_string();
    if api_base.is_empty() {
        return Err(anyhow!("ultra.api_quote_base 不能为空"));
    }

    let proxy_override = ultra_cfg
        .api_proxy
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());
    let module_proxy = resolve_proxy_profile(&config.galileo.global, "quote");
    let global_proxy = resolve_global_http_proxy(&config.galileo.global);
    let effective_proxy =
        override_proxy_selection(proxy_override, module_proxy.clone(), global_proxy.clone());

    if let Some(url) = proxy_override {
        info!(
            target: "ultra",
            proxy = %url,
            "Ultra API 请求将通过配置的代理发送"
        );
    } else if let Some(selection) = module_proxy {
        info!(
            target: "ultra",
            proxy = %selection.url,
            per_request = selection.per_request,
            "Ultra API 请求将通过 profile 代理发送"
        );
    } else if let Some(selection) = global_proxy.clone() {
        info!(
            target: "ultra",
            proxy = %selection.url,
            per_request = selection.per_request,
            "Ultra API 请求将通过全局代理发送"
        );
    }

    let http_client = build_http_client_with_options(effective_proxy.as_ref(), false, None, None)?;
    let http_pool = build_http_client_pool(effective_proxy.clone(), false, None);

    let ultra_client = UltraApiClient::with_ip_pool(
        http_client,
        api_base,
        &config.galileo.engine.time_out,
        &config.galileo.global.logging,
        Some(http_pool),
    );
    let request_taker_override = ultra_cfg
        .quote_config
        .taker
        .as_ref()
        .map(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Pubkey::from_str(trimmed)
                    .map(Some)
                    .map_err(|err| anyhow!("ultra.quote_config.payer 解析失败: {err}"))
            }
        })
        .transpose()?
        .flatten();

    let provider = UltraLegProvider::new(
        ultra_client,
        LegSide::from(leg),
        ultra_cfg.quote_config.clone(),
        identity.pubkey,
        request_taker_override,
    );
    orchestrator.register_owned_provider(provider);
    Ok(())
}

fn register_dflow_leg(orchestrator: &mut MultiLegOrchestrator, config: &AppConfig) -> Result<()> {
    let dflow_cfg = &config.galileo.engine.dflow;
    let leg = dflow_cfg
        .leg
        .ok_or_else(|| anyhow!("dflow.leg 必须在 multi-legs 模式下配置"))?;

    let leg_allowed = match leg {
        LegRole::Buy => config
            .galileo
            .bot
            .engines
            .buy_leg_enabled(EngineLegBackend::Dflow),
        LegRole::Sell => config
            .galileo
            .bot
            .engines
            .sell_leg_enabled(EngineLegBackend::Dflow),
    };
    if !leg_allowed {
        return Ok(());
    }
    let quote_base = dflow_cfg
        .api_quote_base
        .as_ref()
        .ok_or_else(|| anyhow!("dflow.api_quote_base 未配置"))?
        .trim()
        .to_string();
    if quote_base.is_empty() {
        return Err(anyhow!("dflow.api_quote_base 不能为空"));
    }
    let swap_base = dflow_cfg
        .api_swap_base
        .as_ref()
        .unwrap_or(&quote_base)
        .trim()
        .to_string();

    let proxy_override = dflow_cfg
        .api_proxy
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());
    let module_proxy = resolve_proxy_profile(&config.galileo.global, "quote");
    let global_proxy = resolve_global_http_proxy(&config.galileo.global);
    let effective_proxy =
        override_proxy_selection(proxy_override, module_proxy.clone(), global_proxy.clone());

    if let Some(url) = proxy_override {
        info!(
            target: "dflow",
            proxy = %url,
            "DFlow API 请求将通过配置的代理发送"
        );
    } else if let Some(selection) = module_proxy {
        info!(
            target: "dflow",
            proxy = %selection.url,
            per_request = selection.per_request,
            "DFlow API 请求将通过 profile 代理发送"
        );
    } else if let Some(selection) = global_proxy.clone() {
        info!(
            target: "dflow",
            proxy = %selection.url,
            per_request = selection.per_request,
            "DFlow API 请求将通过全局代理发送"
        );
    }

    let http_client = build_http_client_with_options(effective_proxy.as_ref(), false, None, None)?;
    let http_pool = build_http_client_pool(effective_proxy.clone(), false, None);

    let dflow_client = DflowApiClient::with_ip_pool(
        http_client,
        quote_base,
        swap_base,
        &config.galileo.engine.time_out,
        &config.galileo.global.logging,
        Some(http_pool),
    );

    let provider = DflowLegProvider::new(
        dflow_client,
        dflow_cfg.quote_config.clone(),
        dflow_cfg.swap_config.clone(),
        LegSide::from(leg),
        Vec::new(),
        Vec::new(),
    );
    orchestrator.register_owned_provider(provider);
    Ok(())
}

fn register_kamino_leg(
    orchestrator: &mut MultiLegOrchestrator,
    config: &AppConfig,
    rpc_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    alt_cache: AltCache,
) -> Result<()> {
    let kamino_cfg = &config.galileo.engine.kamino;
    let leg = kamino_cfg
        .leg
        .ok_or_else(|| anyhow!("kamino.leg 必须在 multi-legs 模式下配置"))?;

    let leg_allowed = match leg {
        LegRole::Buy => config
            .galileo
            .bot
            .engines
            .buy_leg_enabled(EngineLegBackend::Kamino),
        LegRole::Sell => config
            .galileo
            .bot
            .engines
            .sell_leg_enabled(EngineLegBackend::Kamino),
    };
    if !leg_allowed {
        return Ok(());
    }
    let quote_base = kamino_cfg
        .api_quote_base
        .as_ref()
        .ok_or_else(|| anyhow!("kamino.api_quote_base 未配置"))?
        .trim()
        .to_string();
    if quote_base.is_empty() {
        return Err(anyhow!("kamino.api_quote_base 不能为空"));
    }
    let swap_base = kamino_cfg
        .api_swap_base
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| quote_base.clone());

    let proxy_override = kamino_cfg
        .api_proxy
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());
    let module_proxy = resolve_proxy_profile(&config.galileo.global, "quote");
    let global_proxy = resolve_global_http_proxy(&config.galileo.global);
    let effective_proxy =
        override_proxy_selection(proxy_override, module_proxy.clone(), global_proxy.clone());

    if let Some(url) = proxy_override {
        info!(
            target: "kamino",
            proxy = %url,
            "Kamino API 请求将通过配置的代理发送"
        );
    } else if let Some(selection) = module_proxy {
        info!(
            target: "kamino",
            proxy = %selection.url,
            per_request = selection.per_request,
            "Kamino API 请求将通过 profile 代理发送"
        );
    } else if let Some(selection) = global_proxy.clone() {
        info!(
            target: "kamino",
            proxy = %selection.url,
            per_request = selection.per_request,
            "Kamino API 请求将通过全局代理发送"
        );
    }

    let http_client = build_http_client_with_options(effective_proxy.as_ref(), false, None, None)?;
    let http_pool = build_http_client_pool(effective_proxy.clone(), false, None);

    let kamino_client = KaminoApiClient::with_ip_pool(
        http_client,
        quote_base,
        swap_base,
        &config.galileo.engine.time_out,
        &config.galileo.global.logging,
        Some(http_pool),
    );

    let provider = KaminoLegProvider::new(
        kamino_client,
        LegSide::from(leg),
        kamino_cfg.quote_config.clone(),
        rpc_client,
        alt_cache,
    );
    orchestrator.register_owned_provider(provider);
    Ok(())
}

fn parse_lighthouse_settings(cfg: &config::LightHouseBotConfig) -> Result<LighthouseSettings> {
    if !cfg.enable {
        return Ok(LighthouseSettings::default());
    }

    let mut mints = Vec::with_capacity(cfg.profit_guard_mints.len());
    for mint_text in &cfg.profit_guard_mints {
        let trimmed = mint_text.trim();
        if trimmed.is_empty() {
            continue;
        }
        let pubkey = Pubkey::from_str(trimmed).map_err(|err| {
            anyhow!(
                "light_house.profit_guard_mints 中的 mint `{}` 无效: {err}",
                trimmed
            )
        })?;
        mints.push(pubkey);
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

fn register_titan_leg(orchestrator: &mut MultiLegOrchestrator, config: &AppConfig) -> Result<()> {
    let titan_cfg = &config.galileo.engine.titan;
    if !config
        .galileo
        .bot
        .engines
        .buy_leg_enabled(EngineLegBackend::Titan)
    {
        return Ok(());
    }

    if titan_cfg.leg != Some(LegRole::Buy) {
        return Err(anyhow!(
            "Titan 仅支持 buy 腿，请设置 engine.titan.leg = \"buy\""
        ));
    }

    let ws_url = titan_cfg
        .ws_url
        .as_ref()
        .ok_or_else(|| anyhow!("titan.ws_url 未配置"))?
        .trim();
    if ws_url.is_empty() {
        return Err(anyhow!("titan.ws_url 不能为空"));
    }
    let ws_url = Url::parse(ws_url).map_err(|err| anyhow!("titan.ws_url 无效: {err}"))?;
    let ws_proxy = match titan_cfg
        .ws_proxy
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        Some(proxy) => {
            Some(Url::parse(proxy).map_err(|err| anyhow!("titan.ws_proxy 无效: {err}"))?)
        }
        None => None,
    };

    let jwt = titan_cfg
        .jwt
        .as_ref()
        .ok_or_else(|| anyhow!("titan.jwt 未配置"))?
        .trim()
        .to_string();
    if jwt.is_empty() {
        return Err(anyhow!("titan.jwt 不能为空"));
    }

    let tx_user_pubkey = titan_cfg
        .tx_config
        .user_public_key
        .as_deref()
        .ok_or_else(|| anyhow!("titan.tx_config.user_public_key 未配置"))?
        .trim();
    if tx_user_pubkey.is_empty() {
        return Err(anyhow!("titan.tx_config.user_public_key 不能为空字符串"));
    }
    let user_pubkey = solana_sdk::pubkey::Pubkey::from_str(tx_user_pubkey)
        .map_err(|err| anyhow!("Titan user public key 无效: {err}"))?;

    let sanitize_list = |values: &[String]| -> Vec<String> {
        values
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string())
            .collect()
    };
    let swap_providers = {
        let from_swap = sanitize_list(&titan_cfg.swap_config.providers);
        if !from_swap.is_empty() {
            from_swap
        } else {
            sanitize_list(&titan_cfg.providers)
        }
    };
    let dexes = sanitize_list(&titan_cfg.swap_config.dexes);
    let exclude_dexes = sanitize_list(&titan_cfg.swap_config.exclude_dexes);

    let subscription_cfg = TitanSubscriptionConfig {
        ws_url,
        ws_proxy,
        jwt,
        user_pubkey,
        providers: swap_providers,
        dexes,
        exclude_dexes,
        only_direct_routes: titan_cfg.swap_config.only_direct_routes,
        update_interval_ms: titan_cfg.interval_ms,
        update_num_quotes: titan_cfg.num_quotes,
        close_input_token_account: false,
        create_output_token_account: titan_cfg.tx_config.create_output_token_account,
    };

    let first_quote_timeout = titan_cfg
        .first_quote_timeout_ms
        .and_then(|ms| (ms > 0).then_some(Duration::from_millis(ms)));

    let quote_source = TitanWsQuoteSource::new(subscription_cfg, first_quote_timeout);
    let provider = TitanLegProvider::new(
        quote_source,
        LegSide::Buy,
        user_pubkey,
        titan_cfg.tx_config.use_wsol,
    );
    orchestrator.register_owned_provider(provider);
    Ok(())
}

fn resolve_quote_cadence(
    engine: &config::EngineConfig,
    backend: &StrategyBackend<'_>,
) -> QuoteCadence {
    match backend {
        StrategyBackend::Jupiter { .. } => {
            QuoteCadence::from_config(&engine.jupiter.quote_config.cadence)
        }
        StrategyBackend::Dflow { .. } => {
            QuoteCadence::from_config(&engine.dflow.quote_config.cadence)
        }
        StrategyBackend::Kamino { .. } => {
            QuoteCadence::from_config(&engine.kamino.quote_config.cadence)
        }
        StrategyBackend::Ultra { .. } => {
            QuoteCadence::from_config(&engine.ultra.quote_config.cadence)
        }
        StrategyBackend::None => QuoteCadence::default(),
    }
}

async fn run_pure_blind_engine(
    config: &AppConfig,
    backend: &StrategyBackend<'_>,
    dry_run: &DryRunMode,
) -> Result<()> {
    let pure_config = &config.galileo.pure_blind_strategy;
    if !config
        .galileo
        .bot
        .strategy_enabled(StrategyToggle::PureBlindStrategy)
    {
        warn!(target: "strategy", "纯盲发策略未启用，直接退出");
        return Ok(());
    }

    let dry_run_enabled = dry_run.is_enabled();
    let compute_unit_price_mode = derive_compute_unit_price_mode(&config.lander.lander);

    let resolved_rpc = resolve_rpc_client(&config.galileo.global, dry_run.rpc_override())?;
    let rpc_client = resolved_rpc.client.clone();
    let rpc_endpoints = resolved_rpc.endpoints.clone();
    let mut identity = EngineIdentity::from_private_key(&config.galileo.private_key)
        .map_err(|err| anyhow!(err))?;
    let alt_cache = AltCache::new();

    let (quote_executor, swap_preparer, _unused_defaults) = prepare_swap_components(
        config,
        backend,
        &mut identity,
        &compute_unit_price_mode,
        alt_cache.clone(),
    )
    .await?;

    let trade_pairs = build_pure_trade_pairs(pure_config)?;
    let trade_profiles = build_pure_trade_profiles(pure_config)?;
    let profit_config = build_pure_profit_config(&config.lander.lander, &compute_unit_price_mode);
    let quote_config = build_pure_quote_config();
    let landing_timeout = resolve_landing_timeout(&config.galileo.engine.time_out);
    let ip_allocator = build_ip_allocator(&config.galileo.bot.network)?;

    let enable_yellowstone = !dry_run_enabled && config.galileo.bot.get_block_hash_by_grpc;
    let builder_config =
        BuilderConfig::new(resolve_instruction_memo(&config.galileo.global.instruction))
            .with_yellowstone(
                config.galileo.global.yellowstone_grpc_url.clone(),
                config.galileo.global.yellowstone_grpc_token.clone(),
                enable_yellowstone,
            );
    let global_proxy = if dry_run_enabled {
        None
    } else {
        resolve_global_http_proxy(&config.galileo.global)
    };
    let rpc_client_pool = build_rpc_client_pool(rpc_endpoints.clone(), global_proxy.clone());

    let lander_proxy = if dry_run_enabled {
        None
    } else {
        resolve_proxy_profile(&config.galileo.global, "lander")
    };
    let effective_lander_proxy =
        override_proxy_selection(None, lander_proxy.clone(), global_proxy.clone());
    let submission_client =
        build_http_client_with_options(effective_lander_proxy.as_ref(), false, None, None)?;
    let submission_client_pool =
        build_http_client_pool(effective_lander_proxy.clone(), false, None);
    let tx_builder = TransactionBuilder::new(
        rpc_client.clone(),
        builder_config,
        Arc::clone(&ip_allocator),
        Some(rpc_client_pool),
        alt_cache.clone(),
        dry_run_enabled,
    );

    let marginfi_cfg = &config.galileo.flashloan.marginfi;
    let marginfi_accounts = parse_marginfi_accounts(marginfi_cfg)?;
    let flashloan_enabled = config
        .galileo
        .bot
        .flashloan_enabled(FlashloanProduct::Marginfi);
    let prefer_wallet_balance = config.galileo.bot.flashloan.prefer_wallet_balance;

    let prechecker = AccountPrechecker::new(rpc_client.clone(), marginfi_accounts.clone());
    let (summary, flashloan_precheck) = prechecker
        .ensure_accounts(&identity, &trade_pairs, flashloan_enabled)
        .await
        .map_err(|err| anyhow!(err))?;
    let skipped = summary.total_mints.saturating_sub(summary.processed_mints);
    events::accounts_precheck(
        "pure_blind",
        summary.total_mints,
        summary.created_accounts,
        skipped,
    );

    if identity.skip_user_accounts_rpc_calls() {
        info!(
            target: "strategy",
            "skip_user_accounts_rpc_calls 仅作用于 swap-instructions 请求，账户预检查仍已执行"
        );
    }

    if let Some(prep) = &flashloan_precheck {
        events::flashloan_account_precheck("pure_blind", &prep.account, prep.created);
    }
    let mut recorded_accounts: BTreeSet<Pubkey> = BTreeSet::new();
    if let Some(prep) = &flashloan_precheck {
        recorded_accounts.insert(prep.account);
    }
    if flashloan_precheck.is_none() {
        if let Some(account) = marginfi_accounts.default() {
            if recorded_accounts.insert(account) {
                events::flashloan_account_precheck("pure_blind", &account, false);
            }
        }
    }

    let mut flashloan_manager = MarginfiFlashloanManager::new(
        marginfi_cfg,
        flashloan_enabled,
        prefer_wallet_balance,
        rpc_client.clone(),
        marginfi_accounts.clone(),
    );
    let mut flashloan_precheck = flashloan_precheck;
    if let Some(prep) = flashloan_precheck.clone() {
        flashloan_manager.adopt_preparation(prep);
    } else if flashloan_manager.is_enabled() {
        flashloan_precheck = flashloan_manager
            .prepare(&identity)
            .await
            .map_err(|err| anyhow!(err))?;
        if let Some(prep) = &flashloan_precheck {
            events::flashloan_account_precheck("pure_blind", &prep.account, prep.created);
        }
    }
    let flashloan = flashloan_manager.try_into_enabled();

    let lander_factory = LanderFactory::new(
        rpc_client.clone(),
        submission_client.clone(),
        Some(Arc::clone(&submission_client_pool)),
        dry_run_enabled,
        config.galileo.bot.enable_simulation,
    );
    let default_landers = ["rpc"];

    let requested_landers: Vec<String> = if dry_run_enabled {
        if pure_config.enable_landers.is_empty() {
            vec!["rpc".to_string()]
        } else {
            pure_config.enable_landers.clone()
        }
    } else {
        pure_config.enable_landers.clone()
    };

    let lander_stack = lander_factory
        .build_stack(
            &config.lander.lander,
            &requested_landers,
            &default_landers,
            0,
            Arc::clone(&ip_allocator),
        )
        .map_err(|err| anyhow!(err))?;
    let lander_stack = Arc::new(lander_stack);

    let quote_cadence = resolve_quote_cadence(&config.galileo.engine, backend);
    let console_summary_settings = ConsoleSummarySettings {
        enable: config.galileo.engine.enable_console_summary,
    };

    let engine_settings = EngineSettings::new(quote_config)
        .with_quote_cadence(quote_cadence)
        .with_dispatch_strategy(config.lander.lander.sending_strategy)
        .with_landing_timeout(landing_timeout)
        .with_dry_run(dry_run_enabled)
        .with_cu_multiplier(pure_config.cu_multiplier)
        .with_compute_unit_price_mode(compute_unit_price_mode.clone())
        .with_console_summary(console_summary_settings);

    let decay_duration = Duration::from_secs(pure_config.activation.decay_seconds);
    let activation_policy = PoolActivationPolicy::new(
        pure_config.activation.min_hits,
        pure_config.activation.min_estimated_profit,
        decay_duration,
    );
    let observer_queue_capacity = pure_config
        .observer
        .as_ref()
        .map(|cfg| cfg.queue_capacity)
        .unwrap_or(1024);
    let pool_catalog = Arc::new(PoolCatalog::new(
        activation_policy,
        observer_queue_capacity,
        pure_config.cache.max_pools,
    ));
    let route_catalog = Arc::new(RouteCatalog::new(
        RouteActivationPolicy::new(
            pure_config.activation.min_hits,
            pure_config.activation.min_estimated_profit,
            decay_duration,
        ),
        observer_queue_capacity,
        pure_config.cache.max_routes,
    ));

    let cache_manager = PureBlindCacheManager::new(&pure_config.cache);

    if let Some(observer_cfg) = pure_config.observer.as_ref().filter(|cfg| cfg.enable) {
        let endpoint = observer_cfg
            .grpc_endpoint
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow!("pure_blind_strategy.observer.grpc_endpoint 不能为空"))?;

        let token = observer_cfg
            .grpc_token
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .map(|value| value.parse::<AsciiMetadataValue>())
            .transpose()
            .map_err(|err| anyhow!("pure_blind_strategy.observer.grpc_token 解析失败: {err}"))?;

        let mut wallets = Vec::with_capacity(observer_cfg.wallets.len());
        for (idx, wallet) in observer_cfg.wallets.iter().enumerate() {
            let trimmed = wallet.trim();
            if trimmed.is_empty() {
                continue;
            }
            let pubkey = Pubkey::from_str(trimmed).map_err(|err| {
                anyhow!("pure_blind_strategy.observer.wallets[{idx}] `{trimmed}` 解析失败: {err}")
            })?;
            wallets.push(pubkey);
        }

        let settings = PoolObserverSettings {
            endpoint: endpoint.to_string(),
            token,
            wallets,
        };

        if let Err(err) = spawn_pool_observer(
            settings,
            Arc::clone(&pool_catalog),
            Arc::clone(&route_catalog),
        )
        .await
        {
            warn!(
                target: "pure_blind::observer",
                error = %err,
                "启动池子观察器失败"
            );
        }
    }

    let dynamic_rx = spawn_dynamic_worker(
        Arc::clone(&route_catalog),
        Arc::clone(&rpc_client),
        decay_duration,
    );

    if let Err(err) = cache_manager.restore(&pool_catalog, &route_catalog).await {
        warn!(
            target: "pure_blind::cache",
            error = %err,
            "恢复纯盲发缓存快照失败"
        );
    }

    let mut cache_task: Option<JoinHandle<()>> =
        cache_manager.spawn(Arc::clone(&pool_catalog), Arc::clone(&route_catalog));

    let routes = PureBlindRouteBuilder::new(pure_config, rpc_client.as_ref())
        .build()
        .await
        .map_err(|err| anyhow!(err))?;

    let strategy_engine = StrategyEngine::new(
        PureBlindStrategy::new(routes, pure_config, pool_catalog, route_catalog, dynamic_rx)
            .map_err(|err| anyhow!(err))?,
        lander_stack.clone(),
        identity,
        ip_allocator,
        quote_executor,
        ProfitEvaluator::new(profit_config, config.galileo.bot.network.enable_multiple_ip),
        swap_preparer,
        tx_builder,
        Scheduler::new(),
        flashloan,
        engine_settings,
        trade_pairs,
        trade_profiles,
        None,
    );
    let result = drive_engine(strategy_engine).await;

    if let Some(handle) = cache_task.take() {
        handle.abort();
    }

    result.map_err(|err| anyhow!(err))?;

    Ok(())
}

async fn prepare_swap_components(
    config: &AppConfig,
    backend: &StrategyBackend<'_>,
    identity: &mut EngineIdentity,
    compute_unit_price_mode: &Option<ComputeUnitPriceMode>,
    alt_cache: AltCache,
) -> Result<(QuoteExecutor, SwapPreparer, (bool, bool))> {
    match backend {
        StrategyBackend::Jupiter { api_client } => {
            info!(target: "strategy", "使用 Jupiter 聚合器");
            let quote_defaults = config.galileo.engine.jupiter.quote_config.clone();
            let swap_defaults = config.galileo.engine.jupiter.swap_config.clone();
            identity.set_skip_user_accounts_rpc_calls(swap_defaults.skip_user_accounts_rpc_calls);
            let quote_executor =
                QuoteExecutor::for_jupiter((*api_client).clone(), quote_defaults.clone());
            let swap_preparer = SwapPreparer::for_jupiter(
                (*api_client).clone(),
                swap_defaults,
                compute_unit_price_mode.clone(),
            );
            Ok((
                quote_executor,
                swap_preparer,
                (quote_defaults.only_direct_routes, true),
            ))
        }
        StrategyBackend::Dflow { api_client } => {
            info!(target: "strategy", "使用 DFlow 聚合器");
            identity.set_skip_user_accounts_rpc_calls(false);

            let dflow_quote_cfg = config.galileo.engine.dflow.quote_config.clone();
            let quote_executor =
                QuoteExecutor::for_dflow((*api_client).clone(), dflow_quote_cfg.clone());
            let swap_preparer = SwapPreparer::for_dflow(
                (*api_client).clone(),
                config.galileo.engine.dflow.swap_config.clone(),
                compute_unit_price_mode.clone(),
            );
            Ok((
                quote_executor,
                swap_preparer,
                (dflow_quote_cfg.only_direct_routes, true),
            ))
        }
        StrategyBackend::Kamino {
            api_client,
            rpc_client,
        } => {
            info!(target: "strategy", "使用 Kamino 聚合器");
            identity.set_skip_user_accounts_rpc_calls(false);

            let kamino_quote_cfg = config.galileo.engine.kamino.quote_config.clone();
            let quote_executor =
                QuoteExecutor::for_kamino((*api_client).clone(), kamino_quote_cfg.clone());
            let swap_preparer = SwapPreparer::for_kamino(
                rpc_client.clone(),
                kamino_quote_cfg.clone(),
                compute_unit_price_mode.clone(),
                alt_cache.clone(),
            );
            Ok((quote_executor, swap_preparer, (false, false)))
        }
        StrategyBackend::Ultra {
            api_client,
            rpc_client,
        } => {
            identity.set_skip_user_accounts_rpc_calls(false);
            let ultra_quote_cfg = config.galileo.engine.ultra.quote_config.clone();
            let ultra_swap_cfg = config.galileo.engine.ultra.swap_config.clone();
            let quote_executor = QuoteExecutor::for_ultra((*api_client).clone(), ultra_quote_cfg);
            let swap_preparer = SwapPreparer::for_ultra(
                rpc_client.clone(),
                ultra_swap_cfg,
                compute_unit_price_mode.clone(),
                alt_cache.clone(),
            );
            Ok((quote_executor, swap_preparer, (true, true)))
        }
        StrategyBackend::None => {
            identity.set_skip_user_accounts_rpc_calls(false);
            let quote_executor = QuoteExecutor::disabled();
            let swap_preparer = SwapPreparer::disabled();
            Ok((quote_executor, swap_preparer, (true, true)))
        }
    }
}

async fn drive_engine<S>(engine: StrategyEngine<S>) -> EngineResult<()>
where
    S: Strategy<Event = StrategyEvent>,
{
    tokio::select! {
        res = engine.run() => res,
        _ = tokio::signal::ctrl_c() => {
            info!(target: "strategy", "收到终止信号，停止运行");
            Ok(())
        }
    }
}

fn build_blind_trade_pairs(
    config: &config::BlindStrategyConfig,
    intermedium: &IntermediumConfig,
) -> EngineResult<Vec<crate::strategy::types::TradePair>> {
    if config.base_mints.is_empty() {
        return Err(EngineError::InvalidConfig(
            "blind_strategy.base_mints 不能为空".into(),
        ));
    }

    let disabled: std::collections::BTreeSet<String> = intermedium
        .disable_mints
        .iter()
        .map(|mint| mint.trim().to_string())
        .filter(|mint| !mint.is_empty())
        .collect();

    let intermediates: Vec<String> = intermedium
        .mints
        .iter()
        .map(|mint| mint.trim())
        .filter(|mint| !mint.is_empty())
        .filter(|mint| !disabled.contains(&mint.to_string()))
        .map(|mint| mint.to_string())
        .collect();

    if intermediates.is_empty() {
        return Err(EngineError::InvalidConfig(
            "intermedium.mints 不能为空".into(),
        ));
    }

    let mut pairs_set: BTreeSet<(String, String)> = BTreeSet::new();
    for base in &config.base_mints {
        let base_mint = base.mint.trim();
        if base_mint.is_empty() {
            continue;
        }

        for intermediate in &intermediates {
            if intermediate == base_mint {
                continue;
            }
            pairs_set.insert((base_mint.to_string(), intermediate.clone()));
        }
    }

    if pairs_set.is_empty() {
        return Err(EngineError::InvalidConfig(
            "盲发策略未生成任何交易对".into(),
        ));
    }

    pairs_set
        .into_iter()
        .map(|(input_mint, output_mint)| {
            crate::strategy::types::TradePair::try_new(&input_mint, &output_mint).map_err(|err| {
                EngineError::InvalidConfig(format!(
                    "盲发交易对配置无效 ({input_mint} -> {output_mint}): {err}"
                ))
            })
        })
        .collect()
}

fn build_pure_trade_pairs(
    config: &config::PureBlindStrategyConfig,
) -> EngineResult<Vec<crate::strategy::types::TradePair>> {
    if config.assets.base_mints.is_empty() {
        return Err(EngineError::InvalidConfig(
            "pure_blind_strategy.assets.base_mints 不能为空".into(),
        ));
    }

    let blacklist: BTreeSet<String> = config
        .assets
        .blacklist_mints
        .iter()
        .map(|mint| mint.trim().to_string())
        .filter(|mint| !mint.is_empty())
        .collect();

    let intermediates: Vec<String> = config
        .assets
        .intermediates
        .iter()
        .map(|mint| mint.trim())
        .filter(|mint| !mint.is_empty())
        .filter(|mint| !blacklist.contains(&mint.to_string()))
        .map(|mint| mint.to_string())
        .collect();

    if intermediates.is_empty() {
        return Err(EngineError::InvalidConfig(
            "pure_blind_strategy.assets.intermediates 不能为空".into(),
        ));
    }

    let mut pairs_set: BTreeSet<(String, String)> = BTreeSet::new();
    for base in &config.assets.base_mints {
        let base_mint = base.mint.trim();
        if base_mint.is_empty() {
            continue;
        }
        if blacklist.contains(base_mint) {
            continue;
        }
        for intermediate in &intermediates {
            if intermediate == base_mint {
                continue;
            }
            if blacklist.contains(intermediate) {
                continue;
            }
            pairs_set.insert((base_mint.to_string(), intermediate.clone()));
        }
    }

    if pairs_set.is_empty() {
        return Err(EngineError::InvalidConfig(
            "纯盲发策略未生成任何交易对".into(),
        ));
    }

    pairs_set
        .into_iter()
        .map(|(input_mint, output_mint)| {
            crate::strategy::types::TradePair::try_new(&input_mint, &output_mint).map_err(|err| {
                EngineError::InvalidConfig(format!(
                    "纯盲发交易对配置无效 ({input_mint} -> {output_mint}): {err}"
                ))
            })
        })
        .collect()
}

fn build_blind_trade_profiles(
    config: &config::BlindStrategyConfig,
    ip_capacity_hint: usize,
) -> EngineResult<BTreeMap<Pubkey, TradeProfile>> {
    #[derive(Clone)]
    struct BaseState {
        mint: Pubkey,
        lane_indices: Vec<usize>,
    }

    #[derive(Clone)]
    struct LanePlan {
        min: u64,
        max: u64,
        strategy: config::TradeRangeStrategy,
        weight: f64,
        final_count: usize,
        max_count: usize,
        allow_scale: bool,
    }

    let mut base_states: Vec<BaseState> = Vec::new();
    let mut lane_plans: Vec<LanePlan> = Vec::new();

    for base in &config.base_mints {
        let mint_str = base.mint.trim();
        if mint_str.is_empty() {
            continue;
        }
        let mint = Pubkey::from_str(mint_str).map_err(|err| {
            EngineError::InvalidConfig(format!(
                "blind_strategy.base_mints 中的 mint `{mint_str}` 解析失败: {err}"
            ))
        })?;

        if base.lanes.is_empty() {
            return Err(EngineError::InvalidConfig(format!(
                "盲发策略中 mint `{mint_str}` 未配置任何 lane"
            )));
        }

        let mut lane_indices = Vec::with_capacity(base.lanes.len());

        for lane in &base.lanes {
            if lane.count == 0 {
                return Err(EngineError::InvalidConfig(format!(
                    "盲发策略中 mint `{mint_str}` 的 lane `count` 必须大于 0"
                )));
            }

            let min = lane.min;
            let max = lane.max.max(min);
            if min == 0 && max == 0 {
                return Err(EngineError::InvalidConfig(format!(
                    "盲发策略中 mint `{mint_str}` 的 lane min/max 不能同时为 0"
                )));
            }

            let base_count = lane.count as usize;
            let allow_scale = min < max;
            let max_multiplier = config.auto_scale_to_ip.max_multiplier.max(1.0);
            let mut max_count = base_count;
            if config.auto_scale_to_ip.enable && allow_scale {
                let scaled = ((base_count as f64) * max_multiplier).ceil() as usize;
                max_count = max_count.max(scaled);
            }
            if !allow_scale {
                max_count = base_count;
            }

            let weight = if lane.weight.is_sign_negative() {
                0.0
            } else {
                lane.weight
            };

            let lane_index = lane_plans.len();
            lane_indices.push(lane_index);
            lane_plans.push(LanePlan {
                min,
                max,
                strategy: lane.strategy,
                weight,
                final_count: base_count,
                max_count,
                allow_scale,
            });
        }

        base_states.push(BaseState { mint, lane_indices });
    }

    if base_states.is_empty() {
        return Err(EngineError::InvalidConfig(
            "盲发策略未配置有效的 base mint".into(),
        ));
    }

    let mut total_count: usize = lane_plans.iter().map(|lane| lane.final_count).sum();
    if total_count == 0 {
        return Err(EngineError::InvalidConfig(
            "盲发策略未配置有效的交易规模".into(),
        ));
    }

    if config.auto_scale_to_ip.enable && ip_capacity_hint > total_count {
        let max_possible: usize = lane_plans.iter().map(|lane| lane.max_count).sum();
        let target = ip_capacity_hint.min(max_possible).max(total_count);
        if target > total_count {
            let mut remaining = target - total_count;

            let mut weighted_total: f64 = lane_plans
                .iter()
                .filter(|lane| {
                    lane.allow_scale && lane.final_count < lane.max_count && lane.weight > 0.0
                })
                .map(|lane| lane.weight)
                .sum();

            if weighted_total == 0.0 {
                weighted_total = lane_plans
                    .iter()
                    .filter(|lane| lane.allow_scale && lane.final_count < lane.max_count)
                    .count() as f64;
            }

            if weighted_total > 0.0 {
                for lane in lane_plans.iter_mut() {
                    if remaining == 0 {
                        break;
                    }
                    if !lane.allow_scale || lane.final_count >= lane.max_count {
                        continue;
                    }
                    let lane_weight = if lane.weight > 0.0 && weighted_total > 0.0 {
                        lane.weight / weighted_total
                    } else if weighted_total > 0.0 {
                        1.0 / weighted_total
                    } else {
                        0.0
                    };
                    if lane_weight <= 0.0 {
                        continue;
                    }
                    let extra = ((remaining as f64) * lane_weight).floor() as usize;
                    let extra = extra.min(lane.max_count - lane.final_count);
                    if extra > 0 {
                        lane.final_count += extra;
                        total_count += extra;
                        remaining = target.saturating_sub(total_count);
                        if remaining == 0 {
                            break;
                        }
                    }
                }
            }

            while remaining > 0 {
                let mut progressed = false;
                for lane in lane_plans.iter_mut() {
                    if remaining == 0 {
                        break;
                    }
                    if !lane.allow_scale || lane.final_count >= lane.max_count {
                        continue;
                    }
                    lane.final_count += 1;
                    total_count += 1;
                    remaining -= 1;
                    progressed = true;
                }
                if !progressed {
                    break;
                }
            }
        }
    }

    let mut per_mint: BTreeMap<Pubkey, TradeProfile> = BTreeMap::new();

    for base_state in base_states {
        let mut lane_values: BTreeSet<u64> = BTreeSet::new();
        for lane_index in base_state.lane_indices {
            let lane = &lane_plans[lane_index];
            let values = generate_lane_amounts(lane.min, lane.max, lane.final_count, lane.strategy);
            for value in values {
                if value > 0 {
                    lane_values.insert(value);
                }
            }
        }

        if lane_values.is_empty() {
            continue;
        }

        let mut rng = rand::rng();
        let mut tweaked: BTreeSet<u64> = BTreeSet::new();
        for amount in lane_values {
            let basis_points: u16 = rng.random_range(930..=999);
            let adjusted = (((amount as u128) * basis_points as u128) + 999) / 1_000;
            let normalized = adjusted.max(1).min(u128::from(u64::MAX)) as u64;
            tweaked.insert(normalized);
        }

        let amounts: Vec<u64> = tweaked.into_iter().collect();
        if amounts.is_empty() {
            continue;
        }

        if per_mint
            .insert(base_state.mint, TradeProfile { amounts })
            .is_some()
        {
            return Err(EngineError::InvalidConfig(format!(
                "盲发策略中存在重复的 base mint `{}`",
                base_state.mint
            )));
        }
    }

    if per_mint.is_empty() {
        return Err(EngineError::InvalidConfig(
            "盲发策略未配置有效的交易规模".into(),
        ));
    }

    Ok(per_mint)
}

fn build_pure_trade_profiles(
    config: &config::PureBlindStrategyConfig,
) -> EngineResult<BTreeMap<Pubkey, TradeProfile>> {
    let blacklist: BTreeSet<String> = config
        .assets
        .blacklist_mints
        .iter()
        .map(|mint| mint.trim().to_string())
        .filter(|mint| !mint.is_empty())
        .collect();

    let mut per_mint: BTreeMap<Pubkey, TradeProfile> = BTreeMap::new();

    for base in &config.assets.base_mints {
        let mint_str = base.mint.trim();
        if mint_str.is_empty() || blacklist.contains(mint_str) {
            continue;
        }
        let mint = Pubkey::from_str(mint_str).map_err(|err| {
            EngineError::InvalidConfig(format!(
                "pure_blind_strategy.assets.base_mints 中的 mint `{mint_str}` 解析失败: {err}"
            ))
        })?;

        let amounts = normalize_pure_lane_amounts(&base.lanes);
        if amounts.is_empty() {
            continue;
        }

        per_mint.insert(mint, TradeProfile { amounts });
    }

    if per_mint.is_empty() {
        return Err(EngineError::InvalidConfig(
            "纯盲发策略未配置有效的交易规模".into(),
        ));
    }

    Ok(per_mint)
}

fn generate_lane_amounts(
    min: u64,
    max: u64,
    count: usize,
    strategy: config::TradeRangeStrategy,
) -> Vec<u64> {
    if count == 0 {
        return Vec::new();
    }
    if min == max {
        return vec![min];
    }
    let capped = count.min(u32::MAX as usize) as u32;
    match strategy {
        config::TradeRangeStrategy::Linear => generate_amounts(min, max, capped, "linear"),
        config::TradeRangeStrategy::Exponential => {
            generate_amounts(min, max, capped, "exponential")
        }
        config::TradeRangeStrategy::Random => generate_random_amounts(min, max, count),
    }
}

fn generate_random_amounts(min: u64, max: u64, count: usize) -> Vec<u64> {
    if min == max {
        return vec![min];
    }
    let mut rng = rand::rng();
    let mut values: BTreeSet<u64> = BTreeSet::new();
    let mut attempts = 0usize;
    let max_attempts = count.saturating_mul(10).max(100);

    while values.len() < count && attempts < max_attempts {
        let sample = rng.random_range(min..=max);
        values.insert(sample);
        attempts += 1;
    }

    if values.len() < count {
        let fallback = generate_amounts(min, max, count.min(u32::MAX as usize) as u32, "linear");
        return fallback;
    }

    values.into_iter().collect()
}

fn normalize_pure_lane_amounts(lanes: &[config::TradeSizeLaneConfig]) -> Vec<u64> {
    if lanes.is_empty() {
        return Vec::new();
    }

    let mut lane_values: BTreeSet<u64> = BTreeSet::new();
    for lane in lanes {
        let count = lane.count.max(1) as usize;
        let values = generate_lane_amounts(lane.min, lane.max, count, lane.strategy);
        for value in values {
            if value > 0 {
                lane_values.insert(value);
            }
        }
    }

    if lane_values.is_empty() {
        return Vec::new();
    }

    let mut rng = rand::rng();
    let mut tweaked: BTreeSet<u64> = BTreeSet::new();
    for amount in lane_values {
        let basis_points: u16 = rng.random_range(930..=999);
        let adjusted = (((amount as u128) * basis_points as u128) + 999) / 1_000;
        let normalized = adjusted.max(1).min(u128::from(u64::MAX)) as u64;
        tweaked.insert(normalized);
    }

    tweaked.into_iter().collect()
}

fn build_blind_profit_config(
    config: &config::BlindStrategyConfig,
    _lander_settings: &config::LanderSettings,
    _compute_unit_price_mode: &Option<ComputeUnitPriceMode>,
) -> ProfitConfig {
    let threshold = config
        .base_mints
        .iter()
        .filter_map(|mint| mint.min_quote_profit)
        .min()
        .unwrap_or(0);

    ProfitConfig {
        min_profit_threshold_lamports: threshold,
        max_tip_lamports: 0,
        tip: TipConfig::default(),
    }
}

fn build_pure_profit_config(
    _lander_settings: &config::LanderSettings,
    _compute_unit_price_mode: &Option<ComputeUnitPriceMode>,
) -> ProfitConfig {
    ProfitConfig {
        min_profit_threshold_lamports: 0,
        max_tip_lamports: 0,
        tip: TipConfig::default(),
    }
}

fn build_blind_quote_config(
    config: &config::BlindStrategyConfig,
    only_direct_routes_default: bool,
) -> QuoteConfig {
    QuoteConfig {
        slippage_bps: 0,
        only_direct_routes: only_direct_routes_default,
        dex_whitelist: config.enable_dexs.clone(),
        dex_blacklist: config.exclude_dexes.clone(),
    }
}

fn build_pure_quote_config() -> QuoteConfig {
    QuoteConfig {
        slippage_bps: 0,
        only_direct_routes: true,
        dex_whitelist: Vec::new(),
        dex_blacklist: Vec::new(),
    }
}

fn generate_amounts(min: u64, max: u64, count: u32, strategy: &str) -> Vec<u64> {
    if count <= 1 || min == max {
        return vec![min.max(max)];
    }

    let steps = count as usize - 1;
    match strategy {
        "exponential" if min > 0 && max > min => {
            let ratio = (max as f64 / min as f64).max(1.0);
            (0..=steps)
                .map(|i| {
                    let exponent = i as f64 / steps as f64;
                    let value = (min as f64 * ratio.powf(exponent)).round() as u64;
                    value.clamp(min, max)
                })
                .collect()
        }
        _ => {
            let range = max.saturating_sub(min);
            (0..=steps)
                .map(|i| {
                    let fraction = i as f64 / steps as f64;
                    let value = min as f64 + fraction * range as f64;
                    value.round() as u64
                })
                .collect()
        }
    }
}

fn resolve_landing_timeout(timeouts: &config::EngineTimeoutConfig) -> Duration {
    let ms = timeouts.landing_ms.max(1);
    Duration::from_millis(ms)
}

fn derive_compute_unit_price_mode(
    settings: &config::LanderSettings,
) -> Option<ComputeUnitPriceMode> {
    let strategy = settings
        .compute_unit_price_strategy
        .trim()
        .to_ascii_lowercase();
    match strategy.as_str() {
        "" | "none" => None,
        "fixed" => match settings.fixed_compute_unit_price {
            Some(value) => Some(ComputeUnitPriceMode::Fixed(value)),
            None => {
                warn!(
                    target: "strategy",
                    "固定 compute unit price 策略需要提供 fixed_compute_unit_price，已忽略配置"
                );
                None
            }
        },
        "random" => {
            let range = &settings.random_compute_unit_price_range;
            if range.len() >= 2 {
                Some(ComputeUnitPriceMode::Random {
                    min: range[0],
                    max: range[1],
                })
            } else {
                warn!(
                    target: "strategy",
                    "随机 compute unit price 需要提供上下限，已忽略配置"
                );
                None
            }
        }
        other => {
            warn!(
                target: "strategy",
                strategy = other,
                "未知的 compute_unit_price_strategy，已忽略配置"
            );
            None
        }
    }
}

fn parse_marginfi_accounts(
    cfg: &config::FlashloanMarginfiConfig,
) -> Result<MarginfiAccountRegistry> {
    let default = match cfg.marginfi_account.as_deref().map(str::trim) {
        Some("") | None => None,
        Some(value) => Some(
            Pubkey::from_str(value)
                .map_err(|err| anyhow!("flashloan.marginfi.marginfi_account 无效: {err}"))?,
        ),
    };

    Ok(MarginfiAccountRegistry::new(default))
}
