//! `dla correlate <H> [--series <S>]`: tie native growth at a foreign-function boundary to
//! the managed worker that calls it.
//!
//! A JVM harness whose project calls a native library through JNI preloads the allocator
//! shim, so its series holds `native.live_bytes@<file:line>` columns for the native
//! allocation sites. The harness run also records which of those sites sit inside an
//! exported JNI function and the managed method that function implements. The managed worker
//! has no census entry for such a site (the holder is native code) and a tie never crosses
//! runtimes, so the native column would tie to nothing there.
//!
//! This command reads the series and the run record of harness H and, for each native site
//! under a boundary method, compares per-iteration slopes (Theil-Sen over the post-warm-up
//! points, the same gate `dla trend` uses):
//!
//! * the site's `native.live_bytes@site` column must pass the slope gate (the allocator shim
//!   counts the site's live bytes directly: this is the growth);
//! * the site's slope must be between [`MIN_SHARE`] and [`MAX_RATIO`] of the managed process
//!   footprint slope (`os.rss_anon`, else `os.rss`): native bytes held through malloc show up
//!   in the anonymous resident set, so a site cannot grow much faster than the process does,
//!   and one that grows far slower is not what the process footprint shows. The footprint's
//!   Theil-Sen slope is used even when its column misses the gate: whole-process columns are
//!   noisy (several MB on CI runners) while the slope estimate stays sound; the evidence says
//!   whether the footprint passed (`managed.gated`), and a footprint with no positive slope
//!   rejects the site.
//!
//! With those it writes one `alloc-stack` evidence (both slopes, the boundary, the stack) and
//! adds a candidate to the ledger of the worker it runs in: origin `correlation`, site the
//! native site, holder `native-handle/global-ref`, region `off-heap`, owner the managed
//! method, writer the method's declaration. The shim column then ties to the finding by site.
//! Confirmation goes through the normal ledger rules.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use dla_core::{
    EvidenceId, FindingId, HarnessManifest, Ledger, RunState, SeriesFile,
    error::codes,
    evidence::{
        Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal, result_size, tool_class,
    },
    finding::{Holder, Region},
    fsutil::{is_written, reserve_numbered, to_pretty_json, write_atomic},
    series::{SERIES_DIR, SeriesSourceKind, numbered},
    toolctx::{Candidate, CandidateWriter, EvidenceSpec, ToolCtx},
};
use dla_stats::{Analysis, GateThresholds, GateVerdict, analyze};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    stage,
};

/// Directory (inside the run directory) holding the full correlation results.
const CORRELATE_DIR: &str = "correlate";
/// Prefix of the shim's per-site live-byte columns.
const SITE_PREFIX: &str = "native.live_bytes@";
/// Columns that stand for the managed process footprint, best first.
const FOOTPRINT: [&str; 2] = ["os.rss_anon", "os.rss"];
/// Managed heap columns reported next to the footprint, best first.
const HEAP: [&str; 2] = ["jvm.heap_used", "jvm.heap_committed"];
/// The smallest share of the footprint slope a native site must account for.
pub const MIN_SHARE: f64 = 0.2;
/// The largest ratio of the site slope to the footprint slope (a quarter above 1 allows for
/// page granularity and sampling noise).
pub const MAX_RATIO: f64 = 1.25;
/// Most sites examined in one run.
const MAX_SITES: usize = 3;
/// Bytes of an evidence result before the signal is added.
const RESULT_BUDGET: usize = 1680;
/// Most rejected sites listed in the command output.
const MAX_REJECTED: usize = 3;

/// Flags of `dla correlate`.
#[derive(Args, Debug)]
pub struct CorrelateArgs {
    /// Harness ID from `dla harness gen` (`H1`); its manifest must list an FFI boundary.
    #[arg(value_name = "H")]
    harness: String,
    /// The series of the run (`S1`); default: the newest series of the harness.
    #[arg(long, value_name = "S")]
    series: Option<String>,
}

/// A native allocation site under a boundary method, as the harness run recorded it.
#[derive(Debug, Clone, PartialEq)]
struct BoundarySite {
    site: String,
    managed: String,
    export: Option<String>,
    declared_at: Option<String>,
}

/// Slope figures of one column.
#[derive(Debug, Clone, PartialEq)]
struct Slope {
    column: String,
    slope: f64,
    lo: f64,
    p: Option<f64>,
    n: u32,
    /// The column passed the slope gate.
    gated: bool,
}

/// Why a site was not correlated.
#[derive(Debug, Clone, PartialEq)]
struct Rejected {
    site: String,
    why: String,
}

/// The verdict for one site.
#[derive(Debug, Clone, PartialEq)]
enum Verdict {
    Match {
        native: Slope,
        managed: Slope,
        heap: Option<Slope>,
        ratio: f64,
    },
    No(String),
}

fn round(x: f64) -> f64 {
    if x.is_finite() {
        (x * 1000.0).round() / 1000.0
    } else {
        0.0
    }
}

/// The slope of `column` when it grows by the gate, else why not.
fn gated_slope(series: &SeriesFile, column: &str, th: &GateThresholds) -> Result<Slope, String> {
    let Some(i) = series.column_index(column) else {
        return Err(format!("the series has no column {column}"));
    };
    let pts = series.post_warmup(i);
    let a: Analysis = analyze(&pts, None, false, th);
    let Some(sen) = a.sen else {
        return Err(format!(
            "{column} has {} usable points, too few for a slope",
            pts.len()
        ));
    };
    if a.gate != GateVerdict::Pass {
        return Err(format!(
            "{column} does not grow by the slope gate ({})",
            a.gate_failed.join(", ")
        ));
    }
    Ok(Slope {
        column: column.to_owned(),
        slope: sen.slope,
        lo: sen.lo,
        p: a.mk.map(|m| m.p_increasing),
        n: a.n,
        gated: true,
    })
}

/// The Theil-Sen slope of `column` whether or not it passes the gate (`gated` says which).
fn any_slope(series: &SeriesFile, column: &str, th: &GateThresholds) -> Option<Slope> {
    let i = series.column_index(column)?;
    let pts = series.post_warmup(i);
    let a: Analysis = analyze(&pts, None, false, th);
    let sen = a.sen?;
    Some(Slope {
        column: column.to_owned(),
        slope: sen.slope,
        lo: sen.lo,
        p: a.mk.map(|m| m.p_increasing),
        n: a.n,
        gated: a.gate == GateVerdict::Pass,
    })
}

/// Compares the slope of the site column with the managed process footprint.
fn judge(series: &SeriesFile, site: &str, th: &GateThresholds) -> Verdict {
    let column = format!("{SITE_PREFIX}{site}");
    let native = match gated_slope(series, &column, th) {
        Ok(s) => s,
        Err(why) => return Verdict::No(why),
    };
    let mut why_not: Vec<String> = Vec::new();
    let mut managed = None;
    for c in FOOTPRINT {
        if series.column_index(c).is_none() {
            continue;
        }
        match gated_slope(series, c, th) {
            Ok(s) => {
                managed = Some(s);
                break;
            }
            Err(w) => why_not.push(w),
        }
    }
    // No footprint column passed the gate: its slope estimate still bounds the site.
    let managed = managed.or_else(|| {
        FOOTPRINT
            .iter()
            .find_map(|c| any_slope(series, c, th).filter(|s| s.slope > 0.0))
    });
    let Some(managed) = managed else {
        let why = if why_not.is_empty() {
            "the series has no os.rss_anon or os.rss column to compare with".to_owned()
        } else {
            format!(
                "the managed footprint shows no growth: {}",
                why_not.join("; ")
            )
        };
        return Verdict::No(why);
    };
    if managed.slope <= 0.0 {
        return Verdict::No(format!("{} has no positive slope", managed.column));
    }
    let ratio = native.slope / managed.slope;
    if !(MIN_SHARE..=MAX_RATIO).contains(&ratio) {
        return Verdict::No(format!(
            "the site grows {:.0} B per unit and {} {:.0} B (ratio {:.2}, outside {MIN_SHARE} to {MAX_RATIO})",
            native.slope, managed.column, managed.slope, ratio
        ));
    }
    let heap = HEAP.iter().find_map(|c| gated_slope(series, c, th).ok());
    Verdict::Match {
        native,
        managed,
        heap,
        ratio,
    }
}

/// The series files of the run, oldest first.
fn series_ids(run_dir: &Path) -> Vec<(u32, String)> {
    let Ok(rd) = fs::read_dir(run_dir.join(SERIES_DIR)) else {
        return Vec::new();
    };
    let mut ids: Vec<(u32, String)> = rd
        .filter_map(Result::ok)
        .filter(|e| is_written(&e.path()))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let id = name.strip_suffix(".json")?.to_owned();
            Some((numbered('S', &id)?, id))
        })
        .collect();
    ids.sort();
    ids
}

/// The series to read: the named one, else the newest harness series of `harness` that
/// holds shim site columns.
fn pick_series(tc: &ToolCtx, harness: &str, arg: Option<&str>) -> Res<SeriesFile> {
    let ids = series_ids(&tc.run_dir);
    if let Some(id) = arg {
        if !ids.iter().any(|(_, i)| i == id) {
            let have: Vec<&str> = ids.iter().rev().take(8).map(|(_, i)| i.as_str()).collect();
            return Err(Failure::refuse(
                codes::INVALID_VALUE,
                format!("{id:?} is not a series in run {}", tc.run_id),
                "an existing series ID (S3)",
                if have.is_empty() {
                    format!("dla harness run {harness}   to write a series")
                } else {
                    format!(
                        "dla correlate {harness} --series {}   (series in this run: {})",
                        have[0],
                        have.join(", ")
                    )
                },
            ));
        }
        let s = SeriesFile::load(&tc.run_dir, id)?;
        if s.source.harness.as_deref() != Some(harness) {
            return Err(Failure::refuse(
                codes::INVALID_VALUE,
                format!(
                    "series {id} was written by {}, not by harness {harness}",
                    s.source.harness.as_deref().unwrap_or("no harness")
                ),
                format!("a series of harness {harness}"),
                format!("dla correlate {harness}"),
            ));
        }
        return Ok(s);
    }
    for (_, id) in ids.iter().rev() {
        let s = SeriesFile::load(&tc.run_dir, id)?;
        if s.source.kind == SeriesSourceKind::Harness
            && s.source.harness.as_deref() == Some(harness)
            && s.columns.iter().any(|c| c.name.starts_with(SITE_PREFIX))
        {
            return Ok(s);
        }
    }
    Err(Failure::refuse(
        codes::INVALID_VALUE,
        format!(
            "run {} has no series of harness {harness} with native allocation sites",
            tc.run_id
        ),
        format!("a series written by dla harness run {harness} with the allocator shim loaded"),
        format!("dla harness run {harness}"),
    ))
}

/// The sites under a boundary method that the harness run recorded for `series`,
/// largest first, with the method's declaration from the manifest.
fn boundary_sites(tc: &ToolCtx, manifest: &HarnessManifest, series: &str) -> Vec<BoundarySite> {
    let Some(ffi) = manifest.ffi.as_ref() else {
        return Vec::new();
    };
    let dir = tc.run_dir.join("harness").join(&manifest.id);
    let Ok(rd) = fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = rd
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("run-") && n.ends_with(".json"))
        })
        .collect();
    files.sort();
    let mut out: Vec<BoundarySite> = Vec::new();
    for f in files.iter().rev() {
        let Ok(text) = fs::read_to_string(f) else {
            continue;
        };
        let Ok(doc) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let lists_series = doc["series"]
            .as_array()
            .is_some_and(|a| a.iter().any(|s| s.as_str() == Some(series)));
        if !lists_series {
            continue;
        }
        for s in doc["ffi_sites"].as_array().into_iter().flatten() {
            let (Some(site), Some(method)) = (s["site"].as_str(), s["method"].as_str()) else {
                continue;
            };
            let m = ffi.methods.iter().find(|m| m.managed == method);
            if out.iter().any(|o| o.site == site) {
                continue;
            }
            out.push(BoundarySite {
                site: site.to_owned(),
                managed: method.to_owned(),
                export: m.map(|m| m.export.clone()),
                declared_at: m.and_then(|m| m.declared_at.clone()),
            });
        }
        break;
    }
    out
}

fn slope_json(s: &Slope) -> Value {
    json!({
        "column": s.column,
        "slope": round(s.slope),
        "lo": round(s.lo),
        "p": s.p.map(|p| (p * 1e6).round() / 1e6),
        "n": s.n,
        "gated": s.gated,
    })
}

/// Reserves `correlate/C<n>.json` and writes `doc` to it; returns the relative path and
/// the SHA-256 of what was written.
fn write_raw(tc: &ToolCtx, doc: &Value) -> Res<(String, String)> {
    let dir = tc.run_dir.join(CORRELATE_DIR);
    fs::create_dir_all(&dir).map_err(|e| Failure::from(dla_core::Error::io(&dir, e)))?;
    let (n, path) =
        reserve_numbered(&dir, 'C', ".json", false).map_err(|e| Failure::from(e.to_refusal()))?;
    let body = to_pretty_json(doc);
    write_atomic(&path, body.as_bytes()).map_err(|e| Failure::from(e.to_refusal()))?;
    Ok((
        format!("{CORRELATE_DIR}/C{n}.json"),
        hex::encode(Sha256::digest(body.as_bytes())),
    ))
}

/// The bounded evidence result of one correlated site.
#[allow(clippy::too_many_arguments)]
fn evidence_result(
    manifest: &HarnessManifest,
    series: &SeriesFile,
    s: &BoundarySite,
    native: &Slope,
    managed: &Slope,
    heap: Option<&Slope>,
    ratio: f64,
    signal: &Signal,
) -> Value {
    let ffi = manifest.ffi.as_ref();
    let mut v = json!({
        "harness": manifest.id,
        "series": series.id,
        "boundary": {
            "kind": ffi.map(|f| f.kind.as_str()),
            "library": ffi.and_then(|f| f.library.as_deref()),
        },
        "site": s.site,
        "export": s.export,
        "method": s.managed,
        "declared_at": s.declared_at,
        "native": slope_json(native),
        "managed": slope_json(managed),
        "ratio": round(ratio),
        "tolerance": {"min_share": MIN_SHARE, "max_ratio": MAX_RATIO},
        "stack": [
            format!("{} in {}", s.site, s.export.as_deref().unwrap_or("?")),
            format!(
                "{}{}",
                s.managed,
                s.declared_at.as_ref().map_or_else(String::new, |d| format!(" ({d})"))
            ),
        ],
        "signal": signal.to_value(),
    });
    if let Some(h) = heap {
        v["heap"] = slope_json(h);
    }
    // Drop optional parts, not the pointers, when the result does not fit.
    if result_size(&v) > RESULT_BUDGET + 380 {
        v["stack"] = json!([]);
    }
    v
}

/// Runs `dla correlate`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: CorrelateArgs) -> Res<Value> {
    RunState::load(&ctx.run_dir)?;
    let tc = stage::tool_ctx(project, ctx, data);
    let manifest = tc.harness(&a.harness)?;
    let Some(ffi) = manifest.ffi.clone() else {
        return Err(Failure::refuse(
            codes::INVALID_VALUE,
            format!(
                "harness {} has no foreign-function boundary: nothing native to correlate",
                manifest.id
            ),
            "a harness whose probe lists an FFI boundary (probe summary `boundaries`)",
            "dla probe   then   dla harness gen --lang jvm   (a JVM project that loads a JNI library gets the allocator shim)",
        ));
    };
    let series = pick_series(&tc, &manifest.id, a.series.as_deref())?;
    let sites = boundary_sites(&tc, &manifest, &series.id);
    let th = GateThresholds::default();

    let mut ledger = tc.ledger()?;
    let mut writer = CandidateWriter::for_ctx(&tc, &ledger)?;
    let series_rel = format!("{SERIES_DIR}/{}.json", series.id);
    let source = tc.artifact_source(&series_rel)?;
    let series_sha = match &source {
        dla_core::evidence::Source::Artifact { sha256, .. } => sha256.clone(),
        _ => String::new(),
    };
    let signal = Signal::new("dla-correlate", tool_class::ALLOC_SHIM, &series_sha);

    let mut rejected: Vec<Rejected> = Vec::new();
    let mut matches: Vec<(BoundarySite, Verdict)> = Vec::new();
    for s in sites.iter().take(MAX_SITES) {
        match judge(&series, &s.site, &th) {
            Verdict::No(why) => rejected.push(Rejected {
                site: s.site.clone(),
                why,
            }),
            v @ Verdict::Match { .. } => matches.push((s.clone(), v)),
        }
    }

    // The full result: every judged site with both slopes.
    let full = json!({
        "harness": manifest.id,
        "series": series.id,
        "boundary": serde_json::to_value(&ffi).unwrap_or(Value::Null),
        "tolerance": {"min_share": MIN_SHARE, "max_ratio": MAX_RATIO},
        "sites": sites.iter().take(MAX_SITES).map(|s| json!({
            "site": s.site, "method": s.managed, "export": s.export, "declared_at": s.declared_at,
        })).collect::<Vec<_>>(),
        "matched": matches.iter().filter_map(|(s, v)| match v {
            Verdict::Match { native, managed, heap, ratio } => Some(json!({
                "site": s.site,
                "native": slope_json(native),
                "managed": slope_json(managed),
                "heap": heap.as_ref().map(slope_json),
                "ratio": round(*ratio),
            })),
            Verdict::No(_) => None,
        }).collect::<Vec<_>>(),
        "rejected": rejected.iter().map(|r| json!({"site": r.site, "why": r.why})).collect::<Vec<_>>(),
        "note": "the stack is the project frame of the largest shim allocation stack at the site, mapped to the exported function and the managed method that calls it",
    });

    let mut evidence: Vec<String> = Vec::new();
    let mut added: Vec<Value> = Vec::new();
    let mut existing: Vec<Value> = Vec::new();
    let mut skipped = 0u64;
    let mut out_of_scope = 0u64;
    let mut raw: Option<(String, String)> = None;
    for (s, v) in &matches {
        let Verdict::Match {
            native,
            managed,
            heap,
            ratio,
        } = v
        else {
            continue;
        };
        // A site that already has a correlation finding is done.
        if ledger
            .findings()
            .any(|f| f.origin.as_str() == "correlation" && f.site.as_deref() == Some(&s.site))
        {
            skipped += 1;
            continue;
        }
        // Ties never cross runtimes: only a finding of this worker can tie the managed
        // worker's shim column, so a finding another worker merged into the core ledger
        // at the same site does not make the site covered. Checked before any evidence
        // is written, so a skipped site leaves no orphan record.
        let others: Vec<String> = ledger
            .findings()
            .filter(|f| f.site.as_deref() == Some(&s.site))
            .filter(|f| f.id.prefix() == tc.worker.as_deref())
            .map(|f| f.id.to_string())
            .collect();
        if !others.is_empty() {
            existing.push(json!({"site": s.site, "findings": others}));
            continue;
        }
        let (rel, _) = match &raw {
            Some(r) => r.clone(),
            None => {
                let r = write_raw(&tc, &full)?;
                raw = Some(r.clone());
                r
            }
        };
        let mut spec = EvidenceSpec::new(
            EvidenceKind::AllocStack,
            "dla-correlate",
            env!("CARGO_PKG_VERSION"),
            evidence_result(
                &manifest,
                &series,
                s,
                native,
                managed,
                heap.as_ref(),
                *ratio,
                &signal,
            ),
        );
        spec.authority = Authority::Test;
        spec.raw = Some(rel);
        spec.source = Some(source.clone());
        spec.coverage = EvidenceCoverage {
            status: CoverageStatus::Partial,
            examined: Some(1),
            total: Some(sites.len().min(MAX_SITES) as u64),
        };
        spec.limitations = vec![
            "the stack is the project frame of the largest allocation stack at the site, not every frame".to_owned(),
            "slopes are per unit of the series axis; the thresholds are provisional".to_owned(),
        ];
        let ev: EvidenceId = tc.add_evidence(&mut ledger, spec)?;
        evidence.push(ev.to_string());

        let cand = Candidate {
            origin: "correlation".to_owned(),
            evidence: vec![ev],
            site: s.site.clone(),
            holder: Some(Holder::NativeHandleGlobalRef),
            owner: Some(s.managed.clone()),
            region: Some(Region::OffHeap),
            suggested_archetype: None,
        };
        match writer.add(&mut ledger, cand)? {
            Some(fid) => {
                annotate_writer(&mut ledger, &fid, s)?;
                added.push(json!({
                    "finding": fid.to_string(),
                    "site": s.site,
                    "method": s.managed,
                    "native_slope": round(native.slope),
                    "managed_slope": round(managed.slope),
                    "ratio": round(*ratio),
                }));
            }
            None => {
                skipped = writer.skipped();
                out_of_scope = writer.out_of_scope();
            }
        }
    }

    let sha = raw.as_ref().map(|(_, h)| h.clone());
    let mut out = json!({
        "ok": true,
        "command": "correlate",
        "harness": manifest.id,
        "series": series.id,
        "boundary": ffi.kind,
        "evidence": evidence,
        "candidates": added.len(),
        "correlated": added,
        "skipped_duplicates": skipped,
        "sites": sites.len(),
        "rejected": rejected.iter().take(MAX_REJECTED).map(|r| json!({"site": r.site, "why": r.why})).collect::<Vec<_>>(),
    });
    if let Some((rel, _)) = &raw {
        out["raw"] = json!({"path": rel, "sha256": sha});
        out["artifacts"] = json!([tc.pointer(rel)]);
    }
    if !existing.is_empty() {
        out["existing"] = Value::Array(existing);
    }
    if out_of_scope > 0 {
        out["out_of_scope"] = json!(out_of_scope);
    }
    out["next"] = next_steps(&out, &series.id, sites.is_empty(), skipped > 0);
    Ok(out)
}

/// Records the managed method's declaration as the finding's writer.
fn annotate_writer(ledger: &mut Ledger, fid: &FindingId, s: &BoundarySite) -> Res<()> {
    let writers: Vec<String> = s.declared_at.iter().cloned().collect();
    let note = format!(
        "native allocation under {}{}; found by dla correlate",
        s.managed,
        s.export
            .as_ref()
            .map_or_else(String::new, |e| format!(" ({e})"))
    );
    ledger
        .annotate(fid, writers, Vec::new(), Some(note))
        .map_err(|e| Failure::from(e.to_refusal()))?;
    Ok(())
}

fn next_steps(out: &Value, series: &str, no_sites: bool, already: bool) -> Value {
    let mut next: Vec<String> = Vec::new();
    let found = out["correlated"].as_array().is_some_and(|a| !a.is_empty());
    if found {
        next.push(format!(
            "dla trend {series}   (the shim column at the site ties to the new finding by site)"
        ));
        next.push(
            "dla ledger test <F> --evidence <harness evidence>, then alloc-test and confirm as for any finding"
                .to_owned(),
        );
    } else if already {
        next.push(format!(
            "the matched sites already have a correlation finding: dla trend {series}, then the normal ledger steps for it"
        ));
    } else if no_sites {
        next.push(
            "no native allocation site sits in a boundary method: the library may allocate before the first measured point, or the shim saw no project frame (build the native code with debug info)"
                .to_owned(),
        );
    } else {
        next.push(
            "no site matched the managed footprint: an empty result does not clear the native side; raise the iteration count or measure with a native tool (dla harness run --tool valgrind)"
                .to_owned(),
        );
    }
    json!(next)
}

#[cfg(test)]
mod tests {
    use dla_core::series::{
        Axis, Column, ColumnKind, ColumnScope, Point, SeriesRegion, SeriesSource, Unit,
    };

    use super::*;

    fn col(name: &str, region: SeriesRegion) -> Column {
        Column {
            name: name.to_owned(),
            unit: Unit::Bytes,
            kind: ColumnKind::Gauge,
            region,
            baseline: false,
            after_gc: false,
            scope: ColumnScope::Process,
            site: name.split_once('@').map(|(_, s)| s.to_owned()),
            finding: None,
            source: "test".to_owned(),
        }
    }

    /// A series with a warm-up point and ten more; each column is `(name, start, step)`.
    fn series(cols: &[(&str, f64, f64)]) -> SeriesFile {
        let points = (0..11_u32)
            .map(|k| Point {
                x: f64::from(k) + 1.0,
                t_ms: 50 * u64::from(k),
                values: cols
                    .iter()
                    // A little deterministic wobble keeps the rise above the noise floor
                    // test honest without hiding the trend.
                    .map(|(_, s, st)| Some(s + st * f64::from(k) + f64::from(k % 3) * st * 0.01))
                    .collect(),
            })
            .collect();
        SeriesFile {
            schema_version: 1,
            id: "S1".to_owned(),
            source: SeriesSource::new(SeriesSourceKind::Harness),
            axis: Axis::N,
            warmup: 1,
            columns: cols
                .iter()
                .map(|(n, _, _)| col(n, SeriesRegion::Process))
                .collect(),
            points,
            events: Vec::new(),
            limitations: Vec::new(),
        }
    }

    const SITE: &str = "native/src/lib.rs:40";

    #[test]
    fn a_site_that_explains_part_of_the_footprint_matches() {
        let s = series(&[
            ("native.live_bytes@native/src/lib.rs:40", 6000.0, 3074.0),
            ("os.rss_anon", 5.0e7, 7200.0),
            ("jvm.heap_used", 2.5e6, 1450.0),
        ]);
        let v = judge(&s, SITE, &GateThresholds::default());
        let Verdict::Match {
            native,
            managed,
            heap,
            ratio,
        } = v
        else {
            panic!("{v:?}")
        };
        assert!((native.slope - 3074.0).abs() < 100.0, "{}", native.slope);
        assert_eq!(managed.column, "os.rss_anon");
        assert!(heap.is_some());
        assert!((0.3..0.6).contains(&ratio), "{ratio}");
    }

    #[test]
    fn a_site_far_slower_than_the_footprint_does_not_match() {
        let s = series(&[
            ("native.live_bytes@native/src/lib.rs:40", 6000.0, 100.0),
            ("os.rss_anon", 5.0e7, 7200.0),
        ]);
        let Verdict::No(why) = judge(&s, SITE, &GateThresholds::default()) else {
            panic!("matched")
        };
        assert!(why.contains("ratio"), "{why}");
    }

    #[test]
    fn a_site_far_faster_than_the_footprint_does_not_match() {
        let s = series(&[
            ("native.live_bytes@native/src/lib.rs:40", 6000.0, 9000.0),
            ("os.rss_anon", 5.0e7, 3000.0),
        ]);
        assert!(matches!(
            judge(&s, SITE, &GateThresholds::default()),
            Verdict::No(_)
        ));
    }

    #[test]
    fn a_flat_site_or_footprint_does_not_match() {
        let flat_site = series(&[
            ("native.live_bytes@native/src/lib.rs:40", 6000.0, 0.0),
            ("os.rss_anon", 5.0e7, 7200.0),
        ]);
        let Verdict::No(why) = judge(&flat_site, SITE, &GateThresholds::default()) else {
            panic!("matched")
        };
        assert!(why.contains("slope gate"), "{why}");
        let flat_rss = series(&[
            ("native.live_bytes@native/src/lib.rs:40", 6000.0, 3074.0),
            ("os.rss_anon", 5.0e7, 0.0),
            ("os.rss", 5.0e7, 0.0),
        ]);
        let Verdict::No(why) = judge(&flat_rss, SITE, &GateThresholds::default()) else {
            panic!("matched")
        };
        assert!(why.contains("footprint"), "{why}");
    }

    #[test]
    fn a_missing_site_or_footprint_column_is_named() {
        let no_site = series(&[("os.rss_anon", 5.0e7, 7200.0)]);
        let Verdict::No(why) = judge(&no_site, SITE, &GateThresholds::default()) else {
            panic!("matched")
        };
        assert!(why.contains("no column"), "{why}");
        let no_rss = series(&[("native.live_bytes@native/src/lib.rs:40", 6000.0, 3074.0)]);
        let Verdict::No(why) = judge(&no_rss, SITE, &GateThresholds::default()) else {
            panic!("matched")
        };
        assert!(why.contains("os.rss"), "{why}");
    }

    #[test]
    fn a_noisy_footprint_that_misses_the_gate_still_bounds_the_site() {
        // CI runner: os.rss_anon wobbles by several MB, so its column misses the gate, while
        // the shim's site column grows cleanly.
        let mut s = series(&[
            ("native.live_bytes@native/src/lib.rs:40", 6000.0, 3074.0),
            ("os.rss_anon", 5.0e7, 7200.0),
        ]);
        for (k, p) in s.points.iter_mut().enumerate() {
            let wobble = if k % 2 == 0 { 4.0e6 } else { -4.0e6 };
            p.values[1] = p.values[1].map(|v| v + wobble);
        }
        assert!(gated_slope(&s, "os.rss_anon", &GateThresholds::default()).is_err());
        let Verdict::Match { managed, ratio, .. } = judge(&s, SITE, &GateThresholds::default())
        else {
            panic!("no match")
        };
        assert_eq!(managed.column, "os.rss_anon");
        assert!(!managed.gated);
        assert!((0.3..0.6).contains(&ratio), "{ratio}");
        assert_eq!(slope_json(&managed)["gated"], false);
    }

    #[test]
    fn the_footprint_falls_back_to_rss() {
        let s = series(&[
            ("native.live_bytes@native/src/lib.rs:40", 6000.0, 3074.0),
            ("os.rss", 5.0e7, 7200.0),
        ]);
        let Verdict::Match { managed, .. } = judge(&s, SITE, &GateThresholds::default()) else {
            panic!("no match")
        };
        assert_eq!(managed.column, "os.rss");
    }

    #[test]
    fn the_evidence_result_fits_the_bound_with_a_signal() {
        let manifest_json = json!({
            "schema_version": 1, "id": "H1", "runtime": "jvm", "mode": "command",
            "scratch": "harness/H1/work",
            "run": {"argv": ["sh"], "cwd": "."},
            "warmup": 1, "iterations": 8, "step": 100, "key_modes": ["native"],
            "agent": "jvm-agent", "timeout_secs": 60,
            "ffi": {"kind": "jni", "library": "codec", "native_dir": "native", "methods": []}
        });
        let manifest: HarnessManifest = serde_json::from_value(manifest_json).unwrap();
        let s = series(&[
            ("native.live_bytes@native/src/lib.rs:40", 6000.0, 3074.0),
            ("os.rss_anon", 5.0e7, 7200.0),
            ("jvm.heap_used", 2.5e6, 1450.0),
        ]);
        let Verdict::Match {
            native,
            managed,
            heap,
            ratio,
        } = judge(&s, SITE, &GateThresholds::default())
        else {
            panic!("no match")
        };
        let b = BoundarySite {
            site: SITE.to_owned(),
            managed: "shop.codec.Native.open".to_owned(),
            export: Some("Java_shop_codec_Native_open".to_owned()),
            declared_at: Some("kotlin/shop/Native.kt:9".to_owned()),
        };
        let sig = Signal::new("dla-correlate", tool_class::ALLOC_SHIM, &"a".repeat(64));
        let v = evidence_result(
            &manifest,
            &s,
            &b,
            &native,
            &managed,
            heap.as_ref(),
            ratio,
            &sig,
        );
        assert!(
            result_size(&v) <= dla_core::MAX_RESULT_BYTES,
            "{}",
            result_size(&v)
        );
        assert_eq!(v["method"], "shop.codec.Native.open");
        assert_eq!(v["tolerance"]["max_ratio"], MAX_RATIO);
        assert!(v["signal"]["source_sha256"].is_string());
    }
}
