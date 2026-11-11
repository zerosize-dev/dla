//! `dla-bench calib-extract` on a hand-written `dla-bench dynamic --log-dir` tree.

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_bench::calib::extract;
use predicates::prelude::*;
use serde_json::{Value, json};

fn put(p: &Path, v: &Value) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, serde_json::to_string(v).unwrap()).unwrap();
}

fn expected(dir: &Path, id: &str, category: &str, leak: Option<&str>) {
    let mut t = format!(
        "schema_version = 1\nid = \"{id}\"\nruntime = \"python\"\ncategory = \"{category}\"\ndynamic_confirmation = true\n[task]\nprompt = \"p\"\n"
    );
    if let Some(site) = leak {
        t.push_str(&format!(
            "[[leak]]\nid = \"L1\"\nsite = \"{site}\"\nholder = \"container-field\"\nregion = \"heap\"\ncause = \"L-COLLECT\"\n"
        ));
    }
    fs::create_dir_all(dir.join(id)).unwrap();
    fs::write(dir.join(id).join("expected.toml"), t).unwrap();
}

fn col(name: &str, cands: &[&str], gate: &str, ratio: Option<f64>) -> Value {
    let mut halves = json!({"first": 10.0, "second": 8.0});
    if let Some(r) = ratio {
        halves["ratio"] = json!(r);
    }
    json!({"name": name, "unit": "bytes", "kind": "gauge", "region": "heap", "candidates": cands,
        "n": 8, "mk": {"s": 28, "z": 3.3, "p_increasing": 0.0156, "tau": 1.0},
        "sen": {"slope": 100.0, "intercept": 1.0, "lo": 40.0, "hi": 160.0},
        "halves": halves, "noise": 5.0, "rise": 60.0, "shape": "steady", "gate": gate,
        "gate_failed": if gate == "pass" { json!([]) } else { json!(["decaying half-split"]) }})
}

fn series(path: &Path, id: &str) {
    // jvm.heap_used: x = 100..=400 after one warm-up point.
    put(
        &path.join("series").join(format!("{id}.json")),
        &json!({"schema_version": 1, "id": id, "axis": "n", "warmup": 1,
            "source": {"kind": "harness", "harness": "H1"},
            "columns": [{"name": "os.rss"}, {"name": "jvm.heap_used"}],
            "points": [
                {"x": 50.0, "values": [1.0, 1.0]},
                {"x": 100.0, "values": [1.0, 10.0]},
                {"x": 200.0, "values": [1.0, 20.0]},
                {"x": 300.0, "values": [1.0, 30.0]},
                {"x": 400.0, "values": [1.0, 40.0]}]}),
    );
}

fn tree(root: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let (logs, fixtures) = (root.join("logs"), root.join("fixtures"));
    expected(&fixtures, "fx-leak", "archetype-leak", Some("src/a.py:10"));
    expected(&fixtures, "fx-decoy", "decoy-control", None);
    for id in ["fx-leak", "fx-decoy"] {
        let d = logs.join(id);
        series(&d, "S1");
        put(
            &d.join("harness/H1/harness.json"),
            &json!({"workload": "py:main", "mode": "fresh", "iterations": 12, "warmup": 2, "step": 50}),
        );
        put(
            &d.join("trend/T1.json"),
            &json!({"id": "T1", "series": "S1",
                "thresholds": {"min_points": 8, "alpha": 0.05, "min_half_ratio": 0.5, "noise_multiple": 3.0},
                "columns": [
                    col("jvm.heap_used", &["F1"], "pass", Some(0.8)),
                    col("os.rss", &[], "fail", None)]}),
        );
        put(
            &d.join("alloc/A1.json"),
            &json!({"id": "A1", "finding": "F1", "harness": "H1", "iterations": 12, "extra": 8, "favors": "leak",
                "columns": [{"name": "jvm.heap_used", "favors": "leak", "supports": ["leak"], "voting": true,
                    "plateau": {"extra_rise": 900.0, "extra_slope": 5.0, "main_slope": 5.0, "noise": 2.0, "verdict": "growth-continues"},
                    "purge": {"kind": "gc", "before": 10.0, "after": 9.0, "drop": 1.0, "growth": 0.5, "verdict": "retained"}}]}),
        );
        let ledger = [
            json!({"type": "finding_added", "finding": {"id": "F1", "site": "src/a.py:10"}}),
            json!({"type": "evidence", "evidence_id": "E3", "kind": "residual", "result": {"series": "S1",
                "rows": [
                    {"region": "heap", "total": 40.0, "attributed": 0.0, "residual": 40.0, "threshold": 10.0, "unit": "bytes/n", "open_item": "O1"},
                    {"region": "non-heap", "total": -3.0, "attributed": 0.0, "residual": 0.0, "threshold": 1.0, "unit": "bytes/n", "open_item": null}]}}),
        ];
        fs::write(
            d.join("ledger.jsonl"),
            ledger
                .iter()
                .map(|r| r.to_string())
                .collect::<Vec<_>>()
                .join("\n")
                + "\n",
        )
        .unwrap();
    }
    (logs, fixtures)
}

#[test]
fn trend_columns_carry_the_statistics_and_the_expected_gate() {
    let t = tempfile::tempdir().unwrap();
    let (logs, fixtures) = tree(t.path());
    let doc = extract(&logs, &fixtures, &[("trial".into(), "2".into())]).unwrap();
    assert_eq!(doc["schema_version"], 1);
    assert_eq!(doc["meta"]["trial"], "2");
    let leak = &doc["fixtures"]["fx-leak"];
    assert_eq!(leak["expected"]["category"], "archetype-leak");
    assert_eq!(leak["expected"]["leaks"][0]["cause"], "L-COLLECT");
    let heap = &leak["columns"][0];
    assert_eq!(heap["name"], "jvm.heap_used");
    assert_eq!(heap["fixture"], "fx-leak");
    assert_eq!(heap["trend"], "T1");
    assert_eq!(heap["series"], "S1");
    assert_eq!(heap["n"], 8);
    assert_eq!(heap["p_increasing"], 0.0156);
    assert_eq!(heap["slope"], 100.0);
    assert_eq!(heap["sen_lo"], 40.0);
    assert_eq!(heap["half_ratio"], 0.8);
    assert_eq!(heap["rise"], 60.0);
    assert_eq!(heap["noise"], 5.0);
    assert_eq!(heap["rise_over_noise"], 12.0);
    assert_eq!(heap["shape"], "steady");
    assert_eq!(heap["gate"], "pass");
    assert_eq!(heap["tied"], true);
    assert_eq!(heap["expected_gate"], "pass");
    // A column that is not tied to the expected leak has no expectation on a leak fixture.
    let rss = &leak["columns"][1];
    assert_eq!(rss["tied"], false);
    assert_eq!(rss["expected_gate"], Value::Null);
    assert_eq!(
        rss["half_ratio"],
        Value::Null,
        "a column without a ratio reads null"
    );
    assert_eq!(rss["gate_failed"][0], "decaying half-split");
    // Every column of a decoy must fail the gate, tied or not.
    let decoy = &doc["fixtures"]["fx-decoy"]["columns"];
    assert_eq!(decoy[0]["expected_gate"], "fail");
    assert_eq!(decoy[1]["expected_gate"], "fail");
    assert_eq!(
        decoy[0]["tied"], false,
        "the decoy has no expected leak to tie to"
    );
}

#[test]
fn the_harness_settings_of_the_series_are_recorded_per_trend() {
    let t = tempfile::tempdir().unwrap();
    let (logs, fixtures) = tree(t.path());
    let doc = extract(&logs, &fixtures, &[]).unwrap();
    let tr = &doc["fixtures"]["fx-leak"]["trends"]["T1"];
    assert_eq!(tr["series"], "S1");
    assert_eq!(tr["harness"]["iterations"], 12);
    assert_eq!(tr["harness"]["warmup"], 2);
    assert_eq!(tr["harness"]["step"], 50);
    assert_eq!(tr["thresholds"]["min_half_ratio"], 0.5);
}

#[test]
fn alloc_tests_keep_the_plateau_and_purge_numbers() {
    let t = tempfile::tempdir().unwrap();
    let (logs, fixtures) = tree(t.path());
    let doc = extract(&logs, &fixtures, &[]).unwrap();
    let a = &doc["fixtures"]["fx-leak"]["alloc"][0];
    assert_eq!(a["finding"], "F1");
    assert_eq!(a["extra"], 8);
    let c = &a["columns"][0];
    assert_eq!(c["extra_rise"], 900.0);
    assert_eq!(c["extra_slope"], 5.0);
    assert_eq!(c["noise"], 2.0);
    assert_eq!(c["plateau_verdict"], "growth-continues");
    assert_eq!(c["purge_verdict"], "retained");
    assert_eq!(c["purge_drop"], 1.0);
    assert_eq!(c["expected_favors"], Value::Null);
    let d = &doc["fixtures"]["fx-decoy"]["alloc"][0]["columns"][0];
    assert_eq!(d["expected_favors"], "non-leak");
}

#[test]
fn residual_rows_get_the_share_the_span_and_the_noise_of_the_primary_column() {
    let t = tempfile::tempdir().unwrap();
    let (logs, fixtures) = tree(t.path());
    let doc = extract(&logs, &fixtures, &[]).unwrap();
    let rows = doc["fixtures"]["fx-leak"]["residual"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    let heap = &rows[0];
    assert_eq!(heap["region"], "heap");
    assert_eq!(heap["column"], "jvm.heap_used");
    assert_eq!(heap["total"], 40.0);
    assert_eq!(heap["residual"], 40.0);
    assert_eq!(heap["threshold"], 10.0);
    assert_eq!(heap["share"], 1.0);
    assert_eq!(heap["over"], true);
    assert_eq!(heap["open_item"], "O1");
    // Post-warm-up x runs 100..400.
    assert_eq!(heap["span"], 300.0);
    assert_eq!(heap["noise"], 5.0);
    assert_eq!(heap["noise_floor"], 3.0 * 5.0 / 300.0);
    // A shrinking region: no share, not over, and no primary column in the series.
    let non_heap = &rows[1];
    assert_eq!(non_heap["share"], Value::Null);
    assert_eq!(non_heap["over"], false);
    assert_eq!(non_heap["column"], Value::Null);
    assert_eq!(non_heap["noise_floor"], Value::Null);
}

#[test]
fn directories_without_data_are_skipped_and_an_empty_tree_is_an_error() {
    let t = tempfile::tempdir().unwrap();
    let (logs, fixtures) = tree(t.path());
    fs::create_dir_all(logs.join("not-a-fixture")).unwrap();
    let doc = extract(&logs, &fixtures, &[]).unwrap();
    assert!(doc["fixtures"].get("not-a-fixture").is_none());
    let empty = t.path().join("empty");
    fs::create_dir_all(&empty).unwrap();
    Command::cargo_bin("dla-bench")
        .unwrap()
        .arg("calib-extract")
        .arg(&empty)
        .arg("--out")
        .arg(t.path().join("o.json"))
        .assert()
        .code(1)
        .stderr(predicate::str::contains("no fixture directory"));
}

#[test]
fn the_command_writes_one_json_file() {
    let t = tempfile::tempdir().unwrap();
    let (logs, fixtures) = tree(t.path());
    let out = t.path().join("sub/calib.json");
    Command::cargo_bin("dla-bench")
        .unwrap()
        .arg("calib-extract")
        .arg(&logs)
        .arg("--fixtures")
        .arg(&fixtures)
        .arg("--out")
        .arg(&out)
        .args(["--meta", "iterations=12", "--meta", "trial=1"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "2 fixture(s), 4 trend column row(s)",
        ));
    let doc: Value = serde_json::from_str(&fs::read_to_string(&out).unwrap()).unwrap();
    assert_eq!(doc["meta"]["iterations"], "12");
    assert_eq!(doc["fixtures"].as_object().unwrap().len(), 2);
    Command::cargo_bin("dla-bench")
        .unwrap()
        .arg("calib-extract")
        .arg(&logs)
        .arg("--out")
        .arg(&out)
        .args(["--meta", "oops"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("KEY=VALUE"));
}

/// A stand-in for `dla` that logs its arguments, reports one runtime on `probe` and fails
/// `harness gen` (the test reads what `harness gen` was given).
#[cfg(unix)]
const FAKE_DLA: &str = r#"#!/bin/sh
echo "$*" >> "$FAKE_LOG"
P="$2"
shift 2
case "$1:$2" in
run:init) mkdir -p "$P/.dla/R1"; echo R1 > "$P/.dla/current"; : > "$P/.dla/R1/ledger.jsonl"; echo '{"ok":true}' ;;
probe:--build) echo '{"ok":true,"summary":{"runtimes":["python"],"builds":{}}}' ;;
harness:gen) echo '{"ok":false,"code":"fake","message":"stop here"}' ;;
*) echo '{"ok":true}' ;;
esac
"#;

/// Runs `dla-bench dynamic` on one repository fixture with the fake `dla`; returns the lines
/// the fake logged for `harness gen`.
#[cfg(unix)]
fn harness_gen_calls(extra: &[&str]) -> Vec<String> {
    use std::os::unix::fs::PermissionsExt;
    let t = tempfile::tempdir().unwrap();
    let fixtures = t.path().join("fixtures");
    expected(&fixtures, "fx-gen", "non-leak", None);
    fs::create_dir_all(fixtures.join("fx-gen/project/src")).unwrap();
    fs::write(fixtures.join("fx-gen/project/src/main.py"), "print('x')\n").unwrap();
    let fake = t.path().join("dla");
    fs::write(&fake, FAKE_DLA).unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    let log = t.path().join("calls.log");
    // The fixture fails (the fake has no harness); the exit code is not what is tested.
    let _ = Command::cargo_bin("dla-bench")
        .unwrap()
        .env("FAKE_LOG", &log)
        .args(["dynamic", "--dla-bin"])
        .arg(&fake)
        .arg("--repo-root")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .arg("--fixtures")
        .arg(&fixtures)
        .arg("--log-dir")
        .arg(t.path().join("logs"))
        .args(extra)
        .output()
        .unwrap();
    fs::read_to_string(&log)
        .unwrap_or_default()
        .lines()
        .filter(|l| l.contains("harness gen"))
        .map(str::to_owned)
        .collect()
}

#[cfg(unix)]
#[test]
fn dynamic_passes_iterations_step_and_warmup_to_harness_gen() {
    let calls = harness_gen_calls(&["--iterations", "24", "--step", "500", "--warmup", "3"]);
    assert!(!calls.is_empty(), "harness gen was never called");
    for c in &calls {
        assert!(
            c.contains("harness gen --lang python --iterations 24 --step 500 --warmup 3"),
            "{c}"
        );
    }
    // Without the flags `harness gen` gets none of them: the harness defaults apply.
    let calls = harness_gen_calls(&[]);
    assert!(!calls.is_empty());
    for c in &calls {
        assert!(
            !c.contains("--iterations") && !c.contains("--step") && !c.contains("--warmup"),
            "{c}"
        );
    }
}

#[test]
fn dynamic_help_lists_the_calibration_flags() {
    let out = Command::cargo_bin("dla-bench")
        .unwrap()
        .args(["dynamic", "--help"])
        .output()
        .unwrap();
    let help = String::from_utf8_lossy(&out.stdout);
    for flag in ["--iterations", "--step", "--warmup"] {
        assert!(help.contains(flag), "{flag} missing:\n{help}");
    }
}

/// A tree with one fixture whose ledger holds `finding_ids` (all at the expected leak's site)
/// and whose single trend column names `candidates` and `finding`.
fn tie_tree(
    root: &Path,
    finding_ids: &[&str],
    candidates: &[&str],
    finding: Option<&str>,
) -> Value {
    let (logs, fixtures) = (root.join("logs"), root.join("fixtures"));
    expected(&fixtures, "fx-tie", "archetype-leak", Some("src/a.py:10"));
    let d = logs.join("fx-tie");
    series(&d, "S1");
    let mut c = col("jvm.heap_used", candidates, "pass", Some(0.8));
    if let Some(f) = finding {
        c["finding"] = json!(f);
    }
    put(
        &d.join("trend/T1.json"),
        &json!({"id": "T1", "series": "S1", "thresholds": {}, "columns": [c]}),
    );
    let ledger: Vec<String> = finding_ids
        .iter()
        .map(|id| {
            json!({"type": "finding_added", "finding": {"id": id, "site": "src/a.py:10"}})
                .to_string()
        })
        .collect();
    fs::write(d.join("ledger.jsonl"), ledger.join("\n") + "\n").unwrap();
    extract(&logs, &fixtures, &[]).unwrap()
}

#[test]
fn tied_compares_exact_ids_and_does_not_strip_the_worker_prefix() {
    let t = tempfile::tempdir().unwrap();
    // The leak is finding jvm/F1; the column names go/F1 (another worker's F1): not tied.
    let doc = tie_tree(t.path(), &["jvm/F1"], &["go/F1"], None);
    let c = &doc["fixtures"]["fx-tie"]["columns"][0];
    assert_eq!(c["tied"], false);
    assert_eq!(c["expected_gate"], Value::Null);
    // The bare F1 of a single-worker ledger is not jvm/F1 either.
    let t = tempfile::tempdir().unwrap();
    let doc = tie_tree(t.path(), &["jvm/F1"], &["F1"], None);
    assert_eq!(doc["fixtures"]["fx-tie"]["columns"][0]["tied"], false);
    // The same id with its prefix is tied.
    let t = tempfile::tempdir().unwrap();
    let doc = tie_tree(t.path(), &["jvm/F1"], &["go/F1", "jvm/F1"], None);
    assert_eq!(doc["fixtures"]["fx-tie"]["columns"][0]["tied"], true);
}

#[test]
fn a_column_is_also_tied_through_its_own_finding_field() {
    let t = tempfile::tempdir().unwrap();
    let doc = tie_tree(t.path(), &["F2"], &[], Some("F2"));
    let c = &doc["fixtures"]["fx-tie"]["columns"][0];
    assert_eq!(c["tied"], true);
    assert_eq!(c["expected_gate"], "pass");
    let t = tempfile::tempdir().unwrap();
    let doc = tie_tree(t.path(), &["F2"], &[], Some("F3"));
    assert_eq!(doc["fixtures"]["fx-tie"]["columns"][0]["tied"], false);
}

#[test]
fn fit_noise_is_the_scaled_mad_around_the_theil_sen_line() {
    use dla_bench::calib::fit_noise;
    // An exact line has no noise; fewer than three points or one x has none to give.
    let line: Vec<(f64, f64)> = (0..6).map(|i| (f64::from(i), 3.0 * f64::from(i))).collect();
    assert_eq!(fit_noise(&line), Some(0.0));
    assert_eq!(fit_noise(&line[..2]), None);
    assert_eq!(fit_noise(&[(1.0, 1.0), (1.0, 2.0), (1.0, 3.0)]), None);
    // Slope 17/12 (median of the pairwise slopes), intercept 0; the median absolute deviation
    // of the residuals is 0.5833..., so the noise is 1.4826 x that (checked with a separate
    // computation of the same definition).
    let pts = [(0.0, 0.0), (1.0, 2.0), (2.0, 1.0), (3.0, 3.0), (4.0, 6.0)];
    let n = fit_noise(&pts).unwrap();
    assert!((n - 0.86485).abs() < 1e-9, "{n}");
}

#[test]
fn an_off_heap_row_takes_its_noise_and_span_from_the_gap_series_it_was_fitted_on() {
    let t = tempfile::tempdir().unwrap();
    let (logs, fixtures) = (t.path().join("logs"), t.path().join("fixtures"));
    expected(&fixtures, "fx-gap", "archetype-leak", Some("src/a.py:10"));
    let d = logs.join("fx-gap");
    // S1: node.external over x 100..=500 with big noise; S2: the gap series derived from S1,
    // gap.unexplained over x 1000..=5000 (span 4000) with the noise of `pts` below.
    put(
        &d.join("series/S1.json"),
        &json!({"schema_version": 1, "id": "S1", "axis": "n", "warmup": 0,
            "source": {"kind": "harness"},
            "columns": [{"name": "node.external"}],
            "points": [
                {"x": 100.0, "values": [0.0]}, {"x": 200.0, "values": [50.0]},
                {"x": 300.0, "values": [10.0]}, {"x": 400.0, "values": [90.0]},
                {"x": 500.0, "values": [20.0]}]}),
    );
    let ys = [0.0, 2.0, 1.0, 3.0, 6.0];
    let points: Vec<Value> = ys
        .iter()
        .enumerate()
        .map(|(i, y)| json!({"x": 1000.0 * (i as f64 + 1.0), "values": [y]}))
        .collect();
    put(
        &d.join("series/S2.json"),
        &json!({"schema_version": 1, "id": "S2", "axis": "n", "warmup": 0,
            "source": {"kind": "gap", "derived_from": "S1"},
            "columns": [{"name": "gap.unexplained"}], "points": points}),
    );
    // An older gap series of another source must not be picked.
    put(
        &d.join("series/S3.json"),
        &json!({"schema_version": 1, "id": "S3", "axis": "n", "warmup": 0,
            "source": {"kind": "gap", "derived_from": "S9"},
            "columns": [{"name": "gap.unexplained"}], "points": []}),
    );
    let ledger = json!({"type": "evidence", "evidence_id": "E1", "kind": "residual",
        "result": {"series": "S1", "rows": [
            {"region": "off-heap", "total": 3.0, "attributed": 0.0, "residual": 3.0,
             "threshold": 0.5, "unit": "bytes/n", "open_item": null}]}});
    fs::write(d.join("ledger.jsonl"), ledger.to_string() + "\n").unwrap();
    let doc = extract(&logs, &fixtures, &[]).unwrap();
    let row = &doc["fixtures"]["fx-gap"]["residual"][0];
    assert_eq!(row["column"], "gap.unexplained");
    assert_eq!(row["span"], 4000.0);
    // x spacing 1000 and the y values of the fit_noise test: the noise is 0.86485 per unit y
    // scaled by the x spacing of 1000.
    let noise = row["noise"].as_f64().unwrap();
    assert!((noise - 0.86485).abs() < 1e-9, "{noise}");
    let floor = row["noise_floor"].as_f64().unwrap();
    assert!((floor - 3.0 * noise / 4000.0).abs() < 1e-12);
    // Without a gap series the row falls back to node.external of the source series.
    fs::remove_file(d.join("series/S2.json")).unwrap();
    let doc = extract(&logs, &fixtures, &[]).unwrap();
    let row = &doc["fixtures"]["fx-gap"]["residual"][0];
    assert_eq!(row["column"], "node.external");
    assert_eq!(row["span"], 400.0);
}

#[test]
fn an_off_heap_row_uses_the_gap_series_its_raw_file_names_when_the_evidence_names_none() {
    let t = tempfile::tempdir().unwrap();
    let (logs, fixtures) = (t.path().join("logs"), t.path().join("fixtures"));
    expected(&fixtures, "fx-raw", "archetype-leak", Some("src/a.py:10"));
    let d = logs.join("fx-raw");
    put(
        &d.join("series/S1.json"),
        &json!({"schema_version": 1, "id": "S1", "axis": "n", "warmup": 0,
            "source": {"kind": "harness"},
            "columns": [{"name": "node.external"}],
            "points": [{"x": 100.0, "values": [0.0]}, {"x": 200.0, "values": [50.0]},
                       {"x": 300.0, "values": [10.0]}]}),
    );
    let gap = |id: &str, scale: f64| {
        let points: Vec<Value> = [0.0, 2.0, 1.0, 3.0, 6.0]
            .iter()
            .enumerate()
            .map(|(i, y)| json!({"x": scale * (i as f64 + 1.0), "values": [y]}))
            .collect();
        json!({"schema_version": 1, "id": id, "axis": "n", "warmup": 0,
            "source": {"kind": "gap", "derived_from": "S1"},
            "columns": [{"name": "gap.unexplained"}], "points": points})
    };
    // S2 (span 4000) is the one fitted; S3 is a newer gap series of the same source (span 40).
    put(&d.join("series/S2.json"), &gap("S2", 1000.0));
    put(&d.join("series/S3.json"), &gap("S3", 10.0));
    put(
        &d.join("gap/residual-S1-E1.json"),
        &json!({"series": "S1", "derived": "S2", "rows": []}),
    );
    let ledger = json!({"type": "evidence", "evidence_id": "E1", "kind": "residual",
        "raw": "gap/residual-S1-E1.json",
        "result": {"series": "S1", "rows": [
            {"region": "off-heap", "total": 3.0, "attributed": 0.0, "residual": 3.0,
             "threshold": 0.5, "unit": "bytes/n", "open_item": null}]}});
    fs::write(d.join("ledger.jsonl"), ledger.to_string() + "\n").unwrap();
    let doc = extract(&logs, &fixtures, &[]).unwrap();
    let row = &doc["fixtures"]["fx-raw"]["residual"][0];
    assert_eq!(row["column"], "gap.unexplained");
    assert_eq!(row["span"], 4000.0, "the newest gap series was used: {row}");
    // A raw file that names no derived series means none was used: the source series is.
    put(
        &d.join("gap/residual-S1-E1.json"),
        &json!({"series": "S1", "derived": null, "rows": []}),
    );
    let doc = extract(&logs, &fixtures, &[]).unwrap();
    let row = &doc["fixtures"]["fx-raw"]["residual"][0];
    assert_eq!(row["column"], "node.external");
}
