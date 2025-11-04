#![allow(dead_code)]

use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct WhirlpoolSwapAccounts {
    pub swap_program: Pubkey,
    pub token_program: Pubkey,
    pub token_authority: Pubkey,
    pub whirlpool: Pubkey,
    pub user_token_a: Pubkey,
    pub vault_a: Pubkey,
    pub user_token_b: Pubkey,
    pub vault_b: Pubkey,
    pub tick_array0: Pubkey,
    pub tick_array1: Pubkey,
    pub tick_array2: Pubkey,
    pub oracle: Pubkey,
}

pub fn parse_whirlpool_swap(accounts: &[Pubkey]) -> Option<WhirlpoolSwapAccounts> {
    if accounts.len() < 12 {
        return None;
    }
    Some(WhirlpoolSwapAccounts {
        swap_program: accounts[0],
        token_program: accounts[1],
        token_authority: accounts[2],
        whirlpool: accounts[3],
        user_token_a: accounts[4],
        vault_a: accounts[5],
        user_token_b: accounts[6],
        vault_b: accounts[7],
        tick_array0: accounts[8],
        tick_array1: accounts[9],
        tick_array2: accounts[10],
        oracle: accounts[11],
    })
}

#[derive(Debug, Clone)]
pub struct WhirlpoolSwapV2Accounts {
    pub swap_program: Pubkey,
    pub token_program_a: Pubkey,
    pub token_program_b: Pubkey,
    pub memo_program: Pubkey,
    pub token_authority: Pubkey,
    pub whirlpool: Pubkey,
    pub token_mint_a: Pubkey,
    pub token_mint_b: Pubkey,
    pub user_token_a: Pubkey,
    pub vault_a: Pubkey,
    pub user_token_b: Pubkey,
    pub vault_b: Pubkey,
    pub tick_array0: Pubkey,
    pub tick_array1: Pubkey,
    pub tick_array2: Pubkey,
    pub oracle: Pubkey,
}

pub fn parse_whirlpool_swap_v2(accounts: &[Pubkey]) -> Option<WhirlpoolSwapV2Accounts> {
    if accounts.len() < 16 {
        return None;
    }
    Some(WhirlpoolSwapV2Accounts {
        swap_program: accounts[0],
        token_program_a: accounts[1],
        token_program_b: accounts[2],
        memo_program: accounts[3],
        token_authority: accounts[4],
        whirlpool: accounts[5],
        token_mint_a: accounts[6],
        token_mint_b: accounts[7],
        user_token_a: accounts[8],
        vault_a: accounts[9],
        user_token_b: accounts[10],
        vault_b: accounts[11],
        tick_array0: accounts[12],
        tick_array1: accounts[13],
        tick_array2: accounts[14],
        oracle: accounts[15],
    })
}
