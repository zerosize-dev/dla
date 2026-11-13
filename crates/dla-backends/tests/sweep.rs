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
        worker: None,
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

    let raw: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(f.ctx.run_dir.join("sweep/python-ruff.json")).unwrap(),
    )
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
fn module_scope_drops_hits_outside_it_and_counts_the_files() {
    let f = fixture(
        TABLE,
        &[
            ("pkg/a.py", "x = 1\n"),
            ("pkg/b.py", "y = 2\n"),
            ("other/c.py", "z = 3\n"),
        ],
    );
    fake_tool(&f, "ruff", FAKE_RUFF);
    RunState::update(&f.ctx.run_dir, |s| {
        s.scope = Some(dla_core::state::Scope {
            kind: dla_core::state::ScopeKind::Module,
            path: "pkg".into(),
            excluded: vec!["pkg/b.py".into()],
        });
        Ok(())
    })
    .unwrap();
    let out = sweep_with(&f.ctx, Runtime::Python, &env(&f)).unwrap();
    // The fake tool reports pkg/a.py (in scope) and pkg/b.py (excluded).
    assert_eq!(out.candidates, 1, "{out:?}");
    assert_eq!(out.summary["out_of_scope"], 2);
    assert_eq!(out.summary["out_of_scope_hits"], 1);
    let ledger = f.ctx.ledger().unwrap();
    let sites: Vec<_> = ledger.findings().filter_map(|x| x.site.clone()).collect();
    assert_eq!(sites, ["pkg/a.py:7"]);
    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    assert_eq!(cov.workers["python"].out_of_scope, 2);
}

/// A ruff stand-in that reports one hit in a project file and two in build logic.
#[cfg(unix)]
const FAKE_RUFF_BUILD_LOGIC: &str = r#"
if [ "$1" = "--version" ]; then echo "ruff 9.9.9"; exit 0; fi
cat <<'EOF'
{"runs":[{"results":[
 {"ruleId":"B019","message":{"text":"cache on method"},"locations":[{"physicalLocation":{"artifactLocation":{"uri":"pkg/a.py"},"region":{"startLine":7}}}]},
 {"ruleId":"B019","message":{"text":"cache in a build script"},"locations":[{"physicalLocation":{"artifactLocation":{"uri":"docker.gradle.kts"},"region":{"startLine":4}}}]},
 {"ruleId":"B019","message":{"text":"cache in buildSrc"},"locations":[{"physicalLocation":{"artifactLocation":{"uri":"buildSrc/b.py"},"region":{"startLine":2}}}]}
]}]}
EOF
exit 1
"#;

#[cfg(unix)]
#[test]
fn hits_in_build_logic_are_dropped_named_and_counted() {
    let f = fixture(
        TABLE,
        &[
            ("pkg/a.py", "x = 1\n"),
            ("buildSrc/b.py", "y = 2\n"),
            ("docker.gradle.kts", "val d = 1\n"),
        ],
    );
    fake_tool(&f, "ruff", FAKE_RUFF_BUILD_LOGIC);
    let out = sweep_with(&f.ctx, Runtime::Python, &env(&f)).unwrap();
    assert_eq!(out.candidates, 1, "{out:?}");
    let ledger = f.ctx.ledger().unwrap();
    let sites: Vec<_> = ledger.findings().filter_map(|x| x.site.clone()).collect();
    assert_eq!(sites, ["pkg/a.py:7"]);
    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    // `buildSrc/b.py` is a Python source that was not read; `docker.gradle.kts` is Kotlin.
    assert_eq!(cov.workers["python"].build_logic, 1);
    let raw: serde_json::Value = serde_json::from_slice(
        &fs::read(f.ctx.run_dir.join("sweep").join("python-ruff.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(raw["hits"].as_array().unwrap().len(), 1);
    assert_eq!(raw["build_logic_hits"], 2);
    let named = ledger.evidence().any(|e| {
        e.limitations
            .iter()
            .any(|l| l.contains("2 hit(s) in build logic"))
    });
    assert!(named, "the evidence names the dropped build-logic hits");
}

#[cfg(unix)]
const JVM_CODEQL_TABLE: &str = r#"
[[detector]]
id = "codeql/java/input-resource-leak"
backend = "codeql"
rule = "java/input-resource-leak"
suggested_archetype = "L-HANDLE"
verified = false
"#;

/// A `codeql` stand-in: records every call (arguments and working directory) in `log`,
/// writes one SARIF result for `analyze`. Not real CodeQL output.
#[cfg(unix)]
fn fake_codeql(f: &Fixture, log: &Path) {
    fs::write(f.data.join("detectors/jvm.toml"), JVM_CODEQL_TABLE).unwrap();
    let ql = f
        .tools
        .join("bin/qlpacks/codeql/java-queries/1.0.0/Likely Bugs/Resource Leaks/CloseReader.ql");
    fs::create_dir_all(ql.parent().unwrap()).unwrap();
    fs::write(&ql, "").unwrap();
    fake_tool(
        f,
        "codeql",
        &format!(
            r#"
if [ "$1" = "version" ]; then echo 2.99.0; exit 0; fi
echo "$1 $2 cwd=$(pwd) :: $*" >> '{log}'
case "$*" in *--build-mode=manual*) if [ -f '{log}.failbuild' ]; then echo "Error: the traced build failed" >&2; echo "Could not resolve org.jetbrains.kotlin:kotlin-stdlib:1.9.24" >&2; exit 2; fi;; esac
if [ "$2" = "analyze" ] || [ "$1 $2" = "database analyze" ]; then
  for a in "$@"; do case "$a" in --output=*) out="${{a#--output=}}";; esac; done
  cat > "$out" <<'SARIF'
{{"runs":[{{"results":[{{"ruleId":"java/input-resource-leak","message":{{"text":"not closed"}},"locations":[{{"physicalLocation":{{"artifactLocation":{{"uri":"src/main/java/app/B.java"}},"region":{{"startLine":5}}}}}}]}}]}}]}}
SARIF
fi
exit 0
"#,
            log = log.display()
        ),
    );
}

#[cfg(unix)]
fn gradle_project(f: &Fixture, java: bool) {
    let put = |rel: &str, body: &str| {
        let p = f.project.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    };
    put("build.gradle.kts", "plugins { kotlin(\"jvm\") }\n");
    put("docker.gradle.kts", "val d = 1\n");
    put("src/main/kotlin/app/A.kt", "package app\nclass A\n");
    if java {
        put("src/main/java/app/B.java", "package app;\nclass B {}\n");
    }
    use std::os::unix::fs::PermissionsExt;
    put("gradlew", "#!/bin/sh\nexit 0\n");
    fs::set_permissions(f.project.join("gradlew"), fs::Permissions::from_mode(0o755)).unwrap();
}

#[cfg(unix)]
#[test]
fn a_kotlin_project_gets_a_traced_build_in_a_copy() {
    let f = fixture(TABLE, &[]);
    let log = f.project.parent().unwrap().join("codeql.log");
    fake_codeql(&f, &log);
    gradle_project(&f, false);
    let out = sweep_with(&f.ctx, Runtime::Jvm, &env(&f)).unwrap();
    let calls = fs::read_to_string(&log).unwrap();
    let create = calls
        .lines()
        .find(|l| l.starts_with("database create"))
        .unwrap();
    assert!(create.contains("--build-mode=manual"), "{create}");
    assert!(!create.contains("--build-mode=none"), "{create}");
    assert!(
        create.contains("--command ./gradlew --no-daemon --offline"),
        "{create}"
    );
    // The build runs in a copy under the run's scratch directory, never in the project.
    let root = create
        .split("--source-root ")
        .nth(1)
        .unwrap()
        .split(' ')
        .next()
        .unwrap();
    assert!(root.ends_with(".dla/r1/tmp/jvm-codeql/src"), "{root}");
    // CodeQL ignores the process directory for --command: the build directory is an argument.
    let wd = create
        .split("--working-dir ")
        .nth(1)
        .unwrap()
        .split(' ')
        .next()
        .unwrap();
    assert!(wd.ends_with(".dla/r1/tmp/jvm-codeql/src"), "{wd}");
    // The copy is not kept once the database is analysed.
    assert!(!f.ctx.run_dir.join("tmp/jvm-codeql/src").exists());
    assert!(!f.project.join("build").exists());
    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    let c = &cov.workers["jvm"].backends["codeql"];
    assert_eq!(c.status, BackendStatus::Ran, "{c:?} {out:?}");
    // The hit is mapped back to the project's path; build logic is counted, not read.
    assert_eq!(c.enumerated, 1);
    assert_eq!(
        cov.workers["jvm"].build_logic, 2,
        "build.gradle.kts and docker.gradle.kts"
    );
}

#[cfg(unix)]
#[test]
fn a_failed_traced_build_falls_back_to_the_java_and_says_why() {
    let f = fixture(TABLE, &[]);
    let log = f.project.parent().unwrap().join("codeql.log");
    fake_codeql(&f, &log);
    gradle_project(&f, true);
    fs::write(f.project.parent().unwrap().join("codeql.log.failbuild"), "").unwrap();
    sweep_with(&f.ctx, Runtime::Jvm, &env(&f)).unwrap();
    let calls = fs::read_to_string(&log).unwrap();
    assert!(calls.contains("--build-mode=manual"), "{calls}");
    assert!(
        calls.contains("--build-mode=none"),
        "the Java is still analysed: {calls}"
    );
    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    let c = &cov.workers["jvm"].backends["codeql"];
    assert_eq!(c.status, BackendStatus::Ran, "{c:?}");
    let lims: Vec<String> = f
        .ctx
        .ledger()
        .unwrap()
        .evidence()
        .flat_map(|e| e.limitations.clone())
        .collect();
    let named = lims
        .iter()
        .find(|l| l.starts_with("1 Kotlin file not extracted: the traced build failed (exit 2)"))
        .unwrap_or_else(|| panic!("{lims:?}"));
    assert!(named.contains("Error: the traced build failed"), "{named}");
    assert!(
        named.contains("run the project's own build online"),
        "{named}"
    );
    assert!(!named.contains("probe --build"), "{named}");
    // The coverage row's reason is one cut line: it leads with the Kotlin note, cut at a word.
    let why = c.reason.clone().unwrap();
    assert!(why.starts_with("1 Kotlin file not extracted"), "{why}");
    assert!(why.chars().count() <= 104, "{why}");
    assert!(
        why.ends_with(" ...") && !why.ends_with("fai ..."),
        "cut at a word: {why}"
    );
}

#[cfg(unix)]
#[test]
fn a_kotlin_only_project_whose_traced_build_fails_is_blocked() {
    let f = fixture(TABLE, &[]);
    let log = f.project.parent().unwrap().join("codeql.log");
    fake_codeql(&f, &log);
    gradle_project(&f, false);
    fs::write(f.project.parent().unwrap().join("codeql.log.failbuild"), "").unwrap();
    sweep_with(&f.ctx, Runtime::Jvm, &env(&f)).unwrap();
    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    let c = &cov.workers["jvm"].backends["codeql"];
    assert_eq!(c.status, BackendStatus::Blocked, "{c:?}");
    let why = c.reason.clone().unwrap();
    assert!(why.contains("failed (exit 2)"), "{why}");
    assert!(!why.contains("probe --build"), "{why}");
}

#[cfg(unix)]
#[test]
fn kotlin_only_in_tests_does_not_trigger_a_traced_build() {
    let f = fixture(TABLE, &[]);
    let log = f.project.parent().unwrap().join("codeql.log");
    fake_codeql(&f, &log);
    gradle_project(&f, true);
    let p = f.project.join("src/main/kotlin/app/A.kt");
    let t = f.project.join("src/test/kotlin/app/ATest.kt");
    fs::create_dir_all(t.parent().unwrap()).unwrap();
    fs::rename(p, t).unwrap();
    sweep_with(&f.ctx, Runtime::Jvm, &env(&f)).unwrap();
    let calls = fs::read_to_string(&log).unwrap();
    assert!(calls.contains("--build-mode=none"), "{calls}");
    assert!(!calls.contains("--build-mode=manual"), "{calls}");
}

#[cfg(unix)]
#[test]
fn kotlin_only_without_tier_1_is_blocked_and_codeql_is_not_run() {
    let f = fixture(TABLE, &[]);
    let log = f.project.parent().unwrap().join("codeql.log");
    fake_codeql(&f, &log);
    gradle_project(&f, false);
    RunState::update(&f.ctx.run_dir, |s| {
        s.consent = None;
        Ok(())
    })
    .unwrap();
    sweep_with(&f.ctx, Runtime::Jvm, &env(&f)).unwrap();
    assert!(
        !log.exists(),
        "codeql must not be called: {:?}",
        fs::read_to_string(&log)
    );
    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    let c = &cov.workers["jvm"].backends["codeql"];
    assert_eq!(c.status, BackendStatus::Blocked);
    let why = c.reason.clone().unwrap();
    assert!(why.contains("Kotlin"), "{why}");
    assert!(why.contains("traced build"), "{why}");
    assert!(why.contains("tier 1"), "{why}");
}

#[cfg(unix)]
#[test]
fn mixed_sources_without_tier_1_run_on_the_java_and_name_the_kotlin_not_extracted() {
    let f = fixture(TABLE, &[]);
    let log = f.project.parent().unwrap().join("codeql.log");
    fake_codeql(&f, &log);
    gradle_project(&f, true);
    RunState::update(&f.ctx.run_dir, |s| {
        s.consent = None;
        Ok(())
    })
    .unwrap();
    sweep_with(&f.ctx, Runtime::Jvm, &env(&f)).unwrap();
    let calls = fs::read_to_string(&log).unwrap();
    assert!(calls.contains("--build-mode=none"), "{calls}");
    let named = f.ctx.ledger().unwrap().evidence().any(|e| {
        e.limitations
            .iter()
            .any(|l| l.starts_with("1 Kotlin file not extracted"))
    });
    assert!(
        named,
        "the evidence names the Kotlin that was not extracted"
    );
}

#[cfg(unix)]
#[test]
fn sonar_is_a_server_only_blocked_row_without_an_install_hint() {
    let f = fixture(TABLE, &[]);
    let log = f.project.parent().unwrap().join("codeql.log");
    fake_codeql(&f, &log);
    // A Java-only project: CodeQL runs buildless, so it counts among the offline backends.
    let put = |rel: &str, body: &str| {
        let p = f.project.join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, body).unwrap();
    };
    put("src/main/java/app/B.java", "package app;\nclass B {}\n");
    let mut table = JVM_CODEQL_TABLE.to_owned();
    table.push_str(
        r#"
[[detector]]
id = "sonar/S2095"
backend = "sonar"
rule = "S2095"
suggested_archetype = "L-HANDLE"
verified = false
"#,
    );
    fs::write(f.data.join("detectors/jvm.toml"), table).unwrap();
    sweep_with(&f.ctx, Runtime::Jvm, &env(&f)).unwrap();
    let cov = CoverageManifest::load(&f.ctx.run_dir).unwrap().unwrap();
    let b = &cov.workers["jvm"].backends;
    assert_eq!(b["codeql"].status, BackendStatus::Ran, "{:?}", b["codeql"]);
    let sonar = &b["sonar"];
    assert_eq!(sonar.status, BackendStatus::Blocked);
    assert_eq!(sonar.install_hint, None, "an install cannot lead to a run");
    let why = sonar.reason.clone().unwrap();
    assert!(why.starts_with("server-only: S5164 and S2095"), "{why}");
    assert!(
        why.contains("offline backends that ran for the same shapes: codeql"),
        "{why}"
    );
    assert!(!why.contains("install"), "{why}");
}

#[test]
fn two_languages_never_share_a_raw_file() {
    assert_eq!(
        dla_backends::sweep::raw_file(Runtime::Jvm, "dla"),
        "jvm-dla.json"
    );
    assert_ne!(
        dla_backends::sweep::raw_file(Runtime::Jvm, "codeql"),
        dla_backends::sweep::raw_file(Runtime::Python, "codeql")
    );
}

/// Two runtimes can share a backend (codeql); each gets its own scratch directory.
#[cfg(unix)]
#[test]
fn the_scratch_directory_is_per_language_and_backend() {
    let f = fixture(TABLE, &PY);
    fake_tool(
        &f,
        "ruff",
        "if [ \"$1\" = \"--version\" ]; then echo 'ruff 1.0.0'; exit 0; fi\nmkdir -p \"$RUFF_CACHE_DIR\"\necho '{\"runs\":[{\"results\":[]}]}'",
    );
    sweep_with(&f.ctx, Runtime::Python, &env(&f)).unwrap();
    let tmp = f.ctx.run_dir.join("tmp");
    assert!(tmp.join("python-ruff/cache/ruff").is_dir());
    assert!(!tmp.join("ruff").exists());
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
    let raw = fs::read_to_string(f.ctx.run_dir.join("sweep/python-ruff.json")).unwrap();
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
