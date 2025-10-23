use rust_decimal::Decimal;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;

use crate::api::dflow;
use crate::api::jupiter;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregatorKind {
    Jupiter,
    Dflow,
}

#[derive(Debug, Clone)]
pub enum QuoteResponseVariant {
    Jupiter(jupiter::QuoteResponse),
    Dflow(dflow::QuoteResponse),
}

impl QuoteResponseVariant {
    pub fn kind(&self) -> AggregatorKind {
        match self {
            QuoteResponseVariant::Jupiter(_) => AggregatorKind::Jupiter,
            QuoteResponseVariant::Dflow(_) => AggregatorKind::Dflow,
        }
    }

    pub fn input_mint(&self) -> Pubkey {
        match self {
            QuoteResponseVariant::Jupiter(resp) => resp.input_mint,
            QuoteResponseVariant::Dflow(resp) => resp.payload().input_mint,
        }
    }

    pub fn output_mint(&self) -> Pubkey {
        match self {
            QuoteResponseVariant::Jupiter(resp) => resp.output_mint,
            QuoteResponseVariant::Dflow(resp) => resp.payload().output_mint,
        }
    }

    pub fn in_amount(&self) -> u64 {
        match self {
            QuoteResponseVariant::Jupiter(resp) => resp.in_amount,
            QuoteResponseVariant::Dflow(resp) => resp.payload().in_amount,
        }
    }

    pub fn out_amount(&self) -> u64 {
        match self {
            QuoteResponseVariant::Jupiter(resp) => resp.out_amount,
            QuoteResponseVariant::Dflow(resp) => resp.payload().out_amount,
        }
    }

    pub fn clone_payload(&self) -> QuotePayloadVariant {
        match self {
            QuoteResponseVariant::Jupiter(resp) => QuotePayloadVariant::Jupiter((**resp).clone()),
            QuoteResponseVariant::Dflow(resp) => QuotePayloadVariant::Dflow(resp.payload().clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum QuotePayloadVariant {
    Jupiter(jupiter::QuoteResponsePayload),
    Dflow(dflow::QuoteResponsePayload),
}

impl QuotePayloadVariant {
    pub fn kind(&self) -> AggregatorKind {
        match self {
            QuotePayloadVariant::Jupiter(_) => AggregatorKind::Jupiter,
            QuotePayloadVariant::Dflow(_) => AggregatorKind::Dflow,
        }
    }

    pub fn input_mint(&self) -> Pubkey {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.input_mint,
            QuotePayloadVariant::Dflow(payload) => payload.input_mint,
        }
    }

    pub fn output_mint(&self) -> Pubkey {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.output_mint,
            QuotePayloadVariant::Dflow(payload) => payload.output_mint,
        }
    }

    pub fn out_amount(&self) -> u64 {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.out_amount,
            QuotePayloadVariant::Dflow(payload) => payload.out_amount,
        }
    }

    pub fn other_amount_threshold(&self) -> u64 {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.other_amount_threshold,
            QuotePayloadVariant::Dflow(payload) => payload.other_amount_threshold,
        }
    }

    pub fn set_out_amount(&mut self, value: u64) {
        match self {
            QuotePayloadVariant::Jupiter(payload) => {
                payload.out_amount = value;
                payload.other_amount_threshold = value;
            }
            QuotePayloadVariant::Dflow(payload) => {
                payload.out_amount = value;
                payload.other_amount_threshold = value;
                payload.min_out_amount = value;
            }
        }
    }

    pub fn set_context_slot(&mut self, slot: u64) {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.context_slot = slot,
            QuotePayloadVariant::Dflow(payload) => payload.context_slot = slot,
        }
    }

    pub fn context_slot(&self) -> u64 {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.context_slot,
            QuotePayloadVariant::Dflow(payload) => payload.context_slot,
        }
    }

    pub fn time_taken(&self) -> f64 {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.time_taken,
            QuotePayloadVariant::Dflow(_) => 0.0,
        }
    }

    pub fn set_time_taken(&mut self, value: f64) {
        if let QuotePayloadVariant::Jupiter(payload) = self {
            payload.time_taken = value;
        }
    }

    pub fn route_len(&self) -> usize {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.route_plan.len(),
            QuotePayloadVariant::Dflow(payload) => payload.route_plan.len(),
        }
    }

    pub fn extend_route(&mut self, other: &Self) {
        match (self, other) {
            (QuotePayloadVariant::Jupiter(lhs), QuotePayloadVariant::Jupiter(rhs)) => {
                lhs.route_plan.extend(rhs.route_plan.iter().cloned())
            }
            (QuotePayloadVariant::Dflow(lhs), QuotePayloadVariant::Dflow(rhs)) => {
                lhs.route_plan.extend(rhs.route_plan.iter().cloned())
            }
            _ => {}
        }
    }

    pub fn set_output_mint(&mut self, mint: Pubkey) {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.output_mint = mint,
            QuotePayloadVariant::Dflow(payload) => payload.output_mint = mint,
        }
    }

    pub fn set_price_impact_zero(&mut self) {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.price_impact_pct = Decimal::ZERO,
            QuotePayloadVariant::Dflow(payload) => payload.price_impact_pct = Decimal::ZERO,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SwapInstructionsVariant {
    Jupiter(jupiter::SwapInstructionsResponse),
    Dflow(dflow::SwapInstructionsResponse),
}

impl SwapInstructionsVariant {
    pub fn kind(&self) -> AggregatorKind {
        match self {
            SwapInstructionsVariant::Jupiter(_) => AggregatorKind::Jupiter,
            SwapInstructionsVariant::Dflow(_) => AggregatorKind::Dflow,
        }
    }

    pub fn compute_unit_limit(&self) -> u32 {
        match self {
            SwapInstructionsVariant::Jupiter(response) => response.compute_unit_limit,
            SwapInstructionsVariant::Dflow(response) => response.compute_unit_limit,
        }
    }

    pub fn prioritization_fee_lamports(&self) -> Option<u64> {
        match self {
            SwapInstructionsVariant::Jupiter(response) => {
                Some(response.prioritization_fee_lamports)
            }
            SwapInstructionsVariant::Dflow(response) => response.prioritization_fee_lamports,
        }
    }

    pub fn compute_budget_instructions(&self) -> &[Instruction] {
        match self {
            SwapInstructionsVariant::Jupiter(response) => {
                response.compute_budget_instructions.as_slice()
            }
            SwapInstructionsVariant::Dflow(response) => {
                response.compute_budget_instructions.as_slice()
            }
        }
    }

    pub fn flatten_instructions(&self) -> Vec<Instruction> {
        match self {
            SwapInstructionsVariant::Jupiter(response) => response.flatten_instructions(),
            SwapInstructionsVariant::Dflow(response) => response.flatten_instructions(),
        }
    }

    pub fn resolved_lookup_tables(&self) -> &[AddressLookupTableAccount] {
        match self {
            SwapInstructionsVariant::Jupiter(response) => {
                response.resolved_lookup_tables.as_slice()
            }
            SwapInstructionsVariant::Dflow(response) => response.resolved_lookup_tables.as_slice(),
        }
    }

    pub fn address_lookup_table_addresses(&self) -> &[Pubkey] {
        match self {
            SwapInstructionsVariant::Jupiter(response) => {
                response.address_lookup_table_addresses.as_slice()
            }
            SwapInstructionsVariant::Dflow(response) => {
                response.address_lookup_table_addresses.as_slice()
            }
        }
    }
}
