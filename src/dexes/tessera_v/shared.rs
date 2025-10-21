use anyhow::{Result, ensure};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
use solana_client::rpc_response::RpcKeyedAccount;
use solana_sdk::{account::Account, pubkey::Pubkey};
use std::convert::TryInto;

pub(crate) fn read_mint_metadata(mint: Pubkey, account: &Account) -> Result<(Pubkey, u8)> {
    ensure!(
        account.data.len() > 44,
        "mint account {mint} data too small: {} bytes",
        account.data.len()
    );
    let decimals = account.data[44];
    Ok((account.owner, decimals))
}

pub(crate) fn extract_largest_balance_account(accounts: Vec<RpcKeyedAccount>) -> Option<Pubkey> {
    let mut best: Option<(Pubkey, u64)> = None;
    for keyed in accounts {
        let pubkey = match keyed.pubkey.parse() {
            Ok(key) => key,
            Err(_) => continue,
        };
        if let Some(amount) = parse_token_amount(&keyed.account.data) {
            if best.map_or(true, |(_, current)| amount > current) {
                best = Some((pubkey, amount));
            }
        }
    }
    best.map(|(pubkey, _)| pubkey)
}

fn parse_token_amount(data: &UiAccountData) -> Option<u64> {
    match data {
        UiAccountData::Json(json_account) => {
            let info = json_account.parsed.get("info")?.as_object()?;
            let token_amount = info.get("tokenAmount")?.as_object()?;
            let amount = token_amount.get("amount")?.as_str()?.parse().ok()?;
            Some(amount)
        }
        UiAccountData::Binary(encoded, UiAccountEncoding::Base64) => {
            let raw = BASE64.decode(encoded.as_bytes()).ok()?;
            parse_amount_from_raw(&raw)
        }
        _ => None,
    }
}

fn parse_amount_from_raw(data: &[u8]) -> Option<u64> {
    const AMOUNT_OFFSET: usize = 64;
    const AMOUNT_LEN: usize = 8;
    let slice = data.get(AMOUNT_OFFSET..AMOUNT_OFFSET + AMOUNT_LEN)?;
    let bytes: [u8; AMOUNT_LEN] = slice.try_into().ok()?;
    Some(u64::from_le_bytes(bytes))
}
