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

use super::decoder::{
    OBRIC_V2_PROGRAM_ID, ObricSwapOrder, ObricV2MarketMeta, decode_trading_pair_accounts,
};

#[derive(Default)]
pub struct ObricV2Adapter;

impl ObricV2Adapter {
    pub fn shared() -> &'static Self {
        &ADAPTER
    }
}

static ADAPTER: ObricV2Adapter = ObricV2Adapter;

impl DexMarketMeta for ObricV2MarketMeta {
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

impl DexMetaProvider for ObricV2Adapter {
    type MarketMeta = ObricV2MarketMeta;

    type FetchFuture<'a>
        = Pin<Box<dyn Future<Output = Result<Arc<Self::MarketMeta>>> + Send + 'a>>
    where
        Self: 'a;

    fn program_id(&self) -> Pubkey {
        OBRIC_V2_PROGRAM_ID
    }

    fn fetch_market_meta<'a>(
        &'a self,
        client: &'a RpcClient,
        market: Pubkey,
        account: &'a Account,
    ) -> Self::FetchFuture<'a> {
        Box::pin(async move {
            let layout = decode_trading_pair_accounts(client, market, &account.data).await?;

            let meta = ObricV2MarketMeta {
                order: layout.order,
                trading_pair: AccountMeta::new(market, false),
                second_reference_oracle: layout
                    .second_reference_oracle
                    .map(|pk| AccountMeta::new_readonly(pk, false)),
                third_reference_oracle: layout
                    .third_reference_oracle
                    .map(|pk| AccountMeta::new_readonly(pk, false)),
                reserve_x: AccountMeta::new(layout.reserve_x, false),
                reserve_y: AccountMeta::new(layout.reserve_y, false),
                reference_oracle: AccountMeta::new(layout.reference_oracle, false),
                x_price_feed: AccountMeta::new_readonly(layout.x_price_feed, false),
                y_price_feed: AccountMeta::new_readonly(layout.y_price_feed, false),
                mint_x_pool: layout
                    .mint_x_pool
                    .map(|pk| AccountMeta::new_readonly(pk, false)),
                mint_y_pool: layout
                    .mint_y_pool
                    .map(|pk| AccountMeta::new_readonly(pk, false)),
                sysvar_instructions: layout
                    .sysvar_instructions
                    .map(|pk| AccountMeta::new_readonly(pk, false)),
                swap_authority: AccountMeta::new(layout.swap_authority, false),
                token_program: AccountMeta::new_readonly(layout.token_program, false),
                base_mint: layout.base_mint,
                quote_mint: layout.quote_mint,
                base_token_program: layout.token_program,
                quote_token_program: layout.token_program,
            };

            Ok(Arc::new(meta))
        })
    }
}

impl SwapAccountAssembler for ObricV2Adapter {
    type MarketMeta = ObricV2MarketMeta;

    fn assemble_remaining_accounts(
        &self,
        meta: &Self::MarketMeta,
        ctx: SwapAccountsContext,
        output: &mut Vec<AccountMeta>,
    ) {
        debug_assert_eq!(meta.trading_pair.pubkey, ctx.market);

        let x_to_y = matches!(ctx.flow, SwapFlow::BaseToQuote);
        let (source_token, destination_token) = if x_to_y {
            (ctx.user_base, ctx.user_quote)
        } else {
            (ctx.user_quote, ctx.user_base)
        };

        output.push(AccountMeta::new_readonly(OBRIC_V2_PROGRAM_ID, false));
        output.push(AccountMeta::new_readonly(ctx.payer, true));
        output.push(AccountMeta::new(source_token, false));
        output.push(AccountMeta::new(destination_token, false));
        output.push(meta.trading_pair.clone());

        match meta.order {
            ObricSwapOrder::Swap => {
                let second = meta
                    .second_reference_oracle
                    .as_ref()
                    .expect("swap missing second oracle")
                    .clone();
                let third = meta
                    .third_reference_oracle
                    .as_ref()
                    .expect("swap missing third oracle")
                    .clone();

                output.push(second);
                output.push(third);
                output.push(meta.reserve_x.clone());
                output.push(meta.reserve_y.clone());
                output.push(meta.reference_oracle.clone());
                output.push(meta.x_price_feed.clone());
                output.push(meta.y_price_feed.clone());
                output.push(meta.token_program.clone());
            }
            ObricSwapOrder::Swap2 => {
                let mint_x_pool = meta
                    .mint_x_pool
                    .as_ref()
                    .expect("swap2 missing mint_x_pool account")
                    .clone();
                let mint_y_pool = meta
                    .mint_y_pool
                    .as_ref()
                    .expect("swap2 missing mint_y_pool account")
                    .clone();
                let sysvar = meta
                    .sysvar_instructions
                    .as_ref()
                    .expect("swap2 missing sysvar instructions")
                    .clone();

                output.push(mint_x_pool);
                output.push(mint_y_pool);
                output.push(meta.reserve_x.clone());
                output.push(meta.reserve_y.clone());
                output.push(meta.reference_oracle.clone());
                output.push(meta.x_price_feed.clone());
                output.push(sysvar);
                output.push(meta.swap_authority.clone());
                output.push(meta.token_program.clone());
            }
        }
    }
}
