use std::fmt;
use std::str::FromStr;

use serde::de::{Error as DeError, SeqAccess, Visitor};
use serde::{Deserializer, Serializer};
use solana_sdk::hash::Hash;

/// 支持同时从 base58 字符串或字节数组解析 `Hash`。
pub fn serialize<S>(value: &Hash, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&value.to_string())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Hash, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(HashVisitor)
}

struct HashVisitor;

impl<'de> Visitor<'de> for HashVisitor {
    type Value = Hash;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a base58 string or 32-byte array representing a Solana hash")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: DeError,
    {
        Hash::from_str(value).map_err(|err| E::custom(format!("invalid hash string: {err}")))
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut bytes = Vec::with_capacity(32);
        while let Some(byte) = seq.next_element::<u8>()? {
            bytes.push(byte);
        }

        if bytes.len() != 32 {
            return Err(A::Error::custom(format!(
                "expected 32 bytes for hash, got {}",
                bytes.len()
            )));
        }

        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(Hash::new_from_array(array))
    }
}
