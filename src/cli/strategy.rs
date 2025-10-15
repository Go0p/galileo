use std::collections::BTreeSet;
use std::str::FromStr;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use tracing::{info, warn};

use crate::api::JupiterApiClient;
use crate::cli::context::{
    resolve_instruction_memo, resolve_rpc_client, resolve_titan_ws_endpoint,
};
use crate::config;
use crate::config::{AppConfig, ArbEngine, IntermediumConfig, RequestParamsConfig};
use crate::engine::{
    AccountPrechecker, BuilderConfig, EngineError, EngineIdentity, EngineResult, EngineSettings,
    ProfitConfig, ProfitEvaluator, QuoteConfig, QuoteExecutor, Scheduler, StrategyEngine,
    SwapInstructionFetcher, TipConfig, TransactionBuilder,
};
use crate::flashloan::FlashloanManager;
use crate::jupiter::{JupiterBinaryManager, JupiterError};
use crate::lander::{Deadline, LanderFactory};
use crate::monitoring::events;
use crate::strategy::{
    BlindStrategy, CopyStrategy, CopySwapParams, Strategy, StrategyEvent,
    compute_associated_token_address, usdc_mint, wsol_mint,
};
use crate::titan::{TitanStreamConfig, spawn_quote_streams};
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
    match config.galileo.bot.arb_engine {
        ArbEngine::Jupiter => {
            run_blind_engine(config, manager, api_client, dry_run, ArbEngine::Jupiter).await
        }
        ArbEngine::Titan => {
            run_blind_engine(config, manager, api_client, dry_run, ArbEngine::Titan).await
        }
        ArbEngine::Dflow => {
            if !config.galileo.copy_strategy.enable {
                return Err(anyhow!("arb_engine=dflow 时必须启用 copy_strategy 配置"));
            }
            run_copy_strategy(config, dry_run).await
        }
    }
}

async fn run_blind_engine(
    config: &AppConfig,
    manager: &JupiterBinaryManager,
    api_client: &JupiterApiClient,
    dry_run: bool,
    engine: ArbEngine,
) -> Result<()> {
    let blind_config = &config.galileo.blind_strategy;
    if !blind_config.enable {
        warn!(target: "strategy", "盲发策略未启用，直接退出");
        return Ok(());
    }

    let mut jupiter_started = false;
    if matches!(engine, ArbEngine::Jupiter) && !manager.disable_local_binary {
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
    } else if matches!(engine, ArbEngine::Jupiter) {
        info!(
            target: "strategy",
            "已禁用本地 Jupiter 二进制，将使用远端 API"
        );
    } else {
        info!(
            target: "strategy",
            arb_engine = ?engine,
            "跳过启动 Jupiter 二进制"
        );
    }

    let rpc_client = resolve_rpc_client(&config.galileo.global)?;
    let mut identity =
        EngineIdentity::from_wallet(&config.galileo.global.wallet).map_err(|err| anyhow!(err))?;
    identity.set_skip_user_accounts_rpc_calls(
        config.galileo.request_params.skip_user_accounts_rpc_calls,
    );
    let trade_pairs = build_blind_trade_pairs(blind_config, &config.galileo.intermedium)?;
    let trade_amounts = build_blind_trade_amounts(blind_config)?;
    let profit_config = build_blind_profit_config(blind_config);
    let scheduler_delay = resolve_blind_scheduler_delay(blind_config);
    let quote_config = build_blind_quote_config(blind_config, &config.galileo.request_params);
    tracing::debug!(
        target: "engine::config",
        dex_whitelist = ?quote_config.dex_whitelist,
        "盲发策略 DEX 白名单"
    );
    let landing_timeout = resolve_landing_timeout(&config.galileo.bot);

    let builder_config =
        BuilderConfig::new(resolve_instruction_memo(&config.galileo.global.instruction));
    let request_defaults = config.galileo.request_params.clone();
    let submission_client = reqwest::Client::builder().build()?;

    let prechecker = AccountPrechecker::new(rpc_client.clone());
    let (summary, flashloan_precheck) = prechecker
        .ensure_accounts(&identity, &trade_pairs, config.galileo.flashloan.enable)
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
        FlashloanManager::new(&config.galileo.flashloan, rpc_client.clone());
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

    let titan_stream = if matches!(engine, ArbEngine::Titan) {
        if let Some(endpoint) = resolve_titan_ws_endpoint(&config.galileo.global)? {
            let titan_stream_user =
                Pubkey::from_str("Titan11111111111111111111111111111111111111")
                    .expect("valid Titan placeholder pubkey");
            let titan_config = TitanStreamConfig {
                endpoint,
                user_pubkey: titan_stream_user,
                trade_pairs: &trade_pairs,
                trade_amounts: &trade_amounts,
                quote: &quote_config,
                strategy_label: "blind",
            };
            match spawn_quote_streams(titan_config).await {
                Ok(stream) => {
                    info!(
                        target: "strategy",
                        "Titan quote stream enabled for blind strategy"
                    );
                    Some(stream)
                }
                Err(err) => return Err(anyhow!(err)),
            }
        } else {
            warn!(
                target: "strategy",
                "Titan quote stream未启用：缺少 titan_jwt"
            );
            None
        }
    } else {
        None
    };

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

    let strategy_engine = StrategyEngine::new(
        BlindStrategy::new(),
        lander_stack,
        identity,
        QuoteExecutor::new(api_client.clone(), request_defaults.clone()),
        ProfitEvaluator::new(profit_config),
        SwapInstructionFetcher::new(api_client.clone(), request_defaults),
        TransactionBuilder::new(rpc_client.clone(), builder_config),
        Scheduler::new(scheduler_delay),
        flashloan,
        EngineSettings::new(quote_config)
            .with_landing_timeout(landing_timeout)
            .with_dry_run(dry_run),
        trade_pairs,
        trade_amounts,
        titan_stream,
    );
    drive_engine(strategy_engine)
        .await
        .map_err(|err| anyhow!(err))?;

    if jupiter_started {
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

async fn run_copy_strategy(config: &AppConfig, dry_run: bool) -> Result<()> {
    use tokio::time::sleep;

    let copy_cfg = &config.galileo.copy_strategy;
    let mut plans = build_copy_plan_states(copy_cfg).map_err(|err| anyhow!(err))?;

    let rpc_client = resolve_rpc_client(&config.galileo.global)?;
    let mut identity =
        EngineIdentity::from_wallet(&config.galileo.global.wallet).map_err(|err| anyhow!(err))?;
    identity.set_skip_user_accounts_rpc_calls(
        config.galileo.request_params.skip_user_accounts_rpc_calls,
    );

    let wsol_mint = wsol_mint();
    let usdc_mint = usdc_mint();
    let wsol_ata = compute_associated_token_address(&identity.pubkey, &wsol_mint);
    let usdc_ata = compute_associated_token_address(&identity.pubkey, &usdc_mint);

    let compute_unit_limit = 450_000u32;
    let compute_unit_price_micro_lamports =
        resolve_priority_fee_micro_lamports(&config.lander.lander, compute_unit_limit);
    let strategy = CopyStrategy::new(
        identity.pubkey,
        wsol_ata,
        usdc_ata,
        compute_unit_limit,
        compute_unit_price_micro_lamports,
    );

    let submission_client = reqwest::Client::builder().build()?;
    let lander_factory = LanderFactory::new(rpc_client.clone(), submission_client.clone());
    let default_landers = ["rpc"];
    let lander_stack = lander_factory
        .build_stack(
            &config.lander.lander,
            &copy_cfg.enable_landers,
            &default_landers,
            0,
        )
        .map_err(|err| anyhow!(err))?;

    if lander_stack.is_empty() {
        return Err(anyhow!("未配置可用的落地器"));
    }

    let builder_config =
        BuilderConfig::new(resolve_instruction_memo(&config.galileo.global.instruction));
    let tx_builder = TransactionBuilder::new(rpc_client.clone(), builder_config);
    let landing_timeout = resolve_landing_timeout(&config.galileo.bot);

    let mut idx = 0usize;
    loop {
        if plans.is_empty() {
            warn!(target: "strategy::copy", "没有可执行的交易规模，退出");
            break;
        }
        if idx >= plans.len() {
            idx = 0;
        }

        let wait_duration = {
            let plan = &plans[idx];
            if Instant::now() < plan.next_ready_at {
                plan.next_ready_at - Instant::now()
            } else {
                Duration::ZERO
            }
        };

        if !wait_duration.is_zero() {
            sleep(wait_duration).await;
        }

        let (amount_in, reverse_amount, process_delay, sending_cooldown) = {
            let plan = &plans[idx];
            (
                plan.amount_in,
                plan.reverse_amount,
                plan.process_delay,
                plan.sending_cooldown,
            )
        };

        if dry_run {
            // dry-run 下仅校验规模是否合理，不走构建与落地流程。
            info!(
                target: "strategy::copy",
                amount_in,
                reverse_amount,
                "dry-run：仅构建 copy 策略交易，不发送"
            );
        } else {
            let response = strategy.build_swap_instructions(CopySwapParams {
                amount_in,
                reverse_amount,
            });
            match tx_builder
                .build(&identity, &response, 0)
                .await
                .map_err(|err| anyhow!(err))
            {
                Ok(prepared) => {
                    let tx_signature = prepared
                        .transaction
                        .signatures
                        .get(0)
                        .map(|sig| sig.to_string())
                        .unwrap_or_default();
                    let deadline = Deadline::from_instant(Instant::now() + landing_timeout);
                    match lander_stack.submit(&prepared, deadline, "copy").await {
                        Ok(receipt) => {
                            info!(
                                target: "strategy::copy",
                                slot = receipt.slot,
                                signature = receipt.signature.as_deref().unwrap_or(&tx_signature),
                                "copy 策略 transaction 已发送"
                            );
                        }
                        Err(err) => {
                            let summary = err.to_string();
                            let short = summary.lines().next().unwrap_or_else(|| summary.as_str());
                            warn!(
                                target: "strategy::copy",
                                signature = %tx_signature,
                                error = %short,
                                "copy 策略落地失败"
                            );
                        }
                    }
                }
                Err(err) => {
                    let summary = err.to_string();
                    let short = summary.lines().next().unwrap_or_else(|| summary.as_str());
                    warn!(
                        target: "strategy::copy",
                        error = %short,
                        "构建 copy 策略交易失败"
                    );
                }
            }
        }

        plans[idx].next_ready_at = Instant::now() + sending_cooldown;
        sleep(process_delay).await;
        idx = (idx + 1) % plans.len();
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

    Ok(pairs_set
        .into_iter()
        .map(
            |(input_mint, output_mint)| crate::strategy::types::TradePair {
                input_mint,
                output_mint,
            },
        )
        .collect())
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
    request_defaults: &RequestParamsConfig,
) -> QuoteConfig {
    let only_direct_routes = if config.base_mints.iter().any(|mint| {
        mint.route_types
            .iter()
            .any(|t| t.eq_ignore_ascii_case("2hop"))
    }) {
        true
    } else {
        request_defaults.only_direct_routes
    };

    QuoteConfig {
        slippage_bps: 0,
        only_direct_routes,
        restrict_intermediate_tokens: request_defaults.restrict_intermediate_tokens,
        quote_max_accounts: None,
        dex_whitelist: config.enable_dexs.clone(),
    }
}

struct CopyPlanState {
    amount_in: u64,
    reverse_amount: u64,
    process_delay: Duration,
    sending_cooldown: Duration,
    next_ready_at: Instant,
}

fn build_copy_plan_states(config: &config::CopyStrategyConfig) -> EngineResult<Vec<CopyPlanState>> {
    let mut plans = Vec::new();
    for base in &config.base_mints {
        let reverse_amount = base.reverse_amount.unwrap_or(200_000_000_000);
        let process_delay = Duration::from_millis(base.process_delay.unwrap_or(1_000).max(1));
        let sending_cooldown = Duration::from_millis(base.sending_cooldown.unwrap_or(1_000).max(1));
        for amount in generate_amounts_for_copy(base) {
            plans.push(CopyPlanState {
                amount_in: amount,
                reverse_amount,
                process_delay,
                sending_cooldown,
                next_ready_at: Instant::now(),
            });
        }
    }

    if plans.is_empty() {
        return Err(EngineError::InvalidConfig(
            "copy_strategy 未配置有效的交易规模".into(),
        ));
    }

    Ok(plans)
}

fn generate_amounts_for_copy(base: &config::CopyBaseMintConfig) -> Vec<u64> {
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

fn resolve_priority_fee_micro_lamports(
    lander: &config::LanderSettings,
    compute_unit_limit: u32,
) -> u64 {
    const DEFAULT_PRICE: u64 = 413;
    if compute_unit_limit == 0 {
        return DEFAULT_PRICE;
    }

    let priority_lamports = if let Some(fixed) = lander.fixed_priority_fee {
        Some(fixed)
    } else if let Some(&value) = lander.random_priority_fee_range.first() {
        Some(value)
    } else {
        None
    };

    priority_lamports
        .and_then(|fee| {
            fee.checked_mul(1_000_000)
                .map(|micro| micro / u64::from(compute_unit_limit.max(1)))
        })
        .map(|price| price.max(1))
        .unwrap_or(DEFAULT_PRICE)
}

fn resolve_landing_timeout(bot: &config::BotConfig) -> Duration {
    let ms = bot.landing_timeout_ms.unwrap_or(2_000).max(1);
    Duration::from_millis(ms)
}
