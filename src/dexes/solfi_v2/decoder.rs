use std::mem::size_of;

use anyhow::{Result, ensure};
use solana_sdk::{pubkey::Pubkey, sysvar};

use super::fetch::SolfiV2SwapInfo;

const MARKET_ACCOUNT_SIZE: usize = 1728;
const MARKET_HEADER_SIZE: usize = 704;

pub fn decode_swap_info(
    market: Pubkey,
    market_account_data: &[u8],
    payer: Pubkey,
    user_base_account: Pubkey,
    user_quote_account: Pubkey,
) -> Result<SolfiV2SwapInfo> {
    ensure!(
        market_account_data.len() >= MARKET_ACCOUNT_SIZE,
        "solfi v2 market account {market} too small: {} bytes",
        market_account_data.len()
    );

    let header = MarketAccountHeader::new_from_slice(&market_account_data[..MARKET_HEADER_SIZE]);

    Ok(SolfiV2SwapInfo {
        payer,
        pair: market,
        oracle_account: Pubkey::new_from_array(header.oracle_account),
        config_account: Pubkey::new_from_array(header.config_account),
        base_vault: Pubkey::new_from_array(header.base_vault),
        quote_vault: Pubkey::new_from_array(header.quote_vault),
        user_base_account,
        user_quote_account,
        base_mint: Pubkey::new_from_array(header.base_mint),
        quote_mint: Pubkey::new_from_array(header.quote_mint),
        base_token_program: Pubkey::new_from_array(header.base_token_program),
        quote_token_program: Pubkey::new_from_array(header.quote_token_program),
        sysvar: sysvar::instructions::ID,
    })
}

pub const SOLFI_V2_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("SoLFiHG9TfgtdUXUjWAxi3LtvYuFyDLVhBWxdMZxyCe");

#[derive(Debug, Clone)]
pub struct SolfiV2MarketMeta {
    pub oracle: Pubkey,
    pub config: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
}

pub fn decode_market_meta(market: Pubkey, market_account_data: &[u8]) -> Result<SolfiV2MarketMeta> {
    ensure!(
        market_account_data.len() >= MARKET_HEADER_SIZE,
        "solfi v2 market account {market} too small for header: {} bytes",
        market_account_data.len()
    );

    let header = MarketAccountHeader::new_from_slice(&market_account_data[..MARKET_HEADER_SIZE]);

    Ok(SolfiV2MarketMeta {
        oracle: Pubkey::new_from_array(header.oracle_account),
        config: Pubkey::new_from_array(header.config_account),
        base_mint: Pubkey::new_from_array(header.base_mint),
        quote_mint: Pubkey::new_from_array(header.quote_mint),
        base_vault: Pubkey::new_from_array(header.base_vault),
        quote_vault: Pubkey::new_from_array(header.quote_vault),
        base_token_program: Pubkey::new_from_array(header.base_token_program),
        quote_token_program: Pubkey::new_from_array(header.quote_token_program),
    })
}

#[repr(C)]
#[derive(Clone, Copy)]
struct MarketAccountHeader {
    _bump: u8,
    _market_num: u8,
    _config_version: u8,
    _padding_0: [u8; 5],
    _sequence_number: u64,
    _sequence_number_prev_slot: u64,
    oracle_account: [u8; 32],
    base_mint: [u8; 32],
    quote_mint: [u8; 32],
    base_vault: [u8; 32],
    quote_vault: [u8; 32],
    base_token_program: [u8; 32],
    quote_token_program: [u8; 32],
    base_mint_decimals: u8,
    quote_mint_decimals: u8,
    _padding_1: [u8; 6],
    config_account: [u8; 32],
    _padding_2: [u8; 416],
}

const _: [u8; MARKET_HEADER_SIZE] = [0; size_of::<MarketAccountHeader>()];

impl MarketAccountHeader {
    fn new_from_slice(data: &[u8]) -> &Self {
        // SAFETY: layout matches SolFi V2 market header and size checked above.
        unsafe { &*(data.as_ptr() as *const MarketAccountHeader) }
    }
}
