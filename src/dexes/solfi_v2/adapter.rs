use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;

use crate::dexes::framework::{
    DexMarketMeta, DexMetaProvider, SwapAccountAssembler, SwapAccountsContext, SwapFlow,
};

use super::decoder::decode_market_meta;
use super::{SOLFI_V2_PROGRAM_ID, SolfiV2MarketMeta};

#[derive(Default)]
pub struct SolFiV2Adapter;

impl SolFiV2Adapter {
    pub fn shared() -> &'static Self {
        &ADAPTER
    }
}

static ADAPTER: SolFiV2Adapter = SolFiV2Adapter;

impl DexMarketMeta for SolfiV2MarketMeta {
    fn base_mint(&self) -> Pubkey {
        self.base_mint.pubkey
    }

    fn quote_mint(&self) -> Pubkey {
        self.quote_mint.pubkey
    }

    fn base_token_program(&self) -> Pubkey {
        self.base_token_program.pubkey
    }

    fn quote_token_program(&self) -> Pubkey {
        self.quote_token_program.pubkey
    }
}

impl DexMetaProvider for SolFiV2Adapter {
    type MarketMeta = SolfiV2MarketMeta;

    type FetchFuture<'a>
        = Pin<Box<dyn Future<Output = Result<Arc<Self::MarketMeta>>> + Send + 'a>>
    where
        Self: 'a;

    fn program_id(&self) -> Pubkey {
        SOLFI_V2_PROGRAM_ID
    }

    fn fetch_market_meta<'a>(
        &'a self,
        _client: &'a RpcClient,
        market: Pubkey,
        account: &'a Account,
    ) -> Self::FetchFuture<'a> {
        Box::pin(async move {
            let meta = decode_market_meta(market, &account.data)?;
            Ok(Arc::new(meta))
        })
    }
}

impl SwapAccountAssembler for SolFiV2Adapter {
    type MarketMeta = SolfiV2MarketMeta;

    fn assemble_remaining_accounts(
        &self,
        meta: &Self::MarketMeta,
        ctx: SwapAccountsContext,
        output: &mut Vec<AccountMeta>,
    ) {
        let (user_source, user_destination) = match ctx.flow {
            SwapFlow::QuoteToBase => (ctx.user_quote, ctx.user_base),
            SwapFlow::BaseToQuote => (ctx.user_base, ctx.user_quote),
        };

        debug_assert_eq!(meta.pair_account.pubkey, ctx.market);

        output.push(AccountMeta::new_readonly(SOLFI_V2_PROGRAM_ID, false));
        output.extend_from_slice(&[
            AccountMeta::new(ctx.payer, true),
            meta.pair_account.clone(),
            meta.oracle_account.clone(),
            meta.config_account.clone(),
            meta.base_vault.clone(),
            meta.quote_vault.clone(),
            AccountMeta::new(user_source, false),
            AccountMeta::new(user_destination, false),
            meta.base_mint.clone(),
            meta.quote_mint.clone(),
            meta.base_token_program.clone(),
            meta.quote_token_program.clone(),
            meta.sysvar_instructions.clone(),
        ]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    #[test]
    #[ignore]
    fn decode_live_market() {}
}
