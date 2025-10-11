pub mod client;
pub mod error;
pub mod manager;
pub mod process;
pub mod types;
pub mod updater;

pub use error::JupiterError;
pub use types::{BinaryInstall, BinaryStatus, JupiterBinaryManager, ReleaseAsset, ReleaseInfo};
