//! Python workloads: unittest/pytest files, `__main__` modules, console scripts,
//! dev servers.

use std::{collections::BTreeMap, sync::LazyLock};

use dla_core::{runtime::Runtime, workload::WorkloadKind};
use regex::Regex;

use crate::model::{FileInfo, Raw, Scan, cwd_of, join, stem, strip_dir};

static MAIN_GUARD: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^if\s+__name__\s*==\s*['"]__main__['"]"#).expect("static regex")
});
static SERVER: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"uvicorn\.run\(|\bapp\.run\(|serve_forever\(|runserver|hypercorn|waitress")
        .expect("static regex")
});

fn is_pytest(scan: &Scan, root: &str, f: &FileInfo) -> bool {
    let t = &f.text;
    if t.contains("import pytest") || t.contains("from pytest") {
        return true;
    }
    if t.contains("unittest") {
        return false;
    }
    scan.has(&join(root, "pytest.ini"))
        || scan.has(&join(root, "conftest.py"))
        || scan
            .read(&join(root, "pyproject.toml"))
            .is_some_and(|p| p.contains("pytest"))
        || t.contains("\ndef test_")
        || t.starts_with("def test_")
}

/// Dotted module name when every directory between `root` and the file is a package.
fn module_name(scan: &Scan, root: &str, rel: &str) -> Option<String> {
    let inner = strip_dir(rel, root);
    let mut parts: Vec<&str> = inner.split('/').collect();
    let file = parts.pop()?;
    let mut dir = root.to_owned();
    for p in &parts {
        dir = join(&dir, p);
        if !scan.has(&join(&dir, "__init__.py")) {
            return None;
        }
    }
    let m = stem(file);
    parts.push(if m == "__main__" { "" } else { m });
    Some(
        parts
            .into_iter()
            .filter(|p| !p.is_empty())
            .collect::<Vec<_>>()
            .join("."),
    )
}

pub fn discover(scan: &Scan) -> Vec<Raw> {
    let mut by_root: BTreeMap<String, Vec<&FileInfo>> = BTreeMap::new();
    for f in scan.files_of(Runtime::Python) {
        let root = scan
            .nearest(&f.rel, &["pyproject.toml", "setup.py", "setup.cfg"])
            .map(|(d, _)| d)
            .unwrap_or_default();
        by_root.entry(root).or_default().push(f);
    }
    let mut out = Vec::new();
    for (root, files) in by_root {
        let cwd = cwd_of(&root);
        for f in files.iter().filter(|f| {
            f.is_test
                && stem(&f.rel) != "conftest"
                && stem(&f.rel) != "setup"
                && stem(&f.rel) != "__main__"
        }) {
            let local = strip_dir(&f.rel, &root);
            let command = if is_pytest(scan, &root, f) {
                format!("python3 -m pytest -q {local}")
            } else {
                let dotted = local.trim_end_matches(".py").replace('/', ".");
                format!("python3 -m unittest {dotted}")
            };
            out.push(Raw {
                id: format!("python:test:{}", f.rel),
                kind: WorkloadKind::Test,
                runtime: Runtime::Python,
                command,
                cwd: cwd.clone(),
                file: f.rel.clone(),
                srcs: vec![f.rel.clone()],
                extra: Default::default(),
            });
        }
        for f in files
            .iter()
            .filter(|f| !f.is_test || stem(&f.rel) == "__main__")
        {
            let is_dunder_main = stem(&f.rel) == "__main__";
            if !(is_dunder_main || MAIN_GUARD.is_match(&f.text)) || stem(&f.rel) == "setup" {
                continue;
            }
            let local = strip_dir(&f.rel, &root);
            let command = match module_name(scan, &root, &f.rel) {
                Some(m) if !m.is_empty() => format!("python3 -m {m}"),
                _ => format!("python3 {local}"),
            };
            let server = SERVER.is_match(&f.text)
                || stem(&f.rel) == "manage" && f.text.contains("runserver");
            let command =
                if stem(&f.rel) == "manage" && f.text.contains("execute_from_command_line") {
                    "python3 manage.py runserver".to_owned()
                } else {
                    command
                };
            out.push(Raw {
                id: format!("python:main:{}", f.rel),
                kind: if server {
                    WorkloadKind::DevServer
                } else {
                    WorkloadKind::Main
                },
                runtime: Runtime::Python,
                command,
                cwd: cwd.clone(),
                file: f.rel.clone(),
                srcs: vec![f.rel.clone()],
                extra: Default::default(),
            });
        }
        // Console scripts from pyproject.toml.
        if let Some(t) = scan
            .read(&join(&root, "pyproject.toml"))
            .and_then(|s| s.parse::<toml::Table>().ok())
        {
            let mut scripts: Vec<(String, String)> = Vec::new();
            let tables = [
                t.get("project").and_then(|p| p.get("scripts")),
                t.get("tool")
                    .and_then(|p| p.get("poetry"))
                    .and_then(|p| p.get("scripts")),
            ];
            for tbl in tables.into_iter().flatten().filter_map(|v| v.as_table()) {
                for (k, v) in tbl {
                    if let Some(s) = v.as_str() {
                        scripts.push((k.clone(), s.to_owned()));
                    }
                }
            }
            for (name, target) in scripts {
                let Some((module, func)) = target.split_once(':') else {
                    continue;
                };
                let file = join(&root, &format!("{}.py", module.replace('.', "/")));
                out.push(Raw {
                    id: format!("python:script:{name}"),
                    kind: WorkloadKind::Script,
                    runtime: Runtime::Python,
                    command: format!(
                        "python3 -c \"import sys; from {module} import {func}; sys.exit({func}())\""
                    ),
                    cwd: cwd.clone(),
                    file: if scan.has(&file) {
                        file.clone()
                    } else {
                        join(&root, "pyproject.toml")
                    },
                    srcs: if scan.has(&file) {
                        vec![file]
                    } else {
                        Vec::new()
                    },
                    extra: Default::default(),
                });
            }
        }
    }
    out
}
