//! A growth series from captures: the series file, the dump-series rule and one
//! `growth-series` evidence whose passing columns are tied to findings.
//!
//! Used by the diff operations (type histograms, allocation stacks, non-heap counters).
//! The rule is the heap-dump one (`rule::gate`): at least three captures, every figure
//! strictly increasing across every pair (a count may instead be the same positive number at
//! every capture: one block that is resized), the last interval at least half of the first.
//! A column that passes is listed with the finding it is tied to; a column with no tie is
//! listed without one, and the ledger will not accept a confirmation that rests on it.

use dla_core::{
    FindingId, Ledger,
    evidence::{Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal},
    runtime::Runtime,
    series::{
        Axis, Column, ColumnKind, ColumnScope, Point, SeriesFile, SeriesRegion, SeriesSource,
        SeriesSourceKind, Unit,
    },
    toolctx::EvidenceSpec,
};
use serde_json::{Value, json};

use super::{Ctx, Recorded, emit, rule, tie::TieReason};
use crate::fail::Res;

/// Passing columns named in the evidence result.
const SHOWN_PASSED: usize = 4;
/// Columns a series file holds.
const MAX_COLUMNS: usize = 64;

/// One thing followed across captures.
#[derive(Debug, Clone)]
pub struct Track {
    /// Column name, e.g. `type:java.lang.String`.
    pub name: String,
    /// Unit of `values`.
    pub unit: Unit,
    /// The main figure per capture; `None` where the capture has none.
    pub values: Vec<Option<u64>>,
    /// A second figure (object count) per capture, written as `<name>.count`.
    pub counts: Option<Vec<Option<u64>>>,
    /// `file:line` the column is tied to.
    pub site: Option<String>,
    /// The finding the column belongs to, when that is certain.
    pub finding: Option<FindingId>,
    /// Findings the column may belong to.
    pub candidates: Vec<FindingId>,
    /// Memory region of the column.
    pub region: SeriesRegion,
    /// Why the column is tied to `finding` (rule and matched key); set with `finding`.
    pub tie: Option<TieReason>,
    /// Findings the column matches by the tie rules without being one: no tie is made.
    pub ambiguous: Vec<FindingId>,
}

impl Track {
    /// Ties the column to `f` for `why`.
    pub fn tie_to(&mut self, f: FindingId, why: TieReason, site: Option<String>) {
        self.tie_to_all(f, &[], why, site);
    }

    /// Ties the column to `f` and to every finding in `also` (all at the same exact
    /// identity), for `why`.
    pub fn tie_to_all(
        &mut self,
        f: FindingId,
        also: &[FindingId],
        why: TieReason,
        site: Option<String>,
    ) {
        self.candidates = std::iter::once(f.clone())
            .chain(also.iter().cloned())
            .collect();
        self.finding = Some(f);
        self.tie = Some(why);
        self.site = site;
    }

    fn observed(&self) -> rule::Observed {
        rule::Observed {
            bytes: self.values.clone(),
            counts: self.counts.clone().unwrap_or_else(|| self.values.clone()),
        }
    }

    fn last(&self) -> u64 {
        self.values
            .iter()
            .rev()
            .flatten()
            .next()
            .copied()
            .unwrap_or(0)
    }

    fn tied(&self) -> bool {
        self.finding.is_some() || !self.candidates.is_empty()
    }
}

/// What to write.
pub struct Plan<'a> {
    /// Milliseconds since the first capture, per capture.
    pub t_ms: Vec<u64>,
    /// The things followed.
    pub tracks: Vec<Track>,
    /// The signal the evidence carries.
    pub signal: Signal,
    /// Tool name of the evidence record.
    pub tool: &'a str,
    /// Authority of the evidence record.
    pub authority: Authority,
    /// The command that made the series.
    pub command: String,
    /// Series source kind.
    pub source_kind: SeriesSourceKind,
    /// Runtime of the measured process, when known.
    pub runtime: Option<Runtime>,
    /// How the figures were obtained, for the evidence limitations.
    pub limitations: Vec<String>,
    /// What a point is (`capture`).
    pub what: &'a str,
    /// The finding `--finding` names: its tied columns are shown first.
    pub focus: Option<FindingId>,
}

/// The outcome.
pub struct Growth {
    /// The series id (`S3`).
    #[allow(dead_code)]
    pub series: String,
    /// The growth evidence.
    pub evidence: Recorded,
    /// True when at least one column passes the rule.
    pub pass: bool,
    /// Column names written into the evidence result.
    #[allow(dead_code)]
    pub shown: Vec<String>,
    /// Shown columns that are tied, with their finding. Only these may be named as tied.
    pub tied: Vec<(String, FindingId)>,
    /// Why nothing passes.
    #[allow(dead_code)]
    pub reasons: Vec<String>,
    /// The site of the largest shown passing column that has one: the site `dla ledger add
    /// --origin growth-series --evidence <this record>` gives the finding it makes.
    pub site: Option<String>,
    /// The harness every capture of the call lies under: what the series is about.
    pub harness: Option<String>,
    /// The compact JSON for the command's output.
    pub json: Value,
}

/// The limitation lines of the series: what x is, and how the points were made.
fn axis_of(allocated: bool, work: bool) -> Axis {
    if allocated {
        Axis::AllocatedBytes
    } else if work {
        Axis::Iteration
    } else {
        Axis::Dump
    }
}

fn axis_limitations(
    plan: &Plan<'_>,
    allocated: bool,
    work: bool,
    fallback: Option<&str>,
) -> Vec<String> {
    let mut out = Vec::new();
    if work {
        out.push(format!(
            "x is the units of work the process had done when the {} was taken (the run's own x), not the {} number: the gate compares the rates over the x intervals; t_ms is the time since the first {} where the inputs record one",
            plan.what, plan.what, plan.what
        ));
    } else if allocated {
        out.push(format!(
            "x is the number of bytes the process had allocated in total when the {} was taken (the sum of alloc_space over its stacks), not the {} number: it is proportional to the work done at that point; t_ms is the time since the first {} where the inputs record one",
            plan.what, plan.what, plan.what
        ));
    } else {
        out.push(format!(
            "x is the {} number in capture order; t_ms is the time since the first {} where the inputs record one",
            plan.what, plan.what
        ));
    }
    out.push(format!(
        "each point comes from a separate {} of one process; a figure absent from a {} has no value at that point",
        plan.what, plan.what
    ));
    if let Some(why) = fallback {
        out.push(why.to_owned());
    }
    out
}

/// Increase of the last present figure over the first, per unit of x, as a JSON number
/// (4 significant digits); `null` when a figure is missing.
fn rate(values: &[Option<u64>], xs: &[f64]) -> Value {
    let (Some(Some(a)), Some(Some(b))) = (values.first(), values.last()) else {
        return Value::Null;
    };
    let span = xs[xs.len() - 1] - xs[0];
    let r = (*b as f64 - *a as f64) / span;
    format!("{r:.3e}")
        .parse::<f64>()
        .ok()
        .map_or(Value::Null, |r| json!(r))
}

fn mean_delta(v: &[u64]) -> u64 {
    if v.is_empty() {
        0
    } else {
        v.iter().sum::<u64>() / v.len() as u64
    }
}

/// Writes the series and the growth evidence for `plan`, one capture per point and the
/// capture number as x. Needs at least two captures.
pub fn write(ctx: &Ctx<'_>, ledger: &mut Ledger, plan: Plan<'_>) -> Res<Growth> {
    write_at(ctx, ledger, plan, &XAxis::default())
}

/// What x is for the points of a capture series.
#[derive(Debug, Clone, Default)]
pub struct XAxis {
    /// The bytes the process had allocated when each capture was taken (strictly
    /// increasing, one per capture); `None` makes x the capture number.
    pub allocated: Option<Vec<f64>>,
    /// The units of work the process had done when each capture was taken (strictly
    /// increasing, one per capture; the run's own `x`); used when `allocated` is `None`.
    pub work: Option<Vec<f64>>,
    /// Why x is the capture number where an allocated-bytes axis was wanted; written as
    /// a limitation of the series and of the evidence.
    pub fallback: Option<String>,
    /// A line for the limitations of the series and of the evidence (the order the
    /// captures were put in).
    pub note: Option<String>,
}

/// [`write`] with the x axis chosen by the caller. With allocated-bytes positions the gate
/// compares rates over the x intervals and the slope shown is bytes per allocated byte.
pub fn write_at(
    ctx: &Ctx<'_>,
    ledger: &mut Ledger,
    mut plan: Plan<'_>,
    xaxis: &XAxis,
) -> Res<Growth> {
    let n = plan.t_ms.len();
    let allocated: Option<&[f64]> = xaxis.allocated.as_deref().filter(|x| x.len() == n);
    let work: Option<&[f64]> = xaxis
        .work
        .as_deref()
        .filter(|x| x.len() == n && allocated.is_none());
    let xs: Option<&[f64]> = allocated.or(work);
    // Keep the column count within the series cap.
    let mut cols = 0;
    plan.tracks.retain(|t| {
        let c = 1 + usize::from(t.counts.is_some());
        if cols + c > MAX_COLUMNS {
            false
        } else {
            cols += c;
            true
        }
    });

    let tc = ctx.tc;
    let id = tc.next_series_id();
    let mut columns = Vec::new();
    for t in &plan.tracks {
        let kind = if t.unit == Unit::Bytes {
            ColumnKind::Gauge
        } else {
            ColumnKind::Counter
        };
        columns.push(Column {
            name: t.name.clone(),
            unit: t.unit,
            kind,
            region: t.region,
            baseline: false,
            after_gc: false,
            scope: ColumnScope::Process,
            site: t.site.clone(),
            finding: t.finding.clone(),
            source: plan.tool.to_owned(),
        });
        if t.counts.is_some() {
            columns.push(Column {
                name: format!("{}.count", t.name),
                unit: Unit::Count,
                kind: ColumnKind::Counter,
                region: t.region,
                baseline: false,
                after_gc: false,
                scope: ColumnScope::Process,
                site: t.site.clone(),
                finding: t.finding.clone(),
                source: plan.tool.to_owned(),
            });
        }
    }
    let points: Vec<Point> = (0..n)
        .map(|i| Point {
            x: xs.map_or((i + 1) as f64, |x| x[i]),
            t_ms: plan.t_ms[i],
            values: plan
                .tracks
                .iter()
                .flat_map(|t| {
                    let mut v = vec![t.values[i].map(|x| x as f64)];
                    if let Some(c) = &t.counts {
                        v.push(c[i].map(|x| x as f64));
                    }
                    v
                })
                .collect(),
        })
        .collect();
    let mut source = SeriesSource::new(plan.source_kind);
    source.command = Some(plan.command.clone());
    source.runtime = plan.runtime;
    // Captures of one harness: the series is about that harness (an alloc-test of it can decide a
    // finding that cites the series), though no run of the harness wrote it.
    let source_harness = (plan.source_kind == SeriesSourceKind::Dump)
        .then(|| ctx.capture_harness().map(str::to_owned))
        .flatten();
    source.harness.clone_from(&source_harness);
    let series = SeriesFile {
        schema_version: 1,
        id: id.clone(),
        source,
        axis: axis_of(allocated.is_some(), work.is_some()),
        warmup: 0,
        columns,
        points,
        events: Vec::new(),
        limitations: axis_limitations(
            &plan,
            allocated.is_some(),
            work.is_some(),
            xaxis.fallback.as_deref().or(xaxis.note.as_deref()),
        ),
    };
    series.save(&tc.run_dir)?;
    let series_rel = format!("series/{id}.json");

    // The rule.
    let mut passed: Vec<usize> = Vec::new();
    let mut first_failure: Option<String> = None;
    let mut shapes = Vec::new();
    let mut resized = Vec::new();
    let mut slopes = Vec::new();
    for (i, t) in plan.tracks.iter().enumerate() {
        match rule::gate_at(&t.observed(), xs) {
            Ok(g) => {
                passed.push(i);
                slopes.push(match xs {
                    // Bytes kept per byte allocated, first capture to last.
                    Some(x) => rate(&t.values, x),
                    None => json!(mean_delta(&g.byte_deltas)),
                });
                shapes.push(rule::shape(&g));
                resized.push(g.resized);
            }
            Err(why) => {
                if first_failure.is_none() && n >= rule::MIN_DUMPS {
                    first_failure = Some(format!("{}: {}", t.name, why.join("; ")));
                }
            }
        }
    }
    let mut order: Vec<usize> = (0..passed.len()).collect();
    order.sort_by_key(|&k| {
        let t = &plan.tracks[passed[k]];
        (!t.tied(), std::cmp::Reverse(t.last()))
    });
    // Which columns the bounded result shows: the ones tied to `--finding` first, then
    // the best column of each other tied finding, then the rest. A tie is never shown
    // without its column, so what the output names as tied is what the ledger can read.
    let focus = plan.focus.as_ref();
    let mut priority: Vec<usize> = Vec::new();
    let mut covered: Vec<&FindingId> = Vec::new();
    for &k in &order {
        let t = &plan.tracks[passed[k]];
        if focus.is_some() && t.finding.as_ref() == focus {
            priority.push(k);
        }
    }
    if let Some(f) = focus {
        covered.push(f);
    }
    for &k in &order {
        let t = &plan.tracks[passed[k]];
        if let Some(f) = &t.finding
            && !covered.contains(&f)
        {
            covered.push(f);
            priority.push(k);
        }
    }
    for &k in &order {
        if !priority.contains(&k) {
            priority.push(k);
        }
    }
    let gate_pass = !passed.is_empty();
    let mut reasons: Vec<String> = Vec::new();
    if n < rule::MIN_DUMPS {
        reasons.push(format!(
            "{} < {}",
            plan.what.to_owned() + "s",
            rule::MIN_DUMPS
        ));
    } else if !gate_pass {
        reasons.push(format!(
            "nothing grows strictly across all {n} {}s",
            plan.what
        ));
        if let Some(f) = first_failure {
            reasons.push(clip(&f, 160));
        }
    }
    let rows: Vec<Value> = priority
        .iter()
        .take(SHOWN_PASSED)
        .map(|&k| {
            let t = &plan.tracks[passed[k]];
            let mut v = json!({
                "column": t.name,
                "region": t.region,
                "unit": t.unit,
                "slope": slopes[k],
                "shape": shapes[k],
            });
            if resized[k] {
                // One block (or a fixed set of blocks) whose bytes grew: a buffer that is resized.
                v["resized"] = json!(true);
            }
            if let Some(s) = &t.site {
                v["site"] = json!(clip(s, 160));
            }
            if let Some(f) = &t.finding {
                v["finding"] = json!(f);
                v["candidates"] = json!(t.candidates);
                if let Some(why) = &t.tie {
                    v["tie"] = why.to_json();
                }
            }
            v
        })
        .collect();
    let base = json!({
        "series": id,
        "axis": axis_of(allocated.is_some(), work.is_some()),
        "n": n,
        "gate": if gate_pass { "pass" } else { "fail" },
        "failed": plan.tracks.len() - passed.len(),
        "reasons": reasons,
    });
    let (result, shown_n) = fit_passed(base, rows, passed.len());
    let shown_idx: Vec<usize> = priority.iter().take(shown_n).map(|&k| passed[k]).collect();
    let mut spec = EvidenceSpec::new(
        EvidenceKind::GrowthSeries,
        plan.tool,
        env!("CARGO_PKG_VERSION"),
        result,
    );
    spec.authority = plan.authority;
    spec.source = Some(tc.artifact_source(&series_rel)?);
    spec.coverage = EvidenceCoverage {
        status: CoverageStatus::Complete,
        examined: Some(n as u64),
        total: Some(n as u64),
    };
    spec.limitations = plan.limitations.clone();
    if let Some(why) = xaxis.fallback.as_ref().or(xaxis.note.as_ref()) {
        spec.limitations.push(why.clone());
    }
    spec.limitations.push(format!(
        "capture-series rule: at least {} captures, every figure strictly increasing across every pair (a count may instead be one constant number of blocks whose bytes grow), last interval at least {:.0}% of the first (provisional, not calibrated)",
        rule::MIN_DUMPS,
        rule::LAST_OVER_FIRST * 100.0
    ));
    let raw = emit::RawRef {
        rel: series_rel.clone(),
        sha256: match &spec.source {
            Some(dla_core::evidence::Source::Artifact { sha256, .. }) => sha256.clone(),
            _ => String::new(),
        },
    };
    let evidence = ctx.record(ledger, spec, &plan.signal, &raw)?;
    let shown: Vec<String> = shown_idx
        .iter()
        .map(|&i| plan.tracks[i].name.clone())
        .collect();
    // What `dla ledger add --origin growth-series --evidence <this record>` takes as the site:
    // the shown passing column with the largest slope that has one (its rule, read back).
    let site = priority
        .iter()
        .take(shown_idx.len())
        .filter_map(|&k| {
            let size = slopes[k].as_f64().map_or(0.0, f64::abs);
            plan.tracks[passed[k]].site.as_ref().map(|s| (size, s))
        })
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, s)| clip(s, 160));
    let tied: Vec<(String, FindingId)> = shown_idx
        .iter()
        .filter_map(|&i| {
            let t = &plan.tracks[i];
            t.finding.clone().map(|_| t)
        })
        .flat_map(|t| t.candidates.iter().map(|f| (t.name.clone(), f.clone())))
        .collect();
    let json = json!({
        "series": id,
        "gate": if gate_pass { "pass" } else { "fail" },
        "growth_evidence": evidence.id,
        "passed": shown,
        "reasons": reasons,
    });
    Ok(Growth {
        series: id,
        evidence,
        pass: gate_pass,
        shown,
        tied,
        reasons,
        site,
        harness: source_harness,
        json,
    })
}

/// The `needed` text of a summary whose growth passed its gate and ties to no finding: the
/// command that makes the finding the growth stands on (its site is the largest passing column's),
/// then the alloc-test that decides it. `why` is the summary's own reading of why nothing tied.
pub fn untied_needed(g: &Growth, why: &str) -> String {
    let site = match &g.site {
        Some(s) => format!("the finding takes the site of the largest passing column, {s}"),
        None => "no passing column has a site: add --site <file:line> and --owner <the holder>"
            .to_owned(),
    };
    format!(
        "the growth passed its gate and ties to no finding: dla ledger add --origin growth-series --evidence {}   ({site}); then dla alloc-test {} on that finding (the final gate and the refusal to close it print the command); {why}",
        g.evidence.id,
        g.harness
            .as_deref()
            .unwrap_or("<the harness the captures came from>"),
    )
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

/// Adds `rows` as `passed` to the result `base` (an object without it), as many as keep
/// the result within the evidence budget, and `truncated` for the passing columns left
/// out (`total` is how many passed). Returns the result and the number of rows kept.
pub fn fit_passed(base: Value, rows: Vec<Value>, total: usize) -> (Value, usize) {
    for keep in (0..=rows.len()).rev() {
        let mut r = base.clone();
        r["passed"] = Value::Array(rows[..keep].to_vec());
        if total > keep {
            r["truncated"] = json!(total - keep);
        }
        if keep == 0 || dla_core::evidence::result_size(&r) <= emit::RESULT_BUDGET {
            return (r, keep);
        }
    }
    unreachable!("keep = 0 always returns")
}

/// The findings the shown, tied columns belong to, in column order; with `only`, just
/// that finding when a shown column is tied to it.
pub fn tied_ids(g: &Growth, only: Option<&FindingId>) -> Vec<FindingId> {
    let mut out: Vec<FindingId> = Vec::new();
    for (_, f) in &g.tied {
        if only.is_none_or(|o| o == f) && !out.contains(f) {
            out.push(f.clone());
        }
    }
    out
}

/// `{column: [findings]}` for columns that match several findings (at most 3 columns of
/// 4 findings each); `None` when there are none.
pub fn ambiguous_json(items: &[(String, Vec<FindingId>)]) -> Option<Value> {
    if items.is_empty() {
        return None;
    }
    let mut m = serde_json::Map::new();
    for (name, ids) in items.iter().take(3) {
        m.insert(
            clip(name, 60),
            json!(ids.iter().take(4).collect::<Vec<_>>()),
        );
    }
    Some(Value::Object(m))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holder_kind_survives_result_bounding() {
        let rows: Vec<Value> = (0..12)
            .map(|i| {
                json!({
                    "column": format!("holder:static:{}", "src.module.Name".repeat(6) + &i.to_string()),
                    "region": "heap",
                    "unit": "bytes",
                    "slope": 123456,
                    "shape": "linear",
                    "finding": format!("F{i}"),
                    "candidates": [format!("F{i}")],
                    "holder_kind": "native-handle/global-ref",
                })
            })
            .collect();
        let base = json!({"series": "S1", "axis": "n", "n": 3, "gate": "pass", "failed": 0, "reasons": []});
        let (r, kept) = fit_passed(base, rows, 12);
        assert!(kept > 0 && kept < 12, "kept {kept}");
        assert!(dla_core::evidence::result_size(&r) <= emit::RESULT_BUDGET);
        let passed = r["passed"].as_array().unwrap();
        assert_eq!(passed.len(), kept);
        assert!(
            passed
                .iter()
                .all(|c| c["holder_kind"] == "native-handle/global-ref"),
            "{r}"
        );
        assert_eq!(r["truncated"], 12 - kept);
    }
}
