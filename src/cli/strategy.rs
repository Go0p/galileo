use std::collections::{BTreeMap, BTreeSet};
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Result, anyhow};
use tracing::{info, warn};

use crate::api::dflow::DflowApiClient;
use crate::api::jupiter::JupiterApiClient;
use crate::api::kamino::KaminoApiClient;
use crate::api::titan::TitanSubscriptionConfig;
use crate::api::ultra::UltraApiClient;
use crate::cli::context::{
    RpcEndpointRotator, resolve_global_http_proxy, resolve_instruction_memo, resolve_rpc_client,
};
use crate::config;
use crate::config::{AppConfig, IntermediumConfig, LegRole, QuoteParallelism};
use crate::copy_strategy;
use crate::engine::{
    AccountPrechecker, BuilderConfig, ComputeUnitPriceMode, EngineError, EngineIdentity,
    EngineResult, EngineSettings, FALLBACK_CU_LIMIT, MultiLegEngineContext, ProfitConfig,
    ProfitEvaluator, QuoteConfig, QuoteExecutor, Scheduler, StrategyEngine, SwapPreparer,
    TipConfig, TradeProfile, TransactionBuilder,
};
use crate::flashloan::marginfi::{MarginfiAccountRegistry, MarginfiFlashloanManager};
use crate::jupiter::{JupiterBinaryManager, JupiterError};
use crate::lander::LanderFactory;
use crate::monitoring::events;
use crate::multi_leg::{
    alt_cache::AltCache,
    orchestrator::MultiLegOrchestrator,
    providers::{
        dflow::DflowLegProvider,
        titan::{TitanLegProvider, TitanWsQuoteSource},
        ultra::UltraLegProvider,
    },
    runtime::{MultiLegRuntime, MultiLegRuntimeConfig},
    types::{AggregatorKind as MultiLegAggregatorKind, LegSide},
};
use crate::network::{
    CooldownConfig, IpAllocator, IpBoundClientPool, IpInventory, IpInventoryConfig, NetworkError,
    ReqwestClientFactoryFn, RpcClientFactoryFn,
};
use crate::pure_blind::market_cache::init_market_cache;
use crate::strategy::{
    BlindStrategy, PureBlindRouteBuilder, PureBlindStrategy, Strategy, StrategyEvent,
};
use dashmap::DashMap;
use rand::Rng as _;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_client::RpcClientConfig;
use solana_rpc_client::http_sender::HttpSender;
use solana_sdk::pubkey::Pubkey;
use url::Url;

/// 控制策略以正式模式还是 dry-run 模式运行。
pub enum StrategyMode {
    Live,
    DryRun,
}

pub enum StrategyBackend<'a> {
    Jupiter {
        manager: &'a JupiterBinaryManager,
        api_client: &'a JupiterApiClient,
    },
    Dflow {
        api_client: &'a DflowApiClient,
    },
    Kamino {
        api_client: &'a KaminoApiClient,
    },
    Ultra {
        api_client: &'a UltraApiClient,
        rpc_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    },
    None,
}

const DEFAULT_PROCESS_DELAY_MS: u64 = 200;

/// 主策略入口，按配置在盲发与 copy 之间切换。
pub async fn run_strategy(
    config: &AppConfig,
    backend: &StrategyBackend<'_>,
    mode: StrategyMode,
) -> Result<()> {
    let dry_run = matches!(mode, StrategyMode::DryRun) || config.galileo.bot.dry_run;

    if config.galileo.copy_strategy.enable {
        if config.galileo.blind_strategy.enable || config.galileo.pure_blind_strategy.enable {
            warn!(
                target: "strategy",
                "copy_strategy 启用，将忽略 blind/pure 配置"
            );
        }
        return copy_strategy::run_copy_strategy(config, backend, mode).await;
    }

    match config.galileo.engine.backend {
        crate::config::EngineBackend::MultiLegs => {
            if !config.galileo.blind_strategy.enable {
                return Err(anyhow!(
                    "multi-legs 模式暂需 blind_strategy.enable = true 以提供 trade pair 配置"
                ));
            }
            run_blind_engine(config, backend, dry_run, true).await
        }
        crate::config::EngineBackend::None => {
            if config.galileo.blind_strategy.enable {
                return Err(anyhow!(
                    "engine.backend=none 仅支持纯盲发策略，请关闭 blind_strategy.enable"
                ));
            }
            let pure_config = &config.galileo.pure_blind_strategy;
            if !pure_config.enable {
                warn!(target: "strategy", "纯盲发策略未启用，直接退出");
                return Ok(());
            }
            run_pure_blind_engine(config, backend, dry_run).await
        }
        _ => {
            let blind_config = &config.galileo.blind_strategy;
            let pure_config = &config.galileo.pure_blind_strategy;

            match (blind_config.enable, pure_config.enable) {
                (false, false) => {
                    warn!(target: "strategy", "盲发策略未启用，直接退出");
                    Ok(())
                }
                (true, true) => Err(anyhow!(
                    "blind_strategy.enable 与 pure_blind_strategy.enable 不能同时为 true"
                )),
                (true, false) => run_blind_engine(config, backend, dry_run, false).await,
                (false, true) => run_pure_blind_engine(config, backend, dry_run).await,
            }
        }
    }
}

async fn run_blind_engine(
    config: &AppConfig,
    backend: &StrategyBackend<'_>,
    dry_run: bool,
    multi_leg_mode: bool,
) -> Result<()> {
    let blind_config = &config.galileo.blind_strategy;

    if !blind_config.enable {
        warn!(target: "strategy", "盲发策略未启用，直接退出");
        return Ok(());
    }

    let compute_unit_price_mode = derive_compute_unit_price_mode(&config.lander.lander);

    let resolved_rpc = resolve_rpc_client(&config.galileo.global)?;
    let rpc_client = resolved_rpc.client.clone();
    let rpc_endpoints = resolved_rpc.endpoints.clone();
    let mut identity =
        EngineIdentity::from_wallet(&config.galileo.global.wallet).map_err(|err| anyhow!(err))?;

    let ip_allocator = build_ip_allocator(&config.galileo.bot.network)?;

    let multi_leg_runtime = if multi_leg_mode {
        let runtime = Arc::new(build_multi_leg_runtime(
            config,
            &identity,
            rpc_client.clone(),
            Arc::clone(&ip_allocator),
        )?);
        log_multi_leg_init(runtime.as_ref(), dry_run);
        Some(runtime)
    } else {
        None
    };

    let (quote_executor, swap_preparer, quote_defaults_tuple, jupiter_started) =
        prepare_swap_components(
            config,
            backend,
            &mut identity,
            &compute_unit_price_mode,
            true,
        )
        .await?;

    let (only_direct_default, restrict_default) = quote_defaults_tuple;
    let trade_pairs = build_blind_trade_pairs(blind_config, &config.galileo.intermedium)?;
    let trade_profiles = build_blind_trade_profiles(blind_config)?;
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
        ctx
    });
    let profit_config = build_blind_profit_config(
        blind_config,
        &config.lander.lander,
        &compute_unit_price_mode,
    );
    let quote_config =
        build_blind_quote_config(blind_config, only_direct_default, restrict_default);
    tracing::info!(
        target: "engine::config",
        dex_whitelist = ?quote_config.dex_whitelist,
        "盲发策略 DEX 白名单"
    );
    let landing_timeout = resolve_landing_timeout(&config.galileo.bot);

    let builder_config =
        BuilderConfig::new(resolve_instruction_memo(&config.galileo.global.instruction))
            .with_yellowstone(
                config.galileo.global.yellowstone_grpc_url.clone(),
                config.galileo.global.yellowstone_grpc_token.clone(),
                config.galileo.bot.get_block_hash_by_grpc,
            );
    let global_proxy = resolve_global_http_proxy(&config.galileo.global);
    let rpc_client_pool = build_rpc_client_pool(rpc_endpoints.clone(), global_proxy.clone());
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

    let marginfi_cfg = &config.galileo.flashloan.marginfi;
    let marginfi_accounts = parse_marginfi_accounts(marginfi_cfg)?;

    let prechecker = AccountPrechecker::new(rpc_client.clone(), marginfi_accounts.clone());
    let (summary, flashloan_precheck) = prechecker
        .ensure_accounts(
            &identity,
            &trade_pairs,
            config.galileo.flashloan.marginfi.enable,
        )
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

    let mut flashloan_manager =
        MarginfiFlashloanManager::new(marginfi_cfg, rpc_client.clone(), marginfi_accounts.clone());
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
    );
    let default_landers = ["rpc"];

    let lander_stack = Arc::new(
        lander_factory
            .build_stack(
                &config.lander.lander,
                &blind_config.enable_landers,
                &default_landers,
                0,
                Arc::clone(&ip_allocator),
            )
            .map_err(|err| anyhow!(err))?,
    );

    let (quote_parallelism, quote_batch_interval) =
        resolve_quote_dispatch_config(&config.galileo.engine, backend);

    let engine_settings = EngineSettings::new(quote_config)
        .with_quote_parallelism(quote_parallelism)
        .with_quote_batch_interval(quote_batch_interval)
        .with_dispatch_strategy(config.lander.lander.sending_strategy)
        .with_landing_timeout(landing_timeout)
        .with_dry_run(dry_run)
        .with_cu_multiplier(1.0)
        .with_compute_unit_price_mode(compute_unit_price_mode.clone());

    let strategy_engine = StrategyEngine::new(
        BlindStrategy::new(),
        lander_stack.clone(),
        identity,
        ip_allocator,
        quote_executor,
        ProfitEvaluator::new(profit_config),
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

    if jupiter_started {
        if let StrategyBackend::Jupiter { manager, .. } = backend {
            if let Err(err) = manager.stop().await {
                warn!(
                    target: "strategy",
                    error = %err,
                    "停止 Jupiter 二进制失败"
                );
            }
        }
    }

    Ok(())
}

fn build_multi_leg_runtime(
    config: &AppConfig,
    identity: &EngineIdentity,
    rpc_client: Arc<solana_client::nonblocking::rpc_client::RpcClient>,
    ip_allocator: Arc<IpAllocator>,
) -> Result<MultiLegRuntime> {
    let mut orchestrator = MultiLegOrchestrator::new();

    register_ultra_leg(&mut orchestrator, config, identity)?;
    register_dflow_leg(&mut orchestrator, config)?;
    register_titan_leg(&mut orchestrator, config)?;

    if orchestrator.buy_legs().is_empty() {
        return Err(anyhow!("multi-legs 模式至少需要一个 buy 腿"));
    }
    if orchestrator.sell_legs().is_empty() {
        return Err(anyhow!("multi-legs 模式至少需要一个 sell 腿"));
    }

    let alt_cache = AltCache::new();
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
    info!(
        target: "multi_leg::init",
        buy_legs = buy.len(),
        sell_legs = sell.len(),
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
}

fn register_ultra_leg(
    orchestrator: &mut MultiLegOrchestrator,
    config: &AppConfig,
    identity: &EngineIdentity,
) -> Result<()> {
    let ultra_cfg = &config.galileo.engine.ultra;
    if !ultra_cfg.enable {
        return Ok(());
    }

    let leg = ultra_cfg
        .leg
        .ok_or_else(|| anyhow!("ultra.leg 必须在 multi-legs 模式下配置"))?;
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
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let global_proxy = resolve_global_http_proxy(&config.galileo.global)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let http_client = build_http_client_with_options(
        proxy_override.as_deref(),
        global_proxy.clone(),
        false,
        None,
        None,
    )?;
    let http_pool =
        build_http_client_pool(proxy_override.clone(), global_proxy.clone(), false, None);

    let ultra_client = UltraApiClient::with_ip_pool(
        http_client,
        api_base,
        &config.galileo.bot,
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
        ultra_cfg.swap_config.clone(),
        identity.pubkey,
        request_taker_override,
    );
    orchestrator.register_owned_provider(provider);
    Ok(())
}

fn register_dflow_leg(orchestrator: &mut MultiLegOrchestrator, config: &AppConfig) -> Result<()> {
    let dflow_cfg = &config.galileo.engine.dflow;
    if !dflow_cfg.enable {
        return Ok(());
    }

    let leg = dflow_cfg
        .leg
        .ok_or_else(|| anyhow!("dflow.leg 必须在 multi-legs 模式下配置"))?;
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
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let global_proxy = resolve_global_http_proxy(&config.galileo.global)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let http_client = build_http_client_with_options(
        proxy_override.as_deref(),
        global_proxy.clone(),
        false,
        None,
        None,
    )?;
    let http_pool =
        build_http_client_pool(proxy_override.clone(), global_proxy.clone(), false, None);

    let dflow_client = DflowApiClient::with_ip_pool(
        http_client,
        quote_base,
        swap_base,
        &config.galileo.bot,
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

fn register_titan_leg(orchestrator: &mut MultiLegOrchestrator, config: &AppConfig) -> Result<()> {
    let titan_cfg = &config.galileo.engine.titan;
    if !titan_cfg.enable {
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

    let default_pubkey = titan_cfg
        .default_pubkey
        .as_ref()
        .ok_or_else(|| anyhow!("titan.default_pubkey 未配置"))?
        .trim();
    let default_pubkey = solana_sdk::pubkey::Pubkey::from_str(default_pubkey)
        .map_err(|err| anyhow!("titan.default_pubkey 无效: {err}"))?;

    let subscription_cfg = TitanSubscriptionConfig {
        ws_url,
        ws_proxy,
        jwt,
        default_pubkey,
        providers: titan_cfg.providers.clone(),
        reverse_slippage_bps: titan_cfg.reverse_slippage_bps,
        update_interval_ms: titan_cfg.interval_ms,
        update_num_quotes: titan_cfg.num_quotes,
    };

    let quote_source = TitanWsQuoteSource::new(subscription_cfg);
    let provider = TitanLegProvider::new(quote_source, LegSide::Buy);
    orchestrator.register_owned_provider(provider);
    Ok(())
}

pub(crate) fn build_http_client_with_options(
    proxy: Option<&str>,
    global_proxy: Option<String>,
    no_proxy: bool,
    local_ip: Option<IpAddr>,
    user_agent: Option<&str>,
) -> Result<reqwest::Client> {
    let mut builder = reqwest::Client::builder();

    if let Some(agent) = user_agent {
        builder = builder.user_agent(agent);
    }

    if let Some(ip) = local_ip {
        builder = builder.local_address(ip);
    }

    if no_proxy {
        builder = builder.no_proxy();
    } else if let Some(proxy_url) = proxy.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }) {
        let proxy = reqwest::Proxy::all(&proxy_url)
            .map_err(|err| anyhow!("HTTP 代理地址无效 {proxy_url}: {err}"))?;
        builder = builder.proxy(proxy).danger_accept_invalid_certs(true);
    } else if let Some(proxy_url) = global_proxy {
        let trimmed = proxy_url.trim().to_string();
        if !trimmed.is_empty() {
            let proxy = reqwest::Proxy::all(&trimmed)
                .map_err(|err| anyhow!("global.proxy 地址无效 {trimmed}: {err}"))?;
            builder = builder.proxy(proxy).danger_accept_invalid_certs(true);
        }
    }

    builder
        .build()
        .map_err(|err| anyhow!("构建 HTTP 客户端失败: {err}"))
}

#[allow(dead_code)]
pub(crate) fn build_http_client(
    proxy: Option<&str>,
    global_proxy: Option<String>,
) -> Result<reqwest::Client> {
    build_http_client_with_options(proxy, global_proxy, false, None, None)
}

pub(crate) fn build_http_client_pool(
    proxy: Option<String>,
    global_proxy: Option<String>,
    no_proxy: bool,
    user_agent: Option<String>,
) -> Arc<IpBoundClientPool<ReqwestClientFactoryFn>> {
    let proxy_clone = proxy.clone();
    let global_clone = global_proxy.clone();
    let agent_clone = user_agent.clone();
    let factory: ReqwestClientFactoryFn = Box::new(move |ip: IpAddr| {
        build_http_client_with_options(
            proxy_clone.as_deref(),
            global_clone.clone(),
            no_proxy,
            Some(ip),
            agent_clone.as_deref(),
        )
        .map_err(|err| NetworkError::ClientPool(err.to_string()))
    });
    Arc::new(IpBoundClientPool::new(factory))
}

pub(crate) fn build_rpc_client_pool(
    endpoints: Arc<RpcEndpointRotator>,
    global_proxy: Option<String>,
) -> Arc<IpBoundClientPool<RpcClientFactoryFn>> {
    let cache: Arc<DashMap<(IpAddr, usize), Arc<RpcClient>>> = Arc::new(DashMap::new());
    let proxy = Arc::new(global_proxy);
    let rotator = endpoints;

    let factory: RpcClientFactoryFn = Box::new(move |ip: IpAddr| {
        let (index, endpoint) = rotator.next();
        let key = (ip, index);
        if let Some(existing) = cache.get(&key) {
            return Ok(existing.clone());
        }

        let mut builder = reqwest::Client::builder()
            .default_headers(HttpSender::default_headers())
            .timeout(Duration::from_secs(30))
            .pool_idle_timeout(Duration::from_secs(30))
            .local_address(ip);

        if let Some(proxy_url) = proxy.as_ref() {
            let trimmed = proxy_url.trim();
            if trimmed.is_empty() {
                builder = builder.no_proxy();
            } else {
                let proxy = reqwest::Proxy::all(trimmed).map_err(|err| {
                    NetworkError::ClientPool(format!("global.proxy 地址无效 {trimmed}: {err}"))
                })?;
                builder = builder.proxy(proxy).danger_accept_invalid_certs(true);
            }
        } else {
            builder = builder.no_proxy();
        }

        let client = builder.build().map_err(|err| {
            NetworkError::ClientPool(format!("构建绑定 IP 的 RPC 客户端失败: {err}"))
        })?;

        let rpc_url = endpoint.to_string();
        let sender = HttpSender::new_with_client(rpc_url.clone(), client);
        let rpc = Arc::new(RpcClient::new_sender(
            sender,
            RpcClientConfig::with_commitment(solana_commitment_config::CommitmentConfig::default()),
        ));

        if let Some(previous) = cache.insert(key, rpc.clone()) {
            return Ok(previous);
        }

        Ok(rpc)
    });

    Arc::new(IpBoundClientPool::new(factory))
}

fn resolve_quote_dispatch_config(
    engine: &config::EngineConfig,
    backend: &StrategyBackend<'_>,
) -> (Option<u16>, Duration) {
    match backend {
        StrategyBackend::Jupiter { .. } => (
            parallelism_override(engine.jupiter.quote_config.parallelism),
            batch_interval_duration(engine.jupiter.quote_config.batch_interval_ms),
        ),
        StrategyBackend::Dflow { .. } => (
            parallelism_override(engine.dflow.quote_config.parallelism),
            batch_interval_duration(engine.dflow.quote_config.batch_interval_ms),
        ),
        StrategyBackend::Kamino { .. } => {
            (parallelism_override(QuoteParallelism::Auto), Duration::ZERO)
        }
        StrategyBackend::Ultra { .. } => (
            parallelism_override(engine.ultra.quote_config.parallelism),
            batch_interval_duration(engine.ultra.quote_config.batch_interval_ms),
        ),
        StrategyBackend::None => (parallelism_override(QuoteParallelism::Auto), Duration::ZERO),
    }
}

fn parallelism_override(value: QuoteParallelism) -> Option<u16> {
    match value {
        QuoteParallelism::Auto => None,
        QuoteParallelism::Fixed(limit) => Some(limit.max(1)),
    }
}

fn batch_interval_duration(value: Option<u64>) -> Duration {
    value.map(Duration::from_millis).unwrap_or(Duration::ZERO)
}

async fn run_pure_blind_engine(
    config: &AppConfig,
    backend: &StrategyBackend<'_>,
    dry_run: bool,
) -> Result<()> {
    let pure_config = &config.galileo.pure_blind_strategy;
    if !pure_config.enable {
        warn!(target: "strategy", "纯盲发策略未启用，直接退出");
        return Ok(());
    }

    let compute_unit_price_mode = derive_compute_unit_price_mode(&config.lander.lander);

    let resolved_rpc = resolve_rpc_client(&config.galileo.global)?;
    let rpc_client = resolved_rpc.client.clone();
    let rpc_endpoints = resolved_rpc.endpoints.clone();
    let mut identity =
        EngineIdentity::from_wallet(&config.galileo.global.wallet).map_err(|err| anyhow!(err))?;

    let (quote_executor, swap_preparer, _unused_defaults, jupiter_started) =
        prepare_swap_components(
            config,
            backend,
            &mut identity,
            &compute_unit_price_mode,
            false,
        )
        .await?;

    let trade_pairs = build_pure_trade_pairs(pure_config)?;
    let trade_profiles = build_pure_trade_profiles(pure_config)?;
    let profit_config = build_pure_profit_config(&config.lander.lander, &compute_unit_price_mode);
    let quote_config = build_pure_quote_config();
    let landing_timeout = resolve_landing_timeout(&config.galileo.bot);
    let ip_allocator = build_ip_allocator(&config.galileo.bot.network)?;

    let builder_config =
        BuilderConfig::new(resolve_instruction_memo(&config.galileo.global.instruction))
            .with_yellowstone(
                config.galileo.global.yellowstone_grpc_url.clone(),
                config.galileo.global.yellowstone_grpc_token.clone(),
                config.galileo.bot.get_block_hash_by_grpc,
            );
    let global_proxy = resolve_global_http_proxy(&config.galileo.global);
    let rpc_client_pool = build_rpc_client_pool(rpc_endpoints.clone(), global_proxy.clone());
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

    let market_cache_handle = init_market_cache(
        &pure_config.market_cache,
        &pure_config.assets,
        submission_client.clone(),
    )
    .await
    .map_err(|err| anyhow!("初始化纯盲发市场缓存失败: {err}"))?;
    let market_count = market_cache_handle
        .try_snapshot()
        .map_or(0, |records| records.len());
    info!(
        target: "strategy",
        markets = market_count,
        path = %pure_config.market_cache.path,
        "纯盲发市场缓存已就绪"
    );

    let marginfi_cfg = &config.galileo.flashloan.marginfi;
    let marginfi_accounts = parse_marginfi_accounts(marginfi_cfg)?;

    let prechecker = AccountPrechecker::new(rpc_client.clone(), marginfi_accounts.clone());
    let (summary, flashloan_precheck) = prechecker
        .ensure_accounts(
            &identity,
            &trade_pairs,
            config.galileo.flashloan.marginfi.enable,
        )
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

    let mut flashloan_manager =
        MarginfiFlashloanManager::new(marginfi_cfg, rpc_client.clone(), marginfi_accounts.clone());
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
    );
    let default_landers = ["rpc"];

    let lander_stack = Arc::new(
        lander_factory
            .build_stack(
                &config.lander.lander,
                &pure_config.enable_landers,
                &default_landers,
                0,
                Arc::clone(&ip_allocator),
            )
            .map_err(|err| anyhow!(err))?,
    );

    let (quote_parallelism, quote_batch_interval) =
        resolve_quote_dispatch_config(&config.galileo.engine, backend);

    let engine_settings = EngineSettings::new(quote_config)
        .with_quote_parallelism(quote_parallelism)
        .with_quote_batch_interval(quote_batch_interval)
        .with_dispatch_strategy(config.lander.lander.sending_strategy)
        .with_landing_timeout(landing_timeout)
        .with_dry_run(dry_run)
        .with_cu_multiplier(pure_config.cu_multiplier)
        .with_compute_unit_price_mode(compute_unit_price_mode.clone());

    let routes = PureBlindRouteBuilder::new(pure_config, rpc_client.as_ref(), &market_cache_handle)
        .build()
        .await
        .map_err(|err| anyhow!(err))?;

    let strategy_engine = StrategyEngine::new(
        PureBlindStrategy::new(routes).map_err(|err| anyhow!(err))?,
        lander_stack.clone(),
        identity,
        ip_allocator,
        quote_executor,
        ProfitEvaluator::new(profit_config),
        swap_preparer,
        tx_builder,
        Scheduler::new(),
        flashloan,
        engine_settings,
        trade_pairs,
        trade_profiles,
        None,
    );
    drive_engine(strategy_engine)
        .await
        .map_err(|err| anyhow!(err))?;

    if jupiter_started {
        if let StrategyBackend::Jupiter { manager, .. } = backend {
            if let Err(err) = manager.stop().await {
                warn!(
                    target: "strategy",
                    error = %err,
                    "停止 Jupiter 二进制失败"
                );
            }
        }
    }

    Ok(())
}

async fn prepare_swap_components(
    config: &AppConfig,
    backend: &StrategyBackend<'_>,
    identity: &mut EngineIdentity,
    compute_unit_price_mode: &Option<ComputeUnitPriceMode>,
    start_local_jupiter: bool,
) -> Result<(QuoteExecutor, SwapPreparer, (bool, bool), bool)> {
    match backend {
        StrategyBackend::Jupiter {
            manager,
            api_client,
        } => {
            let mut jupiter_started = false;
            if start_local_jupiter {
                if !manager.disable_local_binary {
                    match manager.start(false).await {
                        Ok(()) => {
                            info!(target: "strategy", "已启动本地 Jupiter 二进制");
                            jupiter_started = true;
                        }
                        Err(JupiterError::AlreadyRunning) => {
                            info!(target: "strategy", "本地 Jupiter 二进制已在运行");
                            jupiter_started = true;
                        }
                        Err(err) => return Err(err.into()),
                    }
                } else {
                    info!(
                        target: "strategy",
                        "已禁用本地 Jupiter 二进制，将使用远端 API"
                    );
                }
            } else {
                info!(
                    target: "strategy",
                    "跳过本地 Jupiter 二进制启动（纯盲发模式）"
                );
            }

            identity.set_skip_user_accounts_rpc_calls(
                config
                    .galileo
                    .engine
                    .jupiter
                    .swap_config
                    .skip_user_accounts_rpc_calls,
            );

            let quote_defaults_cfg = config.galileo.engine.jupiter.quote_config.clone();
            let swap_defaults_cfg = config.galileo.engine.jupiter.swap_config.clone();
            let quote_executor =
                QuoteExecutor::for_jupiter((*api_client).clone(), quote_defaults_cfg.clone());
            let swap_preparer = SwapPreparer::for_jupiter(
                (*api_client).clone(),
                swap_defaults_cfg.clone(),
                compute_unit_price_mode.clone(),
            );
            Ok((
                quote_executor,
                swap_preparer,
                (
                    quote_defaults_cfg.only_direct_routes,
                    quote_defaults_cfg.restrict_intermediate_tokens,
                ),
                jupiter_started,
            ))
        }
        StrategyBackend::Dflow { api_client } => {
            info!(target: "strategy", "使用 DFlow 聚合器，不启动本地 Jupiter 二进制");
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
                false,
            ))
        }
        StrategyBackend::Kamino { api_client } => {
            info!(target: "strategy", "使用 Kamino 聚合器");
            identity.set_skip_user_accounts_rpc_calls(false);

            let kamino_quote_cfg = config.galileo.engine.kamino.quote_config.clone();
            let quote_executor =
                QuoteExecutor::for_kamino((*api_client).clone(), kamino_quote_cfg.clone());
            let swap_preparer = SwapPreparer::for_kamino(compute_unit_price_mode.clone());
            Ok((quote_executor, swap_preparer, (true, true), false))
        }
        StrategyBackend::Ultra {
            api_client,
            rpc_client,
        } => {
            identity.set_skip_user_accounts_rpc_calls(false);
            let ultra_quote_cfg = config.galileo.engine.ultra.quote_config.clone();
            let quote_executor = QuoteExecutor::for_ultra((*api_client).clone(), ultra_quote_cfg);
            let swap_preparer =
                SwapPreparer::for_ultra(rpc_client.clone(), compute_unit_price_mode.clone());
            Ok((quote_executor, swap_preparer, (true, true), false))
        }
        StrategyBackend::None => {
            identity.set_skip_user_accounts_rpc_calls(false);
            let quote_executor = QuoteExecutor::disabled();
            let swap_preparer = SwapPreparer::disabled();
            Ok((quote_executor, swap_preparer, (true, true), false))
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
) -> EngineResult<BTreeMap<Pubkey, TradeProfile>> {
    let mut per_mint: BTreeMap<Pubkey, TradeProfile> = BTreeMap::new();

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

        let amounts = generate_amounts_for_base(base);
        if amounts.is_empty() {
            continue;
        }
        let delay_ms = base
            .process_delay
            .unwrap_or(DEFAULT_PROCESS_DELAY_MS)
            .max(1);
        let process_delay = Duration::from_millis(delay_ms);
        per_mint.insert(
            mint,
            TradeProfile {
                amounts,
                process_delay,
            },
        );
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

        let amounts = normalize_pure_trade_sizes(&base.trade_sizes);
        if amounts.is_empty() {
            continue;
        }

        let delay_ms = base
            .process_delay
            .unwrap_or(DEFAULT_PROCESS_DELAY_MS)
            .max(1);
        let process_delay = Duration::from_millis(delay_ms);
        per_mint.insert(
            mint,
            TradeProfile {
                amounts,
                process_delay,
            },
        );
    }

    if per_mint.is_empty() {
        return Err(EngineError::InvalidConfig(
            "纯盲发策略未配置有效的交易规模".into(),
        ));
    }

    Ok(per_mint)
}

fn generate_amounts_for_base(base: &config::BlindBaseMintConfig) -> Vec<u64> {
    if base.trade_size_range.is_empty() {
        return Vec::new();
    }

    let mut values: BTreeSet<u64> = base.trade_size_range.iter().copied().collect();
    if base.trade_size_range.len() >= 2 {
        if let Some(count) = base.trade_range_count {
            if count >= 2 {
                let mut sorted = base.trade_size_range.clone();
                sorted.sort_unstable();
                let min = sorted.first().copied().unwrap_or(0);
                let max = sorted.last().copied().unwrap_or(min);
                let strategy = base
                    .trade_range_strategy
                    .as_deref()
                    .map(|s| s.to_ascii_lowercase())
                    .unwrap_or_else(|| "linear".to_string());
                for amount in generate_amounts(min, max, count, &strategy) {
                    values.insert(amount);
                }
            }
        }
    }

    let mut rng = rand::rng();
    let mut tweaked: BTreeSet<u64> = BTreeSet::new();
    for amount in values {
        let basis_points: u16 = rng.random_range(930..=999);
        let adjusted = (((amount as u128) * basis_points as u128) + 999) / 1_000;
        let normalized = adjusted.max(1).min(u128::from(u64::MAX)) as u64;
        tweaked.insert(normalized);
    }

    tweaked.into_iter().collect()
}

fn normalize_pure_trade_sizes(values: &[u64]) -> Vec<u64> {
    if values.is_empty() {
        return Vec::new();
    }

    let unique: BTreeSet<u64> = values.iter().copied().filter(|value| *value > 0).collect();
    if unique.is_empty() {
        return Vec::new();
    }

    let mut rng = rand::rng();
    let mut tweaked: BTreeSet<u64> = BTreeSet::new();
    for amount in unique {
        let basis_points: u16 = rng.random_range(930..=999);
        let adjusted = (((amount as u128) * basis_points as u128) + 999) / 1_000;
        let normalized = adjusted.max(1).min(u128::from(u64::MAX)) as u64;
        tweaked.insert(normalized);
    }

    tweaked.into_iter().collect()
}

fn build_blind_profit_config(
    config: &config::BlindStrategyConfig,
    lander_settings: &config::LanderSettings,
    compute_unit_price_mode: &Option<ComputeUnitPriceMode>,
) -> ProfitConfig {
    let min_profit_from_routes = config
        .base_mints
        .iter()
        .filter_map(|mint| mint.min_quote_profit)
        .min()
        .unwrap_or(0);

    let compute_unit_fee = compute_unit_price_mode
        .as_ref()
        .map(|mode| match mode {
            ComputeUnitPriceMode::Fixed(value) => compute_unit_fee_lamports(*value),
            ComputeUnitPriceMode::Random { min, max } => {
                let upper = (*min).max(*max);
                compute_unit_fee_lamports(upper)
            }
        })
        .unwrap_or(0);

    let tip_fee = lander_settings
        .jito
        .as_ref()
        .and_then(|cfg| {
            if let Some(fixed) = cfg.fixed_tip {
                Some(fixed)
            } else if !cfg.range_tips.is_empty() {
                cfg.range_tips.iter().copied().max()
            } else {
                None
            }
        })
        .unwrap_or(0);

    let required_profit_floor = compute_unit_fee.saturating_add(tip_fee);

    let threshold = min_profit_from_routes.max(required_profit_floor);

    ProfitConfig {
        min_profit_threshold_lamports: threshold,
        max_tip_lamports: 0,
        tip: TipConfig::default(),
    }
}

fn build_pure_profit_config(
    lander_settings: &config::LanderSettings,
    compute_unit_price_mode: &Option<ComputeUnitPriceMode>,
) -> ProfitConfig {
    let compute_unit_fee = compute_unit_price_mode
        .as_ref()
        .map(|mode| match mode {
            ComputeUnitPriceMode::Fixed(value) => compute_unit_fee_lamports(*value),
            ComputeUnitPriceMode::Random { min, max } => {
                let upper = (*min).max(*max);
                compute_unit_fee_lamports(upper)
            }
        })
        .unwrap_or(0);

    let tip_fee = lander_settings
        .jito
        .as_ref()
        .and_then(|cfg| {
            if let Some(fixed) = cfg.fixed_tip {
                Some(fixed)
            } else if !cfg.range_tips.is_empty() {
                cfg.range_tips.iter().copied().max()
            } else {
                None
            }
        })
        .unwrap_or(0);

    ProfitConfig {
        min_profit_threshold_lamports: compute_unit_fee.saturating_add(tip_fee),
        max_tip_lamports: 0,
        tip: TipConfig::default(),
    }
}

fn compute_unit_fee_lamports(price_micro_lamports: u64) -> u64 {
    let limit = FALLBACK_CU_LIMIT as u128;
    let numerator = (price_micro_lamports as u128).saturating_mul(limit);
    ((numerator + 999_999) / 1_000_000).min(u128::from(u64::MAX)) as u64
}

fn build_blind_quote_config(
    config: &config::BlindStrategyConfig,
    only_direct_routes_default: bool,
    restrict_intermediate_tokens_default: bool,
) -> QuoteConfig {
    let has_three_hop = config.base_mints.iter().any(|mint| {
        mint.route_types
            .iter()
            .any(|t| t.eq_ignore_ascii_case("3hop"))
    });

    let mut only_direct_routes = only_direct_routes_default;
    if has_three_hop {
        only_direct_routes = false;
    }

    QuoteConfig {
        slippage_bps: 0,
        only_direct_routes,
        restrict_intermediate_tokens: restrict_intermediate_tokens_default,
        quote_max_accounts: None,
        dex_whitelist: config.enable_dexs.clone(),
        dex_blacklist: config.exclude_dexes.clone(),
    }
}

fn build_pure_quote_config() -> QuoteConfig {
    QuoteConfig {
        slippage_bps: 0,
        only_direct_routes: true,
        restrict_intermediate_tokens: true,
        quote_max_accounts: None,
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

pub(crate) fn build_ip_allocator(cfg: &config::NetworkConfig) -> Result<Arc<IpAllocator>> {
    let inventory_cfg = IpInventoryConfig {
        enable_multiple_ip: cfg.enable_multiple_ip,
        manual_ips: cfg.manual_ips.clone(),
        blacklist: cfg.blacklist_ips.clone(),
        allow_loopback: cfg.allow_loopback,
    };

    let inventory =
        IpInventory::new(inventory_cfg).map_err(|err| anyhow!("初始化本地 IP 资源失败: {err}"))?;

    let cooldown = CooldownConfig {
        rate_limited_start: Duration::from_millis(cfg.cooldown_ms.rate_limited_start.max(1)),
        timeout_start: Duration::from_millis(cfg.cooldown_ms.timeout_start.max(1)),
    };

    let allocator = IpAllocator::from_inventory(
        inventory,
        cfg.per_ip_inflight_limit.map(|limit| limit as usize),
        cooldown,
    );

    let summary = allocator.summary();
    let ips: Vec<String> = allocator
        .slot_ips()
        .into_iter()
        .map(|ip| ip.to_string())
        .collect();

    info!(
        target: "network::allocator",
        total_slots = summary.total_slots,
        per_ip_limit = ?summary.per_ip_inflight_limit,
        source = ?summary.source,
        ips = ?ips,
        "IP 资源池已初始化"
    );

    Ok(Arc::new(allocator))
}

fn resolve_landing_timeout(bot: &config::BotConfig) -> Duration {
    let ms = bot.landing_ms.unwrap_or(2_000).max(1);
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
