use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct GalileoConfig {
    #[serde(default)]
    pub logging: LoggingConfig,
    pub jupiter: JupiterConfig,
    #[serde(default)]
    pub http: HttpConfig,
}

impl Default for GalileoConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
            jupiter: JupiterConfig::default(),
            http: HttpConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "LoggingConfig::default_level")]
    pub level: String,
    #[serde(default)]
    pub json: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: Self::default_level(),
            json: false,
        }
    }
}

impl LoggingConfig {
    fn default_level() -> String {
        "info".to_string()
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct JupiterConfig {
    #[serde(default = "JupiterConfig::default_repo_owner")]
    pub repo_owner: String,
    #[serde(default = "JupiterConfig::default_repo_name")]
    pub repo_name: String,
    #[serde(default = "JupiterConfig::default_binary_name")]
    pub binary_name: String,
    #[serde(default = "JupiterConfig::default_install_dir")]
    pub install_dir: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub environment: BTreeMap<String, String>,
    #[serde(default)]
    pub health_check: Option<HealthCheckConfig>,
    #[serde(default)]
    pub disable_update: bool,
    #[serde(default)]
    pub download_preference: Vec<String>,
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            repo_owner: Self::default_repo_owner(),
            repo_name: Self::default_repo_name(),
            binary_name: Self::default_binary_name(),
            install_dir: Self::default_install_dir(),
            args: vec![],
            environment: BTreeMap::default(),
            health_check: None,
            disable_update: false,
            download_preference: Vec::default(),
        }
    }
}

impl JupiterConfig {
    fn default_repo_owner() -> String {
        "jup-ag".to_string()
    }

    fn default_repo_name() -> String {
        "jupiter-swap-api".to_string()
    }

    fn default_binary_name() -> String {
        "jupiter-swap-api".to_string()
    }

    fn default_install_dir() -> PathBuf {
        PathBuf::from("bin")
    }

    pub fn binary_path(&self) -> PathBuf {
        self.install_dir.join(&self.binary_name)
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct HealthCheckConfig {
    pub url: String,
    #[serde(default)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub expected_status: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HttpConfig {
    #[serde(default = "HttpConfig::default_base_url")]
    pub base_url: String,
    #[serde(default = "HttpConfig::default_request_timeout_ms")]
    pub request_timeout_ms: u64,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8080".to_string(),
            request_timeout_ms: Self::default_request_timeout_ms(),
        }
    }
}

impl HttpConfig {
    fn default_base_url() -> String {
        "http://127.0.0.1:8080".to_string()
    }

    fn default_request_timeout_ms() -> u64 {
        2_000
    }
}
