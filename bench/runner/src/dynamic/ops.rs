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

/// What a passing summarize call tied: the findings, the evidence the call wrote, its growth
/// evidence, and the `favors leak` discriminators it wrote (finding, evidence).
struct Tied {
    set: BTreeSet<String>,
    evidence: Vec<String>,
    growth: String,
    discs: Vec<(String, String)>,
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

/// The adapters of a listing: `ops` maps an op to its runtimes and their backends, and
/// `missing` names the backends whose tool is absent. The older row form
/// (`adapters: [{runtime, backend, available}]`) is read too.
fn parse_adapters(v: &Value) -> Vec<Adapter> {
    let mut list: Vec<Adapter> = v["adapters"]
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
        .unwrap_or_default();
    let missing = strings(&v["missing"]);
    if let Some(ops) = v["ops"].as_object() {
        for runtimes in ops.values().filter_map(Value::as_object) {
            for (runtime, backends) in runtimes {
                for b in strings(backends) {
                    list.push(Adapter {
                        runtime: runtime.clone(),
                        available: !missing.contains(&b),
                        backend: b,
                    });
                }
            }
        }
    }
    list
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
        "node" | "go" | "python" => {
            // The capture kinds each runtime's agent writes; the harness drops the rest and
            // names them. Node command mode captures snapshots per point, and the printed
            // heap-graph command forms the per-point series.
            let kinds = match runtime {
                "python" => "histogram,alloc,nonheap",
                "node" => "heap,alloc,nonheap",
                _ => "alloc,nonheap",
            };
            fx.row.captures.push(format!("{harness}:capture-{kinds}"));
            let out = fx.call_quiet(&["harness", "run", harness, "--capture", kinds]);
            if out.ok {
                let n = printed_commands(fx, &out);
                fx.step(format!(
                    "{harness} --capture {kinds}: {} summarize command(s)",
                    n.len()
                ));
                cmds.extend(n);
            } else {
                let msg = format!("harness run {harness} --capture {kinds}: {}", out.message());
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

/// `dla correlate H` for a JVM harness: a harness without a foreign-function boundary
/// refuses, which is the normal answer and not a failure. A correlation writes a candidate
/// and an `alloc-stack` evidence that the trend step ties to the native column.
pub(super) fn correlate(fx: &mut Fx, harness: &str) {
    let out = fx.call_quiet(&["correlate", harness]);
    fx.row.max_stdout = fx.row.max_stdout.max(out.bytes);
    if out.bytes > LIMIT {
        fx.row.failures.push(format!(
            "correlate: stdout is {} bytes, over {LIMIT}",
            out.bytes
        ));
    }
    if out.json.is_null() {
        fx.row
            .failures
            .push(format!("correlate: stdout is not JSON (exit {})", out.exit));
        return;
    }
    if out.ok {
        let view = fx.view();
        for id in out.evidence() {
            match view.evidence.get(&id) {
                None => fx
                    .row
                    .failures
                    .push(format!("correlate: evidence {id} is not in the ledger")),
                Some(ev) => {
                    let size = serde_json::to_string(&ev.result).map_or(0, |s| s.len());
                    if size > LIMIT {
                        fx.row.failures.push(format!(
                            "correlate: result of {id} is {size} bytes, over {LIMIT}"
                        ));
                    }
                }
            }
        }
        fx.row.ops.push("correlate".to_owned());
        fx.step(format!(
            "{harness} correlate: {}",
            out.json["summary"].as_str().unwrap_or("done")
        ));
    } else if out.message().contains("no foreign-function boundary") {
        fx.step(format!("{harness} correlate: no foreign-function boundary"));
    } else {
        fx.step(format!(
            "{harness} correlate refused ({}): {}",
            out.code(),
            out.message()
        ));
    }
}

/// Alloc-tests the late-tied findings of one fixture may run in all (on top of
/// [`super::MAX_ALLOC_TESTS`] of the trend steps).
const MAX_LATE_ALLOC_TESTS: usize = 2;

/// Steps 1 to 5. `grew` holds the harnesses with a passing column (id, runtime).
pub(super) fn run(fx: &mut Fx, grew: &[(String, String)]) {
    let pending = std::mem::take(&mut fx.pending);
    let mut tied: Vec<(BTreeSet<String>, Vec<String>)> = Vec::new();
    let mut late: Vec<(String, Tied)> = Vec::new();
    if !grew.is_empty() {
        let adapters = list_adapters(fx);
        let mut cmds: Vec<(String, String)> = Vec::new();
        for (h, rt) in grew {
            for c in capture(fx, h, rt, &adapters) {
                if !cmds.iter().any(|(x, _)| *x == c) {
                    cmds.push((c, h.clone()));
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
        for (cmd, harness) in &cmds {
            if let Some(t) = summarize(fx, cmd) {
                tied.push((t.set.clone(), t.evidence.clone()));
                late.push((harness.clone(), t));
            }
        }
    }
    drive_tied(fx, &late, &pending, &tied, grew);
    for p in pending {
        confirm(fx, &p, &tied);
    }
}

/// A finding that a passing growth column of a summarize call is tied to, and that no harness
/// step took (a census finding whose in-process probe saw nothing, such as a closure context,
/// or a finding the summarize step added): driven like a harness-tied finding. The N-WARM
/// sibling is added, the finding tested with the workload of the harness, the sibling refuted
/// with the call's `favors leak` discriminator, and the finding confirmed citing the call's
/// growth evidence. A finding with no discriminator from the call gets its own alloc-test
/// (a budget of [`MAX_LATE_ALLOC_TESTS`] per run), and is taken only when the test favors leak
/// and a holder is known ([`alloc_route_holder`]). A finding that is not a candidate (a refuted
/// one stays refuted), or has no site on a repository run, is left alone.
fn drive_tied(
    fx: &mut Fx,
    late: &[(String, Tied)],
    pending: &[Pending],
    tied: &[(BTreeSet<String>, Vec<String>)],
    grew: &[(String, String)],
) {
    let late_tests = &mut 0usize;
    let artifact = fx.plan.entry == crate::model::Entry::Artifact;
    let mut taken: BTreeSet<String> = pending.iter().map(|p| p.finding.clone()).collect();
    for (harness, t) in late {
        for f in &t.set {
            if taken.contains(f) {
                continue;
            }
            let view = fx.view();
            let Some(fnd) = view.finding(f) else { continue };
            if fnd.state != "candidate" || fnd.is_nonleak() {
                continue;
            }
            let located = if artifact {
                fnd.owner.is_some()
            } else {
                fnd.site.is_some()
            };
            let known = t.discs.iter().find(|(x, _)| x == f).map(|(_, d)| d.clone());
            let same_site = fnd.site.as_ref().is_some_and(|site| {
                taken
                    .iter()
                    .filter_map(|x| view.finding(x))
                    .any(|x| x.site.as_ref() == Some(site))
            });
            if !located || same_site {
                continue;
            }
            // A tied finding the summarize call wrote no discriminator for: the alloc-test of
            // the finding is the discriminator, as for a column that carries candidates. A
            // verdict that does not favor leak leaves the finding alone, and so does a finding
            // whose holder the evidence does not give ([`alloc_route_holder`]).
            let mut annotate: Option<(String, String)> = None;
            let disc = match known {
                Some(d) => d,
                None => {
                    if *late_tests >= MAX_LATE_ALLOC_TESTS {
                        fx.step(format!(
                            "{f} via summarize: no discriminator and the alloc-test budget is used up"
                        ));
                        continue;
                    }
                    *late_tests += 1;
                    let a = super::alloc_test(fx, harness, Some(f));
                    let e = match (a.verdict, a.evidence) {
                        (super::Verdict::Leak, Some(e)) => e,
                        (v, _) => {
                            fx.step(format!(
                                "{f} via summarize: no discriminator; alloc-test says {v:?}, left alone"
                            ));
                            continue;
                        }
                    };
                    let runtime = grew
                        .iter()
                        .find(|(h, _)| h == harness)
                        .map_or("", |(_, r)| r.as_str());
                    match alloc_route_holder(fx, f, runtime, &e) {
                        Ok(h) => annotate = h,
                        Err(why) => {
                            fx.step(format!(
                                "{f} via summarize: alloc-test favors leak ({e}) but no holder: {why}; left alone"
                            ));
                            continue;
                        }
                    }
                    e
                }
            };
            let disc = &disc;
            taken.insert(f.clone());
            let Some(sib) = super::add_sibling(fx, f, "N-WARM", &t.growth) else {
                fx.step(format!(
                    "{f} via summarize: could not add the N-WARM sibling"
                ));
                continue;
            };
            if !super::test_finding(fx, harness, f) {
                fx.step(format!("{f} via summarize: ledger test refused"));
                continue;
            }
            let r = fx.call(&["ledger", "refute", &sib, "--evidence", disc]);
            if !r.ok {
                fx.step(format!(
                    "{f} via summarize: sibling {sib} not refuted by {disc} ({})",
                    r.code()
                ));
                continue;
            }
            let mut note = String::new();
            if let Some((holder, source)) = annotate {
                let r = fx.call(&["ledger", "annotate", f, "--holder", &holder]);
                if r.ok {
                    note = format!(", holder {holder} ({source})");
                } else {
                    let why = if r.code().is_empty() {
                        format!("annotate gave no ledger answer: {}", r.message())
                    } else {
                        format!("refused by the ledger ({})", r.code())
                    };
                    note = format!(", holder {holder} ({source}) {why}, confirming without it");
                }
            }
            fx.step(format!(
                "{f} via summarize: sibling {sib} refuted by {disc}{note}, confirmation next"
            ));
            let p = Pending {
                finding: f.clone(),
                evidence: t.growth.clone(),
            };
            confirm(fx, &p, tied);
        }
    }
}

/// The holder of a finding that the alloc-test route confirms, in this order: (1) the holder
/// the finding has (`Ok(None)`: nothing to annotate); (2) the holder of a known-leaks row that
/// the probe evidence of the run lists as `true` for the runtime and whose `when` source regex
/// matches the finding's site line (rows that disagree give none); (3) a holder the
/// alloc-test evidence itself names. The census entry of another finding at the same site is
/// never a source. `Err` says why there is none.
fn alloc_route_holder(
    fx: &Fx,
    id: &str,
    runtime: &str,
    alloc_evidence: &str,
) -> Result<Option<(String, String)>, String> {
    let view = fx.view();
    let Some(f) = view.finding(id) else {
        return Err("the finding is not in the ledger".to_owned());
    };
    if f.holder.is_some() {
        return Ok(None);
    }
    let truths: BTreeSet<String> = view
        .evidence
        .values()
        .filter(|e| e.kind == "probe")
        .flat_map(|e| strings(&e.result["known_leaks"]["true"]))
        .collect();
    let line = f.site.as_deref().and_then(|s| site_line(&fx.project, s));
    if let (Some(line), false) = (&line, truths.is_empty())
        && let Ok(text) = fs::read_to_string(fx.plugin_root.join("data").join("known-leaks.toml"))
    {
        match known_leak_holder(&text, &truths, runtime, line) {
            KnownHolder::One {
                id,
                holder,
                archetype,
            } => {
                let row = archetype.map_or(id.clone(), |a| format!("{id}, {a}"));
                return Ok(Some((holder, format!("known-leaks row {row}"))));
            }
            KnownHolder::Ambiguous => {
                return Err(
                    "known-leaks rows that match the site line name different holders".to_owned(),
                );
            }
            KnownHolder::None => {}
        }
    }
    if let Some(h) = view
        .evidence
        .get(alloc_evidence)
        .and_then(|e| e.result["holder"].as_str())
    {
        return Ok(Some((h.to_owned(), format!("named by {alloc_evidence}"))));
    }
    Err(if truths.is_empty() {
        "the finding has no holder, the probe lists no true known-leaks row and the alloc-test names none"
    } else {
        "the finding has no holder, no true known-leaks row matches its site line and the alloc-test names none"
    }
    .to_owned())
}

/// The source line `file:line` of a site in the project, trimmed.
fn site_line(project: &std::path::Path, site: &str) -> Option<String> {
    let (file, line) = site.rsplit_once(':')?;
    let n: usize = line.parse().ok()?;
    let text = fs::read_to_string(project.join(file)).ok()?;
    text.lines()
        .nth(n.checked_sub(1)?)
        .map(|l| l.trim().to_owned())
}

/// What the known-leaks rows say about the holder at one source line.
#[derive(Debug, PartialEq)]
enum KnownHolder {
    One {
        id: String,
        holder: String,
        archetype: Option<String>,
    },
    Ambiguous,
    None,
}

/// The rows of `data/known-leaks.toml` text that are `true` for the project, are for the
/// runtime and have a `when` source regex (outside a `not`) that matches `line`.
fn known_leak_holder(
    text: &str,
    truths: &BTreeSet<String>,
    runtime: &str,
    line: &str,
) -> KnownHolder {
    let Ok(file) = text.parse::<toml::Table>() else {
        return KnownHolder::None;
    };
    let mut found: Vec<(String, String, Option<String>)> = Vec::new();
    for row in file
        .get("leak")
        .and_then(toml::Value::as_array)
        .into_iter()
        .flatten()
    {
        let (Some(id), Some(rt), Some(holder)) = (
            row.get("id").and_then(toml::Value::as_str),
            row.get("runtime").and_then(toml::Value::as_str),
            row.get("holder").and_then(toml::Value::as_str),
        ) else {
            continue;
        };
        if rt != runtime || !truths.contains(id) {
            continue;
        }
        let mut regexes = Vec::new();
        if let Some(w) = row.get("when") {
            source_regexes(w, &mut regexes);
        }
        let hit = regexes
            .iter()
            .any(|r| regex::Regex::new(r).is_ok_and(|re| re.is_match(line)));
        if hit {
            let archetype = row
                .get("archetype")
                .and_then(toml::Value::as_str)
                .map(str::to_owned);
            found.push((id.to_owned(), holder.to_owned(), archetype));
        }
    }
    let mut holders: Vec<&str> = found.iter().map(|(_, h, _)| h.as_str()).collect();
    holders.sort_unstable();
    holders.dedup();
    match (found.first(), holders.len()) {
        (None, _) => KnownHolder::None,
        (Some((id, holder, archetype)), 1) => KnownHolder::One {
            id: id.clone(),
            holder: holder.clone(),
            archetype: archetype.clone(),
        },
        _ => KnownHolder::Ambiguous,
    }
}

/// Every `source_regex` of a `when` condition, leaving out the ones under `not`.
fn source_regexes(v: &toml::Value, out: &mut Vec<String>) {
    match v {
        toml::Value::Table(t) => {
            for (k, x) in t {
                match (k.as_str(), x) {
                    ("not", _) => {}
                    ("source_regex", toml::Value::String(r)) => out.push(r.clone()),
                    _ => source_regexes(x, out),
                }
            }
        }
        toml::Value::Array(a) => a.iter().for_each(|x| source_regexes(x, out)),
        _ => {}
    }
}

/// Runs one printed command and checks it; returns what it tied.
fn summarize(fx: &mut Fx, cmd: &str) -> Option<Tied> {
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
        let gap = format!("coverage gap, backend {} missing", missing.join(","));
        fx.step(format!("{op}: {gap}"));
        log_failed_summarize(fx, &op, &backend, "gap", gap);
        None
    } else if out.ok {
        fx.row.ops.push(format!("{op}:{backend}"));
        fx.row
            .summarize_cmds
            .push(args.iter().map(|a| (*a).to_owned()).collect());
        annotate_observed_holders(fx, &out);
        for id in out.evidence() {
            if !fx.row.summarize_evidence.contains(&id) {
                fx.row.summarize_evidence.push(id);
            }
        }
        let t = tied_findings(&out, &view).map(|(set, evidence)| Tied {
            set,
            evidence,
            growth: out.json["growth_evidence"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            discs: out.json["discriminators"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|d| d["favors"].as_str() == Some("leak"))
                .filter_map(|d| {
                    Some((
                        d["finding"].as_str()?.to_owned(),
                        d["evidence"].as_str()?.to_owned(),
                    ))
                })
                .collect(),
        });
        let gate = out.json["gate"].as_str().unwrap_or("-").to_owned();
        let tied_text = t.as_ref().map_or_else(
            || "none".to_owned(),
            |t| t.set.iter().cloned().collect::<Vec<_>>().join(","),
        );
        let mut reasons: Vec<String> = out.json["reasons"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|r| r.as_str().map(str::to_owned))
            .collect();
        if reasons.is_empty()
            && let Some(n) = out.json["needed"].as_str()
        {
            reasons.push(n.chars().take(200).collect());
        }
        fx.step(format!("{op} via {backend}: gate {gate}, tied {tied_text}"));
        fx.row
            .summarize_log
            .push(crate::dynamic_cmd::SummarizeStep {
                op: op.clone(),
                backend: backend.clone(),
                gate,
                tied: tied_text,
                reasons,
            });
        t
    } else {
        log_failed_summarize(fx, &op, &backend, "error", out.message());
        let msg = format!("{op}: {}", out.message());
        fx.step(format!("summarize {msg}"));
        fx.row.errors.push(format!("dla summarize {msg}"));
        None
    }
}

/// Records a summarize call that ran but gave no evidence (a coverage gap or an error), so a
/// refused label's failure line does not say the step "did not run". `gate` is `gap` or
/// `error`; `why` carries the code and message.
fn log_failed_summarize(fx: &mut Fx, op: &str, backend: &str, gate: &str, why: String) {
    fx.row
        .summarize_log
        .push(crate::dynamic_cmd::SummarizeStep {
            op: op.to_owned(),
            backend: backend.to_owned(),
            gate: gate.to_owned(),
            tied: "none".to_owned(),
            reasons: vec![why],
        });
}

/// A holder annotation a summarize call asks for: finding, holder, and the growth evidence
/// id the command cites (`--evidence`), when the text names one.
pub(super) type HolderAnnotation = (String, String, Option<String>);

/// The annotations a summarize call asks for: its `needed` text names
/// `dla ledger annotate F --holder H --evidence E` where the call's `observed_holder` for F
/// differs from the finding's holder. Only those where `observed_holder` agrees with H are
/// returned. Reads dla JSON only, so it is the same for every fixture.
pub(super) fn holder_annotations(json: &Value) -> Vec<HolderAnnotation> {
    let Some(needed) = json["needed"].as_str() else {
        return Vec::new();
    };
    let observed = &json["observed_holder"];
    let clean = |w: &str| w.trim_end_matches([';', ',', '.']).to_owned();
    let words: Vec<&str> = needed.split_whitespace().collect();
    let mut found: Vec<HolderAnnotation> = Vec::new();
    for (i, w) in words.iter().enumerate() {
        if *w != "dla"
            || words.get(i + 1) != Some(&"ledger")
            || words.get(i + 2) != Some(&"annotate")
        {
            continue;
        }
        let (Some(f), Some(&"--holder"), Some(h)) =
            (words.get(i + 3), words.get(i + 4), words.get(i + 5))
        else {
            continue;
        };
        let h = clean(h);
        let evidence = match (words.get(i + 6), words.get(i + 7)) {
            (Some(&"--evidence"), Some(e)) => Some(clean(e)),
            _ => None,
        };
        if observed[*f].as_str() == Some(h.as_str()) && !found.iter().any(|(x, ..)| x == f) {
            found.push(((*f).to_owned(), h, evidence));
        }
    }
    found
}

/// The tied findings other than those `needed` names whose set holder differs from the
/// observed one: `needed` speaks of the first tied finding only. The command has the same
/// form `needed` prints, citing the call's growth evidence.
pub(super) fn more_annotations(
    json: &Value,
    named: &[HolderAnnotation],
    holder_of: impl Fn(&str) -> Option<String>,
) -> Vec<HolderAnnotation> {
    let growth = json["growth_evidence"].as_str();
    let mut found = Vec::new();
    for (f, h) in json["observed_holder"].as_object().into_iter().flatten() {
        let (Some(h), Some(g)) = (h.as_str(), growth) else {
            continue;
        };
        if named.iter().any(|(x, ..)| x == f) {
            continue;
        }
        if holder_of(f).is_some_and(|cur| cur != h) {
            found.push((f.clone(), h.to_owned(), Some(g.to_owned())));
        }
    }
    found
}

/// Runs the holder annotations a passing summarize call asks for, exactly as its `needed`
/// text prints them (with `--evidence`), so a confirmation can cite its growth evidence.
fn annotate_observed_holders(fx: &mut Fx, out: &Out) {
    let mut todo = holder_annotations(&out.json);
    let view = fx.view();
    let more = more_annotations(&out.json, &todo, |f| {
        view.findings
            .iter()
            .find(|x| x.id == f)
            .and_then(|x| x.holder.clone())
    });
    todo.extend(more);
    for (f, h, e) in todo {
        let mut args = vec!["ledger", "annotate", f.as_str(), "--holder", h.as_str()];
        if let Some(e) = e.as_deref() {
            args.extend(["--evidence", e]);
        }
        let a = fx.call_quiet(&args);
        if a.ok {
            fx.step(format!("{f} annotated with observed holder {h}"));
        } else {
            let msg = format!("dla {}: {}", args[..].join(" "), a.message());
            fx.step(format!("annotate refused ({})", a.code()));
            fx.row.errors.push(msg);
        }
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
    fn an_observed_holder_that_differs_is_annotated() {
        let j = json!({"observed_holder": {"F7": "static/global", "F9": "registration-table"},
            "needed": "the snapshots show holder static/global for F7: dla ledger annotate F7 --holder static/global --evidence E3; then a refuted non-leak sibling"});
        assert_eq!(
            holder_annotations(&j),
            [(
                "F7".to_owned(),
                "static/global".to_owned(),
                Some("E3".to_owned())
            )]
        );
        // Without a named evidence the command is run without one.
        let bare = json!({"observed_holder": {"F7": "static/global"},
            "needed": "dla ledger annotate F7 --holder static/global"});
        assert_eq!(
            holder_annotations(&bare),
            [("F7".to_owned(), "static/global".to_owned(), None)]
        );
        // Nothing is named when the holders agree, or when the observation does not match.
        let same = json!({"observed_holder": {"F7": "static/global"},
            "needed": "a refuted non-leak sibling and a holder, then dla ledger confirm F7 --evidence E3"});
        assert!(holder_annotations(&same).is_empty());
        let other = json!({"observed_holder": {"F7": "container-field"},
            "needed": "dla ledger annotate F7 --holder static/global --evidence E3"});
        assert!(holder_annotations(&other).is_empty());
    }

    #[test]
    fn a_tied_finding_that_needed_does_not_name_is_annotated_too() {
        let j = json!({"growth_evidence": "E22",
            "observed_holder": {"F5": "static/global", "F8": "static/global", "F9": "container-field"},
            "needed": "a refuted non-leak sibling and a holder, then dla ledger confirm F8 --evidence E22"});
        let holder = |f: &str| match f {
            "F5" | "F9" => Some("container-field".to_owned()),
            "F8" => Some("static/global".to_owned()),
            _ => None,
        };
        // F8 agrees, F9 agrees; F5 differs.
        assert_eq!(
            more_annotations(&j, &[], holder),
            [(
                "F5".to_owned(),
                "static/global".to_owned(),
                Some("E22".to_owned())
            )]
        );
        // A finding the text already names is not repeated.
        let named = [("F5".to_owned(), "static/global".to_owned(), None)];
        assert!(more_annotations(&j, &named, holder).is_empty());
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
  if [ -n "$FAKE_SUMMARIZE_ERR" ]; then echo '{"ok":false,"error":{"code":"malformed-artifact","message":"not a histogram"}}'; exit 2; fi
  if [ -n "$FAKE_SUMMARIZE_GAP" ]; then echo '{"ok":true,"evidence":[],"coverage_gap":[{"backend":"b"}]}'; exit 0; fi
  mkdir -p $R/histograms
  printf raw > $R/histograms/H1.json
  tie='"tie":{"rule":"type","key":"X"},'
  [ -n "$FAKE_NO_TIE" ] && tie=''
  echo '{"type":"evidence","evidence_id":"E1","kind":"histogram-diff","authority":"live","result":{}}' >> $R/ledger.jsonl
  echo '{"type":"evidence","evidence_id":"E2","kind":"growth-series","authority":"live","result":{"gate":"pass","passed":[{"column":"type:X",'"$tie"'"finding":"F1"}]}}' >> $R/ledger.jsonl
  echo '{"ok":true,"backend":"b","command":"summarize type-histogram-diff","evidence":["E1","E2"],"gate":"pass","growth_evidence":"E2","raw":{"path":"histograms/H1.json","sha256":"'"$FAKE_SHA"'"},"pad":"'"$FAKE_PAD"'"}';;
"ledger add")
  echo '{"ok":true,"id":"F2.n1"}';;
"ledger refute")
  if [ -n "$FAKE_NO_REFUTE" ]; then echo '{"ok":false,"error":{"code":"not-refutable"}}'; exit 2; fi
  echo '{"ok":true}';;
"alloc-test H1")
  echo '{"type":"evidence","evidence_id":"E9","kind":"discriminator","authority":"live","result":{"favors":"'"${FAKE_ALLOC_FAVORS:-leak}"'"'"${FAKE_ALLOC_HOLDER:+,\"holder\":\"$FAKE_ALLOC_HOLDER\"}"'}}' >> $R/ledger.jsonl
  echo '{"ok":true,"evidence":["E9"]}';;
"ledger annotate")
  if [ -n "$FAKE_ANNOTATE_CRASH" ]; then exit 3; fi
  if [ -n "$FAKE_NO_ANNOTATE" ]; then echo '{"ok":false,"error":{"code":"holder-refused"}}'; exit 2; fi
  echo '{"ok":true}';;
"ledger confirm")
  if [ -n "$FAKE_CRASH_CONFIRM" ]; then echo "thread main panicked" >&2; exit 101; fi
  echo '{"ok":true}';;
*) echo '{"ok":true}';;
esac
"#;

    #[cfg(unix)]
    fn drive_fake(env: &[(&str, &str)]) -> (FixtureRow, String) {
        drive_fake_with(env, "")
    }

    #[cfg(unix)]
    fn drive_fake_with(env: &[(&str, &str)], ledger: &str) -> (FixtureRow, String) {
        use std::os::unix::fs::PermissionsExt;
        let t = tempfile::tempdir().unwrap();
        let bin = t.path().join("dla");
        fs::write(&bin, format!("#!/bin/sh\n{FAKE_DLA}")).unwrap();
        fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
        let project = t.path().join("project");
        fs::create_dir_all(project.join(".dla/R1")).unwrap();
        fs::write(project.join(".dla/current"), "R1\n").unwrap();
        fs::write(project.join(".dla/R1/ledger.jsonl"), ledger).unwrap();
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
            provided: None,
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
        // Each successful call is logged with its gate and tie, for a refused label's failure line.
        assert_eq!(row.summarize_log.len(), 1, "{:?}", row.summarize_log);
        let step = &row.summarize_log[0];
        assert_eq!(
            (step.op.as_str(), step.backend.as_str(), step.gate.as_str()),
            ("type-histogram-diff", "b", "pass")
        );
        assert_eq!(step.tied, "F1");
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
    fn a_summarize_call_with_an_error_or_a_gap_is_logged_for_the_failure_line() {
        let (row, _) = drive_fake(&[("FAKE_SUMMARIZE_ERR", "1")]);
        assert_eq!(row.summarize_log.len(), 1, "{:?}", row.summarize_log);
        let s = &row.summarize_log[0];
        assert_eq!(
            (s.op.as_str(), s.gate.as_str()),
            ("type-histogram-diff", "error")
        );
        assert_eq!(s.tied, "none");
        assert_eq!(s.reasons, ["malformed-artifact: not a histogram"]);
        let (row, _) = drive_fake(&[("FAKE_SUMMARIZE_GAP", "1")]);
        assert_eq!(row.summarize_log.len(), 1, "{:?}", row.summarize_log);
        let s = &row.summarize_log[0];
        assert_eq!(
            (s.op.as_str(), s.gate.as_str()),
            ("type-histogram-diff", "gap")
        );
        assert_eq!(s.reasons, ["coverage gap, backend b missing"]);
    }

    /// `drive_tied` against the stand-in: the ledger holds the given finding records.
    #[cfg(unix)]
    fn drive_late(
        ledger: &str,
        env: &[(&str, &str)],
        discs: &[(&str, &str)],
    ) -> (FixtureRow, String) {
        drive_late_in(ledger, env, discs, &[], &[])
    }

    /// [`drive_late`] with files in the project and in the plugin's `data/` directory.
    #[cfg(unix)]
    fn drive_late_in(
        ledger: &str,
        env: &[(&str, &str)],
        discs: &[(&str, &str)],
        project_files: &[(&str, &str)],
        data_files: &[(&str, &str)],
    ) -> (FixtureRow, String) {
        use std::os::unix::fs::PermissionsExt;
        let t = tempfile::tempdir().unwrap();
        let bin = t.path().join("dla");
        fs::write(&bin, format!("#!/bin/sh\n{FAKE_DLA}")).unwrap();
        fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
        let project = t.path().join("project");
        fs::create_dir_all(project.join(".dla/R1")).unwrap();
        fs::write(project.join(".dla/current"), "R1\n").unwrap();
        fs::write(project.join(".dla/R1/ledger.jsonl"), ledger).unwrap();
        let log = t.path().join("calls.log");
        let mut settings = format!("FAKE_LOG='{}'\nFAKE_SHA='x'\n", log.display());
        for (k, v) in env {
            settings.push_str(&format!("{k}='{v}'\n"));
        }
        fs::write(project.join("fake.env"), settings).unwrap();
        for (name, text) in project_files {
            let path = project.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        fs::create_dir_all(t.path().join("data")).unwrap();
        for (name, text) in data_files {
            fs::write(t.path().join("data").join(name), text).unwrap();
        }
        let mut fx = Fx {
            plan: super::super::Plan {
                entry: crate::model::Entry::Repository,
                runtimes: 1,
                artifacts: Vec::new(),
            },
            bin: &bin,
            project,
            plugin_root: t.path(),
            data_dir: t.path().join("run-data"),
            log_dir: t.path().join("logs"),
            seq: 0,
            row: FixtureRow::default(),
            timeout: std::time::Duration::from_secs(30),
            timed_out: false,
            worker: None,
            path_dir: None,
            unset_vars: Vec::new(),
            pending: Vec::new(),
            provided: None,
        };
        let tied = Tied {
            set: ["F1", "F2", "F3", "F4"]
                .iter()
                .map(|s| (*s).to_owned())
                .collect(),
            evidence: vec!["E7".into(), "E8".into()],
            growth: "E7".into(),
            discs: discs
                .iter()
                .map(|(f, e)| ((*f).to_owned(), (*e).to_owned()))
                .collect(),
        };
        let sets = [(tied.set.clone(), tied.evidence.clone())];
        let grew = [("H1".to_owned(), "go".to_owned())];
        drive_tied(&mut fx, &[("H1".to_owned(), tied)], &[], &sets, &grew);
        let calls = fs::read_to_string(&log).unwrap_or_default();
        (std::mem::take(&mut fx.row), calls)
    }

    #[cfg(unix)]
    const LATE_LEDGER: &str = concat!(
        r#"{"type":"evidence","evidence_id":"E5","kind":"workload","authority":"test","result":{"harness":"H1"}}"#,
        "\n",
        r#"{"type":"finding_added","finding":{"id":"F1","origin":"census","site":"src/a.js:7","owner":"a.A.x"}}"#,
        "\n",
        r#"{"type":"finding_added","finding":{"id":"F2","origin":"growth-series","owner":"mangled.owner"}}"#,
        "\n",
        r#"{"type":"finding_added","finding":{"id":"F3","origin":"census","site":"src/b.js:9"}}"#,
        "\n",
        r#"{"type":"transition","finding":"F3","from":"candidate","to":"refuted","evidence":["E6"]}"#,
        "\n",
        r#"{"type":"finding_added","finding":{"id":"F4","origin":"census","site":"src/c.js:3"}}"#,
        "\n",
    );

    #[cfg(unix)]
    #[test]
    fn a_candidate_tied_by_a_summarize_call_is_driven_to_confirmation() {
        // F1 is a located candidate with a leak discriminator: sibling, test, refute, confirm.
        // F2 has no site, F3 is refuted (it stays refuted), F4 has no discriminator.
        let (row, calls) = drive_late(
            LATE_LEDGER,
            &[],
            &[("F1", "E8"), ("F2", "E8"), ("F3", "E8")],
        );
        assert!(row.failures.is_empty(), "{:?}", row.failures);
        let lines: Vec<&str> = calls.lines().collect();
        assert!(
            lines[0].starts_with("ledger add --sibling-of F1 "),
            "{lines:?}"
        );
        assert!(lines[0].ends_with("--evidence E7"), "{lines:?}");
        assert_eq!(lines[1], "ledger test F1 --evidence E5");
        assert_eq!(lines[2], "ledger refute F2.n1 --evidence E8");
        assert_eq!(lines[3], "ledger confirm F1 --evidence E7,E8");
        // F4 has no discriminator from the summarize call: its alloc-test is the discriminator.
        // It favors leak, but F4 has no holder and nothing names one: F4 is left alone.
        assert_eq!(lines[4], "alloc-test H1 --finding F4");
        assert_eq!(lines.len(), 5, "{lines:?}");
        assert!(
            row.steps
                .iter()
                .any(|s| s
                    .starts_with("F4 via summarize: alloc-test favors leak (E9) but no holder")),
            "{:?}",
            row.steps
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_tied_finding_without_a_discriminator_stays_alone_when_the_alloc_test_does_not_favor_leak()
    {
        let (row, calls) = drive_late(LATE_LEDGER, &[("FAKE_ALLOC_FAVORS", "non-leak")], &[]);
        assert!(row.failures.is_empty(), "{:?}", row.failures);
        assert!(!calls.contains("ledger confirm"), "{calls}");
        assert!(!calls.contains("--sibling-of"), "{calls}");
        // F1 and F4 are located candidates: each got one alloc-test, then the budget of two ended.
        assert_eq!(calls.matches("alloc-test H1").count(), 2, "{calls}");
    }

    #[cfg(unix)]
    const TICKER_PROBE: &str = r#"{"type":"evidence","evidence_id":"E1","kind":"probe","authority":"static","result":{"known_leaks":{"maybe":[],"true":["go-timer-unstopped-before-1.23"]}}}"#;

    #[cfg(unix)]
    const TICKER_ROWS: &str = r#"
[[leak]]
id = "go-timer-unstopped-before-1.23"
runtime = "go"
holder = "registration-table"
archetype = "L-REGISTER"
when = { source_regex = 'time\.(?:NewTimer|NewTicker|Tick|After|AfterFunc)\(' }

[[leak]]
id = "go-timer-asynctimerchan-revert"
runtime = "go"
holder = "registration-table"
archetype = "L-REGISTER"
when = { all = [ { config = { key = "asynctimerchan", value = "1" } }, { source_regex = 'time\.NewTicker\(' } ] }
"#;

    #[cfg(unix)]
    const WAIT_GO: &str = "package reserve\n\nfunc wait() {\n\t// a line\n\tt := time.NewTicker(time.Second)\n\t_ = t\n}\n";

    /// The ledger of go-ticker-legacy: F1 is the detector's finding at the ticker (no holder),
    /// F3 the census blocked-stack entry at the same line.
    #[cfg(unix)]
    fn ticker_ledger() -> String {
        [
            r#"{"type":"evidence","evidence_id":"E5","kind":"workload","authority":"test","result":{"harness":"H1"}}"#,
            TICKER_PROBE,
            r#"{"type":"finding_added","finding":{"id":"F1","origin":"detector:dla/go-ticker-no-stop","site":"internal/wait.go:5"}}"#,
            r#"{"type":"finding_added","finding":{"id":"F3","origin":"census","holder":"blocked-stack","site":"internal/wait.go:5"}}"#,
        ]
        .join("\n")
            + "\n"
    }

    #[cfg(unix)]
    #[test]
    fn a_detector_finding_gets_the_holder_of_the_true_known_leaks_row_at_its_site_line() {
        // F1 and F3 share the site; F3 (the census entry) is not taken from.
        let (row, calls) = drive_late_in(
            &ticker_ledger(),
            &[],
            &[],
            &[("internal/wait.go", WAIT_GO)],
            &[("known-leaks.toml", TICKER_ROWS)],
        );
        assert!(row.failures.is_empty(), "{:?}", row.failures);
        let lines: Vec<&str> = calls.lines().collect();
        assert_eq!(lines[0], "alloc-test H1 --finding F1", "{lines:?}");
        assert!(
            lines[1].starts_with("ledger add --sibling-of F1 "),
            "{lines:?}"
        );
        assert_eq!(lines[2], "ledger test F1 --evidence E5");
        assert_eq!(lines[3], "ledger refute F2.n1 --evidence E9");
        assert_eq!(lines[4], "ledger annotate F1 --holder registration-table");
        assert_eq!(lines[5], "ledger confirm F1 --evidence E7,E8");
        assert_eq!(lines.len(), 6, "{lines:?}");
        assert!(!calls.contains("F3"), "{calls}");
        assert!(
            row.steps
                .iter()
                .any(|s| s.contains("known-leaks row go-timer-unstopped-before-1.23, L-REGISTER")),
            "{:?}",
            row.steps
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_row_that_is_not_true_or_does_not_match_the_site_line_gives_no_holder() {
        // The probe lists no true row: nothing names a holder.
        let ledger = ticker_ledger().replace("go-timer-unstopped-before-1.23", "other-row");
        let (_, calls) = drive_late_in(
            &ledger,
            &[],
            &[],
            &[("internal/wait.go", WAIT_GO)],
            &[("known-leaks.toml", TICKER_ROWS)],
        );
        assert!(!calls.contains("ledger confirm F1"), "{calls}");
        assert!(!calls.contains("annotate F1"), "{calls}");
        // The site line is not a timer call.
        let (row, calls) = drive_late_in(
            &ticker_ledger().replace("wait.go:5", "wait.go:3"),
            &[],
            &[],
            &[("internal/wait.go", WAIT_GO)],
            &[("known-leaks.toml", TICKER_ROWS)],
        );
        assert!(!calls.contains("ledger confirm F1"), "{calls}");
        assert!(!calls.contains("annotate F1"), "{calls}");
        assert!(
            row.steps
                .iter()
                .any(|s| s.contains("no true known-leaks row matches its site line")),
            "{:?}",
            row.steps
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_finding_with_its_own_holder_keeps_it_on_the_alloc_test_route() {
        // The census entry at the site: its own holder, no annotate.
        let ledger = ticker_ledger().replace(
            r#""origin":"detector:dla/go-ticker-no-stop""#,
            r#""origin":"census","holder":"container-field""#,
        );
        let (_, calls) = drive_late_in(&ledger, &[], &[], &[], &[]);
        assert!(!calls.contains("annotate"), "{calls}");
        assert!(
            calls.contains("ledger confirm F1 --evidence E7,E8"),
            "{calls}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_holder_the_alloc_test_evidence_names_is_used_last() {
        let ledger = ticker_ledger();
        let (_, calls) = drive_late_in(
            &ledger,
            &[("FAKE_ALLOC_HOLDER", "thread-local")],
            &[],
            &[],
            &[],
        );
        assert!(
            calls.contains("ledger annotate F1 --holder thread-local"),
            "{calls}"
        );
        assert!(calls.contains("ledger confirm F1 "), "{calls}");
    }

    #[cfg(unix)]
    #[test]
    fn the_holder_sources_apply_in_order_when_several_are_present() {
        let files = [("internal/wait.go", WAIT_GO)];
        let rows = [("known-leaks.toml", TICKER_ROWS)];
        let env = [("FAKE_ALLOC_HOLDER", "thread-local")];
        // Row and alloc-test holder both present: the row comes first.
        let (_, calls) = drive_late_in(&ticker_ledger(), &env, &[], &files, &rows);
        assert!(
            calls.contains("ledger annotate F1 --holder registration-table"),
            "{calls}"
        );
        assert!(!calls.contains("thread-local"), "{calls}");
        // The finding's own holder, the row and the alloc-test holder: the own holder is kept.
        let own = ticker_ledger().replace(
            r#""origin":"detector:dla/go-ticker-no-stop""#,
            r#""origin":"census","holder":"container-field""#,
        );
        let (_, calls) = drive_late_in(&own, &env, &[], &files, &rows);
        assert!(!calls.contains("annotate"), "{calls}");
        assert!(
            calls.contains("ledger confirm F1 --evidence E7,E8"),
            "{calls}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn an_annotate_that_prints_no_json_is_not_called_a_ledger_refusal() {
        let (row, _) = drive_late_in(
            &ticker_ledger(),
            &[("FAKE_ANNOTATE_CRASH", "1")],
            &[],
            &[("internal/wait.go", WAIT_GO)],
            &[("known-leaks.toml", TICKER_ROWS)],
        );
        assert!(
            row.steps
                .iter()
                .any(|s| s.contains("annotate gave no ledger answer") && !s.contains("()")),
            "{:?}",
            row.steps
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_holder_the_ledger_refuses_is_logged_not_reported_as_set() {
        let (row, _) = drive_late_in(
            &ticker_ledger(),
            &[("FAKE_NO_ANNOTATE", "1")],
            &[],
            &[("internal/wait.go", WAIT_GO)],
            &[("known-leaks.toml", TICKER_ROWS)],
        );
        assert!(
            row.steps
                .iter()
                .any(|s| s.contains("refused by the ledger") && s.contains("registration-table")),
            "{:?}",
            row.steps
        );
    }

    #[cfg(unix)]
    #[test]
    fn the_alloc_test_budget_of_the_late_findings_ends_with_a_step_line() {
        // Three located candidates on three sites: two alloc-tests, then the budget is used up.
        let ledger = [
            r#"{"type":"evidence","evidence_id":"E5","kind":"workload","authority":"test","result":{"harness":"H1"}}"#,
            r#"{"type":"finding_added","finding":{"id":"F1","origin":"census","site":"src/a.js:1"}}"#,
            r#"{"type":"finding_added","finding":{"id":"F2","origin":"census","site":"src/b.js:2"}}"#,
            r#"{"type":"finding_added","finding":{"id":"F4","origin":"census","site":"src/c.js:3"}}"#,
        ]
        .join("\n")
            + "\n";
        let (row, calls) = drive_late(&ledger, &[], &[]);
        assert_eq!(calls.matches("alloc-test H1").count(), 2, "{calls}");
        assert!(
            row.steps
                .iter()
                .any(|s| s
                    == "F4 via summarize: no discriminator and the alloc-test budget is used up"),
            "{:?}",
            row.steps
        );
    }

    #[test]
    fn known_leak_rows_that_name_different_holders_at_a_line_give_none() {
        let text = r#"
[[leak]]
id = "a"
runtime = "go"
holder = "registration-table"
when = { source_regex = 'time\.NewTicker' }
[[leak]]
id = "b"
runtime = "go"
holder = "static/global"
when = { all = [ { source_regex = 'NewTicker' } ] }
[[leak]]
id = "c"
runtime = "go"
holder = "thread-local"
when = { not = { source_regex = 'NewTicker' } }
"#;
        let truths: BTreeSet<String> = ["a", "b", "c"].iter().map(|s| (*s).to_owned()).collect();
        assert_eq!(
            known_leak_holder(text, &truths, "go", "t := time.NewTicker(d)"),
            KnownHolder::Ambiguous
        );
        let only_a: BTreeSet<String> = ["a", "c"].iter().map(|s| (*s).to_owned()).collect();
        // `c` has its regex under `not`: it never matches.
        assert!(matches!(
            known_leak_holder(text, &only_a, "go", "t := time.NewTicker(d)"),
            KnownHolder::One { ref holder, .. } if holder == "registration-table"
        ));
        assert_eq!(
            known_leak_holder(text, &truths, "python", "time.NewTicker"),
            KnownHolder::None
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_sibling_the_discriminator_cannot_refute_blocks_the_confirmation() {
        let (row, calls) = drive_late(LATE_LEDGER, &[("FAKE_NO_REFUTE", "1")], &[("F1", "E8")]);
        assert!(!calls.contains("ledger confirm"), "{calls}");
        assert!(row.failures.is_empty(), "{:?}", row.failures);
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
        let v = json!({"ops": {"alloc-stack-diff": {"native": ["valgrind-memcheck", "dhat"],
                                                    "go": ["go-pprof-heap"]}},
                       "missing": ["dhat"]});
        let got = parse_adapters(&v);
        assert_eq!(got.len(), 3);
        assert!(
            got.iter()
                .any(|a| a.backend == "dhat" && a.runtime == "native" && !a.available)
        );
        assert!(
            got.iter()
                .any(|a| a.backend == "go-pprof-heap" && a.runtime == "go" && a.available)
        );
    }
}
