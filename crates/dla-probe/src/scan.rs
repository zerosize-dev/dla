//! Static reading of the project tree: runtimes, declared versions, test runners and
//! libraries.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use dla_core::{
    probe::{LibraryProbe, RuntimeProbe, TestRunnerProbe},
    runtime::Runtime,
};
use regex::Regex;

/// Directory names never entered.
pub const SKIP_DIRS: &[&str] = &[
    ".git",
    ".dla",
    "target",
    "build",
    "node_modules",
    "venv",
    ".venv",
    "out",
    "dist",
    "__pycache__",
    ".gradle",
    ".tox",
    ".mypy_cache",
    ".pytest_cache",
    ".idea",
    "site-packages",
];

const MAX_FILES: usize = 100_000;
const MAX_READ_BYTES: u64 = 512 * 1024;
const MAX_TOTAL_READ: u64 = 128 * 1024 * 1024;

/// The files of a project, project-relative with `/` separators, sorted.
#[derive(Debug, Clone)]
pub struct Tree {
    /// Absolute root.
    pub root: PathBuf,
    /// Relative file paths, sorted.
    pub files: Vec<String>,
    /// True when the walk stopped at the file limit.
    pub truncated: bool,
}

impl Tree {
    /// Walks `root`, skipping [`SKIP_DIRS`] and symlinks.
    pub fn walk(root: &Path) -> Tree {
        let mut files = Vec::new();
        let mut truncated = false;
        let mut stack = vec![(root.to_path_buf(), String::new())];
        'outer: while let Some((dir, rel)) = stack.pop() {
            let Ok(rd) = fs::read_dir(&dir) else { continue };
            let mut entries: Vec<_> = rd.filter_map(Result::ok).collect();
            entries.sort_by_key(|e| e.file_name());
            for e in entries {
                let name = e.file_name().to_string_lossy().into_owned();
                let Ok(ft) = e.file_type() else { continue };
                let child_rel = if rel.is_empty() {
                    name.clone()
                } else {
                    format!("{rel}/{name}")
                };
                if ft.is_dir() {
                    if !SKIP_DIRS.contains(&name.as_str()) {
                        stack.push((e.path(), child_rel));
                    }
                } else if ft.is_file() {
                    if files.len() >= MAX_FILES {
                        truncated = true;
                        break 'outer;
                    }
                    files.push(child_rel);
                }
            }
        }
        files.sort();
        Tree {
            root: root.to_path_buf(),
            files,
            truncated,
        }
    }

    /// A tree over an explicit file list (tests).
    pub fn read(&self, rel: &str) -> Option<String> {
        let p = self.root.join(rel);
        let md = fs::metadata(&p).ok()?;
        if md.len() > MAX_READ_BYTES {
            return None;
        }
        let bytes = fs::read(&p).ok()?;
        Some(String::from_utf8_lossy(&bytes).into_owned())
    }
}

fn basename(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel)
}

fn dir_of(rel: &str) -> &str {
    rel.rfind('/').map_or("", |i| &rel[..i])
}

fn depth(rel: &str) -> usize {
    rel.matches('/').count()
}

/// How a file relates to a runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Role {
    Build,
    Source,
}

fn classify(rel: &str) -> Option<(Runtime, Role)> {
    let name = basename(rel);
    let ext = name.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
    let lower = name.to_ascii_lowercase();
    let r = match name {
        "pom.xml"
        | "build.gradle"
        | "build.gradle.kts"
        | "settings.gradle"
        | "settings.gradle.kts" => (Runtime::Jvm, Role::Build),
        "go.mod" => (Runtime::Go, Role::Build),
        "Cargo.toml" => (Runtime::Rust, Role::Build),
        "package.json" => (Runtime::Node, Role::Build),
        "pyproject.toml" | "setup.py" | "setup.cfg" => (Runtime::Python, Role::Build),
        "Makefile" | "makefile" | "GNUmakefile" | "CMakeLists.txt" => {
            (Runtime::Native, Role::Build)
        }
        _ if lower.starts_with("requirements") && lower.ends_with(".txt") => {
            (Runtime::Python, Role::Build)
        }
        _ => match ext {
            "java" | "kt" => (Runtime::Jvm, Role::Source),
            "go" => (Runtime::Go, Role::Source),
            "rs" => (Runtime::Rust, Role::Source),
            "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => (Runtime::Node, Role::Source),
            "py" => (Runtime::Python, Role::Source),
            "c" | "cc" | "cpp" | "cxx" | "h" | "hpp" => (Runtime::Native, Role::Source),
            _ => return None,
        },
    };
    Some(r)
}

/// A detected runtime plus the facts later steps need.
#[derive(Debug, Clone)]
pub struct Detected {
    /// The report entry (toolchain filled in later).
    pub probe: RuntimeProbe,
    /// Project-relative directory holding the primary build file ("" = project root).
    pub root: String,
    /// Build files found, shallowest first.
    pub build_files: Vec<String>,
    /// True when Kotlin sources exist (JVM).
    pub has_kotlin: bool,
}

/// Detects the runtimes present in `tree`.
pub fn detect(tree: &Tree) -> Vec<Detected> {
    let mut build: BTreeMap<Runtime, Vec<String>> = BTreeMap::new();
    let mut src: BTreeMap<Runtime, Vec<String>> = BTreeMap::new();
    for f in &tree.files {
        match classify(f) {
            Some((rt, Role::Build)) => build.entry(rt).or_default().push(f.clone()),
            Some((rt, Role::Source)) => src.entry(rt).or_default().push(f.clone()),
            None => {}
        }
    }
    let mut out = Vec::new();
    for &rt in Runtime::ALL {
        let srcs = src.get(&rt).cloned().unwrap_or_default();
        let mut builds = build.get(&rt).cloned().unwrap_or_default();
        // Native needs C or C++ sources; a Makefile alone (a Python project's `make
        // test`) is not a native runtime.
        if rt == Runtime::Native && srcs.is_empty() {
            continue;
        }
        if builds.is_empty() && srcs.is_empty() {
            continue;
        }
        builds.sort_by(|a, b| (depth(a), a).cmp(&(depth(b), b)));
        let root = builds
            .first()
            .map(|b| dir_of(b).to_owned())
            .unwrap_or_default();
        let mut detected_by: Vec<String> = builds.iter().take(5).cloned().collect();
        if detected_by.is_empty() {
            detected_by = srcs.iter().take(3).cloned().collect();
        }
        let declared = builds
            .first()
            .and_then(|b| tree.read(b).map(|t| (b.clone(), t)))
            .and_then(|(b, t)| declared_version(rt, basename(&b), &t));
        let has_kotlin = srcs.iter().any(|s| s.ends_with(".kt"));
        out.push(Detected {
            probe: RuntimeProbe {
                runtime: rt,
                detected_by,
                source_files: srcs.len() as u64,
                toolchain: Vec::new(),
                declared_version: declared,
            },
            root,
            build_files: builds,
            has_kotlin,
        });
    }
    out
}

fn cap(re: &str, text: &str) -> Option<String> {
    Regex::new(re)
        .ok()?
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_owned())
}

/// The version a build file declares for its runtime.
pub fn declared_version(rt: Runtime, file: &str, text: &str) -> Option<String> {
    match rt {
        Runtime::Go => cap(r"(?m)^\s*go\s+(\S+)", text),
        Runtime::Rust => cap(r#"(?m)^\s*rust-version\s*=\s*"([^"]+)""#, text)
            .map(|v| format!("rust-version {v}"))
            .or_else(|| {
                cap(r#"(?m)^\s*edition\s*=\s*"([^"]+)""#, text).map(|v| format!("edition {v}"))
            }),
        Runtime::Node => {
            let v: serde_json::Value = serde_json::from_str(text).ok()?;
            v.get("engines")?
                .get("node")?
                .as_str()
                .map(|s| format!("node {s}"))
        }
        Runtime::Python => cap(r#"(?m)^\s*requires-python\s*=\s*["']([^"']+)["']"#, text)
            .or_else(|| cap(r"(?m)^\s*python_requires\s*=\s*([^\n]+)", text))
            .map(|v| format!("python {v}")),
        Runtime::Jvm => {
            if file == "pom.xml" {
                cap(
                    r"<(?:maven\.compiler\.release|maven\.compiler\.source|java\.version|release|source)>\s*([\d.]+)\s*<",
                    text,
                )
            } else {
                cap(r"jvmToolchain\(\s*(\d+)\s*\)", text)
                    .or_else(|| {
                        cap(r"JavaVersion\.VERSION_([\d_]+)", text).map(|v| v.replace('_', "."))
                    })
                    .or_else(|| {
                        cap(
                            r#"(?:source|target)Compatibility\s*=?\s*['"]?([\d.]+)"#,
                            text,
                        )
                    })
                    .or_else(|| {
                        cap(
                            r"languageVersion\s*(?:=|\.set\()\s*JavaLanguageVersion\.of\(\s*(\d+)",
                            text,
                        )
                    })
            }
        }
        Runtime::Native => cap(r"-std=([\w+]+)", text)
            .map(|v| format!("-std={v}"))
            .or_else(|| {
                cap(r"CMAKE_C(?:XX)?_STANDARD\s+(\d+)", text).map(|v| format!("c-standard {v}"))
            }),
    }
}

/// How a rule selects files within its runtime.
#[derive(Clone, Copy)]
enum Sel {
    /// Any file of the runtime (build files and sources).
    Any,
    /// Files whose basename ends with the suffix.
    Suffix(&'static str),
    /// Files with exactly one of these basenames.
    Names(&'static [&'static str]),
    /// Files with this extension.
    Ext(&'static str),
}

impl Sel {
    fn matches(self, rel: &str) -> bool {
        let n = basename(rel);
        match self {
            Sel::Any => true,
            Sel::Suffix(s) => n.ends_with(s),
            Sel::Names(ns) => ns.contains(&n),
            Sel::Ext(e) => n.rsplit_once('.').is_some_and(|(_, x)| x == e),
        }
    }
}

struct Rule {
    rt: Runtime,
    name: &'static str,
    re: &'static str,
    sel: Sel,
    /// Harness flags (libraries) or empty (runners).
    flags: &'static [&'static str],
    library: bool,
}

const REQ: Sel = Sel::Names(&[
    "requirements.txt",
    "requirements-dev.txt",
    "requirements_dev.txt",
    "requirements-test.txt",
]);

const RULES: &[Rule] = &[
    // Test runners.
    Rule {
        rt: Runtime::Jvm,
        name: "junit",
        re: r"org\.junit|junit-jupiter|junit-vintage|junit:junit",
        sel: Sel::Any,
        flags: &[],
        library: false,
    },
    Rule {
        rt: Runtime::Jvm,
        name: "testng",
        re: r"org\.testng|<artifactId>testng<|org\.testng:testng",
        sel: Sel::Any,
        flags: &[],
        library: false,
    },
    Rule {
        rt: Runtime::Python,
        name: "pytest",
        re: r"(?m)^\s*(?:import|from)\s+pytest\b|\[tool\.pytest|\[tool:pytest\]",
        sel: Sel::Any,
        flags: &[],
        library: false,
    },
    Rule {
        rt: Runtime::Python,
        name: "pytest",
        re: r"(?mi)^\s*pytest\b",
        sel: REQ,
        flags: &[],
        library: false,
    },
    Rule {
        rt: Runtime::Python,
        name: "unittest",
        re: r"(?m)^\s*import\s+unittest\b|^\s*from\s+unittest\s+import|unittest\.TestCase",
        sel: Sel::Ext("py"),
        flags: &[],
        library: false,
    },
    Rule {
        rt: Runtime::Node,
        name: "node --test",
        re: r#"from\s+['"]node:test['"]|require\(\s*['"]node:test['"]\s*\)|node\s+--test"#,
        sel: Sel::Any,
        flags: &[],
        library: false,
    },
    Rule {
        rt: Runtime::Node,
        name: "jest",
        re: r#"["']jest["']\s*:|from\s+['"]@jest/globals['"]|"jest"\s*:"#,
        sel: Sel::Any,
        flags: &[],
        library: false,
    },
    Rule {
        rt: Runtime::Node,
        name: "vitest",
        re: r#"["']vitest["']\s*:|from\s+['"]vitest['"]"#,
        sel: Sel::Any,
        flags: &[],
        library: false,
    },
    Rule {
        rt: Runtime::Go,
        name: "go test",
        re: r"(?m)^func\s+Test\w*\(",
        sel: Sel::Suffix("_test.go"),
        flags: &[],
        library: false,
    },
    Rule {
        rt: Runtime::Rust,
        name: "cargo test",
        re: r"#\[(?:tokio::|async_std::)?test\b|#\[cfg\(test\)\]",
        sel: Sel::Ext("rs"),
        flags: &[],
        library: false,
    },
    Rule {
        rt: Runtime::Native,
        name: "make test",
        re: r"(?m)^test\s*:",
        sel: Sel::Names(&["Makefile", "makefile", "GNUmakefile"]),
        flags: &[],
        library: false,
    },
    Rule {
        rt: Runtime::Native,
        name: "ctest",
        re: r"enable_testing|add_test\s*\(",
        sel: Sel::Names(&["CMakeLists.txt"]),
        flags: &[],
        library: false,
    },
    // Libraries and the harness flags they imply.
    Rule {
        rt: Runtime::Jvm,
        name: "netty",
        re: r"io\.netty|netty-(?:all|handler|transport|buffer|codec)",
        sel: Sel::Any,
        flags: &["-Dio.netty.leakDetection.level=paranoid"],
        library: true,
    },
    Rule {
        rt: Runtime::Jvm,
        name: "hikaricp",
        re: r"com\.zaxxer\.hikari|HikariCP|HikariConfig|HikariDataSource",
        sel: Sel::Any,
        flags: &["HikariConfig.leakDetectionThreshold=2000 (ms)"],
        library: true,
    },
    Rule {
        rt: Runtime::Jvm,
        name: "micrometer",
        re: r"io\.micrometer|micrometer-core",
        sel: Sel::Any,
        flags: &["io.micrometer.core.instrument.HighCardinalityTagsDetector"],
        library: true,
    },
    Rule {
        rt: Runtime::Jvm,
        name: "tomcat",
        re: r"org\.apache\.catalina|org\.apache\.tomcat|tomcat-embed|spring-boot-starter-web\b",
        sel: Sel::Any,
        flags: &[
            "listener:org.apache.catalina.core.JreMemoryLeakPreventionListener",
            "listener:org.apache.catalina.core.ThreadLocalLeakPreventionListener",
            "note: Tomcat logs leaked threadlocals and timers on context stop",
        ],
        library: true,
    },
    Rule {
        rt: Runtime::Python,
        name: "asyncio",
        re: r"(?m)^\s*import\s+asyncio\b|^\s*from\s+asyncio\b|\basyncio\.run\(",
        sel: Sel::Ext("py"),
        flags: &["PYTHONASYNCIODEBUG=1", "-X dev"],
        library: true,
    },
    Rule {
        rt: Runtime::Go,
        name: "goleak",
        re: r"go\.uber\.org/goleak",
        sel: Sel::Any,
        flags: &["goleak.VerifyTestMain", "goleak.VerifyNone"],
        library: true,
    },
];

/// Test runners and libraries found by reading the tree.
#[derive(Debug, Default, Clone)]
pub struct Findings {
    /// Test runners.
    pub test_runners: Vec<TestRunnerProbe>,
    /// Libraries with harness flags.
    pub libraries: Vec<LibraryProbe>,
}

/// Reads the files of the `detected` runtimes and applies the rules.
pub fn find_runners_and_libraries(tree: &Tree, detected: &[Runtime]) -> Findings {
    let compiled: Vec<(&Rule, Regex)> = RULES
        .iter()
        .filter(|r| detected.contains(&r.rt))
        .filter_map(|r| Regex::new(r.re).ok().map(|re| (r, re)))
        .collect();
    // First evidence file per (runtime, name, kind).
    let mut first: BTreeMap<(Runtime, &'static str, bool), String> = BTreeMap::new();
    let mut read_total = 0u64;
    let mut pkg_jsons: Vec<String> = Vec::new();
    let mut has_python = false;
    let mut first_py: Option<String> = None;
    let mut rust_tests_dir: Option<String> = None;
    for f in &tree.files {
        let Some((rt, _)) = classify(f) else { continue };
        if !detected.contains(&rt) {
            continue;
        }
        if rt == Runtime::Python {
            has_python = true;
            if f.ends_with(".py") && first_py.is_none() {
                first_py = Some(f.clone());
            }
        }
        if rt == Runtime::Node && basename(f) == "package.json" {
            pkg_jsons.push(f.clone());
        }
        if rt == Runtime::Rust
            && f.ends_with(".rs")
            && (f.starts_with("tests/") || f.contains("/tests/"))
            && rust_tests_dir.is_none()
        {
            rust_tests_dir = Some(f.clone());
        }
        let applicable: Vec<&(&Rule, Regex)> = compiled
            .iter()
            .filter(|(r, _)| {
                r.rt == rt && r.sel.matches(f) && !first.contains_key(&(r.rt, r.name, r.library))
            })
            .collect();
        if applicable.is_empty() || read_total > MAX_TOTAL_READ {
            continue;
        }
        let Some(text) = tree.read(f) else { continue };
        read_total += text.len() as u64;
        for (r, re) in applicable {
            if re.is_match(&text) {
                first.insert((r.rt, r.name, r.library), f.clone());
            }
        }
    }
    let mut out = Findings::default();
    for r in RULES {
        if let Some(ev) = first.get(&(r.rt, r.name, r.library)) {
            if r.library {
                if out.libraries.iter().any(|l| l.library == r.name) {
                    continue;
                }
                out.libraries.push(LibraryProbe {
                    runtime: r.rt,
                    library: r.name.into(),
                    evidence: ev.clone(),
                    harness_flags: r.flags.iter().map(|s| (*s).to_owned()).collect(),
                });
            } else if !out
                .test_runners
                .iter()
                .any(|t| t.runtime == r.rt && t.name == r.name)
            {
                out.test_runners.push(TestRunnerProbe {
                    runtime: r.rt,
                    name: r.name.into(),
                    evidence: ev.clone(),
                });
            }
        }
    }
    if has_python && let Some(ev) = first_py {
        out.libraries.push(LibraryProbe {
            runtime: Runtime::Python,
            library: "python-resourcewarning".into(),
            evidence: ev,
            harness_flags: vec!["-W error::ResourceWarning".into()],
        });
    }
    if detected.contains(&Runtime::Rust)
        && let Some(ev) = rust_tests_dir
        && !out.test_runners.iter().any(|t| t.name == "cargo test")
    {
        out.test_runners.push(TestRunnerProbe {
            runtime: Runtime::Rust,
            name: "cargo test".into(),
            evidence: ev,
        });
    }
    // package.json scripts: the test script names the runner; an unrecognised one is
    // recorded as `npm test`.
    for p in pkg_jsons {
        let Some(text) = tree.read(&p) else { continue };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let Some(script) = v.pointer("/scripts/test").and_then(|s| s.as_str()) else {
            continue;
        };
        let name = if script.contains("node --test") {
            "node --test"
        } else if script.contains("jest") {
            "jest"
        } else if script.contains("vitest") {
            "vitest"
        } else {
            "npm test"
        };
        if !out
            .test_runners
            .iter()
            .any(|t| t.runtime == Runtime::Node && t.name == name)
        {
            out.test_runners.push(TestRunnerProbe {
                runtime: Runtime::Node,
                name: name.into(),
                evidence: p,
            });
        }
    }
    out.test_runners.sort_by(|a, b| {
        (a.runtime.as_str(), &a.name, &a.evidence).cmp(&(b.runtime.as_str(), &b.name, &b.evidence))
    });
    out.libraries
        .sort_by(|a, b| (a.runtime.as_str(), &a.library).cmp(&(b.runtime.as_str(), &b.library)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn runtimes_of(d: &[Detected]) -> BTreeSet<Runtime> {
        d.iter().map(|x| x.probe.runtime).collect()
    }

    fn tree(files: &[(&str, &str)]) -> (tempfile::TempDir, Tree) {
        let d = tempfile::tempdir().unwrap();
        for (p, c) in files {
            let path = d.path().join(p);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, c).unwrap();
        }
        let t = Tree::walk(d.path());
        (d, t)
    }

    #[test]
    fn walk_skips_generated_directories() {
        let (_d, t) = tree(&[
            ("a.py", ""),
            ("node_modules/x/i.js", ""),
            ("target/classes/A.java", ""),
            (".git/config", ""),
            (".dla/r/x.json", ""),
            ("venv/lib/z.py", ""),
            ("src/__pycache__/q.py", ""),
        ]);
        assert_eq!(t.files, vec!["a.py"]);
    }

    #[test]
    fn detects_every_runtime_from_build_files_and_sources() {
        let (_d, t) = tree(&[
            (
                "pom.xml",
                "<project><properties><maven.compiler.release>17</maven.compiler.release></properties></project>",
            ),
            ("src/A.java", "class A {}"),
            ("svc/go.mod", "module m\n\ngo 1.22.1\n"),
            ("svc/main.go", "package main"),
            (
                "Cargo.toml",
                "[package]\nname=\"x\"\nedition = \"2021\"\nrust-version = \"1.80\"\n",
            ),
            ("web/package.json", "{\"engines\":{\"node\":\">=20\"}}"),
            ("tool/run.py", ""),
            (
                "pyproject.toml",
                "[project]\nrequires-python = \">=3.10\"\n",
            ),
            ("c/Makefile", "CFLAGS=-std=c11\n"),
            ("c/a.c", ""),
        ]);
        let d = detect(&t);
        let by: BTreeMap<_, _> = d.iter().map(|x| (x.probe.runtime, x)).collect();
        assert_eq!(by.len(), 6);
        assert_eq!(
            by[&Runtime::Jvm].probe.declared_version.as_deref(),
            Some("17")
        );
        assert_eq!(
            by[&Runtime::Go].probe.declared_version.as_deref(),
            Some("1.22.1")
        );
        assert_eq!(by[&Runtime::Go].root, "svc");
        assert_eq!(
            by[&Runtime::Rust].probe.declared_version.as_deref(),
            Some("rust-version 1.80")
        );
        assert_eq!(
            by[&Runtime::Node].probe.declared_version.as_deref(),
            Some("node >=20")
        );
        assert_eq!(
            by[&Runtime::Python].probe.declared_version.as_deref(),
            Some("python >=3.10")
        );
        assert_eq!(
            by[&Runtime::Native].probe.declared_version.as_deref(),
            Some("-std=c11")
        );
        assert_eq!(by[&Runtime::Native].root, "c");
        assert_eq!(by[&Runtime::Jvm].probe.detected_by, vec!["pom.xml"]);
    }

    #[test]
    fn a_makefile_without_c_sources_is_not_native() {
        let (_d, t) = tree(&[("Makefile", "test:\n\tpytest\n"), ("a.py", "")]);
        let rts = runtimes_of(&detect(&t));
        assert_eq!(rts.into_iter().collect::<Vec<_>>(), vec![Runtime::Python]);
    }

    #[test]
    fn sources_alone_detect_a_runtime() {
        let (_d, t) = tree(&[("x/Main.kt", "fun main() {}"), ("a.ts", "")]);
        let d = detect(&t);
        assert_eq!(d.len(), 2);
        let j = d.iter().find(|x| x.probe.runtime == Runtime::Jvm).unwrap();
        assert!(j.has_kotlin);
        assert_eq!(j.probe.detected_by, vec!["x/Main.kt"]);
    }

    #[test]
    fn gradle_declared_versions() {
        assert_eq!(
            declared_version(
                Runtime::Jvm,
                "build.gradle.kts",
                "kotlin { jvmToolchain(21) }"
            )
            .as_deref(),
            Some("21")
        );
        assert_eq!(
            declared_version(
                Runtime::Jvm,
                "build.gradle",
                "sourceCompatibility = JavaVersion.VERSION_1_8"
            )
            .as_deref(),
            Some("1.8")
        );
    }

    #[test]
    fn libraries_map_to_harness_flags() {
        let (_d, t) = tree(&[
            (
                "pom.xml",
                "<dependency><groupId>io.netty</groupId></dependency>",
            ),
            (
                "A.java",
                "import com.zaxxer.hikari.HikariConfig;\nimport io.micrometer.core.instrument.Counter;\nimport org.apache.catalina.Context;\n",
            ),
            (
                "t_test.go",
                "package x\nimport \"go.uber.org/goleak\"\nfunc TestMain(m *testing.M) {}\nfunc TestA(t *testing.T) {}\n",
            ),
            ("go.mod", "go 1.22"),
            ("s.py", "import asyncio\n"),
        ]);
        let rts: Vec<_> = runtimes_of(&detect(&t)).into_iter().collect();
        let f = find_runners_and_libraries(&t, &rts);
        let names: Vec<&str> = f.libraries.iter().map(|l| l.library.as_str()).collect();
        for n in [
            "netty",
            "hikaricp",
            "micrometer",
            "tomcat",
            "goleak",
            "asyncio",
            "python-resourcewarning",
        ] {
            assert!(names.contains(&n), "{n} missing from {names:?}");
        }
        let netty = f.libraries.iter().find(|l| l.library == "netty").unwrap();
        assert_eq!(
            netty.harness_flags,
            vec!["-Dio.netty.leakDetection.level=paranoid"]
        );
        assert_eq!(netty.evidence, "pom.xml");
        let asyncio = f.libraries.iter().find(|l| l.library == "asyncio").unwrap();
        assert!(
            asyncio
                .harness_flags
                .contains(&"PYTHONASYNCIODEBUG=1".to_owned())
        );
        assert!(
            f.test_runners
                .iter()
                .any(|t| t.name == "go test" && t.evidence == "t_test.go")
        );
    }

    #[test]
    fn libraries_in_undetected_runtimes_are_ignored_and_absent_ones_not_invented() {
        let (_d, t) = tree(&[("A.java", "class A {}")]);
        let f = find_runners_and_libraries(&t, &[Runtime::Jvm]);
        assert!(f.libraries.is_empty() && f.test_runners.is_empty());
    }

    #[test]
    fn test_runners_from_build_files_and_imports() {
        let (_d, t) = tree(&[
            (
                "build.gradle",
                "testImplementation 'org.junit.jupiter:junit-jupiter:5'",
            ),
            ("tests/test_a.py", "import pytest\n"),
            (
                "tests/test_b.py",
                "import unittest\nclass T(unittest.TestCase): pass\n",
            ),
            (
                "package.json",
                "{\"scripts\":{\"test\":\"node --test test/\"}}",
            ),
            ("x/test/a.test.js", "const t = require('node:test');"),
            ("Cargo.toml", "[package]\nname=\"a\"\n"),
            ("src/lib.rs", "#[cfg(test)]\nmod tests {}\n"),
            ("Makefile", "all:\n\tcc a.c\ntest: all\n\t./a\n"),
            ("a.c", "int main(){}"),
        ]);
        let rts: Vec<_> = runtimes_of(&detect(&t)).into_iter().collect();
        let f = find_runners_and_libraries(&t, &rts);
        let names: BTreeSet<_> = f.test_runners.iter().map(|t| t.name.as_str()).collect();
        for n in [
            "junit",
            "pytest",
            "unittest",
            "node --test",
            "cargo test",
            "make test",
        ] {
            assert!(names.contains(n), "{n} missing from {names:?}");
        }
    }

    #[test]
    fn an_unrecognised_npm_test_script_is_recorded_generically() {
        let (_d, t) = tree(&[("package.json", "{\"scripts\":{\"test\":\"mocha\"}}")]);
        let f = find_runners_and_libraries(&t, &[Runtime::Node]);
        assert_eq!(f.test_runners.len(), 1);
        assert_eq!(f.test_runners[0].name, "npm test");
    }
}
