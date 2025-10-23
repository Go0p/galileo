use serde::{Serialize, Serializer};

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
