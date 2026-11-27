//! One verdict for a capture series. `dla summarize` writes a series from a few captures of a
//! process and passes a column by the capture rule (at least three captures, bytes rising at
//! every pair, a block count rising or one constant number of blocks: a buffer that is resized,
//! the last interval at least half of the first). `dla trend` judges that series by the same rule
//! (not the eight-point slope gate), so a column that passed in the summary passes in the trend,
//! carries `resized`, and the final gate offers the passing record to `dla ledger add`. The
//! series names the harness the captures came from, so the finding made from it is about that
//! harness, is a lead, and cannot be closed by text while the budget is above its floor.
//!
//! The scenario is the one a real Rust run met: allocation stacks of a hook driver at 1, 2 and 4
//! times the step, one stack that grows at a project site no finding is at, one block that is
//! resized at a second site. The shim's exit records are written by hand under `harness/H4/out/`
//! over a small C program built here (the stacks are symbolized by `addr2line`), so the tests
//! skip when `gcc`, `nm` or `addr2line` is absent. A JVM class histogram stands for a runtime
//! that is not Rust.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command as Proc,
};

use assert_cmd::Command;
use dla_core::harness::{AgentKind, CommandSpec, HarnessManifest, HarnessMode};
use dla_core::{runtime::Runtime, series::KeyMode};
use serde_json::{Value, json};

fn dla(project: &Path) -> Command {
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
    c.env("DLA_CONSENT_TIERS", "1,2");
    c
}

fn have(tool: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(tool).is_file()))
}

struct Project {
    dir: tempfile::TempDir,
    run_dir: PathBuf,
}

impl Project {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let p = Self {
            run_dir: PathBuf::new(),
            dir,
        };
        let init = p.ok(&["run", "init", "--session-id", "t", "--entry", "repository"]);
        let run_dir = p
            .dir
            .path()
            .join(".dla")
            .join(init["run_id"].as_str().unwrap());
        Self { run_dir, ..p }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn run(&self, args: &[&str]) -> (i32, Value) {
        let o = dla(self.path()).args(args).output().unwrap();
        let text = String::from_utf8(o.stdout).unwrap();
        let v = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
        (o.status.code().unwrap(), v)
    }

    fn ok(&self, args: &[&str]) -> Value {
        let (code, v) = self.run(args);
        assert_eq!(code, 0, "{args:?}: {v}");
        assert_eq!(v["ok"], true, "{v}");
        v
    }

    fn put(&self, rel: &str, text: &str) -> String {
        let p = self.path().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
        rel.to_owned()
    }

    fn json_file(&self, rel: &str) -> Value {
        serde_json::from_str(&fs::read_to_string(self.run_dir.join(rel)).unwrap()).unwrap()
    }

    fn ledger(&self) -> Vec<Value> {
        fs::read_to_string(self.run_dir.join("ledger.jsonl"))
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn evidence(&self, id: &str) -> Value {
        self.ledger()
            .into_iter()
            .find(|l| l["type"] == "evidence" && l["evidence_id"] == id)
            .unwrap()
    }

    /// The finding `id` as the ledger line that added it.
    fn finding(&self, id: &str) -> Value {
        self.ledger()
            .into_iter()
            .find(|l| l["type"] == "finding_added" && l["finding"]["id"] == id)
            .unwrap()["finding"]
            .clone()
    }

    /// What the final gate lists (the full file).
    fn final_gate(&self) -> Vec<Value> {
        let _ = self.run(&["gate", "check", "--final"]);
        let full = self.json_file("gate/final.json");
        full["missing"].as_array().unwrap().clone()
    }

    /// A census-like finding at `site`, with `--region off-heap`.
    fn census_finding(&self, owner: &str, site: &str) -> String {
        let v = self.ok(&[
            "ledger",
            "add",
            "--origin",
            "census",
            "--evidence",
            "E1",
            "--site",
            site,
            "--holder",
            "static/global",
            "--owner",
            owner,
            "--region",
            "off-heap",
        ]);
        v["id"].as_str().unwrap().to_owned()
    }

    /// The hook driver `id`: a Rust driver harness that targets `targets`.
    fn driver(&self, id: &str, targets: &[&str]) {
        self.driver_of(id, Runtime::Rust, AgentKind::DriverRust, targets);
    }

    /// The hook driver `id` of `runtime`.
    fn driver_of(&self, id: &str, runtime: Runtime, agent: AgentKind, targets: &[&str]) {
        let m = HarnessManifest {
            sibling_of: Default::default(),
            purpose: None,
            targets: targets.iter().map(|t| t.parse().unwrap()).collect(),
            schema_version: 1,
            id: id.into(),
            runtime,
            mode: HarnessMode::Driver,
            workload: Some("W1".into()),
            scratch: format!("harness/{id}/work"),
            build: None,
            run: CommandSpec {
                argv: vec!["true".into()],
                cwd: ".".into(),
                env: vec![],
            },
            iter_arg: None,
            warmup: 1,
            iterations: 8,
            step: 440,
            key_modes: vec![KeyMode::Fresh],
            probes: vec![],
            agent,
            detectors: vec![],
            tools: vec![],
            gaps: vec![],
            timeout_secs: 300,
            ffi: None,
            captures: vec![],
        };
        m.save(&self.run_dir).unwrap();
    }
}

/// Compiles two functions of one source file (`grow_site` on line 3, `resize_site` on line 4)
/// without PIE and returns the address inside each body.
fn build(dir: &Path) -> Option<(u64, u64)> {
    let c = "#include <stdlib.h>\n\nvoid *grow_site(void) { return malloc(4096); }\nvoid *resize_site(void) { return malloc(8192); }\nint main(void) { return grow_site() == 0 || resize_site() == 0; }\n";
    fs::create_dir_all(dir.join("src")).ok()?;
    fs::write(dir.join("src/leak.c"), c).ok()?;
    let st = Proc::new("gcc")
        .current_dir(dir)
        .args(["-g", "-O0", "-no-pie", "-o", "prog", "src/leak.c"])
        .status()
        .ok()?;
    if !st.success() {
        return None;
    }
    let nm = Proc::new("nm").current_dir(dir).arg("prog").output().ok()?;
    let text = String::from_utf8(nm.stdout).ok()?;
    let at = |name: &str| -> Option<u64> {
        let line = text.lines().find(|l| l.ends_with(&format!(" {name}")))?;
        Some(u64::from_str_radix(line.split_whitespace().next()?, 16).ok()? + 8)
    };
    Some((at("grow_site")?, at("resize_site")?))
}

/// One exit record of the shim at `x` units of work: the stack of `grow_site` holds `grow` bytes
/// in `blocks` blocks, the stack of `resize_site` holds `resize` bytes in one block, and a libc
/// stack holds 4 KiB in one block at every point (it fails the rule, with no site).
fn dump(prog: &Path, addrs: (u64, u64), x: u64, grow: (u64, u64), resize: u64) -> String {
    format!(
        "{{\"dla_harness\":1,\"phase\":\"exit\",\"pid\":{pid},\"exe\":\"{exe}\",\"runtime\":\"native\",\"x\":{x},\"t_ms\":10,\"values\":{{\"native.live_bytes\":{total}}},\"raw_sites\":{{\"0xaaaa{x}\":{{\"live_bytes\":{gb},\"live_blocks\":{gk},\"allocs\":{x},\"frames\":[{{\"a\":\"0x1\",\"m\":0,\"o\":\"{a:#x}\"}}]}},\"0xbbbb{x}\":{{\"live_bytes\":{resize},\"live_blocks\":1,\"allocs\":1,\"frames\":[{{\"a\":\"0x2\",\"m\":0,\"o\":\"{b:#x}\"}}]}},\"0xcccc{x}\":{{\"live_bytes\":4096,\"live_blocks\":1,\"allocs\":1,\"frames\":[{{\"a\":\"0x3\",\"m\":1,\"o\":\"0x10\"}}]}}}},\"modules\":[{{\"path\":\"{exe}\",\"bias\":\"0x0\"}},{{\"path\":\"/nonexistent/libc.so.6\",\"bias\":\"0x7f0000000000\"}}]}}\n",
        pid = 600 + x,
        exe = prog.display(),
        total = grow.0 + resize,
        gb = grow.0,
        gk = grow.1,
        a = addrs.0,
        b = addrs.1,
    )
}

/// The three captures of `harness` (`p1-alloc` to `p3-alloc`, x = 440, 880, 1760) with the given
/// bytes of the growing stack (blocks = bytes / 4096) and of the resized block; returns their
/// project-relative paths.
fn captures(
    p: &Project,
    harness: &str,
    addrs: (u64, u64),
    grow: [u64; 3],
    resize: [u64; 3],
) -> Vec<String> {
    let prog = p.run_dir.join(format!("harness/{harness}/work/prog"));
    let run_id = p
        .run_dir
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    (0..3)
        .map(|k| {
            let rel = format!(
                ".dla/{run_id}/harness/{harness}/out/p{}-alloc/shim-dump.jsonl",
                k + 1
            );
            p.put(
                &rel,
                &dump(&prog, addrs, 440 << k, (grow[k], grow[k] / 4096), resize[k]),
            );
            rel
        })
        .collect()
}

/// A project of the r8 shape: the program built under `harness/H4/work`, the project's own copy
/// of its source, a finding F1 at a line the stacks do not reach, and the hook driver H4 that
/// targets F1. `None` when the tools are missing.
fn scene() -> Option<(Project, (u64, u64))> {
    if !(have("gcc") && have("nm") && have("addr2line")) {
        return None;
    }
    let p = Project::new();
    let work = p.run_dir.join("harness/H4/work");
    let addrs = build(&work)?;
    let src = fs::read_to_string(work.join("src/leak.c")).unwrap();
    p.put("src/leak.c", &src);
    p.put("src/other.c", &"int y;\n".repeat(20));
    let f1 = p.census_finding("src.other.register", "src/other.c:9");
    assert_eq!(f1, "F1");
    p.driver("H4", &["F1"]);
    Some((p, addrs))
}

const MIB: u64 = 1 << 20;
/// The growing stack: 1000, 2000 and 4000 blocks of 4 KiB.
const GROW: [u64; 3] = [4_096_000, 8_192_000, 16_384_000];
/// The resized block: 1, 2 and 4 MiB in one block.
const RESIZE: [u64; 3] = [MIB, 2 * MIB, 4 * MIB];

fn summarize(p: &Project, files: &[String]) -> Value {
    let mut args = vec!["summarize", "alloc-stack-diff"];
    args.extend(files.iter().map(String::as_str));
    p.ok(&args)
}

/// The `column -> gate` of the report of trend `t`.
fn trend_gates(p: &Project, t: &str) -> Vec<(String, bool, bool)> {
    p.json_file(&format!("trend/{t}.json"))["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["name"].as_str().unwrap().to_owned(),
                c["gate"] == "pass",
                c["resized"] == true,
            )
        })
        .collect()
}

#[test]
fn a_passing_summary_is_judged_the_same_by_the_trend_and_leads_to_the_alloc_test_of_its_harness() {
    let Some((p, addrs)) = scene() else { return };
    let files = captures(&p, "H4", addrs, GROW, RESIZE);

    // The summary passes two stacks at project sites no finding is at, ties nothing, and prints
    // the command that makes the finding the growth stands on.
    let v = summarize(&p, &files);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([]), "{v}");
    let summary_ev = v["growth_evidence"].as_str().unwrap().to_owned();
    let needed = v["needed"].as_str().unwrap();
    assert!(
        needed.contains(&format!(
            "dla ledger add --origin growth-series --evidence {summary_ev}"
        )),
        "{needed}"
    );
    assert!(
        needed.contains("dla alloc-test H4 on that finding"),
        "{needed}"
    );
    // The series is about the harness the captures came from; no run of it wrote the series.
    let series = p.json_file("series/S1.json");
    assert_eq!(series["source"]["kind"], "dump", "{series}");
    assert_eq!(series["source"]["harness"], "H4", "{series}");
    let e = p.evidence(&summary_ev);
    let passed: Vec<String> = e["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["column"].as_str().unwrap().to_owned())
        .collect();
    assert_eq!(passed.len(), 2, "{e}");

    // The final gate lists the series for its trend (it is not exempt).
    let missing = p.final_gate();
    assert!(
        missing
            .iter()
            .any(|m| m["kind"] == "series" && m["id"] == "S1"),
        "{missing:?}"
    );

    // The trend passes the same columns and no others (each with its block count), and the
    // count column of the resized block reads `resized`, not flat.
    let t = p.ok(&["trend", "S1"]);
    assert_eq!(t["failed"], 2, "{t}");
    let gates = trend_gates(&p, "T1");
    let summary_passed: Vec<&(String, bool, bool)> = gates
        .iter()
        .filter(|(n, ..)| {
            passed
                .iter()
                .any(|c| n == c || n.strip_suffix(".count") == Some(c.as_str()))
        })
        .collect();
    assert_eq!(summary_passed.len(), 4, "{gates:?}");
    assert!(summary_passed.iter().all(|(_, pass, _)| *pass), "{gates:?}");
    assert_eq!(
        gates.iter().filter(|(_, pass, _)| *pass).count(),
        4,
        "a column the summary failed passes in the trend: {gates:?}"
    );
    let resized: Vec<&str> = gates
        .iter()
        .filter(|(_, _, r)| *r)
        .map(|(n, ..)| n.as_str())
        .collect();
    assert_eq!(resized.len(), 2, "{gates:?}");
    assert!(
        resized.iter().all(|n| n.contains("resize_site")),
        "{resized:?}"
    );
    let report = p.json_file("trend/T1.json");
    assert_eq!(report["thresholds"]["min_points"], 3, "{report}");
    let growth_ev = t["evidence"][0].as_str().unwrap().to_owned();
    let trend_ev = p.evidence(&growth_ev);
    assert_eq!(trend_ev["result"]["gate"], "pass", "{trend_ev}");
    // The record carries `resized` on the column of the resized block.
    let recorded: Vec<(String, bool)> = trend_ev["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            (
                c["column"].as_str().unwrap().to_owned(),
                c["resized"] == true,
            )
        })
        .collect();
    assert!(
        recorded
            .iter()
            .any(|(n, r)| n.contains("resize_site") && *r)
            && recorded.iter().any(|(n, r)| n.contains("grow_site") && !*r),
        "{recorded:?}"
    );
    assert!(
        trend_ev["limitations"]
            .to_string()
            .contains("capture-series rule of `dla summarize`"),
        "{trend_ev}"
    );

    // The gate item offers the passing record and says the summary passed.
    let missing = p.final_gate();
    let item = missing
        .iter()
        .find(|m| m["kind"] == "trend_column" && m["id"].as_str().unwrap().starts_with("T1:"))
        .unwrap_or_else(|| panic!("{missing:?}"));
    assert!(
        item["description"]
            .as_str()
            .unwrap()
            .contains("passed its gate"),
        "{item}"
    );
    let fix = item["fix"].as_str().unwrap();
    assert!(fix.contains("dla alloc-test H4 --finding"), "{fix}");
    let offered = fix
        .strip_prefix("dla ledger add --origin growth-series --evidence ")
        .and_then(|r| r.split_whitespace().next())
        .unwrap_or_else(|| panic!("{fix}"));
    let offered_ev = p.evidence(offered);
    assert_eq!(offered_ev["result"]["gate"], "pass", "{offered_ev}");
    assert!(
        !offered_ev["result"]["passed"]
            .as_array()
            .unwrap()
            .is_empty(),
        "{offered_ev}"
    );

    // `ledger add` makes a finding with the site of the largest passing column.
    let added = p.ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        offered,
    ]);
    let f = added["id"].as_str().unwrap().to_owned();
    assert_eq!(p.finding(&f)["site"], "src/leak.c:3");
    // It ties both columns of the capture (the bytes and the blocks): one finding is enough.
    let missing = p.final_gate();
    assert!(
        !missing.iter().any(|m| m["kind"] == "trend_column"
            && m["id"].as_str().is_some_and(|i| i.starts_with("T1:"))),
        "{missing:?}"
    );

    // Closing it by text is refused while the budget is above its floor, and the refusal names
    // the alloc-test on the harness the captures came from.
    for reason in ["inconclusive", "not_exercised"] {
        let (code, r) = p.run(&[
            "ledger",
            "resolve",
            &f,
            "--as",
            "unresolved",
            "--reason",
            reason,
            "--remedy",
            "a backlog of the reporter, not shown to persist",
            "--evidence",
            offered,
        ]);
        assert_ne!(code, 0, "{reason}: {r}");
        assert_eq!(r["error"]["code"], "growth-not-measured", "{r}");
        assert!(
            r["error"]["fix"]
                .as_str()
                .unwrap()
                .contains(&format!("dla alloc-test H4 --finding {f}")),
            "{r}"
        );
    }
    // The final gate lists the growth with the same command.
    let missing = p.final_gate();
    assert!(
        missing.iter().any(|m| m["fix"]
            .as_str()
            .is_some_and(|x| x.contains(&format!("dla alloc-test H4 --finding {f}")))),
        "{missing:?}"
    );

    // The alloc-test of H4 for the finding is not refused as `harness-not-about`.
    let (_, r) = p.run(&["alloc-test", "H4", "--finding", &f]);
    assert_ne!(r["error"]["code"], "harness-not-about", "{r}");
}

#[test]
fn captures_that_rise_and_then_stop_fail_in_summary_and_trend_give_no_lead_and_can_be_closed() {
    let Some((p, addrs)) = scene() else { return };
    // 1000, 2000 and 2050 blocks: the last interval is 5% of the first (a warm-up that filled).
    let stop = [4_096_000, 8_192_000, 8_396_800];
    let files = captures(&p, "H4", addrs, stop, [MIB, MIB, MIB]);
    let v = summarize(&p, &files);
    assert_eq!(v["gate"], "fail", "{v}");
    let t = p.ok(&["trend", "S1"]);
    assert_eq!(t["passed"], json!([]), "{t}");
    let gates = trend_gates(&p, "T1");
    assert!(gates.iter().all(|(_, pass, r)| !pass && !r), "{gates:?}");
    let report = p.json_file("trend/T1.json");
    let why = report["columns"][0]["gate_failed"].to_string();
    assert!(why.contains("last interval"), "{why}");
    assert!(!why.contains("n < 8"), "{why}");
    // Nothing is offered, so the final gate has no item for the capture.
    let missing = p.final_gate();
    assert!(
        !missing
            .iter()
            .any(|m| m["id"].as_str().is_some_and(|i| i.starts_with("T1:"))),
        "{missing:?}"
    );
    // A finding at that site is no lead: it can be closed.
    let f = p.census_finding("src.leak.grow_site", "src/leak.c:3");
    let (code, r) = p.run(&[
        "ledger",
        "resolve",
        &f,
        "--as",
        "unresolved",
        "--reason",
        "inconclusive",
        "--remedy",
        "it filled and stopped over the three captures",
        "--evidence",
        v["growth_evidence"].as_str().unwrap(),
    ]);
    assert_ne!(
        r["error"]["code"], "growth-not-measured",
        "no growth passed, so nothing holds the close: {code} {r}"
    );
}

#[test]
fn captures_of_two_harnesses_give_the_series_no_harness() {
    let Some((p, addrs)) = scene() else { return };
    p.driver("H5", &[]);
    // H5 has the same program, built in its own scratch copy (the stacks resolve to the project's
    // source through the copy they were built in), so its stacks key and tie as H4's do.
    let addrs5 = build(&p.run_dir.join("harness/H5/work")).unwrap();
    let mut files = captures(&p, "H4", addrs, GROW, RESIZE);
    let other = captures(&p, "H5", addrs5, GROW, RESIZE);
    files[2].clone_from(&other[2]);
    let v = summarize(&p, &files);
    assert_eq!(v["gate"], "pass", "{v}");
    let series = p.json_file("series/S1.json");
    assert!(series["source"].get("harness").is_none(), "{series}");
    // A capture outside any harness names none either.
    let mut files = captures(&p, "H4", addrs, GROW, RESIZE);
    let outside = p.put(
        "in/p3-alloc/shim-dump.jsonl",
        &fs::read_to_string(p.path().join(&files[2])).unwrap(),
    );
    files[2] = outside;
    summarize(&p, &files);
    let series = p.json_file("series/S2.json");
    assert!(series["source"].get("harness").is_none(), "{series}");
}

/// The three `jcmd GC.class_histogram` outputs the conformance fixtures keep, under the harness
/// directory of a JVM driver: `shop.Leaker$Entry` grows in count and bytes at every capture.
fn jvm_histograms(p: &Project, harness: &str) -> Vec<String> {
    let from =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/summarize/jcmd-class-histogram");
    let run_id = p
        .run_dir
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    (1..=3)
        .map(|k| {
            let rel = format!(".dla/{run_id}/harness/{harness}/out/p{k}-histogram/histo{k}.txt");
            p.put(
                &rel,
                &fs::read_to_string(from.join(format!("histo{k}.txt"))).unwrap(),
            );
            rel
        })
        .collect()
}

#[test]
fn a_jvm_type_histogram_is_judged_by_the_same_rule_and_its_finding_is_a_lead() {
    let p = Project::new();
    p.driver_of("H2", Runtime::Jvm, AgentKind::DriverJvm, &[]);
    let files = jvm_histograms(&p, "H2");
    let mut args = vec!["summarize", "type-histogram-diff"];
    args.extend(files.iter().map(String::as_str));
    let v = p.ok(&args);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], json!([]), "{v}");
    let ev = v["growth_evidence"].as_str().unwrap().to_owned();
    assert!(
        v["needed"].as_str().unwrap().contains(&format!(
            "dla ledger add --origin growth-series --evidence {ev}"
        )),
        "{v}"
    );
    assert_eq!(p.json_file("series/S1.json")["source"]["harness"], "H2");
    // The trend agrees column by column.
    let summary = p.evidence(&ev);
    let passed: Vec<String> = summary["result"]["passed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["column"].as_str().unwrap().to_owned())
        .collect();
    assert!(
        passed.iter().any(|c| c.contains("shop.Leaker$Entry")),
        "{passed:?}"
    );
    p.ok(&["trend", "S1"]);
    let gates = trend_gates(&p, "T1");
    for (name, pass, _) in &gates {
        let base = name.strip_suffix(".count").unwrap_or(name);
        assert_eq!(*pass, passed.iter().any(|c| c == base), "{name}: {gates:?}");
    }
    // The gate offers a passing record; the finding made from it is a lead that holds the close.
    let missing = p.final_gate();
    let item = missing
        .iter()
        .find(|m| m["kind"] == "trend_column" && m["id"].as_str().unwrap().starts_with("T1:"))
        .unwrap_or_else(|| panic!("{missing:?}"));
    let offered = item["fix"]
        .as_str()
        .unwrap()
        .strip_prefix("dla ledger add --origin growth-series --evidence ")
        .and_then(|r| r.split_whitespace().next())
        .unwrap()
        .to_owned();
    assert_eq!(p.evidence(&offered)["result"]["gate"], "pass");
    let f = p.ok(&[
        "ledger",
        "add",
        "--origin",
        "growth-series",
        "--evidence",
        &offered,
    ])["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let (code, r) = p.run(&[
        "ledger",
        "resolve",
        &f,
        "--as",
        "unresolved",
        "--reason",
        "inconclusive",
        "--remedy",
        "a cache, not shown to persist",
        "--evidence",
        &offered,
    ]);
    assert_ne!(code, 0, "{r}");
    assert_eq!(r["error"]["code"], "growth-not-measured", "{r}");
    assert!(
        r["error"]["fix"]
            .as_str()
            .unwrap()
            .contains(&format!("dla alloc-test H2 --finding {f}")),
        "{r}"
    );
}
