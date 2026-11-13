//! `dla alloc-test` on a real JVM, the non-heap route: a loop that warms up (hot loops for the
//! JIT and a bounded set of classes defined on the way) shows non-heap growth that the
//! alloc-test judges a plateau, so the root is refuted and `N-WARM` confirmed with the printed
//! commands; the same loop without the bound defines a class per iteration and stays a leak.
//! Both skip, printing the reason, when `java`, `javac` or `sh` is missing.

#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
};

use assert_cmd::Command;
use serde_json::Value;

fn has(tool: &str, args: &[&str]) -> bool {
    Std::new(tool)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// True (after printing why) when the test cannot run here.
fn skip() -> bool {
    if has("java", &["-version"]) && has("javac", &["-version"]) && has("sh", &["-c", "true"]) {
        return false;
    }
    eprintln!("skipping: java, javac or sh is not available");
    true
}

fn dla(project: &Path, args: &[&str]) -> (i32, Value, String) {
    dla_env(project, args, &[])
}

fn dla_env(project: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, Value, String) {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.envs(env.iter().copied());
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
    ] {
        c.env_remove(var);
    }
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    (
        o.status.code().unwrap(),
        serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    )
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let (code, json, text) = dla(project, args);
    assert_eq!(code, 0, "{args:?}: {text}");
    assert_eq!(json["ok"], true, "{text}");
    json
}

const BUILD: &str = r#"#!/bin/sh
set -e
cd "$(dirname "$0")"
rm -rf out
mkdir -p out
javac -d out src/zz/*.java
if [ "$1" = "run" ]; then
    java -cp out zz.Main "${2:-2000}"
fi
"#;

const GEN: &str = r#"package zz;

/** A class the loop defines again and again. */
final class Gen {
    static int one() { return 1; }
}
"#;

/// The loop: hot methods for the JIT, and one hidden class defined per iteration while the
/// set holds fewer than `CAP` of them (`CAP` is unbounded for the leak).
fn main_java(cap: Option<u32>) -> String {
    let cap = cap.map_or("Integer.MAX_VALUE".to_owned(), |c| c.to_string());
    format!(
        r#"package zz;

import java.lang.invoke.MethodHandles;
import java.util.ArrayList;
import java.util.List;

public final class Main {{
    static final List<Class<?>> KEPT = new ArrayList<>();
    static final int CAP = {cap};

    static long hot(int k, long seed) {{
        long s = seed;
        for (int i = 0; i < 1000; i++) {{
            s = s * 31 + (i ^ k);
        }}
        return s;
    }}

    public static void main(String[] args) throws Exception {{
        int total = args.length > 0 ? Integer.parseInt(args[0]) : 2000;
        byte[] bytes = Main.class.getResourceAsStream("Gen.class").readAllBytes();
        long sum = 0;
        for (int i = 0; i < total; i++) {{
            if (KEPT.size() < CAP) {{
                KEPT.add(MethodHandles.lookup().defineHiddenClass(bytes, true).lookupClass());
            }}
            sum += switch (i % 4) {{
                case 0 -> hot(0, sum);
                case 1 -> hot(1, sum);
                case 2 -> hot(2, sum);
                default -> hot(3, sum);
            }};
        }}
        System.out.println("done " + KEPT.size() + " " + sum);
    }}
}}
"#
    )
}

/// A project with discovery and one harness run (S1) done, and a non-heap finding with a
/// non-leak sibling under test. Returns the project, the finding and the workload evidence.
fn prepared(cap: Option<u32>) -> (tempfile::TempDir, PathBuf, String, String) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    let src = project.join("src/zz");
    fs::create_dir_all(&src).unwrap();
    fs::write(project.join("build.sh"), BUILD).unwrap();
    fs::write(src.join("Gen.java"), GEN).unwrap();
    fs::write(src.join("Main.java"), main_java(cap)).unwrap();
    ok(&project, &["run", "init", "--session-id", "t"]);
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", "jvm"]);
    ok(&project, &["workload", "discover"]);
    ok(
        &project,
        &["harness", "gen", "--lang", "jvm", "--step", "1000"],
    );
    let run = ok(&project, &["harness", "run", "H1"]);
    let workload = run["evidence"][0].as_str().unwrap().to_owned();
    ok(&project, &["trend", "S1"]);
    // The non-heap hypothesis, the way the route prints it: a finding of the region and an
    // `N-WARM` sibling, then under test.
    let f = ok(
        &project,
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--region",
            "non-heap",
            "--evidence",
            &workload,
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    ok(
        &project,
        &[
            "ledger",
            "add",
            "--sibling-of",
            &f,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM: JIT profiling data and class metadata level off",
            "--refute-if",
            "non-heap keeps rising over the extra window",
            "--evidence",
            &workload,
        ],
    );
    ok(&project, &["ledger", "test", &f, "--evidence", &workload]);
    (dir, project, f, workload)
}

fn column<'a>(a: &'a Value, name: &str) -> &'a Value {
    a["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == name)
        .unwrap_or_else(|| panic!("no column {name}: {a}"))
}

#[test]
fn a_loop_that_warms_up_is_judged_a_plateau_on_the_non_heap_bytes_and_the_root_is_refuted() {
    if skip() {
        return;
    }
    let (_dir, project, f, _) = prepared(Some(5000));
    let a = ok(&project, &["alloc-test", "H1", "--finding", &f]);
    assert_eq!(a["favors"], "non-leak", "{a}");
    assert!(
        a["supports"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s == "N-WARM"),
        "{a}"
    );
    // The byte column decides, not the thread count the region used to fall back to.
    let nonheap = column(&a, "jvm.nonheap_used");
    assert_eq!(nonheap["plateau"], "plateau", "{a}");
    assert!(
        a["columns"]
            .as_array()
            .unwrap()
            .iter()
            .all(|c| c["name"] != "jvm.threads"),
        "{a}"
    );
    // The signs are reported next to it.
    for sign in [
        "jvm.classes_loaded",
        "jvm.classes_unloaded",
        "jvm.jit_seconds",
    ] {
        column(&a, sign);
    }
    // The printed commands refute the root and confirm the sibling.
    let mut done = Vec::new();
    for n in a["next"].as_array().unwrap() {
        let words: Vec<&str> = n.as_str().unwrap().split_whitespace().collect();
        if words.first() == Some(&"dla")
            && words.get(1) == Some(&"ledger")
            && matches!(words.get(2), Some(&("refute" | "test" | "confirm")))
        {
            ok(&project, &words[1..]);
            done.push(words[2].to_owned());
        }
    }
    assert_eq!(done, ["refute", "test", "confirm"], "{}", a["next"]);
}

#[test]
fn a_loop_that_defines_a_class_per_iteration_stays_a_leak() {
    if skip() {
        return;
    }
    let (_dir, project, f, _) = prepared(None);
    let a = ok(&project, &["alloc-test", "H1", "--finding", &f]);
    assert_eq!(a["favors"], "leak", "{a}");
    let nonheap = column(&a, "jvm.nonheap_used");
    assert_eq!(nonheap["plateau"], "growth-continues", "{a}");
    assert_eq!(
        column(&a, "jvm.classes_loaded")["plateau"],
        "growth-continues"
    );
    // No refute of the root is offered for a leak verdict.
    assert!(
        !a["next"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n.as_str().unwrap().contains(&format!("confirm {f}.n1"))),
        "{a}"
    );
}

/// The non-heap bytes and the class counts of an idle single-process driver (its unit does
/// nothing), over `iterations` checkpoints, with the agent's warm-up rounds set to `warm`
/// (`None`: its default). Returns the series' two columns.
fn idle_driver(warm: Option<&str>, iterations: &str) -> (Vec<f64>, Vec<f64>) {
    let (_dir, project, run) = idle_run(warm, iterations);
    let rel = run["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|a| a.as_str().filter(|a| a.contains("series/S")))
        .unwrap();
    let series: Value =
        serde_json::from_str(&fs::read_to_string(project.join(rel)).unwrap()).unwrap();
    let index = |name: &str| {
        series["columns"]
            .as_array()
            .unwrap()
            .iter()
            .position(|c| c["name"] == name)
            .unwrap_or_else(|| panic!("no column {name}"))
    };
    let (nonheap, classes) = (index("jvm.nonheap_used"), index("jvm.classes_loaded"));
    let values = |i: usize| -> Vec<f64> {
        series["points"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["values"][i].as_f64().unwrap())
            .collect()
    };
    (values(nonheap), values(classes))
}

/// The project of an idle single-process driver after its harness run (H1, S1).
fn idle_run(warm: Option<&str>, iterations: &str) -> (tempfile::TempDir, PathBuf, Value) {
    driver_run(
        warm,
        iterations,
        "if (key == -1) { throw new Exception(); }",
    )
}

/// The project of a single-process driver whose unit runs `unit` (Java statements), after its
/// harness run (H1, S1).
fn driver_run(
    warm: Option<&str>,
    iterations: &str,
    unit: &str,
) -> (tempfile::TempDir, PathBuf, Value) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    fs::create_dir_all(project.join("src/zz")).unwrap();
    fs::write(
        project.join("src/zz/App.java"),
        "package zz;\npublic final class App { public static int f(int k) { return k; } }\n",
    )
    .unwrap();
    ok(&project, &["run", "init", "--session-id", "t"]);
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", "jvm"]);
    ok(&project, &["workload", "discover"]);
    let g = ok(
        &project,
        &[
            "harness",
            "gen",
            "--lang",
            "jvm",
            "--driver",
            "--package",
            "zz",
            "--step",
            "50",
            "--iterations",
            iterations,
        ],
    );
    let edit = project.join(g["summary"]["edit"].as_str().unwrap());
    let text = fs::read_to_string(&edit).unwrap();
    let text = text
        .replace(
            "throw new UnsupportedOperationException(\"DLA-HOOK setup has not been filled in\");",
            "",
        )
        .replace(
            "throw new UnsupportedOperationException(\"DLA-HOOK unit has not been filled in\");",
            unit,
        );
    assert!(!text.contains("has not been filled in"));
    fs::write(&edit, text).unwrap();
    let env: Vec<(&str, &str)> = warm.map(|w| ("DLA_WARM_ROUNDS", w)).into_iter().collect();
    let (code, run, out) = dla_env(&project, &["harness", "run", "H1", "--mode", "fresh"], &env);
    assert_eq!(code, 0, "{out}");
    (dir, project, run)
}

#[test]
fn the_agents_warm_up_takes_most_of_its_own_early_non_heap_growth_off_an_idle_driver() {
    if skip() {
        return;
    }
    let (cold, cold_classes) = idle_driver(Some("0"), "20");
    let (warm, warm_classes) = idle_driver(None, "20");
    // Nothing is loaded by the workload: the class count is flat after the first point.
    for classes in [&cold_classes, &warm_classes] {
        assert!(
            classes.windows(2).skip(1).all(|w| w[1] - w[0] <= 4.0),
            "{classes:?}"
        );
    }
    // The checkpoints compile and load their own code over the first points; run before
    // the first one, that no longer lands in the series.
    let rise = |v: &[f64]| v[15] - v[1];
    assert!(
        rise(&warm) < 0.75 * rise(&cold),
        "warm {} against cold {}: {warm:?} {cold:?}",
        rise(&warm),
        rise(&cold)
    );
}

const GARBAGE: &str = r#"package zz;

public final class Main {
    static volatile Object sink;

    public static void main(String[] args) {
        // About 200 MB of garbage that is unreachable when the exit record is taken.
        for (int i = 0; i < 200_000; i++) {
            sink = new byte[1000];
        }
        sink = null;
        System.out.println("done");
    }
}
"#;

#[test]
fn a_command_mode_exit_record_still_reads_the_heap_before_the_collection() {
    if skip() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    let src = project.join("src/zz");
    fs::create_dir_all(&src).unwrap();
    fs::write(project.join("build.sh"), BUILD).unwrap();
    fs::write(src.join("Main.java"), GARBAGE).unwrap();
    ok(&project, &["run", "init", "--session-id", "t"]);
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", "jvm"]);
    ok(&project, &["workload", "discover"]);
    ok(
        &project,
        &["harness", "gen", "--lang", "jvm", "--step", "1000"],
    );
    let run = ok(&project, &["harness", "run", "H1"]);
    let rel = run["artifacts"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|a| a.as_str().filter(|a| a.contains("series/S")))
        .unwrap();
    let series: Value =
        serde_json::from_str(&fs::read_to_string(project.join(rel)).unwrap()).unwrap();
    let index = |name: &str| {
        series["columns"]
            .as_array()
            .unwrap()
            .iter()
            .position(|c| c["name"] == name)
            .unwrap_or_else(|| panic!("no column {name}: {series}"))
    };
    let (post, pre) = (index("jvm.heap_used"), index("jvm.heap_used.pre_gc"));
    let last = series["points"].as_array().unwrap().last().unwrap();
    let (post, pre) = (
        last["values"][post].as_f64().unwrap(),
        last["values"][pre].as_f64().unwrap(),
    );
    // The agent's own warm-up must not collect before the pre-GC reading: that would leave no
    // headroom for the gc purge and the pre-GC twin to see.
    assert!(pre > post + 5.0e6, "pre-GC {pre} against post-GC {post}");
}

#[test]
fn a_small_non_heap_rise_on_an_idle_driver_is_never_a_leak_verdict_and_prints_no_refute_or_confirm()
{
    if skip() {
        return;
    }
    // The agent's own checkpoints make the non-heap bytes drift by some KB per point; the
    // result of a workload that does nothing must not be read as a leak, however the drift falls.
    let (_dir, project, run) = idle_run(None, "20");
    let workload = run["evidence"][0].as_str().unwrap().to_owned();
    ok(&project, &["trend", "S1"]);
    let f = ok(
        &project,
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--region",
            "non-heap",
            "--evidence",
            &workload,
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    ok(
        &project,
        &[
            "ledger",
            "add",
            "--sibling-of",
            &f,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM: JIT profiling data and class metadata level off",
            "--refute-if",
            "non-heap keeps rising over the extra window",
            "--evidence",
            &workload,
        ],
    );
    ok(&project, &["ledger", "test", &f, "--evidence", &workload]);
    let a = ok(&project, &["alloc-test", "H1", "--finding", &f]);
    assert_ne!(a["favors"], "leak", "{a}");
    if a.get("caution").is_some() {
        let next = a["next"].to_string();
        assert!(
            !next.contains("ledger refute") && !next.contains("ledger confirm"),
            "{next}"
        );
    }
}

/// An idle driver's finding, bound and ready for `alloc-test`.
fn bound_non_heap_finding(project: &Path, run: &Value) -> String {
    let workload = run["evidence"][0].as_str().unwrap().to_owned();
    ok(project, &["trend", "S1"]);
    let f = ok(
        project,
        &[
            "ledger",
            "add",
            "--origin",
            "hypothesis",
            "--region",
            "non-heap",
            "--evidence",
            &workload,
        ],
    )["id"]
        .as_str()
        .unwrap()
        .to_owned();
    ok(
        project,
        &[
            "ledger",
            "add",
            "--sibling-of",
            &f,
            "--origin",
            "hypothesis",
            "--hypothesis",
            "N-WARM: JIT profiling data and class metadata level off",
            "--refute-if",
            "non-heap keeps rising over the extra window",
            "--evidence",
            &workload,
        ],
    );
    ok(project, &["ledger", "test", &f, "--evidence", &workload]);
    f
}

#[test]
fn an_idle_driver_is_never_a_leak_verdict_over_a_long_extra_window_either() {
    if skip() {
        return;
    }
    // The agent's drift grows with the window: at extra 100 the line is the measured drift of
    // every extra checkpoint with a margin, not a flat 1 MiB.
    let (_dir, project, run) = idle_run(None, "20");
    let f = bound_non_heap_finding(&project, &run);
    for extra in ["20", "100"] {
        let a = ok(
            &project,
            &["alloc-test", "H1", "--finding", &f, "--extra", extra],
        );
        assert_ne!(a["favors"], "leak", "extra {extra}: {a}");
        let next = a["next"].to_string();
        assert!(!next.contains("ledger confirm"), "extra {extra}: {next}");
    }
}

#[test]
fn a_driver_that_keeps_daemon_threads_is_not_blamed_on_the_agents_drift() {
    if skip() {
        return;
    }
    // The unit starts daemon threads that never end: the thread counts rise at every point,
    // and the small non-heap rise is not the agent's own checkpoint work.
    let unit = "for (int n = 0; n < 3; n++) { Thread t = new Thread(() -> { try { Thread.sleep(600000L); } catch (InterruptedException e) { } }); t.setDaemon(true); t.start(); }";
    let (_dir, project, run) = driver_run(None, "20", unit);
    let f = bound_non_heap_finding(&project, &run);
    let a = ok(&project, &["alloc-test", "H1", "--finding", &f]);
    let text = a.to_string();
    assert!(a.get("caution").is_none(), "{text}");
    assert!(
        !text.contains("agent's own checkpoints add by themselves"),
        "{text}"
    );
    let next = a["next"].to_string();
    assert!(next.contains("--capture nonheap"), "{a}");
    assert!(!next.contains("--step <larger>"), "{next}");
}
