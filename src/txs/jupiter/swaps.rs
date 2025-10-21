use anyhow::Result;
use borsh::BorshSerialize;

use super::types::EncodedSwap;

/// SolFiV2 swap 编码器封装。
#[derive(Debug, Clone, Copy)]
pub struct SolFiV2Swap {
    pub is_quote_to_base: bool,
}

impl SolFiV2Swap {
    pub fn encode(&self) -> Result<EncodedSwap> {
        EncodedSwap::from_name("SolFiV2", &self.is_quote_to_base)
    }
}

/// HumidiFi swap 编码器封装。
#[derive(Debug, Clone, Copy)]
pub struct HumidiFiSwap {
    pub swap_id: u64,
    pub is_base_to_quote: bool,
}

impl HumidiFiSwap {
    pub fn encode(&self) -> Result<EncodedSwap> {
        let payload = HumidiFiSwapPayload {
            swap_id: self.swap_id,
            is_base_to_quote: self.is_base_to_quote,
        };
        EncodedSwap::from_name("HumidiFi", &payload)
    }
}

#[derive(BorshSerialize)]
struct HumidiFiSwapPayload {
    swap_id: u64,
    is_base_to_quote: bool,
}

/// TesseraV swap 编码器封装。
#[derive(Debug, Clone, Copy)]
pub struct TesseraVSwap {
    pub side: TesseraVSide,
}

impl TesseraVSwap {
    pub fn encode(&self) -> Result<EncodedSwap> {
        let payload = TesseraSwapPayload {
            side: match self.side {
                TesseraVSide::Bid => TesseraSide::Bid,
                TesseraVSide::Ask => TesseraSide::Ask,
            },
        };
        EncodedSwap::from_name("TesseraV", &payload)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TesseraVSide {
    Bid,
    Ask,
}

#[derive(BorshSerialize)]
struct TesseraSwapPayload {
    side: TesseraSide,
}

#[derive(BorshSerialize)]
enum TesseraSide {
    Bid,
    Ask,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_sol_fi_swap() {
        let swap = SolFiV2Swap {
            is_quote_to_base: true,
        };
        let encoded = swap.encode().expect("encode solfi");
        assert_eq!(encoded.variant().unwrap(), "SolFiV2");
    }

    #[test]
    fn encode_humidifi_swap() {
        let swap = HumidiFiSwap {
            swap_id: 42,
            is_base_to_quote: false,
        };
        let encoded = swap.encode().expect("encode humidifi");
        assert_eq!(encoded.variant().unwrap(), "HumidiFi");
    }

    #[test]
    fn encode_tessera_swap() {
        let swap = TesseraVSwap {
            side: TesseraVSide::Bid,
        };
        let encoded = swap.encode().expect("encode tessera");
        assert_eq!(encoded.variant().unwrap(), "TesseraV");
    }
}
