use anyhow::{Result, ensure};
use solana_sdk::{instruction::AccountMeta, pubkey, pubkey::Pubkey};

pub const OBRIC_V2_PROGRAM_ID: Pubkey = pubkey!("obriQD1zbpyLz95G5n7nJe6a4DPjpFwa5XYPoNm113y");

const TRADING_PAIR_DISCRIMINATOR: [u8; 8] = [0x3b, 0xde, 0x0f, 0xec, 0x62, 0x66, 0x5a, 0xe0];

#[derive(Debug, Clone)]
pub struct ObricTradingPairAccounts {
    pub x_price_feed: Pubkey,
    pub y_price_feed: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub second_reference_oracle: Pubkey,
    pub third_reference_oracle: Pubkey,
    pub reference_oracle: Pubkey,
}

#[derive(Debug, Clone)]
pub struct ObricV2MarketMeta {
    pub trading_pair: AccountMeta,
    pub second_reference_oracle: AccountMeta,
    pub third_reference_oracle: AccountMeta,
    pub reserve_x: AccountMeta,
    pub reserve_y: AccountMeta,
    pub reference_oracle: AccountMeta,
    pub x_price_feed: AccountMeta,
    pub y_price_feed: AccountMeta,
    pub token_program: AccountMeta,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
}

pub fn decode_trading_pair_accounts(data: &[u8]) -> Result<ObricTradingPairAccounts> {
    ensure!(
        data.starts_with(&TRADING_PAIR_DISCRIMINATOR),
        "obric trading pair missing discriminator"
    );

    let body = &data[TRADING_PAIR_DISCRIMINATOR.len()..];
    let offset = find_double_feed_offset(body)
        .ok_or_else(|| anyhow::anyhow!("failed to locate obric trading pair offset"))?;

    let mut pubkeys = Vec::new();
    let mut pos = offset;
    while pos + 32 <= body.len() {
        let mut buf = [0u8; 32];
        buf.copy_from_slice(&body[pos..pos + 32]);
        pubkeys.push(Pubkey::new_from_array(buf));
        pos += 32;
    }

    ensure!(
        pubkeys.len() >= 9,
        "obric trading pair account list truncated: {} entries",
        pubkeys.len()
    );

    Ok(ObricTradingPairAccounts {
        x_price_feed: pubkeys[0],
        y_price_feed: pubkeys[1],
        reserve_x: pubkeys[2],
        reserve_y: pubkeys[3],
        second_reference_oracle: pubkeys[6],
        third_reference_oracle: pubkeys[7],
        reference_oracle: pubkeys[8],
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
