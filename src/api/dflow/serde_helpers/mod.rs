pub use crate::api::serde_helpers::field_as_string;

pub mod option_field_as_string {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
    use std::fmt::Debug;
    use std::str::FromStr;

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
        <T as FromStr>::Err: Debug,
        D: Deserializer<'de>,
    {
        let raw = Option::<String>::deserialize(deserializer)?;
        match raw {
            Some(value) => value
                .parse()
                .map(Some)
                .map_err(|err| de::Error::custom(format!("parse error: {err:?}"))),
            None => Ok(None),
        }
    }
}

pub mod decimal_from_string {
    use rust_decimal::Decimal;
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

    pub fn serialize<S>(value: &Decimal, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.to_string().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse()
            .map_err(|err| de::Error::custom(format!("decimal parse error: {err:?}")))
    }
}
