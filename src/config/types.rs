#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::net::IpAddr;
use std::path::PathBuf;

use serde::Deserialize;
use serde::de::{Deserializer, Error as DeError, Unexpected, Visitor};
use serde_with::serde_as;

use crate::engine::DispatchStrategy;

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
    pub engine: EngineConfig,
    #[serde(default)]
    pub intermedium: IntermediumConfig,
    #[serde(default)]
    pub bot: BotConfig,
    #[serde(default)]
    pub flashloan: FlashloanConfig,
    #[serde(default)]
    pub blind_strategy: BlindStrategyConfig,
    #[serde(default)]
    pub pure_blind_strategy: PureBlindStrategyConfig,
    #[serde(default)]
    pub back_run_strategy: BackRunStrategyConfig,
    #[serde(default)]
    pub copy_strategy: CopyStrategyConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalConfig {
    #[serde(default, deserialize_with = "super::deserialize_rpc_urls")]
    pub rpc_urls: Vec<String>,
    #[serde(default)]
    pub proxy: Option<String>,
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

impl GlobalConfig {
    pub fn rpc_urls(&self) -> &[String] {
        &self.rpc_urls
    }

    pub fn primary_rpc_url(&self) -> Option<&str> {
        self.rpc_urls.first().map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Wrapper {
        global: GlobalConfig,
    }

    #[test]
    fn deserialize_single_rpc_url_string() {
        let yaml = "global:\n  rpc_urls: http://localhost:8899\n";
        let wrapper: Wrapper = serde_yaml::from_str(yaml).expect("parse yaml");
        assert_eq!(
            wrapper.global.rpc_urls(),
            &["http://localhost:8899".to_string()]
        );
    }

    #[test]
    fn deserialize_multiple_rpc_urls_dedup() {
        let yaml =
            "global:\n  rpc_urls:\n    - http://a:8899\n    - http://b:8899\n    - http://a:8899\n";
        let wrapper: Wrapper = serde_yaml::from_str(yaml).expect("parse yaml");
        assert_eq!(
            wrapper.global.rpc_urls(),
            &["http://a:8899".to_string(), "http://b:8899".to_string()]
        );
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WalletConfig {
    #[serde(default)]
    pub private_key: String,
    #[serde(default)]
    pub auto_unwrap: AutoUnwrapConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AutoUnwrapConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default = "super::default_auto_unwrap_amount_lamports")]
    #[serde(alias = "unwarp_amount_lamports")]
    pub unwrap_amount_lamports: u64,
    #[serde(default = "super::default_auto_unwrap_min_balance_lamports")]
    pub min_sol_balance_lamports: u64,
    #[serde(default)]
    pub compute_unit_price_micro_lamports: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstructionConfig {
    #[serde(default)]
    pub memo: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EngineConfig {
    #[serde(default)]
    pub backend: EngineBackend,
    #[serde(default)]
    pub jupiter: JupiterEngineConfig,
    #[serde(default)]
    pub dflow: DflowEngineConfig,
    #[serde(default)]
    pub ultra: UltraEngineConfig,
    #[serde(default)]
    pub titan: TitanEngineConfig,
    #[serde(default)]
    pub kamino: KaminoEngineConfig,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EngineBackend {
    Jupiter,
    Dflow,
    Ultra,
    Kamino,
    #[serde(rename = "multi-legs")]
    MultiLegs,
    None,
}

impl Default for EngineBackend {
    fn default() -> Self {
        Self::Jupiter
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuoteParallelism {
    Auto,
    Fixed(u16),
}

impl Default for QuoteParallelism {
    fn default() -> Self {
        Self::Auto
    }
}

impl QuoteParallelism {
    pub fn as_option(self) -> Option<u16> {
        match self {
            QuoteParallelism::Auto => None,
            QuoteParallelism::Fixed(value) => Some(value),
        }
    }
}

impl<'de> Deserialize<'de> for QuoteParallelism {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct QuoteParallelismVisitor;

        impl<'de> Visitor<'de> for QuoteParallelismVisitor {
            type Value = QuoteParallelism;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("正整数或字符串 \"auto\"")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                if value.eq_ignore_ascii_case("auto") {
                    Ok(QuoteParallelism::Auto)
                } else {
                    let parsed = value
                        .parse::<u16>()
                        .map_err(|_| DeError::invalid_value(Unexpected::Str(value), &self))?;
                    if parsed == 0 {
                        Err(DeError::invalid_value(Unexpected::Unsigned(0), &"正整数"))
                    } else {
                        Ok(QuoteParallelism::Fixed(parsed))
                    }
                }
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                if value == 0 {
                    return Err(DeError::invalid_value(Unexpected::Unsigned(0), &"正整数"));
                }
                if value > u16::MAX as u64 {
                    return Err(DeError::invalid_value(
                        Unexpected::Unsigned(value),
                        &"不大于 u16::MAX 的正整数",
                    ));
                }
                Ok(QuoteParallelism::Fixed(value as u16))
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                if value <= 0 {
                    return Err(DeError::invalid_value(
                        Unexpected::Signed(value),
                        &"大于 0 的整数",
                    ));
                }
                if value > u16::MAX as i64 {
                    return Err(DeError::invalid_value(
                        Unexpected::Signed(value),
                        &"不大于 u16::MAX 的正整数",
                    ));
                }
                Ok(QuoteParallelism::Fixed(value as u16))
            }
        }

        deserializer.deserialize_any(QuoteParallelismVisitor)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterEngineConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub api_proxy: Option<String>,
    #[serde(default)]
    pub args_included_dexes: Vec<String>,
    #[serde(default)]
    pub quote_config: JupiterQuoteConfig,
    #[serde(default)]
    pub swap_config: JupiterSwapConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterQuoteConfig {
    #[serde(default)]
    pub only_direct_routes: bool,
    #[serde(default = "super::default_true")]
    pub restrict_intermediate_tokens: bool,
    #[serde(default)]
    pub parallelism: QuoteParallelism,
    #[serde(default)]
    pub batch_interval_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterSwapConfig {
    #[serde(default)]
    pub skip_user_accounts_rpc_calls: bool,
    #[serde(default = "super::default_true")]
    pub dynamic_compute_unit_limit: bool,
    #[serde(default)]
    pub wrap_and_unwrap_sol: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DflowEngineConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub leg: Option<LegRole>,
    #[serde(default)]
    pub api_quote_base: Option<String>,
    #[serde(default)]
    pub api_swap_base: Option<String>,
    #[serde(default)]
    pub api_proxy: Option<String>,
    #[serde(default)]
    pub quote_config: DflowQuoteConfig,
    #[serde(default)]
    pub swap_config: DflowSwapConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TitanEngineConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub leg: Option<LegRole>,
    #[serde(default)]
    pub ws_url: Option<String>,
    #[serde(default)]
    pub ws_proxy: Option<String>,
    #[serde(default)]
    pub jwt: Option<String>,
    #[serde(default)]
    pub providers: Vec<String>,
    #[serde(default)]
    pub interval_ms: Option<u64>,
    #[serde(default)]
    pub num_quotes: Option<u32>,
    #[serde(default)]
    pub first_quote_timeout_ms: Option<u64>,
    #[serde(default)]
    pub swap_config: TitanSwapConfig,
    #[serde(default)]
    pub tx_config: TitanTxConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TitanSwapConfig {
    #[serde(default)]
    pub dexes: Vec<String>,
    #[serde(default)]
    pub exclude_dexes: Vec<String>,
    #[serde(default)]
    pub only_direct_routes: Option<bool>,
    #[serde(default)]
    pub providers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TitanTxConfig {
    #[serde(default)]
    pub user_public_key: Option<String>,
    #[serde(default)]
    pub create_output_token_account: Option<bool>,
    #[serde(default)]
    pub use_wsol: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct KaminoEngineConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub leg: Option<LegRole>,
    #[serde(default)]
    pub api_quote_base: Option<String>,
    #[serde(default)]
    pub api_swap_base: Option<String>,
    #[serde(default)]
    pub api_proxy: Option<String>,
    #[serde(default)]
    pub quote_config: KaminoQuoteConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KaminoQuoteConfig {
    #[serde(default)]
    pub max_slippage_bps: u16,
    #[serde(default)]
    pub executor: String,
    #[serde(default)]
    pub referrer_pda: String,
    #[serde(default = "super::default_true")]
    pub include_setup_ixs: bool,
    #[serde(default)]
    pub wrap_and_unwrap_sol: bool,
    #[serde(default)]
    pub routes: Vec<String>,
    #[serde(default = "default_cu_limit_multiplier")]
    pub cu_limit_multiplier: f64,
    #[serde(default)]
    pub parallelism: QuoteParallelism,
    #[serde(default)]
    pub batch_interval_ms: Option<u64>,
    #[serde(default)]
    pub resolve_lookup_tables_via_rpc: bool,
}

impl Default for KaminoQuoteConfig {
    fn default() -> Self {
        Self {
            max_slippage_bps: 0,
            executor: String::new(),
            referrer_pda: String::new(),
            include_setup_ixs: super::default_true(),
            wrap_and_unwrap_sol: false,
            routes: Vec::new(),
            cu_limit_multiplier: default_cu_limit_multiplier(),
            parallelism: QuoteParallelism::default(),
            batch_interval_ms: None,
            resolve_lookup_tables_via_rpc: false,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DflowQuoteConfig {
    #[serde(default = "super::default_true")]
    pub use_auto_slippage: bool,
    #[serde(default)]
    pub only_direct_routes: bool,
    #[serde(default)]
    pub max_route_length: Option<u8>,
    #[serde(default)]
    pub parallelism: QuoteParallelism,
    #[serde(default)]
    pub batch_interval_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DflowSwapConfig {
    #[serde(default = "super::default_true")]
    pub dynamic_compute_unit_limit: bool,
    #[serde(default)]
    pub wrap_and_unwrap_sol: bool,
    #[serde(default = "default_cu_limit_multiplier")]
    pub cu_limit_multiplier: f64,
}

const fn default_cu_limit_multiplier() -> f64 {
    1.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct UltraEngineConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub leg: Option<LegRole>,
    #[serde(default)]
    pub legs: Vec<LegRole>,
    #[serde(default)]
    pub api_quote_base: Option<String>,
    #[serde(default)]
    pub api_swap_base: Option<String>,
    #[serde(default)]
    pub api_proxy: Option<String>,
    #[serde(default)]
    pub quote_config: UltraQuoteConfig,
    #[serde(default)]
    pub swap_config: UltraSwapConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UltraQuoteConfig {
    #[serde(default)]
    pub include_routers: Vec<String>,
    #[serde(default)]
    pub exclude_routers: Vec<String>,
    #[serde(default)]
    pub taker: Option<String>,
    #[serde(default)]
    pub use_wsol: bool,
    #[serde(default)]
    pub broadcast_fee_type: Option<String>,
    #[serde(default)]
    pub jito_tip_lamports: Option<u64>,
    #[serde(default = "default_priority_fee_lamports")]
    pub priority_fee_lamports: Option<u64>,
    #[serde(default)]
    pub parallelism: QuoteParallelism,
    #[serde(default)]
    pub batch_interval_ms: Option<u64>,
}

fn default_priority_fee_lamports() -> Option<u64> {
    Some(10)
}

#[derive(Debug, Clone, Deserialize)]
pub struct UltraSwapConfig {
    #[serde(default = "default_cu_limit_multiplier")]
    pub cu_limit_multiplier: f64,
}

impl Default for UltraEngineConfig {
    fn default() -> Self {
        Self {
            enable: false,
            leg: None,
            legs: Vec::new(),
            api_quote_base: None,
            api_swap_base: None,
            api_proxy: None,
            quote_config: UltraQuoteConfig::default(),
            swap_config: UltraSwapConfig::default(),
        }
    }
}

impl Default for UltraQuoteConfig {
    fn default() -> Self {
        Self {
            include_routers: Vec::new(),
            exclude_routers: Vec::new(),
            taker: None,
            use_wsol: false,
            broadcast_fee_type: None,
            jito_tip_lamports: None,
            priority_fee_lamports: default_priority_fee_lamports(),
            parallelism: QuoteParallelism::default(),
            batch_interval_ms: None,
        }
    }
}

impl Default for UltraSwapConfig {
    fn default() -> Self {
        Self {
            cu_limit_multiplier: default_cu_limit_multiplier(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LegRole {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LoggingProfile {
    Lean,
    Verbose,
}

impl Default for LoggingProfile {
    fn default() -> Self {
        Self::Lean
    }
}

impl LoggingProfile {
    pub fn is_verbose(self) -> bool {
        matches!(self, Self::Verbose)
    }

    pub fn is_lean(self) -> bool {
        matches!(self, Self::Lean)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "super::default_logging_level")]
    pub level: String,
    #[serde(default)]
    pub json: bool,
    #[serde(default = "super::default_logging_profile")]
    pub profile: LoggingProfile,
    #[serde(default = "super::default_slow_quote_warn_ms")]
    pub slow_quote_warn_ms: u64,
    #[serde(default = "super::default_slow_swap_warn_ms")]
    pub slow_swap_warn_ms: u64,
    #[serde(default = "super::default_timezone_offset_hours")]
    pub timezone_offset_hours: i8,
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
    #[serde(default)]
    pub cpu_affinity: CpuAffinityConfig,
    #[serde(default = "super::default_quote_timeout_ms")]
    pub quote_ms: u64,
    #[serde(default)]
    pub swap_ms: Option<u64>,
    #[serde(default)]
    pub landing_ms: Option<u64>,
    #[serde(default)]
    pub auto_restart_minutes: u64,
    #[serde(default)]
    pub get_block_hash_by_grpc: bool,
    #[serde(default)]
    pub enable_simulation: bool,
    #[serde(default = "super::default_true")]
    pub show_jupiter_logs: bool,
    #[serde(default)]
    pub dry_run: bool,
    #[serde(default)]
    pub prometheus: PrometheusConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub auto_refresh_wallet_minute: u64,
    #[serde(default)]
    pub light_house: LightHouseBotConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CpuAffinityConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub worker_cores: Vec<usize>,
    #[serde(default)]
    pub max_blocking_threads: Option<usize>,
    #[serde(default)]
    pub strict: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkConfig {
    #[serde(default)]
    pub enable_multiple_ip: bool,
    #[serde(default)]
    pub manual_ips: Vec<IpAddr>,
    #[serde(default)]
    pub blacklist_ips: Vec<IpAddr>,
    #[serde(default)]
    pub allow_loopback: bool,
    #[serde(default)]
    pub per_ip_inflight_limit: Option<u32>,
    #[serde(default)]
    pub cooldown_ms: NetworkCooldownConfig,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enable_multiple_ip: false,
            manual_ips: Vec::new(),
            blacklist_ips: Vec::new(),
            allow_loopback: false,
            per_ip_inflight_limit: None,
            cooldown_ms: NetworkCooldownConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct NetworkCooldownConfig {
    #[serde(default = "default_rate_limited_cooldown_ms")]
    pub rate_limited_start: u64,
    #[serde(default = "default_timeout_cooldown_ms")]
    pub timeout_start: u64,
}

impl Default for NetworkCooldownConfig {
    fn default() -> Self {
        Self {
            rate_limited_start: default_rate_limited_cooldown_ms(),
            timeout_start: default_timeout_cooldown_ms(),
        }
    }
}

fn default_rate_limited_cooldown_ms() -> u64 {
    500
}

fn default_timeout_cooldown_ms() -> u64 {
    250
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LightHouseBotConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub profit_guard_mints: Vec<String>,
    #[serde(default)]
    pub memory_slots: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FlashloanConfig {
    #[serde(default)]
    pub marginfi: FlashloanMarginfiConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FlashloanMarginfiConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub prefer_wallet_balance: bool,
    #[serde(default)]
    pub marginfi_account: Option<String>,
    #[serde(default = "super::default_flashloan_compute_unit_overhead")]
    pub compute_unit_overhead: u32,
}

impl Default for FlashloanMarginfiConfig {
    fn default() -> Self {
        Self {
            enable: false,
            prefer_wallet_balance: false,
            marginfi_account: None,
            compute_unit_overhead: super::default_flashloan_compute_unit_overhead(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlindStrategyConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub memo: String,
    #[serde(default)]
    pub enable_dexs: Vec<String>,
    #[serde(default)]
    pub exclude_dexes: Vec<String>,
    #[serde(default)]
    pub enable_landers: Vec<String>,
    #[serde(default)]
    pub auto_scale_to_ip: AutoScaleToIpConfig,
    #[serde(default)]
    pub base_mints: Vec<BlindBaseMintConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlindBaseMintConfig {
    #[serde(default)]
    pub mint: String,
    #[serde(default)]
    pub lanes: Vec<TradeSizeLaneConfig>,
    #[serde(default)]
    #[serde(alias = "min_quote_profit_lamports")]
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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AutoScaleToIpConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default = "default_auto_scale_multiplier")]
    pub max_multiplier: f64,
}

fn default_auto_scale_multiplier() -> f64 {
    3.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradeSizeLaneConfig {
    #[serde(deserialize_with = "deserialize_u64_value")]
    pub min: u64,
    #[serde(deserialize_with = "deserialize_u64_value")]
    pub max: u64,
    pub count: u32,
    #[serde(default)]
    pub strategy: TradeRangeStrategy,
    #[serde(default = "default_lane_weight")]
    pub weight: f64,
}

fn default_lane_weight() -> f64 {
    1.0
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeRangeStrategy {
    Linear,
    Exponential,
    Random,
}

impl Default for TradeRangeStrategy {
    fn default() -> Self {
        Self::Linear
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PureBlindStrategyConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub enable_landers: Vec<String>,
    #[serde(default = "super::default_one")]
    pub cu_multiplier: f64,
    #[serde(default)]
    pub market_cache: PureBlindMarketCacheConfig,
    #[serde(default)]
    pub assets: PureBlindAssetsConfig,
    #[serde(default)]
    pub overrides: Vec<PureBlindOverrideConfig>,
    #[serde(default)]
    pub monitoring: PureBlindMonitoringConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PureBlindMarketCacheConfig {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub download_url: String,
    #[serde(default)]
    pub proxy: Option<String>,
    #[serde(default)]
    pub auto_refresh_minutes: u64,
    #[serde(default)]
    pub exclude_other_dex_program_ids: bool,
    #[serde(default)]
    pub exclude_dex_program_ids: Vec<String>,
    #[serde(default)]
    pub min_liquidity_usd: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PureBlindAssetsConfig {
    #[serde(default)]
    pub base_mints: Vec<PureBlindBaseMintConfig>,
    #[serde(default)]
    pub intermediates: Vec<String>,
    #[serde(default)]
    pub blacklist_mints: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PureBlindBaseMintConfig {
    #[serde(default)]
    pub mint: String,
    #[serde(default, deserialize_with = "deserialize_trade_sizes")]
    pub trade_sizes: Vec<u64>,
    #[serde(default)]
    pub min_profit: Option<u64>,
    #[serde(default)]
    pub process_delay: Option<u64>,
    #[serde(default)]
    pub route_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PureBlindOverrideConfig {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub legs: Vec<PureBlindOverrideLegConfig>,
    #[serde(default)]
    pub lookup_tables: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_trade_sizes")]
    pub trade_sizes: Vec<u64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PureBlindOverrideLegConfig {
    pub market: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PureBlindMonitoringConfig {
    #[serde(default)]
    pub enable_metrics: bool,
    #[serde(default)]
    pub route_labels: bool,
}

fn deserialize_trade_size_range<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RawValue {
        Number(u64),
        String(String),
    }

    let raw = Vec::<RawValue>::deserialize(deserializer)?;
    let mut values = Vec::with_capacity(raw.len());
    for entry in raw {
        match entry {
            RawValue::Number(value) => values.push(value),
            RawValue::String(text) => {
                let normalized = text.replace('_', "").trim().to_string();
                if normalized.is_empty() {
                    continue;
                }
                let parsed = normalized.parse::<u64>().map_err(|err| {
                    D::Error::custom(format!(
                        "invalid trade size `{text}`: failed to parse u64 ({err})"
                    ))
                })?;
                values.push(parsed);
            }
        }
    }
    Ok(values)
}

fn deserialize_trade_sizes<'de, D>(deserializer: D) -> Result<Vec<u64>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_trade_size_range(deserializer)
}

fn deserialize_u64_value<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Raw {
        Number(u64),
        String(String),
    }

    match Raw::deserialize(deserializer)? {
        Raw::Number(value) => Ok(value),
        Raw::String(text) => {
            let normalized = text.replace('_', "").trim().to_string();
            if normalized.is_empty() {
                return Ok(0);
            }
            normalized
                .parse::<u64>()
                .map_err(|err| D::Error::custom(format!("invalid u64 value `{text}`: {err}")))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BackRunStrategyConfig {
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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CopyStrategyConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub copy_dispatch: CopyDispatchConfig,
    #[serde(default)]
    pub wallets: Vec<CopyWalletConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CopyWalletConfig {
    pub address: String,
    #[serde(default)]
    pub source: CopySourceConfig,
    #[serde(default = "default_copy_cu_limit_multiplier")]
    pub cu_limit_multiplier: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CopySourceConfig {
    #[serde(rename = "type")]
    #[serde(default)]
    pub kind: CopySourceKind,
    #[serde(default)]
    pub rpc: CopyRpcConfig,
    #[serde(default)]
    pub grpc: CopyGrpcConfig,
    #[serde(default)]
    pub enable_landers: Vec<String>,
}

impl Default for CopySourceConfig {
    fn default() -> Self {
        Self {
            kind: CopySourceKind::Rpc,
            rpc: CopyRpcConfig::default(),
            grpc: CopyGrpcConfig::default(),
            enable_landers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CopySourceKind {
    Rpc,
    Grpc,
}

impl Default for CopySourceKind {
    fn default() -> Self {
        Self::Rpc
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CopyRpcConfig {
    #[serde(default = "default_copy_pull_interval_minutes")]
    pub pull_interval_minutes: u64,
    #[serde(default = "default_copy_pull_count")]
    pub pull_count: u64,
    #[serde(default)]
    pub pull_endpoints: Vec<String>,
}

impl Default for CopyRpcConfig {
    fn default() -> Self {
        Self {
            pull_interval_minutes: default_copy_pull_interval_minutes(),
            pull_count: default_copy_pull_count(),
            pull_endpoints: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CopyGrpcConfig {
    #[serde(default)]
    pub yellowstone_grpc_url: String,
    #[serde(default)]
    pub yellowstone_grpc_token: String,
    #[serde(default)]
    pub include_program_ids: Vec<String>,
    #[serde(default)]
    pub exclude_program_ids: Vec<String>,
}

impl Default for CopyGrpcConfig {
    fn default() -> Self {
        Self {
            yellowstone_grpc_url: String::new(),
            yellowstone_grpc_token: String::new(),
            include_program_ids: Vec::new(),
            exclude_program_ids: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct CopyDispatchConfig {
    #[serde(default = "default_copy_dispatch_mode")]
    pub mode: CopyDispatchMode,
    #[serde(default = "default_copy_dispatch_max_inflight")]
    pub max_inflight: u32,
    #[serde(default = "default_copy_dispatch_queue_capacity")]
    pub queue_capacity: u32,
    #[serde(default = "default_copy_dispatch_queue_worker_count")]
    pub queue_worker_count: u32,
    #[serde(default = "default_copy_dispatch_replay_interval_ms")]
    pub replay_interval_ms: u64,
    #[serde(default = "default_copy_dispatch_queue_send_interval_ms")]
    pub queue_send_interval_ms: u64,
    #[serde(default = "default_copy_dispatch_fanout_count")]
    pub fanout_count: u32,
}

impl Default for CopyDispatchConfig {
    fn default() -> Self {
        Self {
            mode: default_copy_dispatch_mode(),
            max_inflight: default_copy_dispatch_max_inflight(),
            queue_capacity: default_copy_dispatch_queue_capacity(),
            queue_worker_count: default_copy_dispatch_queue_worker_count(),
            replay_interval_ms: default_copy_dispatch_replay_interval_ms(),
            queue_send_interval_ms: default_copy_dispatch_queue_send_interval_ms(),
            fanout_count: default_copy_dispatch_fanout_count(),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CopyDispatchMode {
    Parallel,
    Queued,
}

impl Default for CopyDispatchMode {
    fn default() -> Self {
        Self::Parallel
    }
}

const fn default_copy_dispatch_mode() -> CopyDispatchMode {
    CopyDispatchMode::Parallel
}

const fn default_copy_dispatch_max_inflight() -> u32 {
    32
}

const fn default_copy_dispatch_queue_capacity() -> u32 {
    256
}

const fn default_copy_dispatch_queue_worker_count() -> u32 {
    1
}

const fn default_copy_dispatch_replay_interval_ms() -> u64 {
    100
}

const fn default_copy_dispatch_queue_send_interval_ms() -> u64 {
    0
}

const fn default_copy_dispatch_fanout_count() -> u32 {
    1
}

const fn default_copy_cu_limit_multiplier() -> f64 {
    1.0
}

const fn default_copy_pull_interval_minutes() -> u64 {
    10
}

const fn default_copy_pull_count() -> u64 {
    100
}
#[derive(Debug, Clone, Deserialize)]
pub struct PrometheusConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default = "super::default_prometheus_listen")]
    pub listen: String,
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
    #[serde(default = "super::default_auto_download_market_cache")]
    pub auto_download_market_cache: bool,
    #[serde(default = "super::default_market_cache")]
    pub market_cache: String,
    #[serde(default = "super::default_market_cache_download_url")]
    pub market_cache_download_url: String,
    #[serde(default)]
    pub exclude_other_dex_program_ids: bool,
    #[serde(default = "super::default_market_mode")]
    pub market_mode: MarketMode,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JupiterLaunchConfig {
    #[serde(default = "super::default_true")]
    pub allow_circular_arbitrage: bool,
    #[serde(default = "super::default_true")]
    pub enable_new_dexes: bool,
    #[serde(default)]
    pub enable_add_market: bool,
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

    pub fn effective_args(
        &self,
        overrides: &LaunchOverrides,
        market_cache_override: Option<&str>,
    ) -> Vec<String> {
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

        if let Some(local_cache) = market_cache_override {
            args.push("--market-cache".to_string());
            args.push(local_cache.to_string());
        } else if !core.market_cache.trim().is_empty() {
            args.push("--market-cache".to_string());
            args.push(core.market_cache.clone());
        }

        let market_mode = if core.use_local_market_cache {
            MarketMode::File
        } else {
            core.market_mode
        };

        args.push("--market-mode".to_string());
        args.push(market_mode.as_str().to_string());

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
        if launch.enable_add_market {
            args.push("--enable-add-market".to_string());
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

#[derive(Debug, Clone, Deserialize)]
pub struct HealthCheckConfig {
    #[serde(default = "super::default_health_check_interval_secs")]
    pub interval_secs: u64,
    #[serde(default = "super::default_health_check_max_wait_secs")]
    pub max_wait_secs: u64,
    #[serde(default = "super::default_health_check_retry_count")]
    pub retry_count: u32,
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
    #[serde(default = "super::default_compute_unit_price_strategy")]
    pub compute_unit_price_strategy: String,
    #[serde(default)]
    pub sending_strategy: DispatchStrategy,
    #[serde(default)]
    pub fixed_compute_unit_price: Option<u64>,
    #[serde(default)]
    pub random_compute_unit_price_range: Vec<u64>,
    #[serde(default)]
    pub jito: Option<LanderJitoConfig>,
    #[serde(default)]
    pub staked: Option<LanderEndpointConfig>,
    #[serde(default)]
    pub temporal: Option<LanderEndpointConfig>,
    #[serde(default)]
    pub astralane: Option<LanderEndpointConfig>,
    #[serde(default)]
    pub skip_preflight: Option<bool>,
    #[serde(default)]
    pub max_retries: Option<usize>,
    #[serde(default)]
    pub min_context_slot: Option<u64>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct LanderJitoConfig {
    #[serde(default)]
    pub endpoints: Vec<String>,
    #[serde(default = "super::default_tip_strategy")]
    pub tip_strategy: TipStrategyKind,
    #[serde(default)]
    pub fixed_tip: Option<u64>,
    #[serde(default)]
    pub range_tips: Vec<u64>,
    #[serde(default)]
    pub stream_tip_level: Option<TipStreamLevel>,
    #[serde(default)]
    pub max_stream_tip_lamports: Option<u64>,
    #[serde(default)]
    pub uuid_config: Vec<LanderJitoUuidConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipStrategyKind {
    Fixed,
    Range,
    Stream,
}

impl Default for TipStrategyKind {
    fn default() -> Self {
        TipStrategyKind::Fixed
    }
}

impl<'de> Deserialize<'de> for TipStrategyKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KindVisitor;

        impl<'de> Visitor<'de> for KindVisitor {
            type Value = TipStrategyKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("one of: fixed, range, stream")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                match value.trim().to_ascii_lowercase().as_str() {
                    "fixed" => Ok(TipStrategyKind::Fixed),
                    "range" => Ok(TipStrategyKind::Range),
                    "stream" => Ok(TipStrategyKind::Stream),
                    other => Err(DeError::unknown_variant(
                        other,
                        &["fixed", "range", "stream"],
                    )),
                }
            }
        }

        deserializer.deserialize_str(KindVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipStreamLevel {
    Percentile25,
    Percentile50,
    Percentile75,
    Percentile95,
    Percentile99,
    Ema50,
}

impl TipStreamLevel {
    pub fn field_name(self) -> &'static str {
        match self {
            TipStreamLevel::Percentile25 => "landed_tips_25th_percentile",
            TipStreamLevel::Percentile50 => "landed_tips_50th_percentile",
            TipStreamLevel::Percentile75 => "landed_tips_75th_percentile",
            TipStreamLevel::Percentile95 => "landed_tips_95th_percentile",
            TipStreamLevel::Percentile99 => "landed_tips_99th_percentile",
            TipStreamLevel::Ema50 => "ema_landed_tips_50th_percentile",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            TipStreamLevel::Percentile25 => "25th",
            TipStreamLevel::Percentile50 => "50th",
            TipStreamLevel::Percentile75 => "75th",
            TipStreamLevel::Percentile95 => "95th",
            TipStreamLevel::Percentile99 => "99th",
            TipStreamLevel::Ema50 => "ema50",
        }
    }
}

impl<'de> Deserialize<'de> for TipStreamLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LevelVisitor;

        impl<'de> Visitor<'de> for LevelVisitor {
            type Value = TipStreamLevel;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("one of: 25th, 50th, 75th, 95th, 99th, ema50")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                match value.trim().to_ascii_lowercase().as_str() {
                    "25th" => Ok(TipStreamLevel::Percentile25),
                    "50th" => Ok(TipStreamLevel::Percentile50),
                    "75th" => Ok(TipStreamLevel::Percentile75),
                    "95th" => Ok(TipStreamLevel::Percentile95),
                    "99th" => Ok(TipStreamLevel::Percentile99),
                    "ema50" => Ok(TipStreamLevel::Ema50),
                    _other => Err(DeError::invalid_value(Unexpected::Str(value), &self)),
                }
            }
        }

        deserializer.deserialize_str(LevelVisitor)
    }
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
