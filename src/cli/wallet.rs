use std::path::PathBuf;

use anyhow::{Result, anyhow};
use serde_yaml;
use solana_sdk::signer::Signer;

use crate::cli::args::WalletCmd;
use crate::config::GalileoConfig;
use crate::config::loader::DEFAULT_CONFIG_PATHS;
use crate::config::wallet::{WalletProcessingResult, add_wallet_to_config, parse_keypair_string};

pub fn handle_wallet_command(cmd: &WalletCmd, override_path: Option<PathBuf>) -> Result<()> {
    match cmd {
        WalletCmd::Add(_args) => handle_wallet_add(override_path),
    }
}

fn handle_wallet_add(override_path: Option<PathBuf>) -> Result<()> {
    let target_path = resolve_config_path(override_path)?;
    let contents = std::fs::read_to_string(&target_path)
        .map_err(|err| anyhow!("读取配置文件失败 {}: {err}", target_path.display()))?;
    let mut config: GalileoConfig = serde_yaml::from_str(&contents)
        .map_err(|err| anyhow!("解析配置文件失败 {}: {err}", target_path.display()))?;

    let WalletProcessingResult {
        selected_remark, ..
    } = add_wallet_to_config(&mut config, Some(target_path.as_path()))
        .map_err(|err| anyhow!(err.to_string()))?;

    if let Some(remark) = selected_remark.as_ref() {
        match parse_keypair_string(config.private_key.trim()) {
            Ok(keypair) => {
                println!("🔐 已新增钱包 [{}]，公钥 {}", remark, keypair.pubkey());
            }
            Err(err) => {
                println!("🔐 已新增钱包 [{}]，但解析公钥失败: {err}", remark);
            }
        }
    }

    println!("配置文件位置：{}", target_path.display());
    println!("请重新启动 Galileo 以加载最新钱包。");
    Ok(())
}

fn resolve_config_path(override_path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = override_path {
        if path.exists() {
            return Ok(path);
        } else {
            return Err(anyhow!("指定的配置文件不存在: {}", path.display()));
        }
    }

    for candidate in DEFAULT_CONFIG_PATHS {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Ok(path);
        }
    }

    Err(anyhow!(
        "未找到配置文件，请先运行 `galileo init` 或提供 --config <FILE>"
    ))
}
