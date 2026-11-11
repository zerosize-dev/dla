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
    env_at("a.go:1")
}

/// The same run with the finding's site at `site` (its language names the runtime).
fn env_at(site: &str) -> Env {
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
            "passed": [{"column": "probe:CACHE", "region": "heap", "site": site, "finding": "F1", "shape": "steady"}]}),
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
    nf.site = Some(site.into());
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
    // E5 ties by its allocation site, but F1 does not cite it, so it is a record to name.
    let first = rows[0].as_str().unwrap();
    assert!(first.starts_with("L-COLLECT met [E4,E5]: "), "{first}");
    let err = refused(
        &e,
        &[],
        &[
            "ledger",
            "label",
            "F1",
            "--as",
            "L-COLLECT",
            "--note",
            "fits none of the go rows",
        ],
    );
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
            "--note",
            "fits none of the go rows",
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
                "--note",
                "fits none of the go rows",
            ],
        );
        assert_eq!(err["code"], "archetypes-off");
        let err = refused(&e, &env, &["ledger", "label", "F1", "--as", "N-WARM"]);
        assert_eq!(err["code"], "archetypes-off");
        let (code, s) = run_with(&e, &env, &["ledger", "label", "F1", "--suggest"]);
        assert_eq!(code, 0);
        assert_eq!(s["archetypes"], "off");
        // Even then the answer ends on the UNCLASSIFIED choice.
        let rows = s["rows"].as_array().unwrap();
        assert_eq!(rows.len(), 1, "{s}");
        assert!(
            rows[0]
                .as_str()
                .unwrap()
                .contains("--as UNCLASSIFIED --note")
        );
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
            "--note",
            "fits none of the go rows",
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
            "--note",
            "fits none of the go rows",
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

// ---------------------------------------------------------------------------- bundle

fn all_bundle_text(dir: &Path) -> String {
    let mut all = String::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for ent in fs::read_dir(d).unwrap() {
            let path = ent.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                all.push_str(&String::from_utf8_lossy(&fs::read(path).unwrap()));
            }
        }
    }
    all
}

/// Adds a sibling of F1 that cites E5 (the retention record), so F1's bundle carries it.
fn cite_retention(e: &Env) {
    let mut l = reload(e);
    let mut nf = NewFinding::new(
        Origin::parse("hypothesis").unwrap(),
        vec!["E5".parse().unwrap()],
    );
    nf.sibling_of = Some("F1".parse().unwrap());
    nf.hypothesis = Some("N-SCALE".into());
    nf.refute_if = Some("memory stays after load falls".into());
    l.add_finding(nf).unwrap();
    write_ledger(e, &l);
}

#[test]
fn a_labelled_explained_verified_findings_bundle_shows_none_of_that() {
    let e = env();
    cite_retention(&e);
    let src: String = (1..=60).map(|n| format!("int line{n};\n")).collect();
    fs::write(e.project.path().join("a.go"), &src).unwrap();
    // Not named by any evidence: must never become an excerpt.
    fs::write(e.project.path().join("z.go"), &src).unwrap();
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
            "--note",
            "fits none of the go rows",
        ],
    );
    ok(
        &e,
        &[
            "ledger",
            "annotate",
            "F1",
            "--writer",
            "a.go:9",
            "--note",
            "SECRET-NOTE",
        ],
    );
    ok(
        &e,
        &[
            "ledger",
            "explain",
            "F1",
            "--tag",
            "inferred",
            "--removal",
            "never",
            "--adds",
            "a.go:9",
            "--text",
            GOOD,
        ],
    );
    let first = ok(&e, &["bundle", "F1"]);
    let sha = first["manifest_sha256"].as_str().unwrap().to_owned();
    ok(
        &e,
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
            "L-COLLECT",
            "--bundle",
            &sha,
        ],
    );

    let v = ok(&e, &["bundle", "F1"]);
    let dir = e.project.path().join(v["dir"].as_str().unwrap());
    let all = all_bundle_text(&dir);
    for secret in [
        "L-COLLECT",
        "A collection gains entries",
        "SECRET-NOTE",
        GOOD,
        "no code removes them",
        "static/global table",
    ] {
        assert!(!all.contains(secret), "{secret} in the bundle: {all}");
    }
    let index: Value = serde_json::from_slice(&fs::read(dir.join("index.json")).unwrap()).unwrap();
    for key in [
        "site",
        "label",
        "explanation",
        "archetype",
        "annotation",
        "notes",
    ] {
        assert!(index.get(key).is_none(), "{key} in {index}");
    }
    // The excerpt is the frame the bundled retention evidence names, not the tied growth
    // column's site and not the finding's own site field.
    assert_eq!(index["frames"], json!(["a.go:2"]), "{index}");
    let excerpt = fs::read_to_string(dir.join("excerpt.txt")).unwrap();
    assert!(excerpt.starts_with("a.go:2 (lines 1-22)"), "{excerpt}");
    assert!(!excerpt.contains("z.go"));
}

#[test]
fn a_tied_growth_record_leaves_no_site_or_finding_id_in_the_bundle() {
    let e = env();
    cite_retention(&e);
    let src: String = (1..=60).map(|n| format!("int line{n};\n")).collect();
    fs::write(e.project.path().join("a.go"), &src).unwrap();
    // E4 is growth evidence whose column is tied to F1 by site `a.go:1` and by finding id.
    let v = ok(&e, &["bundle", "F1"]);
    let dir = e.project.path().join(v["dir"].as_str().unwrap());
    let all = all_bundle_text(&dir);
    assert!(all.contains("probe:CACHE"), "the column stays: {all}");
    for secret in ["a.go:1\"", "a.go:1 ", "\"F1\"", "tie"] {
        assert!(!all.contains(secret), "{secret} in the bundle: {all}");
    }
    let index: Value = serde_json::from_slice(&fs::read(dir.join("index.json")).unwrap()).unwrap();
    assert!(index.get("finding").is_none(), "{index}");
    let growth = index["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "growth-series")
        .unwrap();
    let col = &growth["result"]["passed"][0];
    for key in ["site", "finding", "candidates", "tie", "tied"] {
        assert!(col.get(key).is_none(), "{key} in {col}");
    }
}

#[test]
fn no_bundled_record_names_the_finding_or_its_site_outside_frame_fields() {
    let e = env();
    let src: String = (1..=60).map(|n| format!("int line{n};\n")).collect();
    fs::write(e.project.path().join("a.go"), &src).unwrap();
    fs::create_dir_all(e.run_dir.join("raw")).unwrap();
    fs::write(e.run_dir.join("raw/hist.json"), r#"{"finding":"F1","candidates":["F1"],"rows":[{"class":"Item","where":"a.go:1","note":"F1 grows"}]}"#).unwrap();
    fs::write(
        e.run_dir.join("raw/alloc.json"),
        r#"{"finding":"F1","stacks":[{"site":"a.go:1","note":"F1","frames":[{"function":"grow","file":"a.go","line":2}]}]}"#,
    )
    .unwrap();
    let mut l = reload(&e);
    let mut add =
        |kind: EvidenceKind, auth: Authority, src: &str, result: Value, raw: Option<&str>| {
            let mut n = evidence(kind, auth, "t", "x", src, result);
            n.raw = raw.map(str::to_owned);
            l.add_evidence(n).unwrap();
        };
    add(
        EvidenceKind::Discriminator,
        Authority::Test,
        "d1",
        json!({"test": "plateau", "favors": "leak", "supports": ["leak"],
               "basis": "F1 at a.go:1", "limitations": ["a.go:1 only; F1 untested"]}),
        None,
    );
    add(
        EvidenceKind::HistogramDiff,
        Authority::Live,
        "h1",
        json!({"finding": "F1", "candidates": ["F1"], "top": [{"class": "Item", "site": "a.go:1"}],
               "ties": [{"stack": "x", "finding": "F1"}]}),
        Some("raw/hist.json"),
    );
    add(
        EvidenceKind::AllocStack,
        Authority::Live,
        "a1",
        json!({"finding": "F1", "candidates": ["F1"], "ties": [{"finding": "F1"}],
               "top": [{"at": "grow", "site": "a.go:1"}],
               "stacks": [{"site": "a.go:1", "frames": [{"function": "grow", "file": "a.go", "line": 2}]}]}),
        Some("raw/alloc.json"),
    );
    let mut nf = NewFinding::new(
        Origin::parse("hypothesis").unwrap(),
        vec![
            "E5".parse().unwrap(),
            "E6".parse().unwrap(),
            "E7".parse().unwrap(),
            "E8".parse().unwrap(),
        ],
    );
    nf.sibling_of = Some("F1".parse().unwrap());
    nf.hypothesis = Some("N-SCALE".into());
    nf.refute_if = Some("memory stays after load falls".into());
    l.add_finding(nf).unwrap();
    write_ledger(&e, &l);
    let v = ok(&e, &["bundle", "F1"]);
    let dir = e.project.path().join(v["dir"].as_str().unwrap());
    let all = all_bundle_text(&dir);
    assert!(all.contains("Item") && all.contains("grow"), "{all}");
    for secret in [
        "F1",
        "a.go:1",
        "basis",
        "candidates",
        "\"ties\"",
        "\"finding\"",
    ] {
        assert!(!all.contains(secret), "{secret} in the bundle: {all}");
    }
    // The structured allocation frame stays.
    assert!(
        all.contains("\"file\": \"a.go\"") && all.contains("\"line\": 2"),
        "{all}"
    );
}

#[test]
fn no_evidence_frame_means_no_excerpt_even_with_a_site() {
    let e = env();
    fs::write(e.project.path().join("z.go"), "package z\n".repeat(30)).unwrap();
    let mut l = reload(&e);
    let mut nf = NewFinding::new(
        Origin::parse("census").unwrap(),
        vec!["E1".parse().unwrap()],
    );
    nf.site = Some("z.go:5".into());
    nf.holder = Some(Holder::StaticGlobal);
    l.add_finding(nf).unwrap();
    write_ledger(&e, &l);
    let v = ok(&e, &["bundle", "F2"]);
    assert_eq!(v["excerpt"], false, "{v}");
    let dir = e.project.path().join(v["dir"].as_str().unwrap());
    assert!(!dir.join("excerpt.txt").exists());
    let index: Value = serde_json::from_slice(&fs::read(dir.join("index.json")).unwrap()).unwrap();
    assert_eq!(index["frames"], json!([]));
    assert!(index.get("site").is_none());
    assert!(!all_bundle_text(&dir).contains("z.go"));
}

// ------------------------------------------------------------------- confirm: holder warning

/// Adds a leak-side finding at `site` with holder `holder`, its refuted N-WARM sibling and the
/// growth evidence `kind_in_column` ties to it (`None`: the column states no holder kind);
/// moves it to under_test and returns its id and the growth evidence id.
fn under_test_with_column(
    e: &Env,
    site: &str,
    holder: Holder,
    kind_in_column: Option<&str>,
) -> (String, String) {
    let mut l = reload(e);
    let next_f = l.findings().filter(|f| f.parent.is_none()).count() + 1;
    let fid = format!("F{next_f}");
    let mut col = json!({"column": format!("holder:{site}"), "region": "heap", "site": site,
        "finding": fid, "shape": "steady"});
    if let Some(k) = kind_in_column {
        col["holder_kind"] = json!(k);
    }
    let ev = l
        .add_evidence(evidence(
            EvidenceKind::GrowthSeries,
            Authority::Dump,
            "dla-hprof",
            "heap-graph",
            &format!("src-{fid}"),
            json!({"trend": "T9", "series": "S9", "n": 8, "gate": "pass", "failed": 0,
            "passed": [col]}),
        ))
        .unwrap();
    let ev_id = match ev.record {
        dla_core::Record::Evidence(x) => x.evidence_id,
        other => panic!("{other:?}"),
    };
    let mut nf = NewFinding::new(
        Origin::parse("census").unwrap(),
        vec!["E1".parse().unwrap()],
    );
    nf.site = Some(site.into());
    nf.holder = Some(holder);
    l.add_finding(nf).unwrap();
    let mut nf = NewFinding::new(
        Origin::parse("hypothesis").unwrap(),
        vec!["E1".parse().unwrap()],
    );
    nf.sibling_of = Some(fid.parse().unwrap());
    nf.hypothesis = Some("N-WARM".into());
    nf.refute_if = Some("growth continues".into());
    l.add_finding(nf).unwrap();
    let id = |s: &str| s.parse().unwrap();
    l.transition(
        &id(&format!("{fid}.n1")),
        FindingState::Refuted,
        None,
        None,
        vec!["E3".parse().unwrap()],
        "t",
    )
    .unwrap();
    l.transition(
        &id(&fid),
        FindingState::UnderTest,
        None,
        None,
        vec!["E2".parse().unwrap()],
        "t",
    )
    .unwrap();
    write_ledger(e, &l);
    (fid, ev_id.to_string())
}

#[test]
fn confirm_warns_when_the_tied_column_observed_another_holder() {
    let e = env();
    let (f2, e2) = under_test_with_column(
        &e,
        "b.go:1",
        Holder::StaticGlobal,
        Some("registration-table"),
    );
    let v = ok(&e, &["ledger", "confirm", &f2, "--evidence", &e2]);
    assert_eq!(v["to"], "confirmed", "{v}");
    let w = v["warning"].as_str().unwrap();
    assert!(w.contains("has holder static/global"), "{w}");
    assert!(w.contains("observed registration-table"), "{w}");
    assert!(
        w.contains(&format!(
            "dla ledger annotate {f2} --holder registration-table --evidence {e2}"
        )),
        "{w}"
    );
    // The command it names is accepted (a warning, not a dead end).
    ok(
        &e,
        &[
            "ledger",
            "annotate",
            &f2,
            "--holder",
            "registration-table",
            "--evidence",
            &e2,
        ],
    );
}

#[test]
fn confirm_is_silent_when_the_observed_holder_matches_or_is_not_stated() {
    let e = env();
    let (f2, e2) =
        under_test_with_column(&e, "b.go:1", Holder::StaticGlobal, Some("static/global"));
    let v = ok(&e, &["ledger", "confirm", &f2, "--evidence", &e2]);
    assert!(v.get("warning").is_none(), "{v}");
    let (f3, e3) = under_test_with_column(&e, "c.go:1", Holder::StaticGlobal, None);
    let v = ok(&e, &["ledger", "confirm", &f3, "--evidence", &e3]);
    assert!(v.get("warning").is_none(), "{v}");
}

// ------------------------------------------------------------------- suggest, every row

const EXTRA_ROWS: &str = r#"
[L-WORKER]
holders = ["blocked-stack"]
regions = ["heap"]
requires = [["intervention"]]
disconfirm = []
repair = ["stop/cancel"]
summary = "A worker is started for each unit of work and never stopped, so its stack stays."
"#;

fn add_alloc_stack(e: &Env) {
    let mut l = reload(e);
    l.add_evidence(evidence(
        EvidenceKind::AllocStack,
        Authority::Live,
        "jfr",
        "alloc-profiler",
        "cc",
        json!({"mode": "diff", "finding": "F1", "top": []}),
    ))
    .unwrap();
    write_ledger(e, &l);
}

#[test]
fn suggest_lists_met_then_missing_then_blocked_and_ends_with_unclassified() {
    let e = env();
    fs::write(
        e.data.path().join("archetypes.toml"),
        format!("{TABLE}{EXTRA_ROWS}"),
    )
    .unwrap();
    add_alloc_stack(&e);
    let s = ok(&e, &["ledger", "label", "F1", "--suggest"]);
    let rows: Vec<&str> = s["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap())
        .collect();
    assert_eq!(rows.len(), 4, "{s}");
    assert!(rows[0].starts_with("L-COLLECT met [E4,E5]: "), "{rows:?}");
    assert!(
        rows[1].starts_with("L-WORKER missing intervention: "),
        "{rows:?}"
    );
    // Blocked by the holder: the row names it.
    assert!(
        rows[2].starts_with(
            "L-HANDLE blocked, holder static/global is not among native-handle/global-ref: "
        ),
        "{rows:?}"
    );
    // Each row prints its full summary.
    assert!(rows[0].ends_with("A collection gains entries and nothing removes them."));
    assert!(rows[1].ends_with("never stopped, so its stack stays."));
    assert!(rows[2].ends_with("A native handle is acquired and never released."));
    // The command that produces the missing kind, and the fix for a wrong holder.
    assert!(
        s["produce"]["intervention"]
            .as_str()
            .unwrap()
            .starts_with("dla intervene <H> --finding F1"),
        "{s}"
    );
    assert!(
        s["holder_fix"]
            .as_str()
            .unwrap()
            .contains("dla ledger annotate F1 --holder"),
        "{s}"
    );
    // The last row is always UNCLASSIFIED and says how to choose it.
    assert!(rows[3].starts_with("UNCLASSIFIED "), "{rows:?}");
    assert!(rows[3].contains("compare each summary above with the mechanism you stated"));
    assert!(rows[3].contains("dla ledger label F1 --as UNCLASSIFIED --note"));
}

#[test]
fn suggest_with_the_shipped_table_lists_every_leak_archetype_within_the_bound() {
    let e = env();
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    fs::copy(
        shipped.join("archetypes.toml"),
        e.data.path().join("archetypes.toml"),
    )
    .unwrap();
    let table = dla_core::archetypes::load(&shipped).unwrap().unwrap();
    let leak: Vec<_> = table.entries().filter(|x| x.id.starts_with("L-")).collect();
    assert!(leak.len() > 6, "the old cap would hide rows");
    // The bound (2048 bytes, no `truncated`) is asserted by `ok`.
    let s = ok(&e, &["ledger", "label", "F1", "--suggest"]);
    assert!(s.get("truncated").is_none(), "{s}");
    let rows: Vec<&str> = s["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap())
        .collect();
    assert_eq!(rows.len(), leak.len() + 1, "{s}");
    for x in &leak {
        let row = rows
            .iter()
            .find(|r| r.starts_with(&format!("{} ", x.id)))
            .unwrap_or_else(|| panic!("{} is not listed: {rows:?}", x.id));
        assert!(row.ends_with(&x.summary), "{row} lacks the full summary");
    }
    assert!(rows.last().unwrap().starts_with("UNCLASSIFIED "));
}

#[test]
fn suggest_for_a_jvm_finding_with_only_growth_tied_keeps_the_rows_and_the_heap_graph_command() {
    // The shape of jvm-extension-reload: a confirmed jvm finding, only growth tied.
    let e = env_at("A.java:1");
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data");
    fs::copy(
        shipped.join("archetypes.toml"),
        e.data.path().join("archetypes.toml"),
    )
    .unwrap();
    let s = ok(&e, &["ledger", "label", "F1", "--suggest"]);
    assert!(s.get("truncated").is_none(), "{s}");
    let table = dla_core::archetypes::load(&shipped).unwrap().unwrap();
    let rows: Vec<&str> = s["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap())
        .collect();
    for x in table.entries().filter(|x| x.id.starts_with("L-")) {
        let row = rows
            .iter()
            .find(|r| r.starts_with(&format!("{} ", x.id)))
            .unwrap_or_else(|| panic!("{} is not listed: {rows:?}", x.id));
        assert!(row.ends_with(&x.summary), "{row} lacks the full summary");
    }
    let produce = s["produce"].to_string();
    assert!(
        produce.contains("dla summarize heap-graph"),
        "the retention-path producer is kept: {s}"
    );
    assert!(s["next"].as_str().is_some(), "the next step fits: {s}");
    // F1 is confirmed and cannot cite a non-heap record any more: the L-UNLOAD row names the
    // alternative it can still meet.
    let unload = rows
        .iter()
        .find(|r| r.starts_with("L-UNLOAD "))
        .unwrap_or_else(|| panic!("L-UNLOAD is not listed: {rows:?}"));
    assert!(
        unload.starts_with("L-UNLOAD missing retention-path"),
        "{unload}"
    );
}

#[test]
fn suggest_never_drops_next_for_the_sake_of_the_long_form_of_produce() {
    // The commands in `produce` shrink first: `next` is dropped only when it does not fit
    // even beside the short form of `produce`. The rows grow with a padded summary until
    // the answer is full; around that point no answer may lack `next` while the short
    // `produce` and `next` would have fitted.
    let e = env();
    let answer = |pad: usize| {
        let table = format!(
            "{TABLE}\n[L-STALE]\nholders = [\"static/global\"]\nregions = [\"heap\"]\nrequires = [[\"growth-series\", \"should-be-dead\"]]\ndisconfirm = []\nrepair = [\"deregister\"]\nsummary = \"{}\"\n",
            "x".repeat(pad)
        );
        fs::write(e.data.path().join("archetypes.toml"), table).unwrap();
        let (code, s) = run_with(&e, &[], &["ledger", "label", "F1", "--suggest"]);
        assert_eq!(code, 0, "pad {pad}: {s}");
        s
    };
    let first_without_next = (0..1900)
        .step_by(20)
        .find(|&p| answer(p).get("next").is_none())
        .expect("the answer never filled up");
    let mut long_forms = 0;
    for pad in first_without_next.saturating_sub(60)..first_without_next + 20 {
        let mut s = answer(pad);
        if s.get("next").is_some() {
            continue;
        }
        let long = s["produce"].to_string().contains("--finding F1");
        long_forms += usize::from(long);
        // The same answer with the short produce and the short next.
        if let Some(p) = s.get_mut("produce").and_then(Value::as_object_mut) {
            for v in p.values_mut() {
                *v = json!(v.as_str().unwrap().replace(" --finding F1", ""));
            }
        }
        s["next"] = json!("dla ledger label F1 --as <ID> --evidence E..");
        assert!(
            s.to_string().len() >= dla_core::MAX_OUTPUT_BYTES,
            "pad {pad}: next was dropped although it fits beside the short produce"
        );
    }
    assert!(
        long_forms > 0,
        "the scan never met the long form without next"
    );
}

// ------------------------------------------------------------- label and the runtime

#[test]
fn a_label_with_no_row_in_the_runtime_reference_is_refused_without_a_note() {
    let e = env();
    // F1's site is a Go file; the go reference has the L-WORKER row only.
    let before = reload(&e).lines().len();
    let r = refused(
        &e,
        &[],
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
    assert_eq!(r["code"], "label-runtime-row", "{r}");
    let m = r["message"].as_str().unwrap();
    assert!(
        m.contains("go archetype reference") && m.contains("L-WORKER"),
        "{m}"
    );
    let fix = r["fix"].as_str().unwrap();
    assert!(
        fix.contains("UNCLASSIFIED") && fix.contains("--note"),
        "{fix}"
    );
    assert_eq!(reload(&e).lines().len(), before, "nothing is written");
    assert_eq!(
        reload(&e)
            .finding(&"F1".parse().unwrap())
            .unwrap()
            .archetype,
        "UNCLASSIFIED"
    );
}

#[test]
fn a_note_keeps_a_label_with_no_row_in_the_runtime_reference() {
    let e = env();
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
            "--note",
            "a map keyed by request id; the go worker row is a goroutine pool, not this",
        ],
    );
    assert_eq!(l["label"], "L-COLLECT");
    let led = reload(&e);
    assert_eq!(
        led.finding(&"F1".parse().unwrap()).unwrap().archetype,
        "L-COLLECT"
    );
    let rec = led.labels(&"F1".parse().unwrap());
    assert!(
        rec.last()
            .unwrap()
            .note
            .as_deref()
            .is_some_and(|n| n.contains("goroutine pool")),
        "the note is stored with the label"
    );
}

#[test]
fn archetypes_off_is_refused_before_the_runtime_row_check() {
    // F1 is a go finding and L-COLLECT is no go row: with archetypes off the refusal is
    // archetypes-off, which no --note can get past, not label-runtime-row.
    let e = env();
    let r = refused(
        &e,
        &[("DLA_ARCHETYPES", "off")],
        &["ledger", "label", "F1", "--as", "L-COLLECT"],
    );
    assert_eq!(r["code"], "archetypes-off", "{r}");
}

#[test]
fn the_labels_own_state_and_polarity_come_before_the_runtime_row_check() {
    let e = env();
    // F1.n1 is a refuted non-leak sibling: its state is the first thing wrong.
    let r = refused(&e, &[], &["ledger", "label", "F1.n1", "--as", "L-COLLECT"]);
    assert_eq!(r["code"], "label-state", "{r}");
    // An N-* label on the leak-side finding: polarity, not the runtime row.
    let r = refused(&e, &[], &["ledger", "label", "F1", "--as", "N-WARM"]);
    assert_ne!(r["code"], "label-runtime-row", "{r}");
    // A finding that is not confirmed yet.
    let mut l = reload(&e);
    let mut nf = NewFinding::new(
        Origin::parse("census").unwrap(),
        vec!["E1".parse().unwrap()],
    );
    nf.site = Some("b.go:5".into());
    l.add_finding(nf).unwrap();
    write_ledger(&e, &l);
    let r = refused(&e, &[], &["ledger", "label", "F2", "--as", "L-COLLECT"]);
    assert_eq!(r["code"], "label-state", "{r}");
}

/// A confirmed finding of the jvm worker at `site` with the growth and the allocation
/// profile L-HANDLE needs. Returns the environment, the finding's bare ID and the evidence
/// list to cite.
fn jvm_worker_handle_case(site: &str) -> (Env, String, String) {
    let e = env();
    let out = dla(&e)
        .args(["run", "dispatch", "--workers", "jvm"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let mut l = Ledger::load_for(&e.run_dir, Some("jvm")).unwrap();
    let mut add = |k, a, tool: &str, class: &str, src: &str, r: Value| {
        let line = l.add_evidence(evidence(k, a, tool, class, src, r)).unwrap();
        let dla_core::Record::Evidence(rec) = line.record else {
            unreachable!()
        };
        rec.evidence_id
    };
    let hit = add(
        EvidenceKind::StaticHit,
        Authority::Static,
        "t",
        "x",
        "s1",
        json!({}),
    );
    let work = add(
        EvidenceKind::Workload,
        Authority::Test,
        "t",
        "x",
        "s2",
        json!({"harness": "H1", "mode": "command", "runtime": "jvm"}),
    );
    let disc = add(
        EvidenceKind::Discriminator,
        Authority::Test,
        "dla",
        "x",
        "s3",
        json!({"test": "plateau", "favors": "leak", "supports": ["leak"], "finding": null}),
    );
    let growth = add(
        EvidenceKind::GrowthSeries,
        Authority::Test,
        "dla-trend",
        "harness-series",
        "aa",
        json!({"trend": "T1", "series": "S1", "n": 8, "gate": "pass", "failed": 0,
        "passed": [{"column": "probe:BUF", "region": "off-heap", "site": site, "finding": "F1", "shape": "steady"}]}),
    );
    let alloc = add(
        EvidenceKind::AllocStack,
        Authority::Live,
        "jfr",
        "alloc-profiler",
        "cc",
        json!({"mode": "diff", "finding": "jvm/F1", "top": []}),
    );
    let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![hit]);
    nf.site = Some(site.into());
    nf.holder = Some(Holder::NativeHandleGlobalRef);
    let line = l.add_finding(nf).unwrap();
    let dla_core::Record::FindingAdded { finding } = line.record else {
        unreachable!()
    };
    let f = finding.id;
    let mut sib = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![disc.clone()]);
    sib.sibling_of = Some(f.clone());
    sib.hypothesis = Some("N-WARM".into());
    sib.refute_if = Some("growth continues".into());
    let line = l.add_finding(sib).unwrap();
    let dla_core::Record::FindingAdded { finding: n } = line.record else {
        unreachable!()
    };
    l.transition(&n.id, FindingState::Refuted, None, None, vec![disc], "t")
        .unwrap();
    l.transition(&f, FindingState::UnderTest, None, None, vec![work], "t")
        .unwrap();
    l.transition(
        &f,
        FindingState::Confirmed,
        None,
        None,
        vec![growth.clone()],
        "t",
    )
    .unwrap();
    drop(l);
    let fid = f.to_string();
    let bare = fid.split('/').next_back().unwrap();
    let ev = format!("{growth},{alloc}");
    (e, bare.to_owned(), ev)
}

fn label_handle(e: &Env, bare: &str, ev: &str, extra: &[&str]) -> (i32, Value) {
    let mut args = vec![
        "--worker",
        "jvm",
        "ledger",
        "label",
        bare,
        "--as",
        "L-HANDLE",
        "--evidence",
        ev,
    ];
    args.extend(extra);
    run_with(e, &[], &args)
}

#[test]
fn a_worker_finding_with_a_c_site_is_labelled_by_the_language_of_its_site() {
    // A C finding of the jvm worker takes the native rows (L-HANDLE), without a note,
    // although its worker is jvm.
    let (e, bare, ev) = jvm_worker_handle_case("codec.c:7");
    let (code, v) = label_handle(&e, &bare, &ev, &[]);
    assert_eq!(
        code, 0,
        "no note needed for a C finding of the jvm worker: {v}"
    );
    assert_eq!(v["label"], "L-HANDLE");
}

#[test]
fn a_jvm_finding_at_a_rust_site_needs_a_note_for_l_handle() {
    // jni-order-codec: the JNI code is Rust (native/src/lib.rs), so the runtime is rust,
    // whose reference has no L-HANDLE row. The designed escape is --note.
    let (e, bare, ev) = jvm_worker_handle_case("native/src/lib.rs:7");
    let (code, v) = label_handle(&e, &bare, &ev, &[]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "label-runtime-row", "{v}");
    assert!(
        v["error"]["message"].as_str().unwrap().contains("rust"),
        "{v}"
    );
    // The refusal names the library boundary and keeps the label with a note.
    let m = v["error"]["message"].as_str().unwrap();
    assert!(
        m.contains("jvm worker finding")
            && m.contains("native/src/lib.rs")
            && m.contains("JNI or FFI"),
        "{v}"
    );
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(
        fix.contains("--as L-HANDLE --note \"library boundary:"),
        "{v}"
    );
    // A finding of the same language as its worker gets the plain text.
    let (e2, bare2, ev2) = jvm_worker_handle_case("kotlin/shop/Audit.kt:7");
    let (code2, v2) = label_handle(&e2, &bare2, &ev2, &[]);
    assert_eq!(code2, 2, "{v2}");
    assert!(
        !v2["error"]["message"].as_str().unwrap().contains("JNI"),
        "{v2}"
    );
    let (code, v) = label_handle(
        &e,
        &bare,
        &ev,
        &["--note", "JNI handle table in Rust code; no rust row fits"],
    );
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["label"], "L-HANDLE");
}

/// A jvm worker ledger whose leak-side finding is refuted and whose N-WARM sibling is
/// confirmed; returns the environment and the sibling's bare ID (`F1.n1`).
fn jvm_worker_confirmed_nonleak_case() -> (Env, String) {
    let e = env();
    let out = dla(&e)
        .args(["run", "dispatch", "--workers", "jvm"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let mut l = Ledger::load_for(&e.run_dir, Some("jvm")).unwrap();
    let mut add = |k, a, tool: &str, class: &str, src: &str, r: Value| {
        let line = l.add_evidence(evidence(k, a, tool, class, src, r)).unwrap();
        let dla_core::Record::Evidence(rec) = line.record else {
            unreachable!()
        };
        rec.evidence_id
    };
    let hit = add(
        EvidenceKind::StaticHit,
        Authority::Static,
        "t",
        "x",
        "s1",
        json!({}),
    );
    let work = add(
        EvidenceKind::Workload,
        Authority::Test,
        "t",
        "x",
        "s2",
        json!({"harness": "H1", "mode": "command", "runtime": "jvm"}),
    );
    let disc = add(
        EvidenceKind::Discriminator,
        Authority::Test,
        "dla",
        "x",
        "s3",
        json!({"test": "plateau", "favors": "non-leak", "supports": ["non-leak"], "finding": null}),
    );
    let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![hit]);
    nf.site = Some("A.java:3".into());
    nf.holder = Some(Holder::StaticGlobal);
    let line = l.add_finding(nf).unwrap();
    let dla_core::Record::FindingAdded { finding } = line.record else {
        unreachable!()
    };
    let f = finding.id;
    let mut sib = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![disc.clone()]);
    sib.sibling_of = Some(f.clone());
    sib.hypothesis = Some("N-WARM".into());
    sib.refute_if = Some("growth continues".into());
    let line = l.add_finding(sib).unwrap();
    let dla_core::Record::FindingAdded { finding: n } = line.record else {
        unreachable!()
    };
    l.transition(
        &f,
        FindingState::UnderTest,
        None,
        None,
        vec![work.clone()],
        "t",
    )
    .unwrap();
    l.transition(
        &f,
        FindingState::Refuted,
        None,
        None,
        vec![disc.clone()],
        "t",
    )
    .unwrap();
    l.transition(&n.id, FindingState::UnderTest, None, None, vec![work], "t")
        .unwrap();
    l.transition(&n.id, FindingState::Confirmed, None, None, vec![disc], "t")
        .unwrap();
    drop(l);
    let nid = n.id.to_string();
    (e, nid.split('/').next_back().unwrap().to_owned())
}

#[test]
fn the_labels_polarity_comes_before_the_runtime_row_check() {
    // A confirmed non-leak sibling of the jvm worker: L-HANDLE has no jvm row, but the
    // first thing wrong is that an L-* label does not fit a non-leak finding.
    let (e, n) = jvm_worker_confirmed_nonleak_case();
    let (code, v) = run_with(
        &e,
        &[],
        &["--worker", "jvm", "ledger", "label", &n, "--as", "L-HANDLE"],
    );
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "label-polarity", "{v}");
}

#[test]
fn an_unknown_finding_is_reported_before_the_runtime_row_check() {
    let (e, _, ev) = jvm_worker_handle_case("A.java:3");
    let (code, v) = label_handle(&e, "F99", &ev, &[]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "unknown-finding", "{v}");
}

// --------------------------------------------------------------------- site lines

fn write_src(e: &Env, rel: &str, text: &str) {
    let p = e.project.path().join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

/// 1 line comment, 2 class, 3-4 block comment, 5 blank, 6 signature, 7 holder, 8-9 braces.
const BOARD: &str = "// board\nclass Board {\n  /* header\n  still comment */\n\n  add(item) {\n    this.previous = item;\n  }\n}\n";

fn lines_in_ledger(e: &Env) -> usize {
    reload(e).lines().len()
}

#[test]
fn annotate_writer_refuses_a_comment_blank_or_past_the_end_and_points_to_grep() {
    let e = env();
    write_src(&e, "src/board.js", BOARD);
    let before = lines_in_ledger(&e);
    for (site, what) in [
        ("src/board.js:1", "a comment"),
        ("src/board.js:3", "a comment"),
        ("src/board.js:4", "a comment"),
        ("src/board.js:5", "blank"),
        ("src/board.js:10", "outside src/board.js"),
        ("src/board.js:99", "outside src/board.js"),
    ] {
        let err = refused(
            &e,
            &[],
            &[
                "ledger",
                "annotate",
                "F1",
                "--writer",
                site,
                "--owner",
                "Board.previous",
            ],
        );
        assert_eq!(err["code"], "invalid-site", "{site}");
        assert!(
            err["message"].as_str().unwrap().contains(what),
            "{site}: {err}"
        );
        let fix = err["fix"].as_str().unwrap();
        assert!(fix.contains("grep -n 'previous' src/board.js"), "{fix}");
    }
    assert_eq!(lines_in_ledger(&e), before, "a refusal writes nothing");
}

#[test]
fn annotate_writer_accepts_a_code_line_that_holds_the_identifier() {
    let e = env();
    write_src(&e, "src/board.js", BOARD);
    let v = ok(
        &e,
        &[
            "ledger",
            "annotate",
            "F1",
            "--writer",
            "src/board.js:7",
            "--owner",
            "Board.previous",
        ],
    );
    assert_eq!(v["writers"], 1);
    assert!(v.get("warnings").is_none(), "{v}");
}

#[test]
fn annotate_writer_warns_when_the_line_lacks_the_identifier_and_names_the_nearest_lines() {
    let e = env();
    write_src(&e, "src/board.js", BOARD);
    // Line 6 is the signature, line 7 holds the field.
    let v = ok(
        &e,
        &[
            "ledger",
            "annotate",
            "F1",
            "--writer",
            "src/board.js:6",
            "--owner",
            "Board.previous",
        ],
    );
    assert_eq!(v["writers"], 1, "a warning does not refuse");
    let w = v["warnings"][0].as_str().unwrap();
    assert!(w.contains("does not contain `previous`"), "{w}");
    assert!(
        w.contains("7:") && w.contains("this.previous = item;"),
        "{w}"
    );
}

#[test]
fn annotate_checks_the_identifier_of_the_owner_the_finding_already_has() {
    let e = env();
    write_src(&e, "src/board.js", BOARD);
    ok(
        &e,
        &["ledger", "annotate", "F1", "--owner", "Board.previous"],
    );
    let v = ok(
        &e,
        &["ledger", "annotate", "F1", "--writer", "src/board.js:6"],
    );
    assert!(
        v["warnings"][0].as_str().unwrap().contains("`previous`"),
        "{v}"
    );
}

#[test]
fn site_lines_use_the_comment_syntax_of_the_language() {
    let e = env();
    write_src(
        &e,
        "src/a.py",
        "# helper\nitems = []\n\ndef put(x):\n    items.append(x)\n",
    );
    let err = refused(
        &e,
        &[],
        &[
            "ledger",
            "annotate",
            "F1",
            "--writer",
            "src/a.py:1",
            "--owner",
            "items",
        ],
    );
    assert!(
        err["message"].as_str().unwrap().contains("a comment"),
        "{err}"
    );
    // `//` is not a comment in Python, and a `#` inside a string is code.
    write_src(
        &e,
        "src/b.py",
        "// not a comment in python\nurl = \"http://x\"  # tail\n",
    );
    ok(
        &e,
        &[
            "ledger",
            "annotate",
            "F1",
            "--writer",
            "src/b.py:1",
            "--owner",
            "items",
        ],
    );
    ok(
        &e,
        &[
            "ledger",
            "annotate",
            "F1",
            "--writer",
            "src/b.py:2",
            "--owner",
            "items",
        ],
    );
    let v = ok(
        &e,
        &[
            "ledger",
            "annotate",
            "F1",
            "--writer",
            "src/a.py:5",
            "--owner",
            "items",
        ],
    );
    assert!(v.get("warnings").is_none(), "{v}");
}

#[test]
fn a_site_in_a_file_the_project_lacks_is_noted_not_refused() {
    let e = env();
    let v = ok(
        &e,
        &["ledger", "annotate", "F1", "--writer", "src/nowhere.js:3"],
    );
    assert!(
        v["warnings"][0]
            .as_str()
            .unwrap()
            .contains("not found under the project"),
        "{v}"
    );
}

#[test]
fn a_site_outside_the_project_is_never_read() {
    let e = env();
    // A file one level above the project that holds the owner's identifier: were it read,
    // its line would be quoted back in the warning.
    let name = format!(
        "{}-outside.js",
        e.project.path().file_name().unwrap().to_str().unwrap()
    );
    let outside = e.project.path().parent().unwrap().join(name);
    fs::write(&outside, "const previous = 1;\n").unwrap();
    let up = format!("../{}:1", outside.file_name().unwrap().to_str().unwrap());
    let abs = format!("{}:1", outside.display());
    for site in [up.as_str(), abs.as_str()] {
        let v = ok(&e, &["ledger", "annotate", "F1", "--writer", site]);
        let w = v["warnings"][0].as_str().unwrap();
        assert!(w.contains("not found under the project"), "{site}: {v}");
        assert!(!w.contains("const previous"), "{site}: {v}");
    }
    fs::remove_file(&outside).unwrap();
}

#[test]
fn ledger_add_checks_the_site_line_too() {
    let e = env();
    write_src(&e, "src/board.js", BOARD);
    let before = lines_in_ledger(&e);
    for site in ["src/board.js:1", "src/board.js:5", "src/board.js:12"] {
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
                "--site",
                site,
                "--owner",
                "Board.previous",
            ],
        );
        assert_eq!(err["code"], "invalid-site", "{site}");
        assert!(err["fix"].as_str().unwrap().contains("grep -n 'previous'"));
    }
    assert_eq!(lines_in_ledger(&e), before);
    let v = ok(
        &e,
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            "src/board.js:6",
            "--owner",
            "Board.previous",
        ],
    );
    assert!(v["warnings"][0].as_str().unwrap().contains("7:"), "{v}");
    let v = ok(
        &e,
        &[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            "src/board.js:7",
            "--owner",
            "Board.previous",
        ],
    );
    assert!(v.get("warnings").is_none(), "{v}");
}

#[test]
fn duplicate_refuses_a_finding_against_itself_and_an_unknown_one() {
    let e = env();
    let r = refused(&e, &[], &["ledger", "duplicate", "F1", "--of", "F1"]);
    assert_eq!(r["code"], "duplicate-state", "{r}");
    let r = refused(&e, &[], &["ledger", "duplicate", "F1", "--of", "F77"]);
    assert_eq!(r["code"], "unknown-finding", "{r}");
}
