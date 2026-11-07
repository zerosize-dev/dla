//! JVM workloads: JUnit/TestNG classes, `*Test` mains, `main` classes, Spring Boot apps.

use std::{collections::BTreeMap, sync::LazyLock};

use dla_core::{runtime::Runtime, workload::WorkloadKind};
use regex::Regex;

use crate::model::{FileInfo, Raw, Scan, cwd_of, join, stem};

static PACKAGE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*package\s+([\w.]+)").expect("static regex"));
static JAVA_MAIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"static\s+void\s+main\s*\(").expect("static regex"));
static KT_MAIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*(?:suspend\s+)?fun\s+main\s*\(").expect("static regex"));
static RUN_ARG: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"["']run["']|\brun\)"#).expect("static regex"));

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Tool {
    Maven,
    Gradle,
    Script,
    Bare,
}

fn has_main(f: &FileInfo) -> bool {
    JAVA_MAIN.is_match(&f.text) || KT_MAIN.is_match(&f.text)
}

fn is_kotlin(f: &FileInfo) -> bool {
    f.rel.ends_with(".kt") || f.rel.ends_with(".kts")
}

fn fqcn(f: &FileInfo) -> String {
    let pkg = PACKAGE
        .captures(&f.text)
        .map(|c| c[1].to_owned())
        .unwrap_or_default();
    let mut name = stem(&f.rel).to_owned();
    if is_kotlin(f) && !f.text.contains(&format!("object {name}")) {
        name.push_str("Kt");
    }
    if pkg.is_empty() {
        name
    } else {
        format!("{pkg}.{name}")
    }
}

fn shell_for(script: &str) -> &'static str {
    let first = script.lines().next().unwrap_or("");
    if first.contains("bash") { "bash" } else { "sh" }
}

fn compile_all(scan: &Scan, root: &str) -> String {
    let kotlin = scan
        .files_of(Runtime::Jvm)
        .any(|f| is_kotlin(f) && (root.is_empty() || f.rel.starts_with(root)));
    if kotlin {
        "mkdir -p out && kotlinc $(find . -name '*.kt' -o -name '*.java' | grep -v '^./out/') -d out"
            .to_owned()
    } else {
        "mkdir -p out && javac -d out $(find . -name '*.java' -not -path './out/*')".to_owned()
    }
}

pub fn discover(scan: &Scan) -> Vec<Raw> {
    let mut groups: BTreeMap<(String, Tool), Vec<&FileInfo>> = BTreeMap::new();
    for f in scan.files_of(Runtime::Jvm) {
        let (root, tool) = match scan.nearest(
            &f.rel,
            &[
                "pom.xml",
                "build.gradle",
                "build.gradle.kts",
                "settings.gradle",
                "settings.gradle.kts",
                "build.sh",
            ],
        ) {
            Some((d, n)) if n == "pom.xml" => (d, Tool::Maven),
            Some((d, n)) if n == "build.sh" => (d, Tool::Script),
            Some((d, _)) => (d, Tool::Gradle),
            None => (String::new(), Tool::Bare),
        };
        groups.entry((root, tool)).or_default().push(f);
    }
    let mut out = Vec::new();
    for ((root, tool), files) in groups {
        let cwd = cwd_of(&root);
        let rt = Runtime::Jvm;
        let java_run = |f: &FileInfo, cp: &str, ea: bool| {
            format!("java{} -cp {cp} {}", if ea { " -ea" } else { "" }, fqcn(f))
        };
        let tests: Vec<&FileInfo> = files.iter().copied().filter(|f| f.is_test).collect();
        let mains: Vec<&FileInfo> = files
            .iter()
            .copied()
            .filter(|f| !f.is_test && has_main(f))
            .collect();
        let spring: Vec<&FileInfo> = files
            .iter()
            .copied()
            .filter(|f| !f.is_test && f.text.contains("@SpringBootApplication"))
            .collect();
        let build_file = |name: &str| join(&root, name);
        match tool {
            Tool::Script => {
                let script = build_file("build.sh");
                let body = scan.read(&script).unwrap_or_default();
                let sh = shell_for(&body);
                let main_tests: Vec<&FileInfo> =
                    tests.iter().copied().filter(|f| has_main(f)).collect();
                if !main_tests.is_empty() {
                    out.push(Raw {
                        id: format!("jvm:test:{script}"),
                        kind: WorkloadKind::Test,
                        runtime: rt,
                        command: format!("{sh} ./build.sh"),
                        cwd: cwd.clone(),
                        file: script.clone(),
                        srcs: main_tests.iter().map(|f| f.rel.clone()).collect(),
                        extra: Default::default(),
                    });
                }
                if !mains.is_empty() {
                    let (id, command) = if RUN_ARG.is_match(&body) {
                        (
                            format!("jvm:main:{script}:run"),
                            format!("{sh} ./build.sh run"),
                        )
                    } else {
                        (
                            format!("jvm:main:{}", mains[0].rel),
                            format!(
                                "{} && {}",
                                compile_all(scan, &root),
                                java_run(mains[0], "out", false)
                            ),
                        )
                    };
                    out.push(Raw {
                        id,
                        kind: WorkloadKind::Main,
                        runtime: rt,
                        command,
                        cwd: cwd.clone(),
                        file: mains[0].rel.clone(),
                        srcs: mains.iter().map(|f| f.rel.clone()).collect(),
                        extra: Default::default(),
                    });
                }
            }
            Tool::Maven | Tool::Gradle | Tool::Bare => {
                for f in &tests {
                    let junit = f.text.contains("@Test") || f.text.contains("@ParameterizedTest");
                    let class = fqcn(f);
                    let simple = stem(&f.rel);
                    let command = match (tool, junit) {
                        (Tool::Maven, true) => format!("mvn -q -o test -Dtest={simple}"),
                        (Tool::Gradle, true) => format!("gradle -q --offline test --tests {class}"),
                        (Tool::Maven, false) if has_main(f) => format!(
                            "mvn -q -o test-compile && {}",
                            java_run(f, "target/classes:target/test-classes", true)
                        ),
                        (Tool::Gradle, false) if has_main(f) => format!(
                            "gradle -q --offline testClasses && {}",
                            java_run(
                                f,
                                "build/classes/java/main:build/classes/java/test:build/classes/kotlin/main:build/classes/kotlin/test",
                                true
                            )
                        ),
                        (Tool::Bare, false) if has_main(f) => {
                            format!(
                                "{} && {}",
                                compile_all(scan, &root),
                                java_run(f, "out", true)
                            )
                        }
                        _ => continue,
                    };
                    out.push(Raw {
                        id: format!("jvm:test:{}", f.rel),
                        kind: WorkloadKind::Test,
                        runtime: rt,
                        command,
                        cwd: cwd.clone(),
                        file: f.rel.clone(),
                        srcs: vec![f.rel.clone()],
                        extra: Default::default(),
                    });
                }
                for f in &spring {
                    let command = match tool {
                        Tool::Maven => "mvn -q -o spring-boot:run".to_owned(),
                        Tool::Gradle => "gradle -q --offline bootRun".to_owned(),
                        _ => continue,
                    };
                    out.push(Raw {
                        id: format!("jvm:dev-server:{}", f.rel),
                        kind: WorkloadKind::DevServer,
                        runtime: rt,
                        command,
                        cwd: cwd.clone(),
                        file: f.rel.clone(),
                        srcs: vec![f.rel.clone()],
                        extra: Default::default(),
                    });
                }
                for f in mains
                    .iter()
                    .filter(|m| !spring.iter().any(|s| s.rel == m.rel))
                {
                    let command = match tool {
                        Tool::Maven => format!(
                            "mvn -q -o compile && {}",
                            java_run(f, "target/classes", false)
                        ),
                        Tool::Gradle => format!(
                            "gradle -q --offline classes && {}",
                            java_run(
                                f,
                                "build/classes/java/main:build/classes/kotlin/main",
                                false
                            )
                        ),
                        _ => format!(
                            "{} && {}",
                            compile_all(scan, &root),
                            java_run(f, "out", false)
                        ),
                    };
                    out.push(Raw {
                        id: format!("jvm:main:{}", f.rel),
                        kind: WorkloadKind::Main,
                        runtime: rt,
                        command,
                        cwd: cwd.clone(),
                        file: f.rel.clone(),
                        srcs: vec![f.rel.clone()],
                        extra: Default::default(),
                    });
                }
            }
        }
    }
    out
}
