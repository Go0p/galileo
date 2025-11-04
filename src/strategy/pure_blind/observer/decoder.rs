#![allow(dead_code)]

use anyhow::{Context, Result};
use serde_json::Value;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

use crate::instructions::jupiter::decoder::{self as swap_decoder, ParsedSwapAccounts};
use crate::instructions::jupiter::parser::{RouteKind, classify};
use crate::instructions::jupiter::route_v2::decode_route_v2_payload;
use crate::instructions::jupiter::types::EncodedSwap;

#[derive(Debug, Clone)]
pub struct DecodedJupiterRoute {
    pub kind: RouteKind,
    pub in_amount: u64,
    pub quoted_out_amount: u64,
    pub slippage_bps: u16,
    pub platform_fee_bps: u16,
    pub positive_slippage_bps: u16,
    pub steps: Vec<DecodedJupiterStep>,
}

#[derive(Debug, Clone)]
pub struct DecodedJupiterStep {
    pub index: usize,
    pub input_index: u8,
    pub output_index: u8,
    pub swap: EncodedSwap,
    pub variant: String,
    pub payload: Value,
    pub direction_hint: DirectionHint,
    pub accounts: Option<ParsedSwapAccounts>,
}

impl DecodedJupiterStep {
    pub fn parse_accounts(&self, accounts: &[Pubkey]) -> Option<ParsedSwapAccounts> {
        swap_decoder::parse_swap_accounts(&self.variant, accounts)
    }

    pub fn with_accounts(mut self, accounts: Option<ParsedSwapAccounts>) -> Self {
        self.accounts = accounts;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionHint {
    Known(DirectionFlag),
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectionFlag {
    Forward,
    Reverse,
}

pub fn decode_route_instruction(ix: &Instruction) -> Result<DecodedJupiterRoute> {
    let kind = classify(&ix.data);
    match kind {
        RouteKind::RouteV2 => decode_route_v2(&ix.data),
        RouteKind::SharedRouteV2 => Err(anyhow::anyhow!("shared route v2 is not yet supported")),
        RouteKind::Route => Err(anyhow::anyhow!("legacy route is not supported")),
        RouteKind::ExactRouteV2 => Err(anyhow::anyhow!("exact route v2 is not supported")),
        RouteKind::Other => Err(anyhow::anyhow!("instruction is not a Jupiter route")),
    }
}

fn decode_route_v2(data: &[u8]) -> Result<DecodedJupiterRoute> {
    use anyhow::anyhow;

    let payload = decode_route_v2_payload(data)?;
    if payload.route_plan.is_empty() {
        return Err(anyhow!("route plan empty"));
    }

    let mut steps = Vec::with_capacity(payload.route_plan.len());
    for (index, step) in payload.route_plan.iter().enumerate() {
        let (variant, fields) = extract_swap_fields(&step.swap)
            .with_context(|| format!("failed to parse swap payload for step {index}"))?;
        let direction_hint = infer_direction(&variant, &fields);
        steps.push(DecodedJupiterStep {
            index,
            input_index: step.input_index,
            output_index: step.output_index,
            swap: step.swap.clone(),
            variant,
            payload: fields,
            direction_hint,
            accounts: None,
        });
    }

    Ok(DecodedJupiterRoute {
        kind: RouteKind::RouteV2,
        in_amount: payload.in_amount,
        quoted_out_amount: payload.quoted_out_amount,
        slippage_bps: payload.slippage_bps,
        platform_fee_bps: payload.platform_fee_bps,
        positive_slippage_bps: payload.positive_slippage_bps,
        steps,
    })
}

fn extract_swap_fields(swap: &EncodedSwap) -> Result<(String, Value)> {
    let value = swap.to_variant_value()?;
    let obj = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("swap variant expected object"))?;
    let (variant, payload) = obj
        .iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("swap variant object missing discriminant entry"))?;
    Ok((variant.clone(), payload.clone()))
}

fn infer_direction(variant: &str, payload: &Value) -> DirectionHint {
    match variant {
        "Whirlpool" | "WhirlpoolSwapV2" | "Crema" | "Heaven" | "DefiTuna" | "RaydiumClmmV2" => {
            payload
                .get("a_to_b")
                .and_then(Value::as_bool)
                .map(|flag| {
                    if flag {
                        DirectionHint::Known(DirectionFlag::Forward)
                    } else {
                        DirectionHint::Known(DirectionFlag::Reverse)
                    }
                })
                .unwrap_or(DirectionHint::Unknown)
        }
        "HumidiFi" => payload
            .get("is_base_to_quote")
            .and_then(Value::as_bool)
            .map(|flag| {
                if flag {
                    DirectionHint::Known(DirectionFlag::Forward)
                } else {
                    DirectionHint::Known(DirectionFlag::Reverse)
                }
            })
            .unwrap_or(DirectionHint::Unknown),
        "SolFi" | "SolFiV2" => payload
            .get("is_quote_to_base")
            .and_then(Value::as_bool)
            .map(|flag| {
                if flag {
                    DirectionHint::Known(DirectionFlag::Reverse)
                } else {
                    DirectionHint::Known(DirectionFlag::Forward)
                }
            })
            .unwrap_or(DirectionHint::Unknown),
        "Obric" | "ObricV2" => payload
            .get("x_to_y")
            .and_then(Value::as_bool)
            .map(|flag| {
                if flag {
                    DirectionHint::Known(DirectionFlag::Forward)
                } else {
                    DirectionHint::Known(DirectionFlag::Reverse)
                }
            })
            .unwrap_or(DirectionHint::Unknown),
        "Clone" => payload
            .get("quantity_is_input")
            .and_then(Value::as_bool)
            .map(|is_input| {
                if is_input {
                    DirectionHint::Known(DirectionFlag::Forward)
                } else {
                    DirectionHint::Known(DirectionFlag::Reverse)
                }
            })
            .unwrap_or(DirectionHint::Unknown),
        _ => DirectionHint::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::jupiter::route_v2::{RouteV2Accounts, RouteV2InstructionBuilder};
    use crate::instructions::jupiter::types::{EncodedSwap, RoutePlanStepV2};
    use solana_sdk::pubkey::Pubkey;

    fn build_test_instruction(swap: EncodedSwap) -> Instruction {
        let accounts = RouteV2Accounts::with_defaults(
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        );

        let builder = RouteV2InstructionBuilder {
            accounts,
            route_plan: vec![RoutePlanStepV2 {
                swap,
                bps: 10_000,
                input_index: 0,
                output_index: 1,
            }],
            in_amount: 10,
            quoted_out_amount: 10,
            slippage_bps: 0,
            platform_fee_bps: 0,
            positive_slippage_bps: 0,
        };

        builder.build().expect("instruction")
    }

    #[test]
    fn decode_route_v2_whirlpool_step() {
        let swap = EncodedSwap::from_variant_value(serde_json::json!({
            "Whirlpool": { "a_to_b": true }
        }))
        .expect("encode");

        let instruction = build_test_instruction(swap);
        let decoded = decode_route_instruction(&instruction).expect("decode");

        assert_eq!(decoded.steps.len(), 1);
        let step = &decoded.steps[0];
        assert_eq!(step.variant, "Whirlpool");
        assert!(matches!(
            step.direction_hint,
            DirectionHint::Known(DirectionFlag::Forward)
        ));
    }

    #[test]
    fn decode_route_v2_solif_step() {
        let swap = EncodedSwap::from_variant_value(serde_json::json!({
            "SolFiV2": {
                "is_quote_to_base": true
            }
        }))
        .expect("encode");

        let instruction = build_test_instruction(swap);
        let decoded = decode_route_instruction(&instruction).expect("decode");

        assert_eq!(decoded.steps.len(), 1);
        let step = &decoded.steps[0];
        assert_eq!(step.variant, "SolFiV2");
        assert!(matches!(
            step.direction_hint,
            DirectionHint::Known(DirectionFlag::Reverse)
        ));
    }
}
