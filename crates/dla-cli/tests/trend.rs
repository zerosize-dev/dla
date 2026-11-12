//! End-to-end tests of `dla trend` and the ledger moves that consume its evidence.

use std::{fs, io::Write, path::Path};

use assert_cmd::Command;
use serde_json::{Value, json};

fn dla(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

fn run(project: &Path, args: &[&str]) -> (i32, Value) {
    let out = dla(project).args(args).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.len() <= 2048, "output over the bound: {}", text.len());
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    (out.status.code().unwrap(), v)
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let (code, v) = run(project, args);
    assert_eq!(code, 0, "{args:?}: {v}");
    assert_eq!(v["ok"], true, "{v}");
    v
}

fn refused(project: &Path, args: &[&str]) -> Value {
    let (code, v) = run(project, args);
    assert_eq!(code, 2, "{args:?}: {v}");
    let e = &v["error"];
    for k in ["code", "message", "missing", "fix"] {
        assert!(e[k].as_str().is_some_and(|s| !s.is_empty()), "no {k}: {v}");
    }
    e.clone()
}

/// A project with a run, a Go source file and a hand-written harness record.
struct Project {
    dir: tempfile::TempDir,
    run_dir: std::path::PathBuf,
    harness: String,
}

fn project() -> Project {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    let init = ok(p, &["run", "init", "--session-id", "t"]);
    let run_id = init["run_id"].as_str().unwrap().to_owned();
    let run_dir = p.join(".dla").join(&run_id);
    fs::create_dir_all(p.join("src")).unwrap();
    fs::write(
        p.join("src/cache.go"),
        "package cache\n\nvar entries = map[string]int{}\n\nfunc Put(k string) {\n\tentries[k] = 1\n}\n\nfunc Other() {\n\tprintln(1)\n}\n",
    )
    .unwrap();
    let ledger = run_dir.join("ledger.jsonl");
    let text = fs::read_to_string(&ledger).unwrap();
    let seq = text.lines().count() + 1;
    let n = text.matches("\"type\":\"evidence\"").count() + 1;
    let line = format!(
        r#"{{"schema_version":2,"seq":{seq},"ts":"2026-01-02T03:05:00Z","type":"evidence","evidence_id":"E{n}","source":{{"repo":{{"commit":"abc"}}}},"tool":{{"name":"dla","version":"1"}},"authority":"test","kind":"workload","result":{{"harness":"H1","mode":"driver"}},"coverage":{{"status":"complete"}}}}"#
    );
    let mut f = fs::OpenOptions::new().append(true).open(&ledger).unwrap();
    writeln!(f, "{line}").unwrap();
    Project {
        dir,
        run_dir,
        harness: format!("E{n}"),
    }
}

/// One column of a hand-written series: (name, kind, scope, site, values).
type Col<'a> = (&'a str, &'a str, &'a str, Option<&'a str>, Vec<f64>);

fn write_series(pr: &Project, id: &str, mode: &str, cols: &[Col<'_>]) {
    let n = cols.iter().map(|c| c.4.len()).max().unwrap();
    let columns: Vec<Value> = cols
        .iter()
        .map(|(name, kind, scope, site, _)| {
            json!({"name": name, "unit": if *kind == "counter" {"count"} else {"bytes"},
                "kind": kind, "region": "heap", "baseline": false, "after_gc": true,
                "scope": scope, "site": site, "source": "test"})
        })
        .collect();
    let points: Vec<Value> = (0..n)
        .map(|k| {
            json!({"x": 1000.0 * (k as f64 + 1.0), "t_ms": 50 * k,
                "values": cols.iter().map(|c| c.4[k]).collect::<Vec<_>>()})
        })
        .collect();
    let doc = json!({"schema_version": 1, "id": id,
        "source": {"kind": "harness", "harness": "H1", "workload": "W1", "mode": mode, "runtime": "go"},
        "axis": "iteration", "warmup": 1, "columns": columns, "points": points,
        "events": [], "limitations": ["driver run"]});
    let dir = pr.run_dir.join("series");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{id}.json")), doc.to_string()).unwrap();
}

fn rising(start: f64, step: f64) -> Vec<f64> {
    (0..9).map(|k| start + step * f64::from(k)).collect()
}

fn flat(v: f64) -> Vec<f64> {
    vec![v; 9]
}

#[test]
fn an_untied_type_count_column_is_hinted_to_the_residual_accounting_as_the_gate_prints() {
    let pr = project();
    let p = pr.dir.path();
    write_series(
        &pr,
        "S1",
        "fresh",
        &[(
            "jvm.class:shop.codec.Entry.count",
            "counter",
            "process",
            None,
            rising(10.0, 25.0),
        )],
    );
    let t = ok(p, &["trend", "S1"]);
    let next: Vec<&str> = t["next"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap())
        .collect();
    assert!(
        next.iter().any(|n| n.starts_with("dla gap S1 --residual")),
        "{t}"
    );
    assert!(next.iter().all(|n| !n.contains("growth-series")), "{t}");
}

#[test]
fn a_whole_process_column_is_hinted_to_the_residual_accounting_not_ledger_add() {
    let pr = project();
    let p = pr.dir.path();
    write_series(
        &pr,
        "S1",
        "fresh",
        &[
            ("os.rss", "gauge", "process", None, rising(2.0e7, 4.0e5)),
            (
                "probe:Cache.entries",
                "counter",
                "process",
                None,
                rising(10.0, 25.0),
            ),
        ],
    );
    let t = ok(p, &["trend", "S1"]);
    let passed: Vec<&str> = t["passed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["column"].as_str().unwrap())
        .collect();
    assert!(passed.contains(&"os.rss"), "{t}");
    let next: Vec<&str> = t["next"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap())
        .collect();
    assert!(
        next.iter().any(|n| n.starts_with("dla gap S1 --residual")),
        "{next:?}"
    );
    // The unbound probe column still gets the ledger hint; the whole-process one does not.
    assert!(
        next.iter()
            .all(|n| !(n.contains("growth-series") && n.contains("os.rss"))),
        "{next:?}"
    );
}

#[test]
fn a_series_with_a_gap_row_is_not_hinted_to_run_gap_again() {
    let pr = project();
    let p = pr.dir.path();
    write_series(
        &pr,
        "S1",
        "fresh",
        &[("os.rss", "gauge", "process", None, rising(2.0e7, 4.0e5))],
    );
    let next_of = || -> Vec<String> {
        ok(p, &["trend", "S1"])["next"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_str().unwrap().to_owned())
            .collect()
    };
    assert!(
        next_of()
            .iter()
            .any(|n| n.starts_with("dla gap S1 --residual"))
    );
    // A row over its threshold that names no item: only `dla gap` raises one, as the final
    // gate says too, so the hint stays gap.
    let row = dla_core::coverage::ResidualRow {
        series: Some("S1".into()),
        region: dla_core::finding::Region::Heap,
        total_slope: 10.0,
        attributed_slope: 2.0,
        residual: 8.0,
        unit: "bytes/iteration".into(),
        threshold: 1.0,
        open_item: None,
    };
    dla_core::CoverageManifest::update(&pr.run_dir, |m| m.set_residual(vec![row.clone()])).unwrap();
    assert!(
        next_of()
            .iter()
            .any(|n| n.starts_with("dla gap S1 --residual"))
    );
    // Each `dla gap` over an unexplained growth opens another item, so once the row has an
    // open item the hint points at resolving it instead.
    let mut with_item = row.clone();
    with_item.open_item = Some("O1".parse().unwrap());
    dla_core::CoverageManifest::update(&pr.run_dir, |m| m.set_residual(vec![with_item.clone()]))
        .unwrap();
    let next = next_of();
    assert!(next.iter().all(|n| !n.starts_with("dla gap ")), "{next:?}");
    assert!(
        next.iter().any(|n| n.starts_with("dla gate check --final")),
        "{next:?}"
    );
}

#[test]
fn a_leak_goes_from_trend_to_confirmed() {
    let pr = project();
    let p = pr.dir.path();
    write_series(
        &pr,
        "S1",
        "fresh",
        &[
            (
                "probe:Cache.entries",
                "counter",
                "process",
                Some("src/cache.go:6"),
                rising(10.0, 25.0),
            ),
            ("os.rss", "gauge", "process", None, rising(2.0e7, 4.0e5)),
            (
                "cgroup.current",
                "gauge",
                "container",
                None,
                rising(3.0e7, 9.0e5),
            ),
            ("probe:Quiet.entries", "counter", "process", None, flat(5.0)),
        ],
    );
    write_series(
        &pr,
        "S2",
        "repeated",
        &[
            (
                "probe:Cache.entries",
                "counter",
                "process",
                Some("src/cache.go:6"),
                flat(10.0),
            ),
            ("os.rss", "gauge", "process", None, flat(2.0e7)),
        ],
    );
    ok(
        p,
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            "src/cache.go:6",
        ],
    );

    // Candidates are found by site; evidence kinds follow the column kinds.
    let t = ok(p, &["trend", "S1"]);
    assert_eq!(t["trend"], "T1");
    assert_eq!(t["series"], "S1");
    assert_eq!(t["evidence"], json!(["E3", "E4"]));
    assert_eq!(t["columns"], 4);
    assert_eq!(t["failed"], 2, "{t}");
    let passed = t["passed"].as_array().unwrap();
    assert_eq!(passed[0]["column"], "probe:Cache.entries");
    assert_eq!(passed[0]["candidates"], json!(["F1"]));
    assert!(passed.iter().all(|c| c["column"] != "cgroup.current"));
    let next = t["next"].as_array().unwrap();
    assert!(
        next[0]
            .as_str()
            .unwrap()
            .starts_with("dla ledger add --sibling-of F1 --origin hypothesis --hypothesis N-WARM"),
        "{next:?}"
    );
    assert!(next[0].as_str().unwrap().ends_with("--evidence E4"));
    assert_eq!(
        next[1],
        format!("dla ledger test F1 --evidence {}", pr.harness)
    );

    // The trend file and the evidence records.
    let report: Value =
        serde_json::from_str(&fs::read_to_string(pr.run_dir.join("trend/T1.json")).unwrap())
            .unwrap();
    assert_eq!(report["columns"].as_array().unwrap().len(), 4);
    let ledger = fs::read_to_string(pr.run_dir.join("ledger.jsonl")).unwrap();
    let lines: Vec<Value> = ledger
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let growth = lines.iter().find(|l| l["kind"] == "growth-series").unwrap();
    assert_eq!(growth["authority"], "test");
    assert_eq!(growth["raw"], "trend/T1.json");
    assert_eq!(growth["source"]["artifact"]["path"], "series/S1.json");
    assert_eq!(growth["result"]["gate"], "pass");
    assert_eq!(growth["result"]["passed"][0]["column"], "os.rss");
    assert_eq!(growth["result"]["failed"], 1); // cgroup.current
    // The growth record carries a signal: tool dla-trend, class harness-series, and the
    // sha256 of the series file as its source, so `dla ledger strong` can judge
    // independence from a retention record or an intervention.
    let sig = &growth["result"]["signal"];
    assert_eq!(sig["tool"], "dla-trend");
    assert_eq!(sig["class"], "harness-series");
    assert_eq!(sig["source_sha256"], growth["source"]["artifact"]["sha256"]);
    assert_eq!(sig["source_sha256"].as_str().unwrap().len(), 64);
    let counts = lines
        .iter()
        .find(|l| l["kind"] == "unit-count-series")
        .unwrap();
    assert_eq!(counts["result"]["passed"][0]["candidates"], json!(["F1"]));
    assert_eq!(counts["result"]["shapes"]["flat"], 1);

    // Fresh versus repeated keys: growth only with new keys that never stops = leak.
    let k = ok(
        p,
        &[
            "trend",
            "S1",
            "--against",
            "S2",
            "--finding",
            "F1",
            "--column",
            "probe:Cache.entries",
        ],
    );
    assert_eq!(k["trend"], "T2");
    assert_eq!(k["keys"]["favors"], "leak");
    let d = k["evidence"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();

    // The ledger rules accept the trend evidence.
    ok(
        p,
        &[
            "ledger",
            "add",
            "--sibling-of",
            "F1",
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth continues over the extra cycles",
            "--evidence",
            "E4",
        ],
    );
    ok(p, &["ledger", "test", "F1", "--evidence", &pr.harness]);
    let e = refused(p, &["ledger", "confirm", "F1", "--evidence", "E4"]);
    assert_eq!(e["code"], "holder-required");
    ok(
        p,
        &["ledger", "annotate", "F1", "--holder", "static/global"],
    );
    let e = refused(p, &["ledger", "confirm", "F1", "--evidence", "E4"]);
    assert_eq!(e["code"], "nonleak-refutation-required");
    let r = ok(p, &["ledger", "refute", "F1.n1", "--evidence", &d]);
    assert_eq!(r["to"], "refuted");
    let c = ok(p, &["ledger", "confirm", "F1", "--evidence", "E4"]);
    assert_eq!(c["to"], "confirmed");
    assert!(c.get("warning").is_none(), "{c}");
}

#[test]
fn a_non_leak_ends_refuted_by_the_keys_discriminator() {
    let pr = project();
    let p = pr.dir.path();
    // A cache that fills up with fresh keys and stays flat with repeated keys: both quiet.
    let mut plateau = vec![0.0];
    plateau.extend((1..9).map(|k: i32| 1000.0 * (1.0 - (-f64::from(k) * 1.2).exp())));
    write_series(
        &pr,
        "S1",
        "fresh",
        &[(
            "probe:Cache.entries",
            "counter",
            "process",
            Some("src/cache.go:6"),
            plateau,
        )],
    );
    write_series(
        &pr,
        "S2",
        "repeated",
        &[(
            "probe:Cache.entries",
            "counter",
            "process",
            Some("src/cache.go:6"),
            flat(40.0),
        )],
    );
    ok(
        p,
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            "src/cache.go:6",
        ],
    );
    let k = ok(p, &["trend", "S1", "--against", "S2", "--finding", "F1"]);
    assert_eq!(k["keys"]["favors"], "non-leak");
    assert_eq!(k["keys"]["supports"], json!(["N-WARM"]));
    assert_eq!(k["passed"], json!([]));
    let d = k["evidence"]
        .as_array()
        .unwrap()
        .last()
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(k["next"][0], format!("dla ledger refute F1 --evidence {d}"));
    let r = ok(p, &["ledger", "refute", "F1", "--evidence", &d]);
    assert_eq!(r["to"], "refuted");
    // The discriminator record carries the convention fields.
    let ledger = fs::read_to_string(pr.run_dir.join("ledger.jsonl")).unwrap();
    let rec: Value = ledger
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|l| l["kind"] == "discriminator")
        .unwrap();
    assert_eq!(rec["result"]["test"], "keys");
    assert_eq!(rec["result"]["finding"], "F1");
    assert_eq!(rec["result"]["column"], "probe:Cache.entries");
    assert!(pr.run_dir.join("trend/T1-keys.json").is_file());
}

#[test]
fn a_finding_with_no_tied_column_is_inconclusive_and_the_discriminator_is_not_bound() {
    let pr = project();
    let p = pr.dir.path();
    // Only os.rss was sampled; F1's probe column never was. os.rss is quiet in both
    // series, which used to refute F1.
    write_series(
        &pr,
        "S1",
        "fresh",
        &[("os.rss", "gauge", "process", None, flat(1e7))],
    );
    write_series(
        &pr,
        "S2",
        "repeated",
        &[("os.rss", "gauge", "process", None, flat(1e7))],
    );
    ok(
        p,
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            "src/cache.go:6",
        ],
    );
    let k = ok(p, &["trend", "S1", "--against", "S2", "--finding", "F1"]);
    assert_eq!(k["keys"]["favors"], "inconclusive");
    assert_eq!(k["keys"]["compared"], 0);
    assert!(k["keys"]["finding"].is_null(), "{k}");
    let next = k["next"].as_array().unwrap();
    assert!(
        next.iter()
            .all(|n| !n.as_str().unwrap().contains("ledger refute")),
        "{next:?}"
    );
    let ledger = fs::read_to_string(pr.run_dir.join("ledger.jsonl")).unwrap();
    let rec: Value = ledger
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|l| l["kind"] == "discriminator")
        .unwrap();
    assert!(rec["result"].get("finding").is_none(), "{rec}");
    let lim = rec["limitations"].to_string();
    assert!(lim.contains("no column of the series is tied"), "{lim}");
}

#[test]
fn refusals_name_the_missing_item_and_the_fix() {
    let pr = project();
    let p = pr.dir.path();
    let e = refused(p, &["trend", "S9"]);
    assert_eq!(e["code"], "invalid-value");
    assert!(e["fix"].as_str().unwrap().contains("dla harness run"));
    write_series(
        &pr,
        "S1",
        "fresh",
        &[("os.rss", "gauge", "process", None, rising(1e7, 1e6))],
    );
    write_series(
        &pr,
        "S2",
        "repeated",
        &[("os.rss", "gauge", "process", None, flat(1e7))],
    );
    let e = refused(p, &["trend", "S1", "--column", "nope"]);
    assert!(
        e["fix"].as_str().unwrap().contains("--column os.rss"),
        "{e}"
    );
    // Reversed key modes name the swapped command.
    let e = refused(p, &["trend", "S2", "--against", "S1"]);
    assert_eq!(e["fix"], "dla trend S1 --against S2");
    // An unknown finding.
    let e = refused(p, &["trend", "S1", "--against", "S2", "--finding", "F7"]);
    assert_eq!(e["code"], "unknown-finding");
    // Nothing was written by the refused calls.
    assert!(!pr.run_dir.join("trend").join("T1.json").exists());
}

#[test]
fn a_series_given_by_path_is_analysed_and_cited_by_digest() {
    let pr = project();
    let p = pr.dir.path();
    write_series(
        &pr,
        "S1",
        "fresh",
        &[("os.rss", "gauge", "process", None, rising(1e7, 1e6))],
    );
    let outside = p.join("copy.json");
    fs::copy(pr.run_dir.join("series/S1.json"), &outside).unwrap();
    let t = ok(p, &["trend", outside.to_str().unwrap()]);
    assert_eq!(t["series"], "S1");
    let ledger = fs::read_to_string(pr.run_dir.join("ledger.jsonl")).unwrap();
    let rec: Value = ledger
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|l| l["kind"] == "growth-series")
        .unwrap();
    assert_eq!(
        rec["source"]["artifact"]["sha256"].as_str().unwrap().len(),
        64
    );
}
