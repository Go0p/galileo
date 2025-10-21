use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;

use crate::dexes::framework::{DexMarketMeta, DexMetaProvider, SwapAccountAssembler, SwapAccountsContext};

use super::decoder::{HUMIDIFI_PROGRAM_ID, HumidiFiMarketMeta, fetch_market_meta};

#[derive(Default)]
pub struct HumidiFiAdapter;

impl HumidiFiAdapter {
    #[allow(dead_code)]
    pub const fn new() -> Self {
        Self
    }

    pub fn shared() -> &'static Self {
        &ADAPTER
    }
}

static ADAPTER: HumidiFiAdapter = HumidiFiAdapter;

impl DexMarketMeta for HumidiFiMarketMeta {
    fn base_mint(&self) -> Pubkey {
        self.base_mint.pubkey
    }

    fn quote_mint(&self) -> Pubkey {
        self.quote_mint.pubkey
    }

    fn base_token_program(&self) -> Pubkey {
        self.token_program.pubkey
    }

    fn quote_token_program(&self) -> Pubkey {
        self.token_program.pubkey
    }
}

impl DexMarketMeta for Arc<HumidiFiMarketMeta> {
    fn base_mint(&self) -> Pubkey {
        self.base_mint.pubkey
    }

    fn quote_mint(&self) -> Pubkey {
        self.quote_mint.pubkey
    }

    fn base_token_program(&self) -> Pubkey {
        self.token_program.pubkey
    }

    fn quote_token_program(&self) -> Pubkey {
        self.token_program.pubkey
    }
}

impl DexMetaProvider for HumidiFiAdapter {
    type MarketMeta = HumidiFiMarketMeta;

    type FetchFuture<'a>
        = Pin<Box<dyn Future<Output = Result<Arc<Self::MarketMeta>>> + Send + 'a>>
    where
        Self: 'a;

    fn program_id(&self) -> Pubkey {
        HUMIDIFI_PROGRAM_ID
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

impl SwapAccountAssembler for HumidiFiAdapter {
    type MarketMeta = HumidiFiMarketMeta;

    fn assemble_remaining_accounts(
        &self,
        meta: &Self::MarketMeta,
        ctx: SwapAccountsContext,
        output: &mut Vec<AccountMeta>,
    ) {
        debug_assert_eq!(meta.pool_account.pubkey, ctx.market);

        output.push(AccountMeta::new_readonly(HUMIDIFI_PROGRAM_ID, false));
        output.extend_from_slice(&[
            AccountMeta::new(ctx.payer, true),
            meta.pool_account.clone(),
            meta.base_vault.clone(),
            meta.quote_vault.clone(),
            AccountMeta::new(ctx.user_base, false),
            AccountMeta::new(ctx.user_quote, false),
            meta.sysvar_clock.clone(),
            meta.token_program.clone(),
            meta.sysvar_instructions.clone(),
        ]);
    }
}
