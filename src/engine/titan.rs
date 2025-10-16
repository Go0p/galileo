use std::collections::{BTreeSet, HashMap, HashSet};
use std::str::FromStr;

use serde_json::{Value, json};
use solana_sdk::instruction::{AccountMeta as SolAccountMeta, Instruction as SolInstruction};
use solana_sdk::pubkey::Pubkey;
use tracing::debug;

use crate::api::{SwapInstructionsResponse, swap_instructions::PrioritizationType};
use crate::config::LanderSettings;
use crate::lander::compute_priority_fee_micro_lamports;
use crate::strategy::types::TradePair;
use crate::titan::types::{SwapQuotes, SwapRoute};
use crate::titan::{TitanLeg, TitanQuoteSignal};

const TITAN_PROVIDER_ID: &str = "Titan";
const COMPUTE_BUDGET_PROGRAM: Pubkey =
    solana_sdk::pubkey!("ComputeBudget111111111111111111111111111111");
const TITAN_PLACEHOLDER_PUBKEY: Pubkey =
    solana_sdk::pubkey!("Titan11111111111111111111111111111111111111");
const ASSOCIATED_TOKEN_PROGRAM: Pubkey =
    solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
const TOKEN_PROGRAM: Pubkey = solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const TOKEN_2022_PROGRAM: Pubkey =
    solana_sdk::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
const SYSTEM_PROGRAM: Pubkey = solana_sdk::pubkey!("11111111111111111111111111111111");

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

        let forward_out = forward_pick.route.out_amount;
        let reverse_in = reverse_pick.route.in_amount;
        if forward_out < reverse_in {
            debug!(
                target: "engine::titan",
                input_mint = %base_pair.input_mint,
                intermediate_mint = %base_pair.output_mint,
                amount,
                forward_out,
                reverse_in,
                "忽略 Titan 报价：逆向 ExactOut 需求超过正向可兑换金额"
            );
            return None;
        }

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
    let route = quote.quotes.quotes.get(TITAN_PROVIDER_ID)?;
    Some(TitanRoutePick {
        provider: TITAN_PROVIDER_ID.to_string(),
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
    wallet_pubkey: &Pubkey,
    lander_settings: &LanderSettings,
    compute_unit_price_override: Option<u64>,
) -> Option<SwapInstructionsResponse> {
    let mut forward_instrs = convert_instructions(&opportunity.forward.route.instructions);
    let mut reverse_instrs = convert_instructions(&opportunity.reverse.route.instructions);

    remove_create_associated_token(&mut forward_instrs);
    remove_create_associated_token(&mut reverse_instrs);
    remove_token_close_accounts(&mut forward_instrs);
    remove_token_close_accounts(&mut reverse_instrs);

    let mut compute_budget_instructions = Vec::new();
    extract_compute_budget(&mut forward_instrs, &mut compute_budget_instructions);
    extract_compute_budget(&mut reverse_instrs, &mut compute_budget_instructions);

    if forward_instrs.is_empty() && reverse_instrs.is_empty() {
        return None;
    }

    let (mut setup_instructions, mut swap_instruction, mut other_instructions) =
        arrange_instruction_sequence(forward_instrs, reverse_instrs)?;

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
    let compute_unit_price = compute_unit_price_override.unwrap_or_else(|| {
        compute_priority_fee_micro_lamports(lander_settings, compute_unit_limit)
    });

    ensure_compute_budget_instructions(
        &mut compute_budget_instructions,
        compute_unit_limit,
        compute_unit_price,
    );

    let ata_rewrites = build_placeholder_ata_map(opportunity, wallet_pubkey);
    remove_native_wrap_primitives(&mut setup_instructions);
    remove_token_close_accounts(&mut setup_instructions);

    rewrite_placeholder_accounts(
        &mut compute_budget_instructions,
        wallet_pubkey,
        &ata_rewrites,
    );
    rewrite_placeholder_accounts(&mut setup_instructions, wallet_pubkey, &ata_rewrites);
    rewrite_instruction_accounts(&mut swap_instruction, wallet_pubkey, &ata_rewrites);
    rewrite_placeholder_accounts(&mut other_instructions, wallet_pubkey, &ata_rewrites);
    remove_token_close_accounts(&mut other_instructions);
    let raw = build_quote_value(opportunity);
    let address_lookup_table_addresses = collect_lookup_tables(
        &opportunity.forward.route.address_lookup_tables,
        &opportunity.reverse.route.address_lookup_tables,
    );

    let prioritization_type = if compute_unit_price > 0 {
        Some(PrioritizationType::ComputeBudget {
            micro_lamports: compute_unit_price,
            estimated_micro_lamports: Some(compute_unit_price),
        })
    } else {
        None
    };

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
        prioritization_type,
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

fn rewrite_placeholder_accounts(
    instructions: &mut [SolInstruction],
    replacement: &Pubkey,
    ata_rewrites: &HashMap<Pubkey, Pubkey>,
) {
    for instruction in instructions {
        rewrite_instruction_accounts(instruction, replacement, ata_rewrites);
    }
}

fn rewrite_instruction_accounts(
    instruction: &mut SolInstruction,
    replacement: &Pubkey,
    ata_rewrites: &HashMap<Pubkey, Pubkey>,
) {
    for account in &mut instruction.accounts {
        if account.pubkey == TITAN_PLACEHOLDER_PUBKEY {
            account.pubkey = *replacement;
            continue;
        }

        if let Some(mapped) = ata_rewrites.get(&account.pubkey) {
            account.pubkey = *mapped;
        }
    }
}

fn remove_create_associated_token(instructions: &mut Vec<SolInstruction>) {
    instructions.retain(|instruction| {
        if instruction.program_id != ASSOCIATED_TOKEN_PROGRAM {
            return true;
        }

        match instruction.data.first() {
            Some(1) => false,
            _ => true,
        }
    });
}

fn remove_native_wrap_primitives(instructions: &mut Vec<SolInstruction>) {
    instructions.retain(|instruction| {
        if instruction.program_id == SYSTEM_PROGRAM && is_system_transfer(instruction) {
            return false;
        }
        if instruction.program_id == TOKEN_PROGRAM && is_sync_native(instruction) {
            return false;
        }
        true
    });
}

fn is_system_transfer(instruction: &SolInstruction) -> bool {
    if instruction.data.len() != 12 {
        return false;
    }
    let mut opcode = [0u8; 4];
    opcode.copy_from_slice(&instruction.data[0..4]);
    u32::from_le_bytes(opcode) == 2
}

fn is_sync_native(instruction: &SolInstruction) -> bool {
    instruction.data.len() == 1 && instruction.data[0] == 17
}

fn remove_token_close_accounts(instructions: &mut Vec<SolInstruction>) {
    instructions.retain(|instruction| {
        if instruction.program_id != TOKEN_PROGRAM {
            return true;
        }
        if instruction.data.first().copied() != Some(9) {
            return true;
        }
        false
    });
}

fn ensure_compute_budget_instructions(
    instructions: &mut Vec<SolInstruction>,
    compute_unit_limit: u32,
    compute_unit_price_micro_lamports: u64,
) {
    let mut prepend = Vec::new();

    let has_limit = instructions
        .iter()
        .any(|ix| ix.program_id == COMPUTE_BUDGET_PROGRAM && matches!(ix.data.first(), Some(0x02)));
    if compute_unit_limit > 0 && !has_limit {
        prepend.push(build_compute_unit_limit_instruction(compute_unit_limit));
    }

    let has_price = instructions
        .iter()
        .any(|ix| ix.program_id == COMPUTE_BUDGET_PROGRAM && matches!(ix.data.first(), Some(0x03)));
    if !has_price {
        prepend.push(build_compute_unit_price_instruction(
            compute_unit_price_micro_lamports,
        ));
    }

    if !prepend.is_empty() {
        instructions.splice(0..0, prepend);
    }
}

fn build_compute_unit_limit_instruction(limit: u32) -> SolInstruction {
    let mut data = Vec::with_capacity(5);
    data.push(0x02);
    data.extend_from_slice(&limit.to_le_bytes());
    SolInstruction {
        program_id: COMPUTE_BUDGET_PROGRAM,
        accounts: Vec::new(),
        data,
    }
}

fn build_compute_unit_price_instruction(price: u64) -> SolInstruction {
    let mut data = Vec::with_capacity(9);
    data.push(0x03);
    data.extend_from_slice(&price.to_le_bytes());
    SolInstruction {
        program_id: COMPUTE_BUDGET_PROGRAM,
        accounts: Vec::new(),
        data,
    }
}

fn build_placeholder_ata_map(
    opportunity: &TitanOpportunity,
    wallet_pubkey: &Pubkey,
) -> HashMap<Pubkey, Pubkey> {
    let mut mapping = HashMap::new();
    let mints = collect_relevant_mints(opportunity);

    for mint in mints {
        let placeholder_default = compute_associated_token_address_for_program(
            &TITAN_PLACEHOLDER_PUBKEY,
            &mint,
            &TOKEN_PROGRAM,
        );
        let wallet_default =
            compute_associated_token_address_for_program(wallet_pubkey, &mint, &TOKEN_PROGRAM);
        mapping.insert(placeholder_default, wallet_default);

        let placeholder_token_2022 = compute_associated_token_address_for_program(
            &TITAN_PLACEHOLDER_PUBKEY,
            &mint,
            &TOKEN_2022_PROGRAM,
        );
        let wallet_token_2022 =
            compute_associated_token_address_for_program(wallet_pubkey, &mint, &TOKEN_2022_PROGRAM);
        mapping.insert(placeholder_token_2022, wallet_token_2022);
    }

    mapping
}

fn collect_relevant_mints(opportunity: &TitanOpportunity) -> HashSet<Pubkey> {
    let mut mints = HashSet::new();

    if let Ok(mint) = Pubkey::from_str(&opportunity.base_pair.input_mint) {
        mints.insert(mint);
    }

    if let Ok(mint) = Pubkey::from_str(&opportunity.base_pair.output_mint) {
        mints.insert(mint);
    }

    for step in &opportunity.forward.route.steps {
        mints.insert(step.input_mint);
        mints.insert(step.output_mint);
        if let Some(fee_mint) = step.fee_mint {
            mints.insert(fee_mint);
        }
    }

    for step in &opportunity.reverse.route.steps {
        mints.insert(step.input_mint);
        mints.insert(step.output_mint);
        if let Some(fee_mint) = step.fee_mint {
            mints.insert(fee_mint);
        }
    }

    mints
}

fn compute_associated_token_address_for_program(
    owner: &Pubkey,
    mint: &Pubkey,
    token_program: &Pubkey,
) -> Pubkey {
    let seeds: [&[u8]; 3] = [owner.as_ref(), token_program.as_ref(), mint.as_ref()];
    Pubkey::find_program_address(&seeds, &ASSOCIATED_TOKEN_PROGRAM).0
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
