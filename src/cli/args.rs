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
    /// Jupiter 二进制管理命令
    #[command(subcommand)]
    Jupiter(JupiterCmd),
    /// 钱包 / 工具类命令
    #[command(subcommand)]
    Tools(ToolsCmd),
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
pub enum WalletCmd {
    /// 交互式添加钱包私钥
    #[command(name = "add")]
    Add(WalletAddArgs),
}

#[derive(Args, Debug, Default)]
pub struct WalletAddArgs {}

#[derive(Subcommand, Debug, Clone)]
pub enum ToolsCmd {
    /// 打开交互式工具面板
    #[command(name = "interactive", alias = "tui")]
    Interactive,
}

#[derive(Subcommand, Debug, Clone)]
pub enum JupiterCmd {
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
