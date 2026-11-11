//! Final-gate items a normal run could not meet: the trends `dla alloc-test` writes, the
//! residual open item of a merged ledger, the explanation of an artifact finding, the
//! runtimes a capture tool introduces, the sweep row a probe creates, and the archetype
//! tables for a worker.

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, EvidenceKind, Ledger, NewEvidence, NewFinding, Origin, Source, Tool,
    evidence::{EvidenceCoverage, Signal},
};
use serde_json::{Value, json};

pub struct Run {
    pub dir: tempfile::TempDir,
    pub run_id: String,
}

#[derive(Debug)]
pub struct Out {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
    pub json: Value,
}

pub fn base(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_ARCHETYPES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "CLAUDE_PROJECT_DIR",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
    ] {
        c.env_remove(var);
    }
    c.arg("--project").arg(project);
    c
}

impl Run {
    pub fn path(&self) -> &Path {
        self.dir.path()
    }
    pub fn run_dir(&self) -> std::path::PathBuf {
        self.path().join(".dla").join(&self.run_id)
    }
    pub fn run(&self, args: &[&str]) -> Out {
        let o = base(self.path()).args(args).output().unwrap();
        let stdout = String::from_utf8(o.stdout).unwrap();
        Out {
            code: o.status.code().unwrap(),
            json: serde_json::from_str(&stdout).unwrap_or(Value::Null),
            stdout,
            stderr: String::from_utf8(o.stderr).unwrap(),
        }
    }
    pub fn ok(&self, args: &[&str]) -> Value {
        let o = self.run(args);
        assert_eq!(o.code, 0, "{args:?}: {} {}", o.stdout, o.stderr);
        o.json
    }
    /// Every item the gate lists, from the full file.
    pub fn items(&self, target: &[&str]) -> Vec<Value> {
        let mut args = vec!["gate", "check"];
        args.extend_from_slice(target);
        let o = self.run(&args);
        let name = o.json["stage"].as_str().unwrap().to_owned();
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir().join("gate").join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }
    /// `(kind, id)` of every item the gate lists (the full file, not the first five).
    pub fn missing(&self, target: &[&str]) -> Vec<(String, String)> {
        let mut args = vec!["gate", "check"];
        args.extend_from_slice(target);
        let o = self.run(&args);
        let name = o.json["stage"].as_str().unwrap().to_owned();
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir().join("gate").join(format!("{name}.json"))).unwrap(),
        )
        .unwrap();
        full["missing"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| {
                (
                    m["kind"].as_str().unwrap().to_owned(),
                    m["id"].as_str().unwrap().to_owned(),
                )
            })
            .collect()
    }
}

pub fn init(entry: &str) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let mut args = vec!["run", "init", "--session-id", "t", "--entry", entry];
    if entry == "artifact" {
        fs::create_dir_all(dir.path().join("dumps")).unwrap();
        for d in ["dumps/h1.hprof", "dumps/h2.hprof"] {
            fs::write(dir.path().join(d), b"JAVA PROFILE 1.0.2").unwrap();
            args.extend(["--artifact", d]);
        }
    }
    let o = base(dir.path())
        .env("DLA_CONSENT_TIERS", "1,2")
        .args(&args)
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    Run {
        run_id: v["run_id"].as_str().unwrap().to_owned(),
        dir,
    }
}

pub fn evidence(
    kind: EvidenceKind,
    authority: Authority,
    tool: &str,
    class: &str,
    src: &str,
    mut result: Value,
) -> NewEvidence {
    result["signal"] = Signal::new(tool, class, src).to_value();
    NewEvidence {
        source: Source::Repo {
            commit: "abc".into(),
            path: None,
        },
        tool: Tool {
            name: tool.into(),
            version: "0".into(),
        },
        authority,
        kind,
        result,
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    }
}

// ------------------------------------------------------------ trends of alloc-test

fn column(name: &str) -> Value {
    json!({"name": name, "unit": "bytes", "kind": "gauge", "region": "heap", "n": 10,
           "noise": 1.0, "rise": 900.0, "shape": "steady", "gate": "pass"})
}

fn write_trend(r: &Run, id: &str, series: &str, columns: &[&str]) {
    let dir = r.run_dir().join("trend");
    fs::create_dir_all(&dir).unwrap();
    let doc = json!({
        "schema_version": 1, "id": id, "series": series,
        "thresholds": serde_json::to_value(dla_stats::GateThresholds::default()).unwrap(),
        "columns": columns.iter().map(|c| column(c)).collect::<Vec<_>>(),
    });
    fs::write(dir.join(format!("{id}.json")), doc.to_string()).unwrap();
}

fn trend_columns(m: &[(String, String)]) -> Vec<String> {
    let mut v: Vec<String> = m
        .iter()
        .filter(|(k, _)| k == "trend_column")
        .map(|(_, id)| id.clone())
        .collect();
    v.sort();
    v
}

/// The discriminator `dla alloc-test` writes for the trend it wrote.
fn alloc_discriminator(r: &Run, trend: &str, columns: &[&str]) -> String {
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    let ev = evidence(
        EvidenceKind::Discriminator,
        Authority::Test,
        "dla",
        "x",
        "alloc",
        json!({"test": "plateau+purge", "favors": "leak", "supports": ["leak"], "finding": null,
               "trend": trend, "series": "S1",
               "columns": columns.iter().map(|c| json!({"name": c})).collect::<Vec<_>>()}),
    );
    l.add_evidence(ev).unwrap();
    l.evidence().last().unwrap().evidence_id.to_string()
}

fn finding_citing(r: &Run, evidence: &str) -> String {
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    let mut nf = NewFinding::new(
        Origin::parse("hypothesis").unwrap(),
        vec![evidence.parse().unwrap()],
    );
    nf.site = Some("a.go:1".into());
    l.add_finding(nf).unwrap();
    l.findings().last().unwrap().id.to_string()
}

/// A trend whose columns are `(name, flat)`.
fn write_trend_shapes(r: &Run, id: &str, series: &str, columns: &[(&str, bool)]) {
    let dir = r.run_dir().join("trend");
    fs::create_dir_all(&dir).unwrap();
    let cols: Vec<Value> = columns
        .iter()
        .map(|(name, flat)| {
            let mut c = column(name);
            if *flat {
                c["shape"] = json!("flat");
                c["gate"] = json!("fail");
                c["rise"] = json!(0.0);
            }
            c
        })
        .collect();
    let doc = json!({
        "schema_version": 1, "id": id, "series": series,
        "thresholds": serde_json::to_value(dla_stats::GateThresholds::default()).unwrap(),
        "columns": cols,
    });
    fs::write(dir.join(format!("{id}.json")), doc.to_string()).unwrap();
}

#[test]
fn a_trend_replaced_by_a_later_trend_of_the_series_needs_no_disposition_of_its_own() {
    let r = init("repository");
    // T1 is what `dla alloc-test` wrote: no growth evidence names it.
    write_trend(&r, "T1", "S1", &["probe:CACHE", "go.heap_live"]);
    let m = r.missing(&["--stage", "classify"]);
    assert_eq!(
        trend_columns(&m),
        ["T1:probe:CACHE"].map(String::from),
        "a whole-process column is left to the final gate"
    );
    let m = r.missing(&["--final"]);
    assert_eq!(
        trend_columns(&m),
        ["T1:go.heap_live", "T1:probe:CACHE"].map(String::from)
    );
    let item = r
        .items(&["--stage", "classify"])
        .into_iter()
        .find(|m| m["id"] == "T1:probe:CACHE")
        .unwrap();
    assert_eq!(item["fix"], "dla trend S1", "{item}");
    // The fix writes a new trend of the series and growth evidence for its columns.
    write_trend(&r, "T2", "S1", &["probe:CACHE", "go.heap_live"]);
    let m = r.missing(&["--final"]);
    assert_eq!(
        trend_columns(&m),
        ["T2:go.heap_live", "T2:probe:CACHE"].map(String::from),
        "the old trend is replaced, the new one is held to the rule"
    );
    // A later trend that dropped a column does not release it.
    write_trend(&r, "T3", "S1", &["go.heap_live"]);
    let m = r.missing(&["--final"]);
    assert_eq!(
        trend_columns(&m),
        ["T2:probe:CACHE", "T3:go.heap_live"].map(String::from)
    );
    // A trend of another series is not replaced by it.
    write_trend(&r, "T4", "S9", &["probe:OTHER"]);
    let m = r.missing(&["--stage", "classify"]);
    assert!(
        trend_columns(&m).contains(&"T4:probe:OTHER".to_owned()),
        "{m:?}"
    );
}

#[test]
fn a_flat_re_trend_does_not_retire_an_older_growing_column() {
    let r = init("repository");
    write_trend(&r, "T1", "S1", &["probe:CACHE"]);
    // The series was trended again over a shorter window: the column is flat there.
    write_trend_shapes(&r, "T2", "S1", &[("probe:CACHE", true)]);
    let m = r.missing(&["--stage", "classify"]);
    assert_eq!(
        trend_columns(&m),
        ["T1:probe:CACHE"].map(String::from),
        "the older growth stays held to the gate: {m:?}"
    );
    // A later trend in which the column is still growing does take over.
    write_trend(&r, "T3", "S1", &["probe:CACHE"]);
    let m = r.missing(&["--stage", "classify"]);
    assert_eq!(trend_columns(&m), ["T3:probe:CACHE"].map(String::from));
    // The older column is released once a finding is tied to it (the other way out).
    let r = init("repository");
    write_trend(&r, "T1", "S1", &["probe:CACHE"]);
    write_trend_shapes(&r, "T2", "S1", &[("probe:CACHE", true)]);
    let disc = alloc_discriminator(&r, "T1", &["probe:CACHE"]);
    finding_citing(&r, &disc);
    assert!(trend_columns(&r.missing(&["--stage", "classify"])).is_empty());
}

#[test]
fn a_whole_process_column_is_never_tied_and_is_accounted_for_by_the_residual() {
    let r = init("repository");
    write_trend(
        &r,
        "T1",
        "S1",
        &["probe:CACHE", "os.rss", "cgroup.current", "go.heap_live"],
    );
    // The discriminator lists every column, the whole-process ones too.
    let disc = alloc_discriminator(
        &r,
        "T1",
        &["probe:CACHE", "os.rss", "cgroup.current", "go.heap_live"],
    );
    let f = finding_citing(&r, &disc);
    // Classify: probe:CACHE is tied to the finding; the others are not asked for.
    assert!(trend_columns(&r.missing(&["--stage", "classify"])).is_empty());
    r.ok(&[
        "ledger",
        "resolve",
        &f,
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "run the workload",
    ]);
    // Final: the finding is dispositioned, but a finding that cites the discriminator does
    // not account for os.rss, cgroup.current or go.heap_live.
    let items = r.items(&["--final"]);
    let mut ids: Vec<&str> = items
        .iter()
        .filter(|m| m["kind"] == "trend_column")
        .map(|m| m["id"].as_str().unwrap())
        .collect();
    ids.sort_unstable();
    assert_eq!(ids, ["T1:cgroup.current", "T1:go.heap_live", "T1:os.rss"]);
    let item = items.iter().find(|m| m["id"] == "T1:os.rss").unwrap();
    assert!(
        item["fix"]
            .as_str()
            .unwrap()
            .contains("dla gap S1 --residual"),
        "{item}"
    );
    assert!(
        item["description"].as_str().unwrap().contains("residual"),
        "{item}"
    );
    // Residual rows under their threshold account for them.
    let row = |residual: f64| dla_core::coverage::ResidualRow {
        series: Some("S1".into()),
        region: dla_core::finding::Region::Heap,
        total_slope: 10.0,
        attributed_slope: 10.0 - residual,
        residual,
        unit: "bytes/iteration".into(),
        threshold: 1.0,
        open_item: None,
    };
    dla_core::CoverageManifest::update(&r.run_dir(), |m| m.set_residual(vec![row(0.5)])).unwrap();
    let m = r.missing(&["--final"]);
    assert!(trend_columns(&m).is_empty(), "{m:?}");
    // A row over its threshold without a resolved open item does not.
    dla_core::CoverageManifest::update(&r.run_dir(), |m| m.set_residual(vec![row(8.0)])).unwrap();
    let m = r.missing(&["--final"]);
    assert!(trend_columns(&m).contains(&"T1:os.rss".to_owned()), "{m:?}");
    // Rows computed for another series, or rows that record no series, cover nothing.
    for series in [Some("S9".to_owned()), None] {
        let other = dla_core::coverage::ResidualRow { series, ..row(0.5) };
        dla_core::CoverageManifest::update(&r.run_dir(), |m| {
            m.residual.clear();
            m.set_residual(vec![other.clone()]);
        })
        .unwrap();
        let m = r.missing(&["--final"]);
        assert!(trend_columns(&m).contains(&"T1:os.rss".to_owned()), "{m:?}");
    }
    // The rows of the analysed series cover it again.
    dla_core::CoverageManifest::update(&r.run_dir(), |m| {
        m.residual.clear();
        m.set_residual(vec![row(0.5)]);
    })
    .unwrap();
    let m = r.missing(&["--final"]);
    assert!(trend_columns(&m).is_empty(), "{m:?}");
}

#[test]
fn a_type_count_column_with_no_tie_is_accounted_for_by_the_residual() {
    // The jni-order-codec shape: the probe column is tied to the finding, the instance
    // counts of the entry type and of `byte[]` grow with it and tie to no finding (a
    // generic type never ties; the entry type has no type tie in a live series).
    let r = init("repository");
    write_trend(
        &r,
        "T1",
        "S1",
        &[
            "probe:CACHE",
            "jvm.class:shop.codec.Entry.count",
            "jvm.class:byte[].count",
        ],
    );
    let disc = alloc_discriminator(&r, "T1", &["probe:CACHE"]);
    let f = finding_citing(&r, &disc);
    // Classify asks nothing of the counts.
    assert!(trend_columns(&r.missing(&["--stage", "classify"])).is_empty());
    r.ok(&[
        "ledger",
        "resolve",
        &f,
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "run the workload",
    ]);
    // Final: the counts need the residual rows, and the printed fix says so.
    let items = r.items(&["--final"]);
    let mut ids: Vec<&str> = items
        .iter()
        .filter(|m| m["kind"] == "trend_column")
        .map(|m| m["id"].as_str().unwrap())
        .collect();
    ids.sort_unstable();
    assert_eq!(
        ids,
        [
            "T1:jvm.class:byte[].count",
            "T1:jvm.class:shop.codec.Entry.count"
        ]
    );
    let item = items
        .iter()
        .find(|m| m["id"] == "T1:jvm.class:shop.codec.Entry.count")
        .unwrap();
    let fix = item["fix"].as_str().unwrap();
    assert!(fix.contains("dla gap S1 --residual"), "{item}");
    assert!(!fix.contains("dla trend"), "{item}");
    // Residual rows under their threshold lead to a passing state.
    let row = |residual: f64| dla_core::coverage::ResidualRow {
        series: Some("S1".into()),
        region: dla_core::finding::Region::Heap,
        total_slope: 10.0,
        attributed_slope: 10.0 - residual,
        residual,
        unit: "bytes/iteration".into(),
        threshold: 1.0,
        open_item: None,
    };
    dla_core::CoverageManifest::update(&r.run_dir(), |m| m.set_residual(vec![row(0.5)])).unwrap();
    let m = r.missing(&["--final"]);
    assert!(trend_columns(&m).is_empty(), "{m:?}");
    // Rows over their threshold with no resolved open item do not account for them.
    dla_core::CoverageManifest::update(&r.run_dir(), |m| m.set_residual(vec![row(8.0)])).unwrap();
    let m = r.missing(&["--final"]);
    assert!(
        trend_columns(&m).contains(&"T1:jvm.class:shop.codec.Entry.count".to_owned()),
        "{m:?}"
    );
}

#[test]
fn an_untied_py_type_column_is_not_left_to_the_residual_rows() {
    // The M7 decision: only `jvm.class:*` counts have the residual route (a Python heap
    // residual does not account for per-type counts; references/python.md). An untied
    // `py.type:*` column keeps the ordinary path: it needs a finding tied to it.
    let r = init("repository");
    write_trend(&r, "T1", "S1", &["probe:CACHE", "py.type:shop.Entry.count"]);
    let disc = alloc_discriminator(&r, "T1", &["probe:CACHE"]);
    let f = finding_citing(&r, &disc);
    r.ok(&[
        "ledger",
        "resolve",
        &f,
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "run the workload",
    ]);
    let row = dla_core::coverage::ResidualRow {
        series: Some("S1".into()),
        region: dla_core::finding::Region::Heap,
        total_slope: 10.0,
        attributed_slope: 9.5,
        residual: 0.5,
        unit: "bytes/iteration".into(),
        threshold: 1.0,
        open_item: None,
    };
    dla_core::CoverageManifest::update(&r.run_dir(), |m| m.set_residual(vec![row])).unwrap();
    let items = r.items(&["--final"]);
    let item = items
        .iter()
        .find(|m| m["id"] == "T1:py.type:shop.Entry.count")
        .unwrap_or_else(|| panic!("the residual rows covered a py.type column: {items:?}"));
    let fix = item["fix"].as_str().unwrap();
    assert!(!fix.contains("dla gap"), "{item}");
    assert!(
        !item["description"]
            .as_str()
            .unwrap()
            .contains("residual rows"),
        "{item}"
    );
}

#[test]
fn the_total_of_a_summarize_series_is_accounted_for_by_its_tied_groups() {
    // A summarize-made series has no byte column, so the residual cannot account for its
    // total (`dla gap S6 --residual` is refused); the total is the sum of its groups.
    let r = init("repository");
    let total = "metric:goroutine/total";
    let group = "group:goroutine/wait:select";
    write_trend_shapes(
        &r,
        "T1",
        "S6",
        &[
            (total, false),
            (group, false),
            ("group:goroutine/wait:idle", true),
        ],
    );
    let items = r.items(&["--final"]);
    let item = items
        .iter()
        .find(|m| m["id"] == format!("T1:{total}"))
        .unwrap();
    let fix = item["fix"].as_str().unwrap();
    assert!(!fix.contains("dla gap"), "{item}");
    assert!(
        item["description"].as_str().unwrap().contains(group),
        "{item}"
    );
    // Tie the group to a finding: the finding needs its disposition first.
    let disc = alloc_discriminator(&r, "T1", &[group]);
    let f = finding_citing(&r, &disc);
    let m = r.missing(&["--final"]);
    assert_eq!(
        trend_columns(&m),
        [format!("T1:{group}"), format!("T1:{total}")],
        "{m:?}"
    );
    r.ok(&[
        "ledger",
        "resolve",
        &f,
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "run the workload",
    ]);
    let m = r.missing(&["--final"]);
    assert!(trend_columns(&m).is_empty(), "{m:?}");
    // A total whose groups are all flat is not accounted for by them.
    let r = init("repository");
    write_trend_shapes(&r, "T1", "S6", &[(total, false), (group, true)]);
    let items = r.items(&["--final"]);
    let item = items
        .iter()
        .find(|m| m["id"] == format!("T1:{total}"))
        .unwrap();
    assert!(
        item["description"]
            .as_str()
            .unwrap()
            .contains("none of which"),
        "{item}"
    );
    // A whole-process column that is not a group total is still never accounted for by
    // tied columns.
    let r = init("repository");
    write_trend_shapes(&r, "T1", "S6", &[("os.rss", false), (group, false)]);
    let disc = alloc_discriminator(&r, "T1", &[group]);
    let f = finding_citing(&r, &disc);
    r.ok(&[
        "ledger",
        "resolve",
        &f,
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
    ]);
    let m = r.missing(&["--final"]);
    assert_eq!(trend_columns(&m), ["T1:os.rss".to_owned()], "{m:?}");
}

#[test]
fn an_alloc_test_trend_is_dispositioned_through_the_discriminator_a_finding_cites() {
    let r = init("repository");
    write_trend(&r, "T1", "S1", &["probe:CACHE", "probe:OTHER"]);
    // The discriminator tested probe:CACHE only.
    let disc = alloc_discriminator(&r, "T1", &["probe:CACHE"]);
    let m = r.missing(&["--stage", "classify"]);
    assert_eq!(
        trend_columns(&m),
        ["T1:probe:CACHE", "T1:probe:OTHER"].map(String::from),
        "evidence nobody cites ties nothing"
    );
    let f = finding_citing(&r, &disc);
    // Classify: tied to a finding. Final: the finding needs its disposition.
    let m = r.missing(&["--stage", "classify"]);
    assert_eq!(trend_columns(&m), ["T1:probe:OTHER".to_owned()], "{m:?}");
    let m = r.missing(&["--final"]);
    assert!(
        trend_columns(&m).contains(&"T1:probe:CACHE".to_owned()),
        "{m:?}"
    );
    r.ok(&[
        "ledger",
        "resolve",
        &f,
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        "run the workload",
    ]);
    let m = r.missing(&["--final"]);
    assert_eq!(
        trend_columns(&m),
        ["T1:probe:OTHER".to_owned()],
        "the untested column is not released: {m:?}"
    );
    // The fix for it leads to a state that passes: `dla trend S1` replaces T1.
    write_trend(&r, "T2", "S1", &["probe:CACHE", "probe:OTHER"]);
    let m = r.missing(&["--final"]);
    assert_eq!(
        trend_columns(&m),
        ["T2:probe:CACHE", "T2:probe:OTHER"].map(String::from)
    );
}

// -------------------------------------------------- residual item of a merged ledger

#[test]
fn a_residual_row_naming_a_workers_open_item_matches_the_prefixed_item_after_the_merge() {
    use dla_core::{Resolution, UnresolvedReason};
    let r = init("repository");
    r.ok(&["run", "dispatch", "--workers", "jvm"]);
    // The worker raises O1 for a residual in its slice and resolves it; the coverage row
    // the worker wrote names the plain id.
    {
        let mut l = Ledger::load_for(&r.run_dir(), Some("jvm")).unwrap();
        l.open_item_add(
            "unexplained growth: 5.0 MiB in heap (S1); add a hypothesis or record it unresolved",
            None,
            "dla-gap",
        )
        .unwrap();
        l.resolve_open_item(
            &"O1".parse().unwrap(),
            Resolution::Unresolved {
                reason: UnresolvedReason::NotExercised,
                remedy: "profile the native side".into(),
            },
        )
        .unwrap();
    }
    r.ok(&["ledger", "merge"]);
    let l = Ledger::load(&r.run_dir()).unwrap();
    assert!(
        l.open_item(&"jvm/O1".parse().unwrap())
            .is_some_and(|o| o.resolution.is_some()),
        "the merge renames the item"
    );
    // The core's own O1 (whatever it is) is not the worker's item.
    assert!(
        l.open_item(&"O1".parse().unwrap())
            .is_none_or(|o| o.resolution.is_none() || !o.item.text.contains("heap")),
    );
    r.ok(&["probe"]);
    let path = r.run_dir().join("coverage.json");
    let mut cov: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    cov["residual"] = json!([{"region": "heap", "total_slope": 9.0, "attributed_slope": 4.0,
        "residual": 5.0, "unit": "MiB", "threshold": 1.0, "open_item": "O1"}]);
    fs::write(&path, cov.to_string()).unwrap();
    let m = r.missing(&["--final"]);
    assert!(!m.iter().any(|(k, _)| k == "residual"), "{m:?}");
    // A residual in another region is not covered by the item.
    cov["residual"][0]["region"] = json!("off-heap");
    fs::write(&path, cov.to_string()).unwrap();
    let m = r.missing(&["--final"]);
    assert!(
        m.iter().any(|(k, id)| k == "residual" && id == "off-heap"),
        "{m:?}"
    );
}

// --------------------------------------------------- explanation on an artifact run

#[test]
fn the_final_gate_explanation_of_an_artifact_finding_can_be_met() {
    let r = init("artifact");
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    let add = |l: &mut Ledger, kind, auth, class: &str, src: &str, result: Value| {
        l.add_evidence(evidence(kind, auth, "t", class, src, result))
            .unwrap();
        l.evidence().last().unwrap().evidence_id.clone()
    };
    let growth = add(
        &mut l,
        EvidenceKind::GrowthSeries,
        Authority::Dump,
        "x",
        "s1",
        json!({"trend": "T1", "series": "S1", "n": 3, "gate": "pass", "failed": 0,
        "passed": [{"column": "holder:static:com.acme.Cache.entries", "region": "heap", "shape": "steady"}]}),
    );
    let disc = add(
        &mut l,
        EvidenceKind::Discriminator,
        Authority::Dump,
        "x",
        "s2",
        json!({"test": "dump-plateau", "favors": "leak", "supports": ["leak"], "finding": null}),
    );
    let workload = add(
        &mut l,
        EvidenceKind::Workload,
        Authority::Test,
        "x",
        "s3",
        json!({"harness": "H1", "mode": "command", "runtime": "jvm"}),
    );
    let mut nf = NewFinding::new(
        Origin::parse("growth-series").unwrap(),
        vec![growth.clone()],
    );
    nf.owner = Some("com.acme.Cache.entries".into());
    nf.holder = Some(dla_core::finding::Holder::StaticGlobal);
    l.add_finding(nf).unwrap();
    let mut nf = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![growth.clone()]);
    nf.sibling_of = Some("F1".parse().unwrap());
    nf.hypothesis = Some("N-WARM".into());
    nf.refute_if = Some("growth continues".into());
    l.add_finding(nf).unwrap();
    let id = |s: &str| -> dla_core::FindingId { s.parse().unwrap() };
    use dla_core::FindingState as S;
    l.transition(&id("F1.n1"), S::Refuted, None, None, vec![disc], "t")
        .unwrap();
    l.transition(&id("F1"), S::UnderTest, None, None, vec![workload], "t")
        .unwrap();
    l.transition(&id("F1"), S::Confirmed, None, None, vec![growth], "t")
        .unwrap();
    drop(l);

    let m = r.items(&["--final"]);
    let item = m
        .iter()
        .find(|x| x["kind"] == "explanation")
        .unwrap_or_else(|| panic!("{m:?}"));
    let fix = item["fix"].as_str().unwrap();
    assert!(
        fix.contains("--removal not-found") && !fix.contains("--adds FILE:LINE"),
        "{fix}"
    );
    // The fix works as printed: no writer is recorded and none is needed.
    let text = "Entries are added to com.acme.Cache.entries, a static/global map; no remover found in the dump, so it only grows.";
    let v = r.ok(&[
        "ledger",
        "explain",
        "F1",
        "--tag",
        "inferred",
        "--removal",
        "not-found",
        "--text",
        text,
    ]);
    assert_eq!(v["removal"], "not-found");
    assert!(
        !r.missing(&["--final"])
            .iter()
            .any(|(k, _)| k == "explanation")
    );
    // A removal the dump cannot show is refused.
    let o = r.run(&[
        "ledger",
        "explain",
        "F1",
        "--tag",
        "inferred",
        "--removal",
        "never",
        "--text",
        text,
    ]);
    assert_eq!(o.code, 2, "{o:?}");
}

// ------------------------------------------------------- the sweep stage, per runtime

const PY: &str = "_seen = []\n\ndef handle(req):\n    _seen.append(req)\n    return len(_seen)\n";

fn ids(m: &[(String, String)]) -> Vec<String> {
    m.iter()
        .filter(|(k, _)| k == "coverage")
        .map(|(_, id)| id.clone())
        .collect()
}

#[test]
fn a_probe_does_not_count_as_a_sweep() {
    let r = init("repository");
    fs::write(r.path().join("app.py"), PY).unwrap();
    r.ok(&["probe"]);
    // The probe wrote the runtime row and the tool rows, not the built-in rules' row.
    let m = r.missing(&["--stage", "sweep"]);
    assert_eq!(
        ids(&m),
        ["python/census", "python/sweep"].map(String::from),
        "{m:?}"
    );
    r.ok(&["sweep", "--lang", "python"]);
    let m = r.missing(&["--stage", "sweep"]);
    assert_eq!(ids(&m), ["python/census".to_owned()], "{m:?}");
    r.ok(&["census", "--lang", "python"]);
    assert!(ids(&r.missing(&["--stage", "sweep"])).is_empty());
}

#[test]
fn a_capture_tool_does_not_make_its_runtime_need_a_census_and_a_sweep() {
    let r = init("repository");
    fs::write(r.path().join("app.py"), PY).unwrap();
    r.ok(&["probe"]);
    r.ok(&["sweep", "--lang", "python"]);
    r.ok(&["census", "--lang", "python"]);
    // A valgrind capture of the Python project's native extension adds `native`.
    let path = r.run_dir().join("coverage.json");
    let mut cov: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    cov["workers"]["native"] = json!({"runtime": "native", "backends": {
        "valgrind-memcheck": {"status": "ran", "enumerated": 5, "dispositioned": 0}}});
    fs::write(&path, cov.to_string()).unwrap();
    let m = r.missing(&["--stage", "sweep"]);
    assert!(ids(&m).is_empty(), "{m:?}");
    // A runtime the probe found in the sources is still held to the sweep.
    fs::write(r.path().join("main.c"), "int main(void) { return 0; }\n").unwrap();
    r.ok(&["probe"]);
    let found = Ledger::load(&r.run_dir())
        .unwrap()
        .evidence()
        .filter(|e| e.kind == EvidenceKind::Probe)
        .flat_map(|e| e.result["runtimes"].as_array().unwrap().clone())
        .any(|v| v == "native");
    assert!(found, "the probe should detect the C source");
    let m = r.missing(&["--stage", "sweep"]);
    assert!(ids(&m).contains(&"native/census".to_owned()), "{m:?}");
}

/// A gap-derived series (`dla gap` writes `gap.accounted` and `gap.unexplained` for
/// another series): the gates ask neither for its trend nor for a disposition of its
/// columns, since the residual rows of the source series govern it.
fn write_gap_series(r: &Run, id: &str, from: &str) {
    let dir = r.run_dir().join("series");
    fs::create_dir_all(&dir).unwrap();
    let doc = json!({
        "schema_version": 1, "id": id,
        "source": {"kind": "gap", "harness": "H1", "mode": "fresh", "runtime": "go",
                   "derived_from": from},
        "axis": "iteration", "warmup": 1,
        "columns": [
            {"name": "gap.accounted", "unit": "bytes", "kind": "gauge", "region": "process",
             "baseline": false, "after_gc": false, "scope": "process", "source": "gap"},
            {"name": "gap.unexplained", "unit": "bytes", "kind": "gauge", "region": "off-heap",
             "baseline": false, "after_gc": false, "scope": "process", "source": "gap"}
        ],
        "points": [{"x": 440.0, "t_ms": 13, "values": [9991432.0, -1139976.0]},
                   {"x": 880.0, "t_ms": 40, "values": [14013704.0, -914696.0]},
                   {"x": 1320.0, "t_ms": 70, "values": [18013704.0, -814696.0]}],
        "events": [], "limitations": []
    });
    fs::write(dir.join(format!("{id}.json")), doc.to_string()).unwrap();
}

#[test]
fn a_gap_derived_series_needs_no_trend_and_its_trend_columns_no_disposition() {
    let r = init("repository");
    write_gap_series(&r, "S6", "S1");
    // Untrended: no "series has no trend analysis" item for the gap series.
    let items = r.items(&["--final"]);
    assert!(
        !items
            .iter()
            .any(|m| m["kind"] == "series" && m["id"] == "S6"),
        "{items:?}"
    );
    // Trended (a model may still run dla trend on it): its growing gap.accounted column
    // asks for nothing, so the stop hook cannot loop on the residual accounting itself.
    write_trend(&r, "T6", "S6", &["gap.accounted", "gap.unexplained"]);
    let items = r.items(&["--final"]);
    assert!(
        !items
            .iter()
            .any(|m| m["id"].as_str().is_some_and(|id| id.starts_with("T6:"))),
        "{items:?}"
    );
    // A growing whole-process column of a measured series is still asked for.
    write_trend(&r, "T1", "S1", &["os.rss"]);
    let items = r.items(&["--final"]);
    assert!(items.iter().any(|m| m["id"] == "T1:os.rss"), "{items:?}");
}
