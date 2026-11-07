//! Rust workloads: `cargo test` (inline and integration tests) and `cargo run --bin`.

use std::collections::{BTreeMap, BTreeSet};

use dla_core::{runtime::Runtime, workload::WorkloadKind};

use crate::model::{FileInfo, Raw, Scan, cwd_of, join, stem, strip_dir};

pub fn discover(scan: &Scan) -> Vec<Raw> {
    // Package roots: directories whose Cargo.toml has a [package] table.
    let mut roots: BTreeMap<String, toml::Table> = BTreeMap::new();
    for rel in scan
        .all
        .iter()
        .filter(|r| r.rsplit('/').next() == Some("Cargo.toml"))
    {
        if let Some(t) = scan.read(rel).and_then(|s| s.parse::<toml::Table>().ok())
            && t.contains_key("package")
        {
            roots.insert(crate::model::dir_of(rel).to_owned(), t);
        }
    }
    let mut by_root: BTreeMap<String, Vec<&FileInfo>> = BTreeMap::new();
    for f in scan.files_of(Runtime::Rust) {
        if let Some((root, _)) = scan.nearest(&f.rel, &["Cargo.toml"])
            && roots.contains_key(&root)
        {
            by_root.entry(root).or_default().push(f);
        }
    }
    let mut out = Vec::new();
    for (root, files) in by_root {
        let manifest = &roots[&root];
        let pkg_name = manifest
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("")
            .to_owned();
        let cargo = join(&root, "Cargo.toml");
        let cwd = cwd_of(&root);
        // Inline tests.
        let extra: BTreeSet<String> = files
            .iter()
            .filter(|f| !f.is_test)
            .flat_map(|f| f.test_idents.iter().cloned())
            .collect();
        if !extra.is_empty() {
            out.push(Raw {
                id: format!("rust:test:{cargo}"),
                kind: WorkloadKind::Test,
                runtime: Runtime::Rust,
                command: "cargo test".into(),
                cwd: cwd.clone(),
                file: cargo.clone(),
                srcs: Vec::new(),
                extra,
            });
        }
        // Integration tests: tests/<name>.rs and tests/<name>/main.rs.
        for f in files.iter().filter(|f| f.is_test) {
            let rel = strip_dir(&f.rel, &root);
            let parts: Vec<&str> = rel.split('/').collect();
            let name = match parts.as_slice() {
                ["tests", file] => stem(file).to_owned(),
                ["tests", dir, "main.rs"] => (*dir).to_owned(),
                _ => continue,
            };
            out.push(Raw {
                id: format!("rust:test:{}", f.rel),
                kind: WorkloadKind::Test,
                runtime: Runtime::Rust,
                command: format!("cargo test --test {name}"),
                cwd: cwd.clone(),
                file: f.rel.clone(),
                srcs: vec![f.rel.clone()],
                extra: Default::default(),
            });
        }
        // Binaries.
        for f in files.iter().filter(|f| !f.is_test) {
            let rel = strip_dir(&f.rel, &root);
            let parts: Vec<&str> = rel.split('/').collect();
            let name = match parts.as_slice() {
                ["src", "main.rs"] => pkg_name.clone(),
                ["src", "bin", file] => stem(file).to_owned(),
                ["src", "bin", dir, "main.rs"] => (*dir).to_owned(),
                _ => continue,
            };
            if name.is_empty() {
                continue;
            }
            let server = f.text.contains("TcpListener::bind") || f.text.contains("axum::serve");
            out.push(Raw {
                id: format!("rust:main:{}", f.rel),
                kind: if server {
                    WorkloadKind::DevServer
                } else {
                    WorkloadKind::Main
                },
                runtime: Runtime::Rust,
                command: format!("cargo run -q --bin {name}"),
                cwd: cwd.clone(),
                file: f.rel.clone(),
                srcs: vec![f.rel.clone()],
                extra: Default::default(),
            });
        }
    }
    out
}
