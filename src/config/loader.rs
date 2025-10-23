use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use thiserror::Error;

use super::wallet::{encrypted_wallet_path, process_wallet, sanitize_config_file};
use super::{AppConfig, GalileoConfig, JupiterConfig, LanderConfig};
use tracing::{info, warn};

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
    #[error(
        "已将私钥加密写入 {encrypted:?}，并清理配置文件。请确认配置已同步提交或备份后，重新启动 Galileo。"
    )]
    WalletEncrypted { encrypted: PathBuf },
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
    let enc_path = encrypted_wallet_path(galileo_path.as_deref());
    let original_private_key = galileo.global.wallet.private_key.clone();

    let wallet_result = process_wallet(&mut galileo.global.wallet, &enc_path)?;
    if wallet_result.sanitized_config {
        info!(
            target: "config",
            encrypted = %enc_path.display(),
            "检测到配置中的私钥，已写入 AES 加密后的 wallet.enc"
        );
        if let Some(config_path) = galileo_path.as_ref() {
            let trimmed = original_private_key.trim();
            if !trimmed.is_empty() {
                if let Err(err) = sanitize_config_file(config_path, trimmed) {
                    warn!(
                        target: "config",
                        path = %config_path.display(),
                        "无法自动清除配置文件中的明文私钥: {err}"
                    );
                }
            }
        }

        return Err(ConfigError::WalletEncrypted {
            encrypted: enc_path,
        });
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
