use std::mem::size_of;

use anyhow::{Result, ensure};
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, sysvar};

const MARKET_ACCOUNT_SIZE: usize = 1728;
const MARKET_HEADER_SIZE: usize = 704;

pub const SOLFI_V2_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("SV2EYYJyRz2YhfXwXnhNAevDEui5Q6yrfyo13WtupPF");

/*
swap accounts:
user: writable,signer,
pair: writable,
oracle_account: readonly,
config_account: readonly,
base_vault: writable,
quote_vault: writable,
user_base_account: writable,
user_quote_account: writable,
base_mint: readonly,
quote_mint: readonly,
base_token_program: readonly,
quote_token_program: readonly,
sysvar: readonly,
*/
#[derive(Debug, Clone)]
pub struct SolfiV2MarketMeta {
    pub pair_account: AccountMeta,
    pub oracle_account: AccountMeta,
    pub config_account: AccountMeta,
    pub base_vault: AccountMeta,
    pub quote_vault: AccountMeta,
    pub base_mint: AccountMeta,
    pub quote_mint: AccountMeta,
    pub base_token_program: AccountMeta,
    pub quote_token_program: AccountMeta,
    pub sysvar_instructions: AccountMeta,
}

pub fn decode_market_meta(market: Pubkey, market_account_data: &[u8]) -> Result<SolfiV2MarketMeta> {
    ensure!(
        market_account_data.len() >= MARKET_ACCOUNT_SIZE,
        "solfi v2 market account {market} too small for header: {} bytes",
        market_account_data.len()
    );

    let header = MarketAccountHeader::new_from_slice(&market_account_data[..MARKET_HEADER_SIZE]);

    let oracle = Pubkey::new_from_array(header.oracle_account);
    let config = Pubkey::new_from_array(header.config_account);
    let base_vault = Pubkey::new_from_array(header.base_vault);
    let quote_vault = Pubkey::new_from_array(header.quote_vault);
    let base_mint = Pubkey::new_from_array(header.base_mint);
    let quote_mint = Pubkey::new_from_array(header.quote_mint);
    let base_token_program = Pubkey::new_from_array(header.base_token_program);
    let quote_token_program = Pubkey::new_from_array(header.quote_token_program);

    Ok(SolfiV2MarketMeta {
        pair_account: AccountMeta::new(market, false),
        oracle_account: AccountMeta::new_readonly(oracle, false),
        config_account: AccountMeta::new_readonly(config, false),
        base_vault: AccountMeta::new(base_vault, false),
        quote_vault: AccountMeta::new(quote_vault, false),
        base_mint: AccountMeta::new_readonly(base_mint, false),
        quote_mint: AccountMeta::new_readonly(quote_mint, false),
        base_token_program: AccountMeta::new_readonly(base_token_program, false),
        quote_token_program: AccountMeta::new_readonly(quote_token_program, false),
        sysvar_instructions: AccountMeta::new_readonly(sysvar::instructions::ID, false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    #[ignore]
    fn dump_market_meta() {
        let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
        let market = Pubkey::from_str("65ZHSArs5XxPseKQbB1B4r16vDxMWnCxHMzogDAqiDUc")
            .expect("market pubkey");
        let account = client
            .get_account(&market)
            .expect("failed to fetch SolFiV2 market account");
        let meta = decode_market_meta(market, &account.data).expect("decode SolFiV2 meta");
        println!("{meta:#?}");
    }
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
