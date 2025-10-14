use std::collections::BTreeMap;
use std::path::PathBuf;

pub mod loader;
pub mod types;

pub use loader::*;
pub use types::*;

use self::types as cfg;

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_logging_level() -> String {
    "info".to_string()
}

pub(crate) fn default_max_tokens_limit() -> u32 {
    20
}

pub(crate) fn default_request_timeout_ms() -> u64 {
    2_000
}

pub(crate) fn default_repo_owner() -> String {
    "jup-ag".to_string()
}

pub(crate) fn default_repo_name() -> String {
    "jupiter-swap-api".to_string()
}

pub(crate) fn default_binary_name() -> String {
    "jupiter-swap-api".to_string()
}

pub(crate) fn default_install_dir() -> PathBuf {
    PathBuf::from("bin")
}

pub(crate) fn default_host() -> String {
    "0.0.0.0".to_string()
}

pub(crate) fn default_port() -> u16 {
    18_080
}

pub(crate) fn default_metrics_port() -> u16 {
    18_081
}

pub(crate) fn default_market_cache() -> String {
    "https://cache.jup.ag/markets?v=6".to_string()
}

pub(crate) fn default_market_cache_download_url() -> String {
    default_market_cache()
}

pub(crate) fn default_market_mode() -> cfg::MarketMode {
    cfg::MarketMode::Remote
}

pub(crate) fn default_prometheus_listen() -> String {
    "0.0.0.0:9898".to_string()
}

pub(crate) fn default_total_thread_count() -> u16 {
    64
}

pub(crate) fn default_webserver_thread_count() -> u16 {
    24
}

pub(crate) fn default_update_thread_count() -> u16 {
    5
}

pub(crate) fn default_max_restart_attempts() -> u32 {
    3
}

pub(crate) fn default_graceful_shutdown_timeout_ms() -> u64 {
    5_000
}

pub(crate) fn default_priority_fee_strategy() -> String {
    "fixed".to_string()
}

pub(crate) fn default_environment() -> BTreeMap<String, String> {
    BTreeMap::from_iter([(String::from("RUST_LOG"), String::from("info"))])
}

pub(crate) fn default_auto_download_market_cache() -> bool {
    true
}

pub(crate) fn default_health_check_interval_secs() -> u64 {
    10
}

pub(crate) fn default_health_check_max_wait_secs() -> u64 {
    120
}

pub(crate) fn default_health_check_retry_count() -> u32 {
    3
}

impl Default for cfg::GalileoConfig {
    fn default() -> Self {
        Self {
            global: cfg::GlobalConfig::default(),
            request_params: cfg::RequestParamsConfig::default(),
            intermedium: cfg::IntermediumConfig::default(),
            bot: cfg::BotConfig::default(),
            flashloan: cfg::FlashloanConfig::default(),
            blind_strategy: cfg::BlindStrategyConfig::default(),
            back_run_strategy: cfg::BackRunStrategyConfig::default(),
        }
    }
}

impl Default for cfg::GlobalConfig {
    fn default() -> Self {
        Self {
            rpc_url: None,
            yellowstone_grpc_url: None,
            yellowstone_grpc_token: None,
            wallet: cfg::WalletConfig::default(),
            instruction: cfg::InstructionConfig::default(),
            logging: cfg::LoggingConfig::default(),
        }
    }
}

impl Default for cfg::WalletConfig {
    fn default() -> Self {
        Self {
            private_key: String::new(),
            min_sol_balance: String::new(),
            warp_or_unwrap_sol: cfg::WarpOrUnwrapSolConfig::default(),
        }
    }
}

impl Default for cfg::WarpOrUnwrapSolConfig {
    fn default() -> Self {
        Self {
            wrap_and_unwrap_sol: false,
            compute_unit_price_micro_lamports: 0,
        }
    }
}

impl Default for cfg::InstructionConfig {
    fn default() -> Self {
        Self {
            memo: String::new(),
        }
    }
}

impl Default for cfg::LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_logging_level(),
            json: false,
        }
    }
}

impl Default for cfg::RequestParamsConfig {
    fn default() -> Self {
        Self {
            included_dexes: Vec::new(),
            excluded_dexes: Vec::new(),
            only_direct_routes: false,
            restrict_intermediate_tokens: true,
            skip_user_accounts_rpc_calls: false,
            dynamic_compute_unit_limit: true,
        }
    }
}

impl Default for cfg::IntermediumConfig {
    fn default() -> Self {
        Self {
            load_mints_from_files: Vec::new(),
            load_mints_from_url: String::new(),
            max_tokens_limit: default_max_tokens_limit(),
            mints: Vec::new(),
            disable_mints: Vec::new(),
        }
    }
}

impl Default for cfg::BotConfig {
    fn default() -> Self {
        Self {
            disable_local_binary: false,
            disable_running: false,
            request_timeout_ms: default_request_timeout_ms(),
            auto_restart_minutes: 30,
            get_block_hash_by_grpc: true,
            enable_simulation: false,
            show_jupiter_logs: true,
            dry_run: false,
            prometheus: cfg::PrometheusConfig::default(),
        }
    }
}

impl Default for cfg::FlashloanConfig {
    fn default() -> Self {
        Self { enable: false }
    }
}

impl Default for cfg::BlindStrategyConfig {
    fn default() -> Self {
        Self {
            enable: false,
            memo: String::new(),
            enable_dexs: Vec::new(),
            enable_landers: Vec::new(),
            base_mints: Vec::new(),
        }
    }
}

impl Default for cfg::BackRunStrategyConfig {
    fn default() -> Self {
        Self {
            enable: false,
            enable_dexs: Vec::new(),
            enable_landers: Vec::new(),
            memo: String::new(),
            trigger_memo: String::new(),
            base_mints: Vec::new(),
        }
    }
}

impl Default for cfg::PrometheusConfig {
    fn default() -> Self {
        Self {
            enable: false,
            listen: default_prometheus_listen(),
        }
    }
}

impl Default for cfg::JupiterConfig {
    fn default() -> Self {
        Self {
            binary: cfg::JupiterBinaryConfig::default(),
            core: cfg::JupiterCoreConfig::default(),
            launch: cfg::JupiterLaunchConfig::default(),
            performance: cfg::JupiterPerformanceConfig::default(),
            process: cfg::JupiterProcessConfig::default(),
            environment: default_environment(),
            health_check: None,
        }
    }
}

impl Default for cfg::JupiterBinaryConfig {
    fn default() -> Self {
        Self {
            repo_owner: default_repo_owner(),
            repo_name: default_repo_name(),
            binary_name: default_binary_name(),
            install_dir: default_install_dir(),
            proxy: None,
        }
    }
}

impl Default for cfg::JupiterCoreConfig {
    fn default() -> Self {
        Self {
            rpc_url: String::new(),
            secondary_rpc_urls: Vec::new(),
            host: default_host(),
            port: default_port(),
            metrics_port: default_metrics_port(),
            use_local_market_cache: false,
            auto_download_market_cache: default_auto_download_market_cache(),
            market_cache: default_market_cache(),
            market_cache_download_url: default_market_cache_download_url(),
            exclude_other_dex_program_ids: false,
            market_mode: default_market_mode(),
        }
    }
}

impl Default for cfg::JupiterLaunchConfig {
    fn default() -> Self {
        Self {
            allow_circular_arbitrage: true,
            enable_new_dexes: true,
            expose_quote_and_simulate: true,
            yellowstone: None,
        }
    }
}

impl Default for cfg::MarketMode {
    fn default() -> Self {
        cfg::MarketMode::Remote
    }
}

impl Default for cfg::JupiterPerformanceConfig {
    fn default() -> Self {
        Self {
            total_thread_count: default_total_thread_count(),
            webserver_thread_count: default_webserver_thread_count(),
            update_thread_count: default_update_thread_count(),
        }
    }
}

impl Default for cfg::JupiterProcessConfig {
    fn default() -> Self {
        Self {
            auto_restart_minutes: 60,
            max_restart_attempts: default_max_restart_attempts(),
            graceful_shutdown_timeout_ms: default_graceful_shutdown_timeout_ms(),
        }
    }
}

impl Default for cfg::LanderConfig {
    fn default() -> Self {
        Self {
            lander: cfg::LanderSettings::default(),
        }
    }
}

impl Default for cfg::LanderSettings {
    fn default() -> Self {
        Self {
            enable_log: false,
            priority_fee_strategy: default_priority_fee_strategy(),
            fixed_priority_fee: None,
            random_priority_fee_range: Vec::new(),
            jito: None,
            staked: None,
            temporal: None,
            astralane: None,
        }
    }
}
