//! DFlow 聚合器 API 定义，与 Jupiter 模块保持相同的分层结构，方便后续实现客户端逻辑。

pub mod quote;
pub mod serde_helpers;
pub mod swap_instructions;

#[allow(unused_imports)]
pub use quote::{
    PlatformFee, PlatformFeeMode, QuoteRequest, QuoteResponse, QuoteResponsePayload, RoutePlanLeg,
    RoutePlanLegWithData, RoutePlanStep, SlippageBps, SlippagePreset,
};
#[allow(unused_imports)]
pub use swap_instructions::{
    BlockhashWithMetadata, ComputeUnitPriceMicroLamports, CreateFeeAccount,
    DestinationAssociatedTokenAccount, DestinationTokenAccount, DestinationTokenAccountViaOwner,
    PositiveSlippageConfig, PrioritizationFeeLamports, PrioritizationFeeLamportsConfig,
    PrioritizationFeePreset, PrioritizationType, PriorityLevel, PriorityLevelWithMaxLamports,
    SwapInstructionsRequest, SwapInstructionsResponse,
};
