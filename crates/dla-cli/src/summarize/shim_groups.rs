//! `type-histogram-diff` over the allocator shim's exit records
//! (`harness/H<n>/out/<label>/dump.jsonl`): live bytes and blocks per allocation stack.
//!
//! The shim records, per distinct call stack, the first return addresses outside itself as
//! module offsets. This reader resolves them with `addr2line` against the modules the record
//! names (the binary of the harness's scratch copy), keeps the first eight frames as the
//! stack's key and takes the first frame inside the project as its site, the way the
//! harness does for its own `native.live_bytes@file:line` columns.
//!
//! Rows are keyed by stack, not by the shim's hash (which changes with address-space
//! randomization). A stack with a project site ties to a finding only by that site (rule 2
//! of the tie rule): the finding's own site or one of its recorded writers. A stack with no
//! project frame has no site and ties to nothing. A passing column needs three or more
//! captures; a site tie learns no writer.

use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use dla_workload::harness::dumps;

use dla_core::{
    FindingId, HarnessDump,
    coverage::BackendCoverage,
    evidence::{Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal, tool_class},
    series::{SeriesRegion, SeriesSourceKind, Unit},
    toolctx::EvidenceSpec,
};
use serde_json::{Map, Value, json};

use super::{
    Adapter, AdapterOut, CandidateRow, CoverageRow, Ctx, Op, Opts, Recorded, emit,
    exec::run_to_file,
    growth,
    input::Input,
    redact::Redactor,
    rule::{self, Observed},
    tie,
};
use crate::fail::{Failure, Res};

/// Frames that make a stack's key.
const KEY_FRAMES: usize = 8;
/// Stacks followed in the series.
const TRACKED: usize = 32;
/// Stacks kept in the raw file.
const RAW_STACKS: usize = 200;
/// Longest label shown.
const NAME_CLIP: usize = 72;
/// Largest dump file read.
const MAX_BYTES: u64 = 64 << 20;
/// Offsets handed to one addr2line call.
const CHUNK: usize = 512;
/// Words of a command line shown to name a process.
const ARGV_TAIL: usize = 3;
/// Longest word of a command line shown whole.
const ARGV_WORD: usize = 28;

/// The shim stack group reader.
pub struct ShimStackGroups;

fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_owned()
    } else {
        let mut t: String = s.chars().take(n).collect();
        t.push_str("...");
        t
    }
}

fn hex_or_dec(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => {
            let t = s.trim();
            match t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
                Some(h) => u64::from_str_radix(h, 16).ok(),
                None => t.parse().ok(),
            }
        }
        _ => None,
    }
}

/// One raw stack: live figures and `(module, offset)` frames, innermost first.
struct RawStack {
    bytes: u64,
    blocks: u64,
    frames: Vec<(usize, u64)>,
}

fn raw_stacks(raw_sites: &Value) -> Vec<RawStack> {
    let Some(map) = raw_sites.as_object() else {
        return Vec::new();
    };
    map.values()
        .filter_map(|s| {
            let o = s.as_object()?;
            let frames = o
                .get("frames")?
                .as_array()?
                .iter()
                .filter_map(|f| {
                    Some((
                        usize::try_from(hex_or_dec(f.get("m")?)?).ok()?,
                        hex_or_dec(f.get("o")?)?,
                    ))
                })
                .collect();
            Some(RawStack {
                bytes: o
                    .get("live_bytes")
                    .and_then(Value::as_f64)
                    .map_or(0, |b| b.max(0.0) as u64),
                blocks: o
                    .get("live_blocks")
                    .and_then(Value::as_f64)
                    .map_or(0, |b| b.max(0.0) as u64),
                frames,
            })
        })
        .collect()
}

/// `(file, line, function)` of each inline frame of an address, innermost first.
type Resolved = Vec<(String, u32, String)>;

fn parse_addr2line(out: &str, count: usize) -> Vec<Resolved> {
    let mut groups: Vec<Resolved> = Vec::new();
    let mut pending_fn = false;
    let mut func = String::new();
    for line in out.lines() {
        let t = line.trim_end();
        if t.starts_with("0x") && t[2..].bytes().all(|b| b.is_ascii_hexdigit()) && !pending_fn {
            groups.push(Vec::new());
            continue;
        }
        let Some(group) = groups.last_mut() else {
            continue;
        };
        if !pending_fn {
            pending_fn = true;
            func = t.to_owned();
            continue;
        }
        pending_fn = false;
        // Only a trailing ` (discriminator N)` is dropped: a directory may hold ` (`.
        let loc = match t.rfind(" (discriminator ") {
            Some(i) if t.ends_with(')') => &t[..i],
            _ => t,
        };
        if let Some((file, ln)) = loc.rsplit_once(':')
            && let Ok(ln) = ln.parse::<u32>()
            && file != "??"
            && ln > 0
        {
            group.push((file.to_owned(), ln, func.clone()));
        }
    }
    groups.resize(count, Vec::new());
    groups
}

/// Longest one `addr2line` call may run.
const ADDR2LINE_TIMEOUT: Duration = Duration::from_secs(60);
/// Most `addr2line` calls per record, and the longest they may take in all.
const ADDR2LINE_CALLS: usize = 64;
const ADDR2LINE_BUDGET: Duration = Duration::from_secs(180);

fn run_addr2line(
    module: &Path,
    offsets: &[u64],
    tmp: &Path,
    left: Duration,
) -> Option<Vec<Resolved>> {
    let mut args: Vec<OsString> = ["-a", "-f", "-C", "-i", "-e"]
        .iter()
        .map(OsString::from)
        .collect();
    args.push(module.as_os_str().to_owned());
    args.extend(offsets.iter().map(|o| OsString::from(format!("{o:#x}"))));
    let out = tmp.join(format!("addr2line-{}.txt", std::process::id()));
    let run = run_to_file(
        Path::new("addr2line"),
        &args,
        &out,
        ADDR2LINE_TIMEOUT.min(left),
    );
    let text = run
        .as_ref()
        .ok()
        .filter(|r| r.ok)
        .and_then(|_| fs::read(&out).ok());
    let _ = fs::remove_file(&out);
    text.map(|b| parse_addr2line(&String::from_utf8_lossy(&b), offsets.len()))
}

/// The scratch project of the harness run an input file belongs to
/// (`.dla/<run>/harness/H<n>/work`), when the path says so.
fn scratch_of(input: &Input) -> Option<PathBuf> {
    let s = input.abs.to_string_lossy().replace('\\', "/");
    let i = s.find("/harness/H")?;
    let rest = &s[i + "/harness/H".len()..];
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    (digits > 0).then(|| PathBuf::from(format!("{}/harness/H{}/work", &s[..i], &rest[..digits])))
}

/// A source file as a project-relative path: under the scratch copy or under the project.
/// Build output and files outside both are not project frames.
fn project_file(scratch: Option<&Path>, project: &Path, file: &str) -> Option<String> {
    let f = file.replace('\\', "/");
    let p = Path::new(&f);
    let rel = if p.is_absolute() {
        let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        let from_scratch = scratch.and_then(|w| {
            let w = w.canonicalize().unwrap_or_else(|_| w.to_path_buf());
            canon
                .strip_prefix(&w)
                .or_else(|_| p.strip_prefix(&w))
                .ok()
                .map(Path::to_path_buf)
        });
        from_scratch.or_else(|| {
            let rel = canon
                .strip_prefix(project)
                .or_else(|_| p.strip_prefix(project))
                .ok()?;
            (!rel.starts_with(".dla")).then(|| rel.to_path_buf())
        })?
    } else {
        let trimmed = p.strip_prefix(".").unwrap_or(p);
        let in_scratch = scratch.is_some_and(|w| w.join(trimmed).exists());
        if !in_scratch && !project.join(trimmed).exists() {
            return None;
        }
        trimmed.to_path_buf()
    };
    let s = rel.to_string_lossy().replace('\\', "/");
    let outside = s.starts_with("../")
        || s.starts_with("target/")
        || s.starts_with("node_modules/")
        || s.starts_with(".cargo/");
    (!outside && !s.is_empty()).then_some(s)
}

/// One stack of one capture, symbolized.
#[derive(Debug, Clone)]
struct Stack {
    key: String,
    /// `function (file:line)` of the first project frame, else of the first frame.
    label: String,
    /// `file:line` of the first project frame.
    site: Option<String>,
    frames: Vec<String>,
    bytes: u64,
    blocks: u64,
}

struct ShimCapture {
    label: String,
    /// Process the record was read from, and the last words of its command line.
    pid: Option<u32>,
    argv_tail: String,
    x: Option<f64>,
    sha256: String,
    stacks: Vec<Stack>,
    unattributed: u64,
}

/// One process's shim record with stacks.
struct ShimProc<'a> {
    d: &'a HarnessDump,
    raw: Vec<RawStack>,
    total: u64,
}

/// The last words of a command line, enough to tell the project's tests from the program.
/// Each word passes the redactor before it is cut, so a `key=value` word keeps its `key=`
/// for the redactor to see, however long the value is.
fn argv_tail(argv: Option<&[String]>, red: &Redactor) -> String {
    let words = argv.unwrap_or_default();
    let from = words.len().saturating_sub(ARGV_TAIL);
    words[from..]
        .iter()
        .map(|w| {
            let w = red.name(w);
            // A long word (a class path) keeps its end, where the file name is.
            let n = w.chars().count();
            if n > ARGV_WORD {
                let end: String = w.chars().skip(n - ARGV_WORD).collect();
                format!("...{end}")
            } else {
                w
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// The iteration count the capture was taken at, when it is a whole number.
fn count_of(x: Option<f64>) -> Option<u64> {
    x.filter(|x| *x >= 1.0 && x.fract() == 0.0 && *x < 1e15)
        .map(|x| x as u64)
}

/// Chooses the process whose record stands for one capture: tool processes are dropped
/// (unless nothing else is left), then the processes whose argv carries the capture's
/// iteration count are preferred (the rule of the harness's `select_measured`, whose steps
/// are used here), and only when several processes are still left the one with most live
/// bytes is kept, and `limits` says so.
fn choose_process<'a>(
    mut procs: Vec<ShimProc<'a>>,
    shown: &str,
    red: &Redactor,
    limits: &mut BTreeSet<String>,
) -> Option<ShimProc<'a>> {
    if procs.is_empty() {
        return None;
    }
    let wrote = procs.len();
    let tool = |p: &ShimProc<'_>| {
        p.d.argv
            .as_deref()
            .and_then(|a| a.first())
            .map(String::as_str)
            .or(p.d.exe.as_deref())
            .is_some_and(dumps::is_tool_program)
    };
    if procs.iter().any(|p| !tool(p)) {
        procs.retain(|p| !tool(p));
    }
    let n = procs.iter().find_map(|p| count_of(p.d.x));
    let several = procs.len() > 1;
    let by_identity = n.is_some_and(|n| {
        dumps::keep_with_n(&mut procs, |p| {
            p.d.argv.as_deref().is_some_and(|a| dumps::argv_has_n(a, n))
        })
    });
    if procs.len() == 1 {
        if several && by_identity {
            // One line per capture file's choice, merged into one text by the set: the choice
            // rests on the number being in an argv.
            limits.insert(
                "several processes wrote shim records with stacks in a capture; the one whose argv carries the capture's iteration count was read (the same number in another process's argv would have chosen that one)"
                    .to_owned(),
            );
        }
        return procs.pop();
    }
    let sizes: Vec<u64> = procs.iter().map(|p| p.total).collect();
    let at = {
        let best = dumps::largest_by(&procs, |p| p.total as f64)?;
        procs.iter().position(|p| std::ptr::eq(p, best))?
    };
    let kept = procs.swap_remove(at);
    // Command-line words pass the redactor, as in the raw file: a limitation reaches the
    // ledger and the CI log.
    let words = argv_tail(kept.d.argv.as_deref(), red);
    let words = if words.is_empty() {
        String::new()
    } else {
        format!(" ({words})")
    };
    limits.insert(format!(
        "{shown}: {wrote} processes wrote shim records with stacks, {} were left after the tool and identity steps and {}; kept pid {}{} as the one with most live bytes ({} of {})",
        sizes.len(),
        match (by_identity, n) {
            (true, Some(n)) => format!("{n} (the iteration count) is in the argv of several"),
            (_, Some(n)) => format!("none had {n} (the iteration count) in its argv"),
            _ => "the capture has no iteration count to tell them apart".to_owned(),
        },
        kept.d.pid.map_or_else(|| "?".to_owned(), |p| p.to_string()),
        words,
        kept.total,
        sizes.iter().map(u64::to_string).collect::<Vec<_>>().join(", "),
    ));
    Some(kept)
}

fn module_name(path: &str) -> String {
    path.rsplit(['/', '\\']).next().unwrap_or(path).to_owned()
}

/// Symbolizes the stacks of one record.
fn symbolize(
    raw: &[RawStack],
    modules: Option<&Value>,
    exe: Option<&str>,
    scratch: Option<&Path>,
    project: &Path,
    tmp: Option<&Path>,
    limits: &mut BTreeSet<String>,
) -> Vec<Stack> {
    let paths: Vec<String> = modules
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .enumerate()
                .map(|(i, m)| {
                    let p = m.get("path").and_then(Value::as_str).unwrap_or("");
                    if p.is_empty() && i == 0 {
                        exe.unwrap_or("").to_owned()
                    } else {
                        p.to_owned()
                    }
                })
                .collect()
        })
        .unwrap_or_default();
    let mut wanted: BTreeMap<usize, BTreeSet<u64>> = BTreeMap::new();
    for s in raw {
        for (m, o) in &s.frames {
            if paths
                .get(*m)
                .is_some_and(|p| !p.is_empty() && Path::new(p).is_file())
            {
                wanted.entry(*m).or_default().insert(*o);
            }
        }
    }
    let mut resolved: BTreeMap<(usize, u64), Resolved> = BTreeMap::new();
    let mut tool_failed = false;
    let mut calls = 0usize;
    let started = Instant::now();
    'modules: for (m, offsets) in &wanted {
        let offsets: Vec<u64> = offsets.iter().copied().collect();
        for chunk in offsets.chunks(CHUNK) {
            let left = ADDR2LINE_BUDGET.saturating_sub(started.elapsed());
            if calls >= ADDR2LINE_CALLS || left.is_zero() {
                limits.insert(format!(
                    "addr2line was stopped after {calls} calls or {} s: the remaining offsets have no site",
                    ADDR2LINE_BUDGET.as_secs()
                ));
                break 'modules;
            }
            calls += 1;
            let Some(tmp) = tmp else {
                continue 'modules;
            };
            match run_addr2line(Path::new(&paths[*m]), chunk, tmp, left) {
                Some(groups) => {
                    for (o, g) in chunk.iter().zip(groups) {
                        resolved.insert((*m, *o), g);
                    }
                }
                None => tool_failed = true,
            }
        }
    }
    if tool_failed {
        limits.insert(
            "addr2line is missing or failed (install binutils): stacks are keyed by module offsets and have no site, so none ties to a finding".into(),
        );
    }
    let mut cache: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut out = Vec::new();
    for s in raw {
        let mut parts: Vec<String> = Vec::new();
        let mut site: Option<(String, String)> = None;
        for (m, o) in &s.frames {
            let r = resolved.get(&(*m, *o));
            let mut named = false;
            for (file, line, func) in r.map(|v| v.as_slice()).unwrap_or_default() {
                let rel = cache
                    .entry(file.clone())
                    .or_insert_with(|| project_file(scratch, project, file))
                    .clone();
                match rel {
                    Some(rel) => {
                        let at = format!("{rel}:{line}");
                        parts.push(format!("{func}@{at}"));
                        if site.is_none() && !func.is_empty() && func != "??" {
                            site = Some((at, func.clone()));
                        }
                    }
                    None => parts.push(func.clone()),
                }
                named = true;
            }
            if !named {
                let module = paths.get(*m).map_or("?", |p| p.as_str());
                parts.push(format!("{}+{o:#x}", module_name(module)));
            }
        }
        let key = parts
            .iter()
            .take(KEY_FRAMES)
            .cloned()
            .collect::<Vec<_>>()
            .join(" < ");
        let label = match &site {
            Some((at, func)) => format!("{func} ({at})"),
            None => parts
                .first()
                .cloned()
                .unwrap_or_else(|| "(no frames)".into()),
        };
        out.push(Stack {
            key,
            label,
            site: site.map(|(at, _)| at),
            frames: parts.into_iter().take(KEY_FRAMES).collect(),
            bytes: s.bytes,
            blocks: s.blocks,
        });
    }
    out
}

fn not_found() -> Failure {
    Failure::refuse(
        "malformed-artifact",
        "the dump files hold no allocator shim stack groups (raw_sites)".to_owned(),
        "dump files of a native or Rust harness run, or of a JVM run with the shim preloaded",
        "dla harness run <H>   on Linux or macOS, then dla summarize type-histogram-diff <the dump files>",
    )
}

impl Adapter for ShimStackGroups {
    fn name(&self) -> &'static str {
        "shim-stack-groups"
    }
    fn op(&self) -> Op {
        Op::TypeHistogramDiff
    }
    fn runtime(&self) -> &'static str {
        "native"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::ALLOC_SHIM
    }
    fn formats(&self) -> &'static str {
        "harness dump jsonl (harness/H<n>/out/<label>/dump.jsonl) with the allocator shim's raw_sites"
    }
    fn sniff(&self, input: &Input) -> bool {
        let h = input.head_text();
        let first = h.lines().find(|l| !l.trim().is_empty()).unwrap_or_default();
        first.starts_with('{')
            && first.contains("\"dla_harness\"")
            && h.contains("\"raw_sites\"")
            && !h.contains("\"raw_sites\":{}")
            && !h.contains("\"types\":[")
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        shim_run(ctx, Op::TypeHistogramDiff, inputs, opts)
    }
}

fn shim_run(ctx: &Ctx<'_>, op: Op, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
    {
        let tc = ctx.tc;
        let mut limits: BTreeSet<String> = BTreeSet::new();
        let mut captures: Vec<ShimCapture> = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let (dumps, _bad) = HarnessDump::parse_lines(&text);
            let scratch = scratch_of(i);
            // One record per process: its last record that carries stacks. Of several
            // processes (a build tool, the project's tests, the program) the record is
            // chosen by identity, the way the harness chooses the measured process.
            let mut procs: Vec<ShimProc<'_>> = Vec::new();
            for d in &dumps {
                let Some(raw_sites) = &d.raw_sites else {
                    continue;
                };
                let raw = raw_stacks(raw_sites);
                if raw.is_empty() {
                    continue;
                }
                let total: u64 = raw.iter().fold(0u64, |a, s| a.saturating_add(s.bytes));
                let p = ShimProc { d, raw, total };
                match d
                    .pid
                    .and_then(|pid| procs.iter().position(|q| q.d.pid == Some(pid)))
                {
                    Some(at) => procs[at] = p,
                    None => procs.push(p),
                }
            }
            let Some(chosen) = choose_process(procs, i.shown(), &ctx.redactor, &mut limits) else {
                continue;
            };
            let (d, raw) = (chosen.d, chosen.raw);
            let stacks = symbolize(
                &raw,
                d.modules.as_ref(),
                d.exe.as_deref(),
                scratch.as_deref(),
                &tc.project,
                // A dump from another machine names modules DLA must not run tools on.
                (!ctx.artifact_run)
                    .then(|| tc.tmp_dir().ok())
                    .flatten()
                    .as_deref(),
                &mut limits,
            );
            let unattributed = stacks
                .iter()
                .filter(|s| s.site.is_none())
                .fold(0u64, |a, s| a.saturating_add(s.bytes));
            captures.push(ShimCapture {
                label: i.shown().to_owned(),
                pid: d.pid,
                argv_tail: argv_tail(d.argv.as_deref(), &ctx.redactor),
                x: d.x,
                sha256: i.sha256.clone(),
                stacks,
                unattributed,
            });
        }
        if captures.is_empty() {
            return Err(not_found());
        }
        let ordered_by = if captures.len() == 1 {
            "single"
        } else if captures.iter().all(|c| c.x.is_some()) {
            captures.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
            "x"
        } else {
            "argument"
        };
        let addr2line_gap = limits.iter().any(|l| l.starts_with("addr2line is missing"))
            && Command::new("addr2line")
                .arg("--version")
                .stdin(Stdio::null())
                .output()
                .is_err();
        let mut out = run_diff(ctx, op, opts, captures, ordered_by, limits)?;
        if addr2line_gap {
            out.coverage.push(CoverageRow {
                worker: "native".to_owned(),
                backend: "addr2line".to_owned(),
                cov: BackendCoverage::missing(
                    "addr2line was not found on PATH, so allocation stacks have no source sites",
                    "install binutils (apt-get install binutils, or xcode-select --install)",
                ),
            });
        }
        Ok(out)
    }
}

struct Row {
    key: String,
    label: String,
    site: Option<String>,
    frames: Vec<String>,
    bytes: Vec<Option<u64>>,
    blocks: Vec<Option<u64>>,
}

fn run_diff(
    ctx: &Ctx<'_>,
    op: Op,
    opts: &Opts,
    captures: Vec<ShimCapture>,
    ordered_by: &'static str,
    limits: BTreeSet<String>,
) -> Res<AdapterOut> {
    let tc = ctx.tc;
    let red = &ctx.redactor;
    let n = captures.len();
    // ---- rows: one per stack key
    let mut rows: BTreeMap<String, Row> = BTreeMap::new();
    for (ci, c) in captures.iter().enumerate() {
        for s in &c.stacks {
            let r = rows.entry(s.key.clone()).or_insert_with(|| Row {
                key: s.key.clone(),
                label: s.label.clone(),
                site: if ctx.artifact_run {
                    None
                } else {
                    s.site.clone()
                },
                frames: s.frames.clone(),
                bytes: vec![None; n],
                blocks: vec![None; n],
            });
            let add = |slot: &mut Option<u64>, v: u64| *slot = Some(slot.unwrap_or(0) + v);
            add(&mut r.bytes[ci], s.bytes);
            add(&mut r.blocks[ci], s.blocks);
        }
    }
    let delta = |v: &[Option<u64>]| -> i128 {
        let first = v.first().copied().flatten().unwrap_or(0);
        let last = v.last().copied().flatten().unwrap_or(0);
        if n == 1 {
            i128::from(last)
        } else {
            i128::from(last) - i128::from(first)
        }
    };
    let mut ranked: Vec<&Row> = rows.values().collect();
    ranked.sort_by(|a, b| {
        delta(&b.bytes)
            .cmp(&delta(&a.bytes))
            .then_with(|| a.key.cmp(&b.key))
    });
    let monotone = |r: &Row| -> bool {
        n >= rule::MIN_DUMPS
            && rule::gate(&Observed {
                bytes: r.bytes.clone(),
                counts: r.blocks.clone(),
            })
            .is_ok()
    };
    let mono: Vec<&Row> = ranked.iter().copied().filter(|r| monotone(r)).collect();

    // ---- the stacks that grow, and the findings their first project frame ties to (rule 2
    // of the tie rule: the finding's own site or one of its recorded writers)
    let alloc = op == Op::AllocStackDiff;
    let mut ledger_ref = ctx.ledger()?;
    let ledger = &mut *ledger_ref;
    let who = tie::Who::new(tc.worker.as_deref(), None);
    let mut picked: Vec<&Row> = Vec::new();
    if n >= rule::MIN_DUMPS {
        // The monotone stacks first, then the largest.
        picked = mono.clone();
        for r in &ranked {
            if picked.len() >= TRACKED {
                break;
            }
            if !picked.iter().any(|p| std::ptr::eq(*p, *r)) {
                picked.push(r);
            }
        }
        picked.truncate(TRACKED);
    }
    let resolved: Vec<tie::Resolved> = picked
        .iter()
        .map(|r| {
            let matches: Vec<_> = r
                .site
                .as_deref()
                .map(|site| {
                    tie::findings_at_site(ledger, &who, site)
                        .into_iter()
                        .map(|f| (f, tie::TieReason::site(site)))
                        .collect()
                })
                .unwrap_or_default();
            tie::resolve(matches, opts.finding.as_ref())
        })
        .collect();

    // ---- raw file
    let (hid, hrel) = emit::next_histogram(tc)?;
    let mut by_peak: Vec<&Row> = rows.values().collect();
    by_peak.sort_by_key(|r| {
        (
            std::cmp::Reverse(r.bytes.iter().flatten().max().copied().unwrap_or(0)),
            r.key.clone(),
        )
    });
    let raw_value = json!({
        "id": hid,
        "tool": "shim-stack-groups",
        "ordered_by": ordered_by,
        "captures": captures.iter().map(|c| json!({
            "label": c.label,
            "x": c.x,
            "pid": c.pid,
            "argv_tail": c.argv_tail,
            "stacks": c.stacks.len(),
            "unattributed_bytes": c.unattributed,
            "sha256": c.sha256,
        })).collect::<Vec<_>>(),
        "stacks": by_peak.iter().take(RAW_STACKS).map(|r| json!({
            "label": red.name(&r.label),
            "site": r.site,
            "frames": r.frames.iter().map(|f| red.name(f)).collect::<Vec<_>>(),
            "bytes": r.bytes,
            "blocks": r.blocks,
        })).collect::<Vec<_>>(),
        "monotone": mono.iter().map(|r| red.name(&r.label)).collect::<Vec<_>>(),
        "redaction": red.mode(),
    });
    let raw = emit::write_raw(tc, &hrel, &raw_value)?;

    // ---- the histogram-diff evidence
    let shas: Vec<&str> = captures.iter().map(|c| c.sha256.as_str()).collect();
    let signal = Signal::new(
        "shim-stack-groups",
        tool_class::ALLOC_SHIM,
        &emit::combined_sha(&shas),
    );
    let authority = ctx.authority(Authority::Live);
    let top_row = |r: &Row, short: bool| -> Value {
        let mut v = json!({"at": clip(&red.name(&r.label), if short { 40 } else { NAME_CLIP })});
        if let Some(s) = &r.site {
            v["site"] = json!(s);
        }
        v["d_bytes"] = json!(emit::jn(delta(&r.bytes)));
        v["d_blocks"] = json!(emit::jn(delta(&r.blocks)));
        v
    };
    let mut base = Map::new();
    base.insert("mode".into(), json!(if n == 1 { "single" } else { "diff" }));
    base.insert("captures".into(), json!(n));
    base.insert("ordered_by".into(), json!(ordered_by));
    base.insert("stacks".into(), json!(rows.len()));
    let mut list = emit::Bounded::new(base, "top", emit::RESULT_BUDGET - 200);
    for r in ranked.iter().take(8) {
        if delta(&r.bytes) <= 0 {
            break;
        }
        list.push(top_row(r, false));
    }
    let mut result = list.finish();
    result["monotone"] = json!(mono.len());
    if alloc {
        // The ties of the growing stacks, stated in the record itself: `dla ledger label`
        // counts an allocation-stack record for the findings named here (the ties the
        // growth columns get below, by site) and for no other.
        let mut rows: Vec<Value> = Vec::new();
        let mut ids: Vec<FindingId> = Vec::new();
        for (r, rs) in picked.iter().zip(&resolved) {
            let Some((f, why)) = &rs.tie else { continue };
            if !monotone(r) || rows.len() >= 8 {
                continue;
            }
            let mut all: Vec<FindingId> = std::iter::once(f.clone())
                .chain(rs.also.iter().cloned())
                .collect();
            all.sort();
            all.dedup();
            rows.push(json!({
                "stack": clip(&red.name(&r.label), 40),
                "finding": f,
                "candidates": all,
                "tie": why.to_json(),
            }));
            ids.extend(all);
        }
        ids.sort();
        ids.dedup();
        if let Some(first) = ids.first() {
            result["finding"] = json!(first);
            result["candidates"] = json!(ids);
            result["ties"] = json!(rows);
        }
    }
    let mut spec = EvidenceSpec::new(
        if alloc {
            EvidenceKind::AllocStack
        } else {
            EvidenceKind::HistogramDiff
        },
        "shim-stack-groups",
        env!("CARGO_PKG_VERSION"),
        result,
    );
    spec.authority = authority;
    spec.source = Some(tc.artifact_source(&hrel)?);
    spec.coverage = EvidenceCoverage {
        status: CoverageStatus::Complete,
        examined: Some(rows.len() as u64),
        total: None,
    };
    spec.limitations = vec![
        "stacks are the shim's live allocations at exit, keyed by their first eight resolved frames; the shim's own hash changes between processes".into(),
        "each capture is the exit record of a separate process: the figures grow with the work done before exit".into(),
    ];
    spec.limitations.extend(limits.iter().cloned());
    if n < rule::MIN_DUMPS {
        spec.limitations.push(
            "fewer than three captures cannot tell a warm-up fill from a leak: no growth column is written".into(),
        );
    }
    let unattributed: u64 = captures.iter().map(|c| c.unattributed).max().unwrap_or(0);
    if unattributed > 0 {
        spec.limitations.push(format!(
            "up to {unattributed} live bytes sit in stacks with no frame in the project: they have no site and tie to nothing"
        ));
    }
    let diff_ev: Recorded = ctx.record(ledger, spec, &signal, &raw)?;
    let mut out = AdapterOut {
        raw: Some(raw.clone()),
        enumerated: rows.len() as u64,
        ..AdapterOut::default()
    };
    let mut body = Map::new();
    body.insert("mode".into(), json!(if n == 1 { "single" } else { "diff" }));
    body.insert("captures".into(), json!(n));
    body.insert("ordered_by".into(), json!(ordered_by));
    body.insert(
        "top".into(),
        json!(
            ranked
                .iter()
                .take(opts.top.clamp(1, 5))
                .filter(|r| delta(&r.bytes) > 0)
                .map(|r| top_row(r, true))
                .collect::<Vec<_>>()
        ),
    );
    body.insert("monotone".into(), json!(mono.len()));
    body.insert("redaction".into(), json!(red.mode()));
    out.evidence.push(diff_ev);
    if n < rule::MIN_DUMPS {
        body.insert("series".into(), Value::Null);
        body.insert("gate".into(), json!("fail"));
        body.insert("reasons".into(), json!(["captures < 3"]));
        body.insert(
            "needed".into(),
            json!("2 more captures of runs with more work (3 confirm growth); rerun with all dump files"),
        );
        out.body = body;
        return Ok(out);
    }

    // ---- series and growth
    let mut tracks: Vec<growth::Track> = Vec::new();
    for (r, r_tie) in picked.iter().zip(resolved) {
        let mut t = growth::Track {
            name: format!("stack:{}", clip(&red.name(&r.label), 100)),
            unit: Unit::Bytes,
            values: r.bytes.clone(),
            counts: Some(r.blocks.clone()),
            site: None,
            finding: None,
            candidates: Vec::new(),
            region: SeriesRegion::OffHeap,
            tie: None,
            ambiguous: r_tie.ambiguous,
        };
        if let Some((f, why)) = r_tie.tie {
            let site = tie::site_of(ledger, &f);
            t.tie_to_all(f, &r_tie.also, why, site);
        }
        tracks.push(t);
    }
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for t in &mut tracks {
        let k = seen.entry(t.name.clone()).or_insert(0);
        *k += 1;
        if *k > 1 {
            t.name = format!("{}#{k}", t.name);
        }
    }
    let ambiguous: Vec<(String, Vec<FindingId>)> = tracks
        .iter()
        .zip(&picked)
        .filter(|(t, r)| !t.ambiguous.is_empty() && monotone(r))
        .map(|(t, _)| (t.name.clone(), t.ambiguous.clone()))
        .collect();
    let g = growth::write(
        ctx,
        ledger,
        growth::Plan {
            t_ms: vec![0; n],
            tracks,
            signal,
            tool: "shim-stack-groups",
            authority,
            command: if alloc {
                "dla summarize alloc-stack-diff"
            } else {
                "dla summarize type-histogram-diff"
            }
            .into(),
            source_kind: SeriesSourceKind::Dump,
            runtime: None,
            limitations: spec_limits(&limits),
            what: "capture",
            focus: opts.finding.clone(),
        },
    )?;
    let tied_ids = growth::tied_ids(&g, opts.finding.as_ref());
    for f in &tied_ids {
        let fnd = ledger.finding(f);
        out.candidates.push(CandidateRow {
            finding: f.clone(),
            owner: fnd.and_then(|x| x.owner.clone()),
            holder: fnd.and_then(|x| x.holder),
            site: fnd.and_then(|x| x.site.clone()),
            region: fnd.and_then(|x| x.region),
            origin: "growth-series".into(),
            added: false,
            via: None,
        });
    }
    if let Value::Object(m) = g.json.clone() {
        body.extend(m);
    }
    body.insert("tied".into(), json!(tied_ids));
    if let Some(a) = growth::ambiguous_json(&ambiguous) {
        body.insert("ambiguous".into(), a);
    }
    let needed = if !g.pass {
        "no stack's live bytes grow at every step: capture a point with more work and rerun with all dump files".to_owned()
    } else if let Some(f) = tied_ids.first() {
        format!(
            "a refuted non-leak sibling and a holder, then dla ledger confirm {f} --evidence {}",
            g.evidence.id
        )
    } else if let Some(f) = &opts.finding {
        format!(
            "no passing column ties to {f} by holder, site or type: a stack ties to the finding whose site or recorded writer is its first project frame"
        )
    } else {
        "the growing stacks tie to no finding: a stack ties to the finding at its first project frame (dla census so a holder exists there); a stack with no project frame ties to nothing".to_owned()
    };
    body.insert("needed".into(), json!(needed));
    out.evidence.push(g.evidence);
    out.body = body;
    Ok(out)
}

/// The shim's exit records as an `alloc-stack-diff` input: the same stacks, keyed and tied
/// the same way (by site only), recorded as allocation-stack evidence so that a native
/// leak's stacks can back a label that asks for one.
pub struct ShimAllocStacks;

impl Adapter for ShimAllocStacks {
    fn name(&self) -> &'static str {
        "shim-alloc-stacks"
    }
    fn op(&self) -> Op {
        Op::AllocStackDiff
    }
    fn runtime(&self) -> &'static str {
        "native"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::ALLOC_SHIM
    }
    fn formats(&self) -> &'static str {
        ShimStackGroups.formats()
    }
    fn sniff(&self, input: &Input) -> bool {
        ShimStackGroups.sniff(input)
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        shim_run(ctx, Op::AllocStackDiff, inputs, opts)
    }
}

fn spec_limits(limits: &BTreeSet<String>) -> Vec<String> {
    let mut v = vec![
        "stacks are keyed by their first eight resolved frames; each point is the exit record of a separate process".to_owned(),
    ];
    v.extend(limits.iter().cloned());
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One shim record per `(pid, argv, bytes)`, all at iteration count 500.
    fn records(spec: &[(u32, &[&str], u64)]) -> Vec<HarnessDump> {
        let text: String = spec
            .iter()
            .map(|(pid, argv, bytes)| {
                format!(
                    "{{\"dla_harness\":1,\"phase\":\"exit\",\"pid\":{pid},\"argv\":{},\"runtime\":\"native\",\"x\":500,\"raw_sites\":{{\"0x1\":{{\"live_bytes\":{bytes},\"live_blocks\":1,\"frames\":[]}}}}}}\n",
                    serde_json::to_string(argv).unwrap()
                )
            })
            .collect();
        HarnessDump::parse_lines(&text).0
    }

    fn choose(spec: &[(u32, &[&str], u64)]) -> (Option<u32>, Vec<String>) {
        choose_with(spec, &Redactor::new(b"s", true))
    }

    fn choose_with(spec: &[(u32, &[&str], u64)], red: &Redactor) -> (Option<u32>, Vec<String>) {
        let dumps = records(spec);
        let procs: Vec<ShimProc<'_>> = dumps
            .iter()
            .map(|d| ShimProc {
                d,
                raw: Vec::new(),
                total: d
                    .raw_sites
                    .as_ref()
                    .map(|r| r["0x1"]["live_bytes"].as_u64().unwrap())
                    .unwrap(),
            })
            .collect();
        let mut limits = BTreeSet::new();
        let kept = choose_process(procs, "f.jsonl", red, &mut limits).map(|p| p.d.pid.unwrap());
        (kept, limits.into_iter().collect())
    }

    #[test]
    fn the_process_with_the_iteration_count_wins_over_a_larger_one_and_a_tool_is_dropped() {
        let (kept, limits) = choose(&[
            (1, &["sh", "./build.sh", "run", "500"], 900),
            (2, &["java", "CodecTestKt"], 800),
            (3, &["java", "MainKt", "500"], 10),
        ]);
        assert_eq!(kept, Some(3));
        // Identity chose among two programs: one line says so (the same for every capture).
        assert_eq!(limits.len(), 1, "{limits:?}");
        assert!(
            limits[0].starts_with("several processes wrote shim records")
                && limits[0].contains("iteration count"),
            "{limits:?}"
        );
        // Only a tool next to the program: nothing was chosen between programs.
        let (kept, limits) = choose(&[(1, &["sh", "run", "500"], 900), (3, &["java", "500"], 1)]);
        assert_eq!((kept, limits.len()), (Some(3), 0));
        // A tool process is dropped even when it carries the count and nothing else does.
        let (kept, _) = choose(&[(1, &["sh", "run", "500"], 900), (2, &["java", "A"], 5)]);
        assert_eq!(kept, Some(2));
    }

    #[test]
    fn size_decides_only_among_equals_and_the_limitation_names_the_choice() {
        let (kept, limits) = choose(&[
            (7, &["java", "A"], 50),
            (8, &["java", "B"], 70),
            (9, &["sh", "x"], 1000),
        ]);
        assert_eq!(kept, Some(8));
        assert_eq!(limits.len(), 1, "{limits:?}");
        assert!(
            limits[0].contains(
                "3 processes wrote shim records with stacks, 2 were left after the tool and identity steps"
            ) && limits[0].contains("none had 500 (the iteration count) in its argv")
                && limits[0].contains("kept pid 8 (java B)")
                && limits[0].contains("(70 of 50, 70)"),
            "{limits:?}"
        );
        // Two with the count: the larger of those, and it is said.
        let (kept, limits) = choose(&[(1, &["java", "500"], 5), (2, &["java", "a=500"], 6)]);
        assert_eq!(kept, Some(2));
        assert!(
            limits[0].contains("is in the argv of several"),
            "{limits:?}"
        );
        // Three programs, two of them with the count: the number written is the three, not
        // the two left to choose between.
        let (kept, limits) = choose(&[
            (1, &["java", "Test"], 90),
            (2, &["java", "500"], 5),
            (3, &["java", "a=500"], 6),
        ]);
        assert_eq!(kept, Some(3));
        assert!(
            limits[0].contains(
                "3 processes wrote shim records with stacks, 2 were left after the tool and identity steps and 500 (the iteration count) is in the argv of several"
            ),
            "{limits:?}"
        );
    }

    #[test]
    fn the_limitation_shapes_secret_looking_argv_words_and_omits_empty_brackets() {
        let spec: &[(u32, &[&str], u64)] = &[
            (7, &["java", "shop.Main", "password=hunter2hunter2"], 50),
            (8, &["java", "shop.Main", "token=abc"], 40),
        ];
        let (kept, limits) = choose_with(spec, &Redactor::new(b"s", false));
        assert_eq!(kept, Some(7));
        assert!(
            limits[0].contains("shop.Main") && !limits[0].contains("hunter2hunter2"),
            "{limits:?}"
        );
        // With raw consent the words are shown, like the raw file's.
        let (_, raw) = choose_with(spec, &Redactor::new(b"s", true));
        assert!(raw[0].contains("hunter2hunter2"), "{raw:?}");
        // A value longer than the cut of a word is shaped as well: the redactor sees the
        // word before it is cut.
        let long = "password=abcdefghijklmnopqrstuvwxyzABCDEFGH";
        let spec: &[(u32, &[&str], u64)] = &[(7, &["java", long], 50), (8, &["java", "x"], 40)];
        let (_, limits) = choose_with(spec, &Redactor::new(b"s", false));
        assert!(!limits[0].contains("mnopqrstuvwxyz"), "{limits:?}");
        // A kept record without argv has no empty brackets.
        let (_, limits) = choose(&[(7, &[], 50), (8, &[], 40)]);
        assert!(
            limits[0].contains("kept pid 7 as the one") && !limits[0].contains("()"),
            "{limits:?}"
        );
    }

    #[test]
    fn a_lone_process_and_a_lone_tool_are_read_without_a_limitation() {
        let (kept, limits) = choose(&[(4, &["java", "A"], 5)]);
        assert_eq!((kept, limits.len()), (Some(4), 0));
        let (kept, limits) = choose(&[(4, &["cc", "A"], 5)]);
        assert_eq!((kept, limits.len()), (Some(4), 0));
    }

    #[test]
    fn a_long_argv_word_keeps_its_end() {
        let long = format!("out:{}/kotlin-stdlib.jar", "d/".repeat(40));
        let tail = argv_tail(
            Some(&[
                "java".to_owned(),
                long,
                "shop.codec.MainKt".to_owned(),
                "2220".to_owned(),
            ]),
            &Redactor::new(b"s", false),
        );
        assert!(
            tail.starts_with("...") && tail.contains("kotlin-stdlib.jar"),
            "{tail}"
        );
        assert!(tail.ends_with("shop.codec.MainKt 2220"), "{tail}");
    }

    #[test]
    fn a_path_holding_a_space_and_parenthesis_is_kept_and_only_a_discriminator_is_cut() {
        let out = "0x10\nfill\n/Users/me/proj (1)/src/a.c:12\n0x20\nfill2\n/Users/me/proj (1)/src/a.c:30 (discriminator 3)\n0x30\nplain\n/src/b.c:7 (discriminator 12)\n0x40\nnone\n??:0\n";
        let g = parse_addr2line(out, 4);
        assert_eq!(
            g[0],
            vec![(
                "/Users/me/proj (1)/src/a.c".to_owned(),
                12,
                "fill".to_owned()
            )]
        );
        assert_eq!(
            g[1],
            vec![(
                "/Users/me/proj (1)/src/a.c".to_owned(),
                30,
                "fill2".to_owned()
            )]
        );
        assert_eq!(g[2], vec![("/src/b.c".to_owned(), 7, "plain".to_owned())]);
        assert!(g[3].is_empty());
    }
}
