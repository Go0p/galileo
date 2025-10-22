use anyhow::{Context, Result, anyhow, bail, ensure};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use std::convert::TryInto;

use yellowstone_vixen_meteora_parser::accounts::LbPair;
use yellowstone_vixen_meteora_parser::accounts_parser::LbClmmProgramState;

pub const METEORA_DLMM_PROGRAM_ID: Pubkey = pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
pub const METEORA_DLMM_EVENT_AUTHORITY: Pubkey =
    pubkey!("D1ZN9Wj1fRSUQfCjhvnu1hqDMT7hzjzBBpi12nVniYD6");
const MEMO_PROGRAM_ID: Pubkey = pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

const BIN_ARRAY_RANGE: i32 = 50;
const BINS_PER_ARRAY: i32 = 70;
const BIN_ARRAY_SEED: &[u8] = b"bin_array";

#[derive(Debug, Clone)]
pub struct MeteoraDlmmMarketMeta {
    pub lb_pair: AccountMeta,
    pub reserve_x: AccountMeta,
    pub reserve_y: AccountMeta,
    pub token_x_mint: Pubkey,
    pub token_y_mint: Pubkey,
    pub token_x_mint_meta: AccountMeta,
    pub token_y_mint_meta: AccountMeta,
    pub token_x_program: Pubkey,
    pub token_y_program: Pubkey,
    pub token_x_program_meta: AccountMeta,
    pub token_y_program_meta: AccountMeta,
    pub oracle: AccountMeta,
    pub event_authority: AccountMeta,
    pub bin_array_bitmap_extension: Option<AccountMeta>,
    pub bin_arrays: Vec<AccountMeta>,
    pub memo_program: AccountMeta,
}

impl MeteoraDlmmMarketMeta {
    pub fn uses_token_2022(&self) -> bool {
        let token_2022 = token_program_2022_pubkey();
        self.token_x_program == token_2022 || self.token_y_program == token_2022
    }
}

pub async fn fetch_market_meta(
    client: &RpcClient,
    market: Pubkey,
    account: &Account,
) -> Result<MeteoraDlmmMarketMeta> {
    ensure!(
        account.owner == METEORA_DLMM_PROGRAM_ID,
        "Meteora DLMM 池 {market} 的 owner ({}) 与预期不符",
        account.owner
    );

    let pair = decode_pair(&account.data)
        .with_context(|| format!("解析 Meteora DLMM 池 {market} 账户数据失败"))?;

    let reserve_x = to_sdk_pubkey(&pair.reserve_x);
    let reserve_y = to_sdk_pubkey(&pair.reserve_y);
    let token_x_mint = to_sdk_pubkey(&pair.token_x_mint);
    let token_y_mint = to_sdk_pubkey(&pair.token_y_mint);
    let oracle = to_sdk_pubkey(&pair.oracle);

    let vault_accounts = client
        .get_multiple_accounts(&[reserve_x, reserve_y])
        .await
        .map_err(|err| anyhow!(err))
        .with_context(|| {
            format!(
                "获取 Meteora DLMM 池 {market} 的 vault 账户 {} / {}",
                reserve_x, reserve_y
            )
        })?;

    let reserve_x_account = vault_accounts
        .get(0)
        .and_then(|acc| acc.as_ref())
        .ok_or_else(|| anyhow!("Meteora DLMM 池 {market} 缺少 reserve_x 账户 {reserve_x}"))?;
    let reserve_y_account = vault_accounts
        .get(1)
        .and_then(|acc| acc.as_ref())
        .ok_or_else(|| anyhow!("Meteora DLMM 池 {market} 缺少 reserve_y 账户 {reserve_y}"))?;

    let token_x_program = reserve_x_account.owner;
    let token_y_program = reserve_y_account.owner;

    let bin_arrays = derive_bin_array_metas(&market, pair.active_id, &METEORA_DLMM_PROGRAM_ID);

    Ok(MeteoraDlmmMarketMeta {
        lb_pair: AccountMeta::new(market, false),
        reserve_x: AccountMeta::new(reserve_x, false),
        reserve_y: AccountMeta::new(reserve_y, false),
        token_x_mint,
        token_y_mint,
        token_x_mint_meta: AccountMeta::new_readonly(token_x_mint, false),
        token_y_mint_meta: AccountMeta::new_readonly(token_y_mint, false),
        token_x_program,
        token_y_program,
        token_x_program_meta: AccountMeta::new_readonly(token_x_program, false),
        token_y_program_meta: AccountMeta::new_readonly(token_y_program, false),
        oracle: AccountMeta::new(oracle, false),
        event_authority: AccountMeta::new_readonly(METEORA_DLMM_EVENT_AUTHORITY, false),
        bin_array_bitmap_extension: None,
        bin_arrays,
        memo_program: AccountMeta::new_readonly(MEMO_PROGRAM_ID, false),
    })
}

fn decode_pair(data: &[u8]) -> Result<LbPair> {
    match LbClmmProgramState::try_unpack(data)
        .map_err(|err| anyhow!("解析 Meteora DLMM 池账户失败: {:?}", err))?
    {
        LbClmmProgramState::LbPair(pair) => Ok(pair),
        other => bail!("账户不是 Meteora DLMM LbPair，实际为 {:?}", other),
    }
}

fn derive_bin_array_metas(
    pair_address: &Pubkey,
    active_id: i32,
    program_id: &Pubkey,
) -> Vec<AccountMeta> {
    let lower_bin_id = active_id.saturating_sub(BIN_ARRAY_RANGE);
    let upper_bin_id = active_id.saturating_add(BIN_ARRAY_RANGE);

    let lower_index = bin_id_to_array_index(lower_bin_id);
    let upper_index = bin_id_to_array_index(upper_bin_id);

    (lower_index..=upper_index)
        .map(|index| {
            let address = derive_bin_array_address(pair_address, index, program_id);
            AccountMeta::new(address, false)
        })
        .collect()
}

fn bin_id_to_array_index(bin_id: i32) -> i32 {
    if bin_id >= 0 {
        bin_id / BINS_PER_ARRAY
    } else {
        (bin_id - BINS_PER_ARRAY + 1) / BINS_PER_ARRAY
    }
}

fn derive_bin_array_address(pair_address: &Pubkey, index: i32, program_id: &Pubkey) -> Pubkey {
    let index_bytes = index.to_le_bytes();
    Pubkey::find_program_address(
        &[BIN_ARRAY_SEED, pair_address.as_ref(), &index_bytes],
        program_id,
    )
    .0
}

fn to_sdk_pubkey(pk: &impl AsRef<[u8]>) -> Pubkey {
    let bytes: [u8; 32] = pk.as_ref().try_into().expect("pubkey length");
    Pubkey::new_from_array(bytes)
}

fn token_program_2022_pubkey() -> Pubkey {
    Pubkey::new_from_array(spl_token_2022::ID.to_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bin_id_to_array_index_handles_positive() {
        assert_eq!(bin_id_to_array_index(0), 0);
        assert_eq!(bin_id_to_array_index(140), 2);
    }

    #[test]
    fn bin_id_to_array_index_handles_negative() {
        assert_eq!(bin_id_to_array_index(-1), -1);
        assert_eq!(bin_id_to_array_index(-71), -2);
    }
}
