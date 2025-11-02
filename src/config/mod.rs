use std::collections::HashSet;

use crate::engine::DispatchStrategy;

use serde::Deserialize;
use serde::de::Deserializer;

pub mod launch;
pub mod loader;
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

pub(crate) fn default_prometheus_listen() -> String {
    "0.0.0.0:9898".to_string()
}

pub(crate) fn default_flashloan_compute_unit_overhead() -> u32 {
    110_000
}

pub(crate) fn default_compute_unit_price_strategy() -> String {
    "fixed".to_string()
}

pub(crate) fn default_tip_strategy() -> cfg::TipStrategyKind {
    cfg::TipStrategyKind::Fixed
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
            proxy: None,
            yellowstone_grpc_url: None,
            yellowstone_grpc_token: None,
            wallet: cfg::WalletConfig::default(),
            instruction: cfg::InstructionConfig::default(),
            logging: cfg::LoggingConfig::default(),
        }
    }
}

impl Default for cfg::EngineConfig {
    fn default() -> Self {
        Self {
            backend: cfg::EngineBackend::default(),
            time_out: cfg::EngineTimeoutConfig::default(),
            console_summary: cfg::ConsoleSummaryConfig::default(),
            dflow: cfg::DflowEngineConfig::default(),
            ultra: cfg::UltraEngineConfig::default(),
            titan: cfg::TitanEngineConfig::default(),
            kamino: cfg::KaminoEngineConfig::default(),
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
            swap_config: cfg::TitanSwapConfig::default(),
            tx_config: cfg::TitanTxConfig::default(),
        }
    }
}

impl Default for cfg::DflowQuoteConfig {
    fn default() -> Self {
        Self {
            use_auto_slippage: true,
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

impl Default for cfg::WalletConfig {
    fn default() -> Self {
        Self {
            private_key: String::new(),
            auto_unwrap: cfg::AutoUnwrapConfig::default(),
            wallet_keys: Vec::new(),
            legacy_wallet_keys: None,
        }
    }
}

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
