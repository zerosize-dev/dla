//! Trend reports: `.dla/<run>/trend/T<n>.json`, and the bounded summary `dla trend`
//! writes into the `result` of its growth evidence.
//!
//! The statistics themselves live in `dla-stats`; this module holds the file format
//! and the summary shape the ledger reads when it checks the growth gate.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use dla_stats::{
    EveryDelta, GateThresholds, GateVerdict, HalfSplit, MannKendall, TheilSen, TrendShape,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    error::Error,
    finding::{Region, Shape},
    fsutil::{to_pretty_json, write_atomic},
    ids::FindingId,
    series::{ColumnKind, SeriesRegion, Unit, numbered},
    version::{SchemaError, SchemaErrorKind, read_version},
};

/// Schema version of a trend file.
pub const TREND_SCHEMA_VERSION: u32 = 1;
/// Directory (inside the run directory) holding trend files.
pub const TREND_DIR: &str = "trend";

fn default_one() -> u32 {
    TREND_SCHEMA_VERSION
}

/// The trend of one series column over its post-warm-up points.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ColumnTrend {
    /// Column name.
    pub name: String,
    /// Unit of the column.
    pub unit: Unit,
    /// Gauge or counter.
    pub kind: ColumnKind,
    /// Region of the column.
    pub region: SeriesRegion,
    /// `file:line` the column is tied to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    /// Finding the column was probed for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding: Option<FindingId>,
    /// Findings whose site matches the column.
    #[serde(default)]
    pub candidates: Vec<FindingId>,
    /// Post-warm-up points with a value.
    pub n: u32,
    /// Mann-Kendall test.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mk: Option<MannKendall>,
    /// Theil-Sen slope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sen: Option<TheilSen>,
    /// Slopes of the two halves.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub halves: Option<HalfSplit>,
    /// Every-delta rule (counters only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub every_delta: Option<EveryDelta>,
    /// Robust residual standard deviation around the Sen fit.
    pub noise: f64,
    /// Slope times the x span of the post-warm-up points.
    pub rise: f64,
    /// Shape of the curve.
    pub shape: TrendShape,
    /// Gate verdict.
    pub gate: GateVerdict,
    /// Why the gate failed, one reason per failed condition.
    #[serde(default)]
    pub gate_failed: Vec<String>,
}

/// `trend/T<n>.json`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct TrendReport {
    /// Schema version (1).
    #[serde(default = "default_one")]
    pub schema_version: u32,
    /// `T2`.
    pub id: String,
    /// The series analysed (`S3`).
    pub series: String,
    /// Thresholds the gate used.
    pub thresholds: GateThresholds,
    /// One entry per analysed column.
    pub columns: Vec<ColumnTrend>,
}

impl TrendReport {
    /// `<run_dir>/trend/<id>.json`.
    pub fn path_in(run_dir: &Path, id: &str) -> PathBuf {
        run_dir.join(TREND_DIR).join(format!("{id}.json"))
    }

    /// Loads `<run_dir>/trend/<id>.json`.
    pub fn load(run_dir: &Path, id: &str) -> Result<Self, Error> {
        let path = Self::path_in(run_dir, id);
        let text = fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
        let malformed = |message: String| Error::Malformed {
            path: path.clone(),
            message,
        };
        let doc: Value = serde_json::from_str(&text).map_err(|e| malformed(e.to_string()))?;
        if let Some(found) = read_version(&doc)
            && found > TREND_SCHEMA_VERSION
        {
            return Err(Error::Schema(SchemaError {
                what: format!("{id}.json"),
                found,
                supported: TREND_SCHEMA_VERSION,
                kind: SchemaErrorKind::Newer,
            }));
        }
        serde_json::from_value(doc).map_err(|e| malformed(e.to_string()))
    }

    /// Every readable `trend/T<n>.json` of the run, in numeric order. A file that cannot be
    /// read is skipped.
    pub fn load_all(run_dir: &Path) -> Vec<Self> {
        let Ok(entries) = fs::read_dir(run_dir.join(TREND_DIR)) else {
            return Vec::new();
        };
        let mut ids: Vec<(u32, String)> = entries
            .filter_map(Result::ok)
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().into_owned();
                let id = name.strip_suffix(".json")?.to_owned();
                numbered('T', &id).map(|n| (n, id))
            })
            .collect();
        ids.sort();
        ids.into_iter()
            .filter_map(|(_, id)| Self::load(run_dir, &id).ok())
            .collect()
    }

    /// Writes `<run_dir>/trend/<id>.json` atomically.
    pub fn save(&self, run_dir: &Path) -> Result<PathBuf, Error> {
        let path = Self::path_in(run_dir, &self.id);
        if numbered('T', &self.id).is_none() {
            return Err(Error::Malformed {
                path,
                message: format!("trend id {:?} is not T<n>", self.id),
            });
        }
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
        }
        write_atomic(&path, to_pretty_json(self).as_bytes())?;
        Ok(path)
    }
}

/// The finding shape a trend shape stands for: steady, stair, spike and plateau map;
/// the others (flat, declining, insufficient) describe no leak shape.
pub fn finding_shape(shape: TrendShape) -> Option<Shape> {
    match shape {
        TrendShape::Steady => Some(Shape::Steady),
        TrendShape::Stair => Some(Shape::Stair),
        TrendShape::Spike => Some(Shape::Spike),
        TrendShape::Plateau => Some(Shape::Plateau),
        TrendShape::Insufficient | TrendShape::Flat | TrendShape::Declining => None,
    }
}

/// The finding region a series region stands for; `process` maps to none.
pub fn finding_region(region: SeriesRegion) -> Option<Region> {
    match region {
        SeriesRegion::Heap => Some(Region::Heap),
        SeriesRegion::OffHeap => Some(Region::OffHeap),
        SeriesRegion::NonHeap => Some(Region::NonHeap),
        SeriesRegion::Process => None,
    }
}

/// One entry of the `passed` list in a growth evidence `result`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct PassedColumn {
    /// Column name.
    #[serde(default)]
    pub column: String,
    /// Region of the column; absent in records written by older or foreign tools.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<SeriesRegion>,
    /// `file:line` the column is tied to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    /// Finding the column was probed for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding: Option<String>,
    /// Findings whose site matches the column.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub candidates: Vec<String>,
    /// Theil-Sen slope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slope: Option<f64>,
    /// Lower end of the slope interval.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lo: Option<f64>,
    /// One-sided Mann-Kendall p-value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p: Option<f64>,
    /// Second-half to first-half slope ratio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ratio: Option<f64>,
    /// Curve shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<TrendShape>,
    /// Unit of the column, when the writer recorded it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<Unit>,
}

/// True when the probe column `column` (`probe:<Class>.<field>`, optionally with a
/// `.size` or `.bytes` suffix) measures the holder named `owner` (`<Class>.<field>`,
/// possibly package-qualified). A probe column matches when the names are equal or one is
/// a trailing run of dotted components of the other (probes may be named without the
/// package). A heap-dump column (`holder:<key>[.count]`) always carries the qualified
/// name, so it matches only the same holder: equal names after
/// [`crate::attribution::canonical`], never a same-named holder of another package.
pub fn probe_column_matches_owner(column: &str, owner: &str) -> bool {
    let name = if let Some(n) = column.strip_prefix("probe:") {
        n
    } else if let Some(n) = column.strip_prefix("holder:") {
        let n = n.strip_suffix(".count").unwrap_or(n);
        let owner = owner.trim();
        return !owner.is_empty()
            && match (
                crate::attribution::canonical(n),
                crate::attribution::canonical(owner),
            ) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            };
    } else {
        return false;
    };
    let stem = ["size", "bytes"]
        .iter()
        .find_map(|s| name.strip_suffix(s).and_then(|n| n.strip_suffix('.')));
    let dotted_suffix = |long: &str, short: &str| {
        long == short
            || long
                .strip_suffix(short)
                .is_some_and(|rest| rest.ends_with('.'))
    };
    let owner = owner.trim();
    !owner.is_empty()
        && [Some(name), stem]
            .into_iter()
            .flatten()
            .any(|n| !n.is_empty() && (dotted_suffix(owner, n) || dotted_suffix(n, owner)))
}

impl PassedColumn {
    /// True when the column is tied to the finding `id` with `site`: by its `finding`,
    /// by an equal site, or by listing the finding among its candidates.
    pub fn tied_to(&self, id: &FindingId, site: Option<&str>) -> bool {
        let id = id.to_string();
        self.finding.as_deref() == Some(id.as_str())
            || site.is_some() && self.site.as_deref() == site
            || self.candidates.contains(&id)
    }

    /// [`Self::tied_to`], or a probe column named after the finding's `owner`
    /// (`probe:Audit.trail` for the owner `shop.codec.Audit.trail`).
    pub fn tied_to_owner(&self, id: &FindingId, site: Option<&str>, owner: Option<&str>) -> bool {
        self.tied_to(id, site) || owner.is_some_and(|o| probe_column_matches_owner(&self.column, o))
    }
}

/// The bounded `result` of a `growth-series` or `unit-count-series` evidence.
///
/// Parsing is lenient: any field may be missing, and unknown fields are ignored.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct GrowthSummary {
    /// Trend ID (`T2`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trend: Option<String>,
    /// Series ID (`S3`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    /// Workload ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workload: Option<String>,
    /// Key mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// Axis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub axis: Option<String>,
    /// Post-warm-up points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Columns that passed the gate, most important first.
    #[serde(default)]
    pub passed: Vec<PassedColumn>,
    /// Number of columns that failed the gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failed: Option<u32>,
    /// Count of columns per shape.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub shapes: BTreeMap<String, u32>,
    /// `pass` when at least one column of this kind passed. Written by `dla trend` only;
    /// the raw `growth-series` record of `dla sample` has no gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<GateVerdict>,
    /// How many passed columns were cut to keep the result bounded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub truncated: Option<u32>,
}

impl GrowthSummary {
    /// Reads a summary from an evidence `result`; `None` when it is not an object of
    /// this shape (for example a `gate` that is neither `pass` nor `fail`).
    pub fn from_result(result: &Value) -> Option<Self> {
        if !result.is_object() {
            return None;
        }
        serde_json::from_value(result.clone()).ok()
    }

    /// True when the top-level gate says `pass`.
    pub fn passed_gate(&self) -> bool {
        self.gate == Some(GateVerdict::Pass)
    }

    /// The first passing column bound to the finding `id` (or `site`), else the first
    /// passing column.
    pub fn best_column(&self, id: &FindingId, site: Option<&str>) -> Option<&PassedColumn> {
        self.best_column_for(id, site, None)
    }

    /// [`Self::best_column`], also tying a probe column to the finding's `owner`.
    pub fn best_column_for(
        &self,
        id: &FindingId,
        site: Option<&str>,
        owner: Option<&str>,
    ) -> Option<&PassedColumn> {
        self.passed
            .iter()
            .find(|c| c.tied_to_owner(id, site, owner))
            .or_else(|| self.passed.first())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_probe_column_ties_to_the_owner_of_a_finding() {
        assert!(probe_column_matches_owner(
            "probe:Audit.trail",
            "shop.codec.Audit.trail"
        ));
        assert!(probe_column_matches_owner(
            "probe:Audit.trail.size",
            "shop.codec.Audit.trail"
        ));
        assert!(probe_column_matches_owner(
            "probe:Audit.trail.bytes",
            "Audit.trail"
        ));
        assert!(probe_column_matches_owner(
            "probe:shop.codec.Audit.trail",
            "Audit.trail"
        ));
        assert!(probe_column_matches_owner("probe:Cache.size", "Cache.size"));
        assert!(!probe_column_matches_owner(
            "probe:Audit.trail",
            "shop.codec.Audit.other"
        ));
        assert!(!probe_column_matches_owner(
            "probe:Audit.trail",
            "xAudit.trail"
        ));
        assert!(!probe_column_matches_owner("os.rss", "Audit.trail"));
        assert!(!probe_column_matches_owner("probe:Audit.trail", ""));
        let col = PassedColumn {
            column: "probe:Audit.trail".into(),
            ..PassedColumn::default()
        };
        let id = FindingId::new(2);
        assert!(!col.tied_to(&id, None));
        assert!(col.tied_to_owner(&id, None, Some("shop.codec.Audit.trail")));
        // Heap-dump series columns tie by owner too.
        assert!(probe_column_matches_owner(
            "holder:static:quotes.RfqDesk.OPEN",
            "quotes.RfqDesk.OPEN"
        ));
        assert!(probe_column_matches_owner(
            "holder:static:quotes.RfqDesk.OPEN.count",
            "quotes.RfqDesk.OPEN"
        ));
        assert!(!probe_column_matches_owner(
            "holder:static:quotes.RfqDesk.RECENT",
            "quotes.RfqDesk.OPEN"
        ));
        // A heap-dump column names one qualified holder: a shorter owner, or the same
        // class name in another package, is not that holder.
        assert!(!probe_column_matches_owner(
            "holder:static:quotes.RfqDesk.OPEN.count",
            "RfqDesk.OPEN"
        ));
        assert!(!probe_column_matches_owner(
            "holder:static:b.Cache.MAP",
            "Cache.MAP"
        ));
        assert!(!probe_column_matches_owner(
            "holder:static:b.Cache.MAP",
            "a.Cache.MAP"
        ));
        // Kotlin names the census and the JVM spell differently are one holder.
        assert!(probe_column_matches_owner(
            "holder:static:shop.Svc.Companion.CACHE",
            "shop.Svc.CACHE"
        ));
    }

    fn trend() -> TrendReport {
        TrendReport {
            schema_version: 1,
            id: "T2".into(),
            series: "S3".into(),
            thresholds: GateThresholds::default(),
            columns: vec![ColumnTrend {
                name: "probe:CACHE".into(),
                unit: Unit::Count,
                kind: ColumnKind::Counter,
                region: SeriesRegion::Heap,
                site: Some("src/a.go:9".into()),
                finding: None,
                candidates: vec![FindingId::new(3)],
                n: 8,
                mk: None,
                sen: Some(TheilSen {
                    slope: 2.0,
                    intercept: 1.0,
                    lo: 1.5,
                    hi: 2.5,
                }),
                halves: None,
                every_delta: Some(EveryDelta {
                    deltas: 7,
                    all_at_least_one: true,
                    min: 1.0,
                    max: 3.0,
                }),
                noise: 0.1,
                rise: 14.0,
                shape: TrendShape::Steady,
                gate: GateVerdict::Pass,
                gate_failed: vec![],
            }],
        }
    }

    #[test]
    fn report_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        let t = trend();
        let p = t.save(dir.path()).unwrap();
        assert_eq!(p, dir.path().join("trend/T2.json"));
        assert_eq!(TrendReport::load(dir.path(), "T2").unwrap(), t);
    }

    #[test]
    fn shapes_and_regions_map_to_finding_values() {
        assert_eq!(finding_shape(TrendShape::Stair), Some(Shape::Stair));
        assert_eq!(finding_shape(TrendShape::Flat), None);
        assert_eq!(finding_region(SeriesRegion::OffHeap), Some(Region::OffHeap));
        assert_eq!(finding_region(SeriesRegion::Process), None);
    }

    #[test]
    fn summary_parses_leniently_and_finds_the_bound_column() {
        let r = json!({"trend":"T2","series":"S3","gate":"pass","extra":1,
            "passed":[{"column":"a","region":"heap","shape":"steady"},
                      {"column":"b","region":"off-heap","finding":"F3","shape":"stair","novel":true}]});
        let s = GrowthSummary::from_result(&r).unwrap();
        assert!(s.passed_gate());
        let f3 = FindingId::new(3);
        assert_eq!(s.best_column(&f3, None).unwrap().column, "b");
        assert_eq!(s.best_column(&FindingId::new(9), None).unwrap().column, "a");
        assert!(GrowthSummary::from_result(&json!({"gate":"maybe"})).is_none());
        assert!(GrowthSummary::from_result(&json!([1])).is_none());
        assert!(
            !GrowthSummary::from_result(&json!({}))
                .unwrap()
                .passed_gate()
        );
    }

    #[test]
    fn tied_by_finding_site_or_candidates() {
        let f3 = FindingId::new(3);
        let by_site = PassedColumn {
            site: Some("a.go:1".into()),
            ..PassedColumn::default()
        };
        assert!(by_site.tied_to(&f3, Some("a.go:1")));
        assert!(!by_site.tied_to(&f3, Some("a.go:2")));
        assert!(!by_site.tied_to(&f3, None));
        let by_cand = PassedColumn {
            candidates: vec!["F3".into()],
            ..PassedColumn::default()
        };
        assert!(by_cand.tied_to(&f3, None));
    }
}
