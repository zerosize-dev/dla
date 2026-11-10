//! `dla gap <S> --residual`: growth per region that no confirmed finding explains.

use std::collections::{BTreeMap, HashSet};

use dla_core::{
    EvidenceId, FindingId, FindingState, Ledger, OpenItemId, Resolution, RunState,
    coverage::{CoverageManifest, ResidualRow},
    evidence::{CoverageStatus, EvidenceCoverage, EvidenceKind},
    finding::Region,
    fsutil::{to_pretty_json, write_atomic},
    series::{Axis, SeriesFile, Unit, column_meta},
    toolctx::{EvidenceSpec, ToolCtx},
    trend::{GrowthSummary, finding_region},
};
use serde_json::{Value, json};

use super::{fit, series_authority};
use crate::fail::{Failure, Res};

/// `opened_by` of the open items this command raises.
pub const OPENED_BY: &str = "dla gap --residual";

/// Heap columns in order of preference.
const HEAP: [&str; 5] = [
    "jvm.heap_used",
    "go.heap_live",
    "node.heap_used",
    "py.traced",
    "native.live_bytes",
];
/// Non-heap byte columns in order of preference.
const NON_HEAP: [&str; 2] = ["jvm.nonheap_used", "go.stacks"];

/// One byte column credited to a finding.
#[derive(Debug, Clone)]
pub struct Credit {
    /// The finding.
    pub finding: FindingId,
    /// The column.
    pub column: String,
    /// Its slope.
    pub slope: f64,
}

/// The residual arithmetic of one region.
#[derive(Debug, Clone)]
pub struct RegionCalc {
    /// The region.
    pub region: Region,
    /// The primary column the total comes from.
    pub column: String,
    /// Total slope.
    pub total: f64,
    /// Attributed slope, capped at the total.
    pub attributed: f64,
    /// `max(0, total - attributed)`.
    pub residual: f64,
    /// Provisional threshold.
    pub threshold: f64,
    /// Columns credited to findings in this region.
    pub credits: Vec<Credit>,
}

impl RegionCalc {
    /// True when the residual must be explained.
    pub fn over(&self) -> bool {
        self.residual > 0.0 && self.residual > self.threshold
    }
}

/// The unit of a slope on `axis`.
pub fn unit_of(axis: Axis) -> String {
    format!(
        "bytes/{}",
        match axis {
            Axis::Iteration => "iteration",
            Axis::Seconds => "second",
            Axis::N => "n",
            Axis::Dump => "dump",
        }
    )
}

fn region_name(r: Region) -> &'static str {
    match r {
        Region::Heap => "heap",
        Region::OffHeap => "off-heap",
        Region::NonHeap => "non-heap",
    }
}

/// The series columns the totals come from.
struct Sources<'a> {
    series: &'a SeriesFile,
    /// The newest gap series derived from `series`.
    derived: Option<&'a SeriesFile>,
}

impl Sources<'_> {
    /// First preferred column of `names` with a fit.
    fn first_of(&self, names: &[&str]) -> Option<(String, fit::Fit)> {
        names.iter().find_map(|n| {
            let i = self.series.column_index(n)?;
            Some(((*n).to_owned(), fit::fit_column(self.series, i)?))
        })
    }

    fn primary(&self, region: Region) -> Option<(String, fit::Fit)> {
        match region {
            Region::Heap => self.first_of(&HEAP),
            Region::NonHeap => self.first_of(&NON_HEAP),
            Region::OffHeap => self
                .derived
                .and_then(|d| {
                    let i = d.column_index("gap.unexplained")?;
                    Some(("gap.unexplained".to_owned(), fit::fit_column(d, i)?))
                })
                .or_else(|| self.first_of(&["node.external"])),
        }
    }
}

/// What the leak-side evidence of confirmed and strong findings attributes.
struct Attribution {
    /// Credits by region.
    credits: BTreeMap<Region, Vec<Credit>>,
    limitations: Vec<String>,
}

fn attribute(ledger: &Ledger, series: &SeriesFile) -> Attribution {
    let mut credits: BTreeMap<Region, Vec<Credit>> = BTreeMap::new();
    let mut limitations = Vec::new();
    let mut counted: HashSet<String> = HashSet::new();
    for f in ledger
        .findings()
        .filter(|f| matches!(f.state, FindingState::Confirmed | FindingState::Strong))
    {
        let mut growth: Vec<GrowthSummary> = Vec::new();
        for e in f.supports.iter().filter_map(|id| ledger.evidence_by_id(id)) {
            if matches!(
                e.kind,
                EvidenceKind::GrowthSeries | EvidenceKind::UnitCountSeries
            ) && let Some(g) = GrowthSummary::from_result(&e.result)
            {
                growth.push(g);
            }
        }
        if growth.is_empty() {
            continue;
        }
        let here: Vec<&GrowthSummary> = growth
            .iter()
            .filter(|g| g.series.as_deref() == Some(series.id.as_str()))
            .collect();
        if here.is_empty() {
            let other = growth
                .iter()
                .filter_map(|g| g.series.as_deref())
                .next()
                .unwrap_or("another series");
            limitations.push(format!(
                "{}: growth evidence is from {other}, not {}; not subtracted",
                f.id, series.id
            ));
            continue;
        }
        let tied: Vec<_> = here
            .iter()
            .flat_map(|g| g.passed.iter())
            .filter(|c| c.tied_to_owner(&f.id, f.site.as_deref(), f.owner.as_deref()))
            .collect();
        if tied.is_empty() {
            limitations.push(format!(
                "{}: no passing column is tied to it; nothing attributed",
                f.id
            ));
            continue;
        }
        let mut any_bytes = false;
        for c in tied {
            let unit = c.unit.unwrap_or_else(|| column_meta(&c.column).unit);
            if unit != Unit::Bytes {
                continue;
            }
            any_bytes = true;
            let Some(region) = c.region.and_then(finding_region).or(f.region) else {
                limitations.push(format!(
                    "{}: column {} belongs to no region; not attributed",
                    f.id, c.column
                ));
                continue;
            };
            if !counted.insert(c.column.clone()) {
                continue;
            }
            let slope = series
                .column_index(&c.column)
                .and_then(|i| fit::fit_column(series, i))
                .map(|ft| ft.slope)
                .or(c.slope)
                .unwrap_or(0.0);
            if slope > 0.0 {
                credits.entry(region).or_default().push(Credit {
                    finding: f.id.clone(),
                    column: c.column.clone(),
                    slope,
                });
            }
        }
        if !any_bytes {
            limitations.push(format!(
                "{}: growth measured as a count; bytes not attributed",
                f.id
            ));
        }
    }
    Attribution {
        credits,
        limitations,
    }
}

/// The residual arithmetic for every region that has a primary column.
fn calculate(
    ledger: &Ledger,
    series: &SeriesFile,
    derived: Option<&SeriesFile>,
) -> (Vec<RegionCalc>, Vec<String>) {
    let src = Sources { series, derived };
    let mut attr = attribute(ledger, series);
    let mut rows = Vec::new();
    for region in [Region::Heap, Region::OffHeap, Region::NonHeap] {
        let Some((column, ft)) = src.primary(region) else {
            continue;
        };
        let credits = attr.credits.remove(&region).unwrap_or_default();
        let sum: f64 = credits.iter().map(|c| c.slope).sum();
        let attributed = sum.min(ft.slope).max(0.0);
        let noise_floor = if ft.span > 0.0 {
            3.0 * ft.noise / ft.span
        } else {
            0.0
        };
        rows.push(RegionCalc {
            region,
            column,
            total: ft.slope,
            attributed,
            residual: (ft.slope - attributed).max(0.0),
            threshold: (0.25 * ft.slope).max(noise_floor),
            credits,
        });
    }
    // Credits in a region without a primary column cannot be subtracted from anything.
    for (region, cs) in attr.credits {
        for c in cs {
            attr.limitations.push(format!(
                "{}: {} grows in {} but the series has no total for that region",
                c.finding,
                c.column,
                region_name(region)
            ));
        }
    }
    (rows, attr.limitations)
}

/// The region an open item of this command was raised for.
fn item_region(text: &str) -> Option<Region> {
    let head = text
        .strip_prefix("unexplained growth: ")?
        .split(';')
        .next()?;
    let (_, tail) = head.rsplit_once(" in ")?;
    let name = tail.split(" (").next()?;
    [Region::Heap, Region::OffHeap, Region::NonHeap]
        .into_iter()
        .find(|r| region_name(*r) == name)
}

/// The lowest still-open item this command raised for `region`.
fn open_item_for(ledger: &Ledger, region: Region) -> Option<OpenItemId> {
    ledger
        .open_items()
        .filter(|s| s.resolution.is_none() && s.item.opened_by == OPENED_BY)
        .find(|s| item_region(&s.item.text) == Some(region))
        .map(|s| s.item.id.clone())
}

/// The finding to cite when an open item of `calc`'s region is resolved: one raised from
/// a residual first, then the newest finding (the one that closed the gap most recently).
fn citing_finding(ledger: &Ledger, calc: &RegionCalc) -> Option<FindingId> {
    calc.credits
        .iter()
        .max_by(|a, b| {
            let from_residual = |c: &Credit| {
                ledger
                    .finding(&c.finding)
                    .is_some_and(|f| f.origin.as_str() == "residual")
            };
            from_residual(a)
                .cmp(&from_residual(b))
                .then(a.finding.number().cmp(&b.finding.number()))
        })
        .map(|c| c.finding.clone())
}

/// What happens to the open item of a region.
enum Action {
    None,
    Keep(OpenItemId),
    Open(OpenItemId),
    Resolve(OpenItemId, FindingId),
    LeaveOpen(OpenItemId),
}

impl Action {
    fn item(&self) -> Option<&OpenItemId> {
        match self {
            Action::Keep(i) | Action::Open(i) | Action::LeaveOpen(i) => Some(i),
            Action::None | Action::Resolve(..) => None,
        }
    }
}

fn row_json(c: &RegionCalc, unit: &str, open: Option<&OpenItemId>) -> Value {
    json!({
        "region": region_name(c.region),
        "total": fit::round(c.total),
        "attributed": fit::round(c.attributed),
        "residual": fit::round(c.residual),
        "threshold": fit::round(c.threshold),
        "unit": unit,
        "open_item": open.map(ToString::to_string),
    })
}

/// Runs the residual table on series `src`.
pub fn run(tc: &ToolCtx, src: &SeriesFile, derived: Option<&SeriesFile>) -> Res<Value> {
    RunState::load(&tc.run_dir)?;
    let mut ledger = tc.ledger()?;
    let (calcs, mut limitations) = calculate(&ledger, src, derived);
    if derived.is_none() {
        limitations.push(format!(
            "off-heap: no gap series for {}; run dla gap {} first (node.external is used when present)",
            src.id, src.id
        ));
    }
    if calcs.is_empty() {
        return Err(Failure::refuse(
            "residual-unavailable",
            format!(
                "series {} has no heap, off-heap or non-heap byte column with enough points to total",
                src.id
            ),
            format!(
                "a runtime heap column (jvm.heap_used, go.heap_live, ...) in {}",
                src.id
            ),
            format!(
                "dla harness run {}   (a runtime agent or driver writes them)",
                src.source.harness.as_deref().unwrap_or("<H>")
            ),
        ));
    }
    let unit = unit_of(src.axis);

    // Decide what happens to each region's open item.
    let mut next_open = ledger.next_open_item_id().number();
    let mut actions = Vec::new();
    for c in &calcs {
        let existing = open_item_for(&ledger, c.region);
        let action = match (c.over(), existing) {
            (true, Some(id)) => Action::Keep(id),
            (true, None) => {
                let id = OpenItemId::new(next_open);
                next_open += 1;
                Action::Open(id)
            }
            (false, Some(id)) => match citing_finding(&ledger, c) {
                Some(f) => Action::Resolve(id, f),
                None => {
                    limitations.push(format!(
                        "{id} ({}) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve {id} --as unresolved --reason time_budget --remedy \"<text>\"",
                        region_name(c.region)
                    ));
                    Action::LeaveOpen(id)
                }
            },
            (false, None) => Action::None,
        };
        actions.push(action);
    }

    // The full table goes to a file; the evidence carries the bounded rows.
    let ev_id: EvidenceId = ledger.next_evidence_id();
    let dir = tc.stage_dir("gap")?;
    // A worker's evidence id carries a `<worker>/` prefix, which is no file name part.
    let ev_file = ev_id.to_string().replace('/', "_");
    let file = format!("gap/residual-{}-{ev_file}.json", src.id);
    let full: Vec<Value> = calcs
        .iter()
        .zip(&actions)
        .map(|(c, a)| {
            let mut v = row_json(c, &unit, a.item());
            v["column"] = json!(c.column);
            v["credits"] = json!(
                c.credits
                    .iter()
                    .map(|k| json!({"finding": k.finding.to_string(), "column": k.column, "slope": k.slope}))
                    .collect::<Vec<_>>()
            );
            v
        })
        .collect();
    let doc = json!({"series": src.id, "derived": derived.map(|d| d.id.clone()),
        "calibrated": false, "rows": full, "limitations": limitations});
    write_atomic(
        &dir.join(format!("residual-{}-{ev_file}.json", src.id)),
        to_pretty_json(&doc).as_bytes(),
    )?;

    let rows: Vec<Value> = calcs
        .iter()
        .zip(&actions)
        .map(|(c, a)| row_json(c, &unit, a.item()))
        .collect();
    let mut spec = EvidenceSpec::new(
        EvidenceKind::Residual,
        "dla",
        env!("CARGO_PKG_VERSION"),
        json!({"series": src.id, "rows": rows, "calibrated": false}),
    );
    spec.authority = series_authority(src);
    spec.raw = Some(file.clone());
    spec.source = Some(tc.artifact_source(&format!("series/{}.json", src.id))?);
    spec.coverage = EvidenceCoverage {
        status: CoverageStatus::Partial,
        examined: Some(calcs.len() as u64),
        total: Some(3),
    };
    spec.limitations = limitations.iter().take(8).cloned().collect();
    spec.limitations
        .push("threshold is provisional (not calibrated)".to_owned());
    let ev = tc.add_evidence(&mut ledger, spec)?;
    if ev != ev_id {
        return Err(Failure::refuse(
            "ledger-race",
            format!(
                "the ledger changed while writing the residual evidence ({ev_id} expected, got {ev})"
            ),
            "an unchanged ledger",
            format!("dla gap {} --residual", src.id),
        ));
    }

    // Ledger moves, then the coverage rows.
    let (mut opened, mut kept, mut resolved) = (Vec::new(), Vec::new(), Vec::new());
    for (c, a) in calcs.iter().zip(&actions) {
        match a {
            Action::Open(id) => {
                let text = format!(
                    "unexplained growth: {} {unit} in {} ({}); add a hypothesis (dla ledger add --origin residual --evidence {ev} ...) or record it unresolved",
                    fit::human(c.residual),
                    region_name(c.region),
                    src.id
                );
                ledger.open_item_add(&text, None, OPENED_BY)?;
                opened.push(id.to_string());
            }
            Action::Keep(id) | Action::LeaveOpen(id) => kept.push(id.to_string()),
            Action::Resolve(id, f) => {
                ledger.resolve_open_item(id, Resolution::Hypothesis(f.clone()))?;
                resolved.push(json!({"item": id.to_string(), "by": f.to_string()}));
            }
            Action::None => {}
        }
    }
    let cov_rows: Vec<ResidualRow> = calcs
        .iter()
        .zip(&actions)
        .map(|(c, a)| ResidualRow {
            series: Some(src.id.clone()),
            region: c.region,
            total_slope: c.total,
            attributed_slope: c.attributed,
            residual: c.residual,
            unit: unit.clone(),
            threshold: c.threshold,
            open_item: a.item().cloned(),
        })
        .collect();
    CoverageManifest::update(&tc.run_dir, |m| m.set_residual(cov_rows))?;

    let mut next = Vec::new();
    for (c, a) in calcs.iter().zip(&actions) {
        if let Some(id) = a.item().filter(|_| c.over()) {
            next.push(format!(
                "{id}: dla ledger add --origin residual --evidence {ev} ...   or   dla ledger resolve {id} --as unresolved --reason <r> --remedy \"<text>\""
            ));
        }
    }
    if next.is_empty() {
        next.push("no region has unexplained growth over its threshold".to_owned());
    }
    let more = limitations.len().saturating_sub(5);
    limitations.truncate(5);
    if more > 0 {
        limitations.push(format!("+{more} more in {file}"));
    }
    Ok(json!({
        "ok": true,
        "command": "gap",
        "residual": true,
        "series": src.id,
        "evidence": [ev.to_string()],
        "rows": rows,
        "calibrated": false,
        "open_items": {"opened": opened, "kept": kept, "resolved": resolved},
        "artifacts": [tc.pointer(&file)],
        "limitations": limitations,
        "next": next,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_item_text_names_its_region() {
        let t = "unexplained growth: 412 bytes/iteration in off-heap (S3); add a hypothesis (dla ledger add --origin residual --evidence E7 ...) or record it unresolved";
        assert_eq!(item_region(t), Some(Region::OffHeap));
        assert_eq!(
            item_region("unexplained growth: 1.5 bytes/n in heap (S1); x in non-heap (S9)"),
            Some(Region::Heap)
        );
        assert_eq!(item_region("something else in heap (S1)"), None);
    }

    #[test]
    fn units_follow_the_axis() {
        assert_eq!(unit_of(Axis::N), "bytes/n");
        assert_eq!(unit_of(Axis::Iteration), "bytes/iteration");
        assert_eq!(unit_of(Axis::Seconds), "bytes/second");
    }
}
