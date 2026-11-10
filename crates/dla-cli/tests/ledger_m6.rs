//! `dla ledger label`, `strong`, `explain` and the verifier's label, through the binary.
//! The ledgers are written with the `dla-core` API (evidence has no command), then driven
//! by the commands.

use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use dla_core::{
    Authority, EvidenceKind, FindingState, Ledger, NewEvidence, NewFinding, Origin, Source, Tool,
    evidence::{EvidenceCoverage, Signal},
    finding::Holder,
};
use serde_json::{Value, json};

const TABLE: &str = r#"
[L-COLLECT]
holders = ["static/global", "container-field"]
regions = ["heap"]
requires = [["growth-series", "retention-path"]]
disconfirm = ["N-WARM"]
repair = ["bound/evict"]
summary = "A collection gains entries and nothing removes them."

[L-HANDLE]
holders = ["native-handle/global-ref"]
regions = ["off-heap"]
requires = [["growth-series", "alloc-stack"]]
disconfirm = []
repair = ["release-regular"]
summary = "A native handle is acquired and never released."
"#;

struct Env {
    project: tempfile::TempDir,
    data: tempfile::TempDir,
    run_dir: PathBuf,
}

fn dla(e: &Env) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(e.project.path());
    c.arg("--data").arg(e.data.path());
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_ARCHETYPES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

fn run_with(e: &Env, env: &[(&str, &str)], args: &[&str]) -> (i32, Value) {
    let mut c = dla(e);
    for (k, v) in env {
        c.env(k, v);
    }
    let out = c.args(args).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.len() <= dla_core::MAX_OUTPUT_BYTES,
        "{} bytes: {stdout}",
        stdout.len()
    );
    (
        out.status.code().unwrap(),
        serde_json::from_str(&stdout).unwrap_or(Value::Null),
    )
}

fn ok(e: &Env, args: &[&str]) -> Value {
    let (code, v) = run_with(e, &[], args);
    assert_eq!(code, 0, "{args:?}: {v}");
    assert_eq!(v["ok"], true, "{v}");
    v
}

fn refused(e: &Env, env: &[(&str, &str)], args: &[&str]) -> Value {
    let (code, v) = run_with(e, env, args);
    assert_eq!(code, 2, "{args:?}: {v}");
    assert_eq!(v["ok"], false);
    for k in ["code", "message", "missing", "fix"] {
        assert!(v["error"][k].as_str().is_some_and(|s| !s.is_empty()), "{v}");
    }
    v["error"].clone()
}

fn evidence(
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

fn write_ledger(e: &Env, l: &Ledger) {
    let text: String = l
        .lines()
        .iter()
        .map(|x| serde_json::to_string(x).unwrap() + "\n")
        .collect();
    fs::write(e.run_dir.join("ledger.jsonl"), text).unwrap();
}

/// A run whose ledger holds: E1 static hit, E2 harness, E3 leak discriminator, E4 trend growth
/// (signed), E5 vgdb retention (signed); F1 confirmed with F1.n1 (N-WARM) refuted.
fn env() -> Env {
    let project = tempfile::tempdir().unwrap();
    let data = tempfile::tempdir().unwrap();
    fs::write(data.path().join("archetypes.toml"), TABLE).unwrap();
    let init = Command::cargo_bin("dla")
        .unwrap()
        .args(["--project"])
        .arg(project.path())
        .args(["run", "init", "--session-id", "t"])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&init.stdout).unwrap();
    let run_dir = project
        .path()
        .join(".dla")
        .join(v["run_id"].as_str().unwrap());
    let e = Env {
        project,
        data,
        run_dir,
    };
    let mut l = Ledger::from_text("").unwrap();
    let ev = |l: &mut Ledger, n: NewEvidence| {
        l.add_evidence(n).unwrap();
    };
    ev(
        &mut l,
        evidence(
            EvidenceKind::StaticHit,
            Authority::Static,
            "t",
            "x",
            "s1",
            json!({}),
        ),
    );
    ev(
        &mut l,
        evidence(
            EvidenceKind::Workload,
            Authority::Test,
            "t",
            "x",
            "s2",
            json!({"harness": "H1", "mode": "command", "runtime": "go"}),
        ),
    );
    ev(
        &mut l,
        evidence(
            EvidenceKind::Discriminator,
            Authority::Test,
            "dla",
            "x",
            "s3",
            json!({"test": "plateau", "favors": "leak", "supports": ["leak"], "finding": null}),
        ),
    );
    ev(
        &mut l,
        evidence(
            EvidenceKind::GrowthSeries,
            Authority::Test,
            "dla-trend",
            "harness-series",
            "aa",
            json!({"trend": "T1", "series": "S1", "n": 8, "gate": "pass", "failed": 0,
            "passed": [{"column": "probe:CACHE", "region": "heap", "site": "a.go:1", "finding": "F1", "shape": "steady"}]}),
        ),
    );
    ev(
        &mut l,
        evidence(
            EvidenceKind::RetentionPath,
            Authority::Live,
            "valgrind-vgdb",
            "leak-checker",
            "bb",
            json!({"root": "static table", "path": [], "alloc": "grow (a.go:2)"}),
        ),
    );
    let mut nf = NewFinding::new(
        Origin::parse("census").unwrap(),
        vec!["E1".parse().unwrap()],
    );
    nf.site = Some("a.go:1".into());
    nf.holder = Some(Holder::StaticGlobal);
    l.add_finding(nf).unwrap();
    let mut nf = NewFinding::new(
        Origin::parse("hypothesis").unwrap(),
        vec!["E1".parse().unwrap()],
    );
    nf.sibling_of = Some("F1".parse().unwrap());
    nf.hypothesis = Some("N-WARM".into());
    nf.refute_if = Some("growth continues".into());
    l.add_finding(nf).unwrap();
    let id = |s: &str| s.parse().unwrap();
    l.transition(
        &id("F1.n1"),
        FindingState::Refuted,
        None,
        None,
        vec!["E3".parse().unwrap()],
        "t",
    )
    .unwrap();
    l.transition(
        &id("F1"),
        FindingState::UnderTest,
        None,
        None,
        vec!["E2".parse().unwrap()],
        "t",
    )
    .unwrap();
    l.transition(
        &id("F1"),
        FindingState::Confirmed,
        None,
        None,
        vec!["E4".parse().unwrap()],
        "t",
    )
    .unwrap();
    write_ledger(&e, &l);
    e
}

fn reload(e: &Env) -> Ledger {
    Ledger::load(&e.run_dir).unwrap()
}

const GOOD: &str = "Requests add entries to the static/global table at a.go:1 through put at a.go:9 and no code removes them, so it only grows.";

// ----------------------------------------------------------------------------- label

#[test]
fn label_suggest_then_label_then_relabel() {
    let e = env();
    let s = ok(&e, &["ledger", "label", "F1", "--suggest"]);
    assert_eq!(s["archetypes"], "on");
    let rows = s["rows"].as_array().unwrap();
    assert_eq!(rows[0]["id"], "L-COLLECT");
    // E5 ties by its allocation site, but F1 does not cite it, so it is a record to name.
    assert_eq!(rows[0]["status"], "met");
    assert_eq!(rows[0]["evidence"], json!(["E4", "E5"]));
    let err = refused(&e, &[], &["ledger", "label", "F1", "--as", "L-COLLECT"]);
    assert_eq!(err["code"], "label-evidence-missing");
    assert_eq!(
        err["fix"],
        "dla ledger label F1 --as L-COLLECT --evidence E4,E5"
    );
    assert_eq!(
        reload(&e)
            .finding(&"F1".parse().unwrap())
            .unwrap()
            .archetype,
        "UNCLASSIFIED"
    );
    // Evidence named explicitly: the retention record names the site.
    let l = ok(
        &e,
        &[
            "ledger",
            "label",
            "F1",
            "--as",
            "L-COLLECT",
            "--evidence",
            "E4,E5",
        ],
    );
    assert_eq!(l["label"], "L-COLLECT");
    assert_eq!(l["previous"], "UNCLASSIFIED");
    assert_eq!(l["kinds"], json!(["growth-series", "retention-path"]));
    assert!(l.get("warning").is_none());
    assert_eq!(
        reload(&e)
            .finding(&"F1".parse().unwrap())
            .unwrap()
            .archetype,
        "L-COLLECT"
    );
    // Relabel as UNCLASSIFIED keeps the previous label in the record.
    let l = ok(
        &e,
        &[
            "ledger",
            "label",
            "F1",
            "--as",
            "UNCLASSIFIED",
            "--note",
            "verifier disputes it",
        ],
    );
    assert_eq!(l["previous"], "L-COLLECT");
    let err = refused(&e, &[], &["ledger", "label", "F1", "--as", "UNCLASSIFIED"]);
    assert_eq!(err["code"], "empty-field");
    let err = refused(&e, &[], &["ledger", "label", "F1", "--as", "L-NOPE"]);
    assert_eq!(err["code"], "unknown-archetype");
    assert!(err["fix"].as_str().unwrap().contains("L-COLLECT"));
}

#[test]
fn archetypes_off_leaves_findings_unclassified() {
    let e = env();
    for env in [vec![("DLA_ARCHETYPES", "off")], vec![]] {
        let data_missing = env.is_empty();
        let mut c = dla(&e);
        if data_missing {
            fs::remove_file(e.data.path().join("archetypes.toml")).unwrap();
        }
        for (k, v) in &env {
            c.env(k, v);
        }
        drop(c);
        let err = refused(
            &e,
            &env,
            &[
                "ledger",
                "label",
                "F1",
                "--as",
                "L-COLLECT",
                "--evidence",
                "E4,E5",
            ],
        );
        assert_eq!(err["code"], "archetypes-off");
        let err = refused(&e, &env, &["ledger", "label", "F1", "--as", "N-WARM"]);
        assert_eq!(err["code"], "archetypes-off");
        let (code, s) = run_with(&e, &env, &["ledger", "label", "F1", "--suggest"]);
        assert_eq!(code, 0);
        assert_eq!(s["archetypes"], "off");
        assert_eq!(
            reload(&e)
                .finding(&"F1".parse().unwrap())
                .unwrap()
                .archetype,
            "UNCLASSIFIED"
        );
    }
    let (code, l) = run_with(
        &e,
        &[("DLA_ARCHETYPES", "off")],
        &[
            "ledger",
            "label",
            "F1",
            "--as",
            "UNCLASSIFIED",
            "--note",
            "holder is the table at a.go:1",
        ],
    );
    assert_eq!(code, 0, "{l}");
}

#[test]
fn a_holder_the_archetype_does_not_fit_is_refused_with_the_annotate_fix() {
    let e = env();
    let mut l = reload(&e);
    l.add_evidence(evidence(
        EvidenceKind::AllocStack,
        Authority::Live,
        "jfr",
        "alloc-profiler",
        "cc",
        json!({"mode": "diff", "finding": "F1", "top": []}),
    ))
    .unwrap();
    write_ledger(&e, &l);
    let err = refused(
        &e,
        &[],
        &[
            "ledger",
            "label",
            "F1",
            "--as",
            "L-HANDLE",
            "--evidence",
            "E4,E6",
        ],
    );
    assert_eq!(err["code"], "label-holder");
    assert!(
        err["fix"]
            .as_str()
            .unwrap()
            .contains("ledger annotate F1 --holder")
    );
}

// ----------------------------------------------------------------------------- strong

#[test]
fn strong_takes_the_independent_retention_record() {
    let e = env();
    let r = ok(&e, &["ledger", "strong", "F1", "--evidence", "E5"]);
    assert_eq!(r["to"], "strong");
    assert_eq!(
        reload(&e).finding(&"F1".parse().unwrap()).unwrap().state,
        FindingState::Strong
    );
}

#[test]
fn strong_refuses_what_is_not_a_second_class() {
    let e = env();
    let err = refused(&e, &[], &["ledger", "strong", "F1", "--evidence", "E3"]);
    assert_eq!(err["code"], "strong-second-class-missing");
    assert!(err["fix"].as_str().unwrap().contains("dla intervene"));
    let err = refused(&e, &[], &["ledger", "strong", "F1", "--evidence", "E9"]);
    assert_eq!(err["code"], "unknown-evidence");
    let err = refused(&e, &[], &["ledger", "strong", "F1.n1", "--evidence", "E5"]);
    assert!(
        ["illegal-transition", "strong-leak-only"].contains(&err["code"].as_str().unwrap()),
        "{err}"
    );
}

// ----------------------------------------------------------------------------- explain

#[test]
fn explain_checks_the_text_against_the_recorded_sites() {
    let e = env();
    let text_args = |extra: &[&'static str]| -> Vec<&'static str> {
        let mut a = vec![
            "ledger",
            "explain",
            "F1",
            "--tag",
            "inferred",
            "--removal",
            "never",
            "--text",
            GOOD,
        ];
        a.extend_from_slice(extra);
        a
    };
    let err = refused(&e, &[], &text_args(&["--adds", "a.go:9"]));
    assert_eq!(err["code"], "explain-no-writer");
    assert!(
        err["fix"]
            .as_str()
            .unwrap()
            .contains("ledger annotate F1 --writer")
    );
    ok(
        &e,
        &[
            "ledger",
            "annotate",
            "F1",
            "--writer",
            "a.go:9",
            "--remover",
            "a.go:40",
        ],
    );
    let err = refused(&e, &[], &text_args(&[]));
    assert_eq!(err["code"], "explain-remover-recorded");
    let x = ok(
        &e,
        &[
            "ledger",
            "explain",
            "F1",
            "--tag",
            "inferred",
            "--removal",
            "conditional",
            "--remover",
            "a.go:40",
            "--text",
            "Requests add entries to the static/global table at a.go:1 through put at a.go:9, and a.go:40 removes them only when the flag is set.",
        ],
    );
    assert_eq!(x["removal"], "conditional");
    assert_eq!(x["adds"], json!(["a.go:9"]));
    let f = reload(&e).finding(&"F1".parse().unwrap()).unwrap().clone();
    assert!(f.explanation.unwrap().text.contains("only when"));
    // `ledger add --explanation` is refused from schema 6.
    let err = refused(
        &e,
        &[],
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--explanation",
            "x",
            "--explanation-tag",
            "inferred",
        ],
    );
    assert_eq!(err["code"], "explanation-locked");
}

// ---------------------------------------------------------------------------- verify

fn bundle_for(e: &Env, finding: &str) -> String {
    use sha2::{Digest, Sha256};
    let dir = e.run_dir.join("bundle").join(finding);
    fs::create_dir_all(&dir).unwrap();
    let manifest = format!("{{\"finding\":\"{finding}\"}}");
    fs::write(dir.join("manifest.json"), &manifest).unwrap();
    hex::encode(Sha256::digest(manifest.as_bytes()))
}

#[test]
fn the_verifier_label_is_compared_with_the_findings() {
    let e = env();
    ok(
        &e,
        &[
            "ledger",
            "label",
            "F1",
            "--as",
            "L-COLLECT",
            "--evidence",
            "E4,E5",
        ],
    );
    let bundle = bundle_for(&e, "F1");
    let verify = |label: &str| {
        run_with(
            &e,
            &[],
            &[
                "ledger",
                "verify",
                "F1",
                "--site",
                "a.go:1",
                "--holder",
                "static/global",
                "--mechanism",
                "the table keeps every entry",
                "--label",
                label,
                "--bundle",
                &bundle,
            ],
        )
    };
    let (code, v) = verify("L-COLLECT");
    assert_eq!(
        (code, &v["agrees_label"], &v["verified"]),
        (0, &json!(true), &json!(true)),
        "{v}"
    );
    let (code, v) = verify("UNCLASSIFIED");
    assert_eq!(code, 0);
    assert_eq!(v["agrees_label"], false);
    assert_eq!(v["verified"], false);
    assert!(
        v["disagreement"]
            .as_str()
            .unwrap()
            .contains("label: finding L-COLLECT vs verifier UNCLASSIFIED"),
        "{v}"
    );
    let (code, v) = verify("L-NOPE");
    assert_eq!(code, 2);
    assert_eq!(v["error"]["code"], "unknown-archetype");
    // Archetypes off: only UNCLASSIFIED, compared with nothing.
    let (code, v) = run_with(
        &e,
        &[("DLA_ARCHETYPES", "off")],
        &[
            "ledger",
            "verify",
            "F1",
            "--site",
            "a.go:1",
            "--holder",
            "static/global",
            "--mechanism",
            "m",
            "--label",
            "L-COLLECT",
            "--bundle",
            &bundle,
        ],
    );
    assert_eq!(code, 2);
    assert_eq!(v["error"]["code"], "archetypes-off");
    let (code, v) = run_with(
        &e,
        &[("DLA_ARCHETYPES", "off")],
        &[
            "ledger",
            "verify",
            "F1",
            "--site",
            "a.go:1",
            "--holder",
            "static/global",
            "--mechanism",
            "m",
            "--bundle",
            &bundle,
        ],
    );
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["agrees_label"], Value::Null);
    assert_eq!(v["verified"], true);
}

#[allow(dead_code)]
fn unused(_: &Path) {}
