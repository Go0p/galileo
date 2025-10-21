use crate::rpc::batch::{RpcBatchRequest, send_batch as send_rpc_batch};
use anyhow::{Context, Result};
use serde_json::json;
use solana_account_decoder::UiAccountEncoding;
use solana_client::rpc_config::{RpcAccountInfoConfig, RpcTokenAccountsFilter};
use solana_client::rpc_request::{RpcRequest, TokenAccountsFilter};
use solana_client::rpc_response::{Response, RpcKeyedAccount};
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_client::RpcClient as BlockingRpcClient,
};
use solana_sdk::{account::Account, instruction::AccountMeta, pubkey::Pubkey};

use super::shared::{extract_largest_balance_account, read_mint_metadata};
use super::types::PoolState;

pub const TESSERA_V_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH");
pub const TESSERA_V_GLOBAL_STATE: Pubkey =
    solana_sdk::pubkey!("8ekCy2jHHUbW2yeNGFWYJT9Hm9FW7SvZcZK66dSZCDiF");

/*
swap accounts:
authority: readonly,
pool: writable,
user: writable,signer,
base_vault: writable,
quote_vault: writable,
user_base_token: writable,
user_quote_token: writable,
base_mint: readonly,
quote_mint: readonly,
base_token_program: readonly,
quote_token_program: readonly,
sysvar_instructions: readonly,
*/
#[derive(Debug, Clone)]
pub struct TesseraVMarketMeta {
    pub global_state: AccountMeta,
    pub pool_account: AccountMeta,
    pub base_vault: AccountMeta,
    pub quote_vault: AccountMeta,
    pub base_mint: AccountMeta,
    pub quote_mint: AccountMeta,
    pub base_token_program: AccountMeta,
    pub quote_token_program: AccountMeta,
    pub sysvar_instructions: AccountMeta,
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

    let [base_mint_account, quote_mint_account] = {
        let accounts = client
            .get_multiple_accounts(&[pool_state.base_mint, pool_state.quote_mint])
            .await
            .with_context(|| {
                format!(
                    "fetch mint accounts {} & {}",
                    pool_state.base_mint, pool_state.quote_mint
                )
            })?;
        extract_mint_pair(accounts, pool_state.base_mint, pool_state.quote_mint)?
    };

    let account_config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::JsonParsed),
        commitment: Some(client.commitment()),
        data_slice: None,
        min_context_slot: None,
    };

    let batch_requests = [
        RpcBatchRequest::new(
            RpcRequest::GetTokenAccountsByOwner,
            json!([
                TESSERA_V_GLOBAL_STATE.to_string(),
                RpcTokenAccountsFilter::Mint(pool_state.base_mint.to_string()),
                account_config.clone(),
            ]),
        ),
        RpcBatchRequest::new(
            RpcRequest::GetTokenAccountsByOwner,
            json!([
                TESSERA_V_GLOBAL_STATE.to_string(),
                RpcTokenAccountsFilter::Mint(pool_state.quote_mint.to_string()),
                account_config.clone(),
            ]),
        ),
    ];

    let batch_responses = send_rpc_batch(client, &batch_requests)
        .await
        .map_err(|err| anyhow::Error::new(err))
        .with_context(|| "batch fetch Tessera vault accounts")?;

    let base_vault_accounts: Vec<RpcKeyedAccount> =
        serde_json::from_value::<Response<Vec<RpcKeyedAccount>>>(batch_responses[0].clone())
            .with_context(|| {
                format!(
                    "decode Tessera vault accounts for mint {}",
                    pool_state.base_mint
                )
            })?
            .value;
    let quote_vault_accounts: Vec<RpcKeyedAccount> =
        serde_json::from_value::<Response<Vec<RpcKeyedAccount>>>(batch_responses[1].clone())
            .with_context(|| {
                format!(
                    "decode Tessera vault accounts for mint {}",
                    pool_state.quote_mint
                )
            })?
            .value;

    build_market_meta(
        market,
        pool_state,
        base_mint_account,
        quote_mint_account,
        base_vault_accounts,
        quote_vault_accounts,
    )
}

pub fn fetch_market_meta_blocking(
    client: &BlockingRpcClient,
    market: Pubkey,
) -> Result<TesseraVMarketMeta> {
    let account = client
        .get_account(&market)
        .with_context(|| format!("fetch Tessera V pool account {market}"))?;

    anyhow::ensure!(
        account.owner == TESSERA_V_PROGRAM_ID,
        "账户 {market} 所属程序不是 TesseraV: {}",
        account.owner
    );

    let pool_state = PoolState::parse(&account.data)
        .with_context(|| format!("decode Tessera V pool account {market}"))?;

    let [base_mint_account, quote_mint_account] = {
        let accounts = client
            .get_multiple_accounts(&[pool_state.base_mint, pool_state.quote_mint])
            .with_context(|| {
                format!(
                    "fetch mint accounts {} & {}",
                    pool_state.base_mint, pool_state.quote_mint
                )
            })?;
        extract_mint_pair(accounts, pool_state.base_mint, pool_state.quote_mint)?
    };

    let base_vault_accounts: Vec<RpcKeyedAccount> = client
        .get_token_accounts_by_owner(
            &TESSERA_V_GLOBAL_STATE,
            TokenAccountsFilter::Mint(pool_state.base_mint),
        )
        .with_context(|| {
            format!(
                "fetch Tessera vault accounts for mint {}",
                pool_state.base_mint
            )
        })?;
    let quote_vault_accounts: Vec<RpcKeyedAccount> = client
        .get_token_accounts_by_owner(
            &TESSERA_V_GLOBAL_STATE,
            TokenAccountsFilter::Mint(pool_state.quote_mint),
        )
        .with_context(|| {
            format!(
                "fetch Tessera vault accounts for mint {}",
                pool_state.quote_mint
            )
        })?;

    build_market_meta(
        market,
        pool_state,
        base_mint_account,
        quote_mint_account,
        base_vault_accounts,
        quote_vault_accounts,
    )
}

fn extract_mint_pair(
    accounts: Vec<Option<Account>>,
    base_mint: Pubkey,
    quote_mint: Pubkey,
) -> Result<[Account; 2]> {
    let mut iter = accounts.into_iter();
    let base = iter
        .next()
        .flatten()
        .with_context(|| format!("mint account {base_mint} not found"))?;
    let quote = iter
        .next()
        .flatten()
        .with_context(|| format!("mint account {quote_mint} not found"))?;
    Ok([base, quote])
}

fn build_market_meta(
    market: Pubkey,
    pool_state: PoolState,
    base_mint_account: Account,
    quote_mint_account: Account,
    base_vault_accounts: Vec<RpcKeyedAccount>,
    quote_vault_accounts: Vec<RpcKeyedAccount>,
) -> Result<TesseraVMarketMeta> {
    let (base_token_program, _) = read_mint_metadata(pool_state.base_mint, &base_mint_account)?;
    let (quote_token_program, _) = read_mint_metadata(pool_state.quote_mint, &quote_mint_account)?;

    let base_vault = extract_largest_balance_account(base_vault_accounts).with_context(|| {
        format!(
            "owner {TESSERA_V_GLOBAL_STATE} has no vault for mint {}",
            pool_state.base_mint
        )
    })?;
    let quote_vault = extract_largest_balance_account(quote_vault_accounts).with_context(|| {
        format!(
            "owner {TESSERA_V_GLOBAL_STATE} has no vault for mint {}",
            pool_state.quote_mint
        )
    })?;

    Ok(TesseraVMarketMeta {
        global_state: AccountMeta::new_readonly(TESSERA_V_GLOBAL_STATE, false),
        pool_account: AccountMeta::new(market, false),
        base_vault: AccountMeta::new(base_vault, false),
        quote_vault: AccountMeta::new(quote_vault, false),
        base_mint: AccountMeta::new_readonly(pool_state.base_mint, false),
        quote_mint: AccountMeta::new_readonly(pool_state.quote_mint, false),
        base_token_program: AccountMeta::new_readonly(base_token_program, false),
        quote_token_program: AccountMeta::new_readonly(quote_token_program, false),
        sysvar_instructions: AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_client::rpc_client::RpcClient;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    #[ignore]
    fn dump_live_market_meta() {
        let client = RpcClient::new("https://api.mainnet-beta.solana.com".to_string());
        let market = Pubkey::from_str("FLckHLGMJy5gEoXWwcE68Nprde1D4araK4TGLw4pQq2n")
            .expect("market pubkey");
        let meta = fetch_market_meta_blocking(&client, market).expect("fetch TesseraV meta");
        println!("{meta:#?}");
    }
}
