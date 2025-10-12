use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use thiserror::Error;

use super::{AppConfig, GalileoConfig, LanderConfig};

pub const DEFAULT_CONFIG_PATHS: &[&str] = &["galileo.yaml", "config/galileo.yaml"];
pub const DEFAULT_LANDER_PATHS: &[&str] = &["lander.yaml", "config/lander.yaml"];

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
        source: serde_yaml::Error,
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

    let (galileo, galileo_dir) = load_first_available::<GalileoConfig>(&candidate_paths)?;

    let mut lander_candidates = Vec::new();
    if let Some(dir) = galileo_dir {
        lander_candidates.push(dir.join("lander.yaml"));
    }
    lander_candidates.extend(
        DEFAULT_LANDER_PATHS
            .iter()
            .map(PathBuf::from),
    );

    let (lander, _) = load_first_available::<LanderConfig>(&lander_candidates)?;

    Ok(AppConfig { galileo, lander })
}

fn load_first_available<T>(paths: &[PathBuf]) -> Result<(T, Option<PathBuf>), ConfigError>
where
    T: DeserializeOwned + Default,
{
    for candidate in paths {
        if let Some(config) = try_load_file::<T>(candidate)? {
            let parent = candidate
                .parent()
                .map(|p| p.to_path_buf());
            return Ok((config, parent));
        }
    }

    Ok((T::default(), None))
}

fn try_load_file<T>(path: &Path) -> Result<Option<T>, ConfigError>
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

    let config: T = serde_yaml::from_str(&contents).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })?;

    Ok(Some(config))
}
