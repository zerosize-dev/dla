//! C and C++ workloads: `make test`, CMake/ctest, built binaries.

use std::{collections::BTreeMap, sync::LazyLock};

use dla_core::{runtime::Runtime, workload::WorkloadKind};
use regex::Regex;

use crate::model::{FileInfo, Raw, Scan, cwd_of, join};

static C_MAIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*(?:int|void)\s+main\s*\(").expect("static regex"));
static MAKE_TEST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^(?:check|test)\s*:").expect("static regex"));
static VAR: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^([A-Za-z_][A-Za-z0-9_]*)\s*(?:[:?+]?=)\s*(.*)$").expect("static regex")
});
static ALL_RULE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^all\s*:\s*(.*)$").expect("static regex"));
static RULE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^([A-Za-z0-9_./-]+)\s*:[^=]").expect("static regex"));
static ADD_EXE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"add_executable\s*\(\s*([A-Za-z0-9_.-]+)").expect("static regex"));

static MAKE_REF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\$[({]([A-Za-z0-9_]+)[)}]").expect("static regex"));

fn expand(s: &str, vars: &BTreeMap<String, String>) -> String {
    let mut cur = s.to_owned();
    for _ in 0..5 {
        let next = MAKE_REF
            .replace_all(&cur, |c: &regex::Captures| {
                vars.get(&c[1]).cloned().unwrap_or_default()
            })
            .into_owned();
        if next == cur {
            break;
        }
        cur = next;
    }
    cur
}

/// The program a Makefile builds, found from `all:` or the first rule.
fn make_binary(text: &str) -> Option<String> {
    let mut vars = BTreeMap::new();
    for c in VAR.captures_iter(text) {
        vars.entry(c[1].to_owned())
            .or_insert_with(|| c[2].trim().to_owned());
    }
    let is_prog = |t: &str| {
        !t.is_empty()
            && !t.starts_with('.')
            && !t.contains('%')
            && !t.starts_with("test")
            && !t.starts_with("check")
            && ![".o", ".a", ".so", ".c", ".h", ".cc", ".cpp"]
                .iter()
                .any(|e| t.ends_with(e))
            && !matches!(t, "all" | "clean" | "install" | "distclean" | "FORCE")
    };
    if let Some(c) = ALL_RULE.captures(text) {
        for t in expand(&c[1], &vars).split_whitespace() {
            if is_prog(t) {
                return Some(t.to_owned());
            }
        }
    }
    for c in RULE.captures_iter(text) {
        let t = expand(&c[1], &vars);
        if is_prog(&t) {
            return Some(t);
        }
    }
    None
}

pub fn discover(scan: &Scan) -> Vec<Raw> {
    let mut by_root: BTreeMap<String, (String, Vec<&FileInfo>)> = BTreeMap::new();
    for f in scan.files_of(Runtime::Native) {
        if let Some((root, name)) = scan.nearest(
            &f.rel,
            &["Makefile", "makefile", "GNUmakefile", "CMakeLists.txt"],
        ) {
            by_root
                .entry(root)
                .or_insert_with(|| (name, Vec::new()))
                .1
                .push(f);
        }
    }
    let mut out = Vec::new();
    for (root, (build_name, files)) in by_root {
        let build_rel = join(&root, &build_name);
        let body = scan.read(&build_rel).unwrap_or_default();
        let cmake = build_name == "CMakeLists.txt";
        let cwd = cwd_of(&root);
        let tests: Vec<&FileInfo> = files.iter().copied().filter(|f| f.is_test).collect();
        let mains: Vec<&FileInfo> = files
            .iter()
            .copied()
            .filter(|f| !f.is_test && C_MAIN.is_match(&f.text))
            .collect();
        let cmake_build = "cmake -S . -B build && cmake --build build";
        let test_cmd = if cmake {
            (body.contains("enable_testing") || body.contains("add_test"))
                .then(|| format!("{cmake_build} && ctest --test-dir build --output-on-failure"))
        } else {
            MAKE_TEST.is_match(&body).then(|| "make test".to_owned())
        };
        if let Some(command) = test_cmd {
            out.push(Raw {
                id: format!("native:test:{build_rel}"),
                kind: WorkloadKind::Test,
                runtime: Runtime::Native,
                command,
                cwd: cwd.clone(),
                file: tests.first().map_or(build_rel.clone(), |f| f.rel.clone()),
                srcs: tests.iter().map(|f| f.rel.clone()).collect(),
                extra: Default::default(),
            });
        }
        if mains.is_empty() {
            continue;
        }
        let command = if cmake {
            ADD_EXE
                .captures_iter(&body)
                .map(|c| c[1].to_owned())
                .find(|n| {
                    !n.to_ascii_lowercase().contains("test") && n != "main" || mains.len() == 1
                })
                .map(|n| format!("{cmake_build} && ./build/{n}"))
        } else {
            make_binary(&body).map(|b| format!("make && ./{b}"))
        };
        let Some(command) = command else { continue };
        out.push(Raw {
            id: format!("native:main:{build_rel}"),
            kind: WorkloadKind::Main,
            runtime: Runtime::Native,
            command,
            cwd,
            file: mains[0].rel.clone(),
            srcs: mains.iter().map(|f| f.rel.clone()).collect(),
            extra: Default::default(),
        });
    }
    out
}
