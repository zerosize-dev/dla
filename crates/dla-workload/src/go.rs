//! Go workloads: `go test` per package, `go run` per main package.

use std::{collections::BTreeMap, sync::LazyLock};

use dla_core::{runtime::Runtime, workload::WorkloadKind};
use regex::Regex;

use crate::model::{FileInfo, Raw, Scan, cwd_of, dir_of, strip_dir};

static TEST_FN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^func\s+(?:Test|Example)\w*\s*\(").expect("static regex"));
static PKG_MAIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^package\s+main\b").expect("static regex"));
static FUNC_MAIN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^func\s+main\s*\(\s*\)").expect("static regex"));

fn pkg_arg(module_root: &str, dir: &str) -> String {
    let rel = strip_dir(dir, module_root);
    if dir == module_root || rel.is_empty() {
        ".".to_owned()
    } else {
        format!("./{rel}")
    }
}

pub fn discover(scan: &Scan) -> Vec<Raw> {
    let mut pkgs: BTreeMap<(String, String), Vec<&FileInfo>> = BTreeMap::new();
    for f in scan.files_of(Runtime::Go) {
        let Some((module_root, _)) = scan.nearest(&f.rel, &["go.mod"]) else {
            continue;
        };
        pkgs.entry((module_root, dir_of(&f.rel).to_owned()))
            .or_default()
            .push(f);
    }
    let mut out = Vec::new();
    for ((module_root, dir), files) in pkgs {
        let arg = pkg_arg(&module_root, &dir);
        let tests: Vec<&FileInfo> = files
            .iter()
            .copied()
            .filter(|f| f.rel.ends_with("_test.go") && TEST_FN.is_match(&f.text))
            .collect();
        if let Some(first) = tests.first() {
            out.push(Raw {
                id: format!("go:test:{}:{arg}", cwd_of(&module_root)),
                kind: WorkloadKind::Test,
                runtime: Runtime::Go,
                command: format!("go test -count=1 {arg}"),
                cwd: cwd_of(&module_root),
                file: first.rel.clone(),
                srcs: tests.iter().map(|f| f.rel.clone()).collect(),
                extra: Default::default(),
            });
        }
        let non_test: Vec<&FileInfo> = files
            .iter()
            .copied()
            .filter(|f| !f.rel.ends_with("_test.go"))
            .collect();
        let main_file = non_test
            .iter()
            .find(|f| PKG_MAIN.is_match(&f.text) && FUNC_MAIN.is_match(&f.text));
        if let Some(m) = main_file {
            let server = non_test
                .iter()
                .any(|f| f.text.contains("ListenAndServe") || f.text.contains("http.Server{"));
            out.push(Raw {
                id: format!("go:main:{}:{arg}", cwd_of(&module_root)),
                kind: if server {
                    WorkloadKind::DevServer
                } else {
                    WorkloadKind::Main
                },
                runtime: Runtime::Go,
                command: format!("go run {arg}"),
                cwd: cwd_of(&module_root),
                file: m.rel.clone(),
                srcs: non_test.iter().map(|f| f.rel.clone()).collect(),
                extra: Default::default(),
            });
        }
    }
    out
}
