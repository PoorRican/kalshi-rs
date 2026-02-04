use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::fmt;

/// Serialize Option<Vec<T>> as a single comma-separated query param.
pub fn serialize_csv_opt<T, S>(value: &Option<Vec<T>>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: fmt::Display,
    S: Serializer,
{
    match value {
        None => serializer.serialize_none(),
        Some(items) => {
            let s = items
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(",");
            serializer.serialize_str(&s)
        }
    }
}

/// Fixed-point dollar string (e.g. "0.5600").
pub type FixedPointDollars = String;

/// Fixed-point contract count string (e.g. "10.00").
pub type FixedPointCount = String;

/// Typed wrapper for arbitrary JSON payloads.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AnyJson(pub Value);

impl AnyJson {
    pub fn as_value(&self) -> &Value {
        &self.0
    }
}

impl From<Value> for AnyJson {
    fn from(value: Value) -> Self {
        Self(value)
    }
}

impl Serialize for AnyJson {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for AnyJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Self(Value::deserialize(deserializer)?))
    }
}
