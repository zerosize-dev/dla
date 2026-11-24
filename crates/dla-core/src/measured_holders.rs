//! What a measured Python holder implies.
//!
//! `dla summarize heap-graph` over the Python harness's holder chains writes a `retention-path`
//! record whose `holders` say who keeps the objects of the growing lines, and whose entries say
//! whose each holder is (`of`):
//!
//! * `project`: the root is a module global or a class attribute of a module whose file is a
//!   project source file that is not test code;
//! * `tests`: the root is in test code (the driver module, the generated harness module, a file
//!   under a directory named `test` or `tests`, `test_*.py`, `*_test.py`, `conftest.py`) or in a
//!   module of the test runner.
//!
//! A holder of the project that keeps at least one major line and that no finding covers is the
//! first thing to act on: one `dla ledger add` at the holder accounts for every line it keeps.
//! A line whose only major holders are of the tests is no finding; the gate says how many there
//! are. A growing line with no retention record of its harness gets one capture item for its
//! series, not one add command per line.
//!
//! The record in the ledger is bounded, so the full holders are read from its raw file when that
//! is there; the bounded entries serve when it is not.

use std::{collections::BTreeSet, fs, path::Path};

use serde_json::Value;

use crate::{
    attribution::owners_match,
    evidence::{Authority, EvidenceKind, EvidenceRecord, Source},
    finding::{Finding, Polarity},
    holders,
    ledger::Ledger,
    view::RunView,
};

/// The tool name of the Python holder-chain reader in the evidence.
pub const TOOL: &str = "dla-py-retain";

/// The holder kind of a holder added from a module global or a class attribute.
pub const HOLDER_KIND: &str = "static/global";

/// A holder of the project that keeps at least one major line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectHolder {
    /// The retention record it was read from (the latest that names it).
    pub evidence: String,
    /// The harness whose capture that record was read from, when its path says.
    pub harness: Option<String>,
    /// `static:<module>.<name>`.
    pub key: String,
    /// `<module>.<name>`, the owner a finding at it has.
    pub owner: String,
    /// The major lines it keeps, `file:line`.
    pub lines: Vec<String>,
    /// The line the adapter's `finding_site` picks.
    pub site: String,
}

/// What the retention records of a run say about whose the holders are.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Scan {
    /// Holders of the project, one per owner.
    pub project: Vec<ProjectHolder>,
    /// Lines whose only major holders are of the tests.
    pub tests_only: BTreeSet<String>,
    /// Harnesses (`H3`) some retention record was read from.
    pub harnesses: BTreeSet<String>,
    /// Harnesses (`H3`) with a summarized capture of any tool: a retention record or an
    /// allocation-stack record whose source lies under `harness/<H>/`.
    pub summarized: BTreeSet<String>,
}

/// The harness a file under `harness/<H>/...` belongs to.
pub fn harness_of_path(rel: &str) -> Option<&str> {
    let mut it = rel.split(['/', '\\']);
    while let Some(p) = it.next() {
        if p == "harness" {
            return it.next().filter(|h| !h.is_empty());
        }
    }
    None
}

/// True when `f` covers the holder whose owner is `owner`: a leak-side finding with that owner
/// (a shorter form of one, or a longer one, counts: [`owners_match`]).
pub fn finding_covers(f: &Finding, owner: &str) -> bool {
    f.polarity() == Polarity::Leak && f.owner.as_deref().is_some_and(|o| owners_match(owner, o))
}

/// True when some finding of the ledger covers the holder whose owner is `owner`.
pub fn covered(ledger: &Ledger, owner: &str) -> bool {
    ledger.findings().any(|f| finding_covers(f, owner))
}

/// The command that adds the finding a holder of the project needs.
pub fn add_command(evidence: &str, owner: &str, site: &str) -> String {
    format!(
        "dla ledger add --origin growth-series --evidence {evidence} --owner {owner} --site {site} --holder {HOLDER_KIND} --region heap"
    )
}

/// The `holders` of a retention record: the raw file's when it can be read, else the bounded
/// entries of the result.
fn holders_of(run_dir: Option<&Path>, rec: &EvidenceRecord) -> Vec<Value> {
    let from_raw = run_dir
        .zip(rec.raw.as_deref())
        .and_then(|(dir, raw)| fs::read_to_string(dir.join(raw)).ok())
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| v.get("holders").and_then(Value::as_array).cloned());
    from_raw.unwrap_or_else(|| {
        rec.result
            .get("holders")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
    })
}

fn lines_of(entry: &Value) -> Vec<String> {
    holders::entry_sites(entry)
        .into_iter()
        .map(str::to_owned)
        .collect()
}

/// Reads whose each holder of the retention records is.
pub fn scan(view: &RunView) -> Scan {
    scan_in(&view.ledger, view.run_dir.as_deref())
}

/// [`scan`] over a ledger and the run directory its raw files are in.
pub fn scan_in(ledger: &Ledger, run_dir: Option<&Path>) -> Scan {
    let mut out = Scan::default();
    for rec in ledger.evidence() {
        if matches!(
            rec.kind,
            EvidenceKind::RetentionPath | EvidenceKind::AllocStack
        ) && rec.authority != Authority::Static
            && let Source::Artifact { path, .. } = &rec.source
            && let Some(h) = harness_of_path(path)
        {
            out.summarized.insert(h.to_owned());
        }
        if rec.kind != EvidenceKind::RetentionPath
            || rec.authority == Authority::Static
            || rec.tool.name != TOOL
        {
            continue;
        }
        let mut harness: Option<String> = None;
        if let Source::Artifact { path, .. } = &rec.source
            && let Some(h) = harness_of_path(path)
        {
            out.harnesses.insert(h.to_owned());
            harness = Some(h.to_owned());
        }
        let entries = holders_of(run_dir, rec);
        let mut tests: BTreeSet<String> = BTreeSet::new();
        let mut others: BTreeSet<String> = BTreeSet::new();
        for e in &entries {
            let lines = lines_of(e);
            match e.get("of").and_then(Value::as_str) {
                Some("tests") => tests.extend(lines),
                Some("project") => {
                    let key = e.get("key").and_then(Value::as_str).unwrap_or_default();
                    let owner = e
                        .get("owner")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                        .or_else(|| {
                            key.strip_prefix("static:")
                                .map(|k| k.trim().to_owned())
                                .filter(|k| !k.is_empty())
                        });
                    others.extend(lines.iter().cloned());
                    let Some(owner) = owner else { continue };
                    if lines.is_empty() {
                        continue;
                    }
                    let site = e
                        .get("finding_site")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                        .unwrap_or_else(|| lines[0].clone());
                    let holder = ProjectHolder {
                        evidence: rec.evidence_id.to_string(),
                        harness: harness.clone(),
                        key: key.to_owned(),
                        owner,
                        lines,
                        site,
                    };
                    match out.project.iter_mut().find(|p| p.owner == holder.owner) {
                        Some(p) => {
                            let mut merged: BTreeSet<String> = p.lines.iter().cloned().collect();
                            merged.extend(holder.lines.iter().cloned());
                            *p = ProjectHolder {
                                lines: merged.into_iter().collect(),
                                ..holder
                            };
                        }
                        None => out.project.push(holder),
                    }
                }
                _ => others.extend(lines),
            }
        }
        out.tests_only.extend(tests.difference(&others).cloned());
    }
    // A line a holder of the project keeps is never "only the tests'".
    for p in &out.project {
        for l in &p.lines {
            out.tests_only.remove(l);
        }
    }
    out
}

impl Scan {
    /// The holders of the project that no finding covers, in the order the records name them.
    pub fn uncovered(&self, ledger: &Ledger) -> Vec<&ProjectHolder> {
        self.project
            .iter()
            .filter(|h| !covered(ledger, &h.owner))
            .collect()
    }

    /// One sentence on how many growing lines the test code holds; `None` when it holds none.
    pub fn tests_note(&self) -> Option<String> {
        Self::note_for(self.tests_only.len())
    }

    /// The sentence for `n` lines.
    pub fn note_for(n: usize) -> Option<String> {
        (n > 0).then(|| {
            format!(
                "the test code holds {n} growing line{} (a holder in the driver or in the tests keeps their objects): {} not a finding of the project",
                if n == 1 { "" } else { "s" },
                if n == 1 { "it is" } else { "they are" }
            )
        })
    }
}

/// The description of the gate item for an uncovered holder of the project.
pub fn item_text(h: &ProjectHolder) -> String {
    format!(
        "{} keeps {} growing line{} and no finding has the owner {}: one finding at the holder covers them",
        h.key,
        h.lines.len(),
        if h.lines.len() == 1 { "" } else { "s" },
        h.owner
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        evidence::{EvidenceCoverage, NewEvidence, Tool},
        ledger::Ledger,
    };
    use serde_json::json;

    /// A ledger with one retention record of harness H4 whose bounded result lists `result` and
    /// whose raw file in the returned directory lists `raw`.
    fn ledger_with(result: Value, raw: Option<Value>) -> (tempfile::TempDir, Ledger) {
        let dir = tempfile::tempdir().unwrap();
        let mut l = Ledger::create(dir.path()).unwrap();
        let raw_rel = raw.map(|v| {
            fs::create_dir_all(dir.path().join("retain")).unwrap();
            fs::write(dir.path().join("retain/R1.json"), v.to_string()).unwrap();
            "retain/R1.json".to_owned()
        });
        l.add_evidence(NewEvidence {
            source: Source::Artifact {
                path: "harness/H4/out/p1/capture/1/py-1.retain.json".into(),
                sha256: "0".into(),
            },
            tool: Tool {
                name: TOOL.into(),
                version: "1".into(),
            },
            authority: Authority::Live,
            kind: EvidenceKind::RetentionPath,
            result,
            raw: raw_rel,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
        (dir, l)
    }

    #[test]
    fn the_raw_file_gives_every_line_a_holder_keeps_when_the_bounded_record_lists_few() {
        let entry = |lines: &[&str]| {
            json!({"key": "static:pkg.m._reg", "owner": "pkg.m._reg", "of": "project",
                "finding_site": lines[1],
                "sites": lines.iter().map(|l| json!({"site": l})).collect::<Vec<_>>()})
        };
        let (dir, l) = ledger_with(
            json!({"holders": [entry(&["a.py:1", "a.py:2"])]}),
            Some(json!({"holders": [entry(&["a.py:1", "a.py:2", "a.py:3", "a.py:4"])]})),
        );
        let s = scan_in(&l, Some(dir.path()));
        assert_eq!(s.project.len(), 1);
        assert_eq!(s.project[0].lines, ["a.py:1", "a.py:2", "a.py:3", "a.py:4"]);
        assert_eq!(s.project[0].site, "a.py:2");
        assert_eq!(s.project[0].evidence, "E1");
        assert_eq!(
            s.harnesses.iter().map(String::as_str).collect::<Vec<_>>(),
            ["H4"]
        );
        // A raw file that is gone leaves the bounded record's lines.
        let s = scan_in(&l, None);
        assert_eq!(s.project[0].lines, ["a.py:1", "a.py:2"]);
        // An older record has no `of`: no holder is the project's or the tests'.
        let (dir, l) = ledger_with(
            json!({"holders": [{"key": "static:pkg.m._reg", "sites": [{"site": "a.py:1"}]}]}),
            None,
        );
        let s = scan_in(&l, Some(dir.path()));
        assert!(s.project.is_empty() && s.tests_only.is_empty());
        assert_eq!(Scan::note_for(0), None);
        assert!(Scan::note_for(1).unwrap().contains("1 growing line "));
        assert!(Scan::note_for(3).unwrap().contains("3 growing lines "));
    }
}
