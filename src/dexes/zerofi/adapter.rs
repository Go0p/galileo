use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey::Pubkey;
use tracing::debug;

use crate::dexes::framework::{
    DexMarketMeta, DexMetaProvider, SwapAccountAssembler, SwapAccountsContext, SwapFlow,
};

use super::decoder::{ZEROFI_PROGRAM_ID, ZeroFiMarketMeta, decode_market_meta};

#[derive(Default)]
pub struct ZeroFiAdapter;

impl ZeroFiAdapter {
    pub fn shared() -> &'static Self {
        &ADAPTER
    }
}

static ADAPTER: ZeroFiAdapter = ZeroFiAdapter;

impl DexMarketMeta for ZeroFiMarketMeta {
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

impl DexMetaProvider for ZeroFiAdapter {
    type MarketMeta = ZeroFiMarketMeta;

    type FetchFuture<'a>
        = Pin<Box<dyn Future<Output = Result<Arc<Self::MarketMeta>>> + Send + 'a>>
    where
        Self: 'a;

    fn program_id(&self) -> Pubkey {
        ZEROFI_PROGRAM_ID
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

impl SwapAccountAssembler for ZeroFiAdapter {
    type MarketMeta = ZeroFiMarketMeta;

    fn assemble_remaining_accounts(
        &self,
        meta: &Self::MarketMeta,
        ctx: SwapAccountsContext,
        output: &mut Vec<AccountMeta>,
    ) {
        let (vault_info_in, vault_in, vault_info_out, vault_out, user_source, user_destination) =
            match ctx.flow {
                SwapFlow::QuoteToBase => (
                    meta.vault_info_quote.clone(),
                    meta.vault_quote.clone(),
                    meta.vault_info_base.clone(),
                    meta.vault_base.clone(),
                    ctx.user_quote,
                    ctx.user_base,
                ),
                SwapFlow::BaseToQuote => (
                    meta.vault_info_base.clone(),
                    meta.vault_base.clone(),
                    meta.vault_info_quote.clone(),
                    meta.vault_quote.clone(),
                    ctx.user_base,
                    ctx.user_quote,
                ),
            };

        if meta.swap_authority_hint() != ctx.payer {
            debug!(
                target: "dex::zerofi",
                market = %meta.pair_account.pubkey,
                hint = %meta.swap_authority_hint(),
                payer = %ctx.payer,
                "ZeroFi swap authority 与账户内记载不一致，将使用 payer 作为签名者"
            );
        }

        output.push(AccountMeta::new_readonly(ZEROFI_PROGRAM_ID, false));
        output.extend_from_slice(&[
            meta.pair_account.clone(),
            vault_info_in,
            vault_in,
            vault_info_out,
            vault_out,
            AccountMeta::new(user_source, false),
            AccountMeta::new(user_destination, false),
            AccountMeta::new_readonly(ctx.payer, true),
            meta.token_program.clone(),
            meta.sysvar_instructions.clone(),
        ]);
    }
}
