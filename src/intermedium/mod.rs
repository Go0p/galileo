pub mod loader;

use std::collections::HashSet;
use std::str::FromStr;

use solana_sdk::pubkey::Pubkey;
use tracing::warn;

use crate::config::IntermediumConfig;
use crate::dexes::framework::PoolMeta;

#[allow(dead_code)]
fn parse_pubkeys(items: &[String], label: &str) -> HashSet<Pubkey> {
    let mut result = HashSet::with_capacity(items.len());
    for value in items {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        match Pubkey::from_str(trimmed) {
            Ok(pk) => {
                result.insert(pk);
            }
            Err(err) => warn!(
                target: "intermedium::filter",
                "{label} 条目解析失败: {trimmed}, 错误: {err}"
            ),
        }
    }
    result
}

/// 依据 intermedium 配置筛选池子，仅保留双方资产均在白名单中的条目。
#[allow(dead_code)]
pub fn filter_pools(
    cfg: &IntermediumConfig,
    pools: impl IntoIterator<Item = PoolMeta>,
) -> Vec<PoolMeta> {
    let mut allow = parse_pubkeys(&cfg.mints, "mints");
    if allow.is_empty() {
        warn!(
            target: "intermedium::filter",
            "intermedium.mints 为空，池子过滤将返回空列表"
        );
        return Vec::new();
    }

    let disabled = parse_pubkeys(&cfg.disable_mints, "disable_mints");
    allow.retain(|mint| !disabled.contains(mint));

    pools
        .into_iter()
        .filter(|pool| allow.contains(&pool.base_mint) && allow.contains(&pool.quote_mint))
        .collect()
}
