//! `dla sweep` end to end with fake tools: candidates, evidence, coverage, raw files.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use dla_backends::{fsx::ToolEnv, sweep::sweep_with};
use dla_core::{
    Ledger, RunState,
    coverage::{BackendStatus, CoverageManifest},
    runtime::Runtime,
    state::{Consent, ConsentGiver, now_utc},
    toolctx::ToolCtx,
};

#[allow(dead_code)]
struct Fixture {
    _dir: tempfile::TempDir,
    project: PathBuf,
    data: PathBuf,
    tools: PathBuf,
    ctx: ToolCtx,
}

fn fixture(table: &str, files: &[(&str, &str)]) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("proj");
    fs::create_dir_all(&project).unwrap();
    for (p, c) in files {
        let f = project.join(p);
        fs::create_dir_all(f.parent().unwrap()).unwrap();
        fs::write(f, c).unwrap();
    }
    let data = dir.path().join("data");
    fs::create_dir_all(data.join("detectors")).unwrap();
    fs::write(data.join("detectors/python.toml"), table).unwrap();
    let tools = dir.path().join("tools");
    fs::create_dir_all(tools.join("bin")).unwrap();
    let run_dir = project.join(".dla/r1");
    fs::create_dir_all(&run_dir).unwrap();
    let mut st =
        RunState::init(project.to_str().unwrap(), Some("s"), 3600, None, now_utc()).unwrap();
    st.run_id = "r1".into();
    st.consent = Some(Consent {
        tiers: vec![1],
        raw_values: false,
        given_by: ConsentGiver::Runner,
        recorded_at: now_utc(),
    });
    st.save(&run_dir).unwrap();
    Ledger::create(&run_dir).unwrap();
    let ctx = ToolCtx {
        project: project.clone(),
        run_dir,
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: Some(data.clone()),
    };
    Fixture {
        _dir: dir,
        project,
        data,
        tools,
        ctx,
    }
}

fn env(f: &Fixture) -> ToolEnv {
    ToolEnv {
        tools_dir: Some(f.tools.clone()),
        path: Vec::new(),
        vars: Default::default(),
    }
}

#[cfg(unix)]
fn fake_tool(f: &Fixture, name: &str, script: &str) {
    use std::os::unix::fs::PermissionsExt;
    let p = f.tools.join("bin").join(name);
    fs::write(&p, format!("#!/bin/sh\n{script}\n")).unwrap();
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
}

const TABLE: &str = r#"
[[detector]]
id = "ruff/B019"
backend = "ruff"
rule = "B019"
suggested_archetype = "L-COLLECT"
verified = true
precision_prior = 0.5

[[detector]]
id = "ruff/SIM115"
backend = "ruff"
rule = "SIM115"
suggested_archetype = ""
verified = true

[[detector]]
id = "codeql/py/file-not-closed"
backend = "codeql"
rule = "py/file-not-closed"
suggested_archetype = "L-HANDLE"
verified = true
"#;

/// A ruff stand-in: `--version`, otherwise a SARIF log with three results (one for a
/// rule the table does not list, one duplicated).
#[cfg(unix)]
const FAKE_RUFF: &str = r#"
if [ "$1" = "--version" ]; then echo "ruff 9.9.9"; exit 0; fi
cat <<'EOF'
{"runs":[{"results":[
 {"ruleId":"B019","message":{"text":"cache on method"},"locations":[{"physicalLocation":{"artifactLocation":{"uri":"pkg/a.py"},"region":{"startLine":7}}}]},
 {"ruleId":"B019","message":{"text":"cache on method again"},"locations":[{"physicalLocation":{"artifactLocation":{"uri":"pkg/a.py"},"region":{"startLine":7,"startColumn":3}}}]},
 {"ruleId":"SIM115","message":{"text":"open without with"},"locations":[{"physicalLocation":{"artifactLocation":{"uri":"pkg/b.py"},"region":{"startLine":2}}}]},
 {"ruleId":"E501","message":{"text":"line too long"},"locations":[{"physicalLocation":{"artifactLocation":{"uri":"pkg/b.py"},"region":{"startLine":3}}}]}
]}]}
EOF
exit 1
"#;

const PY: [(&str, &str); 2] = [("pkg/a.py", "x = 1\n"), ("pkg/b.py", "y = 2\n")];

#[cfg(unix)]
fn tree(root: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if e.file_name() == ".dla" {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
            } else {
                out.insert(format!(
                    "{}:{}",
                    p.strip_prefix(root).unwrap().display(),
                    fs::read(&p).unwrap().len()
                ));
            }
        }
    }
    out
}

#[cfg(unix)]
#[test]
fn hits_become_candidates_with_evidence_raw_file_and_coverage() {
    let f = fixture(TABLE, &PY);
    fake_tool(&f, "ruff", FAKE_RUFF);
    let before = tree(&f.project);
    let out = sweep_with(&f.ctx, Runtime::Python, &env(&f)).unwrap();
    assert_eq!(out.candidates, 2, "{out:?}");
    assert_eq!(out.skipped_duplicates, 0);
    // One evidence record for ruff and one for the dla backend; codeql is missing.
    assert_eq!(out.evidence.len(), 2);

    let ledger = f.ctx.ledger().unwrap();
    let mut found: Vec<_> = ledger
        .findings()
        .map(|x| {
            (
                x.origin.to_string(),
                x.site.clone().unwrap(),
                x.suggested_archetype.clone(),
                x.supports.len(),
            )
        })
        .collect();
    found.sort();
    assert_eq!(
        found,
        vec![
            (
                "detector:ruff/B019".to_owned(),
                "pkg/a.py:7".to_owned(),
                Some("L-COLLECT".to_owned()),
                1
            ),
            (
                "detector:ruff/SIM115".to_owned(),
                "pkg/b.py:2".to_owned(),
                None,
                1
            ),
        ]
    );

    let raw: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(f.ctx.run_dir.join("sweep/ruff.json")).unwrap())
            .unwrap();
    assert_eq!(raw["status"], "ran");
    assert_eq!(raw["version"], "9.9.9");
    assert_eq!(raw["hits"].as_array().unwrap().len(), 2);
    assert!(raw["capture"]["runs"][0]["stdout_json"].is_object());

    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    let py = &cov.workers["python"].backends;
    assert_eq!(py["ruff"].status, BackendStatus::Ran);
    assert_eq!(py["ruff"].enumerated, 2);
    assert_eq!(py["ruff"].version.as_deref(), Some("9.9.9"));
    assert_eq!(py["codeql"].status, BackendStatus::Missing);
    assert_eq!(
        py["codeql"].install_hint.as_deref(),
        Some("dla install codeql")
    );
    assert_eq!(py["dla"].status, BackendStatus::Ran);

    // The tool never wrote into the project outside .dla.
    assert_eq!(tree(&f.project), before);
}

#[cfg(unix)]
#[test]
fn a_second_sweep_skips_existing_candidates_and_keeps_the_counts() {
    let f = fixture(TABLE, &PY);
    fake_tool(&f, "ruff", FAKE_RUFF);
    let e = env(&f);
    sweep_with(&f.ctx, Runtime::Python, &e).unwrap();
    let again = sweep_with(&f.ctx, Runtime::Python, &e).unwrap();
    assert_eq!(again.candidates, 0);
    assert_eq!(again.skipped_duplicates, 2);
    assert_eq!(f.ctx.ledger().unwrap().findings().count(), 2);
    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    assert_eq!(cov.workers["python"].backends["ruff"].enumerated, 2);
}

#[cfg(unix)]
#[test]
fn a_failing_tool_is_blocked_with_its_output_kept() {
    let f = fixture(TABLE, &PY);
    fake_tool(
        &f,
        "ruff",
        "if [ \"$1\" = \"--version\" ]; then echo 'ruff 1.0.0'; exit 0; fi\necho 'boom: config error' >&2\nexit 2",
    );
    let out = sweep_with(&f.ctx, Runtime::Python, &env(&f)).unwrap();
    assert_eq!(out.candidates, 0);
    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    let ruff = &cov.workers["python"].backends["ruff"];
    assert_eq!(ruff.status, BackendStatus::Blocked);
    assert!(
        ruff.reason
            .as_deref()
            .unwrap()
            .contains("boom: config error")
    );
    let raw = fs::read_to_string(f.ctx.run_dir.join("sweep/ruff.json")).unwrap();
    assert!(raw.contains("boom: config error"));
    assert_eq!(out.evidence.len(), 1, "only the dla backend ran");
}

#[cfg(unix)]
#[test]
fn a_hanging_tool_times_out_and_is_blocked() {
    let f = fixture(TABLE, &PY);
    fake_tool(
        &f,
        "ruff",
        "if [ \"$1\" = \"--version\" ]; then echo 'ruff 1.0.0'; exit 0; fi\nsleep 30",
    );
    let mut e = env(&f);
    e.vars
        .insert(dla_backends::sweep::ENV_TIMEOUT.to_owned(), "1".to_owned());
    let started = std::time::Instant::now();
    sweep_with(&f.ctx, Runtime::Python, &e).unwrap();
    assert!(started.elapsed().as_secs() < 20);
    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    let ruff = &cov.workers["python"].backends["ruff"];
    assert_eq!(ruff.status, BackendStatus::Blocked);
    assert!(ruff.reason.as_deref().unwrap().contains("timed out"));
}

#[test]
fn missing_tools_are_recorded_with_install_hints() {
    let f = fixture(TABLE, &PY);
    let out = sweep_with(&f.ctx, Runtime::Python, &env(&f)).unwrap();
    assert_eq!(out.candidates, 0);
    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    for b in ["ruff", "codeql"] {
        let c = &cov.workers["python"].backends[b];
        assert_eq!(c.status, BackendStatus::Missing, "{b}");
        assert_eq!(
            c.install_hint.as_deref(),
            Some(format!("dla install {b}").as_str())
        );
    }
    assert_eq!(out.summary["ruff"]["status"], "missing");
}

#[test]
fn an_invalid_table_is_refused_with_file_and_line() {
    let bad = "[[detector]]\nid = \"ruff/B019\"\nbackend = \"nonesuch\"\nrule = \"B019\"\n";
    let f = fixture(bad, &PY);
    let e = sweep_with(&f.ctx, Runtime::Python, &env(&f)).unwrap_err();
    assert_eq!(e.code, "invalid-data-file");
    assert!(
        e.message.contains("data/detectors/python.toml:3"),
        "{}",
        e.message
    );
    assert!(e.message.contains("nonesuch"));
}

#[test]
fn a_language_without_a_table_still_runs_the_dla_backend() {
    let f = fixture(TABLE, &PY);
    let out = sweep_with(&f.ctx, Runtime::Native, &env(&f)).unwrap();
    assert_eq!(out.summary["dla"]["status"], "ran");
    assert_eq!(out.evidence.len(), 1);
}

#[cfg(unix)]
#[test]
fn blanking_archetypes_changes_no_count_or_site() {
    let with = fixture(TABLE, &PY);
    fake_tool(&with, "ruff", FAKE_RUFF);
    let blank = fixture(&TABLE.replace("L-COLLECT", "").replace("L-HANDLE", ""), &PY);
    fake_tool(&blank, "ruff", FAKE_RUFF);
    let a = sweep_with(&with.ctx, Runtime::Python, &env(&with)).unwrap();
    let b = sweep_with(&blank.ctx, Runtime::Python, &env(&blank)).unwrap();
    assert_eq!(a.candidates, b.candidates);
    let sites = |f: &Fixture| -> BTreeSet<(String, String)> {
        f.ctx
            .ledger()
            .unwrap()
            .findings()
            .map(|x| (x.origin.to_string(), x.site.clone().unwrap()))
            .collect()
    };
    assert_eq!(sites(&with), sites(&blank));
    let tagged = |f: &Fixture| {
        f.ctx
            .ledger()
            .unwrap()
            .findings()
            .filter(|x| x.suggested_archetype.is_some())
            .count()
    };
    assert_eq!(tagged(&with), 1);
    assert_eq!(tagged(&blank), 0);
}

#[cfg(unix)]
#[test]
fn rapx_needs_build_consent_because_cargo_runs_build_scripts() {
    let table = "[[detector]]\nid = \"rapx/leak\"\nbackend = \"rapx\"\nrule = \"leak\"\nsuggested_archetype = \"L-OWNERSHIP\"\nverified = true\n";
    let f = fixture(
        table,
        &[("Cargo.toml", "[package]\nname=\"x\"\nversion=\"0.1.0\"\n")],
    );
    fs::write(f.data.join("detectors/rust.toml"), table).unwrap();
    fake_tool(&f, "cargo-rapx", "exit 0");
    fake_tool(&f, "cargo", "exit 0");
    let mut st = RunState::load(&f.ctx.run_dir).unwrap();
    st.consent = None;
    st.save(&f.ctx.run_dir).unwrap();
    sweep_with(&f.ctx, Runtime::Rust, &env(&f)).unwrap();
    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    let rapx = &cov.workers["rust"].backends["rapx"];
    assert_eq!(rapx.status, BackendStatus::Blocked);
    assert!(rapx.reason.as_deref().unwrap().contains("consent tier 1"));
    assert!(
        rapx.reason
            .as_deref()
            .unwrap()
            .contains("dla consent --tiers 1-4")
    );
}

/// The shipped tables parse and carry the rule ids the spec lists. Which rows
/// are `verified` is checked against docs/VERIFY.md in `detector_flags.rs`.
#[test]
fn shipped_detector_tables_are_valid_and_complete() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/detectors");
    let mut ids: BTreeSet<String> = BTreeSet::new();
    for lang in ["go", "jvm", "rust", "node", "python", "native"] {
        let text = fs::read_to_string(root.join(format!("{lang}.toml"))).unwrap();
        let t =
            dla_core::detectors::DetectorFile::parse(&text, &format!("data/detectors/{lang}.toml"))
                .unwrap_or_else(|e| panic!("{lang}: {}", e.message));
        for d in &t.detector {
            ids.insert(d.id.clone());
            if d.backend == "dla" {
                assert!(
                    d.verified,
                    "{} is a DLA-owned rule and must be verified",
                    d.id
                );
            }
        }
    }
    for want in [
        "codeql/java/input-resource-leak",
        "codeql/java/output-resource-leak",
        "codeql/java/database-resource-leak",
        "errorprone/StreamResourceLeak",
        "errorprone/MustBeClosedChecker",
        "spotbugs/OBL_UNSATISFIED_OBLIGATION_EXCEPTION_EDGE",
        "sonar/S5164",
        "sonar/S2095",
        "detekt/GlobalCoroutineUsage",
        "detekt/SuspendFunSwallowedCancellation",
        "detekt/SuspendFunInFinallySection",
        "staticcheck/SA1015",
        "staticcheck/SA2001",
        "staticcheck/SA2003",
        "staticcheck/SA5003",
        "staticcheck/SA5005",
        "staticcheck/SA6002",
        "staticcheck/SA9001",
        "golangci-lint/bodyclose",
        "golangci-lint/sqlclosecheck",
        "golangci-lint/rowserrcheck",
        "golangci-lint/noctx",
        "golangci-lint/contextcheck",
        "golangci-lint/containedctx",
        "golangci-lint/fatcontext",
        "golangci-lint/spancheck",
        "govet/lostcancel",
        "rapx/leak",
        "eslint/rxjs-x/no-ignored-subscription",
        "eslint/rxjs-x/no-floating-observables",
        "eslint/rxjs-x/no-nested-subscribe",
        "eslint/rxjs-x/no-unsafe-takeuntil",
        "eslint/@typescript-eslint/no-floating-promises",
        "ruff/B019",
        "ruff/RUF006",
        "pylint/W1518",
        "codeql/py/file-not-closed",
    ] {
        assert!(ids.contains(want), "missing detector row {want}");
    }
}
