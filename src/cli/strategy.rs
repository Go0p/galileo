use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;
use std::time::Duration;

use anyhow::{Result, anyhow};
use tracing::{info, warn};

use crate::api::dflow::DflowApiClient;
use crate::api::jupiter::JupiterApiClient;
use crate::cli::context::{
    resolve_global_http_proxy, resolve_instruction_memo, resolve_rpc_client,
};
use crate::config;
use crate::config::{AppConfig, IntermediumConfig};
use crate::engine::{
    AccountPrechecker, BuilderConfig, ComputeUnitPriceMode, EngineError, EngineIdentity,
    EngineResult, EngineSettings, FALLBACK_CU_LIMIT, ProfitConfig, ProfitEvaluator, QuoteConfig,
    QuoteExecutor, Scheduler, StrategyEngine, SwapInstructionFetcher, TipConfig, TradeProfile,
    TransactionBuilder,
};
use crate::flashloan::marginfi::{MarginfiAccountRegistry, MarginfiFlashloanManager};
use crate::jupiter::{JupiterBinaryManager, JupiterError};
use crate::lander::LanderFactory;
use crate::monitoring::events;
use crate::pure_blind::market_cache::init_market_cache;
use crate::strategy::{
    BlindStrategy, PureBlindRouteBuilder, PureBlindStrategy, Strategy, StrategyEvent,
};
use rand::Rng as _;
use solana_sdk::pubkey::Pubkey;

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
    let blind_config = &config.galileo.blind_strategy;
    let pure_config = &config.galileo.pure_blind_strategy;

    if matches!(
        config.galileo.engine.backend,
        crate::config::EngineBackend::None
    ) && blind_config.enable
    {
        return Err(anyhow!(
            "engine.backend=none 仅支持纯盲发策略，请关闭 blind_strategy.enable"
        ));
    }

    match (blind_config.enable, pure_config.enable) {
        (false, false) => {
            warn!(target: "strategy", "盲发策略未启用，直接退出");
            Ok(())
        }
        (true, true) => Err(anyhow!(
            "blind_strategy.enable 与 pure_blind_strategy.enable 不能同时为 true"
        )),
        (true, false) => run_blind_engine(config, backend, dry_run).await,
        (false, true) => run_pure_blind_engine(config, backend, dry_run).await,
    }
}

async fn run_blind_engine(
    config: &AppConfig,
    backend: &StrategyBackend<'_>,
    dry_run: bool,
) -> Result<()> {
    let blind_config = &config.galileo.blind_strategy;

    if !blind_config.enable {
        warn!(target: "strategy", "盲发策略未启用，直接退出");
        return Ok(());
    }

    let compute_unit_price_mode = derive_compute_unit_price_mode(&config.lander.lander);

    let rpc_client = resolve_rpc_client(&config.galileo.global)?;
    let mut identity =
        EngineIdentity::from_wallet(&config.galileo.global.wallet).map_err(|err| anyhow!(err))?;

    let (quote_executor, swap_fetcher, quote_defaults_tuple, jupiter_started) =
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
    let profit_config = build_blind_profit_config(
        blind_config,
        &config.lander.lander,
        &compute_unit_price_mode,
    );
    let scheduler_delay = resolve_blind_scheduler_delay(&trade_profiles);
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
    let mut submission_builder = reqwest::Client::builder();
    if let Some(proxy_url) = resolve_global_http_proxy(&config.galileo.global) {
        let proxy = reqwest::Proxy::all(&proxy_url)
            .map_err(|err| anyhow!("global.proxy 地址无效 {proxy_url}: {err}"))?;
        submission_builder = submission_builder
            .proxy(proxy)
            .danger_accept_invalid_certs(true);
    }
    let submission_client = submission_builder.build()?;

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

    let lander_factory = LanderFactory::new(rpc_client.clone(), submission_client.clone());
    let default_landers = ["rpc"];

    let lander_stack = lander_factory
        .build_stack(
            &config.lander.lander,
            &blind_config.enable_landers,
            &default_landers,
            0,
        )
        .map_err(|err| anyhow!(err))?;

    let failure_limit = if matches!(
        config.galileo.engine.backend,
        crate::config::EngineBackend::Dflow
    ) {
        let raw = config.galileo.engine.dflow.max_consecutive_failures as usize;
        if raw == 0 { None } else { Some(raw) }
    } else {
        None
    };

    let engine_settings = EngineSettings::new(quote_config)
        .with_dispatch_strategy(config.lander.lander.sending_strategy)
        .with_landing_timeout(landing_timeout)
        .with_dry_run(dry_run)
        .with_failure_tolerance(failure_limit)
        .with_cu_multiplier(1.0)
        .with_compute_unit_price_mode(compute_unit_price_mode.clone());

    let strategy_engine = StrategyEngine::new(
        BlindStrategy::new(),
        lander_stack,
        identity,
        quote_executor,
        ProfitEvaluator::new(profit_config),
        swap_fetcher,
        TransactionBuilder::new(rpc_client.clone(), builder_config),
        Scheduler::new(scheduler_delay),
        flashloan,
        engine_settings,
        trade_pairs,
        trade_profiles,
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

    let rpc_client = resolve_rpc_client(&config.galileo.global)?;
    let mut identity =
        EngineIdentity::from_wallet(&config.galileo.global.wallet).map_err(|err| anyhow!(err))?;

    let (quote_executor, swap_fetcher, _unused_defaults, jupiter_started) =
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
    let scheduler_delay = resolve_blind_scheduler_delay(&trade_profiles);
    let quote_config = build_pure_quote_config();
    let landing_timeout = resolve_landing_timeout(&config.galileo.bot);

    let builder_config =
        BuilderConfig::new(resolve_instruction_memo(&config.galileo.global.instruction))
            .with_yellowstone(
                config.galileo.global.yellowstone_grpc_url.clone(),
                config.galileo.global.yellowstone_grpc_token.clone(),
                config.galileo.bot.get_block_hash_by_grpc,
            );
    let mut submission_builder = reqwest::Client::builder();
    if let Some(proxy_url) = resolve_global_http_proxy(&config.galileo.global) {
        let proxy = reqwest::Proxy::all(&proxy_url)
            .map_err(|err| anyhow!("global.proxy 地址无效 {proxy_url}: {err}"))?;
        submission_builder = submission_builder
            .proxy(proxy)
            .danger_accept_invalid_certs(true);
    }
    let submission_client = submission_builder.build()?;

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

    let lander_factory = LanderFactory::new(rpc_client.clone(), submission_client.clone());
    let default_landers = ["rpc"];

    let lander_stack = lander_factory
        .build_stack(
            &config.lander.lander,
            &pure_config.enable_landers,
            &default_landers,
            0,
        )
        .map_err(|err| anyhow!(err))?;

    let failure_limit = if matches!(
        config.galileo.engine.backend,
        crate::config::EngineBackend::Dflow
    ) {
        let raw = config.galileo.engine.dflow.max_consecutive_failures as usize;
        if raw == 0 { None } else { Some(raw) }
    } else {
        None
    };

    let engine_settings = EngineSettings::new(quote_config)
        .with_dispatch_strategy(config.lander.lander.sending_strategy)
        .with_landing_timeout(landing_timeout)
        .with_dry_run(dry_run)
        .with_failure_tolerance(failure_limit)
        .with_cu_multiplier(pure_config.cu_multiplier)
        .with_compute_unit_price_mode(compute_unit_price_mode.clone());

    let routes = PureBlindRouteBuilder::new(pure_config, rpc_client.as_ref(), &market_cache_handle)
        .build()
        .await
        .map_err(|err| anyhow!(err))?;

    let strategy_engine = StrategyEngine::new(
        PureBlindStrategy::new(routes).map_err(|err| anyhow!(err))?,
        lander_stack,
        identity,
        quote_executor,
        ProfitEvaluator::new(profit_config),
        swap_fetcher,
        TransactionBuilder::new(rpc_client.clone(), builder_config),
        Scheduler::new(scheduler_delay),
        flashloan,
        engine_settings,
        trade_pairs,
        trade_profiles,
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
) -> Result<(QuoteExecutor, SwapInstructionFetcher, (bool, bool), bool)> {
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
            let swap_fetcher = SwapInstructionFetcher::for_jupiter(
                (*api_client).clone(),
                swap_defaults_cfg.clone(),
                compute_unit_price_mode.clone(),
            );
            Ok((
                quote_executor,
                swap_fetcher,
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
            let swap_fetcher = SwapInstructionFetcher::for_dflow(
                (*api_client).clone(),
                config.galileo.engine.dflow.swap_config.clone(),
                compute_unit_price_mode.clone(),
            );
            Ok((
                quote_executor,
                swap_fetcher,
                (dflow_quote_cfg.only_direct_routes, true),
                false,
            ))
        }
        StrategyBackend::None => {
            identity.set_skip_user_accounts_rpc_calls(false);
            let quote_executor = QuoteExecutor::disabled();
            let swap_fetcher = SwapInstructionFetcher::disabled();
            Ok((quote_executor, swap_fetcher, (true, true), false))
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

fn resolve_blind_scheduler_delay(profiles: &BTreeMap<Pubkey, TradeProfile>) -> Duration {
    profiles
        .values()
        .map(|profile| profile.process_delay)
        .min()
        .unwrap_or_else(|| Duration::from_millis(DEFAULT_PROCESS_DELAY_MS))
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
