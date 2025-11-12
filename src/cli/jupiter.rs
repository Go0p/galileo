use anyhow::{Result, anyhow};
use tracing::info;

use crate::cli::args::JupiterCmd;
use crate::jupiter::{BinaryStatus, JupiterBinaryManager};

/// Jupiter 子命令处理：封装二进制生命周期管理。
pub async fn handle_jupiter_cmd(cmd: JupiterCmd, manager: &JupiterBinaryManager) -> Result<()> {
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
