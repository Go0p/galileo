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

use super::decoder::{RAYDIUM_CLMM_PROGRAM_ID, RaydiumClmmMarketMeta, fetch_market_meta};

#[derive(Default)]
pub struct RaydiumClmmAdapter;

impl RaydiumClmmAdapter {
    pub fn shared() -> &'static Self {
        &ADAPTER
    }
}

static ADAPTER: RaydiumClmmAdapter = RaydiumClmmAdapter;

impl DexMarketMeta for RaydiumClmmMarketMeta {
    fn base_mint(&self) -> Pubkey {
        self.base_mint
    }

    fn quote_mint(&self) -> Pubkey {
        self.quote_mint
    }

    fn base_token_program(&self) -> Pubkey {
        self.base_token_program
    }

    fn quote_token_program(&self) -> Pubkey {
        self.quote_token_program
    }
}

impl DexMarketMeta for Arc<RaydiumClmmMarketMeta> {
    fn base_mint(&self) -> Pubkey {
        self.base_mint
    }

    fn quote_mint(&self) -> Pubkey {
        self.quote_mint
    }

    fn base_token_program(&self) -> Pubkey {
        self.base_token_program
    }

    fn quote_token_program(&self) -> Pubkey {
        self.quote_token_program
    }
}

impl DexMetaProvider for RaydiumClmmAdapter {
    type MarketMeta = RaydiumClmmMarketMeta;

    type FetchFuture<'a>
        = Pin<Box<dyn Future<Output = Result<Arc<Self::MarketMeta>>> + Send + 'a>>
    where
        Self: 'a;

    fn program_id(&self) -> Pubkey {
        RAYDIUM_CLMM_PROGRAM_ID
    }

    fn fetch_market_meta<'a>(
        &'a self,
        client: &'a RpcClient,
        market: Pubkey,
        account: &'a Account,
    ) -> Self::FetchFuture<'a> {
        Box::pin(async move {
            let meta = fetch_market_meta(client, market, account).await?;
            Ok(Arc::new(meta))
        })
    }
}

impl SwapAccountAssembler for RaydiumClmmAdapter {
    type MarketMeta = RaydiumClmmMarketMeta;

    fn assemble_remaining_accounts(
        &self,
        meta: &Self::MarketMeta,
        ctx: SwapAccountsContext,
        output: &mut Vec<AccountMeta>,
    ) {
        debug_assert_eq!(meta.pool_account.pubkey, ctx.market);

        let use_v2 = meta.uses_token_2022();

        let (
            source_user,
            destination_user,
            source_vault,
            destination_vault,
            source_mint,
            destination_mint,
        ) = match ctx.flow {
            SwapFlow::BaseToQuote => (
                ctx.user_base,
                ctx.user_quote,
                meta.base_vault.clone(),
                meta.quote_vault.clone(),
                meta.base_mint_meta.clone(),
                meta.quote_mint_meta.clone(),
            ),
            SwapFlow::QuoteToBase => (
                ctx.user_quote,
                ctx.user_base,
                meta.quote_vault.clone(),
                meta.base_vault.clone(),
                meta.quote_mint_meta.clone(),
                meta.base_mint_meta.clone(),
            ),
        };

        output.push(AccountMeta::new_readonly(RAYDIUM_CLMM_PROGRAM_ID, false));
        output.push(AccountMeta::new_readonly(ctx.payer, true));
        output.push(meta.amm_config.clone());
        output.push(meta.pool_account.clone());
        output.push(AccountMeta::new(source_user, false));
        output.push(AccountMeta::new(destination_user, false));
        output.push(source_vault);
        output.push(destination_vault);
        output.push(meta.observation_account.clone());
        output.push(meta.token_program.clone());
        if use_v2 {
            output.push(meta.token_program_2022.clone());
            output.push(meta.memo_program.clone());
            output.push(source_mint);
            output.push(destination_mint);
        }
        output.extend(meta.tick_arrays.iter().cloned());
    }
}
