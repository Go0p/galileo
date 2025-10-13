use std::{collections::BTreeSet, fs, path::PathBuf};

use anyhow::{Result, anyhow};
use clap::{Args, Parser, Subcommand};
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};

mod config;
mod jupiter;
mod metrics;
mod strategy;

use config::{
    AppConfig, BotConfig, ConfigError, GlobalConfig, IntermediumConfig, JupiterConfig,
    LaunchOverrides, RequestParamsConfig, YellowstoneConfig, load_config,
};
use jupiter::{
    BinaryStatus, JupiterApiClient, JupiterBinaryManager, JupiterError, QuoteRequest, SwapRequest,
};
use strategy::engine::ArbitrageEngine;

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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = load_configuration(cli.config.clone())?;
    init_tracing(&config.galileo.global.logging)?;

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
            let mut request =
                QuoteRequest::new(args.input, args.output, args.amount, args.slippage_bps);
            request.only_direct_routes = args.direct_only;
            request.restrict_intermediate_tokens = !args.allow_intermediate;
            for (k, v) in args.extra {
                request.extra.insert(k, v);
            }
            apply_request_defaults_to_quote(&mut request, &config.galileo.request_params);

            let quote = api_client.quote(&request).await?;
            println!("{}", serde_json::to_string_pretty(&quote.raw)?);
        }
        Command::SwapInstructions(args) => {
            ensure_running(&manager).await?;
            let quote_raw = tokio::fs::read_to_string(&args.quote_path).await?;
            let quote_value: serde_json::Value = serde_json::from_str(&quote_raw)?;
            let mut request = SwapRequest::new(quote_value, args.user);
            request.wrap_and_unwrap_sol = Some(args.wrap_sol);
            request.use_shared_accounts = Some(args.shared_accounts);
            request.fee_account = args.fee_account;
            request.compute_unit_price_micro_lamports = args.compute_unit_price;

            let instructions = api_client.swap_instructions(&request).await?;
            println!("{}", serde_json::to_string_pretty(&instructions.raw)?);
        }
        Command::Strategy => {
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

            let engine = ArbitrageEngine::new(
                strategy_config,
                config.galileo.bot.clone(),
                config.galileo.global.wallet.clone(),
                api_client.clone(),
                config.galileo.request_params.clone(),
            )
            .map_err(|err| anyhow!(err))?;

            tokio::select! {
                res = engine.run() => res.map_err(|err| anyhow!(err))?,
                _ = tokio::signal::ctrl_c() => {
                    info!(target: "strategy", "收到终止信号，停止运行");
                }
            }
        }
        Command::Init(args) => {
            init_configs(args)?;
        }
    }

    Ok(())
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
    if request.extra.get("onlyDexes").is_none() && !params.included_dexes.is_empty() {
        request
            .extra
            .insert("onlyDexes".to_string(), params.included_dexes.join(","));
    }

    if request.extra.get("excludeDexes").is_none() && !params.excluded_dexes.is_empty() {
        request
            .extra
            .insert("excludeDexes".to_string(), params.excluded_dexes.join(","));
    }

    if !request.only_direct_routes && params.only_direct_routes {
        request.only_direct_routes = true;
    }

    if request.restrict_intermediate_tokens && !params.restrict_intermediate_tokens {
        request.restrict_intermediate_tokens = false;
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
