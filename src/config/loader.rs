use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use thiserror::Error;

use super::{AppConfig, GalileoConfig, JupiterConfig, LanderConfig};

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
    Parse {
        path: PathBuf,
        message: String,
    },
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

    let mut lander_candidates = Vec::new();
    if let Some(dir) = galileo_dir.as_ref() {
        lander_candidates.push(dir.join("lander.yaml"));
    }
    lander_candidates.extend(
        DEFAULT_LANDER_PATHS
            .iter()
            .map(PathBuf::from),
    );

    let (lander, _) = load_first_available_yaml::<LanderConfig>(&lander_candidates)?;

    let mut jupiter_candidates = Vec::new();
    if let Some(dir) = galileo_dir.as_ref() {
        jupiter_candidates.push(dir.join("jupiter.toml"));
    }
    jupiter_candidates.extend(
        DEFAULT_JUPITER_PATHS
            .iter()
            .map(PathBuf::from),
    );

    let (jupiter, _) = load_first_available_toml::<JupiterConfig>(&jupiter_candidates)?;

    // If request params指定 API URL，覆盖 bot 配置，方便统一维护。
    if let Some(api_url) = &galileo.request_params.api_url {
        galileo.bot.jupiter_api_url = api_url.clone();
    }

    Ok(AppConfig {
        galileo,
        lander,
        jupiter,
    })
}

fn load_first_available_yaml<T>(paths: &[PathBuf]) -> Result<(T, Option<PathBuf>), ConfigError>
where
    T: DeserializeOwned + Default,
{
    for candidate in paths {
        if let Some(config) = try_load_file_yaml::<T>(candidate)? {
            let parent = candidate
                .parent()
                .map(|p| p.to_path_buf());
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
            let parent = candidate
                .parent()
                .map(|p| p.to_path_buf());
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

    let config: T = toml::from_str(&contents).map_err(|err| ConfigError::Parse {
        path: path.to_path_buf(),
        message: err.to_string(),
    })?;

    Ok(Some(config))
}
