use std::future::Future;
use std::sync::Arc;

use anyhow::Result;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::account::Account;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

/// 用户在构建 swap 指令时持有的关键上下文。
#[derive(Debug, Clone, Copy)]
pub struct SwapAccountsContext {
    pub market: Pubkey,
    pub payer: Pubkey,
    pub user_base: Pubkey,
    pub user_quote: Pubkey,
    pub flow: SwapFlow,
}

#[derive(Debug, Clone, Copy)]
pub enum SwapFlow {
    QuoteToBase,
    BaseToQuote,
}

/// DEX 市场元数据最小接口。
pub trait DexMarketMeta: Send + Sync {
    fn base_mint(&self) -> Pubkey;
    fn quote_mint(&self) -> Pubkey;
    fn base_token_program(&self) -> Pubkey;
    fn quote_token_program(&self) -> Pubkey;
}

/// 拉取并解析市场元数据的提供者。
pub trait DexMetaProvider: Send + Sync {
    type MarketMeta: DexMarketMeta + 'static;

    /// 关联的异步 Future 类型，用于零成本地返回不同实现的拉取逻辑。
    type FetchFuture<'a>: Future<Output = Result<Arc<Self::MarketMeta>>> + Send + 'a
    where
        Self: 'a;

    #[allow(dead_code)]
    fn program_id(&self) -> Pubkey;

    fn fetch_market_meta<'a>(
        &'a self,
        client: &'a RpcClient,
        market: Pubkey,
        account: &'a Account,
    ) -> Self::FetchFuture<'a>;
}

/// 负责根据市场元数据与用户上下文生成 remaining accounts。
#[allow(dead_code)]
pub trait SwapAccountAssembler: Send + Sync {
    type MarketMeta: DexMarketMeta + 'static;

    fn assemble_remaining_accounts(
        &self,
        meta: &Self::MarketMeta,
        ctx: SwapAccountsContext,
        output: &mut Vec<AccountMeta>,
    );
}

/// trait object 友好的助手类型。
#[allow(dead_code)]
pub type MetaArc<T> = Arc<T>;
