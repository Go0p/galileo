use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "galileo", version, about = "Galileo 高性能套利调度器")]
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "FILE",
        help = "配置文件路径（默认查找 galileo.yaml 或 config/galileo.yaml）"
    )]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Lander 工具
    #[command(subcommand)]
    Lander(LanderCmd),
    /// 运行已配置的套利策略循环
    #[command(name = "run", alias = "strategy")]
    Run,
    /// 运行套利策略（dry-run 模式）
    #[command(name = "dry-run")]
    StrategyDryRun,
    /// 初始化配置模版文件
    Init(InitCmd),
    /// 钱包管理
    #[command(subcommand)]
    Wallet(WalletCmd),
}

#[derive(Args, Debug)]
pub struct InitCmd {
    #[arg(long, value_name = "DIR", help = "可选输出目录（默认当前目录）")]
    pub output: Option<PathBuf>,
    #[arg(long, help = "若文件存在则覆盖")]
    pub force: bool,
}

#[derive(Subcommand, Debug)]
pub enum LanderCmd {
    /// 使用指定落地器直接发送交易
    #[command(name = "send")]
    Send(LanderSendArgs),
}

#[derive(Subcommand, Debug)]
pub enum WalletCmd {
    /// 交互式添加钱包私钥
    #[command(name = "add")]
    Add(WalletAddArgs),
}

#[derive(Args, Debug, Default)]
pub struct WalletAddArgs {}

#[derive(Args, Debug)]
pub struct LanderSendArgs {
    #[arg(
        long,
        value_name = "FILE",
        help = "包含 SwapInstructionsResponse 的 JSON 文件"
    )]
    pub instructions: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        help = "优先测试的落地器列表，逗号分隔；默认为配置文件中的 enable_landers"
    )]
    pub landers: Vec<String>,
    #[arg(
        long,
        default_value_t = 5_000u64,
        help = "提交截止时间（毫秒），默认 5000"
    )]
    pub deadline_ms: u64,
    #[arg(long, default_value_t = 0u64, help = "为交易附加的小费（lamports）")]
    pub tip_lamports: u64,
}
