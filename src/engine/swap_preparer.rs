use std::collections::HashSet;
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
use crate::config::{DflowSwapConfig, JupiterSwapConfig};
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
    Kamino,
    Ultra {
        rpc: Arc<RpcClient>,
        alt_cache: AltCache,
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

    pub fn for_kamino(compute_unit_price: Option<ComputeUnitPriceMode>) -> Self {
        Self {
            backend: SwapPreparerBackend::Kamino,
            compute_unit_price,
        }
    }

    pub fn for_ultra(
        rpc: Arc<RpcClient>,
        compute_unit_price: Option<ComputeUnitPriceMode>,
    ) -> Self {
        Self {
            backend: SwapPreparerBackend::Ultra {
                rpc,
                alt_cache: AltCache::new(),
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

                let response = client.swap_instructions(&request, local_ip).await?;
                SwapInstructionsVariant::Dflow(response)
            }
            (SwapPreparerBackend::Kamino, QuotePayloadVariant::Kamino(payload)) => {
                let instructions = payload.route.instructions.flatten();
                let mut compute_budget_instructions = Vec::new();
                let mut main_instructions = Vec::new();
                let mut compute_unit_limit: Option<u32> = None;
                let mut compute_unit_price: Option<u64> = None;

                for ix in instructions {
                    if ix.program_id == COMPUTE_BUDGET_PROGRAM_ID {
                        if let Some(parsed) = parse_compute_budget_instruction(&ix) {
                            match parsed {
                                ParsedComputeBudget::Limit(value) => {
                                    compute_unit_limit = Some(value)
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
                for address in &payload.route.lookup_table_accounts_bs58 {
                    let trimmed = address.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    match Pubkey::from_str(trimmed) {
                        Ok(pubkey) => lookup_table_addresses.push(pubkey),
                        Err(err) => {
                            debug!(
                                target: "engine::swap_preparer",
                                address = trimmed,
                                error = %err,
                                "解析 Kamino lookup table 地址失败，忽略"
                            );
                        }
                    }
                }

                let limit = compute_unit_limit.unwrap_or(FALLBACK_CU_LIMIT);
                if compute_unit_limit.is_none() {
                    compute_budget_instructions
                        .insert(0, ComputeBudgetInstruction::set_compute_unit_limit(limit));
                }

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
                    prioritization_fee_lamports,
                    limit,
                );
                SwapInstructionsVariant::Kamino(bundle)
            }
            (SwapPreparerBackend::Ultra { rpc, alt_cache }, QuotePayloadVariant::Ultra(_)) => {
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

                let finalized =
                    combine_ultra_swaps(forward, reverse, FALLBACK_CU_LIMIT, override_price);

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
            | (SwapPreparerBackend::Kamino, QuotePayloadVariant::Jupiter(_))
            | (SwapPreparerBackend::Kamino, QuotePayloadVariant::Dflow(_))
            | (SwapPreparerBackend::Kamino, QuotePayloadVariant::Ultra(_)) => {
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
        combined_limit,
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
        compute_unit_limit: combined_limit,
    }
}

fn dedup_pubkeys(keys: &mut Vec<Pubkey>) {
    let mut seen = HashSet::new();
    keys.retain(|key| seen.insert(*key));
}

fn dedup_lookup_tables(tables: &mut Vec<AddressLookupTableAccount>) {
    let mut seen = HashSet::new();
    tables.retain(|table| seen.insert(table.key));
}
