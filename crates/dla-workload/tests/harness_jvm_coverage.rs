//! The JVM agent's coverage section, checked with a real JVM: which classes count as the
//! project's (a directory, a jar, a Spring Boot nested jar location, a package the sources
//! declare), the `SourceFile` attribute read from class files, and a driver's last
//! checkpoint carrying the section (the exit hook stays quiet once a checkpoint was
//! written). A test prints why it is skipped when `java`, `javac` or `jar` is missing.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde_json::Value;

fn jvm_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../harness/jvm")
}

fn have_jdk() -> bool {
    ["java", "javac", "jar"].iter().all(|t| {
        let ok = Command::new(t)
            .arg(if *t == "jar" { "--version" } else { "-version" })
            .env_remove("JAVA_TOOL_OPTIONS")
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

fn run_ok(cmd: &mut Command) -> String {
    let out = cmd
        .env_remove("JAVA_TOOL_OPTIONS")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "{cmd:?}\n{text}");
    text
}

/// The agent compiled into `dir/agent` (classes and a jar with the manifest).
fn build_agent(dir: &Path) -> (PathBuf, PathBuf) {
    let classes = dir.join("agent/classes");
    fs::create_dir_all(&classes).unwrap();
    let src = jvm_dir();
    run_ok(
        Command::new("javac")
            .arg("-d")
            .arg(&classes)
            .args(["DlaAgent.java", "DlaDump.java", "DlaCapture.java"].map(|f| src.join(f))),
    );
    let jar = dir.join("agent/dla-agent.jar");
    run_ok(
        Command::new("jar")
            .args(["--create", "--file"])
            .arg(&jar)
            .arg("--manifest")
            .arg(src.join("MANIFEST.MF"))
            .arg("-C")
            .arg(&classes)
            .arg("."),
    );
    (classes, jar)
}

const CHECK: &str = r#"package dla.harness;

import java.io.DataInputStream;
import java.io.FileInputStream;
import java.nio.file.Path;

public final class CheckCoverage {
    static int bad;

    static void is(String what, Object got, Object want) {
        if (!java.util.Objects.equals(got, want)) {
            bad++;
            System.out.println("FAIL " + what + ": got " + got + ", want " + want);
        }
    }

    static String source(String classFile) throws Exception {
        try (DataInputStream in = new DataInputStream(new FileInputStream(classFile))) {
            return DlaDump.sourceFileOf(in);
        }
    }

    public static void main(String[] a) throws Exception {
        Path project = Path.of("/w");
        String[][] cases = {
            {"file:/w/build/classes/kotlin/main/", "true"},
            {"file:/w/build/libs/app.jar", "true"},
            {"file:/other/classes/", "false"},
            {"jar:file:/w/lib/dep.jar!/", "true"},
            {"jar:file:/w/build/libs/app.jar!/BOOT-INF/classes!/", "true"},
            {"jar:nested:/w/build/libs/app.jar/!BOOT-INF/classes/!/", "true"},
            {"jar:nested:/w/my%20dir/app.jar/!BOOT-INF/classes/!/", "true"},
            {"jar:nested:/w/build/libs/app.jar/!BOOT-INF/lib/spring-core-6.1.jar!/", "false"},
            {"jar:file:/w/build/libs/app.jar!/BOOT-INF/lib/spring-core-6.1.jar!/", "false"},
            {"jar:nested:/other/app.jar/!BOOT-INF/classes/!/", "false"},
            {"jrt:/java.base", "false"},
            {"jar:http://example.org/x.jar!/", "false"},
        };
        for (String[] c : cases) {
            is(c[0], String.valueOf(DlaDump.locationUnder(c[0], project)), c[1]);
        }
        is("a jar root", DlaDump.isJarRoot("file:/w/app.jar") + "/" + DlaDump.isJarRoot("jar:file:/w/app.jar!/"), "true/true");
        is("a directory in a jar", DlaDump.isJarRoot("jar:nested:/w/app.jar/!BOOT-INF/classes/!/"), false);
        is("a directory", DlaDump.isJarRoot("file:/w/build/classes/"), false);
        String[][] tests = {
            {"file:/w/build/classes/kotlin/test/", "true"},
            {"file:/w/build/classes/java/test/", "true"},
            {"file:/w/target/test-classes/", "true"},
            // A Windows path below the project has backslashes.
            {"file:/w/build\\classes\\kotlin\\test/", "true"},
            {"file:/w/build/classes/kotlin/main/", "false"},
            {"file:/w/target/classes/", "false"},
            {"jar:nested:/w/build/libs/app.jar/!BOOT-INF/classes/!/", "false"},
            // The project itself may live below a directory named test.
            {"file:/w/java/test/build/classes/kotlin/main/", "false"},
        };
        for (String[] c : tests) {
            Path root = c[0].contains("/java/test/") ? Path.of("/w/java/test") : project;
            is("test location " + c[0], String.valueOf(DlaDump.isTestLocation(c[0], root)), c[1]);
        }
        // DLA's own driver is compiled next to the project's classes but is not the project's code.
        is("a project class", DlaDump.isProjectClass(zz.app.Thing.class, project, java.util.List.of("zz.app")), true);
        is("the DLA loop driver", DlaDump.isProjectClass(zz.app.DlaLoop.class, project, java.util.List.of("zz.app")), false);
        is("the DLA JUnit driver", DlaDump.isProjectClass(zz.app.DlaJUnitLoop.class, project, java.util.List.of("zz.app")), false);
        // The helper JVM Byte Buddy starts to attach its agent is a tool, not the target.
        System.setProperty("sun.java.command", "net.bytebuddy.agent.Attacher 1234 /tmp/agent.jar");
        is("the Byte Buddy attach helper", DlaDump.isToolJvm(), true);
        System.setProperty("sun.java.command", "zz.shop.Main");
        is("an application", DlaDump.isToolJvm(), false);
        // The file a class names is not the class's own name: `Other` lives in Holder.java.
        is("other", source(a[0] + "/zz/shop/Other.class"), "Holder.java");
        is("main", source(a[0] + "/zz/shop/Main.class"), "Main.java");
        is("a big constant pool", source(a[1] + "/dla/harness/DlaDump.class"), "DlaDump.java");
        is("not a class file", source(a[0] + "/../src/zz/shop/Main.java"), null);
        System.out.println(bad == 0 ? "OK" : "BAD " + bad);
        System.exit(bad == 0 ? 0 : 1);
    }
}
"#;

const MAIN: &str = r#"package zz.shop;

import dla.harness.DlaDump;

public final class Main {
    public static void main(String[] args) throws Exception {
        Object a = new Other();
        Object b = new Helper();
        DlaDump.checkpoint(1, false);
        DlaDump.checkpoint(2, false);
        DlaDump.checkpoint(3, false);
        if (System.getenv("ZZ_CUT") != null) {
            // A driver killed at its timeout: no last checkpoint, no shutdown hook.
            Runtime.getRuntime().halt(0);
        }
        DlaDump.checkpoint(4, true);
        System.out.println(a.hashCode() + b.hashCode());
    }

    static final class Helper {}
}
"#;

const HOLDER: &str = "package zz.shop;\n\nclass Other {}\n";

/// A compiled `zz.shop` project: `dir/proj/classes/zz/shop/{Main,Other,Main$Helper}.class`.
fn build_project(dir: &Path, agent_classes: &Path) -> PathBuf {
    let src = dir.join("proj/src/zz/shop");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("Main.java"), MAIN).unwrap();
    fs::write(src.join("Holder.java"), HOLDER).unwrap();
    let classes = dir.join("proj/classes");
    fs::create_dir_all(&classes).unwrap();
    run_ok(
        Command::new("javac")
            .arg("-cp")
            .arg(agent_classes)
            .arg("-d")
            .arg(&classes)
            .arg(src.join("Main.java"))
            .arg(src.join("Holder.java")),
    );
    classes
}

#[test]
fn locations_and_the_source_file_attribute_are_read_by_the_agent() {
    if !have_jdk() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let (agent_classes, _) = build_agent(dir.path());
    let classes = build_project(dir.path(), &agent_classes);
    let check = dir.path().join("check/dla/harness");
    fs::create_dir_all(&check).unwrap();
    fs::write(check.join("CheckCoverage.java"), CHECK).unwrap();
    let out_classes = dir.path().join("check/classes");
    fs::create_dir_all(&out_classes).unwrap();
    let app = dir.path().join("check/zz/app");
    fs::create_dir_all(&app).unwrap();
    let mut files = vec![check.join("CheckCoverage.java")];
    for name in ["Thing", "DlaLoop", "DlaJUnitLoop"] {
        let file = app.join(format!("{name}.java"));
        fs::write(
            &file,
            format!("package zz.app;\npublic class {name} {{}}\n"),
        )
        .unwrap();
        files.push(file);
    }
    run_ok(
        Command::new("javac")
            .arg("-cp")
            .arg(&agent_classes)
            .arg("-d")
            .arg(&out_classes)
            .args(&files),
    );
    let sep = if cfg!(windows) { ";" } else { ":" };
    let text = run_ok(
        Command::new("java")
            .arg("-cp")
            .arg(format!(
                "{}{sep}{}",
                out_classes.display(),
                agent_classes.display()
            ))
            .arg("dla.harness.CheckCoverage")
            .arg(&classes)
            .arg(&agent_classes),
    );
    assert!(text.trim_end().ends_with("OK"), "{text}");
}

/// The coverage records a `zz.shop` driver wrote, as `(phase, record)`.
fn coverage_records(dump: &Path) -> Vec<(String, Value)> {
    fs::read_to_string(dump)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| serde_json::from_str::<Value>(l).ok())
        .map(|v| (v["phase"].as_str().unwrap_or("").to_owned(), v))
        .collect()
}

fn run_driver(dir: &Path, project_env: &Path, packages: Option<&str>) -> Vec<(String, Value)> {
    run_driver_cut(dir, project_env, packages, false)
}

/// `cut`: the driver halts after its third checkpoint, as a timeout would end it.
fn run_driver_cut(
    dir: &Path,
    project_env: &Path,
    packages: Option<&str>,
    cut: bool,
) -> Vec<(String, Value)> {
    let (agent_classes, jar) = build_agent(dir);
    let classes = build_project(dir, &agent_classes);
    let dump = dir.join("dump.jsonl");
    let _ = fs::remove_file(&dump);
    let mut cmd = Command::new("java");
    cmd.arg(format!("-javaagent:{}", jar.display()))
        .arg("-cp")
        .arg(format!(
            "{}{}{}",
            classes.display(),
            if cfg!(windows) { ";" } else { ":" },
            jar.display()
        ))
        .arg("zz.shop.Main")
        .env("DLA_HARNESS_OUT", &dump)
        .env("DLA_PROJECT", project_env)
        .env_remove("DLA_PROJECT_PACKAGES")
        .env_remove("ZZ_CUT");
    if cut {
        cmd.env("ZZ_CUT", "1");
    }
    if let Some(p) = packages {
        cmd.env("DLA_PROJECT_PACKAGES", p);
    }
    run_ok(&mut cmd);
    coverage_records(&dump)
}

#[test]
fn a_drivers_last_checkpoint_carries_the_coverage_with_the_project_first() {
    if !have_jdk() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    // The project root holds the classes: the code source is under it, no package is given.
    let root = dir.path().join("proj");
    let recs = run_driver(dir.path(), &root, None);
    let phases: Vec<&str> = recs.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(phases, ["checkpoint"; 4], "no exit record after a driver");
    // The first, second, fourth and last checkpoints carry the section; the third does not.
    let has: Vec<bool> = recs
        .iter()
        .map(|(_, r)| r.get("coverage").is_some())
        .collect();
    assert_eq!(has, [true, true, false, true]);
    let cov = &recs[3].1["coverage"];
    let items: Vec<&str> = cov["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert_eq!(
        &items[..3],
        ["zz.shop.Main", "zz.shop.Main$Helper", "zz.shop.Other"],
        "{items:?}"
    );
    // Main and Other are the two top-level classes; Helper is inside Main.
    assert_eq!(cov["project"], 2);
    assert_eq!(
        cov["sources"],
        serde_json::json!(["zz/shop/Holder.java", "zz/shop/Main.java"])
    );
}

#[test]
fn a_package_the_sources_declare_ranks_classes_whose_jar_lies_elsewhere() {
    if !have_jdk() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    // DLA_PROJECT names another directory: only the package says the classes are the project's.
    let elsewhere = dir.path().join("elsewhere");
    fs::create_dir_all(&elsewhere).unwrap();
    let recs = run_driver(dir.path(), &elsewhere, Some("zz.shop"));
    let cov = &recs[3].1["coverage"];
    let items: Vec<&str> = cov["items"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert_eq!(
        &items[..3],
        ["zz.shop.Main", "zz.shop.Main$Helper", "zz.shop.Other"],
        "{items:?}"
    );
    assert_eq!(cov["project"], 2);
    // Without the package or the location nothing is the project's, and the count says so.
    let dir2 = tempfile::tempdir().unwrap();
    let elsewhere = dir2.path().join("elsewhere");
    fs::create_dir_all(&elsewhere).unwrap();
    let recs = run_driver(dir2.path(), &elsewhere, None);
    let cov = &recs[3].1["coverage"];
    assert_eq!(cov["project"], 0);
}

#[test]
fn a_driver_cut_off_before_its_last_checkpoint_has_still_written_the_coverage() {
    if !have_jdk() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("proj");
    let recs = run_driver_cut(dir.path(), &root, None, true);
    // Three checkpoints, no last one and (halt) no exit record.
    assert_eq!(recs.len(), 3, "{recs:?}");
    let cov = recs
        .iter()
        .find_map(|(_, r)| r.get("coverage"))
        .expect("an earlier checkpoint carries the section");
    assert_eq!(cov["project"], 2);
}
