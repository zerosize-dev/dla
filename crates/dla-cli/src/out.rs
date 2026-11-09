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

/// Top-level keys that carry pointers and verdicts. They are never dropped, and their
/// values are cut only as a last step (hashes and paths stay whole): `ok` and `evidence`
/// say what happened, `raw`, `summaries`, `artifacts` and `files` say where the full
/// output is.
const PROTECTED: [&str; 8] = [
    "ok",
    "evidence",
    "raw",
    "summaries",
    "artifacts",
    "files",
    "id",
    "run_id",
];

/// Longest string kept whole in a protected value.
const PROTECTED_STR: usize = 160;

fn protected_copy(v: &Value, max_str: usize, max_arr: usize) -> Value {
    shrink(v, max_str, max_arr, &mut false)
}

/// Returns `v` unchanged if it fits, else a truncated copy with `"truncated": true`.
pub fn bound(v: Value) -> Value {
    if size(&v) <= LIMIT {
        return v;
    }
    for (max_str, max_arr) in LEVELS {
        let mut changed = false;
        let mut s = shrink(&v, max_str, max_arr, &mut changed);
        // The pointers keep their strings whole at every level (an array of evidence ids
        // may still be cut).
        if let (Value::Object(o), Value::Object(orig)) = (&mut s, &v) {
            for k in PROTECTED {
                if let Some(x) = orig.get(k) {
                    o.insert(
                        k.to_owned(),
                        protected_copy(x, PROTECTED_STR.max(max_str), max_arr.max(32)),
                    );
                }
            }
            o.insert("truncated".to_owned(), Value::Bool(true));
        }
        if size(&s) <= LIMIT {
            return s;
        }
    }
    // Last resort: keep only the pointers and the verdict, whole while they fit.
    for (max_str, max_arr) in [(PROTECTED_STR, 32), (100, 16), (60, 8), (60, 4)] {
        let mut keep = Map::new();
        if let Value::Object(o) = &v {
            for k in PROTECTED.iter().chain(&["record", "command"]) {
                if let Some(x) = o.get(*k) {
                    keep.insert((*k).to_owned(), protected_copy(x, max_str, max_arr));
                }
            }
            if let Some(Value::Object(e)) = o.get("error") {
                let code = e.get("code").cloned().unwrap_or(Value::Null);
                keep.insert("error".to_owned(), json!({ "code": code }));
            }
        }
        keep.insert("truncated".to_owned(), Value::Bool(true));
        let s = Value::Object(keep);
        if size(&s) <= LIMIT {
            return s;
        }
    }
    json!({"truncated": true})
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

    #[test]
    fn the_last_resort_keeps_raw_evidence_ok_and_the_pointers() {
        let mut o = Map::new();
        o.insert("ok".into(), json!(true));
        o.insert("evidence".into(), json!(["E1", "E2", "E3"]));
        let sha = "a".repeat(64);
        o.insert(
            "raw".into(),
            json!({"path": "summaries/x-12345678.json", "sha256": sha}),
        );
        o.insert("summaries".into(), json!(".dla/r1/summaries/"));
        o.insert("artifacts".into(), json!([".dla/r1/histograms/H1.json"]));
        o.insert("files".into(), json!(["a", "b"]));
        for i in 0..400 {
            o.insert(format!("key{i}"), json!(i));
        }
        let b = bound(Value::Object(o));
        assert!(serde_json::to_string(&b).unwrap().len() < MAX_OUTPUT_BYTES);
        assert_eq!(b["truncated"], true);
        assert_eq!(b["ok"], true);
        assert_eq!(b["evidence"], json!(["E1", "E2", "E3"]));
        assert_eq!(b["raw"]["sha256"], "a".repeat(64));
        assert_eq!(b["summaries"], ".dla/r1/summaries/");
        assert!(b["artifacts"].is_array() && b["files"].is_array());
    }

    #[test]
    fn shrinking_a_long_value_keeps_the_raw_hash_whole() {
        let v = json!({
            "ok": true,
            "raw": {"path": "summaries/x.json", "sha256": "b".repeat(64)},
            "evidence": ["E1"],
            "rows": vec!["a long text ".repeat(20); 40],
        });
        let b = bound(v);
        assert!(serde_json::to_string(&b).unwrap().len() < MAX_OUTPUT_BYTES);
        assert_eq!(b["raw"]["sha256"], "b".repeat(64));
        assert_eq!(b["evidence"], json!(["E1"]));
    }
}
