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

use super::SAROS_PROGRAM_ID;
use super::decoder::{SarosMarketMeta, decode_market_meta};

#[derive(Default)]
pub struct SarosAdapter;

impl SarosAdapter {
    pub fn shared() -> &'static Self {
        &ADAPTER
    }
}

static ADAPTER: SarosAdapter = SarosAdapter;

impl DexMarketMeta for SarosMarketMeta {
    fn base_mint(&self) -> Pubkey {
        self.base_mint()
    }

    fn quote_mint(&self) -> Pubkey {
        self.quote_mint()
    }

    fn base_token_program(&self) -> Pubkey {
        self.base_token_program()
    }

    fn quote_token_program(&self) -> Pubkey {
        self.quote_token_program()
    }
}

impl DexMetaProvider for SarosAdapter {
    type MarketMeta = SarosMarketMeta;

    type FetchFuture<'a>
        = Pin<Box<dyn Future<Output = Result<Arc<Self::MarketMeta>>> + Send + 'a>>
    where
        Self: 'a;

    fn program_id(&self) -> Pubkey {
        SAROS_PROGRAM_ID
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

impl SwapAccountAssembler for SarosAdapter {
    type MarketMeta = SarosMarketMeta;

    fn assemble_remaining_accounts(
        &self,
        meta: &Self::MarketMeta,
        ctx: SwapAccountsContext,
        output: &mut Vec<AccountMeta>,
    ) {
        let (pool_source, pool_destination, user_source, user_destination) = match ctx.flow {
            SwapFlow::BaseToQuote => (
                meta.token_a_vault.clone(),
                meta.token_b_vault.clone(),
                ctx.user_base,
                ctx.user_quote,
            ),
            SwapFlow::QuoteToBase => (
                meta.token_b_vault.clone(),
                meta.token_a_vault.clone(),
                ctx.user_quote,
                ctx.user_base,
            ),
        };

        output.push(AccountMeta::new_readonly(SAROS_PROGRAM_ID, false));
        output.extend_from_slice(&[
            meta.swap_account.clone(),
            meta.authority_account.clone(),
            AccountMeta::new_readonly(ctx.payer, true),
            AccountMeta::new(user_source, false),
            pool_source,
            pool_destination,
            AccountMeta::new(user_destination, false),
            meta.pool_mint.clone(),
            meta.fee_account.clone(),
            AccountMeta::new_readonly(meta.token_program, false),
        ]);
    }
}
