use anyhow::{Result, anyhow, ensure};
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, sysvar};
use tracing::debug;

pub const ZEROFI_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("ZERor4xhbUycZ6gb9ntrhqscUcZmAbQDjEAtCf4hbZY");

const AUTHORITY_WHITELIST: &[Pubkey] = &[
    solana_sdk::pubkey!("Sett1ereLzRw7neSzoUSwp6vvstBkEgAgQeP6wFcw5F"),
    solana_sdk::pubkey!("ELF5Z2V7ocaSnxE8cVESrKjwyydyn3kKqwPcj57ADvKm"),
    solana_sdk::pubkey!("2UUgGySTVXmKFatH7pGQo84ZrzdSYF5zw9iqrGwBMuuj"),
];

const OFFSET_BASE_MINT: usize = 0x0048;
const OFFSET_QUOTE_MINT: usize = 0x0068;
const OFFSET_VAULT_BASE: usize = 0x0088;
const OFFSET_VAULT_INFO_BASE: usize = 0x00A8;
const OFFSET_VAULT_QUOTE: usize = 0x00C8;
const OFFSET_VAULT_INFO_QUOTE: usize = 0x00E8;
const OFFSET_SWAP_AUTHORITY: usize = 0x0100;
const FLAG_OFFSET_TOKEN_2022: usize = 0x0791;
const MIN_ACCOUNT_LEN: usize = OFFSET_SWAP_AUTHORITY + 32;

/*
swap accounts:
    pair: writable
    vault_info_base: writable
    vault_base: writable
    vault_info_quote: writable
    vault_quote: writable
    user_base_token_account: writable
    user_quote_token_account: writable
    payer: writable,signer
    token_program: readonly
    sysvar_instructions: readonly
*/
#[derive(Debug, Clone)]
pub struct ZeroFiMarketMeta {
    pub pair_account: AccountMeta,
    pub vault_info_base: AccountMeta,
    pub vault_base: AccountMeta,
    pub vault_info_quote: AccountMeta,
    pub vault_quote: AccountMeta,
    pub token_program: AccountMeta,
    pub sysvar_instructions: AccountMeta,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    swap_authority_hint: Pubkey,
}

impl ZeroFiMarketMeta {
    pub fn base_mint(&self) -> Pubkey {
        self.base_mint
    }

    pub fn quote_mint(&self) -> Pubkey {
        self.quote_mint
    }

    pub fn base_token_program(&self) -> Pubkey {
        self.token_program.pubkey
    }

    pub fn quote_token_program(&self) -> Pubkey {
        self.token_program.pubkey
    }

    pub fn swap_authority_hint(&self) -> Pubkey {
        self.swap_authority_hint
    }
}

pub fn decode_market_meta(market: Pubkey, data: &[u8]) -> Result<ZeroFiMarketMeta> {
    ensure!(
        data.len() >= MIN_ACCOUNT_LEN,
        "ZeroFi 市场 {market} 数据长度不足: {} 字节",
        data.len()
    );

    let base_mint = read_pubkey(data, OFFSET_BASE_MINT)?;
    let quote_mint = read_pubkey(data, OFFSET_QUOTE_MINT)?;
    let vault_base = read_pubkey(data, OFFSET_VAULT_BASE)?;
    let vault_info_base = read_pubkey(data, OFFSET_VAULT_INFO_BASE)?;
    let vault_quote = read_pubkey(data, OFFSET_VAULT_QUOTE)?;
    let vault_info_quote = read_pubkey(data, OFFSET_VAULT_INFO_QUOTE)?;
    let swap_authority_hint = read_pubkey(data, OFFSET_SWAP_AUTHORITY)?;

    if !AUTHORITY_WHITELIST.contains(&swap_authority_hint) {
        debug!(
            target: "dex::zerofi",
            market = %market,
            swap_authority = %swap_authority_hint,
            "ZeroFi 池 swap authority 不在默认白名单中"
        );
    }

    ensure!(
        FLAG_OFFSET_TOKEN_2022 < data.len(),
        "ZeroFi 市场 {market} 缺少 token program 标志位"
    );
    let uses_token_2022 = (data[FLAG_OFFSET_TOKEN_2022] & 1) != 0;
    let token_program = if uses_token_2022 {
        Pubkey::new_from_array(spl_token_2022::id().to_bytes())
    } else {
        Pubkey::new_from_array(spl_token::id().to_bytes())
    };

    Ok(ZeroFiMarketMeta {
        pair_account: AccountMeta::new(market, false),
        vault_info_base: AccountMeta::new(vault_info_base, false),
        vault_base: AccountMeta::new(vault_base, false),
        vault_info_quote: AccountMeta::new(vault_info_quote, false),
        vault_quote: AccountMeta::new(vault_quote, false),
        token_program: AccountMeta::new_readonly(token_program, false),
        sysvar_instructions: AccountMeta::new_readonly(sysvar::instructions::ID, false),
        base_mint,
        quote_mint,
        swap_authority_hint,
    })
}

fn read_pubkey(data: &[u8], offset: usize) -> Result<Pubkey> {
    let end = offset
        .checked_add(32)
        .ok_or_else(|| anyhow!("ZeroFi 偏移溢出"))?;
    ensure!(
        end <= data.len(),
        "ZeroFi 市场账户数据在偏移 {offset:#x} 长度不足"
    );
    let mut buffer = [0u8; 32];
    buffer.copy_from_slice(&data[offset..end]);
    Ok(Pubkey::new_from_array(buffer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_pubkey_out_of_bounds() {
        let data = vec![0u8; 16];
        let err = read_pubkey(&data, 0).unwrap_err();
        assert!(err.to_string().contains("长度不足"));
    }
}
