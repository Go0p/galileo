use std::collections::HashMap;

use anyhow::{Result, anyhow, ensure};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{account::Account, instruction::AccountMeta, pubkey, pubkey::Pubkey};
use spl_token::{solana_program::program_pack::Pack, state::Account as SplTokenAccount};

pub const OBRIC_V2_PROGRAM_ID: Pubkey = pubkey!("obriQD1zbpyLz95G5n7nJe6a4DPjpFwa5XYPoNm113y");
const TOKEN_PROGRAM_V1: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const TOKEN_PROGRAM_2022: Pubkey = pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const SYSVAR_INSTRUCTIONS_ID: Pubkey = pubkey!("Sysvar1nstructions1111111111111111111111111");

const PYTH_V2_PROGRAM_IDS: [Pubkey; 3] = [
    pubkey!("Fs2X9M7wrp7YjgJvDkXsp1p8Dd1zv9VHtV6nQWmvMRdq"),
    pubkey!("FsLevCLxwJhi3F7S3w7Dnk3a1JpN96CBrum1BgqsSVqP"),
    pubkey!("Minimox7jqQmMpF6Z34DTNwE9iJyNkruzvvYQRaHpAP"),
];

const TRADING_PAIR_DISCRIMINATOR: [u8; 8] = [0x3b, 0xde, 0x0f, 0xec, 0x62, 0x66, 0x5a, 0xe0];
const MAX_SAMPLE_ACCOUNTS: usize = 8;
const MULTI_FETCH_CHUNK: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObricSwapOrder {
    Swap,
    Swap2,
}

#[derive(Debug, Clone)]
pub struct ObricTradingPairAccounts {
    pub order: ObricSwapOrder,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub swap_authority: Pubkey,
    pub x_price_feed: Pubkey,
    pub y_price_feed: Pubkey,
    pub reference_oracle: Pubkey,
    pub second_reference_oracle: Option<Pubkey>,
    pub third_reference_oracle: Option<Pubkey>,
    pub mint_x_pool: Option<Pubkey>,
    pub mint_y_pool: Option<Pubkey>,
    pub sysvar_instructions: Option<Pubkey>,
    pub token_program: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
}

#[derive(Debug, Clone)]
pub struct ObricV2MarketMeta {
    pub order: ObricSwapOrder,
    pub trading_pair: AccountMeta,
    pub second_reference_oracle: Option<AccountMeta>,
    pub third_reference_oracle: Option<AccountMeta>,
    pub reserve_x: AccountMeta,
    pub reserve_y: AccountMeta,
    pub reference_oracle: AccountMeta,
    pub x_price_feed: AccountMeta,
    pub y_price_feed: AccountMeta,
    pub mint_x_pool: Option<AccountMeta>,
    pub mint_y_pool: Option<AccountMeta>,
    pub sysvar_instructions: Option<AccountMeta>,
    pub swap_authority: AccountMeta,
    pub token_program: AccountMeta,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
}

#[derive(Clone)]
struct TokenCandidate {
    index: usize,
    pubkey: Pubkey,
    amount: u64,
    mint: Pubkey,
    owner: Pubkey,
    token_program: Pubkey,
}

pub async fn decode_trading_pair_accounts(
    client: &RpcClient,
    _market: Pubkey,
    data: &[u8],
) -> Result<ObricTradingPairAccounts> {
    ensure!(
        data.starts_with(&TRADING_PAIR_DISCRIMINATOR),
        "obric trading pair missing discriminator"
    );

    let body = &data[TRADING_PAIR_DISCRIMINATOR.len()..];
    ensure!(!body.is_empty(), "obric trading pair account body is empty");

    let mut cache: HashMap<Pubkey, Option<Account>> = HashMap::new();
    let pubkeys = if let Some(offset) = find_double_feed_offset(body) {
        let keys = extract_pubkeys_from_offset(body, offset);
        fetch_accounts(client, &keys, &mut cache).await?;
        keys
    } else {
        let candidates = build_candidate_offsets(body);
        ensure!(
            !candidates.is_empty(),
            "obric trading pair missing candidate layouts"
        );

        let mut sample: Vec<Pubkey> = Vec::new();
        for keys in &candidates {
            sample.extend(keys.iter().take(MAX_SAMPLE_ACCOUNTS).copied());
        }
        fetch_accounts(client, &sample, &mut cache).await?;

        let mut best_idx: Option<usize> = None;
        let mut best_hits = -1i32;
        for (idx, keys) in candidates.iter().enumerate() {
            let hits = keys
                .iter()
                .take(MAX_SAMPLE_ACCOUNTS)
                .filter(|key| cache.get(key).and_then(|v| v.as_ref()).is_some())
                .count() as i32;
            if hits > best_hits {
                best_hits = hits;
                best_idx = Some(idx);
            }
        }

        let chosen = best_idx.ok_or_else(|| anyhow!("failed to align obric trading pair"))?;
        ensure!(best_hits > 0, "obric trading pair alignment score is zero");
        let keys = candidates.into_iter().nth(chosen).unwrap();
        fetch_accounts(client, &keys, &mut cache).await?;
        keys
    };

    ensure!(
        !pubkeys.is_empty(),
        "obric trading pair contains no embedded accounts"
    );

    let mut token_by_mint: HashMap<Pubkey, Vec<TokenCandidate>> = HashMap::new();
    let mut price_feeds: Vec<(usize, Pubkey)> = Vec::new();
    let mut oracle_candidates: Vec<(usize, Pubkey)> = Vec::new();
    let mut obric_owned: Vec<(usize, Pubkey)> = Vec::new();
    let mut sysvar_present = false;

    for (index, key) in pubkeys.iter().enumerate() {
        if *key == SYSVAR_INSTRUCTIONS_ID {
            sysvar_present = true;
            continue;
        }

        let Some(account_opt) = cache.get(key) else {
            continue;
        };
        let Some(account) = account_opt else {
            continue;
        };

        if PYTH_V2_PROGRAM_IDS
            .iter()
            .any(|program_id| account.owner == *program_id)
        {
            price_feeds.push((index, *key));
            continue;
        }

        if account.owner == TOKEN_PROGRAM_V1 || account.owner == TOKEN_PROGRAM_2022 {
            if let Ok(state) = SplTokenAccount::unpack(&account.data) {
                let candidate = TokenCandidate {
                    index,
                    pubkey: *key,
                    amount: state.amount,
                    mint: Pubkey::new_from_array(state.mint.to_bytes()),
                    owner: Pubkey::new_from_array(state.owner.to_bytes()),
                    token_program: account.owner,
                };
                token_by_mint
                    .entry(candidate.mint)
                    .or_default()
                    .push(candidate);
            }
            continue;
        }

        if account.owner == OBRIC_V2_PROGRAM_ID {
            obric_owned.push((index, *key));
            continue;
        }

        oracle_candidates.push((index, *key));
    }

    ensure!(
        !price_feeds.is_empty(),
        "obric trading pair missing price feeds"
    );
    price_feeds.sort_by_key(|(idx, _)| *idx);
    let x_price_feed = price_feeds[0].1;
    let y_price_feed = price_feeds
        .get(1)
        .map(|(_, pk)| *pk)
        .unwrap_or(x_price_feed);

    let mut reserves: Vec<TokenCandidate> = Vec::new();
    for candidates in token_by_mint.values_mut() {
        candidates.sort_by(|a, b| b.amount.cmp(&a.amount));
        if let Some(best) = candidates.first() {
            reserves.push(best.clone());
        }
    }

    ensure!(
        reserves.len() >= 2,
        "obric trading pair reserves insufficient: {} mints",
        reserves.len()
    );
    reserves.sort_by_key(|candidate| candidate.index);
    let reserve_x = reserves[0].clone();
    let reserve_y = reserves[1].clone();
    ensure!(
        reserve_x.owner == reserve_y.owner,
        "obric reserves have different owners"
    );

    let mut oracles = oracle_candidates;
    oracles.sort_by_key(|(idx, _)| *idx);
    let reference_oracle = oracles
        .get(0)
        .map(|(_, pk)| *pk)
        .ok_or_else(|| anyhow!("obric trading pair missing reference oracle"))?;
    let second_reference_oracle = oracles.get(1).map(|(_, pk)| *pk);
    let third_reference_oracle = oracles.get(2).map(|(_, pk)| *pk);

    let order = if sysvar_present {
        ObricSwapOrder::Swap2
    } else {
        ensure!(
            second_reference_oracle.is_some() && third_reference_oracle.is_some(),
            "swap order missing secondary oracles"
        );
        ObricSwapOrder::Swap
    };

    obric_owned.sort_by_key(|(idx, _)| *idx);
    let mint_x_pool = obric_owned.get(0).map(|(_, pk)| *pk);
    let mint_y_pool = Some(reserve_y.mint);

    Ok(ObricTradingPairAccounts {
        order,
        reserve_x: reserve_x.pubkey,
        reserve_y: reserve_y.pubkey,
        swap_authority: reserve_x.owner,
        x_price_feed,
        y_price_feed,
        reference_oracle,
        second_reference_oracle,
        third_reference_oracle,
        mint_x_pool,
        mint_y_pool,
        sysvar_instructions: if sysvar_present {
            Some(SYSVAR_INSTRUCTIONS_ID)
        } else {
            None
        },
        token_program: reserve_x.token_program,
        base_mint: reserve_x.mint,
        quote_mint: reserve_y.mint,
    })
}

fn find_double_feed_offset(body: &[u8]) -> Option<usize> {
    if body.len() < 64 {
        return None;
    }
    for idx in 0..=(body.len() - 64) {
        if &body[idx..idx + 32] == &body[idx + 32..idx + 64] {
            return Some(idx % 32);
        }
    }
    None
}

fn extract_pubkeys_from_offset(body: &[u8], offset: usize) -> Vec<Pubkey> {
    let mut keys = Vec::new();
    let mut pos = offset;
    while pos + 32 <= body.len() {
        let mut buf = [0u8; 32];
        buf.copy_from_slice(&body[pos..pos + 32]);
        keys.push(Pubkey::new_from_array(buf));
        pos += 32;
    }
    keys
}

fn build_candidate_offsets(body: &[u8]) -> Vec<Vec<Pubkey>> {
    (0..32)
        .map(|offset| extract_pubkeys_from_offset(body, offset))
        .filter(|keys| !keys.is_empty())
        .collect()
}

async fn fetch_accounts(
    client: &RpcClient,
    keys: &[Pubkey],
    cache: &mut HashMap<Pubkey, Option<Account>>,
) -> Result<()> {
    let mut pending: Vec<Pubkey> = Vec::new();
    for key in keys {
        if !cache.contains_key(key) {
            pending.push(*key);
            if pending.len() == MULTI_FETCH_CHUNK {
                load_accounts_batch(client, &pending, cache).await?;
                pending.clear();
            }
        }
    }
    if !pending.is_empty() {
        load_accounts_batch(client, &pending, cache).await?;
    }
    Ok(())
}

async fn load_accounts_batch(
    client: &RpcClient,
    keys: &[Pubkey],
    cache: &mut HashMap<Pubkey, Option<Account>>,
) -> Result<()> {
    let accounts = client.get_multiple_accounts(keys).await?;
    for (key, account) in keys.iter().copied().zip(accounts.into_iter()) {
        cache.insert(key, account);
    }
    Ok(())
}
