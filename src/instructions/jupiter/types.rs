use std::collections::HashMap;
use std::io::{Cursor, Read};

use anyhow::{Result, anyhow, bail};
use borsh::{BorshSerialize, io::Write};
use once_cell::sync::Lazy;
use serde::de::Error as DeError;
use serde::ser::Error as SerError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

pub const JUPITER_V6_PROGRAM_ID: Pubkey = pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
pub const JUPITER_V6_EVENT_AUTHORITY: Pubkey =
    pubkey!("D8cy77BBepLMngZx6ZukaTff5hCt1HrWyKk3Hnd9oitf");

#[derive(Clone)]
struct VariantSpec {
    discriminant: u8,
    fields: Vec<FieldSpec>,
}

#[derive(Clone)]
struct FieldSpec {
    name: String,
    ty: FieldType,
}

#[derive(Clone)]
enum FieldType {
    Bool,
    U8,
    U32,
    U64,
    Side,
    RemainingAccountsInfo,
    OptionalRemainingAccountsInfo,
}

static SWAP_VARIANTS: Lazy<HashMap<String, VariantSpec>> =
    Lazy::new(|| parse_swap_variants().expect("failed to parse swap variants from jup6 idl"));

static SWAP_DISCRIMINANTS: Lazy<HashMap<u8, String>> = Lazy::new(|| {
    SWAP_VARIANTS
        .iter()
        .map(|(name, spec)| (spec.discriminant, name.clone()))
        .collect()
});

fn parse_swap_variants() -> Result<HashMap<String, VariantSpec>> {
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
            let mut fields = Vec::new();
            if let Some(field_list) = variant.get("fields").and_then(Value::as_array) {
                for field in field_list {
                    let field_name = field
                        .get("name")
                        .and_then(Value::as_str)
                        .ok_or_else(|| anyhow!("swap variant field missing name"))?;
                    let field_ty = field
                        .get("type")
                        .ok_or_else(|| anyhow!("swap variant field missing type"))?;
                    fields.push(FieldSpec {
                        name: field_name.to_string(),
                        ty: parse_field_type(field_ty)?,
                    });
                }
            }
            let discriminant = u8::try_from(idx)
                .map_err(|_| anyhow!("swap discriminant overflow for variant {name}"))?;
            mapping.insert(
                name.to_string(),
                VariantSpec {
                    discriminant,
                    fields,
                },
            );
        }
        return Ok(mapping);
    }

    Err(anyhow!("jup6 idl missing Swap type definition"))
}

fn parse_field_type(value: &Value) -> Result<FieldType> {
    if let Some(kind) = value.as_str() {
        return match kind {
            "bool" => Ok(FieldType::Bool),
            "u8" => Ok(FieldType::U8),
            "u32" => Ok(FieldType::U32),
            "u64" => Ok(FieldType::U64),
            other => Err(anyhow!("unsupported primitive swap field type: {other}")),
        };
    }

    if let Some(defined) = value.get("defined") {
        let name = defined
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("defined swap field missing name"))?;
        return match name {
            "Side" => Ok(FieldType::Side),
            "RemainingAccountsInfo" => Ok(FieldType::RemainingAccountsInfo),
            other => Err(anyhow!("unsupported defined swap field type: {other}")),
        };
    }

    if let Some(option) = value.get("option") {
        let defined = option
            .get("defined")
            .and_then(Value::as_object)
            .ok_or_else(|| anyhow!("unsupported option field definition"))?;
        let name = defined
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("option defined type missing name"))?;
        return match name {
            "RemainingAccountsInfo" => Ok(FieldType::OptionalRemainingAccountsInfo),
            other => Err(anyhow!(
                "unsupported optional defined swap field type: {other}"
            )),
        };
    }

    Err(anyhow!("unsupported swap field type: {value:?}"))
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

    #[cfg(test)]
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

    #[cfg(test)]
    pub fn resolve_raw<S>(name: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        Self::from_name(name, &())
    }

    #[cfg(test)]
    pub fn discriminant(&self) -> u8 {
        self.discriminant
    }

    pub fn variant(&self) -> Result<&str> {
        SWAP_DISCRIMINANTS
            .get(&self.discriminant)
            .map(|s| s.as_str())
            .ok_or_else(|| anyhow!("unknown swap discriminant {}", self.discriminant))
    }

    pub fn from_variant_value(value: Value) -> Result<Self> {
        let (variant, fields_value) = extract_variant_entry(value)?;
        let spec = SWAP_VARIANTS
            .get(variant.as_str())
            .ok_or_else(|| anyhow!("unknown swap variant: {variant}"))?;
        let data = encode_variant_fields(spec, fields_value)?;
        Ok(Self {
            discriminant: spec.discriminant,
            data,
        })
    }

    pub fn to_variant_value(&self) -> Result<Value> {
        let name = self.variant()?.to_string();
        let spec = SWAP_VARIANTS
            .get(&name)
            .ok_or_else(|| anyhow!("missing spec for swap variant: {name}"))?;
        let fields = decode_variant_fields(spec, &self.data)?;
        let mut map = serde_json::Map::new();
        map.insert(name, fields);
        Ok(Value::Object(map))
    }
}

impl BorshSerialize for EncodedSwap {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_all(&[self.discriminant])?;
        writer.write_all(&self.data)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, BorshSerialize, Clone, Debug, PartialEq, Eq)]
pub struct RoutePlanStep {
    pub swap: EncodedSwap,
    pub percent: u8,
    pub input_index: u8,
    pub output_index: u8,
}

#[derive(Serialize, Deserialize, BorshSerialize, Clone, Debug, PartialEq, Eq)]
pub struct RoutePlanStepV2 {
    pub swap: EncodedSwap,
    pub bps: u16,
    pub input_index: u8,
    pub output_index: u8,
}

pub fn resolve_swap_discriminant(name: &str) -> Result<u8> {
    SWAP_VARIANTS
        .get(name)
        .map(|spec| spec.discriminant)
        .ok_or_else(|| anyhow!("unknown swap variant: {name}"))
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
        borsh::BorshSerialize::serialize(&swap, &mut buf).unwrap();
        assert_eq!(swap.discriminant(), buf.first().copied().unwrap());
    }

    #[test]
    fn encode_decode_humidifi_swap() {
        let value = serde_json::json!({
            "HumidiFi": {
                "swap_id": "14299191219138278284",
                "is_base_to_quote": true
            }
        });
        let encoded = EncodedSwap::from_variant_value(value.clone()).expect("encode");
        let roundtrip = encoded.to_variant_value().expect("decode");
        assert_eq!(value, roundtrip);
    }
}

impl Serialize for EncodedSwap {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_variant_value()
            .map_err(SerError::custom)?
            .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for EncodedSwap {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        EncodedSwap::from_variant_value(value).map_err(DeError::custom)
    }
}

fn extract_variant_entry(value: Value) -> Result<(String, Value)> {
    let obj = value
        .as_object()
        .ok_or_else(|| anyhow!("swap variant must be an object"))?;
    let mut iter = obj.iter();
    let (name, value) = iter
        .next()
        .ok_or_else(|| anyhow!("swap variant object is empty"))?;
    if iter.next().is_some() {
        bail!("swap variant object must contain exactly one entry");
    }
    Ok((name.clone(), value.clone()))
}

fn encode_variant_fields(spec: &VariantSpec, value: Value) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut map = value.as_object().cloned().unwrap_or_default();
    for field in &spec.fields {
        let field_value = map.remove(&field.name);
        encode_field(field, field_value, &mut buf)?;
    }
    if !map.is_empty() {
        let extras: Vec<_> = map.keys().cloned().collect();
        bail!("unexpected fields for swap variant: {:?}", extras);
    }
    Ok(buf)
}

fn encode_field(field: &FieldSpec, value: Option<Value>, buf: &mut Vec<u8>) -> Result<()> {
    match field.ty {
        FieldType::Bool => {
            let val = value
                .ok_or_else(|| anyhow!("field `{}` missing for bool", field.name))?
                .as_bool()
                .ok_or_else(|| anyhow!("field `{}` expected bool", field.name))?;
            buf.push(if val { 1 } else { 0 });
        }
        FieldType::U8 => {
            let val = u8::try_from(parse_u64(value, &field.name)?)
                .map_err(|_| anyhow!("field `{}` out of range for u8", field.name))?;
            buf.push(val);
        }
        FieldType::U32 => {
            let val = u32::try_from(parse_u64(value, &field.name)?)
                .map_err(|_| anyhow!("field `{}` out of range for u32", field.name))?;
            buf.extend_from_slice(&val.to_le_bytes());
        }
        FieldType::U64 => {
            let val = parse_u64(value, &field.name)?;
            buf.extend_from_slice(&val.to_le_bytes());
        }
        FieldType::Side => {
            let raw = value.ok_or_else(|| anyhow!("field `{}` missing", field.name))?;
            let disc = match raw {
                Value::String(s) => match s.as_str() {
                    "Bid" | "bid" => 0u8,
                    "Ask" | "ask" => 1u8,
                    other => bail!("unsupported side variant `{other}`"),
                },
                other => bail!("field `{}` expected string, got {other}", field.name),
            };
            buf.push(disc);
        }
        FieldType::RemainingAccountsInfo => {
            let info = value.ok_or_else(|| anyhow!("field `{}` missing", field.name))?;
            encode_remaining_accounts_info(&info, buf)?;
        }
        FieldType::OptionalRemainingAccountsInfo => {
            if let Some(val) = value {
                if val.is_null() {
                    buf.push(0);
                } else {
                    buf.push(1);
                    encode_remaining_accounts_info(&val, buf)?;
                }
            } else {
                buf.push(0);
            }
        }
    }
    Ok(())
}

fn encode_remaining_accounts_info(value: &Value, buf: &mut Vec<u8>) -> Result<()> {
    let info: RemainingAccountsInfoValue =
        serde_json::from_value(value.clone()).map_err(|err| anyhow!(err))?;
    let len = u32::try_from(info.slices.len())?;
    buf.extend_from_slice(&len.to_le_bytes());
    for slice in info.slices {
        buf.push(slice.accounts_type);
        buf.push(slice.length);
    }
    Ok(())
}

fn parse_u64(value: Option<Value>, field: &str) -> Result<u64> {
    let val = value.ok_or_else(|| anyhow!("field `{field}` missing"))?;
    if let Some(number) = val.as_u64() {
        return Ok(number);
    }
    if let Some(text) = val.as_str() {
        return text
            .parse::<u64>()
            .map_err(|err| anyhow!("field `{field}` parse error: {err}"));
    }
    Err(anyhow!("field `{field}` expected number or string"))
}

fn decode_variant_fields(spec: &VariantSpec, data: &[u8]) -> Result<Value> {
    let mut cursor = Cursor::new(data);
    let mut map = serde_json::Map::new();
    for field in &spec.fields {
        let value = decode_field(field, &mut cursor)?;
        map.insert(field.name.clone(), value);
    }
    Ok(Value::Object(map))
}

fn decode_field(field: &FieldSpec, cursor: &mut Cursor<&[u8]>) -> Result<Value> {
    match field.ty {
        FieldType::Bool => {
            let mut buf = [0u8; 1];
            cursor.read_exact(&mut buf)?;
            Ok(Value::Bool(buf[0] != 0))
        }
        FieldType::U8 => {
            let mut buf = [0u8; 1];
            cursor.read_exact(&mut buf)?;
            Ok(Value::Number(buf[0].into()))
        }
        FieldType::U32 => {
            let mut buf = [0u8; 4];
            cursor.read_exact(&mut buf)?;
            Ok(Value::Number(u32::from_le_bytes(buf).into()))
        }
        FieldType::U64 => {
            let mut buf = [0u8; 8];
            cursor.read_exact(&mut buf)?;
            Ok(Value::String(u64::from_le_bytes(buf).to_string()))
        }
        FieldType::Side => {
            let mut buf = [0u8; 1];
            cursor.read_exact(&mut buf)?;
            let value = match buf[0] {
                0 => "Bid",
                1 => "Ask",
                other => bail!("unsupported side discriminant {other}"),
            };
            Ok(Value::String(value.to_string()))
        }
        FieldType::RemainingAccountsInfo => decode_remaining_accounts_info(cursor),
        FieldType::OptionalRemainingAccountsInfo => {
            let mut flag = [0u8; 1];
            cursor.read_exact(&mut flag)?;
            if flag[0] == 0 {
                Ok(Value::Null)
            } else {
                decode_remaining_accounts_info(cursor)
            }
        }
    }
}

fn decode_remaining_accounts_info(cursor: &mut Cursor<&[u8]>) -> Result<Value> {
    let mut len_buf = [0u8; 4];
    cursor.read_exact(&mut len_buf)?;
    let len = u32::from_le_bytes(len_buf);
    let mut slices = Vec::with_capacity(len as usize);
    for _ in 0..len {
        let mut buf = [0u8; 1];
        cursor.read_exact(&mut buf)?;
        let accounts_type = buf[0];
        cursor.read_exact(&mut buf)?;
        let length = buf[0];
        slices.push(RemainingAccountsSliceValue {
            accounts_type,
            length,
        });
    }
    let info = RemainingAccountsInfoValue { slices };
    serde_json::to_value(info).map_err(|err| anyhow!(err))
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
struct RemainingAccountsInfoValue {
    slices: Vec<RemainingAccountsSliceValue>,
}

#[derive(Serialize, Deserialize)]
struct RemainingAccountsSliceValue {
    #[serde(rename = "accounts_type", alias = "accountsType")]
    accounts_type: u8,
    length: u8,
}
