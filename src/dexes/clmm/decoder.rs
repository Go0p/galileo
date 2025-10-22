use anyhow::{Context, Result, anyhow, bail, ensure};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use std::convert::TryInto;

use yellowstone_vixen_raydium_clmm_parser::accounts::PoolState;
use yellowstone_vixen_raydium_clmm_parser::accounts_parser::AmmV3ProgramState;

pub const RAYDIUM_CLMM_PROGRAM_ID: Pubkey = pubkey!("CAMMCz6GM1DNKyvHRAAmcX6vLLANBYdmsKtfgfcJQ68X");
const MEMO_PROGRAM_ID: Pubkey = pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

const TICK_ARRAY_OFFSETS: [i32; 3] = [-1, 0, 1];
const TICK_ARRAY_SEED: &[u8] = b"tick_array";
const TICK_ARRAY_SIZE: i32 = 60;

#[derive(Debug, Clone)]
pub struct RaydiumClmmMarketMeta {
    pub amm_config: AccountMeta,
    pub pool_account: AccountMeta,
    pub observation_account: AccountMeta,
    pub base_vault: AccountMeta,
    pub quote_vault: AccountMeta,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_mint_meta: AccountMeta,
    pub quote_mint_meta: AccountMeta,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
    pub token_program: AccountMeta,
    pub token_program_2022: AccountMeta,
    pub memo_program: AccountMeta,
    pub tick_arrays: Vec<AccountMeta>,
}

impl RaydiumClmmMarketMeta {
    pub fn uses_token_2022(&self) -> bool {
        let token_2022 = token_2022_program_pubkey();
        self.base_token_program == token_2022 || self.quote_token_program == token_2022
    }
}

pub async fn fetch_market_meta(
    client: &RpcClient,
    market: Pubkey,
    account: &Account,
) -> Result<RaydiumClmmMarketMeta> {
    ensure!(
        account.owner == RAYDIUM_CLMM_PROGRAM_ID,
        "Raydium CLMM 池 {market} 的 owner ({}) 与预期不符",
        account.owner
    );

    let pool = decode_pool_state(&account.data)
        .with_context(|| format!("解析 Raydium CLMM 池 {market} 账户数据失败"))?;

    let pool_pubkey = market;
    let amm_config = to_sdk_pubkey(&pool.amm_config);
    let observation = to_sdk_pubkey(&pool.observation_key);
    let base_mint = to_sdk_pubkey(&pool.token_mint0);
    let quote_mint = to_sdk_pubkey(&pool.token_mint1);
    let base_vault_pubkey = to_sdk_pubkey(&pool.token_vault0);
    let quote_vault_pubkey = to_sdk_pubkey(&pool.token_vault1);

    let vault_accounts = client
        .get_multiple_accounts(&[base_vault_pubkey, quote_vault_pubkey])
        .await
        .map_err(|err| anyhow!(err))
        .with_context(|| {
            format!(
                "获取 Raydium CLMM 池 {market} 的 vault 账户 {} / {}",
                base_vault_pubkey, quote_vault_pubkey
            )
        })?;

    let base_vault_account = vault_accounts
        .get(0)
        .and_then(|acc| acc.as_ref())
        .ok_or_else(|| {
            anyhow!("Raydium CLMM 池 {market} 缺少 base vault 账户 {base_vault_pubkey}")
        })?;
    let quote_vault_account = vault_accounts
        .get(1)
        .and_then(|acc| acc.as_ref())
        .ok_or_else(|| {
            anyhow!("Raydium CLMM 池 {market} 缺少 quote vault 账户 {quote_vault_pubkey}")
        })?;

    let base_token_program = base_vault_account.owner;
    let quote_token_program = quote_vault_account.owner;

    let tick_arrays = derive_tick_array_metas(
        &pool_pubkey,
        pool.tick_current,
        pool.tick_spacing,
        &RAYDIUM_CLMM_PROGRAM_ID,
    );

    Ok(RaydiumClmmMarketMeta {
        amm_config: AccountMeta::new_readonly(amm_config, false),
        pool_account: AccountMeta::new(pool_pubkey, false),
        observation_account: AccountMeta::new(observation, false),
        base_vault: AccountMeta::new(base_vault_pubkey, false),
        quote_vault: AccountMeta::new(quote_vault_pubkey, false),
        base_mint,
        quote_mint,
        base_mint_meta: AccountMeta::new_readonly(base_mint, false),
        quote_mint_meta: AccountMeta::new_readonly(quote_mint, false),
        base_token_program,
        quote_token_program,
        token_program: AccountMeta::new_readonly(token_program_pubkey(), false),
        token_program_2022: AccountMeta::new_readonly(token_2022_program_pubkey(), false),
        memo_program: AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
        tick_arrays,
    })
}

fn decode_pool_state(data: &[u8]) -> Result<PoolState> {
    match AmmV3ProgramState::try_unpack(data)
        .map_err(|err| anyhow!("解析 Raydium CLMM 池账户失败: {:?}", err))?
    {
        AmmV3ProgramState::PoolState(pool) => Ok(pool),
        other => bail!("账户不是 Raydium CLMM PoolState，实际为 {:?}", other),
    }
}

fn derive_tick_array_metas(
    pool: &Pubkey,
    tick_current: i32,
    tick_spacing: u16,
    program_id: &Pubkey,
) -> Vec<AccountMeta> {
    TICK_ARRAY_OFFSETS
        .iter()
        .map(|offset| {
            let start_index = compute_tick_array_start_index(tick_current, tick_spacing, *offset);
            let tick_array = derive_tick_array_address(pool, start_index, program_id);
            AccountMeta::new(tick_array, false)
        })
        .collect()
}

fn compute_tick_array_start_index(tick_current: i32, tick_spacing: u16, offset: i32) -> i32 {
    let ticks_in_array = TICK_ARRAY_SIZE * tick_spacing as i32;
    let mut start = tick_current / ticks_in_array;
    if tick_current < 0 && tick_current % ticks_in_array != 0 {
        start -= 1;
    }
    let base_start = start * ticks_in_array;
    base_start + offset * ticks_in_array
}

fn derive_tick_array_address(pool: &Pubkey, start_index: i32, program_id: &Pubkey) -> Pubkey {
    let start_bytes = start_index.to_be_bytes();
    Pubkey::find_program_address(&[TICK_ARRAY_SEED, pool.as_ref(), &start_bytes], program_id).0
}

fn to_sdk_pubkey(pk: &impl AsRef<[u8]>) -> Pubkey {
    let bytes: [u8; 32] = pk.as_ref().try_into().expect("pubkey length");
    Pubkey::new_from_array(bytes)
}

fn token_program_pubkey() -> Pubkey {
    Pubkey::new_from_array(spl_token::ID.to_bytes())
}

fn token_2022_program_pubkey() -> Pubkey {
    Pubkey::new_from_array(spl_token_2022::ID.to_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_tick_array_start_index_handles_positive_ticks() {
        let start = compute_tick_array_start_index(120, 1, 0);
        assert_eq!(start, 120);
        let prev = compute_tick_array_start_index(120, 1, -1);
        assert_eq!(prev, 60);
    }

    #[test]
    fn compute_tick_array_start_index_handles_negative_ticks() {
        let start = compute_tick_array_start_index(-1, 1, 0);
        assert_eq!(start, -60);
        let next = compute_tick_array_start_index(-1, 1, 1);
        assert_eq!(next, 0);
    }
}
