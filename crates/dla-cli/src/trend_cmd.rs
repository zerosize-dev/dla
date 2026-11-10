//! `dla trend <S|path> [--column <name>]... [--against <S2>] [--finding <F>]`.
//!
//! Reads a measurement series, takes the post-warm-up points of every column (or the
//! named ones), runs the `dla-stats` analysis (every-delta gate for counters, slope gate
//! for gauges), ties the columns to findings by site, writes `trend/T<n>.json` and adds
//! one `growth-series` and/or one `unit-count-series` evidence record. With `--against`
//! it also compares the columns with a repeated-keys series and adds a `keys`
//! discriminator.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use dla_core::{
    EvidenceId, FindingId, FindingState, Ledger, MAX_RESULT_BYTES, RunState,
    discriminator::{DiscriminatorResult, Favors},
    error::codes,
    evidence::{
        Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal, Source, tool_class,
    },
    finding::Polarity,
    fsutil::{to_pretty_json, write_atomic},
    paths::to_slash,
    series::{ColumnKind, KeyMode, SeriesFile, numbered},
    toolctx::{EvidenceSpec, ToolCtx},
    trend::{ColumnTrend, TREND_DIR, TrendReport, finding_region, finding_shape},
};
use dla_stats::GateThresholds;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    parse::parse_enum,
    stage,
};

#[path = "trend/against.rs"]
mod against;
#[path = "trend/analysis.rs"]
pub(crate) mod analysis;
#[path = "trend/candidates.rs"]
mod candidates;
#[path = "trend/summary.rs"]
mod summary;

/// Flags of `dla trend`.
#[derive(Args, Debug)]
pub struct TrendArgs {
    /// The series: an ID in this run's series/ directory (S3) or a path to a series file.
    series: String,
    /// Analyse only this column (repeatable); default: every column.
    #[arg(long = "column", value_name = "NAME")]
    columns: Vec<String>,
    /// A repeated-keys series (S4) to compare with this fresh-keys series: writes one
    /// `keys` discriminator.
    #[arg(long, value_name = "S2")]
    against: Option<String>,
    /// With --against: the finding the discriminator is bound to.
    #[arg(long, value_parser = parse_enum::<FindingId>, requires = "against")]
    finding: Option<FindingId>,
}

/// A series read from disk, with where it came from.
struct Loaded {
    series: SeriesFile,
    /// Path relative to the run directory when the file lives there.
    rel: Option<String>,
    path: PathBuf,
}

fn load(tc: &ToolCtx, arg: &str) -> Res<Loaded> {
    let in_run = numbered('S', arg).map(|_| SeriesFile::path_in(&tc.run_dir, arg));
    if let Some(p) = in_run.as_ref().filter(|p| p.is_file()) {
        return Ok(Loaded {
            series: SeriesFile::load_path(p)?,
            rel: Some(format!("series/{arg}.json")),
            path: p.clone(),
        });
    }
    let path = PathBuf::from(arg);
    if !path.is_file() {
        let have = existing_series(&tc.run_dir);
        return Err(Failure::refuse(
            codes::INVALID_VALUE,
            format!(
                "{arg:?} is neither a series in run {} nor a file",
                tc.run_id
            ),
            "an existing series ID (S3) or a path to a series file",
            if have.is_empty() {
                "dla harness run H1 --mode both   (or dla sample --pid <pid>) to write a series"
                    .to_owned()
            } else {
                format!(
                    "dla trend {}   (series in this run: {})",
                    have[0],
                    have.join(", ")
                )
            },
        ));
    }
    let abs = std::path::absolute(&path).unwrap_or(path);
    let rel = abs.strip_prefix(&tc.run_dir).ok().map(to_slash);
    Ok(Loaded {
        series: SeriesFile::load_path(&abs)?,
        rel,
        path: abs,
    })
}

fn existing_series(run_dir: &Path) -> Vec<String> {
    let Ok(rd) = fs::read_dir(run_dir.join(dla_core::series::SERIES_DIR)) else {
        return Vec::new();
    };
    let mut ids: Vec<(u32, String)> = rd
        .filter_map(Result::ok)
        // An empty file is an id reserved by a writer that has not finished.
        .filter(|e| dla_core::fsutil::is_written(&e.path()))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let id = name.strip_suffix(".json")?.to_owned();
            Some((numbered('S', &id)?, id))
        })
        .collect();
    ids.sort();
    ids.into_iter().map(|(_, id)| id).take(8).collect()
}

/// The `Artifact` source of a series file.
fn series_source(tc: &ToolCtx, l: &Loaded) -> Res<Source> {
    if let Some(rel) = &l.rel {
        return Ok(tc.artifact_source(rel)?);
    }
    let bytes = fs::read(&l.path).map_err(|e| dla_core::Error::io(&l.path, e))?;
    Ok(Source::Artifact {
        path: to_slash(&l.path),
        sha256: hex::encode(Sha256::digest(&bytes)),
    })
}

fn check_modes(a: &Loaded, b: &Loaded, a_arg: &str, b_arg: &str) -> Res<()> {
    let ok = matches!(a.series.source.mode, None | Some(KeyMode::Fresh))
        && matches!(b.series.source.mode, None | Some(KeyMode::Repeated));
    if ok {
        return Ok(());
    }
    let swapped = matches!(a.series.source.mode, Some(KeyMode::Repeated))
        && matches!(b.series.source.mode, Some(KeyMode::Fresh));
    let (msg, fix) = if swapped {
        (
            format!("{a_arg} is a repeated-keys series and {b_arg} a fresh-keys series"),
            format!("dla trend {b_arg} --against {a_arg}"),
        )
    } else {
        (
            format!(
                "--against compares a fresh-keys series with a repeated-keys one, but {a_arg} is {} and {b_arg} is {}",
                mode_name(&a.series),
                mode_name(&b.series)
            ),
            "dla harness run H1 --mode both   (a driver harness writes a fresh and a repeated series)"
                .to_owned(),
        )
    };
    Err(Failure::refuse(
        codes::INVALID_VALUE,
        msg,
        "a fresh-keys series and a repeated-keys series",
        fix,
    ))
}

fn mode_name(s: &SeriesFile) -> &'static str {
    s.source.mode.map_or("of unknown mode", |m| m.as_str())
}

/// `dla trend`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: TrendArgs) -> Res<Value> {
    RunState::load(&ctx.run_dir)?;
    let tc = stage::tool_ctx(project, ctx, data);
    let main = load(&tc, &a.series)?;
    let idx = analysis::select(&main.series, &a.columns)?;
    let other = match &a.against {
        Some(arg) => {
            let o = load(&tc, arg)?;
            check_modes(&main, &o, &a.series, arg)?;
            Some(o)
        }
        None => None,
    };
    let mut ledger = tc.ledger()?;
    if let Some(f) = &a.finding
        && ledger.finding(f).is_none()
    {
        return Err(Failure::refuse(
            codes::UNKNOWN_FINDING,
            format!("{f} is not in the ledger"),
            "an existing finding for --finding",
            "dla run status   (lists findings)",
        ));
    }

    let th = GateThresholds::default();
    let mut cols = analysis::analyze_series(&main.series, &idx, &th);
    candidates::Matcher::new(project, &ledger).fill(&mut cols);

    // Persist the full report first: the evidence points at it.
    let trend_id = tc.next_trend_id();
    let report = TrendReport {
        schema_version: dla_core::trend::TREND_SCHEMA_VERSION,
        id: trend_id.clone(),
        series: main.series.id.clone(),
        thresholds: th,
        columns: cols.clone(),
    };
    report.save(&tc.run_dir)?;
    let raw = format!("{TREND_DIR}/{trend_id}.json");
    let source = series_source(&tc, &main)?;
    let authority = crate::gap_cmd::series_authority(&main.series);
    // The growth measurement's own signal: `dla ledger strong` judges it independent of a
    // retention record or an intervention by tool, class and source.
    let signal = Signal::new(
        "dla-trend",
        tool_class::HARNESS_SERIES,
        match &source {
            Source::Artifact { sha256, .. } => sha256,
            _ => "",
        },
    );

    let (gauges, counters) = summary::by_kind(&cols);
    let mut evidence: Vec<(EvidenceKind, EvidenceId)> = Vec::new();
    for (kind, group, ckind) in [
        (EvidenceKind::GrowthSeries, &gauges, ColumnKind::Gauge),
        (
            EvidenceKind::UnitCountSeries,
            &counters,
            ColumnKind::Counter,
        ),
    ] {
        if group.is_empty() {
            continue;
        }
        let total = main
            .series
            .columns
            .iter()
            .filter(|c| c.kind == ckind)
            .count() as u64;
        let mut spec = EvidenceSpec::new(
            kind,
            "dla",
            env!("CARGO_PKG_VERSION"),
            summary::summarize(&trend_id, &main.series, group, &signal),
        );
        spec.authority = authority;
        spec.raw = Some(raw.clone());
        spec.source = Some(source.clone());
        spec.coverage = EvidenceCoverage {
            status: if group.len() as u64 == total {
                CoverageStatus::Complete
            } else {
                CoverageStatus::Partial
            },
            examined: Some(group.len() as u64),
            total: Some(total),
        };
        spec.limitations = main.series.limitations.clone();
        spec.limitations
            .push("gate thresholds are provisional (not calibrated)".to_owned());
        let id = tc.add_evidence(&mut ledger, spec)?;
        evidence.push((kind, id));
    }

    // Key-mode comparison.
    let keys = match &other {
        Some(o) => Some(keys_test(
            &tc,
            &mut ledger,
            &main,
            o,
            &cols,
            &idx,
            &a,
            &trend_id,
            &source,
        )?),
        None => None,
    };

    let mut out = Map::new();
    out.insert("ok".into(), json!(true));
    out.insert("command".into(), json!("trend"));
    out.insert("trend".into(), json!(trend_id));
    out.insert("series".into(), json!(main.series.id));
    out.insert("columns".into(), json!(cols.len()));
    let mut ev: Vec<String> = evidence.iter().map(|(_, e)| e.to_string()).collect();
    if let Some(k) = &keys {
        ev.push(k.evidence.to_string());
    }
    out.insert("evidence".into(), json!(ev));
    let all: Vec<&ColumnTrend> = cols.iter().collect();
    let passed = summary::passing(&all);
    out.insert(
        "passed".into(),
        Value::Array(
            passed
                .iter()
                .take(8)
                .map(|c| {
                    json!({
                        "column": c.name,
                        "shape": c.shape,
                        "slope": c.sen.map(|s| s.slope),
                        "candidates": c.candidates.iter().map(ToString::to_string).collect::<Vec<_>>(),
                    })
                })
                .collect(),
        ),
    );
    if passed.len() > 8 {
        out.insert("passed_more".into(), json!(passed.len() - 8));
    }
    out.insert("failed".into(), json!(cols.len() - passed.len()));
    if let Some(k) = &keys {
        out.insert("keys".into(), k.summary.clone());
    }
    let next = suggestions(
        &ledger,
        &main.series,
        &passed,
        &evidence,
        keys.as_ref(),
        a.finding.as_ref(),
    );
    out.insert("next".into(), json!(next));
    Ok(Value::Object(out))
}

/// What the key-mode comparison produced.
struct Keys {
    evidence: EvidenceId,
    favors: Favors,
    summary: Value,
}

#[allow(clippy::too_many_arguments)]
fn keys_test(
    tc: &ToolCtx,
    ledger: &mut Ledger,
    main: &Loaded,
    other: &Loaded,
    fresh: &[ColumnTrend],
    idx: &[usize],
    a: &TrendArgs,
    trend_id: &str,
    source: &Source,
) -> Res<Keys> {
    let th = GateThresholds::default();
    let names: Vec<&str> = idx
        .iter()
        .map(|&i| main.series.columns[i].name.as_str())
        .collect();
    let oidx: Vec<usize> = (0..other.series.columns.len())
        .filter(|&i| names.contains(&other.series.columns[i].name.as_str()))
        .collect();
    let repeated = analysis::analyze_series(&other.series, &oidx, &th);
    let tie = a
        .finding
        .as_ref()
        .and_then(|f| ledger.finding(f).map(|fi| (f.clone(), fi.site.clone())));
    let decision = against::compare(
        fresh,
        &repeated,
        tie.as_ref().map(|(f, s)| (f, s.as_deref())),
    );

    // Full comparison file next to the trend report.
    let keys_rel = format!("{TREND_DIR}/{trend_id}-keys.json");
    let (all_cols, _) = decision.columns_json(usize::MAX);
    let full = json!({
        "trend": trend_id, "series": main.series.id, "against": other.series.id,
        "finding": a.finding, "favors": decision.favors, "supports": decision.supports,
        "columns": all_cols,
    });
    write_atomic(
        &tc.run_dir.join(&keys_rel),
        to_pretty_json(&full).as_bytes(),
    )?;

    let mut result = DiscriminatorResult::new("keys", decision.favors);
    result.supports.clone_from(&decision.supports);
    if !decision.untied {
        result.finding.clone_from(&a.finding);
    }
    result.extra.insert("series".into(), json!(main.series.id));
    result
        .extra
        .insert("against".into(), json!(other.series.id));
    result.extra.insert("trend".into(), json!(trend_id));
    result
        .extra
        .insert("compared".into(), json!(decision.columns.len()));
    // The most telling column: the first by verdict rank.
    if let Some(t) = decision.columns_json(1).0.first() {
        result.extra.insert("column".into(), t["column"].clone());
    }
    let mut max = 8;
    let value = loop {
        let (cols, cut) = decision.columns_json(max);
        result.extra.insert("columns".into(), Value::Array(cols));
        if cut > 0 {
            result.extra.insert("truncated".into(), json!(cut));
        }
        let v = result.to_value();
        if serde_json::to_string(&v).map_or(usize::MAX, |s| s.len()) <= MAX_RESULT_BYTES || max == 0
        {
            break v;
        }
        max /= 2;
    };
    let mut spec = EvidenceSpec::new(
        EvidenceKind::Discriminator,
        "dla",
        env!("CARGO_PKG_VERSION"),
        value,
    );
    spec.authority = Authority::Test;
    spec.raw = Some(keys_rel);
    spec.source = Some(source.clone());
    spec.limitations = vec![
        "gate thresholds are provisional (not calibrated)".to_owned(),
        format!(
            "fresh keys {} against repeated keys {}",
            main.series.id, other.series.id
        ),
    ];
    if decision.untied {
        spec.limitations.push(
            "no column of the series is tied to the finding, so the comparison is not bound to it"
                .to_owned(),
        );
    }
    let evidence = tc.add_evidence(ledger, spec)?;
    let (cols, cut) = decision.columns_json(6);
    Ok(Keys {
        evidence,
        favors: decision.favors,
        summary: json!({
            "favors": decision.favors,
            "supports": decision.supports,
            "finding": if decision.untied { None } else { a.finding.clone() },
            "against": other.series.id,
            "compared": decision.columns.len(),
            "columns": cols,
            "columns_more": cut,
        }),
    })
}

/// The evidence recorded for the kind of `c`.
fn evidence_of(evidence: &[(EvidenceKind, EvidenceId)], c: &ColumnTrend) -> Option<String> {
    let want = match c.kind {
        ColumnKind::Gauge => EvidenceKind::GrowthSeries,
        ColumnKind::Counter => EvidenceKind::UnitCountSeries,
    };
    evidence
        .iter()
        .find(|(k, _)| *k == want)
        .map(|(_, e)| e.to_string())
}

/// The latest `workload` evidence of the series' harness.
fn harness_evidence(ledger: &Ledger, series: &SeriesFile) -> Option<String> {
    let h = series.source.harness.as_deref()?;
    ledger
        .evidence()
        .filter(|e| e.kind == EvidenceKind::Workload)
        .filter(|e| e.result.get("harness").and_then(Value::as_str) == Some(h))
        .last()
        .map(|e| e.evidence_id.to_string())
}

const REFUTE_IF: &str = "growth continues over the extra cycles";
const MAX_NEXT: usize = 8;

fn suggestions(
    ledger: &Ledger,
    series: &SeriesFile,
    passed: &[&ColumnTrend],
    evidence: &[(EvidenceKind, EvidenceId)],
    keys: Option<&Keys>,
    bound: Option<&FindingId>,
) -> Vec<String> {
    let mut next: Vec<String> = Vec::new();
    let mut seen_roots: Vec<FindingId> = Vec::new();
    let mut unbound = 0;
    let harness = harness_evidence(ledger, series).unwrap_or_else(|| "<harness evidence>".into());
    for c in passed {
        let Some(e) = evidence_of(evidence, c) else {
            continue;
        };
        let cand = c.candidates.iter().find_map(|f| ledger.finding(f));
        match cand {
            Some(f) => {
                let root = f.root_id();
                if seen_roots.contains(&root) {
                    continue;
                }
                seen_roots.push(root.clone());
                let has_nonleak = ledger
                    .findings()
                    .any(|s| s.parent.as_ref() == Some(&root) && s.polarity() == Polarity::NonLeak);
                if !has_nonleak {
                    next.push(format!(
                        "dla ledger add --sibling-of {root} --origin hypothesis --hypothesis N-WARM --refute-if \"{REFUTE_IF}\" --evidence {e}"
                    ));
                }
                if ledger
                    .finding(&root)
                    .is_some_and(|r| r.state == FindingState::Candidate)
                {
                    next.push(format!("dla ledger test {root} --evidence {harness}"));
                }
                if let Some(h) = series.source.harness.as_deref() {
                    next.push(format!("dla alloc-test {h} --finding {root}"));
                }
            }
            None if dla_core::gate::is_whole_process(&c.name) => {
                // The gate never ties a whole-process column to a finding: the residual
                // accounting covers it, so a `ledger add` would open a finding that can
                // never be tied.
                let hint = format!(
                    "dla gap {} --residual   # column {} measures the whole process: account for it with the residual rows",
                    series.id, c.name
                );
                if !next.contains(&hint)
                    && next.iter().filter(|n| n.starts_with("dla gap ")).count() < 1
                {
                    next.push(hint);
                }
            }
            None if unbound < 3 => {
                unbound += 1;
                let mut cmd = format!("dla ledger add --origin growth-series --evidence {e}");
                if let Some(r) = finding_region(c.region) {
                    cmd.push_str(&format!(" --region {r}"));
                }
                if let Some(s) = finding_shape(c.shape) {
                    cmd.push_str(&format!(" --shape {s}"));
                }
                cmd.push_str(&format!("   # column {}", c.name));
                next.push(cmd);
            }
            None => {}
        }
    }
    if let (Some(k), Some(f)) = (keys, bound) {
        let root = ledger.finding(f).map_or_else(|| f.clone(), |x| x.root_id());
        match k.favors {
            Favors::NonLeak => next.push(format!(
                "dla ledger refute {root} --evidence {}",
                k.evidence
            )),
            Favors::Leak => {
                for s in ledger.findings() {
                    if s.parent.as_ref() == Some(&root)
                        && s.polarity() == Polarity::NonLeak
                        && !s.state.is_dispositioned()
                    {
                        next.push(format!(
                            "dla ledger refute {} --evidence {}",
                            s.id, k.evidence
                        ));
                    }
                }
            }
            Favors::Inconclusive => {}
        }
    }
    next.truncate(MAX_NEXT);
    next
}
