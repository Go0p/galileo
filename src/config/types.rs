use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

use crate::strategy::config::StrategyConfig;

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct GalileoConfig {
    #[serde(default)]
    pub logging: LoggingConfig,
    pub jupiter: JupiterConfig,
    #[serde(default)]
    pub bot: BotConfig,
    #[serde(default)]
    pub strategy: Option<StrategyConfig>,
}

impl Default for GalileoConfig {
    fn default() -> Self {
        Self {
            logging: LoggingConfig::default(),
            jupiter: JupiterConfig::default(),
            bot: BotConfig::default(),
            strategy: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    pub galileo: GalileoConfig,
    #[allow(dead_code)]
    pub lander: LanderConfig,
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

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    #[serde(default)]
    #[allow(dead_code)]
    pub rpc_url: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub yellowstone_grpc_url: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub yellowstone_grpc_token: Option<String>,
    #[serde(default = "BotConfig::default_jupiter_api_url")]
    pub jupiter_api_url: String,
    #[serde(default = "BotConfig::default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default)]
    pub identity: BotIdentityConfig,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            rpc_url: None,
            yellowstone_grpc_url: None,
            yellowstone_grpc_token: None,
            jupiter_api_url: Self::default_jupiter_api_url(),
            request_timeout_ms: Self::default_request_timeout_ms(),
            identity: BotIdentityConfig::default(),
        }
    }
}

impl BotConfig {
    fn default_jupiter_api_url() -> String {
        "http://127.0.0.1:8080".to_string()
    }

    fn default_request_timeout_ms() -> u64 {
        2_000
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotIdentityConfig {
    #[serde(default)]
    pub user_pubkey: Option<String>,
    #[serde(default)]
    pub fee_account: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub tip_account: Option<String>,
    #[serde(default)]
    pub wrap_and_unwrap_sol: bool,
    #[serde(default)]
    pub use_shared_accounts: bool,
    #[serde(default)]
    pub compute_unit_price_micro_lamports: Option<u64>,
    #[serde(default)]
    pub skip_user_accounts_rpc_calls: Option<bool>,
}

impl Default for BotIdentityConfig {
    fn default() -> Self {
        Self {
            user_pubkey: None,
            fee_account: None,
            tip_account: None,
            wrap_and_unwrap_sol: false,
            use_shared_accounts: false,
            compute_unit_price_micro_lamports: Some(1),
            skip_user_accounts_rpc_calls: Some(true),
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct JupiterConfig {
    #[serde(default)]
    pub binary: JupiterBinaryConfig,
    #[serde(default)]
    pub launch: JupiterLaunchConfig,
    #[serde(default)]
    pub performance: JupiterPerformanceConfig,
    #[serde(default)]
    pub tokens: JupiterTokenConfig,
    #[serde(default)]
    pub process: JupiterProcessConfig,
    #[serde(default = "JupiterConfig::default_environment")]
    pub environment: BTreeMap<String, String>,
    #[serde(default)]
    pub health_check: Option<HealthCheckConfig>,
    #[serde(default)]
    pub disable_update: bool,
    #[serde(default)]
    pub download_preference: Vec<String>,
    #[serde(default)]
    pub extra_args: Vec<String>,
}

impl Default for JupiterConfig {
    fn default() -> Self {
        Self {
            binary: JupiterBinaryConfig::default(),
            launch: JupiterLaunchConfig::default(),
            performance: JupiterPerformanceConfig::default(),
            tokens: JupiterTokenConfig::default(),
            process: JupiterProcessConfig::default(),
            environment: Self::default_environment(),
            health_check: None,
            disable_update: false,
            download_preference: Vec::default(),
            extra_args: Vec::default(),
        }
    }
}

impl JupiterConfig {
    fn default_environment() -> BTreeMap<String, String> {
        let mut env = BTreeMap::new();
        env.insert("RUST_LOG".to_string(), "info".to_string());
        env
    }

    pub fn binary_path(&self) -> PathBuf {
        self.binary.install_dir.join(&self.binary.binary_name)
    }

    pub fn effective_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        let launch = &self.launch;
        if let Some(rpc_url) = &launch.rpc_url {
            args.push("--rpc-url".to_string());
            args.push(rpc_url.clone());
        }

        if !launch.market_cache.is_empty() {
            args.push("--market-cache".to_string());
            args.push(launch.market_cache.clone());
        }

        args.push("--market-mode".to_string());
        args.push(launch.market_mode.as_str().to_string());

        args.push("--host".to_string());
        args.push(launch.host.clone());

        args.push("--port".to_string());
        args.push(launch.port.to_string());

        if let Some(metrics_port) = launch.metrics_port {
            args.push("--metrics-port".to_string());
            args.push(metrics_port.to_string());
        }

        if launch.allow_circular_arbitrage {
            args.push("--allow-circular-arbitrage".to_string());
        }

        if launch.enable_new_dexes {
            args.push("--enable-new-dexes".to_string());
        }

        if launch.expose_quote_and_simulate {
            args.push("--expose-quote-and-simulate".to_string());
        }

        if launch.enable_markets {
            args.push("--enable-markets".to_string());
        }

        if launch.enable_tokens {
            args.push("--enable-tokens".to_string());
        }

        if launch.skip_user_accounts_rpc {
            args.push("--skip-user-accounts-rpc-calls".to_string());
        }

        if let Some(yellowstone) = &launch.yellowstone {
            args.push("--yellowstone-grpc-endpoint".to_string());
            args.push(yellowstone.endpoint.clone());
            if let Some(token) = &yellowstone.x_token {
                if !token.is_empty() {
                    args.push("--yellowstone-grpc-x-token".to_string());
                    args.push(token.clone());
                }
            }
        }

        if !self.tokens.filter_markets_with_mints.is_empty() {
            args.push("--filter-markets-with-mints".to_string());
            args.push(self.tokens.filter_markets_with_mints.join(","));
        }

        if !self.tokens.exclude_dex_program_ids.is_empty() {
            args.push("--exclude-dex-program-ids".to_string());
            args.push(self.tokens.exclude_dex_program_ids.join(","));
        }

        let performance = &self.performance;
        let total_threads = std::cmp::max(performance.total_thread_count, 1);
        args.push("--total-thread-count".to_string());
        args.push(total_threads.to_string());

        let web_threads = std::cmp::max(
            std::cmp::min(performance.webserver_thread_count, total_threads),
            1,
        );
        args.push("--webserver-thread-count".to_string());
        args.push(web_threads.to_string());

        let update_threads = std::cmp::max(performance.update_thread_count, 1);
        args.push("--update-thread-count".to_string());
        args.push(update_threads.to_string());

        args.extend(self.extra_args.clone());

        args
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterBinaryConfig {
    #[serde(default = "JupiterBinaryConfig::default_repo_owner")]
    pub repo_owner: String,
    #[serde(default = "JupiterBinaryConfig::default_repo_name")]
    pub repo_name: String,
    #[serde(default = "JupiterBinaryConfig::default_binary_name")]
    pub binary_name: String,
    #[serde(default = "JupiterBinaryConfig::default_install_dir")]
    pub install_dir: PathBuf,
}

impl Default for JupiterBinaryConfig {
    fn default() -> Self {
        Self {
            repo_owner: Self::default_repo_owner(),
            repo_name: Self::default_repo_name(),
            binary_name: Self::default_binary_name(),
            install_dir: Self::default_install_dir(),
        }
    }
}

impl JupiterBinaryConfig {
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterLaunchConfig {
    #[serde(default)]
    pub rpc_url: Option<String>,
    #[serde(default = "JupiterLaunchConfig::default_host")]
    pub host: String,
    #[serde(default = "JupiterLaunchConfig::default_port")]
    pub port: u16,
    #[serde(default)]
    pub metrics_port: Option<u16>,
    #[serde(default = "JupiterLaunchConfig::default_market_cache")]
    pub market_cache: String,
    #[serde(default)]
    pub market_mode: MarketMode,
    #[serde(default)]
    pub disable_local_binary: bool,
    #[serde(default = "JupiterLaunchConfig::default_true")]
    pub allow_circular_arbitrage: bool,
    #[serde(default = "JupiterLaunchConfig::default_true")]
    pub enable_new_dexes: bool,
    #[serde(default = "JupiterLaunchConfig::default_true")]
    pub expose_quote_and_simulate: bool,
    #[serde(default = "JupiterLaunchConfig::default_true")]
    pub enable_markets: bool,
    #[serde(default = "JupiterLaunchConfig::default_true")]
    pub enable_tokens: bool,
    #[serde(default = "JupiterLaunchConfig::default_true")]
    pub skip_user_accounts_rpc: bool,
    #[serde(default)]
    pub yellowstone: Option<YellowstoneConfig>,
}

impl Default for JupiterLaunchConfig {
    fn default() -> Self {
        Self {
            rpc_url: None,
            host: Self::default_host(),
            port: Self::default_port(),
            metrics_port: Some(18081),
            market_cache: Self::default_market_cache(),
            market_mode: MarketMode::Remote,
            disable_local_binary: false,
            allow_circular_arbitrage: Self::default_true(),
            enable_new_dexes: Self::default_true(),
            expose_quote_and_simulate: Self::default_true(),
            enable_markets: Self::default_true(),
            enable_tokens: Self::default_true(),
            skip_user_accounts_rpc: Self::default_true(),
            yellowstone: None,
        }
    }
}

impl JupiterLaunchConfig {
    fn default_true() -> bool {
        true
    }

    fn default_host() -> String {
        "0.0.0.0".to_string()
    }

    fn default_port() -> u16 {
        18_080
    }

    fn default_market_cache() -> String {
        "https://cache.jup.ag/markets?v=4".to_string()
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketMode {
    Local,
    Remote,
}

impl Default for MarketMode {
    fn default() -> Self {
        MarketMode::Remote
    }
}

impl MarketMode {
    fn as_str(&self) -> &'static str {
        match self {
            MarketMode::Local => "local",
            MarketMode::Remote => "remote",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct YellowstoneConfig {
    pub endpoint: String,
    #[serde(default)]
    pub x_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterPerformanceConfig {
    #[serde(default = "JupiterPerformanceConfig::default_total_thread_count")]
    pub total_thread_count: u16,
    #[serde(default = "JupiterPerformanceConfig::default_webserver_thread_count")]
    pub webserver_thread_count: u16,
    #[serde(default = "JupiterPerformanceConfig::default_update_thread_count")]
    pub update_thread_count: u16,
}

impl Default for JupiterPerformanceConfig {
    fn default() -> Self {
        Self {
            total_thread_count: Self::default_total_thread_count(),
            webserver_thread_count: Self::default_webserver_thread_count(),
            update_thread_count: Self::default_update_thread_count(),
        }
    }
}

impl JupiterPerformanceConfig {
    fn default_total_thread_count() -> u16 {
        64
    }

    fn default_webserver_thread_count() -> u16 {
        24
    }

    fn default_update_thread_count() -> u16 {
        5
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct JupiterTokenConfig {
    #[serde(default)]
    pub filter_markets_with_mints: Vec<String>,
    #[serde(default)]
    pub exclude_dex_program_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterProcessConfig {
    #[serde(default)]
    pub auto_restart_minutes: u64,
    #[serde(default = "JupiterProcessConfig::default_max_restart_attempts")]
    pub max_restart_attempts: u32,
    #[serde(default = "JupiterProcessConfig::default_graceful_shutdown_timeout_ms")]
    pub graceful_shutdown_timeout_ms: u64,
}

impl Default for JupiterProcessConfig {
    fn default() -> Self {
        Self {
            auto_restart_minutes: 0,
            max_restart_attempts: Self::default_max_restart_attempts(),
            graceful_shutdown_timeout_ms: Self::default_graceful_shutdown_timeout_ms(),
        }
    }
}

impl JupiterProcessConfig {
    fn default_graceful_shutdown_timeout_ms() -> u64 {
        5_000
    }

    fn default_max_restart_attempts() -> u32 {
        3
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

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct LanderConfig {
    #[serde(default)]
    pub lander: LanderSettings,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct LanderSettings {
    #[serde(default)]
    pub enable_log: bool,
    #[serde(default = "LanderSettings::default_priority_fee_strategy")]
    pub priority_fee_strategy: String,
    #[serde(default)]
    pub fixed_priority_fee: Option<u64>,
    #[serde(default)]
    pub random_priority_fee_range: Vec<u64>,
    #[serde(default)]
    pub jito: Option<LanderJitoConfig>,
    #[serde(default)]
    pub staked: Option<LanderEndpointConfig>,
    #[serde(default)]
    pub temporal: Option<LanderEndpointConfig>,
    #[serde(default)]
    pub astralane: Option<LanderEndpointConfig>,
}

impl Default for LanderSettings {
    fn default() -> Self {
        Self {
            enable_log: false,
            priority_fee_strategy: Self::default_priority_fee_strategy(),
            fixed_priority_fee: None,
            random_priority_fee_range: Vec::new(),
            jito: None,
            staked: None,
            temporal: None,
            astralane: None,
        }
    }
}

impl LanderSettings {
    fn default_priority_fee_strategy() -> String {
        "fixed".to_string()
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct LanderJitoConfig {
    #[serde(default)]
    pub endpoints: Vec<String>,
    #[serde(default)]
    pub tip_strategies: Vec<String>,
    #[serde(default)]
    pub static_tip_bp: Option<u64>,
    #[serde(default)]
    pub static_tip_bps: Vec<u64>,
    #[serde(default)]
    pub fixed_tip: Option<u64>,
    #[serde(default)]
    pub fixed_tips: Vec<u64>,
    #[serde(default)]
    pub floor_tip: Option<String>,
    #[serde(default)]
    pub floor_tips: Vec<String>,
    #[serde(default)]
    pub max_floor_tip_lamports: Option<u64>,
    #[serde(default)]
    pub uuid_config: Vec<LanderJitoUuidConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct LanderJitoUuidConfig {
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub rate_limit: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct LanderEndpointConfig {
    #[serde(default)]
    pub endpoints: Vec<String>,
}
