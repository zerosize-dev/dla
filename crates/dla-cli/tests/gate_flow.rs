//! `dla gate check` through a run: the stages fail and then pass in order, the final gate
//! fails on a missing explanation and passes after `dla ledger explain`; the integrity seals
//! catch an edit made outside `dla` and `dla run reseal` accepts it.

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Authority, EvidenceKind, FindingState, Ledger, NewEvidence, NewFinding, Origin, Source, Tool,
    evidence::{EvidenceCoverage, Signal},
    finding::Holder,
};
use serde_json::{Value, json};

struct Run {
    dir: tempfile::TempDir,
    run_id: String,
}

#[derive(Debug)]
struct Out {
    code: i32,
    stdout: String,
    stderr: String,
    json: Value,
}

fn base(project: &Path) -> Command {
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
    fn path(&self) -> &Path {
        self.dir.path()
    }
    fn run_dir(&self) -> std::path::PathBuf {
        self.path().join(".dla").join(&self.run_id)
    }
    fn run(&self, args: &[&str]) -> Out {
        let o = base(self.path()).args(args).output().unwrap();
        let stdout = String::from_utf8(o.stdout).unwrap();
        Out {
            code: o.status.code().unwrap(),
            json: serde_json::from_str(&stdout).unwrap_or(Value::Null),
            stdout,
            stderr: String::from_utf8(o.stderr).unwrap(),
        }
    }
    fn ok(&self, args: &[&str]) -> Value {
        let o = self.run(args);
        assert_eq!(o.code, 0, "{args:?}: {} {}", o.stdout, o.stderr);
        o.json
    }
    fn state(&self) -> Value {
        serde_json::from_str(&fs::read_to_string(self.run_dir().join("state.json")).unwrap())
            .unwrap()
    }
    fn gate(&self, stage: &str) -> Out {
        if stage == "final" {
            self.run(&["gate", "check", "--final"])
        } else {
            self.run(&["gate", "check", "--stage", stage])
        }
    }
    fn answer_intake(&self) {
        for item in [
            "intake.target",
            "intake.scope",
            "intake.symptom",
            "intake.runtimes",
            "intake.build",
            "intake.workload",
            "intake.consent",
            "intake.budget",
        ] {
            self.ok(&[
                "ledger",
                "resolve",
                item,
                "--status",
                "answered",
                "--evidence",
                "E1",
            ]);
        }
    }
}

fn init(entry: &str) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let o = base(dir.path())
        .env("DLA_CONSENT_TIERS", "1,2")
        .args(["run", "init", "--session-id", "t", "--entry", entry])
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    Run {
        run_id: v["run_id"].as_str().unwrap().to_owned(),
        dir,
    }
}

fn missing_ids(o: &Out) -> Vec<String> {
    o.json["missing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| m["id"].as_str().unwrap().to_owned())
        .collect()
}

// ------------------------------------------------------------- stages in order

#[test]
fn stages_fail_and_then_pass_through_a_swept_project() {
    let r = init("repository");
    fs::write(
        r.path().join("app.py"),
        "_seen = []\n_cache = {}\n\ndef handle(req):\n    _seen.append(req)\n    _cache[req] = len(_seen)\n    return len(_seen)\n",
    )
    .unwrap();

    // intake: the checklist is open.
    let o = r.gate("intake");
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert_eq!(o.json["ok"], false);
    assert_eq!(o.json["stage"], "intake");
    assert_eq!(o.json["missing_total"], 8);
    assert_eq!(o.json["missing"].as_array().unwrap().len(), 5);
    assert!(o.stderr.contains("intake.target"), "{}", o.stderr);
    assert!(
        o.json["pointer"]
            .as_str()
            .unwrap()
            .ends_with("/gate/intake.json")
    );
    // The full list is in the file.
    let full: Value =
        serde_json::from_str(&fs::read_to_string(r.run_dir().join("gate/intake.json")).unwrap())
            .unwrap();
    assert_eq!(full["missing"].as_array().unwrap().len(), 8);
    assert!(r.state()["stages_passed"].is_null());
    r.answer_intake();
    let o = r.gate("intake");
    assert_eq!(o.code, 0, "{}", o.stdout);
    assert_eq!(o.json["ok"], true);
    assert_eq!(o.json["recorded"], true);
    assert_eq!(r.state()["stages_passed"], json!(["intake"]));
    assert_eq!(r.state()["stage"], "probe");

    // probe: no probe evidence yet.
    let o = r.gate("probe");
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert!(missing_ids(&o).contains(&r.run_id), "{o:?}");
    assert_eq!(o.json["missing"][0]["fix"], "dla probe");
    r.ok(&["probe"]);
    assert_eq!(r.gate("probe").code, 0);

    // sweep: no census and no sweep row yet.
    let o = r.gate("sweep");
    assert_eq!(o.code, 2, "{}", o.stdout);
    let ids = missing_ids(&o);
    assert!(ids.contains(&"python/census".to_owned()), "{ids:?}");
    r.ok(&["sweep", "--lang", "python"]);
    r.ok(&["census", "--lang", "python"]);
    let o = r.gate("sweep");
    assert_eq!(o.code, 0, "{} {}", o.stdout, o.stderr);

    // The later stages have nothing to say about a project with no growth series.
    for stage in ["classify", "confirm", "attribute", "rule_out"] {
        let o = r.gate(stage);
        assert_eq!(o.code, 0, "{stage}: {} {}", o.stdout, o.stderr);
    }
    assert_eq!(
        r.state()["stages_passed"],
        json!([
            "intake",
            "probe",
            "sweep",
            "classify",
            "confirm",
            "attribute",
            "rule_out"
        ])
    );

    // final: the candidates have no disposition.
    let o = r.gate("final");
    assert_eq!(o.code, 2, "{}", o.stdout);
    assert_eq!(o.json["stage"], "final");
    let ledger = Ledger::load(&r.run_dir()).unwrap();
    let candidates: Vec<String> = ledger
        .findings()
        .filter(|f| f.state == FindingState::Candidate)
        .map(|f| f.id.to_string())
        .collect();
    assert!(candidates.len() >= 2, "{candidates:?}");
    let listed = missing_ids(&o);
    assert!(listed.contains(&candidates[0]), "{listed:?}");
    // The derived counts were written back, none dispositioned yet.
    let cov: Value =
        serde_json::from_str(&fs::read_to_string(r.run_dir().join("coverage.json")).unwrap())
            .unwrap();
    assert_eq!(
        cov["workers"]["python"]["backends"]["dla"]["dispositioned"],
        0
    );
    for id in &candidates {
        r.ok(&[
            "ledger",
            "resolve",
            id,
            "--as",
            "unresolved",
            "--reason",
            "not_exercised",
            "--remedy",
            "write a workload that calls handle",
        ]);
    }
    let o = r.gate("final");
    assert_eq!(o.code, 0, "{} {}", o.stdout, o.stderr);
    assert_eq!(o.json["ok"], true);
    let cov: Value =
        serde_json::from_str(&fs::read_to_string(r.run_dir().join("coverage.json")).unwrap())
            .unwrap();
    let dla = &cov["workers"]["python"]["backends"]["dla"];
    assert_eq!(dla["enumerated"], dla["dispositioned"], "{cov}");
    assert!(dla["dispositioned"].as_u64().unwrap() >= 1);
    let census = &cov["workers"]["python"]["backends"]["census"];
    assert_eq!(census["enumerated"], census["dispositioned"], "{cov}");
    // run status lists the same blocking items as the gate (here: none).
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["complete"], true, "{st}");
    assert_eq!(st["blocking_total"], 0);
}

#[test]
fn a_hit_without_a_candidate_is_reported() {
    let r = init("repository");
    r.answer_intake();
    r.ok(&["probe"]);
    r.ok(&["sweep", "--lang", "python"]);
    r.ok(&["census", "--lang", "python"]);
    // Claim two hits that never became candidates.
    let p = r.run_dir().join("coverage.json");
    let mut cov: Value = serde_json::from_str(&fs::read_to_string(&p).unwrap()).unwrap();
    cov["workers"]["python"]["backends"]["dla"]["enumerated"] = json!(2);
    fs::write(&p, serde_json::to_string_pretty(&cov).unwrap()).unwrap();
    let o = r.gate("sweep");
    assert_eq!(o.code, 2, "{}", o.stdout);
    let m = o.json["missing"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["kind"] == "accounting")
        .expect("an accounting item");
    assert!(
        m["description"]
            .as_str()
            .unwrap()
            .contains("hits without a candidate"),
        "{m}"
    );
}

// ---------------------------------------------------------------- final gate

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

/// Adds, through the ledger API, a confirmed leak-side finding F1 with its refuted
/// N-WARM sibling, to a symptom run.
fn confirmed_finding(r: &Run) {
    let mut l = Ledger::load(&r.run_dir()).unwrap();
    l.add_evidence(evidence(
        EvidenceKind::StaticHit,
        Authority::Static,
        "t",
        "x",
        "s1",
        json!({}),
    ))
    .unwrap();
    l.add_evidence(evidence(
        EvidenceKind::Workload,
        Authority::Test,
        "t",
        "x",
        "s2",
        json!({"harness": "H1", "mode": "command", "runtime": "go"}),
    ))
    .unwrap();
    l.add_evidence(evidence(
        EvidenceKind::Discriminator,
        Authority::Test,
        "dla",
        "x",
        "s3",
        json!({"test": "plateau", "favors": "leak", "supports": ["leak"], "finding": null}),
    ))
    .unwrap();
    l.add_evidence(evidence(
        EvidenceKind::GrowthSeries,
        Authority::Test,
        "dla-trend",
        "harness-series",
        "aa",
        json!({"trend": "T1", "series": "S1", "n": 8, "gate": "pass", "failed": 0,
        "passed": [{"column": "probe:CACHE", "region": "heap", "site": "a.go:1", "finding": "F1", "shape": "steady"}]}),
    ))
    .unwrap();
    // The ids follow the evidence the run itself wrote (run config and consent).
    let first = l
        .evidence()
        .find(|x| x.kind == EvidenceKind::StaticHit)
        .unwrap()
        .evidence_id
        .clone();
    let harness = l
        .evidence()
        .find(|x| x.kind == EvidenceKind::Workload)
        .unwrap()
        .evidence_id
        .clone();
    let disc = l
        .evidence()
        .find(|x| x.kind == EvidenceKind::Discriminator)
        .unwrap()
        .evidence_id
        .clone();
    let growth = l
        .evidence()
        .find(|x| x.kind == EvidenceKind::GrowthSeries)
        .unwrap()
        .evidence_id
        .clone();
    let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![first.clone()]);
    nf.site = Some("a.go:1".into());
    nf.holder = Some(Holder::StaticGlobal);
    l.add_finding(nf).unwrap();
    let mut nf = NewFinding::new(Origin::parse("hypothesis").unwrap(), vec![first]);
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
        vec![disc],
        "t",
    )
    .unwrap();
    l.transition(
        &id("F1"),
        FindingState::UnderTest,
        None,
        None,
        vec![harness],
        "t",
    )
    .unwrap();
    l.transition(
        &id("F1"),
        FindingState::Confirmed,
        None,
        None,
        vec![growth],
        "t",
    )
    .unwrap();
}

#[test]
fn the_final_gate_needs_an_explanation_for_a_confirmed_finding() {
    let r = init("symptom");
    r.answer_intake();
    confirmed_finding(&r);
    // The ledger written through the API is sealed: no tamper.
    assert_eq!(r.ok(&["run", "status"])["tamper"], json!([]));
    let o = r.gate("final");
    assert_eq!(o.code, 2, "{}", o.stdout);
    let m = &o.json["missing"][0];
    assert_eq!(m["kind"], "explanation", "{o:?}");
    assert_eq!(m["id"], "F1");
    assert!(m["fix"].as_str().unwrap().contains("dla ledger explain F1"));
    // run status shows the same blocker.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["complete"], false);
    assert_eq!(st["blocking"][0]["id"], "F1");
    // The stop hook blocks on it too.
    let stop = base(r.path())
        .args(["gate", "stop-hook"])
        .write_stdin(json!({"cwd": r.path(), "hook_event_name": "Stop"}).to_string())
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&stop.stdout).unwrap();
    assert_eq!(v["decision"], "block", "{v}");
    assert!(v["reason"].as_str().unwrap().contains("F1"));

    r.ok(&[
        "ledger",
        "annotate",
        "F1",
        "--writer",
        "a.go:9",
        "--remover",
        "a.go:40",
    ]);
    r.ok(&[
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
    ]);
    // Explained, but not yet labelled: the label decision is the one thing left.
    let o = r.gate("final");
    assert_eq!(o.code, 2, "{} {}", o.stdout, o.stderr);
    assert_eq!(missing_ids(&o), ["F1"], "{o:?}");
    assert_eq!(o.json["missing"][0]["kind"], "label");
    r.ok(&[
        "ledger",
        "label",
        "F1",
        "--as",
        "UNCLASSIFIED",
        "--note",
        "static table filled by requests; nothing in the archetypes fits",
    ]);
    let o = r.gate("final");
    assert_eq!(o.code, 0, "{} {}", o.stdout, o.stderr);
    assert_eq!(r.ok(&["run", "status"])["complete"], true);
}

fn explain_f1(r: &Run) {
    r.ok(&["ledger", "annotate", "F1", "--writer", "a.go:9"]);
    r.ok(&[
        "ledger",
        "explain",
        "F1",
        "--tag",
        "inferred",
        "--removal",
        "never",
        "--text",
        "Requests add entries to the static/global table at a.go:1 through put at a.go:9 and nothing removes them, so it only grows.",
    ]);
}

const LABEL_TABLE: &str = r#"
[L-COLLECT]
holders = ["static/global", "container-field"]
regions = ["heap"]
requires = [["growth-series"]]
disconfirm = ["N-WARM"]
repair = ["bound/evict"]
summary = "A collection gains entries and nothing removes them."
"#;

fn with_data(r: &Run, data: &Path, off: bool, args: &[&str]) -> Out {
    let mut c = base(r.path());
    c.arg("--data").arg(data).args(args);
    if off {
        c.env("DLA_ARCHETYPES", "off");
    }
    let o = c.output().unwrap();
    let stdout = String::from_utf8(o.stdout).unwrap();
    Out {
        code: o.status.code().unwrap(),
        json: serde_json::from_str(&stdout).unwrap_or(Value::Null),
        stdout,
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

#[test]
fn the_final_gate_needs_a_label_decision_with_archetypes_on() {
    let r = init("symptom");
    r.answer_intake();
    confirmed_finding(&r);
    explain_f1(&r);
    let data = tempfile::tempdir().unwrap();
    fs::write(data.path().join("archetypes.toml"), LABEL_TABLE).unwrap();
    let o = with_data(&r, data.path(), false, &["gate", "check", "--final"]);
    assert_eq!(o.code, 2, "{o:?}");
    let m = &o.json["missing"][0];
    assert_eq!(
        (m["kind"].as_str(), m["id"].as_str()),
        (Some("label"), Some("F1"))
    );
    let fix = m["fix"].as_str().unwrap();
    assert!(fix.contains("dla ledger label F1 --suggest"), "{fix}");
    assert!(fix.contains("--as UNCLASSIFIED --note"), "{fix}");
    // An archetype ID is a decision.
    let l = with_data(
        &r,
        data.path(),
        false,
        &["ledger", "label", "F1", "--as", "L-COLLECT"],
    );
    assert_eq!(l.code, 0, "{l:?}");
    let o = with_data(&r, data.path(), false, &["gate", "check", "--final"]);
    assert_eq!(o.code, 0, "{o:?}");
}

#[test]
fn unclassified_with_a_note_satisfies_the_final_gate_with_archetypes_on() {
    let r = init("symptom");
    r.answer_intake();
    confirmed_finding(&r);
    explain_f1(&r);
    let data = tempfile::tempdir().unwrap();
    fs::write(data.path().join("archetypes.toml"), LABEL_TABLE).unwrap();
    let l = with_data(
        &r,
        data.path(),
        false,
        &[
            "ledger",
            "label",
            "F1",
            "--as",
            "UNCLASSIFIED",
            "--note",
            "no row fits",
        ],
    );
    assert_eq!(l.code, 0, "{l:?}");
    let o = with_data(&r, data.path(), false, &["gate", "check", "--final"]);
    assert_eq!(o.code, 0, "{o:?}");
}

#[test]
fn unclassified_with_a_note_satisfies_the_final_gate_with_archetypes_off() {
    let r = init("symptom");
    r.answer_intake();
    confirmed_finding(&r);
    explain_f1(&r);
    let data = tempfile::tempdir().unwrap();
    fs::write(data.path().join("archetypes.toml"), LABEL_TABLE).unwrap();
    let o = with_data(&r, data.path(), true, &["gate", "check", "--final"]);
    assert_eq!(o.code, 2, "{o:?}");
    assert_eq!(o.json["missing"][0]["kind"], "label");
    // An archetype ID is refused while they are off; UNCLASSIFIED with a note is not.
    let l = with_data(
        &r,
        data.path(),
        true,
        &["ledger", "label", "F1", "--as", "L-COLLECT"],
    );
    assert_eq!(l.code, 2, "{l:?}");
    assert_eq!(l.json["error"]["code"], "archetypes-off");
    let l = with_data(
        &r,
        data.path(),
        true,
        &[
            "ledger",
            "label",
            "F1",
            "--as",
            "UNCLASSIFIED",
            "--note",
            "archetypes are off",
        ],
    );
    assert_eq!(l.code, 0, "{l:?}");
    let o = with_data(&r, data.path(), true, &["gate", "check", "--final"]);
    assert_eq!(o.code, 0, "{o:?}");
}

// ----------------------------------------------------------------- integrity

fn append_line(path: &Path, text: &str) {
    let mut s = fs::read_to_string(path).unwrap();
    s.push_str(text);
    fs::write(path, s).unwrap();
}

#[test]
fn a_new_run_is_sealed_and_every_dla_write_keeps_the_seals_in_step() {
    let r = init("symptom");
    assert!(r.run_dir().join("integrity/ledger.jsonl.seal").is_file());
    assert!(r.run_dir().join("integrity/state.json.seal").is_file());
    r.answer_intake();
    r.ok(&["gate", "check", "--stage", "intake"]);
    r.ok(&["report"]);
    assert!(r.run_dir().join("integrity/report.md.seal").is_file());
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["tamper"], json!([]));
    assert!(st.get("unsealed_run").is_none());
}

#[test]
fn an_edit_outside_dla_is_tamper_in_every_consumer_until_resealed() {
    let r = init("symptom");
    r.answer_intake();
    // A valid but unsealed edit: one extra space inside the last record.
    {
        let p = r.run_dir().join("ledger.jsonl");
        let text = fs::read_to_string(&p).unwrap();
        let at = text.trim_end().rfind('{').unwrap() + 1;
        fs::write(&p, format!("{} {}", &text[..at], &text[at..])).unwrap();
    }
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["tamper"][0]["file"], "ledger.jsonl");
    assert_eq!(st["tamper"][0]["kind"], "modified");
    assert_eq!(st["complete"], false);
    assert_eq!(st["blocking"][0]["kind"], "integrity");
    // The gate (a stage and the final) names it.
    for stage in ["intake", "final"] {
        let o = r.gate(stage);
        assert_eq!(o.code, 2, "{stage}: {}", o.stdout);
        let m = o.json["missing"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["kind"] == "integrity")
            .unwrap_or_else(|| panic!("{stage}: {}", o.stdout));
        assert_eq!(m["id"], "ledger.jsonl");
        assert!(m["fix"].as_str().unwrap().contains("dla run reseal"));
    }
    // The report renders, says INCOMPLETE, names the file and exits non-zero.
    let o = base(r.path()).arg("report").output().unwrap();
    assert_eq!(o.status.code(), Some(2));
    let md = String::from_utf8(o.stdout).unwrap();
    assert!(md.contains("Status: INCOMPLETE"), "{md}");
    assert!(md.contains("ledger.jsonl does not match the seal"), "{md}");
    // Further dla writes do not seal the edit away.
    let again = r.run(&["run", "status"]);
    assert_eq!(again.json["tamper"][0]["file"], "ledger.jsonl");
    // state.json edited by hand is caught the same way.
    let sp = r.run_dir().join("state.json");
    let text = fs::read_to_string(&sp)
        .unwrap()
        .replace("\"intake\"", "\"report\"");
    fs::write(&sp, text).unwrap();
    let files: Vec<String> = r.ok(&["run", "status"])["tamper"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["file"].as_str().unwrap().to_owned())
        .collect();
    assert!(files.contains(&"state.json".to_owned()), "{files:?}");

    // Reseal needs a reason, records an audit line and seals again.
    let no_reason = r.run(&["run", "reseal", "--reason", "  "]);
    assert_eq!(no_reason.code, 2);
    assert_eq!(no_reason.json["error"]["code"], "empty-field");
}

#[test]
fn reseal_records_the_audit_line_and_the_report_lists_it() {
    let r = init("symptom");
    r.answer_intake();
    let sp = r.run_dir().join("state.json");
    let text = fs::read_to_string(&sp).unwrap();
    fs::write(&sp, text.replace("\"t\"", "\"u\"")).unwrap();
    assert_eq!(r.ok(&["run", "status"])["tamper"][0]["file"], "state.json");
    let v = r.ok(&["run", "reseal", "--reason", "fixed the session id by hand"]);
    assert_eq!(v["resealed"], json!(["state.json"]));
    assert_eq!(v["tamper"], 0);
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["tamper"], json!([]));
    let ledger = Ledger::load(&r.run_dir()).unwrap();
    assert_eq!(ledger.reseals().len(), 1);
    assert_eq!(ledger.reseals()[0].reason, "fixed the session id by hand");
    let o = base(r.path()).arg("report").output().unwrap();
    assert_eq!(o.status.code(), Some(0));
    let md = String::from_utf8(o.stdout).unwrap();
    assert!(md.contains("## Integrity"), "{md}");
    assert!(
        md.contains("Resealed `state.json`: fixed the session id by hand"),
        "{md}"
    );
    // Nothing left to reseal.
    let again = r.run(&["run", "reseal", "--reason", "again"]);
    assert_eq!(again.code, 2);
}

#[test]
fn a_run_without_seals_is_an_unsealed_run_and_not_tamper() {
    let r = init("symptom");
    r.answer_intake();
    fs::remove_dir_all(r.run_dir().join("integrity")).unwrap();
    // Edit the ledger freely: there is nothing to compare with.
    append_line(&r.run_dir().join("ledger.jsonl"), "");
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["unsealed_run"], true);
    assert_eq!(st["tamper"], json!([]));
    assert_eq!(r.gate("intake").code, 0);
    let o = base(r.path()).arg("report").output().unwrap();
    assert_eq!(o.status.code(), Some(0));
    // dla writes did not start sealing the run behind the operator's back.
    assert!(!r.run_dir().join("integrity").exists());
    // `dla run reseal` is how an old run starts to be sealed.
    let v = r.ok(&["run", "reseal", "--reason", "sealing an old run"]);
    assert_eq!(v["started_sealing"], true);
    assert!(r.run_dir().join("integrity/ledger.jsonl.seal").is_file());
    assert_eq!(r.ok(&["run", "status"])["tamper"], json!([]));
}

#[test]
fn editing_report_md_by_hand_is_noted_in_the_next_report() {
    let r = init("symptom");
    r.ok(&["report"]);
    let rp = r.run_dir().join("report.md");
    fs::write(&rp, "Status: COMPLETE\n").unwrap();
    // It is not a tamper of the run: the report is regenerated from the ledger.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["tamper"], json!([]));
    assert_eq!(st["report_modified"], true);
    let o = base(r.path()).arg("report").output().unwrap();
    assert_eq!(o.status.code(), Some(0));
    let md = String::from_utf8(o.stdout).unwrap();
    assert!(
        md.contains("report.md` was modified after it was rendered"),
        "{md}"
    );
    assert!(
        !fs::read_to_string(&rp)
            .unwrap()
            .starts_with("Status: COMPLETE\n")
    );
    assert!(r.ok(&["run", "status"]).get("report_modified").is_none());
}

// bin/dla is a bash script. On the Windows runner `bash` on PATH is the WSL launcher,
// which has no distribution; Claude Code runs hooks through Git Bash there, and the
// shim's Windows behaviour is covered by the "shim and downloader" job (tests/shim).
#[cfg(unix)]
#[test]
fn the_shim_runs_a_gate_hook_with_only_the_plugin_root_set() {
    // A hook gets CLAUDE_PLUGIN_ROOT and nothing on PATH: bin/dla must find the binary.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let dla = assert_cmd::cargo::cargo_bin("dla");
    let r = init("symptom");
    let o = std::process::Command::new("bash")
        .arg(root.join("bin/dla"))
        .args(["gate", "write-hook"])
        .env_clear()
        .env("HOME", r.path())
        .env("PATH", "/usr/bin:/bin")
        .env("CLAUDE_PLUGIN_ROOT", root)
        .env("DLA_BIN", &dla)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut c| {
            use std::io::Write;
            let report = r.run_dir().join("report.md");
            c.stdin.take().unwrap().write_all(
                json!({
                    "cwd": r.path(),
                    "hook_event_name": "PreToolUse",
                    "tool_name": "Write",
                    "tool_input": {"file_path": report, "content": "x"},
                })
                .to_string()
                .as_bytes(),
            )?;
            c.wait_with_output()
        })
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");
}
