use std::collections::HashMap;

use anyhow::{Result, anyhow};
use borsh::{BorshSerialize, io::Write};
use once_cell::sync::Lazy;
use serde_json::Value;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

pub const JUPITER_V6_PROGRAM_ID: Pubkey = pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
pub const JUPITER_V6_EVENT_AUTHORITY: Pubkey =
    pubkey!("D8cy77BBepLMngZx6ZukaTff5hCt1HrWyKk3Hnd9oitf");

static SWAP_VARIANTS: Lazy<HashMap<String, u16>> =
    Lazy::new(|| parse_swap_variants().expect("failed to parse swap variants from jup6 idl"));

fn parse_swap_variants() -> Result<HashMap<String, u16>> {
    let idl: Value = serde_json::from_str(include_str!("../../../idls/jup6.json"))?;
    let mut mapping = HashMap::new();

    let types = idl
        .get("types")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("jup6 idl missing types array"))?;

    for ty in types {
        if ty.get("name").and_then(Value::as_str) != Some("Swap") {
            continue;
        }
        let variants = ty
            .get("type")
            .and_then(|val| val.get("variants"))
            .and_then(Value::as_array)
            .ok_or_else(|| anyhow!("jup6 idl Swap variants missing"))?;
        for (idx, variant) in variants.iter().enumerate() {
            let name = variant
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("swap variant missing name"))?;
            mapping.insert(name.to_string(), u16::try_from(idx)?);
        }
        return Ok(mapping);
    }

    Err(anyhow!("jup6 idl missing Swap type definition"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodedSwap {
    discriminant: u8,
    data: Vec<u8>,
}

impl EncodedSwap {
    pub fn new(discriminant: u8, data: Vec<u8>) -> Self {
        Self { discriminant, data }
    }

    pub fn simple(discriminant: u8) -> Self {
        Self::new(discriminant, Vec::new())
    }

    pub fn from_name<S, T>(name: S, payload: &T) -> Result<Self>
    where
        S: AsRef<str>,
        T: BorshSerialize,
    {
        let discriminant = resolve_swap_discriminant(name.as_ref())?;
        let mut data = Vec::new();
        payload.serialize(&mut data)?;
        Ok(Self::new(discriminant, data))
    }

    pub fn resolve_raw<S>(name: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        Self::from_name(name, &())
    }

    pub fn discriminant(&self) -> u8 {
        self.discriminant
    }

    #[allow(dead_code)]
    pub fn payload(&self) -> &[u8] {
        &self.data
    }
}

impl BorshSerialize for EncodedSwap {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[self.discriminant])?;
        writer.write_all(&self.data)?;
        Ok(())
    }
}

#[derive(BorshSerialize, Clone, Debug, PartialEq, Eq)]
pub struct RoutePlanStep {
    pub swap: EncodedSwap,
    pub percent: u8,
    pub input_index: u8,
    pub output_index: u8,
}

#[derive(BorshSerialize, Clone, Debug, PartialEq, Eq)]
pub struct RoutePlanStepV2 {
    pub swap: EncodedSwap,
    pub bps: u16,
    pub input_index: u8,
    pub output_index: u8,
}

pub fn resolve_swap_discriminant(name: &str) -> Result<u8> {
    let discriminant = SWAP_VARIANTS
        .get(name)
        .copied()
        .ok_or_else(|| anyhow!("unknown swap variant: {name}"))?;

    u8::try_from(discriminant).map_err(|_| anyhow!("swap discriminant overflow for {name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_variant_map_not_empty() {
        assert!(!SWAP_VARIANTS.is_empty());
    }

    #[test]
    fn encode_swap_without_payload() {
        let swap = EncodedSwap::resolve_raw("Raydium").expect("raydium");
        let mut buf = Vec::new();
        swap.serialize(&mut buf).unwrap();
        assert_eq!(swap.discriminant(), buf.first().copied().unwrap());
    }
}
