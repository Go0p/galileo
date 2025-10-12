use std::{fs, path::PathBuf};

use anyhow::{Result, anyhow};
use clap::{Args, Parser, Subcommand};
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, fmt};

mod config;
mod jupiter;
mod metrics;
mod strategy;

use config::{AppConfig, ConfigError, LoggingConfig, load_config};
use jupiter::{
    BinaryStatus, JupiterBinaryManager, JupiterError,
    client::{JupiterApiClient, QuoteRequest, SwapRequest},
};
use strategy::engine::ArbitrageEngine;

#[derive(Parser, Debug)]
#[command(
    name = "galileo",
    version,
    about = "Jupiter 自托管调度机器人"
)]
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
    /// 请求 Jupiter API Swap 交易
    Swap(SwapCmd),
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
    #[arg(
        long,
        default_value_t = 50,
        help = "允许滑点（基点）"
    )]
    slippage_bps: u16,
    #[arg(long, help = "仅限一跳直连路线")]
    direct_only: bool,
    #[arg(
        long,
        help = "允许中间代币（对应关闭 restrictIntermediateTokens）"
    )]
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
struct SwapCmd {
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
    #[arg(
        long,
        help = "优先费（微 lamports）"
    )]
    compute_unit_price: Option<u64>,
}

#[derive(Args, Debug)]
struct InitCmd {
    #[arg(
        long,
        value_name = "DIR",
        help = "可选输出目录（默认当前目录）"
    )]
    output: Option<PathBuf>,
    #[arg(long, help = "若文件存在则覆盖")]
    force: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = load_configuration(cli.config.clone())?;
    init_tracing(&config.galileo.logging)?;

    let manager = JupiterBinaryManager::new(config.galileo.jupiter.clone())?;
    let api_client = JupiterApiClient::new(manager.client.clone(), &config.galileo.bot);

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

            let quote = api_client.quote(&request).await?;
            println!("{}", serde_json::to_string_pretty(&quote.raw)?);
        }
        Command::Swap(args) => {
            ensure_running(&manager).await?;
            let quote_raw = tokio::fs::read_to_string(&args.quote_path).await?;
            let quote_value: serde_json::Value = serde_json::from_str(&quote_raw)?;
            let mut request = SwapRequest::new(quote_value, args.user);
            request.wrap_and_unwrap_sol = Some(args.wrap_sol);
            request.use_shared_accounts = Some(args.shared_accounts);
            request.fee_account = args.fee_account;
            request.compute_unit_price_micro_lamports = args.compute_unit_price;

            let swap = api_client.swap(&request).await?;
            println!("{}", serde_json::to_string_pretty(&swap.raw)?);
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

            if !config.galileo.jupiter.launch.disable_local_binary {
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

            let engine =
                ArbitrageEngine::new(strategy_config, config.galileo.bot.clone(), api_client.clone())
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

fn init_tracing(config: &LoggingConfig) -> Result<()> {
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
    if manager.config.launch.disable_local_binary {
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
    Update,
    /// 查看当前二进制状态
    Status,
}

async fn handle_jupiter_cmd(cmd: JupiterCmd, manager: &JupiterBinaryManager) -> Result<()> {
    match cmd {
        JupiterCmd::Start { force_update } => {
            manager.start(force_update).await?;
        }
        JupiterCmd::Stop => {
            manager.stop().await?;
        }
        JupiterCmd::Restart => {
            manager.restart().await?;
        }
        JupiterCmd::Update => {
            manager.update().await?;
        }
        JupiterCmd::Status => {
            let status = manager.status().await;
            println!("{status:?}");
        }
    }
    Ok(())
}
