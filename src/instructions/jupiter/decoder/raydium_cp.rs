#![allow(dead_code)]

use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct RaydiumCpSwapAccounts {
    pub swap_program: Pubkey,
    pub amm_id: Pubkey,
    pub authority: Pubkey,
    pub open_orders: Pubkey,
    pub target_orders: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub user_base: Pubkey,
    pub user_quote: Pubkey,
    pub serum_market: Pubkey,
    pub serum_bids: Pubkey,
    pub serum_asks: Pubkey,
    pub serum_event_queue: Pubkey,
    pub serum_coin_vault: Pubkey,
    pub serum_pc_vault: Pubkey,
    pub serum_vault_signer: Pubkey,
    pub token_program: Pubkey,
}

pub fn parse_raydium_cp_swap(accounts: &[Pubkey]) -> Option<RaydiumCpSwapAccounts> {
    if accounts.len() < 17 {
        return None;
    }
    Some(RaydiumCpSwapAccounts {
        swap_program: accounts[0],
        amm_id: accounts[1],
        authority: accounts[2],
        open_orders: accounts[3],
        target_orders: accounts[4],
        base_vault: accounts[5],
        quote_vault: accounts[6],
        user_base: accounts[7],
        user_quote: accounts[8],
        serum_market: accounts[9],
        serum_bids: accounts[10],
        serum_asks: accounts[11],
        serum_event_queue: accounts[12],
        serum_coin_vault: accounts[13],
        serum_pc_vault: accounts[14],
        serum_vault_signer: accounts[15],
        token_program: accounts[16],
    })
}
