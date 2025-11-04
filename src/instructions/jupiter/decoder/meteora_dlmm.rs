#![allow(dead_code)]

use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct MeteoraDlmmSwapAccounts {
    pub swap_program: Pubkey,
    pub lb_pair: Pubkey,
    pub bin_array_bitmap_extension: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub user_token_in: Pubkey,
    pub user_token_out: Pubkey,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub oracle: Pubkey,
    pub host_fee_in: Pubkey,
    pub user: Pubkey,
    pub token_x_program: Pubkey,
    pub token_y_program: Pubkey,
    pub event_authority: Pubkey,
    pub program: Pubkey,
}

pub fn parse_meteora_dlmm_swap(accounts: &[Pubkey]) -> Option<MeteoraDlmmSwapAccounts> {
    if accounts.len() < 16 {
        return None;
    }
    Some(MeteoraDlmmSwapAccounts {
        swap_program: accounts[0],
        lb_pair: accounts[1],
        bin_array_bitmap_extension: accounts[2],
        reserve_x: accounts[3],
        reserve_y: accounts[4],
        user_token_in: accounts[5],
        user_token_out: accounts[6],
        token_x_mint: accounts[7],
        token_y_mint: accounts[8],
        oracle: accounts[9],
        host_fee_in: accounts[10],
        user: accounts[11],
        token_x_program: accounts[12],
        token_y_program: accounts[13],
        event_authority: accounts[14],
        program: accounts[15],
    })
}
