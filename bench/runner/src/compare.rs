//! `dla-bench compare`: equality of two JSON files as parsed values.

use std::{fs, path::Path};

use serde_json::Value;

use crate::Res;

/// Differences between two JSON documents as `path: description` lines.
/// Numbers compare by value, so `1` equals `1.0`; object key order is ignored.
pub fn diff(a: &Value, b: &Value) -> Vec<String> {
    let mut out = Vec::new();
    walk("$", a, b, &mut out);
    out
}

fn num_eq(a: &serde_json::Number, b: &serde_json::Number) -> bool {
    if let (Some(x), Some(y)) = (a.as_i64(), b.as_i64()) {
        return x == y;
    }
    if let (Some(x), Some(y)) = (a.as_u64(), b.as_u64()) {
        return x == y;
    }
    a.as_f64() == b.as_f64()
}

fn walk(path: &str, a: &Value, b: &Value, out: &mut Vec<String>) {
    match (a, b) {
        (Value::Object(x), Value::Object(y)) => {
            for (k, v) in x {
                let p = format!("{path}.{k}");
                match y.get(k) {
                    Some(w) => walk(&p, v, w, out),
                    None => out.push(format!("{p}: only in the first file")),
                }
            }
            for k in y.keys() {
                if !x.contains_key(k) {
                    out.push(format!("{path}.{k}: only in the second file"));
                }
            }
        }
        (Value::Array(x), Value::Array(y)) => {
            if x.len() != y.len() {
                out.push(format!("{path}: array length {} != {}", x.len(), y.len()));
            }
            for (i, (v, w)) in x.iter().zip(y).enumerate() {
                walk(&format!("{path}[{i}]"), v, w, out);
            }
        }
        (Value::Number(x), Value::Number(y)) => {
            if !num_eq(x, y) {
                out.push(format!("{path}: {x} != {y}"));
            }
        }
        _ => {
            if a != b {
                out.push(format!("{path}: {a} != {b}"));
            }
        }
    }
}

/// Reads and compares two files; the lines of differences, empty when equal.
pub fn compare_files(a: &Path, b: &Path) -> Res<Vec<String>> {
    let read = |p: &Path| -> Res<Value> {
        let t = fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
        serde_json::from_str(&t).map_err(|e| format!("{}: {e}", p.display()))
    };
    Ok(diff(&read(a)?, &read(b)?))
}
