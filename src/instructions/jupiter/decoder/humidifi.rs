#![allow(dead_code)]

use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct HumidifiSwapAccounts {
    pub swap_program: Pubkey,
    pub swap_authority: Pubkey,
    pub user_source: Pubkey,
    pub user_destination: Pubkey,
    pub humidifi_param: Pubkey,
    pub pool: Pubkey,
    pub pool_base_vault: Pubkey,
    pub pool_quote_vault: Pubkey,
    pub clock_sysvar: Pubkey,
    pub token_program: Pubkey,
    pub sysvar_instructions: Pubkey,
}

pub fn parse_humidifi_swap(accounts: &[Pubkey]) -> Option<HumidifiSwapAccounts> {
    if accounts.len() < 11 {
        return None;
    }
    Some(HumidifiSwapAccounts {
        swap_program: accounts[0],
        swap_authority: accounts[1],
        user_source: accounts[2],
        user_destination: accounts[3],
        humidifi_param: accounts[4],
        pool: accounts[5],
        pool_base_vault: accounts[6],
        pool_quote_vault: accounts[7],
        clock_sysvar: accounts[8],
        token_program: accounts[9],
        sysvar_instructions: accounts[10],
    })
}
