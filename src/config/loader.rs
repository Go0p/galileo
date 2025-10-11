use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

use super::GalileoConfig;

pub const DEFAULT_CONFIG_PATHS: &[&str] = &["galileo.toml", "config/galileo.toml"];

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config at {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("failed to parse config at {path}: {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
}

pub fn load_config(path: Option<PathBuf>) -> Result<GalileoConfig, ConfigError> {
    let candidate_paths = match path {
        Some(p) => vec![p],
        None => DEFAULT_CONFIG_PATHS
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<PathBuf>>(),
    };

    for candidate in candidate_paths {
        if let Some(config) = try_load_file(&candidate)? {
            return Ok(config);
        }
    }

    Ok(GalileoConfig::default())
}

fn try_load_file(path: &Path) -> Result<Option<GalileoConfig>, ConfigError> {
    if !path.exists() {
        return Ok(None);
    }

    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let config: GalileoConfig = toml::from_str(&contents).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })?;

    Ok(Some(config))
}
