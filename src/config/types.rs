#![allow(dead_code)]

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Deserialize;
use serde_with::{DisplayFromStr, serde_as};

use crate::strategy::config::StrategyConfig;

#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    pub galileo: GalileoConfig,
    #[allow(dead_code)]
    pub lander: LanderConfig,
    pub jupiter: JupiterConfig,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct GalileoConfig {
    #[serde(default)]
    pub global: GlobalConfig,
    #[serde(default)]
    pub request_params: RequestParamsConfig,
    #[serde(default)]
    pub intermedium: IntermediumConfig,
    #[serde(default)]
    pub bot: BotConfig,
    #[serde(default)]
    pub flashloan: FlashloanConfig,
    #[serde(default)]
    pub spam: SpamConfig,
    #[serde(default)]
    pub blind: BlindConfig,
    #[serde(default)]
    pub back_run: BackRunConfig,
    #[serde(default)]
    pub strategy: Option<StrategyConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub rpc_url: Option<String>,
    #[serde(default)]
    pub yellowstone_grpc_url: Option<String>,
    #[serde(default)]
    pub yellowstone_grpc_token: Option<String>,
    #[serde(default)]
    pub wallet: WalletConfig,
    #[serde(default)]
    pub instruction: InstructionConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WalletConfig {
    #[serde(default)]
    pub private_key: String,
    #[serde(default)]
    pub min_sol_balance: String,
    #[serde(default)]
    pub warp_or_unwrap_sol: WarpOrUnwrapSolConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WarpOrUnwrapSolConfig {
    #[serde(default)]
    pub wrap_and_unwrap_sol: bool,
    #[serde(default)]
    pub compute_unit_price_micro_lamports: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstructionConfig {
    #[serde(default)]
    pub memo: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "super::default_logging_level")]
    pub level: String,
    #[serde(default)]
    pub json: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RequestParamsConfig {
    #[serde(default)]
    pub included_dexes: Vec<String>,
    #[serde(default)]
    pub excluded_dexes: Vec<String>,
    #[serde(default)]
    pub only_direct_routes: bool,
    #[serde(default = "super::default_true")]
    pub restrict_intermediate_tokens: bool,
    #[serde(default)]
    pub skip_user_accounts_rpc_calls: bool,
    #[serde(default = "super::default_true")]
    pub dynamic_compute_unit_limit: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IntermediumConfig {
    #[serde(default)]
    pub load_mints_from_files: Vec<String>,
    #[serde(default)]
    pub load_mints_from_url: String,
    #[serde(default = "super::default_max_tokens_limit")]
    pub max_tokens_limit: u32,
    #[serde(default)]
    pub mints: Vec<String>,
    #[serde(default)]
    pub disable_mints: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    #[serde(default)]
    pub disable_local_binary: bool,
    #[serde(default)]
    pub disable_running: bool,
    #[serde(default = "super::default_request_timeout_ms")]
    pub request_timeout_ms: u64,
    #[serde(default)]
    pub auto_restart_minutes: u64,
    #[serde(default)]
    pub get_block_hash_by_grpc: bool,
    #[serde(default)]
    pub enable_simulation: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FlashloanConfig {
    #[serde(default)]
    pub enable: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpamConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub enable_log: bool,
    #[serde(default)]
    pub skip_preflight: bool,
    #[serde(default)]
    pub max_retries: u32,
    #[serde(default)]
    pub compute_unit_price_micro_lamports: u64,
    #[serde(default)]
    pub enable_dexs: Vec<String>,
    #[serde(default)]
    pub enable_landers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlindConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub memo: String,
    #[serde(default)]
    pub enable_dexs: Vec<String>,
    #[serde(default)]
    pub enable_landers: Vec<String>,
    #[serde(default)]
    pub base_mints: Vec<BlindBaseMintConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlindBaseMintConfig {
    #[serde(default)]
    pub mint: String,
    #[serde(default)]
    pub trade_size_range: Vec<u64>,
    #[serde(default)]
    pub trade_range_count: Option<u32>,
    #[serde(default)]
    pub trade_range_strategy: Option<String>,
    #[serde(default)]
    pub min_quote_profit: Option<u64>,
    #[serde(default)]
    pub process_delay: Option<u64>,
    #[serde(default)]
    pub sending_cooldown: Option<u64>,
    #[serde(default)]
    pub route_types: Vec<String>,
    #[serde(default)]
    pub three_hop_mints: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackRunConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub enable_dexs: Vec<String>,
    #[serde(default)]
    pub enable_landers: Vec<String>,
    #[serde(default)]
    pub memo: String,
    #[serde(default)]
    pub trigger_memo: String,
    #[serde(default)]
    pub base_mints: Vec<BackRunBaseMintConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackRunBaseMintConfig {
    #[serde(default)]
    pub trigger_amount: u64,
    #[serde(default)]
    pub mint: String,
    #[serde(default)]
    pub min_quote_profit: u64,
    #[serde(default)]
    pub min_simulated_profit: u64,
    #[serde(default)]
    pub skip_profit_check_for_quote: bool,
    #[serde(default)]
    pub min_trade_size: u64,
    #[serde(default)]
    pub max_trade_size: u64,
    #[serde(default)]
    pub trade_configs: Vec<BackRunTradeConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackRunTradeConfig {
    #[serde(default)]
    pub min_trade_bp: Option<u64>,
    #[serde(default)]
    pub max_trade_bp: Option<u64>,
    #[serde(default)]
    pub fixed_size: Option<u64>,
    #[serde(default)]
    pub route_types: Vec<String>,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct JupiterConfig {
    /// `[jupiter.binary]`：GitHub Release 下载来源及安装目录。
    #[serde(default)]
    pub binary: JupiterBinaryConfig,
    /// `[jupiter.core]`：RPC、监听端口等核心启动参数。
    #[serde(default)]
    pub core: JupiterCoreConfig,
    /// `[jupiter.launch]`：路由与功能开关相关的 CLI 选项。
    #[serde(default)]
    pub launch: JupiterLaunchConfig,
    /// `[jupiter.performance]`：线程数等性能配置。
    #[serde(default)]
    pub performance: JupiterPerformanceConfig,
    /// `[jupiter.process]`：守护与自动重启策略。
    #[serde(default)]
    pub process: JupiterProcessConfig,
    /// `[jupiter.environment]`：附加环境变量。
    #[serde(default = "super::default_environment")]
    pub environment: BTreeMap<String, String>,
    /// `[jupiter.health_check]`：启动后健康检查配置。
    #[serde(default)]
    pub health_check: Option<HealthCheckConfig>,
}

#[derive(Debug, Clone, Default)]
pub struct LaunchOverrides {
    pub filter_markets_with_mints: Vec<String>,
    pub exclude_dex_program_ids: Vec<String>,
    pub include_dex_program_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterBinaryConfig {
    #[serde(default = "super::default_repo_owner")]
    pub repo_owner: String,
    #[serde(default = "super::default_repo_name")]
    pub repo_name: String,
    #[serde(default = "super::default_binary_name")]
    pub binary_name: String,
    #[serde(default = "super::default_install_dir")]
    pub install_dir: PathBuf,
    #[serde(default)]
    pub proxy: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterCoreConfig {
    #[serde(default)]
    pub rpc_url: String,
    #[serde(default)]
    pub secondary_rpc_urls: Vec<String>,
    #[serde(default = "super::default_host")]
    pub host: String,
    #[serde(default = "super::default_port")]
    pub port: u16,
    #[serde(default = "super::default_metrics_port")]
    pub metrics_port: u16,
    #[serde(default)]
    pub use_local_market_cache: bool,
    #[serde(default = "super::default_market_cache")]
    pub market_cache: String,
    #[serde(default = "super::default_market_mode")]
    pub market_mode: MarketMode,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterLaunchConfig {
    #[serde(default = "super::default_true")]
    pub allow_circular_arbitrage: bool,
    #[serde(default = "super::default_true")]
    pub enable_new_dexes: bool,
    #[serde(default = "super::default_true")]
    pub expose_quote_and_simulate: bool,
    #[serde(default)]
    pub yellowstone: Option<YellowstoneConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YellowstoneConfig {
    pub endpoint: String,
    #[serde(default)]
    pub x_token: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketMode {
    Remote,
    File,
    Europa,
}

impl MarketMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            MarketMode::Remote => "remote",
            MarketMode::File => "file",
            MarketMode::Europa => "europa",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterPerformanceConfig {
    #[serde(default = "super::default_total_thread_count")]
    pub total_thread_count: u16,
    #[serde(default = "super::default_webserver_thread_count")]
    pub webserver_thread_count: u16,
    #[serde(default = "super::default_update_thread_count")]
    pub update_thread_count: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterProcessConfig {
    #[serde(default)]
    pub auto_restart_minutes: u64,
    #[serde(default = "super::default_max_restart_attempts")]
    pub max_restart_attempts: u32,
    #[serde(default = "super::default_graceful_shutdown_timeout_ms")]
    pub graceful_shutdown_timeout_ms: u64,
}

impl JupiterConfig {
    pub fn binary_path(&self) -> PathBuf {
        self.binary.install_dir.join(&self.binary.binary_name)
    }

    pub fn effective_args(&self, overrides: &LaunchOverrides) -> Vec<String> {
        let mut args = Vec::new();
        let core = &self.core;

        if !core.rpc_url.trim().is_empty() {
            args.push("--rpc-url".to_string());
            args.push(core.rpc_url.clone());
        }

        for url in core
            .secondary_rpc_urls
            .iter()
            .filter(|url| !url.trim().is_empty())
        {
            args.push("--secondary-rpc-url".to_string());
            args.push(url.clone());
        }

        if core.use_local_market_cache {
            args.push("--use-local-market-cache".to_string());
        }

        if !core.market_cache.trim().is_empty() {
            args.push("--market-cache".to_string());
            args.push(core.market_cache.clone());
        }

        args.push("--market-mode".to_string());
        args.push(core.market_mode.as_str().to_string());

        args.push("--host".to_string());
        args.push(core.host.clone());

        args.push("--port".to_string());
        args.push(core.port.to_string());

        args.push("--metrics-port".to_string());
        args.push(core.metrics_port.to_string());

        let launch = &self.launch;
        if launch.allow_circular_arbitrage {
            args.push("--allow-circular-arbitrage".to_string());
        }
        if launch.enable_new_dexes {
            args.push("--enable-new-dexes".to_string());
        }
        if launch.expose_quote_and_simulate {
            args.push("--expose-quote-and-simulate".to_string());
        }
        if let Some(yellowstone) = &launch.yellowstone {
            if !yellowstone.endpoint.trim().is_empty() {
                args.push("--yellowstone-grpc-endpoint".to_string());
                args.push(yellowstone.endpoint.clone());
            }
            if let Some(token) = yellowstone.x_token.as_ref() {
                if !token.is_empty() {
                    args.push("--yellowstone-grpc-x-token".to_string());
                    args.push(token.clone());
                }
            }
        }

        if !overrides.filter_markets_with_mints.is_empty() {
            args.push("--filter-markets-with-mints".to_string());
            args.push(overrides.filter_markets_with_mints.join(","));
        }

        if !overrides.exclude_dex_program_ids.is_empty() {
            args.push("--exclude-dex-program-ids".to_string());
            args.push(overrides.exclude_dex_program_ids.join(","));
        }

        if !overrides.include_dex_program_ids.is_empty() {
            args.push("--dex-program-ids".to_string());
            args.push(overrides.include_dex_program_ids.join(","));
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

        args
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
#[allow(dead_code)]
pub struct LanderConfig {
    #[serde(default)]
    pub lander: LanderSettings,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LanderSettings {
    #[serde(default)]
    pub enable_log: bool,
    #[serde(default = "super::default_priority_fee_strategy")]
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

#[derive(Debug, Clone, Deserialize, Default)]
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
pub struct LanderJitoUuidConfig {
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub rate_limit: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LanderEndpointConfig {
    #[serde(default)]
    pub endpoints: Vec<String>,
}
