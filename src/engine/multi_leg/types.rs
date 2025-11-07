use std::fmt;

use serde::{Deserialize, Serialize};
use solana_sdk::{
    hash::Hash, instruction::Instruction, message::AddressLookupTableAccount, pubkey::Pubkey,
    transaction::VersionedTransaction,
};

use crate::config::types::LegRole;

/// 标记在套利流程中承担的腿方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LegSide {
    Buy,
    Sell,
}

impl From<LegRole> for LegSide {
    fn from(role: LegRole) -> Self {
        match role {
            LegRole::Buy => LegSide::Buy,
            LegRole::Sell => LegSide::Sell,
        }
    }
}

impl fmt::Display for LegSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LegSide::Buy => f.write_str("buy"),
            LegSide::Sell => f.write_str("sell"),
        }
    }
}

/// 支持的聚合器类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AggregatorKind {
    Jupiter,
    Ultra,
    Dflow,
    Titan,
    Kamino,
}

impl fmt::Display for AggregatorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AggregatorKind::Ultra => f.write_str("ultra"),
            AggregatorKind::Jupiter => f.write_str("jupiter"),
            AggregatorKind::Dflow => f.write_str("dflow"),
            AggregatorKind::Titan => f.write_str("titan"),
            AggregatorKind::Kamino => f.write_str("kamino"),
        }
    }
}

/// 腿提供方的基础描述信息。
#[derive(Debug, Clone)]
pub struct LegDescriptor {
    pub kind: AggregatorKind,
    pub side: LegSide,
}

impl LegDescriptor {
    pub fn new(kind: AggregatorKind, side: LegSide) -> Self {
        Self { kind, side }
    }
}

/// 通用的报价意图，描述一条腿的输入/输出与滑点要求。
#[derive(Debug, Clone)]
pub struct QuoteIntent {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub amount: u64,
    pub slippage_bps: u16,
    pub dex_whitelist: Vec<String>,
    pub dex_blacklist: Vec<String>,
}

impl QuoteIntent {
    pub fn new(input_mint: Pubkey, output_mint: Pubkey, amount: u64, slippage_bps: u16) -> Self {
        Self {
            input_mint,
            output_mint,
            amount,
            slippage_bps,
            dex_whitelist: Vec::new(),
            dex_blacklist: Vec::new(),
        }
    }
}

/// 报价阶段的核心信息，用于收益评估与风控。
#[derive(Debug, Clone)]
pub struct LegQuote {
    pub amount_in: u64,
    pub amount_out: u64,
    pub min_out_amount: Option<u64>,
    pub slippage_bps: u16,
    pub latency_ms: Option<f64>,
    pub request_id: Option<String>,
    pub quote_id: Option<String>,
    pub provider: Option<String>,
    pub context_slot: Option<u64>,
    pub expires_at_ms: Option<u64>,
    pub expires_after_slot: Option<u64>,
}

impl LegQuote {
    pub fn new(amount_in: u64, amount_out: u64, slippage_bps: u16) -> Self {
        Self {
            amount_in,
            amount_out,
            min_out_amount: None,
            slippage_bps,
            latency_ms: None,
            request_id: None,
            quote_id: None,
            provider: None,
            context_slot: None,
            expires_at_ms: None,
            expires_after_slot: None,
        }
    }
}

/// 构建腿执行计划时需要的上下文。
#[derive(Debug, Clone, Default)]
pub struct LegBuildContext {
    pub payer: Pubkey,
    pub compute_unit_price_micro_lamports: Option<u64>,
    pub fee_account: Option<Pubkey>,
    pub sponsor: Option<Pubkey>,
    pub wrap_and_unwrap_sol: Option<bool>,
    pub dynamic_compute_unit_limit: Option<bool>,
    pub compute_unit_limit_multiplier: Option<f64>,
}

/// 腿执行计划，包含组合指令和可能的附加元数据。
#[derive(Debug, Clone)]
pub struct LegPlan {
    pub descriptor: LegDescriptor,
    pub quote: LegQuote,
    pub instructions: Vec<Instruction>,
    pub compute_budget_instructions: Vec<Instruction>,
    pub address_lookup_table_addresses: Vec<Pubkey>,
    pub resolved_lookup_tables: Vec<AddressLookupTableAccount>,
    pub prioritization_fee_lamports: Option<u64>,
    pub blockhash: Option<Hash>,
    pub raw_transaction: Option<VersionedTransaction>,
    pub signer_rewrite: Option<SignerRewrite>,
    pub account_rewrites: Vec<(Pubkey, Pubkey)>,
    pub requested_compute_unit_limit: Option<u32>,
    pub requested_compute_unit_price_micro_lamports: Option<u64>,
    pub requested_tip_lamports: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub struct SignerRewrite {
    pub original: Pubkey,
    pub replacement: Pubkey,
}
