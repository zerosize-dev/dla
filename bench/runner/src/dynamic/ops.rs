//! The per-operation part of the fixed policy: capture, summarize, check, confirm.
//!
//! It runs after the growth steps of [`super::analyse`] and reads only the JSON that `dla`
//! prints. It never names a backend: the adapters come from `dla summarize --list`, the
//! commands come from what `dla harness run` prints.
//!
//! 1. `dla summarize --list`.
//! 2. For every harness that grew: a JVM harness runs again with `--capture all`; a rust or
//!    native harness runs again with `--tool valgrind` and `--tool heaptrack` (a tool whose
//!    adapters are listed; a missing one refuses with `tool-missing` and a coverage row).
//! 3. Every `dla summarize` command a capture run printed (or listed in the capture manifest
//!    when the output was cut) runs once.
//! 4. Every summarize call is checked: stdout within 2048 bytes, `raw` present with a
//!    matching sha256, evidence ids in the ledger, every evidence result within 2048 bytes,
//!    a `tie` reason on every tied column. A violation fails the fixture.
//! 5. The leak confirmations held back by [`super::test_candidate`] run now: a summarize
//!    growth evidence that ties a passing column to the chosen finding is cited as well, so
//!    the confirmation is dump backed.

use std::{collections::BTreeSet, fs};

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{Fx, Out, View, strings};

/// Bound for the stdout of a command and for the result of an evidence record.
pub(super) const LIMIT: usize = 2048;

/// Most summarize commands run per fixture.
const MAX_COMMANDS: usize = 24;

/// A leak confirmation held back until the summarize steps ran.
#[derive(Debug, Clone)]
pub(super) struct Pending {
    /// The finding to confirm.
    pub finding: String,
    /// The growth evidence of the trend that passed its column.
    pub evidence: String,
}

/// One adapter of `dla summarize --list`.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Adapter {
    pub runtime: String,
    pub backend: String,
    pub available: bool,
}

/// Reads the adapters from `dla summarize --list`.
fn list_adapters(fx: &mut Fx) -> Vec<Adapter> {
    let out = fx.call(&["summarize", "--list"]);
    parse_adapters(&out.json)
}

fn parse_adapters(v: &Value) -> Vec<Adapter> {
    v["adapters"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|x| Adapter {
                    runtime: x["runtime"].as_str().unwrap_or("").to_owned(),
                    backend: x["backend"].as_str().unwrap_or("").to_owned(),
                    available: x["available"].as_bool().unwrap_or(false),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The native tools to run a harness under: those with a listed native adapter, listed or
/// not as available (a missing tool must leave its coverage row).
fn native_tools(adapters: &[Adapter]) -> Vec<(&'static str, bool)> {
    ["valgrind", "heaptrack"]
        .into_iter()
        .filter_map(|t| {
            let mine: Vec<&Adapter> = adapters
                .iter()
                .filter(|a| a.runtime == "native" && a.backend.starts_with(t))
                .collect();
            (!mine.is_empty()).then(|| (t, mine.iter().any(|a| a.available)))
        })
        .collect()
}

/// Splits a command line the way the harness prints it: spaces separate words, single or
/// double quotes keep a word together.
pub(super) fn split_command(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut open = false;
    for c in line.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => cur.push(c),
            (None, '\'' | '"') => {
                quote = Some(c);
                open = true;
            }
            (None, c) if c.is_whitespace() => {
                if open || !cur.is_empty() {
                    words.push(std::mem::take(&mut cur));
                    open = false;
                }
            }
            (None, c) => cur.push(c),
        }
    }
    if open || !cur.is_empty() {
        words.push(cur);
    }
    words
}

/// The summarize commands a `harness run` printed: the list in its summary, or the full list
/// of the capture manifest when the summary says more exist.
fn printed_commands(fx: &Fx, run: &Out) -> Vec<String> {
    let summary = &run.json["summary"];
    let mut cmds = strings(&summary["summarize"]);
    if let Some(more) = summary["more_commands"].as_str()
        && let Some(rel) = more
            .split_whitespace()
            .find(|w| w.ends_with("captures.json"))
        && let Ok(dir) = fx.run_dir()
        && let Ok(text) = fs::read_to_string(dir.join(rel))
        && let Ok(v) = serde_json::from_str::<Value>(&text)
    {
        let full = strings(&v["summarize"]);
        if full.len() >= cmds.len() {
            cmds = full;
        }
    }
    cmds
}

/// Step 2: the capture runs. Returns the commands they printed.
fn capture(fx: &mut Fx, harness: &str, runtime: &str, adapters: &[Adapter]) -> Vec<String> {
    let mut cmds = Vec::new();
    match runtime {
        "jvm" => {
            fx.row.captures.push(format!("{harness}:capture-all"));
            let out = fx.call_quiet(&["harness", "run", harness, "--capture", "all"]);
            if out.ok {
                let n = printed_commands(fx, &out);
                fx.step(format!(
                    "{harness} --capture all: {} summarize command(s)",
                    n.len()
                ));
                cmds.extend(n);
            } else {
                let msg = format!("harness run {harness} --capture all: {}", out.message());
                fx.step(msg.clone());
                fx.row.errors.push(msg);
            }
        }
        "rust" | "native" => {
            for (tool, listed_available) in native_tools(adapters) {
                fx.row.captures.push(format!("{harness}:tool-{tool}"));
                let out = fx.call_quiet(&["harness", "run", harness, "--tool", tool]);
                if out.ok {
                    let n = printed_commands(fx, &out);
                    fx.step(format!(
                        "{harness} --tool {tool}: {} summarize command(s)",
                        n.len()
                    ));
                    cmds.extend(n);
                } else if out.code() == "tool-missing" {
                    fx.step(format!(
                        "{harness} --tool {tool}: tool missing (listed {}), coverage row written",
                        if listed_available {
                            "available"
                        } else {
                            "unavailable"
                        }
                    ));
                } else {
                    let msg = format!("harness run {harness} --tool {tool}: {}", out.message());
                    fx.step(msg.clone());
                    fx.row.errors.push(msg);
                }
            }
        }
        _ => {}
    }
    cmds
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The violations of the output rules for one summarize call. `view` is the ledger after the
/// call. Pure, so it is unit tested.
pub(super) fn check_call(
    label: &str,
    out: &Out,
    view: &View,
    read_raw: &dyn Fn(&str) -> Option<Vec<u8>>,
) -> Vec<String> {
    let mut bad = Vec::new();
    if out.bytes > LIMIT {
        bad.push(format!(
            "{label}: stdout is {} bytes, over {LIMIT}",
            out.bytes
        ));
    }
    if out.json.is_null() {
        bad.push(format!("{label}: stdout is not JSON (exit {})", out.exit));
        return bad;
    }
    if !out.ok || is_coverage_gap(&out.json) {
        return bad;
    }
    match (
        out.json["raw"]["path"].as_str(),
        out.json["raw"]["sha256"].as_str(),
    ) {
        (Some(path), Some(sha)) => match read_raw(path) {
            None => bad.push(format!("{label}: raw file {path} does not exist")),
            Some(bytes) if sha256_hex(&bytes) != sha => {
                bad.push(format!(
                    "{label}: raw file {path} does not match its sha256"
                ));
            }
            Some(_) => {}
        },
        _ => bad.push(format!("{label}: no raw path and sha256")),
    }
    for id in out.evidence() {
        let Some(ev) = view.evidence.get(&id) else {
            bad.push(format!("{label}: evidence {id} is not in the ledger"));
            continue;
        };
        let size = serde_json::to_string(&ev.result).map_or(0, |s| s.len());
        if size > LIMIT {
            bad.push(format!(
                "{label}: result of {id} is {size} bytes, over {LIMIT}"
            ));
        }
        for col in ev.result["passed"].as_array().into_iter().flatten() {
            let tied = col.get("finding").is_some_and(|f| !f.is_null())
                || col["candidates"].as_array().is_some_and(|c| !c.is_empty());
            if tied && !valid_tie(&col["tie"]) {
                bad.push(format!(
                    "{label}: {id} ties column {} without a tie reason",
                    col["column"].as_str().unwrap_or("?")
                ));
            }
        }
    }
    bad
}

/// An auto-selected backend whose tool is missing: the call succeeds with a `coverage_gap`
/// list, no evidence and no raw file.
fn is_coverage_gap(json: &Value) -> bool {
    json["ok"].as_bool() == Some(true)
        && json["coverage_gap"]
            .as_array()
            .is_some_and(|g| !g.is_empty())
        && json["evidence"].as_array().is_none_or(Vec::is_empty)
}

/// The findings a passing column of the call's growth evidence is tied to, with the evidence
/// ids the call wrote.
/// A tie names its rule (holder, site or type) and the key that matched.
fn valid_tie(tie: &Value) -> bool {
    matches!(tie["rule"].as_str(), Some("holder" | "site" | "type"))
        && tie["key"].as_str().is_some_and(|k| !k.is_empty())
}

fn tied_findings(out: &Out, view: &View) -> Option<(BTreeSet<String>, Vec<String>)> {
    if !out.ok {
        return None;
    }
    let growth = out.json["growth_evidence"].as_str()?;
    let ev = view.evidence.get(growth)?;
    if ev.result["gate"].as_str() != Some("pass") {
        return None;
    }
    let mut set = BTreeSet::new();
    for col in ev.result["passed"].as_array().into_iter().flatten() {
        if !valid_tie(&col["tie"]) {
            continue;
        }
        if let Some(f) = col["finding"].as_str() {
            set.insert(f.to_owned());
        }
        set.extend(strings(&col["candidates"]));
    }
    (!set.is_empty()).then(|| (set, out.evidence()))
}

/// Steps 1 to 5. `grew` holds the harnesses with a passing column (id, runtime).
pub(super) fn run(fx: &mut Fx, grew: &[(String, String)]) {
    let pending = std::mem::take(&mut fx.pending);
    let mut tied: Vec<(BTreeSet<String>, Vec<String>)> = Vec::new();
    if !grew.is_empty() {
        let adapters = list_adapters(fx);
        let mut cmds: Vec<String> = Vec::new();
        for (h, rt) in grew {
            for c in capture(fx, h, rt, &adapters) {
                if !cmds.contains(&c) {
                    cmds.push(c);
                }
            }
        }
        if cmds.len() > MAX_COMMANDS {
            fx.step(format!(
                "{} summarize commands printed, the first {MAX_COMMANDS} run",
                cmds.len()
            ));
            cmds.truncate(MAX_COMMANDS);
        }
        for cmd in &cmds {
            if let Some(t) = summarize(fx, cmd) {
                tied.push(t);
            }
        }
    }
    for p in pending {
        confirm(fx, &p, &tied);
    }
}

/// Runs one printed command and checks it; returns what it tied.
fn summarize(fx: &mut Fx, cmd: &str) -> Option<(BTreeSet<String>, Vec<String>)> {
    let words = split_command(cmd);
    if words.first().map(String::as_str) != Some("dla")
        || words.get(1).map(String::as_str) != Some("summarize")
    {
        fx.row
            .errors
            .push(format!("not a summarize command: {cmd}"));
        return None;
    }
    let args: Vec<&str> = words[1..].iter().map(String::as_str).collect();
    let op = args.get(1).copied().unwrap_or("?").to_owned();
    let out = fx.call_quiet(&args);
    let view = fx.view();
    let run_dir = fx.run_dir().unwrap_or_default();
    let read = |rel: &str| fs::read(run_dir.join(rel)).ok();
    let backend = out.json["backend"].as_str().unwrap_or("-").to_owned();
    let label = format!("summarize {op}");
    for m in check_call(&label, &out, &view, &read) {
        let msg = format!("summarize check: {m}");
        if !fx.row.failures.contains(&msg) {
            fx.row.failures.push(msg);
        }
    }
    fx.row.max_stdout = fx.row.max_stdout.max(out.bytes);
    if is_coverage_gap(&out.json) {
        let missing: Vec<&str> = out.json["coverage_gap"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|g| g["backend"].as_str())
            .collect();
        fx.step(format!(
            "{op}: coverage gap, backend {} missing",
            missing.join(",")
        ));
        None
    } else if out.ok {
        fx.row.ops.push(format!("{op}:{backend}"));
        for id in out.evidence() {
            if !fx.row.summarize_evidence.contains(&id) {
                fx.row.summarize_evidence.push(id);
            }
        }
        let t = tied_findings(&out, &view);
        fx.step(format!(
            "{op} via {backend}: gate {}, tied {}",
            out.json["gate"].as_str().unwrap_or("-"),
            t.as_ref().map_or_else(
                || "none".to_owned(),
                |(s, _)| s.iter().cloned().collect::<Vec<_>>().join(",")
            )
        ));
        t
    } else {
        let msg = format!("{op}: {}", out.message());
        fx.step(format!("summarize {msg}"));
        fx.row.errors.push(format!("dla summarize {msg}"));
        None
    }
}

/// Step 5 for one held-back confirmation.
fn confirm(fx: &mut Fx, p: &Pending, tied: &[(BTreeSet<String>, Vec<String>)]) {
    let mut ids: Vec<String> = vec![p.evidence.clone()];
    for (set, ev) in tied {
        if set.contains(&p.finding) {
            for id in ev {
                if !ids.contains(id) {
                    ids.push(id.clone());
                }
            }
        }
    }
    let with_ops = ids.len() > 1;
    let mut c = fx.call_quiet(&[
        "ledger",
        "confirm",
        &p.finding,
        "--evidence",
        &ids.join(","),
    ]);
    if !c.ok && with_ops {
        fx.step(format!(
            "{} confirm with summarize evidence refused ({}): retry with the trend alone",
            p.finding,
            c.code()
        ));
        c = fx.call_quiet(&["ledger", "confirm", &p.finding, "--evidence", &p.evidence]);
        if c.ok {
            fx.row.errors.push(format!(
                "dla ledger confirm {}: the summarize evidence was refused",
                p.finding
            ));
        }
    }
    if c.ok {
        fx.step(format!(
            "{} confirmed{}",
            p.finding,
            if with_ops {
                " citing summarize evidence"
            } else {
                ""
            }
        ));
    } else {
        fx.row
            .errors
            .push(format!("dla ledger confirm {}: {}", p.finding, c.message()));
        fx.step(format!("{} confirm refused ({})", p.finding, c.code()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn out(json: Value, bytes: usize) -> Out {
        Out {
            ok: json["ok"].as_bool().unwrap_or(false),
            json,
            bytes,
            exit: 0,
        }
    }

    fn view_with(id: &str, result: Value) -> View {
        let mut v = View::default();
        v.order.push(id.to_owned());
        v.evidence.insert(
            id.to_owned(),
            super::super::Ev {
                kind: "growth-series".into(),
                authority: "live".into(),
                result,
            },
        );
        v
    }

    fn good(sha: &str) -> Value {
        json!({"ok": true, "backend": "b", "evidence": ["E1"], "growth_evidence": "E1",
               "raw": {"path": "x.json", "sha256": sha}})
    }

    #[test]
    fn commands_split_on_spaces_and_quotes() {
        assert_eq!(
            split_command("dla summarize non-heap a.txt 'b c.txt' \"d e\" ''"),
            [
                "dla",
                "summarize",
                "non-heap",
                "a.txt",
                "b c.txt",
                "d e",
                ""
            ]
        );
        assert!(split_command("  ").is_empty());
    }

    #[test]
    fn a_clean_call_passes_every_check() {
        let sha = sha256_hex(b"raw");
        let v = view_with(
            "E1",
            json!({"gate": "pass", "passed": [
            {"column": "c", "finding": "F1", "tie": {"rule": "site", "key": "a:1"}},
            {"column": "d"}]}),
        );
        let o = out(good(&sha), 900);
        let read = |_: &str| Some(b"raw".to_vec());
        assert!(check_call("s", &o, &v, &read).is_empty());
        let (set, ev) = tied_findings(&o, &v).unwrap();
        assert_eq!(set.into_iter().collect::<Vec<_>>(), ["F1"]);
        assert_eq!(ev, ["E1"]);
    }

    #[test]
    fn each_violation_is_named() {
        let sha = sha256_hex(b"raw");
        let v = view_with(
            "E1",
            json!({"gate": "pass", "passed": [
            {"column": "c", "candidates": ["F1"]}]}),
        );
        let read = |_: &str| Some(b"other".to_vec());
        let o = out(good(&sha), 3000);
        let bad = check_call("s", &o, &v, &read);
        assert_eq!(bad.len(), 3, "{bad:?}");
        assert!(bad[0].contains("3000 bytes"));
        assert!(bad[1].contains("sha256"));
        assert!(bad[2].contains("without a tie reason"));
        // A missing raw file, a missing raw object and an unknown evidence id.
        let none = |_: &str| None;
        assert!(check_call("s", &out(good(&sha), 10), &v, &none)[0].contains("does not exist"));
        let mut j = good(&sha);
        j["raw"] = Value::Null;
        j["evidence"] = json!(["E9"]);
        let bad = check_call("s", &out(j, 10), &v, &read);
        assert_eq!(bad.len(), 2, "{bad:?}");
        assert!(bad[0].contains("no raw"));
        assert!(bad[1].contains("E9 is not in the ledger"));
    }

    #[test]
    fn an_oversized_evidence_result_fails() {
        let sha = sha256_hex(b"raw");
        let v = view_with("E1", json!({"gate": "pass", "pad": "x".repeat(2100)}));
        let read = |_: &str| Some(b"raw".to_vec());
        let bad = check_call("s", &out(good(&sha), 10), &v, &read);
        assert_eq!(bad.len(), 1, "{bad:?}");
        assert!(bad[0].contains("result of E1"));
    }

    #[test]
    fn a_refusal_is_checked_for_size_and_json_only() {
        let v = View::default();
        let read = |_: &str| None;
        let refusal = json!({"ok": false, "error": {"code": "tool-missing"}});
        assert!(check_call("s", &out(refusal.clone(), 100), &v, &read).is_empty());
        assert_eq!(check_call("s", &out(refusal, 5000), &v, &read).len(), 1);
        let none = Out {
            ok: false,
            json: Value::Null,
            bytes: 4,
            exit: 101,
        };
        assert!(check_call("s", &none, &v, &read)[0].contains("not JSON"));
    }

    #[test]
    fn a_missing_tool_in_auto_mode_is_a_coverage_gap_not_a_violation() {
        let gap = json!({"ok": true, "backend": null, "evidence": [],
            "coverage_gap": [{"backend": "b", "reason": "tool missing"}]});
        assert!(is_coverage_gap(&gap));
        let v = View::default();
        let read = |_: &str| None;
        assert!(check_call("s", &out(gap, 300), &v, &read).is_empty());
        // Evidence without raw is still a violation.
        let bad = json!({"ok": true, "evidence": ["E1"], "coverage_gap": [{"backend": "b"}]});
        assert!(!is_coverage_gap(&bad));
    }

    #[test]
    fn a_failed_gate_ties_nothing() {
        let sha = sha256_hex(b"raw");
        let v = view_with(
            "E1",
            json!({"gate": "fail", "passed": [
            {"column": "c", "finding": "F1", "tie": {"rule": "site"}}]}),
        );
        assert!(tied_findings(&out(good(&sha), 10), &v).is_none());
    }

    /// A stand-in dla: lists one adapter, prints one summarize command for the capture run and
    /// answers the summarize call with a ledger record, like the real one.
    #[cfg(unix)]
    const FAKE_DLA: &str = r#"
. ./fake.env
shift 2
echo "$*" >> "$FAKE_LOG"
R=.dla/R1
case "$1 $2" in
"summarize --list")
  echo '{"adapters":[{"runtime":"jvm","backend":"b","available":true}],"ok":true}';;
"harness run")
  if [ -n "$FAKE_CRASH_HARNESS" ]; then echo "thread main panicked" >&2; exit 101; fi
  echo '{"ok":true,"summary":{"summarize":["dla summarize type-histogram-diff a.txt \"b c.txt\" d.txt"]}}';;
"summarize type-histogram-diff")
  mkdir -p $R/histograms
  printf raw > $R/histograms/H1.json
  tie='"tie":{"rule":"type","key":"X"},'
  [ -n "$FAKE_NO_TIE" ] && tie=''
  echo '{"type":"evidence","evidence_id":"E1","kind":"histogram-diff","authority":"live","result":{}}' >> $R/ledger.jsonl
  echo '{"type":"evidence","evidence_id":"E2","kind":"growth-series","authority":"live","result":{"gate":"pass","passed":[{"column":"type:X",'"$tie"'"finding":"F1"}]}}' >> $R/ledger.jsonl
  echo '{"ok":true,"backend":"b","command":"summarize type-histogram-diff","evidence":["E1","E2"],"gate":"pass","growth_evidence":"E2","raw":{"path":"histograms/H1.json","sha256":"'"$FAKE_SHA"'"},"pad":"'"$FAKE_PAD"'"}';;
"ledger confirm")
  if [ -n "$FAKE_CRASH_CONFIRM" ]; then echo "thread main panicked" >&2; exit 101; fi
  echo '{"ok":true}';;
*) echo '{"ok":true}';;
esac
"#;

    #[cfg(unix)]
    fn drive_fake(env: &[(&str, &str)]) -> (FixtureRow, String) {
        use std::os::unix::fs::PermissionsExt;
        let t = tempfile::tempdir().unwrap();
        let bin = t.path().join("dla");
        fs::write(&bin, format!("#!/bin/sh\n{FAKE_DLA}")).unwrap();
        fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
        let project = t.path().join("project");
        fs::create_dir_all(project.join(".dla/R1")).unwrap();
        fs::write(project.join(".dla/current"), "R1\n").unwrap();
        fs::write(project.join(".dla/R1/ledger.jsonl"), "").unwrap();
        let log = t.path().join("calls.log");
        // The stand-in reads its settings from a file in the project.
        let mut settings = format!(
            "FAKE_LOG='{}'\nFAKE_SHA='{}'\n",
            log.display(),
            sha256_hex(b"raw")
        );
        for (k, v) in env {
            settings.push_str(&format!("{k}='{v}'\n"));
        }
        fs::write(project.join("fake.env"), settings).unwrap();
        let mut fx = Fx {
            plan: super::super::Plan {
                entry: crate::model::Entry::Repository,
                runtimes: 1,
                artifacts: Vec::new(),
            },
            bin: &bin,
            project,
            plugin_root: t.path(),
            data_dir: t.path().join("data"),
            log_dir: t.path().join("logs"),
            seq: 0,
            row: FixtureRow::default(),
            timeout: std::time::Duration::from_secs(30),
            timed_out: false,
            worker: None,
            path_dir: None,
            unset_vars: Vec::new(),
            pending: vec![Pending {
                finding: "F1".into(),
                evidence: "E0".into(),
            }],
        };
        run(&mut fx, &[("H1".to_owned(), "jvm".to_owned())]);
        let calls = fs::read_to_string(&log).unwrap_or_default();
        (std::mem::take(&mut fx.row), calls)
    }

    #[cfg(unix)]
    use super::super::FixtureRow;

    #[cfg(unix)]
    #[test]
    fn the_policy_captures_summarizes_and_cites_the_tied_evidence() {
        let (row, calls) = drive_fake(&[]);
        assert!(row.failures.is_empty(), "{:?}", row.failures);
        assert_eq!(row.captures, ["H1:capture-all"]);
        assert_eq!(row.ops, ["type-histogram-diff:b"]);
        assert_eq!(row.summarize_evidence, ["E1", "E2"]);
        assert!(row.max_stdout > 0 && row.max_stdout < LIMIT);
        let lines: Vec<&str> = calls.lines().collect();
        assert_eq!(lines[0], "summarize --list");
        assert_eq!(lines[1], "harness run H1 --capture all");
        // The quoted word stays one argument; the listed command ran once.
        assert_eq!(
            lines[2],
            "summarize type-histogram-diff a.txt b c.txt d.txt"
        );
        // The held-back confirmation cites the trend evidence and the tied call's evidence.
        assert_eq!(lines[3], "ledger confirm F1 --evidence E0,E1,E2");
        assert_eq!(lines.len(), 4);
    }

    #[cfg(unix)]
    #[test]
    fn a_tied_column_without_a_reason_and_an_oversized_output_fail_the_fixture() {
        let pad = "x".repeat(2100);
        let (row, calls) = drive_fake(&[("FAKE_NO_TIE", "1"), ("FAKE_PAD", &pad)]);
        assert_eq!(row.failures.len(), 2, "{:?}", row.failures);
        assert!(row.failures.iter().any(|f| f.contains("over 2048")));
        assert!(
            row.failures
                .iter()
                .any(|f| f.contains("without a tie reason"))
        );
        // Nothing is tied to F1 without a reason: the confirmation cites the trend alone.
        assert!(
            calls.contains("ledger confirm F1 --evidence E0\n"),
            "{calls}"
        );
    }

    #[test]
    fn a_tie_needs_a_known_rule_and_a_key() {
        let sha = sha256_hex(b"raw");
        let read = |_: &str| Some(b"raw".to_vec());
        for tie in [
            json!({"rule": "holder"}),
            json!({"rule": "holder", "key": ""}),
            json!({"rule": "bogus", "key": "k"}),
            json!({"key": "k"}),
        ] {
            let v = view_with(
                "E1",
                json!({"gate": "pass", "passed": [
                {"column": "c", "finding": "F1", "tie": tie}]}),
            );
            let o = out(good(&sha), 100);
            let bad = check_call("s", &o, &v, &read);
            assert_eq!(bad.len(), 1, "{tie}: {bad:?}");
            assert!(tied_findings(&o, &v).is_none(), "{tie}");
        }
        for rule in ["holder", "site", "type"] {
            let v = view_with(
                "E1",
                json!({"gate": "pass", "passed": [
                {"column": "c", "finding": "F1", "tie": {"rule": rule, "key": "k"}}]}),
            );
            let o = out(good(&sha), 100);
            assert!(check_call("s", &o, &v, &read).is_empty());
            assert!(tied_findings(&o, &v).is_some());
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_crashed_capture_run_fails_a_normal_fixture() {
        let (row, _) = drive_fake(&[("FAKE_CRASH_HARNESS", "1")]);
        assert!(
            row.failures
                .iter()
                .any(|f| f.contains("harness run H1 --capture all crashed (exit 101")),
            "{:?}",
            row.failures
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_crashed_ledger_confirm_fails_a_normal_fixture() {
        let (row, _) = drive_fake(&[("FAKE_CRASH_CONFIRM", "1")]);
        assert!(
            row.failures
                .iter()
                .any(|f| f.contains("ledger confirm F1") && f.contains("crashed")),
            "{:?}",
            row.failures
        );
    }

    #[test]
    fn native_tools_follow_the_listed_adapters() {
        let l = |rt: &str, b: &str| Adapter {
            runtime: rt.into(),
            backend: b.into(),
            available: false,
        };
        let list = [
            l("jvm", "jfr-old-object-sample"),
            l("native", "valgrind-memcheck"),
            l("native", "heaptrack"),
            l("any", "dla-alloc-json"),
        ];
        assert_eq!(
            native_tools(&list),
            [("valgrind", false), ("heaptrack", false)]
        );
        assert_eq!(native_tools(&list[..2]), [("valgrind", false)]);
        assert!(native_tools(&[]).is_empty());
        let v = json!({"adapters": [{"runtime": "jvm", "backend": "x", "available": true}]});
        assert_eq!(
            parse_adapters(&v),
            [Adapter {
                runtime: "jvm".into(),
                backend: "x".into(),
                available: true
            }]
        );
    }
}
