//! Hostile numbers and honest readers of `dla summarize`: counts of `u64::MAX` must give a
//! JSON answer (never a panic), a folded heaptrack file needs its cost type said, and a
//! heaptrack report that elides stacks says so.

use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use serde_json::{Value, json};

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

/// Exit code, raw stdout and stderr of a command.
fn run(project: &Path, args: &[&str]) -> (i32, String, String) {
    let out = dla(project).args(args).output().unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Runs and requires a JSON answer on stdout with exit 0 or 2: a panic exits 101 with
/// nothing on stdout.
fn answer(project: &Path, args: &[&str]) -> (i32, Value) {
    let (code, text, err) = run(project, args);
    assert!(
        code == 0 || code == 2,
        "{args:?} exited {code}: stdout {text:?} stderr {err}"
    );
    let v: Value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{e}: {text}"));
    (code, v)
}

struct Project {
    dir: tempfile::TempDir,
}

impl Project {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let (code, text, err) = run(dir.path(), &["run", "init", "--session-id", "t"]);
        assert_eq!(code, 0, "{text} {err}");
        Self { dir }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    fn put(&self, rel: &str, text: &str) -> String {
        let p = self.path().join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
        rel.to_owned()
    }

    fn run_texts(&self) -> Vec<(PathBuf, String)> {
        fn walk(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
            for e in fs::read_dir(dir).unwrap().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if let Ok(t) = fs::read_to_string(&p) {
                    out.push((p, t));
                }
            }
        }
        let mut v = Vec::new();
        walk(&self.path().join(".dla"), &mut v);
        v
    }
}

fn fixture(backend: &str, name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/summarize")
            .join(backend)
            .join(name),
    )
    .unwrap()
}

const MAX: u64 = u64::MAX;

#[test]
fn a_jcmd_histogram_with_u64_max_counts_gives_json_not_a_panic() {
    let p = Project::new();
    let h = |n: u64, b: u64| {
        format!(
            " num     #instances         #bytes  class name (module)\n-------------------------------------------------------\n   1: {n:>14} {b:>14}  app.Item (app@1)\n"
        )
    };
    let files: Vec<String> = [(MAX, MAX), (100, 200), (300, 400)]
        .iter()
        .enumerate()
        .map(|(i, (n, b))| p.put(&format!("in/h{i}.txt"), &h(*n, *b)))
        .collect();
    let mut args = vec!["summarize", "type-histogram-diff"];
    args.extend(files.iter().map(String::as_str));
    answer(p.path(), &args);
    // And the other way round: a growth to the maximum.
    let files: Vec<String> = [(1, 1), (100, 200), (MAX, MAX)]
        .iter()
        .enumerate()
        .map(|(i, (n, b))| p.put(&format!("in/g{i}.txt"), &h(*n, *b)))
        .collect();
    let mut args = vec!["summarize", "type-histogram-diff"];
    args.extend(files.iter().map(String::as_str));
    answer(p.path(), &args);
}

#[test]
fn harness_types_with_u64_max_counts_give_json_not_a_panic() {
    let p = Project::new();
    let line = |x: u64, count: f64, bytes: f64| {
        json!({"dla_harness": 1, "phase": "checkpoint", "pid": 1, "runtime": "jvm", "x": x as f64,
            "t_ms": x, "gc_forced": true,
            "types": [{"name": "app.Item", "count": count, "bytes": bytes}]})
        .to_string()
    };
    let big = 1.8446744073709552e19;
    let text = [
        line(0, big, big),
        line(1000, 100.0, 200.0),
        line(2000, 300.0, 400.0),
    ]
    .join("\n");
    p.put("in/types.jsonl", &text);
    answer(
        p.path(),
        &["summarize", "type-histogram-diff", "in/types.jsonl"],
    );
}

#[test]
fn byte_counts_that_overflow_u64_are_summed_saturating() {
    // Two events of the same stack, each u64::MAX bytes: the sum must not panic.
    let p = Project::new();
    let group = |bytes: u64| {
        json!({
            "frames": [{"function": "shop::fill", "file": "src/fill.rs", "line": 3, "module": null}],
            "live_bytes": bytes, "live_blocks": 1, "total_bytes": bytes,
            "root_path": [], "kind": null
        })
    };
    let profile = |bytes: Vec<u64>| {
        json!({"source": "dhat", "timestamp_ms": 1, "sampled": false,
            "groups": bytes.into_iter().map(group).collect::<Vec<_>>()})
        .to_string()
    };
    let files: Vec<String> = [vec![MAX, MAX], vec![MAX, MAX], vec![1]]
        .into_iter()
        .enumerate()
        .map(|(i, b)| p.put(&format!("in/p{i}.json"), &profile(b)))
        .collect();
    let mut args = vec!["summarize", "alloc-stack-diff"];
    args.extend(files.iter().map(String::as_str));
    answer(p.path(), &args);
    // The first capture at the maximum and a small value later: the delta is clamped.
    let files: Vec<String> = [vec![MAX], vec![5], vec![9]]
        .into_iter()
        .enumerate()
        .map(|(i, b)| p.put(&format!("in/q{i}.json"), &profile(b)))
        .collect();
    let mut args = vec!["summarize", "alloc-stack-diff"];
    args.extend(files.iter().map(String::as_str));
    answer(p.path(), &args);
}

#[test]
fn shim_stack_groups_with_u64_max_bytes_give_json_not_a_panic() {
    let p = Project::new();
    let dump = |x: u64, a: u64, b: u64| {
        format!(
            "{{\"dla_harness\":1,\"phase\":\"exit\",\"pid\":1,\"argv\":[\"x\"],\"exe\":\"/nonexistent/x\",\"runtime\":\"native\",\"x\":{x},\"t_ms\":1,\"gc_forced\":false,\"values\":{{}},\"groups\":{{}},\"raw_sites\":{{\"0x1\":{{\"live_bytes\":{a},\"live_blocks\":1,\"allocs\":1,\"frames\":[{{\"a\":\"0x10\",\"m\":0,\"o\":\"0x10\"}}]}},\"0x2\":{{\"live_bytes\":{b},\"live_blocks\":1,\"allocs\":1,\"frames\":[{{\"a\":\"0x20\",\"m\":0,\"o\":\"0x20\"}}]}}}},\"modules\":[{{\"path\":\"/nonexistent/x\",\"bias\":\"0x0\"}}]}}\n"
        )
    };
    let files: Vec<String> = [(1, MAX, MAX), (2, 5, 5), (3, MAX, 7)]
        .iter()
        .enumerate()
        .map(|(i, (x, a, b))| p.put(&format!("in/d{i}.jsonl"), &dump(*x, *a, *b)))
        .collect();
    let mut args = vec![
        "summarize",
        "type-histogram-diff",
        "--backend",
        "shim-stack-groups",
    ];
    args.extend(files.iter().map(String::as_str));
    answer(p.path(), &args);
}

#[test]
fn a_bare_folded_heaptrack_file_is_refused_until_it_says_leaked() {
    let p = Project::new();
    p.put("src/leak.c", &"/* line */\n".repeat(30));
    let text = fixture("heaptrack", "leaks1.txt");
    assert!(!text.is_empty());
    let folded = "main (leak.c);add_session (leak.c); 3024000\n";
    p.put("in/stacks.folded", folded);
    let (code, v) = answer(
        p.path(),
        &["summarize", "alloc-stack-diff", "in/stacks.folded"],
    );
    assert_eq!(code, 2, "{v}");
    assert_eq!(v["error"]["code"], "unsupported-format", "{v}");
    assert!(
        v["error"]["fix"]
            .as_str()
            .unwrap()
            .contains("--flamegraph-cost-type leaked"),
        "{v}"
    );
    // Said to be leaked bytes in the name: read, and the limitation names the assumption.
    p.put("in/leaked.folded", folded);
    let (code, v) = answer(
        p.path(),
        &["summarize", "alloc-stack-diff", "in/leaked.folded"],
    );
    assert_eq!(code, 0, "{v}");
    let all: String = p.run_texts().into_iter().map(|(_, t)| t).collect();
    assert!(all.contains("no cost-type marker"), "{all}");
    // No site is invented from a frame that has no line.
    assert!(!all.contains("src/leak.c:1\""), "{all}");
}

#[test]
fn a_report_that_elides_stacks_says_so_and_an_unmerged_one_lists_every_site() {
    let p = Project::new();
    p.put("src/a.c", &"/* line */\n".repeat(10));
    let merged: Vec<String> = (0..3)
        .map(|i| p.put(&format!("in/m{i}.txt"), &fixture("heaptrack", "elided.txt")))
        .collect();
    let mut args = vec!["summarize", "alloc-stack-diff", "--backend", "heaptrack"];
    args.extend(merged.iter().map(String::as_str));
    answer(p.path(), &args);
    let all: String = p.run_texts().into_iter().map(|(_, t)| t).collect();
    assert!(all.contains("other places"), "{all}");
    assert!(all.contains("elides 1 group"), "{all}");
    // The unmerged print (heaptrack_print -m0 -s 50, which the adapter asks for) holds all
    // seven sites and no elision.
    let q = Project::new();
    q.put("src/a.c", &"/* line */\n".repeat(10));
    let un: Vec<String> = (0..3)
        .map(|i| {
            q.put(
                &format!("in/u{i}.txt"),
                &fixture("heaptrack", "unmerged.txt"),
            )
        })
        .collect();
    let mut args = vec!["summarize", "alloc-stack-diff", "--backend", "heaptrack"];
    args.extend(un.iter().map(String::as_str));
    answer(q.path(), &args);
    let all: String = q.run_texts().into_iter().map(|(_, t)| t).collect();
    assert!(!all.contains("elides"), "{all}");
    for site in ["s1", "s2", "s7"] {
        assert!(
            all.contains(&format!("\"{site}\"")),
            "{site} missing: {all}"
        );
    }
}
