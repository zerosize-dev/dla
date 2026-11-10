//! Statistics for DLA: robust trend tests and growth-shape classification over measured
//! series.
//!
//! Everything here is pure: callers pass `(x, y)` points (warm-up already removed, missing
//! values dropped) and get plain values back. `dla trend` turns them into evidence. See
//! [`trend`] for the Mann-Kendall, Theil-Sen, half-split, every-delta and noise algorithms
//! and the `classify` module docs (re-exported as [`classify()`], [`gate()`], [`analyze()`]) for the shape rules and the gate.

mod classify;
pub mod intervene;
pub mod normal;
pub mod trend;
mod types;

pub use classify::{Analysis, analyze, classify, gate, gate_counter};
pub use intervene::{ColumnView, Effect, Guards, Treated, judge_intervention};
pub use trend::{every_delta, half_split, mann_kendall, noise, theil_sen};
pub use types::{
    EveryDelta, GateThresholds, GateVerdict, HalfSplit, MannKendall, TheilSen, TrendShape,
};
