//! 轻量级 Jupiter Route 指令解析工具，用于 copy 策略。
//! 数据来源于 `/home/go0p/code/rust/yellowstone-vixen/crates/jupiter-swap-parser`。

use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

/// Jupiter v6 程序 ID。
pub const PROGRAM_ID: Pubkey = solana_sdk::pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");

/// 指令 discriminator 常量。
pub mod discriminator {
    pub const ROUTE: [u8; 8] = [229, 23, 203, 151, 122, 227, 173, 42];
    pub const ROUTE_V2: [u8; 8] = [187, 100, 250, 204, 49, 196, 175, 20];
    pub const SHARED_ROUTE_V2: [u8; 8] = [102, 118, 210, 18, 29, 110, 15, 147];
    pub const EXACT_ROUTE_V2: [u8; 8] = [157, 138, 184, 82, 21, 244, 243, 36];
}

/// Route 指令类别。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteKind {
    Route,
    RouteV2,
    SharedRouteV2,
    ExactRouteV2,
    Other,
}

/// 根据指令数据判定路由类别。
pub fn classify(data: &[u8]) -> RouteKind {
    if data.len() < 8 {
        return RouteKind::Other;
    }
    let disc = &data[..8];
    if disc == discriminator::ROUTE {
        RouteKind::Route
    } else if disc == discriminator::ROUTE_V2 {
        RouteKind::RouteV2
    } else if disc == discriminator::SHARED_ROUTE_V2 {
        RouteKind::SharedRouteV2
    } else if disc == discriminator::EXACT_ROUTE_V2 {
        RouteKind::ExactRouteV2
    } else {
        RouteKind::Other
    }
}

/// Route v2 系列指令的账户布局。
#[derive(Clone, Debug)]
pub struct RouteV2Accounts {
    pub user_transfer_authority: Pubkey,
    pub user_source_token_account: Pubkey,
    pub user_destination_token_account: Pubkey,
    pub source_mint: Pubkey,
    pub destination_mint: Pubkey,
    pub source_token_program: Pubkey,
    pub destination_token_program: Pubkey,
}

impl RouteV2Accounts {
    /// 按固定顺序读取账户。
    pub fn parse(ix: &Instruction) -> Option<Self> {
        if ix.program_id != PROGRAM_ID {
            return None;
        }
        match classify(&ix.data) {
            RouteKind::RouteV2 | RouteKind::SharedRouteV2 | RouteKind::ExactRouteV2 => {}
            _ => return None,
        }
        if ix.accounts.len() < 10 {
            return None;
        }
        let accounts = &ix.accounts;
        Some(Self {
            user_transfer_authority: accounts[0].pubkey,
            user_source_token_account: accounts[1].pubkey,
            user_destination_token_account: accounts[2].pubkey,
            source_mint: accounts[3].pubkey,
            destination_mint: accounts[4].pubkey,
            source_token_program: accounts[5].pubkey,
            destination_token_program: accounts[6].pubkey,
        })
    }
}
