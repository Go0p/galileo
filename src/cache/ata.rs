use dashmap::DashMap;
use once_cell::sync::Lazy;
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AtaKey {
    owner: Pubkey,
    mint: Pubkey,
    token_program: Pubkey,
}

static ATA_CACHE: Lazy<DashMap<AtaKey, Pubkey>> = Lazy::new(DashMap::new);

/// 返回缓存的 ATA 地址，未命中时计算并写入缓存。
pub fn cached_associated_token_address(
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Pubkey {
    let key = AtaKey {
        owner: *owner,
        mint: *mint,
        token_program: *token_program,
    };
    if let Some(entry) = ATA_CACHE.get(&key) {
        return *entry;
    }
    let ata_program = spl_associated_token_account::id();
    let address = Pubkey::find_program_address(
        &[owner.as_ref(), token_program.as_ref(), mint.as_ref()],
        &ata_program,
    )
    .0;
    ATA_CACHE.insert(key, address);
    address
}
