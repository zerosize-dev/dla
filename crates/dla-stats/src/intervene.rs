//! The decision rule of `dla intervene`: did an edit remove the growth?
//!
//! [`judge_intervention`] is pure. The caller measures the control column (the trend the
//! finding was confirmed on) and the treated column (the same column after the edit, in a
//! scratch copy), checks the guards, and asks for the [`Effect`]. The two slope fractions
//! are placeholders that M7 calibrates on the benchmark.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{GateVerdict, TrendShape};

/// A treated slope at or below this fraction of the control slope counts as removed.
/// Provisional until the M7 calibration.
pub const REMOVED_FRACTION: f64 = 0.2;
/// A treated slope at or above this fraction of the control slope, with the gate still
/// passing, counts as remaining. Provisional until the M7 calibration.
pub const REMAINS_FRACTION: f64 = 0.5;

/// What the edit did to the growth.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Effect {
    /// The column no longer grows: evidence for the finding.
    GrowthRemoved,
    /// The column still grows: another writer, or the edited site is not the cause. Never
    /// evidence against the finding.
    GrowthRemains,
    /// Neither: the growth fell but not clearly.
    Inconclusive,
    /// A guard failed, so the run says nothing.
    Invalid,
}

impl Effect {
    /// The word used in evidence and output.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GrowthRemoved => "growth-removed",
            Self::GrowthRemains => "growth-remains",
            Self::Inconclusive => "inconclusive",
            Self::Invalid => "invalid",
        }
    }
}

/// One analysed column: the figures the rule reads.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ColumnView {
    /// Theil-Sen slope; `None` when there were too few points.
    pub slope: Option<f64>,
    /// Lower end of the slope interval.
    pub lo: Option<f64>,
    /// Gate verdict.
    pub gate: GateVerdict,
    /// Shape of the curve.
    pub shape: TrendShape,
}

/// The treated series as far as the rule needs it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Treated {
    /// The column of the control's name; `None` when the treated series has no such column.
    pub column: Option<ColumnView>,
    /// True when the treated series has its other columns (the agent or shim worked), so a
    /// missing column means the edit removed what wrote it.
    pub other_columns: bool,
}

/// The mechanical checks around an intervention. `None` means not applicable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
pub struct Guards {
    /// The scratch copy built.
    pub build_ok: bool,
    /// Every measured process exited 0, none timed out.
    pub run_ok: bool,
    /// The treated series has the control's modes, point count and step.
    pub same_points: bool,
    /// The edit touches only the allowed sites.
    pub edit_scoped: bool,
    /// The edited line ran in the control run (coverage data); `None` without coverage.
    pub exercised: Option<bool>,
    /// A control column that was not the finding's is still present after the edit.
    pub positive_control: Option<bool>,
    /// The unedited clone reproduced the control (`--rerun-control`).
    pub control_reproduced: Option<bool>,
}

impl Guards {
    /// Names of the guards that are false; a `None` guard is not false.
    pub fn failed(&self) -> Vec<&'static str> {
        let mut out = Vec::new();
        for (name, ok) in [
            ("build_ok", Some(self.build_ok)),
            ("run_ok", Some(self.run_ok)),
            ("same_points", Some(self.same_points)),
            ("edit_scoped", Some(self.edit_scoped)),
            ("exercised", self.exercised),
            ("positive_control", self.positive_control),
            ("control_reproduced", self.control_reproduced),
        ] {
            if ok == Some(false) {
                out.push(name);
            }
        }
        out
    }
}

/// Judges an intervention.
///
/// | Condition | Effect |
/// |---|---|
/// | any guard false, or the control has no positive slope | `Invalid` |
/// | treated column absent, other columns present | `GrowthRemoved` |
/// | treated column absent, nothing else present | `Invalid` |
/// | treated gate fails, shape flat or plateau, slope <= 0.2 x control slope or interval low end <= 0 | `GrowthRemoved` |
/// | treated gate passes, slope >= 0.5 x control slope | `GrowthRemains` |
/// | otherwise | `Inconclusive` |
pub fn judge_intervention(control: &ColumnView, treated: &Treated, guards: &Guards) -> Effect {
    if !guards.failed().is_empty() {
        return Effect::Invalid;
    }
    let Some(c) = control.slope.filter(|s| *s > 0.0) else {
        return Effect::Invalid;
    };
    let Some(t) = &treated.column else {
        return if treated.other_columns {
            Effect::GrowthRemoved
        } else {
            Effect::Invalid
        };
    };
    match t.gate {
        GateVerdict::Fail => {
            let flat = matches!(t.shape, TrendShape::Flat | TrendShape::Plateau);
            let small = t.slope.is_some_and(|s| s <= REMOVED_FRACTION * c)
                || t.lo.is_some_and(|lo| lo <= 0.0);
            if flat && small {
                Effect::GrowthRemoved
            } else {
                Effect::Inconclusive
            }
        }
        GateVerdict::Pass => {
            if t.slope.is_some_and(|s| s >= REMAINS_FRACTION * c) {
                Effect::GrowthRemains
            } else {
                Effect::Inconclusive
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn col(slope: f64, lo: f64, gate: GateVerdict, shape: TrendShape) -> ColumnView {
        ColumnView {
            slope: Some(slope),
            lo: Some(lo),
            gate,
            shape,
        }
    }

    fn control() -> ColumnView {
        col(12.4, 9.1, GateVerdict::Pass, TrendShape::Steady)
    }

    fn treated(c: ColumnView) -> Treated {
        Treated {
            column: Some(c),
            other_columns: true,
        }
    }

    fn ok() -> Guards {
        Guards {
            build_ok: true,
            run_ok: true,
            same_points: true,
            edit_scoped: true,
            exercised: None,
            positive_control: None,
            control_reproduced: None,
        }
    }

    #[test]
    fn a_flat_failed_column_is_growth_removed() {
        let t = treated(col(0.0, -0.3, GateVerdict::Fail, TrendShape::Flat));
        assert_eq!(
            judge_intervention(&control(), &t, &ok()),
            Effect::GrowthRemoved
        );
        // A plateau with a small slope also counts.
        let t = treated(col(1.0, 0.4, GateVerdict::Fail, TrendShape::Plateau));
        assert_eq!(
            judge_intervention(&control(), &t, &ok()),
            Effect::GrowthRemoved
        );
    }

    #[test]
    fn a_failed_gate_with_a_slope_above_a_fifth_is_inconclusive() {
        let t = treated(col(4.0, 0.5, GateVerdict::Fail, TrendShape::Plateau));
        assert_eq!(
            judge_intervention(&control(), &t, &ok()),
            Effect::Inconclusive
        );
        // A failed gate that is not flat: the shape decides.
        let t = treated(col(0.0, -0.3, GateVerdict::Fail, TrendShape::Spike));
        assert_eq!(
            judge_intervention(&control(), &t, &ok()),
            Effect::Inconclusive
        );
    }

    #[test]
    fn an_interval_reaching_zero_removes_growth_even_with_a_larger_slope() {
        let t = treated(col(3.0, -0.1, GateVerdict::Fail, TrendShape::Flat));
        assert_eq!(
            judge_intervention(&control(), &t, &ok()),
            Effect::GrowthRemoved
        );
    }

    #[test]
    fn a_passing_column_at_half_the_control_slope_remains() {
        let t = treated(col(6.2, 3.0, GateVerdict::Pass, TrendShape::Steady));
        assert_eq!(
            judge_intervention(&control(), &t, &ok()),
            Effect::GrowthRemains
        );
        let t = treated(col(12.0, 8.0, GateVerdict::Pass, TrendShape::Steady));
        assert_eq!(
            judge_intervention(&control(), &t, &ok()),
            Effect::GrowthRemains
        );
    }

    #[test]
    fn a_passing_column_below_half_is_inconclusive() {
        let t = treated(col(3.0, 1.0, GateVerdict::Pass, TrendShape::Steady));
        assert_eq!(
            judge_intervention(&control(), &t, &ok()),
            Effect::Inconclusive
        );
    }

    #[test]
    fn any_false_guard_is_invalid() {
        let t = treated(col(0.0, -0.3, GateVerdict::Fail, TrendShape::Flat));
        for g in [
            Guards {
                build_ok: false,
                ..ok()
            },
            Guards {
                run_ok: false,
                ..ok()
            },
            Guards {
                same_points: false,
                ..ok()
            },
            Guards {
                edit_scoped: false,
                ..ok()
            },
            Guards {
                exercised: Some(false),
                ..ok()
            },
            Guards {
                positive_control: Some(false),
                ..ok()
            },
            Guards {
                control_reproduced: Some(false),
                ..ok()
            },
        ] {
            assert_eq!(judge_intervention(&control(), &t, &g), Effect::Invalid);
        }
        // A guard that is not applicable does not block.
        let g = Guards {
            exercised: Some(true),
            positive_control: Some(true),
            control_reproduced: Some(true),
            ..ok()
        };
        assert_eq!(
            judge_intervention(&control(), &t, &g),
            Effect::GrowthRemoved
        );
    }

    #[test]
    fn a_missing_column_counts_only_when_the_rest_of_the_series_is_there() {
        let gone = Treated {
            column: None,
            other_columns: true,
        };
        assert_eq!(
            judge_intervention(&control(), &gone, &ok()),
            Effect::GrowthRemoved
        );
        let nothing = Treated {
            column: None,
            other_columns: false,
        };
        assert_eq!(
            judge_intervention(&control(), &nothing, &ok()),
            Effect::Invalid
        );
    }

    #[test]
    fn a_control_without_a_positive_slope_is_invalid() {
        let c = col(0.0, -1.0, GateVerdict::Pass, TrendShape::Steady);
        let t = treated(col(0.0, -0.3, GateVerdict::Fail, TrendShape::Flat));
        assert_eq!(judge_intervention(&c, &t, &ok()), Effect::Invalid);
    }

    #[test]
    fn too_few_points_are_inconclusive() {
        let t = treated(ColumnView {
            slope: None,
            lo: None,
            gate: GateVerdict::Fail,
            shape: TrendShape::Insufficient,
        });
        assert_eq!(
            judge_intervention(&control(), &t, &ok()),
            Effect::Inconclusive
        );
    }

    #[test]
    fn effect_words_are_the_ledger_words() {
        assert_eq!(Effect::GrowthRemoved.as_str(), "growth-removed");
        assert_eq!(
            serde_json::to_string(&Effect::GrowthRemains).unwrap(),
            "\"growth-remains\""
        );
    }
}
