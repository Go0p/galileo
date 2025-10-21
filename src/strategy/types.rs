use std::fmt;
use std::str::FromStr;
use std::sync::Arc;

use anyhow::{Result, anyhow};
use solana_sdk::pubkey::Pubkey;

use crate::dexes::humidifi::HumidiFiMarketMeta;
use crate::dexes::solfi_v2::SolfiV2MarketMeta;
use crate::dexes::tessera_v::TesseraVMarketMeta;
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
}

impl BlindDex {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SolFiV2 => "SolFiV2",
            Self::HumidiFi => "HumidiFi",
            Self::TesseraV => "TesseraV",
            Self::ZeroFi => "ZeroFi",
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
            other => anyhow::bail!("不支持的盲发 DEX: {other}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlindStep {
    pub dex: BlindDex,
    pub market: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
    pub meta: BlindMarketMeta,
    pub direction: BlindSwapDirection,
}

#[derive(Debug, Clone)]
pub struct BlindOrder {
    pub amount_in: u64,
    pub steps: Vec<BlindStep>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlindSwapDirection {
    QuoteToBase,
    BaseToQuote,
}

#[derive(Debug, Clone)]
pub struct BlindRoutePlan {
    pub forward: Vec<BlindStep>,
    pub reverse: Vec<BlindStep>,
}

#[derive(Debug, Clone)]
pub enum BlindMarketMeta {
    HumidiFi(Arc<HumidiFiMarketMeta>),
    SolFiV2(Arc<SolfiV2MarketMeta>),
    TesseraV(Arc<TesseraVMarketMeta>),
    ZeroFi(Arc<ZeroFiMarketMeta>),
}
