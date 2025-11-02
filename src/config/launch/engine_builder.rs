//! Engine 构建相关逻辑将逐步迁移至此。

use anyhow::Result;

use crate::cache::AltCache;
use crate::config::AppConfig;
use crate::engine::{BuilderConfig, TransactionBuilder};

use super::resources::{build_ip_allocator, build_rpc_resources};
use crate::cli::context::resolve_instruction_memo;

/// TODO: 完整迁移 CLI 中的 TransactionBuilder 装配逻辑。
#[allow(dead_code)]
pub fn build_transaction_builder(config: &AppConfig) -> Result<TransactionBuilder> {
    let (rpc_client, _) = build_rpc_resources(config)?;
    let builder_config =
        BuilderConfig::new(resolve_instruction_memo(&config.galileo.global.instruction));
    let ip_allocator = build_ip_allocator(&config.galileo.bot.network)?;

    Ok(TransactionBuilder::new(
        rpc_client,
        builder_config,
        ip_allocator,
        None,
        AltCache::new(),
    ))
}
