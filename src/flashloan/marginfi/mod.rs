mod account;
mod instructions;
mod manager;

pub use account::{find_marginfi_account_by_authority, marginfi_account_matches_authority};
#[allow(unused_imports)]
pub use instructions::build_close_instruction;
pub use instructions::build_initialize_instruction;
pub use manager::{
    MarginfiAccountRegistry, MarginfiFlashloanManager, MarginfiFlashloanPreparation,
};

use once_cell::sync::Lazy;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub(super) static PROGRAM_ID: Lazy<Pubkey> =
    Lazy::new(|| parse_pubkey("MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA"));
pub(super) static GROUP_ID: Lazy<Pubkey> =
    Lazy::new(|| parse_pubkey("4qp6Fx6tnZkY5Wropq9wUYgtFxXKwE6viZxFHg3rdAG8"));
pub(super) static TOKEN_PROGRAM_ID: Lazy<Pubkey> =
    Lazy::new(|| parse_pubkey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"));
pub(super) static SYSTEM_PROGRAM_ID: Lazy<Pubkey> =
    Lazy::new(|| parse_pubkey("11111111111111111111111111111111"));

pub(super) const ACCOUNT_INITIALIZE_DISCRIMINATOR: [u8; 8] = [43, 78, 61, 255, 148, 52, 249, 154];
#[allow(dead_code)]
pub(super) const CLOSE_ACCOUNT_DISCRIMINATOR: [u8; 8] = [186, 221, 93, 34, 50, 97, 194, 241];
pub(super) const BEGIN_DISCRIMINATOR: [u8; 8] = [14, 131, 33, 220, 81, 186, 180, 107];
pub(super) const END_DISCRIMINATOR: [u8; 8] = [105, 124, 201, 106, 153, 2, 8, 156];
pub(super) const BORROW_DISCRIMINATOR: [u8; 8] = [4, 126, 116, 53, 48, 5, 212, 31];
pub(super) const REPAY_DISCRIMINATOR: [u8; 8] = [79, 209, 172, 177, 222, 51, 173, 151];

pub(super) const PUBKEY_BYTES: usize = 32;
pub(super) const GROUP_OFFSET: usize = 8;
pub(super) const AUTHORITY_OFFSET: usize = GROUP_OFFSET + PUBKEY_BYTES;
pub(super) const ACCOUNT_HEADER_MIN_LEN: usize = AUTHORITY_OFFSET + PUBKEY_BYTES;

const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const SPL_TOKEN_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub(super) fn parse_pubkey(raw: &str) -> Pubkey {
    Pubkey::from_str(raw).expect("invalid pubkey constant")
}

pub(super) fn compute_associated_token_address(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[owner.as_ref(), SPL_TOKEN_PROGRAM_ID.as_ref(), mint.as_ref()],
        &ASSOCIATED_TOKEN_PROGRAM_ID,
    )
    .0
}

#[cfg(test)]
mod tests;
