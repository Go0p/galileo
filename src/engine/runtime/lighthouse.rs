use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use reqwest::Client;
use solana_sdk::pubkey::Pubkey;
use tracing::{debug, warn};

use crate::engine::{EngineResult, LighthouseSettings, SolPriceFeedSettings};

const MIN_LIGHTHOUSE_MEMORY_SLOTS: usize = 1;
const MAX_LIGHTHOUSE_MEMORY_SLOTS: usize = 128;
const LAMPORTS_PER_SOL: u64 = 1_000_000_000;
const WSOL_MINT: Pubkey = solana_sdk::pubkey!("So11111111111111111111111111111111111111112");
const USDC_MINT: Pubkey = solana_sdk::pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

pub(crate) struct LighthouseRuntime {
    pub(super) enabled: bool,
    guard_assets: HashMap<Pubkey, GuardAssetConfig>,
    pub(super) memory_slots: usize,
    pub(super) available_ids: Vec<u8>,
    pub(super) cursor: usize,
    sol_price_feed: Option<SolPriceFeed>,
}

impl LighthouseRuntime {
    pub(crate) fn new(settings: &LighthouseSettings, ip_capacity_hint: usize) -> Self {
        let guard_mints: HashSet<Pubkey> = settings.profit_guard_mints.iter().copied().collect();
        let enabled = settings.enable && !guard_mints.is_empty();

        let mut available_ids: Vec<u8> = settings.existing_memory_ids.clone();
        available_ids.sort_unstable();
        available_ids.dedup();

        let derived_slots = if !available_ids.is_empty() {
            available_ids.len()
        } else {
            ip_capacity_hint
                .max(MIN_LIGHTHOUSE_MEMORY_SLOTS)
                .min(MAX_LIGHTHOUSE_MEMORY_SLOTS)
        };
        let configured_slots = settings.memory_slots.map(|value| usize::from(value.max(1)));
        let slot_count = configured_slots.unwrap_or(derived_slots);
        let memory_slots = slot_count
            .max(available_ids.len())
            .max(MIN_LIGHTHOUSE_MEMORY_SLOTS)
            .min(MAX_LIGHTHOUSE_MEMORY_SLOTS);

        let mut guard_assets = HashMap::new();
        for mint in &guard_mints {
            if let Some(config) = GuardAssetConfig::infer_from_mint(*mint) {
                guard_assets.insert(*mint, config);
            } else {
                warn!(
                    target: "engine::lighthouse",
                    mint = %mint,
                    "profit guard 未识别的 mint，默认使用 9 位精度并按本币计价"
                );
                guard_assets.insert(
                    *mint,
                    GuardAssetConfig {
                        decimals: 9,
                        denomination: GuardDenomination::Native,
                    },
                );
            }
        }

        let sol_price_feed = settings
            .sol_price_feed
            .as_ref()
            .map(SolPriceFeed::from_settings);

        Self {
            enabled,
            guard_assets,
            memory_slots,
            available_ids,
            cursor: 0,
            sol_price_feed,
        }
    }

    pub(crate) fn should_guard(&self, mint: &Pubkey) -> bool {
        self.enabled && self.guard_assets.contains_key(mint)
    }

    pub(crate) fn next_memory_id(&mut self) -> u8 {
        if !self.enabled {
            return 0;
        }
        if self.available_ids.is_empty() {
            let id = 0u8;
            if self.memory_slots > 1 {
                self.available_ids
                    .reserve(self.memory_slots.saturating_sub(1));
            }
            self.available_ids.push(id);
            return id;
        }

        let idx = if self.available_ids.len() == 1 {
            0
        } else {
            let current = self.cursor % self.available_ids.len();
            self.cursor = (self.cursor + 1) % self.available_ids.len();
            current
        };
        self.available_ids[idx]
    }

    pub(crate) async fn guard_amount_for(
        &mut self,
        mint: &Pubkey,
        lamports_required: u64,
    ) -> EngineResult<Option<u64>> {
        if lamports_required == 0 || !self.should_guard(mint) {
            return Ok(None);
        }
        let Some(config) = self.guard_assets.get(mint).copied() else {
            return Ok(None);
        };

        let amount = match config.denomination {
            GuardDenomination::Native => lamports_required,
            GuardDenomination::SolEquivalent => {
                let feed = self.sol_price_feed.as_mut().ok_or_else(|| {
                    crate::engine::EngineError::InvalidConfig(
                        "未配置 sol_usd 价格源，无法计算 USDC 守护阈值".to_string(),
                    )
                })?;
                let price = feed.latest().await?;
                convert_lamports_to_token(lamports_required, config.decimals, &price)
            }
        };

        Ok(Some(amount))
    }
}

#[derive(Clone, Copy)]
struct GuardAssetConfig {
    decimals: u8,
    denomination: GuardDenomination,
}

#[derive(Clone, Copy)]
enum GuardDenomination {
    Native,
    SolEquivalent,
}

impl GuardAssetConfig {
    fn infer_from_mint(mint: Pubkey) -> Option<Self> {
        if mint == WSOL_MINT {
            Some(Self {
                decimals: 9,
                denomination: GuardDenomination::Native,
            })
        } else if mint == USDC_MINT {
            Some(Self {
                decimals: 6,
                denomination: GuardDenomination::SolEquivalent,
            })
        } else {
            None
        }
    }
}

struct SolPriceFeed {
    client: Client,
    url: String,
    refresh: Duration,
    last_updated: Option<Instant>,
    last_price: Option<SolUsdPrice>,
}

impl SolPriceFeed {
    fn from_settings(settings: &SolPriceFeedSettings) -> Self {
        Self {
            client: Client::new(),
            url: settings.url.clone(),
            refresh: settings.refresh,
            last_updated: None,
            last_price: None,
        }
    }

    async fn latest(&mut self) -> EngineResult<SolUsdPrice> {
        let should_refresh = match self.last_updated {
            Some(instant) => instant.elapsed() >= self.refresh,
            None => true,
        };

        if should_refresh {
            let fetched = self.fetch().await?;
            self.last_price = Some(fetched);
            self.last_updated = Some(Instant::now());
        }

        self.last_price.as_ref().copied().ok_or_else(|| {
            crate::engine::EngineError::InvalidConfig("无法获取 sol_usd 价格".into())
        })
    }

    async fn fetch(&self) -> EngineResult<SolUsdPrice> {
        let response = self
            .client
            .get(&self.url)
            .send()
            .await?
            .error_for_status()?;

        let parsed: PythPriceResponse = response.json().await?;
        let price = parsed
            .parsed
            .get(0)
            .and_then(|entry| entry.price.as_ref())
            .ok_or_else(|| {
                crate::engine::EngineError::InvalidConfig("pyth 返回内容缺少价格字段".to_string())
            })?;

        let price_int = price
            .price
            .parse::<i64>()
            .map_err(|err| crate::engine::EngineError::InvalidConfig(err.to_string()))?;

        debug!(
            target: "engine::lighthouse",
            price = price_int,
            expo = price.expo,
            "sol_usd 价格已刷新"
        );

        Ok(SolUsdPrice {
            price: price_int,
            expo: price.expo,
        })
    }
}

#[derive(Clone, Copy)]
struct SolUsdPrice {
    price: i64,
    expo: i32,
}

#[derive(serde::Deserialize)]
struct PythPriceResponse {
    parsed: Vec<PythParsedPrice>,
}

#[derive(serde::Deserialize)]
struct PythParsedPrice {
    #[serde(default)]
    price: Option<PythPriceData>,
}

#[derive(serde::Deserialize)]
struct PythPriceData {
    price: String,
    expo: i32,
}

fn convert_lamports_to_token(lamports: u64, token_decimals: u8, price: &SolUsdPrice) -> u64 {
    if lamports == 0 {
        return 0;
    }

    let price_int = price.price as i128;
    let lamports_i128 = lamports as i128;
    let mut numerator = lamports_i128 * price_int;
    let mut denominator = LAMPORTS_PER_SOL as i128;
    let decimals_factor = 10_i128.pow(token_decimals as u32);
    numerator *= decimals_factor;

    if price.expo >= 0 {
        numerator *= 10_i128.pow(price.expo as u32);
    } else {
        denominator *= 10_i128.pow((-price.expo) as u32);
    }

    if numerator <= 0 {
        return 0;
    }

    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    let mut amount = if remainder == 0 {
        quotient
    } else {
        quotient + 1
    };

    if amount < 0 {
        amount = 0;
    }

    amount.min(u64::MAX as i128) as u64
}
