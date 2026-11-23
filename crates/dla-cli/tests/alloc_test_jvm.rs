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
    // A failed command explains itself with the tails of the harness logs, whichever step it was.
    assert_eq!(code, 0, "{args:?}: {text}\n{}", harness_log_tails(project));
    assert_eq!(json["ok"], true, "{text}\n{}", harness_log_tails(project));
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
    prepared_citing(cap, true)
}

/// [`prepared`]; with `cite_growth` false the finding cites only the workload evidence, so the
/// harness is about it only as a command that runs the project's own workload.
fn prepared_citing(
    cap: Option<u32>,
    cite_growth: bool,
) -> (tempfile::TempDir, PathBuf, String, String) {
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
    let trend = ok(&project, &["trend", "S1"]);
    // The growth evidence of the trend of the harness: the finding cites it, which is what makes
    // the harness about the finding (an alloc-test says nothing about a finding on a harness
    // that neither targets it nor measured it).
    let growth = trend["evidence"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(",");
    let cited = if cite_growth {
        format!("{workload},{growth}")
    } else {
        workload.clone()
    };
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
            &cited,
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
fn an_unbound_alloc_test_of_a_command_that_runs_the_projects_workload_can_refute_a_finding() {
    if skip() {
        return;
    }
    // The harness runs the project's own workload (command mode): it is about every finding of
    // the run, so its evidence, bound or not, is cited as before.
    let (_dir, project, f, _) = prepared_citing(Some(5000), false);
    let a = ok(&project, &["alloc-test", "H1"]);
    assert_eq!(a["favors"], "non-leak", "{a}");
    let r = ok(
        &project,
        &[
            "ledger",
            "refute",
            &f,
            "--evidence",
            a["evidence"].as_str().unwrap(),
        ],
    );
    assert_eq!(r["to"], "refuted", "{r}");
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
    driver_run_step(warm, iterations, "50", unit)
}

/// `driver_run` with the units per checkpoint (`--step`) given.
fn driver_run_step(
    warm: Option<&str>,
    iterations: &str,
    step: &str,
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
            step,
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
    assert_eq!(code, 0, "{out}\n{}", harness_log_tails(&project));
    (dir, project, run)
}

/// The last lines of every log of the project's harnesses, for a failed run to explain itself
/// (the run's own error only names the log).
fn harness_log_tails(project: &Path) -> String {
    let mut out = String::new();
    let mut logs = Vec::new();
    let mut stack = vec![project.join(".dla")];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let path = e.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|x| x == "log")
                && path.parent().is_some_and(|p| p.ends_with("logs"))
            {
                logs.push(path);
            }
        }
    }
    logs.sort();
    for log in logs {
        let text = fs::read_to_string(&log).unwrap_or_default();
        let lines: Vec<&str> = text.lines().collect();
        let tail = &lines[lines.len().saturating_sub(40)..];
        out.push_str(&format!(
            "--- tail of {} ({} lines) ---\n{}\n",
            log.display(),
            lines.len(),
            tail.join("\n")
        ));
    }
    if out.is_empty() {
        out.push_str("(no harness log found under .dla)");
    }
    out
}

/// Both runs of one measurement: the non-heap bytes of an idle driver with the agent's warm-up
/// switched off (cold) and on (warm), 20 checkpoints each. The class count of both is flat after
/// the first point: the workload loads nothing.
fn warm_up_pair() -> (Vec<f64>, Vec<f64>) {
    let (cold, cold_classes) = idle_driver(Some("0"), "20");
    let (warm, warm_classes) = idle_driver(None, "20");
    for classes in [&cold_classes, &warm_classes] {
        assert!(
            classes.windows(2).skip(1).all(|w| w[1] - w[0] <= 4.0),
            "{classes:?}"
        );
    }
    (cold, warm)
}

/// Whether the warm-up shows in these pairs, taken together (the values are summed over the
/// pairs). Two relations, both on the rise of the non-heap bytes over the first checkpoints:
///
/// * From the first reading to point 15 the warm run rises by less than 0.85 of the cold run:
///   the warm-up has loaded and compiled the checkpoints' own code before the first reading, so
///   the first step and the early growth are mostly gone. Measured here, with four harness runs
///   at once, on JDK 21 and JDK 25: warm over cold 0.28 to 0.54 in 88 pairs, two cold runs
///   against each other 0.88 to 1.15 in 64 pairs. Measured elsewhere: up to 0.58 on Linux and
///   0.66 and 0.74 on the macOS runner, and 0.72 to 1.38 for two cold runs. The line lies between
///   the two ranges measured here but inside the one measured elsewhere. A first pair that
///   passes ends the test (a pass is not measured twice); only a first pair that fails is
///   measured again, which guards a working warm-up that reads near the line, and the two are
///   judged together.
/// * On Linux, where the cold run rises by 250 KB or more from point 1 to point 15, the warm run
///   rises by less than 0.75 of that (measured 0.29 to 0.64 with four runs at once). This one
///   gives way first when the machine is loaded far beyond that: with sixteen copies at once
///   (the warm-up stops after 3 s and warms less) it failed in 4 of 16 runs even with the second
///   pair, while the 0.85 relation held. Not claimed on macOS, where that rise is the steady
///   drift of about 25 KB per point and the warm run rises as much (cold 336 to 347 KB, warm 327
///   to 364 KB), nor where it is under 250 KB (a JVM told it has many processors compiles the
///   checkpoints' code early too).
fn warm_up_shows(pairs: &[(Vec<f64>, Vec<f64>)]) -> Result<(), String> {
    let total = |pick: fn(&(Vec<f64>, Vec<f64>)) -> f64| -> f64 { pairs.iter().map(pick).sum() };
    let (cold, warm) = (total(|p| p.0[15] - p.0[0]), total(|p| p.1[15] - p.1[0]));
    if warm >= 0.85 * cold {
        return Err(format!(
            "the rise from the first reading to point 15 is {warm} bytes with the warm-up and \
             {cold} without it (sums over {} pair(s)): not under 0.85 of the cold rise",
            pairs.len()
        ));
    }
    let (cold, warm) = (total(|p| p.0[15] - p.0[1]), total(|p| p.1[15] - p.1[1]));
    if cfg!(target_os = "linux") && cold >= 2.5e5 * pairs.len() as f64 && warm >= 0.75 * cold {
        return Err(format!(
            "the rise from point 1 to point 15 is {warm} bytes with the warm-up and {cold} \
             without it (sums over {} pair(s)): not under 0.75 of the cold rise",
            pairs.len()
        ));
    }
    Ok(())
}

#[test]
fn the_agents_warm_up_takes_most_of_its_own_early_non_heap_growth_off_an_idle_driver() {
    if skip() {
        return;
    }
    // How much CPU the JVM gets moves the readings (the warm-up stops after 3 s), so a pair that
    // does not show the warm-up is measured again, and the two are judged together.
    let mut pairs = vec![warm_up_pair()];
    let Err(first) = warm_up_shows(&pairs) else {
        return;
    };
    eprintln!("the first pair does not show the warm-up, measuring a second: {first}");
    pairs.push(warm_up_pair());
    if let Err(both) = warm_up_shows(&pairs) {
        panic!("{first}; with a second pair: {both}; cold and warm series: {pairs:?}");
    }
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
    // headroom for the gc purge and the pre-GC twin to see (a collection leaves them equal; the
    // young generation of a JVM told it has many processors holds less garbage, about 4 MB here).
    assert!(pre > post + 2.0e6, "pre-GC {pre} against post-GC {post}");
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
    declare_target(&project, &f);
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

/// Makes harness H1 declare `finding` as a target, as `dla harness gen --probe` would have if the
/// finding had existed then: an alloc-test says nothing about a finding on a harness that neither
/// targets it nor measured something tied to it.
fn declare_target(project: &Path, finding: &str) {
    for e in fs::read_dir(project.join(".dla")).unwrap() {
        let path = e.unwrap().path().join("harness/H1/harness.json");
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let mut doc: Value = serde_json::from_str(&text).unwrap();
        doc["targets"] = serde_json::json!([finding]);
        fs::write(&path, doc.to_string()).unwrap();
    }
}

/// An idle driver's finding, bound and ready for `alloc-test`.
fn bound_non_heap_finding(project: &Path, run: &Value) -> String {
    bound_finding(project, run, true)
}

/// A finding bound to the run's workload evidence, with the trend of the run (`dla trend S1`)
/// computed first or not. With no trend, the thread counts are required columns.
fn bound_finding(project: &Path, run: &Value, trend: bool) -> String {
    let workload = run["evidence"][0].as_str().unwrap().to_owned();
    if trend {
        ok(project, &["trend", "S1"]);
    }
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
    declare_target(project, &f);
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
        // A JVM told it has many processors may read the idle driver as a non-leak, whose `next`
        // confirms the warm-up sibling: that is right and is no confirmation of the finding.
        if a.get("caution").is_some() {
            let next = a["next"].to_string();
            assert!(!next.contains("ledger confirm"), "extra {extra}: {next}");
        }
    }
}

#[test]
fn a_driver_that_keeps_daemon_threads_is_not_blamed_on_the_agents_drift() {
    if skip() {
        return;
    }
    // The unit starts daemon threads that never end: the thread counts rise at every point,
    // and the small non-heap rise is not the agent's own checkpoint work. One thread per five
    // units, ten per point and about two hundred in all: macOS refuses a process more than about
    // two thousand threads (the JVM then dies with "unable to create native thread"), which a
    // thread for every unit of every point would reach.
    let unit = "if (i % 5 == 0) { Thread t = new Thread(() -> { try { Thread.sleep(600000L); } catch (InterruptedException e) { } }); t.setDaemon(true); t.start(); }";
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

/// The full result of the latest alloc-test of the project (stdout carries no limitations).
fn latest_result(project: &Path) -> Value {
    let mut best: Option<(u32, PathBuf)> = None;
    for e in fs::read_dir(project.join(".dla")).unwrap() {
        let dir = e.unwrap().path().join("alloc");
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for f in rd {
            let path = f.unwrap().path();
            let n = path
                .file_stem()
                .and_then(|s| s.to_str())
                .and_then(|s| s.strip_prefix('A'))
                .and_then(|s| s.parse::<u32>().ok());
            if let Some(n) = n
                && best.as_ref().is_none_or(|(m, _)| n > *m)
            {
                best = Some((n, path));
            }
        }
    }
    let (_, path) = best.expect("an alloc result");
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

/// The thread columns of a full result: their plateau verdicts and net rises.
fn thread_columns(full: &Value) -> String {
    full["columns"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["name"].as_str().is_some_and(|n| n.ends_with("threads")))
        .map(|c| format!("{} {}", c["name"], c["plateau"]))
        .collect::<Vec<_>>()
        .join("; ")
}

#[test]
fn a_driver_that_starts_a_daemon_thread_every_tenth_unit_keeps_the_leak_reading_and_names_the_thread_count()
 {
    if skip() {
        return;
    }
    // With no trend the thread counts are required columns. One thread per ten units at a step of
    // five is a thread every other checkpoint: the plateau test cannot judge the count (not every
    // delta is a rise), but its net rise over the extra window is about ten threads.
    let unit = "if (i % 10 == 0) { Thread t = new Thread(() -> { try { Thread.sleep(600000L); } catch (InterruptedException e) { } }); t.setDaemon(true); t.start(); }";
    let (_dir, project, run) = driver_run_step(None, "20", "5", unit);
    let f = bound_finding(&project, &run, false);
    let a = ok(&project, &["alloc-test", "H1", "--finding", &f]);
    let text = a.to_string();
    let full = latest_result(&project);
    // The leak reading stays where jvm.nonheap_used keeps drifting. Where it plateaus (a JVM told
    // it has many processors compiles faster and settles sooner) the thread counts, which do
    // not plateau, hold the verdict back as inconclusive: the capture in `next` decides, and
    // never a caution that blames the agent.
    assert!(
        a["favors"] == "leak" || a["favors"] == "inconclusive",
        "{text} {}",
        thread_columns(&full)
    );
    assert!(a.get("caution").is_none(), "{text}");
    assert!(
        !text.contains("agent's own checkpoints add by themselves"),
        "{text}"
    );
    if a["favors"] == "leak" {
        // The limitation (in the full result, not on stdout) names the thread count as rising.
        let limitations = full["limitations"].to_string();
        assert!(
            limitations.contains("rising over the extra window")
                && (limitations.contains("jvm.threads") || limitations.contains("os.threads")),
            "the thread count is named: {limitations}"
        );
    }
    let next = a["next"].to_string();
    assert!(next.contains("--capture nonheap"), "{a}");
    assert!(next.contains("the thread count keeps rising"), "{next}");
    assert!(!next.contains("--step <larger>"), "{next}");
}

#[test]
fn an_idle_driver_without_a_trend_is_never_a_leak_verdict_and_names_no_thread_count() {
    if skip() {
        return;
    }
    // The compiler threads the JVM starts and stops by itself are no thread leak. A trend of the
    // harness would drop the thread counts that pass its gate, and an alloc-test writes a trend
    // of its own, so each window gets a fresh project: the thread counts are required columns in
    // both (the test asserts they are in the result).
    for extra in ["20", "100"] {
        let (_dir, project, run) = idle_run(None, "20");
        let f = bound_finding(&project, &run, false);
        let a = ok(
            &project,
            &["alloc-test", "H1", "--finding", &f, "--extra", extra],
        );
        let text = a.to_string();
        let full = latest_result(&project);
        let columns = thread_columns(&full);
        assert!(
            columns.contains("jvm.threads") && columns.contains("os.threads"),
            "extra {extra}: the thread counts are tested: {columns}"
        );
        assert_ne!(a["favors"], "leak", "extra {extra}: {text} {columns}");
        let limitations = full["limitations"].to_string();
        assert!(
            !limitations.contains("keep rising over the extra window")
                && !limitations.contains("keeps rising over the extra window"),
            "extra {extra}: {limitations} {columns}"
        );
        let next = a["next"].to_string();
        // A JVM told it has many processors may read the idle driver as a non-leak, whose `next`
        // confirms the warm-up sibling: that is right and is no confirmation of the finding.
        if a.get("caution").is_some() {
            assert!(!next.contains("ledger confirm"), "extra {extra}: {next}");
        }
        assert!(
            !next.contains("the thread count keeps rising"),
            "extra {extra}: {next}"
        );
    }
}
