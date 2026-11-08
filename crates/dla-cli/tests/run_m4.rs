//! M4 run commands: entry paths, worker registry, abort and the verifier bundle.

use std::{fs, path::Path};

use assert_cmd::Command;
use dla_core::{
    Ledger, NewFinding, Origin, RunState,
    evidence::{Authority, EvidenceCoverage, EvidenceKind, NewEvidence, Source, Tool},
    state::{ProcessKind, ProcessRecord, ProcessRole, RunStatus, now_utc, run_dir as run_dir_of},
};
use serde_json::{Value, json};

fn dla(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
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
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.len() <= dla_core::MAX_OUTPUT_BYTES,
        "stdout is {} bytes: {stdout}",
        stdout.len()
    );
    (
        out.status.code().unwrap(),
        serde_json::from_str(&stdout).unwrap_or(Value::Null),
    )
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let (code, v) = run(project, args);
    assert_eq!((code, &v["ok"]), (0, &Value::Bool(true)), "{args:?}: {v}");
    v
}

fn refused(project: &Path, args: &[&str]) -> Value {
    let (code, v) = run(project, args);
    assert_eq!(code, 2, "{args:?}: {v}");
    let e = &v["error"];
    for k in ["code", "message", "missing", "fix"] {
        assert!(
            e[k].as_str().is_some_and(|s| !s.is_empty()),
            "refusal lacks {k}: {v}"
        );
    }
    e.clone()
}

fn init(project: &Path, extra: &[&str]) -> (String, std::path::PathBuf) {
    let mut args = vec!["run", "init"];
    args.extend_from_slice(extra);
    let v = ok(project, &args);
    let id = v["run_id"].as_str().unwrap().to_owned();
    let dir = run_dir_of(project, &id);
    (id, dir)
}

fn project() -> tempfile::TempDir {
    let d = tempfile::tempdir().unwrap();
    fs::create_dir_all(d.path().join("dumps")).unwrap();
    fs::write(d.path().join("dumps/h1.hprof"), b"x").unwrap();
    d
}

#[test]
fn entry_paths_and_artifacts_are_validated_and_recorded() {
    let p = project();
    let (_, dir) = init(p.path(), &[]);
    assert_eq!(RunState::load(&dir).unwrap().entry.as_str(), "repository");

    let e = refused(p.path(), &["run", "init", "--entry", "artifact"]);
    assert!(e["fix"].as_str().unwrap().contains("--artifact"));
    let e = refused(p.path(), &["run", "init", "--artifact", "dumps/h1.hprof"]);
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("only allowed with --entry artifact")
    );
    let e = refused(
        p.path(),
        &[
            "run",
            "init",
            "--entry",
            "artifact",
            "--artifact",
            "dumps/none.hprof",
        ],
    );
    assert!(e["message"].as_str().unwrap().contains("does not exist"));
    refused(
        p.path(),
        &["run", "init", "--entry", "artifact", "--artifact", "../x"],
    );
    let (code, _) = run(p.path(), &["run", "init", "--entry", "bogus"]);
    assert_ne!(code, 0);

    let (id, dir) = init(
        p.path(),
        &[
            "--entry",
            "artifact",
            "--artifact",
            "./dumps/h1.hprof",
            "--artifact",
            "dumps",
        ],
    );
    let st = RunState::load(&dir).unwrap();
    assert_eq!(st.entry.as_str(), "artifact");
    assert_eq!(st.artifacts, ["dumps/h1.hprof", "dumps"]);
    let ledger = fs::read_to_string(dir.join("ledger.jsonl")).unwrap();
    assert!(ledger.contains("\"entry\":\"artifact\""), "{ledger}");
    assert!(ledger.contains("dumps/h1.hprof"), "{ledger}");
    let s = ok(p.path(), &["--run", &id, "run", "status"]);
    assert_eq!(s["entry"], "artifact");
}

#[test]
fn dispatch_registers_workers_and_status_lists_them() {
    let p = project();
    let (id, dir) = init(p.path(), &["--budget", "100m"]);
    let v = ok(p.path(), &["run", "dispatch", "--workers", "jvm,rust"]);
    let ws = v["workers"].as_array().unwrap();
    assert_eq!(ws.len(), 2);
    assert_eq!(ws[0]["name"], "jvm");
    assert_eq!(ws[0]["prefix"], format!("dla --run {id} --worker jvm"));
    assert!(
        ws[0]["reference"]
            .as_str()
            .unwrap()
            .ends_with("skills/detect-leak-anywhere/references/jvm.md")
    );
    assert!(ws[0]["reference_exists"].is_boolean());
    // 6000 s * 0.85 / 2
    assert_eq!(ws[0]["share_secs"], 2550);
    assert_eq!(ws[1]["share_secs"], 2550);
    let st = RunState::load(&dir).unwrap();
    assert_eq!(st.workers.len(), 2);
    assert_eq!(st.workers[0].slice, "ledger.jvm.jsonl");
    assert_eq!(st.workers[0].status.as_str(), "registered");
    assert!(st.workers[0].deadline < st.budget.deadline);

    let e = refused(p.path(), &["run", "dispatch", "--workers", "jvm"]);
    assert_eq!(e["code"], "duplicate-id");
    let e = refused(p.path(), &["run", "dispatch", "--workers", "cobol"]);
    assert_eq!(e["code"], "invalid-value");
    refused(p.path(), &["run", "dispatch", "--workers", "go,go"]);
    refused(
        p.path(),
        &["run", "dispatch", "--workers", "go", "--reserve", "1.5"],
    );
    let e = refused(
        p.path(),
        &["--worker", "jvm", "run", "dispatch", "--workers", "go"],
    );
    assert!(e["message"].as_str().unwrap().contains("core command"));
    let e = refused(p.path(), &["--worker", "jvm", "run", "abort"]);
    assert!(e["message"].as_str().unwrap().contains("core command"));

    fs::write(dir.join("ledger.jvm.jsonl"), "{}\n{}\n").unwrap();
    let s = ok(p.path(), &["run", "status"]);
    assert_eq!(s["workers"][0]["name"], "jvm");
    assert_eq!(s["workers"][0]["slice_lines"], 2);
    assert_eq!(s["workers"][1]["slice_lines"], 0);
    assert_eq!(s["running_total"], 0);
}

#[test]
fn dispatch_refuses_when_nothing_remains() {
    let p = project();
    let (_, dir) = init(p.path(), &[]);
    RunState::update(&dir, |s| {
        s.budget.deadline = now_utc() - chrono::TimeDelta::try_seconds(5).unwrap();
        Ok(())
    })
    .unwrap();
    let e = refused(p.path(), &["run", "dispatch", "--workers", "go"]);
    assert_eq!(e["code"], "time-budget");
}

#[cfg(unix)]
mod abort {
    use std::{os::unix::process::CommandExt, process::Child, process::Command as Std};

    use super::*;

    fn sleeper() -> Child {
        let mut c = Std::new("sleep");
        c.arg("60").process_group(0);
        c.spawn().unwrap()
    }

    fn record(
        pid: u32,
        start: Option<String>,
        role: ProcessRole,
        cleanup: Vec<String>,
    ) -> ProcessRecord {
        ProcessRecord {
            pid,
            start_time: start,
            kind: ProcessKind::Harness,
            command: "sleep 60".into(),
            started_at: now_utc(),
            stopped_at: None,
            role,
            worker: None,
            cleanup,
        }
    }

    #[test]
    fn abort_kills_verified_groups_cleans_attachments_and_is_idempotent() {
        let p = project();
        let (id, dir) = init(p.path(), &[]);
        let mut target = sleeper();
        let mut reused = sleeper();
        let target_start = dla_sample::os::process_start_time(target.id());
        assert!(target_start.is_some());
        let junk = p.path().join("attach-junk.txt");
        fs::write(&junk, "x").unwrap();
        let cleanup = vec![
            format!("rm {}", junk.display()),
            "note the inspector stays open".to_owned(),
            r#"run ["true"]"#.to_owned(),
        ];
        RunState::update(&dir, |s| {
            s.processes.push(record(
                target.id(),
                target_start,
                ProcessRole::Started,
                vec![],
            ));
            // Same pid space, wrong identity: must not be signalled.
            s.processes.push(record(
                reused.id(),
                Some("2001-01-01T00:00:00Z".into()),
                ProcessRole::Started,
                vec![],
            ));
            // Attached: cleanup runs, the target (the live `reused` pid) is never signalled.
            s.processes
                .push(record(reused.id(), None, ProcessRole::Attached, cleanup));
            Ok(())
        })
        .unwrap();

        let v = ok(
            p.path(),
            &["--run", &id, "run", "abort", "--reason", "user stopped"],
        );
        assert_eq!(
            (
                v["killed"].as_u64(),
                v["cleaned"].as_u64(),
                v["gone"].as_u64()
            ),
            (Some(1), Some(1), Some(1)),
            "{v}"
        );
        assert_eq!(
            (v["skipped"].as_u64(), v["failed"].as_u64()),
            (Some(0), Some(0))
        );
        assert_eq!(v["status"], "aborted");
        // The verified group is dead; the mismatched pid and the attach target live on.
        assert!(
            target.wait().unwrap().code().is_none(),
            "killed by a signal"
        );
        assert!(
            reused.try_wait().unwrap().is_none(),
            "mismatch was signalled"
        );
        assert!(!junk.exists(), "cleanup rm did not run");

        let st = RunState::load(&dir).unwrap();
        assert_eq!(st.status, RunStatus::Aborted);
        assert_eq!(st.incomplete_reason.as_deref(), Some("user stopped"));
        assert!(st.processes.iter().all(|p| p.stopped_at.is_some()));
        assert!(
            st.processes[1]
                .cleanup
                .iter()
                .any(|c| c.starts_with("note abort:"))
        );

        let v = ok(p.path(), &["--run", &id, "run", "abort"]);
        assert_eq!(
            (
                v["skipped"].as_u64(),
                v["killed"].as_u64(),
                v["cleaned"].as_u64(),
                v["gone"].as_u64()
            ),
            (Some(3), Some(0), Some(0), Some(0)),
            "{v}"
        );
        assert_eq!(
            v["reason"], "user stopped",
            "a repeated abort keeps the first reason"
        );
        reused.kill().unwrap();
        reused.wait().unwrap();
    }

    #[test]
    fn a_failed_cleanup_keeps_the_record_for_a_retry() {
        let p = project();
        let (id, dir) = init(p.path(), &[]);
        RunState::update(&dir, |s| {
            s.processes.push(record(
                1,
                None,
                ProcessRole::Attached,
                vec![r#"run ["false"]"#.to_owned()],
            ));
            Ok(())
        })
        .unwrap();
        let (code, v) = run(p.path(), &["--run", &id, "run", "abort"]);
        assert_eq!((code, &v["ok"]), (0, &Value::Bool(false)), "{v}");
        assert_eq!(v["failed"], 1, "{v}");
        assert!(
            RunState::load(&dir).unwrap().processes[0]
                .stopped_at
                .is_none()
        );
    }
}

fn evidence(
    kind: EvidenceKind,
    authority: Authority,
    result: Value,
    raw: Option<&str>,
) -> NewEvidence {
    NewEvidence {
        source: Source::Repo {
            commit: "abc".into(),
            path: None,
        },
        tool: Tool {
            name: "dla".into(),
            version: "0".into(),
        },
        authority,
        kind,
        result,
        raw: raw.map(str::to_owned),
        coverage: EvidenceCoverage::default(),
        limitations: Vec::new(),
        unknowns: Vec::new(),
    }
}

fn forbidden_in(v: &Value, path: &str, hits: &mut Vec<String>) {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                if dla_cli_forbidden().contains(&k.as_str()) {
                    hits.push(format!("{path}/{k}"));
                }
                forbidden_in(x, &format!("{path}/{k}"), hits);
            }
        }
        Value::Array(a) => a.iter().for_each(|x| forbidden_in(x, path, hits)),
        _ => {}
    }
}

fn dla_cli_forbidden() -> [&'static str; 13] {
    [
        "hypothesis",
        "refute_if",
        "archetype",
        "suggested_archetype",
        "explanation",
        "origin",
        "annotation",
        "annotations",
        "note",
        "notes",
        "label",
        "labels",
        "remedy",
    ]
}

#[test]
fn bundle_carries_site_excerpt_and_evidence_without_labels() {
    let p = project();
    let (id, dir) = init(p.path(), &[]);
    fs::create_dir_all(p.path().join("src")).unwrap();
    let src: String = (1..=60).map(|n| format!("int line{n};\n")).collect();
    fs::write(p.path().join("src/Cache.java"), src).unwrap();
    fs::create_dir_all(dir.join("series")).unwrap();
    fs::write(
        dir.join("series/S1.json"),
        r#"{"axis":"iteration","archetype":"L-SECRET","rows":[{"label":"x","v":1}],"notes":["n"]}"#,
    )
    .unwrap();
    fs::write(dir.join("series/S2.json"), "").unwrap();
    let mut ledger = Ledger::load(&dir).unwrap();
    let growth = ledger
        .add_evidence(evidence(
            EvidenceKind::GrowthSeries,
            Authority::Test,
            json!({"gate": "pass", "hypothesis": "leak in Cache", "inner": {"origin": "census"}}),
            Some("series/S1.json"),
        ))
        .unwrap();
    let growth_id = match growth.record {
        dla_core::Record::Evidence(e) => e.evidence_id,
        other => panic!("{other:?}"),
    };
    let statics = ledger
        .add_evidence(evidence(
            EvidenceKind::StaticHit,
            Authority::Static,
            json!({"hits": 1}),
            None,
        ))
        .unwrap();
    let static_id = match statics.record {
        dla_core::Record::Evidence(e) => e.evidence_id,
        other => panic!("{other:?}"),
    };
    let gone = ledger
        .add_evidence(evidence(
            EvidenceKind::AllocStack,
            Authority::Live,
            json!({"frames": 3}),
            Some("series/S2.json"),
        ))
        .unwrap();
    let gone_id = match gone.record {
        dla_core::Record::Evidence(e) => e.evidence_id,
        other => panic!("{other:?}"),
    };
    let mut nf = NewFinding::new(
        Origin::parse("detector:ruff/B019").unwrap(),
        vec![growth_id.clone(), static_id.clone(), gone_id.clone()],
    );
    nf.site = Some("src/Cache.java:30".into());
    nf.suggested_archetype = Some("L-SECRET".into());
    ledger.add_finding(nf).unwrap();
    drop(ledger);

    let v = ok(p.path(), &["--run", &id, "bundle", "F1"]);
    let bundle = dir.join("bundle/F1");
    assert!(bundle.is_dir(), "{v}");
    assert_eq!(v["excerpt"], true);
    // Growth and allocation evidence only (the static hit is not in the four classes).
    assert_eq!(
        v["evidence"],
        json!([growth_id.to_string(), gone_id.to_string()])
    );
    let excerpt = fs::read_to_string(bundle.join("excerpt.txt")).unwrap();
    assert!(
        excerpt.starts_with("src/Cache.java:30 (lines 10-50)"),
        "{excerpt}"
    );
    assert!(excerpt.contains(">    30  int line30;"));

    let index: Value =
        serde_json::from_slice(&fs::read(bundle.join("index.json")).unwrap()).unwrap();
    assert_eq!(index["site"], "src/Cache.java:30");
    let recs = index["evidence"].as_array().unwrap();
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0]["kind"], "growth-series");
    assert_eq!(recs[0]["authority"], "test");
    assert_eq!(recs[0]["result"], json!({"gate": "pass", "inner": {}}));
    assert!(
        recs[1].get("file").is_none(),
        "an empty placeholder is not copied"
    );
    assert!(index["omitted"].is_array());

    // No forbidden key in any JSON file of the bundle; every file is hashed in the manifest.
    let mut hits = Vec::new();
    let mut names = Vec::new();
    let mut stack = vec![bundle.clone()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(d).unwrap() {
            let path = e.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else {
                names.push(
                    path.strip_prefix(&bundle)
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                );
                if path.extension().is_some_and(|x| x == "json") {
                    let doc: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                    forbidden_in(&doc, &names.last().unwrap().clone(), &mut hits);
                }
            }
        }
    }
    assert!(hits.is_empty(), "forbidden keys: {hits:?}");
    let copied: Value =
        serde_json::from_slice(&fs::read(bundle.join(recs[0]["file"].as_str().unwrap())).unwrap())
            .unwrap();
    assert_eq!(copied, json!({"axis": "iteration", "rows": [{"v": 1}]}));

    let manifest_bytes = fs::read(bundle.join("manifest.json")).unwrap();
    let manifest: Value = serde_json::from_slice(&manifest_bytes).unwrap();
    let listed: Vec<&str> = manifest["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["path"].as_str().unwrap())
        .collect();
    let mut expect: Vec<String> = names.into_iter().filter(|n| n != "manifest.json").collect();
    expect.sort();
    assert_eq!(listed, expect);
    use sha2::{Digest, Sha256};
    assert_eq!(
        v["manifest_sha256"],
        hex::encode(Sha256::digest(&manifest_bytes))
    );

    // A second bundle of the same finding gets its own directory.
    let v2 = ok(p.path(), &["--run", &id, "bundle", "F1"]);
    assert!(v2["dir"].as_str().unwrap().ends_with("bundle/F1.2"), "{v2}");
    assert_eq!(v2["manifest_sha256"], v["manifest_sha256"]);

    let e = refused(p.path(), &["--run", &id, "bundle", "F9"]);
    assert_eq!(e["code"], "unknown-finding");
}
