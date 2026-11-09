//! A growth series from captures: the series file, the dump-series rule and one
//! `growth-series` evidence whose passing columns are tied to findings.
//!
//! Used by the diff operations (type histograms, allocation stacks, non-heap counters).
//! The rule is the heap-dump one (`rule::gate`): at least three captures, every figure
//! strictly increasing across every pair, the last interval at least half of the first.
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

use super::{Ctx, Recorded, emit, rule};
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
}

impl Track {
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
    /// Passing column names, tied ones first.
    pub passed: Vec<String>,
    /// Why nothing passes.
    #[allow(dead_code)]
    pub reasons: Vec<String>,
    /// The compact JSON for the command's output.
    pub json: Value,
}

fn mean_delta(v: &[u64]) -> u64 {
    if v.is_empty() {
        0
    } else {
        v.iter().sum::<u64>() / v.len() as u64
    }
}

/// Writes the series and the growth evidence for `plan`. Needs at least two captures.
pub fn write(ctx: &Ctx<'_>, ledger: &mut Ledger, mut plan: Plan<'_>) -> Res<Growth> {
    let n = plan.t_ms.len();
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
            x: (i + 1) as f64,
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
            format!(
                "x is the {} number in capture order; t_ms is the time since the first capture where the inputs record one",
                plan.what
            ),
            format!(
                "each point comes from a separate {} of one process; a figure absent from a {} has no value at that point",
                plan.what, plan.what
            ),
        ],
    };
    series.save(&tc.run_dir)?;
    let series_rel = format!("series/{id}.json");

    // The rule.
    let mut passed: Vec<usize> = Vec::new();
    let mut first_failure: Option<String> = None;
    let mut shapes = Vec::new();
    let mut slopes = Vec::new();
    for (i, t) in plan.tracks.iter().enumerate() {
        match rule::gate(&t.observed()) {
            Ok(g) => {
                passed.push(i);
                slopes.push(mean_delta(&g.byte_deltas));
                shapes.push(rule::shape(&g));
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
            reasons.push(f);
        }
    }
    let passed_json: Vec<Value> = order
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
            if let Some(s) = &t.site {
                v["site"] = json!(s);
            }
            if let Some(f) = &t.finding {
                v["finding"] = json!(f);
            }
            if !t.candidates.is_empty() {
                v["candidates"] = json!(t.candidates);
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
        "failed": plan.tracks.len() - passed.len(),
        "reasons": reasons,
    });
    if passed.len() > SHOWN_PASSED {
        result["truncated"] = json!(passed.len() - SHOWN_PASSED);
    }
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
    spec.limitations.push(format!(
        "capture-series rule: at least {} captures, every figure strictly increasing across every pair, last interval at least {:.0}% of the first (provisional, not calibrated)",
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
    let names: Vec<String> = order
        .iter()
        .map(|&k| plan.tracks[passed[k]].name.clone())
        .collect();
    let json = json!({
        "series": id,
        "gate": if gate_pass { "pass" } else { "fail" },
        "growth_evidence": evidence.id,
        "passed": names.iter().take(SHOWN_PASSED).collect::<Vec<_>>(),
        "reasons": reasons,
    });
    Ok(Growth {
        series: id,
        evidence,
        pass: gate_pass,
        passed: names,
        reasons,
        json,
    })
}
