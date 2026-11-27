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
        Self::of(project)
    }

    fn of(make: fn(&Path)) -> Self {
        Self::build(make, true)
    }

    fn build(make: fn(&Path), census: bool) -> Self {
        let dir = tempfile::tempdir().unwrap();
        make(dir.path());
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
        if census {
            r.ok(&["census", "--lang", "rust"]);
        }
        r.ok(&["workload", "discover", "--lang", "rust"]);
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

/// A crate whose `leak_one` keeps a boxed value per call in a static and whose `no_leak` frees
/// what it allocates; `forget_one` forgets a box through `mem::forget` where no holder is declared.
fn leaky(dir: &Path) {
    write(
        dir,
        "Cargo.toml",
        "[package]\nname = \"leaky\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        dir,
        "src/lib.rs",
        "use std::sync::Mutex;\n\nstatic KEPT: Mutex<Vec<Box<[u8; 64]>>> = Mutex::new(Vec::new());\n\npub fn leak_one(key: u64) {\n    let b = Box::new([key as u8; 64]);\n    KEPT.lock().unwrap().push(b);\n}\n\npub fn no_leak(key: u64) -> u64 {\n    let v: Vec<u8> = vec![key as u8; 64];\n    v.iter().map(|x| u64::from(*x)).sum()\n}\n\npub fn forget_one(key: u64) {\n    std::mem::forget(std::hint::black_box(Box::new([key as u8; 48])));\n}\n",
    );
}

fn have_alloc_tools() -> bool {
    let ok = have_cargo()
        && cfg!(target_os = "linux")
        && ["cc", "addr2line"].iter().all(|t| {
            Proc::new(t)
                .arg("--version")
                .output()
                .is_ok_and(|o| o.status.success())
        });
    if !ok {
        eprintln!("skipping: cc or addr2line is missing");
    }
    ok
}

/// The `dla summarize alloc-stack-diff` command a capturing run printed, as arguments.
fn stack_diff_args(run_out: &Value) -> Vec<String> {
    let cmd = run_out["summary"]["summarize"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .find(|c| c.starts_with("dla summarize alloc-stack-diff"))
        .unwrap_or_else(|| panic!("no alloc-stack-diff command in {run_out}"))
        .to_owned();
    cmd.split_whitespace().skip(1).map(str::to_owned).collect()
}

#[test]
fn capture_alloc_names_the_leaking_function_hints_the_finding_and_an_annotate_ties_it() {
    if !have_alloc_tools() {
        return;
    }
    let r = Run::of(leaky);
    let kept = r.id_at("src/lib.rs:3");
    r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    assert!(r.run_dir.join("harness/H1/agent/libdlashim.so").is_file());
    r.fill(
        "H1",
        "leaky::leak_one(key);\n    let _ = leaky::no_leak(key);",
    );
    // Without --capture the trend of the whole-process column proposes the capture run.
    r.ok(&["harness", "run", "H1", "--mode", "fresh"]);
    let trend = r.ok(&["trend", "S1"]);
    assert!(
        trend["next"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .any(|n| n.starts_with("dla harness run H1 --mode fresh --capture alloc")),
        "{trend}"
    );
    let out = r.ok(&[
        "harness",
        "run",
        "H1",
        "--mode",
        "fresh",
        "--capture",
        "alloc",
    ]);
    // The series of the capture run itself does not propose the capture again.
    let again = r.ok(&["trend", out["summary"]["series"][0].as_str().unwrap()]);
    assert!(
        !again["next"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .any(|n| n.contains("--capture alloc")),
        "{again}"
    );
    let args = stack_diff_args(&out);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let mut cmd: Vec<&str> = Vec::new();
    cmd.extend(args);
    let sum = r.ok(&cmd);
    assert_eq!(sum["gate"], "pass", "{sum}");
    let passed: Vec<&str> = sum["passed"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        passed
            .iter()
            .any(|p| p.starts_with("stack:leak_one (src/lib.rs:")),
        "{sum}"
    );
    assert!(!passed.iter().any(|p| p.contains("no_leak")), "{sum}");
    // The first project frame, not the driver's counting allocator or std.
    assert!(
        sum["top"][0]["at"]
            .as_str()
            .unwrap()
            .starts_with("leak_one (src/lib.rs:"),
        "{sum}"
    );
    // The box is allocated at one line and the writer of the static is at another line of the
    // function: that stack is not tied, and the finding is named with the command that would tie
    // it. The `Vec` that keeps the boxes is one block that is resized, allocated at the line of
    // the push (the writer): it passes as one block whose bytes rise at every point and ties
    // (before the constant-count rule it did not pass, and nothing tied until the annotation).
    assert_eq!(sum["tied"], serde_json::json!([kept]), "{sum}");
    assert!(
        passed
            .iter()
            .any(|p| p.starts_with("stack:leak_one (src/lib.rs:7)")),
        "{sum}"
    );
    assert_eq!(sum["same_function"], serde_json::json!([kept]), "{sum}");
    assert!(!sum.to_string().contains("no_leak"), "{sum}");
    let next = sum["next"][0].as_str().unwrap();
    let command = next.split("   #").next().unwrap();
    assert!(
        command.starts_with(&format!("dla ledger annotate {kept} --writer src/lib.rs:")),
        "{next}"
    );
    assert!(next.contains("only if the block allocated there"), "{next}");
    // Running the printed command and the summarize again ties it, by the site rule.
    let words: Vec<&str> = command.split_whitespace().skip(1).collect();
    r.ok(&words);
    let again = r.ok(&cmd);
    assert_eq!(again["tied"], serde_json::json!([kept]), "{again}");
    assert!(again.get("same_function").is_none(), "{again}");
}

/// A crate whose `submit` pushes onto a static `Vec<u64>` that nothing drains: one heap block
/// whose capacity doubles, no block per push; `batch` fills a second static `Vec<u64>` and clears
/// it with its capacity kept (a bounded buffer).
fn one_buffer(dir: &Path) {
    write(
        dir,
        "Cargo.toml",
        "[package]\nname = \"vecq\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        dir,
        "src/lib.rs",
        "use std::sync::Mutex;\n\nstatic PENDING: Mutex<Vec<u64>> = Mutex::new(Vec::new());\n\npub fn submit(job: u64) {\n    PENDING.lock().unwrap().push(job);\n}\n",
    );
}

fn bounded_buffer(dir: &Path) {
    write(
        dir,
        "Cargo.toml",
        "[package]\nname = \"vecq\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        dir,
        "src/lib.rs",
        "use std::sync::Mutex;\n\nstatic BATCH: Mutex<Vec<u64>> = Mutex::new(Vec::new());\n\npub fn flush(job: u64) -> usize {\n    let mut b = BATCH.lock().unwrap();\n    for j in 0..1000u64 {\n        b.push(job + j);\n    }\n    let n = b.len();\n    b.clear();\n    n\n}\n",
    );
}

impl Run {
    /// The items of the final gate, from the full file.
    fn final_items(&self) -> Vec<Value> {
        // The exit code says whether anything is missing; the items are in the file.
        self.run(&["gate", "check", "--final"]);
        let full: Value = serde_json::from_str(
            &fs::read_to_string(self.run_dir.join("gate/final.json")).unwrap(),
        )
        .unwrap();
        full["missing"].as_array().unwrap().clone()
    }
}

#[test]
fn a_vec_field_that_grows_as_one_buffer_ties_leads_and_is_not_closed_as_no_growth() {
    if !have_alloc_tools() {
        return;
    }
    let r = Run::of(one_buffer);
    let f = r.id_at("src/lib.rs:3");
    r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    // 2,000 pushes per unit: the one block of the field passes 8 MB at the last point.
    r.fill(
        "H1",
        "for j in 0..2000u64 {\n        vecq::submit(key + j);\n    }",
    );
    let out = r.ok(&[
        "harness",
        "run",
        "H1",
        "--mode",
        "fresh",
        "--capture",
        "alloc",
    ]);
    // The points are 1, 2, 4 and 8 steps of work, each a process of its own.
    let xs: Vec<f64> = (1..=4)
        .map(|k| {
            let text = fs::read_to_string(
                r.run_dir
                    .join(format!("harness/H1/out/p{k}-alloc/shim-dump.jsonl")),
            )
            .unwrap();
            serde_json::from_str::<Value>(text.lines().next().unwrap()).unwrap()["x"]
                .as_f64()
                .unwrap()
        })
        .collect();
    assert!(xs.windows(2).all(|w| w[1] >= 2.0 * w[0]), "{xs:?}");
    let args = stack_diff_args(&out);
    let cmd: Vec<&str> = args.iter().map(String::as_str).collect();
    let sum = r.ok(&cmd);
    // One block whose bytes rise at every point passes, at the line of the push, and ties.
    assert_eq!(sum["gate"], "pass", "{sum}");
    assert_eq!(sum["tied"], serde_json::json!([f]), "{sum}");
    assert!(
        sum["passed"][0]
            .as_str()
            .unwrap()
            .starts_with("stack:submit (src/lib.rs:6)"),
        "{sum}"
    );
    // The finding has the lead: the final gate lists it with the alloc-test route, and closing
    // it as no growth is refused (by `resolve` and by `close-rest`).
    let item = r
        .final_items()
        .into_iter()
        .find(|m| m["kind"] == "finding" && m["id"] == f.as_str())
        .unwrap_or_else(|| panic!("no item for {f}"));
    assert!(
        item["fix"]
            .as_str()
            .unwrap()
            .contains(&format!("dla alloc-test H1 --finding {f}")),
        "{item}"
    );
    let o = r.run(&[
        "ledger",
        "resolve",
        &f,
        "--as",
        "unresolved",
        "--reason",
        "inconclusive",
        "--remedy",
        "x",
    ]);
    assert_ne!(o.code, 0, "{}", o.text);
    assert_eq!(o.json["error"]["code"], "growth-not-measured", "{}", o.text);
    let rest = r.ok(&["ledger", "close-rest"]);
    assert_eq!(rest["closed"], 0, "{rest}");
    assert!(
        rest["left"][0]["why"]
            .as_str()
            .unwrap()
            .contains("grew over every dump"),
        "{rest}"
    );
}

#[test]
fn a_buffer_cleared_with_its_capacity_kept_does_not_pass_the_summary() {
    if !have_alloc_tools() {
        return;
    }
    let r = Run::of(bounded_buffer);
    let f = r.id_at("src/lib.rs:3");
    r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    r.fill("H1", "let _ = vecq::flush(key);");
    let out = r.ok(&[
        "harness",
        "run",
        "H1",
        "--mode",
        "fresh",
        "--capture",
        "alloc",
    ]);
    let args = stack_diff_args(&out);
    let cmd: Vec<&str> = args.iter().map(String::as_str).collect();
    let sum = r.ok(&cmd);
    // The capacity is reached in the first unit and kept: the bytes of the one block are the
    // same at every point, so nothing passes and nothing ties.
    assert_eq!(sum["monotone"], 0, "{sum}");
    assert_eq!(sum["gate"], "fail", "{sum}");
    assert!(
        sum["reasons"].to_string().contains("do not increase"),
        "{sum}"
    );
    assert_eq!(sum["tied"], serde_json::json!([]), "{sum}");
    assert!(sum["passed"].as_array().is_none_or(Vec::is_empty), "{sum}");
    let lead = r.final_items().into_iter().any(|m| {
        m["id"] == f.as_str()
            && m["fix"]
                .as_str()
                .is_some_and(|x| x.contains(&format!("dla alloc-test H1 --finding {f}")))
    });
    assert!(!lead, "the decoy has no lead");
}

#[test]
fn a_finding_added_from_an_untied_allocation_stack_takes_its_site_from_the_tool() {
    if !have_alloc_tools() {
        return;
    }
    // No census: no finding names any function of the crate.
    let r = Run::build(leaky, false);
    r.ok(&["harness", "gen", "--lang", "rust", "--driver"]);
    // Only `forget_one` runs: the stack is in a function no finding names.
    r.fill("H1", "leaky::forget_one(key);");
    let out = r.ok(&[
        "harness",
        "run",
        "H1",
        "--mode",
        "fresh",
        "--capture",
        "alloc",
    ]);
    let args = stack_diff_args(&out);
    let mut cmd: Vec<&str> = Vec::new();
    cmd.extend(args.iter().map(String::as_str));
    let sum = r.ok(&cmd);
    assert_eq!(sum["gate"], "pass", "{sum}");
    assert_eq!(sum["tied"], serde_json::json!([]), "{sum}");
    let site = sum["top"][0]["site"].as_str().unwrap().to_owned();
    assert!(site.starts_with("src/lib.rs:"), "{sum}");
    let evidence = sum["growth_evidence"].as_str().unwrap();
    let added = r.ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        evidence,
    ]);
    let id = added["id"].as_str().unwrap();
    assert_eq!(r.id_at(&site), id, "{added}");
}

#[test]
fn a_capture_run_without_the_shim_records_it_wrote_no_stacks_and_one_with_it_does_not() {
    if !have_alloc_tools() {
        return;
    }
    let r = Run::of(leaky);
    r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    r.fill("H1", "leaky::leak_one(key);");
    let shim = r.run_dir.join("harness/H1/agent/libdlashim.so");
    assert!(shim.is_file());
    // The shim is there: the run wrote stacks, so nothing releases the harness from a summary.
    r.ok(&[
        "harness",
        "run",
        "H1",
        "--mode",
        "fresh",
        "--capture",
        "alloc",
    ]);
    assert!(!dla_core::gate::capture_wrote_no_stacks(&r.run_dir, "H1"));
    // The shim is gone: the run says it could not write stacks, in the line the gate reads.
    fs::remove_file(&shim).unwrap();
    for k in 1..=4 {
        fs::remove_dir_all(r.run_dir.join(format!("harness/H1/out/p{k}-alloc"))).unwrap();
    }
    let out = r.ok(&[
        "harness",
        "run",
        "H1",
        "--mode",
        "fresh",
        "--capture",
        "alloc",
    ]);
    assert!(
        out["summary"]["summarize"]
            .as_array()
            .is_none_or(|c| c.iter().all(|c| !c
                .as_str()
                .unwrap()
                .starts_with("dla summarize alloc-stack-diff"))),
        "{out}"
    );
    assert!(dla_core::gate::capture_wrote_no_stacks(&r.run_dir, "H1"));
    let text = fs::read_to_string(r.run_dir.join("harness/H1/captures.json")).unwrap();
    assert!(text.contains("needs the allocator shim"), "{text}");
}

#[test]
fn a_heap_residual_of_a_rust_driver_is_not_left_unresolved_before_its_stacks_are_summarized() {
    if !have_alloc_tools() {
        return;
    }
    let r = Run::of(leaky);
    r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    // 3,000 kept boxes per unit: far over the growth floor of a residual that demands a capture.
    r.fill(
        "H1",
        "for j in 0..3000u64 {\n        leaky::leak_one(key + j);\n    }",
    );
    r.ok(&["harness", "run", "H1", "--mode", "fresh"]);
    let want = "dla harness run H1 --mode fresh --capture alloc, then the `dla summarize` commands it prints";
    let trend = r.ok(&["trend", "S1"]);
    let next = trend["next"].as_array().unwrap();
    assert_eq!(next[0], want, "{trend}");
    assert_eq!(
        next.iter()
            .filter(|n| n.as_str().is_some_and(|n| n.contains("--capture alloc")))
            .count(),
        1,
        "{trend}"
    );
    assert!(
        trend["_attention"][0]
            .as_str()
            .unwrap()
            .contains("capture H1 first")
    );
    let leave = [
        "ledger",
        "resolve",
        "O1",
        "--as",
        "unresolved",
        "--reason",
        "inconclusive",
        "--remedy",
        "a guess",
    ];
    let o = r.run(&leave);
    assert_ne!(o.code, 0, "{}", o.text);
    assert_eq!(
        o.json["error"]["code"], "capture-not-summarized",
        "{}",
        o.text
    );
    assert_eq!(o.json["error"]["fix"], want, "{}", o.text);
    let o = r.run(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    assert_eq!(o.json["error"]["code"], "capture-first", "{}", o.text);
    assert_eq!(o.json["error"]["fix"], want, "{}", o.text);
    // The capture is taken: the next entry is its summary, not the capture again, and the item
    // is still not to be left.
    let out = r.ok(&[
        "harness",
        "run",
        "H1",
        "--mode",
        "fresh",
        "--capture",
        "alloc",
    ]);
    let again = r.ok(&["trend", out["summary"]["series"][0].as_str().unwrap()]);
    assert!(
        again["next"][0]
            .as_str()
            .unwrap()
            .starts_with("dla summarize alloc-stack-diff"),
        "{again}"
    );
    assert_eq!(
        r.run(&leave).json["error"]["code"],
        "capture-not-summarized"
    );
    let args = stack_diff_args(&out);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    r.ok(&args);
    let o = r.run(&leave);
    assert_eq!(o.code, 0, "{}", o.text);
}

fn ledger_sync(dir: &Path) {
    fn copy(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for e in fs::read_dir(from).unwrap().flatten() {
            let (src, dst) = (e.path(), to.join(e.file_name()));
            if e.file_type().unwrap().is_dir() {
                copy(&src, &dst);
            } else {
                fs::copy(&src, &dst).unwrap();
            }
        }
    }
    copy(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../bench/fixtures/rust-ledger-sync/project"),
        dir,
    );
}

#[test]
fn a_command_mode_rust_run_with_capture_alloc_leaves_stacks_and_the_command() {
    if !have_alloc_tools() {
        return;
    }
    let r = Run::of(ledger_sync);
    r.ok(&["harness", "gen", "--lang", "rust"]);
    let out = r.ok(&["harness", "run", "H1", "--capture", "alloc"]);
    // The first command printed reads the allocation stacks.
    let first = out["summary"]["summarize"][0].as_str().unwrap();
    assert!(first.starts_with("dla summarize alloc-stack-diff"), "{out}");
    let args = stack_diff_args(&out);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let sum = r.ok(&args);
    assert_eq!(sum["gate"], "pass", "{sum}");
    assert!(
        sum["passed"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .any(|p| p.starts_with("stack:add (src/journal.rs:")),
        "{sum}"
    );
    assert!(!sum["tied"].as_array().unwrap().is_empty(), "{sum}");
}

/// Two holders written from one function that also allocates the growing block.
fn pair(dir: &Path) {
    write(
        dir,
        "Cargo.toml",
        "[package]\nname = \"pairy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        dir,
        "src/lib.rs",
        "use std::sync::Mutex;\n\nstatic KEPT_A: Mutex<Vec<Box<[u8; 64]>>> = Mutex::new(Vec::new());\nstatic KEPT_B: Mutex<Vec<u64>> = Mutex::new(Vec::new());\n\npub fn leak_pair(key: u64) {\n    let b = Box::new([key as u8; 64]);\n    KEPT_A.lock().unwrap().push(b);\n    KEPT_B.lock().unwrap().push(key);\n}\n",
    );
}

#[test]
fn two_findings_in_the_function_of_a_stack_are_both_named_and_no_single_command_is_preferred() {
    if !have_alloc_tools() {
        return;
    }
    let r = Run::of(pair);
    let (a, b) = (r.id_at("src/lib.rs:3"), r.id_at("src/lib.rs:4"));
    r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    r.fill("H1", "pairy::leak_pair(key);");
    let out = r.ok(&[
        "harness",
        "run",
        "H1",
        "--mode",
        "fresh",
        "--capture",
        "alloc",
    ]);
    let args = stack_diff_args(&out);
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    let sum = r.ok(&args);
    assert_eq!(sum["gate"], "pass", "{sum}");
    // The `Vec` of a static is one block that is resized, allocated at the line of its push (a
    // recorded writer of that static): it ties to that finding alone, whichever of the two
    // buffers shows a rise at every point in the run (glibc moves or extends a large block
    // either way). The stack of the box (the other line of the function) ties to none, and is
    // the one both findings are named for.
    let tied = sum["tied"].as_array().unwrap();
    assert_eq!(tied.len(), 1, "{sum}");
    assert!(tied[0] == a.as_str() || tied[0] == b.as_str(), "{sum}");
    let mut named: Vec<&str> = sum["same_function"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    named.sort_unstable();
    let mut want = vec![a.as_str(), b.as_str()];
    want.sort_unstable();
    assert_eq!(named, want, "{sum}");
    let next = sum["next"][0].as_str().unwrap();
    assert!(next.contains(&a) && next.contains(&b), "{next}");
    assert!(!next.starts_with("dla ledger annotate"), "{next}");
}

/// A crate whose public `record` fills a static registry through a function that exists only under
/// `#[cfg(feature = "enable")]`, called from a gated statement, the way a tracing library compiles to a no-op without its feature. No
/// `mod` is gated and no feature is on by default.
fn gated(dir: &Path) {
    write(
        dir,
        "Cargo.toml",
        "[package]\nname = \"spanner\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nenable = []\n",
    );
    write(
        dir,
        "src/lib.rs",
        "use std::sync::Mutex;\n\nstatic REGISTRY: Mutex<Vec<String>> = Mutex::new(Vec::new());\n\n#[cfg(feature = \"enable\")]\nfn collect(key: &str) {\n    REGISTRY.lock().unwrap().push(key.to_owned());\n}\n\npub fn record(key: &str) {\n    #[cfg(feature = \"enable\")]\n    collect(key);\n    #[cfg(not(feature = \"enable\"))]\n    let _ = key;\n}\n",
    );
}

const GATED_COMMAND: &str = "dla harness gen --lang rust --driver --features enable --probe";

#[test]
fn a_feature_gating_code_inside_functions_is_said_in_the_block_and_the_attention() {
    if !have_cargo() {
        return;
    }
    let r = Run::of(gated);
    let target = r.id_at("src/lib.rs:3");
    let out = r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    // The first key of the output, and whole: the output bound never cuts it.
    let attention = out["_attention"].as_array().unwrap();
    assert_eq!(attention.len(), 1, "{out}");
    let line = attention[0].as_str().unwrap();
    assert_eq!(
        line,
        "feature enable is off by default: 1 target has code behind it (gated sites in the crate: 3); run the command under regenerate"
    );
    assert_eq!(
        out["regenerate"],
        format!("{GATED_COMMAND} {target}"),
        "{out}"
    );
    assert!(
        out.as_object().unwrap().keys().next().map(String::as_str) == Some("_attention"),
        "{out}"
    );
    let block = fs::read_to_string(r.driver("H1")).unwrap();
    assert!(
        block.contains(&format!(
            "//   feature enable (3 gated sites in src/lib.rs, 3 in the crate) off by default and the harness is built without it, so the code behind it does not run in it: regenerate it with {GATED_COMMAND} {target}"
        )),
        "{block}"
    );
    let features: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H1/features.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        features["gaps"][&target][0]["feature"], "enable",
        "{features}"
    );

    // With the feature on, the output and the block say nothing about features and nothing is
    // recorded; the build has the argument.
    let r = Run::of(gated);
    let out = r.ok(&[
        "harness",
        "gen",
        "--lang",
        "rust",
        "--driver",
        "--next",
        "--features",
        "enable",
    ]);
    assert!(out.get("_attention").is_none(), "{out}");
    assert!(!out.to_string().contains("feature enable"), "{out}");
    let block = fs::read_to_string(r.driver("H1")).unwrap();
    assert!(!block.contains("feature enable"), "{block}");
    assert!(!r.run_dir.join("harness/H1/features.json").exists());
    let m: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H1/harness.json")).unwrap(),
    )
    .unwrap();
    assert!(
        m["build"]["argv"][2]
            .as_str()
            .unwrap()
            .ends_with("--features enable")
    );
}

#[test]
fn a_target_that_did_not_run_without_its_feature_is_told_the_feature_first_and_stays_open() {
    if !have_cargo() || !have_llvm_tools() {
        return;
    }
    let r = Run::of(gated);
    let target = r.id_at("src/lib.rs:3");
    r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    r.fill("H1", "spanner::record(&key.to_string());");
    r.ok(&["harness", "run", "H1", "--mode", "both"]);
    // The call ran, the feature was off: the code behind it did not.
    let trend = r.ok(&["trend", "S1"]);
    let v = verdict(&trend, &target);
    assert_eq!(v["verdict"], "not_executed", "{trend}");
    let detail = v["detail"].as_str().unwrap();
    assert!(
        detail.starts_with(
            "feature enable (3 gated sites in src/lib.rs, 3 in the crate) off by default"
        ),
        "{detail}"
    );
    assert!(
        detail.contains(&format!("{GATED_COMMAND} {target}. Then: function ")),
        "{detail}"
    );

    // A focus harness with another driver and still no feature: the queue does not settle it as
    // not reached, and refuses to close it, with the feature first.
    let out = r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    assert!(out["summary"]["focus"].is_string(), "{out}");
    r.fill("H2", "spanner::record(&format!(\"k{key}\"));");
    // A run in both modes writes two series: S3 and S4 are the ones of H2.
    r.ok(&["harness", "run", "H2", "--mode", "both"]);
    let trend = r.ok(&["trend", "S3"]);
    assert_eq!(
        verdict(&trend, &target)["verdict"],
        "not_executed",
        "{trend}"
    );
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["rust"]["open"], true, "{st}");
    assert_eq!(st["queue"]["rust"]["pending"], 1, "{st}");
    let o = r.run(&[
        "ledger",
        "resolve",
        &target,
        "--as",
        "suspected_static_only",
        "--reason",
        "not_exercised",
        "--remedy",
        "x",
    ]);
    assert_eq!(o.json["error"]["code"], "target-pending", "{}", o.text);
    let msg = o.json["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("not_executed feature enable (3 gated sites"),
        "{msg}"
    );
    // `close-rest` leaves it, with the same feature first.
    let cr = r.run(&["ledger", "close-rest"]);
    assert!(
        cr.text.contains(
            "pending in the rust queue: feature enable (3 gated sites in src/lib.rs, 3 in the crate) off by default"
        ),
        "{}",
        cr.text
    );

    // With the feature on in a third harness: the target is reached, nothing is said about
    // features, and the queue closes it.
    let out = r.ok(&[
        "harness",
        "gen",
        "--lang",
        "rust",
        "--driver",
        "--probe",
        &target,
        "--features",
        "enable",
    ]);
    assert!(out.get("_attention").is_none(), "{out}");
    r.fill("H3", "spanner::record(&format!(\"j{key}\"));");
    r.ok(&["harness", "run", "H3", "--mode", "both"]);
    let trend = r.ok(&["trend", "S5"]);
    let v = verdict(&trend, &target);
    assert_eq!(v["verdict"], "executed", "{trend}");
    assert!(
        !v["detail"].as_str().unwrap().contains("feature"),
        "{trend}"
    );
    let st = r.ok(&["run", "status"]);
    assert_eq!(st["queue"]["rust"]["pending"], 0, "{st}");
}

/// A workspace with no package at its root and two member library crates; the feature of `libm`
/// is off by default and gates the function that fills its registry.
fn workspace(dir: &Path) {
    write(
        dir,
        "Cargo.toml",
        "[workspace]\nmembers = [\"libm\", \"tool\"]\nresolver = \"2\"\n",
    );
    write(
        dir,
        "libm/Cargo.toml",
        "[package]\nname = \"libm\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[features]\nenable = []\n",
    );
    write(
        dir,
        "libm/src/lib.rs",
        "use std::sync::Mutex;\n\nstatic REGISTRY: Mutex<Vec<String>> = Mutex::new(Vec::new());\n\n#[cfg(feature = \"enable\")]\nfn collect(key: &str) {\n    REGISTRY.lock().unwrap().push(key.to_owned());\n}\n\npub fn record(key: &str) {\n    #[cfg(feature = \"enable\")]\n    collect(key);\n    #[cfg(not(feature = \"enable\"))]\n    let _ = key;\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn records() {\n        super::record(\"a\");\n    }\n}\n",
    );
    write(
        dir,
        "tool/Cargo.toml",
        "[package]\nname = \"tool\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(dir, "tool/src/lib.rs", "pub fn nothing() {}\n");
}

#[test]
fn a_workspace_root_refuses_a_driver_and_names_the_member_crates() {
    if !have_cargo() {
        return;
    }
    let r = Run::of(workspace);
    let o = r.run(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    assert_ne!(o.code, 0, "{}", o.text);
    assert_eq!(o.json["error"]["code"], "driver-unavailable", "{}", o.text);
    let msg = o.json["error"]["message"].as_str().unwrap();
    assert!(msg.contains("is a workspace with no package"), "{msg}");
    assert!(!msg.contains("only binaries"), "{msg}");
    let fix = o.json["error"]["fix"].as_str().unwrap();
    assert!(
        fix.starts_with(
            "dla harness gen --lang rust --driver --workload rust:test:libm/Cargo.toml"
        ),
        "{fix}"
    );
    assert!(fix.contains("the others: tool"), "{fix}");
}

/// A workspace whose member crate has a library target can have a driver (its refusal names
/// one), so a refused `--driver --probe` there is not recorded as "no driver can be built": the
/// growth of that finding on a shared harness stays a lead.
#[test]
fn a_driver_refused_at_a_workspace_root_with_a_library_member_is_not_recorded() {
    if !have_cargo() {
        return;
    }
    let r = Run::of(workspace);
    let target = r.id_at("libm/src/lib.rs:3");
    let o = r.run(&[
        "harness", "gen", "--lang", "rust", "--driver", "--probe", &target,
    ]);
    assert_ne!(o.code, 0, "{}", o.text);
    assert_eq!(o.json["error"]["code"], "driver-unavailable", "{}", o.text);
    let fix = o.json["error"]["fix"].as_str().unwrap();
    assert!(
        fix.contains("--driver --workload rust:test:libm/Cargo.toml"),
        "{fix}"
    );
    assert!(
        !r.run_dir.join("harness/driver-unavailable.json").exists(),
        "a driver can be built on the member crate"
    );
}

#[test]
fn the_command_a_workspace_target_prints_runs_as_printed_and_reaches_the_target() {
    if !have_cargo() || !have_llvm_tools() {
        return;
    }
    let r = Run::of(workspace);
    let target = r.id_at("libm/src/lib.rs:3");
    let out = r.ok(&[
        "harness",
        "gen",
        "--lang",
        "rust",
        "--driver",
        "--next",
        "--workload",
        "rust:test:libm/Cargo.toml",
    ]);
    let printed = out["regenerate"].as_str().unwrap();
    assert_eq!(
        printed,
        format!(
            "dla harness gen --lang rust --driver --workload rust:test:libm/Cargo.toml --features enable --probe {target}"
        ),
        "{out}"
    );
    // The same command from the file, for the queue and the trend.
    let features: Value = serde_json::from_str(
        &fs::read_to_string(r.run_dir.join("harness/H1/features.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(features["command"], printed, "{features}");
    // Run as printed.
    let args: Vec<&str> = printed.split_whitespace().skip(1).collect();
    let out = r.ok(&args);
    assert!(out.get("_attention").is_none(), "{out}");
    r.fill("H2", "libm::record(&key.to_string());");
    r.ok(&["harness", "run", "H2", "--mode", "both"]);
    let trend = r.ok(&["trend", "S1"]);
    assert_eq!(verdict(&trend, &target)["verdict"], "executed", "{trend}");
}

/// As `workspace`, with eight registries each filled through its own gated function: eight
/// targets in one gated file, enough for the output of `harness gen` to pass its bound.
fn workspace_many(dir: &Path) {
    workspace(dir);
    let mut src = String::from("use std::sync::Mutex;\n\n");
    for i in 0..8 {
        src.push_str(&format!(
            "static REG{i}: Mutex<Vec<String>> = Mutex::new(Vec::new());\n"
        ));
    }
    src.push('\n');
    for i in 0..8 {
        src.push_str(&format!(
            "#[cfg(feature = \"enable\")]\nfn collect{i}(key: &str) {{\n    REG{i}.lock().unwrap().push(key.to_owned());\n}}\n\n"
        ));
    }
    src.push_str("pub fn record(key: &str) {\n");
    for i in 0..8 {
        src.push_str(&format!(
            "    #[cfg(feature = \"enable\")]\n    collect{i}(key);\n"
        ));
    }
    src.push_str("    let _ = key;\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn records() {\n        super::record(\"a\");\n    }\n}\n");
    write(dir, "libm/src/lib.rs", &src);
}

#[test]
fn the_attention_of_many_targets_is_one_short_line_and_the_command_is_never_cut() {
    if !have_cargo() {
        return;
    }
    let r = Run::of(workspace_many);
    let out = r.ok(&[
        "harness",
        "gen",
        "--lang",
        "rust",
        "--driver",
        "--next",
        "8",
        "--workload",
        "rust:test:libm/Cargo.toml",
    ]);
    assert_eq!(
        out["truncated"], true,
        "the output must be over its bound: {out}"
    );
    let attention = out["_attention"].as_array().unwrap();
    assert_eq!(attention.len(), 1, "{out}");
    let line = attention[0].as_str().unwrap();
    assert!(!line.ends_with("..."), "{line}");
    assert_eq!(
        line,
        "feature enable is off by default: 8 targets have code behind it (gated sites in the crate: 16); run the command under regenerate"
    );
    assert!(line.len() < 150, "{}", line.len());
    // The command is whole, names every target, and runs as printed.
    let printed = out["regenerate"].as_str().unwrap();
    assert!(!printed.ends_with("..."), "{printed}");
    assert!(printed.len() > 160, "{}", printed.len());
    assert_eq!(printed.matches("--probe").count(), 8, "{printed}");
    let args: Vec<&str> = printed.split_whitespace().skip(1).collect();
    let again = r.ok(&args);
    assert!(again.get("_attention").is_none(), "{again}");
    assert!(again.get("regenerate").is_none(), "{again}");
}

fn have_toolchain(channel: &str) -> bool {
    let ok = Proc::new("cargo")
        .arg(format!("+{channel}"))
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success());
    if !ok {
        eprintln!("skipping: toolchain {channel} is not installed");
    }
    ok
}

/// A crate of edition 2021 that pins an old toolchain, which rejects `unsafe extern` blocks.
fn pinned(dir: &Path) {
    write(
        dir,
        "Cargo.toml",
        "[package]\nname = \"old\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        dir,
        "rust-toolchain.toml",
        "[toolchain]\nchannel = \"1.80.0\"\n",
    );
    write(
        dir,
        "src/lib.rs",
        "use std::sync::Mutex;\n\nstatic KEPT: Mutex<Vec<String>> = Mutex::new(Vec::new());\n\npub fn keep(key: &str) {\n    KEPT.lock().unwrap().push(key.to_owned());\n}\n",
    );
}

#[test]
fn a_crate_that_pins_an_old_toolchain_gets_a_driver_that_builds_and_a_failure_names_the_channel() {
    if !have_cargo() || !have_toolchain("1.80.0") {
        return;
    }
    let r = Run::of(pinned);
    r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    let text = fs::read_to_string(r.driver("H1")).unwrap();
    assert!(text.contains("    extern \"C\" {"), "plain extern for 2021");
    assert!(!text.contains("unsafe extern"));
    r.fill("H1", "old::keep(&key.to_string());");
    let out = r.ok(&["harness", "run", "H1", "--mode", "fresh"]);
    assert!(out["summary"]["series"][0].is_string(), "{out}");

    // A hook that does not compile: the refusal says which compiler spoke.
    let r = Run::of(pinned);
    r.ok(&["harness", "gen", "--lang", "rust", "--driver", "--next"]);
    r.fill("H1", "old::keep(&key.to_string()) +;");
    let o = r.run(&["harness", "run", "H1", "--mode", "fresh"]);
    assert_ne!(o.code, 0, "{}", o.text);
    assert_eq!(o.json["error"]["code"], "harness-failed", "{}", o.text);
    let msg = o.json["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("did not build with the pinned toolchain 1.80.0 (rust-toolchain.toml)"),
        "{msg}"
    );
}
