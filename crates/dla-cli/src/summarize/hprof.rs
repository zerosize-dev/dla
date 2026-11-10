//! `dla summarize heap-graph` over HPROF heap dumps (`.hprof`, `.hprof.gz`), read with DLA's
//! own reader.
//!
//! Each dump is summarized once (`summaries/`) and recorded as one `retention-path`
//! evidence. Two or more dumps, ordered by their header timestamps, also form a dump
//! series (`series/S<n>.json`): one `growth-series` evidence that applies the dump-series
//! rule, one `dump-plateau` discriminator per holder of interest, and a candidate finding
//! (owner, holder, region) for each holder that passes.
//!
//! On a repository run a passing holder is tied to the census: the finding with that
//! owner or site (no duplicate is added), or a new candidate whose site is the field's
//! declaration found by the owner-to-site resolver. An artifact run has no source and
//! never gets a site.

use std::collections::BTreeSet;

use dla_core::{
    Error, FindingId, Ledger, NewFinding, Origin,
    discriminator::DiscriminatorResult,
    error::codes as core_codes,
    evidence::{
        Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal, Source, tool_class,
    },
    finding::{Holder as FindingHolder, Region},
    fsutil::{to_pretty_json, write_atomic},
    ids::EvidenceId,
    series::{
        Axis, Column, ColumnKind, ColumnScope, Point, SeriesFile, SeriesRegion, SeriesSource,
        SeriesSourceKind, Unit,
    },
    toolctx::EvidenceSpec,
};
use dla_parsers::hprof::{self, HeapSummary, Holder, HolderKind, Options, Sniff};
use serde_json::{Map, Value, json};

use super::{
    Adapter, AdapterOut, CandidateRow, Ctx, Op, Opts, Recorded, compat, emit, growth,
    input::Input,
    rule::{self, Observed},
    tie,
};
use crate::fail::{Failure, Res};

/// Holder keys with their peak retained bytes.
type Peaks = Vec<(String, u64)>;

/// Most holders a dump series tracks (two columns each; a series file holds 64).
const SERIES_HOLDERS: usize = 32;
/// Most `dump-plateau` discriminators one run writes.
const MAX_DISCRIMINATORS: usize = 5;
/// Holders named in a retention-path evidence.
const EVIDENCE_HOLDERS: usize = 3;
/// Passing holders named in a growth-series evidence.
const EVIDENCE_PASSED: usize = 4;
/// Via holders named in a retention-path evidence.
const EVIDENCE_VIA: usize = 2;
/// Via holders named in the command output.
const OUTPUT_VIA: usize = 3;
/// Via holders always followed in the series, whatever their size.
const SERIES_VIA: usize = 8;

/// The DLA HPROF reader.
pub struct DlaHprof;

impl Adapter for DlaHprof {
    fn name(&self) -> &'static str {
        "dla-hprof"
    }
    fn op(&self) -> Op {
        Op::HeapGraph
    }
    fn runtime(&self) -> &'static str {
        "jvm"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::DUMP_PARSER
    }
    fn formats(&self) -> &'static str {
        SUPPORTED_SHORT
    }
    fn sniff(&self, input: &Input) -> bool {
        is_hprof(input)
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        heap_graph(ctx, inputs, opts)
    }
}

const SUPPORTED_SHORT: &str =
    ".hprof and .hprof.gz heap dumps (other dump formats arrive in a later milestone)";

/// Named like a dump, or starting like one.
pub fn is_hprof(input: &Input) -> bool {
    let named = input.has_suffix(".hprof") || input.has_suffix(".hprof.gz");
    named || hprof::sniff(&input.abs).is_ok_and(|s| s != Sniff::Other)
}

/// One parsed dump.
struct Dump {
    /// Project-relative (or absolute) path with `/` separators.
    rel: String,
    sha256: String,
    summary: HeapSummary,
    /// `summaries/<name>.json` relative to the run directory.
    summary_file: String,
    /// The iteration count `n` of the point, for a per-point series (separate processes).
    x: Option<f64>,
}

fn fail(code: &str, message: String, missing: &str, fix: &str) -> Failure {
    Failure::refuse(code, message, missing, fix)
}

/// `<stem>-<sha8>.json`, with the stem cleaned and shortened.
pub fn summary_name(rel: &str, sha: &str) -> String {
    let file = rel.rsplit('/').next().unwrap_or(rel);
    let stem = file
        .strip_suffix(".gz")
        .unwrap_or(file)
        .strip_suffix(".hprof")
        .unwrap_or(file.strip_suffix(".gz").unwrap_or(file));
    format!("{}-{}.json", emit::clean_stem(stem), &sha[..8])
}

fn parse_failure(rel: &str, e: &hprof::HprofError) -> Failure {
    use hprof::HprofError as H;
    match e {
        H::NotHprof(m) => fail(
            "unsupported-format",
            format!("{rel} is not an HPROF heap dump: {m} (supported: .hprof and .hprof.gz)"),
            "a .hprof or .hprof.gz file",
            "dla summarize heap-graph <dump>.hprof",
        ),
        H::Truncated(_) | H::Malformed(_) | H::Gzip(_) => fail(
            "malformed-artifact",
            format!("{rel} cannot be read: {e}"),
            "a complete heap dump",
            "take the dump again (jcmd <pid> GC.heap_dump <file>) and rerun dla summarize heap-graph",
        ),
        H::InflateTooLarge(_) => fail(
            "insufficient-resources",
            format!("{rel} cannot be read: {e}"),
            "a gzip heap dump of a plausible size",
            "inflate it yourself if it is genuine (gunzip -k <dump>.hprof.gz) and pass the .hprof, or take the dump again",
        ),
        H::DiskFull => fail(
            "insufficient-resources",
            format!("{rel} cannot be read: {e}"),
            "free disk space for the inflated dump (several times the .gz size)",
            "free disk space or point the run directory at a larger volume, then rerun dla summarize heap-graph",
        ),
        H::Io(io) => Failure::from(Error::io(
            rel,
            std::io::Error::new(io.kind(), io.to_string()),
        )),
    }
}

/// The path as shown in output: as given when short, else its last two components.
fn short_path(rel: &str) -> String {
    if rel.len() <= 48 {
        return rel.to_owned();
    }
    let parts: Vec<&str> = rel.rsplitn(3, '/').collect();
    format!(".../{}/{}", parts.get(1).unwrap_or(&""), parts[0])
}

fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_owned()
    } else {
        let mut t: String = s.chars().take(n).collect();
        t.push_str("...");
        t
    }
}

/// A holder as the output and the retention-path evidence show it. `holder_kind` is the
/// kind the dump shows (the same value the growth columns carry), so a record that names
/// the holder of a finding also says what kind of holder it is.
fn top_json(h: &Holder) -> Value {
    json!({
        "key": h.key,
        "retained": h.retained,
        "dominated": h.dominated_count,
        "holder_kind": observed_holder(h),
    })
}

/// The `holders` of a dump's retention-path evidence: the largest `n`, and the holder the
/// finding named with `--finding` (`bound`) when the dump has it and it is not among
/// them. A finding's holder is rarely among the largest of a process; without its entry
/// no retention record of the run would name it. The bound holder is kept when the
/// result must be cut to fit, the smallest of the largest go first.
fn evidence_holders(summary: &HeapSummary, n: usize, bound: Option<&str>) -> Vec<Value> {
    let mut top: Vec<&Holder> = summary.holders.iter().take(n).collect();
    if let Some(key) = bound
        && !top.iter().any(|h| h.key == key)
        && let Some(h) = summary
            .holders
            .iter()
            .chain(&summary.tracked)
            .find(|h| h.key == key)
    {
        top.push(h);
    }
    top.into_iter().map(top_json).collect()
}

fn finding_holder(h: &Holder) -> FindingHolder {
    match h.kind {
        HolderKind::Static => FindingHolder::StaticGlobal,
        HolderKind::JniGlobal => FindingHolder::NativeHandleGlobalRef,
        HolderKind::Instance => FindingHolder::ContainerField,
        HolderKind::ThreadLocal => FindingHolder::ThreadLocal,
        HolderKind::Thread => {
            let on_thread_local = h.path.iter().any(|p| p.contains("ThreadLocal"))
                || h.accumulation
                    .as_ref()
                    .is_some_and(|a| a.class.contains("ThreadLocal"));
            if on_thread_local {
                FindingHolder::ThreadLocal
            } else {
                FindingHolder::BlockedStack
            }
        }
    }
}

/// The holder kind DLA reports for a holder: the kind of its retention root. An instance
/// field whose retention path starts at a static field of a class is static/global; the
/// other kinds are the root's own.
fn observed_holder(h: &Holder) -> FindingHolder {
    if h.kind == HolderKind::Instance
        && let Some(rest) = h.path.first().and_then(|p| p.strip_prefix("root "))
        && (rest.starts_with("class ") || !rest.contains([' ', ':']))
    {
        return FindingHolder::StaticGlobal;
    }
    finding_holder(h)
}

/// `class.field` for a static or a via-holder instance field (`field:<Decl>.<f>`), else
/// the key.
fn owner_of(h: &Holder) -> String {
    match (h.kind, &h.field) {
        (HolderKind::Static | HolderKind::ThreadLocal, Some(f)) => {
            format!("{}.{f}", h.owner_class)
        }
        (HolderKind::Instance, Some(f)) if h.key.starts_with("field:") => {
            format!("{}.{f}", h.owner_class)
        }
        _ => h.key.clone(),
    }
}

/// Adds the summary's via holders (fields that keep a set alive although no root field
/// dominates it) to the holders tracked across dumps, under their own key
/// (`static:<C>.<f>` or `field:<Decl>.<f>`). A key the dump already tracks as a root
/// holder keeps its entry, with the larger of the two figures: the dominator figure
/// counts only what the field alone keeps alive (often just its container), the via
/// figure what is kept alive through it. The accumulation class is filled in when the
/// root holder has none. From here on they are holders like any other: they get
/// `holder:<key>` columns, pass the same gate, and tie by the same rule.
fn add_via_holders(summary: &mut HeapSummary) {
    let mut added: Vec<Holder> = Vec::new();
    for v in &summary.via_holders {
        match summary.tracked.iter_mut().find(|h| h.key == v.key) {
            Some(h) => {
                h.retained = h.retained.max(v.retained);
                h.dominated_count = h.dominated_count.max(v.dominated_count);
                if h.accumulation.is_none() {
                    h.accumulation = Some(v.accumulation.clone());
                }
            }
            None => added.push(Holder {
                key: v.key.clone(),
                kind: v.kind,
                owner_class: v.owner_class.clone(),
                field: Some(v.field.clone()),
                raw_owner_class: v.raw_owner_class.clone(),
                retained: v.retained,
                dominated_count: v.dominated_count,
                path: v.path.clone(),
                accumulation: Some(v.accumulation.clone()),
                context: Vec::new(),
            }),
        }
    }
    summary.tracked.extend(added);
}

/// The via holders of a summary, compact, for the output and the retention evidence.
fn via_json(summary: &HeapSummary, n: usize) -> Vec<Value> {
    summary
        .via_holders
        .iter()
        .take(n)
        .map(|v| {
            let mut j = json!({
                "key": clip(&v.key, 100),
                "retained": v.retained,
                "dominated": v.dominated_count,
                "by": clip(&v.dominator, 40),
            });
            // A via holder is tracked under its own key once added (see `add_via_holders`).
            if let Some(h) = summary.tracked.iter().find(|h| h.key == v.key) {
                j["holder_kind"] = json!(observed_holder(h));
            }
            j
        })
        .collect()
}

/// Longest letter word of a plain thread name, and longest digit run (an ordinal such as
/// the `3` of `pool-1-thread-3`, or a port; a longer run is an id).
const NAME_WORD_MAX: usize = 20;
const NAME_DIGITS_MAX: usize = 5;

/// True when a thread name is program structure: words of letters and short ordinals around
/// `-`, `_` and `.`. A token that mixes letters and digits (a hex id, `a0e6`), a long digit
/// run (an account id), a long word or any other character makes it a value. A `.` is kept
/// only between camel-case words (`ForkJoinPool.commonPool-worker-1`): `alice.smith` is a
/// user and is shaped.
fn plain_thread_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 40 {
        return false;
    }
    let inner_upper = |t: &str| t.chars().skip(1).any(|c| c.is_ascii_uppercase());
    if name.contains('.') && !name.split('.').all(inner_upper) {
        return false;
    }
    name.split(['-', '_', '.']).all(|t| {
        t.is_empty()
            || (t.bytes().all(|b| b.is_ascii_alphabetic()) && t.len() <= NAME_WORD_MAX)
            || (t.bytes().all(|b| b.is_ascii_digit()) && t.len() <= NAME_DIGITS_MAX)
    })
}

/// A thread name read from the heap, shaped unless the run has raw-values consent. A name
/// of plain program structure (`main`, `pool-1-thread-3`, see [`plain_thread_name`]) stays;
/// anything else (a request thread named after its URL, a user, an id) becomes a salted
/// shape, the same in every dump of the run, so holder keys still join across dumps.
fn thread_label(name: &str, red: &super::redact::Redactor) -> String {
    if plain_thread_name(name) && red.name(name) == name {
        name.to_owned()
    } else {
        red.value(name)
    }
}

/// Replaces thread names in a summary (holder keys, owners, paths, dominators) by
/// [`thread_label`], before anything is written, keyed or matched.
fn redact_threads(summary: &mut HeapSummary, red: &super::redact::Redactor) {
    let mut renamed: Vec<(String, String)> = Vec::new();
    for h in summary.holders.iter_mut().chain(summary.tracked.iter_mut()) {
        if h.kind != HolderKind::Thread {
            continue;
        }
        let old = h.owner_class.clone();
        let new = thread_label(&old, red);
        if new == old {
            continue;
        }
        h.owner_class = new.clone();
        h.key = format!("thread:{new}");
        for p in &mut h.path {
            *p = p.replace(&old, &new);
        }
        if !renamed.iter().any(|(o, _)| *o == old) {
            renamed.push((old, new));
        }
    }
    for v in &mut summary.via_holders {
        if let Some(name) = v.dominator.strip_prefix("thread:") {
            let new = renamed
                .iter()
                .find(|(o, _)| o == name)
                .map_or_else(|| thread_label(name, red), |(_, n)| n.clone());
            let old = name.to_owned();
            v.dominator = format!("thread:{new}");
            for p in &mut v.path {
                *p = p.replace(&old, &new);
            }
        }
    }
}

fn limitations(d: &Dump) -> Vec<String> {
    let mut v = vec![d.summary.notes.first().cloned().unwrap_or_default()];
    v.extend(d.summary.warnings.iter().take(2).cloned());
    v.retain(|s| !s.is_empty());
    v
}

fn heap_graph(ctx: &Ctx<'_>, inputs: &[Input], a: &Opts) -> Res<AdapterOut> {
    let tc = ctx.tc;
    // ---- parse (no writes yet)
    let tmp = tc.tmp_dir()?;
    let opts = Options {
        tmp_dir: Some(tmp),
        max_objects: a.max_objects,
        ..Options::default()
    };
    // Harness captures of command mode are one process per point: a per-point series.
    let per_point = compat::per_point(&tc.run_dir, inputs).map_err(|why| {
        fail(
            "dumps-not-comparable",
            format!("the dumps are harness captures that do not form one per-point series: {why}"),
            "three or more heap dumps of one command-mode harness run, one per point, at distinct n",
            "dla summarize heap-graph <the heap-*.hprof files of one dla harness run --capture heap, one per point>",
        )
    })?;
    let mut dumps: Vec<Dump> = Vec::new();
    for (k, i) in inputs.iter().enumerate() {
        let mut summary = hprof::summarize(&i.abs, &opts).map_err(|e| parse_failure(&i.rel, &e))?;
        redact_threads(&mut summary, &ctx.redactor);
        add_via_holders(&mut summary);
        let summary_file = format!("summaries/{}", summary_name(&i.rel, &i.sha256));
        dumps.push(Dump {
            rel: i.rel.clone(),
            sha256: i.sha256.clone(),
            summary,
            summary_file,
            x: per_point.as_ref().map(|xs| xs[k]),
        });
        tc.require_time("dla summarize heap-graph")?;
    }
    if per_point.is_some() {
        dumps.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
        for w in dumps.windows(2) {
            compat::check_format(&w[0].summary, &w[1].summary).map_err(|why| {
                fail(
                    "dumps-not-comparable",
                    format!(
                        "{} and {} cannot be compared: {why}",
                        short_path(&w[0].rel),
                        short_path(&w[1].rel)
                    ),
                    "per-point dumps of one JVM build (same HPROF version and identifier size)",
                    "dla summarize heap-graph <the heap dumps of one harness run>",
                )
            })?;
        }
    } else {
        dumps.sort_by(|x, y| {
            x.summary
                .header
                .timestamp_ms
                .cmp(&y.summary.header.timestamp_ms)
                .then_with(|| x.rel.cmp(&y.rel))
        });
    }
    for w in dumps.windows(2).filter(|_| per_point.is_none()) {
        compat::check(&w[0].summary, &w[1].summary).map_err(|why| {
            fail(
                "dumps-not-comparable",
                format!(
                    "{} and {} do not look like dumps of one process: {why}",
                    short_path(&w[0].rel),
                    short_path(&w[1].rel)
                ),
                "dumps of one running process (what can be checked: HPROF version, identifier size, strictly increasing header timestamps, and that the later dump still has 90% of the earlier dump's classes and application classes)",
                "dla summarize heap-graph <the dumps of one process only>",
            )
        })?;
    }
    let mut ledger_ref = ctx.ledger()?;
    let ledger: &mut Ledger = &mut ledger_ref;
    // The finding the discriminator binds to, resolved before anything is written.
    let bound = match &a.finding {
        Some(f) => Some(bound_holder(ledger, f, &dumps)?),
        None => None,
    };

    // ---- per-dump summaries and retention-path evidence
    let dir = tc.stage_dir("summaries")?;
    let mut dump_json = Vec::new();
    let mut retention: Vec<EvidenceId> = Vec::new();
    let mut recorded: Vec<Recorded> = Vec::new();
    let mut last_raw: Option<emit::RawRef> = None;
    for d in &dumps {
        let path = dir.join(d.summary_file.trim_start_matches("summaries/"));
        let body = to_pretty_json(&d.summary);
        write_atomic(&path, body.as_bytes())?;
        let raw = emit::RawRef {
            rel: d.summary_file.clone(),
            sha256: emit::sha_hex(body.as_bytes()),
        };
        let bound_key = bound.as_ref().map(|(_, k)| k.as_str());
        let mut result = json!({
            "file": d.rel,
            "timestamp_ms": d.summary.header.timestamp_ms,
            "objects": d.summary.counts.objects,
            "truncated": d.summary.truncated,
            "holders": evidence_holders(&d.summary, EVIDENCE_HOLDERS, bound_key),
        });
        let via = via_json(&d.summary, EVIDENCE_VIA);
        if !via.is_empty() {
            result["via"] = json!(via);
        }
        // Bounded: drop the smallest of the largest holders, then the via holders, until
        // the record fits; the holder the finding named stays.
        while dla_core::evidence::result_size(&result) > emit::RESULT_BUDGET {
            let keep_bound = bound_key.unwrap_or("");
            let holders = result["holders"].as_array().cloned().unwrap_or_default();
            if let Some(at) = holders
                .iter()
                .rposition(|h| h["key"].as_str() != Some(keep_bound))
                .filter(|_| holders.len() > 1)
            {
                if let Some(a) = result["holders"].as_array_mut() {
                    a.remove(at);
                }
            } else if result.get("via").is_some() {
                if let Some(o) = result.as_object_mut() {
                    o.remove("via");
                }
            } else {
                break;
            }
        }
        let mut spec = EvidenceSpec::new(
            EvidenceKind::RetentionPath,
            "dla-hprof",
            env!("CARGO_PKG_VERSION"),
            result,
        );
        spec.authority = Authority::Dump;
        spec.source = Some(Source::Artifact {
            path: d.rel.clone(),
            sha256: d.sha256.clone(),
        });
        spec.coverage = EvidenceCoverage {
            status: if d.summary.truncated {
                CoverageStatus::Partial
            } else {
                CoverageStatus::Complete
            },
            examined: Some(d.summary.counts.objects),
            total: None,
        };
        spec.limitations = limitations(d);
        let signal = Signal::new("dla-hprof", tool_class::DUMP_PARSER, &d.sha256);
        let rec = ctx.record(ledger, spec, &signal, &raw)?;
        let id = rec.id.clone();
        dump_json.push(json!({
            "file": short_path(&d.rel),
            "timestamp_ms": d.summary.header.timestamp_ms,
            "top_holder": d.summary.holders.first().map(top_json),
            "evidence": id,
            "truncated": d.summary.truncated,
        }));
        retention.push(id);
        recorded.push(rec);
        last_raw = Some(raw);
    }
    let mut out = Map::new();
    out.insert("summaries".into(), json!(tc.pointer("summaries/")));
    out.insert("dumps".into(), json!(dump_json));
    out.insert("redaction".into(), json!(ctx.redactor.mode()));
    let via = dumps
        .last()
        .map(|d| via_json(&d.summary, OUTPUT_VIA))
        .unwrap_or_default();
    if !via.is_empty() {
        out.insert("via_holders".into(), json!(via));
    }
    let n = dumps.len();
    let enumerated = dumps.iter().map(|d| d.summary.counts.objects).sum();
    if n == 1 {
        out.insert("series".into(), Value::Null);
        out.insert("gate".into(), json!("fail"));
        out.insert("reasons".into(), json!(["dumps < 3"]));
        out.insert("needed".into(), json!(needed(n)));
        return Ok(AdapterOut {
            evidence: recorded,
            raw: last_raw,
            body: out,
            enumerated,
            ..AdapterOut::default()
        });
    }
    let mut cands: Vec<CandidateRow> = Vec::new();
    let series_raw = series_part(
        ctx,
        ledger,
        &dumps,
        &retention,
        bound,
        &mut out,
        &mut recorded,
        &mut cands,
    )?;
    Ok(AdapterOut {
        evidence: recorded,
        candidates: cands,
        raw: Some(series_raw),
        body: out,
        enumerated,
        ..AdapterOut::default()
    })
}

fn complete(dumps: u64) -> EvidenceCoverage {
    EvidenceCoverage {
        status: CoverageStatus::Complete,
        examined: Some(dumps),
        total: Some(dumps),
    }
}

fn needed(n: usize) -> String {
    let more = rule::MIN_DUMPS.saturating_sub(n);
    format!(
        "{more} more dump{} of the same process after more traffic ({} confirm growth; two cannot tell a warm-up fill from a leak); rerun with all dumps",
        if more == 1 { "" } else { "s" },
        rule::MIN_DUMPS
    )
}

/// The holder key a `--finding` binds to, after checking the finding exists and names an
/// owner that matches a holder of the latest dump. An exact owner wins over a normalized
/// one, and that over a shorter owner (`Cache.MAP`); a shorter owner binds only when one
/// holder matches it.
fn bound_holder(ledger: &Ledger, f: &FindingId, dumps: &[Dump]) -> Res<(FindingId, String)> {
    let Some(finding) = ledger.finding(f) else {
        return Err(Failure::from(dla_core::Refusal::new(
            core_codes::UNKNOWN_FINDING,
            format!("no finding {f} in the ledger"),
            "an existing finding",
            "dla ledger show",
        )));
    };
    let Some(owner) = finding.owner.clone() else {
        return Err(fail(
            "owner-required",
            format!("{f} has no owner, so no holder can be tied to it"),
            "the finding's owner (class.field)",
            &format!("dla ledger annotate {f} --owner <class.field>"),
        ));
    };
    let last = &dumps[dumps.len() - 1].summary;
    let canon = dla_core::attribution::canonical(&owner);
    let mut exact: Vec<&Holder> = last
        .tracked
        .iter()
        .filter(|h| h.key == owner || owner_of(h) == owner || format!("static:{owner}") == h.key)
        .collect();
    // A static ThreadLocal field is both a static holder (the small ThreadLocal object) and a
    // thread-local holder (its values on live threads): the values are what grows.
    if exact.iter().any(|h| h.kind == HolderKind::ThreadLocal) {
        exact.retain(|h| h.kind == HolderKind::ThreadLocal);
    }
    let normalized: Vec<&Holder> = last
        .tracked
        .iter()
        .filter(|h| {
            h.kind == HolderKind::Static
                && canon.is_some()
                && dla_core::attribution::canonical(&owner_of(h)) == canon
        })
        .collect();
    let suffix: Vec<&Holder> = last
        .tracked
        .iter()
        .filter(|h| {
            h.kind == HolderKind::Static
                && dla_core::attribution::owners_match(&owner_of(h), &owner)
        })
        .collect();
    let tier = [exact, normalized, suffix]
        .into_iter()
        .find(|t| !t.is_empty())
        .unwrap_or_default();
    match tier.as_slice() {
        [h] => Ok((f.clone(), h.key.clone())),
        [] if dumps.len() == 1 => {
            // One dump has no series to bind: the finding is only checked to exist and to
            // have an owner, and the retention-path evidence is still recorded.
            Ok((f.clone(), owner))
        }
        [] => Err(fail(
            "holder-not-found",
            format!(
                "no holder of the latest dump matches {f}'s owner {owner:?}; the largest are {}",
                last.holders
                    .iter()
                    .take(3)
                    .map(|h| h.key.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            "a finding whose owner is a holder in these dumps",
            &format!("dla ledger annotate {f} --owner <class.field of a holder>"),
        )),
        many => Err(fail(
            "ambiguous-holder",
            format!(
                "{f}'s owner {owner:?} matches {} holders of the latest dump: {}",
                many.len(),
                many.iter()
                    .take(4)
                    .map(|h| h.key.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            "a finding whose owner names one holder",
            &format!("dla ledger annotate {f} --owner <fully qualified class.field>"),
        )),
    }
}

/// The finding a passing holder belongs to: by holder identity (the owner), else by the
/// declaration site the resolver finds when that finding names no other owner.
struct Tie {
    finding: FindingId,
    /// The finding's own site, when no other finding has it.
    site: Option<String>,
    why: tie::TieReason,
    /// The holder kind the dumps show for the column.
    holder: FindingHolder,
}

#[allow(clippy::too_many_arguments)]
fn series_part(
    ctx: &Ctx<'_>,
    ledger: &mut Ledger,
    dumps: &[Dump],
    retention: &[EvidenceId],
    bound: Option<(FindingId, String)>,
    out: &mut Map<String, Value>,
    recorded: &mut Vec<Recorded>,
    cands: &mut Vec<CandidateRow>,
) -> Res<emit::RawRef> {
    let tc = ctx.tc;
    let n = dumps.len();
    // ---- holders tracked: the largest by peak retained bytes, plus the bound one
    let mut peak: Vec<(String, u64)> = Vec::new();
    for d in dumps {
        for h in &d.summary.tracked {
            match peak.iter_mut().find(|(k, _)| *k == h.key) {
                Some((_, p)) => *p = (*p).max(h.retained),
                None => peak.push((h.key.clone(), h.retained)),
            }
        }
    }
    peak.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    // Via holders are followed whatever their size: they are the holders a dominator
    // search cannot see, and are often small next to the roots.
    let via_keys: BTreeSet<&str> = dumps
        .iter()
        .flat_map(|d| d.summary.via_holders.iter().map(|v| v.key.as_str()))
        .collect();
    let (mut via_first, rest): (Peaks, Peaks) = peak
        .into_iter()
        .partition(|(k, _)| via_keys.contains(k.as_str()));
    via_first.truncate(SERIES_VIA);
    let mut keys: Vec<String> = via_first
        .into_iter()
        .chain(rest)
        .take(SERIES_HOLDERS)
        .map(|(k, _)| k)
        .collect();
    if let Some((_, k)) = &bound
        && !keys.contains(k)
    {
        keys.pop();
        keys.push(k.clone());
    }
    fn find<'d>(d: &'d Dump, key: &str) -> Option<&'d Holder> {
        d.summary.tracked.iter().find(|h| h.key == key)
    }
    let observed = |key: &str| Observed {
        bytes: dumps
            .iter()
            .map(|d| find(d, key).map(|h| h.retained))
            .collect(),
        counts: dumps
            .iter()
            .map(|d| find(d, key).map(|h| h.dominated_count))
            .collect(),
    };
    // The latest facts about each key (kind, owner, path) come from the latest dump that
    // has it.
    let latest = |key: &str| dumps.iter().rev().find_map(|d| find(d, key));

    // ---- the dump-series rule
    let mut passed: Vec<(String, rule::Growth)> = Vec::new();
    let mut best_failure: Option<(String, Vec<String>, u64)> = None;
    for key in &keys {
        match rule::gate(&observed(key)) {
            Ok(g) => passed.push((key.clone(), g)),
            Err(why) => {
                let size = latest(key).map_or(0, |h| h.retained);
                if best_failure.as_ref().is_none_or(|b| size > b.2) && n >= rule::MIN_DUMPS {
                    best_failure = Some((key.clone(), why, size));
                }
            }
        }
    }
    passed.sort_by_key(|(k, _)| std::cmp::Reverse(latest(k).map_or(0, |h| h.retained)));

    // ---- candidates for the passing holders, tied to an existing finding by owner or
    // by the field's declaration site
    let latest_retention = retention[retention.len() - 1].clone();
    let who = tie::Who::new(tc.worker.as_deref(), Some(dla_core::runtime::Runtime::Jvm));
    let holder_owners: Vec<String> = keys
        .iter()
        .filter_map(|k| latest(k))
        .map(owner_of)
        .collect();
    let mut cand_json = Vec::new();
    let mut tie_of: Vec<(String, Tie)> = Vec::new();
    for (key, _) in &passed {
        let Some(h) = latest(key) else { continue };
        let owner = owner_of(h);
        let resolved = if ctx.artifact_run {
            None
        } else {
            ctx.resolver().resolve(&owner).filter(|r| {
                let file = r.site.rsplit_once(':').map_or(r.site.as_str(), |(f, _)| f);
                tc.in_scope(file)
            })
        };
        let site = resolved.as_ref().map(|r| r.site.clone());
        // The holder `--finding` bound is tied to that finding (its owner matched the
        // holder under the same rules); any other holder ties by its own identity.
        let existing = match &bound {
            Some((f, k)) if k == key => Some((f.clone(), tie::TieReason::holder(&owner))),
            _ => tie::finding_for_holder(ledger, &who, &owner, site.as_deref(), &holder_owners),
        };
        let (fid, fsite, added, why) = match existing {
            Some((f, why)) => {
                let fsite = tie::site_of(ledger, &f);
                (f, fsite, false, why)
            }
            None => {
                let mut nf = NewFinding::new(
                    Origin::parse("growth-series").map_err(dla_core::Refusal::from)?,
                    vec![latest_retention.clone()],
                );
                nf.holder = Some(observed_holder(h));
                nf.owner = Some(owner.clone());
                nf.region = Some(Region::Heap);
                nf.site = site.clone();
                let line = ledger.add_finding(nf).map_err(|e| e.to_refusal())?;
                let dla_core::Record::FindingAdded { finding } = line.record else {
                    continue;
                };
                if let Some(r) = &resolved
                    && !r.writers.is_empty()
                {
                    let _ =
                        ledger.annotate(&finding.id, r.writers.clone(), r.removers.clone(), None);
                }
                (
                    finding.id.clone(),
                    tie::site_of(ledger, &finding.id),
                    true,
                    tie::TieReason::holder(&owner),
                )
            }
        };
        if added {
            let mut c = json!({"finding": fid, "owner": owner, "holder": observed_holder(h)});
            if let Some(s) = &site {
                c["site"] = json!(s);
                c["via"] = json!(resolved.as_ref().map_or("", |r| r.via.as_str()));
            }
            cand_json.push(c);
        }
        cands.push(CandidateRow {
            finding: fid.clone(),
            owner: Some(owner),
            holder: Some(observed_holder(h)),
            site: fsite.clone(),
            region: Some(Region::Heap),
            origin: "growth-series".into(),
            added,
            via: resolved.as_ref().map(|r| r.via.as_str()),
        });
        tie_of.push((
            key.clone(),
            Tie {
                finding: fid,
                site: fsite,
                why,
                holder: observed_holder(h),
            },
        ));
    }
    let tie_for = |key: &str| tie_of.iter().find(|(k, _)| k == key).map(|(_, t)| t);

    // ---- the series file
    let id = tc.next_series_id();
    let t0 = dumps[0].summary.header.timestamp_ms;
    let per_point: Option<Vec<f64>> = dumps.iter().map(|d| d.x).collect();
    let mut columns = Vec::new();
    for key in &keys {
        let tie = tie_for(key);
        for (suffix, unit, kind) in [
            ("", Unit::Bytes, ColumnKind::Gauge),
            (".count", Unit::Count, ColumnKind::Counter),
        ] {
            columns.push(Column {
                name: format!("holder:{key}{suffix}"),
                unit,
                kind,
                region: SeriesRegion::Heap,
                baseline: false,
                after_gc: false,
                scope: ColumnScope::Process,
                site: tie.and_then(|t| t.site.clone()),
                finding: tie.map(|t| t.finding.clone()),
                source: "hprof".into(),
            });
        }
    }
    let points: Vec<Point> = dumps
        .iter()
        .enumerate()
        .map(|(i, d)| Point {
            x: d.x.unwrap_or((i + 1) as f64),
            // Separate processes share no clock: the header timestamps order nothing.
            t_ms: if d.x.is_some() {
                0
            } else {
                d.summary.header.timestamp_ms.saturating_sub(t0)
            },
            values: keys
                .iter()
                .flat_map(|k| {
                    let h = find(d, k);
                    [
                        h.map(|h| h.retained as f64),
                        h.map(|h| h.dominated_count as f64),
                    ]
                })
                .collect(),
        })
        .collect();
    let mut source = SeriesSource::new(SeriesSourceKind::Dump);
    source.command = Some("dla summarize heap-graph".into());
    let series = SeriesFile {
        schema_version: 1,
        id: id.clone(),
        source,
        axis: if per_point.is_some() {
            Axis::N
        } else {
            Axis::Dump
        },
        warmup: 0,
        columns,
        points,
        events: Vec::new(),
        limitations: match &per_point {
            Some(xs) => vec![
                format!("series: per-point; x is the iteration count n of each point ({}); every point is a separate process and heap dump", compat::n_list(xs)),
                "retained bytes are approximate (see the dump summaries); t_ms is not recorded".into(),
                "holders absent from a dump's 200 largest have no figure at that point".into(),
            ],
            None => vec![
                "x is the dump number in header-timestamp order; t_ms is the header timestamp difference from the first dump".into(),
                "each point comes from a separate heap dump of one process; retained bytes are approximate (see the dump summaries)".into(),
                "holders absent from a dump's 200 largest have no figure at that point".into(),
            ],
        },
    };
    series.save(&tc.run_dir)?;

    // ---- the growth-series evidence
    let gate_pass = !passed.is_empty();
    let mut reasons: Vec<String> = Vec::new();
    if n < rule::MIN_DUMPS {
        reasons.push(format!("dumps < {}", rule::MIN_DUMPS));
    } else if !gate_pass {
        reasons.push("no holder grows strictly in bytes and count across all dumps".into());
        if let Some((k, why, _)) = &best_failure {
            reasons.push(format!("largest: {k}: {}", why.join("; ")));
        }
    }
    // Which passing holders the bounded result shows: the bound one first, then the best
    // holder of each other tied finding, then the rest by size. What the output names as
    // tied is only what the result shows.
    let mut order: Vec<usize> = Vec::new();
    let mut covered: Vec<&FindingId> = Vec::new();
    if let Some((f, bk)) = &bound
        && let Some(i) = passed.iter().position(|(k, _)| k == bk)
    {
        order.push(i);
        covered.push(f);
    }
    for (i, (k, _)) in passed.iter().enumerate() {
        if let Some(t) = tie_for(k)
            && !covered.contains(&&t.finding)
        {
            covered.push(&t.finding);
            order.push(i);
        }
    }
    for i in 0..passed.len() {
        if !order.contains(&i) {
            order.push(i);
        }
    }
    let rows: Vec<Value> = order
        .iter()
        .take(EVIDENCE_PASSED)
        .map(|&i| {
            let (key, g) = &passed[i];
            let h = latest(key);
            let slope = g.byte_deltas.iter().sum::<u64>() as f64 / g.byte_deltas.len() as f64;
            let mut v = json!({
                "column": format!("holder:{key}"),
                "region": "heap",
                "unit": "bytes",
                "slope": slope.round() as u64,
                "shape": rule::shape(g),
                "owner": h.map(|h| clip(&owner_of(h), 120)),
                "last_over_first": (g.last_over_first * 100.0).round() / 100.0,
            });
            // The holder kind observed for the column: the ledger reads it to accept a
            // corrected holder.
            if let Some(h) = h {
                v["holder_kind"] = json!(observed_holder(h));
            }
            if let Some(t) = tie_for(key) {
                v["finding"] = json!(t.finding);
                v["candidates"] = json!([t.finding]);
                v["tie"] = t.why.to_json();
                if let Some(s) = &t.site {
                    v["site"] = json!(clip(s, 160));
                }
            }
            v
        })
        .collect();
    let mut base = json!({
        "series": id,
        "axis": if per_point.is_some() { "n" } else { "dump" },
        "n": n,
        "gate": if gate_pass { "pass" } else { "fail" },
        "failed": keys.len() - passed.len(),
        "reasons": reasons,
    });
    if let Some(xs) = &per_point {
        base["series_kind"] = json!("per-point");
        base["n_values"] = json!(xs);
    }
    let (result, kept) = growth::fit_passed(base, rows, passed.len());
    let shown: Vec<usize> = order.iter().take(kept).copied().collect();
    let series_rel = format!("series/{id}.json");
    let mut spec = EvidenceSpec::new(
        EvidenceKind::GrowthSeries,
        "dla-hprof",
        env!("CARGO_PKG_VERSION"),
        result,
    );
    spec.authority = Authority::Dump;
    let source = tc.artifact_source(&series_rel)?;
    let series_sha = match &source {
        Source::Artifact { sha256, .. } => sha256.clone(),
        _ => String::new(),
    };
    spec.source = Some(source);
    spec.coverage = complete(n as u64);
    spec.limitations = vec![
        format!(
            "dump-series rule: at least {} dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least {:.0}% of the first (provisional, not calibrated)",
            rule::MIN_DUMPS,
            rule::LAST_OVER_FIRST * 100.0
        ),
        match &per_point {
            Some(xs) => format!("series: per-point; one process per point at n = {} (harness captures; checked: HPROF version, identifier size)", compat::n_list(xs)),
            None => "the dumps are assumed to come from one process (checked: HPROF version, identifier size, timestamps, class set)".into(),
        },
    ];
    let dump_shas: Vec<&str> = dumps.iter().map(|d| d.sha256.as_str()).collect();
    let signal = Signal::new(
        "dla-hprof",
        tool_class::DUMP_PARSER,
        &emit::combined_sha(&dump_shas),
    );
    let series_raw = emit::RawRef {
        rel: series_rel.clone(),
        sha256: series_sha,
    };
    let growth = ctx.record(ledger, spec, &signal, &series_raw)?;
    let growth_id = growth.id.clone();
    recorded.push(growth);

    // ---- discriminators
    let mut interest: Vec<String> = Vec::new();
    if let Some((_, k)) = &bound {
        interest.push(k.clone());
    } else {
        interest.extend(passed.iter().map(|(k, _)| k.clone()));
        // Other holders that changed and are not negligible next to the largest.
        let biggest = keys
            .iter()
            .filter_map(|k| latest(k))
            .map(|h| h.retained)
            .max()
            .unwrap_or(0);
        for key in &keys {
            let o = observed(key);
            let moved = o.bytes.iter().flatten().collect::<BTreeSet<_>>().len() > 1;
            let sizable = latest(key).is_some_and(|h| h.retained * 10 >= biggest);
            if moved && sizable && !interest.contains(key) {
                interest.push(key.clone());
            }
        }
    }
    interest.truncate(MAX_DISCRIMINATORS);
    let mut disc_json = Vec::new();
    for key in &interest {
        let Some(p) = rule::plateau(&observed(key)) else {
            continue;
        };
        let finding: Option<String> = match &bound {
            Some((f, k)) if k == key => Some(f.to_string()),
            _ => tie_for(key).map(|t| t.finding.to_string()).or_else(|| {
                latest(key).and_then(|h| {
                    let owner = owner_of(h);
                    ledger
                        .findings()
                        .find(|f| f.owner.as_deref() == Some(owner.as_str()))
                        .map(|f| f.id.to_string())
                })
            }),
        };
        let mut r = DiscriminatorResult::new("dump-plateau", p.favors);
        r.supports = p.supports.iter().map(|s| (*s).to_owned()).collect();
        r.finding = finding.as_deref().and_then(|f| f.parse().ok());
        r.extra
            .insert("column".into(), json!(format!("holder:{key}")));
        r.extra.insert("series".into(), json!(id));
        r.extra.insert("dumps".into(), json!(n));
        if let Value::Object(m) = p.extra() {
            r.extra.extend(m);
        }
        let mut spec = EvidenceSpec::new(
            EvidenceKind::Discriminator,
            "dla-hprof",
            env!("CARGO_PKG_VERSION"),
            r.to_value(),
        );
        spec.authority = Authority::Dump;
        spec.source = Some(tc.artifact_source(&series_rel)?);
        spec.coverage = complete(n as u64);
        spec.limitations = vec![
            "dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)".into(),
        ];
        let rec = ctx.record(ledger, spec, &signal, &series_raw)?;
        disc_json.push(
            json!({"holder": key, "evidence": rec.id, "favors": p.favors, "finding": finding}),
        );
        recorded.push(rec);
    }

    out.insert("series".into(), json!(id));
    if let Some(xs) = &per_point {
        out.insert("series_kind".into(), json!("per-point"));
        out.insert("n_values".into(), json!(xs));
    }
    out.insert(
        "gate".into(),
        json!(if gate_pass { "pass" } else { "fail" }),
    );
    out.insert("reasons".into(), json!(reasons));
    out.insert("growth_evidence".into(), json!(growth_id));
    out.insert(
        "passed".into(),
        json!(shown.iter().map(|&i| &passed[i].0).collect::<Vec<_>>()),
    );
    let mut tied_ids: Vec<&FindingId> = Vec::new();
    for &i in &shown {
        if let Some(t) = tie_for(&passed[i].0)
            && !tied_ids.contains(&&t.finding)
        {
            tied_ids.push(&t.finding);
        }
    }
    out.insert("tied".into(), json!(tied_ids));
    out.insert("candidates".into(), json!(cand_json));
    out.insert("discriminators".into(), json!(disc_json));
    let needed_text = if n < rule::MIN_DUMPS {
        needed(n)
    } else if !gate_pass {
        "growth over every interval: take another dump after more traffic, rerun with all dumps"
            .to_owned()
    } else {
        match tied_ids.first() {
            Some(f) => {
                let seen = shown
                    .iter()
                    .filter_map(|&i| tie_for(&passed[i].0))
                    .find(|t| &t.finding == *f)
                    .map(|t| t.holder);
                match (seen, ledger.finding(f).and_then(|x| x.holder)) {
                    (Some(h), Some(named)) if h != named => format!(
                        "the dumps show holder {h} for {f}, which has holder {named}: dla ledger annotate {f} --holder {h} --evidence {growth_id}; then a refuted non-leak sibling, then dla ledger confirm {f} --evidence {growth_id}"
                    ),
                    _ => format!(
                        "a refuted non-leak sibling and a holder, then dla ledger confirm {f} --evidence {growth_id}"
                    ),
                }
            }
            None => "no passing holder is tied in the evidence: rerun with fewer holders or --finding <F>".to_owned(),
        }
    };
    out.insert("needed".into(), json!(needed_text));
    Ok(series_raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn holder(key: &str, kind: HolderKind, owner: &str, field: Option<&str>) -> Holder {
        Holder {
            key: key.into(),
            kind,
            owner_class: owner.into(),
            field: field.map(str::to_owned),
            raw_owner_class: None,
            retained: 0,
            dominated_count: 0,
            path: Vec::new(),
            accumulation: None,
            context: Vec::new(),
        }
    }

    #[test]
    fn the_owner_of_a_via_instance_field_is_class_dot_field() {
        let st = holder(
            "static:shop.Waiters.gates",
            HolderKind::Static,
            "shop.Waiters",
            Some("gates"),
        );
        assert_eq!(owner_of(&st), "shop.Waiters.gates");
        let inst = holder(
            "field:shop.Registry.table",
            HolderKind::Instance,
            "shop.Registry",
            Some("table"),
        );
        assert_eq!(owner_of(&inst), "shop.Registry.table");
        // A root instance holder has no field: its owner is its key.
        let root = holder("instance:x>y", HolderKind::Instance, "y", None);
        assert_eq!(owner_of(&root), "instance:x>y");
        assert_eq!(finding_holder(&inst), FindingHolder::ContainerField);
    }

    #[test]
    fn thread_local_values_are_owned_by_the_static_field_and_report_thread_local() {
        let h = holder(
            "thread-local:shop.Ctx.BUF",
            HolderKind::ThreadLocal,
            "shop.Ctx",
            Some("BUF"),
        );
        // The owner is the static field, so it ties to a census finding with that owner.
        assert_eq!(owner_of(&h), "shop.Ctx.BUF");
        assert_eq!(finding_holder(&h), FindingHolder::ThreadLocal);
        assert_eq!(observed_holder(&h), FindingHolder::ThreadLocal);
    }

    #[test]
    fn an_instance_field_under_a_static_root_is_static_global() {
        let mut inst = holder(
            "field:shop.Hub.data",
            HolderKind::Instance,
            "shop.Hub",
            Some("data"),
        );
        // No path: the kind of the field itself.
        assert_eq!(observed_holder(&inst), FindingHolder::ContainerField);
        for head in ["root class shop.Waiters", "root shop.Waiters"] {
            inst.path = vec![head.into(), "static gates -> shop.Registry".into()];
            assert_eq!(
                observed_holder(&inst),
                FindingHolder::StaticGlobal,
                "{head}"
            );
        }
        // Roots that are not statics keep the field's kind.
        for head in [
            "root instance shop.Hub",
            "root thread:main",
            "root jni-global:x.Y",
            "root instance:vm-internal>shop.Hub",
        ] {
            inst.path = vec![head.into()];
            assert_eq!(
                observed_holder(&inst),
                FindingHolder::ContainerField,
                "{head}"
            );
        }
        // A static holder stays static, a thread stays a thread kind.
        let st = holder("static:a.B.c", HolderKind::Static, "a.B", Some("c"));
        assert_eq!(observed_holder(&st), FindingHolder::StaticGlobal);
        let th = holder("thread:main", HolderKind::Thread, "main", None);
        assert_eq!(observed_holder(&th), FindingHolder::BlockedStack);
    }

    #[test]
    fn ids_and_users_in_thread_names_are_shaped_and_program_names_stay() {
        let r = super::super::redact::Redactor::new(b"salt", false);
        for keep in [
            "main",
            "pool-1-thread-3",
            "http-nio-8080-exec-12",
            "ForkJoinPool.commonPool-worker-1",
            "Reference_Handler",
        ] {
            assert_eq!(thread_label(keep, &r), keep, "{keep}");
        }
        for shape in [
            "session-8f14e45f-ceea-467f-a0e6",
            "user-12345678",
            "alice.smith",
            "worker@host",
            "acct_9f8e7d6c",
        ] {
            let l = thread_label(shape, &r);
            assert!(l.starts_with('<') && l != shape, "{shape} -> {l}");
            // The same name gives the same label in every dump.
            assert_eq!(l, thread_label(shape, &r));
        }
    }
}
