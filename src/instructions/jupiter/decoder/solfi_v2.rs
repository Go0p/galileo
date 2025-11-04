#![allow(dead_code)]

use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct SolfiV1SwapAccounts {
    pub swap_program: Pubkey,
    pub swap_authority: Pubkey,
    pub user_source: Pubkey,
    pub user_destination: Pubkey,
    pub pair_account: Pubkey,
    pub pool_token_a: Pubkey,
    pub pool_token_b: Pubkey,
    pub token_program: Pubkey,
    pub sysvar_instructions: Pubkey,
}

pub fn parse_solfi_v1_swap(accounts: &[Pubkey]) -> Option<SolfiV1SwapAccounts> {
    if accounts.len() < 9 {
        return None;
    }
    Some(SolfiV1SwapAccounts {
        swap_program: accounts[0],
        swap_authority: accounts[1],
        user_source: accounts[2],
        user_destination: accounts[3],
        pair_account: accounts[4],
        pool_token_a: accounts[5],
        pool_token_b: accounts[6],
        token_program: accounts[7],
        sysvar_instructions: accounts[8],
    })
}

#[derive(Debug, Clone)]
pub struct SolfiV2SwapAccounts {
    pub swap_program: Pubkey,
    pub swap_authority: Pubkey,
    pub user_source: Pubkey,
    pub user_destination: Pubkey,
    pub market: Pubkey,
    pub oracle: Pubkey,
    pub global_config: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
    pub sysvar_instructions: Pubkey,
}

pub fn parse_solfi_v2_swap(accounts: &[Pubkey]) -> Option<SolfiV2SwapAccounts> {
    if accounts.len() < 14 {
        return None;
    }
    Some(SolfiV2SwapAccounts {
        swap_program: accounts[0],
        swap_authority: accounts[1],
        user_source: accounts[2],
        user_destination: accounts[3],
        market: accounts[4],
        oracle: accounts[5],
        global_config: accounts[6],
        base_vault: accounts[7],
        quote_vault: accounts[8],
        base_mint: accounts[9],
        quote_mint: accounts[10],
        base_token_program: accounts[11],
        quote_token_program: accounts[12],
        sysvar_instructions: accounts[13],
    })
}
