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
