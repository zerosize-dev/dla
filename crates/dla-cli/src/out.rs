//! Bounded single-object JSON output.

use dla_core::MAX_OUTPUT_BYTES;
use serde_json::{Map, Value, json};

/// Largest JSON body, leaving one byte for the trailing newline.
const LIMIT: usize = MAX_OUTPUT_BYTES - 1;

/// (max string chars, max array items) tried in order.
const LEVELS: [(usize, usize); 5] = [(400, 8), (200, 4), (100, 2), (60, 1), (30, 1)];

fn shrink(v: &Value, max_str: usize, max_arr: usize, changed: &mut bool) -> Value {
    match v {
        Value::String(s) if s.chars().count() > max_str => {
            *changed = true;
            let mut t: String = s.chars().take(max_str).collect();
            t.push_str("...");
            Value::String(t)
        }
        Value::Array(a) => {
            if a.len() > max_arr {
                *changed = true;
            }
            Value::Array(
                a.iter()
                    .take(max_arr)
                    .map(|x| shrink(x, max_str, max_arr, changed))
                    .collect(),
            )
        }
        Value::Object(o) => Value::Object(
            o.iter()
                .map(|(k, x)| (k.clone(), shrink(x, max_str, max_arr, changed)))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn size(v: &Value) -> usize {
    serde_json::to_string(v).map_or(usize::MAX, |s| s.len())
}

/// Returns `v` unchanged if it fits, else a truncated copy with `"truncated": true`.
pub fn bound(v: Value) -> Value {
    if size(&v) <= LIMIT {
        return v;
    }
    for (max_str, max_arr) in LEVELS {
        let mut changed = false;
        let mut s = shrink(&v, max_str, max_arr, &mut changed);
        if let Value::Object(o) = &mut s {
            o.insert("truncated".to_owned(), Value::Bool(true));
        }
        if size(&s) <= LIMIT {
            return s;
        }
    }
    // Last resort: keep only the identifying keys.
    let mut keep = Map::new();
    if let Value::Object(o) = &v {
        for k in ["ok", "id", "run_id", "record"] {
            if let Some(x) = o.get(k) {
                keep.insert(k.to_owned(), shrink(x, 60, 1, &mut false));
            }
        }
        if let Some(Value::Object(e)) = o.get("error") {
            let code = e.get("code").cloned().unwrap_or(Value::Null);
            keep.insert("error".to_owned(), json!({ "code": code }));
        }
    }
    keep.insert("truncated".to_owned(), Value::Bool(true));
    Value::Object(keep)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_values_pass_through() {
        let v = json!({"ok": true, "id": "F1"});
        assert_eq!(bound(v.clone()), v);
    }

    #[test]
    fn long_strings_and_arrays_are_truncated() {
        let v = json!({
            "ok": false,
            "error": {"code": "x", "message": "m".repeat(5000), "missing": "a", "fix": "b"},
            "list": vec!["item"; 400],
        });
        let b = bound(v);
        let text = serde_json::to_string(&b).unwrap();
        assert!(text.len() < MAX_OUTPUT_BYTES, "{}", text.len());
        assert_eq!(b["truncated"], true);
        assert_eq!(b["ok"], false);
    }

    #[test]
    fn hopeless_values_fall_back_to_identifying_keys() {
        let mut o = Map::new();
        o.insert("ok".into(), json!(true));
        o.insert("id".into(), json!("F1"));
        for i in 0..400 {
            o.insert(format!("key{i}"), json!(i));
        }
        let b = bound(Value::Object(o));
        assert!(serde_json::to_string(&b).unwrap().len() < MAX_OUTPUT_BYTES);
        assert_eq!(b["id"], "F1");
        assert_eq!(b["truncated"], true);
    }
}
