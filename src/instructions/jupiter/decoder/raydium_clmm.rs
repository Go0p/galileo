#![allow(dead_code)]

use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub struct RaydiumClmmSwapAccounts {
    pub swap_program: Pubkey,
    pub payer: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub user_token_in: Pubkey,
    pub user_token_out: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub observation_state: Pubkey,
    pub token_program: Pubkey,
    pub tick_array: Pubkey,
}

pub fn parse_raydium_clmm_swap(accounts: &[Pubkey]) -> Option<RaydiumClmmSwapAccounts> {
    if accounts.len() < 11 {
        return None;
    }
    Some(RaydiumClmmSwapAccounts {
        swap_program: accounts[0],
        payer: accounts[1],
        amm_config: accounts[2],
        pool_state: accounts[3],
        user_token_in: accounts[4],
        user_token_out: accounts[5],
        input_vault: accounts[6],
        output_vault: accounts[7],
        observation_state: accounts[8],
        token_program: accounts[9],
        tick_array: accounts[10],
    })
}

#[derive(Debug, Clone)]
pub struct RaydiumClmmSwapV2Accounts {
    pub swap_program: Pubkey,
    pub payer: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub user_token_in: Pubkey,
    pub user_token_out: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub observation_state: Pubkey,
    pub token_program: Pubkey,
    pub tick_array: Pubkey,
    pub vault_program: Pubkey,
    pub reward_infos: Vec<Pubkey>,
}

pub fn parse_raydium_clmm_swap_v2(accounts: &[Pubkey]) -> Option<RaydiumClmmSwapV2Accounts> {
    if accounts.len() < 12 {
        return None;
    }
    let base = parse_raydium_clmm_swap(accounts)?;
    let vault_program = accounts[11];
    let reward_infos = accounts[12..].to_vec();

    Some(RaydiumClmmSwapV2Accounts {
        swap_program: base.swap_program,
        payer: base.payer,
        amm_config: base.amm_config,
        pool_state: base.pool_state,
        user_token_in: base.user_token_in,
        user_token_out: base.user_token_out,
        input_vault: base.input_vault,
        output_vault: base.output_vault,
        observation_state: base.observation_state,
        token_program: base.token_program,
        tick_array: base.tick_array,
        vault_program,
        reward_infos,
    })
}
