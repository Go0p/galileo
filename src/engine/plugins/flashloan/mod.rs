pub mod marginfi;

pub use crate::instructions::flashloan::{
    FlashloanError, FlashloanMetadata, FlashloanOutcome, FlashloanResult,
};
pub use marginfi::{
    MarginfiAccountRegistry, MarginfiFlashloanManager, MarginfiFlashloanPreparation,
    find_marginfi_account_by_authority, marginfi_account_matches_authority,
};
