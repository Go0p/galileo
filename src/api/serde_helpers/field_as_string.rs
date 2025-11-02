use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    T: ToString,
    S: Serializer,
{
    value.to_string().serialize(serializer)
}

pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: FromStr,
    T::Err: std::fmt::Debug,
    D: Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    raw.parse()
        .map_err(|err| de::Error::custom(format!("parse error: {err:?}")))
}
