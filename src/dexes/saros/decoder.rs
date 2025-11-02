use anyhow::{Result, anyhow, ensure};
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

pub const SAROS_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("SSwapUtytfBdBn1b9NUGG6foMVPtcWgpRU32HToDUZr");

const OFFSET_IS_INITIALIZED: usize = 0x00;
const OFFSET_BUMP: usize = 0x02;
const OFFSET_TOKEN_PROGRAM: usize = 0x03;
const OFFSET_TOKEN_A: usize = 0x23;
const OFFSET_TOKEN_B: usize = 0x43;
const OFFSET_POOL_MINT: usize = 0x63;
const OFFSET_TOKEN_A_MINT: usize = 0x83;
const OFFSET_TOKEN_B_MINT: usize = 0xA3;
const OFFSET_FEE_ACCOUNT: usize = 0xC3;
const MIN_ACCOUNT_LEN: usize = 0x144;

#[derive(Debug, Clone)]
pub struct SarosMarketMeta {
    pub swap_account: AccountMeta,
    pub authority_account: AccountMeta,
    pub token_a_vault: AccountMeta,
    pub token_b_vault: AccountMeta,
    pub pool_mint: AccountMeta,
    pub fee_account: AccountMeta,
    pub token_program: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
}

impl SarosMarketMeta {
    pub fn base_mint(&self) -> Pubkey {
        self.token_a_mint
    }

    pub fn quote_mint(&self) -> Pubkey {
        self.token_b_mint
    }

    pub fn base_token_program(&self) -> Pubkey {
        self.token_program
    }

    pub fn quote_token_program(&self) -> Pubkey {
        self.token_program
    }
}

pub fn decode_market_meta(market: Pubkey, data: &[u8]) -> Result<SarosMarketMeta> {
    ensure!(
        data.len() >= MIN_ACCOUNT_LEN,
        "Saros 池子 {market} 数据长度不足: {} 字节",
        data.len()
    );

    ensure!(
        data[OFFSET_IS_INITIALIZED] != 0,
        "Saros 池子 {market} 尚未初始化"
    );

    let bump_seed = data[OFFSET_BUMP];
    let token_program = read_pubkey(data, OFFSET_TOKEN_PROGRAM)?;
    let token_program_bytes = token_program.to_bytes();
    let spl_token_program = spl_token::id().to_bytes();
    let spl_token_2022_program = spl_token_2022::id().to_bytes();
    ensure!(
        token_program_bytes == spl_token_program || token_program_bytes == spl_token_2022_program,
        "Saros 池子 {market} token_program 非法: {token_program}"
    );

    let swap_account = AccountMeta::new(market, false);
    let (authority, derived_bump) =
        Pubkey::find_program_address(&[market.as_ref()], &SAROS_PROGRAM_ID);
    ensure!(
        bump_seed == derived_bump,
        "Saros 池子 {market} bump_seed 不匹配: 账户记录 {bump_seed} vs PDA {derived_bump}"
    );
    let authority_account = AccountMeta::new_readonly(authority, false);

    let token_a_vault = AccountMeta::new(read_pubkey(data, OFFSET_TOKEN_A)?, false);
    let token_b_vault = AccountMeta::new(read_pubkey(data, OFFSET_TOKEN_B)?, false);
    let pool_mint = AccountMeta::new(read_pubkey(data, OFFSET_POOL_MINT)?, false);
    let fee_account = AccountMeta::new(read_pubkey(data, OFFSET_FEE_ACCOUNT)?, false);
    let token_a_mint = read_pubkey(data, OFFSET_TOKEN_A_MINT)?;
    let token_b_mint = read_pubkey(data, OFFSET_TOKEN_B_MINT)?;

    Ok(SarosMarketMeta {
        swap_account,
        authority_account,
        token_a_vault,
        token_b_vault,
        pool_mint,
        fee_account,
        token_program,
        token_a_mint,
        token_b_mint,
    })
}

fn read_pubkey(data: &[u8], offset: usize) -> Result<Pubkey> {
    let end = offset
        .checked_add(32)
        .ok_or_else(|| anyhow!("Saros 偏移溢出"))?;
    ensure!(
        end <= data.len(),
        "Saros 池子账户数据在偏移 {offset:#x} 长度不足"
    );
    let mut buffer = [0u8; 32];
    buffer.copy_from_slice(&data[offset..end]);
    Ok(Pubkey::new_from_array(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_pubkey_oob() {
        let data = vec![0u8; 16];
        let err = read_pubkey(&data, 0).unwrap_err();
        assert!(err.to_string().contains("长度不足"));
    }
}
