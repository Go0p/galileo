use std::collections::BTreeSet;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{Result, anyhow};
use tracing::{info, warn};

use crate::api::JupiterApiClient;
use crate::cli::context::{
    resolve_global_http_proxy, resolve_instruction_memo, resolve_rpc_client,
};
use crate::config;
use crate::config::{AppConfig, IntermediumConfig};
use crate::engine::{
    AccountPrechecker, BuilderConfig, ComputeUnitPriceMode, EngineError, EngineIdentity,
    EngineResult, EngineSettings, ProfitConfig, ProfitEvaluator, QuoteConfig, QuoteExecutor,
    Scheduler, StrategyEngine, SwapInstructionFetcher, TipConfig, TransactionBuilder,
};
use crate::flashloan::FlashloanManager;
use crate::jupiter::{JupiterBinaryManager, JupiterError};
use crate::lander::LanderFactory;
use crate::monitoring::events;
use crate::strategy::{
    BlindStrategy, PureBlindRouteBuilder, PureBlindStrategy, Strategy, StrategyEvent,
};
use solana_sdk::pubkey::Pubkey;

/// 控制策略以正式模式还是 dry-run 模式运行。
pub enum StrategyMode {
    Live,
    DryRun,
}

/// 主策略入口，按配置在盲发与 copy 之间切换。
pub async fn run_strategy(
    config: &AppConfig,
    manager: &JupiterBinaryManager,
    api_client: &JupiterApiClient,
    mode: StrategyMode,
) -> Result<()> {
    let dry_run = matches!(mode, StrategyMode::DryRun) || config.galileo.bot.dry_run;
    if !config.galileo.blind_strategy.enable {
        warn!(target: "strategy", "盲发策略未启用，直接退出");
        return Ok(());
    }
    run_blind_engine(config, manager, api_client, dry_run).await
}

async fn run_blind_engine(
    config: &AppConfig,
    manager: &JupiterBinaryManager,
    api_client: &JupiterApiClient,
    dry_run: bool,
) -> Result<()> {
    let blind_config = &config.galileo.blind_strategy;

    if !blind_config.enable {
        warn!(target: "strategy", "盲发策略未启用，直接退出");
        return Ok(());
    }

    let pure_mode = blind_config.pure_mode;
    let mut jupiter_started = false;
    if pure_mode {
        info!(
            target: "strategy",
            "纯盲发模式已开启，不启动本地 Jupiter 二进制"
        );
    } else if !manager.disable_local_binary {
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

    let rpc_client = resolve_rpc_client(&config.galileo.global)?;
    let mut identity =
        EngineIdentity::from_wallet(&config.galileo.global.wallet).map_err(|err| anyhow!(err))?;
    identity.set_skip_user_accounts_rpc_calls(
        config
            .galileo
            .engine
            .jupiter
            .swap_config
            .skip_user_accounts_rpc_calls,
    );
    let trade_pairs = build_blind_trade_pairs(blind_config, &config.galileo.intermedium)?;
    let trade_amounts = build_blind_trade_amounts(blind_config)?;
    let profit_config = build_blind_profit_config(blind_config);
    let scheduler_delay = resolve_blind_scheduler_delay(blind_config);
    let quote_defaults = config.galileo.engine.jupiter.quote_config.clone();
    let quote_config = build_blind_quote_config(blind_config, &quote_defaults);
    tracing::info!(
        target: "engine::config",
        dex_whitelist = ?quote_config.dex_whitelist,
        "盲发策略 DEX 白名单"
    );
    let landing_timeout = resolve_landing_timeout(&config.galileo.bot);

    let builder_config =
        BuilderConfig::new(resolve_instruction_memo(&config.galileo.global.instruction));
    let swap_defaults = config.galileo.engine.jupiter.swap_config.clone();
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
    let configured_marginfi = parse_marginfi_account(marginfi_cfg)?;

    let prechecker = AccountPrechecker::new(rpc_client.clone(), configured_marginfi);
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

    let mut flashloan_manager =
        FlashloanManager::new(marginfi_cfg, rpc_client.clone(), configured_marginfi);
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

    let compute_unit_price_mode = derive_compute_unit_price_mode(&config.lander.lander);
    let engine_settings = EngineSettings::new(quote_config)
        .with_landing_timeout(landing_timeout)
        .with_dry_run(dry_run);

    if pure_mode {
        let routes = PureBlindRouteBuilder::new(blind_config, rpc_client.as_ref())
            .build()
            .await
            .map_err(|err| anyhow!(err))?;
        let strategy_engine = StrategyEngine::new(
            PureBlindStrategy::new(routes).map_err(|err| anyhow!(err))?,
            lander_stack,
            identity,
            QuoteExecutor::new(api_client.clone(), quote_defaults),
            ProfitEvaluator::new(profit_config),
            SwapInstructionFetcher::new(api_client.clone(), swap_defaults, compute_unit_price_mode),
            TransactionBuilder::new(rpc_client.clone(), builder_config),
            Scheduler::new(scheduler_delay),
            flashloan,
            engine_settings,
            trade_pairs,
            trade_amounts,
        );
        drive_engine(strategy_engine)
            .await
            .map_err(|err| anyhow!(err))?;
    } else {
        let strategy_engine = StrategyEngine::new(
            BlindStrategy::new(),
            lander_stack,
            identity,
            QuoteExecutor::new(api_client.clone(), quote_defaults),
            ProfitEvaluator::new(profit_config),
            SwapInstructionFetcher::new(api_client.clone(), swap_defaults, compute_unit_price_mode),
            TransactionBuilder::new(rpc_client.clone(), builder_config),
            Scheduler::new(scheduler_delay),
            flashloan,
            engine_settings,
            trade_pairs,
            trade_amounts,
        );
        drive_engine(strategy_engine)
            .await
            .map_err(|err| anyhow!(err))?;
    }

    if !pure_mode && jupiter_started {
        if let Err(err) = manager.stop().await {
            warn!(
                target: "strategy",
                error = %err,
                "停止 Jupiter 二进制失败"
            );
        }
    }

    Ok(())
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

fn build_blind_trade_amounts(config: &config::BlindStrategyConfig) -> EngineResult<Vec<u64>> {
    let mut amounts: BTreeSet<u64> = BTreeSet::new();
    for mint in &config.base_mints {
        for value in generate_amounts_for_base(mint) {
            amounts.insert(value);
        }
    }

    if amounts.is_empty() {
        return Err(EngineError::InvalidConfig(
            "盲发策略未配置有效的交易规模".into(),
        ));
    }

    Ok(amounts.into_iter().collect())
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

    values.into_iter().collect()
}

fn build_blind_profit_config(config: &config::BlindStrategyConfig) -> ProfitConfig {
    let min_profit = config
        .base_mints
        .iter()
        .filter_map(|mint| mint.min_quote_profit)
        .min()
        .unwrap_or(0);

    ProfitConfig {
        min_profit_threshold_lamports: min_profit,
        max_tip_lamports: 0,
        tip: TipConfig::default(),
    }
}

fn resolve_blind_scheduler_delay(config: &config::BlindStrategyConfig) -> Duration {
    let delay = config
        .base_mints
        .iter()
        .filter_map(|mint| mint.process_delay)
        .min()
        .unwrap_or(200)
        .max(1);
    Duration::from_millis(delay as u64)
}

fn build_blind_quote_config(
    config: &config::BlindStrategyConfig,
    defaults: &config::JupiterQuoteConfig,
) -> QuoteConfig {
    let only_direct_routes = if config.base_mints.iter().any(|mint| {
        mint.route_types
            .iter()
            .any(|t| t.eq_ignore_ascii_case("2hop"))
    }) {
        true
    } else {
        defaults.only_direct_routes
    };

    QuoteConfig {
        slippage_bps: 0,
        only_direct_routes,
        restrict_intermediate_tokens: defaults.restrict_intermediate_tokens,
        quote_max_accounts: None,
        dex_whitelist: config.enable_dexs.clone(),
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

fn parse_marginfi_account(cfg: &config::FlashloanMarginfiConfig) -> Result<Option<Pubkey>> {
    match cfg.marginfi_account.as_deref().map(str::trim) {
        Some("") | None => Ok(None),
        Some(value) => Pubkey::from_str(value)
            .map(Some)
            .map_err(|err| anyhow!("flashloan.marginfi.marginfi_account 无效: {err}")),
    }
}
