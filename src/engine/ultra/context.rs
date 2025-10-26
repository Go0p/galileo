use std::sync::Arc;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

use crate::multi_leg::alt_cache::AltCache;

/// Ultra 交易预处理所需的运行环境。
#[derive(Clone)]
pub struct UltraContext {
    pub expected_signer: Pubkey,
    pub lookup_resolver: UltraLookupResolver,
}

impl UltraContext {
    pub fn new(expected_signer: Pubkey, lookup_resolver: UltraLookupResolver) -> Self {
        Self {
            expected_signer,
            lookup_resolver,
        }
    }
}

/// 标识不同的地址查找表解析策略。
#[derive(Clone)]
pub enum UltraLookupResolver {
    /// 立即通过 RPC + AltCache 解析查找表。
    Fetch { rpc: Arc<RpcClient>, alt_cache: AltCache },
    /// 延迟解析：仅返回查找表地址，由调用方稍后处理。
    Deferred,
}
