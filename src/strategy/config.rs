use std::collections::BTreeSet;
use std::time::Duration;

use serde::Deserialize;
use serde_with::serde_as;

use super::types::TradePair;

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct StrategyConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub base_mint: Option<String>,
    #[serde(default)]
    pub quote_mints: Vec<String>,
    #[serde(default)]
    pub trade_pairs: Vec<TradePairConfig>,
    #[serde(default)]
    pub trade_range: Vec<u64>,
    #[serde(default)]
    pub trade_range_strategy: TradeRangeStrategy,
    #[serde(default = "StrategyConfig::default_min_profit_threshold")]
    pub min_profit_threshold_lamports: u64,
    #[serde(default)]
    pub max_tip_lamports: u64,
    #[serde(default = "StrategyConfig::default_slippage_bps")]
    pub slippage_bps: u16,
    #[serde(default)]
    pub only_direct_routes: bool,
    #[serde(default = "StrategyConfig::default_true")]
    pub restrict_intermediate_tokens: bool,
    #[serde(default)]
    pub quote_max_accounts: Option<u32>,
    #[serde(default)]
    pub bot: BotConfig,
    #[serde(default)]
    #[allow(dead_code)]
    pub blind: BlindConfig,
    #[serde(default)]
    #[allow(dead_code)]
    pub spam: SpamConfig,
    #[serde(default)]
    pub identity: StrategyIdentityConfig,
    #[serde(default)]
    #[allow(dead_code)]
    pub jito: JitoConfig,
}

impl Default for StrategyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            base_mint: None,
            quote_mints: Vec::new(),
            trade_pairs: Vec::new(),
            trade_range: vec![1_000_000], // default 0.001 token (assuming 6 decimals)
            trade_range_strategy: TradeRangeStrategy::default(),
            min_profit_threshold_lamports: Self::default_min_profit_threshold(),
            max_tip_lamports: 0,
            slippage_bps: Self::default_slippage_bps(),
            only_direct_routes: false,
            restrict_intermediate_tokens: Self::default_true(),
            quote_max_accounts: None,
            bot: BotConfig::default(),
            blind: BlindConfig::default(),
            spam: SpamConfig::default(),
            identity: StrategyIdentityConfig::default(),
            jito: JitoConfig::default(),
        }
    }
}

impl StrategyConfig {
    fn default_min_profit_threshold() -> u64 {
        0
    }

    fn default_slippage_bps() -> u16 {
        50
    }

    fn default_true() -> bool {
        true
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn resolved_pairs(&self) -> Vec<TradePair> {
        if !self.trade_pairs.is_empty() {
            return self
                .trade_pairs
                .iter()
                .map(|p| TradePair {
                    input_mint: p.input_mint.clone(),
                    output_mint: p.output_mint.clone(),
                })
                .collect();
        }

        match (&self.base_mint, self.quote_mints.is_empty()) {
            (Some(base), false) => self
                .quote_mints
                .iter()
                .map(|q| TradePair {
                    input_mint: q.clone(),
                    output_mint: base.clone(),
                })
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn effective_trade_amounts(&self) -> Vec<u64> {
        if !self.trade_range.is_empty() {
            let set: BTreeSet<u64> = self.trade_range.iter().copied().collect();
            return set.iter().copied().collect();
        }

        if self.trade_range_strategy.enable_strategy {
            let mut set: BTreeSet<u64> = BTreeSet::new();
            for bucket in &self.trade_range_strategy.ranges {
                let mut current = bucket.from;
                let end = bucket.to.max(bucket.from);
                let step = bucket.step.max(1);
                while current <= end {
                    set.insert(current);
                    if current == u64::MAX {
                        break;
                    }
                    match current.checked_add(step) {
                        Some(next) => current = next,
                        None => break,
                    }
                }
            }
            return set.into_iter().collect();
        }

        vec![1_000_000]
    }

    pub fn trade_delay(&self) -> Duration {
        let delay = self.bot.over_trade_process_delay_ms.unwrap_or(200).max(1);
        Duration::from_millis(delay as u64)
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TradePairConfig {
    pub input_mint: String,
    pub output_mint: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradeRangeStrategy {
    #[serde(default)]
    pub enable_strategy: bool,
    #[serde(default)]
    pub ranges: Vec<TradeRangeBucket>,
}

impl Default for TradeRangeStrategy {
    fn default() -> Self {
        Self {
            enable_strategy: false,
            ranges: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TradeRangeBucket {
    #[serde(default)]
    pub from: u64,
    #[serde(default)]
    pub to: u64,
    #[serde(default = "TradeRangeBucket::default_step")]
    pub step: u64,
    #[serde(default)]
    #[allow(dead_code)]
    pub size: u32,
}

impl TradeRangeBucket {
    fn default_step() -> u64 {
        1
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    #[serde(default)]
    pub only_quote_dexs: Vec<String>,
    #[serde(default)]
    pub enable_reverse_trade: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub enable_random_base_mint: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub enable_sandwich_mitigation: bool,
    #[serde(default)]
    pub over_trade_process_delay_ms: Option<u64>,
    #[serde(default)]
    pub static_tip_config: StaticTipConfig,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            only_quote_dexs: Vec::new(),
            enable_reverse_trade: false,
            enable_random_base_mint: false,
            enable_sandwich_mitigation: true,
            over_trade_process_delay_ms: Some(200),
            static_tip_config: StaticTipConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct StaticTipConfig {
    #[serde(default)]
    pub enable_random: bool,
    #[serde(default = "StaticTipConfig::default_static_tip_percentage")]
    pub static_tip_percentage: f64,
    #[serde(default)]
    pub random_percentage: Vec<f64>,
}

impl Default for StaticTipConfig {
    fn default() -> Self {
        Self {
            enable_random: false,
            static_tip_percentage: Self::default_static_tip_percentage(),
            random_percentage: vec![0.5],
        }
    }
}

impl StaticTipConfig {
    fn default_static_tip_percentage() -> f64 {
        0.5
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct BlindConfig {
    #[serde(default)]
    pub only_quote_dexs: Vec<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub dynamic_au_jito_tip: bool,
    #[serde(default)]
    pub log_jito_tip_update: bool,
    #[serde(default)]
    pub static_jito_tip: Vec<u64>,
    #[serde(default)]
    pub dynamic_jito_tip_percentile: Vec<String>,
}

impl Default for BlindConfig {
    fn default() -> Self {
        Self {
            only_quote_dexs: Vec::new(),
            enabled: false,
            dynamic_au_jito_tip: false,
            log_jito_tip_update: false,
            static_jito_tip: Vec::new(),
            dynamic_jito_tip_percentile: Vec::new(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct SpamConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub enable_trade_log: bool,
    #[serde(default)]
    pub skip_preflight: bool,
    #[serde(default)]
    pub node1_config: Option<SpamProviderConfig>,
    #[serde(default)]
    pub helius_config: Option<SpamProviderConfig>,
    #[serde(default)]
    pub astralane_config: Option<SpamProviderConfig>,
    #[serde(default)]
    pub normal_rpc_config: Option<SpamRpcConfig>,
    #[serde(default)]
    pub nonce_config: Option<SpamNonceConfig>,
}

impl Default for SpamConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            enable_trade_log: false,
            skip_preflight: true,
            node1_config: None,
            helius_config: None,
            astralane_config: None,
            normal_rpc_config: None,
            nonce_config: None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct SpamProviderConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub only_back_run_active: bool,
    pub url: Option<String>,
    #[serde(default)]
    pub api_key: Option<String>,
    #[serde(default)]
    pub trigger_mint_profit_sol: f64,
    #[serde(default)]
    pub gas_amount_sol: f64,
    #[serde(default)]
    pub compute_unit_price_sol: f64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct SpamRpcConfig {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub only_back_run_active: bool,
    #[serde(default)]
    pub compute_unit_price_sol: f64,
    #[serde(default)]
    pub trigger_mint_profit_sol: f64,
    #[serde(default)]
    pub rpcs: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct SpamNonceConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub max_accounts: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct StrategyIdentityConfig {
    #[serde(default)]
    pub user_pubkey: Option<String>,
    #[serde(default)]
    pub fee_account: Option<String>,
    #[serde(default)]
    pub wrap_and_unwrap_sol: bool,
    #[serde(default)]
    pub use_shared_accounts: bool,
    #[serde(default)]
    pub compute_unit_price_micro_lamports: Option<u64>,
    #[serde(default)]
    pub skip_user_accounts_rpc_calls: Option<bool>,
    #[serde(default)]
    #[allow(dead_code)]
    pub tip_account: Option<String>,
}

impl Default for StrategyIdentityConfig {
    fn default() -> Self {
        Self {
            user_pubkey: None,
            fee_account: None,
            wrap_and_unwrap_sol: false,
            use_shared_accounts: false,
            compute_unit_price_micro_lamports: None,
            skip_user_accounts_rpc_calls: Some(true),
            tip_account: None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct JitoConfig {
    #[serde(default)]
    pub engine_urls: Vec<String>,
    #[serde(default)]
    pub random_engine: bool,
    #[serde(default)]
    pub uuid_config: Vec<JitoUuidConfig>,
}

impl Default for JitoConfig {
    fn default() -> Self {
        Self {
            engine_urls: Vec::new(),
            random_engine: true,
            uuid_config: Vec::new(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct JitoUuidConfig {
    pub uuid: String,
    #[serde(default)]
    pub rate_limit: u32,
}
