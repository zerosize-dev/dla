//! Rust: the RAPx leak checker (`cargo +nightly rapx check -m`), parsed from stderr.

use std::{ffi::OsString, fs, path::Path};

use dla_core::static_hit::StaticHit;
use regex::Regex;
use serde_json::json;

use super::{Job, Outcome, Ran, dedupe, hit, run_record, version_of};
use crate::{
    exec::{self, Cmd},
    fsx,
};

/// The rule id RAPx's leak checker reports under.
pub const RULE: &str = "leak";

/// Parses RAPx's warnings. `crate_dir` is the crate's directory relative to the
/// project (empty for the root crate); locations are relative to the crate.
pub fn parse(text: &str, crate_dir: &str) -> Vec<StaticHit> {
    let ansi = Regex::new("\x1b\\[[0-9;]*[A-Za-z]").expect("static regex");
    let func = Regex::new(r"Memory Leak detected in function (\S+)").expect("static regex");
    let loc = Regex::new(r"^-->\s*(?P<file>.+?):(?P<line>\d+):\d+\s*$").expect("static regex");
    let mut hits = Vec::new();
    let mut last_fn = String::new();
    let mut awaiting = false;
    for raw in text.lines() {
        let line = ansi.replace_all(raw, "");
        let t = line.trim();
        if let Some(c) = func.captures(t) {
            last_fn = c[1].to_owned();
        }
        if t.contains("warning: Memory Leak detected") {
            awaiting = true;
            continue;
        }
        if awaiting && let Some(c) = loc.captures(t) {
            let file = c["file"].replace('\\', "/");
            let file = if crate_dir.is_empty() {
                file
            } else {
                format!("{}/{file}", crate_dir.trim_end_matches('/'))
            };
            let msg = if last_fn.is_empty() {
                "Memory Leak detected".to_owned()
            } else {
                format!("Memory Leak detected in function {last_fn}")
            };
            hits.push(hit(
                "rapx",
                RULE,
                &file,
                c["line"].parse().unwrap_or(1),
                &msg,
            ));
            awaiting = false;
        }
    }
    hits
}

/// Directories (project-relative, `""` is the root) that hold a `Cargo.toml` and are
/// not inside another such directory.
pub fn crate_dirs(project: &Path) -> Vec<String> {
    let manifests = fsx::walk(project, 10_000, &|r| r.ends_with("Cargo.toml"));
    let mut dirs: Vec<String> = manifests
        .iter()
        .filter(|m| *m == "Cargo.toml" || m.ends_with("/Cargo.toml"))
        .map(|m| {
            m.strip_suffix("Cargo.toml")
                .unwrap_or("")
                .trim_end_matches('/')
                .to_owned()
        })
        .collect();
    dirs.sort();
    if dirs.first().is_some_and(String::is_empty) {
        return vec![String::new()];
    }
    let mut out: Vec<String> = Vec::new();
    for d in dirs {
        if !out.iter().any(|o| d.starts_with(&format!("{o}/"))) {
            out.push(d);
        }
    }
    out.truncate(8);
    out
}

/// Runs RAPx on a copy of each crate (cargo writes `Cargo.lock`).
pub fn run(job: &Job) -> Outcome {
    let Some(rapx) = job.env.find_program("cargo-rapx") else {
        return Outcome::missing("cargo-rapx is not on PATH or in the tools directory");
    };
    let Some(cargo) = job.env.find_program("cargo") else {
        return Outcome::missing("cargo is not on PATH");
    };
    // `cargo check` executes the project's build scripts and procedural macros, so it
    // counts as building the project: tier 1 consent.
    if let Err(r) = job
        .ctx
        .require_tier(1, "rapx (cargo check runs build scripts)")
    {
        return Outcome::blocked(format!("{} (fix: {})", r.message, r.fix), vec![]);
    }
    let base = |c: Cmd| c.env_remove("RUSTC_WRAPPER");
    let version = version_of(
        base(Cmd::new(&cargo).args(["+nightly", "rapx", "--version"]))
            .env("PATH", path_with(&rapx, job)),
    );
    let crates: Vec<String> = crate_dirs(job.project())
        .into_iter()
        .filter(|c| job.may_contain(c))
        .collect();
    if crates.is_empty() {
        return Outcome::Ran(Box::new(Ran {
            version,
            note: Some("not applicable: no Cargo.toml".to_owned()),
            raw: json!({ "runs": [] }),
            ..Ran::default()
        }));
    }
    let copy = job.scratch.join("project");
    let _ = fs::remove_dir_all(&copy);
    if let Err(e) = fsx::copy_tree(job.project(), &copy) {
        return Outcome::blocked(format!("cannot copy the project: {e}"), vec![]);
    }
    let mut hits = Vec::new();
    let mut runs = Vec::new();
    let mut first_cmd = Vec::new();
    let mut failures = Vec::new();
    let mut exit = None;
    for (i, dir) in crates.iter().enumerate() {
        let cwd = if dir.is_empty() {
            copy.clone()
        } else {
            copy.join(dir)
        };
        let cmd = base(Cmd::new(&cargo))
            .cwd(&cwd)
            .args(["+nightly", "rapx", "check", "-m"])
            .env("PATH", path_with(&rapx, job))
            .env("RAPX_LOG", "warn")
            .env("CARGO_NET_OFFLINE", "true")
            .env(
                "CARGO_TARGET_DIR",
                job.scratch.join(format!("target-{i}")).into_os_string(),
            )
            .timeout(job.timeout);
        if first_cmd.is_empty() {
            first_cmd = cmd.display();
        }
        let out = exec::run(&cmd);
        runs.push(run_record(&cmd, &out));
        if let Some(r) = out.failure_reason() {
            failures.push(format!("{}: {r}", display_dir(dir)));
            continue;
        }
        exit = out.exit;
        let text = format!("{}\n{}", out.stderr, out.stdout);
        let found = parse(&text, dir);
        if out.exit != Some(0) && found.is_empty() {
            failures.push(format!(
                "{}: exit {:?}: {}",
                display_dir(dir),
                out.exit,
                out.tail(3)
            ));
            continue;
        }
        hits.extend(found);
    }
    if failures.len() == crates.len() {
        return Outcome::blocked(
            format!("rapx failed on every crate: {}", failures.join("; ")),
            runs,
        );
    }
    let limitations = failures
        .into_iter()
        .map(|f| format!("rapx failed on {f}"))
        .collect();
    Outcome::Ran(Box::new(Ran {
        version,
        command: first_cmd,
        exit,
        hits: dedupe(hits),
        load_errors: 0,
        limitations,
        note: None,
        raw: json!({ "runs": runs, "crates": crates }),
    }))
}

fn display_dir(d: &str) -> &str {
    if d.is_empty() { "." } else { d }
}

/// `PATH` with the directory of `cargo-rapx` first, so `cargo rapx` finds it.
fn path_with(rapx: &Path, job: &Job) -> OsString {
    let mut dirs = Vec::new();
    if let Some(d) = rapx.parent() {
        dirs.push(d.to_path_buf());
    }
    dirs.extend(job.env.path.iter().cloned());
    std::env::join_paths(dirs).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rapx_golden() {
        let text =
            fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/rapx.txt"))
                .unwrap();
        let hits = parse(&text, "");
        assert_eq!(hits.len(), 1, "{hits:?}");
        assert_eq!(hits[0].rule, "leak");
        assert_eq!(hits[0].site(), "src/lib.rs:6");
        assert_eq!(hits[0].message, "Memory Leak detected in function b");
        let nested = parse(&text, "crates/x/");
        assert_eq!(nested[0].site(), "crates/x/src/lib.rs:6");
    }

    #[test]
    fn ansi_colours_and_unrelated_arrows_are_ignored() {
        let t = "\x1b[33mwarning: unused\x1b[0m\n --> src/a.rs:1:1\nWARN: Memory Leak detected in function f\n\x1b[1mwarning: Memory Leak detected.\x1b[0m\n --> src/a.rs:9:5\n";
        let h = parse(t, "");
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].site(), "src/a.rs:9");
    }

    #[test]
    fn crate_dirs_prefers_the_root_manifest() {
        let d = tempfile::tempdir().unwrap();
        for p in ["Cargo.toml", "sub/Cargo.toml"] {
            let f = d.path().join(p);
            fs::create_dir_all(f.parent().unwrap()).unwrap();
            fs::write(f, "").unwrap();
        }
        assert_eq!(crate_dirs(d.path()), vec![String::new()]);
        fs::remove_file(d.path().join("Cargo.toml")).unwrap();
        fs::create_dir_all(d.path().join("sub/inner")).unwrap();
        fs::write(d.path().join("sub/inner/Cargo.toml"), "").unwrap();
        assert_eq!(crate_dirs(d.path()), vec!["sub".to_owned()]);
    }
}
