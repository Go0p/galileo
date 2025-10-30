use rust_decimal::Decimal;
use std::collections::HashSet;
use std::ops::Deref;

use solana_sdk::instruction::Instruction;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;

use crate::api::dflow;
use crate::api::jupiter;
use crate::api::kamino;
use crate::api::ultra::{OrderResponse, OrderResponsePayload, RoutePlanStep};
use crate::engine::ultra::UltraFinalizedSwap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregatorKind {
    Jupiter,
    Dflow,
    Ultra,
    Kamino,
}

#[derive(Debug, Clone)]
pub enum QuoteResponseVariant {
    Jupiter(jupiter::QuoteResponse),
    Dflow(dflow::QuoteResponse),
    Ultra(OrderResponse),
    Kamino(KaminoQuote),
}

#[derive(Debug, Clone)]
pub struct KaminoQuote {
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub route: kamino::Route,
}

#[derive(Debug, Clone)]
pub struct KaminoQuotePayload {
    pub route: kamino::Route,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub context_slot: u64,
    pub time_taken_ms: f64,
}

impl QuoteResponseVariant {
    pub fn kind(&self) -> AggregatorKind {
        match self {
            QuoteResponseVariant::Jupiter(_) => AggregatorKind::Jupiter,
            QuoteResponseVariant::Dflow(_) => AggregatorKind::Dflow,
            QuoteResponseVariant::Ultra(_) => AggregatorKind::Ultra,
            QuoteResponseVariant::Kamino(_) => AggregatorKind::Kamino,
        }
    }

    #[allow(dead_code)]
    pub fn input_mint(&self) -> Pubkey {
        match self {
            QuoteResponseVariant::Jupiter(resp) => resp.input_mint,
            QuoteResponseVariant::Dflow(resp) => resp.payload().input_mint,
            QuoteResponseVariant::Ultra(resp) => ultra_input_mint(&resp),
            QuoteResponseVariant::Kamino(resp) => resp.input_mint,
        }
    }

    #[allow(dead_code)]
    pub fn output_mint(&self) -> Pubkey {
        match self {
            QuoteResponseVariant::Jupiter(resp) => resp.output_mint,
            QuoteResponseVariant::Dflow(resp) => resp.payload().output_mint,
            QuoteResponseVariant::Ultra(resp) => ultra_output_mint(&resp),
            QuoteResponseVariant::Kamino(resp) => resp.output_mint,
        }
    }

    #[allow(dead_code)]
    pub fn in_amount(&self) -> u64 {
        match self {
            QuoteResponseVariant::Jupiter(resp) => resp.in_amount,
            QuoteResponseVariant::Dflow(resp) => resp.payload().in_amount,
            QuoteResponseVariant::Ultra(resp) => ultra_in_amount(&resp),
            QuoteResponseVariant::Kamino(resp) => resp.route.amount_in(),
        }
    }

    pub fn out_amount(&self) -> u64 {
        match self {
            QuoteResponseVariant::Jupiter(resp) => resp.out_amount,
            QuoteResponseVariant::Dflow(resp) => resp.payload().out_amount,
            QuoteResponseVariant::Ultra(resp) => ultra_out_amount(&resp),
            QuoteResponseVariant::Kamino(resp) => resp.route.amount_out(),
        }
    }

    pub fn clone_payload(&self) -> QuotePayloadVariant {
        match self {
            QuoteResponseVariant::Jupiter(resp) => QuotePayloadVariant::Jupiter((**resp).clone()),
            QuoteResponseVariant::Dflow(resp) => QuotePayloadVariant::Dflow(resp.payload().clone()),
            QuoteResponseVariant::Ultra(resp) => {
                let (_, payload) = resp.clone().into_parts();
                QuotePayloadVariant::Ultra(UltraQuotePayload {
                    context_slot: 0,
                    time_taken_ms: resp.total_time.unwrap_or_default() as f64,
                    payload,
                })
            }
            QuoteResponseVariant::Kamino(resp) => {
                let route = resp.route.clone();
                let time_taken_ms = route.response_time_get_quote_ms as f64;
                QuotePayloadVariant::Kamino(KaminoQuotePayload {
                    route,
                    input_mint: resp.input_mint,
                    output_mint: resp.output_mint,
                    context_slot: 0,
                    time_taken_ms,
                })
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum QuotePayloadVariant {
    Jupiter(jupiter::QuoteResponsePayload),
    Dflow(dflow::QuoteResponsePayload),
    Ultra(UltraQuotePayload),
    Kamino(KaminoQuotePayload),
}

#[derive(Debug, Clone)]
pub struct UltraQuotePayload {
    pub payload: OrderResponsePayload,
    pub context_slot: u64,
    pub time_taken_ms: f64,
}

impl QuotePayloadVariant {
    #[allow(dead_code)]
    pub fn kind(&self) -> AggregatorKind {
        match self {
            QuotePayloadVariant::Jupiter(_) => AggregatorKind::Jupiter,
            QuotePayloadVariant::Dflow(_) => AggregatorKind::Dflow,
            QuotePayloadVariant::Ultra(_) => AggregatorKind::Ultra,
            QuotePayloadVariant::Kamino(_) => AggregatorKind::Kamino,
        }
    }

    #[allow(dead_code)]
    pub fn input_mint(&self) -> Pubkey {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.input_mint,
            QuotePayloadVariant::Dflow(payload) => payload.input_mint,
            QuotePayloadVariant::Ultra(payload) => ultra_input_mint_from_payload(&payload.payload),
            QuotePayloadVariant::Kamino(payload) => payload.input_mint,
        }
    }

    pub fn output_mint(&self) -> Pubkey {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.output_mint,
            QuotePayloadVariant::Dflow(payload) => payload.output_mint,
            QuotePayloadVariant::Ultra(payload) => ultra_output_mint_from_payload(&payload.payload),
            QuotePayloadVariant::Kamino(payload) => payload.output_mint,
        }
    }

    #[allow(dead_code)]
    pub fn out_amount(&self) -> u64 {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.out_amount,
            QuotePayloadVariant::Dflow(payload) => payload.out_amount,
            QuotePayloadVariant::Ultra(payload) => ultra_out_amount_from_payload(&payload.payload),
            QuotePayloadVariant::Kamino(payload) => payload.route.amount_out(),
        }
    }

    #[allow(dead_code)]
    pub fn other_amount_threshold(&self) -> u64 {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.other_amount_threshold,
            QuotePayloadVariant::Dflow(payload) => payload.other_amount_threshold,
            QuotePayloadVariant::Ultra(payload) => payload
                .payload
                .other_amount_threshold
                .unwrap_or_else(|| ultra_out_amount_from_payload(&payload.payload)),
            QuotePayloadVariant::Kamino(payload) => {
                payload.route.amounts_exact_in.amount_out_guaranteed
            }
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
            QuotePayloadVariant::Ultra(payload) => {
                payload.payload.out_amount = Some(value);
                payload.payload.other_amount_threshold = Some(value);
            }
            QuotePayloadVariant::Kamino(payload) => {
                payload.route.amounts_exact_in.amount_out = value;
                payload.route.amounts_exact_in.amount_out_guaranteed = value;
                payload.route.amounts_exact_out.amount_out = value;
                payload.route.amounts_exact_out.amount_in = value;
                payload.route.amounts_exact_out.amount_in_guaranteed = value;
            }
        }
    }

    pub fn set_context_slot(&mut self, slot: u64) {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.context_slot = slot,
            QuotePayloadVariant::Dflow(payload) => payload.context_slot = slot,
            QuotePayloadVariant::Ultra(payload) => payload.context_slot = slot,
            QuotePayloadVariant::Kamino(payload) => payload.context_slot = slot,
        }
    }

    pub fn context_slot(&self) -> u64 {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.context_slot,
            QuotePayloadVariant::Dflow(payload) => payload.context_slot,
            QuotePayloadVariant::Ultra(payload) => payload.context_slot,
            QuotePayloadVariant::Kamino(payload) => payload.context_slot,
        }
    }

    pub fn time_taken(&self) -> f64 {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.time_taken,
            QuotePayloadVariant::Dflow(_) => 0.0,
            QuotePayloadVariant::Ultra(payload) => payload.time_taken_ms,
            QuotePayloadVariant::Kamino(payload) => payload.time_taken_ms,
        }
    }

    pub fn set_time_taken(&mut self, value: f64) {
        if let QuotePayloadVariant::Jupiter(payload) = self {
            payload.time_taken = value;
        } else if let QuotePayloadVariant::Ultra(payload) = self {
            payload.time_taken_ms = value;
            payload.payload.total_time = Some(value.max(0.0).round() as u64);
        } else if let QuotePayloadVariant::Kamino(payload) = self {
            payload.time_taken_ms = value;
        }
    }

    pub fn route_len(&self) -> usize {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.route_plan.len(),
            QuotePayloadVariant::Dflow(payload) => payload.route_plan.len(),
            QuotePayloadVariant::Ultra(payload) => payload.payload.route_plan.len(),
            QuotePayloadVariant::Kamino(payload) => payload.route.instructions.swap_ixs.len(),
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
            (QuotePayloadVariant::Ultra(lhs), QuotePayloadVariant::Ultra(rhs)) => lhs
                .payload
                .route_plan
                .extend(rhs.payload.route_plan.iter().cloned()),
            (QuotePayloadVariant::Kamino(lhs), QuotePayloadVariant::Kamino(rhs)) => {
                lhs.route
                    .instructions
                    .append_from(&rhs.route.instructions);
                for value in &rhs.route.lookup_table_accounts_bs58 {
                    if let Some(existing) = lhs
                        .route
                        .lookup_table_accounts_bs58
                        .iter_mut()
                        .find(|entry| entry.key == value.key)
                    {
                        let mut seen: HashSet<String> =
                            existing.addresses.iter().cloned().collect();
                        for addr in &value.addresses {
                            if seen.insert(addr.clone()) {
                                existing.addresses.push(addr.clone());
                            }
                        }
                    } else {
                        lhs.route
                            .lookup_table_accounts_bs58
                            .push(value.clone());
                    }
                }
                lhs.route.response_time_get_quote_ms = lhs
                    .route
                    .response_time_get_quote_ms
                    .saturating_add(rhs.route.response_time_get_quote_ms);
            }
            _ => {}
        }
    }

    pub fn set_output_mint(&mut self, mint: Pubkey) {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.output_mint = mint,
            QuotePayloadVariant::Dflow(payload) => payload.output_mint = mint,
            QuotePayloadVariant::Ultra(payload) => {
                payload.payload.output_mint = Some(mint);
            }
            QuotePayloadVariant::Kamino(payload) => {
                payload.output_mint = mint;
            }
        }
    }

    pub fn set_price_impact_zero(&mut self) {
        match self {
            QuotePayloadVariant::Jupiter(payload) => payload.price_impact_pct = Decimal::ZERO,
            QuotePayloadVariant::Dflow(payload) => payload.price_impact_pct = Decimal::ZERO,
            QuotePayloadVariant::Ultra(payload) => {
                payload.payload.price_impact = Decimal::ZERO;
                payload.payload.price_impact_pct = Some("0".to_string());
            }
            QuotePayloadVariant::Kamino(payload) => {
                payload.route.price_impact_bps = Some(0);
                payload.route.guaranteed_price_impact_bps = Some(0);
                payload.route.price_impact_amount = Some("0".to_string());
                payload.route.guaranteed_price_impact_amount = Some("0".to_string());
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum SwapInstructionsVariant {
    Jupiter(jupiter::SwapInstructionsResponse),
    Dflow(dflow::SwapInstructionsResponse),
    Ultra(UltraFinalizedSwap),
    MultiLeg(MultiLegInstructions),
    Kamino(KaminoSwapBundle),
}

#[derive(Debug, Clone)]
pub struct MultiLegInstructions {
    pub compute_budget_instructions: Vec<Instruction>,
    pub main_instructions: Vec<Instruction>,
    pub address_lookup_table_addresses: Vec<Pubkey>,
    pub resolved_lookup_tables: Vec<AddressLookupTableAccount>,
    pub prioritization_fee_lamports: Option<u64>,
    pub compute_unit_limit: u32,
}

impl MultiLegInstructions {
    pub fn new(
        compute_budget_instructions: Vec<Instruction>,
        main_instructions: Vec<Instruction>,
        address_lookup_table_addresses: Vec<Pubkey>,
        resolved_lookup_tables: Vec<AddressLookupTableAccount>,
        prioritization_fee_lamports: Option<u64>,
        compute_unit_limit: u32,
    ) -> Self {
        Self {
            compute_budget_instructions,
            main_instructions,
            address_lookup_table_addresses,
            resolved_lookup_tables,
            prioritization_fee_lamports,
            compute_unit_limit,
        }
    }

    pub fn flatten_instructions(&self) -> Vec<Instruction> {
        let mut combined = Vec::with_capacity(
            self.compute_budget_instructions.len() + self.main_instructions.len(),
        );
        combined.extend(self.compute_budget_instructions.iter().cloned());
        combined.extend(self.main_instructions.iter().cloned());
        combined
    }

    pub fn dedup_lookup_tables(&mut self) {
        let mut seen: HashSet<Pubkey> = HashSet::new();
        self.address_lookup_table_addresses
            .retain(|key| seen.insert(*key));

        let mut seen_accounts: HashSet<Pubkey> = HashSet::new();
        self.resolved_lookup_tables
            .retain(|account| seen_accounts.insert(account.key));
    }
}

#[derive(Debug, Clone)]
pub struct KaminoSwapBundle {
    pub compute_budget_instructions: Vec<Instruction>,
    pub main_instructions: Vec<Instruction>,
    pub lookup_table_addresses: Vec<Pubkey>,
    pub resolved_lookup_tables: Vec<AddressLookupTableAccount>,
    pub prioritization_fee_lamports: Option<u64>,
    pub compute_unit_limit: u32,
}

impl KaminoSwapBundle {
    pub fn new(
        compute_budget_instructions: Vec<Instruction>,
        main_instructions: Vec<Instruction>,
        lookup_table_addresses: Vec<Pubkey>,
        resolved_lookup_tables: Vec<AddressLookupTableAccount>,
        prioritization_fee_lamports: Option<u64>,
        compute_unit_limit: u32,
    ) -> Self {
        Self {
            compute_budget_instructions,
            main_instructions,
            lookup_table_addresses,
            resolved_lookup_tables,
            prioritization_fee_lamports,
            compute_unit_limit,
        }
    }

    pub fn flatten_instructions(&self) -> Vec<Instruction> {
        let mut combined = Vec::with_capacity(
            self.compute_budget_instructions.len() + self.main_instructions.len(),
        );
        combined.extend(self.compute_budget_instructions.iter().cloned());
        combined.extend(self.main_instructions.iter().cloned());
        combined
    }
}

impl SwapInstructionsVariant {
    #[allow(dead_code)]
    pub fn kind(&self) -> AggregatorKind {
        match self {
            SwapInstructionsVariant::Jupiter(_) => AggregatorKind::Jupiter,
            SwapInstructionsVariant::Dflow(_) => AggregatorKind::Dflow,
            SwapInstructionsVariant::Ultra(_) => AggregatorKind::Ultra,
            SwapInstructionsVariant::MultiLeg(_) => AggregatorKind::Jupiter,
            SwapInstructionsVariant::Kamino(_) => AggregatorKind::Kamino,
        }
    }

    pub fn compute_unit_limit(&self) -> u32 {
        match self {
            SwapInstructionsVariant::Jupiter(response) => response.compute_unit_limit,
            SwapInstructionsVariant::Dflow(response) => response.compute_unit_limit,
            SwapInstructionsVariant::Ultra(bundle) => bundle.compute_unit_limit,
            SwapInstructionsVariant::MultiLeg(bundle) => bundle.compute_unit_limit,
            SwapInstructionsVariant::Kamino(bundle) => bundle.compute_unit_limit,
        }
    }

    pub fn prioritization_fee_lamports(&self) -> Option<u64> {
        match self {
            SwapInstructionsVariant::Jupiter(response) => {
                Some(response.prioritization_fee_lamports)
            }
            SwapInstructionsVariant::Dflow(response) => response.prioritization_fee_lamports,
            SwapInstructionsVariant::Ultra(bundle) => bundle.prioritization_fee_lamports,
            SwapInstructionsVariant::MultiLeg(bundle) => bundle.prioritization_fee_lamports,
            SwapInstructionsVariant::Kamino(bundle) => bundle.prioritization_fee_lamports,
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
            SwapInstructionsVariant::Ultra(bundle) => bundle.compute_budget_instructions.as_slice(),
            SwapInstructionsVariant::MultiLeg(bundle) => {
                bundle.compute_budget_instructions.as_slice()
            }
            SwapInstructionsVariant::Kamino(bundle) => {
                bundle.compute_budget_instructions.as_slice()
            }
        }
    }

    pub fn flatten_instructions(&self) -> Vec<Instruction> {
        match self {
            SwapInstructionsVariant::Jupiter(response) => response.flatten_instructions(),
            SwapInstructionsVariant::Dflow(response) => response.flatten_instructions(),
            SwapInstructionsVariant::Ultra(bundle) => {
                let mut combined = Vec::with_capacity(
                    bundle.compute_budget_instructions.len() + bundle.main_instructions.len(),
                );
                combined.extend(bundle.compute_budget_instructions.iter().cloned());
                combined.extend(bundle.main_instructions.iter().cloned());
                combined
            }
            SwapInstructionsVariant::MultiLeg(bundle) => bundle.flatten_instructions(),
            SwapInstructionsVariant::Kamino(bundle) => bundle.flatten_instructions(),
        }
    }

    pub fn resolved_lookup_tables(&self) -> &[AddressLookupTableAccount] {
        match self {
            SwapInstructionsVariant::Jupiter(response) => {
                response.resolved_lookup_tables.as_slice()
            }
            SwapInstructionsVariant::Dflow(_) => &[],
            SwapInstructionsVariant::Ultra(bundle) => bundle.resolved_lookup_tables.as_slice(),
            SwapInstructionsVariant::MultiLeg(bundle) => bundle.resolved_lookup_tables.as_slice(),
            SwapInstructionsVariant::Kamino(bundle) => bundle.resolved_lookup_tables.as_slice(),
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
            SwapInstructionsVariant::Ultra(bundle) => {
                bundle.address_lookup_table_addresses.as_slice()
            }
            SwapInstructionsVariant::MultiLeg(bundle) => {
                bundle.address_lookup_table_addresses.as_slice()
            }
            SwapInstructionsVariant::Kamino(bundle) => bundle.lookup_table_addresses.as_slice(),
        }
    }

    pub fn blockhash_with_metadata(&self) -> Option<&dflow::BlockhashWithMetadata> {
        match self {
            SwapInstructionsVariant::Jupiter(_) => None,
            SwapInstructionsVariant::Dflow(response) => Some(response.blockhash_with_metadata()),
            SwapInstructionsVariant::Ultra(_) => None,
            SwapInstructionsVariant::MultiLeg(_) => None,
            SwapInstructionsVariant::Kamino(_) => None,
        }
    }
}

fn ultra_input_mint(response: &OrderResponse) -> Pubkey {
    ultra_input_mint_from_payload(response.deref())
}

fn ultra_output_mint(response: &OrderResponse) -> Pubkey {
    ultra_output_mint_from_payload(response.deref())
}

fn ultra_in_amount(response: &OrderResponse) -> u64 {
    ultra_in_amount_from_payload(response.deref())
}

fn ultra_out_amount(response: &OrderResponse) -> u64 {
    ultra_out_amount_from_payload(response.deref())
}

fn ultra_input_mint_from_payload(payload: &OrderResponsePayload) -> Pubkey {
    payload
        .input_mint
        .or_else(|| first_route_step(payload).map(|step| step.swap_info.input_mint))
        .unwrap_or_default()
}

fn ultra_output_mint_from_payload(payload: &OrderResponsePayload) -> Pubkey {
    payload
        .output_mint
        .or_else(|| last_route_step(payload).map(|step| step.swap_info.output_mint))
        .unwrap_or_default()
}

fn ultra_in_amount_from_payload(payload: &OrderResponsePayload) -> u64 {
    payload
        .in_amount
        .or_else(|| sum_route_plan_amount(&payload.route_plan, |step| step.swap_info.in_amount))
        .unwrap_or_default()
}

fn ultra_out_amount_from_payload(payload: &OrderResponsePayload) -> u64 {
    payload
        .out_amount
        .or_else(|| sum_route_plan_amount(&payload.route_plan, |step| step.swap_info.out_amount))
        .unwrap_or_default()
}

fn first_route_step(payload: &OrderResponsePayload) -> Option<&RoutePlanStep> {
    payload.route_plan.first()
}

fn last_route_step(payload: &OrderResponsePayload) -> Option<&RoutePlanStep> {
    payload.route_plan.last()
}

fn sum_route_plan_amount<F>(steps: &[RoutePlanStep], mut extractor: F) -> Option<u64>
where
    F: FnMut(&RoutePlanStep) -> u64,
{
    if steps.is_empty() {
        return None;
    }
    steps
        .iter()
        .try_fold(0u64, |acc, step| acc.checked_add(extractor(step)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::kamino::quote::{
        AmountsExactIn, AmountsExactOut, LookupTableEntry, RawInstruction, RouteInstructions,
    };
    use crate::api::kamino::Route;
    use solana_sdk::pubkey::Pubkey;

    fn build_route(
        swap_program: Pubkey,
        lookup_addr: &str,
        response_ms: u64,
        amount_in: u64,
        amount_out: u64,
    ) -> Route {
        let instructions = RouteInstructions {
            create_in_ata_ixs: Vec::new(),
            create_out_ata_ixs: Vec::new(),
            wrap_sol_ixs: Vec::new(),
            limo_logs_start_ixs: Vec::new(),
            limo_ledger_start_ixs: Vec::new(),
            swap_ixs: vec![RawInstruction {
                program_id: swap_program,
                data: vec![1],
                keys: Vec::new(),
            }],
            limo_ledger_end_ixs: Vec::new(),
            limo_logs_end_ixs: Vec::new(),
            unwrap_sol_ixs: Vec::new(),
        };
        Route {
            router_type: "kamino".to_string(),
            response_time_get_quote_ms: response_ms,
            price_impact_bps: None,
            guaranteed_price_impact_bps: None,
            price_impact_amount: None,
            guaranteed_price_impact_amount: None,
            lookup_table_accounts_bs58: vec![LookupTableEntry {
                key: lookup_addr.to_string(),
                addresses: vec!["11111111111111111111111111111111".to_string()],
            }],
            amounts_exact_in: AmountsExactIn {
                amount_in,
                amount_out_guaranteed: amount_out,
                amount_out,
            },
            amounts_exact_out: AmountsExactOut {
                amount_out: 0,
                amount_in_guaranteed: 0,
                amount_in: 0,
            },
            instructions,
        }
    }

    #[test]
    fn extend_route_combines_kamino_instructions() {
        let input_mint = Pubkey::new_unique();
        let output_mint = Pubkey::new_unique();
        let intermediate = Pubkey::new_unique();

        let route_a = build_route(
            Pubkey::new_unique(),
            "LookupA",
            5,
            100,
            110,
        );
        let route_b = build_route(
            Pubkey::new_unique(),
            "LookupB",
            7,
            110,
            120,
        );

        let mut lhs = QuotePayloadVariant::Kamino(KaminoQuotePayload {
            route: route_a,
            input_mint,
            output_mint: intermediate,
            context_slot: 0,
            time_taken_ms: 5.0,
        });
        let rhs = QuotePayloadVariant::Kamino(KaminoQuotePayload {
            route: route_b,
            input_mint: intermediate,
            output_mint,
            context_slot: 0,
            time_taken_ms: 7.0,
        });

        lhs.extend_route(&rhs);

        if let QuotePayloadVariant::Kamino(payload) = lhs {
            assert_eq!(payload.route.instructions.swap_ixs.len(), 2);
            assert_eq!(payload.route.lookup_table_accounts_bs58.len(), 2);
            assert!(
                payload
                    .route
                    .lookup_table_accounts_bs58
                    .iter()
                    .any(|entry| entry.key == "LookupA")
            );
            assert!(
                payload
                    .route
                    .lookup_table_accounts_bs58
                    .iter()
                    .any(|entry| entry.key == "LookupB")
            );
            assert_eq!(payload.route.response_time_get_quote_ms, 12);
        } else {
            panic!("Kamino payload expected");
        }
    }
}
