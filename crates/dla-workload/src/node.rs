//! Node workloads: test files, package scripts, `bin` entries, runnable scripts.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::LazyLock,
};

use dla_core::{runtime::Runtime, workload::WorkloadKind};
use regex::Regex;
use serde_json::Value;

use crate::model::{FileInfo, Raw, Scan, cwd_of, dir_of, join, strip_dir};

static REQUIRE_MAIN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"require\.main\s*===?\s*module|import\.meta\.main").expect("static regex")
});
static SCRIPT_FILE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(\S+\.(?:m?js|cjs|ts))\b").expect("static regex"));

const SKIP_SCRIPTS: &[&str] = &[
    "test",
    "pretest",
    "posttest",
    "lint",
    "build",
    "prepare",
    "prepublish",
    "prepublishOnly",
    "format",
    "typecheck",
    "postinstall",
    "preinstall",
    "install",
    "clean",
    "prebuild",
    "postbuild",
    "coverage",
];
const SERVER_NAMES: &[&str] = &["dev", "start", "serve", "watch", "develop"];
const RUNNERS: &[&str] = &[
    "node",
    "tsx",
    "ts-node",
    "nodemon",
    "babel-node",
    "bun",
    "deno",
];

fn runner_command(pkg_text: &str, rel_in_pkg: &str) -> String {
    if pkg_text.contains("\"vitest\"") {
        format!("npx --no-install vitest run {rel_in_pkg}")
    } else if pkg_text.contains("\"jest\"") {
        format!("npx --no-install jest {rel_in_pkg}")
    } else if pkg_text.contains("\"mocha\"") {
        format!("npx --no-install mocha {rel_in_pkg}")
    } else {
        format!("node --test {rel_in_pkg}")
    }
}

pub fn discover(scan: &Scan) -> Vec<Raw> {
    let mut by_pkg: BTreeMap<String, Vec<&FileInfo>> = BTreeMap::new();
    for f in scan.files_of(Runtime::Node) {
        let dir = scan
            .nearest(&f.rel, &["package.json"])
            .map(|(d, _)| d)
            .unwrap_or_default();
        by_pkg.entry(dir).or_default().push(f);
    }
    // Packages that have a manifest but no source file still contribute scripts.
    for rel in scan
        .all
        .iter()
        .filter(|r| r.rsplit('/').next() == Some("package.json"))
    {
        by_pkg.entry(dir_of(rel).to_owned()).or_default();
    }
    let mut out = Vec::new();
    for (dir, files) in by_pkg {
        let manifest_rel = join(&dir, "package.json");
        let manifest_text = scan.read(&manifest_rel).unwrap_or_default();
        let manifest: Value = serde_json::from_str(&manifest_text).unwrap_or(Value::Null);
        let cwd = cwd_of(&dir);
        let mut used: BTreeSet<String> = BTreeSet::new();

        let tests: Vec<&FileInfo> = files.iter().copied().filter(|f| f.is_test).collect();
        for f in &tests {
            if f.rel.contains(".config.") {
                continue;
            }
            out.push(Raw {
                id: format!("node:test:{}", f.rel),
                kind: WorkloadKind::Test,
                runtime: Runtime::Node,
                command: runner_command(&manifest_text, strip_dir(&f.rel, &dir)),
                cwd: cwd.clone(),
                file: f.rel.clone(),
                srcs: vec![f.rel.clone()],
                extra: Default::default(),
            });
        }
        let scripts = manifest.get("scripts").and_then(Value::as_object);
        if tests.iter().all(|f| f.rel.contains(".config."))
            && scripts.is_some_and(|s| s.contains_key("test"))
        {
            out.push(Raw {
                id: format!("node:test:{manifest_rel}"),
                kind: WorkloadKind::Test,
                runtime: Runtime::Node,
                command: "npm test".into(),
                cwd: cwd.clone(),
                file: manifest_rel.clone(),
                srcs: Vec::new(),
                extra: Default::default(),
            });
        }
        if let Some(scripts) = scripts {
            for (name, value) in scripts {
                let Some(cmd) = value.as_str() else { continue };
                if SKIP_SCRIPTS.contains(&name.as_str())
                    || name.starts_with("pre")
                    || name.starts_with("post")
                {
                    continue;
                }
                let first = cmd.split_whitespace().next().unwrap_or("");
                let server_name = SERVER_NAMES.contains(&name.as_str());
                if !RUNNERS.contains(&first) && !server_name {
                    continue;
                }
                if cmd.contains("--test") || first == "tsc" {
                    continue;
                }
                let file = SCRIPT_FILE
                    .captures(cmd)
                    .map(|c| join(&dir, c[1].trim_start_matches("./")))
                    .filter(|p| scan.has(p));
                let server = server_name
                    || file
                        .as_ref()
                        .and_then(|p| scan.infos.get(p))
                        .is_some_and(|f| f.text.contains(".listen("));
                if let Some(p) = &file {
                    used.insert(p.clone());
                }
                out.push(Raw {
                    id: format!("node:script:{manifest_rel}:{name}"),
                    kind: if server {
                        WorkloadKind::DevServer
                    } else {
                        WorkloadKind::Main
                    },
                    runtime: Runtime::Node,
                    command: if name == "start" {
                        "npm start".into()
                    } else {
                        format!("npm run {name}")
                    },
                    cwd: cwd.clone(),
                    file: file.clone().unwrap_or_else(|| manifest_rel.clone()),
                    srcs: file.into_iter().collect(),
                    extra: Default::default(),
                });
            }
        }
        // `bin` entries.
        let bins: Vec<String> = match manifest.get("bin") {
            Some(Value::String(s)) => vec![s.clone()],
            Some(Value::Object(m)) => m
                .values()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect(),
            _ => Vec::new(),
        };
        for b in bins {
            let p = join(&dir, b.trim_start_matches("./"));
            if scan.has(&p) && used.insert(p.clone()) {
                out.push(Raw {
                    id: format!("node:main:{p}"),
                    kind: WorkloadKind::Main,
                    runtime: Runtime::Node,
                    command: format!("node {}", strip_dir(&p, &dir)),
                    cwd: cwd.clone(),
                    file: p.clone(),
                    srcs: vec![p],
                    extra: Default::default(),
                });
            }
        }
        // Runnable files that no script or bin entry names.
        for f in files.iter().filter(|f| !f.is_test) {
            if used.contains(&f.rel) || !REQUIRE_MAIN.is_match(&f.text) {
                continue;
            }
            out.push(Raw {
                id: format!("node:main:{}", f.rel),
                kind: WorkloadKind::Main,
                runtime: Runtime::Node,
                command: format!("node {}", strip_dir(&f.rel, &dir)),
                cwd: cwd.clone(),
                file: f.rel.clone(),
                srcs: vec![f.rel.clone()],
                extra: Default::default(),
            });
        }
    }
    out
}
