use std::collections::{BTreeSet, HashMap};

use serde_json::{Value, json};
use solana_sdk::instruction::{AccountMeta as SolAccountMeta, Instruction as SolInstruction};
use solana_sdk::pubkey::Pubkey;

use crate::api::SwapInstructionsResponse;
use crate::strategy::types::TradePair;
use crate::titan::types::{SwapQuotes, SwapRoute};
use crate::titan::{TitanLeg, TitanQuoteSignal};

const COMPUTE_BUDGET_PROGRAM: Pubkey =
    solana_sdk::pubkey!("ComputeBudget111111111111111111111111111111");

#[derive(Debug, Clone)]
pub struct TitanRoutePick {
    pub provider: String,
    pub route: SwapRoute,
}

#[derive(Debug, Clone)]
pub struct TitanOpportunity {
    pub base_pair: TradePair,
    pub amount_in: u64,
    pub forward: TitanRoutePick,
    pub reverse: TitanRoutePick,
}

impl TitanOpportunity {
    pub fn forward_out_amount(&self) -> u64 {
        self.forward.route.out_amount
    }

    pub fn reverse_in_amount(&self) -> u64 {
        self.reverse.route.in_amount
    }

    pub fn gross_profit(&self) -> i128 {
        self.forward.route.out_amount as i128 - self.reverse.route.in_amount as i128
    }
}

pub struct TitanAggregator {
    cache: HashMap<TitanKey, TitanPairQuotes>,
}

impl TitanAggregator {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    pub fn update(&mut self, signal: TitanQuoteSignal) -> Option<TitanOpportunity> {
        let key = TitanKey::new(signal.base_pair.clone(), signal.amount);
        let entry = self.cache.entry(key).or_default();
        let cached = CachedQuote {
            quotes: signal.quotes,
        };

        match signal.leg {
            TitanLeg::Forward => entry.forward = Some(cached),
            TitanLeg::Reverse => entry.reverse = Some(cached),
        }

        entry.try_build(signal.base_pair, signal.amount)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TitanKey {
    pair: TradePair,
    amount: u64,
}

impl TitanKey {
    fn new(pair: TradePair, amount: u64) -> Self {
        Self { pair, amount }
    }
}

#[derive(Default)]
struct TitanPairQuotes {
    forward: Option<CachedQuote>,
    reverse: Option<CachedQuote>,
}

impl TitanPairQuotes {
    fn try_build(&self, base_pair: TradePair, amount: u64) -> Option<TitanOpportunity> {
        let forward = self.forward.as_ref()?;
        let reverse = self.reverse.as_ref()?;

        let forward_pick = select_forward(forward)?;
        let reverse_pick = select_reverse(reverse)?;

        Some(TitanOpportunity {
            base_pair,
            amount_in: amount,
            forward: forward_pick,
            reverse: reverse_pick,
        })
    }
}

struct CachedQuote {
    quotes: SwapQuotes,
}

fn select_forward(quote: &CachedQuote) -> Option<TitanRoutePick> {
    let (provider, route) = quote
        .quotes
        .quotes
        .iter()
        .max_by_key(|(_, route)| route.out_amount)?;

    Some(TitanRoutePick {
        provider: provider.clone(),
        route: route.clone(),
    })
}

fn select_reverse(quote: &CachedQuote) -> Option<TitanRoutePick> {
    let (provider, route) = quote
        .quotes
        .quotes
        .iter()
        .min_by_key(|(_, route)| route.in_amount)?;

    Some(TitanRoutePick {
        provider: provider.clone(),
        route: route.clone(),
    })
}

pub fn build_quote_value(opportunity: &TitanOpportunity) -> Value {
    json!({
        "engine": "titan",
        "forward": {
            "provider": opportunity.forward.provider,
            "referenceId": opportunity.forward.route.reference_id.clone(),
            "inAmount": opportunity.forward.route.in_amount,
            "outAmount": opportunity.forward.route.out_amount,
        },
        "reverse": {
            "provider": opportunity.reverse.provider,
            "referenceId": opportunity.reverse.route.reference_id.clone(),
            "inAmount": opportunity.reverse.route.in_amount,
            "outAmount": opportunity.reverse.route.out_amount,
        }
    })
}

pub fn build_swap_instructions_response(
    opportunity: &TitanOpportunity,
) -> Option<SwapInstructionsResponse> {
    let mut forward_instrs = convert_instructions(&opportunity.forward.route.instructions);
    let mut reverse_instrs = convert_instructions(&opportunity.reverse.route.instructions);

    let mut compute_budget_instructions = Vec::new();
    extract_compute_budget(&mut forward_instrs, &mut compute_budget_instructions);
    extract_compute_budget(&mut reverse_instrs, &mut compute_budget_instructions);

    if forward_instrs.is_empty() && reverse_instrs.is_empty() {
        return None;
    }

    let (setup_instructions, swap_instruction, other_instructions) =
        arrange_instruction_sequence(forward_instrs, reverse_instrs)?;

    let raw = build_quote_value(opportunity);
    let address_lookup_table_addresses = collect_lookup_tables(
        &opportunity.forward.route.address_lookup_tables,
        &opportunity.reverse.route.address_lookup_tables,
    );

    let forward_cu = opportunity
        .forward
        .route
        .compute_units_safe
        .or(opportunity.forward.route.compute_units)
        .unwrap_or(0);
    let reverse_cu = opportunity
        .reverse
        .route
        .compute_units_safe
        .or(opportunity.reverse.route.compute_units)
        .unwrap_or(0);
    let compute_unit_limit = (forward_cu.saturating_add(reverse_cu)).min(u32::MAX as u64) as u32;

    Some(SwapInstructionsResponse {
        raw,
        token_ledger_instruction: None,
        compute_budget_instructions,
        setup_instructions,
        swap_instruction,
        cleanup_instruction: None,
        other_instructions,
        address_lookup_table_addresses,
        prioritization_fee_lamports: 0,
        compute_unit_limit,
        prioritization_type: None,
        dynamic_slippage_report: None,
        simulation_error: None,
    })
}

fn convert_instructions(src: &[crate::titan::types::Instruction]) -> Vec<SolInstruction> {
    src.iter()
        .map(|instruction| SolInstruction {
            program_id: instruction.program_id,
            accounts: instruction
                .accounts
                .iter()
                .map(|meta| SolAccountMeta {
                    pubkey: meta.pubkey,
                    is_signer: meta.signer,
                    is_writable: meta.writable,
                })
                .collect(),
            data: instruction.data.clone(),
        })
        .collect()
}

fn extract_compute_budget(
    instructions: &mut Vec<SolInstruction>,
    bucket: &mut Vec<SolInstruction>,
) {
    let program_id = COMPUTE_BUDGET_PROGRAM;
    let mut idx = 0;
    while idx < instructions.len() {
        if instructions[idx].program_id == program_id {
            bucket.push(instructions.remove(idx));
        } else {
            idx += 1;
        }
    }
}

fn arrange_instruction_sequence(
    mut forward: Vec<SolInstruction>,
    mut reverse: Vec<SolInstruction>,
) -> Option<(Vec<SolInstruction>, SolInstruction, Vec<SolInstruction>)> {
    if !forward.is_empty() {
        let swap_instruction = forward.pop()?;
        return Some((forward, swap_instruction, reverse));
    }

    if reverse.is_empty() {
        return None;
    }
    let swap_instruction = reverse.remove(0);
    Some((Vec::new(), swap_instruction, reverse))
}

fn collect_lookup_tables(forward: &[Pubkey], reverse: &[Pubkey]) -> Vec<Pubkey> {
    let mut set = BTreeSet::new();
    for key in forward {
        set.insert(*key);
    }
    for key in reverse {
        set.insert(*key);
    }
    set.into_iter().collect()
}
