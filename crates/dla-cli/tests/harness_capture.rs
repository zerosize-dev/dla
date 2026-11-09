//! `dla harness run --capture` and `--tool` through the binary on copies of the benchmark
//! fixtures: raw captures of a JVM driver (heap dumps that pass the heap-graph gate), the
//! JNI boundary of `jni-order-codec` (native stack groups resolved to `lib.rs:40`), and the
//! native tools (valgrind, heaptrack) with their missing-tool refusal.
//!
//! Each test skips, printing the reason, when a tool it needs is not installed.
#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use serde_json::Value;

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench/fixtures")
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap().flatten() {
        let (src, dst) = (e.path(), to.join(e.file_name()));
        if e.file_type().unwrap().is_dir() {
            copy_dir(&src, &dst);
        } else {
            fs::copy(&src, &dst).unwrap();
        }
    }
}

fn on_path(tool: &str) -> bool {
    std::env::var_os("PATH")
        .is_some_and(|p| std::env::split_paths(&p).any(|d| d.join(tool).is_file()))
}

/// True (after printing why) when one of `tools` is missing.
fn skip_without(tools: &[&str]) -> bool {
    for t in tools {
        if !on_path(t) {
            eprintln!("skipping: {t} is not on PATH");
            return true;
        }
    }
    false
}

struct Out {
    code: i32,
    json: Value,
    text: String,
}

fn dla_env(project: &Path, args: &[&str], path: Option<&std::ffi::OsStr>) -> Out {
    let mut c = Command::cargo_bin("dla").unwrap();
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
    if let Some(p) = path {
        c.env("PATH", p);
    }
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    assert!(
        text.len() <= dla_core::MAX_OUTPUT_BYTES,
        "output over the bound: {} bytes: {text}",
        text.len()
    );
    Out {
        code: o.status.code().unwrap(),
        json: serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    }
}

/// Like [`dla`] with one more environment variable.
fn dla_var(project: &Path, args: &[&str], key: &str, value: &Path) -> Out {
    let mut c = Command::cargo_bin("dla").unwrap();
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
    c.env(key, value);
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    Out {
        code: o.status.code().unwrap(),
        json: serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    }
}

fn dla(project: &Path, args: &[&str]) -> Out {
    dla_env(project, args, None)
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let o = dla(project, args);
    assert_eq!(o.code, 0, "{args:?}: {}", o.text);
    assert_eq!(o.json["ok"], true, "{}", o.text);
    o.json
}

/// A project copy of `fixture` with a run, consent for tiers 1 and 2, and discovery for
/// `lang` done.
fn prepared(fixture: &str, lang: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    copy_dir(&fixtures_root().join(fixture).join("project"), &project);
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", lang]);
    ok(&project, &["workload", "discover", "--lang", lang]);
    (dir, project, run_dir)
}

fn manifest(run_dir: &Path, id: &str) -> Value {
    let p = run_dir.join("harness").join(id).join("harness.json");
    serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
}

/// The manifest's capture files of `kind`, in sequence order, as project-relative paths.
fn capture_paths(m: &Value, run_dir: &Path, project: &Path, kind: &str) -> Vec<String> {
    let mut files: Vec<(u64, String)> = m["captures"]
        .as_array()
        .map(Vec::as_slice)
        .unwrap_or_default()
        .iter()
        .filter(|c| c["kind"] == kind)
        .map(|c| {
            let abs = run_dir.join(c["path"].as_str().unwrap());
            assert!(abs.is_file(), "{} is listed but missing", abs.display());
            assert_eq!(
                fs::metadata(&abs).unwrap().len(),
                c["bytes"].as_u64().unwrap()
            );
            assert!(c["captured_at"].as_str().unwrap().ends_with('Z'));
            let rel = abs
                .strip_prefix(project)
                .unwrap()
                .to_string_lossy()
                .into_owned();
            (c["seq"].as_u64().unwrap(), rel)
        })
        .collect();
    files.sort();
    files.into_iter().map(|(_, p)| p).collect()
}

/// Fills the two hooks of a generated JVM driver.
fn fill_hooks(driver: &Path, fields: &str, setup: &str, unit: &str) {
    let text = fs::read_to_string(driver).unwrap();
    let text = text
        .replace(
            "        throw new UnsupportedOperationException(\"DLA-HOOK setup has not been filled in\");",
            setup,
        )
        .replace(
            "        throw new UnsupportedOperationException(\"DLA-HOOK unit has not been filled in\");",
            unit,
        )
        .replace("    private DlaLoop() {}", &format!("    private DlaLoop() {{}}\n\n{fields}"));
    assert!(!text.contains("has not been filled in"));
    fs::write(driver, text).unwrap();
}

#[test]
fn jvm_driver_captures_dumps_that_pass_the_heap_graph_gate() {
    if skip_without(&["java", "javac", "jar"]) {
        return;
    }
    let (_d, project, run_dir) = prepared("jvm-audit-chain", "jvm");
    let gen_out = ok(
        &project,
        &[
            "harness",
            "gen",
            "--lang",
            "jvm",
            "--driver",
            "--package",
            "gateway",
        ],
    );
    let edit = gen_out["summary"]["edit"].as_str().unwrap();
    fill_hooks(
        &project.join(edit),
        "    static Gateway gateway;\n    static long clock;",
        "        gateway = new Gateway(new Handlers(new Ledger(64, 500_000)), line -> { });",
        r#"        clock += 1_000 + (key % 7) * 100;
        String path = key % 3 == 0 ? "/balance" : key % 3 == 1 ? "/statement" : "/transfer";
        String payload = Ledger.account(key % 64);
        if (path.equals("/transfer")) {
            payload = payload + "," + Ledger.account((key * 5 + 1) % 64) + "," + (100 + key % 900);
        }
        StringBuilder body = new StringBuilder(payload);
        while (body.length() < 384) {
            body.append(' ');
        }
        gateway.handle("client-" + (key % 12), path, body.toString().getBytes(java.nio.charset.StandardCharsets.UTF_8), clock);"#,
    );
    let run = ok(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--mode",
            "fresh",
            "--capture",
            "heap,histogram,jfr,nonheap",
        ],
    );
    let s = &run["summary"];
    // Costs are in the output.
    assert!(s["capture"]["files"].as_u64().unwrap() >= 10, "{s}");
    assert!(s["capture"]["bytes"].as_u64().unwrap() > 1_000_000);
    let heap = &s["capture"]["kinds"]["heap"];
    assert!(
        heap[0].as_u64().unwrap() >= 3,
        "three or more dumps: {heap}"
    );
    assert!(
        heap[2].as_u64().unwrap() < 60_000,
        "the dump time is milliseconds: {heap}"
    );
    // The commands that read the files are printed (or listed in captures.json).
    let shown = s["summarize"].as_array().map_or(0, Vec::len);
    let more = s["more_commands"].is_string();
    assert!(shown >= 1 && (shown >= 3 || more), "{s}");
    assert!(
        s["summarize"][0]
            .as_str()
            .unwrap()
            .starts_with("dla summarize heap-graph ")
    );
    let listed: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("harness/H1/captures.json")).unwrap(),
    )
    .unwrap();
    let ops: Vec<&str> = listed["summarize"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap().split_whitespace().nth(2).unwrap())
        .collect();
    for op in [
        "heap-graph",
        "type-histogram-diff",
        "alloc-stack-diff",
        "non-heap",
    ] {
        assert!(ops.contains(&op), "{op} is missing from {ops:?}");
    }
    // The manifest lists every file with its size and time.
    let m = manifest(&run_dir, "H1");
    let heaps = capture_paths(&m, &run_dir, &project, "heap");
    assert!(heaps.len() >= 3, "{heaps:?}");
    let histograms = capture_paths(&m, &run_dir, &project, "histogram");
    assert!(
        histograms.len() >= 9,
        "one histogram per round: {}",
        histograms.len()
    );
    assert_eq!(capture_paths(&m, &run_dir, &project, "jfr").len(), 1);
    assert!(capture_paths(&m, &run_dir, &project, "nonheap").len() >= 3);
    // The gate passes on the dumps of the one process, with the right holder.
    let mut args = vec!["summarize", "heap-graph"];
    args.extend(heaps.iter().take(3).map(String::as_str));
    let g = ok(&project, &args);
    assert_eq!(g["gate"], "pass", "{g}");
    assert_eq!(g["passed"][0], "static:gateway.RequestContext.latest");
    // The histogram diff reads the text files.
    let mut args = vec!["summarize", "type-histogram-diff"];
    args.extend(histograms.iter().take(3).map(String::as_str));
    let h = dla(&project, &args);
    assert_eq!(h.code, 0, "{}", h.text);
    // The series is the usual one.
    assert_eq!(s["series"][0], "S1");
    // Coverage rows say what ran.
    let cov: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("coverage.json")).unwrap()).unwrap();
    assert_eq!(
        cov["workers"]["jvm"]["backends"]["harness-heap"]["status"],
        "ran"
    );
}

#[test]
fn jvm_command_mode_dumps_at_exit_of_the_selected_points_only() {
    if skip_without(&["java", "javac", "jar", "sh"]) {
        return;
    }
    let (_d, project, run_dir) = prepared("jvm-audit-chain", "jvm");
    ok(&project, &["harness", "gen", "--lang", "jvm"]);
    let run = ok(
        &project,
        &[
            "harness",
            "run",
            "H1",
            "--capture",
            "heap,histogram",
            "--dumps",
            "3",
        ],
    );
    assert_eq!(run["summary"]["capture"]["kinds"]["heap"][0], 3, "{run}");
    let m = manifest(&run_dir, "H1");
    // Histograms at every point, dumps at three; the unit-test JVM of build.sh left nothing.
    assert_eq!(capture_paths(&m, &run_dir, &project, "histogram").len(), 9);
    let heaps = capture_paths(&m, &run_dir, &project, "heap");
    assert_eq!(heaps.len(), 3);
    let pids: std::collections::BTreeSet<&str> = m["captures"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|c| c["label"] == "p1-native")
        .map(|c| c["path"].as_str().unwrap().rsplit('/').nth(1).unwrap())
        .collect();
    assert_eq!(pids.len(), 1, "one process per point is kept: {pids:?}");
}

#[test]
fn a_jni_library_gets_the_shim_and_its_native_stack_resolves_to_the_rust_line() {
    if skip_without(&["java", "javac", "jar", "cargo", "cc", "addr2line", "sh"]) {
        return;
    }
    let m2 = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".m2/repository/org/jetbrains/kotlin/kotlin-compiler-embeddable");
    if !m2.is_dir() && !on_path("kotlinc") {
        eprintln!("skipping: no Kotlin compiler (kotlinc, or the embeddable compiler in ~/.m2)");
        return;
    }
    let (_d, project, run_dir) = prepared("jni-order-codec", "jvm");
    let probe = ok(&project, &["probe"]);
    assert_eq!(probe["summary"]["boundaries"][0], "jni:codec");
    let gen_out = ok(&project, &["harness", "gen", "--lang", "jvm"]);
    assert_eq!(gen_out["summary"]["ffi"], "jni:codec");
    let m = manifest(&run_dir, "H1");
    assert_eq!(m["ffi"]["library"], "codec");
    let run_env = m["run"]["env"].as_array().unwrap();
    let env = |k: &str| {
        run_env
            .iter()
            .find(|p| p[0] == k)
            .map(|p| p[1].as_str().unwrap())
    };
    assert_eq!(env("DLA_SHIM_ONLY"), Some("java"));
    assert!(env("LD_PRELOAD").unwrap().ends_with("libdlashim.so"));
    assert_eq!(env("CARGO_PROFILE_RELEASE_DEBUG"), Some("line-tables-only"));
    let run = ok(&project, &["harness", "run", "H1", "--capture", "all"]);
    // The shim's exit records have their own type-histogram-diff command.
    let listed: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("harness/H1/captures.json")).unwrap(),
    )
    .unwrap();
    let shim_cmd = listed["summarize"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .find(|c| c.contains("shim-dump.jsonl"))
        .unwrap_or_else(|| panic!("no shim command in {listed}"));
    assert!(shim_cmd.starts_with("dla summarize type-histogram-diff "));
    let mut args = vec!["summarize", "type-histogram-diff"];
    args.extend(shim_cmd.split_whitespace().skip(3));
    let sg = dla(&project, &args);
    assert_eq!(sg.code, 0, "{}", sg.text);
    assert_eq!(sg.json["backend"], "shim-stack-groups", "{}", sg.text);
    // The leaking native frame, tied to the managed method.
    let ffi = &run["summary"]["ffi"][0];
    assert_eq!(ffi["site"], "native/src/lib.rs:40", "{run}");
    assert_eq!(ffi["method"], "shop.codec.Native.open");
    assert!(ffi["live_bytes"].as_u64().unwrap() > 1_000_000);
    // The series carries the native columns `dla trend` reads.
    let series: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("series/S1.json")).unwrap()).unwrap();
    let names: Vec<&str> = series["columns"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["name"].as_str().unwrap())
        .collect();
    for want in [
        "native.live_bytes",
        "native.live_bytes@native/src/lib.rs:40",
        "fd.socket",
        "fd.file",
    ] {
        assert!(names.contains(&want), "no column {want} in {names:?}");
    }
    let col = series["columns"]
        .as_array()
        .unwrap()
        .iter()
        .position(|c| c["name"] == "native.live_bytes@native/src/lib.rs:40")
        .unwrap();
    assert_eq!(series["columns"][col]["site"], "native/src/lib.rs:40");
    let total = series["columns"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "native.live_bytes")
        .unwrap();
    assert_eq!(
        total["source"], "shim",
        "the figure comes from the shim, not the JVM agent"
    );
    let vals: Vec<f64> = series["points"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["values"][col].as_f64())
        .collect();
    assert!(vals.len() >= 8 && vals.last() > vals.first(), "{vals:?}");
    let t = ok(&project, &["trend", "S1"]);
    let passed = t["passed"].as_array().unwrap();
    assert!(
        passed
            .iter()
            .any(|p| p["column"] == "native.live_bytes@native/src/lib.rs:40"),
        "{t}"
    );
}

#[test]
fn a_capture_kind_that_does_not_exist_is_a_usage_error() {
    let (_d, project, _run) = prepared("jvm-audit-chain", "jvm");
    let o = dla(&project, &["harness", "run", "H1", "--capture", "bogus"]);
    assert_ne!(o.code, 0);
    assert!(
        o.text.contains("bogus") || o.json["error"].is_object(),
        "{}",
        o.text
    );
}

#[test]
fn a_missing_tool_is_a_refusal_with_the_install_command_and_a_coverage_row() {
    if skip_without(&["cc", "make"]) {
        return;
    }
    let (_d, project, run_dir) = prepared("native-frame-decoder", "native");
    ok(&project, &["harness", "gen", "--lang", "native"]);
    // Build and measure once, so the build is done and the tool lookup is the next step.
    ok(&project, &["harness", "run", "H1", "--iterations", "2"]);
    let empty = tempfile::tempdir().unwrap();
    for (tool, hint) in [
        ("valgrind", "apt-get install valgrind"),
        ("heaptrack", "apt-get install heaptrack"),
    ] {
        let o = dla_env(
            &project,
            &["harness", "run", "H1", "--tool", tool],
            Some(empty.path().as_os_str()),
        );
        assert_eq!(o.code, 2, "{}", o.text);
        let e = &o.json["error"];
        assert_eq!(e["code"], "tool-missing", "{}", o.text);
        assert!(e["missing"].as_str().unwrap().contains(tool));
        assert!(e["fix"].as_str().unwrap().contains(hint), "{}", o.text);
        let cov: Value =
            serde_json::from_str(&fs::read_to_string(run_dir.join("coverage.json")).unwrap())
                .unwrap();
        let row = &cov["workers"]["native"]["backends"][tool];
        assert_eq!(row["status"], "missing", "{cov}");
        assert!(row["install_hint"].as_str().unwrap().contains(hint));
    }
}

#[cfg(unix)]
#[test]
fn a_missing_heaptrack_print_is_named_in_the_coverage_row_not_heaptrack() {
    use std::os::unix::fs::PermissionsExt;
    if skip_without(&["cc", "make"]) {
        return;
    }
    let (_d, project, run_dir) = prepared("native-frame-decoder", "native");
    ok(&project, &["harness", "gen", "--lang", "native"]);
    ok(&project, &["harness", "run", "H1", "--iterations", "2"]);
    // A PATH holding a `heaptrack` only.
    let bin = tempfile::tempdir().unwrap();
    let fake = bin.path().join("heaptrack");
    fs::write(&fake, "#!/bin/sh\nexit 0\n").unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    let o = dla_env(
        &project,
        &["harness", "run", "H1", "--tool", "heaptrack"],
        Some(bin.path().as_os_str()),
    );
    assert_eq!(o.code, 2, "{}", o.text);
    let e = &o.json["error"];
    assert_eq!(e["code"], "tool-missing", "{}", o.text);
    assert_eq!(e["missing"], "heaptrack_print", "{}", o.text);
    let cov: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("coverage.json")).unwrap()).unwrap();
    let backends = &cov["workers"]["native"]["backends"];
    assert_eq!(backends["heaptrack_print"]["status"], "missing", "{cov}");
    assert!(
        backends.get("heaptrack").is_none_or(Value::is_null),
        "{cov}"
    );
}

#[test]
fn valgrind_and_heaptrack_leave_reports_for_the_summarizers() {
    if skip_without(&["cc", "make", "valgrind", "heaptrack", "heaptrack_print"]) {
        return;
    }
    let (_d, project, run_dir) = prepared("native-frame-decoder", "native");
    ok(&project, &["harness", "gen", "--lang", "native"]);
    let vg = ok(&project, &["harness", "run", "H1", "--tool", "valgrind"]);
    assert_eq!(vg["summary"]["tool"], "valgrind");
    assert!(
        vg["summary"]["summarize"][0]
            .as_str()
            .unwrap()
            .starts_with("dla summarize heap-graph ")
    );
    assert!(
        vg["summary"]["capture"]["kinds"]["valgrind"][0]
            .as_u64()
            .unwrap()
            >= 2
    );
    let m = manifest(&run_dir, "H1");
    let valgrind_files = capture_paths(&m, &run_dir, &project, "valgrind");
    let xmls: Vec<String> = valgrind_files
        .iter()
        .filter(|p| p.ends_with("valgrind.xml"))
        .cloned()
        .collect();
    assert!(xmls.len() >= 2);
    // One command per op and per input format: the vgdb answers read by heap-graph, the XML
    // by alloc-stack-diff, and nothing mixes the two.
    let cmds: Vec<&str> = vg["summary"]["summarize"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c.as_str().unwrap())
        .collect();
    assert_eq!(cmds.len(), 2, "{cmds:?}");
    assert!(cmds[0].contains("vgdb-1-leak.txt") && !cmds[0].contains(".xml"));
    assert!(cmds[1].starts_with("dla summarize alloc-stack-diff ") && !cmds[1].contains("vgdb"));
    let xml = fs::read_to_string(project.join(&xmls[0])).unwrap();
    assert!(
        xml.contains("<valgrindoutput>") && xml.contains("Leak_"),
        "{xml}"
    );
    assert!(xml.contains("decoder.c"), "frames carry file names");
    let ht = ok(&project, &["harness", "run", "H1", "--tool", "heaptrack"]);
    assert_eq!(ht["summary"]["tool"], "heaptrack");
    let m = manifest(&run_dir, "H1");
    let reports = capture_paths(&m, &run_dir, &project, "heaptrack");
    assert!(reports.len() >= 3, "{reports:?}");
    // The reports and the diff are two formats, so two commands.
    let ht_cmds = ht["summary"]["summarize"].as_array().unwrap();
    assert_eq!(ht_cmds.len(), 2, "{ht_cmds:?}");
    assert!(ht_cmds[0].as_str().unwrap().contains("heaptrack.txt"));
    assert!(!ht_cmds[0].as_str().unwrap().contains("heaptrack-diff"));
    assert!(ht_cmds[1].as_str().unwrap().contains("heaptrack-diff.txt"));
    let diff = reports
        .iter()
        .find(|p| p.ends_with("heaptrack-diff.txt"))
        .unwrap();
    let text = fs::read_to_string(project.join(diff)).unwrap();
    assert!(
        text.contains("MEMORY LEAKS") && text.contains("src/decoder.c:29"),
        "{text}"
    );
    // No series was written by a tool run.
    assert!(vg["summary"]["series"].is_null());
    let cov: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("coverage.json")).unwrap()).unwrap();
    assert_eq!(
        cov["workers"]["native"]["backends"]["valgrind"]["status"],
        "ran"
    );
    assert_eq!(
        cov["workers"]["native"]["backends"]["heaptrack"]["status"],
        "ran"
    );
}

#[test]
fn a_tool_on_a_driver_harness_is_refused_with_the_fix() {
    if skip_without(&["java", "javac", "jar"]) {
        return;
    }
    let (_d, project, _run) = prepared("jvm-audit-chain", "jvm");
    ok(
        &project,
        &[
            "harness",
            "gen",
            "--lang",
            "jvm",
            "--driver",
            "--package",
            "gateway",
        ],
    );
    let o = dla(&project, &["harness", "run", "H1", "--tool", "valgrind"]);
    assert_eq!(o.code, 2, "{}", o.text);
    assert_eq!(o.json["error"]["code"], "invalid-value");
    assert!(
        o.json["error"]["fix"]
            .as_str()
            .unwrap()
            .contains("dla harness gen")
    );
}

/// A project copy with a run, consent and discovery for a C program made of `main.c` and a
/// Makefile, started as `./holdlist --iterations N`.
fn native_project(list_at_tail: bool) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    fs::create_dir_all(project.join("src")).unwrap();
    fs::write(
        project.join("Makefile"),
        "CC ?= cc\nCFLAGS += -std=c11 -O0 -g\nall: holdlist\nholdlist: src/main.c\n\t$(CC) $(CFLAGS) -o $@ $<\nclean:\n\trm -f holdlist\n.PHONY: all clean\n",
    )
    .unwrap();
    let link = if list_at_tail {
        "    if (!registry) { registry = n; } else { struct node *t = registry; while (t->next) t = t->next; t->next = n; }\n"
    } else {
        "    n->next = registry;\n    registry = n;\n"
    };
    let src = format!(
        "#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n\
struct node {{ struct node *next; char payload[56]; }};\n\
static struct node *registry;\n\
static void remember(int i) {{\n    struct node *n = calloc(1, sizeof *n);\n    snprintf(n->payload, sizeof n->payload, \"entry-%d\", i);\n{link}}}\n\
int main(int argc, char **argv) {{\n    int iterations = 10;\n    for (int i = 1; i + 1 < argc; i++) {{\n        if (strcmp(argv[i], \"--iterations\") == 0) iterations = atoi(argv[i + 1]);\n    }}\n    for (int i = 0; i < iterations; i++) remember(i);\n    printf(\"held %d\\n\", iterations);\n    return 0;\n}}\n"
    );
    fs::write(project.join("src/main.c"), src).unwrap();
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", "native"]);
    ok(&project, &["workload", "discover", "--lang", "native"]);
    ok(&project, &["harness", "gen", "--lang", "native"]);
    (dir, project, run_dir)
}

fn point_dirs(run_dir: &Path) -> Vec<PathBuf> {
    let out = run_dir.join("harness/H1/out");
    let mut dirs: Vec<PathBuf> = fs::read_dir(out)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("valgrind.xml").is_file())
        .collect();
    dirs.sort();
    dirs
}

#[test]
fn valgrind_leaves_the_vgdb_chain_that_names_the_static_holder() {
    if skip_without(&["cc", "make", "valgrind", "vgdb"]) {
        return;
    }
    let (_d, project, run_dir) = native_project(false);
    let vg = ok(&project, &["harness", "run", "H1", "--tool", "valgrind"]);
    let points = point_dirs(&run_dir);
    assert_eq!(points.len(), 3, "{vg}");
    for p in &points {
        for f in [
            "vgdb-1-leak.txt",
            "vgdb-2-blocks.txt",
            "vgdb-3-who.txt",
            "valgrind.xml",
        ] {
            assert!(p.join(f).is_file(), "{} lacks {f}", p.display());
        }
        let who = fs::read_to_string(p.join("vgdb-3-who.txt")).unwrap();
        assert!(who.contains("data symbol \"registry\""), "{who}");
        // The XML is complete: the process was let go after the questions.
        let xml = fs::read_to_string(p.join("valgrind.xml")).unwrap();
        assert!(xml.trim_end().ends_with("</valgrindoutput>"));
    }
    // The pipes live under the run directory and are gone afterwards.
    assert!(
        fs::read_dir(run_dir.join("tmp"))
            .map(|d| d
                .flatten()
                .all(|e| !e.file_name().to_string_lossy().starts_with("vgdb")))
            .unwrap_or(true)
    );
    // The printed command runs, and ties the holder column to the census finding.
    let cmd = vg["summary"]["summarize"][0].as_str().unwrap();
    assert!(cmd.starts_with("dla summarize heap-graph "), "{cmd}");
    let args: Vec<&str> = cmd.split_whitespace().skip(1).collect();
    let o = dla(&project, &args);
    assert_eq!(o.code, 0, "{}", o.text);
    assert_eq!(o.json["backend"], "valgrind-vgdb", "{}", o.text);
    assert_eq!(o.json["gate"], "pass", "{}", o.text);
    assert_eq!(o.json["passed"][0], "holder:static:registry", "{}", o.text);
    assert_eq!(o.json["walks"][0]["root"], "static registry");
}

#[test]
fn the_vgdb_walk_falls_back_to_the_oldest_block_for_a_list_that_grows_at_the_tail() {
    if skip_without(&["cc", "make", "valgrind", "vgdb"]) {
        return;
    }
    let (_d, project, run_dir) = native_project(true);
    let vg = ok(&project, &["harness", "run", "H1", "--tool", "valgrind"]);
    let points = point_dirs(&run_dir);
    let last = points.last().unwrap();
    let who = fs::read_to_string(last.join("vgdb-3-who.txt")).unwrap();
    assert!(who.contains("data symbol \"registry\""), "{who}");
    // The walk from the newest block runs out of its calls inside the list; the one from the
    // oldest block (the list head) ends at the static in one query, and only it is kept.
    let queries = who.matches("sending command who_points_at").count();
    assert_eq!(queries, 1, "{who}");
    assert!(
        vg["summary"]["limitations"]
            .as_array()
            .is_none_or(|l| l.iter().all(|x| !x.as_str().unwrap().contains("partial"))),
        "{vg}"
    );
}

#[test]
fn a_kotlin_driver_gets_the_stdlib_from_kotlin_home_without_a_hand_edit() {
    if skip_without(&["java", "javac", "jar", "sh"]) {
        return;
    }
    let (_d, project, run_dir) = prepared("jni-order-codec", "jvm");
    ok(&project, &["probe"]);
    // A Kotlin install is only a directory with lib/kotlin-stdlib.jar for this check.
    let home = tempfile::tempdir().unwrap();
    fs::create_dir_all(home.path().join("lib")).unwrap();
    fs::write(home.path().join("lib/kotlin-stdlib.jar"), b"jar").unwrap();
    let o = dla_var(
        &project,
        &[
            "harness",
            "gen",
            "--lang",
            "jvm",
            "--driver",
            "--package",
            "shop.codec",
        ],
        "KOTLIN_HOME",
        home.path(),
    );
    assert_eq!(o.code, 0, "{}", o.text);
    let m = manifest(&run_dir, "H1");
    let jar = home.path().join("lib/kotlin-stdlib.jar");
    let jar = jar.to_string_lossy();
    let run_cp = m["run"]["argv"][2].as_str().unwrap();
    assert!(run_cp.ends_with(&*jar), "{run_cp}");
    let build = m["build"]["argv"][2].as_str().unwrap();
    assert!(build.contains(&*jar), "{build}");
    assert!(
        m["gaps"].to_string().contains("project is Kotlin"),
        "{}",
        m["gaps"]
    );
}
