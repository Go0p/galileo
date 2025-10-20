use std::fmt;
use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;

use crate::dexes::solfi_v2::SolfiV2MarketMeta;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TradePair {
    pub input_mint: String,
    pub output_mint: String,
}

impl TradePair {
    pub fn reversed(&self) -> TradePair {
        TradePair {
            input_mint: self.output_mint.clone(),
            output_mint: self.input_mint.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlindDex {
    SolFiV2,
    HumidiFi,
    TesseraV,
}

impl BlindDex {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SolFiV2 => "SolFiV2",
            Self::HumidiFi => "HumidiFi",
            Self::TesseraV => "TesseraV",
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
            other => anyhow::bail!("不支持的盲发 DEX: {other}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlindStep {
    pub dex: BlindDex,
    pub market: Pubkey,
    pub meta: SolfiV2MarketMeta,
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
