use std::{collections::BTreeSet, env, fs, path::PathBuf, str::FromStr, sync::Arc, time::Duration};

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

use api::{ComputeUnitPriceMicroLamports, JupiterApiClient, QuoteRequest, SwapInstructionsRequest};
use config::{
    AppConfig, BotConfig, ConfigError, GlobalConfig, IntermediumConfig, JupiterConfig,
    LaunchOverrides, RequestParamsConfig, YellowstoneConfig, load_config,
};
use engine::{
    BuilderConfig, EngineError, EngineIdentity, EngineResult, EngineSettings, ProfitConfig,
    ProfitEvaluator, QuoteConfig, QuoteExecutor, Scheduler, StrategyEngine, SwapInstructionFetcher,
    TransactionBuilder,
};
use jupiter::{BinaryStatus, JupiterBinaryManager, JupiterError};
use lander::LanderFactory;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use strategy::blind::BlindStrategy;
use strategy::config::StrategyConfig;
use strategy::spam::SpamStrategy;
use strategy::{Strategy, StrategyEvent};

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

#[derive(Debug, Clone, Copy)]
enum StrategyMode {
    Spam,
    Blind,
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

    let manager = JupiterBinaryManager::new(
        jupiter_cfg,
        launch_overrides,
        config.galileo.bot.disable_local_binary,
        config.galileo.bot.show_jupiter_logs,
    )?;
    let api_client = JupiterApiClient::new(manager.client.clone(), base_url, &config.galileo.bot);

    match cli.command {
        Command::Jupiter(cmd) => {
            handle_jupiter_cmd(cmd, &manager).await?;
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
            let strategy_config = match config.galileo.strategy.clone() {
                Some(cfg) => cfg,
                None => return Err(anyhow!("未提供策略配置")),
            };

            if !strategy_config.is_enabled() {
                warn!(
                    target: "strategy",
                    "策略在配置中被禁用，直接退出"
                );
                return Ok(());
            }

            let dry_run =
                matches!(cli.command, Command::StrategyDryRun) || config.galileo.bot.dry_run;

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

            let mode = resolve_strategy_mode(&config.galileo);
            let rpc_client = resolve_rpc_client(&config.galileo.global)?;
            let identity = EngineIdentity::from_wallet(&config.galileo.global.wallet)
                .map_err(|err| anyhow!(err))?;

            let trade_pairs = resolve_trade_pairs(&strategy_config)?;
            let trade_amounts = resolve_trade_amounts(&strategy_config)?;
            let profit_config = build_profit_config(&strategy_config);
            let scheduler_delay = strategy_config.trade_delay();
            let quote_config = build_quote_config(
                &strategy_config,
                mode,
                &config.galileo.request_params,
                &config.galileo.spam,
                &config.galileo.blind,
            );
            let compute_unit =
                compute_unit_override(mode, &config.galileo.spam, &config.galileo.blind);
            let landing_timeout = default_landing_timeout();
            let builder_config =
                BuilderConfig::new(resolve_instruction_memo(&config.galileo.global.instruction));
            let request_defaults = config.galileo.request_params.clone();
            let submission_client = reqwest::Client::builder().build()?;

            let lander_factory = LanderFactory::new(rpc_client.clone(), submission_client.clone());
            let default_landers = ["rpc"];

            let result = match mode {
                StrategyMode::Spam => {
                    let lander_stack = lander_factory
                        .build_stack(
                            &config.lander.lander,
                            &config.galileo.spam.enable_landers,
                            &default_landers,
                            config.galileo.spam.max_retries as usize,
                        )
                        .map_err(|err| anyhow!(err))?;

                    let engine = StrategyEngine::new(
                        SpamStrategy::new(),
                        lander_stack,
                        identity.clone(),
                        QuoteExecutor::new(api_client.clone(), request_defaults.clone()),
                        ProfitEvaluator::new(profit_config.clone()),
                        SwapInstructionFetcher::new(api_client.clone()),
                        TransactionBuilder::new(rpc_client.clone(), builder_config.clone()),
                        Scheduler::new(scheduler_delay),
                        EngineSettings::new(quote_config.clone())
                            .with_landing_timeout(landing_timeout)
                            .with_compute_unit_override(compute_unit)
                            .with_dry_run(dry_run),
                        trade_pairs.clone(),
                        trade_amounts.clone(),
                    );
                    drive_engine(engine).await
                }
                StrategyMode::Blind => {
                    let lander_stack = lander_factory
                        .build_stack(
                            &config.lander.lander,
                            &config.galileo.blind.enable_landers,
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
                            .with_compute_unit_override(compute_unit)
                            .with_dry_run(dry_run),
                        trade_pairs,
                        trade_amounts,
                    );
                    drive_engine(engine).await
                }
            };

            result.map_err(|err| anyhow!(err))?;

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
    if request.dexes.is_none() && !params.included_dexes.is_empty() {
        let dexes = params.included_dexes.join(",");
        request.dexes = Some(dexes.clone());
        request
            .extra_query_params
            .entry("onlyDexes".to_string())
            .or_insert(dexes);
    }

    if request.excluded_dexes.is_none() && !params.excluded_dexes.is_empty() {
        let dexes = params.excluded_dexes.join(",");
        request.excluded_dexes = Some(dexes.clone());
        request
            .extra_query_params
            .entry("excludeDexes".to_string())
            .or_insert(dexes);
    }

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

    format!("http://{}:{}", jupiter.core.host, jupiter.core.port)
}

fn resolve_strategy_mode(config: &config::GalileoConfig) -> StrategyMode {
    if let Ok(value) = env::var("GALILEO_STRATEGY_MODE") {
        match value.trim().to_ascii_lowercase().as_str() {
            "spam" => return StrategyMode::Spam,
            "blind" => return StrategyMode::Blind,
            other => {
                warn!(target: "strategy", mode = other, "未知 GALILEO_STRATEGY_MODE，按配置回退")
            }
        }
    }

    if config.spam.enable {
        StrategyMode::Spam
    } else if config.blind.enable {
        StrategyMode::Blind
    } else {
        warn!(target: "strategy", "未启用 spam/blind，默认使用 blind" );
        StrategyMode::Blind
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

fn resolve_trade_pairs(config: &StrategyConfig) -> EngineResult<Vec<strategy::types::TradePair>> {
    let mut pairs = config.resolved_pairs();
    if pairs.is_empty() {
        return Err(EngineError::InvalidConfig(
            "trade pairs or quote mints missing".into(),
        ));
    }

    if config.controls.enable_reverse_trade {
        let mut reversed: Vec<_> = pairs.iter().map(|pair| pair.reversed()).collect();
        pairs.append(&mut reversed);
    }

    Ok(pairs)
}

fn resolve_trade_amounts(config: &StrategyConfig) -> EngineResult<Vec<u64>> {
    let amounts = config.effective_trade_amounts();
    if amounts.is_empty() {
        return Err(EngineError::InvalidConfig(
            "trade_range produced no amounts".into(),
        ));
    }
    Ok(amounts)
}

fn build_profit_config(config: &StrategyConfig) -> ProfitConfig {
    ProfitConfig {
        min_profit_threshold_lamports: config.min_profit_threshold_lamports,
        max_tip_lamports: config.max_tip_lamports,
        tip: config.controls.static_tip_config.clone(),
    }
}

fn build_quote_config(
    config: &StrategyConfig,
    mode: StrategyMode,
    request_defaults: &RequestParamsConfig,
    spam_config: &config::SpamConfig,
    blind_config: &config::BlindConfig,
) -> QuoteConfig {
    let mut only_direct_routes = config.only_direct_routes;
    if request_defaults.only_direct_routes || matches!(mode, StrategyMode::Spam) {
        only_direct_routes = true;
    }

    let restrict_intermediate_tokens =
        if !request_defaults.restrict_intermediate_tokens && !matches!(mode, StrategyMode::Spam) {
            false
        } else {
            config.restrict_intermediate_tokens
        };

    let mut dex_whitelist = if !config.controls.only_quote_dexs.is_empty() {
        config.controls.only_quote_dexs.clone()
    } else if !request_defaults.included_dexes.is_empty() {
        request_defaults.included_dexes.clone()
    } else {
        Vec::new()
    };

    match mode {
        StrategyMode::Spam if !spam_config.enable_dexs.is_empty() => {
            dex_whitelist = spam_config.enable_dexs.clone();
        }
        StrategyMode::Blind if !blind_config.enable_dexs.is_empty() => {
            dex_whitelist = blind_config.enable_dexs.clone();
        }
        _ => {}
    }

    QuoteConfig {
        slippage_bps: config.slippage_bps,
        only_direct_routes,
        restrict_intermediate_tokens,
        quote_max_accounts: config.quote_max_accounts,
        dex_whitelist,
    }
}

fn compute_unit_override(
    mode: StrategyMode,
    spam_config: &config::SpamConfig,
    _blind_config: &config::BlindConfig,
) -> Option<u64> {
    match mode {
        StrategyMode::Spam => {
            let price = spam_config.compute_unit_price_micro_lamports;
            (price > 0).then_some(price)
        }
        StrategyMode::Blind => None,
    }
}

fn default_landing_timeout() -> Duration {
    Duration::from_secs(2)
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
