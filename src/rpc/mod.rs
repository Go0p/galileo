pub mod yellowstone;

use solana_sdk::hash::Hash;

/// 表示一个可供落地使用的区块哈希快照。
#[derive(Clone, Debug)]
pub struct BlockhashSnapshot {
    pub blockhash: Hash,
    pub slot: Option<u64>,
    pub last_valid_block_height: Option<u64>,
}
