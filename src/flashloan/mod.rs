mod error;
pub mod marginfi;

use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;

use crate::engine::EngineError;

pub use error::{FlashloanError, FlashloanResult};

#[derive(Debug, Clone)]
pub struct FlashloanOutcome {
    pub instructions: Vec<Instruction>,
    pub metadata: Option<FlashloanMetadata>,
}

#[derive(Debug, Clone)]
pub struct FlashloanMetadata {
    pub protocol: FlashloanProtocol,
    pub mint: Pubkey,
    pub borrow_amount: u64,
    pub inner_instruction_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlashloanProtocol {
    Marginfi,
}

impl FlashloanProtocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlashloanProtocol::Marginfi => "marginfi",
        }
    }
}

impl From<FlashloanError> for EngineError {
    fn from(value: FlashloanError) -> Self {
        match value {
            FlashloanError::InvalidConfig(msg) => EngineError::InvalidConfig(msg.to_string()),
            FlashloanError::InvalidConfigDetail(msg) => EngineError::InvalidConfig(msg),
            FlashloanError::UnsupportedAsset(msg) => EngineError::InvalidConfig(msg),
            FlashloanError::Rpc(err) => EngineError::Rpc(err),
        }
    }
}
