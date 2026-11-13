//! The hooks driver of a Gradle or Maven JVM project (`dla harness gen --lang jvm --driver`):
//! its class path is resolved by the project's build (the same build step as the JUnit repeat,
//! in its `main` mode), not guessed from the workload's classes directory. `gen` runs no build
//! tool; the end-to-end test uses the real Gradle and skips, saying why, when java, gradle or
//! the dependency caches are missing.

use std::{
    fs,
    process::{Command, Stdio},
};

use dla_core::{
    Ledger,
    harness::{AgentKind, HarnessManifest},
    runtime::Runtime,
    state::{Consent, ConsentGiver, RunState, now_utc},
    toolctx::{StageOutcome, ToolCtx},
};
use dla_workload::harness::{
    r#gen::{GenOptions, generate},
    run::{RunOptions, run_points},
};
use tempfile::TempDir;

const CACHE_KT: &str = "package leak\n\nobject Cache {\n    val items = mutableListOf<ByteArray>()\n\n    fun touch() {\n        items.add(ByteArray(1024))\n    }\n}\n";
const MAIN_KT: &str = "package leak\n\nfun main() {\n    Cache.touch()\n}\n";
const GRADLE_KTS_BUILD: &str =
    "plugins { kotlin(\"jvm\") version \"1.9.24\" }\nrepositories { mavenCentral() }\n";
const BUILD_LOGIC_KTS: &str = "import java.util.Properties\n\nval dockerProperties = Properties()\n\nfun loadDockerProps() {}\n";
const CACHE_JAVA: &str = "package leak;\n\nimport java.util.ArrayList;\nimport java.util.List;\n\npublic final class Cache {\n    static final List<byte[]> ITEMS = new ArrayList<>();\n\n    private Cache() {}\n\n    public static void touch() {\n        ITEMS.add(new byte[1024]);\n    }\n}\n";
const MAIN_JAVA: &str = "package leak;\n\npublic final class Main {\n    public static void main(String[] args) {\n        Cache.touch();\n    }\n}\n";
const POM: &str = "<project xmlns=\"http://maven.apache.org/POM/4.0.0\">\n  <modelVersion>4.0.0</modelVersion>\n  <groupId>leak</groupId>\n  <artifactId>leak</artifactId>\n  <version>1</version>\n  <properties>\n    <maven.compiler.source>17</maven.compiler.source>\n    <maven.compiler.target>17</maven.compiler.target>\n  </properties>\n</project>\n";
const HOOK_SETUP: &str =
    "throw new UnsupportedOperationException(\"DLA-HOOK setup has not been filled in\");";
const HOOK_UNIT: &str =
    "throw new UnsupportedOperationException(\"DLA-HOOK unit has not been filled in\");";

struct Fx {
    _dir: TempDir,
    ctx: ToolCtx,
}

fn project(files: &[(&str, &str)]) -> Fx {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    for (rel, text) in files {
        let path = project.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
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
        project,
        run_dir,
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: None,
        worker: None,
    };
    dla_census::census(&ctx, Runtime::Jvm).unwrap();
    dla_workload::discover(&ctx, Some(Runtime::Jvm)).unwrap();
    Fx { _dir: dir, ctx }
}

fn driver_opts(workload: Option<&str>) -> GenOptions {
    GenOptions {
        lang: Runtime::Jvm,
        workload: workload.map(str::to_owned),
        driver: true,
        iter_arg: None,
        step: Some(1),
        iterations: 3,
        warmup: 1,
        probe_findings: Vec::new(),
        auto_probes: false,
        package: None,
        junit: false,
        tests: Vec::new(),
    }
}

fn gen_ok(fx: &Fx, opts: &GenOptions) -> StageOutcome {
    generate(&fx.ctx, opts).unwrap_or_else(|r| panic!("gen refused: {r:?}"))
}

fn manifest(fx: &Fx, id: &str) -> HarnessManifest {
    fx.ctx.harness(id).unwrap()
}

fn tool_ready(tool: &str) -> bool {
    let ok = ["java", "javac", tool].iter().all(|t| {
        Command::new(t)
            .arg(if *t == "java" || *t == "javac" {
                "-version"
            } else {
                "--version"
            })
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
    });
    if !ok {
        eprintln!("skipped: java, javac or {tool} is not available");
    }
    ok
}

fn kotlin_gradle() -> Vec<(&'static str, &'static str)> {
    vec![
        ("build.gradle.kts", GRADLE_KTS_BUILD),
        ("settings.gradle.kts", "rootProject.name = \"leak\"\n"),
        ("docker.gradle.kts", BUILD_LOGIC_KTS),
        ("gradlew", "#!/bin/sh\nexec gradle \"$@\"\n"),
        ("src/main/kotlin/leak/Cache.kt", CACHE_KT),
        ("src/main/kotlin/leak/Main.kt", MAIN_KT),
    ]
}

#[test]
fn a_gradle_driver_gets_the_build_step_not_a_guessed_classes_directory() {
    let fx = project(&kotlin_gradle());
    let out = gen_ok(&fx, &driver_opts(None));
    let m = manifest(&fx, "H1");
    assert_eq!(m.agent, AgentKind::DriverJvm);
    let build = m.build.expect("a build step");
    let all = build.argv.join(" ");
    // The same script as the JUnit repeat, in main mode, compiling the hooks driver.
    assert_eq!(build.argv[0], "sh");
    assert!(
        build.argv[1].ends_with("driver/dla-junit-build.sh"),
        "{all}"
    );
    assert_eq!(build.argv[2], "gradle");
    assert!(build.argv[3].ends_with("work/gradlew"), "{all}");
    assert_eq!(build.argv[7], "main", "{all}");
    assert!(build.argv[8].ends_with("driver/DlaLoop.java"), "{all}");
    // No classes directory is guessed, no standalone javac, and the project's own Kotlin standard
    // library (on the resolved class path) replaces the newest one in ~/.m2.
    for no in [
        "javac -cp",
        "build/classes/java/main",
        "kotlin-stdlib",
        "-cp out",
    ] {
        assert!(!all.contains(no), "{no} in {all}");
    }
    let run = m.run.argv.join(" ");
    assert!(
        run.contains("run.args") && run.ends_with("leak.DlaLoop"),
        "{run}"
    );
    assert!(
        !run.contains("kotlin-stdlib") && !run.contains("-cp"),
        "{run}"
    );
    let dir = fx.ctx.run_dir.join("harness/H1/driver");
    for f in [
        "dla-junit-build.sh",
        "dla-junit.init.gradle",
        "DlaLoop.java",
    ] {
        assert!(dir.join(f).is_file(), "{f}");
    }
    assert!(
        !dir.join("DlaJUnitLoop.java").exists(),
        "this is not a JUnit repeat"
    );
    assert!(
        out.summary.get("junit").is_none() && !out.summary["hooks"].as_array().unwrap().is_empty(),
        "{}",
        out.summary
    );
    assert!(m.tools.iter().any(|t| t == "gradle"), "{:?}", m.tools);
    assert!(
        m.gaps
            .iter()
            .any(|g| g.contains("main runtime class path") && g.contains("retries online")),
        "{:?}",
        m.gaps
    );
    // The build script is a build-logic file: kept in the copy, never a workload or a candidate.
    assert!(
        fx.ctx
            .run_dir
            .join("harness/H1/work/docker.gradle.kts")
            .is_file()
    );
}

#[test]
fn a_maven_driver_gets_the_maven_branch_of_the_build_step() {
    let fx = project(&[
        ("pom.xml", POM),
        ("mvnw", "#!/bin/sh\nexec mvn \"$@\"\n"),
        ("src/main/java/leak/Cache.java", CACHE_JAVA),
        ("src/main/java/leak/Main.java", MAIN_JAVA),
    ]);
    gen_ok(&fx, &driver_opts(None));
    let m = manifest(&fx, "H1");
    let build = m.build.unwrap();
    assert_eq!(build.argv[2], "maven");
    assert!(build.argv[3].ends_with("work/mvnw"), "{:?}", build.argv);
    assert_eq!(build.argv[7], "main");
}

#[test]
fn a_script_built_project_keeps_the_classes_directory() {
    let fx = project(&[
        (
            "build.sh",
            "#!/bin/sh\nmkdir -p out && javac -d out src/leak/*.java\n",
        ),
        ("src/leak/Cache.java", CACHE_JAVA),
        ("src/leak/Main.java", MAIN_JAVA),
    ]);
    gen_ok(&fx, &driver_opts(None));
    let m = manifest(&fx, "H1");
    let all = m.build.unwrap().argv.join(" ");
    assert!(all.contains("javac -cp out"), "{all}");
    assert!(!all.contains("dla-junit-build.sh"), "{all}");
}

/// The Gradle init script and the build script have a `main` branch for each tool.
#[test]
fn the_build_script_has_a_main_mode_for_both_tools() {
    let script = dla_workload::harness::templates::JVM_JUNIT_BUILD;
    for needle in [
        "mode=${6:-test}",
        "src=${7:-$d/DlaJUnitLoop.java}",
        "dlaPrintMainClasspath",
        "-Dmdep.includeScope=runtime",
        "target/classes",
        "\"$src\"",
    ] {
        assert!(script.contains(needle), "{needle}");
    }
    let init = dla_workload::harness::templates::JVM_JUNIT_INIT;
    assert!(
        init.contains("dlaPrintMainClasspath")
            && init.contains("getByName('main').runtimeClasspath")
    );
    assert!(init.contains("dependsOn 'classes'"));
}

/// Fills the two hooks of the generated driver: one touch of the cache per unit.
fn fill_hooks(fx: &Fx, id: &str) {
    let p = fx
        .ctx
        .run_dir
        .join("harness")
        .join(id)
        .join("driver/DlaLoop.java");
    let text = fs::read_to_string(&p).unwrap();
    assert!(text.contains(HOOK_SETUP) && text.contains(HOOK_UNIT));
    let text = text
        .replace(HOOK_SETUP, "")
        .replace(HOOK_UNIT, "Cache.INSTANCE.touch();");
    fs::write(&p, text).unwrap();
}

const OFFLINE_MARKERS: [&str; 6] = [
    "Could not resolve",
    "Could not GET",
    "UnknownHost",
    "Unable to tunnel",
    "Network is unreachable",
    "Plugin [id",
];

/// A real Gradle Kotlin project: the build step's class path holds `build/classes/kotlin/main`
/// and the project's dependency jars (the declared Kotlin standard library), the hooks driver
/// compiles against it and runs.
#[test]
fn a_gradle_kotlin_driver_runs_on_the_resolved_runtime_class_path() {
    if !tool_ready("gradle") {
        return;
    }
    let fx = project(&kotlin_gradle());
    gen_ok(&fx, &driver_opts(None));
    fill_hooks(&fx, "H1");
    let m = manifest(&fx, "H1");
    let result = run_points(&fx.ctx, &m, &RunOptions::default());
    let h = fx.ctx.run_dir.join("harness/H1");
    let log = fs::read_to_string(h.join("logs/build.log")).unwrap_or_default();
    if let Err(r) = &result {
        let online = log
            .rsplit_once("DLA-BUILD the offline attempt failed")
            .map(|(_, t)| t.to_owned())
            .unwrap_or_default();
        if !h.join("driver/run.args").is_file()
            && OFFLINE_MARKERS.iter().any(|k| online.contains(k))
        {
            eprintln!(
                "skipped: dependencies cannot be resolved here (no network): {}",
                r.message
            );
            return;
        }
        panic!("harness run failed: {r:?}\nbuild log:\n{log}");
    }
    // Gradle writes the platform's path separator and separators ('\\' on Windows).
    let cp = fs::read_to_string(h.join("driver/cp.txt"))
        .unwrap()
        .replace('\\', "/");
    let sep = if cfg!(windows) { ';' } else { ':' };
    let entries: Vec<&str> = cp.trim().split(sep).collect();
    assert!(
        entries
            .iter()
            .any(|e| e.ends_with("build/classes/kotlin/main")),
        "the Kotlin classes are on the class path: {cp}"
    );
    assert!(
        entries
            .iter()
            .any(|e| e.contains("kotlin-stdlib") && e.contains("1.9.24")),
        "the project's declared stdlib is on the class path: {cp}"
    );
    // The build script doubles every backslash in run.args.
    let args = fs::read_to_string(h.join("driver/run.args"))
        .unwrap()
        .replace("\\\\", "/")
        .replace('\\', "/");
    assert!(
        args.contains("build/classes/kotlin/main") && args.contains("driver/classes"),
        "{args}"
    );
    assert!(h.join("driver/classes/leak/DlaLoop.class").is_file());
    let out = result.unwrap();
    assert!(!out.series.is_empty(), "{:?}", out.limitations);
}
