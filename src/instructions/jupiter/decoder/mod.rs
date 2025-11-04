#![allow(dead_code)]

mod humidifi;
mod meteora_dlmm;
mod obric_v2;
mod raydium_clmm;
mod raydium_cp;
mod solfi_v2;
mod tessera;
mod whirlpool;
mod zerofi;

pub use humidifi::{HumidifiSwapAccounts, parse_humidifi_swap};
pub use meteora_dlmm::{MeteoraDlmmSwapAccounts, parse_meteora_dlmm_swap};
pub use obric_v2::{ObricV2SwapAccounts, parse_obric_v2_swap};
pub use raydium_clmm::{
    RaydiumClmmSwapAccounts, RaydiumClmmSwapV2Accounts, parse_raydium_clmm_swap,
    parse_raydium_clmm_swap_v2,
};
pub use raydium_cp::{RaydiumCpSwapAccounts, parse_raydium_cp_swap};
pub use solfi_v2::{
    SolfiV1SwapAccounts, SolfiV2SwapAccounts, parse_solfi_v1_swap, parse_solfi_v2_swap,
};
pub use tessera::{TesseraSwapAccounts, parse_tessera_swap};
pub use whirlpool::{
    WhirlpoolSwapAccounts, WhirlpoolSwapV2Accounts, parse_whirlpool_swap, parse_whirlpool_swap_v2,
};
pub use zerofi::{ZerofiSwapAccounts, parse_zerofi_swap};

use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone)]
pub enum ParsedSwapAccounts {
    Humidifi(HumidifiSwapAccounts),
    Whirlpool(WhirlpoolSwapAccounts),
    WhirlpoolV2(WhirlpoolSwapV2Accounts),
    RaydiumClmm(RaydiumClmmSwapAccounts),
    RaydiumClmmV2(RaydiumClmmSwapV2Accounts),
    RaydiumCp(RaydiumCpSwapAccounts),
    MeteoraDlmm(MeteoraDlmmSwapAccounts),
    SolfiV1(SolfiV1SwapAccounts),
    SolfiV2(SolfiV2SwapAccounts),
    Tessera(TesseraSwapAccounts),
    Zerofi(ZerofiSwapAccounts),
    ObricV2(ObricV2SwapAccounts),
}

impl ParsedSwapAccounts {
    pub fn swap_program(&self) -> Pubkey {
        match self {
            ParsedSwapAccounts::Humidifi(acc) => acc.swap_program,
            ParsedSwapAccounts::Whirlpool(acc) => acc.swap_program,
            ParsedSwapAccounts::WhirlpoolV2(acc) => acc.swap_program,
            ParsedSwapAccounts::RaydiumClmm(acc) => acc.swap_program,
            ParsedSwapAccounts::RaydiumClmmV2(acc) => acc.swap_program,
            ParsedSwapAccounts::RaydiumCp(acc) => acc.swap_program,
            ParsedSwapAccounts::MeteoraDlmm(acc) => acc.swap_program,
            ParsedSwapAccounts::SolfiV1(acc) => acc.swap_program,
            ParsedSwapAccounts::SolfiV2(acc) => acc.swap_program,
            ParsedSwapAccounts::Tessera(acc) => acc.swap_program,
            ParsedSwapAccounts::Zerofi(acc) => acc.swap_program,
            ParsedSwapAccounts::ObricV2(acc) => acc.swap_program,
        }
    }

    pub fn pool_state(&self) -> Pubkey {
        match self {
            ParsedSwapAccounts::Humidifi(acc) => acc.pool,
            ParsedSwapAccounts::Whirlpool(acc) => acc.whirlpool,
            ParsedSwapAccounts::WhirlpoolV2(acc) => acc.whirlpool,
            ParsedSwapAccounts::RaydiumClmm(acc) => acc.pool_state,
            ParsedSwapAccounts::RaydiumClmmV2(acc) => acc.pool_state,
            ParsedSwapAccounts::RaydiumCp(acc) => acc.amm_id,
            ParsedSwapAccounts::MeteoraDlmm(acc) => acc.lb_pair,
            ParsedSwapAccounts::SolfiV1(acc) => acc.pair_account,
            ParsedSwapAccounts::SolfiV2(acc) => acc.market,
            ParsedSwapAccounts::Tessera(acc) => acc.pool_state,
            ParsedSwapAccounts::Zerofi(acc) => acc.pair,
            ParsedSwapAccounts::ObricV2(acc) => acc.trading_pair,
        }
    }

    pub fn user_accounts(&self) -> Option<(Pubkey, Pubkey)> {
        match self {
            ParsedSwapAccounts::Humidifi(acc) => Some((acc.user_source, acc.user_destination)),
            ParsedSwapAccounts::Whirlpool(acc) => Some((acc.user_token_a, acc.user_token_b)),
            ParsedSwapAccounts::WhirlpoolV2(acc) => Some((acc.user_token_a, acc.user_token_b)),
            ParsedSwapAccounts::RaydiumClmm(acc) => Some((acc.user_token_in, acc.user_token_out)),
            ParsedSwapAccounts::RaydiumClmmV2(acc) => Some((acc.user_token_in, acc.user_token_out)),
            ParsedSwapAccounts::RaydiumCp(acc) => Some((acc.user_base, acc.user_quote)),
            ParsedSwapAccounts::MeteoraDlmm(acc) => Some((acc.user_token_in, acc.user_token_out)),
            ParsedSwapAccounts::SolfiV1(acc) => Some((acc.user_source, acc.user_destination)),
            ParsedSwapAccounts::SolfiV2(acc) => Some((acc.user_source, acc.user_destination)),
            ParsedSwapAccounts::Tessera(acc) => Some((acc.user_source, acc.user_destination)),
            ParsedSwapAccounts::Zerofi(acc) => Some((acc.user_source, acc.user_destination)),
            ParsedSwapAccounts::ObricV2(acc) => Some((acc.user_source, acc.user_destination)),
        }
    }

    pub fn dex_label(&self) -> &'static str {
        match self {
            ParsedSwapAccounts::Humidifi(_) => "HumidiFi",
            ParsedSwapAccounts::Whirlpool(_) => "Whirlpool",
            ParsedSwapAccounts::WhirlpoolV2(_) => "WhirlpoolSwapV2",
            ParsedSwapAccounts::RaydiumClmm(_) => "RaydiumClmm",
            ParsedSwapAccounts::RaydiumClmmV2(_) => "RaydiumClmmV2",
            ParsedSwapAccounts::RaydiumCp(_) => "RaydiumCp",
            ParsedSwapAccounts::MeteoraDlmm(_) => "MeteoraDlmm",
            ParsedSwapAccounts::SolfiV1(_) => "SolFi",
            ParsedSwapAccounts::SolfiV2(_) => "SolFiV2",
            ParsedSwapAccounts::Tessera(_) => "TesseraV",
            ParsedSwapAccounts::Zerofi(_) => "ZeroFi",
            ParsedSwapAccounts::ObricV2(_) => "ObricV2",
        }
    }
}

pub fn parse_swap_accounts(variant: &str, accounts: &[Pubkey]) -> Option<ParsedSwapAccounts> {
    match variant {
        "HumidiFi" => parse_humidifi_swap(accounts).map(ParsedSwapAccounts::Humidifi),
        "Whirlpool" => parse_whirlpool_swap(accounts).map(ParsedSwapAccounts::Whirlpool),
        "WhirlpoolSwapV2" => parse_whirlpool_swap_v2(accounts).map(ParsedSwapAccounts::WhirlpoolV2),
        "RaydiumClmm" => parse_raydium_clmm_swap(accounts).map(ParsedSwapAccounts::RaydiumClmm),
        "RaydiumClmmV2" => {
            parse_raydium_clmm_swap_v2(accounts).map(ParsedSwapAccounts::RaydiumClmmV2)
        }
        "RaydiumCp" | "RaydiumCP" => {
            parse_raydium_cp_swap(accounts).map(ParsedSwapAccounts::RaydiumCp)
        }
        "MeteoraDlmm" | "MeteoraDlmmSwapV2" => {
            parse_meteora_dlmm_swap(accounts).map(ParsedSwapAccounts::MeteoraDlmm)
        }
        "SolFi" => parse_solfi_v1_swap(accounts).map(ParsedSwapAccounts::SolfiV1),
        "SolFiV2" => parse_solfi_v2_swap(accounts).map(ParsedSwapAccounts::SolfiV2),
        "TesseraV" | "Tessera" => parse_tessera_swap(accounts).map(ParsedSwapAccounts::Tessera),
        "ZeroFi" => parse_zerofi_swap(accounts).map(ParsedSwapAccounts::Zerofi),
        "ObricV2" => parse_obric_v2_swap(accounts).map(ParsedSwapAccounts::ObricV2),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_accounts(len: usize) -> Vec<Pubkey> {
        (0..len).map(|_| Pubkey::new_unique()).collect()
    }

    #[test]
    fn parse_whirlpool_accounts_layout() {
        let accounts = dummy_accounts(12);
        match parse_swap_accounts("Whirlpool", &accounts) {
            Some(ParsedSwapAccounts::Whirlpool(acc)) => {
                assert_eq!(acc.swap_program, accounts[0]);
                assert_eq!(acc.whirlpool, accounts[3]);
                assert_eq!(acc.oracle, accounts[11]);
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn parse_whirlpool_v2_accounts_layout() {
        let accounts = dummy_accounts(16);
        match parse_swap_accounts("WhirlpoolSwapV2", &accounts) {
            Some(ParsedSwapAccounts::WhirlpoolV2(acc)) => {
                assert_eq!(acc.token_program_a, accounts[1]);
                assert_eq!(acc.whirlpool, accounts[5]);
                assert_eq!(acc.oracle, accounts[15]);
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn parse_raydium_clmm_accounts_layout() {
        let accounts = dummy_accounts(11);
        match parse_swap_accounts("RaydiumClmm", &accounts) {
            Some(ParsedSwapAccounts::RaydiumClmm(acc)) => {
                assert_eq!(acc.swap_program, accounts[0]);
                assert_eq!(acc.pool_state, accounts[3]);
                assert_eq!(acc.tick_array, accounts[10]);
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }

    #[test]
    fn parse_meteora_dlmm_accounts_layout() {
        let accounts = dummy_accounts(16);
        match parse_swap_accounts("MeteoraDlmm", &accounts) {
            Some(ParsedSwapAccounts::MeteoraDlmm(acc)) => {
                assert_eq!(acc.swap_program, accounts[0]);
                assert_eq!(acc.lb_pair, accounts[1]);
                assert_eq!(acc.program, accounts[15]);
            }
            other => panic!("unexpected parse result: {other:?}"),
        }
    }
}
