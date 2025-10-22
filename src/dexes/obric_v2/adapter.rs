use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use spl_token::solana_program::program_pack::Pack;
use spl_token::state::Account as SplTokenAccount;

use crate::dexes::framework::{
    DexMarketMeta, DexMetaProvider, SwapAccountAssembler, SwapAccountsContext, SwapFlow,
};

use super::decoder::{OBRIC_V2_PROGRAM_ID, ObricV2MarketMeta, decode_trading_pair_accounts};

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
            let layout = decode_trading_pair_accounts(&account.data)?;
            let reserve_keys = [layout.reserve_x, layout.reserve_y];
            let reserve_accounts = client
                .get_multiple_accounts(&reserve_keys)
                .await
                .with_context(|| {
                    format!(
                        "读取 Obric reserves {{ {}, {} }} 失败",
                        layout.reserve_x, layout.reserve_y
                    )
                })?;

            let reserve_x_account = reserve_accounts
                .get(0)
                .and_then(|acc| acc.as_ref())
                .ok_or_else(|| anyhow!("reserve_x {} 不存在", layout.reserve_x))?;
            let reserve_y_account = reserve_accounts
                .get(1)
                .and_then(|acc| acc.as_ref())
                .ok_or_else(|| anyhow!("reserve_y {} 不存在", layout.reserve_y))?;

            let reserve_x_state = SplTokenAccount::unpack_from_slice(&reserve_x_account.data)
                .map_err(|err| anyhow!("解析 reserve_x {} 失败: {err}", layout.reserve_x))?;
            let reserve_y_state = SplTokenAccount::unpack_from_slice(&reserve_y_account.data)
                .map_err(|err| anyhow!("解析 reserve_y {} 失败: {err}", layout.reserve_y))?;

            let token_program = Pubkey::new_from_array(spl_token::id().to_bytes());
            let base_mint = Pubkey::new_from_array(reserve_x_state.mint.to_bytes());
            let quote_mint = Pubkey::new_from_array(reserve_y_state.mint.to_bytes());

            let meta = ObricV2MarketMeta {
                trading_pair: AccountMeta::new(market, false),
                second_reference_oracle: AccountMeta::new_readonly(
                    layout.second_reference_oracle,
                    false,
                ),
                third_reference_oracle: AccountMeta::new_readonly(
                    layout.third_reference_oracle,
                    false,
                ),
                reserve_x: AccountMeta::new(layout.reserve_x, false),
                reserve_y: AccountMeta::new(layout.reserve_y, false),
                reference_oracle: AccountMeta::new(layout.reference_oracle, false),
                x_price_feed: AccountMeta::new_readonly(layout.x_price_feed, false),
                y_price_feed: AccountMeta::new_readonly(layout.y_price_feed, false),
                token_program: AccountMeta::new_readonly(token_program, false),
                base_mint,
                quote_mint,
                base_token_program: token_program,
                quote_token_program: token_program,
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
        output.push(meta.second_reference_oracle.clone());
        output.push(meta.third_reference_oracle.clone());
        output.push(meta.reserve_x.clone());
        output.push(meta.reserve_y.clone());
        output.push(meta.reference_oracle.clone());
        output.push(meta.x_price_feed.clone());
        output.push(meta.y_price_feed.clone());
        output.push(meta.token_program.clone());
    }
}
