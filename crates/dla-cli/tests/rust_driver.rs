//! A Rust hook driver through the binary, on a small crate with a library target: the crate
//! features of its build, the target block that names a feature gate, and the coverage data
//! an instrumented process gives (`executed` / `not_executed` for the targets).
//!
//! The crate has two registries in `src/lib.rs` (`remember` and `note_other`, each pushing the
//! key it is given) and one in `src/extra.rs`, a module the crate declares behind the feature
//! `extras`. The tests build with the cargo on PATH and need no network (no dependencies).

use std::{fs, path::Path, process::Command as Proc};

use assert_cmd::Command;
use serde_json::Value;

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn project(dir: &Path) {
    write(
        dir,
        "Cargo.toml",
        "[package]\nname = \"calc\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nextras = []\n",
    );
    write(
        dir,
        "src/lib.rs",
        "use std::sync::Mutex;\n\nstatic CACHE: Mutex<Vec<String>> = Mutex::new(Vec::new());\nstatic SEEN: Mutex<Vec<String>> = Mutex::new(Vec::new());\n\npub fn remember(key: &str) {\n    CACHE.lock().unwrap().push(key.to_owned());\n}\n\npub fn note_other(key: &str) {\n    SEEN.lock().unwrap().push(key.to_owned());\n}\n\n#[cfg(feature = \"extras\")]\npub mod extra;\n",
    );
    write(
        dir,
        "src/extra.rs",
        "use std::sync::Mutex;\n\nstatic EXTRA: Mutex<Vec<String>> = Mutex::new(Vec::new());\n\npub fn keep(key: &str) {\n    EXTRA.lock().unwrap().push(key.to_owned());\n}\n",
    );
}

struct Out {
    code: i32,
    json: Value,
    text: String,
}

struct Run {
    dir: tempfile::TempDir,
    run_dir: std::path::PathBuf,
}

fn command(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
    ] {
        c.env_remove(var);
    }
    c
}

fn have_cargo() -> bool {
    let ok = Proc::new("cargo")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("skipping: cargo is not on PATH");
    }
    ok
}

/// True when the toolchain that builds the driver has `llvm-profdata` and `llvm-cov` of its own
/// LLVM (the `llvm-tools` component).
fn have_llvm_tools() -> bool {
    let sysroot = Proc::new("rustc")
        .args(["--print", "sysroot"])
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .unwrap_or_default();
    let host = Proc::new("rustc")
        .arg("-vV")
        .output()
        .ok()
        .and_then(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .find_map(|l| l.strip_prefix("host: ").map(str::to_owned))
        })
        .unwrap_or_default();
    let bin = Path::new(&sysroot)
        .join("lib/rustlib")
        .join(host)
        .join("bin");
    let ok = bin.join("llvm-profdata").is_file() && bin.join("llvm-cov").is_file();
    if !ok {
        eprintln!("skipping: the toolchain has no llvm-tools component");
    }
    ok
}

impl Run {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        project(dir.path());
        let o = command(dir.path())
            .env("DLA_CONSENT_TIERS", "1-4")
            .env("DLA_CONSENT_RAW_VALUES", "1")
            .args(["run", "init", "--session-id", "t", "--budget", "3h"])
            .output()
            .unwrap();
        let v: Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(v["ok"], true, "{v}");
        let run_dir = dir.path().join(".dla").join(v["run_id"].as_str().unwrap());
        let r = Self { dir, run_dir };
        for a in [
            &["census", "--lang", "rust"][..],
            &["workload", "discover", "--lang", "rust"][..],
        ] {
            r.ok(a);
        }
        r
    }

    fn run(&self, args: &[&str]) -> Out {
        let o = command(self.dir.path()).args(args).output().unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        Out {
            code: o.status.code().unwrap(),
            json: serde_json::from_str(&text).unwrap_or(Value::Null),
            text,
        }
    }

    fn ok(&self, args: &[&str]) -> Value {
        let o = self.run(args);
        assert_eq!(o.code, 0, "{args:?}: {}", o.text);
        o.json
    }

    fn id_at(&self, site: &str) -> String {
        let o = command(self.dir.path())
            .args(["ledger", "list", "--all"])
            .output()
            .unwrap();
        String::from_utf8(o.stdout)
            .unwrap()
            .lines()
            .skip(1)
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .find(|v| v["site"] == site)
            .unwrap_or_else(|| panic!("no finding at {site}"))["id"]
            .as_str()
            .unwrap()
            .to_owned()
    }

    fn driver(&self, h: &str) -> std::path::PathBuf {
        self.run_dir
            .join("harness")
            .join(h)
            .join("driver/dla_harness.rs")
    }

    /// Fills the hooks: an empty state and a unit that runs `body`.
    fn fill(&self, h: &str, body: &str) {
        let p = self.driver(h);
        let text = fs::read_to_string(&p).unwrap();
        let text = text
            .replace(
                "    todo!(\"DLA-HOOK setup has not been filled in\")",
                "    State {}",
            )
            .replace(
                "    let _ = (state, i, key);\n    todo!(\"DLA-HOOK unit has not been filled in\")",
                &format!("    let _ = (state, i);\n    {body}"),
            );
        fs::write(p, text).unwrap();
    }
}

fn verdict(trend: &Value, finding: &str) -> Value {
    trend["targets"]
        .as_array()
        .unwrap()
        .iter()
        .find(|t| t["finding"] == finding)
        .unwrap_or_else(|| panic!("no verdict for {finding}: {trend}"))
        .clone()
}

#[test]
fn a_driver_run_reads_executed_and_not_executed_from_an_instrumented_process() {
    if !have_cargo() || !have_llvm_tools() {
        return;
    }
    let r = Run::new();
    let (called, other) = (r.id_at("src/lib.rs:3"), r.id_at("src/lib.rs:4"));
    r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    // The unit reaches the first registry and never the second.
    r.fill("H1", "calc::remember(&key.to_string());");
    r.ok(&["harness", "run", "H1", "--mode", "both"]);
    let trend = r.ok(&["trend", "S1"]);
    assert_eq!(verdict(&trend, &called)["verdict"], "executed", "{trend}");
    assert_eq!(
        verdict(&trend, &other)["verdict"],
        "not_executed",
        "{trend}"
    );
    // The measured points are the uninstrumented build's; the coverage build is its own.
    let h = r.run_dir.join("harness/H1");
    assert!(h.join("cov-build").is_dir());
    assert!(h.join("cov/p1-fresh-cover.rust.json").is_file());
    let doc: Value =
        serde_json::from_str(&fs::read_to_string(h.join("executed-S1.json")).unwrap()).unwrap();
    assert_eq!(doc["kind"], "functions", "{doc}");
    assert!(
        doc["functions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["file"] == "src/lib.rs" && f["name"] == "calc::remember"),
        "{doc}"
    );
    assert!(
        !doc["functions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["name"] == "calc::note_other"),
        "{doc}"
    );
    // A target that was not executed keeps the queue open for it.
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["rust"]["open"], true, "{st}");
}

#[test]
fn a_gated_module_is_named_in_the_block_and_features_reach_the_build() {
    if !have_cargo() {
        return;
    }
    // Without the feature: the target says so in the block and the summary, and the build
    // has no feature argument.
    let r = Run::new();
    let gated = r.id_at("src/extra.rs:3");
    let out = r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    let note = out["summary"]["feature_note"].as_str().unwrap();
    assert!(
        note == format!("{gated} behind feature extras: generate with --features extras"),
        "{note}"
    );
    let block = fs::read_to_string(r.driver("H1")).unwrap();
    assert!(
        block.contains("//   behind feature extras: generate with --features extras"),
        "{block}"
    );
    let m: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H1/harness.json")).unwrap(),
    )
    .unwrap();
    let build = m["build"]["argv"][2].as_str().unwrap();
    assert!(!build.contains("--features"), "{build}");

    // With it: the build argument is recorded, and the block says the driver is built with it.
    let r = Run::new();
    let out = r.ok(&[
        "harness",
        "gen",
        "--lang",
        "rust",
        "--driver",
        "--next",
        "--features",
        "extras",
        "--no-default-features",
    ]);
    assert!(out["summary"].get("feature_note").is_none(), "{out}");
    assert_eq!(out["summary"]["features"], "extras", "{out}");
    let block = fs::read_to_string(r.driver("H1")).unwrap();
    assert!(
        block.contains("behind feature extras: the driver is built with --features extras"),
        "{block}"
    );
    let m: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H1/harness.json")).unwrap(),
    )
    .unwrap();
    let build = m["build"]["argv"][2].as_str().unwrap();
    assert!(
        build.ends_with(
            "cargo build --release --bin dla_harness --no-default-features --features extras"
        ),
        "{build}"
    );
    // The gated module builds into the driver: its function can be called from the unit, and
    // `harness run` builds with the stored argument, again on a rerun.
    r.fill(
        "H1",
        "calc::extra::keep(&key.to_string());\n    calc::remember(&key.to_string());",
    );
    r.ok(&["harness", "run", "H1", "--mode", "fresh"]);
    let o = r.run(&["harness", "run", "H1", "--mode", "fresh"]);
    assert_eq!(o.code, 0, "{}", o.text);
}

#[test]
fn features_are_for_a_rust_driver_only() {
    let r = Run::new();
    for args in [
        &["harness", "gen", "--lang", "rust", "--features", "extras"][..],
        &[
            "harness",
            "gen",
            "--lang",
            "rust",
            "--driver",
            "--features",
            "a b",
        ][..],
    ] {
        let o = r.run(args);
        assert_ne!(o.code, 0, "{}", o.text);
        assert_eq!(o.json["error"]["code"], "invalid-value", "{}", o.text);
    }
}
