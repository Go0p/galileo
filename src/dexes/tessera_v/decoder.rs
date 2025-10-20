use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_client::rpc_response::RpcKeyedAccount;
use solana_sdk::pubkey::Pubkey;
use std::convert::TryInto;

use super::types::PoolState;

pub const TESSERA_V_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH");
pub const TESSERA_V_GLOBAL_STATE: Pubkey =
    solana_sdk::pubkey!("8ekCy2jHHUbW2yeNGFWYJT9Hm9FW7SvZcZK66dSZCDiF");

#[derive(Debug, Clone)]
pub struct TesseraVMarketMeta {
    pub global_state: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
}

pub async fn fetch_market_meta(
    client: &RpcClient,
    market: Pubkey,
    account: &solana_sdk::account::Account,
) -> Result<TesseraVMarketMeta> {
    anyhow::ensure!(
        account.owner == TESSERA_V_PROGRAM_ID,
        "账户 {market} 所属程序不是 TesseraV: {}",
        account.owner
    );

    let pool_state = PoolState::parse(&account.data)
        .with_context(|| format!("decode Tessera V pool account {market}"))?;

    let (base_token_program, _base_decimals) = load_mint_info(client, pool_state.base_mint).await?;
    let (quote_token_program, _quote_decimals) =
        load_mint_info(client, pool_state.quote_mint).await?;

    let base_vault = fetch_vault_account(client, pool_state.base_mint).await?;
    let quote_vault = fetch_vault_account(client, pool_state.quote_mint).await?;

    Ok(TesseraVMarketMeta {
        global_state: TESSERA_V_GLOBAL_STATE,
        base_mint: pool_state.base_mint,
        quote_mint: pool_state.quote_mint,
        base_vault,
        quote_vault,
        base_token_program,
        quote_token_program,
    })
}

async fn load_mint_info(client: &RpcClient, mint: Pubkey) -> Result<(Pubkey, u8)> {
    let account = client
        .get_account(&mint)
        .await
        .with_context(|| format!("fetch mint account {mint}"))?;
    anyhow::ensure!(
        account.data.len() > 44,
        "mint account {mint} data too small: {} bytes",
        account.data.len()
    );
    let decimals = account.data[44];
    Ok((account.owner, decimals))
}

async fn fetch_vault_account(client: &RpcClient, mint: Pubkey) -> Result<Pubkey> {
    let accounts: Vec<RpcKeyedAccount> = client
        .get_token_accounts_by_owner(&TESSERA_V_GLOBAL_STATE, TokenAccountsFilter::Mint(mint))
        .await
        .with_context(|| format!("fetch Tessera vault accounts for mint {mint}"))?;

    extract_largest_balance_account(accounts)
        .with_context(|| format!("owner {TESSERA_V_GLOBAL_STATE} has no vault for mint {mint}"))
}

fn extract_largest_balance_account(accounts: Vec<RpcKeyedAccount>) -> Option<Pubkey> {
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
