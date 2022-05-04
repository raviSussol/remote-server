use std::{collections::BTreeMap, fmt};

use serde::{Serialize, Serializer};
use serde_json::to_string as to_json_string;

pub type Object = BTreeMap<String, CanonicalJsonValue>;

// This code is mostly copied from
// https://github.com/ruma/ruma/blob/main/crates/ruma-serde/src/canonical_json/value.rs
// Only difference is that it also works with floats

#[derive(Clone, Eq, PartialEq)]
pub enum CanonicalJsonValue {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<CanonicalJsonValue>),
    Object(Object),
}

impl From<serde_json::Value> for CanonicalJsonValue {
    fn from(val: serde_json::Value) -> Self {
        match val {
            serde_json::Value::Bool(b) => Self::Bool(b),
            serde_json::Value::Number(num) => Self::Number(num),
            serde_json::Value::Array(vec) => {
                Self::Array(vec.into_iter().map(Into::into).collect::<Vec<_>>())
            }
            serde_json::Value::String(string) => Self::String(string),
            serde_json::Value::Object(obj) => Self::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, v.into()))
                    .collect::<Object>(),
            ),
            serde_json::Value::Null => Self::Null,
        }
    }
}

impl Serialize for CanonicalJsonValue {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Null => serializer.serialize_unit(),
            Self::Bool(b) => serializer.serialize_bool(*b),
            Self::Number(n) => n.serialize(serializer),
            Self::String(s) => serializer.serialize_str(s),
            Self::Array(v) => v.serialize(serializer),
            Self::Object(m) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(m.len()))?;
                for (k, v) in m {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}

impl fmt::Display for CanonicalJsonValue {
    /// Display this value as a string.
    ///
    /// This `Display` implementation is intentionally unaffected by any formatting parameters,
    /// because adding extra whitespace or otherwise pretty-printing it would make it not the
    /// canonical form anymore.
    ///
    /// If you want to pretty-print a `CanonicalJsonValue` for debugging purposes, use
    /// one of `serde_json::{to_string_pretty, to_vec_pretty, to_writer_pretty}`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", to_json_string(&self).map_err(|_| fmt::Error)?)
    }
}
