//! Metrics added in M7 (see `bench/GRADING.md`, "M7 metrics"): citations, dropped work,
//! refutations without a discriminator, per-row decoy results and per-fixture agreement.
//!
//! The Python grader (`grade.py`) implements the same rules independently.

use std::collections::{BTreeMap, BTreeSet, HashMap};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::grade::{LedgerRead, State, ledger_key};

/// The outcome of one run of a fixture, for the agreement check.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct RunOutcome {
    /// Run index.
    pub run: u32,
    /// `found:<ids>` (the confirmable leaks found, comma separated, `none` when no leak was
    /// found) for fixtures with a leak set, else `claim` or `no-claim`.
    pub outcome: String,
}

/// Whether the runs of one fixture agree on its graded outcome.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct Agreement {
    /// Graded runs of the fixture.
    pub runs: u64,
    /// The outcome of each run, in run order.
    pub outcomes: Vec<RunOutcome>,
    /// All runs have the same outcome. A fixture with one run is unanimous.
    pub unanimous: bool,
}

impl Agreement {
    /// Builds the block from the outcomes in run order.
    pub fn of(outcomes: Vec<RunOutcome>) -> Self {
        let unanimous = outcomes.windows(2).all(|w| w[0].outcome == w[1].outcome);
        Self {
            runs: outcomes.len() as u64,
            outcomes,
            unanimous,
        }
    }
}

/// Runs of the leak fixtures of a row that found every confirmable leak.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct DecoyLeakRuns {
    /// Runs in which every confirmable leak was found.
    pub found_runs: u64,
    /// Graded runs of the leak fixtures.
    pub runs: u64,
}

/// Runs of the decoy fixtures of a row that claimed a leak.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct DecoyClaimRuns {
    /// Runs with at least one claim.
    pub claim_runs: u64,
    /// Graded runs of the decoy fixtures.
    pub runs: u64,
}

/// The decoy and control pair result of one archetype row.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct DecoyRow {
    /// `<leak fixture>/<decoy fixture>` of every pair of the row.
    pub pairs: Vec<String>,
    /// The leak fixtures.
    pub leak: DecoyLeakRuns,
    /// The decoy fixtures.
    pub decoy: DecoyClaimRuns,
    /// The leak was found in every run of the leak fixtures, no run of the decoys claimed a
    /// leak, and both sides have runs.
    pub pass: bool,
}

impl DecoyRow {
    /// Sets `pass` from the counts.
    pub fn settle(&mut self) {
        self.pass = self.leak.runs > 0
            && self.decoy.runs > 0
            && self.leak.found_runs == self.leak.runs
            && self.decoy.claim_runs == 0;
    }
}

/// `(num, den)` counts the ledger gives for the M7 metrics of one run.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LedgerExtra {
    /// Findings (not hypotheses) with no evidence id, over all such findings.
    pub uncited_findings: (u64, u64),
    /// Checklist answers other than `unresolved` with no evidence id, over those answers.
    pub uncited_checklist: (u64, u64),
    /// `(1, 1)` or `(0, 1)` when the run's report is COMPLETE, `(0, 0)` otherwise.
    pub dropped: (u64, u64),
}

fn ids_in(v: Option<&Value>, into: &mut BTreeSet<String>) {
    for e in v.and_then(Value::as_array).into_iter().flatten() {
        if let Some(s) = e.as_str().filter(|s| !s.is_empty()) {
            into.insert(s.to_owned());
        }
    }
}

/// The trailing number of an open item id (`jvm/O12` is 12).
fn item_number(id: &str) -> Option<u64> {
    let digits = id.rsplit('/').next()?.strip_prefix('O')?;
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse().ok()
}

/// What the ledger text says that the finding replay does not keep: the evidence each
/// finding cites, the latest answer of each checklist item and the open items.
#[derive(Debug, Default)]
struct Scan {
    cited: HashMap<String, BTreeSet<String>>,
    answers: BTreeMap<String, (bool, bool)>,
    open_items: HashMap<String, String>,
    resolved: BTreeSet<String>,
}

fn scan(ledger: &[u8]) -> Scan {
    let mut s = Scan::default();
    for line in ledger.split(|b| *b == b'\n') {
        let Ok(line) = std::str::from_utf8(line) else {
            continue;
        };
        let Ok(rec) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        let Some(kind) = rec.get("type").and_then(Value::as_str) else {
            continue;
        };
        if !rec.is_object() {
            continue;
        }
        let key_of = |name: &str| {
            rec.get(name)
                .and_then(Value::as_str)
                .map(|id| ledger_key(id, &rec))
        };
        match kind {
            "finding_added" => {
                let Some(f) = rec.get("finding").filter(|f| f.is_object()) else {
                    continue;
                };
                let Some(id) = f.get("id").and_then(Value::as_str) else {
                    continue;
                };
                let mut ids = BTreeSet::new();
                ids_in(f.get("supports"), &mut ids);
                ids_in(f.get("contradicts"), &mut ids);
                s.cited.insert(ledger_key(id, &rec), ids);
            }
            "transition" => {
                if let Some(key) = key_of("finding")
                    && let Some(ids) = s.cited.get_mut(&key)
                {
                    ids_in(rec.get("evidence"), ids);
                }
            }
            "finding_merged" => {
                let (Some(canonical), Some(duplicate)) = (key_of("canonical"), key_of("duplicate"))
                else {
                    continue;
                };
                if canonical != duplicate
                    && s.cited.contains_key(&canonical)
                    && let Some(dup) = s.cited.remove(&duplicate)
                {
                    s.cited.entry(canonical).or_default().extend(dup);
                }
            }
            "checklist_answer" => {
                let Some(item) = rec.get("item").and_then(Value::as_str) else {
                    continue;
                };
                let unresolved = rec.get("status").and_then(Value::as_str) == Some("unresolved");
                let mut ids = BTreeSet::new();
                ids_in(rec.get("evidence"), &mut ids);
                s.answers
                    .insert(item.to_owned(), (unresolved, !ids.is_empty()));
            }
            "open_item" => {
                if let Some(key) = key_of("id") {
                    let text = rec.get("text").and_then(Value::as_str).unwrap_or("");
                    s.open_items.insert(key, text.to_owned());
                }
            }
            "open_item_resolved" => {
                if let Some(key) = key_of("id") {
                    s.resolved.insert(key);
                }
            }
            _ => {}
        }
    }
    s
}

/// True when the report text has the line `Status: COMPLETE` as its status line (the first
/// line that starts with `Status:`).
pub fn report_is_complete(report: &str) -> bool {
    report
        .lines()
        .map(|l| l.trim_matches(|c| matches!(c, ' ' | '\t' | '\r')))
        .find(|l| l.starts_with("Status:"))
        .is_some_and(|l| l == "Status: COMPLETE")
}

/// True when the open item a coverage row names is resolved: the item exists and has a
/// resolution, or (for an id without a worker prefix) a prefixed item with the same number
/// about the same region is resolved. The same reading as `gate.rs`.
fn row_item_resolved(s: &Scan, id: &str, region: &str) -> bool {
    if s.open_items.contains_key(id) && s.resolved.contains(id) {
        return true;
    }
    if id.contains('/') {
        return false;
    }
    let Some(n) = item_number(id) else {
        return false;
    };
    let tag = format!(" in {region} (");
    s.open_items.iter().any(|(k, text)| {
        k.contains('/')
            && item_number(k) == Some(n)
            && s.resolved.contains(k)
            && text.contains(&tag)
    })
}

/// True when a `coverage.json` residual row is over its threshold with no resolved open item.
fn coverage_has_unresolved_residual(coverage: &str, s: &Scan) -> bool {
    let Ok(v) = serde_json::from_str::<Value>(coverage) else {
        return false;
    };
    let Some(rows) = v.get("residual").and_then(Value::as_array) else {
        return false;
    };
    rows.iter().any(|row| {
        let (Some(residual), Some(threshold)) = (
            row.get("residual").and_then(Value::as_f64),
            row.get("threshold").and_then(Value::as_f64),
        ) else {
            return false;
        };
        if residual <= threshold {
            return false;
        }
        let region = row.get("region").and_then(Value::as_str).unwrap_or("");
        let resolved = row
            .get("open_item")
            .and_then(Value::as_str)
            .is_some_and(|id| row_item_resolved(s, id, region));
        !resolved
    })
}

/// The M7 counts of one run whose findings come from the ledger.
///
/// `read` is the replay of `ledger`; `report` is `report.md` and `coverage` is
/// `coverage.json`, when the run has them and they are UTF-8.
pub fn ledger_extra(
    read: &LedgerRead,
    ledger: &[u8],
    report: Option<&str>,
    coverage: Option<&str>,
) -> LedgerExtra {
    let s = scan(ledger);
    let mut out = LedgerExtra::default();
    for (key, f) in &read.findings {
        if f.hypothesis {
            continue;
        }
        out.uncited_findings.1 += 1;
        if s.cited.get(key).is_none_or(BTreeSet::is_empty) {
            out.uncited_findings.0 += 1;
        }
    }
    for (unresolved, cited) in s.answers.values() {
        if *unresolved {
            continue;
        }
        out.uncited_checklist.1 += 1;
        if !cited {
            out.uncited_checklist.0 += 1;
        }
    }
    if report.is_some_and(report_is_complete) {
        let undispositioned = read.findings.iter().any(|(_, f)| f.state == State::Open);
        let residual = coverage.is_some_and(|c| coverage_has_unresolved_residual(c, &s));
        out.dropped = (u64::from(undispositioned || residual), 1);
    }
    out
}
