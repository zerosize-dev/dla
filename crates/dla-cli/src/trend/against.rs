//! `dla trend --against`: fresh keys versus repeated keys.
//!
//! Decision per column (a = the fresh-keys series, b = the repeated-keys series), where
//! a column "grows" when it passes the gate and is "quiet" when it fails the gate with a
//! flat, plateau, declining or spike shape:
//!
//! | fresh a | repeated b | verdict |
//! |---|---|---|
//! | grows | grows | leak: growth independent of the keys |
//! | grows | quiet, a is not a plateau | leak: growth tied to new keys that never stops |
//! | grows | quiet, a is a plateau | non-leak, N-WARM and N-SCALE: a bounded key space |
//! | quiet | quiet | non-leak, N-WARM: nothing grows |
//! | quiet | grows, or anything unclear | inconclusive |
//!
//! Over the compared columns any leak verdict wins; otherwise the result is non-leak
//! when at least one column says so and none is inconclusive.

use dla_core::{FindingId, discriminator::Favors, trend::ColumnTrend};
use dla_stats::{GateVerdict, TrendShape};
use serde_json::{Value, json};

/// The verdict for one compared column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Growth that keys do not explain.
    Leak,
    /// A bounded key space: warm-up and scale.
    NonLeakScale,
    /// Nothing grows.
    NonLeak,
    /// The pair cannot be read.
    Inconclusive,
}

impl Verdict {
    /// Short name for the output.
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Leak => "leak",
            Verdict::NonLeakScale => "non-leak-scale",
            Verdict::NonLeak => "non-leak",
            Verdict::Inconclusive => "inconclusive",
        }
    }
}

fn grows(c: &ColumnTrend) -> bool {
    c.gate == GateVerdict::Pass
}

/// A column the gate judged: a short series fails the gate only for lack of points, which
/// says nothing about growth.
fn judged(c: &ColumnTrend) -> bool {
    !c.gate_failed.iter().any(|r| r.starts_with("n <"))
}

fn quiet(c: &ColumnTrend) -> bool {
    c.gate == GateVerdict::Fail
        && judged(c)
        && matches!(
            c.shape,
            TrendShape::Flat | TrendShape::Plateau | TrendShape::Declining | TrendShape::Spike
        )
}

/// The verdict for a fresh/repeated pair of the same column.
pub fn verdict(a: &ColumnTrend, b: &ColumnTrend) -> Verdict {
    match (grows(a), grows(b)) {
        (true, true) => Verdict::Leak,
        (true, false) if quiet(b) => {
            if a.shape == TrendShape::Plateau {
                Verdict::NonLeakScale
            } else {
                Verdict::Leak
            }
        }
        (false, false) if quiet(a) && quiet(b) => Verdict::NonLeak,
        _ => Verdict::Inconclusive,
    }
}

/// One compared column.
#[derive(Debug, Clone)]
pub struct Compared {
    /// Column name.
    pub column: String,
    /// Verdict.
    pub verdict: Verdict,
    /// Shape in the fresh-keys series.
    pub fresh: TrendShape,
    /// Gate in the fresh-keys series.
    pub fresh_gate: GateVerdict,
    /// Shape in the repeated-keys series.
    pub repeated: TrendShape,
    /// Gate in the repeated-keys series.
    pub repeated_gate: GateVerdict,
}

/// The overall decision.
#[derive(Debug, Clone)]
pub struct Decision {
    /// What the comparison favors.
    pub favors: Favors,
    /// `N-*` ids for non-leak, `["leak"]` for leak.
    pub supports: Vec<String>,
    /// The compared columns.
    pub columns: Vec<Compared>,
    /// A finding was given but none of the series' columns is tied to it, so nothing was
    /// compared and the discriminator must not be bound to the finding.
    pub untied: bool,
}

/// Compares the columns of `fresh` and `repeated` that share a name and kind. Container
/// columns and pairs with no points on either side are left out. When `finding` is
/// given and some compared column is tied to it (by `finding`, `candidates` or equal
/// `site` to `site`), only the tied columns count; when none is tied the result is
/// inconclusive and `untied` is set.
pub fn compare(
    fresh: &[ColumnTrend],
    repeated: &[ColumnTrend],
    finding: Option<(&FindingId, Option<&str>)>,
) -> Decision {
    let mut pairs: Vec<(&ColumnTrend, &ColumnTrend)> = fresh
        .iter()
        .filter(|a| {
            !a.gate_failed
                .iter()
                .any(|r| r == super::analysis::CONTAINER_REASON)
        })
        .filter_map(|a| {
            let b = repeated
                .iter()
                .find(|b| b.name == a.name && b.kind == a.kind)?;
            (a.n > 0 && b.n > 0).then_some((a, b))
        })
        .collect();
    if let Some((id, site)) = finding {
        let tied = |c: &ColumnTrend| {
            c.finding.as_ref() == Some(id)
                || c.candidates.contains(id)
                || site.is_some() && c.site.as_deref() == site
        };
        if !pairs.iter().any(|(a, _)| tied(a)) {
            return Decision {
                favors: Favors::Inconclusive,
                supports: vec![],
                columns: vec![],
                untied: true,
            };
        }
        pairs.retain(|(a, _)| tied(a));
    }
    let columns: Vec<Compared> = pairs
        .iter()
        .map(|(a, b)| Compared {
            column: a.name.clone(),
            verdict: verdict(a, b),
            fresh: a.shape,
            fresh_gate: a.gate,
            repeated: b.shape,
            repeated_gate: b.gate,
        })
        .collect();
    let has = |v: Verdict| columns.iter().any(|c| c.verdict == v);
    let (favors, supports) = if has(Verdict::Leak) {
        (Favors::Leak, vec!["leak".to_owned()])
    } else if has(Verdict::Inconclusive) || columns.is_empty() {
        (Favors::Inconclusive, vec![])
    } else if has(Verdict::NonLeakScale) {
        (
            Favors::NonLeak,
            vec!["N-WARM".to_owned(), "N-SCALE".to_owned()],
        )
    } else {
        (Favors::NonLeak, vec!["N-WARM".to_owned()])
    };
    Decision {
        favors,
        supports,
        columns,
        untied: false,
    }
}

impl Decision {
    /// The compared columns as JSON, worst first (leak, inconclusive, then the rest),
    /// cut to `max` entries; the second value is how many were cut.
    pub fn columns_json(&self, max: usize) -> (Vec<Value>, usize) {
        let rank = |v: Verdict| match v {
            Verdict::Leak => 0,
            Verdict::Inconclusive => 1,
            Verdict::NonLeakScale => 2,
            Verdict::NonLeak => 3,
        };
        let mut cols: Vec<&Compared> = self.columns.iter().collect();
        cols.sort_by_key(|c| rank(c.verdict));
        let cut = cols.len().saturating_sub(max);
        let out = cols
            .into_iter()
            .take(max)
            .map(|c| {
                json!({
                    "column": c.column,
                    "verdict": c.verdict.as_str(),
                    "fresh": format!("{}/{}", shape(c.fresh), gate(c.fresh_gate)),
                    "repeated": format!("{}/{}", shape(c.repeated), gate(c.repeated_gate)),
                })
            })
            .collect();
        (out, cut)
    }
}

fn shape(s: TrendShape) -> String {
    serde_json::to_value(s)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

fn gate(g: GateVerdict) -> &'static str {
    match g {
        GateVerdict::Pass => "pass",
        GateVerdict::Fail => "fail",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trend_cmd::analysis::{
        analyze_series,
        testutil::{Col, plateau, rising, series},
    };
    use dla_stats::GateThresholds;

    fn flat() -> Vec<f64> {
        vec![100.0; 9]
    }

    fn one(name: &str, v: &[f64]) -> Vec<ColumnTrend> {
        let s = series("S1", 1, vec![Col::counter(name, v)]);
        analyze_series(&s, &[0], &GateThresholds::default())
    }

    fn decide(a: &[f64], b: &[f64]) -> Decision {
        compare(&one("probe:C", a), &one("probe:C", b), None)
    }

    #[test]
    fn both_grow_is_a_leak() {
        let d = decide(&rising(10.0, 5.0), &rising(10.0, 3.0));
        assert_eq!(d.favors, Favors::Leak);
        assert_eq!(d.supports, ["leak"]);
        assert_eq!(d.columns[0].verdict, Verdict::Leak);
    }

    #[test]
    fn growth_with_new_keys_only_is_a_leak_unless_it_plateaus() {
        let d = decide(&rising(10.0, 5.0), &flat());
        assert_eq!(d.favors, Favors::Leak);
        assert_eq!(d.supports, ["leak"]);
        // A fresh-keys column that levels off (shape plateau) is a bounded key space.
        let a = one("probe:C", &plateau(1000.0));
        let b = one("probe:C", &flat());
        assert_eq!(a[0].shape, TrendShape::Plateau);
        assert_eq!(a[0].gate, GateVerdict::Fail);
        // It does not pass the gate, so both are quiet: nothing grows.
        let d = compare(&a, &b, None);
        assert_eq!(d.favors, Favors::NonLeak);
        assert_eq!(d.supports, ["N-WARM"]);
        // A plateau-shaped column that still passes the gate takes the scale branch.
        let mut a2 = a.clone();
        a2[0].gate = GateVerdict::Pass;
        a2[0].gate_failed.clear();
        let d = compare(&a2, &b, None);
        assert_eq!(d.favors, Favors::NonLeak);
        assert_eq!(d.supports, ["N-WARM", "N-SCALE"]);
    }

    #[test]
    fn neither_growing_is_non_leak_warm() {
        let d = decide(&flat(), &flat());
        assert_eq!(d.favors, Favors::NonLeak);
        assert_eq!(d.supports, ["N-WARM"]);
    }

    #[test]
    fn repeated_growth_without_fresh_growth_and_missing_data_are_inconclusive() {
        let d = decide(&flat(), &rising(10.0, 5.0));
        assert_eq!(d.favors, Favors::Inconclusive);
        assert!(d.supports.is_empty());
        let none = compare(&one("a", &flat()), &one("b", &flat()), None);
        assert_eq!(none.favors, Favors::Inconclusive);
        assert!(none.columns.is_empty());
        // Too few points to read (insufficient shape).
        let d = decide(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]);
        assert_eq!(d.favors, Favors::Inconclusive);
    }

    #[test]
    fn a_finding_restricts_the_comparison_to_its_columns() {
        let s_a = series(
            "S1",
            1,
            vec![
                Col::counter("probe:C", &flat()).finding(FindingId::new(1)),
                Col::gauge("os.rss", &rising(1e7, 1e6)),
            ],
        );
        let s_b = series(
            "S2",
            1,
            vec![
                Col::counter("probe:C", &flat()),
                Col::gauge("os.rss", &rising(1e7, 1e6)),
            ],
        );
        let th = GateThresholds::default();
        let a = analyze_series(&s_a, &[0, 1], &th);
        let b = analyze_series(&s_b, &[0, 1], &th);
        let all = compare(&a, &b, None);
        assert_eq!(all.favors, Favors::Leak);
        let f1 = FindingId::new(1);
        let tied = compare(&a, &b, Some((&f1, None)));
        assert_eq!(tied.favors, Favors::NonLeak);
        assert_eq!(tied.columns.len(), 1);
        // A finding no column is tied to is inconclusive, never judged from other columns.
        let f9 = FindingId::new(9);
        let d9 = compare(&a, &b, Some((&f9, None)));
        assert_eq!(d9.favors, Favors::Inconclusive);
        assert!(d9.untied && d9.columns.is_empty());
    }

    #[test]
    fn short_series_are_not_quiet() {
        // Fewer than min_points: the gate fails only for "n <", so it is inconclusive.
        let short = [100.0, 101.0, 99.0, 100.0, 100.0];
        let d = decide(&short, &short);
        assert_eq!(d.favors, Favors::Inconclusive);
        assert_eq!(d.columns[0].verdict, Verdict::Inconclusive);
    }

    #[test]
    fn container_columns_are_not_compared() {
        let s = series(
            "S1",
            1,
            vec![Col::gauge("cgroup.current", &rising(1e7, 1e6)).container()],
        );
        let t = analyze_series(&s, &[0], &GateThresholds::default());
        let d = compare(&t, &t, None);
        assert!(d.columns.is_empty());
        assert_eq!(d.favors, Favors::Inconclusive);
    }

    #[test]
    fn columns_json_is_bounded_and_worst_first() {
        let d = Decision {
            favors: Favors::Leak,
            supports: vec![],
            untied: false,
            columns: (0..20)
                .map(|i| Compared {
                    column: format!("c{i}"),
                    verdict: if i == 19 {
                        Verdict::Leak
                    } else {
                        Verdict::NonLeak
                    },
                    fresh: TrendShape::Flat,
                    fresh_gate: GateVerdict::Fail,
                    repeated: TrendShape::Flat,
                    repeated_gate: GateVerdict::Fail,
                })
                .collect(),
        };
        let (cols, cut) = d.columns_json(8);
        assert_eq!(cols.len(), 8);
        assert_eq!(cut, 12);
        assert_eq!(cols[0]["column"], "c19");
    }
}
