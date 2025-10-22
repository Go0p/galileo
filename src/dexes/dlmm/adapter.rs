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

use super::decoder::{METEORA_DLMM_PROGRAM_ID, MeteoraDlmmMarketMeta, fetch_market_meta};

#[derive(Default)]
pub struct MeteoraDlmmAdapter;

impl MeteoraDlmmAdapter {
    pub fn shared() -> &'static Self {
        &ADAPTER
    }
}

static ADAPTER: MeteoraDlmmAdapter = MeteoraDlmmAdapter;

impl DexMarketMeta for MeteoraDlmmMarketMeta {
    fn base_mint(&self) -> Pubkey {
        self.token_x_mint
    }

    fn quote_mint(&self) -> Pubkey {
        self.token_y_mint
    }

    fn base_token_program(&self) -> Pubkey {
        self.token_x_program
    }

    fn quote_token_program(&self) -> Pubkey {
        self.token_y_program
    }
}

impl DexMarketMeta for Arc<MeteoraDlmmMarketMeta> {
    fn base_mint(&self) -> Pubkey {
        self.token_x_mint
    }

    fn quote_mint(&self) -> Pubkey {
        self.token_y_mint
    }

    fn base_token_program(&self) -> Pubkey {
        self.token_x_program
    }

    fn quote_token_program(&self) -> Pubkey {
        self.token_y_program
    }
}

impl DexMetaProvider for MeteoraDlmmAdapter {
    type MarketMeta = MeteoraDlmmMarketMeta;

    type FetchFuture<'a>
        = Pin<Box<dyn Future<Output = Result<Arc<Self::MarketMeta>>> + Send + 'a>>
    where
        Self: 'a;

    fn program_id(&self) -> Pubkey {
        METEORA_DLMM_PROGRAM_ID
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

impl SwapAccountAssembler for MeteoraDlmmAdapter {
    type MarketMeta = MeteoraDlmmMarketMeta;

    fn assemble_remaining_accounts(
        &self,
        meta: &Self::MarketMeta,
        ctx: SwapAccountsContext,
        output: &mut Vec<AccountMeta>,
    ) {
        debug_assert_eq!(meta.lb_pair.pubkey, ctx.market);

        let user_token_in = match ctx.flow {
            SwapFlow::BaseToQuote => ctx.user_base,
            SwapFlow::QuoteToBase => ctx.user_quote,
        };
        let user_token_out = match ctx.flow {
            SwapFlow::BaseToQuote => ctx.user_quote,
            SwapFlow::QuoteToBase => ctx.user_base,
        };

        let bitmap_extension = meta
            .bin_array_bitmap_extension
            .clone()
            .unwrap_or_else(|| AccountMeta::new_readonly(METEORA_DLMM_PROGRAM_ID, false));
        let host_fee_placeholder = AccountMeta::new_readonly(METEORA_DLMM_PROGRAM_ID, false);
        let use_v2 = meta.uses_token_2022();

        output.push(meta.lb_pair.clone());
        output.push(bitmap_extension);
        output.push(meta.reserve_x.clone());
        output.push(meta.reserve_y.clone());
        output.push(AccountMeta::new(user_token_in, false));
        output.push(AccountMeta::new(user_token_out, false));
        output.push(meta.token_x_mint_meta.clone());
        output.push(meta.token_y_mint_meta.clone());
        output.push(meta.oracle.clone());
        output.push(host_fee_placeholder);
        output.push(AccountMeta::new_readonly(ctx.payer, true));
        output.push(meta.token_x_program_meta.clone());
        output.push(meta.token_y_program_meta.clone());
        if use_v2 {
            output.push(meta.memo_program.clone());
        }
        output.push(meta.event_authority.clone());
        output.push(AccountMeta::new_readonly(METEORA_DLMM_PROGRAM_ID, false));
        output.extend(meta.bin_arrays.iter().cloned());
    }
}
