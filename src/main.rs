use std::{
    collections::BTreeSet,
    env, fs,
    net::IpAddr,
    path::PathBuf,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow};
use clap::{Args, Parser, Subcommand};
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};

mod api;
mod config;
mod engine;
mod jupiter;
mod lander;
mod monitoring;
mod strategy;

use api::{
    ComputeUnitPriceMicroLamports, JupiterApiClient, QuoteRequest, SwapInstructionsRequest,
    SwapInstructionsResponse,
};
use config::{
    AppConfig, BotConfig, ConfigError, GlobalConfig, IntermediumConfig, JupiterConfig,
    LaunchOverrides, RequestParamsConfig, YellowstoneConfig, load_config,
};
use engine::{
    BuilderConfig, EngineError, EngineIdentity, EngineResult, EngineSettings, ProfitConfig,
    ProfitEvaluator, QuoteConfig, QuoteExecutor, Scheduler, StrategyEngine, SwapInstructionFetcher,
    TipConfig, TransactionBuilder,
};
use jupiter::{BinaryStatus, JupiterBinaryManager, JupiterError};
use lander::{Deadline, LanderFactory};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use strategy::{
    BlindStrategy, CopyStrategy, CopySwapParams, Strategy, StrategyEvent,
    compute_associated_token_address, usdc_mint, wsol_mint,
};

#[derive(Parser, Debug)]
#[command(name = "galileo", version, about = "Jupiter 自托管调度机器人")]
struct Cli {
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "配置文件路径（默认查找 galileo.yaml 或 config/galileo.yaml）"
    )]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Jupiter 二进制管理相关命令
    #[command(subcommand)]
    Jupiter(JupiterCmd),
    /// Lander 工具
    #[command(subcommand)]
    Lander(LanderCmd),
    /// 请求 Jupiter API 报价
    Quote(QuoteCmd),
    #[command(name = "swap-instructions")]
    /// 请求 Jupiter API Swap 指令
    SwapInstructions(SwapInstructionsCmd),
    /// 运行已配置的套利策略循环
    Strategy,
    /// 运行套利策略（dry-run 模式）
    #[command(name = "dry-run")]
    StrategyDryRun,
    /// 初始化配置模版文件
    Init(InitCmd),
}

#[derive(Args, Debug)]
struct QuoteCmd {
    #[arg(long, help = "输入代币的 Mint 地址")]
    input: String,
    #[arg(long, help = "输出代币的 Mint 地址")]
    output: String,
    #[arg(long, help = "交易数量（原始单位，lamports/atoms）")]
    amount: u64,
    #[arg(long, default_value_t = 50, help = "允许滑点（基点）")]
    slippage_bps: u16,
    #[arg(long, help = "仅限一跳直连路线")]
    direct_only: bool,
    #[arg(long, help = "允许中间代币（对应关闭 restrictIntermediateTokens）")]
    allow_intermediate: bool,
    #[arg(
        long = "extra",
        value_parser = parse_key_val,
        help = "附加查询参数，格式为 key=value",
        value_name = "KEY=VALUE"
    )]
    extra: Vec<(String, String)>,
}

#[derive(Args, Debug)]
struct SwapInstructionsCmd {
    #[arg(long, help = "包含 quoteResponse 的 JSON 文件路径")]
    quote_path: PathBuf,
    #[arg(long, help = "发起 Swap 的用户公钥")]
    user: String,
    #[arg(long, help = "是否自动 wrap/unwrap SOL")]
    wrap_sol: bool,
    #[arg(long, help = "是否使用 Jupiter 共享账户")]
    shared_accounts: bool,
    #[arg(long, help = "可选的手续费账户")]
    fee_account: Option<String>,
    #[arg(long, help = "优先费（微 lamports）")]
    compute_unit_price: Option<u64>,
}

#[derive(Args, Debug)]
struct InitCmd {
    #[arg(long, value_name = "DIR", help = "可选输出目录（默认当前目录）")]
    output: Option<PathBuf>,
    #[arg(long, help = "若文件存在则覆盖")]
    force: bool,
}

#[derive(Subcommand, Debug)]
enum LanderCmd {
    /// 使用指定落地器直接发送交易
    #[command(name = "send")]
    Send(LanderSendArgs),
}

#[derive(Args, Debug)]
struct LanderSendArgs {
    #[arg(
        long,
        value_name = "FILE",
        help = "包含 SwapInstructionsResponse 的 JSON 文件"
    )]
    instructions: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        help = "优先测试的落地器列表，逗号分隔；默认为配置文件中的 enable_landers"
    )]
    landers: Vec<String>,
    #[arg(
        long,
        default_value_t = 5_000u64,
        help = "提交截止时间（毫秒），默认 5000"
    )]
    deadline_ms: u64,
    #[arg(long, default_value_t = 0u64, help = "为交易附加的小费（lamports）")]
    tip_lamports: u64,
}

async fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = load_configuration(cli.config.clone())?;
    init_tracing(&config.galileo.global.logging)?;

    if config.galileo.bot.prometheus.enable {
        monitoring::try_init_prometheus(&config.galileo.bot.prometheus.listen)
            .map_err(|err| anyhow!(err))?;
    }

    let jupiter_cfg = resolve_jupiter_defaults(config.jupiter.clone(), &config.galileo.global)?;
    let launch_overrides =
        build_launch_overrides(&config.galileo.request_params, &config.galileo.intermedium);
    let base_url = resolve_jupiter_base_url(&config.galileo.bot, &jupiter_cfg);
    let bypass_proxy = should_bypass_proxy(&base_url);
    if bypass_proxy {
        info!(
            target: "jupiter",
            base_url = %base_url,
            "Jupiter API 请求绕过 HTTP 代理"
        );
    }
    let mut api_http_builder =
        reqwest::Client::builder().user_agent(crate::jupiter::updater::USER_AGENT);
    if bypass_proxy {
        api_http_builder = api_http_builder.no_proxy();
    }
    let api_http_client = api_http_builder.build()?;
    let manager = JupiterBinaryManager::new(
        jupiter_cfg,
        launch_overrides,
        config.galileo.bot.disable_local_binary,
        config.galileo.bot.show_jupiter_logs,
    )?;
    let api_client = JupiterApiClient::new(api_http_client, base_url, &config.galileo.bot);

    match cli.command {
        Command::Jupiter(cmd) => {
            handle_jupiter_cmd(cmd, &manager).await?;
        }
        Command::Lander(cmd) => {
            handle_lander_cmd(
                cmd,
                &config,
                &config.lander.lander,
                resolve_instruction_memo(&config.galileo.global.instruction),
            )
            .await?;
        }
        Command::Quote(args) => {
            ensure_running(&manager).await?;
            let input = Pubkey::from_str(&args.input)
                .map_err(|err| anyhow!("输入代币 Mint 无效 {}: {err}", args.input))?;
            let output = Pubkey::from_str(&args.output)
                .map_err(|err| anyhow!("输出代币 Mint 无效 {}: {err}", args.output))?;
            let mut request = QuoteRequest::new(input, output, args.amount, args.slippage_bps);
            request.only_direct_routes = Some(args.direct_only);
            request.restrict_intermediate_tokens = Some(!args.allow_intermediate);
            for (k, v) in args.extra {
                request.extra_query_params.insert(k, v);
            }
            apply_request_defaults_to_quote(&mut request, &config.galileo.request_params);

            let quote = api_client.quote(&request).await?;
            println!("{}", serde_json::to_string_pretty(&quote.raw)?);
        }
        Command::SwapInstructions(args) => {
            ensure_running(&manager).await?;
            let quote_raw = tokio::fs::read_to_string(&args.quote_path).await?;
            let quote_value: serde_json::Value = serde_json::from_str(&quote_raw)?;
            let user = Pubkey::from_str(&args.user)
                .map_err(|err| anyhow!("用户公钥无效 {}: {err}", args.user))?;
            let mut request = SwapInstructionsRequest::new(quote_value, user);
            request.config.wrap_and_unwrap_sol = args.wrap_sol;
            request.config.use_shared_accounts = Some(args.shared_accounts);
            if let Some(fee) = args.fee_account {
                let fee_pubkey = Pubkey::from_str(&fee)
                    .map_err(|err| anyhow!("手续费账户无效 {}: {err}", fee))?;
                request.config.fee_account = Some(fee_pubkey);
            }
            if let Some(price) = args.compute_unit_price {
                request.config.compute_unit_price_micro_lamports =
                    Some(ComputeUnitPriceMicroLamports::MicroLamports(price));
            }

            let instructions = api_client.swap_instructions(&request).await?;
            println!("{}", serde_json::to_string_pretty(&instructions.raw)?);
        }
        Command::Strategy | Command::StrategyDryRun => {
            let dry_run =
                matches!(cli.command, Command::StrategyDryRun) || config.galileo.bot.dry_run;

            if config.galileo.copy_strategy.enable {
                run_copy_strategy(&config, dry_run).await?;
                return Ok(());
            }

            let blind_config = &config.galileo.blind_strategy;
            if !blind_config.enable {
                warn!(target: "strategy", "盲发策略未启用，直接退出");
                return Ok(());
            }

            if !manager.disable_local_binary {
                match manager.start(false).await {
                    Ok(()) => {
                        info!(target: "strategy", "已启动本地 Jupiter 二进制");
                    }
                    Err(JupiterError::AlreadyRunning) => {
                        info!(target: "strategy", "本地 Jupiter 二进制已在运行");
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
            let identity = EngineIdentity::from_wallet(&config.galileo.global.wallet)
                .map_err(|err| anyhow!(err))?;

            let trade_pairs = build_blind_trade_pairs(blind_config, &config.galileo.intermedium)?;
            let trade_amounts = build_blind_trade_amounts(blind_config)?;
            let profit_config = build_blind_profit_config(blind_config);
            let scheduler_delay = resolve_blind_scheduler_delay(blind_config);
            let quote_config =
                build_blind_quote_config(blind_config, &config.galileo.request_params);
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

            let engine = StrategyEngine::new(
                BlindStrategy::new(),
                lander_stack,
                identity,
                QuoteExecutor::new(api_client.clone(), request_defaults),
                ProfitEvaluator::new(profit_config),
                SwapInstructionFetcher::new(api_client),
                TransactionBuilder::new(rpc_client.clone(), builder_config),
                Scheduler::new(scheduler_delay),
                EngineSettings::new(quote_config)
                    .with_landing_timeout(landing_timeout)
                    .with_dry_run(dry_run),
                trade_pairs,
                trade_amounts,
            );
            drive_engine(engine).await.map_err(|err| anyhow!(err))?;

            if !manager.disable_local_binary {
                if let Err(err) = manager.stop().await {
                    warn!(
                        target: "strategy",
                        error = %err,
                        "停止 Jupiter 二进制失败"
                    );
                }
            }
        }
        Command::Init(args) => {
            init_configs(args)?;
        }
    }

    Ok(())
}

async fn run_copy_strategy(config: &AppConfig, dry_run: bool) -> Result<()> {
    use tokio::time::sleep;

    let copy_cfg = &config.galileo.copy_strategy;
    let mut plans = build_copy_plan_states(copy_cfg).map_err(|err| anyhow!(err))?;

    let rpc_client = resolve_rpc_client(&config.galileo.global)?;
    let identity =
        EngineIdentity::from_wallet(&config.galileo.global.wallet).map_err(|err| anyhow!(err))?;

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

#[cfg(any(
    feature = "hotpath-alloc-bytes-total",
    feature = "hotpath-alloc-count-total"
))]
#[tokio::main(flavor = "current_thread")]
#[cfg_attr(feature = "hotpath", hotpath::main(percentiles = [95, 99]))]
async fn main() -> Result<()> {
    run().await
}

#[cfg(not(any(
    feature = "hotpath-alloc-bytes-total",
    feature = "hotpath-alloc-count-total"
)))]
#[tokio::main]
#[cfg_attr(feature = "hotpath", hotpath::main(percentiles = [95, 99]))]
async fn main() -> Result<()> {
    run().await
}

fn init_tracing(config: &config::LoggingConfig) -> Result<()> {
    let filter = EnvFilter::try_new(&config.level).unwrap_or_else(|_| EnvFilter::new("info"));

    if config.json {
        fmt()
            .with_env_filter(filter)
            .json()
            .with_current_span(false)
            .with_span_list(false)
            .init();
    } else {
        fmt().with_env_filter(filter).init();
    }
    Ok(())
}

fn load_configuration(path: Option<PathBuf>) -> Result<AppConfig, ConfigError> {
    load_config(path)
}

async fn ensure_running(manager: &JupiterBinaryManager) -> Result<(), JupiterError> {
    if manager.disable_local_binary {
        return Ok(());
    }
    match manager.status().await {
        BinaryStatus::Running => Ok(()),
        status => {
            error!(
                target: "jupiter",
                ?status,
                "Jupiter 二进制未运行，请先执行 `galileo jupiter start`"
            );
            Err(JupiterError::Schema(format!(
                "二进制未运行，当前状态: {status:?}"
            )))
        }
    }
}

fn parse_key_val(s: &str) -> std::result::Result<(String, String), String> {
    let pos = s
        .find('=')
        .ok_or_else(|| "参数格式需为 key=value".to_string())?;
    Ok((s[..pos].to_string(), s[pos + 1..].to_string()))
}

fn apply_request_defaults_to_quote(request: &mut QuoteRequest, params: &RequestParamsConfig) {
    if !request.only_direct_routes.unwrap_or(false) && params.only_direct_routes {
        request.only_direct_routes = Some(true);
    }

    if request.restrict_intermediate_tokens.unwrap_or(true) && !params.restrict_intermediate_tokens
    {
        request.restrict_intermediate_tokens = Some(false);
    }
}

fn resolve_jupiter_base_url(_bot: &BotConfig, jupiter: &JupiterConfig) -> String {
    if let Ok(url) = std::env::var("JUPITER_URL") {
        let trimmed = url.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    let host = sanitize_jupiter_host(&jupiter.core.host);
    format!("http://{}:{}", host, jupiter.core.port)
}

fn sanitize_jupiter_host(host: &str) -> String {
    let trimmed = host.trim();
    if trimmed.is_empty() {
        return "127.0.0.1".to_string();
    }
    if let Ok(ip) = trimmed.parse::<IpAddr>() {
        if ip.is_unspecified() {
            return match ip {
                IpAddr::V4(_) => "127.0.0.1".to_string(),
                IpAddr::V6(_) => "::1".to_string(),
            };
        }
    }
    trimmed.to_string()
}

fn should_bypass_proxy(base_url: &str) -> bool {
    if let Ok(url) = reqwest::Url::parse(base_url) {
        if let Some(host) = url.host_str() {
            if host.eq_ignore_ascii_case("localhost") {
                return true;
            }
            if let Ok(ip) = host.parse::<IpAddr>() {
                return ip.is_loopback() || ip.is_unspecified();
            }
        }
    }
    false
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

async fn handle_lander_cmd(
    cmd: LanderCmd,
    config: &AppConfig,
    lander_settings: &config::LanderSettings,
    memo: Option<String>,
) -> Result<()> {
    match cmd {
        LanderCmd::Send(args) => {
            let rpc_client = resolve_rpc_client(&config.galileo.global)?;
            let identity = EngineIdentity::from_wallet(&config.galileo.global.wallet)
                .map_err(|err| anyhow!(err))?;

            let builder_config = BuilderConfig::new(memo);
            let builder = TransactionBuilder::new(rpc_client.clone(), builder_config);

            let submission_client = reqwest::Client::builder().build()?;
            let lander_factory = LanderFactory::new(rpc_client.clone(), submission_client);

            let preferred: Vec<String> = if !args.landers.is_empty() {
                args.landers.clone()
            } else {
                config.galileo.blind_strategy.enable_landers.clone()
            };
            let default_landers = ["rpc"];
            let lander_stack = lander_factory
                .build_stack(lander_settings, preferred.as_slice(), &default_landers, 0)
                .map_err(|err| anyhow!(err))?;

            let raw = tokio::fs::read_to_string(&args.instructions).await?;
            let value: serde_json::Value = serde_json::from_str(&raw)?;
            let instructions = SwapInstructionsResponse::try_from(value)
                .map_err(|err| anyhow!("解析 Swap 指令失败: {err}"))?;

            let prepared = builder
                .build(&identity, &instructions, args.tip_lamports)
                .await
                .map_err(|err| anyhow!(err))?;

            let deadline = Deadline::from_instant(
                Instant::now() + Duration::from_millis(args.deadline_ms.max(1)),
            );
            let receipt = lander_stack
                .submit(&prepared, deadline, "lander-test")
                .await
                .map_err(|err| anyhow!(err))?;

            info!(
                target: "lander::cli",
                lander = receipt.lander,
                endpoint = %receipt.endpoint,
                slot = receipt.slot,
                blockhash = %receipt.blockhash,
                signature = receipt.signature.as_deref().unwrap_or("")
            );
            if let Some(signature) = receipt.signature {
                println!("{signature}");
            }
        }
    }

    Ok(())
}

fn build_blind_trade_pairs(
    config: &config::BlindStrategyConfig,
    intermedium: &IntermediumConfig,
) -> EngineResult<Vec<strategy::types::TradePair>> {
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
        .map(|(input_mint, output_mint)| strategy::types::TradePair {
            input_mint,
            output_mint,
        })
        .collect())
}

fn build_blind_trade_amounts(config: &config::BlindStrategyConfig) -> EngineResult<Vec<u64>> {
    let mut amounts: BTreeSet<u64> = BTreeSet::new();
    for base in &config.base_mints {
        for value in generate_amounts_for_base(base) {
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

fn resolve_instruction_memo(instruction: &config::InstructionConfig) -> Option<String> {
    let memo = instruction.memo.trim();
    if memo.is_empty() {
        None
    } else {
        Some(memo.to_string())
    }
}

fn resolve_rpc_client(global: &GlobalConfig) -> Result<Arc<RpcClient>> {
    let url = env::var("GALILEO_RPC_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            global
                .rpc_url
                .clone()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "https://api.mainnet-beta.solana.com".to_string());

    Ok(Arc::new(RpcClient::new(url)))
}

fn build_launch_overrides(
    params: &RequestParamsConfig,
    intermedium: &IntermediumConfig,
) -> LaunchOverrides {
    let mut overrides = LaunchOverrides::default();

    let mut mint_set: BTreeSet<String> = intermedium.mints.iter().cloned().collect();
    for mint in &intermedium.disable_mints {
        mint_set.remove(mint);
    }

    if !mint_set.is_empty() {
        overrides.filter_markets_with_mints = mint_set.into_iter().collect();
    }

    overrides.exclude_dex_program_ids = params.excluded_dexes.clone();
    overrides.include_dex_program_ids = params.included_dexes.clone();

    overrides
}

fn resolve_jupiter_defaults(
    mut jupiter: JupiterConfig,
    global: &GlobalConfig,
) -> Result<JupiterConfig> {
    if jupiter.core.rpc_url.trim().is_empty() {
        if let Some(global_rpc) = &global.rpc_url {
            let trimmed = global_rpc.trim();
            if !trimmed.is_empty() {
                jupiter.core.rpc_url = trimmed.to_string();
            }
        }
    }

    if jupiter.core.rpc_url.trim().is_empty() {
        return Err(anyhow!(
            "未配置 Jupiter RPC：请在 jupiter.toml 或 galileo.yaml 的 global.rpc_url 中设置 rpc_url"
        ));
    }

    let needs_yellowstone = jupiter
        .launch
        .yellowstone
        .as_ref()
        .map(|cfg| cfg.endpoint.trim().is_empty())
        .unwrap_or(true);

    if needs_yellowstone {
        if let Some(endpoint) = &global.yellowstone_grpc_url {
            let trimmed = endpoint.trim();
            if !trimmed.is_empty() {
                let token = global.yellowstone_grpc_token.as_ref().and_then(|t| {
                    let tt = t.trim();
                    if tt.is_empty() {
                        None
                    } else {
                        Some(tt.to_string())
                    }
                });
                jupiter.launch.yellowstone = Some(YellowstoneConfig {
                    endpoint: trimmed.to_string(),
                    x_token: token,
                });
            }
        }
    }

    Ok(jupiter)
}

fn status_indicator(status: BinaryStatus) -> (&'static str, &'static str) {
    match status {
        BinaryStatus::Running => ("🚀", "运行中"),
        BinaryStatus::Starting => ("⏳", "启动中"),
        BinaryStatus::Updating => ("⬇️", "更新中"),
        BinaryStatus::Stopping => ("🛑", "停止中"),
        BinaryStatus::Stopped => ("⛔", "已停止"),
        BinaryStatus::Failed => ("⚠️", "失败"),
    }
}

fn init_configs(args: InitCmd) -> Result<()> {
    let output_dir = match args.output {
        Some(dir) => dir,
        None => std::env::current_dir()?,
    };

    fs::create_dir_all(&output_dir)?;

    let templates: [(&str, &str); 3] = [
        (
            "galileo.yaml",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/galileo.yaml")),
        ),
        (
            "lander.yaml",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/lander.yaml")),
        ),
        (
            "jupiter.toml",
            include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/jupiter.toml")),
        ),
    ];

    for (filename, contents) in templates {
        let target_path = output_dir.join(filename);
        if target_path.exists() && !args.force {
            println!(
                "跳过 {}（文件已存在，如需覆盖请加 --force）",
                target_path.display()
            );
            continue;
        }

        fs::write(&target_path, contents)?;
        println!("已写入 {}", target_path.display());
    }

    Ok(())
}

#[derive(Subcommand, Debug)]
enum JupiterCmd {
    /// 启动 Jupiter 自托管二进制（可选强制更新）
    Start {
        #[arg(long, help = "启动前强制更新二进制")]
        force_update: bool,
    },
    /// 停止已运行的 Jupiter 二进制
    Stop,
    /// 重启 Jupiter 二进制
    Restart,
    /// 下载并安装最新 Jupiter 二进制
    Update {
        #[arg(
            short = 'v',
            long,
            value_name = "TAG",
            help = "指定版本 tag，缺省为最新版本"
        )]
        version: Option<String>,
    },
    /// 查看当前二进制状态
    Status,
    /// 列出最近可用版本
    List {
        #[arg(long, default_value_t = 5, help = "展示最近的版本数量")]
        limit: usize,
    },
}

async fn handle_jupiter_cmd(cmd: JupiterCmd, manager: &JupiterBinaryManager) -> Result<()> {
    match cmd {
        JupiterCmd::Start { force_update } => {
            manager.start(force_update).await?;
            if manager.disable_local_binary {
                info!(
                    target: "jupiter",
                    "本地 Jupiter 二进制已禁用，start 命令仅用于远端模式，直接返回"
                );
                return Ok(());
            }

            info!(
                target: "jupiter",
                "Jupiter 二进制已启动，按 Ctrl+C 停止并退出前台日志"
            );
            tokio::signal::ctrl_c()
                .await
                .map_err(|err| anyhow!("捕获 Ctrl+C 失败: {err}"))?;
            info!(target: "jupiter", "收到终止信号，正在停止 Jupiter 二进制…");
            manager.stop().await?;
        }
        JupiterCmd::Stop => {
            manager.stop().await?;
        }
        JupiterCmd::Restart => {
            manager.restart().await?;
        }
        JupiterCmd::Update { version } => {
            manager.update(version.as_deref()).await?;
        }
        JupiterCmd::Status => {
            if manager.disable_local_binary {
                println!("status: 🚫 已禁用本地 Jupiter（二进制不运行，使用远程 API）");
            } else {
                let status = manager.status().await;
                let (emoji, label) = status_indicator(status);
                println!("status: {emoji} {label} ({status:?})");
                let binary_path = manager.config.binary_path();
                println!("binary: {binary_path}", binary_path = binary_path.display());

                match manager.installed_version().await {
                    Ok(Some(version)) => println!("version: 🎯 {version}"),
                    Ok(None) => println!("version: ❔ 未检测到已安装的二进制"),
                    Err(err) => println!("version: ⚠️ 获取失败：{err}"),
                };
            }
        }
        JupiterCmd::List { limit } => {
            let releases = manager.list_releases(limit).await?;
            for (idx, release) in releases.iter().enumerate() {
                println!("{:<2} {}", idx + 1, release.tag_name);
            }
        }
    }
    Ok(())
}
