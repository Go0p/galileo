use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

use crate::engine::DispatchStrategy;

use serde::Deserialize;
use serde::de::Deserializer;

pub mod launch;
pub mod loader;
pub mod strategy_loader;
pub mod types;
pub mod wallet;

pub use loader::*;
pub use types::*;

use self::types as cfg;

pub(crate) fn default_true() -> bool {
    true
}

pub(crate) fn default_one() -> f64 {
    1.0
}
pub(crate) fn default_logging_level() -> String {
    "info".to_string()
}

pub(crate) fn default_logging_profile() -> cfg::LoggingProfile {
    cfg::LoggingProfile::Lean
}

pub(crate) fn default_slow_quote_warn_ms() -> u64 {
    25
}

pub(crate) fn default_slow_swap_warn_ms() -> u64 {
    200
}

pub(crate) fn default_timezone_offset_hours() -> i8 {
    0
}

pub(crate) fn default_max_tokens_limit() -> u32 {
    20
}

pub(crate) fn default_quote_timeout_ms() -> u64 {
    2_000
}

pub(crate) fn default_auto_unwrap_amount_lamports() -> u64 {
    1_000_000_000
}

pub(crate) fn default_auto_unwrap_min_balance_lamports() -> u64 {
    1_000_000_000
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

pub(crate) fn default_flashloan_compute_unit_overhead() -> u32 {
    110_000
}

pub(crate) fn default_graceful_shutdown_timeout_ms() -> u64 {
    5_000
}

pub(crate) fn default_compute_unit_price_strategy() -> String {
    "fixed".to_string()
}

pub(crate) fn default_tip_strategy() -> cfg::TipStrategyKind {
    cfg::TipStrategyKind::Fixed
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

pub(crate) fn default_strategy_config_dir() -> String {
    "strategies".to_string()
}

#[derive(Deserialize)]
#[serde(untagged)]
enum RpcUrlField {
    Single(String),
    Multiple(Vec<String>),
}

pub(crate) fn deserialize_rpc_urls<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let helper = Option::<RpcUrlField>::deserialize(deserializer)?;
    let mut seen = HashSet::new();
    let mut urls = Vec::new();

    let values = match helper {
        Some(RpcUrlField::Single(url)) => vec![url],
        Some(RpcUrlField::Multiple(list)) => list,
        None => Vec::new(),
    };

    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if seen.insert(trimmed.to_string()) {
            urls.push(trimmed.to_string());
        }
    }

    Ok(urls)
}

impl Default for cfg::GalileoConfig {
    fn default() -> Self {
        Self {
            global: cfg::GlobalConfig::default(),
            engine: cfg::EngineConfig::default(),
            intermedium: cfg::IntermediumConfig::default(),
            wallet_keys: Vec::new(),
            auto_unwrap: cfg::AutoUnwrapConfig::default(),
            private_key: String::new(),
            bot: cfg::BotConfig::default(),
            flashloan: cfg::FlashloanConfig::default(),
            blind_strategy: cfg::BlindStrategyConfig::default(),
            pure_blind_strategy: cfg::PureBlindStrategyConfig::default(),
            back_run_strategy: cfg::BackRunStrategyConfig::default(),
            copy_strategy: cfg::CopyStrategyConfig::default(),
        }
    }
}

impl Default for cfg::GlobalConfig {
    fn default() -> Self {
        Self {
            rpc_urls: Vec::new(),
            proxy: cfg::ProxyConfig::default(),
            yellowstone_grpc_url: None,
            yellowstone_grpc_token: None,
            instruction: cfg::InstructionConfig::default(),
            logging: cfg::LoggingConfig::default(),
            tools: cfg::ToolsConfig::default(),
        }
    }
}

impl Default for cfg::EngineConfig {
    fn default() -> Self {
        Self {
            backend: cfg::EngineBackend::default(),
            time_out: cfg::EngineTimeoutConfig::default(),
            enable_console_summary: false,
            console_summary_only: false,
            jupiter_self_hosted: cfg::JupiterSelfHostedEngineConfig::default(),
            jupiter: cfg::JupiterEngineSet::default(),
            dflow: cfg::DflowEngineConfig::default(),
            ultra: cfg::UltraEngineConfig::default(),
            titan: cfg::TitanEngineConfig::default(),
            kamino: cfg::KaminoEngineConfig::default(),
            multi_leg: cfg::MultiLegEngineConfig::default(),
        }
    }
}

impl Default for cfg::DflowEngineConfig {
    fn default() -> Self {
        Self {
            leg: None,
            api_quote_base: None,
            api_swap_base: None,
            api_proxy: None,
            quote_config: cfg::DflowQuoteConfig::default(),
            swap_config: cfg::DflowSwapConfig::default(),
        }
    }
}

impl Default for cfg::JupiterEngineConfig {
    fn default() -> Self {
        Self {
            enable: false,
            leg: None,
            api_quote_base: None,
            api_swap_base: None,
            api_proxy: None,
            quote_config: cfg::JupiterQuoteConfig::default(),
            swap_config: cfg::JupiterSwapConfig::default(),
        }
    }
}

impl Default for cfg::TitanEngineConfig {
    fn default() -> Self {
        Self {
            leg: None,
            ws_url: None,
            ws_proxy: None,
            jwt: None,
            providers: Vec::new(),
            interval_ms: None,
            num_quotes: None,
            first_quote_timeout_ms: Some(2_000),
            idle_resubscribe_timeout_ms: Some(10_000),
            swap_config: cfg::TitanSwapConfig::default(),
            tx_config: cfg::TitanTxConfig::default(),
        }
    }
}

impl Default for cfg::DflowQuoteConfig {
    fn default() -> Self {
        Self {
            slippage_bps: None,
            only_direct_routes: false,
            max_route_length: None,
            cadence: cfg::QuoteCadenceConfig::default(),
        }
    }
}

impl Default for cfg::DflowSwapConfig {
    fn default() -> Self {
        Self {
            dynamic_compute_unit_limit: true,
            wrap_and_unwrap_sol: false,
            cu_limit_multiplier: 1.0,
        }
    }
}

// WalletConfig 已删除

impl Default for cfg::AutoUnwrapConfig {
    fn default() -> Self {
        Self {
            enable: false,
            unwrap_amount_lamports: default_auto_unwrap_amount_lamports(),
            min_sol_balance_lamports: default_auto_unwrap_min_balance_lamports(),
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
            profile: default_logging_profile(),
            slow_quote_warn_ms: default_slow_quote_warn_ms(),
            slow_swap_warn_ms: default_slow_swap_warn_ms(),
            timezone_offset_hours: default_timezone_offset_hours(),
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
            cpu_affinity: cfg::CpuAffinityConfig::default(),
            get_block_hash_by_grpc: true,
            enable_simulation: false,
            binary: cfg::BotBinaryConfig::default(),
            dry_run: cfg::DryRunConfig::default(),
            prometheus: cfg::PrometheusConfig::default(),
            network: cfg::NetworkConfig::default(),
            auto_refresh_wallet_minute: 0,
            strategies: cfg::StrategyToggleSet::default(),
            engines: cfg::EngineToggleSet::default(),
            flashloan: cfg::BotFlashloanToggle::default(),
            light_house: cfg::LightHouseBotConfig::default(),
        }
    }
}

impl Default for cfg::FlashloanConfig {
    fn default() -> Self {
        Self {
            marginfi: cfg::FlashloanMarginfiConfig::default(),
        }
    }
}

impl Default for cfg::BlindStrategyConfig {
    fn default() -> Self {
        Self {
            memo: String::new(),
            enable_dexs: Vec::new(),
            exclude_dexes: Vec::new(),
            enable_landers: Vec::new(),
            auto_scale_to_ip: cfg::AutoScaleToIpConfig::default(),
            base_mints: Vec::new(),
        }
    }
}

impl Default for cfg::BackRunStrategyConfig {
    fn default() -> Self {
        Self {
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
            compute_unit_price_strategy: default_compute_unit_price_strategy(),
            sending_strategy: DispatchStrategy::default(),
            fixed_compute_unit_price: None,
            random_compute_unit_price_range: Vec::new(),
            jito: None,
            staked: None,
            temporal: None,
            astralane: None,
            skip_preflight: None,
            max_retries: None,
            min_context_slot: None,
        }
    }
}
