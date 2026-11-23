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
const PROTECTED: [&str; 9] = [
    "_attention",
    "ok",
    "evidence",
    "raw",
    "summaries",
    "artifacts",
    "files",
    "id",
    "run_id",
];

/// Sentences that name the next step; a cut one reads as a different instruction: never shrunk.
/// `junit_hint` and `pytest_hint` point at the project's own tests as a hookless harness;
/// `queue_next` is the queue's next command with the count of what is left; `stacks_hint` is the
/// alloc-test's sentence about capturing the stacks that lead to a growing allocation line.
const KEPT_WHOLE: [&str; 4] = ["junit_hint", "pytest_hint", "queue_next", "stacks_hint"];

/// Longest string kept whole in a protected value.
const PROTECTED_STR: usize = 160;

fn protected_copy(v: &Value, max_str: usize, max_arr: usize) -> Value {
    shrink(v, max_str, max_arr, &mut false)
}

/// What gives way first when a status-like object (a list of items to work on, and details
/// around it) is too long for the bound. One place, one order; `fit` follows it:
///
/// 1. `DETAIL`: counts, per-record details and pointers that repeat what an item says, each
///    dropped whole;
/// 2. the later items of the list, from the end, while the first one (and `next_command`)
///    would not fit whole;
/// 3. `CONTEXT`: the triggers, the queue's per-runtime lines, the run's own pointers and, last,
///    the counts by state.
///
/// The first item, `next_command`, the verdict and the totals are never touched here. Every
/// cut is named in `left_out` (a dropped key, or "<list>: N more items"), so nothing reads as
/// complete when it is not.
const DETAIL: [&str; 8] = [
    "revisit",
    "harnesses_generated",
    "harnesses_run",
    "closed_static_only",
    "budget_estimate",
    "budget",
    "stages_passed",
    "actions",
];
const CONTEXT: [&str; 6] = ["triggers", "queue", "ledger", "report", "record", "counts"];

/// Fits `v` (an object with an array `list` of items) into the bound by the order above, so
/// the generic cut in `bound` has nothing to do. Returns `v` unchanged when it fits.
pub fn fit(mut v: Value, list: &str) -> Value {
    if size(&v) <= LIMIT {
        return v;
    }
    let mut left: Vec<String> = Vec::new();
    let fits = |v: &Value, left: &[String]| {
        let mut t = v.clone();
        if !left.is_empty() {
            t["left_out"] = json!(left);
        }
        size(&t) <= LIMIT
    };
    for k in DETAIL {
        if fits(&v, &left) {
            break;
        }
        if v.as_object_mut().and_then(|o| o.remove(k)).is_some() {
            left.push(k.to_owned());
        }
    }
    let mut dropped = 0usize;
    while !fits(&v, &left_with(&left, list, dropped)) {
        let Some(Value::Array(a)) = v.get_mut(list) else {
            break;
        };
        if a.len() <= 1 {
            break;
        }
        a.pop();
        dropped += 1;
    }
    for k in CONTEXT {
        if fits(&v, &left_with(&left, list, dropped)) {
            break;
        }
        if v.as_object_mut().and_then(|o| o.remove(k)).is_some() {
            left.push(k.to_owned());
        }
    }
    let left = left_with(&left, list, dropped);
    if !left.is_empty() {
        v["left_out"] = json!(left);
    }
    v
}

fn left_with(left: &[String], list: &str, dropped: usize) -> Vec<String> {
    let mut l = left.to_vec();
    if dropped > 0 {
        l.insert(0, format!("{list}: {dropped} more item(s)"));
    }
    l
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
            // The hint that names the project's own tests as a hookless first harness is a
            // sentence, not a detail: it is never cut.
            for k in KEPT_WHOLE {
                if let Some(h) = orig.get(k) {
                    o.insert(k.to_owned(), h.clone());
                }
                // `dla harness gen` carries the hint inside its summary.
                if let (Some(h), Some(Value::Object(sum))) = (
                    orig.get("summary").and_then(|s| s.get(k)),
                    o.get_mut("summary"),
                ) {
                    sum.insert(k.to_owned(), h.clone());
                }
            }
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
            // The sentence that names the next step stays whole here too.
            for k in KEPT_WHOLE {
                if let Some(x) = o.get(k).or_else(|| o.get("summary").and_then(|s| s.get(k))) {
                    keep.insert(k.to_owned(), x.clone());
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
    fn the_sentences_that_name_the_next_step_are_never_cut() {
        let hint = format!("this project has JUnit 5 tests: {} END", "x ".repeat(150));
        let py = format!(
            "this project has Python test files: {} END",
            "w ".repeat(150)
        );
        let next = format!(
            "347 candidates in 24 test-loop batches {} END",
            "y ".repeat(150)
        );
        // Shrunk: the top-level sentence and the one inside the summary stay whole.
        let v = json!({
            "ok": true,
            "queue_next": next,
            "summary": {"junit_hint": hint, "pytest_hint": py, "other": "z".repeat(900)},
            "rows": vec!["a long text ".repeat(20); 40],
        });
        let b = bound(v);
        assert!(serde_json::to_string(&b).unwrap().len() < MAX_OUTPUT_BYTES);
        assert_eq!(b["truncated"], true);
        assert_eq!(b["queue_next"], next);
        assert_eq!(b["summary"]["junit_hint"], hint);
        assert_eq!(b["summary"]["pytest_hint"], py);
    }

    #[test]
    fn the_last_resort_keeps_the_sentences_that_name_the_next_step() {
        let hint = format!("this project has JUnit 5 tests: {} END", "x ".repeat(150));
        let py = format!(
            "this project has Python test files: {} END",
            "w ".repeat(150)
        );
        let next = format!(
            "347 candidates in 24 test-loop batches {} END",
            "y ".repeat(150)
        );
        let mut o = Map::new();
        o.insert("ok".into(), json!(true));
        o.insert("queue_next".into(), json!(next));
        o.insert(
            "summary".into(),
            json!({"junit_hint": hint, "pytest_hint": py}),
        );
        for i in 0..400 {
            o.insert(format!("key{i}"), json!(i));
        }
        let b = bound(Value::Object(o));
        assert!(serde_json::to_string(&b).unwrap().len() < MAX_OUTPUT_BYTES);
        assert_eq!(b["queue_next"], next);
        assert_eq!(b["junit_hint"], hint);
        assert_eq!(b["pytest_hint"], py);
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

    /// A status with many blocking items, a long queue item and a stop-stamp field.
    fn status_like(n: usize, key: &str) -> Value {
        let first = json!({
            "id": "python", "kind": "queue",
            "description": "2 candidate(s) of python are ranked for measurement and not targeted yet; 179 of 180 budget minutes remain. Next batch (all in package shop): F1, F2, F3, F4, F5, F6, F7, F8. 1 had a driver run with no coverage data; the second look names F9 (+94 waiting on it: F1, F4, F48, ...)",
            "fix": "dla harness gen --lang python --driver --next, fill the hooks so each unit reaches the listed targets with a new key, dla harness run <H> --mode both, dla trend S<n>",
        });
        let mut items = vec![first];
        for i in 0..n {
            items.push(json!({"id": format!("F{i}"), "kind": "finding",
                "description": format!("finding F{i} is candidate and has no disposition"),
                "fix": format!("dla ledger resolve F{i} --as unresolved --reason not_exercised --remedy \"<text>\"")}));
        }
        json!({
            "ok": true, "run_id": "20260101-000000-abcd", key: items,
            "blocking_total": n + 1, "blocking_dependents": 700,
            "next_command": "dla harness gen --lang python --driver --next",
            "stop_stamp": "the run carries a stop-hook stamp; the next command that works on it lifts the stamp",
            "counts": {"by_state": {"candidate": 40, "confirmed": 3, "refuted": 9}},
            "queue": {"python": {"command": "dla harness gen --lang python --driver --next", "eligible": 95, "next": ["F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8"], "pending": 95}},
            "triggers": {"F1": {"trigger": "reproduced", "harness": "H3", "varies": "the key of every request, a long line of text"}},
            "revisit_total": 611, "harnesses_run": 61,
        })
    }

    #[test]
    fn fit_keeps_the_first_item_and_next_command_whole_and_names_what_it_dropped() {
        for key in ["blocking", "missing"] {
            let v = status_like(40, key);
            let first = v[key][0].clone();
            let out = bound(fit(v, key));
            assert!(out.get("truncated").is_none(), "{out}");
            assert!(serde_json::to_string(&out).unwrap().len() < MAX_OUTPUT_BYTES);
            assert_eq!(out[key][0], first, "{out}");
            assert_eq!(
                out["next_command"],
                "dla harness gen --lang python --driver --next"
            );
            assert_eq!(out["blocking_total"], 41);
            let left = out["left_out"].to_string();
            assert!(left.contains("more item(s)"), "{left}");
            assert!(left.contains("harnesses_run"), "{left}");
            // Details go before the later items: with room for them they stay.
            assert!(
                out[key].as_array().unwrap().len() > 1,
                "later items kept while they fit: {out}"
            );
        }
    }

    #[test]
    fn fit_leaves_a_value_that_fits_alone() {
        let v = json!({"ok": true, "blocking": [{"id": "a"}]});
        assert_eq!(fit(v.clone(), "blocking"), v);
    }
}
