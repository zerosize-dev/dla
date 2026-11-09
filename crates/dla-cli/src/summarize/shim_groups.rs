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
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use dla_core::{
    FindingId, HarnessDump,
    coverage::BackendCoverage,
    evidence::{Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal, tool_class},
    series::{SeriesRegion, SeriesSourceKind, Unit},
    toolctx::EvidenceSpec,
};
use serde_json::{Map, Value, json};

use super::{
    Adapter, AdapterOut, CandidateRow, CoverageRow, Ctx, Op, Opts, Recorded, emit, growth,
    input::Input,
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
        let loc = t.split(" (").next().unwrap_or(t);
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

fn run_addr2line(module: &Path, offsets: &[u64]) -> Option<Vec<Resolved>> {
    let mut cmd = Command::new("addr2line");
    cmd.args(["-a", "-f", "-C", "-i", "-e"]).arg(module);
    for o in offsets {
        cmd.arg(format!("{o:#x}"));
    }
    let out = cmd
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| parse_addr2line(&String::from_utf8_lossy(&out.stdout), offsets.len()))
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
    x: Option<f64>,
    sha256: String,
    stacks: Vec<Stack>,
    unattributed: u64,
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
    for (m, offsets) in &wanted {
        let offsets: Vec<u64> = offsets.iter().copied().collect();
        for chunk in offsets.chunks(CHUNK) {
            match run_addr2line(Path::new(&paths[*m]), chunk) {
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
        let tc = ctx.tc;
        let mut limits: BTreeSet<String> = BTreeSet::new();
        let mut captures: Vec<ShimCapture> = Vec::new();
        for i in inputs {
            let text = i.read_text(MAX_BYTES)?;
            let (dumps, _bad) = HarnessDump::parse_lines(&text);
            let scratch = scratch_of(i);
            // One record per process: its exit record that carries stacks; of several
            // processes (a build tool's JVM next to the program), the one with most live bytes.
            let mut best: Option<(u64, &HarnessDump, Vec<RawStack>)> = None;
            for d in &dumps {
                let Some(raw_sites) = &d.raw_sites else {
                    continue;
                };
                let raw = raw_stacks(raw_sites);
                if raw.is_empty() {
                    continue;
                }
                let total: u64 = raw.iter().map(|s| s.bytes).sum();
                if best.as_ref().is_none_or(|(t, ..)| total >= *t) {
                    best = Some((total, d, raw));
                }
            }
            let Some((_, d, raw)) = best else { continue };
            let stacks = symbolize(
                &raw,
                d.modules.as_ref(),
                d.exe.as_deref(),
                scratch.as_deref(),
                &tc.project,
                &mut limits,
            );
            let unattributed = stacks
                .iter()
                .filter(|s| s.site.is_none())
                .map(|s| s.bytes)
                .sum();
            captures.push(ShimCapture {
                label: i.shown().to_owned(),
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
        let mut out = run_diff(ctx, opts, captures, ordered_by, limits)?;
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
        v["d_bytes"] = json!(delta(&r.bytes));
        v["d_blocks"] = json!(delta(&r.blocks));
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
    let mut spec = EvidenceSpec::new(
        EvidenceKind::HistogramDiff,
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
    let mut ledger_ref = ctx.ledger()?;
    let ledger = &mut *ledger_ref;
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

    // ---- series and growth: the monotone stacks first, then the largest
    let mut picked: Vec<&Row> = mono.clone();
    for r in &ranked {
        if picked.len() >= TRACKED {
            break;
        }
        if !picked.iter().any(|p| std::ptr::eq(*p, *r)) {
            picked.push(r);
        }
    }
    picked.truncate(TRACKED);
    let who = tie::Who::new(tc.worker.as_deref(), None);
    let mut tracks: Vec<growth::Track> = Vec::new();
    for r in &picked {
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
        let r_tie = tie::resolve(matches, opts.finding.as_ref());
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
            command: "dla summarize type-histogram-diff".into(),
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

fn spec_limits(limits: &BTreeSet<String>) -> Vec<String> {
    let mut v = vec![
        "stacks are keyed by their first eight resolved frames; each point is the exit record of a separate process".to_owned(),
    ];
    v.extend(limits.iter().cloned());
    v
}
