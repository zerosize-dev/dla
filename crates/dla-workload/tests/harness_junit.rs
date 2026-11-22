//! The JUnit repeat harness (`dla harness gen --lang jvm --junit`): what `gen` writes for Gradle
//! and Maven projects (no build tool runs), and end to end on small projects with the real
//! tools. An end-to-end test prints why it is skipped when a tool is missing or the
//! dependencies cannot be resolved (no network); it never fails on a network error.

use std::{
    fs,
    process::{Command, Stdio},
    time::Instant,
};

use dla_core::{
    Ledger, Refusal,
    harness::{AgentKind, HarnessManifest, HarnessMode},
    runtime::Runtime,
    series::{KeyMode, SeriesFile},
    state::{Consent, ConsentGiver, RunState, now_utc},
    toolctx::{StageOutcome, ToolCtx},
};
use dla_workload::harness::{
    r#gen::{GenOptions, generate},
    run::{RunOptions, run_points},
};
use tempfile::TempDir;

const CACHE_JAVA: &str = "package leak;\n\nimport java.util.ArrayList;\nimport java.util.List;\n\npublic final class Cache {\n    static final List<byte[]> ITEMS = new ArrayList<>();\n\n    private Cache() {}\n\n    public static void touch() {\n        ITEMS.add(new byte[1024]);\n    }\n}\n";
const CACHE_TEST_JAVA: &str = "package leak;\n\nimport org.junit.jupiter.api.Test;\n\nclass CacheTest {\n    @Test\n    void touches() {\n        Cache.touch();\n    }\n}\n";
const CACHE_KT: &str = "package leak\n\nobject Cache {\n    val items = mutableListOf<ByteArray>()\n\n    fun touch() {\n        items.add(ByteArray(1024))\n    }\n}\n";
const CACHE_TEST_KT: &str = "package leak\n\nimport org.junit.jupiter.api.Test\n\nclass CacheTest {\n    @Test\n    fun touches() {\n        Cache.touch()\n    }\n}\n";
const GRADLE_BUILD: &str = "plugins { id 'java' }\nrepositories { mavenCentral() }\ndependencies {\n    testImplementation platform('org.junit:junit-bom:5.10.2')\n    testImplementation 'org.junit.jupiter:junit-jupiter'\n}\ntest { useJUnitPlatform() }\n";
// The Kotlin Gradle plugin of these tests knows no JVM newer than 21, and Gradle defaults both
// compilers to the JDK it runs on: without a pin, a build on JDK 22 to 24 would stop with
// "Inconsistent JVM-target compatibility detected for tasks compileJava and compileKotlin"
// (inferred from the log of a JDK 25 runner, which printed it for compileJava 25 against
// compileKotlin 21; no JDK 22 to 24 was available to run it, and the tests pass with and
// without the pin on JDK 21). On JDK 25 the Kotlin compiler itself cannot run; those tests skip
// (`kotlin_compiler_runs`).
const GRADLE_KTS_BUILD: &str = "plugins { kotlin(\"jvm\") version \"1.9.24\" }\nrepositories { mavenCentral() }\ndependencies {\n    testImplementation(platform(\"org.junit:junit-bom:5.10.2\"))\n    testImplementation(\"org.junit.jupiter:junit-jupiter\")\n}\ntasks.test { useJUnitPlatform() }\njava {\n    sourceCompatibility = JavaVersion.VERSION_17\n    targetCompatibility = JavaVersion.VERSION_17\n}\ntasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach { kotlinOptions { jvmTarget = \"17\" } }\n";
const POM: &str = "<project xmlns=\"http://maven.apache.org/POM/4.0.0\">\n  <modelVersion>4.0.0</modelVersion>\n  <groupId>leak</groupId>\n  <artifactId>leak</artifactId>\n  <version>1</version>\n  <properties>\n    <maven.compiler.source>17</maven.compiler.source>\n    <maven.compiler.target>17</maven.compiler.target>\n    <project.build.sourceEncoding>UTF-8</project.build.sourceEncoding>\n  </properties>\n  <dependencies>\n    <dependency>\n      <groupId>org.junit.jupiter</groupId>\n      <artifactId>junit-jupiter</artifactId>\n      <version>5.10.2</version>\n      <scope>test</scope>\n    </dependency>\n  </dependencies>\n  <build>\n    <plugins>\n      <plugin>\n        <artifactId>maven-surefire-plugin</artifactId>\n        <version>3.2.5</version>\n      </plugin>\n    </plugins>\n  </build>\n</project>\n";

const BOOT_KTS_BUILD: &str = "plugins {\n    kotlin(\"jvm\") version \"1.9.25\"\n    kotlin(\"plugin.spring\") version \"1.9.25\"\n    id(\"org.springframework.boot\") version \"3.3.5\"\n    id(\"io.spring.dependency-management\") version \"1.1.6\"\n}\nrepositories { mavenCentral() }\ndependencies {\n    implementation(\"org.springframework.boot:spring-boot-starter\")\n    testImplementation(\"org.springframework.boot:spring-boot-starter-test\")\n}\ntasks.withType<Test> { useJUnitPlatform() }\njava {\n    sourceCompatibility = JavaVersion.VERSION_17\n    targetCompatibility = JavaVersion.VERSION_17\n}\ntasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().configureEach { kotlinOptions { jvmTarget = \"17\" } }\n";
const BOOT_APP_KT: &str = "package com.acme.shop\n\nimport org.springframework.boot.autoconfigure.SpringBootApplication\nimport org.springframework.boot.runApplication\n\n@SpringBootApplication\nclass Application\n\nfun main(args: Array<String>) {\n    runApplication<Application>(*args)\n}\n";
// `Routes.kt` holds `UserController`, and a top-level function that compiles to `RoutesKt`.
const BOOT_ROUTES_KT: &str = "package com.acme.shop\n\nimport org.springframework.stereotype.Component\n\n@Component\nclass UserController(private val users: UserService) {\n    fun user(id: String): String = users.lookup(id)\n\n    fun ping(): String = slug(\"Ping Pong\")\n}\n\nfun routeCount(): Int = 2\n";
const BOOT_SERVICE_KT: &str = "package com.acme.shop\n\nimport org.springframework.stereotype.Service\n\n@Service\nclass UserService {\n    private val cache = HashMap<String, String>()\n\n    fun lookup(id: String): String = cache.getOrPut(id) { \"user-$id\" }\n}\n";
const BOOT_UTIL_KT: &str =
    "package com.acme.shop\n\nfun slug(s: String): String = s.lowercase().replace(' ', '-')\n";
// The directory omits the root of the package, as Kotlin projects do.
const BOOT_INVOICES_KT: &str = "package com.acme.shop.billing\n\nimport org.springframework.stereotype.Component\n\n@Component\nclass Invoices {\n    fun total(n: Int): Int = n * 3\n}\n";
const BOOT_TEST_KT: &str = "package com.acme.shop\n\nimport com.acme.shop.billing.Invoices\nimport org.junit.jupiter.api.Test\nimport org.springframework.beans.factory.annotation.Autowired\nimport org.springframework.boot.test.context.SpringBootTest\n\n@SpringBootTest\nclass ApplicationTests(\n    @Autowired val routes: UserController,\n    @Autowired val invoices: Invoices,\n) {\n    @Test\n    fun serves() {\n        routes.user(\"u\" + System.nanoTime())\n        routes.ping()\n        invoices.total(2)\n    }\n}\n";

struct Fx {
    _dir: TempDir,
    ctx: ToolCtx,
}

/// A project with the given files, an initialised run with consent for tier 1, and the JVM
/// census and workload discovery done.
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

fn gradle_java() -> Vec<(&'static str, &'static str)> {
    vec![
        ("build.gradle", GRADLE_BUILD),
        ("settings.gradle", "rootProject.name = 'leak'\n"),
        ("gradlew", "#!/bin/sh\nexec gradle \"$@\"\n"),
        ("src/main/java/leak/Cache.java", CACHE_JAVA),
        ("src/test/java/leak/CacheTest.java", CACHE_TEST_JAVA),
    ]
}

fn junit_opts() -> GenOptions {
    GenOptions {
        lang: Runtime::Jvm,
        workload: None,
        driver: false,
        iter_arg: None,
        step: None,
        iterations: 8,
        warmup: 1,
        probe_findings: Vec::new(),
        entry: Vec::new(),
        next: None,
        auto_probes: true,
        package: None,
        junit: true,
        tests: Vec::new(),
        features: Vec::new(),
        no_default_features: false,
    }
}

fn have_jdk() -> bool {
    ["java", "javac", "jar"].iter().all(|t| {
        let ok = Command::new(t)
            .arg(if *t == "jar" { "--version" } else { "-version" })
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success());
        if !ok {
            eprintln!("skipped: {t} is not available");
        }
        ok
    })
}

fn have_tool(tool: &str) -> bool {
    let ok = Command::new(tool)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success());
    if !ok {
        eprintln!("skipped: {tool} is not available");
    }
    ok
}

fn manifest(fx: &Fx, id: &str) -> HarnessManifest {
    fx.ctx.harness(id).unwrap()
}

fn driver_text(fx: &Fx, id: &str) -> String {
    fs::read_to_string(
        fx.ctx
            .run_dir
            .join("harness")
            .join(id)
            .join("driver/DlaJUnitLoop.java"),
    )
    .unwrap()
}

fn gen_ok(fx: &Fx, opts: &GenOptions) -> StageOutcome {
    generate(&fx.ctx, opts).unwrap_or_else(|r| panic!("gen refused: {r:?}"))
}

fn gen_err(fx: &Fx, opts: &GenOptions) -> Refusal {
    generate(&fx.ctx, opts).expect_err("gen should refuse")
}

#[test]
fn gen_writes_a_driver_that_runs_the_test_class_with_no_hooks() {
    if !have_jdk() {
        return;
    }
    let fx = project(&gradle_java());
    let out = gen_ok(&fx, &junit_opts());
    let m = manifest(&fx, "H1");
    assert_eq!(m.mode, HarnessMode::Driver);
    assert_eq!(m.agent, AgentKind::DriverJvm);
    assert_eq!(
        m.workload.as_deref(),
        Some("jvm:test:src/test/java/leak/CacheTest.java")
    );
    assert_eq!((m.warmup, m.iterations, m.step), (1, 8, 2));
    assert_eq!(m.timeout_secs, 1800);
    assert_eq!(m.key_modes, [KeyMode::Fresh]);
    assert!(m.tools.iter().any(|t| t == "junit-platform-launcher"));

    let hdir = fx.ctx.run_dir.join("harness/H1");
    let driver = hdir.join("driver");
    let build = m.build.expect("a build step");
    assert_eq!(build.argv[0], "sh");
    assert!(
        build.argv[1].ends_with("driver/dla-junit-build.sh"),
        "{:?}",
        build.argv
    );
    assert_eq!(build.argv[2], "gradle");
    // The project's wrapper (here a stand-in) is preferred over the gradle on PATH.
    assert!(
        build.argv[3].ends_with("harness/H1/work/gradlew"),
        "{:?}",
        build.argv
    );
    assert!(build.argv[4].ends_with("harness/H1/driver"));
    assert!(build.argv[5].ends_with("agent/dla-agent.jar"));
    assert_eq!(m.run.argv.len(), 3);
    assert_eq!(m.run.argv[0], "java");
    assert!(m.run.argv[1].starts_with('@') && m.run.argv[1].ends_with("driver/run.args"));
    assert_eq!(m.run.argv[2], "DlaJUnitLoop");
    let jto = m
        .run
        .env
        .iter()
        .find(|(k, _)| k == "JAVA_TOOL_OPTIONS")
        .map(|(_, v)| v.as_str())
        .unwrap();
    assert!(jto.contains("-javaagent:"), "{jto}");

    let text = driver_text(&fx, "H1");
    assert!(text.contains("SELECT = {\"leak.CacheTest\"}"), "{text}");
    assert!(text.contains("intEnv(\"DLA_STEP\", 2)"));
    assert!(text.contains("intEnv(\"DLA_WARMUP\", 1)"));
    assert!(text.contains("intEnv(\"DLA_ROUNDS\", 8)"));
    assert!(text.contains("DlaDump.checkpoint(done, round == total)"));
    assert!(!text.contains("@@DLA_") && !text.contains("DLA-HOOK"));
    assert!(driver.join("dla-junit.init.gradle").is_file());
    assert!(driver.join("dla-junit-build.sh").is_file());

    assert_eq!(out.summary["hooks"], serde_json::json!([]));
    assert_eq!(out.summary["junit"], true);
    assert!(
        out.summary["instructions"]
            .as_str()
            .unwrap()
            .contains("nothing to fill"),
        "{}",
        out.summary
    );
    assert_eq!(out.summary["next"], "dla harness run H1");
    assert!(out.summary.get("edit").is_none());
}

#[test]
fn flags_override_the_defaults_and_tests_select_classes_and_methods() {
    if !have_jdk() {
        return;
    }
    let fx = project(&gradle_java());
    let mut o = junit_opts();
    o.step = Some(5);
    o.warmup = 2;
    o.iterations = 6;
    o.tests = vec!["leak.CacheTest#touches".into(), "leak.OtherTest".into()];
    gen_ok(&fx, &o);
    let m = manifest(&fx, "H1");
    assert_eq!((m.warmup, m.iterations, m.step), (2, 6, 5));
    let text = driver_text(&fx, "H1");
    assert!(
        text.contains("SELECT = {\"leak.CacheTest#touches\", \"leak.OtherTest\"}"),
        "{text}"
    );
    assert!(text.contains("intEnv(\"DLA_STEP\", 5)"));
}

#[test]
fn a_selector_that_is_not_a_class_name_is_refused() {
    if !have_jdk() {
        return;
    }
    let fx = project(&gradle_java());
    let mut o = junit_opts();
    o.tests = vec!["leak.CacheTest\"; evil(); \"".into()];
    let r = gen_err(&fx, &o);
    assert_eq!(r.code, "invalid-value", "{r:?}");
    // --tests without --junit has nothing to apply to.
    let mut o = junit_opts();
    o.junit = false;
    o.tests = vec!["leak.CacheTest".into()];
    let r = gen_err(&fx, &o);
    assert_eq!(r.code, "invalid-value", "{r:?}");
}

#[test]
fn a_kotlin_test_class_is_named_after_its_class_not_the_kt_facade() {
    if !have_jdk() {
        return;
    }
    let fx = project(&[
        ("build.gradle.kts", GRADLE_KTS_BUILD),
        ("settings.gradle.kts", "rootProject.name = \"leak\"\n"),
        ("gradlew", "#!/bin/sh\nexec gradle \"$@\"\n"),
        ("src/main/kotlin/leak/Cache.kt", CACHE_KT),
        ("src/test/kotlin/leak/CacheTest.kt", CACHE_TEST_KT),
    ]);
    gen_ok(&fx, &junit_opts());
    let text = driver_text(&fx, "H1");
    assert!(text.contains("SELECT = {\"leak.CacheTest\"}"), "{text}");
    assert!(!text.contains("CacheTestKt"));
}

#[test]
fn junit4_and_testng_tests_are_refused_naming_the_vintage_engine() {
    if !have_jdk() {
        return;
    }
    let junit4 = "package leak;\n\nimport org.junit.Test;\n\npublic class CacheTest {\n    @Test\n    public void touches() { Cache.touch(); }\n}\n";
    let mut files = gradle_java();
    files.retain(|(rel, _)| !rel.ends_with("CacheTest.java"));
    files.push(("src/test/java/leak/CacheTest.java", junit4));
    let fx = project(&files);
    let r = gen_err(&fx, &junit_opts());
    assert_eq!(r.code, "driver-unavailable", "{r:?}");
    assert!(r.message.contains("vintage"), "{}", r.message);
    // Nothing was created by the refused gen.
    assert!(!fx.ctx.run_dir.join("harness/H1").exists());
}

/// Id of the discovered workload defined by the file ending in `file`.
fn workload_of(fx: &Fx, file: &str) -> String {
    let doc: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            fx.ctx
                .run_dir
                .join("workload")
                .join(dla_workload::report_file(None)),
        )
        .unwrap(),
    )
    .unwrap();
    doc["workloads"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["file"].as_str().is_some_and(|f| f.ends_with(file)))
        .unwrap_or_else(|| panic!("no workload for {file} in {doc}"))["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// An explicit `--driver` is always the plain driver with hooks, also when the project has only
/// JUnit 5 tests and also when the workload named is a JUnit 5 test file.
#[test]
fn driver_is_always_the_plain_driver_with_hooks_never_a_junit_repeat() {
    if !have_jdk() {
        return;
    }
    let fx = project(&gradle_java());
    let mut o = junit_opts();
    o.junit = false;
    o.driver = true;
    let out = gen_ok(&fx, &o);
    let m = manifest(&fx, "H1");
    assert_eq!(m.agent, AgentKind::DriverJvm);
    assert!(out.summary.get("junit").is_none(), "{}", out.summary);
    assert!(!out.summary["hooks"].as_array().unwrap().is_empty());
    assert!(
        out.summary["instructions"]
            .as_str()
            .unwrap()
            .contains("replace the body"),
        "{}",
        out.summary
    );
    // The hook driver names the variant with nothing to fill.
    assert!(
        out.summary["junit_hint"]
            .as_str()
            .unwrap_or_default()
            .contains("dla harness gen --lang jvm --junit"),
        "{}",
        out.summary
    );
    let driver = fx.ctx.run_dir.join("harness/H1/driver");
    assert!(!driver.join("DlaJUnitLoop.java").exists());
    assert!(driver.join("DlaLoop.java").exists());
    // The JUnit 5 test file named as the workload: still the plain driver.
    let id = workload_of(&fx, "CacheTest.java");
    o.workload = Some(id);
    let out = gen_ok(&fx, &o);
    assert!(out.summary.get("junit").is_none(), "{}", out.summary);
    assert!(
        fx.ctx
            .run_dir
            .join("harness/H2/driver/DlaLoop.java")
            .exists()
    );
    assert!(
        !fx.ctx
            .run_dir
            .join("harness/H2/driver/DlaJUnitLoop.java")
            .exists()
    );
    // `--junit` together with `--driver` is the JUnit repeat.
    o.workload = None;
    o.junit = true;
    let out = gen_ok(&fx, &o);
    assert_eq!(out.summary["junit"], true);
    assert!(
        fx.ctx
            .run_dir
            .join("harness/H3/driver/DlaJUnitLoop.java")
            .exists()
    );
}

/// Neither flag on a project of JUnit 5 tests only: the refusal names `--junit`.
#[test]
fn without_a_flag_a_junit_only_project_is_told_about_junit() {
    if !have_jdk() {
        return;
    }
    let fx = project(&gradle_java());
    let mut o = junit_opts();
    o.junit = false;
    let r = gen_err(&fx, &o);
    assert_eq!(r.code, "workload-required", "{r:?}");
    assert!(
        r.fix.contains("dla harness gen --lang jvm --junit"),
        "{}",
        r.fix
    );
}

/// A `main` under src/test of a Gradle project is a test-kind workload that is not JUnit 5: a
/// plain `--driver` on it worked before the JUnit repeat existed and still does.
#[test]
fn driver_on_a_main_under_src_test_is_not_refused_as_a_junit_repeat() {
    if !have_jdk() {
        return;
    }
    let main = "package leak;\n\npublic class CacheMain {\n    public static void main(String[] args) {\n        Cache.touch();\n    }\n}\n";
    let fx = project(&[
        ("build.gradle", GRADLE_BUILD),
        ("settings.gradle", "rootProject.name = 'leak'\n"),
        ("gradlew", "#!/bin/sh\nexec gradle \"$@\"\n"),
        ("src/main/java/leak/Cache.java", CACHE_JAVA),
        ("src/test/java/leak/CacheMain.java", main),
    ]);
    let mut o = junit_opts();
    o.junit = false;
    o.driver = true;
    o.workload = Some(workload_of(&fx, "CacheMain.java"));
    let out = gen_ok(&fx, &o);
    assert!(out.summary.get("junit").is_none(), "{}", out.summary);
    // No JUnit 5 test in the project: no hint.
    assert!(out.summary.get("junit_hint").is_none(), "{}", out.summary);
    assert!(
        fx.ctx
            .run_dir
            .join("harness/H1/driver/DlaLoop.java")
            .exists()
    );
    // `--junit` on it is still refused, naming the reason.
    o.junit = true;
    o.workload = None;
    let r = gen_err(&fx, &o);
    assert_eq!(r.code, "driver-unavailable", "{r:?}");
}

#[test]
fn a_kotlin_backtick_test_name_with_spaces_is_selectable() {
    if !have_jdk() {
        return;
    }
    let fx = project(&gradle_java());
    let mut o = junit_opts();
    o.tests = vec!["leak.CacheTest#publishes to the broker".into()];
    gen_ok(&fx, &o);
    let text = driver_text(&fx, "H1");
    assert!(
        text.contains("\"leak.CacheTest#publishes to the broker\""),
        "{text}"
    );
    for bad in [
        "leak.CacheTest#a\"b",
        "leak.CacheTest#a\\b",
        "leak.CacheTest# ",
        "leak.CacheTest#a\nb",
    ] {
        let mut o = junit_opts();
        o.tests = vec![bad.into()];
        assert_eq!(gen_err(&fx, &o).code, "invalid-value", "{bad:?}");
    }
}

#[test]
fn a_maven_project_gets_its_wrapper_or_mvn_and_the_maven_branch() {
    if !have_jdk() {
        return;
    }
    let fx = project(&[
        ("pom.xml", POM),
        ("mvnw", "#!/bin/sh\nexec mvn \"$@\"\n"),
        ("src/main/java/leak/Cache.java", CACHE_JAVA),
        ("src/test/java/leak/CacheTest.java", CACHE_TEST_JAVA),
    ]);
    gen_ok(&fx, &junit_opts());
    let m = manifest(&fx, "H1");
    let build = m.build.unwrap();
    assert_eq!(build.argv[2], "maven");
    assert!(build.argv[3].ends_with("work/mvnw"), "{:?}", build.argv);
    assert!(driver_text(&fx, "H1").contains("leak.CacheTest"));
}

#[test]
fn the_build_script_has_both_tool_branches() {
    let script = dla_workload::harness::templates::JVM_JUNIT_BUILD;
    for needle in [
        "dlaPrintTestClasspath",
        "--no-daemon",
        "--offline",
        "dependency:build-classpath",
        "-Dmdep.includeScope=test",
        "target/test-classes",
        "junit-platform-launcher",
        "-proc:none",
        "run.args",
    ] {
        assert!(script.contains(needle), "{needle}");
    }
    let init = dla_workload::harness::templates::JVM_JUNIT_INIT;
    assert!(init.contains("sourceSets") && init.contains("detachedConfiguration"));
}

// ---------------------------------------------------------------------------------------
// End to end
// ---------------------------------------------------------------------------------------

/// Markers in a build log that mean the dependencies could not be resolved (no network).
const OFFLINE_MARKERS: [&str; 9] = [
    "Could not resolve",
    "Could not GET",
    "Could not HEAD",
    "UnknownHost",
    "Unable to tunnel",
    "Connect timed out",
    "Network is unreachable",
    "Could not transfer artifact",
    "Plugin [id",
];

/// True when the build step ended without a class path because the dependencies could not be
/// downloaded. The build script tries offline first and online second and logs a
/// `DLA-BUILD` line between them: only the online attempt's output counts, so an offline
/// "Could not resolve" that the online retry fixed (a cold cache) never hides a real failure.
fn build_blocked_by_network(fx: &Fx) -> bool {
    let h = fx.ctx.run_dir.join("harness/H1");
    if h.join("driver/run.args").is_file() {
        return false;
    }
    let log = fs::read_to_string(h.join("logs/build.log")).unwrap_or_default();
    let online = log
        .rsplit_once("DLA-BUILD the offline attempt failed")
        .map(|(_, tail)| tail);
    online.is_some_and(|tail| OFFLINE_MARKERS.iter().any(|m| tail.contains(m)))
}

/// Runs `dla harness run H1` for a generated JUnit harness; returns the series, or `None`
/// (after printing why) when the dependencies could not be resolved.
fn run_harness(fx: &Fx) -> Option<SeriesFile> {
    let m = manifest(fx, "H1");
    let started = Instant::now();
    let result = run_points(&fx.ctx, &m, &RunOptions::default());
    eprintln!("harness run took {:.1} s", started.elapsed().as_secs_f64());
    match result {
        Ok(out) => {
            assert_eq!(out.series.len(), 1, "{:?}", out.limitations);
            Some(SeriesFile::load(&fx.ctx.run_dir, &out.series[0]).unwrap())
        }
        Err(r) => {
            let log = fs::read_to_string(fx.ctx.run_dir.join("harness/H1/logs/build.log"))
                .unwrap_or_default();
            if build_blocked_by_network(fx) {
                eprintln!(
                    "skipped: dependencies cannot be resolved here (no network): {}",
                    r.message
                );
                return None;
            }
            panic!("harness run failed: {r:?}\nbuild log:\n{log}");
        }
    }
}

/// The log of the measured process (every log of the harness except the build's).
fn run_log(fx: &Fx) -> String {
    let dir = fx.ctx.run_dir.join("harness/H1/logs");
    let mut text = String::new();
    for e in fs::read_dir(dir).unwrap().flatten() {
        if e.file_name() != "build.log" {
            text.push_str(&fs::read_to_string(e.path()).unwrap_or_default());
        }
    }
    text
}

fn check_series(fx: &Fx, series: &SeriesFile) {
    // Every round wrote a checkpoint: warm-up + measured points.
    assert_eq!(
        series.points.len(),
        9,
        "{:?}",
        series.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
    );
    let xs: Vec<f64> = series.points.iter().map(|p| p.x).collect();
    // x counts executions of the tests: 2 per round.
    assert_eq!(xs[0], 2.0);
    assert_eq!(xs[8], 18.0);
    // Growth: the probe on Cache's list when the census named it (the Kotlin object's
    // singleton field is a flat probe of size 1, not the list), else the heap.
    let names: Vec<&str> = series.columns.iter().map(|c| c.name.as_str()).collect();
    let rises =
        |col: usize| -> Vec<f64> { series.points.iter().filter_map(|p| p.values[col]).collect() };
    let list = names
        .iter()
        .position(|n| n.starts_with("probe:") && n.to_lowercase().contains("cache.items"));
    if let Some(col) = list {
        let values = rises(col);
        // One touch per execution: the probe rises by 16 over the 8 later rounds of 2.
        assert_eq!(
            values.last().unwrap() - values.first().unwrap(),
            16.0,
            "{values:?}"
        );
    } else {
        let col = series.column_index("jvm.heap_used").expect("a heap column");
        let values = rises(col);
        assert!(values.len() >= 9, "{values:?} {names:?}");
        assert!(
            values.last().unwrap() > values.first().unwrap(),
            "{values:?} {names:?}"
        );
    }
    let log = run_log(fx);
    assert!(
        log.contains("DLA-JUNIT summary: units=18 tests_run=18 aborted=0 skipped=0 failed_units=0"),
        "{log}"
    );
}

fn tool_ready(tool: &str) -> bool {
    have_jdk() && have_tool(tool)
}

/// The major version of the JDK Gradle runs on: `JAVA_HOME` when it is set, else `java` on
/// the path.
fn gradle_java_major() -> Option<u32> {
    let java = std::env::var_os("JAVA_HOME")
        .map(|h| std::path::Path::new(&h).join("bin/java"))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| "java".into());
    let out = Command::new(java)
        .arg("-version")
        .stdin(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stderr).into_owned();
    let v = text.lines().find_map(|l| l.split_once("version \""))?.1;
    let mut parts = v.split(['.', '"', '-', '+', '_']);
    let first: u32 = parts.next()?.parse().ok()?;
    // `1.8.0_402` is JDK 8.
    if first == 1 {
        parts.next()?.parse().ok()
    } else {
        Some(first)
    }
}

/// Whether the Kotlin compiler of the Kotlin Gradle plugin used here can run. Its 1.9 and 2.0
/// releases fail `JavaVersion.parse` (`IllegalArgumentException: 25.0.x`) on JDK 25 and
/// newer, so a Kotlin project cannot be built there, whatever the JVM target is pinned to.
fn kotlin_compiler_runs() -> bool {
    match gradle_java_major() {
        Some(n) if n >= 25 => {
            eprintln!(
                "skipped: the Kotlin compiler of this test's Kotlin Gradle plugin cannot run on JDK {n} (JDK 24 or older needed)"
            );
            false
        }
        _ => true,
    }
}

#[test]
fn gradle_project_series_grows_with_the_repeated_tests() {
    if !tool_ready("gradle") {
        return;
    }
    let fx = project(&gradle_java());
    gen_ok(&fx, &junit_opts());
    let Some(series) = run_harness(&fx) else {
        return;
    };
    check_series(&fx, &series);
}

#[test]
fn mode_both_on_a_junit_repeat_runs_the_one_supported_mode_with_a_note() {
    if !tool_ready("gradle") {
        return;
    }
    let fx = project(&gradle_java());
    gen_ok(&fx, &junit_opts());
    let m = manifest(&fx, "H1");
    assert_eq!(m.key_modes, vec![KeyMode::Fresh]);
    // `both` and `repeated` alone: the note is there, and only the one mode runs.
    for modes in [
        vec![KeyMode::Fresh, KeyMode::Repeated],
        vec![KeyMode::Repeated],
    ] {
        let opts = RunOptions {
            modes: modes.clone(),
            ..RunOptions::default()
        };
        match run_points(&fx.ctx, &m, &opts) {
            Ok(out) => {
                // One series, one measured process: the loop did not run twice.
                assert_eq!(out.series.len(), 1, "{modes:?} {:?}", out.limitations);
                assert!(
                    out.limitations
                        .iter()
                        .any(|l| l.contains("supports the key mode(s) fresh only")),
                    "{modes:?} {:?}",
                    out.limitations
                );
            }
            Err(r) => {
                assert!(build_blocked_by_network(&fx), "{r:?}");
                eprintln!("skipped: dependencies cannot be resolved here (no network)");
                return;
            }
        }
    }
}

#[test]
fn kotlin_gradle_project_series_grows_with_the_repeated_tests() {
    if !tool_ready("gradle") || !kotlin_compiler_runs() {
        return;
    }
    let fx = project(&[
        ("build.gradle.kts", GRADLE_KTS_BUILD),
        ("settings.gradle.kts", "rootProject.name = \"leak\"\n"),
        ("src/main/kotlin/leak/Cache.kt", CACHE_KT),
        ("src/test/kotlin/leak/CacheTest.kt", CACHE_TEST_KT),
    ]);
    gen_ok(&fx, &junit_opts());
    let Some(series) = run_harness(&fx) else {
        return;
    };
    check_series(&fx, &series);
}

#[test]
fn a_kotlin_spring_boot_test_run_exercises_the_entry_points_of_the_classes_it_loads() {
    if !tool_ready("gradle") || !kotlin_compiler_runs() {
        return;
    }
    let fx = project(&[
        ("build.gradle.kts", BOOT_KTS_BUILD),
        ("settings.gradle.kts", "rootProject.name = \"shop\"\n"),
        ("src/main/kotlin/com/acme/shop/Application.kt", BOOT_APP_KT),
        ("src/main/kotlin/com/acme/shop/Routes.kt", BOOT_ROUTES_KT),
        (
            "src/main/kotlin/com/acme/shop/UserService.kt",
            BOOT_SERVICE_KT,
        ),
        ("src/main/kotlin/com/acme/shop/Util.kt", BOOT_UTIL_KT),
        ("src/main/kotlin/billing/Invoices.kt", BOOT_INVOICES_KT),
        (
            "src/test/kotlin/com/acme/shop/ApplicationTests.kt",
            BOOT_TEST_KT,
        ),
    ]);
    gen_ok(&fx, &junit_opts());
    let Some(_series) = run_harness(&fx) else {
        return;
    };
    let cov = dla_core::CoverageManifest::load(&fx.ctx.run_dir)
        .unwrap()
        .unwrap();
    let w = &cov.workers["jvm"];
    let p = "src/main/kotlin/";
    let want = [
        // A class in a file of another name.
        format!("{p}com/acme/shop/Routes.kt::UserController.ping"),
        format!("{p}com/acme/shop/Routes.kt::UserController.user"),
        // A package directory that omits the root of the package.
        format!("{p}billing/Invoices.kt::Invoices.total"),
        // A top-level function, through its facade class.
        format!("{p}com/acme/shop/Util.kt::slug"),
        format!("{p}com/acme/shop/UserService.kt::UserService.lookup"),
    ];
    for id in &want {
        assert!(
            w.entry_points.exercised.contains(id),
            "{id} is not exercised: {:?} of {:?}",
            w.entry_points.exercised,
            w.entry_points.discovered
        );
    }
    // The facade of Routes.kt (`RoutesKt`) was never loaded: its function did not run.
    assert!(
        !w.entry_points
            .exercised
            .contains(&format!("{p}com/acme/shop/Routes.kt::routeCount")),
        "{:?}",
        w.entry_points.exercised
    );
    assert!(
        w.project_classes.is_some_and(|n| n >= 4),
        "{:?}",
        w.project_classes
    );
    assert!(w.iterations.is_some_and(|n| n > 0), "{:?}", w.iterations);
}

#[test]
fn maven_project_series_grows_with_the_repeated_tests() {
    if !tool_ready("mvn") {
        return;
    }
    let fx = project(&[
        ("pom.xml", POM),
        ("src/main/java/leak/Cache.java", CACHE_JAVA),
        ("src/test/java/leak/CacheTest.java", CACHE_TEST_JAVA),
    ]);
    gen_ok(&fx, &junit_opts());
    let Some(series) = run_harness(&fx) else {
        return;
    };
    check_series(&fx, &series);
}

#[test]
fn a_failing_test_is_counted_and_named_in_the_log_but_the_series_is_kept() {
    if !tool_ready("gradle") {
        return;
    }
    let failing = "package leak;\n\nimport org.junit.jupiter.api.Test;\nimport static org.junit.jupiter.api.Assertions.fail;\n\nclass CacheTest {\n    @Test\n    void touches() {\n        Cache.touch();\n        if (Cache.size() == 4) fail(\"cache grew past three\");\n    }\n}\n";
    let cache = CACHE_JAVA.replace(
        "private Cache() {}",
        "private Cache() {}\n\n    public static int size() {\n        return ITEMS.size();\n    }",
    );
    let mut files = gradle_java();
    files.retain(|(rel, _)| !rel.contains("Cache"));
    files.push(("src/main/java/leak/Cache.java", &cache));
    files.push(("src/test/java/leak/CacheTest.java", failing));
    let fx = project(&files);
    gen_ok(&fx, &junit_opts());
    let Some(series) = run_harness(&fx) else {
        return;
    };
    assert_eq!(series.points.len(), 9);
    let log = run_log(&fx);
    assert!(log.contains("DLA-JUNIT first failing unit"), "{log}");
    assert!(log.contains("cache grew past three"), "{log}");
    assert!(log.contains("DLA-JUNIT summary: units=18"), "{log}");
}

#[test]
fn a_class_that_matches_no_test_ends_the_run_before_any_checkpoint() {
    if !tool_ready("gradle") {
        return;
    }
    let fx = project(&gradle_java());
    let mut o = junit_opts();
    o.tests = vec!["leak.NoSuchTest".into()];
    gen_ok(&fx, &o);
    let m = manifest(&fx, "H1");
    match run_points(&fx.ctx, &m, &RunOptions::default()) {
        Ok(out) => {
            let log = run_log(&fx);
            assert!(
                log.contains("DLA-JUNIT cannot run leak.NoSuchTest")
                    || log.contains("DLA-JUNIT no test found for leak.NoSuchTest"),
                "{log}"
            );
            assert!(
                out.limitations
                    .iter()
                    .any(|l| l.contains("no checkpoint") || l.contains("exited with code 3")),
                "{:?}",
                out.limitations
            );
        }
        Err(r) => {
            if build_blocked_by_network(&fx) {
                eprintln!("skipped: dependencies cannot be resolved here (no network)");
                return;
            }
            // No usable point is also an honest outcome: the log names the cause.
            let log = run_log(&fx);
            assert!(
                log.contains("DLA-JUNIT cannot run") || log.contains("DLA-JUNIT no test found"),
                "{r:?}\n{log}"
            );
        }
    }
}

#[test]
fn when_every_unit_fails_the_last_stderr_lines_name_the_first_failure() {
    if !tool_ready("gradle") {
        return;
    }
    let boom = "package leak;\n\nimport org.junit.jupiter.api.Test;\n\nclass CacheTest {\n    @Test\n    void touches() {\n        throw new IllegalStateException(\"Connection refused: kafka:9092\");\n    }\n}\n";
    let mut files = gradle_java();
    files.retain(|(rel, _)| !rel.ends_with("CacheTest.java"));
    files.push(("src/test/java/leak/CacheTest.java", boom));
    let fx = project(&files);
    gen_ok(&fx, &junit_opts());
    let m = manifest(&fx, "H1");
    let result = run_points(&fx.ctx, &m, &RunOptions::default());
    if build_blocked_by_network(&fx) {
        eprintln!("skipped: dependencies cannot be resolved here (no network)");
        return;
    }
    // The only measured process exited with code 2: the run is refused and no series is written.
    let r = result.expect_err("every unit failed: no series");
    assert_eq!(r.code, "harness-failed", "{r:?}");
    let log = run_log(&fx);
    let tail: Vec<&str> = log.lines().rev().take(4).collect();
    assert!(
        tail.iter()
            .any(|l| l.contains("DLA-JUNIT every unit failed")
                && l.contains("Connection refused: kafka:9092")),
        "{tail:?}"
    );
}

#[test]
fn tests_aborted_by_an_assumption_do_not_make_a_measured_series() {
    if !tool_ready("gradle") {
        return;
    }
    // The service-not-running case: every test assumes an environment variable that is not set,
    // so the code under test never runs. The run must not give a clean series.
    let assumed = "package leak;\n\nimport org.junit.jupiter.api.Test;\nimport static org.junit.jupiter.api.Assumptions.assumeTrue;\n\nclass CacheTest {\n    @Test\n    void touches() {\n        assumeTrue(System.getenv(\"DLA_TEST_KAFKA_URL\") != null);\n        Cache.touch();\n    }\n}\n";
    let mut files = gradle_java();
    files.retain(|(rel, _)| !rel.ends_with("CacheTest.java"));
    files.push(("src/test/java/leak/CacheTest.java", assumed));
    let fx = project(&files);
    gen_ok(&fx, &junit_opts());
    let m = manifest(&fx, "H1");
    let result = run_points(&fx.ctx, &m, &RunOptions::default());
    if build_blocked_by_network(&fx) {
        eprintln!("skipped: dependencies cannot be resolved here (no network)");
        return;
    }
    // The only measured process exited with code 3: no series.
    let r = result.expect_err("every test was aborted: no series");
    assert_eq!(r.code, "harness-failed", "{r:?}");
    let log = run_log(&fx);
    assert!(
        log.contains("DLA-JUNIT summary: units=18 tests_run=0 aborted=18 skipped=0 failed_units=0"),
        "{log}"
    );
    assert!(log.contains("by an assumption"), "{log}");
}

#[test]
fn a_class_that_cannot_start_names_its_cause_and_the_summary_ends_the_log() {
    if !tool_ready("gradle") {
        return;
    }
    let head = "package leak;\n\nimport org.junit.jupiter.api.*;\nimport static org.junit.jupiter.api.Assumptions.assumeTrue;\n\n";
    let cases: [(&str, String, &str, &str); 4] = [
        // An assumption in @BeforeAll aborts the container: counted as aborted.
        (
            "assume",
            format!(
                "{head}class CacheTest {{\n    @BeforeAll\n    static void up() {{ assumeTrue(System.getenv(\"DLA_TEST_KAFKA_URL\") != null); }}\n    @Test\n    void touches() {{ Cache.touch(); }}\n}}\n"
            ),
            "tests_run=0 aborted=18 skipped=0",
            "(aborted=18 by an assumption",
        ),
        // A disabled test is counted as skipped, and the line says so.
        (
            "disabled",
            format!(
                "{head}class CacheTest {{\n    @Test\n    @Disabled\n    void touches() {{ Cache.touch(); }}\n}}\n"
            ),
            "tests_run=0 aborted=0 skipped=18",
            "skipped=18 disabled",
        ),
        // A disabled class: its one test is skipped once per unit, not counted again for the
        // class.
        (
            "disabled-class",
            format!(
                "{head}@Disabled\nclass CacheTest {{\n    @Test\n    void touches() {{ Cache.touch(); }}\n}}\n"
            ),
            "tests_run=0 aborted=0 skipped=18",
            "skipped=18 disabled",
        ),
        // A container that throws (Testcontainers without Docker): the cause line names it.
        (
            "throws",
            format!(
                "{head}class CacheTest {{\n    @BeforeAll\n    static void up() throws Exception {{ throw new java.net.ConnectException(\"kafka:9092 refused\"); }}\n    @Test\n    void touches() {{ Cache.touch(); }}\n}}\n"
            ),
            "tests_run=0 aborted=0 skipped=0 failed_units=18",
            "failed before any test ran: ",
        ),
    ];
    for (name, source, summary, cause) in cases {
        let mut files = gradle_java();
        files.retain(|(rel, _)| !rel.ends_with("CacheTest.java"));
        files.push(("src/test/java/leak/CacheTest.java", &source));
        let fx = project(&files);
        gen_ok(&fx, &junit_opts());
        let m = manifest(&fx, "H1");
        let result = run_points(&fx.ctx, &m, &RunOptions::default());
        if build_blocked_by_network(&fx) {
            eprintln!("skipped: dependencies cannot be resolved here (no network)");
            return;
        }
        let r = result.expect_err("no test passed or failed: no series");
        assert_eq!(r.code, "harness-failed", "{name} {r:?}");
        let log = run_log(&fx);
        let last = log
            .lines()
            .rev()
            .find(|l| l.starts_with("DLA-JUNIT"))
            .unwrap_or("");
        assert!(last.starts_with("DLA-JUNIT summary:"), "{name}: {log}");
        assert!(last.contains(summary), "{name}: {last}");
        assert!(log.contains(cause), "{name}: {log}");
        if name == "throws" {
            assert!(log.contains("java.net.ConnectException"), "{log}");
        }
    }
}

#[test]
fn a_test_that_breaks_on_leaked_state_still_gives_a_series() {
    if !tool_ready("gradle") {
        return;
    }
    // A test written for a fresh JVM: it touches the cache, then asserts the state is fresh. Every
    // unit after the first fails, after the leaky call was made, so the series is the evidence.
    let fresh = "package leak;\n\nimport org.junit.jupiter.api.Test;\nimport static org.junit.jupiter.api.Assertions.fail;\n\nclass CacheTest {\n    static int runs;\n\n    @Test\n    void touches() {\n        Cache.touch();\n        if (++runs > 1) fail(\"expected: <1> but was: <\" + runs + \">\");\n    }\n}\n";
    let mut files = gradle_java();
    files.retain(|(rel, _)| !rel.ends_with("CacheTest.java"));
    files.push(("src/test/java/leak/CacheTest.java", fresh));
    let fx = project(&files);
    gen_ok(&fx, &junit_opts());
    let Some(series) = run_harness(&fx) else {
        return;
    };
    assert_eq!(series.points.len(), 9);
    assert!(
        run_log(&fx).contains(
            "DLA-JUNIT summary: units=18 tests_run=18 aborted=0 skipped=0 failed_units=17"
        ),
        "{}",
        run_log(&fx)
    );
}

#[test]
fn a_few_failing_units_keep_the_series_and_are_counted() {
    if !tool_ready("gradle") {
        return;
    }
    // Fails on the fourth and fifth execution only: 2 of 18 units.
    let flaky = "package leak;\n\nimport org.junit.jupiter.api.Test;\nimport static org.junit.jupiter.api.Assertions.fail;\n\nclass CacheTest {\n    static int runs;\n\n    @Test\n    void touches() {\n        Cache.touch();\n        int n = ++runs;\n        if (n == 4 || n == 5) fail(\"blip\");\n    }\n}\n";
    let mut files = gradle_java();
    files.retain(|(rel, _)| !rel.ends_with("CacheTest.java"));
    files.push(("src/test/java/leak/CacheTest.java", flaky));
    let fx = project(&files);
    gen_ok(&fx, &junit_opts());
    let Some(series) = run_harness(&fx) else {
        return;
    };
    assert_eq!(series.points.len(), 9);
    assert!(
        run_log(&fx).contains(
            "DLA-JUNIT summary: units=18 tests_run=18 aborted=0 skipped=0 failed_units=2"
        ),
        "{}",
        run_log(&fx)
    );
}

#[test]
fn a_gradle_project_with_the_configuration_cache_on_still_builds() {
    if !tool_ready("gradle") {
        return;
    }
    let mut files = gradle_java();
    files.push(("gradle.properties", "org.gradle.configuration-cache=true\n"));
    let fx = project(&files);
    gen_ok(&fx, &junit_opts());
    let Some(series) = run_harness(&fx) else {
        return;
    };
    check_series(&fx, &series);
}

#[test]
fn a_parameterized_test_is_selected_by_its_signature() {
    if !tool_ready("gradle") {
        return;
    }
    let param = "package leak;\n\nimport org.junit.jupiter.params.ParameterizedTest;\nimport org.junit.jupiter.params.provider.ValueSource;\n\nclass CacheTest {\n    @ParameterizedTest\n    @ValueSource(ints = {1, 2})\n    void p(int x) {\n        Cache.touch();\n    }\n}\n";
    let mut files = gradle_java();
    files.retain(|(rel, _)| !rel.ends_with("CacheTest.java"));
    files.push(("src/test/java/leak/CacheTest.java", param));
    let fx = project(&files);
    let mut o = junit_opts();
    o.tests = vec!["leak.CacheTest#p(int)".into()];
    gen_ok(&fx, &o);
    let Some(series) = run_harness(&fx) else {
        return;
    };
    assert_eq!(series.points.len(), 9);
    // Two invocations of `p` per unit.
    assert!(
        run_log(&fx).contains(
            "DLA-JUNIT summary: units=18 tests_run=36 aborted=0 skipped=0 failed_units=0"
        ),
        "{}",
        run_log(&fx)
    );
}

#[test]
fn a_selector_with_parameter_types_passes_gen_and_a_broken_one_does_not() {
    if !have_jdk() {
        return;
    }
    let fx = project(&gradle_java());
    let mut o = junit_opts();
    o.tests = vec!["leak.CacheTest#p(java.lang.String, int[])".into()];
    gen_ok(&fx, &o);
    assert!(
        driver_text(&fx, "H1").contains("\"leak.CacheTest#p(java.lang.String, int[])\""),
        "{}",
        driver_text(&fx, "H1")
    );
    for bad in [
        "leak.CacheTest#p(",
        "leak.CacheTest#p(\")",
        "leak.CacheTest#(int)",
    ] {
        let fx = project(&gradle_java());
        let mut o = junit_opts();
        o.tests = vec![bad.into()];
        assert_eq!(gen_err(&fx, &o).code, "invalid-value", "{bad}");
    }
}

#[test]
fn junit_for_another_language_is_refused_as_such() {
    if !have_jdk() {
        return;
    }
    let fx = project(&[
        ("go.mod", "module x\n\ngo 1.22\n"),
        ("main.go", "package main\n\nfunc main() {}\n"),
    ]);
    let mut o = junit_opts();
    o.lang = Runtime::Go;
    let r = gen_err(&fx, &o);
    assert_eq!(r.code, "invalid-value", "{r:?}");
    assert!(r.message.contains("--junit is for JVM"), "{}", r.message);
    // Also before any discovery result exists: the language is checked first.
    fs::remove_dir_all(fx.ctx.run_dir.join("workload")).unwrap();
    let r = gen_err(&fx, &o);
    assert_eq!(r.code, "invalid-value", "{r:?}");
    // `--tests` with another language suggests a command that is not refused.
    let mut o = junit_opts();
    o.lang = Runtime::Go;
    o.junit = false;
    o.driver = true;
    o.tests = vec!["a.B".into()];
    let r = gen_err(&fx, &o);
    assert!(r.fix.contains("--lang jvm --junit --tests a.B"), "{r:?}");
    // Without `--driver` too (the early check in generate).
    o.driver = false;
    let r = gen_err(&fx, &o);
    assert!(r.fix.contains("--lang jvm --junit --tests a.B"), "{r:?}");
    assert!(!r.fix.contains("--lang go"), "{r:?}");
}

#[test]
fn kotlin_test_and_renamed_kotlin_classes_are_accepted() {
    if !have_jdk() {
        return;
    }
    let kt = "package leak\n\nimport kotlin.test.Test\n\nclass GetTest {\n    @Test\n    fun touches() { Cache.touch() }\n}\n";
    let fx = project(&[
        ("build.gradle.kts", GRADLE_KTS_BUILD),
        ("settings.gradle.kts", "rootProject.name = \"leak\"\n"),
        ("gradlew", "#!/bin/sh\nexec gradle \"$@\"\n"),
        ("src/main/kotlin/leak/Cache.kt", CACHE_KT),
        ("src/test/kotlin/leak/ApiTests.kt", kt),
    ]);
    gen_ok(&fx, &junit_opts());
    let text = driver_text(&fx, "H1");
    assert!(text.contains("SELECT = {\"leak.GetTest\"}"), "{text}");
}

#[test]
fn a_kotlin_class_with_a_top_level_main_runs_the_kt_facade() {
    if !have_jdk() {
        return;
    }
    let server = "package app\n\nclass Server {\n    fun run() {}\n}\n\nfun main(args: Array<String>) {\n    Server().run()\n}\n";
    let fx = project(&[
        ("build.gradle.kts", GRADLE_KTS_BUILD),
        ("settings.gradle.kts", "rootProject.name = \"app\"\n"),
        ("src/main/kotlin/app/Server.kt", server),
    ]);
    let dir = fx.ctx.stage_dir("workload").unwrap();
    let mut commands = Vec::new();
    for e in fs::read_dir(dir).unwrap().flatten() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(e.path()).unwrap_or_default(),
        ) else {
            continue;
        };
        for w in v["workloads"].as_array().into_iter().flatten() {
            if w["id"] == "jvm:main:src/main/kotlin/app/Server.kt" {
                commands.push(w["command"].as_str().unwrap().to_owned());
            }
        }
    }
    assert_eq!(commands.len(), 1, "{commands:?}");
    assert!(commands[0].ends_with(" app.ServerKt"), "{}", commands[0]);
}

/// The Maven branch of the build script with a stand-in `mvn` that writes a class path in the
/// form Windows Maven does (backslashes): the paths stay relative to the project directory and
/// the launcher is found next to the engine jar whatever the separators.
#[test]
fn the_maven_branch_keeps_paths_relative_and_reads_backslash_class_paths() {
    if cfg!(windows) {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let repo = root.join("repo/org/junit/platform");
    for (name, ver) in [
        ("junit-platform-engine", "1.10.2"),
        ("junit-platform-launcher", "1.10.2"),
    ] {
        let d = repo.join(name).join(ver);
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join(format!("{name}-{ver}.jar")), b"").unwrap();
    }
    let engine = repo.join("junit-platform-engine/1.10.2/junit-platform-engine-1.10.2.jar");
    let backslashed = engine.to_str().unwrap().replace('/', "\\");
    let proj = root.join("proj");
    let driver = root.join("driver");
    fs::create_dir_all(&proj).unwrap();
    fs::create_dir_all(&driver).unwrap();
    fs::write(driver.join("DlaJUnitLoop.java"), "class DlaJUnitLoop {}\n").unwrap();
    // The stand-in writes the file named by -Dmdep.outputFile=.
    let mvn = root.join("mvn");
    fs::write(
        &mvn,
        format!(
            "#!/bin/sh\nfor a in \"$@\"; do case \"$a\" in -Dmdep.outputFile=*) printf '%s' '{backslashed}' > \"${{a#-Dmdep.outputFile=}}\";; esac; done\n"
        ),
    )
    .unwrap();
    let script = root.join("dla-junit-build.sh");
    fs::write(&script, dla_workload::harness::templates::JVM_JUNIT_BUILD).unwrap();
    // javac may fail on the empty jar entries; the class path is written before it runs.
    let _ = Command::new("sh")
        .arg(&script)
        .args(["maven"])
        .arg(&mvn)
        .arg(&driver)
        .arg(root.join("agent.jar"))
        .arg(":")
        .current_dir(&proj)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let cp = fs::read_to_string(driver.join("cp.txt")).unwrap();
    let entries: Vec<&str> = cp.split(':').collect();
    assert_eq!(entries[0], "target/test-classes", "{cp}");
    assert_eq!(entries[1], "target/classes", "{cp}");
    assert!(!cp.contains('\\'), "{cp}");
    assert!(
        entries
            .last()
            .is_some_and(|l| l
                .ends_with("junit-platform-launcher/1.10.2/junit-platform-launcher-1.10.2.jar")),
        "{cp}"
    );
}

#[test]
fn the_online_retry_of_the_build_step_says_what_it_downloads_and_the_tier_table_agrees() {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let proj = root.join("proj");
    let driver = root.join("driver");
    fs::create_dir_all(&proj).unwrap();
    fs::create_dir_all(&driver).unwrap();
    // A build tool that fails every time and notes whether it was told to work offline.
    let mvn = root.join("mvn");
    let calls = root.join("calls.txt");
    fs::write(
        &mvn,
        format!(
            "#!/bin/sh\ncase \" $* \" in *\" -o \"*) echo offline >> '{0}';; *) echo online >> '{0}';; esac\nexit 1\n",
            calls.display()
        ),
    )
    .unwrap();
    let script = root.join("dla-junit-build.sh");
    fs::write(&script, dla_workload::harness::templates::JVM_JUNIT_BUILD).unwrap();
    let out = Command::new("sh")
        .arg(&script)
        .args(["maven"])
        .arg(&mvn)
        .arg(&driver)
        .arg(root.join("agent.jar"))
        .arg(":")
        .arg("main")
        .current_dir(&proj)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains(
            "DLA-BUILD the offline attempt failed; retrying online (the build tool downloads dependencies into its own cache)"
        ),
        "{stderr}"
    );
    assert_eq!(fs::read_to_string(&calls).unwrap(), "offline\nonline\n");
    // The tier table of the safety reference says the same, with the same marker.
    let table = fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../skills/detect-leak-anywhere/references/safety-and-effects.md"),
    )
    .unwrap();
    assert!(
        table.contains("DLA-BUILD the offline attempt failed; retrying online")
            && table.contains("downloads dependencies into its own cache"),
        "the tier table row of `dla harness run` lost the online retry"
    );
}

/// Runs the build step with a stand-in Maven whose class path is `entries` (after the project's
/// own two directories) and returns `(run.args, startup-agents.txt, stderr)`.
#[cfg(unix)]
fn build_with_class_path(entries: &[std::path::PathBuf]) -> (String, Option<String>, String) {
    let tmp = TempDir::new().unwrap();
    let root = tmp.path();
    let proj = root.join("proj");
    let driver = root.join("driver");
    fs::create_dir_all(&proj).unwrap();
    fs::create_dir_all(&driver).unwrap();
    fs::write(driver.join("DlaJUnitLoop.java"), "class DlaJUnitLoop {}\n").unwrap();
    // The launcher jar need not exist: its name on the class path is all the script reads.
    let launcher = "/repo/junit-platform-launcher-1.10.2.jar";
    let mut cp: Vec<String> = entries.iter().map(|e| e.display().to_string()).collect();
    cp.push(launcher.to_owned());
    let mvn = root.join("mvn");
    fs::write(
        &mvn,
        format!(
            "#!/bin/sh\nfor a in \"$@\"; do case \"$a\" in -Dmdep.outputFile=*) printf '%s' '{}' > \"${{a#-Dmdep.outputFile=}}\";; esac; done\n",
            cp.join(":")
        ),
    )
    .unwrap();
    let script = root.join("dla-junit-build.sh");
    fs::write(&script, dla_workload::harness::templates::JVM_JUNIT_BUILD).unwrap();
    let out = Command::new("sh")
        .arg(&script)
        .args(["maven"])
        .arg(&mvn)
        .arg(&driver)
        .arg(root.join("agent.jar"))
        .arg(":")
        .current_dir(&proj)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(out.status.success(), "{stderr}");
    (
        fs::read_to_string(driver.join("run.args")).unwrap(),
        fs::read_to_string(driver.join("startup-agents.txt")).ok(),
        stderr,
    )
}

/// A jar with `manifest` lines in `dir`.
#[cfg(unix)]
fn jar_with_manifest(dir: &std::path::Path, name: &str, manifest: &str) -> std::path::PathBuf {
    let m = dir.join(format!("{name}.mf"));
    fs::write(&m, format!("{manifest}\n")).unwrap();
    fs::write(dir.join("keep.txt"), "x").unwrap();
    let jar = dir.join(name);
    let st = Command::new("jar")
        .args(["cfm"])
        .arg(&jar)
        .arg(&m)
        .arg("-C")
        .arg(dir)
        .arg("keep.txt")
        .output()
        .unwrap();
    assert!(
        st.status.success(),
        "{}",
        String::from_utf8_lossy(&st.stderr)
    );
    jar
}

// The stand-in Maven of this test is a `#!/bin/sh` script and its class path uses `:`.
#[cfg(unix)]
#[test]
fn a_mocking_jar_with_a_start_up_agent_entry_is_added_to_the_loops_jvm_and_none_otherwise() {
    if !have_jdk() {
        eprintln!("skipped: no JDK");
        return;
    }
    let tmp = TempDir::new().unwrap();
    let d = tmp.path();
    let mk = |sub: &str| {
        let p = d.join(sub);
        fs::create_dir_all(&p).unwrap();
        p
    };
    // mockito-core with a Premain-Class: it is the agent.
    let mockito = jar_with_manifest(
        &mk("m1"),
        "mockito-core-5.23.0.jar",
        "Premain-Class: org.mockito.internal.PremainAttach",
    );
    let bb = jar_with_manifest(
        &mk("b1"),
        "byte-buddy-agent-1.17.7.jar",
        "Premain-Class: net.bytebuddy.agent.Installer",
    );
    let (args, file, stderr) = build_with_class_path(&[mockito.clone(), bb.clone()]);
    assert!(
        args.contains(&format!("\"-javaagent:{}\"", mockito.display())),
        "{args}"
    );
    assert_eq!(
        args.matches("-javaagent:").count(),
        1,
        "one agent only: {args}"
    );
    assert_eq!(file.unwrap().trim(), mockito.display().to_string());
    assert!(stderr.contains("DLA-BUILD start-up agent:"), "{stderr}");
    // mockito-core without the entry (an older version): the byte-buddy-agent jar is.
    let old = jar_with_manifest(&mk("m2"), "mockito-core-4.11.0.jar", "Created-By: x");
    let (args, file, _) = build_with_class_path(&[old.clone(), bb.clone()]);
    assert!(
        args.contains(&format!("\"-javaagent:{}\"", bb.display())),
        "{args}"
    );
    assert_eq!(file.unwrap().trim(), bb.display().to_string());
    // Neither carries one, or neither is on the class path: nothing is added.
    let (args, file, stderr) = build_with_class_path(&[old]);
    assert!(!args.contains("-javaagent"), "{args}");
    assert!(
        file.is_none() && !stderr.contains("start-up agent"),
        "{stderr}"
    );
    let (args, file, _) = build_with_class_path(&[]);
    assert!(!args.contains("-javaagent") && file.is_none(), "{args}");
}

const OTHER_JAVA: &str = "package leak;\n\nimport java.util.HashMap;\nimport java.util.Map;\n\npublic final class Other {\n    static final Map<String, byte[]> REG = new HashMap<>();\n\n    private Other() {}\n\n    public static void put(String k) {\n        REG.put(k, new byte[16]);\n    }\n}\n";

#[test]
fn next_with_junit_takes_the_queues_test_loop_batch_and_says_which_tests_and_why() {
    if !have_jdk() {
        return;
    }
    let mut files = gradle_java();
    files.push(("src/main/java/leak/Other.java", OTHER_JAVA));
    let fx = project(&files);
    let o = GenOptions {
        next: Some(16),
        ..junit_opts()
    };
    let out = gen_ok(&fx, &o);
    let s = &out.summary;
    let chosen = s["tests_chosen"].as_array().unwrap();
    assert_eq!(chosen.len(), 1, "{s}");
    assert!(
        chosen[0]
            .as_str()
            .unwrap()
            .starts_with("CacheTest (1 tests) for F"),
        "{s}"
    );
    assert!(chosen[0].as_str().unwrap().ends_with("by name"), "{s}");
    assert_eq!(s["test_loop"]["targets"], 1, "{s}");
    assert!(
        s["test_loop"]["queue"].as_str().unwrap().contains(
            "1 candidate in 1 test-loop batch, nothing to write; then 1 candidate needs a driver"
        ),
        "{s}"
    );
    // The output is bounded: the whole summary stays small.
    assert!(s.to_string().len() < 1900, "{}", s.to_string().len());
    let m = manifest(&fx, "H1");
    assert_eq!(m.targets.len(), 1);
    assert!(m.is_test_loop());
    assert_eq!(m.timeout_secs, 600);
    assert!(driver_text(&fx, "H1").contains("\"leak.CacheTest\""));
    assert!(fx.ctx.run_dir.join("harness/H1/tests.json").is_file());
    // A second batch has nothing left that has a test: refused, with the driver command.
    let r = gen_err(&fx, &o);
    assert!(r.fix.contains("--driver --next"), "{r:?}");
}

#[test]
fn next_with_junit_and_a_probe_is_refused_and_explicit_tests_keep_the_old_meaning() {
    if !have_jdk() {
        return;
    }
    let fx = project(&gradle_java());
    let mut o = GenOptions {
        next: Some(4),
        probe_findings: vec![dla_core::FindingId::new(1)],
        ..junit_opts()
    };
    assert_eq!(gen_err(&fx, &o).code, "invalid-value");
    o.probe_findings.clear();
    o.tests = vec!["leak.CacheTest".into()];
    gen_ok(&fx, &o);
    assert_eq!(manifest(&fx, "H1").targets.len(), 1);
    // A bare `--next` of a hookless harness asks for 16, its own bound, and is taken; one more
    // is refused, and the fix names the same kind of harness.
    let fx = project(&gradle_java());
    o.next = Some(16);
    gen_ok(&fx, &o);
    o.next = Some(17);
    let r = gen_err(&fx, &o);
    assert_eq!(r.code, "too-many-targets", "{r:?}");
    assert!(
        r.fix.contains("--junit --tests") && r.fix.contains("--next 16"),
        "{r:?}"
    );
}

#[test]
fn more_than_eight_targets_are_declared_by_a_hookless_harness_and_never_by_a_driver() {
    if !have_jdk() {
        return;
    }
    let fx = project(&gradle_java());
    let mut o = junit_opts();
    o.tests = vec!["leak.CacheTest".into()];
    o.probe_findings = (1..=9).map(dla_core::FindingId::new).collect();
    let r = gen_err(
        &fx,
        &GenOptions {
            driver: true,
            junit: false,
            tests: Vec::new(),
            ..o.clone()
        },
    );
    assert_eq!(r.code, "too-many-targets", "{r:?}");
    // The JUnit repeat takes them (findings that do not exist are refused for another reason).
    let r = generate(&fx.ctx, &o);
    if let Err(r) = r {
        assert_ne!(r.code, "too-many-targets", "{r:?}");
    }
}

#[test]
fn a_class_that_cannot_run_is_dropped_and_the_others_are_measured() {
    if !tool_ready("gradle") {
        return;
    }
    let broken = "package leak;\n\nimport org.junit.jupiter.api.Test;\n\nclass BrokenTest {\n    @Test\n    void a() {\n        throw new IllegalStateException(\"Connection refused: kafka:9092\");\n    }\n}\n";
    let mut files = gradle_java();
    files.push(("src/test/java/leak/BrokenTest.java", broken));
    let fx = project(&files);
    let mut o = junit_opts();
    o.tests = vec!["leak.CacheTest".into(), "leak.BrokenTest".into()];
    gen_ok(&fx, &o);
    let Some(series) = run_harness(&fx) else {
        return;
    };
    assert_eq!(series.points.len(), 9);
    let log = run_log(&fx);
    assert!(
        log.contains("DLA-JUNIT dropped test class leak.BrokenTest"),
        "{log}"
    );
    assert!(log.contains("failed_units=1"), "{log}");
    assert!(
        series
            .limitations
            .iter()
            .any(|l| l.contains("dropped test class leak.BrokenTest")),
        "{:?}",
        series.limitations
    );
}

const HOLDER_JAVA: &str = "package leak;\n\nimport java.util.HashMap;\nimport java.util.Map;\n\npublic final class Holder {\n    static final Map<Object, Object> ITEMS = new HashMap<>();\n\n    private Holder() {}\n\n    public static void put(Object key) {\n        ITEMS.put(key, \"v\");\n    }\n}\n";
const FEEDS_TEST_JAVA: &str = "package leak;\n\nimport org.junit.jupiter.api.Test;\n\nclass FeedsTest {\n    @Test\n    void newKeyEachRun() {\n        Holder.put(new Object());\n    }\n}\n";
const IDLE_TEST_JAVA: &str = "package leak;\n\nimport org.junit.jupiter.api.Test;\n\nclass IdleTest {\n    @Test\n    void doesNothing() {}\n}\n";
const FLAT_TEST_JAVA: &str = "package leak;\n\nimport org.junit.jupiter.api.Test;\n\nclass FlatTest {\n    @Test\n    void sameKeyEachRun() {\n        Holder.put(\"constant\");\n    }\n}\n";

fn attribution_project() -> Fx {
    let mut files = gradle_java();
    files.push(("src/main/java/leak/Holder.java", HOLDER_JAVA));
    files.push(("src/test/java/leak/FeedsTest.java", FEEDS_TEST_JAVA));
    files.push(("src/test/java/leak/FlatTest.java", FLAT_TEST_JAVA));
    files.push(("src/test/java/leak/IdleTest.java", IDLE_TEST_JAVA));
    let fx = project(&files);
    let mut o = junit_opts();
    // The idle class runs first, before the holder's class is loaded.
    o.tests = vec![
        "leak.IdleTest".into(),
        "leak.FlatTest".into(),
        "leak.FeedsTest".into(),
    ];
    gen_ok(&fx, &o);
    fx
}

#[test]
fn the_loop_measures_which_test_class_feeds_a_holder_and_writes_nothing_without_the_ask() {
    if !tool_ready("gradle") {
        return;
    }
    let fx = attribution_project();
    let m = manifest(&fx, "H1");
    let file = fx.ctx.run_dir.join("harness/H1/attribution-F1.json");
    // Without the option no file is written.
    if run_harness(&fx).is_none() {
        return;
    }
    assert!(!file.exists(), "no attribution was asked for");
    // With it the loop names the class that adds an entry per run and the one that does not.
    let opts = RunOptions {
        attribute: Some(dla_workload::harness::run::Attribution {
            finding: "F1".into(),
            class: "leak.Holder".into(),
            field: "ITEMS".into(),
        }),
        ..RunOptions::default()
    };
    run_points(&fx.ctx, &m, &opts).unwrap();
    let a = dla_core::testloop::Attributed::parse(&fs::read_to_string(&file).unwrap())
        .expect("a parsable attribution file");
    assert_eq!(a.probe, "leak.Holder#ITEMS");
    assert_eq!(a.later_runs, 17, "{a:?}");
    assert!(!a.parallel);
    let fig = |c: &str| a.classes.iter().find(|f| f.class == c).unwrap().clone();
    let feeds = fig("leak.FeedsTest");
    let flat = fig("leak.FlatTest");
    let idle = fig("leak.IdleTest");
    // One new key per run: one entry per later run; the constant key adds one entry once, in the
    // first run, and nothing after it.
    assert_eq!(feeds.later_runs, 17, "{a:?}");
    assert!(feeds.first_run >= 1, "{a:?}");
    // The class that feeds the holder added in every later run; the others in none.
    assert_eq!(feeds.grew_in, Some(17), "{a:?}");
    assert_eq!(flat.later_runs, 0, "{a:?}");
    assert_eq!(flat.grew_in, Some(0), "{a:?}");
    // A class that runs before the holder's class is loaded added nothing: a negative reading
    // counts as 0, never as a loss.
    assert_eq!((idle.first_run, idle.later_runs), (0, 0), "{a:?}");
    let fed = a.fed_by();
    assert_eq!(fed.len(), 1, "{fed:?}");
    assert_eq!(fed[0].class, "leak.FeedsTest");
    assert!((fed[0].per_run - 1.0).abs() < 1e-9, "{fed:?}");
    let ran_flat: Vec<&str> = a.ran_flat().iter().map(|c| c.class.as_str()).collect();
    assert_eq!(ran_flat, ["leak.IdleTest", "leak.FlatTest"]);
}

#[test]
fn tests_that_run_in_parallel_say_that_the_figures_of_the_classes_overlap() {
    if !tool_ready("gradle") {
        return;
    }
    let props = "junit.jupiter.execution.parallel.enabled=true\njunit.jupiter.execution.parallel.mode.classes.default=concurrent\n";
    let mut files = gradle_java();
    files.push(("src/main/java/leak/Holder.java", HOLDER_JAVA));
    files.push(("src/test/java/leak/FeedsTest.java", FEEDS_TEST_JAVA));
    files.push(("src/test/java/leak/FlatTest.java", FLAT_TEST_JAVA));
    files.push(("src/test/resources/junit-platform.properties", props));
    let fx = project(&files);
    let mut o = junit_opts();
    o.tests = vec!["leak.FlatTest".into(), "leak.FeedsTest".into()];
    gen_ok(&fx, &o);
    let m = manifest(&fx, "H1");
    let opts = RunOptions {
        attribute: Some(dla_workload::harness::run::Attribution {
            finding: "F1".into(),
            class: "leak.Holder".into(),
            field: "ITEMS".into(),
        }),
        ..RunOptions::default()
    };
    match run_points(&fx.ctx, &m, &opts) {
        Ok(_) => {}
        Err(_) if build_blocked_by_network(&fx) => return,
        Err(r) => panic!("{r:?}"),
    }
    let file = fx.ctx.run_dir.join("harness/H1/attribution-F1.json");
    let a = dla_core::testloop::Attributed::parse(&fs::read_to_string(&file).unwrap()).unwrap();
    assert!(a.parallel, "{a:?}");
    assert_eq!(a.classes.len(), 2, "{a:?}");
}

#[test]
fn a_test_that_fails_in_every_run_does_not_sink_the_tests_that_pass() {
    if !tool_ready("gradle") {
        return;
    }
    // One class, two tests: one passes and feeds the cache, one fails every time (a service it
    // needs is not there). Every run has a failure; the runs are still measurements of the test
    // that passed.
    let mixed = "package leak;\n\nimport org.junit.jupiter.api.Test;\n\nclass MixedTest {\n    @Test\n    void passes() {\n        Cache.touch();\n    }\n\n    @Test\n    void needsAService() {\n        throw new IllegalStateException(\"Connection refused: kafka:9092\");\n    }\n}\n";
    let mut files = gradle_java();
    files.push(("src/test/java/leak/MixedTest.java", mixed));
    let fx = project(&files);
    let mut o = junit_opts();
    o.tests = vec!["leak.CacheTest".into(), "leak.MixedTest".into()];
    gen_ok(&fx, &o);
    let Some(series) = run_harness(&fx) else {
        return;
    };
    assert_eq!(series.points.len(), 9);
    let log = run_log(&fx);
    assert!(log.contains("failed_units=18"), "{log}");
    assert!(!log.contains("DLA-JUNIT every unit failed"), "{log}");
    assert!(!log.contains("dropped test class"), "{log}");
    assert!(
        series
            .limitations
            .iter()
            .any(|l| l.contains("run 1 of the tests had 1 failure(s)")
                && l.contains("Connection refused: kafka:9092")),
        "{:?}",
        series.limitations
    );
}

#[test]
fn a_test_loop_that_cannot_build_or_measure_prints_the_driver_command_for_its_targets() {
    if !tool_ready("gradle") {
        return;
    }
    let boom = "package leak;\n\nimport org.junit.jupiter.api.Test;\n\nclass CacheTest {\n    @Test\n    void touches() {\n        throw new IllegalStateException(\"Connection refused: kafka:9092\");\n    }\n}\n";
    let mut files = gradle_java();
    files.retain(|(rel, _)| !rel.ends_with("CacheTest.java"));
    files.push(("src/test/java/leak/CacheTest.java", boom));
    let fx = project(&files);
    let mut o = junit_opts();
    o.probe_findings = vec![dla_core::FindingId::new(1)];
    gen_ok(&fx, &o);
    let m = manifest(&fx, "H1");
    let result = run_points(&fx.ctx, &m, &RunOptions::default());
    if build_blocked_by_network(&fx) {
        return;
    }
    let r = result.expect_err("every unit failed");
    assert!(
        r.fix
            .contains("dla harness gen --lang jvm --driver --probe F1"),
        "{r:?}"
    );
    assert!(r.fix.contains("not counted as measured"), "{r:?}");
}

#[test]
fn the_loop_fits_its_runs_into_the_time_limit_by_lowering_the_step_and_dropping_the_slowest_class()
{
    if !tool_ready("gradle") {
        return;
    }
    let slow = "package leak;\n\nimport org.junit.jupiter.api.Test;\n\nclass SlowTest {\n    @Test\n    void sleeps() throws Exception {\n        Thread.sleep(2500);\n        Cache.touch();\n    }\n}\n";
    let mut files = gradle_java();
    files.push(("src/test/java/leak/SlowTest.java", slow));
    let fx = project(&files);
    let mut o = junit_opts();
    o.tests = vec!["leak.CacheTest".into(), "leak.SlowTest".into()];
    gen_ok(&fx, &o);
    let mut m = manifest(&fx, "H1");
    // The loop keeps 15 percent and 5 s of the limit back and counts the JVM's uptime: 9 runs of
    // 2.5 s (22.5 s) fit a 40 s limit, so nothing would be dropped there. At 24 s the budget is
    // about 15 s less the start of the JVM, the 8 runs left after the step is lowered to one
    // need about 20 s, and the class that sleeps must go.
    m.timeout_secs = 24;
    m.save(&fx.ctx.run_dir).unwrap();
    let Some(series) = run_harness(&fx) else {
        return;
    };
    assert_eq!(series.points.len(), 9);
    let log = run_log(&fx);
    assert!(
        log.contains("DLA-JUNIT dropped test class leak.SlowTest"),
        "{log}"
    );
    assert!(
        series
            .limitations
            .iter()
            .any(|l| l.contains("lowered its step")),
        "{:?}",
        series.limitations
    );
}
