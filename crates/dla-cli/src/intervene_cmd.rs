//! `dla intervene <H> --finding F (--stub-writer FILE:LINE... | --patch FILE.diff) [--rerun-control]`.
//!
//! Edits a clone of a harness, runs the control's points on the clone and judges whether the
//! growth the finding was confirmed on went away. The project and the harness the finding
//! was confirmed on are never changed: the edit lands in the new harness's `work/` copy.
//!
//! # Steps
//!
//! 1. Preconditions: the finding is confirmed and leak-side; one of its cited passing growth
//!    records is a trend of this harness with a passing column tied to the finding (the
//!    control).
//! 2. The edit is planned against the harness's `work/` (read only) and checked against the
//!    finding's recorded sites ([`intervene::plan`]). A refusal here leaves no clone.
//! 3. The harness is cloned ([`intervene::clone`]), the edit written into the clone, and the
//!    clone built and run with the control's mode, point count and warm-up.
//! 4. The treated series is analysed with the control trend's thresholds and judged by
//!    [`dla_stats::judge_intervention`].
//! 5. One `intervention` evidence record is written, whatever the effect. Only
//!    `growth-removed` with every guard true can make the finding `strong`; `growth-remains`
//!    and `inconclusive` never refute anything.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use dla_core::{
    EvidenceId, Finding, FindingId, FindingState, Ledger, MAX_RESULT_BYTES, attribution,
    evidence::{Authority, EvidenceKind, Signal, tool_class},
    finding::Polarity,
    fsutil::{reserve_numbered, to_pretty_json, write_atomic},
    series::{KeyMode, SeriesFile, SeriesSourceKind},
    toolctx::{EvidenceSpec, ToolCtx},
    trend::{ColumnTrend, GrowthSummary, TrendReport, probe_column_matches_owner},
};
use dla_stats::{
    ColumnView, Effect, GateThresholds, GateVerdict, Guards, Treated, intervene::REMAINS_FRACTION,
    judge_intervention,
};
use dla_workload::harness::run::{BUILD_MARKER, RunOptions, run_points};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    parse::parse_enum,
    stage,
    trend_cmd::analysis,
};

#[path = "intervene/clone.rs"]
mod clone;
#[path = "intervene/diff.rs"]
mod diff;
#[path = "intervene/plan.rs"]
mod plan;

/// Refusal codes of `dla intervene`.
pub mod codes {
    /// The finding is not confirmed.
    pub const STATE: &str = "intervene-state";
    /// The finding is not leak-side.
    pub const LEAK_ONLY: &str = "intervene-leak-only";
    /// No cited growth record of this harness has a passing column tied to the finding.
    pub const NO_CONTROL: &str = "intervene-no-control";
    /// A `--stub-writer` site is not a recorded writer of the finding.
    pub const NOT_WRITER: &str = "intervene-not-writer";
    /// The line is not a standalone call, expression or assignment statement.
    pub const NOT_STATEMENT: &str = "intervene-not-statement";
    /// The edit touches a file or a place it may not.
    pub const SCOPE: &str = "intervene-scope";
    /// The diff cannot be read or does not apply.
    pub const PATCH: &str = "intervene-patch";
    /// The diff changes too many lines.
    pub const TOO_LARGE: &str = "intervene-too-large";
}

/// Directory of intervention raw files (`intervene/I<n>/`).
pub const INTERVENE_DIR: &str = "intervene";
/// The sentence every intervention result carries.
const SCRATCH_NOTE: &str = "tested in a scratch copy; the project was not changed";

/// Flags of `dla intervene`.
#[derive(Args, Debug)]
pub struct InterveneArgs {
    /// The harness the finding was confirmed on (H1). It is cloned; the project and this
    /// harness are never changed.
    pub harness: String,
    /// The confirmed finding under test (F3).
    #[arg(long, value_parser = parse_enum::<FindingId>)]
    pub finding: FindingId,
    /// `file:line` of a writer statement to replace with a no-op (repeatable).
    #[arg(
        long = "stub-writer",
        value_name = "FILE:LINE",
        conflicts_with = "patch",
        required_unless_present = "patch"
    )]
    pub stub_writers: Vec<String>,
    /// A unified diff to apply to the scratch copy (files under the harness `work/` only).
    #[arg(long, value_name = "FILE.diff")]
    pub patch: Option<PathBuf>,
    /// Also re-run the unedited clone once, to check the control reproduces.
    #[arg(long)]
    pub rerun_control: bool,
}

/// The control: the trend the finding was confirmed on.
struct Control {
    /// The growth evidence that holds the trend.
    growth: EvidenceId,
    series: SeriesFile,
    report: TrendReport,
    column: ColumnTrend,
}

fn view(c: &ColumnTrend) -> ColumnView {
    ColumnView {
        slope: c.sen.map(|s| s.slope),
        lo: c.sen.map(|s| s.lo),
        gate: c.gate,
        shape: c.shape,
    }
}

/// The finding's control: the newest cited passing growth record whose trend is of harness
/// `h` and has a passing column tied to the finding.
fn find_control(tc: &ToolCtx, ledger: &Ledger, f: &Finding, h: &str) -> Result<Control, Failure> {
    let mut seen: Vec<String> = Vec::new();
    for id in f.supports.iter().rev() {
        let Some(e) = ledger.evidence_by_id(id) else {
            continue;
        };
        if !dla_core::machine::is_passing_growth(e) {
            continue;
        }
        let Some(s) = GrowthSummary::from_result(&e.result) else {
            continue;
        };
        let (Some(trend), Some(series_id)) = (s.trend.as_deref(), s.series.as_deref()) else {
            continue;
        };
        let Some(col) = s
            .passed
            .iter()
            .find(|c| c.tied_to_owner(&f.id, f.site.as_deref(), f.owner.as_deref()))
        else {
            continue;
        };
        let Ok(series) = SeriesFile::load(&tc.run_dir, series_id) else {
            continue;
        };
        if series.source.harness.as_deref() != Some(h) {
            seen.push(format!(
                "{id} ({series_id} is of {})",
                series.source.harness.as_deref().unwrap_or("no harness")
            ));
            continue;
        }
        let Ok(report) = TrendReport::load(&tc.run_dir, trend) else {
            continue;
        };
        let Some(column) = report
            .columns
            .iter()
            .find(|c| c.name == col.column && c.gate == GateVerdict::Pass)
            .cloned()
        else {
            continue;
        };
        return Ok(Control {
            growth: id.clone(),
            series,
            report,
            column,
        });
    }
    Err(Failure::refuse(
        codes::NO_CONTROL,
        format!(
            "{} cites no passing growth record of {h} with a column tied to it{}",
            f.id,
            if seen.is_empty() {
                String::new()
            } else {
                format!(" ({})", seen.join("; "))
            }
        ),
        format!(
            "a trend of a {h} series with a passing column tied to {}",
            f.id
        ),
        format!(
            "dla harness run {h}   then   dla trend <series> --finding {}   then   dla ledger confirm {} --evidence <trend evidence id>",
            f.id, f.id
        ),
    ))
}

fn sites_of(list: &[String]) -> Vec<plan::Site> {
    let mut out: Vec<plan::Site> = list.iter().filter_map(|s| plan::Site::parse(s)).collect();
    out.sort();
    out.dedup();
    out
}

/// The sites an edit may touch, from annotations, the census and the finding.
fn anchors(ctx: &Ctx, project: &Path, ledger: &Ledger, f: &Finding) -> plan::Anchors {
    let (aw, ar) = ledger.recorded_sites(&f.id);
    let resolved = f
        .owner
        .as_deref()
        .and_then(|o| attribution::resolve(&ctx.run_dir, project, o));
    let mut writers = aw;
    let mut removers = ar;
    let mut declared: Vec<String> = Vec::new();
    if let Some(r) = &resolved {
        writers.extend(r.writers.iter().cloned());
        removers.extend(r.removers.iter().cloned());
        declared.push(r.site.clone());
    }
    if let Some(s) = &f.site {
        writers.push(s.clone());
        declared.push(s.clone());
    }
    let mut near: Vec<String> = writers.clone();
    near.extend(removers);
    near.extend(declared);
    plan::Anchors {
        finding: f.id.to_string(),
        writers: sites_of(&writers),
        near: sites_of(&near),
    }
}

fn sha_hex(parts: &[&[u8]]) -> String {
    let mut h = Sha256::new();
    for p in parts {
        h.update(p);
    }
    hex::encode(h.finalize())
}

fn same_points(a: &SeriesFile, b: &SeriesFile) -> bool {
    a.source.mode == b.source.mode
        && a.warmup == b.warmup
        && a.points.len() == b.points.len()
        && a.points
            .iter()
            .zip(&b.points)
            .all(|(p, q)| (p.x - q.x).abs() < 1e-9)
}

/// What one run of the clone gave.
struct Measured {
    series: SeriesFile,
    trend: Vec<ColumnTrend>,
    run_ok: bool,
    limitations: Vec<String>,
}

enum Ran {
    Ok(Box<Measured>),
    /// The harness failed (`harness-failed`); `build` says whether it was the build.
    Failed {
        build: bool,
        why: String,
    },
}

fn run_clone(
    tc: &ToolCtx,
    manifest: &dla_core::harness::HarnessManifest,
    ctl: &Control,
    th: &GateThresholds,
    mode: KeyMode,
) -> Res<Ran> {
    let post = ctl
        .series
        .points
        .len()
        .saturating_sub(ctl.series.warmup as usize);
    let opts = RunOptions {
        modes: vec![mode],
        iterations: Some(u32::try_from(post).unwrap_or(u32::MAX)),
        source_kind: SeriesSourceKind::Intervention,
        ..RunOptions::default()
    };
    let out = match run_points(tc, manifest, &opts) {
        Ok(o) => o,
        Err(r) if r.code == "harness-failed" => {
            return Ok(Ran::Failed {
                build: r.message.contains("did not build"),
                why: r.message,
            });
        }
        Err(r) => return Err(r.into()),
    };
    let Some(id) = out.series.first() else {
        return Ok(Ran::Failed {
            build: false,
            why: format!("harness {} produced no series", manifest.id),
        });
    };
    let series = SeriesFile::load(&tc.run_dir, id)?;
    let idx: Vec<usize> = (0..series.columns.len()).collect();
    let trend = analysis::analyze_series(&series, &idx, th);
    let run_ok = !out.exits.is_empty() && out.exits.iter().all(|(_, c)| *c == Some(0));
    let mut limitations = out.limitations;
    limitations.extend(out.gaps);
    Ok(Ran::Ok(Box::new(Measured {
        series,
        trend,
        run_ok,
        limitations,
    })))
}

fn column_json(series: &str, trend: &str, name: &str, c: Option<&ColumnTrend>) -> Value {
    match c {
        Some(c) => json!({
            "series": series, "trend": trend, "column": name,
            "slope": c.sen.map(|s| round(s.slope)),
            "lo": c.sen.map(|s| round(s.lo)),
            "gate": c.gate, "shape": c.shape,
        }),
        None => json!({"series": series, "trend": trend, "column": name, "present": false}),
    }
}

fn round(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

/// The control's passing columns that are not the finding's: another holder or `os.rss`.
fn other_passing<'a>(ctl: &'a Control, f: &Finding) -> Vec<&'a ColumnTrend> {
    ctl.report
        .columns
        .iter()
        .filter(|c| c.gate == GateVerdict::Pass && c.name != ctl.column.name)
        .filter(|c| {
            !(c.finding.as_ref() == Some(&f.id)
                || c.candidates.contains(&f.id)
                || f.site.is_some() && c.site == f.site
                || f.owner
                    .as_deref()
                    .is_some_and(|o| probe_column_matches_owner(&c.name, o)))
        })
        .collect()
}

fn present(t: &ColumnTrend) -> bool {
    t.n > 0
}

/// Fits the evidence `result` in [`MAX_RESULT_BYTES`] by shortening the site list.
fn fit(mut v: Value) -> Value {
    for keep in [8usize, 4, 2, 1, 0] {
        if serde_json::to_string(&v).map_or(usize::MAX, |s| s.len()) <= MAX_RESULT_BYTES {
            return v;
        }
        if let Some(Value::Array(a)) = v.get_mut("sites") {
            a.truncate(keep);
        }
    }
    v
}

/// `dla intervene`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: InterveneArgs) -> Res<Value> {
    if dla_core::series::numbered('H', &a.harness).is_none() {
        return Err(Failure::usage(
            format!("{:?} is not a harness ID", a.harness),
            "a harness ID such as H1",
            "dla intervene H1 --finding F1 --stub-writer src/Foo.java:41",
        ));
    }
    let tc = stage::tool_ctx(project, ctx, data);
    tc.require_tier(1, "dla intervene")?;
    tc.require_time("intervene")?;
    let src = tc.harness(&a.harness)?;
    let mut ledger = tc.ledger()?;
    let f = ledger.finding(&a.finding).cloned().ok_or_else(|| {
        Failure::refuse(
            dla_core::error::codes::UNKNOWN_FINDING,
            format!("{} is not in the ledger", a.finding),
            "an existing finding for --finding",
            "dla run status   (lists findings)",
        )
    })?;
    if f.polarity() != Polarity::Leak {
        return Err(Failure::refuse(
            codes::LEAK_ONLY,
            format!(
                "{} is a non-leak hypothesis; an intervention tests a leak-side finding",
                f.id
            ),
            "a leak-side finding",
            "dla run status",
        ));
    }
    if f.state != FindingState::Confirmed {
        return Err(Failure::refuse(
            codes::STATE,
            format!(
                "{} is {}; an intervention needs a confirmed finding",
                f.id, f.state
            ),
            format!("{} confirmed", f.id),
            format!(
                "dla ledger confirm {} --evidence <growth evidence id>",
                f.id
            ),
        ));
    }
    let ctl = find_control(&tc, &ledger, &f, &a.harness)?;
    let anchors = anchors(ctx, project, &ledger, &f);
    let src_dir = dla_workload::harness::harness_dir(&tc.run_dir, &src.id);
    let owned = plan::harness_owned_names(&src_dir);
    let work = src_dir.join(dla_workload::harness::WORK);

    // Plan against the harness's own copy; nothing is cloned if this refuses.
    let edit = match &a.patch {
        Some(p) => {
            let text = fs::read_to_string(p).map_err(|e| dla_core::Error::io(p, e))?;
            plan::patch_plan(&work, &owned, &anchors, &text)?
        }
        None => plan::stub_plan(&work, &owned, &anchors, &a.stub_writers)?,
    };

    // Raw directory first: the diff is kept whatever the run does.
    let (n, raw_dir) = reserve_numbered(&tc.run_dir.join(INTERVENE_DIR), 'I', "", true)
        .map_err(|e| e.to_refusal())?;
    let raw_rel = format!("{INTERVENE_DIR}/I{n}");
    let diff_path = raw_dir.join("patch.diff");
    write_atomic(&diff_path, edit.diff.as_bytes())?;
    let diff_sha = sha_hex(&[edit.diff.as_bytes()]);

    // Clone, then the unedited run (only with --rerun-control), then the edit.
    let cloned = clone::clone_harness(&tc, &src)?;
    let th = ctl.report.thresholds;
    let mode = ctl
        .series
        .source
        .mode
        .or_else(|| src.key_modes.first().copied())
        .unwrap_or(KeyMode::Native);
    let ctl_view = view(&ctl.column);
    let mut limitations: Vec<String> = vec![
        SCRATCH_NOTE.to_owned(),
        "gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders".to_owned(),
        "exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked".to_owned(),
    ];
    if matches!(
        src.runtime,
        dla_core::runtime::Runtime::Native | dla_core::runtime::Runtime::Rust
    ) {
        limitations.push(
            "native and Rust harnesses have no holder probes: the column is a site group from the allocator shim or an os column, and an edit elsewhere may not move os columns past the gate"
                .to_owned(),
        );
    }

    let mut reproduced: Option<bool> = None;
    let mut rerun_json = Value::Null;
    if a.rerun_control {
        match run_clone(&tc, &cloned.manifest, &ctl, &th, mode)? {
            Ran::Ok(m) => {
                let col = m.trend.iter().find(|c| c.name == ctl.column.name);
                let ok = m.run_ok
                    && col.is_some_and(|c| {
                        c.gate == GateVerdict::Pass
                            && c.sen
                                .zip(ctl_view.slope)
                                .is_some_and(|(s, cs)| s.slope >= REMAINS_FRACTION * cs)
                    });
                reproduced = Some(ok);
                rerun_json = column_json(&m.series.id, "", &ctl.column.name, col);
            }
            Ran::Failed { why, .. } => {
                reproduced = Some(false);
                limitations.push(format!("the unedited rerun failed: {}", clip(&why, 120)));
            }
        }
    }

    plan::apply_plan(&cloned.work, &edit)?;
    // The build fingerprint does not cover `work/`: force the rebuild.
    let marker =
        dla_workload::harness::harness_dir(&tc.run_dir, &cloned.manifest.id).join(BUILD_MARKER);
    if marker.exists() {
        fs::remove_file(&marker).map_err(|e| dla_core::Error::io(&marker, e))?;
    }

    let ran = run_clone(&tc, &cloned.manifest, &ctl, &th, mode)?;
    let mut guards = Guards {
        build_ok: true,
        run_ok: true,
        same_points: true,
        edit_scoped: true,
        exercised: None,
        positive_control: None,
        control_reproduced: reproduced,
    };
    let (effect, treated_json, treated_series, source_rel);
    match ran {
        Ran::Failed { build, why } => {
            guards.build_ok = !build;
            guards.run_ok = false;
            guards.same_points = false;
            limitations.push(clip(&why, 200));
            effect = Effect::Invalid;
            treated_json =
                json!({"series": null, "trend": null, "column": ctl.column.name, "present": false});
            treated_series = None;
            source_rel = format!("{raw_rel}/patch.diff");
        }
        Ran::Ok(m) => {
            guards.run_ok = m.run_ok;
            guards.same_points = same_points(&ctl.series, &m.series);
            let others = other_passing(&ctl, &f);
            if !others.is_empty() {
                guards.positive_control = Some(
                    others
                        .iter()
                        .any(|o| m.trend.iter().any(|t| t.name == o.name && present(t))),
                );
            }
            let col = m
                .trend
                .iter()
                .find(|c| c.name == ctl.column.name && present(c));
            let other_columns = m
                .trend
                .iter()
                .any(|c| c.name != ctl.column.name && present(c));
            effect = judge_intervention(
                &ctl_view,
                &Treated {
                    column: col.map(view),
                    other_columns,
                },
                &guards,
            );
            // The treated trend: a trend report like any other, plus a copy in the raw dir.
            let tid = tc.next_trend_id();
            let report = TrendReport {
                schema_version: dla_core::trend::TREND_SCHEMA_VERSION,
                id: tid.clone(),
                series: m.series.id.clone(),
                thresholds: th,
                columns: m.trend.clone(),
            };
            report.save(&tc.run_dir)?;
            write_atomic(
                &raw_dir.join("trend.json"),
                to_pretty_json(&report).as_bytes(),
            )?;
            treated_json = column_json(&m.series.id, &tid, &ctl.column.name, col);
            limitations.extend(m.limitations.iter().take(2).map(|l| clip(l, 160)));
            source_rel = format!("series/{}.json", m.series.id);
            treated_series = Some(m.series.id.clone());
        }
    }

    let series_bytes = treated_series
        .as_ref()
        .and_then(|id| fs::read(SeriesFile::path_in(&tc.run_dir, id)).ok())
        .unwrap_or_default();
    let signal = Signal::new(
        "dla-intervene",
        tool_class::INTERVENTION,
        &sha_hex(&[edit.diff.as_bytes(), &series_bytes]),
    );
    let control_trend = ctl.report.id.clone();
    let mut guard_json = serde_json::to_value(guards).unwrap_or(Value::Null);
    if let Some(o) = guard_json.as_object_mut()
        && o.get("control_reproduced") == Some(&Value::Null)
    {
        o.remove("control_reproduced");
    }
    let mut result = json!({
        "test": "intervention",
        "kind": edit.kind,
        "finding": f.id.to_string(),
        "harness": a.harness,
        "treated_harness": cloned.manifest.id,
        "sites": edit.sites,
        "diff_sha256": diff_sha,
        "control": column_json(&ctl.series.id, &control_trend, &ctl.column.name, Some(&ctl.column)),
        "treated": treated_json,
        "effect": effect.as_str(),
        "guards": guard_json,
        "signal": signal.to_value(),
    });
    if !rerun_json.is_null() {
        result["rerun"] = rerun_json;
    }
    let result = fit(result);

    let mut spec = EvidenceSpec::new(
        EvidenceKind::Intervention,
        "dla",
        env!("CARGO_PKG_VERSION"),
        result,
    );
    spec.authority = Authority::Test;
    spec.raw = Some(raw_rel.clone());
    spec.source = Some(tc.artifact_source(&source_rel)?);
    spec.limitations = limitations.clone();
    let evidence = tc.add_evidence(&mut ledger, spec)?;

    let mut out = json!({
        "ok": true,
        "command": "intervene",
        "finding": f.id.to_string(),
        "kind": edit.kind,
        "harness": a.harness,
        "treated_harness": cloned.manifest.id,
        "evidence": evidence.to_string(),
        "effect": effect.as_str(),
        "guards": guard_json,
        "control": column_json(&ctl.series.id, &control_trend, &ctl.column.name, Some(&ctl.column)),
        "treated": treated_json,
        "sites": edit.sites,
        "changed_lines": edit.changed_lines,
        "cloned_files": cloned.files,
        "diff_sha256": diff_sha,
        "artifacts": [tc.pointer(&raw_rel)],
        "note": SCRATCH_NOTE,
        "limitations": limitations.iter().take(4).collect::<Vec<_>>(),
        "next": next_commands(&f, &ctl, &a, effect, &evidence, &guard_json, &edit),
    });
    if !out["treated"]["present"].as_bool().unwrap_or(true) && effect == Effect::GrowthRemoved {
        out["why"] = json!("the treated series has no such column: the edit removed what wrote it");
    }
    Ok(out)
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_owned()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}...")
    }
}

fn next_commands(
    f: &Finding,
    ctl: &Control,
    a: &InterveneArgs,
    effect: Effect,
    evidence: &EvidenceId,
    guards: &Value,
    edit: &plan::Plan,
) -> Vec<String> {
    match effect {
        Effect::GrowthRemoved => vec![format!("dla ledger strong {} --evidence {evidence}", f.id)],
        Effect::GrowthRemains => vec![
            format!(
                "dla ledger annotate {} --writer <file:line>   (another site adds to the holder, or this one is not the cause)",
                f.id
            ),
            format!(
                "dla intervene {} --finding {} --stub-writer <file:line> ...   (stub every writer in one run)",
                a.harness, f.id
            ),
            "growth-remains does not refute the finding".to_owned(),
        ],
        Effect::Inconclusive => vec![format!(
            "dla intervene {} --finding {} --patch <fix.diff>   (a stronger edit), or leave {} confirmed",
            a.harness, f.id, f.id
        )],
        Effect::Invalid => {
            let failed: Vec<&str> = guards
                .as_object()
                .into_iter()
                .flatten()
                .filter(|(_, v)| v.as_bool() == Some(false))
                .map(|(k, _)| k.as_str())
                .collect();
            vec![format!(
                "guard(s) false: {}; read the logs under harness/<treated>/logs, change the edit ({} line(s) changed) and run dla intervene again; {} stays confirmed (control {} in {})",
                failed.join(", "),
                edit.changed_lines,
                f.id,
                ctl.column.name,
                ctl.growth
            )]
        }
    }
}
