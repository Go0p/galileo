use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ToString,
    S: Serializer,
{
    match value {
        Some(inner) => inner.to_string().serialize(serializer),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    D: Deserializer<'de>,
    <T as FromStr>::Err: std::fmt::Debug,
{
    let raw: Option<String> = Option::deserialize(deserializer)?;
    match raw {
        Some(v) => v
            .parse()
            .map(Some)
            .map_err(|err| de::Error::custom(format!("parse error: {err:?}"))),
        None => Ok(None),
    }
}
