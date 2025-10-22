use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;

use crate::dexes::framework::{
    DexMarketMeta, DexMetaProvider, SwapAccountAssembler, SwapAccountsContext,
};

use super::decoder::{ORCA_WHIRLPOOL_PROGRAM_ID, WhirlpoolMarketMeta, fetch_market_meta};

#[derive(Default)]
pub struct WhirlpoolAdapter;

impl WhirlpoolAdapter {
    pub fn shared() -> &'static Self {
        &ADAPTER
    }
}

static ADAPTER: WhirlpoolAdapter = WhirlpoolAdapter;

impl DexMarketMeta for WhirlpoolMarketMeta {
    fn base_mint(&self) -> Pubkey {
        self.token_a_mint
    }

    fn quote_mint(&self) -> Pubkey {
        self.token_b_mint
    }

    fn base_token_program(&self) -> Pubkey {
        self.token_a_program
    }

    fn quote_token_program(&self) -> Pubkey {
        self.token_b_program
    }
}

impl DexMarketMeta for Arc<WhirlpoolMarketMeta> {
    fn base_mint(&self) -> Pubkey {
        self.token_a_mint
    }

    fn quote_mint(&self) -> Pubkey {
        self.token_b_mint
    }

    fn base_token_program(&self) -> Pubkey {
        self.token_a_program
    }

    fn quote_token_program(&self) -> Pubkey {
        self.token_b_program
    }
}

impl DexMetaProvider for WhirlpoolAdapter {
    type MarketMeta = WhirlpoolMarketMeta;

    type FetchFuture<'a>
        = Pin<Box<dyn Future<Output = Result<Arc<Self::MarketMeta>>> + Send + 'a>>
    where
        Self: 'a;

    fn program_id(&self) -> Pubkey {
        ORCA_WHIRLPOOL_PROGRAM_ID
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

impl SwapAccountAssembler for WhirlpoolAdapter {
    type MarketMeta = WhirlpoolMarketMeta;

    fn assemble_remaining_accounts(
        &self,
        meta: &Self::MarketMeta,
        ctx: SwapAccountsContext,
        output: &mut Vec<AccountMeta>,
    ) {
        debug_assert_eq!(meta.pool_account.pubkey, ctx.market);

        let user_token_a = AccountMeta::new(ctx.user_base, false);
        let user_token_b = AccountMeta::new(ctx.user_quote, false);

        output.push(AccountMeta::new_readonly(ORCA_WHIRLPOOL_PROGRAM_ID, false));
        output.push(meta.token_program.clone());
        output.push(AccountMeta::new_readonly(ctx.payer, true));
        output.push(meta.pool_account.clone());
        output.push(user_token_a);
        output.push(meta.vault_a.clone());
        output.push(user_token_b);
        output.push(meta.vault_b.clone());
        output.extend(meta.tick_arrays.iter().cloned());
        output.push(meta.oracle.clone());
    }
}
