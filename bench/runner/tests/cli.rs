//! The small subcommands through the binary.

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_bench::compare::diff;
use predicates::prelude::*;
use serde_json::json;

fn bin() -> Command {
    Command::cargo_bin("dla-bench").unwrap()
}

fn sample() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/rust-sample")
}

#[test]
fn compare_treats_ints_and_floats_alike_and_ignores_key_order() {
    assert!(
        diff(
            &json!({"a": 1, "b": [1.0, 2]}),
            &json!({"b": [1, 2.0], "a": 1.0})
        )
        .is_empty()
    );
    let d = diff(
        &json!({"a": {"b": [1, 2]}, "c": 1}),
        &json!({"a": {"b": [1, 3]}, "d": 1}),
    );
    assert!(d.iter().any(|l| l.starts_with("$.a.b[1]: 2 != 3")), "{d:?}");
    assert!(d.iter().any(|l| l.contains("$.c: only in the first")));
    assert!(d.iter().any(|l| l.contains("$.d: only in the second")));
    assert!(!diff(&json!([1]), &json!([1, 2])).is_empty());
    assert!(!diff(&json!(null), &json!(0)).is_empty());
    assert!(!diff(&json!(0.1), &json!(0.1000001)).is_empty());
}

#[test]
fn compare_exit_codes() {
    let t = tempfile::tempdir().unwrap();
    let (a, b, c) = (
        t.path().join("a.json"),
        t.path().join("b.json"),
        t.path().join("c.json"),
    );
    fs::write(&a, "{\"x\": 1.0, \"y\": [1]}").unwrap();
    fs::write(&b, "{\"y\": [1], \"x\": 1}\n").unwrap();
    fs::write(&c, "{\"y\": [2], \"x\": 1}").unwrap();
    bin().arg("compare").arg(&a).arg(&b).assert().success();
    bin()
        .arg("compare")
        .arg(&a)
        .arg(&c)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("$.y[0]: 1 != 2"));
    bin()
        .arg("compare")
        .arg(&a)
        .arg(t.path().join("none.json"))
        .assert()
        .code(1);
}

#[test]
fn grade_then_table_through_the_binary() {
    let s = sample();
    let t = tempfile::tempdir().unwrap();
    let res = t.path().join("res");
    fs::create_dir_all(&res).unwrap();
    copy_dir(&s.join("results"), &res);
    bin()
        .arg("grade")
        .arg(&res)
        .arg("--fixtures")
        .arg(s.join("fixtures"))
        .assert()
        .success();
    bin()
        .arg("compare")
        .arg(res.join("metrics.json"))
        .arg(s.join("expected-metrics.json"))
        .assert()
        .success();
    bin().arg("table").arg(&res).assert().success();
    let md = fs::read_to_string(res.join("table.md")).unwrap();
    assert!(md.contains("## no-skill (archetypes on)"));
    assert!(
        md.contains("| leak_recall | 0.5833 | 7/12 | 0.5833 ± 0.1179 |"),
        "{md}"
    );
    assert!(md.contains("| forced_fit_rate | n/a | 0/0 | n/a |"), "{md}");
    assert!(md.contains("### Usage"));
    assert!(md.contains("### Per fixture"));
    assert!(
        md.contains("| fx-multi | multi-leak | 3 | 3 | 0 | L1: 2/3/2; L2: 2/2/2 |"),
        "{md}"
    );
    assert!(
        md.contains("| unexplained_growth_share | 0.2787 | over 2 run(s) | |"),
        "{md}"
    );
}

#[test]
fn the_table_shows_the_verification_block_only_where_a_group_has_one() {
    let s = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/verify-sample");
    let t = tempfile::tempdir().unwrap();
    let res = t.path().join("res");
    fs::create_dir_all(&res).unwrap();
    copy_dir(&s.join("results"), &res);
    bin()
        .arg("grade")
        .arg(&res)
        .arg("--fixtures")
        .arg(s.join("fixtures"))
        .assert()
        .success();
    bin()
        .arg("compare")
        .arg(res.join("metrics.json"))
        .arg(s.join("expected-metrics.json"))
        .assert()
        .success();
    bin().arg("table").arg(&res).assert().success();
    let md = fs::read_to_string(res.join("table.md")).unwrap();
    assert_eq!(md.matches("### Verification").count(), 1, "{md}");
    assert!(
        md.contains("| verified share of confirmed leak-side findings | 0.7500 | 3/4 |"),
        "{md}"
    );
    assert!(
        md.contains("| label agreement (verified findings with a label verdict) | 0.5000 | 1/2 |"),
        "{md}"
    );
    // The group without a verification record has no block: it comes after the full group.
    let gates = md.split("## gates").nth(1).unwrap();
    assert!(!gates.contains("Verification"), "{md}");
}

fn copy_dir(from: &Path, to: &Path) {
    for e in fs::read_dir(from).unwrap().flatten() {
        let dest = to.join(e.file_name());
        if e.path().is_dir() {
            fs::create_dir_all(&dest).unwrap();
            copy_dir(&e.path(), &dest);
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}

#[test]
fn table_requires_metrics() {
    let t = tempfile::tempdir().unwrap();
    bin().arg("table").arg(t.path()).assert().code(1);
}

#[test]
fn schema_writes_and_checks() {
    let t = tempfile::tempdir().unwrap();
    let out = t.path().join("schemas");
    bin()
        .arg("schema")
        .arg("--out")
        .arg(&out)
        .assert()
        .success();
    for n in ["expected", "findings", "run", "metrics"] {
        let text = fs::read_to_string(out.join(format!("{n}.schema.json"))).unwrap();
        assert!(text.ends_with("}\n") && !text.contains('\r'), "{n}");
        serde_json::from_str::<serde_json::Value>(&text).unwrap();
    }
    bin()
        .arg("schema")
        .arg("--out")
        .arg(&out)
        .arg("--check")
        .assert()
        .success();
    fs::write(out.join("run.schema.json"), "{}\n").unwrap();
    bin()
        .arg("schema")
        .arg("--out")
        .arg(&out)
        .arg("--check")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("run.schema.json"));
    // CRLF checkouts are fine
    bin()
        .arg("schema")
        .arg("--out")
        .arg(&out)
        .assert()
        .success();
    let crlf = fs::read_to_string(out.join("run.schema.json"))
        .unwrap()
        .replace('\n', "\r\n");
    fs::write(out.join("run.schema.json"), crlf).unwrap();
    bin()
        .arg("schema")
        .arg("--out")
        .arg(&out)
        .arg("--check")
        .assert()
        .success();
}

#[test]
fn committed_schemas_are_current() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../schemas");
    bin()
        .arg("schema")
        .arg("--out")
        .arg(&dir)
        .arg("--check")
        .assert()
        .success();
}

#[test]
fn validate_exit_codes() {
    let t = tempfile::tempdir().unwrap();
    bin()
        .arg("validate")
        .arg("--fixtures")
        .arg(t.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("0 fixture(s)"));
    fs::create_dir_all(t.path().join("broken")).unwrap();
    bin()
        .arg("validate")
        .arg("--fixtures")
        .arg(t.path())
        .assert()
        .code(1)
        .stderr(predicate::str::contains("broken: README.md is missing"))
        .stderr(predicate::str::contains("broken: expected.toml is missing"));
}

#[test]
fn run_json_and_findings_samples_validate_against_their_schemas() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../schemas");
    let res = sample().join("results/runs/no-skill/archetypes-on/fx-trap");
    for (schema, file) in [
        ("run.schema.json", res.join("1/run.json")),
        ("findings.schema.json", res.join("1/findings.json")),
    ] {
        let s: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(dir.join(schema)).unwrap()).unwrap();
        let v = jsonschema::validator_for(&s).unwrap();
        let inst: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
        let errs: Vec<String> = v.iter_errors(&inst).map(|e| e.to_string()).collect();
        assert!(errs.is_empty(), "{}: {errs:?}", file.display());
    }
    let m = fs::read_to_string(sample().join("expected-metrics.json")).unwrap();
    let s: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("metrics.schema.json")).unwrap())
            .unwrap();
    let v = jsonschema::validator_for(&s).unwrap();
    let inst: serde_json::Value = serde_json::from_str(&m).unwrap();
    assert!(v.is_valid(&inst));
}

#[cfg(unix)]
#[test]
fn static_skips_artifact_and_symptom_fixtures_with_a_reason() {
    let t = tempfile::tempdir().unwrap();
    let fixtures = t.path().join("fixtures");
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/m4-sample/fixtures");
    for (id, notes) in [
        ("art-series", "project/dumps/heap-0.hprof.gz"),
        ("sym", "project/NOTES.txt"),
    ] {
        fs::create_dir_all(fixtures.join(id)).unwrap();
        copy_dir(&sample.join(id), &fixtures.join(id));
        let n = fixtures.join(id).join(notes);
        fs::create_dir_all(n.parent().unwrap()).unwrap();
        fs::write(n, "notes").unwrap();
    }
    for cmd in ["static"] {
        let json = t.path().join(format!("{cmd}.json"));
        // The dla binary is never started: both fixtures are skipped before any command runs.
        let out = bin()
            .args([cmd, "--dla-bin", "/bin/true", "--repo-root"])
            .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
            .arg("--fixtures")
            .arg(&fixtures)
            .arg("--json")
            .arg(&json)
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let text = String::from_utf8_lossy(&out);
        assert!(
            text.contains("entry artifact: the project holds heap dumps and no source"),
            "{cmd}: {text}"
        );
        assert!(text.contains("entry symptom:"), "{cmd}: {text}");
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&json).unwrap()).unwrap();
        let rows = v["fixtures"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        for r in rows {
            assert!(r["skipped"].as_str().unwrap().starts_with("entry "), "{r}");
            assert_eq!(r["failures"].as_array().unwrap().len(), 0, "{r}");
        }
    }
}

/// A stand-in for `dla` that logs its arguments and creates the run directory on `run init`.
#[cfg(unix)]
const FAKE_DLA: &str = r#"#!/bin/sh
echo "$*" >> "$FAKE_LOG"
P="$2"
shift 2
case "$1:$2" in
run:init) mkdir -p "$P/.dla/R1"; echo R1 > "$P/.dla/current"; : > "$P/.dla/R1/ledger.jsonl"; echo '{"ok":true}' ;;
summarize:heap-graph) echo '{"ok":true,"gate":"fail","reasons":["dumps < 3"],"dumps":[{"file":"d","timestamp_ms":5,"evidence":"E3","top_holder":{"key":"static:a.B.c"}}]}' ;;
ledger:add) echo '{"ok":true,"id":"F1"}' ;;
report:*) echo report ;;
*) echo '{"ok":true}' ;;
esac
"#;

#[cfg(unix)]
#[test]
fn dynamic_drives_artifact_and_symptom_fixtures_through_their_entry_paths() {
    use std::os::unix::fs::PermissionsExt;
    let t = tempfile::tempdir().unwrap();
    let fixtures = t.path().join("fixtures");
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/m4-sample/fixtures");
    for (id, notes) in [
        ("art-single", "project/dumps/heap-0.hprof.gz"),
        ("sym", "project/NOTES.txt"),
    ] {
        fs::create_dir_all(fixtures.join(id)).unwrap();
        copy_dir(&sample.join(id), &fixtures.join(id));
        let n = fixtures.join(id).join(notes);
        fs::create_dir_all(n.parent().unwrap()).unwrap();
        fs::write(n, "notes").unwrap();
    }
    let fake = t.path().join("dla");
    fs::write(&fake, FAKE_DLA).unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    let log = t.path().join("calls.log");
    let json = t.path().join("dynamic.json");
    bin()
        .env("FAKE_LOG", &log)
        .args(["dynamic", "--dla-bin"])
        .arg(&fake)
        .arg("--repo-root")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .arg("--fixtures")
        .arg(&fixtures)
        .arg("--json")
        .arg(&json)
        .assert()
        .success();
    let v: serde_json::Value = serde_json::from_str(&fs::read_to_string(&json).unwrap()).unwrap();
    let rows = v["fixtures"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    for r in rows {
        assert!(r.get("skipped").is_none(), "{r}");
        assert_eq!(r["failures"].as_array().unwrap().len(), 0, "{r}");
        assert_eq!(r["claims"], false, "{r}");
    }
    let calls = fs::read_to_string(&log).unwrap();
    assert!(
        calls.contains("run init --budget 2h --entry artifact --artifact dumps/heap-0.hprof.gz"),
        "{calls}"
    );
    assert!(
        calls.contains("summarize heap-graph dumps/heap-0.hprof.gz"),
        "{calls}"
    );
    assert!(
        calls.contains("--reason not_exercised --remedy"),
        "the one dump ends suspected with a remedy: {calls}"
    );
    assert!(
        calls.contains("run init --budget 2h --entry symptom"),
        "{calls}"
    );
    assert!(
        calls.contains(
            "ledger resolve intake.target --status unresolved --reason env_unavailable --remedy"
        ),
        "{calls}"
    );
    for stage in [" probe", " sweep", " harness", " census"] {
        assert!(
            !calls.contains(stage),
            "no stage runs on these paths: {calls}"
        );
    }
}

#[test]
fn a_findings_file_may_name_an_owner_instead_of_a_site() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../schemas");
    let s: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(dir.join("findings.schema.json")).unwrap())
            .unwrap();
    let v = jsonschema::validator_for(&s).unwrap();
    let owner_only = json!({"findings": [{
        "site": "", "owner": "quotes.RfqDesk.OPEN", "holder": "static/global",
        "region": "heap", "verdict": "leak-confirmed", "mechanism": "m", "evidence": "e"
    }]});
    assert!(v.is_valid(&owner_only));
    let no_site = json!({"findings": [{
        "owner": "quotes.RfqDesk.OPEN", "holder": "static/global",
        "region": "heap", "verdict": "leak-suspected", "mechanism": "m", "evidence": "e"
    }]});
    assert!(v.is_valid(&no_site));
    let bad_owner = json!({"findings": [{
        "owner": 7, "holder": "h", "region": "heap", "verdict": "v", "mechanism": "m", "evidence": "e"
    }]});
    assert!(!v.is_valid(&bad_owner));
}
