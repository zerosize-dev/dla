//! End-to-end tests of `dla gap` and `dla gap --residual`.

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

struct Project {
    dir: tempfile::TempDir,
    run_dir: std::path::PathBuf,
    harness: String,
    disc: String,
}

/// Appends one hand-written evidence record and returns its ID.
fn add_evidence(run_dir: &Path, kind: &str, authority: &str, result: &Value) -> String {
    let ledger = run_dir.join("ledger.jsonl");
    let text = fs::read_to_string(&ledger).unwrap();
    let seq = text.lines().count() + 1;
    let n = text.matches("\"type\":\"evidence\"").count() + 1;
    let line = json!({"schema_version": 2, "seq": seq, "ts": "2026-01-02T03:05:00Z",
        "type": "evidence", "evidence_id": format!("E{n}"),
        "source": {"repo": {"commit": "abc"}}, "tool": {"name": "dla", "version": "1"},
        "authority": authority, "kind": kind, "result": result,
        "coverage": {"status": "complete"}});
    let mut f = fs::OpenOptions::new().append(true).open(&ledger).unwrap();
    writeln!(f, "{line}").unwrap();
    format!("E{n}")
}

fn project() -> Project {
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path();
    let init = ok(p, &["run", "init", "--session-id", "t"]);
    let run_id = init["run_id"].as_str().unwrap().to_owned();
    let run_dir = p.join(".dla").join(&run_id);
    let harness = add_evidence(
        &run_dir,
        "workload",
        "test",
        &json!({"harness": "H1", "mode": "driver"}),
    );
    let disc = add_evidence(
        &run_dir,
        "discriminator",
        "test",
        &json!({"test": "plateau", "favors": "leak", "supports": ["leak"], "finding": null}),
    );
    Project {
        dir,
        run_dir,
        harness,
        disc,
    }
}

/// A column: name, kind, region, site, values (one per point, the first is warm-up).
type Col<'a> = (&'a str, &'a str, &'a str, Option<&'a str>, Vec<f64>);

fn write_series(pr: &Project, id: &str, runtime: &str, cols: &[Col<'_>]) {
    let n = cols.iter().map(|c| c.4.len()).max().unwrap();
    let columns: Vec<Value> = cols
        .iter()
        .map(|(name, kind, region, site, _)| {
            json!({"name": name, "unit": if *kind == "counter" {"count"} else {"bytes"},
                "kind": kind, "region": region, "baseline": false, "after_gc": true,
                "scope": "process", "site": site, "source": "test"})
        })
        .collect();
    let points: Vec<Value> = (0..n)
        .map(|k| {
            json!({"x": 1000.0 * (k as f64 + 1.0), "t_ms": 50 * k,
                "values": cols.iter().map(|c| c.4[k]).collect::<Vec<_>>()})
        })
        .collect();
    let doc = json!({"schema_version": 1, "id": id,
        "source": {"kind": "harness", "harness": "H1", "workload": "W1", "mode": "fresh", "runtime": runtime},
        "axis": "iteration", "warmup": 1, "columns": columns, "points": points,
        "events": [], "limitations": []});
    let dir = pr.run_dir.join("series");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join(format!("{id}.json")), doc.to_string()).unwrap();
}

fn rising(start: f64, step: f64) -> Vec<f64> {
    (0..9).map(|k| start + step * f64::from(k)).collect()
}

/// Growth evidence naming `passed` columns of series `series`.
fn growth(pr: &Project, kind: &str, series: &str, passed: Value) -> String {
    add_evidence(
        &pr.run_dir,
        kind,
        "test",
        &json!({"trend": "T1", "series": series, "n": 8, "passed": passed, "failed": 0, "gate": "pass"}),
    )
}

/// Adds the finding for `site`, then walks it to confirmed citing `growth_ev`.
fn confirm(pr: &Project, site: &str, growth_ev: &str) -> String {
    let p = pr.dir.path();
    let added = ok(
        p,
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            site,
        ],
    );
    let f = added["id"].as_str().unwrap().to_owned();
    ok(
        p,
        &[
            "ledger",
            "add",
            "--sibling-of",
            &f,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM",
            "--refute-if",
            "growth stops",
            "--evidence",
            growth_ev,
        ],
    );
    ok(p, &["ledger", "test", &f, "--evidence", &pr.harness]);
    ok(p, &["ledger", "annotate", &f, "--holder", "static/global"]);
    ok(
        p,
        &[
            "ledger",
            "refute",
            &format!("{f}.n1"),
            "--evidence",
            &pr.disc,
        ],
    );
    ok(p, &["ledger", "confirm", &f, "--evidence", growth_ev]);
    f
}

fn passed_bytes(finding: &str, column: &str, site: &str, slope: f64) -> Value {
    json!([{"column": column, "region": "heap", "site": site, "finding": finding,
        "slope": slope, "shape": "steady", "unit": "bytes"}])
}

/// Heap series: total 400 bytes per x, leak A 250, leak B 150.
fn heap_series(pr: &Project) {
    write_series(
        pr,
        "S1",
        "go",
        &[
            ("os.rss", "gauge", "process", None, rising(5.0e7, 6.0e5)),
            ("go.heap_live", "gauge", "heap", None, rising(1.0e7, 4.0e5)),
            (
                "probe:A.bytes",
                "gauge",
                "heap",
                Some("src/a.go:6"),
                rising(0.0, 2.5e5),
            ),
            (
                "probe:B.bytes",
                "gauge",
                "heap",
                Some("src/b.go:6"),
                rising(0.0, 1.5e5),
            ),
        ],
    );
}

#[test]
fn the_split_routes_unexplained_growth_to_native() {
    let pr = project();
    let p = pr.dir.path();
    write_series(
        &pr,
        "S1",
        "go",
        &[
            ("os.rss", "gauge", "process", None, rising(5.0e7, 6.0e5)),
            ("go.total", "gauge", "process", None, rising(3.0e7, 1.0e5)),
            ("go.released", "gauge", "process", None, rising(1.0e6, 0.0)),
            ("go.heap_live", "gauge", "heap", None, rising(1.0e7, 1.0e5)),
        ],
    );
    let g = ok(p, &["gap"]);
    assert_eq!(g["series"], "S1");
    assert_eq!(g["derived"], "S2");
    assert_eq!(g["accounted"], "go.total - go.released");
    assert_eq!(g["unexplained_gate"], "pass");
    assert_eq!(g["route"], "native");
    assert_eq!(g["accounted_slope"].as_f64().unwrap().round(), 100.0);
    assert_eq!(g["unexplained_slope"].as_f64().unwrap().round(), 500.0);
    let next = g["next"].to_string();
    assert!(
        next.contains("outside the runtime's accounting")
            && next.contains("dla alloc-test H1 --purge trim"),
        "{next}"
    );

    let d: Value =
        serde_json::from_str(&fs::read_to_string(pr.run_dir.join("series/S2.json")).unwrap())
            .unwrap();
    assert_eq!(d["source"]["kind"], "gap");
    assert_eq!(d["source"]["derived_from"], "S1");
    assert_eq!(d["warmup"], 1);
    assert_eq!(d["axis"], "iteration");
    assert_eq!(d["points"].as_array().unwrap().len(), 9);
    assert_eq!(d["columns"][0]["name"], "gap.accounted");
    assert_eq!(d["columns"][1]["name"], "gap.unexplained");
    assert_eq!(d["columns"][1]["region"], "off-heap");
    assert!(pr.run_dir.join("trend/T1.json").is_file());

    let ledger = fs::read_to_string(pr.run_dir.join("ledger.jsonl")).unwrap();
    let e: Value = ledger
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|l| l["kind"] == "non-heap")
        .unwrap();
    assert_eq!(e["raw"], "trend/T1.json");
    for k in [
        "series",
        "derived",
        "accounted_slope",
        "unexplained_slope",
        "unexplained_gate",
        "unexplained_shape",
        "route",
    ] {
        assert!(e["result"].get(k).is_some(), "no {k}: {e}");
    }
    assert_eq!(e["result"]["route"], "native");

    // The residual table totals the off-heap region from the derived series.
    let r = ok(p, &["gap", "S1", "--residual"]);
    let rows = r["rows"].as_array().unwrap();
    let off = rows.iter().find(|x| x["region"] == "off-heap").unwrap();
    assert_eq!(off["total"].as_f64().unwrap().round(), 500.0);
    assert_eq!(off["attributed"], 0.0);

    // With a gap row for S1, the split no longer sends the model to run the residual again
    // (each run would open another item for the same growth).
    let again = ok(p, &["gap", "S1"]);
    let next = again["next"].to_string();
    assert!(
        next.contains("dla gate check --final") && !next.contains("dla gap S1 --residual"),
        "{next}"
    );
}

#[test]
fn a_split_without_accounted_columns_is_refused() {
    let pr = project();
    let p = pr.dir.path();
    write_series(
        &pr,
        "S1",
        "go",
        &[("os.rss", "gauge", "process", None, rising(5.0e7, 6.0e5))],
    );
    let e = refused(p, &["gap"]);
    assert_eq!(e["code"], "gap-unavailable");
    assert!(e["message"].as_str().unwrap().contains("go.total"));
    assert!(e["fix"].as_str().unwrap().contains("dla harness run H1"));
    // A gap series cannot be split again.
    write_series(
        &pr,
        "S2",
        "go",
        &[
            ("os.rss", "gauge", "process", None, rising(5.0e7, 6.0e5)),
            ("go.total", "gauge", "process", None, rising(3.0e7, 1.0e5)),
            ("go.released", "gauge", "process", None, rising(0.0, 0.0)),
        ],
    );
    ok(p, &["gap", "S2"]);
    let e = refused(p, &["gap", "S3"]);
    assert_eq!(e["code"], "invalid-value");
    assert!(e["fix"].as_str().unwrap().contains("dla gap S2"));
}

#[test]
fn a_single_leak_is_fully_attributed() {
    let pr = project();
    let p = pr.dir.path();
    write_series(
        &pr,
        "S1",
        "go",
        &[
            ("go.heap_live", "gauge", "heap", None, rising(1.0e7, 2.5e5)),
            (
                "probe:A.bytes",
                "gauge",
                "heap",
                Some("src/a.go:6"),
                rising(0.0, 2.5e5),
            ),
        ],
    );
    let g = growth(
        &pr,
        "growth-series",
        "S1",
        passed_bytes("F1", "probe:A.bytes", "src/a.go:6", 250.0),
    );
    confirm(&pr, "src/a.go:6", &g);
    let r = ok(p, &["gap", "--residual"]);
    let rows = r["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "only the heap has a column: {r}");
    assert_eq!(rows[0]["region"], "heap");
    assert_eq!(rows[0]["total"].as_f64().unwrap().round(), 250.0);
    assert_eq!(rows[0]["attributed"].as_f64().unwrap().round(), 250.0);
    assert_eq!(rows[0]["residual"], 0.0);
    assert_eq!(rows[0]["open_item"], Value::Null);
    assert_eq!(rows[0]["unit"], "bytes/iteration");
    assert_eq!(r["calibrated"], false);
    assert_eq!(r["open_items"]["opened"], json!([]));
    let st = fs::read_to_string(pr.run_dir.join("ledger.jsonl")).unwrap();
    assert!(
        !st.contains("\"type\":\"open_item\""),
        "no open item expected"
    );
}

#[test]
fn two_leaks_one_confirmed_open_an_item_the_second_confirmation_resolves() {
    let pr = project();
    let p = pr.dir.path();
    heap_series(&pr);
    let ga = growth(
        &pr,
        "growth-series",
        "S1",
        passed_bytes("F1", "probe:A.bytes", "src/a.go:6", 250.0),
    );
    let f1 = confirm(&pr, "src/a.go:6", &ga);
    assert_eq!(f1, "F1");

    let r = ok(p, &["gap", "--residual"]);
    let heap = &r["rows"][0];
    assert_eq!(heap["region"], "heap");
    assert_eq!(heap["total"].as_f64().unwrap().round(), 400.0);
    assert_eq!(heap["attributed"].as_f64().unwrap().round(), 250.0);
    assert_eq!(heap["residual"].as_f64().unwrap().round(), 150.0);
    assert_eq!(heap["threshold"].as_f64().unwrap().round(), 100.0);
    assert_eq!(heap["open_item"], "O1");
    assert_eq!(r["open_items"]["opened"], json!(["O1"]));

    // The open item names the residual and the way to explain it.
    let ledger = fs::read_to_string(pr.run_dir.join("ledger.jsonl")).unwrap();
    let item: Value = ledger
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|l| l["type"] == "open_item")
        .unwrap();
    let text = item["text"].as_str().unwrap();
    assert!(
        text.starts_with("unexplained growth: 150 bytes/iteration in heap (S1);"),
        "{text}"
    );
    assert!(
        text.contains("dla ledger add --origin residual --evidence E"),
        "{text}"
    );
    assert_eq!(item["opened_by"], "dla gap --residual");

    // Coverage rows carry the item.
    let cov: Value =
        serde_json::from_str(&fs::read_to_string(pr.run_dir.join("coverage.json")).unwrap())
            .unwrap();
    assert_eq!(cov["residual"][0]["open_item"], "O1");

    // Running again keeps the item.
    let r = ok(p, &["gap", "--residual"]);
    assert_eq!(r["open_items"]["kept"], json!(["O1"]));
    assert_eq!(r["open_items"]["opened"], json!([]));
    let ledger = fs::read_to_string(pr.run_dir.join("ledger.jsonl")).unwrap();
    assert_eq!(ledger.matches("\"type\":\"open_item\"").count(), 1);

    // Confirming the second leak explains the rest: the item is resolved, not reopened.
    let gb = growth(
        &pr,
        "growth-series",
        "S1",
        passed_bytes("F2", "probe:B.bytes", "src/b.go:6", 150.0),
    );
    confirm(&pr, "src/b.go:6", &gb);
    let r = ok(p, &["gap", "--residual"]);
    let heap = &r["rows"][0];
    assert_eq!(heap["attributed"].as_f64().unwrap().round(), 400.0);
    assert_eq!(heap["residual"], 0.0);
    assert_eq!(heap["open_item"], Value::Null);
    assert_eq!(r["open_items"]["resolved"][0]["item"], "O1");
    assert_eq!(r["open_items"]["resolved"][0]["by"], "F2");
    let r = ok(p, &["gap", "--residual"]);
    assert_eq!(r["open_items"]["opened"], json!([]));
    assert_eq!(r["open_items"]["resolved"], json!([]));
    let ledger = fs::read_to_string(pr.run_dir.join("ledger.jsonl")).unwrap();
    assert_eq!(ledger.matches("\"type\":\"open_item\"").count(), 1);
    assert_eq!(ledger.matches("\"type\":\"open_item_resolved\"").count(), 1);

    // coverage.residual has the keys the benchmark grader reads.
    let cov: Value =
        serde_json::from_str(&fs::read_to_string(pr.run_dir.join("coverage.json")).unwrap())
            .unwrap();
    let row = &cov["residual"][0];
    for k in [
        "region",
        "total_slope",
        "attributed_slope",
        "residual",
        "unit",
        "threshold",
    ] {
        assert!(row.get(k).is_some(), "no {k}: {row}");
    }
    // The same reading as bench/runner/src/grade.rs residual_of.
    let (mut res, mut tot) = (0.0, 0.0);
    for row in cov["residual"].as_array().unwrap() {
        res += row.get("residual").and_then(Value::as_f64).unwrap_or(-1.0);
        tot += row
            .get("total_slope")
            .and_then(Value::as_f64)
            .unwrap_or(-1.0);
    }
    assert_eq!(res, 0.0);
    assert_eq!(tot.round(), 400.0);
}

#[test]
fn growth_measured_as_a_count_attributes_no_bytes() {
    let pr = project();
    let p = pr.dir.path();
    heap_series(&pr);
    let counts = growth(
        &pr,
        "unit-count-series",
        "S1",
        json!([{"column": "probe:A.entries", "region": "heap", "site": "src/a.go:6",
            "finding": "F1", "slope": 5.0, "shape": "steady", "unit": "count"}]),
    );
    confirm(&pr, "src/a.go:6", &counts);
    // Evidence from another series is not subtracted either.
    let other = growth(
        &pr,
        "growth-series",
        "S9",
        passed_bytes("F2", "probe:B.bytes", "src/b.go:6", 150.0),
    );
    confirm(&pr, "src/b.go:6", &other);
    let r = ok(p, &["gap", "--residual"]);
    let heap = &r["rows"][0];
    assert_eq!(heap["attributed"], 0.0);
    assert_eq!(heap["open_item"], "O1");
    let lim = r["limitations"].to_string();
    assert!(
        lim.contains("F1: growth measured as a count; bytes not attributed"),
        "{lim}"
    );
    assert!(
        lim.contains("F2: growth evidence is from S9, not S1; not subtracted"),
        "{lim}"
    );
}

#[test]
fn a_series_without_any_region_column_is_refused() {
    let pr = project();
    let p = pr.dir.path();
    write_series(
        &pr,
        "S1",
        "go",
        &[("os.rss", "gauge", "process", None, rising(5.0e7, 6.0e5))],
    );
    let e = refused(p, &["gap", "--residual"]);
    assert_eq!(e["code"], "residual-unavailable");
    let e = refused(p, &["gap", "S7"]);
    assert_eq!(e["code"], "invalid-value");
}
