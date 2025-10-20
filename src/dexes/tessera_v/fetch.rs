use std::str::FromStr;

use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
use solana_client::{
    rpc_client::RpcClient, rpc_request::TokenAccountsFilter, rpc_response::RpcKeyedAccount,
};
use solana_sdk::{pubkey::Pubkey, sysvar};
use std::convert::TryInto;

use super::decoder::TESSERA_V_GLOBAL_STATE;

const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
use super::types::PoolState;

/// 0. global_state          8ekCy2jHHUbW2yeNGFWYJT9Hm9FW7SvZcZK66dSZCDiF
/// 1. pool_state            FLckHLGMJy5gEoXWwcE68Nprde1D4araK4TGLw4pQq2n
/// 2. user_authority
/// 3. base_vault
/// 4. quote_vault
/// 5. user_base_token
/// 6. user_quote_token
/// 7. base_mint
/// 8. quote_mint
/// 9. base_token_program
/// 10. quote_token_program
/// 11. sysvar_instructions
#[derive(Debug, Clone)]
pub struct TesseraVSwapInfo {
    pub global_state: Pubkey,
    pub pool_state: Pubkey,
    pub user_authority: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub user_base_token: Pubkey,
    pub user_quote_token: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
    pub sysvar_instructions: Pubkey,
}

pub fn fetch_tessera_v_swap_info(
    client: &RpcClient,
    market: Pubkey,
    payer: Pubkey,
    user_base_account: Pubkey,
    user_quote_account: Pubkey,
) -> Result<TesseraVSwapInfo> {
    let pool_account = client
        .get_account_data(&market)
        .with_context(|| format!("fetch Tessera V pool account {market}"))?;
    let pool_state = PoolState::parse(&pool_account)?;

    let (base_token_program, _base_decimals) = load_mint_info(client, &pool_state.base_mint)?;
    let (quote_token_program, _quote_decimals) = load_mint_info(client, &pool_state.quote_mint)?;

    let base_vault = fetch_vault_account(client, &pool_state.base_mint)?;
    let quote_vault = fetch_vault_account(client, &pool_state.quote_mint)?;

    let user_authority = payer;
    let resolved_base_token =
        if user_base_account == Pubkey::default() && payer != Pubkey::default() {
            derive_associated_token_address(&payer, &pool_state.base_mint, &base_token_program)
        } else {
            user_base_account
        };
    let resolved_quote_token =
        if user_quote_account == Pubkey::default() && payer != Pubkey::default() {
            derive_associated_token_address(&payer, &pool_state.quote_mint, &quote_token_program)
        } else {
            user_quote_account
        };

    Ok(TesseraVSwapInfo {
        global_state: TESSERA_V_GLOBAL_STATE,
        pool_state: market,
        user_authority,
        base_vault,
        quote_vault,
        user_base_token: resolved_base_token,
        user_quote_token: resolved_quote_token,
        base_mint: pool_state.base_mint,
        quote_mint: pool_state.quote_mint,
        base_token_program,
        quote_token_program,
        sysvar_instructions: sysvar::instructions::ID,
    })
}

fn load_mint_info(client: &RpcClient, mint: &Pubkey) -> Result<(Pubkey, u8)> {
    let account = client
        .get_account(mint)
        .with_context(|| format!("fetch mint account {mint}"))?;
    anyhow::ensure!(
        account.data.len() > 44,
        "mint account {mint} data too small: {} bytes",
        account.data.len()
    );
    let decimals = account.data[44];
    Ok((account.owner, decimals))
}

fn fetch_vault_account(client: &RpcClient, mint: &Pubkey) -> Result<Pubkey> {
    let accounts: Vec<RpcKeyedAccount> = client
        .get_token_accounts_by_owner(&TESSERA_V_GLOBAL_STATE, TokenAccountsFilter::Mint(*mint))
        .with_context(|| format!("fetch Tessera vault accounts for mint {mint}"))?;

    let mut best: Option<(Pubkey, u64)> = None;
    for keyed in accounts {
        let pubkey = Pubkey::from_str(&keyed.pubkey)
            .with_context(|| format!("parse vault pubkey {}", keyed.pubkey))?;
        if let Some(amount) = parse_token_amount(&keyed.account.data) {
            if best.map_or(true, |(_, current)| amount > current) {
                best = Some((pubkey, amount));
            }
        }
    }

    best.map(|(pubkey, _)| pubkey)
        .with_context(|| format!("owner {TESSERA_V_GLOBAL_STATE} has no vault for mint {mint}"))
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

fn derive_associated_token_address(
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Pubkey {
    let program_id = Pubkey::new_from_array(ASSOCIATED_TOKEN_PROGRAM_ID.to_bytes());
    let (address, _) = Pubkey::find_program_address(
        &[owner.as_ref(), token_program.as_ref(), mint.as_ref()],
        &program_id,
    );
    address
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    fn fetch_live_swap_info() {
        let client = RpcClient::new("http://127.0.0.1:8899".to_string());
        let market = Pubkey::from_str("FLckHLGMJy5gEoXWwcE68Nprde1D4araK4TGLw4pQq2n").unwrap();
        let swap_info = fetch_tessera_v_swap_info(
            &client,
            market,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        )
        .unwrap();
        println!("{:#?}", swap_info);
    }
}
