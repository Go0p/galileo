use std::convert::TryInto;
use std::str::FromStr;

use anyhow::{Context, Result, anyhow, bail, ensure};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use solana_account_decoder::{UiAccountData, UiAccountEncoding};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_client::RpcClient as BlockingRpcClient;
use solana_client::rpc_request::TokenAccountsFilter;
use solana_client::rpc_response::RpcKeyedAccount;
use solana_sdk::{account::Account, instruction::AccountMeta, pubkey::Pubkey, sysvar};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

pub const HUMIDIFI_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("9H6tua7jkLhdm3w8BvgpTn5LZNU7g4ZynDmCiNN3q6Rp");
const QUOTE_MINT_OFFSET: usize = 0x180;
const BASE_MINT_OFFSET: usize = 0x1A0;
const SWAP_ID_OFFSET: usize = 0x2B0;
const SWAP_ID_MASK: u64 = 0x6E9DE2B30B19F9EA;
const MINT_MASKS: [u64; 4] = [
    0xFB5CE87AAE443C38,
    0x04A2178451BAC3C7,
    0x04A1178751B9C3C6,
    0x04A0178651B8C3C5,
];

/*
swap accounts:
user_transfer_authority: writable,signer,
pool: writable,
base_vault: writable,
quote_vault: writable,
user_base_token: writable,
user_quote_token: writable,
sysvar_clock: readonly,
token_program: readonly,
sysvar_instructions: readonly,
*/
#[derive(Debug, Clone)]
pub struct HumidiFiMarketMeta {
    pub pool_account: AccountMeta,
    pub base_vault: AccountMeta,
    pub quote_vault: AccountMeta,
    pub base_mint: AccountMeta,
    pub quote_mint: AccountMeta,
    pub token_program: AccountMeta,
    pub sysvar_clock: AccountMeta,
    pub sysvar_instructions: AccountMeta,
    swap_id: Arc<SwapIdTracker>,
}

impl HumidiFiMarketMeta {
    #[allow(clippy::too_many_arguments)]
    fn new(
        pool_state: Pubkey,
        base_mint: Pubkey,
        quote_mint: Pubkey,
        base_vault: Pubkey,
        quote_vault: Pubkey,
        token_program: Pubkey,
        last_swap_id: u64,
    ) -> Result<Self> {
        let next = last_swap_id
            .checked_add(1)
            .ok_or_else(|| anyhow!("HumidiFi 池 {pool_state} 的 swap_id 已达上限"))?;
        Ok(Self {
            pool_account: AccountMeta::new(pool_state, false),
            base_vault: AccountMeta::new(base_vault, false),
            quote_vault: AccountMeta::new(quote_vault, false),
            base_mint: AccountMeta::new_readonly(base_mint, false),
            quote_mint: AccountMeta::new_readonly(quote_mint, false),
            token_program: AccountMeta::new_readonly(token_program, false),
            sysvar_clock: AccountMeta::new_readonly(sysvar::clock::ID, false),
            sysvar_instructions: AccountMeta::new_readonly(sysvar::instructions::ID, false),
            swap_id: Arc::new(SwapIdTracker::new(next)),
        })
    }

    pub fn next_swap_id(&self) -> Result<u64> {
        self.swap_id.next()
    }
}

#[derive(Debug)]
struct SwapIdTracker {
    counter: AtomicU64,
}

impl SwapIdTracker {
    fn new(start: u64) -> Self {
        Self {
            counter: AtomicU64::new(start),
        }
    }

    fn next(&self) -> Result<u64> {
        let mut current = self.counter.load(Ordering::Relaxed);
        loop {
            if current == u64::MAX {
                bail!("HumidiFi swap_id 计数器溢出");
            }
            match self.counter.compare_exchange(
                current,
                current + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Ok(current),
                Err(observed) => current = observed,
            }
        }
    }
}

pub async fn fetch_market_meta(
    client: &RpcClient,
    market: Pubkey,
    account: &Account,
) -> Result<HumidiFiMarketMeta> {
    ensure!(
        account.owner == HUMIDIFI_PROGRAM_ID,
        "账户 {market} 的程序并非 HumidiFi: {}",
        account.owner
    );

    let config = parse_config_account(market, account)?;

    let base_vault_accounts: Vec<RpcKeyedAccount> = client
        .get_token_accounts_by_owner(&market, TokenAccountsFilter::Mint(config.base_mint))
        .await
        .map_err(|err| anyhow::Error::new(err))
        .with_context(|| {
            format!(
                "获取 HumidiFi 池 {market} base mint {} 的 vault 列表",
                config.base_mint
            )
        })?;
    let quote_vault_accounts: Vec<RpcKeyedAccount> = client
        .get_token_accounts_by_owner(&market, TokenAccountsFilter::Mint(config.quote_mint))
        .await
        .map_err(|err| anyhow::Error::new(err))
        .with_context(|| {
            format!(
                "获取 HumidiFi 池 {market} quote mint {} 的 vault 列表",
                config.quote_mint
            )
        })?;

    build_market_meta(market, &config, base_vault_accounts, quote_vault_accounts)
}

#[allow(dead_code)]
pub fn fetch_market_meta_blocking(
    client: &BlockingRpcClient,
    market: Pubkey,
) -> Result<HumidiFiMarketMeta> {
    let account = client
        .get_account(&market)
        .with_context(|| format!("获取 HumidiFi 池配置账户 {market}"))?;
    ensure!(
        account.owner == HUMIDIFI_PROGRAM_ID,
        "账户 {market} 的程序并非 HumidiFi: {}",
        account.owner
    );

    let config = parse_config_account(market, &account)?;

    let base_vault_accounts = client
        .get_token_accounts_by_owner(&market, TokenAccountsFilter::Mint(config.base_mint))
        .with_context(|| {
            format!(
                "查询 HumidiFi 池 {market} base mint {} 的 token 账户",
                config.base_mint
            )
        })?;
    let quote_vault_accounts = client
        .get_token_accounts_by_owner(&market, TokenAccountsFilter::Mint(config.quote_mint))
        .with_context(|| {
            format!(
                "查询 HumidiFi 池 {market} quote mint {} 的 token 账户",
                config.quote_mint
            )
        })?;

    build_market_meta(market, &config, base_vault_accounts, quote_vault_accounts)
}

fn build_market_meta(
    market: Pubkey,
    config: &ConfigAccount,
    base_vault_accounts: Vec<RpcKeyedAccount>,
    quote_vault_accounts: Vec<RpcKeyedAccount>,
) -> Result<HumidiFiMarketMeta> {
    let (base_vault, base_token_program) =
        select_vault(base_vault_accounts, config.base_mint, market, "base")?;
    let (quote_vault, quote_token_program) =
        select_vault(quote_vault_accounts, config.quote_mint, market, "quote")?;

    if base_token_program != quote_token_program {
        bail!(
            "HumidiFi 池 {market} 的 base/quote token 程序不一致: {} vs {}",
            base_token_program,
            quote_token_program
        );
    }

    HumidiFiMarketMeta::new(
        market,
        config.base_mint,
        config.quote_mint,
        base_vault,
        quote_vault,
        base_token_program,
        config.last_swap_id,
    )
}

#[derive(Debug, Clone, Copy)]
struct ConfigAccount {
    base_mint: Pubkey,
    quote_mint: Pubkey,
    last_swap_id: u64,
}

fn parse_config_account(market: Pubkey, account: &Account) -> Result<ConfigAccount> {
    let base_mint = decode_masked_pubkey(&account.data, BASE_MINT_OFFSET).with_context(|| {
        format!(
            "解码 HumidiFi 池 {market} base mint，账户数据长度 {}",
            account.data.len()
        )
    })?;
    let quote_mint = decode_masked_pubkey(&account.data, QUOTE_MINT_OFFSET).with_context(|| {
        format!(
            "解码 HumidiFi 池 {market} quote mint，账户数据长度 {}",
            account.data.len()
        )
    })?;
    let last_swap_id = decode_swap_id(&account.data).with_context(|| {
        format!(
            "解码 HumidiFi 池 {market} 的 last_swap_id，账户数据长度 {}",
            account.data.len()
        )
    })?;

    Ok(ConfigAccount {
        base_mint,
        quote_mint,
        last_swap_id,
    })
}

fn decode_masked_pubkey(data: &[u8], offset: usize) -> Result<Pubkey> {
    ensure!(
        data.len() >= offset + 32,
        "HumidiFi 配置账户长度不足，offset {offset} 无法解码 pubkey (长度 {})",
        data.len()
    );
    let mut out = [0u8; 32];
    for (idx, mask) in MINT_MASKS.iter().enumerate() {
        let start = offset + idx * 8;
        let end = start + 8;
        let chunk: [u8; 8] = data[start..end]
            .try_into()
            .expect("slice bounds verified above");
        let value = u64::from_le_bytes(chunk) ^ mask;
        out[idx * 8..(idx + 1) * 8].copy_from_slice(&value.to_le_bytes());
    }
    Ok(Pubkey::new_from_array(out))
}

fn decode_swap_id(data: &[u8]) -> Result<u64> {
    ensure!(
        data.len() >= SWAP_ID_OFFSET + 8,
        "HumidiFi 配置账户长度不足，无法读取 swap_id (长度 {})",
        data.len()
    );
    let chunk: [u8; 8] = data[SWAP_ID_OFFSET..SWAP_ID_OFFSET + 8]
        .try_into()
        .expect("slice bounds verified above");
    Ok(u64::from_le_bytes(chunk) ^ SWAP_ID_MASK)
}

fn select_vault(
    accounts: Vec<RpcKeyedAccount>,
    expected_mint: Pubkey,
    owner: Pubkey,
    label: &str,
) -> Result<(Pubkey, Pubkey)> {
    let mut best: Option<VaultCandidate> = None;

    for entry in accounts {
        let account_pubkey = Pubkey::from_str(&entry.pubkey)
            .with_context(|| format!("解析 HumidiFi {label} vault pubkey {}", entry.pubkey))?;
        let token_program = Pubkey::from_str(&entry.account.owner).with_context(|| {
            format!(
                "解析 HumidiFi {label} vault {} 的 owner {}",
                entry.pubkey, entry.account.owner
            )
        })?;

        if let Some(snapshot) = parse_token_account_snapshot(&entry.account.data) {
            if snapshot.mint != expected_mint {
                continue;
            }
            if best
                .as_ref()
                .map_or(true, |current| snapshot.amount > current.amount)
            {
                best = Some(VaultCandidate {
                    vault: account_pubkey,
                    token_program,
                    amount: snapshot.amount,
                });
            }
        }
    }

    let chosen = best.ok_or_else(|| {
        anyhow::anyhow!("HumidiFi 池 {owner} 未找到 mint {expected_mint} 的 {label} vault")
    })?;

    Ok((chosen.vault, chosen.token_program))
}

#[derive(Debug)]
struct VaultCandidate {
    vault: Pubkey,
    token_program: Pubkey,
    amount: u64,
}

#[derive(Debug)]
struct TokenAccountSnapshot {
    mint: Pubkey,
    amount: u64,
}

fn parse_token_account_snapshot(data: &UiAccountData) -> Option<TokenAccountSnapshot> {
    match data {
        UiAccountData::Json(json_account) => {
            let parsed = json_account.parsed.as_object()?;
            let info = parsed.get("info")?.as_object()?;
            let mint = info.get("mint")?.as_str()?;
            let mint = Pubkey::from_str(mint).ok()?;
            let token_amount = info.get("tokenAmount")?.as_object()?;
            let amount = token_amount.get("amount")?.as_str()?;
            let amount = amount.parse().ok()?;
            Some(TokenAccountSnapshot { mint, amount })
        }
        UiAccountData::Binary(encoded, UiAccountEncoding::Base64) => {
            let raw = BASE64.decode(encoded.as_bytes()).ok()?;
            if raw.len() < 72 {
                return None;
            }
            let mint = Pubkey::new_from_array(raw[0..32].try_into().ok()?);
            let amount = u64::from_le_bytes(raw[64..72].try_into().ok()?);
            Some(TokenAccountSnapshot { mint, amount })
        }
        UiAccountData::Binary(_encoded, _) => None,
        UiAccountData::LegacyBinary(_) => None,
    }
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
        let pool =
            Pubkey::from_str("HeDpLVZDrsG3UeoBff86cw2S8CgTJNdWNgqafqaprmFN").expect("pool pubkey");
        let meta = fetch_market_meta_blocking(&client, pool).expect("fetch HumidiFi meta");
        println!("{meta:#?}");
    }
}
