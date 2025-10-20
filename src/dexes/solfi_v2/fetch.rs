use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use super::decoder::decode_swap_info;

#[derive(Debug, Clone)]
pub struct SolfiV2SwapInfo {
    pub payer: Pubkey,
    pub pair: Pubkey,
    pub oracle_account: Pubkey,
    pub config_account: Pubkey,
    pub base_vault: Pubkey,
    pub quote_vault: Pubkey,
    pub user_base_account: Pubkey,
    pub user_quote_account: Pubkey,
    pub base_mint: Pubkey,
    pub quote_mint: Pubkey,
    pub base_token_program: Pubkey,
    pub quote_token_program: Pubkey,
    pub sysvar: Pubkey,
}

/// 仅返回执行 swap 所需的账户列表。
pub fn fetch_solfi_v2_swap_info(
    client: &RpcClient,
    market: Pubkey,
    payer: Pubkey,
    user_base_account: Pubkey,
    user_quote_account: Pubkey,
) -> Result<SolfiV2SwapInfo> {
    let market_data = client
        .get_account_data(&market)
        .with_context(|| format!("fetch SolFi v2 market account {market}"))?;
    decode_swap_info(
        market,
        &market_data,
        payer,
        user_base_account,
        user_quote_account,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    #[ignore]
    fn decode_live_market() {
        let client = RpcClient::new("http://127.0.0.1:8899".to_string());
        let market = Pubkey::from_str("65ZHSArs5XxPseKQbB1B4r16vDxMWnCxHMzogDAqiDUc").unwrap();
        let swap_info = fetch_solfi_v2_swap_info(
            &client,
            market,
            Pubkey::default(),
            Pubkey::default(),
            Pubkey::default(),
        )
        .unwrap();
        println!("{:#?}", swap_info);
    }
}
