#![allow(dead_code)]

use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct ObricV2SwapAccounts {
    pub swap_program: Pubkey,
    pub swap_authority: Pubkey,
    pub user_source: Pubkey,
    pub user_destination: Pubkey,
    pub trading_pair: Pubkey,
    pub second_reference_oracle: Pubkey,
    pub third_reference_oracle: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub reference_oracle: Pubkey,
    pub x_price_feed: Pubkey,
    pub y_price_feed: Pubkey,
    pub token_program: Pubkey,
}

pub fn parse_obric_v2_swap(accounts: &[Pubkey]) -> Option<ObricV2SwapAccounts> {
    if accounts.len() < 13 {
        return None;
    }
    Some(ObricV2SwapAccounts {
        swap_program: accounts[0],
        swap_authority: accounts[1],
        user_source: accounts[2],
        user_destination: accounts[3],
        trading_pair: accounts[4],
        second_reference_oracle: accounts[5],
        third_reference_oracle: accounts[6],
        reserve_x: accounts[7],
        reserve_y: accounts[8],
        reference_oracle: accounts[9],
        x_price_feed: accounts[10],
        y_price_feed: accounts[11],
        token_program: accounts[12],
    })
}
