use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};
use serde::Deserializer;
use serde::de::{Error as DeError, Unexpected, Visitor};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

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
