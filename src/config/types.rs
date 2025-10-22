#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

use serde::Deserialize;
use serde::de::{Deserializer, Error as DeError, Unexpected, Visitor};
use serde_with::{OneOrMany, serde_as};

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
    pub back_run_strategy: BackRunStrategyConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub rpc_url: Option<String>,
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
    pub jupiter: JupiterEngineConfig,
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
pub struct FlashloanConfig {
    #[serde(default)]
    pub marginfi: FlashloanMarginfiConfig,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct FlashloanMarginfiConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub prefer_wallet_balance: bool,
    #[serde(default)]
    pub marginfi_account: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlindStrategyConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub pure_mode: bool,
    #[serde(default)]
    pub memo: String,
    #[serde(default)]
    pub enable_dexs: Vec<String>,
    #[serde(default)]
    pub enable_landers: Vec<String>,
    #[serde(default)]
    pub base_mints: Vec<BlindBaseMintConfig>,
    #[serde(default)]
    pub pure_routes: Vec<PureBlindRouteConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlindBaseMintConfig {
    #[serde(default)]
    pub mint: String,
    #[serde(default, deserialize_with = "deserialize_trade_size_range")]
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
pub struct PureBlindRouteConfig {
    pub buy_market: String,
    pub sell_market: String,
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

#[serde_as]
#[derive(Debug, Clone, Deserialize, Default)]
pub struct LanderJitoConfig {
    #[serde(default)]
    pub endpoints: Vec<String>,
    #[serde_as(deserialize_as = "OneOrMany<_>")]
    #[serde(default = "super::default_tip_strategies")]
    pub tip_strategies: Vec<TipStrategyKind>,
    #[serde(default)]
    pub fixed_tip: Option<u64>,
    #[serde(default)]
    pub range_tips: Vec<u64>,
    #[serde(default)]
    pub floor_tip_level: Option<TipFloorLevel>,
    #[serde(default)]
    pub max_floor_tip_lamports: Option<u64>,
    #[serde(default)]
    pub uuid_config: Vec<LanderJitoUuidConfig>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipStrategyKind {
    Fixed,
    Range,
    Floor,
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
                formatter.write_str("one of: fixed, range, floor")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                match value.trim().to_ascii_lowercase().as_str() {
                    "fixed" => Ok(TipStrategyKind::Fixed),
                    "range" => Ok(TipStrategyKind::Range),
                    "floor" => Ok(TipStrategyKind::Floor),
                    other => Err(DeError::unknown_variant(
                        other,
                        &["fixed", "range", "floor"],
                    )),
                }
            }
        }

        deserializer.deserialize_str(KindVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipFloorLevel {
    Percentile25,
    Percentile50,
    Percentile75,
    Percentile95,
    Percentile99,
    Ema50,
}

impl TipFloorLevel {
    pub fn field_name(self) -> &'static str {
        match self {
            TipFloorLevel::Percentile25 => "landed_tips_25th_percentile",
            TipFloorLevel::Percentile50 => "landed_tips_50th_percentile",
            TipFloorLevel::Percentile75 => "landed_tips_75th_percentile",
            TipFloorLevel::Percentile95 => "landed_tips_95th_percentile",
            TipFloorLevel::Percentile99 => "landed_tips_99th_percentile",
            TipFloorLevel::Ema50 => "ema_landed_tips_50th_percentile",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            TipFloorLevel::Percentile25 => "25th",
            TipFloorLevel::Percentile50 => "50th",
            TipFloorLevel::Percentile75 => "75th",
            TipFloorLevel::Percentile95 => "95th",
            TipFloorLevel::Percentile99 => "99th",
            TipFloorLevel::Ema50 => "ema50",
        }
    }
}

impl<'de> Deserialize<'de> for TipFloorLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct LevelVisitor;

        impl<'de> Visitor<'de> for LevelVisitor {
            type Value = TipFloorLevel;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("one of: 25th, 50th, 75th, 95th, 99th, ema50")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: DeError,
            {
                match value.trim().to_ascii_lowercase().as_str() {
                    "25th" => Ok(TipFloorLevel::Percentile25),
                    "50th" => Ok(TipFloorLevel::Percentile50),
                    "75th" => Ok(TipFloorLevel::Percentile75),
                    "95th" => Ok(TipFloorLevel::Percentile95),
                    "99th" => Ok(TipFloorLevel::Percentile99),
                    "ema50" => Ok(TipFloorLevel::Ema50),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tip_strategies_deserialize_from_string() {
        let yaml = r#"
endpoints: []
tip_strategies: fixed
"#;
        let cfg: LanderJitoConfig = serde_yaml::from_str(yaml).expect("parse config");
        assert_eq!(cfg.tip_strategies, vec![TipStrategyKind::Fixed]);
    }

    #[test]
    fn tip_strategies_deserialize_from_list() {
        let yaml = r#"
endpoints: []
tip_strategies:
  - range
  - floor
"#;
        let cfg: LanderJitoConfig = serde_yaml::from_str(yaml).expect("parse config");
        assert_eq!(
            cfg.tip_strategies,
            vec![TipStrategyKind::Range, TipStrategyKind::Floor]
        );
    }

    #[test]
    fn tip_floor_level_deserialize() {
        let level: TipFloorLevel = serde_yaml::from_str("\"95th\"").expect("parse level");
        assert_eq!(level, TipFloorLevel::Percentile95);
    }
}
