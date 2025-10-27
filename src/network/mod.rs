mod allocator;
mod client;
mod error;
mod inventory;
mod pool;
mod slot;

use std::sync::Arc;

#[allow(unused_imports)]
pub use allocator::{
    CooldownConfig, IpAllocator, IpLease, IpLeaseHandle, IpLeaseMode, IpLeaseOutcome, IpTaskKind,
};
pub use client::HttpClientFactory;
pub use error::{NetworkError, NetworkResult};
pub use inventory::{IpInventory, IpInventoryConfig, IpSource};
#[allow(unused_imports)]
pub use pool::IpBoundClientPool;
pub use slot::{IpSlot, IpSlotKind, IpSlotState};

pub type ReqwestClientFactoryFn =
    Box<dyn Fn(std::net::IpAddr) -> NetworkResult<reqwest::Client> + Send + Sync>;
pub type RpcClientFactoryFn = Box<
    dyn Fn(
            std::net::IpAddr,
        ) -> NetworkResult<Arc<solana_client::nonblocking::rpc_client::RpcClient>>
        + Send
        + Sync,
>;
