use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;

use crate::dexes::clmm::RaydiumClmmMarketMeta;
use crate::dexes::dlmm::MeteoraDlmmMarketMeta;
use crate::dexes::framework::SwapFlow;
use crate::dexes::humidifi::HumidiFiMarketMeta;
use crate::dexes::obric_v2::ObricV2MarketMeta;
use crate::dexes::solfi_v2::SolfiV2MarketMeta;
use crate::dexes::tessera_v::TesseraVMarketMeta;
use crate::dexes::whirlpool::WhirlpoolMarketMeta;
use crate::dexes::zerofi::ZeroFiMarketMeta;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TradePair {
    pub input_mint: String,
    pub output_mint: String,
    pub input_pubkey: Pubkey,
    pub output_pubkey: Pubkey,
}

impl TradePair {
    pub fn try_new(input_mint: impl AsRef<str>, output_mint: impl AsRef<str>) -> Result<Self> {
        let input_text = input_mint.as_ref().trim();
        let output_text = output_mint.as_ref().trim();
        if input_text.is_empty() {
            return Err(anyhow!("输入 mint 不能为空"));
        }
        if output_text.is_empty() {
            return Err(anyhow!("输出 mint 不能为空"));
        }

        let input_pubkey = Pubkey::from_str(input_text)
            .map_err(|err| anyhow!("输入 mint 无效 {}: {err}", input_text))?;
        let output_pubkey = Pubkey::from_str(output_text)
            .map_err(|err| anyhow!("输出 mint 无效 {}: {err}", output_text))?;

        Ok(Self {
            input_mint: input_text.to_string(),
            output_mint: output_text.to_string(),
            input_pubkey,
            output_pubkey,
        })
    }

    pub fn from_pubkeys(input_pubkey: Pubkey, output_pubkey: Pubkey) -> Self {
        Self {
            input_mint: input_pubkey.to_string(),
            output_mint: output_pubkey.to_string(),
            input_pubkey,
            output_pubkey,
        }
    }

    pub fn reversed(&self) -> TradePair {
        TradePair::from_pubkeys(self.output_pubkey, self.input_pubkey)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlindDex {
    SolFiV2,
    HumidiFi,
    TesseraV,
    ZeroFi,
    ObricV2,
    RaydiumClmm,
    MeteoraDlmm,
    Whirlpool,
}

impl BlindDex {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SolFiV2 => "SolFiV2",
            Self::HumidiFi => "HumidiFi",
            Self::TesseraV => "TesseraV",
            Self::ZeroFi => "ZeroFi",
            Self::ObricV2 => "ObricV2",
            Self::RaydiumClmm => "RaydiumClmm",
            Self::MeteoraDlmm => "MeteoraDlmm",
            Self::Whirlpool => "Whirlpool",
        }
    }

    pub fn default_cu_budget(&self) -> u32 {
        match self {
            Self::SolFiV2 => 90_000,
            Self::HumidiFi => 40_000,
            Self::TesseraV => 83_000,
            Self::ZeroFi => 46_000,
            Self::ObricV2 => 58_000,
            Self::RaydiumClmm => 180_000,
            Self::MeteoraDlmm => 180_000,
            Self::Whirlpool => 180_000,
        }
    }
}

impl fmt::Display for BlindDex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BlindDex {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "SolFiV2" => Ok(Self::SolFiV2),
            "HumidiFi" => Ok(Self::HumidiFi),
            "TesseraV" => Ok(Self::TesseraV),
            "ZeroFi" => Ok(Self::ZeroFi),
            "ObricV2" => Ok(Self::ObricV2),
            "RaydiumClmm" => Ok(Self::RaydiumClmm),
            "MeteoraDlmm" => Ok(Self::MeteoraDlmm),
            "Whirlpool" => Ok(Self::Whirlpool),
            other => anyhow::bail!("不支持的盲发 DEX: {other}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlindAsset {
    pub mint: Pubkey,
    pub token_program: Pubkey,
}

impl BlindAsset {
    pub fn new(mint: Pubkey, token_program: Pubkey) -> Self {
        Self {
            mint,
            token_program,
        }
    }
}

impl PartialEq for BlindAsset {
    fn eq(&self, other: &Self) -> bool {
        self.mint == other.mint && self.token_program == other.token_program
    }
}

impl Eq for BlindAsset {}

#[derive(Debug, Clone)]
pub struct BlindStep {
    pub dex: BlindDex,
    pub market: Pubkey,
    pub base: BlindAsset,
    pub quote: BlindAsset,
    pub input: BlindAsset,
    pub output: BlindAsset,
    pub meta: BlindMarketMeta,
    pub flow: SwapFlow,
}

#[derive(Debug, Clone)]
pub struct BlindOrder {
    pub amount_in: u64,
    pub steps: Vec<BlindStep>,
    pub lookup_tables: Vec<AddressLookupTableAccount>,
    pub min_profit: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteSource {
    Manual,
    Auto,
}

impl RouteSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::Auto => "auto",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlindRoutePlan {
    pub forward: Vec<BlindStep>,
    pub reverse: Vec<BlindStep>,
    pub lookup_tables: Vec<AddressLookupTableAccount>,
    pub label: String,
    pub source: RouteSource,
    pub min_profit: u64,
}

impl BlindRoutePlan {
    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn source(&self) -> RouteSource {
        self.source
    }

    pub fn min_profit(&self) -> u64 {
        self.min_profit
    }
}

#[derive(Debug, Clone)]
pub enum BlindMarketMeta {
    HumidiFi(Arc<HumidiFiMarketMeta>),
    SolFiV2(Arc<SolfiV2MarketMeta>),
    TesseraV(Arc<TesseraVMarketMeta>),
    ZeroFi(Arc<ZeroFiMarketMeta>),
    ObricV2(Arc<ObricV2MarketMeta>),
    RaydiumClmm(Arc<RaydiumClmmMarketMeta>),
    MeteoraDlmm(Arc<MeteoraDlmmMarketMeta>),
    Whirlpool(Arc<WhirlpoolMarketMeta>),
}
