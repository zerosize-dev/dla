//! `dla probe` and the known-leak table: verdicts for the shipped entries, the
//! three-valued rules, and the archetypes-off ablation.

use std::{
    fs,
    path::{Path, PathBuf},
};

use dla_core::{
    Ledger, RunState,
    probe::{ProbeReport, Verdict},
    state::now_utc,
    toolctx::ToolCtx,
};
use dla_probe::{ProbeOptions, run};

fn repo_data() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data")
}

/// A project with `files`, a run directory and a data directory holding `table`.
fn project(files: &[(&str, &str)], table: Option<&str>) -> (tempfile::TempDir, ToolCtx) {
    let dir = tempfile::tempdir().unwrap();
    for (p, c) in files {
        let path = dir.path().join("proj").join(p);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, c).unwrap();
    }
    let proj = dir.path().join("proj");
    fs::create_dir_all(&proj).unwrap();
    let data = dir.path().join("data");
    fs::create_dir_all(&data).unwrap();
    if let Some(t) = table {
        fs::write(data.join("known-leaks.toml"), t).unwrap();
    }
    let run_dir = proj.join(".dla/r1");
    fs::create_dir_all(&run_dir).unwrap();
    let mut st = RunState::init(proj.to_str().unwrap(), Some("s"), 3600, None, now_utc()).unwrap();
    st.run_id = "r1".into();
    st.save(&run_dir).unwrap();
    Ledger::create(&run_dir).unwrap();
    let ctx = ToolCtx {
        project: proj,
        run_dir,
        run_id: "r1".into(),
        source_commit: "c".into(),
        data_flag: Some(data),
        worker: None,
    };
    (dir, ctx)
}

fn shipped() -> String {
    fs::read_to_string(repo_data().join("known-leaks.toml")).unwrap()
}

fn verdicts(ctx: &ToolCtx) -> Vec<(String, Verdict, String)> {
    run(ctx, &ProbeOptions::default()).unwrap();
    let text = fs::read_to_string(ctx.run_dir.join("probe/probe.json")).unwrap();
    let r: ProbeReport = serde_json::from_str(&text).unwrap();
    r.known_leaks
        .into_iter()
        .map(|k| (k.id, k.verdict, k.why))
        .collect()
}

fn get<'a>(v: &'a [(String, Verdict, String)], id: &str) -> &'a (String, Verdict, String) {
    v.iter()
        .find(|k| k.0 == id)
        .unwrap_or_else(|| panic!("no {id} in {v:?}"))
}

#[test]
fn the_shipped_table_parses_and_cites_public_sources() {
    let t =
        dla_core::known_leaks::KnownLeaksFile::parse(&shipped(), "data/known-leaks.toml").unwrap();
    assert!(t.leak.len() >= 3);
    for e in &t.leak {
        assert!(e.evidence.starts_with("https://"), "{}", e.id);
    }
    // Every regex in the table compiles.
    fn walk(p: &dla_core::known_leaks::Predicate) {
        use dla_core::known_leaks::Predicate::*;
        match p {
            All(v) | Any(v) => v.iter().for_each(walk),
            Not(b) => walk(b),
            SourceRegex(r) => {
                regex::Regex::new(r).unwrap();
            }
            _ => {}
        }
    }
    t.leak.iter().filter_map(|e| e.when.as_ref()).for_each(walk);
}

#[test]
fn go_timers_follow_the_go_line_and_the_code() {
    let code = "package m\nimport \"time\"\nfunc f() { _ = time.NewTicker(time.Second) }\n";
    let (_d, ctx) = project(
        &[("go.mod", "module m\n\ngo 1.21\n"), ("m.go", code)],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(get(&v, "go-timer-unstopped-before-1.23").1, Verdict::True);
    assert_eq!(get(&v, "go-timer-asynctimerchan-revert").1, Verdict::False);

    let (_d, ctx) = project(
        &[("go.mod", "module m\n\ngo 1.23.0\n"), ("m.go", code)],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(get(&v, "go-timer-unstopped-before-1.23").1, Verdict::False);

    // GODEBUG in go.mod brings the old behavior back for a new go line.
    let (_d, ctx) = project(
        &[
            (
                "go.mod",
                "module m\n\ngo 1.23.0\n\ngodebug asynctimerchan=1\n",
            ),
            ("m.go", code),
        ],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(get(&v, "go-timer-asynctimerchan-revert").1, Verdict::True);

    // Go 1.27 removed the setting (go.dev/doc/godebug, Go 1.27 section): the revert row
    // ends before 1.27, so a go line of 1.27 with the godebug line is no longer a match.
    let (_d, ctx) = project(
        &[
            (
                "go.mod",
                "module m\n\ngo 1.27.0\n\ngodebug asynctimerchan=1\n",
            ),
            ("m.go", code),
        ],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(get(&v, "go-timer-asynctimerchan-revert").1, Verdict::False);

    // Old go line but no timers in the code: not applicable.
    let (_d, ctx) = project(
        &[("go.mod", "module m\n\ngo 1.21\n"), ("m.go", "package m\n")],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(get(&v, "go-timer-unstopped-before-1.23").1, Verdict::False);
}

#[test]
fn netty_sampling_depends_on_the_level_flag() {
    let pom = |extra: &str| {
        format!(
            "<project><dependencies><dependency><groupId>io.netty</groupId>\
<artifactId>netty-handler</artifactId><version>4.1.100.Final</version></dependency>\
</dependencies>{extra}</project>"
        )
    };
    let (_d, ctx) = project(&[("pom.xml", &pom(""))], Some(&shipped()));
    let v = verdicts(&ctx);
    assert_eq!(
        get(&v, "jvm-netty-leak-detection-sampling").1,
        Verdict::True
    );

    let (_d, ctx) = project(
        &[
            ("pom.xml", &pom("")),
            (
                "run.sh",
                "java -Dio.netty.leakDetection.level=PARANOID -jar app.jar\n",
            ),
        ],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(
        get(&v, "jvm-netty-leak-detection-sampling").1,
        Verdict::False
    );

    // No netty in a manifest that is not a lock file: a transitive use is not seen.
    let (_d, ctx) = project(&[("pom.xml", "<project></project>")], Some(&shipped()));
    let v = verdicts(&ctx);
    assert_eq!(
        get(&v, "jvm-netty-leak-detection-sampling").1,
        Verdict::Maybe
    );
}

#[test]
fn gradle_unpinned_versions_and_system_properties() {
    // An unversioned netty dependency matches the `*` range.
    let (_d, ctx) = project(
        &[(
            "build.gradle.kts",
            "dependencies { implementation(\"io.netty:netty-all\") }\n",
        )],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(
        get(&v, "jvm-netty-leak-detection-sampling").1,
        Verdict::True
    );

    // The level set with systemProperty is seen.
    let (_d, ctx) = project(
        &[(
            "build.gradle.kts",
            "dependencies { implementation(\"io.netty:netty-all:4.1.100.Final\") }\n\
tasks.test { systemProperty(\"io.netty.leakDetection.level\", \"paranoid\") }\n",
        )],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(
        get(&v, "jvm-netty-leak-detection-sampling").1,
        Verdict::False
    );

    // Surefire form.
    let (_d, ctx) = project(
        &[(
            "pom.xml",
            "<project><dependencies><dependency><groupId>io.netty</groupId>\
<artifactId>netty-handler</artifactId><version>4.1.100.Final</version></dependency></dependencies>\
<build><plugins><plugin><configuration><systemPropertyVariables>\
<io.netty.leakDetection.level>paranoid</io.netty.leakDetection.level>\
</systemPropertyVariables></configuration></plugin></plugins></build></project>",
        )],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(
        get(&v, "jvm-netty-leak-detection-sampling").1,
        Verdict::False
    );

    // A cache library from a version catalog is not seen: never True for the Spring entry.
    let (_d, ctx) = project(
        &[
            (
                "build.gradle.kts",
                "plugins { id(\"org.springframework.boot\") }\ndependencies { implementation(libs.caffeine) }\n",
            ),
            (
                "src/S.java",
                "class S { @Cacheable(\"a\") String f(String k) { return k; } }\n",
            ),
        ],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(get(&v, "jvm-spring-simple-cache").1, Verdict::Maybe);

    // A declared but unversioned caffeine makes `not(any(library ..))` false.
    let (_d, ctx) = project(
        &[
            (
                "build.gradle.kts",
                "plugins { id(\"org.springframework.boot\") }\ndependencies { implementation(\"com.github.ben-manes.caffeine:caffeine\") }\n",
            ),
            (
                "src/S.java",
                "class S { @Cacheable(\"a\") String f(String k) { return k; } }\n",
            ),
        ],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(get(&v, "jvm-spring-simple-cache").1, Verdict::False);
}

#[test]
fn spring_cache_type_simple_still_fires() {
    let pom = "<project><parent><groupId>org.springframework.boot</groupId>\
<artifactId>spring-boot-starter-parent</artifactId><version>3.2.1</version></parent></project>";
    let java = "@Service class S { @Cacheable(\"a\") String f(String k) { return k; } }\n";
    for (ty, want) in [("simple", Verdict::True), ("redis", Verdict::False)] {
        let (_d, ctx) = project(
            &[
                ("pom.xml", pom),
                ("src/S.java", java),
                (
                    "src/main/resources/application.properties",
                    &format!("spring.cache.type={ty}\n"),
                ),
            ],
            Some(&shipped()),
        );
        let v = verdicts(&ctx);
        assert_eq!(get(&v, "jvm-spring-simple-cache").1, want, "{ty}");
    }
}

#[test]
fn spring_simple_cache_needs_cacheable_code_and_no_provider() {
    let pom = "<project><parent><groupId>org.springframework.boot</groupId>\
<artifactId>spring-boot-starter-parent</artifactId><version>3.2.1</version></parent></project>";
    let java = "@Service class S { @Cacheable(\"a\") String f(String k) { return k; } }\n";
    let (_d, ctx) = project(&[("pom.xml", pom), ("src/S.java", java)], Some(&shipped()));
    let v = verdicts(&ctx);
    // No cache library in a pom (transitive use unseen) is not proof of absence.
    assert_eq!(get(&v, "jvm-spring-simple-cache").1, Verdict::Maybe);

    let (_d, ctx) = project(
        &[
            ("pom.xml", pom),
            ("src/S.java", java),
            (
                "src/main/resources/application.yml",
                "spring:\n  cache:\n    type: caffeine\n",
            ),
        ],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(get(&v, "jvm-spring-simple-cache").1, Verdict::False);

    let (_d, ctx) = project(
        &[("pom.xml", pom), ("src/S.java", "class S {}\n")],
        Some(&shipped()),
    );
    let v = verdicts(&ctx);
    assert_eq!(get(&v, "jvm-spring-simple-cache").1, Verdict::False);
}

#[test]
fn unknown_values_give_maybe_and_negation_keeps_it() {
    let table = r#"
[[leak]]
id = "jvm-demo"
runtime = "jvm"
library = "libx"
versions = "<2"
when = { not = { source_regex = "zzz" } }
effect = "e"
holder = "container-field"
archetype = "L-COLLECT"
evidence = "https://example.com/x"
remedy = "r"

[[leak]]
id = "jvm-demo-unpinned"
runtime = "jvm"
library = "liby"
versions = "<2"
effect = "e"
holder = "container-field"
archetype = "L-COLLECT"
evidence = "https://example.com/y"
remedy = "r"

[[leak]]
id = "node-demo"
runtime = "node"
library = "node"
versions = "<18"
effect = "e"
holder = "container-field"
archetype = "L-COLLECT"
evidence = "https://example.com/z"
remedy = "r"
"#;
    // libx is in range, no zzz in the sources: not(false) = true.
    let pom = "<project><dependencies>\
<dependency><groupId>g</groupId><artifactId>libx</artifactId><version>1.5</version></dependency>\
<dependency><groupId>g</groupId><artifactId>liby</artifactId></dependency>\
</dependencies></project>";
    let (_d, ctx) = project(&[("pom.xml", pom), ("A.java", "class A {}\n")], Some(table));
    let v = verdicts(&ctx);
    assert_eq!(get(&v, "jvm-demo").1, Verdict::True);
    // The version is managed elsewhere: cannot decide.
    assert_eq!(get(&v, "jvm-demo-unpinned").1, Verdict::Maybe);
    // The project has no node: the node entry is not reported.
    assert!(v.iter().all(|k| k.0 != "node-demo"));

    // A java project with no manifest at all: maybe.
    let (_d, ctx) = project(&[("A.java", "class A {}\n")], Some(table));
    let v = verdicts(&ctx);
    assert_eq!(get(&v, "jvm-demo").1, Verdict::Maybe);
}

#[test]
fn archetypes_off_removes_the_table_and_probe_still_runs() {
    let (_d, ctx) = project(
        &[("go.mod", "module m\n\ngo 1.21\n"), ("m.go", "package m\n")],
        None,
    );
    assert!(
        !ctx.data_flag
            .as_ref()
            .unwrap()
            .join("known-leaks.toml")
            .exists()
    );
    assert!(verdicts(&ctx).is_empty());
    let text = fs::read_to_string(ctx.run_dir.join("probe/probe.json")).unwrap();
    assert!(!text.contains("known_leaks"));
}

#[test]
fn an_invalid_table_is_a_limitation_not_a_failure() {
    let (_d, ctx) = project(
        &[("go.mod", "module m\n\ngo 1.21\n")],
        Some("[[leak]]\nid = 1\n"),
    );
    let out = run(&ctx, &ProbeOptions::default()).unwrap();
    assert_eq!(out.evidence.len(), 1);
    let ledger = fs::read_to_string(ctx.run_dir.join("ledger.jsonl")).unwrap();
    assert!(ledger.contains("known leaks not checked"), "{ledger}");
}

#[test]
fn evidence_stays_bounded_with_every_entry_listed() {
    let (_d, ctx) = project(
        &[
            ("go.mod", "module m\n\ngo 1.21\n"),
            ("m.go", "package m\nvar _ = time.NewTimer(1)\n"),
        ],
        Some(&shipped()),
    );
    let out = run(&ctx, &ProbeOptions::default()).unwrap();
    let ledger = fs::read_to_string(ctx.run_dir.join("ledger.jsonl")).unwrap();
    assert!(ledger.contains("go-timer-unstopped-before-1.23"));
    assert!(out.summary.to_string().len() <= 2048);
}

/// Sonar is server-only: `dla probe` writes the same `blocked` row the sweep does, never a
/// `missing` row (and never lists it among the tools to install).
#[test]
fn sonar_is_a_blocked_server_only_row_not_a_missing_tool() {
    use dla_core::coverage::{BackendStatus, CoverageManifest};
    let (_d, ctx) = project(&[("pom.xml", "<project/>")], None);
    run(&ctx, &ProbeOptions::default()).unwrap();
    let cov = CoverageManifest::load(&ctx.run_dir).unwrap().unwrap();
    let sonar = &cov.workers["jvm"].backends["sonar"];
    if sonar.status != BackendStatus::Ran {
        assert_eq!(sonar.status, BackendStatus::Blocked, "{sonar:?}");
        let why = sonar.reason.clone().unwrap();
        assert!(why.starts_with("server-only:"), "{why}");
        assert_eq!(sonar.install_hint, None);
    }
    let ledger = ctx.ledger().unwrap();
    let all = serde_json::to_string(&ledger.evidence().collect::<Vec<_>>()).unwrap();
    assert!(
        all.contains("tools_missing"),
        "the probe evidence exists: {all}"
    );
    assert!(
        sonar.status == BackendStatus::Ran || !all.contains("jvm:sonar"),
        "not listed as a missing tool: {all}"
    );
}
