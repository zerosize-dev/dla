//! The native adapters of `dla summarize`: valgrind memcheck and vgdb, heaptrack, dhat,
//! lsan, the allocator shim's stack groups and descriptor counts.
//!
//! The inputs are real tool output (valgrind 3.22, heaptrack 1.5, gcc LeakSanitizer, a
//! harness run of the native-frame-decoder fixture) kept under `tests/fixtures/summarize/`.
//! Each test starts a run in a temporary project, adds the census finding a real run would
//! have, and checks which column ties to it and why. Tests of Linux-only tools skip when
//! the tool is absent.

use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use serde_json::Value;

const SECRET: &str = "AKIAIOSFODNN7EXAMPLE1";

fn dla(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
    ] {
        c.env_remove(var);
    }
    c
}

fn run_env(project: &Path, args: &[&str], env: &[(&str, Option<&str>)]) -> (i32, String, Value) {
    let mut c = dla(project);
    c.args(args);
    for (k, v) in env {
        match v {
            Some(v) => c.env(k, v),
            None => c.env_remove(k),
        };
    }
    let out = c.output().unwrap();
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.len() <= 2048, "output over the bound: {}", text.len());
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    (out.status.code().unwrap(), text, v)
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let (code, _, v) = run_env(project, args, &[]);
    assert_eq!(code, 0, "{args:?}: {v}");
    assert_eq!(v["ok"], true, "{v}");
    v
}

struct Project {
    dir: tempfile::TempDir,
    run_dir: PathBuf,
}

impl Project {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let init = ok(dir.path(), &["run", "init", "--session-id", "t"]);
        let run_dir = dir
            .path()
            .join(".dla")
            .join(init["run_id"].as_str().unwrap());
        Self { dir, run_dir }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Copies fixture files of `backend` into the project; returns their paths.
    fn fixture(&self, backend: &str, names: &[&str]) -> Vec<String> {
        let from = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/summarize")
            .join(backend);
        names
            .iter()
            .map(|n| {
                let to = self.path().join("in").join(n);
                fs::create_dir_all(to.parent().unwrap()).unwrap();
                fs::copy(from.join(n), &to).unwrap();
                format!("in/{n}")
            })
            .collect()
    }

    fn put(&self, rel: &str, text: &str) -> String {
        let p = self.path().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
        rel.to_owned()
    }

    /// A census-like finding: owner, site and holder.
    /// The sources a real project has: the frames of a tool run are looked up in them.
    fn sources(&self) {
        for f in ["src/decoder.c", "src/main.c"] {
            self.put(f, &"/* line */\n".repeat(60));
        }
    }

    fn finding(&self, owner: &str, site: &str) -> String {
        self.sources();
        let v = ok(
            self.path(),
            &[
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
            ],
        );
        v["id"].as_str().unwrap().to_owned()
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

    fn writers(&self, finding: &str) -> Vec<String> {
        self.ledger()
            .iter()
            .filter(|l| l["type"] == "finding_annotated" && l["finding"] == finding)
            .flat_map(|l| l["writers"].as_array().cloned().unwrap_or_default())
            .map(|w| w.as_str().unwrap().to_owned())
            .collect()
    }

    fn coverage(&self) -> Value {
        serde_json::from_str(&fs::read_to_string(self.run_dir.join("coverage.json")).unwrap())
            .unwrap()
    }
}

fn summarize(p: &Project, op: &str, files: &[String]) -> Value {
    let mut args = vec!["summarize", op];
    args.extend(files.iter().map(String::as_str));
    ok(p.path(), &args)
}

/// The tie of the first passing column of a growth evidence.
fn first_tie(p: &Project, v: &Value) -> Value {
    let e = p.evidence(v["growth_evidence"].as_str().unwrap());
    e["result"]["passed"][0].clone()
}

fn have(tool: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|d| d.join(tool).is_file())
}

// ---------------------------------------------------------------------------------------
// alloc-stack-diff readers: the first project frame ties by site
// ---------------------------------------------------------------------------------------

const MEMCHECK: [&str; 3] = ["memcheck1.xml", "memcheck2.xml", "memcheck3.xml"];

#[test]
fn memcheck_stacks_tie_by_their_first_project_frame_and_by_nothing_else() {
    let p = Project::new();
    let f = p.finding("src.decoder.decode_frame", "src/decoder.c:29");
    let other = p.finding("src.main.main", "src/main.c:33");
    let files = p.fixture("valgrind-memcheck", &MEMCHECK);
    let v = summarize(&p, "alloc-stack-diff", &files);
    assert_eq!(v["backend"], "valgrind-memcheck");
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], serde_json::json!([f]), "{v}");
    let col = first_tie(&p, &v);
    assert_eq!(col["tie"]["rule"], "site");
    assert_eq!(col["tie"]["key"], "src/decoder.c:29");
    assert_eq!(col["finding"], f.as_str());
    assert_ne!(col["finding"], other.as_str());
    // A site tie learns no writer.
    assert!(p.writers(&f).is_empty(), "{:?}", p.writers(&f));
    // The leak kind and the stack are in the raw file.
    let raw = fs::read_to_string(p.run_dir.join(v["raw"]["path"].as_str().unwrap())).unwrap();
    assert!(raw.contains("definitely-lost"), "{raw}");
}

#[test]
fn two_findings_at_one_site_both_tie_and_one_report_never_passes() {
    let p = Project::new();
    let f1 = p.finding("src.decoder.decode_frame", "src/decoder.c:29");
    let f2 = p.finding("src.decoder.other", "src/decoder.c:29");
    let files = p.fixture("valgrind-memcheck", &MEMCHECK);
    let v = summarize(&p, "alloc-stack-diff", &files);
    assert_eq!(v["gate"], "pass");
    // Exact site identity is not ambiguous: every finding at the site is tied.
    let tied = v["tied"].as_array().unwrap();
    assert!(
        tied.contains(&serde_json::json!(f1)) && tied.contains(&serde_json::json!(f2)),
        "{v}"
    );
    assert!(v["ambiguous"].is_null(), "{v}");
    // One report: the at-exit view, no series.
    let q = Project::new();
    q.finding("src.decoder.decode_frame", "src/decoder.c:29");
    let one = q.fixture("valgrind-memcheck", &MEMCHECK[..1]);
    let v = summarize(&q, "alloc-stack-diff", &one);
    assert_eq!(v["mode"], "single");
    assert_eq!(v["gate"], "fail");
    assert!(v["series"].is_null());
}

#[test]
fn a_finding_option_forces_no_tie_on_a_stack_that_is_not_its_own() {
    let p = Project::new();
    p.finding("src.decoder.decode_frame", "src/decoder.c:29");
    let other = p.finding("src.main.main", "src/main.c:33");
    let files = p.fixture("valgrind-memcheck", &MEMCHECK);
    let mut args = vec!["summarize", "alloc-stack-diff", "--finding", other.as_str()];
    args.extend(files.iter().map(String::as_str));
    let v = ok(p.path(), &args);
    assert_eq!(v["tied"], serde_json::json!([]), "{v}");
    assert!(
        v["needed"]
            .as_str()
            .unwrap()
            .contains("no passing column ties"),
        "{v}"
    );
    assert!(p.writers(&other).is_empty());
}

#[test]
fn heaptrack_text_dhat_and_lsan_tie_the_same_site() {
    for (backend, names, expect) in [
        (
            "heaptrack",
            vec!["leaks1.txt", "leaks2.txt", "leaks3.txt"],
            "heaptrack",
        ),
        (
            "dhat",
            vec!["dhat1.json", "dhat2.json", "dhat3.json"],
            "dhat",
        ),
        (
            "lsan",
            vec!["report1.txt", "report2.txt", "report3.txt"],
            "lsan",
        ),
    ] {
        let p = Project::new();
        let f = p.finding("src.decoder.decode_frame", "src/decoder.c:29");
        let files = p.fixture(backend, &names);
        let v = summarize(&p, "alloc-stack-diff", &files);
        assert_eq!(v["backend"], expect, "{v}");
        assert_eq!(v["gate"], "pass", "{backend}: {v}");
        assert_eq!(v["tied"], serde_json::json!([f]), "{backend}: {v}");
        let col = first_tie(&p, &v);
        assert_eq!(col["tie"]["rule"], "site", "{backend}");
        assert_eq!(col["tie"]["key"], "src/decoder.c:29", "{backend}");
    }
}

#[test]
fn a_frame_name_with_a_secret_is_shaped_in_the_raw_file() {
    let p = Project::new();
    let files = p.fixture("valgrind-memcheck", &MEMCHECK);
    let v = summarize(&p, "alloc-stack-diff", &files);
    let raw = fs::read_to_string(p.run_dir.join(v["raw"]["path"].as_str().unwrap())).unwrap();
    assert!(!raw.contains(SECRET), "{raw}");
}

#[test]
fn heaptrack_refuses_a_diff_mixed_with_reports_and_reads_a_diff_alone() {
    let p = Project::new();
    let files = p.fixture("heaptrack", &["leaks1.txt", "leaks2.txt"]);
    let diff = p.put(
        "in/diff.txt",
        &fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../dla-parsers/tests/fixtures/native/heaptrack_c_diff.txt"),
        )
        .unwrap(),
    );
    let alone = summarize(&p, "alloc-stack-diff", std::slice::from_ref(&diff));
    assert_eq!(alone["backend"], "heaptrack");
    assert_eq!(alone["mode"], "single");
    let mut args = vec!["summarize", "alloc-stack-diff"];
    args.extend(files.iter().map(String::as_str));
    args.push(&diff);
    let (code, _, v) = run_env(p.path(), &args, &[]);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "mixed-formats", "{v}");
}

#[test]
fn a_raw_heaptrack_recording_needs_heaptrack_print_and_a_gap_names_it() {
    let p = Project::new();
    let raw = p.fixture("heaptrack", &["heaptrack.gz"]);
    // Without the tool: refused, with the install hint and a missing row.
    let empty = tempfile::tempdir().unwrap();
    let env = [
        ("PATH", Some(empty.path().to_str().unwrap())),
        ("JAVA_HOME", None),
        ("DLA_DATA_DIR", None),
    ];
    let mut args = vec!["summarize", "alloc-stack-diff", "--backend", "heaptrack"];
    args.extend(raw.iter().map(String::as_str));
    let (code, _, v) = run_env(p.path(), &args, &env);
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "tool-missing", "{v}");
    assert!(v["error"]["fix"].as_str().is_some_and(|f| !f.is_empty()));
    let cov = p.coverage();
    let row = cov["workers"]
        .as_object()
        .unwrap()
        .values()
        .find_map(|w| w["backends"].get("heaptrack_print"))
        .expect("a heaptrack_print row");
    assert_eq!(row["status"], "missing");
    // With it (Linux with heaptrack installed): printed into tmp/ and read.
    if cfg!(target_os = "linux") && have("heaptrack_print") {
        let q = Project::new();
        let raw = q.fixture("heaptrack", &["heaptrack.gz"]);
        let v = summarize(&q, "alloc-stack-diff", &raw);
        assert_eq!(v["backend"], "heaptrack");
        assert!(v["top"].as_array().is_some_and(|t| !t.is_empty()), "{v}");
        let cov = q.coverage();
        let row = cov["workers"]
            .as_object()
            .unwrap()
            .values()
            .find_map(|w| w["backends"].get("heaptrack_print"))
            .expect("a heaptrack_print row");
        assert_eq!(row["status"], "ran");
    }
}

// ---------------------------------------------------------------------------------------
// vgdb: the holder is a data symbol
// ---------------------------------------------------------------------------------------

const VGDB: [&str; 9] = [
    "pt40/vgdb-1-leak.txt",
    "pt40/vgdb-2-blocks.txt",
    "pt40/vgdb-3-who.txt",
    "pt80/vgdb-1-leak.txt",
    "pt80/vgdb-2-blocks.txt",
    "pt80/vgdb-3-who.txt",
    "pt120/vgdb-1-leak.txt",
    "pt120/vgdb-2-blocks.txt",
    "pt120/vgdb-3-who.txt",
];

/// A C source that defines the static `registry` at file scope.
const C_HOLD: &str = "#include <stdlib.h>\nstruct node { struct node *next; };\nstatic struct node *registry = NULL;\nstatic int cache;\n";

fn heap_graph(p: &Project, files: &[String]) -> Value {
    summarize(p, "heap-graph", files)
}

#[test]
fn a_data_symbol_holder_ties_by_holder_to_the_one_finding_that_ends_with_it() {
    let p = Project::new();
    p.put("src/hold.c", C_HOLD);
    let f = p.finding("src.hold.registry", "src/hold.c:8");
    p.finding("src.hold.cache", "src/hold.c:9");
    let files = p.fixture("valgrind-vgdb", &VGDB);
    let v = heap_graph(&p, &files);
    assert_eq!(v["backend"], "valgrind-vgdb");
    assert_eq!(v["points"], 3);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], serde_json::json!([f]), "{v}");
    let col = first_tie(&p, &v);
    assert_eq!(col["tie"]["rule"], "holder");
    assert_eq!(col["column"], "holder:static:registry");
    // One retention-path evidence per point.
    let kinds: Vec<_> = v["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            p.evidence(e.as_str().unwrap())["kind"]
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(
        kinds.iter().filter(|k| *k == "retention-path").count(),
        3,
        "{kinds:?}"
    );
    assert!(kinds.contains(&"growth-series".to_owned()));
    let e = p.evidence(v["evidence"][0].as_str().unwrap());
    assert_eq!(e["authority"], "live");
    assert_eq!(e["result"]["root"], "static registry");
}

#[test]
fn a_column_at_an_exact_site_lists_every_finding_there_and_finding_picks_the_primary() {
    let growth_column = |p: &Project, v: &serde_json::Value| {
        let id = v["evidence"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| p.evidence(e.as_str().unwrap()))
            .find(|e| e["kind"] == "growth-series")
            .expect("a growth evidence");
        id["result"]["passed"][0].clone()
    };
    let p = Project::new();
    let f1 = p.finding("src.decoder.decode_frame", "src/decoder.c:29");
    let f2 = p.finding("src.decoder.other", "src/decoder.c:29");
    let files = p.fixture("valgrind-memcheck", &MEMCHECK);
    let v = summarize(&p, "alloc-stack-diff", &files);
    let c = growth_column(&p, &v);
    // No `--finding`: the first by id is the finding, both are candidates.
    assert_eq!(c["finding"], serde_json::json!(f1), "{c}");
    assert_eq!(c["candidates"], serde_json::json!([f1, f2]), "{c}");
    assert_eq!(c["tie"]["rule"], "site", "{c}");
    // `--finding F2` is among them: it is the finding.
    let mut args = vec!["summarize", "alloc-stack-diff", "--finding", f2.as_str()];
    args.extend(files.iter().map(String::as_str));
    let v = ok(p.path(), &args);
    let c = growth_column(&p, &v);
    assert_eq!(c["finding"], serde_json::json!(f2), "{c}");
    assert_eq!(c["candidates"], serde_json::json!([f2, f1]), "{c}");
}

#[test]
fn a_bare_symbol_that_several_findings_end_with_ties_none() {
    let p = Project::new();
    p.finding("src.hold.registry", "src/hold.c:8");
    p.finding("src.other.registry", "src/other.c:3");
    let files = p.fixture("valgrind-vgdb", &VGDB);
    let v = heap_graph(&p, &files);
    assert_eq!(v["gate"], "pass");
    assert_eq!(v["tied"], serde_json::json!([]), "{v}");
    assert!(v["ambiguous"].is_object(), "{v}");
}

#[test]
fn a_rust_symbol_matches_its_module_path_and_never_a_bare_name() {
    // A real Rust program: the static is `outline_service::registry::REGISTRY`, which
    // valgrind prints in v0 mangling; the walk has two hops (the box, the Vec buffer).
    let p = Project::new();
    let names: Vec<String> = ["rs40", "rs80", "rs120"]
        .iter()
        .flat_map(|d| {
            [
                "vgdb-1-leak.txt",
                "vgdb-2-blocks.txt",
                "vgdb-3-who.txt",
                "vgdb-4-who.txt",
            ]
            .map(|n| format!("{d}/{n}"))
        })
        .collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let files = p.fixture("valgrind-vgdb", &names);
    // The crate of the program and the modules the findings are declared in.
    p.put(
        "Cargo.toml",
        "[package]\nname = \"outline-service\"\nversion = \"0.1.0\"\n",
    );
    // A finding in another module with the same name, and the right one.
    let right = p.finding("src.registry.REGISTRY", "src/registry.rs:3");
    p.finding("src.cache.REGISTRY", "src/cache.rs:5");
    let v = heap_graph(&p, &files);
    assert_eq!(v["tied"], serde_json::json!([right]), "{v}");
    let col = first_tie(&p, &v);
    assert_eq!(col["tie"]["rule"], "holder");
    assert_eq!(col["tie"]["key"], "outline_service.registry.REGISTRY");
    // The same symbol with only a wrong module's finding ties nothing: the module path
    // must end the owner, a bare name is not enough.
    let q = Project::new();
    let files = q.fixture("valgrind-vgdb", &names);
    q.finding("src.cache.REGISTRY", "src/cache.rs:5");
    let v = heap_graph(&q, &files);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(v["tied"], serde_json::json!([]), "{v}");
}

#[test]
fn a_thread_stack_root_and_fewer_than_three_points_tie_nothing() {
    let p = Project::new();
    p.finding("src.hold.registry", "src/hold.c:8");
    let files = p.fixture("valgrind-vgdb", &VGDB[..6]);
    let v = heap_graph(&p, &files);
    assert_eq!(v["points"], 2);
    assert_eq!(v["gate"], "fail");
    assert!(v["series"].is_null());
    assert!(v["needed"].as_str().unwrap().contains("3 or more points"));
    // Roots that are stacks: no holder column.
    let q = Project::new();
    q.finding("src.hold.registry", "src/hold.c:8");
    let files: Vec<String> = ["pt40", "pt80", "pt120"]
        .iter()
        .flat_map(|d| ["vgdb-1-leak.txt", "vgdb-2-blocks.txt", "vgdb-3-who.txt"].map(|n| (d, n)))
        .map(|(d, n)| {
            let src = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/summarize/valgrind-vgdb")
                .join(d)
                .join(n);
            let t = fs::read_to_string(src).unwrap().replace(
                " Address 0x10c018 is 0 bytes inside data symbol \"registry\"",
                " Address 0x1ffeffe318 is on thread 1's stack\n in frame #3, created by main (hold.c:10)",
            );
            q.put(&format!("in/{d}/{n}"), &t)
        })
        .collect();
    let v = heap_graph(&q, &files);
    assert_eq!(v["gate"], "fail", "{v}");
    assert_eq!(v["tied"].as_array().map_or(0, Vec::len), 0);
    assert!(
        v["walks"][0]["root"].as_str().unwrap().contains("stack"),
        "{v}"
    );
}

#[test]
fn memcheck_xml_alone_gives_records_with_no_holder_and_points_to_alloc_stack_diff() {
    let p = Project::new();
    // One directory per point, as the harness lays them out.
    let files: Vec<String> = p
        .fixture("valgrind-memcheck", &MEMCHECK)
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let to = format!("in/p{i}/valgrind.xml");
            fs::create_dir_all(p.path().join(format!("in/p{i}"))).unwrap();
            fs::rename(p.path().join(f), p.path().join(&to)).unwrap();
            to
        })
        .collect();
    let v = heap_graph(&p, &files);
    assert_eq!(v["backend"], "valgrind-vgdb");
    assert_eq!(v["gate"], "fail");
    assert_eq!(v["walks"][0]["root"], "no chain");
    assert!(
        v["needed"].as_str().unwrap().contains("alloc-stack-diff"),
        "{v}"
    );
}

#[test]
fn a_holder_tied_record_names_its_allocating_line_as_a_writer() {
    let p = Project::new();
    // The program's source is in the project, so the record's `hold.c:14` frame resolves.
    p.put("src/hold.c", &format!("{C_HOLD}{}", "x\n".repeat(30)));
    let f = p.finding("src.hold.registry", "src/hold.c:8");
    let files = p.fixture("valgrind-vgdb", &VGDB);
    let v = heap_graph(&p, &files);
    assert_eq!(v["tied"], serde_json::json!([f]), "{v}");
    let w = p.writers(&f);
    // The allocating function name carries the planted key: it is not a constructor or an
    // allocator, so its line is learned.
    assert_eq!(w, vec!["src/hold.c:14".to_owned()], "{w:?}");
}

// ---------------------------------------------------------------------------------------
// shim-stack-groups: rows keyed by stack, tied by site
// ---------------------------------------------------------------------------------------

#[cfg(unix)]
mod shim {
    use super::*;
    use std::process::Command as Proc;

    /// Compiles `leak.c` (one function on line 3) without PIE; returns the address of the
    /// function's body.
    fn build(dir: &Path) -> Option<u64> {
        let c = "#include <stdlib.h>\n\nvoid *leak_site(void) { return malloc(4096); }\nint main(void) { return leak_site() == 0; }\n";
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
        let line = text.lines().find(|l| l.ends_with(" leak_site"))?;
        let addr = u64::from_str_radix(line.split_whitespace().next()?, 16).ok()?;
        Some(addr + 8)
    }

    fn dump(prog: &Path, addr: u64, x: u64, bytes: u64) -> String {
        format!(
            "{{\"dla_harness\":1,\"phase\":\"exit\",\"pid\":{pid},\"exe\":\"{exe}\",\"runtime\":\"native\",\"x\":{x},\"t_ms\":10,\"values\":{{\"native.live_bytes\":{bytes}}},\"raw_sites\":{{\"0xaaaa{x}\":{{\"live_bytes\":{bytes},\"live_blocks\":{blocks},\"allocs\":{x},\"frames\":[{{\"a\":\"0x1\",\"m\":0,\"o\":\"{addr:#x}\"}}]}},\"0xbbbb{x}\":{{\"live_bytes\":4096,\"live_blocks\":1,\"allocs\":1,\"frames\":[{{\"a\":\"0x2\",\"m\":1,\"o\":\"0x10\"}}]}}}},\"modules\":[{{\"path\":\"{exe}\",\"bias\":\"0x0\"}},{{\"path\":\"/nonexistent/libc.so.6\",\"bias\":\"0x7f0000000000\"}}]}}\n",
            pid = 100 + x,
            exe = prog.display(),
            blocks = bytes / 4096,
        )
    }

    #[test]
    fn stacks_are_resolved_keyed_and_tied_by_their_first_project_frame() {
        if !(have("gcc") && have("nm") && have("addr2line")) {
            return;
        }
        let p = Project::new();
        // The harness's scratch copy: sources and the binary under harness/H1/work.
        let work = p.run_dir.join("harness/H1/work");
        let Some(addr) = build(&work) else { return };
        // The project itself has the same source, as the census sees it.
        p.put(
            "src/leak.c",
            &fs::read_to_string(work.join("src/leak.c")).unwrap(),
        );
        let f = p.finding("src.leak.leak_site", "src/leak.c:3");
        let prog = work.join("prog");
        // Points in the harness's layout, listed out of order on purpose.
        let mut files = Vec::new();
        for (label, x, bytes) in [
            ("p3-native", 3000u64, 12288u64),
            ("p1-native", 1000, 4096),
            ("p2-native", 2000, 8192),
        ] {
            let rel = format!(
                ".dla/{}/harness/H1/out/{label}/dump.jsonl",
                p.run_dir.file_name().unwrap().to_string_lossy()
            );
            let path = p.path().join(&rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, dump(&prog, addr, x, bytes)).unwrap();
            files.push(rel);
        }
        let v = summarize(&p, "type-histogram-diff", &files);
        assert_eq!(v["backend"], "shim-stack-groups", "{v}");
        assert_eq!(v["ordered_by"], "x");
        assert_eq!(v["gate"], "pass", "{v}");
        assert_eq!(v["tied"], serde_json::json!([f]), "{v}");
        let col = first_tie(&p, &v);
        assert_eq!(col["tie"]["rule"], "site");
        assert_eq!(col["tie"]["key"], "src/leak.c:3");
        assert!(
            col["column"].as_str().unwrap().contains("leak_site"),
            "{col}"
        );
        // The unresolvable libc stack stays a module offset: no site, no tie, no growth.
        let raw = fs::read_to_string(p.run_dir.join(v["raw"]["path"].as_str().unwrap())).unwrap();
        assert!(raw.contains("libc.so.6+0x10"), "{raw}");
        // No writer is learned from a site tie.
        assert!(p.writers(&f).is_empty());
    }

    #[test]
    fn without_addr2line_stacks_stay_offsets_and_a_missing_row_names_it() {
        if !(have("gcc") && have("nm") && have("addr2line")) {
            return;
        }
        let p = Project::new();
        let work = p.run_dir.join("harness/H1/work");
        let Some(addr) = build(&work) else { return };
        p.put(
            "src/leak.c",
            &fs::read_to_string(work.join("src/leak.c")).unwrap(),
        );
        let f = p.finding("src.leak.leak_site", "src/leak.c:3");
        let prog = work.join("prog");
        let mut files = Vec::new();
        for (label, x, bytes) in [
            ("p1-native", 1000u64, 4096u64),
            ("p2-native", 2000, 8192),
            ("p3-native", 3000, 12288),
        ] {
            let rel = format!("in/{label}/dump.jsonl");
            p.put(&rel, &dump(&prog, addr, x, bytes));
            files.push(rel);
        }
        let empty = tempfile::tempdir().unwrap();
        let env = [("PATH", Some(empty.path().to_str().unwrap()))];
        let mut args = vec!["summarize", "type-histogram-diff"];
        args.extend(files.iter().map(String::as_str));
        let (code, _, v) = run_env(p.path(), &args, &env);
        assert_eq!(code, 0, "{v}");
        // The growth shows, but no stack has a site, so nothing ties.
        assert_eq!(v["tied"], serde_json::json!([]), "{v}");
        assert_ne!(v["tied"], serde_json::json!([f]));
        let row = p.coverage()["workers"]
            .as_object()
            .unwrap()
            .values()
            .find_map(|w| w["backends"].get("addr2line").cloned())
            .expect("an addr2line row");
        assert_eq!(row["status"], "missing");
        assert!(row["install_hint"].as_str().unwrap().contains("binutils"));
    }

    #[test]
    fn two_points_make_a_diff_and_never_a_growth_column() {
        if !(have("gcc") && have("nm") && have("addr2line")) {
            return;
        }
        let p = Project::new();
        let work = p.run_dir.join("harness/H1/work");
        let Some(addr) = build(&work) else { return };
        let prog = work.join("prog");
        let mut files = Vec::new();
        for (label, x, bytes) in [("p1-native", 1000u64, 4096u64), ("p2-native", 2000, 8192)] {
            let rel = format!("in/{label}/dump.jsonl");
            p.put(&rel, &dump(&prog, addr, x, bytes));
            files.push(rel);
        }
        let v = summarize(&p, "type-histogram-diff", &files);
        assert_eq!(v["backend"], "shim-stack-groups");
        assert_eq!(v["gate"], "fail");
        assert!(v["series"].is_null());
    }
}

#[test]
fn folded_stacks_carry_exact_bytes_and_make_a_single_view() {
    let p = Project::new();
    p.finding("src.leak.add_session", "leak.c:11");
    let text = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../dla-parsers/tests/fixtures/native/heaptrack_c_leaked.folded"),
    )
    .unwrap();
    let rel = p.put("in/leaked.folded", &text);
    let v = summarize(&p, "alloc-stack-diff", &[rel]);
    assert_eq!(v["backend"], "heaptrack");
    assert_eq!(v["mode"], "single");
    assert_eq!(v["top"][0]["d_bytes"], 3_024_000, "{v}");
}

// ---------------------------------------------------------------------------------------
// proc-fd: counts per kind, never tied
// ---------------------------------------------------------------------------------------

#[test]
fn descriptor_kinds_grow_without_any_tie_and_never_show_a_name() {
    let p = Project::new();
    p.finding("src.net.conns", "src/net.c:10");
    let files = p.fixture("proc-fd", &["fds1.txt", "fds2.txt", "fds3.txt"]);
    let v = summarize(&p, "non-heap", &files);
    assert_eq!(v["backend"], "proc-fd");
    assert_eq!(v["gate"], "pass", "{v}");
    assert!(
        v["passed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c.as_str() == Some("group:fd/socket")),
        "{v}"
    );
    assert_eq!(v["tied"], serde_json::json!([]), "{v}");
    let e = p.evidence(v["growth_evidence"].as_str().unwrap());
    assert!(e["result"]["passed"][0].get("finding").is_none(), "{e}");
    // No path or name of a descriptor leaves the reader, even with a planted key in it.
    let all = fs::read_to_string(p.run_dir.join("ledger.jsonl")).unwrap();
    assert!(!all.contains(SECRET));
    let raw = fs::read_to_string(p.run_dir.join(v["raw"]["path"].as_str().unwrap())).unwrap();
    assert!(!raw.contains(SECRET) && !raw.contains("/var/log"), "{raw}");
}

#[test]
fn harness_dump_fd_values_are_read_in_order_of_x() {
    let p = Project::new();
    let mut files = Vec::new();
    for (label, x, sockets) in [("p2", 2000u64, 8u64), ("p1", 1000, 4), ("p3", 3000, 12)] {
        let rel = format!("in/{label}/dump.jsonl");
        p.put(
            &rel,
            &format!(
                "{{\"dla_harness\":1,\"phase\":\"exit\",\"pid\":{x},\"runtime\":\"native\",\"x\":{x},\"values\":{{\"fd.socket\":{sockets},\"fd.pipe\":2,\"fd.file\":3}}}}\n"
            ),
        );
        files.push(rel);
    }
    let v = summarize(&p, "non-heap", &files);
    assert_eq!(v["backend"], "proc-fd");
    assert_eq!(v["gate"], "pass", "{v}");
    assert!(
        v["passed"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c.as_str() == Some("group:fd/socket")),
        "{v}"
    );
    assert_eq!(v["tied"], serde_json::json!([]));
}

// ---------------------------------------------------------------------------------------
// auto selection
// ---------------------------------------------------------------------------------------

#[test]
fn auto_reads_each_native_format_with_its_own_adapter() {
    for (backend, op, names, expect) in [
        (
            "valgrind-memcheck",
            "alloc-stack-diff",
            MEMCHECK.to_vec(),
            "valgrind-memcheck",
        ),
        (
            "heaptrack",
            "alloc-stack-diff",
            vec!["leaks1.txt"],
            "heaptrack",
        ),
        ("dhat", "alloc-stack-diff", vec!["dhat1.json"], "dhat"),
        ("lsan", "alloc-stack-diff", vec!["report1.txt"], "lsan"),
        (
            "shim-stack-groups",
            "type-histogram-diff",
            vec!["exit1.jsonl", "exit2.jsonl"],
            "shim-stack-groups",
        ),
        ("proc-fd", "non-heap", vec!["fds1.txt"], "proc-fd"),
    ] {
        let p = Project::new();
        let files = p.fixture(backend, &names);
        let v = summarize(&p, op, &files);
        assert_eq!(v["backend"], expect, "{backend}: {v}");
    }
    // The shim's dump file is not read as a type histogram of the harness.
    let p = Project::new();
    let files = p.fixture("shim-stack-groups", &["exit1.jsonl", "exit2.jsonl"]);
    let (code, _, v) = run_env(
        p.path(),
        &[
            "summarize",
            "type-histogram-diff",
            "--backend",
            "harness-types",
            &files[0],
            &files[1],
        ],
        &[],
    );
    assert_eq!(code, 2, "{v}");
}

/// The first passing holder column of a heap-graph run, and what it tied.
fn tied_of(v: &Value) -> Value {
    v["tied"].clone()
}

/// The Rust fixture of `outline_service::registry::REGISTRY` with a project whose layout is
/// given: `(path, text)` files and one finding `(owner, site)`.
fn rust_holder_run(files: &[(&str, &str)], finding: (&str, &str)) -> Value {
    let p = Project::new();
    for (path, text) in files {
        p.put(path, text);
    }
    p.finding(finding.0, finding.1);
    let names: Vec<String> = ["rs40", "rs80", "rs120"]
        .iter()
        .flat_map(|d| {
            [
                "vgdb-1-leak.txt",
                "vgdb-2-blocks.txt",
                "vgdb-3-who.txt",
                "vgdb-4-who.txt",
            ]
            .map(|n| format!("{d}/{n}"))
        })
        .collect();
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    let paths = p.fixture("valgrind-vgdb", &names);
    let v = heap_graph(&p, &paths);
    assert_eq!(v["gate"], "pass", "{v}");
    v
}

const OUTLINE_MANIFEST: &str = "[package]\nname = \"outline-service\"\nversion = \"0.1.0\"\n";

#[test]
fn a_rust_symbol_does_not_tie_to_a_static_of_another_module_prefix_or_crate() {
    // The owner ends with the module path, but the file is src/net/registry.rs: the symbol
    // is `outline_service::registry::REGISTRY`, the static of src/registry.rs.
    let v = rust_holder_run(
        &[
            ("Cargo.toml", OUTLINE_MANIFEST),
            ("src/net/registry.rs", "pub static REGISTRY: u8 = 0;\n"),
        ],
        ("src.net.registry.REGISTRY", "src/net/registry.rs:1"),
    );
    assert_eq!(tied_of(&v), serde_json::json!([]), "{v}");
    // The same file in a crate of another name (a dependency or another workspace member).
    let v = rust_holder_run(
        &[
            (
                "Cargo.toml",
                "[package]\nname = \"depcrate\"\nversion = \"0.1.0\"\n",
            ),
            ("src/registry.rs", "pub static REGISTRY: u8 = 0;\n"),
        ],
        ("src.registry.REGISTRY", "src/registry.rs:1"),
    );
    assert_eq!(tied_of(&v), serde_json::json!([]), "{v}");
    // No manifest says which crate the file belongs to.
    let v = rust_holder_run(
        &[("src/registry.rs", "pub static REGISTRY: u8 = 0;\n")],
        ("src.registry.REGISTRY", "src/registry.rs:1"),
    );
    assert_eq!(tied_of(&v), serde_json::json!([]), "{v}");
    // The right file of the right crate, in a workspace member, ties (positive control).
    let v = rust_holder_run(
        &[
            ("crates/svc/Cargo.toml", OUTLINE_MANIFEST),
            (
                "crates/svc/src/registry.rs",
                "pub static REGISTRY: u8 = 0;\n",
            ),
        ],
        (
            "crates.svc.src.registry.REGISTRY",
            "crates/svc/src/registry.rs:1",
        ),
    );
    assert_eq!(tied_of(&v).as_array().map(Vec::len), Some(1), "{v}");
}

#[test]
fn a_bare_c_name_ties_only_to_the_one_file_that_defines_it() {
    // Two files define a file-scope `registry`; the finding is one of them.
    let p = Project::new();
    p.put("src/hold.c", C_HOLD);
    p.put("src/other.c", "static int registry;\n");
    p.finding("src.hold.registry", "src/hold.c:3");
    let files = p.fixture("valgrind-vgdb", &VGDB);
    let v = heap_graph(&p, &files);
    assert_eq!(v["gate"], "pass", "{v}");
    assert_eq!(tied_of(&v), serde_json::json!([]), "{v}");
    // The flagged file is not the one that defines the name.
    let q = Project::new();
    q.put("src/hold.c", C_HOLD);
    q.put("src/a.c", "int unrelated;\n");
    q.finding("src.a.registry", "src/a.c:1");
    let files = q.fixture("valgrind-vgdb", &VGDB);
    let v = heap_graph(&q, &files);
    assert_eq!(tied_of(&v), serde_json::json!([]), "{v}");
}
