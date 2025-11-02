use anyhow::Result;
use borsh::BorshSerialize;

use super::types::EncodedSwap;

/// ZeroFi swap 编码器封装。
#[derive(Debug, Clone, Copy, Default)]
pub struct ZeroFiSwap;

impl ZeroFiSwap {
    pub fn encode() -> Result<EncodedSwap> {
        EncodedSwap::from_name("ZeroFi", &())
    }
}

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

/// Obric swap 编码器封装。
#[derive(Debug, Clone, Copy)]
pub struct ObricSwap {
    pub x_to_y: bool,
}

impl ObricSwap {
    pub fn encode(&self) -> Result<EncodedSwap> {
        EncodedSwap::from_name("Obric", &self.x_to_y)
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

/// Raydium CLMM swap 编码器封装。
#[derive(Debug, Clone, Copy, Default)]
pub struct RaydiumClmmSwap;

impl RaydiumClmmSwap {
    pub fn encode() -> Result<EncodedSwap> {
        EncodedSwap::from_name("RaydiumClmm", &())
    }
}

/// Raydium CLMM v2 swap（Token-2022 支持）。
#[derive(Debug, Clone, Copy, Default)]
pub struct RaydiumClmmSwapV2;

impl RaydiumClmmSwapV2 {
    pub fn encode() -> Result<EncodedSwap> {
        EncodedSwap::from_name("RaydiumClmmV2", &())
    }
}

/// Meteora DLMM swap 编码器封装。
#[derive(Debug, Clone, Copy, Default)]
pub struct MeteoraDlmmSwap;

impl MeteoraDlmmSwap {
    pub fn encode() -> Result<EncodedSwap> {
        EncodedSwap::from_name("MeteoraDlmm", &())
    }
}

/// Meteora DLMM v2 swap（Token-2022 / Transfer Hook）。
#[derive(Debug, Clone, Copy, Default)]
pub struct MeteoraDlmmSwapV2;

impl MeteoraDlmmSwapV2 {
    pub fn encode_default() -> Result<EncodedSwap> {
        let payload = MeteoraDlmmSwapV2Payload {
            remaining_accounts_info: RemainingAccountsInfoPayload::default(),
        };
        EncodedSwap::from_name("MeteoraDlmmSwapV2", &payload)
    }
}

#[derive(BorshSerialize, Default)]
struct MeteoraDlmmSwapV2Payload {
    remaining_accounts_info: RemainingAccountsInfoPayload,
}

/// Whirlpool swap 编码器封装。
#[derive(Debug, Clone, Copy)]
pub struct WhirlpoolSwap {
    pub a_to_b: bool,
}

impl WhirlpoolSwap {
    pub fn encode(&self) -> Result<EncodedSwap> {
        EncodedSwap::from_name("Whirlpool", &self.a_to_b)
    }
}

/// Whirlpool v2 swap，可附带 transfer-hook 相关剩余账户。
#[derive(Debug, Clone)]
pub struct WhirlpoolSwapV2 {
    pub a_to_b: bool,
    pub remaining_accounts: Option<RemainingAccountsInfoPayload>,
}

impl WhirlpoolSwapV2 {
    pub fn encode(&self) -> Result<EncodedSwap> {
        let payload = WhirlpoolSwapV2Payload {
            a_to_b: self.a_to_b,
            remaining_accounts_info: self.remaining_accounts.clone(),
        };
        EncodedSwap::from_name("WhirlpoolSwapV2", &payload)
    }
}

#[derive(BorshSerialize)]
struct WhirlpoolSwapV2Payload {
    a_to_b: bool,
    remaining_accounts_info: Option<RemainingAccountsInfoPayload>,
}

#[derive(Debug, Clone, Default, BorshSerialize)]
pub struct RemainingAccountsInfoPayload {
    pub slices: Vec<RemainingAccountsSlicePayload>,
}

#[derive(Debug, Clone, BorshSerialize)]
pub struct RemainingAccountsSlicePayload {
    pub accounts_type: u8,
    pub length: u8,
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
    fn encode_zero_fi_swap() {
        let encoded = ZeroFiSwap::encode().expect("encode zerofi");
        assert_eq!(encoded.variant().unwrap(), "ZeroFi");
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

    #[test]
    fn encode_obric_swap() {
        let swap = ObricSwap { x_to_y: true };
        let encoded = swap.encode().expect("encode obric");
        assert_eq!(encoded.variant().unwrap(), "Obric");
    }

    #[test]
    fn encode_raydium_clmm_swap() {
        let encoded = RaydiumClmmSwap::encode().expect("encode raydium clmm");
        assert_eq!(encoded.variant().unwrap(), "RaydiumClmm");
    }

    #[test]
    fn encode_meteora_dlmm_swap() {
        let encoded = MeteoraDlmmSwap::encode().expect("encode meteora dlmm");
        assert_eq!(encoded.variant().unwrap(), "MeteoraDlmm");
    }

    #[test]
    fn encode_whirlpool_swap() {
        let swap = WhirlpoolSwap { a_to_b: true };
        let encoded = swap.encode().expect("encode whirlpool");
        assert_eq!(encoded.variant().unwrap(), "Whirlpool");
    }

    #[test]
    fn encode_raydium_clmm_swap_v2() {
        let encoded = RaydiumClmmSwapV2::encode().expect("encode raydium clmm v2");
        assert_eq!(encoded.variant().unwrap(), "RaydiumClmmV2");
    }

    #[test]
    fn encode_meteora_dlmm_swap_v2() {
        let encoded = MeteoraDlmmSwapV2::encode_default().expect("encode meteora dlmm v2");
        assert_eq!(encoded.variant().unwrap(), "MeteoraDlmmSwapV2");
    }

    #[test]
    fn encode_whirlpool_swap_v2() {
        let swap = WhirlpoolSwapV2 {
            a_to_b: false,
            remaining_accounts: None,
        };
        let encoded = swap.encode().expect("encode whirlpool v2");
        assert_eq!(encoded.variant().unwrap(), "WhirlpoolSwapV2");
    }
}
