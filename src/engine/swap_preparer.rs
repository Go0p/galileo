use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

use tracing::{debug, warn};

use super::aggregator::{KaminoSwapBundle, QuotePayloadVariant, SwapInstructionsVariant};
use super::error::{EngineError, EngineResult};
use super::identity::EngineIdentity;
use super::types::SwapOpportunity;
use crate::api::dflow::{
    ComputeUnitPriceMicroLamports as DflowComputeUnitPriceMicroLamports, DflowApiClient,
    SwapInstructionsRequest as DflowSwapInstructionsRequest,
};
use crate::api::jupiter::{
    ComputeUnitPriceMicroLamports, JupiterApiClient, SwapInstructionsRequest,
};
use crate::config::{DflowSwapConfig, JupiterSwapConfig, KaminoQuoteConfig, UltraSwapConfig};
use crate::engine::ultra::{
    UltraAdapter, UltraAdapterError, UltraContext, UltraFinalizedSwap, UltraLookupResolver,
    UltraPreparationParams, UltraPreparedSwap,
};
use crate::multi_leg::alt_cache::AltCache;
use crate::network::IpLeaseHandle;
use rand::Rng;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::AddressLookupTableAccount;
use solana_sdk::pubkey::Pubkey;

use super::{COMPUTE_BUDGET_PROGRAM_ID, FALLBACK_CU_LIMIT};

#[derive(Clone, Debug)]
pub enum ComputeUnitPriceMode {
    Fixed(u64),
    Random { min: u64, max: u64 },
}

impl ComputeUnitPriceMode {
    pub fn sample(&self) -> u64 {
        match self {
            ComputeUnitPriceMode::Fixed(value) => *value,
            ComputeUnitPriceMode::Random { min, max } => {
                let (low, high) = if min <= max {
                    (*min, *max)
                } else {
                    (*max, *min)
                };
                if low == high {
                    low
                } else {
                    let mut rng = rand::rng();
                    rng.random_range(low..=high)
                }
            }
        }
    }
}

#[derive(Clone)]
pub enum SwapPreparerBackend {
    Jupiter {
        client: JupiterApiClient,
        defaults: JupiterSwapConfig,
    },
    Dflow {
        client: DflowApiClient,
        defaults: DflowSwapConfig,
    },
    Kamino {
        cu_limit_multiplier: f64,
    },
    Ultra {
        rpc: Arc<RpcClient>,
        alt_cache: AltCache,
        defaults: UltraSwapConfig,
    },
    Disabled,
}

#[derive(Clone)]
pub struct SwapPreparer {
    backend: SwapPreparerBackend,
    compute_unit_price: Option<ComputeUnitPriceMode>,
}

impl SwapPreparer {
    pub fn for_jupiter(
        client: JupiterApiClient,
        request_defaults: JupiterSwapConfig,
        compute_unit_price: Option<ComputeUnitPriceMode>,
    ) -> Self {
        Self {
            backend: SwapPreparerBackend::Jupiter {
                client,
                defaults: request_defaults,
            },
            compute_unit_price,
        }
    }

    pub fn for_dflow(
        client: DflowApiClient,
        request_defaults: DflowSwapConfig,
        compute_unit_price: Option<ComputeUnitPriceMode>,
    ) -> Self {
        Self {
            backend: SwapPreparerBackend::Dflow {
                client,
                defaults: request_defaults,
            },
            compute_unit_price,
        }
    }

    pub fn for_kamino(
        defaults: KaminoQuoteConfig,
        compute_unit_price: Option<ComputeUnitPriceMode>,
    ) -> Self {
        let multiplier = sanitize_multiplier(defaults.cu_limit_multiplier).unwrap_or(1.0);
        Self {
            backend: SwapPreparerBackend::Kamino {
                cu_limit_multiplier: multiplier,
            },
            compute_unit_price,
        }
    }

    pub fn for_ultra(
        rpc: Arc<RpcClient>,
        defaults: UltraSwapConfig,
        compute_unit_price: Option<ComputeUnitPriceMode>,
    ) -> Self {
        Self {
            backend: SwapPreparerBackend::Ultra {
                rpc,
                alt_cache: AltCache::new(),
                defaults,
            },
            compute_unit_price,
        }
    }

    pub fn disabled() -> Self {
        Self {
            backend: SwapPreparerBackend::Disabled,
            compute_unit_price: None,
        }
    }

    #[cfg_attr(feature = "hotpath", hotpath::measure)]
    pub async fn prepare(
        &self,
        opportunity: &SwapOpportunity,
        identity: &EngineIdentity,
        lease: &IpLeaseHandle,
    ) -> EngineResult<SwapInstructionsVariant> {
        let payload = opportunity
            .merged_quote
            .clone()
            .ok_or_else(|| EngineError::InvalidConfig("套利机会缺少报价数据".into()))?;

        let local_ip = Some(lease.ip());

        let variant = match (&self.backend, payload) {
            (
                SwapPreparerBackend::Jupiter { client, defaults },
                QuotePayloadVariant::Jupiter(inner),
            ) => {
                let mut request = SwapInstructionsRequest::from_payload(inner, identity.pubkey);

                request.wrap_and_unwrap_sol = defaults.wrap_and_unwrap_sol;
                request.dynamic_compute_unit_limit = defaults.dynamic_compute_unit_limit;
                request.use_shared_accounts = Some(identity.use_shared_accounts());
                request.skip_user_accounts_rpc_calls = identity.skip_user_accounts_rpc_calls();
                if let Some(fee) = identity.fee_account() {
                    match solana_sdk::pubkey::Pubkey::from_str(fee) {
                        Ok(pubkey) => request.fee_account = Some(pubkey),
                        Err(err) => {
                            warn!(
                                target = "engine::swap_preparer",
                                fee_account = fee,
                                error = %err,
                                "手续费账户解析失败，忽略配置"
                            );
                        }
                    }
                }
                if let Some(strategy) = &self.compute_unit_price {
                    let price = strategy.sample();
                    if price > 0 {
                        request.compute_unit_price_micro_lamports =
                            Some(ComputeUnitPriceMicroLamports::MicroLamports(price));
                    }
                }

                let response = client.swap_instructions(&request, local_ip).await?;
                SwapInstructionsVariant::Jupiter(response)
            }
            (
                SwapPreparerBackend::Dflow { client, defaults },
                QuotePayloadVariant::Dflow(inner),
            ) => {
                let mut request =
                    DflowSwapInstructionsRequest::from_payload(inner, identity.pubkey);
                request.wrap_and_unwrap_sol = defaults.wrap_and_unwrap_sol;
                request.dynamic_compute_unit_limit = Some(defaults.dynamic_compute_unit_limit);
                if let Some(fee) = identity.fee_account() {
                    match solana_sdk::pubkey::Pubkey::from_str(fee) {
                        Ok(pubkey) => request.fee_account = Some(pubkey),
                        Err(err) => {
                            warn!(
                                target = "engine::swap_preparer",
                                fee_account = fee,
                                error = %err,
                                "手续费账户解析失败，忽略配置"
                            );
                        }
                    }
                }
                if let Some(strategy) = &self.compute_unit_price {
                    let price = strategy.sample();
                    if price > 0 {
                        request.compute_unit_price_micro_lamports =
                            Some(DflowComputeUnitPriceMicroLamports(price));
                    }
                }

                let mut response = client.swap_instructions(&request, local_ip).await?;
                let original_limit = response.compute_unit_limit;
                let adjusted_limit =
                    response.adjust_compute_unit_limit(defaults.cu_limit_multiplier);
                if adjusted_limit != original_limit {
                    debug!(
                        target = "engine::swap_preparer",
                        original = original_limit,
                        adjusted = adjusted_limit,
                        multiplier = defaults.cu_limit_multiplier,
                        "DFlow compute unit limit 已按配置系数调整"
                    );
                }
                SwapInstructionsVariant::Dflow(response)
            }
            (
                SwapPreparerBackend::Kamino {
                    cu_limit_multiplier,
                },
                QuotePayloadVariant::Kamino(payload),
            ) => {
                let instructions = payload.route.instructions.flatten();
                let mut compute_budget_instructions = Vec::new();
                let mut main_instructions = Vec::new();
                let mut compute_unit_limit: Option<u32> = None;
                let mut total_compute_unit_limit: u128 = 0;
                let mut compute_unit_price: Option<u64> = None;

                for ix in instructions {
                    if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID {
                        if let Some(parsed) = parse_compute_budget_instruction(&ix) {
                            match parsed {
                                ParsedComputeBudget::Limit(value) => {
                                    compute_unit_limit = Some(value);
                                    total_compute_unit_limit =
                                        total_compute_unit_limit.saturating_add(value as u128);
                                    continue;
                                }
                                ParsedComputeBudget::Price(value) => {
                                    compute_unit_price = Some(value)
                                }
                                ParsedComputeBudget::Other => {}
                            }
                        }
                        compute_budget_instructions.push(ix);
                    } else {
                        main_instructions.push(ix);
                    }
                }

                let mut lookup_table_addresses = Vec::new();
                let mut resolved_lookup_tables = Vec::new();
                let mut seen_keys = HashSet::new();
                for entry in &payload.route.lookup_table_accounts_bs58 {
                    let key_text = entry.key.trim();
                    if key_text.is_empty() {
                        continue;
                    }
                    match Pubkey::from_str(key_text) {
                        Ok(key) => {
                            if seen_keys.insert(key) {
                                lookup_table_addresses.push(key);
                                let mut table_addresses = Vec::new();
                                let mut seen_addr = HashSet::new();
                                for addr in &entry.addresses {
                                    let trimmed = addr.trim();
                                    if trimmed.is_empty() {
                                        continue;
                                    }
                                    match Pubkey::from_str(trimmed) {
                                        Ok(pubkey) => {
                                            if seen_addr.insert(pubkey) {
                                                table_addresses.push(pubkey);
                                            }
                                        }
                                        Err(err) => {
                                            debug!(
                                                target: "engine::swap_preparer",
                                                address = trimmed,
                                                error = %err,
                                                "解析 Kamino lookup table address 失败，忽略"
                                            );
                                        }
                                    }
                                }
                                if !table_addresses.is_empty() {
                                    resolved_lookup_tables.push(AddressLookupTableAccount {
                                        key,
                                        addresses: table_addresses,
                                    });
                                }
                            }
                        }
                        Err(err) => {
                            debug!(
                                target: "engine::swap_preparer",
                                address = key_text,
                                error = %err,
                                "解析 Kamino lookup table key 失败，忽略"
                            );
                        }
                    }
                }

                let raw_limit = if total_compute_unit_limit > 0 {
                    total_compute_unit_limit
                        .min(u32::MAX as u128)
                        .max(1) as u32
                } else {
                    compute_unit_limit.unwrap_or(FALLBACK_CU_LIMIT)
                };
                let limit = apply_cu_limit_multiplier(raw_limit, *cu_limit_multiplier);
                if limit != raw_limit {
                    debug!(
                        target = "engine::swap_preparer",
                        original = raw_limit,
                        adjusted = limit,
                        multiplier = *cu_limit_multiplier,
                        "Kamino compute unit limit 已按配置系数调整"
                    );
                }
                compute_budget_instructions.insert(
                    0,
                    ComputeBudgetInstruction::set_compute_unit_limit(limit),
                );

                if let Some(strategy) = &self.compute_unit_price {
                    let price = strategy.sample();
                    if price > 0 {
                        compute_budget_instructions.retain(|ix| {
                            !matches!(
                                parse_compute_budget_instruction(ix),
                                Some(ParsedComputeBudget::Price(_))
                            )
                        });
                        compute_budget_instructions
                            .push(ComputeBudgetInstruction::set_compute_unit_price(price));
                        compute_unit_price = Some(price);
                    }
                }

                let prioritization_fee_lamports = compute_unit_price.map(|price| {
                    let fee = (price as u128)
                        .saturating_mul(limit as u128)
                        .checked_div(1_000_000u128)
                        .unwrap_or(0);
                    fee.min(u64::MAX as u128) as u64
                });

                let bundle = KaminoSwapBundle::new(
                    compute_budget_instructions,
                    main_instructions,
                    lookup_table_addresses,
                    resolved_lookup_tables,
                    prioritization_fee_lamports,
                    limit,
                );
                SwapInstructionsVariant::Kamino(bundle)
            }
            (
                SwapPreparerBackend::Ultra {
                    rpc,
                    alt_cache,
                    defaults,
                },
                QuotePayloadVariant::Ultra(_),
            ) => {
                let override_price = self.sample_compute_unit_price();
                let legs = opportunity.ultra_legs.as_ref().ok_or_else(|| {
                    EngineError::InvalidConfig("Ultra 套利机会缺少前后腿明细".into())
                })?;

                let rpc = Arc::clone(rpc);
                let alt_cache = alt_cache.clone();
                let expected_signer = identity.pubkey;

                let forward = UltraAdapter::prepare(
                    UltraPreparationParams::new(&legs.forward),
                    UltraContext::new(
                        expected_signer,
                        UltraLookupResolver::Fetch {
                            rpc: Arc::clone(&rpc),
                            alt_cache: alt_cache.clone(),
                        },
                    ),
                )
                .await
                .map_err(map_adapter_error)?;

                let reverse = UltraAdapter::prepare(
                    UltraPreparationParams::new(&legs.reverse),
                    UltraContext::new(
                        expected_signer,
                        UltraLookupResolver::Fetch { rpc, alt_cache },
                    ),
                )
                .await
                .map_err(map_adapter_error)?;

                let multiplier = if defaults.cu_limit_multiplier.is_finite()
                    && defaults.cu_limit_multiplier > 0.0
                {
                    defaults.cu_limit_multiplier
                } else {
                    1.0
                };
                let finalized = combine_ultra_swaps(
                    forward,
                    reverse,
                    FALLBACK_CU_LIMIT,
                    override_price,
                    multiplier,
                );

                SwapInstructionsVariant::Ultra(finalized)
            }
            (SwapPreparerBackend::Jupiter { .. }, QuotePayloadVariant::Dflow(_))
            | (SwapPreparerBackend::Jupiter { .. }, QuotePayloadVariant::Ultra(_))
            | (SwapPreparerBackend::Jupiter { .. }, QuotePayloadVariant::Kamino(_))
            | (SwapPreparerBackend::Dflow { .. }, QuotePayloadVariant::Jupiter(_))
            | (SwapPreparerBackend::Dflow { .. }, QuotePayloadVariant::Ultra(_))
            | (SwapPreparerBackend::Dflow { .. }, QuotePayloadVariant::Kamino(_))
            | (SwapPreparerBackend::Ultra { .. }, QuotePayloadVariant::Jupiter(_))
            | (SwapPreparerBackend::Ultra { .. }, QuotePayloadVariant::Dflow(_))
            | (SwapPreparerBackend::Ultra { .. }, QuotePayloadVariant::Kamino(_))
            | (SwapPreparerBackend::Kamino { .. }, QuotePayloadVariant::Jupiter(_))
            | (SwapPreparerBackend::Kamino { .. }, QuotePayloadVariant::Dflow(_))
            | (SwapPreparerBackend::Kamino { .. }, QuotePayloadVariant::Ultra(_)) => {
                return Err(EngineError::InvalidConfig(
                    "套利机会聚合器类型与落地器不匹配".into(),
                ));
            }
            (SwapPreparerBackend::Disabled, _) => {
                return Err(EngineError::InvalidConfig(
                    "swap backend 已禁用，无法构造指令".into(),
                ));
            }
        };

        Ok(variant)
    }

    pub fn sample_compute_unit_price(&self) -> Option<u64> {
        self.compute_unit_price
            .as_ref()
            .map(|mode| mode.sample())
            .filter(|price| *price > 0)
    }
}

fn sanitize_multiplier(value: f64) -> Option<f64> {
    if value.is_finite() && value > 0.0 {
        Some(value)
    } else {
        None
    }
}

fn apply_cu_limit_multiplier(base: u32, multiplier: f64) -> u32 {
    let sanitized_multiplier = sanitize_multiplier(multiplier).unwrap_or(1.0);
    let base_limit = base.max(1);
    let mut scaled = (base_limit as f64) * sanitized_multiplier;
    if !scaled.is_finite() {
        return base_limit;
    }
    if scaled < 1.0 {
        scaled = 1.0;
    }
    if scaled > u32::MAX as f64 {
        scaled = u32::MAX as f64;
    }
    scaled.round() as u32
}

#[derive(Debug, Clone, Copy)]
enum ParsedComputeBudget {
    Limit(u32),
    Price(u64),
    Other,
}

fn parse_compute_budget_instruction(ix: &Instruction) -> Option<ParsedComputeBudget> {
    if ix.program_id != COMPUTE_BUDGET_PROGRAM_ID {
        return None;
    }
    let data = ix.data.as_slice();
    if data.is_empty() {
        return Some(ParsedComputeBudget::Other);
    }
    match data[0] {
        2 => {
            if data.len() < 5 {
                return Some(ParsedComputeBudget::Other);
            }
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[1..5]);
            Some(ParsedComputeBudget::Limit(u32::from_le_bytes(bytes)))
        }
        3 => {
            if data.len() < 9 {
                return Some(ParsedComputeBudget::Other);
            }
            let mut bytes = [0u8; 8];
            bytes.copy_from_slice(&data[1..9]);
            Some(ParsedComputeBudget::Price(u64::from_le_bytes(bytes)))
        }
        _ => Some(ParsedComputeBudget::Other),
    }
}

fn map_adapter_error(err: UltraAdapterError) -> EngineError {
    match err {
        UltraAdapterError::MissingTransaction => {
            EngineError::InvalidConfig("Ultra 报价缺少 transaction 字段".into())
        }
        UltraAdapterError::Decode(inner) => {
            EngineError::InvalidConfig(format!("Ultra 交易解码失败: {inner}"))
        }
        UltraAdapterError::Instruction(inner) => {
            EngineError::InvalidConfig(format!("Ultra 指令解析失败: {inner}"))
        }
        UltraAdapterError::LookupFetch(err) => EngineError::Transaction(err),
    }
}

fn combine_ultra_swaps(
    forward: UltraPreparedSwap,
    reverse: UltraPreparedSwap,
    fallback_limit: u32,
    override_price: Option<u64>,
    multiplier: f64,
) -> UltraFinalizedSwap {
    let forward_limit = forward
        .requested_compute_unit_limit
        .unwrap_or(fallback_limit);
    let reverse_limit = reverse
        .requested_compute_unit_limit
        .unwrap_or(fallback_limit);
    let combined_limit = forward_limit
        .saturating_add(reverse_limit)
        .max(fallback_limit);

    let sanitized_multiplier = if multiplier.is_finite() && multiplier > 0.0 {
        multiplier
    } else {
        1.0
    };
    let mut scaled_limit = (combined_limit as f64) * sanitized_multiplier;
    if !scaled_limit.is_finite() {
        scaled_limit = combined_limit as f64;
    }
    let mut rounded_limit = scaled_limit.round();
    if !rounded_limit.is_finite() {
        rounded_limit = combined_limit as f64;
    }
    if rounded_limit < fallback_limit as f64 {
        rounded_limit = fallback_limit as f64;
    }
    if rounded_limit > u32::MAX as f64 {
        rounded_limit = u32::MAX as f64;
    }
    let merged_limit = rounded_limit as u32;

    let mut effective_price = forward
        .requested_compute_unit_price_micro_lamports
        .or(reverse.requested_compute_unit_price_micro_lamports);
    if let Some(price) = override_price {
        if price > 0 {
            effective_price = Some(price);
        }
    }

    let mut compute_budget_instructions = Vec::new();
    compute_budget_instructions.push(ComputeBudgetInstruction::set_compute_unit_limit(
        merged_limit,
    ));
    if let Some(price) = effective_price {
        compute_budget_instructions.push(ComputeBudgetInstruction::set_compute_unit_price(price));
    }
    compute_budget_instructions.extend(forward.compute_budget_instructions.clone());
    compute_budget_instructions.extend(reverse.compute_budget_instructions.clone());

    let mut main_instructions = forward.main_instructions.clone();
    main_instructions.extend(reverse.main_instructions.clone());

    let mut address_lookup_table_addresses = forward.address_lookup_table_addresses.clone();
    address_lookup_table_addresses.extend(reverse.address_lookup_table_addresses.clone());
    dedup_pubkeys(&mut address_lookup_table_addresses);

    let mut resolved_lookup_tables = forward.resolved_lookup_tables.clone();
    resolved_lookup_tables.extend(reverse.resolved_lookup_tables.clone());
    dedup_lookup_tables(&mut resolved_lookup_tables);

    let prioritization_fee_total = forward
        .prioritization_fee_lamports
        .unwrap_or(0)
        .saturating_add(reverse.prioritization_fee_lamports.unwrap_or(0));
    let prioritization_fee = if prioritization_fee_total > 0 {
        Some(prioritization_fee_total)
    } else {
        None
    };

    UltraFinalizedSwap {
        compute_budget_instructions,
        main_instructions,
        address_lookup_table_addresses,
        resolved_lookup_tables,
        prioritization_fee_lamports: prioritization_fee,
        compute_unit_limit: merged_limit,
    }
}

fn dedup_pubkeys(keys: &mut Vec<Pubkey>) {
    let mut seen = HashSet::new();
    keys.retain(|key| seen.insert(*key));
}

fn dedup_lookup_tables(tables: &mut Vec<AddressLookupTableAccount>) {
    let mut merged: HashMap<Pubkey, Vec<Pubkey>> = HashMap::new();
    let mut seen_addresses: HashMap<Pubkey, HashSet<Pubkey>> = HashMap::new();
    let mut order: Vec<Pubkey> = Vec::new();

    for table in std::mem::take(tables) {
        let entry = merged.entry(table.key).or_insert_with(|| {
            order.push(table.key);
            Vec::new()
        });
        let addr_set = seen_addresses
            .entry(table.key)
            .or_insert_with(HashSet::new);
        for address in table.addresses {
            if addr_set.insert(address) {
                entry.push(address);
            }
        }
    }

    *tables = order
        .into_iter()
        .filter_map(|key| merged.remove(&key).map(|addresses| AddressLookupTableAccount {
            key,
            addresses,
        }))
        .collect();
}
