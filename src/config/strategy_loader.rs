use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use tracing::{debug, info, warn};

use super::loader::ConfigError;
use super::types::{
    BackRunStrategyConfig, BlindStrategyConfig, CopyStrategyConfig, GalileoConfig,
    PureBlindStrategyConfig, StrategyToggle,
};

/// 从外部文件加载策略配置
///
/// 优先级：
/// 1. 如果主配置文件中已经配置了策略（非默认值），则使用主配置
/// 2. 否则尝试从外部文件加载
/// 3. 如果外部文件也不存在，使用默认配置
pub fn load_strategy_configs(
    config: &mut GalileoConfig,
    config_dir: Option<&Path>,
) -> Result<(), ConfigError> {
    let strategy_dir = resolve_strategy_dir(&config.bot.strategies.config_dir, config_dir);

    info!(
        target: "config::strategy",
        enabled = ?config.bot.strategies.enabled,
        strategy_dir = %strategy_dir.display(),
        "开始加载策略配置"
    );

    for strategy in &config.bot.strategies.enabled {
        match strategy {
            StrategyToggle::BlindStrategy => {
                load_if_needed(
                    &mut config.blind_strategy,
                    &strategy_dir,
                    "blind_strategy.yaml",
                    "blind_strategy",
                )?;
            }
            StrategyToggle::PureBlindStrategy => {
                load_if_needed(
                    &mut config.pure_blind_strategy,
                    &strategy_dir,
                    "pure_blind_strategy.yaml",
                    "pure_blind_strategy",
                )?;
            }
            StrategyToggle::CopyStrategy => {
                load_if_needed(
                    &mut config.copy_strategy,
                    &strategy_dir,
                    "copy_strategy.yaml",
                    "copy_strategy",
                )?;
            }
            StrategyToggle::BackRunStrategy => {
                load_if_needed(
                    &mut config.back_run_strategy,
                    &strategy_dir,
                    "back_run_strategy.yaml",
                    "back_run_strategy",
                )?;
            }
        }
    }

    Ok(())
}

/// 解析策略配置目录路径
fn resolve_strategy_dir(config_dir: &str, base_dir: Option<&Path>) -> PathBuf {
    let path = PathBuf::from(config_dir);

    // 如果是绝对路径，直接使用
    if path.is_absolute() {
        return path;
    }

    // 相对路径：相对于配置文件所在目录
    if let Some(base) = base_dir {
        base.join(config_dir)
    } else {
        path
    }
}

/// 如果策略配置是默认值，则尝试从外部文件加载
fn load_if_needed<T>(
    current: &mut T,
    strategy_dir: &Path,
    filename: &str,
    strategy_name: &str,
) -> Result<(), ConfigError>
where
    T: DeserializeOwned + Default + IsDefault,
{
    // 如果主配置文件中已有非默认配置，则使用主配置
    if !current.is_default() {
        debug!(
            target: "config::strategy",
            strategy = strategy_name,
            "使用主配置文件中的策略配置"
        );
        return Ok(());
    }

    // 尝试从外部文件加载
    let strategy_path = strategy_dir.join(filename);

    if !strategy_path.exists() {
        debug!(
            target: "config::strategy",
            strategy = strategy_name,
            path = %strategy_path.display(),
            "策略配置文件不存在，使用默认配置"
        );
        return Ok(());
    }

    match load_strategy_file::<T>(&strategy_path) {
        Ok(loaded) => {
            *current = loaded;
            info!(
                target: "config::strategy",
                strategy = strategy_name,
                path = %strategy_path.display(),
                "已从外部文件加载策略配置"
            );
            Ok(())
        }
        Err(err) => {
            warn!(
                target: "config::strategy",
                strategy = strategy_name,
                path = %strategy_path.display(),
                error = %err,
                "加载策略配置文件失败，使用默认配置"
            );
            // 加载失败不中断程序，继续使用默认配置
            Ok(())
        }
    }
}

/// 从文件加载策略配置
fn load_strategy_file<T>(path: &Path) -> Result<T, ConfigError>
where
    T: DeserializeOwned,
{
    let contents = fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let config: T = serde_yaml::from_str(&contents).map_err(|err| ConfigError::Parse {
        path: path.to_path_buf(),
        message: err.to_string(),
    })?;

    Ok(config)
}

/// Trait 用于判断配置是否为默认值
///
/// 用于区分"用户在主配置文件中显式配置了策略"和"使用的是默认值"
trait IsDefault {
    fn is_default(&self) -> bool;
}

impl IsDefault for BlindStrategyConfig {
    fn is_default(&self) -> bool {
        // 简单判断：如果 base_mints 为空，认为是默认配置
        self.base_mints.is_empty()
    }
}

impl IsDefault for PureBlindStrategyConfig {
    fn is_default(&self) -> bool {
        // 简单判断：如果 assets 的 base_mints 为空，认为是默认配置
        self.assets.base_mints.is_empty()
    }
}

impl IsDefault for CopyStrategyConfig {
    fn is_default(&self) -> bool {
        // 简单判断：如果 wallets 为空，认为是默认配置
        self.wallets.is_empty()
    }
}

impl IsDefault for BackRunStrategyConfig {
    fn is_default(&self) -> bool {
        // 简单判断：如果 base_mints 为空，认为是默认配置
        self.base_mints.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_strategy_dir_absolute() {
        let path = resolve_strategy_dir("/absolute/path", None);
        assert_eq!(path, PathBuf::from("/absolute/path"));
    }

    #[test]
    fn test_resolve_strategy_dir_relative_with_base() {
        let base = Path::new("/config");
        let path = resolve_strategy_dir("strategies", Some(base));
        assert_eq!(path, PathBuf::from("/config/strategies"));
    }

    #[test]
    fn test_resolve_strategy_dir_relative_without_base() {
        let path = resolve_strategy_dir("strategies", None);
        assert_eq!(path, PathBuf::from("strategies"));
    }
}
