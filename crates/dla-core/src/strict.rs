//! Strict reading of hand-written ledger lines.
//!
//! `LedgerLine` flattens an internally tagged enum, so serde cannot apply
//! `deny_unknown_fields`. Instead the raw document is compared with the typed
//! record serialized back: a key that vanishes in the round trip was ignored by the
//! typed reader, which means it is unknown or misspelled. Duplicate keys are caught
//! while parsing, because `serde_json::Value` would silently keep the last one.

use std::fmt;

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};

/// A JSON value parsed with duplicate object keys refused.
pub(crate) struct StrictValue(pub Value);

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct V;
        impl<'de> Visitor<'de> for V {
            type Value = Value;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("any JSON value")
            }
            fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
                Ok(Value::Bool(v))
            }
            fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
                Ok(Value::from(v))
            }
            fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
                Ok(Value::from(v))
            }
            fn visit_f64<E>(self, v: f64) -> Result<Value, E> {
                Ok(Value::from(v))
            }
            fn visit_str<E>(self, v: &str) -> Result<Value, E> {
                Ok(Value::String(v.to_owned()))
            }
            fn visit_string<E>(self, v: String) -> Result<Value, E> {
                Ok(Value::String(v))
            }
            fn visit_none<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_unit<E>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
                let mut out = Vec::new();
                while let Some(StrictValue(v)) = seq.next_element()? {
                    out.push(v);
                }
                Ok(Value::Array(out))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
                let mut out = Map::new();
                while let Some(k) = map.next_key::<String>()? {
                    if out.contains_key(&k) {
                        return Err(de::Error::custom(format!("duplicate field `{k}`")));
                    }
                    let StrictValue(v) = map.next_value()?;
                    out.insert(k, v);
                }
                Ok(Value::Object(out))
            }
        }
        d.deserialize_any(V).map(StrictValue)
    }
}

/// Parses `text` as one JSON value, refusing duplicate keys.
pub(crate) fn parse(text: &str) -> Result<Value, serde_json::Error> {
    serde_json::from_str::<StrictValue>(text).map(|v| v.0)
}

/// Paths (`finding.holdr`) of keys present in `raw` but dropped by the typed round trip
/// `back`. Explicit `null` and empty arrays are the serialized-away defaults and do not count.
pub(crate) fn unknown_fields(raw: &Value, back: &Value) -> Vec<String> {
    let mut out = Vec::new();
    walk(raw, back, "", &mut out);
    out
}

fn walk(raw: &Value, back: &Value, path: &str, out: &mut Vec<String>) {
    match (raw, back) {
        (Value::Object(r), Value::Object(b)) => {
            for (k, rv) in r {
                let here = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                match b.get(k) {
                    Some(bv) => walk(rv, bv, &here, out),
                    None => {
                        let default = rv.is_null() || rv.as_array().is_some_and(Vec::is_empty);
                        if !default {
                            out.push(here);
                        }
                    }
                }
            }
        }
        (Value::Array(r), Value::Array(b)) => {
            for (i, (rv, bv)) in r.iter().zip(b).enumerate() {
                walk(rv, bv, &format!("{path}[{i}]"), out);
            }
        }
        _ => {}
    }
}
