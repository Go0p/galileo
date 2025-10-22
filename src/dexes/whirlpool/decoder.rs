use anyhow::{Context, Result, anyhow, bail, ensure};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use std::convert::TryInto;

use yellowstone_vixen_orca_whirlpool_parser::accounts::Whirlpool;
use yellowstone_vixen_orca_whirlpool_parser::accounts_parser::WhirlpoolProgramState;

pub const ORCA_WHIRLPOOL_PROGRAM_ID: Pubkey =
    pubkey!("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc");

const TICK_ARRAY_SEED: &[u8] = b"tick_array";
const TICK_ARRAY_SIZE: i32 = 88;

#[derive(Debug, Clone)]
pub struct WhirlpoolMarketMeta {
    pub pool_account: AccountMeta,
    pub oracle: AccountMeta,
    pub vault_a: AccountMeta,
    pub vault_b: AccountMeta,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_program: Pubkey,
    pub token_b_program: Pubkey,
    pub token_program: AccountMeta,
    pub tick_arrays: Vec<AccountMeta>,
}

impl WhirlpoolMarketMeta {
    pub fn uses_token_2022(&self) -> bool {
        self.token_program.pubkey == token_program_pubkey_2022()
    }
}

pub async fn fetch_market_meta(
    client: &RpcClient,
    market: Pubkey,
    account: &Account,
) -> Result<WhirlpoolMarketMeta> {
    ensure!(
        account.owner == ORCA_WHIRLPOOL_PROGRAM_ID,
        "Whirlpool 池 {market} 的 owner ({}) 与预期不符",
        account.owner
    );

    let whirlpool = decode_whirlpool(&account.data)
        .with_context(|| format!("解析 Whirlpool 池 {market} 账户数据失败"))?;

    let vault_a = to_sdk_pubkey(&whirlpool.token_vault_a);
    let vault_b = to_sdk_pubkey(&whirlpool.token_vault_b);
    let token_a_mint = to_sdk_pubkey(&whirlpool.token_mint_a);
    let token_b_mint = to_sdk_pubkey(&whirlpool.token_mint_b);

    let vault_accounts = client
        .get_multiple_accounts(&[vault_a, vault_b])
        .await
        .map_err(|err| anyhow!(err))
        .with_context(|| {
            format!(
                "获取 Whirlpool 池 {market} 的 vault 账户 {} / {}",
                vault_a, vault_b
            )
        })?;

    let vault_a_account = vault_accounts
        .get(0)
        .and_then(|acc| acc.as_ref())
        .ok_or_else(|| anyhow!("Whirlpool 池 {market} 缺少 token_vault_a 账户 {vault_a}"))?;
    let vault_b_account = vault_accounts
        .get(1)
        .and_then(|acc| acc.as_ref())
        .ok_or_else(|| anyhow!("Whirlpool 池 {market} 缺少 token_vault_b 账户 {vault_b}"))?;

    let token_a_program = vault_a_account.owner;
    let token_b_program = vault_b_account.owner;

    if token_a_program != token_b_program {
        bail!(
            "Whirlpool 池 {market} 出现跨不同 Token Program 的资产: {} vs {}",
            token_a_program,
            token_b_program
        );
    }

    let oracle = derive_oracle_address(&market);
    let tick_arrays = derive_tick_array_metas(
        &market,
        whirlpool.tick_current_index,
        whirlpool.tick_spacing,
        &ORCA_WHIRLPOOL_PROGRAM_ID,
    );

    Ok(WhirlpoolMarketMeta {
        pool_account: AccountMeta::new(market, false),
        oracle: AccountMeta::new_readonly(oracle, false),
        vault_a: AccountMeta::new(vault_a, false),
        vault_b: AccountMeta::new(vault_b, false),
        token_a_mint,
        token_b_mint,
        token_a_program,
        token_b_program,
        token_program: AccountMeta::new_readonly(token_a_program, false),
        tick_arrays,
    })
}

fn decode_whirlpool(data: &[u8]) -> Result<Whirlpool> {
    match WhirlpoolProgramState::try_unpack(data)
        .map_err(|err| anyhow!("解析 Whirlpool 池账户失败: {:?}", err))?
    {
        WhirlpoolProgramState::Whirlpool(pool) => Ok(pool),
        other => bail!("账户不是 Whirlpool PoolState，实际为 {:?}", other),
    }
}

fn derive_oracle_address(pool: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(&[b"oracle", pool.as_ref()], &ORCA_WHIRLPOOL_PROGRAM_ID).0
}

fn derive_tick_array_metas(
    pool: &Pubkey,
    tick_current: i32,
    tick_spacing: u16,
    program_id: &Pubkey,
) -> Vec<AccountMeta> {
    let forward = derive_tick_array_start_indexes(tick_current, tick_spacing, true);
    let reverse = derive_tick_array_start_indexes(tick_current, tick_spacing, false);

    let first = reverse.1.unwrap_or(reverse.0);
    let second = forward.0;
    let third = forward.1.unwrap_or(forward.0);

    [first, second, third]
        .into_iter()
        .map(|start| {
            let address = derive_tick_array_address(pool, start, program_id);
            AccountMeta::new(address, false)
        })
        .collect()
}

fn derive_tick_array_start_indexes(
    curr_tick: i32,
    tick_spacing: u16,
    a_to_b: bool,
) -> (i32, Option<i32>, Option<i32>) {
    let first = derive_first_tick_array_start_tick(curr_tick, tick_spacing, !a_to_b);
    let second = derive_next_start_tick(first, tick_spacing, a_to_b);
    let third = second.and_then(|value| derive_next_start_tick(value, tick_spacing, a_to_b));
    (first, second, third)
}

fn derive_first_tick_array_start_tick(curr_tick: i32, tick_spacing: u16, shifted: bool) -> i32 {
    let tick = if shifted {
        curr_tick + tick_spacing as i32
    } else {
        curr_tick
    };
    derive_start_tick(tick, tick_spacing)
}

fn derive_start_tick(tick: i32, tick_spacing: u16) -> i32 {
    let ticks_per_array = TICK_ARRAY_SIZE * tick_spacing as i32;
    let rem = tick % ticks_per_array;
    if tick < 0 && rem != 0 {
        tick - rem - ticks_per_array
    } else {
        tick - rem
    }
}

fn derive_next_start_tick(start_tick: i32, tick_spacing: u16, a_to_b: bool) -> Option<i32> {
    let ticks_per_array = TICK_ARRAY_SIZE * tick_spacing as i32;
    let next = if a_to_b {
        start_tick - ticks_per_array
    } else {
        start_tick + ticks_per_array
    };
    Some(next)
}

fn derive_tick_array_address(pool: &Pubkey, start_tick: i32, program_id: &Pubkey) -> Pubkey {
    let start_bytes = start_tick.to_string();
    Pubkey::find_program_address(
        &[TICK_ARRAY_SEED, pool.as_ref(), start_bytes.as_bytes()],
        program_id,
    )
    .0
}

fn to_sdk_pubkey(pk: &impl AsRef<[u8]>) -> Pubkey {
    let bytes: [u8; 32] = pk.as_ref().try_into().expect("pubkey length");
    Pubkey::new_from_array(bytes)
}

fn token_program_pubkey_2022() -> Pubkey {
    Pubkey::new_from_array(spl_token_2022::ID.to_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_start_tick_aligns_positive_values() {
        assert_eq!(derive_start_tick(176, 8), 0);
        assert_eq!(derive_start_tick(177, 8), 0);
    }

    #[test]
    fn derive_start_tick_aligns_negative_values() {
        assert_eq!(derive_start_tick(-1, 8), -704);
        assert_eq!(derive_start_tick(-704, 8), -704);
    }
}
