//! `dla summarize heap-graph <file>...`: heap dumps into retention and growth evidence.
//!
//! Each dump is summarized once (`summaries/`) and recorded as one `retention-path`
//! evidence. Two or more dumps, ordered by their header timestamps, also form a dump
//! series (`series/S<n>.json`): one `growth-series` evidence that applies the dump-series
//! rule, one `dump-plateau` discriminator per holder of interest, and a candidate finding
//! (owner, no site) for each holder that passes.

use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use clap::{Args, Subcommand};
use dla_core::{
    Error, FindingId, Ledger, NewFinding, Origin,
    discriminator::DiscriminatorResult,
    error::codes as core_codes,
    evidence::{Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Source},
    finding::{Holder as FindingHolder, Region},
    fsutil::{to_pretty_json, write_atomic},
    ids::EvidenceId,
    paths::to_slash,
    series::{
        Axis, Column, ColumnKind, ColumnScope, Point, SeriesFile, SeriesRegion, SeriesSource,
        SeriesSourceKind, Unit,
    },
    toolctx::{EvidenceSpec, ToolCtx},
};
use dla_parsers::hprof::{
    self, DEFAULT_MAX_OBJECTS, HeapSummary, Holder, HolderKind, Options, Sniff,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    parse::parse_enum,
    stage,
};

#[path = "summarize/compat.rs"]
mod compat;
#[path = "summarize/rule.rs"]
mod rule;

use rule::Observed;

/// Most holders a dump series tracks (two columns each; a series file holds 64).
const SERIES_HOLDERS: usize = 32;
/// Most `dump-plateau` discriminators one run writes.
const MAX_DISCRIMINATORS: usize = 5;
/// Holders named in a retention-path evidence.
const EVIDENCE_HOLDERS: usize = 3;
/// Passing holders named in a growth-series evidence.
const EVIDENCE_PASSED: usize = 4;

/// `dla summarize ...`
#[derive(Subcommand, Debug)]
pub enum SummarizeCmd {
    /// Summarize HPROF heap dumps (`.hprof`, `.hprof.gz`); two or more form a dump series.
    HeapGraph(HeapGraphArgs),
}

/// Flags of `dla summarize heap-graph`.
#[derive(Args, Debug)]
pub struct HeapGraphArgs {
    /// Dump files of one process, relative to the project root or absolute; two or more
    /// are ordered by their header timestamps, not by name.
    #[arg(required = true)]
    pub files: Vec<PathBuf>,
    /// Bind the plateau discriminator to this finding: the holder is found through the
    /// finding's owner (`class.field`).
    #[arg(long, value_parser = parse_enum::<FindingId>)]
    pub finding: Option<FindingId>,
    /// Analyze at most this many objects per dump; above it the summary is truncated.
    #[arg(long, default_value_t = DEFAULT_MAX_OBJECTS)]
    pub max_objects: usize,
}

/// Runs one `dla summarize ...` subcommand.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: SummarizeCmd) -> Res<Value> {
    match a {
        SummarizeCmd::HeapGraph(h) => heap_graph(&stage::tool_ctx(project, ctx, data), h),
    }
}

/// One parsed dump.
struct Dump {
    /// Project-relative (or absolute) path with `/` separators.
    rel: String,
    sha256: String,
    summary: HeapSummary,
    /// `summaries/<name>.json` relative to the run directory.
    summary_file: String,
}

const SUPPORTED: &str =
    "supported: .hprof and .hprof.gz (other dump formats arrive in a later milestone)";

fn fail(code: &str, message: String, missing: &str, fix: &str) -> Failure {
    Failure::refuse(code, message, missing, fix)
}

fn resolve(tc: &ToolCtx, given: &Path) -> Res<(String, PathBuf)> {
    let abs = if given.is_absolute() {
        given.to_path_buf()
    } else {
        tc.project.join(given)
    };
    let shown = to_slash(given);
    if !abs.is_file() {
        return Err(fail(
            "artifact-not-found",
            format!("{shown} is not a file"),
            "an existing heap dump file",
            "dla summarize heap-graph <project-relative or absolute path to a .hprof or .hprof.gz>",
        ));
    }
    let lower = abs
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let named = lower.ends_with(".hprof") || lower.ends_with(".hprof.gz");
    let sniffed = hprof::sniff(&abs).map_err(|e| Error::io(&abs, e))?;
    if !named && sniffed == Sniff::Other {
        return Err(fail(
            "unsupported-format",
            format!("{shown} is not an HPROF heap dump ({SUPPORTED})"),
            "a .hprof or .hprof.gz file",
            "dla summarize heap-graph <dump>.hprof",
        ));
    }
    let rel = match abs.strip_prefix(&tc.project) {
        Ok(r) => {
            let r = to_slash(r);
            if !tc.in_scope(&r) {
                return Err(fail(
                    "out-of-scope",
                    format!("{r} is outside the run's scope"),
                    "a dump inside the run's scope",
                    "dla run init   (with a scope that includes the dump)",
                ));
            }
            r
        }
        Err(_) => to_slash(&abs),
    };
    Ok((rel, abs))
}

fn sha256_of(path: &Path) -> Result<String, Error> {
    let mut f = fs::File::open(path).map_err(|e| Error::io(path, e))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf).map_err(|e| Error::io(path, e))?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}

fn summary_name(rel: &str, sha: &str) -> String {
    let file = rel.rsplit('/').next().unwrap_or(rel);
    let stem = file
        .strip_suffix(".gz")
        .unwrap_or(file)
        .strip_suffix(".hprof")
        .unwrap_or(file.strip_suffix(".gz").unwrap_or(file));
    let clean: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("{clean}-{}.json", &sha[..8])
}

fn parse_failure(rel: &str, e: &hprof::HprofError) -> Failure {
    use hprof::HprofError as H;
    match e {
        H::NotHprof(m) => fail(
            "unsupported-format",
            format!("{rel} is not an HPROF heap dump: {m} ({SUPPORTED})"),
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

fn top_json(h: &Holder) -> Value {
    json!({"key": h.key, "retained": h.retained, "dominated": h.dominated_count})
}

fn finding_holder(h: &Holder) -> FindingHolder {
    match h.kind {
        HolderKind::Static => FindingHolder::StaticGlobal,
        HolderKind::JniGlobal => FindingHolder::NativeHandleGlobalRef,
        HolderKind::Instance => FindingHolder::ContainerField,
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

/// `class.field` for a static, else the key.
fn owner_of(h: &Holder) -> String {
    match (h.kind, &h.field) {
        (HolderKind::Static, Some(f)) => format!("{}.{f}", h.owner_class),
        _ => h.key.clone(),
    }
}

fn limitations(d: &Dump) -> Vec<String> {
    let mut v = vec![d.summary.notes.first().cloned().unwrap_or_default()];
    v.extend(d.summary.warnings.iter().take(2).cloned());
    v.retain(|s| !s.is_empty());
    v
}

fn heap_graph(tc: &ToolCtx, a: HeapGraphArgs) -> Res<Value> {
    tc.require_time("dla summarize heap-graph")?;
    // ---- inputs
    let mut inputs: Vec<(String, PathBuf)> = Vec::new();
    for f in &a.files {
        let (rel, abs) = resolve(tc, f)?;
        let canon = fs::canonicalize(&abs).unwrap_or_else(|_| abs.clone());
        if inputs
            .iter()
            .any(|(_, p)| fs::canonicalize(p).unwrap_or_else(|_| p.clone()) == canon)
        {
            return Err(fail(
                "duplicate-artifact",
                format!("{rel} is given twice"),
                "each dump once",
                "dla summarize heap-graph <dump1> <dump2> <dump3>",
            ));
        }
        inputs.push((rel, abs));
    }
    // ---- parse (no writes yet)
    let tmp = tc.tmp_dir()?;
    let opts = Options {
        tmp_dir: Some(tmp),
        max_objects: a.max_objects,
        ..Options::default()
    };
    let mut dumps: Vec<Dump> = Vec::new();
    for (rel, abs) in inputs {
        let sha256 = sha256_of(&abs)?;
        let summary = hprof::summarize(&abs, &opts).map_err(|e| parse_failure(&rel, &e))?;
        let summary_file = format!("summaries/{}", summary_name(&rel, &sha256));
        dumps.push(Dump {
            rel,
            sha256,
            summary,
            summary_file,
        });
        tc.require_time("dla summarize heap-graph")?;
    }
    dumps.sort_by(|x, y| {
        x.summary
            .header
            .timestamp_ms
            .cmp(&y.summary.header.timestamp_ms)
            .then_with(|| x.rel.cmp(&y.rel))
    });
    for w in dumps.windows(2) {
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
    let mut ledger = tc.ledger()?;
    // The finding the discriminator binds to, resolved before anything is written.
    let bound = match &a.finding {
        Some(f) => Some(bound_holder(&ledger, f, &dumps)?),
        None => None,
    };

    // ---- per-dump summaries and retention-path evidence
    let dir = tc.stage_dir("summaries")?;
    let mut dump_json = Vec::new();
    let mut retention: Vec<EvidenceId> = Vec::new();
    for d in &dumps {
        let path = dir.join(d.summary_file.trim_start_matches("summaries/"));
        write_atomic(&path, to_pretty_json(&d.summary).as_bytes())?;
        let top: Vec<Value> = d
            .summary
            .holders
            .iter()
            .take(EVIDENCE_HOLDERS)
            .map(top_json)
            .collect();
        let mut spec = EvidenceSpec::new(
            EvidenceKind::RetentionPath,
            "dla-hprof",
            env!("CARGO_PKG_VERSION"),
            json!({
                "file": d.rel,
                "timestamp_ms": d.summary.header.timestamp_ms,
                "objects": d.summary.counts.objects,
                "truncated": d.summary.truncated,
                "holders": top,
            }),
        );
        spec.authority = Authority::Dump;
        spec.raw = Some(d.summary_file.clone());
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
        let id = tc.add_evidence(&mut ledger, spec)?;
        dump_json.push(json!({
            "file": short_path(&d.rel),
            "timestamp_ms": d.summary.header.timestamp_ms,
            "top_holder": d.summary.holders.first().map(top_json),
            "evidence": id,
            "truncated": d.summary.truncated,
        }));
        retention.push(id);
    }
    let mut out = json!({
        "ok": true,
        "command": "summarize heap-graph",
        "summaries": tc.pointer("summaries/"),
        "dumps": dump_json,
    });
    let n = dumps.len();
    if n == 1 {
        out["series"] = Value::Null;
        out["gate"] = json!("fail");
        out["reasons"] = json!(["dumps < 3"]);
        out["needed"] = json!(needed(n));
        return Ok(out);
    }
    series_part(tc, &mut ledger, &dumps, &retention, bound, &mut out)?;
    Ok(out)
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
/// owner that matches a holder of the latest dump.
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
    let hit = last
        .tracked
        .iter()
        .find(|h| h.key == owner || owner_of(h) == owner || format!("static:{owner}") == h.key);
    match hit {
        Some(h) => Ok((f.clone(), h.key.clone())),
        // One dump has no series to bind: the finding is only checked to exist and to
        // have an owner, and the retention-path evidence is still recorded.
        None if dumps.len() == 1 => Ok((f.clone(), owner)),
        None => Err(fail(
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
    }
}

fn series_part(
    tc: &ToolCtx,
    ledger: &mut Ledger,
    dumps: &[Dump],
    retention: &[EvidenceId],
    bound: Option<(FindingId, String)>,
    out: &mut Value,
) -> Res<()> {
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
    let mut keys: Vec<String> = peak
        .into_iter()
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

    // ---- the series file
    let id = tc.next_series_id();
    let t0 = dumps[0].summary.header.timestamp_ms;
    let mut columns = Vec::new();
    for key in &keys {
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
                site: None,
                finding: None,
                source: "hprof".into(),
            });
        }
    }
    let points: Vec<Point> = dumps
        .iter()
        .enumerate()
        .map(|(i, d)| Point {
            x: (i + 1) as f64,
            t_ms: d.summary.header.timestamp_ms.saturating_sub(t0),
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
        axis: Axis::Dump,
        warmup: 0,
        columns,
        points,
        events: Vec::new(),
        limitations: vec![
            "x is the dump number in header-timestamp order; t_ms is the header timestamp difference from the first dump".into(),
            "each point comes from a separate heap dump of one process; retained bytes are approximate (see the dump summaries)".into(),
            "holders absent from a dump's 200 largest have no figure at that point".into(),
        ],
    };
    series.save(&tc.run_dir)?;

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

    // ---- candidates for the passing holders, tied to an existing finding by owner
    let latest_retention = retention[retention.len() - 1].clone();
    let mut cand_json = Vec::new();
    let mut tie: Vec<(String, Option<String>)> = Vec::new();
    for (key, _) in &passed {
        let Some(h) = latest(key) else { continue };
        let owner = owner_of(h);
        let existing = ledger
            .findings()
            .find(|f| f.owner.as_deref() == Some(owner.as_str()))
            .map(|f| f.id.clone());
        let fid = match existing {
            Some(f) => f,
            None => {
                let mut nf = NewFinding::new(
                    Origin::parse("growth-series").map_err(dla_core::Refusal::from)?,
                    vec![latest_retention.clone()],
                );
                nf.holder = Some(finding_holder(h));
                nf.owner = Some(owner.clone());
                nf.region = Some(Region::Heap);
                let line = ledger.add_finding(nf).map_err(|e| e.to_refusal())?;
                let dla_core::Record::FindingAdded { finding } = line.record else {
                    continue;
                };
                cand_json
                    .push(json!({"finding": finding.id, "owner": owner, "holder": finding.holder}));
                finding.id
            }
        };
        tie.push((key.clone(), Some(fid.to_string())));
    }
    let finding_of = |key: &str| -> Option<String> {
        tie.iter()
            .find(|(k, _)| k == key)
            .and_then(|(_, f)| f.clone())
    };

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
    let passed_json: Vec<Value> = passed
        .iter()
        .take(EVIDENCE_PASSED)
        .map(|(key, g)| {
            let h = latest(key);
            let slope = g.byte_deltas.iter().sum::<u64>() as f64 / g.byte_deltas.len() as f64;
            let mut v = json!({
                "column": format!("holder:{key}"),
                "region": "heap",
                "unit": "bytes",
                "slope": slope.round() as u64,
                "shape": rule::shape(g),
                "owner": h.map(owner_of),
                "last_over_first": (g.last_over_first * 100.0).round() / 100.0,
            });
            if let Some(f) = finding_of(key) {
                v["finding"] = json!(f);
                v["candidates"] = json!([f]);
            }
            v
        })
        .collect();
    let mut result = json!({
        "series": id,
        "axis": "dump",
        "n": n,
        "gate": if gate_pass { "pass" } else { "fail" },
        "passed": passed_json,
        "failed": keys.len() - passed.len(),
        "reasons": reasons,
    });
    if passed.len() > EVIDENCE_PASSED {
        result["truncated"] = json!(passed.len() - EVIDENCE_PASSED);
    }
    let series_rel = format!("series/{id}.json");
    let mut spec = EvidenceSpec::new(
        EvidenceKind::GrowthSeries,
        "dla-hprof",
        env!("CARGO_PKG_VERSION"),
        result,
    );
    spec.authority = Authority::Dump;
    spec.raw = Some(series_rel.clone());
    spec.source = Some(tc.artifact_source(&series_rel)?);
    spec.coverage = complete(n as u64);
    spec.limitations = vec![
        format!(
            "dump-series rule: at least {} dumps, retained bytes and dominated count strictly increasing across every pair, last interval at least {:.0}% of the first (provisional, not calibrated)",
            rule::MIN_DUMPS,
            rule::LAST_OVER_FIRST * 100.0
        ),
        "the dumps are assumed to come from one process (checked: HPROF version, identifier size, timestamps, class set)".into(),
    ];
    let growth = tc.add_evidence(ledger, spec)?;

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
        let finding = match &bound {
            Some((f, k)) if k == key => Some(f.to_string()),
            _ => finding_of(key).or_else(|| {
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
        spec.raw = Some(series_rel.clone());
        spec.source = Some(tc.artifact_source(&series_rel)?);
        spec.coverage = complete(n as u64);
        spec.limitations = vec![
            "dump-plateau thresholds: growth over every interval favors a leak; a last interval of at most 5% of the first (or none) favors a non-leak (provisional, not calibrated)".into(),
        ];
        let ev = tc.add_evidence(ledger, spec)?;
        disc_json
            .push(json!({"holder": key, "evidence": ev, "favors": p.favors, "finding": finding}));
    }

    out["series"] = json!(id);
    out["gate"] = json!(if gate_pass { "pass" } else { "fail" });
    out["reasons"] = json!(reasons);
    out["growth_evidence"] = json!(growth);
    out["passed"] = json!(
        passed
            .iter()
            .take(EVIDENCE_PASSED)
            .map(|(k, _)| k)
            .collect::<Vec<_>>()
    );
    out["candidates"] = json!(cand_json);
    out["discriminators"] = json!(disc_json);
    if n < rule::MIN_DUMPS {
        out["needed"] = json!(needed(n));
    } else if !gate_pass {
        out["needed"] = json!(
            "growth over every interval: take another dump after more traffic, rerun with all dumps"
        );
    } else {
        out["needed"] = json!(format!(
            "a refuted non-leak sibling and a holder, then dla ledger confirm <finding> --evidence {growth}"
        ));
    }
    Ok(())
}
