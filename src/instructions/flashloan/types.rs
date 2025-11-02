use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;

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
