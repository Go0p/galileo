use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use thiserror::Error;

use super::strategy_loader::load_strategy_configs;
use super::wallet::{parse_keypair_string, process_wallet_keys};
use super::{AppConfig, GalileoConfig, JupiterConfig, LanderConfig};
use solana_sdk::signer::Signer;
use tracing::info;

pub const DEFAULT_CONFIG_PATHS: &[&str] = &["galileo.yaml", "config/galileo.yaml"];
pub const DEFAULT_LANDER_PATHS: &[&str] = &["lander.yaml", "config/lander.yaml"];
pub const DEFAULT_JUPITER_PATHS: &[&str] = &["jupiter.toml", "config/jupiter.toml"];

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse config at {path}: {message}")]
    Parse { path: PathBuf, message: String },
}

pub fn load_config(path: Option<PathBuf>) -> Result<AppConfig, ConfigError> {
    let candidate_paths = match path {
        Some(p) => vec![p],
        None => DEFAULT_CONFIG_PATHS
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>(),
    };

    let (mut galileo, galileo_dir) = load_first_available_yaml::<GalileoConfig>(&candidate_paths)?;
    let galileo_path = first_existing_path(&candidate_paths);

    // 加载策略配置（从外部文件或使用主配置中的值）
    load_strategy_configs(&mut galileo, galileo_dir.as_deref())?;

    // 处理 wallet_keys：解密并填充 private_key
    let wallet_result = process_wallet_keys(&mut galileo, galileo_path.as_deref())?;
    if wallet_result.config_updated {
        if let Some(remark) = wallet_result.selected_remark.as_ref() {
            match parse_keypair_string(galileo.private_key.trim()) {
                Ok(keypair) => {
                    println!("🔐 已保存钱包 [{}]，公钥 {}", remark, keypair.pubkey());
                }
                Err(err) => {
                    println!("🔐 已保存钱包 [{}]，但解析公钥失败: {err}", remark);
                }
            }
        }

        if let Some(path) = galileo_path.as_ref() {
            info!(
                target: "config",
                path = %path.display(),
                "wallet_keys 已更新并写回配置文件"
            );
            println!("配置文件位置：{}", path.display());
        } else {
            info!(
                target: "config",
                "wallet_keys 已在内存中更新（未定位到配置文件路径）"
            );
        }
        println!("请确认配置后重新启动 Galileo。");
        std::process::exit(0);
    }

    if let Some(remark) = wallet_result.selected_remark.as_ref() {
        match parse_keypair_string(galileo.private_key.trim()) {
            Ok(keypair) => {
                println!("🔓 已解锁钱包 [{}]，公钥 {}", remark, keypair.pubkey());
            }
            Err(err) => {
                println!("🔓 已解锁钱包 [{}]，但解析公钥失败: {err}", remark);
            }
        }
    }

    let mut lander_candidates = Vec::new();
    if let Some(dir) = galileo_dir.as_ref() {
        lander_candidates.push(dir.join("lander.yaml"));
    }
    lander_candidates.extend(DEFAULT_LANDER_PATHS.iter().map(PathBuf::from));

    let (lander, _) = load_first_available_yaml::<LanderConfig>(&lander_candidates)?;

    let mut jupiter_candidates = Vec::new();
    if let Some(dir) = galileo_dir.as_ref() {
        jupiter_candidates.push(dir.join("jupiter.toml"));
    }
    jupiter_candidates.extend(DEFAULT_JUPITER_PATHS.iter().map(PathBuf::from));

    let (jupiter, _) = load_first_available_toml::<JupiterConfig>(&jupiter_candidates)?;

    Ok(AppConfig {
        galileo,
        lander,
        jupiter,
    })
}

fn first_existing_path(paths: &[PathBuf]) -> Option<PathBuf> {
    paths.iter().find_map(|path| {
        if path.exists() {
            Some(path.clone())
        } else {
            None
        }
    })
}

fn load_first_available_yaml<T>(paths: &[PathBuf]) -> Result<(T, Option<PathBuf>), ConfigError>
where
    T: DeserializeOwned + Default,
{
    for candidate in paths {
        if let Some(config) = try_load_file_yaml::<T>(candidate)? {
            let parent = candidate.parent().map(|p| p.to_path_buf());
            return Ok((config, parent));
        }
    }

    Ok((T::default(), None))
}

fn try_load_file_yaml<T>(path: &Path) -> Result<Option<T>, ConfigError>
where
    T: DeserializeOwned,
{
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let config: T = serde_yaml::from_str(&contents).map_err(|err| ConfigError::Parse {
        path: path.to_path_buf(),
        message: err.to_string(),
    })?;

    Ok(Some(config))
}

fn load_first_available_toml<T>(paths: &[PathBuf]) -> Result<(T, Option<PathBuf>), ConfigError>
where
    T: DeserializeOwned + Default,
{
    for candidate in paths {
        if let Some(config) = try_load_file_toml::<T>(candidate)? {
            let parent = candidate.parent().map(|p| p.to_path_buf());
            return Ok((config, parent));
        }
    }

    Ok((T::default(), None))
}

fn try_load_file_toml<T>(path: &Path) -> Result<Option<T>, ConfigError>
where
    T: DeserializeOwned,
{
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let raw_value: toml::Value = toml::from_str(&contents).map_err(|err| ConfigError::Parse {
        path: path.to_path_buf(),
        message: err.to_string(),
    })?;

    let config: T = if let Some(subtable) = raw_value.get("jupiter") {
        let nested = toml::to_string(subtable).map_err(|err| ConfigError::Parse {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?;
        toml::from_str(&nested).map_err(|err| ConfigError::Parse {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?
    } else {
        toml::from_str(&contents).map_err(|err| ConfigError::Parse {
            path: path.to_path_buf(),
            message: err.to_string(),
        })?
    };

    Ok(Some(config))
}
