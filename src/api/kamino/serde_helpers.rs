use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use serde::Deserializer;
use serde::de::{Error as DeError, Unexpected, Visitor};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use serde::Deserialize;
use serde_json::Value;

pub fn parse_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    struct U64Visitor;

    impl<'de> Visitor<'de> for U64Visitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("u64 数值或对应的字符串")
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            if value < 0 {
                return Err(DeError::invalid_value(Unexpected::Signed(value), &self));
            }
            Ok(value as u64)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            value
                .parse::<u64>()
                .map_err(|_| DeError::invalid_value(Unexpected::Str(value), &self))
        }
    }

    deserializer.deserialize_any(U64Visitor)
}

pub fn parse_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    struct PubkeyVisitor;

    impl<'de> Visitor<'de> for PubkeyVisitor {
        type Value = Pubkey;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("有效的 base58 公钥字符串")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            Pubkey::from_str(value)
                .map_err(|_| DeError::invalid_value(Unexpected::Str(value), &self))
        }
    }

    deserializer.deserialize_any(PubkeyVisitor)
}

pub fn parse_base64<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    struct Base64Visitor;

    impl<'de> Visitor<'de> for Base64Visitor {
        type Value = Vec<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("base64 编码的字节串")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: DeError,
        {
            BASE64_STANDARD
                .decode(value.as_bytes())
                .map_err(|_| DeError::invalid_value(Unexpected::Str(value), &self))
        }
    }

    deserializer.deserialize_str(Base64Visitor)
}

use crate::api::kamino::quote::LookupTableEntry;

pub fn parse_lookup_table_accounts<'de, D>(deserializer: D) -> Result<Vec<LookupTableEntry>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: Option<Value> = Option::deserialize(deserializer)?;
    let mut result: Vec<LookupTableEntry> = Vec::new();
    if let Some(value) = raw {
        collect_lookup_table_entries(value, &mut result);
    }
    dedup_entries(&mut result);
    Ok(result)
}

fn collect_lookup_table_entries(value: Value, acc: &mut Vec<LookupTableEntry>) {
    match value {
        Value::Null => {}
        Value::String(s) => {
            if let Some(key) = non_empty(s) {
                acc.push(LookupTableEntry {
                    key,
                    addresses: Vec::new(),
                });
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_lookup_table_entries(item, acc);
            }
        }
        Value::Object(mut map) => {
            if let Some(entry) = extract_entry(&mut map) {
                acc.push(entry);
            }
            for (_, value) in map {
                collect_lookup_table_entries(value, acc);
            }
        }
        _ => {}
    }
}

fn extract_entry(map: &mut serde_json::Map<String, Value>) -> Option<LookupTableEntry> {
    let key_sources = [
        "key",
        "address",
        "lookupTable",
        "lookupTableAddress",
        "lookupTableAccountAddress",
        "lookupTablePubkey",
        "pubkey",
    ];
    let mut key: Option<String> = None;
    for source in key_sources {
        if let Some(Value::String(value)) = map.remove(source) {
            if let Some(trimmed) = non_empty(value) {
                key = Some(trimmed);
                break;
            }
        }
    }
    let key = key?;

    let mut addresses: Vec<String> = Vec::new();
    if let Some(Value::Object(mut state)) = map.remove("state") {
        if let Some(Value::Array(items)) = state.remove("addresses") {
            for item in items {
                if let Value::String(addr) = item {
                    if let Some(trimmed) = non_empty(addr) {
                        addresses.push(trimmed);
                    }
                }
            }
        }
    }

    if let Some(Value::Array(items)) = map.remove("addresses") {
        for item in items {
            match item {
                Value::String(value) => {
                    if let Some(trimmed) = non_empty(value) {
                        addresses.push(trimmed);
                    }
                }
                _ => {}
            }
        }
    }

    dedup_strings(&mut addresses);
    Some(LookupTableEntry { key, addresses })
}

fn non_empty(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn dedup_entries(entries: &mut Vec<LookupTableEntry>) {
    let mut seen = std::collections::HashSet::new();
    entries.retain(|entry| seen.insert(entry.key.clone()));
}

fn dedup_strings(values: &mut Vec<String>) {
    let mut seen = std::collections::HashSet::new();
    values.retain(|value| seen.insert(value.clone()));
}
