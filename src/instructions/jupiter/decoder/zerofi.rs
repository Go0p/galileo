#![allow(dead_code)]

use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct ZerofiSwapAccounts {
    pub swap_program: Pubkey,
    pub swap_authority: Pubkey,
    pub user_source: Pubkey,
    pub user_destination: Pubkey,
    pub pair: Pubkey,
    pub vault_info_base: Pubkey,
    pub vault_base: Pubkey,
    pub vault_info_quote: Pubkey,
    pub vault_quote: Pubkey,
    pub token_program: Pubkey,
    pub sysvar_instructions: Pubkey,
}

pub fn parse_zerofi_swap(accounts: &[Pubkey]) -> Option<ZerofiSwapAccounts> {
    if accounts.len() < 11 {
        return None;
    }
    Some(ZerofiSwapAccounts {
        swap_program: accounts[0],
        swap_authority: accounts[1],
        user_source: accounts[2],
        user_destination: accounts[3],
        pair: accounts[4],
        vault_info_base: accounts[5],
        vault_base: accounts[6],
        vault_info_quote: accounts[7],
        vault_quote: accounts[8],
        token_program: accounts[9],
        sysvar_instructions: accounts[10],
    })
}
