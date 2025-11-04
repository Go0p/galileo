#![allow(dead_code)]

use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct TesseraSwapAccounts {
    pub swap_program: Pubkey,
    pub swap_authority: Pubkey,
    pub user_source: Pubkey,
    pub user_destination: Pubkey,
    pub global_state: Pubkey,
    pub pool_state: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
    pub sysvar_instructions: Pubkey,
}

pub fn parse_tessera_swap(accounts: &[Pubkey]) -> Option<TesseraSwapAccounts> {
    if accounts.len() < 13 {
        return None;
    }
    Some(TesseraSwapAccounts {
        swap_program: accounts[0],
        swap_authority: accounts[1],
        user_source: accounts[2],
        user_destination: accounts[3],
        global_state: accounts[4],
        pool_state: accounts[5],
        base_vault: accounts[6],
        quote_vault: accounts[7],
        base_mint: accounts[8],
        quote_mint: accounts[9],
        base_token_program: accounts[10],
        quote_token_program: accounts[11],
        sysvar_instructions: accounts[12],
    })
}
