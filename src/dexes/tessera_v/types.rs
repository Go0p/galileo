use std::convert::TryInto;
use anyhow::{ensure, Result};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoolState {
    pub pool_id: u32,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
}

impl PoolState {
    const POOL_ID_OFFSET: usize = 0x10;
    const BASE_MINT_OFFSET: usize = 0x18;
    const QUOTE_MINT_OFFSET: usize = 0x38;
    const PUBKEY_LEN: usize = 32;

    pub fn parse(data: &[u8]) -> Result<Self> {
        let required = Self::QUOTE_MINT_OFFSET + Self::PUBKEY_LEN;
        ensure!(
            data.len() >= required,
            "tessera pool account too small: {} bytes (need at least {required})",
            data.len()
        );

        let pool_id = u32::from_le_bytes(
            data[Self::POOL_ID_OFFSET..Self::POOL_ID_OFFSET + 4]
                .try_into()
                .expect("slice length verified"),
        );
        let base_mint = Pubkey::new_from_array(
            data[Self::BASE_MINT_OFFSET..Self::BASE_MINT_OFFSET + Self::PUBKEY_LEN]
                .try_into()
                .expect("slice length verified"),
        );
        let quote_mint = Pubkey::new_from_array(
            data[Self::QUOTE_MINT_OFFSET..Self::QUOTE_MINT_OFFSET + Self::PUBKEY_LEN]
                .try_into()
                .expect("slice length verified"),
        );

        Ok(Self {
            pool_id,
            base_mint,
            quote_mint,
        })
    }
}
