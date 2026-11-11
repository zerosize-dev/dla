//! `dla ledger add` and `annotate` warn when an owner names no holder in the growth or
//! retention evidence they cite, and `annotate --owner` never replaces an owner. The ledgers are written with the `dla-core` API.

use std::path::PathBuf;

use assert_cmd::Command;
use dla_core::{
    Authority, EvidenceKind, Ledger, NewEvidence, NewFinding, Origin, Source, Tool,
    evidence::{EvidenceCoverage, Signal},
};
use serde_json::{Value, json};

struct Env {
    project: tempfile::TempDir,
    run_dir: PathBuf,
}

fn dla(e: &Env) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(e.project.path());
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

fn run(e: &Env, args: &[&str]) -> (i32, Value) {
    let out = dla(e).args(args).output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    (
        out.status.code().unwrap(),
        serde_json::from_str(&text).unwrap_or(Value::Null),
    )
}

fn ok(e: &Env, args: &[&str]) -> Value {
    let (code, v) = run(e, args);
    assert_eq!(code, 0, "{args:?}: {v}");
    v
}

fn heap_graph(result: Value, kind: EvidenceKind, src: &str) -> NewEvidence {
    let mut result = result;
    result["signal"] = Signal::new("dla-v8-snapshot", "dump-parser", src).to_value();
    NewEvidence {
        source: Source::Repo {
            commit: "abc".into(),
            path: None,
        },
        tool: Tool {
            name: "dla-v8-snapshot".into(),
            version: "0".into(),
        },
        authority: Authority::Dump,
        kind,
        result,
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    }
}

/// E2 growth (columns `closure:src.board.previous`, `field:src.board.Board.latest`), E3
/// retention (the same two holders); E1 is the run's own record.
/// F1 owns `src.board.Board.publish.previous`, which no holder has.
fn env() -> Env {
    let project = tempfile::tempdir().unwrap();
    let init = Command::cargo_bin("dla")
        .unwrap()
        .arg("--project")
        .arg(project.path())
        .args(["run", "init", "--session-id", "t"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&init.stdout).unwrap();
    let run_dir = project
        .path()
        .join(".dla")
        .join(v["run_id"].as_str().unwrap());
    let mut l = Ledger::load(&run_dir).unwrap();
    l.add_evidence(heap_graph(
        json!({"trend": "T1", "series": "S1", "n": 4, "gate": "pass", "failed": 0, "passed": [
            {"column": "holder:closure:src.board.previous", "region": "heap", "shape": "steady"},
            {"column": "holder:field:src.board.Board.latest", "region": "heap", "shape": "steady"}]}),
        EvidenceKind::GrowthSeries,
        "g1",
    ))
    .unwrap();
    l.add_evidence(heap_graph(
        json!({"file": "a.heapsnapshot", "holders": [
            {"key": "closure:src.board.previous", "holder_kind": "closure/continuation"},
            {"key": "field:src.board.Board.latest", "holder_kind": "container-field"}]}),
        EvidenceKind::RetentionPath,
        "r1",
    ))
    .unwrap();
    let mut nf = NewFinding::new(
        Origin::parse("growth-series").unwrap(),
        vec!["E3".parse().unwrap()],
    );
    nf.owner = Some("src.board.Board.publish.previous".into());
    nf.region = Some(dla_core::finding::Region::Heap);
    l.add_finding(nf).unwrap();
    drop(l);
    let e = Env { project, run_dir };
    let _ = &e.run_dir;
    e
}

fn warnings(v: &Value) -> Vec<String> {
    v["warnings"]
        .as_array()
        .map(|a| a.iter().map(|w| w.as_str().unwrap().to_owned()).collect())
        .unwrap_or_default()
}

#[test]
fn add_warns_when_the_owner_names_no_holder_of_the_cited_evidence() {
    let e = env();
    let v = ok(
        &e,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            "E2,E3",
            "--owner",
            "src.board.Board.publish.previous",
        ],
    );
    let w = warnings(&v);
    assert_eq!(w.len(), 1, "{v}");
    assert!(w[0].contains("src.board.previous"), "{w:?}");
    assert!(w[0].contains("src.board.Board.latest"), "{w:?}");
    assert!(w[0].contains("names no holder"), "{w:?}");
    // The finding is added anyway (a warning, not a refusal).
    assert_eq!(v["state"], "candidate");
    // An owner a holder has, in the form that ties, gives no warning; so does no owner.
    let v = ok(
        &e,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            "E2",
            "--owner",
            "board.previous",
        ],
    );
    assert!(warnings(&v).is_empty(), "{v}");
}

#[test]
fn the_warning_lists_at_most_three_owners() {
    let e = env();
    let mut l = Ledger::load(&e.run_dir).unwrap();
    l.add_evidence(heap_graph(
        json!({"file": "b.heapsnapshot", "holders": [
            {"key": "closure:app.a.v1"}, {"key": "closure:app.a.v2"},
            {"key": "closure:app.a.v3"}, {"key": "closure:app.a.v4"}]}),
        EvidenceKind::RetentionPath,
        "r2",
    ))
    .unwrap();
    drop(l);
    let (_, v) = run(
        &e,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            "E4",
            "--owner",
            "app.b.x",
        ],
    );
    let w = warnings(&v);
    assert_eq!(w.len(), 1, "{v}");
    assert!(
        w[0].contains("app.a.v3") && !w[0].contains("app.a.v4"),
        "{w:?}"
    );
}

#[test]
fn an_owner_already_set_is_refused_with_or_without_evidence_and_the_fix_names_the_way() {
    let e = env();
    for extra in [&[][..], &["--evidence", "E2"][..]] {
        let mut args = vec!["ledger", "annotate", "F1", "--owner", "src.board.previous"];
        args.extend_from_slice(extra);
        let (code, v) = run(&e, &args);
        assert_eq!(code, 2, "{v}");
        assert_eq!(v["error"]["code"], "annotation-conflict");
        let fix = v["error"]["fix"].as_str().unwrap();
        assert!(
            fix.contains("dla ledger add --origin growth-series --evidence <E> --site <the same file:line> --owner src.board.previous"),
            "{v}"
        );
        assert!(fix.contains("dla ledger duplicate F1 --of <new>"), "{v}");
    }
    let l = Ledger::load(&e.run_dir).unwrap();
    assert_eq!(
        l.finding(&"F1".parse().unwrap()).unwrap().owner.as_deref(),
        Some("src.board.Board.publish.previous")
    );
}

#[test]
fn annotate_without_evidence_warns_from_the_evidence_the_finding_cites() {
    let e = env();
    // F2 has no owner yet: a first owner is accepted, with a warning when no holder of the
    // evidence the finding cites has it.
    ok(
        &e,
        &[
            "ledger",
            "add",
            "--origin",
            "growth-series",
            "--evidence",
            "E3",
        ],
    );
    let v = ok(
        &e,
        &["ledger", "annotate", "F2", "--owner", "src.board.nothing"],
    );
    let w = warnings(&v);
    assert_eq!(w.len(), 1, "{v}");
    assert!(w[0].contains("src.board.previous"), "{w:?}");
    let v = ok(
        &e,
        &[
            "ledger",
            "annotate",
            "F2",
            "--owner",
            "src.board.nothing",
            "--note",
            "n",
        ],
    );
    assert_eq!(warnings(&v).len(), 1, "{v}");
}

#[test]
fn an_ambiguous_short_form_and_an_invented_longer_form_are_warned_about() {
    let e = env();
    let mut l = Ledger::load(&e.run_dir).unwrap();
    l.add_evidence(heap_graph(
        json!({"file": "c.heapsnapshot", "holders": [
            {"key": "closure:src.board.previous"}, {"key": "closure:lib.board.previous"}]}),
        EvidenceKind::RetentionPath,
        "r3",
    ))
    .unwrap();
    drop(l);
    let add = |owner: &str| {
        run(
            &e,
            &[
                "ledger",
                "add",
                "--origin",
                "growth-series",
                "--evidence",
                "E4",
                "--owner",
                owner,
            ],
        )
        .1
    };
    let w = warnings(&add("board.previous"));
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("short form of several holders"), "{w:?}");
    let w = warnings(&add("zzz.yyy.src.board.previous"));
    assert_eq!(w.len(), 1, "{w:?}");
    assert!(w[0].contains("names no holder"), "{w:?}");
    // The exact owner is silent.
    assert!(warnings(&add("lib.board.previous")).is_empty());
}
