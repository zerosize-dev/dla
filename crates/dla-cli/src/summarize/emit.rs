//! The shared output builder: raw files with their hash, ids under `histograms/`,
//! `allocs/` and `nonheap/`, bounded rows, and the evidence record with its `signal`.

use std::path::PathBuf;

use dla_core::{
    Error, Ledger, MAX_RESULT_BYTES, Refusal,
    evidence::Signal,
    fsutil::{reserve_numbered, to_pretty_json, write_atomic},
    toolctx::{EvidenceSpec, ToolCtx},
};
use serde::Serialize;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use super::Recorded;

/// Bytes of an evidence result an adapter may use before the `signal` is added.
pub const RESULT_BUDGET: usize = 1680;

/// A pointer to a full output: the path relative to the run directory and its sha256.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawRef {
    /// Relative to the run directory, `/` separators.
    pub rel: String,
    /// SHA-256 of the file's bytes.
    pub sha256: String,
}

impl RawRef {
    /// `{"path", "sha256"}`.
    pub fn to_json(&self) -> Value {
        json!({"path": self.rel, "sha256": self.sha256})
    }
}

/// SHA-256 of bytes, lowercase hex.
/// A delta as a JSON number: counts come from files and can be anything up to `u64::MAX`,
/// and `serde_json` refuses an `i128` outside `i64`/`u64`, so the value is clamped.
pub fn jn(d: i128) -> i64 {
    i64::try_from(d).unwrap_or(if d < 0 { i64::MIN } else { i64::MAX })
}

pub fn sha_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// One hash for several sources: the SHA-256 of their sorted hashes. A single source
/// keeps its own hash.
pub fn combined_sha(parts: &[&str]) -> String {
    match parts {
        [one] => (*one).to_owned(),
        _ => {
            let mut v: Vec<&str> = parts.to_vec();
            v.sort_unstable();
            sha_hex(v.join("\n").as_bytes())
        }
    }
}

/// Writes `value` as pretty JSON to `<run>/<rel>` and returns its pointer.
pub fn write_raw(tc: &ToolCtx, rel: &str, value: &impl Serialize) -> Result<RawRef, Refusal> {
    let body = to_pretty_json(value);
    let path = tc.run_dir.join(rel);
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| Error::io(dir, e).to_refusal())?;
    }
    write_atomic(&path, body.as_bytes()).map_err(|e| e.to_refusal())?;
    Ok(RawRef {
        rel: rel.to_owned(),
        sha256: sha_hex(body.as_bytes()),
    })
}

/// Reserves `<prefix><n>` under `<run>/<dir>/` (an empty `<prefix><n>.json` placeholder)
/// and returns the id and the relative path of the file.
pub fn next_id(tc: &ToolCtx, dir: &str, prefix: char) -> Result<(String, String), Refusal> {
    let base: PathBuf = tc.run_dir.join(dir);
    let (n, _) = reserve_numbered(&base, prefix, ".json", false).map_err(|e| e.to_refusal())?;
    Ok((format!("{prefix}{n}"), format!("{dir}/{prefix}{n}.json")))
}

/// `histograms/H<n>.json`.
pub fn next_histogram(tc: &ToolCtx) -> Result<(String, String), Refusal> {
    next_id(tc, "histograms", 'H')
}

/// `allocs/A<n>.json` (not to be confused with `alloc/A<n>.json`, an alloc-test report).
pub fn next_alloc(tc: &ToolCtx) -> Result<(String, String), Refusal> {
    next_id(tc, "allocs", 'A')
}

/// `nonheap/N<n>.json`.
pub fn next_nonheap(tc: &ToolCtx) -> Result<(String, String), Refusal> {
    next_id(tc, "nonheap", 'N')
}

/// A file stem cleaned for use in a raw file name: ASCII alphanumerics and `._-`, at
/// most 40 characters.
pub fn clean_stem(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .take(40)
        .collect()
}

/// A bounded list: rows are added while the serialized object stays within `limit`
/// bytes. What does not fit is counted in `more`.
pub struct Bounded {
    base: Map<String, Value>,
    key: &'static str,
    rows: Vec<Value>,
    more: usize,
    limit: usize,
}

impl Bounded {
    /// A list under `key` in the object `base`, at most `limit` serialized bytes.
    pub fn new(base: Map<String, Value>, key: &'static str, limit: usize) -> Self {
        Self {
            base,
            key,
            rows: Vec::new(),
            more: 0,
            limit,
        }
    }

    fn size_with(&self, rows: &[Value], more: usize) -> usize {
        let mut m = self.base.clone();
        m.insert(self.key.to_owned(), Value::Array(rows.to_vec()));
        if more > 0 {
            m.insert("more".to_owned(), json!(more));
        }
        serde_json::to_string(&Value::Object(m)).map_or(usize::MAX, |s| s.len())
    }

    /// Adds a row if it fits (and every earlier row did); else counts it as not shown.
    pub fn push(&mut self, row: Value) {
        if self.more == 0 {
            let mut trial = self.rows.clone();
            trial.push(row);
            // Reserve room for the `more` counter.
            if self.size_with(&trial, 1000) <= self.limit {
                self.rows = trial;
                return;
            }
        }
        self.more += 1;
    }

    /// The object with its rows, and `more` when rows were left out.
    pub fn finish(self) -> Value {
        let mut m = self.base;
        m.insert(self.key.to_owned(), Value::Array(self.rows));
        if self.more > 0 {
            m.insert("more".to_owned(), json!(self.more));
        }
        Value::Object(m)
    }
}

/// Records one evidence: the `signal` goes into the result, `raw` is the full output
/// (relative to the run directory), and the result is held to the 2048-byte bound.
pub fn record(
    tc: &ToolCtx,
    ledger: &mut Ledger,
    mut spec: EvidenceSpec,
    signal: &Signal,
    raw: &RawRef,
) -> Result<Recorded, Refusal> {
    if let Value::Object(m) = &mut spec.result {
        m.insert("signal".to_owned(), signal.to_value());
    }
    let size = dla_core::evidence::result_size(&spec.result);
    if size > MAX_RESULT_BYTES {
        return Err(Refusal::new(
            dla_core::error::codes::RESULT_TOO_LARGE,
            format!(
                "a {} summary came to {size} bytes, over the {MAX_RESULT_BYTES}-byte bound",
                spec.kind
            ),
            format!("a result of at most {MAX_RESULT_BYTES} bytes"),
            "report this as a dla bug: the summary builder should have cut the rows",
        ));
    }
    spec.raw = Some(raw.rel.clone());
    let kind = spec.kind;
    let authority = spec.authority;
    let id = tc.add_evidence(ledger, spec)?;
    Ok(Recorded {
        id,
        kind,
        authority,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_rows_stop_at_the_limit_and_count_the_rest() {
        let mut b = Bounded::new(Map::new(), "top", 200);
        for i in 0..50 {
            b.push(json!({"type": format!("pkg.Type{i}"), "d": i}));
        }
        let v = b.finish();
        let text = serde_json::to_string(&v).unwrap();
        assert!(text.len() <= 200, "{}", text.len());
        let shown = v["top"].as_array().unwrap().len();
        assert!(shown > 0 && shown < 50);
        assert_eq!(v["more"].as_u64().unwrap() as usize, 50 - shown);
    }

    #[test]
    fn combined_sha_is_order_free_and_keeps_a_single_hash() {
        assert_eq!(combined_sha(&["aa"]), "aa");
        assert_eq!(combined_sha(&["aa", "bb"]), combined_sha(&["bb", "aa"]));
        assert_ne!(combined_sha(&["aa", "bb"]), combined_sha(&["aa", "cc"]));
    }

    #[test]
    fn stems_are_clean_and_short() {
        assert_eq!(clean_stem("a b/c.txt"), "a_b_c.txt");
        assert_eq!(clean_stem(&"x".repeat(100)).len(), 40);
    }
}
