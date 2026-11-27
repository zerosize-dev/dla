//! The verdicts of `dla alloc-test`: the plateau test over the extra window, the purge
//! test against the purge event's before and after readings, and the gc test over a
//! column and its `.pre_gc` twin. Pure functions over `(x, y)` points.

use dla_core::discriminator::Favors;
use dla_stats::{GateThresholds, GateVerdict, TrendShape, analyze, every_delta, noise, theil_sen};
use serde_json::{Value, json};

/// The extra rise must exceed this many noise units for growth to continue.
const NOISE_MULTIPLE: f64 = 3.0;
/// A step of a column that grows in steps is a new maximum of more than this many units of the
/// main window's own scatter around its trend (see [`stair`]).
const STEP_SCATTERS: f64 = 2.0;
/// The extra slope must keep at least this share of the main slope.
const KEEP_SLOPE: f64 = 0.5;
/// A purge that returns at least this share of the growth releases it.
const RELEASED: f64 = 0.5;
/// A purge that returns at most this share of the growth retains it.
const RETAINED: f64 = 0.2;
/// A rise below this share of the column's level is drift, not growth, however small the
/// noise around it (a smooth series makes any slope "significant").
const MATERIAL: f64 = 0.01;
/// Smallest rise of a byte-unit gauge column that counts as growth, in bytes (placeholder,
/// calibrated in M7). The live heap of a correct Go program moves by a few KB between
/// points (cache and bookkeeping structures of the runtime); the largest rise of that kind
/// measured over an extra window was 13.4 KB, and the noise estimates of a short window
/// can not be trusted below that. Counters and unitless gauges have no such minimum.
pub const MIN_BYTE_RISE: f64 = 32.0 * 1024.0;
/// Fewest points of an extra window the gauge test accepts.
const MIN_GAUGE_POINTS: usize = 3;
/// Fewest rises the extra window would have shown, had the growth gone on at the rate of the
/// main window, for a plateau to stand against a purge that kept the growth: with one or two
/// expected a slow stair can sit between two steps; three means a step missing is no accident.
pub const MIN_EXPECTED_RISES: f64 = 3.0;

/// Keeps four significant digits so the evidence stays small.
pub fn round(x: f64) -> f64 {
    if x == 0.0 || !x.is_finite() {
        return x;
    }
    format!("{x:.3e}").parse().unwrap_or(x)
}

/// What the extra window says about the growth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlateauVerdict {
    /// The growth goes on at about the earlier rate.
    Continues,
    /// The growth stopped.
    Plateau,
    /// Neither.
    Inconclusive,
    /// The extra window is too short.
    NotRun,
    /// The column did not grow in the main window of this series (see [`grew`]): the series
    /// does not reproduce the growth, so its extra window says nothing about it.
    NeverGrew,
}

impl PlateauVerdict {
    /// The word written into the evidence.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Continues => "growth-continues",
            Self::Plateau => "plateau",
            Self::Inconclusive => "inconclusive",
            Self::NotRun => "not-run",
            Self::NeverGrew => "never-grew",
        }
    }
}

/// True when the column GREW in the main window of the series: a counter's net rise over the
/// main window is above zero (a counter has no noise to tell a rise from, and its every step is
/// a whole block or entry); a gauge's Theil-Sen interval over the main window is above zero and
/// its fitted rise is above `min_rise` (the same smallest rise the plateau test takes as growth)
/// and above [`MATERIAL`] of its level (`reference` for a process column). A main window of fewer
/// than two points did not show a rise. Whether the extra window then went flat is the plateau
/// test's question; a column that never grew did not stop growing.
pub fn grew(main: &[Point], counter: bool, reference: f64, min_rise: f64) -> bool {
    let (Some(first), Some(last)) = (main.first(), main.last()) else {
        return false;
    };
    if main.len() < 2 {
        return false;
    }
    if counter {
        return last.1 > first.1;
    }
    let Some(sen) = theil_sen(main, 0.05) else {
        return false;
    };
    let rise = sen.slope * (last.0 - first.0);
    sen.lo > 0.0 && rise > min_rise && rise > MATERIAL * mean_abs(main).max(reference)
}

/// The plateau test of one column.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plateau {
    /// Theil-Sen slope over the main window.
    pub main_slope: Option<f64>,
    /// Theil-Sen slope over the extra window (counters: the net increase).
    pub extra_slope: Option<f64>,
    /// Extra slope times the x span of the extra window (counters: the net increase).
    pub extra_rise: Option<f64>,
    /// Residual noise of the extra window.
    pub noise: Option<f64>,
    /// The verdict.
    pub verdict: PlateauVerdict,
    /// Extra points that would run past the next step expected of the main window's staircase,
    /// when the column grew in steps and its extra window was too short to tell (see [`stair`]).
    pub points_needed: Option<u32>,
}

/// An `(x, y)` point.
pub type Point = (f64, f64);

/// Splits post-warm-up points by `x`: the main window is every point up to `main_end_x`
/// (the x of the last main point of the series), the extra window the points after it.
/// Splitting by x, not by count, keeps the windows right when a column has missing readings
/// or exists only in part of the run.
pub fn split(points: &[Point], main_end_x: f64) -> (&[Point], &[Point]) {
    let k = points.partition_point(|p| p.0 <= main_end_x);
    points.split_at(k)
}

fn mean_abs(points: &[(f64, f64)]) -> f64 {
    if points.is_empty() {
        return 0.0;
    }
    points.iter().map(|p| p.1.abs()).sum::<f64>() / points.len() as f64
}

fn sen_slope(points: &[(f64, f64)]) -> Option<f64> {
    (points.len() >= 2)
        .then(|| theil_sen(points, 0.05).map(|s| s.slope))
        .flatten()
}

/// The plateau test. `reference` is the level a process (`os.*`) column's rise is measured
/// against when that is larger than the column itself (the process's resident memory; zero
/// for every other column, which is judged against its own level only): a rise under 1% of
/// it is drift, not growth, except that a verdict resting only on that floor while the main
/// window grows with confidence (Sen interval above 0) is inconclusive. Gauges: growth continues when the extra slope is at least half the
/// main slope and the extra rise exceeds three noise units; a plateau when the rise is
/// within that. The noise of the extra window is the larger of its own MAD and the residual
/// scatter of the main window around its fitted trend, so scatter the main window shows
/// cannot pass as growth in a short extra window whose MAD happens to read low. Counters:
/// every delta over the extra window (starting from the last main point) at least one means
/// growth; no net increase means a plateau. `min_rise` is the smallest rise of a gauge that
/// is growth at all (zero when the column has no such resolution; [`MIN_BYTE_RISE`] for a
/// byte column): a smaller rise is a plateau whatever the noise reads. A gauge that grew in
/// separate steps in the main window is not read as a plateau on the scatter its steps make, nor
/// on a robust slope that a late step leaves at zero: see [`stair`] (an extra window that ends
/// above the main window's maximum by more than the noise, the minimum rise and the drift floor is
/// growth that continues; a flat stretch shorter than the next gap is expected to be is
/// inconclusive).
pub fn plateau(
    main: &[(f64, f64)],
    extra: &[(f64, f64)],
    counter: bool,
    reference: f64,
    min_rise: f64,
) -> Plateau {
    let main_slope = sen_slope(main);
    let not_run = Plateau {
        main_slope,
        extra_slope: None,
        extra_rise: None,
        noise: None,
        verdict: PlateauVerdict::NotRun,
        points_needed: None,
    };
    if counter {
        if extra.is_empty() {
            return not_run;
        }
        let mut pts: Vec<(f64, f64)> = main.last().copied().into_iter().collect();
        pts.extend_from_slice(extra);
        if pts.len() < 2 {
            return not_run;
        }
        let net = pts[pts.len() - 1].1 - pts[0].1;
        let span = pts[pts.len() - 1].0 - pts[0].0;
        // A count that jitters (reachable objects, entries of a bounded cache) can end a
        // little above where it began: a fitted rise within three noise units of the fit is
        // no growth either.
        let (fit_rise, nz) =
            theil_sen(&pts, 0.05).map_or((net, 0.0), |s| (s.slope * span, noise(&pts, &s)));
        let verdict = if every_delta(&pts).is_some_and(|e| e.all_at_least_one) {
            PlateauVerdict::Continues
        } else if net <= 0.0 || fit_rise <= NOISE_MULTIPLE * nz {
            PlateauVerdict::Plateau
        } else {
            PlateauVerdict::Inconclusive
        };
        return Plateau {
            main_slope,
            extra_slope: (span > 0.0).then_some(net / span),
            extra_rise: Some(net),
            noise: Some(nz),
            verdict,
            points_needed: None,
        };
    }
    if extra.len() < MIN_GAUGE_POINTS {
        return not_run;
    }
    let Some(sen) = theil_sen(extra, 0.05) else {
        return not_run;
    };
    let own_noise = noise(extra, &sen);
    // The main window's scatter around its own trend is a second reading of the same
    // column's point-to-point noise, from more points than the extra window has.
    let main_noise = if main.len() >= MIN_GAUGE_POINTS {
        theil_sen(main, 0.05).map_or(0.0, |m| noise(main, &m))
    } else {
        0.0
    };
    let nz = own_noise.max(main_noise);
    let span = extra[extra.len() - 1].0 - extra[0].0;
    let rise = sen.slope * span;
    let scale = extra.iter().fold(0.0_f64, |m, p| m.max(p.1.abs()));
    let limit = NOISE_MULTIPLE * nz.max(1e-9 * scale);
    let keeps = main_slope.is_none_or(|m| sen.slope >= KEEP_SLOPE * m);
    let material = rise > MATERIAL * mean_abs(extra).max(reference);
    // The floor alone decides when the rise is clearly above the noise and keeps the main
    // slope but is small against the level. If the main window's Sen interval excludes 0
    // the column is growing with confidence, so a verdict resting only on the floor would
    // be a guess: inconclusive rather than plateau.
    let floor_only = rise > limit && keeps && !material;
    let main_grows = theil_sen(main, 0.05).is_some_and(|s| s.lo > 0.0);
    let verdict = if rise > limit && keeps && material && rise > min_rise {
        PlateauVerdict::Continues
    } else if rise <= min_rise {
        PlateauVerdict::Plateau
    } else if floor_only && main_grows {
        PlateauVerdict::Inconclusive
    } else if rise <= limit || !material {
        PlateauVerdict::Plateau
    } else {
        PlateauVerdict::Inconclusive
    };
    // A column that grows in steps scatters around a straight fit by the size of its steps, and
    // its robust slope over an extra window with one late step is zero, so a plateau verdict on
    // the straight fit says nothing about it: the steps themselves tell whether it went on. A new
    // maximum that holds to the end of the extra window is growth whatever the fitted rise reads;
    // it must clear the noise, the column's minimum rise and the drift floor. A window with no
    // step in it is a plateau only once it is longer than the next gap is expected to be.
    let mut points_needed = None;
    let verdict = if verdict == PlateauVerdict::Plateau {
        let main_max = main.iter().fold(f64::NEG_INFINITY, |m, p| m.max(p.1));
        let main_gain = main_max - main.first().map_or(main_max, |p| p.1);
        let end_gain = extra.last().map_or(0.0, |p| p.1) - main_max;
        let clears = |gain: f64| {
            gain > nz.max(1e-9 * scale)
                && gain > min_rise
                && gain > MATERIAL * mean_abs(extra).max(reference)
        };
        match stair(main, extra, nz) {
            Some(Stair::NewMax) if clears(end_gain) => PlateauVerdict::Continues,
            Some(Stair::TooShort { points_needed: n }) if clears(main_gain) => {
                points_needed = Some(n);
                PlateauVerdict::Inconclusive
            }
            _ => PlateauVerdict::Plateau,
        }
    } else {
        verdict
    };
    Plateau {
        main_slope,
        extra_slope: Some(sen.slope),
        extra_rise: Some(rise),
        noise: Some(nz),
        verdict,
        points_needed,
    }
}

/// How often the main window rose, and what that makes of its plateau window.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rises {
    /// Separate rises of the main window.
    pub count: u32,
    /// Rises the extra window would have held at that rate: `count * extra span / main span`.
    pub expected: f64,
    /// Points of the extra window that would reach [`MIN_EXPECTED_RISES`] (none without a rise).
    pub points_needed: Option<u32>,
}

/// The separate rises of `main`: points that set a new maximum by more than `noise`, counted
/// from `start` (the column at the end of the warm-up; the first main point when there is none).
pub fn count_rises(main: &[Point], start: Option<f64>, noise: f64) -> u32 {
    rise_xs(main, start, noise).len() as u32
}

/// The `x` of every point that sets a new maximum by more than `noise` (see [`count_rises`]).
fn rise_xs(main: &[Point], start: Option<f64>, noise: f64) -> Vec<f64> {
    let mut pts = main.iter();
    let first = pts.next();
    let Some(mut max) = start.or(first.map(|p| p.1)) else {
        return Vec::new();
    };
    let mut xs = Vec::new();
    // With a `start`, the first point is a reading like the others; without one it is the base.
    let rest = if start.is_some() { main.iter() } else { pts };
    for &(x, y) in rest {
        if y > max + noise {
            xs.push(x);
        }
        max = max.max(y);
    }
    xs
}

/// How the extra window of a column that grew in steps reads (see [`stair`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stair {
    /// The extra window ENDS above the main window's maximum by more than the noise: a step
    /// happened in it and held, and the growth goes on. (A spike that came back down is not one.)
    NewMax,
    /// No new maximum, and the flat stretch since the last step is no longer than the next gap is
    /// expected to be: a step may simply not have come yet. `points_needed` extra points would
    /// run past that gap.
    TooShort {
        /// Points of the extra window that run past the expected next step.
        points_needed: u32,
    },
    /// No new maximum for longer than the next gap is expected to be: the growth stopped.
    Stopped,
}

/// The step reading of a gauge column: `None` for a column that did not grow in separate steps
/// in the main window (fewer than two rises, or a rise at every interval, which is a ramp: its
/// scatter around a straight fit is small and the ordinary test serves). A step-shaped column
/// grows by jumps, so its scatter around a straight fit is the step size and "within three noise
/// units" says nothing about it; what it can show is a new maximum that holds (a step in the
/// extra window), or that no step came for as long as the next gap was to be.
///
/// The flat stretch is measured from the last step of the main window to the end of the extra
/// window. A container that doubles its capacity steps ever further apart (each gap about twice the
/// one before), so when each gap is at least the one before (one point spacing of slack; a single
/// gap qualifies, and the smaller early steps of a doubling can lie under the scatter that marks a
/// step) the next gap is expected to be `2 * (last gap + one spacing)`;
/// otherwise the longest gap seen. A stretch of at most that is `TooShort` (a step AT the
/// expected end is still to come), a longer one `Stopped`.
pub fn stair(main: &[Point], extra: &[Point], noise: f64) -> Option<Stair> {
    // A step is a new maximum of more than `STEP_SCATTERS` units of the main window's own scatter
    // around its trend: the scatter of a regular staircase is about a third of its step, so its
    // steps pass, and the few-KB jitter of a level column (a rise or two of one scatter) does not.
    let trend = theil_sen(main, 0.05)?;
    let xs = rise_xs(main, None, STEP_SCATTERS * dla_stats::noise(main, &trend));
    if xs.len() < 2 || xs.len() >= main.len().saturating_sub(1) {
        return None;
    }
    let main_max = main.iter().fold(f64::NEG_INFINITY, |m, p| m.max(p.1));
    let (e0, e1) = (extra.first()?, extra.last()?);
    if e1.1 > main_max + noise {
        return Some(Stair::NewMax);
    }
    let gaps: Vec<f64> = xs.windows(2).map(|w| w[1] - w[0]).collect();
    let longest = gaps.iter().copied().fold(0.0_f64, f64::max);
    let span = e1.0 - e0.0;
    let per_point = if extra.len() > 1 {
        span / (extra.len() - 1) as f64
    } else {
        0.0
    };
    let growing = !gaps.is_empty() && gaps.windows(2).all(|w| w[1] + per_point >= w[0]);
    let expected = match gaps.last() {
        Some(last) if growing => longest.max(2.0 * (last + per_point)),
        _ => longest,
    };
    let last_step = xs[xs.len() - 1];
    if e1.0 - last_step > expected {
        return Some(Stair::Stopped);
    }
    let points_needed = if per_point > 0.0 {
        ((last_step + expected - e0.0) / per_point).floor() as u32 + 2
    } else {
        extra.len() as u32 + 1
    };
    Some(Stair::TooShort {
        points_needed: points_needed.max(extra.len() as u32 + 1),
    })
}

/// The rises of the main window against the length of the extra window.
pub fn rises(main: &[Point], extra: &[Point], start: Option<f64>, noise: f64) -> Option<Rises> {
    let (m0, m1) = (main.first()?, main.last()?);
    let (e0, e1) = (extra.first()?, extra.last()?);
    let (main_span, extra_span) = (m1.0 - m0.0, e1.0 - e0.0);
    if main_span <= 0.0 || extra_span <= 0.0 {
        return None;
    }
    let count = count_rises(main, start, noise);
    let expected = f64::from(count) * extra_span / main_span;
    let per_point = extra_span / (extra.len() - 1) as f64;
    let points_needed = (count > 0).then(|| {
        let span = MIN_EXPECTED_RISES * main_span / f64::from(count);
        ((span / per_point).ceil() as u32 + 1).max(extra.len() as u32)
    });
    Some(Rises {
        count,
        expected,
        points_needed,
    })
}

impl Plateau {
    /// The evidence form.
    pub fn to_json(self) -> Value {
        json!({
            "main_slope": self.main_slope.map(round),
            "extra_slope": self.extra_slope.map(round),
            "extra_rise": self.extra_rise.map(round),
            "noise": self.noise.map(round),
            "verdict": self.verdict.as_str(),
        })
    }
}

/// What the purge did to the growth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PurgeVerdict {
    /// At least half of the growth was returned.
    Released,
    /// At most a fifth was returned.
    Retained,
    /// In between.
    Partial,
    /// The column did not grow above its warm-up value by more than its minimum rise, so
    /// there is nothing to release.
    NoGrowth,
    /// No reading of the column before and after the purge.
    NotRun,
}

impl PurgeVerdict {
    /// The word written into the evidence.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Released => "released",
            Self::Retained => "retained",
            Self::Partial => "partial",
            Self::NoGrowth => "no-growth",
            Self::NotRun => "not-run",
        }
    }
}

/// The purge test of one column.
#[derive(Debug, Clone, PartialEq)]
pub struct Purge {
    /// The purge kind that was applied.
    pub kind: String,
    /// Value before the purge.
    pub before: Option<f64>,
    /// Value after the purge.
    pub after: Option<f64>,
    /// `before` minus the value at the end of the warm-up.
    pub growth: Option<f64>,
    /// `before` minus `after`.
    pub drop: Option<f64>,
    /// The verdict.
    pub verdict: PurgeVerdict,
    /// Whether this purge kind can give this column's memory back at all. `false` for a
    /// column of a region no purge releases (file-backed pages, process peaks, virtual sizes):
    /// the reading stays in the evidence, but it says nothing about the column, so the
    /// column is judged on its plateau alone (see [`combine`]).
    pub applies: bool,
}

/// The purge test: `drop` against `growth` (`before` minus the end-of-warm-up value).
/// `min_rise` is the column's smallest growth that is growth at all (the minimum rise the
/// plateau test uses: [`MIN_BYTE_RISE`] for a byte column, zero otherwise); a growth at or
/// below it is `NoGrowth`, because a purge cannot give back what the program still holds (a
/// few KB of live objects) and the plateau test already calls that a plateau. A growth above
/// it is read as before, whatever the scatter of the column: a slow real leak under the
/// scatter must not read as nothing to release.
pub fn purge(
    kind: &str,
    before: Option<f64>,
    after: Option<f64>,
    warm_end: Option<f64>,
    min_rise: f64,
) -> Purge {
    let growth = before.zip(warm_end).map(|(b, w)| b - w);
    let drop = before.zip(after).map(|(b, a)| b - a);
    let verdict = match (growth, drop) {
        (Some(g), Some(d)) => {
            let scale = before.unwrap_or(0.0).abs().max(1.0);
            if g <= (1e-9 * scale).max(min_rise) {
                PurgeVerdict::NoGrowth
            } else if d >= RELEASED * g {
                PurgeVerdict::Released
            } else if d <= RETAINED * g {
                PurgeVerdict::Retained
            } else {
                PurgeVerdict::Partial
            }
        }
        _ => PurgeVerdict::NotRun,
    };
    Purge {
        kind: kind.to_owned(),
        before,
        after,
        growth,
        drop,
        verdict,
        applies: true,
    }
}

impl Purge {
    /// The evidence form; `applies` is written only when it is false.
    pub fn to_json(&self) -> Value {
        let mut v = json!({
            "kind": self.kind,
            "before": self.before.map(round),
            "after": self.after.map(round),
            "growth": self.growth.map(round),
            "drop": self.drop.map(round),
            "verdict": self.verdict.as_str(),
        });
        if !self.applies {
            v["applies"] = json!(false);
        }
        v
    }

    /// The same reading marked as one that does not apply to its column.
    pub fn not_applicable(mut self) -> Self {
        self.applies = false;
        self
    }

    /// The N-* hypothesis a release supports.
    pub fn supports(&self) -> &'static str {
        if self.kind == "gc" {
            "N-HEADROOM"
        } else {
            "N-ALLOC"
        }
    }
}

/// The gc test of a column with a `.pre_gc` twin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Gc {
    /// Rise of the pre-GC twin.
    pub pre_rise: f64,
    /// Rise of the post-GC column.
    pub post_rise: f64,
    /// True when the twin grows or swings (passes the gate, rises over three times as much,
    /// or is a sawtooth) while the post-GC column is flat.
    pub headroom: bool,
}

impl Gc {
    /// The evidence form.
    pub fn to_json(self) -> Value {
        json!({
            "pre_rise": round(self.pre_rise),
            "post_rise": round(self.post_rise),
            "verdict": if self.headroom { "headroom" } else { "none" },
        })
    }
}

/// The gc test over the post-warm-up points of a column and of its pre-GC twin.
pub fn gc(pre: &[(f64, f64)], post: &[(f64, f64)], reference: f64) -> Option<Gc> {
    let th = GateThresholds::default();
    let a = analyze(pre, None, false, &th);
    let b = analyze(post, None, false, &th);
    a.sen?;
    b.sen?;
    let level = mean_abs(post).max(reference);
    let post_flat = b.rise.abs() <= MATERIAL * level
        || (b.gate == GateVerdict::Fail
            && matches!(
                b.shape,
                TrendShape::Flat
                    | TrendShape::Plateau
                    | TrendShape::Declining
                    | TrendShape::Spike
                    | TrendShape::Insufficient
            ));
    // The twin either rises through the gate or swings (a sawtooth) far more than the
    // post-GC column, by an amount that matters.
    let sawtooth =
        a.noise > NOISE_MULTIPLE * b.noise.max(1e-9 * level) && a.noise > MATERIAL * level;
    let pre_grows = (a.gate == GateVerdict::Pass && a.rise > MATERIAL * level)
        || sawtooth
        || (a.rise > NOISE_MULTIPLE * a.noise.max(1e-9)
            && a.rise > NOISE_MULTIPLE * b.rise.max(0.0));
    Some(Gc {
        pre_rise: a.rise,
        post_rise: b.rise,
        headroom: post_flat && pre_grows,
    })
}

/// One column's combined verdict and the hypotheses it supports.
///
/// A purge that released the growth, or a pre-GC twin that carries it, favors a non-leak
/// cause (the memory is allocator or collector headroom). Otherwise a plateau favors
/// non-leak, growth that continues favors leak, and when the plateau test could not
/// decide a purge that kept the growth favors leak. A purge that kept the growth says only
/// that the growth is not headroom, not that it goes on: a plateau together with it stands when
/// the main window's rises, scaled to the extra window (`expected_rises`), say at least
/// [`MIN_EXPECTED_RISES`] more would have shown; below that, or with no count (`None`), the
/// plateau window may be too short to have shown one and the two are inconclusive. A purge
/// that does not apply to the column (`Purge::applies`: a region no purge gives back) is left
/// out altogether: the column is its plateau's, an `Inconclusive` one included.
pub fn combine(
    plateau: PlateauVerdict,
    purge: Option<&Purge>,
    gc: Option<Gc>,
    expected_rises: Option<f64>,
) -> (Favors, Vec<String>) {
    // A column that never grew in this series has nothing to release and nothing that stopped:
    // no purge, twin or plateau turns it into a non-leak.
    if plateau == PlateauVerdict::NeverGrew {
        return (Favors::Inconclusive, Vec::new());
    }
    // A purge that cannot give this column's memory back (file-backed pages under a heap trim)
    // neither released nor retained anything of it: the column is judged on its plateau alone.
    let purge = purge.filter(|p| p.applies);
    let released = purge.filter(|p| p.verdict == PurgeVerdict::Released);
    let headroom = gc.is_some_and(|g| g.headroom);
    if released.is_some() || headroom {
        let mut s: Vec<String> = Vec::new();
        if let Some(p) = released {
            s.push(p.supports().to_owned());
        }
        if headroom {
            s.push("N-HEADROOM".to_owned());
        }
        if plateau == PlateauVerdict::Plateau {
            s.push("N-WARM".to_owned());
            s.push("N-HWM".to_owned());
        }
        let mut seen: Vec<String> = Vec::new();
        for x in s {
            if !seen.contains(&x) {
                seen.push(x);
            }
        }
        return (Favors::NonLeak, seen);
    }
    match plateau {
        // A plateau window too short to have shown another rise does not override a purge
        // that kept the growth.
        PlateauVerdict::Plateau
            if purge.is_some_and(|p| p.verdict == PurgeVerdict::Retained)
                && expected_rises.is_none_or(|e| e < MIN_EXPECTED_RISES) =>
        {
            (Favors::Inconclusive, Vec::new())
        }
        PlateauVerdict::Plateau => (
            Favors::NonLeak,
            vec!["N-WARM".to_owned(), "N-HWM".to_owned()],
        ),
        PlateauVerdict::Continues => (Favors::Leak, vec!["leak".to_owned()]),
        PlateauVerdict::NeverGrew => (Favors::Inconclusive, Vec::new()),
        PlateauVerdict::Inconclusive | PlateauVerdict::NotRun => {
            if purge.is_some_and(|p| p.verdict == PurgeVerdict::Retained) {
                (Favors::Leak, vec!["leak".to_owned()])
            } else {
                (Favors::Inconclusive, Vec::new())
            }
        }
    }
}

/// The overall verdict over the voting columns: non-leak only when every one favors
/// non-leak, leak when any favors leak and none favors non-leak, else inconclusive.
pub fn overall(columns: &[(Favors, Vec<String>)]) -> (Favors, Vec<String>) {
    if columns.is_empty() {
        return (Favors::Inconclusive, Vec::new());
    }
    if columns.iter().all(|c| c.0 == Favors::NonLeak) {
        let mut s: Vec<String> = Vec::new();
        for (_, sup) in columns {
            for x in sup {
                if !s.contains(x) {
                    s.push(x.clone());
                }
            }
        }
        return (Favors::NonLeak, s);
    }
    let leak = columns.iter().any(|c| c.0 == Favors::Leak);
    let non = columns.iter().any(|c| c.0 == Favors::NonLeak);
    if leak && !non {
        (Favors::Leak, vec!["leak".to_owned()])
    } else {
        (Favors::Inconclusive, Vec::new())
    }
}

/// Holds a non-leak verdict back unless every `required` column favours non-leak too: a
/// column that keeps growing next to a plateaued primary (a category of native memory that
/// is not in the pools) leaves the result inconclusive. Any other verdict is unchanged.
pub fn require_non_leak(
    verdict: (Favors, Vec<String>),
    required: &[Favors],
) -> (Favors, Vec<String>) {
    if verdict.0 == Favors::NonLeak && required.iter().any(|f| *f != Favors::NonLeak) {
        (Favors::Inconclusive, Vec::new())
    } else {
        verdict
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_column_grew_in_the_main_window_by_its_interval_rise_and_counter_net_rise() {
        let flat = pts(&[80., 80., 80., 80., 80., 80.], 1000.0);
        let ramp = pts(&[10., 20., 30., 40., 50., 60.], 1000.0);
        assert!(!grew(&flat, false, 0.0, 0.0));
        assert!(grew(&ramp, false, 0.0, 0.0));
        // A rise below the column's smallest rise is none, and one point shows no rise.
        assert!(!grew(&ramp, false, 0.0, 1000.0));
        assert!(!grew(&ramp[..1], false, 0.0, 0.0));
        assert!(!grew(&flat, true, 0.0, 0.0));
        assert!(grew(&ramp, true, 0.0, 0.0));
        // The flat-then-flat column is a plateau for the plateau test, and nothing more.
        let extra = pts(&[80., 80., 80., 80.], 7000.0);
        assert_eq!(
            plateau(&flat, &extra, false, 0.0, 0.0).verdict,
            PlateauVerdict::Plateau
        );
    }

    #[test]
    fn a_column_that_never_grew_is_inconclusive_by_every_route_and_a_plateau_after_growth_is_not() {
        let released = purge("trim", Some(100.0), Some(40.0), Some(20.0), 0.0);
        let headroom = Gc {
            pre_rise: 9.0,
            post_rise: 0.0,
            headroom: true,
        };
        for (p, g) in [
            (None, None),
            (Some(&released), None),
            (Some(&released), Some(headroom)),
        ] {
            assert_eq!(
                combine(PlateauVerdict::NeverGrew, p, g, None),
                (Favors::Inconclusive, Vec::new())
            );
        }
        assert_eq!(PlateauVerdict::NeverGrew.as_str(), "never-grew");
        // A column that grew and then stopped is a non-leak as before.
        assert_eq!(
            combine(PlateauVerdict::Plateau, None, None, None).0,
            Favors::NonLeak
        );
        assert_eq!(
            combine(PlateauVerdict::Plateau, Some(&released), None, None).0,
            Favors::NonLeak
        );
    }

    fn pts(ys: &[f64], x0: f64) -> Vec<(f64, f64)> {
        ys.iter()
            .enumerate()
            .map(|(i, y)| (x0 + 1000.0 * i as f64, *y))
            .collect()
    }

    #[test]
    fn linear_growth_continues_and_a_flat_extra_window_is_a_plateau() {
        let main = pts(&[10., 20., 30., 40., 50., 60., 70., 80.], 1000.0);
        let grow = pts(&[90., 100., 110., 120., 130., 140., 150., 160.], 9000.0);
        let flat = pts(&[80., 80., 80., 80., 80., 80., 80., 80.], 9000.0);
        assert_eq!(
            plateau(&main, &grow, false, 0.0, 0.0).verdict,
            PlateauVerdict::Continues
        );
        assert_eq!(
            plateau(&main, &flat, false, 0.0, 0.0).verdict,
            PlateauVerdict::Plateau
        );
    }

    #[test]
    fn a_required_column_that_does_not_plateau_holds_a_non_leak_verdict_back() {
        let non = (Favors::NonLeak, vec!["N-WARM".to_owned()]);
        let kept = require_non_leak(non.clone(), &[Favors::NonLeak, Favors::NonLeak]);
        assert_eq!(kept, non);
        for bad in [Favors::Leak, Favors::Inconclusive] {
            let held = require_non_leak(non.clone(), &[Favors::NonLeak, bad]);
            assert_eq!(held, (Favors::Inconclusive, Vec::new()), "{bad:?}");
        }
        // Only a non-leak verdict is held back; a leak stays the primary's.
        let leak = (Favors::Leak, vec!["leak".to_owned()]);
        assert_eq!(require_non_leak(leak.clone(), &[Favors::NonLeak]), leak);
        assert_eq!(require_non_leak(non.clone(), &[]), non);
    }

    /// JVM non-heap bytes on a level of about 50 MB, sampled every 10 000 iterations.
    fn nonheap(from: usize, to: usize, f: impl Fn(f64) -> f64) -> Vec<(f64, f64)> {
        (from..to)
            .map(|i| {
                let x = 10_000.0 * (i as f64 + 1.0);
                (x, 50e6 + f(x))
            })
            .collect()
    }

    #[test]
    fn a_jit_shaped_non_heap_curve_that_has_levelled_off_is_a_plateau() {
        // Profiling data and compiled code: a quickly decaying rise to about 1.5 MB, then
        // nothing but a few KB of noise. Over the extra window it rises less than 32 KiB.
        let warm = |x: f64| 1.5e6 * (1.0 - (-x / 15_000.0).exp());
        let main = nonheap(0, 8, warm);
        let extra = nonheap(8, 16, warm);
        let p = plateau(&main, &extra, false, 0.0, MIN_BYTE_RISE);
        assert_eq!(p.verdict, PlateauVerdict::Plateau, "{p:?}");
        assert!(p.extra_rise.unwrap() < MIN_BYTE_RISE);
    }

    #[test]
    fn a_steady_per_iteration_non_heap_rise_keeps_its_slope_and_continues() {
        // 28 bytes per iteration for the whole run, as run 3 saw: 56 320 iterations are
        // 1.5 MB, more than 1% of the level.
        let steady = |x: f64| 28.0 * x;
        let main = nonheap(0, 8, steady);
        let extra = nonheap(8, 16, steady);
        let p = plateau(&main, &extra, false, 0.0, MIN_BYTE_RISE);
        assert_eq!(p.verdict, PlateauVerdict::Continues, "{p:?}");
        // A real Metaspace leak, a class of about a kilobyte per iteration, is far above it.
        let leak = |x: f64| 1_000.0 * x;
        let p = plateau(
            &nonheap(0, 8, leak),
            &nonheap(8, 16, leak),
            false,
            0.0,
            MIN_BYTE_RISE,
        );
        assert_eq!(p.verdict, PlateauVerdict::Continues, "{p:?}");
    }

    #[test]
    fn a_short_extra_window_is_not_run() {
        let main = pts(&[1., 2., 3., 4.], 1000.0);
        let extra = pts(&[5., 6.], 5000.0);
        assert_eq!(
            plateau(&main, &extra, false, 0.0, 0.0).verdict,
            PlateauVerdict::NotRun
        );
        assert_eq!(
            plateau(&main, &[], true, 0.0, 0.0).verdict,
            PlateauVerdict::NotRun
        );
    }

    #[test]
    fn counters_use_the_every_delta_rule_and_the_net_increase() {
        let main = pts(&[10., 20., 30., 40.], 1000.0);
        let grow = pts(&[50., 60., 70., 80.], 5000.0);
        let flat = pts(&[40., 40., 40., 40.], 5000.0);
        let wobble = pts(&[41., 40., 42., 41.], 5000.0);
        let climb = pts(&[44., 50., 49., 56., 62., 61., 68.], 5000.0);
        assert_eq!(
            plateau(&main, &grow, true, 0.0, 0.0).verdict,
            PlateauVerdict::Continues
        );
        assert_eq!(
            plateau(&main, &flat, true, 0.0, 0.0).verdict,
            PlateauVerdict::Plateau
        );
        assert_eq!(
            plateau(&main, &wobble, true, 0.0, 0.0).verdict,
            PlateauVerdict::Plateau
        );
        assert_eq!(
            plateau(&main, &climb, true, 0.0, 0.0).verdict,
            PlateauVerdict::Inconclusive
        );
    }

    #[test]
    fn the_purge_test_compares_the_drop_with_the_growth() {
        assert_eq!(
            purge("trim", Some(100.0), Some(40.0), Some(20.0), 0.0).verdict,
            PurgeVerdict::Released
        );
        assert_eq!(
            purge("trim", Some(100.0), Some(98.0), Some(20.0), 0.0).verdict,
            PurgeVerdict::Retained
        );
        assert_eq!(
            purge("trim", Some(100.0), Some(75.0), Some(20.0), 0.0).verdict,
            PurgeVerdict::Partial
        );
        assert_eq!(
            purge("trim", Some(10.0), Some(8.0), Some(20.0), 0.0).verdict,
            PurgeVerdict::NoGrowth
        );
        assert_eq!(
            purge("trim", None, Some(8.0), Some(20.0), 0.0).verdict,
            PurgeVerdict::NotRun
        );
    }

    #[test]
    fn a_growth_at_or_below_the_columns_minimum_rise_is_no_growth_for_the_purge() {
        let min = MIN_BYTE_RISE;
        let warm = 1.0e6;
        // The Go shape: a few KB of live objects since the warm-up, the purge gives none back.
        for rise in [0.5 * min, min - 1.0, min] {
            let p = purge(
                "free-os-memory",
                Some(warm + rise),
                Some(warm + rise + 100.0),
                Some(warm),
                min,
            );
            assert_eq!(p.verdict, PurgeVerdict::NoGrowth, "rise {rise}");
        }
        // Above the minimum the verdicts are the old ones.
        let above = min + 1.0;
        assert_eq!(
            purge(
                "free-os-memory",
                Some(warm + above),
                Some(warm + above + 100.0),
                Some(warm),
                min
            )
            .verdict,
            PurgeVerdict::Retained
        );
        assert_eq!(
            purge(
                "free-os-memory",
                Some(warm + above),
                Some(warm),
                Some(warm),
                min
            )
            .verdict,
            PurgeVerdict::Released
        );
        assert_eq!(
            purge(
                "free-os-memory",
                Some(warm + above),
                Some(warm + above * 0.65),
                Some(warm),
                min
            )
            .verdict,
            PurgeVerdict::Partial
        );
        // A column with no minimum keeps the old rule: any growth counts.
        assert_eq!(
            purge(
                "trim",
                Some(warm + 17_000.0),
                Some(warm + 17_100.0),
                Some(warm),
                0.0
            )
            .verdict,
            PurgeVerdict::Retained
        );
    }

    #[test]
    fn a_released_purge_beats_continuing_growth() {
        let p = purge("trim", Some(100.0), Some(30.0), Some(20.0), 0.0);
        let (f, s) = combine(PlateauVerdict::Continues, Some(&p), None, None);
        assert_eq!(f, Favors::NonLeak);
        assert_eq!(s, ["N-ALLOC"]);
        let g = purge("gc", Some(100.0), Some(30.0), Some(20.0), 0.0);
        assert_eq!(
            combine(PlateauVerdict::Plateau, Some(&g), None, None).1,
            ["N-HEADROOM", "N-WARM", "N-HWM"]
        );
    }

    fn line(ys: &[f64]) -> Vec<Point> {
        ys.iter()
            .enumerate()
            .map(|(i, &y)| (1000.0 * i as f64, y))
            .collect()
    }

    #[test]
    fn the_rises_of_a_window_are_the_new_maxima_over_the_noise() {
        // A ramp: every point sets a new maximum by 100, over a noise of 10.
        let ramp = line(&[100.0, 200.0, 300.0, 400.0, 500.0, 600.0]);
        assert_eq!(count_rises(&ramp, Some(0.0), 10.0), 6);
        // From the first point itself when nothing precedes it.
        assert_eq!(count_rises(&ramp, None, 10.0), 5);
        // One step, then flat with jitter under the noise.
        let step = line(&[100.0, 100.0, 500.0, 505.0, 497.0, 503.0]);
        assert_eq!(count_rises(&step, Some(100.0), 10.0), 1);
        // A dip and a return to the old level is no rise; nothing rises on a flat line.
        let dip = line(&[500.0, 300.0, 500.0, 500.0]);
        assert_eq!(count_rises(&dip, Some(500.0), 10.0), 0);
        assert_eq!(count_rises(&[], Some(1.0), 10.0), 0);
        // The expectation scales with the windows: 6 rises over 5000 of x; an extra window of
        // 2500 (6 points of 500) expects 3 and is just long enough.
        let main = line(&[100.0, 200.0, 300.0, 400.0, 500.0, 600.0]);
        let extra: Vec<Point> = (0..6).map(|i| (6000.0 + 500.0 * i as f64, 600.0)).collect();
        let r = rises(&main, &extra, Some(0.0), 10.0).unwrap();
        assert_eq!(r.count, 6);
        assert!((r.expected - 3.0).abs() < 1e-9, "{r:?}");
        assert_eq!(r.points_needed, Some(6));
        // Without a rise there is nothing to scale.
        let flat = line(&[100.0; 6]);
        let r = rises(&flat, &extra, Some(100.0), 10.0).unwrap();
        assert_eq!((r.count, r.expected, r.points_needed), (0, 0.0, None));
    }

    /// Points of a byte column in MB, one every 440 iterations from the `first`-th.
    fn mb(first: usize, ys: &[f64]) -> Vec<Point> {
        ys.iter()
            .enumerate()
            .map(|(i, y)| (440.0 * (first + i) as f64, y * 1e6))
            .collect()
    }

    /// The main window of a `Vec` that doubles its capacity, shaped as a measured run's
    /// allocator-shim column was (8 points, the noise of its own steps): 10 MB, a step to 70 and
    /// a step to 130.
    fn doubling_main() -> Vec<Point> {
        mb(1, &[10., 10., 10., 10., 70., 70., 70., 130.])
    }

    #[test]
    fn a_column_that_grows_in_steps_and_reaches_a_new_maximum_in_the_extra_window_continues() {
        // The measured case: the extra window (16 points) rose 62 MB, under three noise units
        // (the steps themselves are the scatter) and at under half the main slope; the old rule
        // read that as a plateau. The extra window holds a new step, 54 MB over the main
        // window's maximum and the noise 22 MB.
        let main = doubling_main();
        let extra = mb(
            9,
            &[
                130., 130., 130., 130., 130., 130., 130., 130., 184., 184., 184., 184., 184., 184.,
                184., 184.,
            ],
        );
        let p = plateau(&main, &extra, false, 0.0, MIN_BYTE_RISE);
        let (rise, noise) = (p.extra_rise.unwrap(), p.noise.unwrap());
        assert!(rise > noise && rise <= NOISE_MULTIPLE * noise, "{p:?}");
        assert!(
            p.extra_slope.unwrap() < KEEP_SLOPE * p.main_slope.unwrap(),
            "{p:?}"
        );
        assert_eq!(p.verdict, PlateauVerdict::Continues, "{p:?}");
        assert_eq!(p.points_needed, None);
        // The same column that did not rise in the extra window is a plateau.
        let flat = mb(9, &[130.0; 16]);
        let p = plateau(&main, &flat, false, 0.0, MIN_BYTE_RISE);
        assert_eq!(p.verdict, PlateauVerdict::Plateau, "{p:?}");
    }

    #[test]
    fn a_stair_whose_extra_window_is_shorter_than_its_step_gap_is_inconclusive_and_says_how_long() {
        // Steps at x = 2200 and 3520: a gap of 1320, so the next is expected 2 * (1320 + 440) =
        // 3520 after the last, at 7040. Four points end at 5720, 2200 after it, and none passes
        // the main window's maximum plus its noise: a step may simply not have come yet.
        let main = doubling_main();
        let extra = mb(10, &[100., 115., 130., 148.]);
        let p = plateau(&main, &extra, false, 0.0, MIN_BYTE_RISE);
        assert!(p.extra_rise.unwrap() > p.noise.unwrap(), "{p:?}");
        assert_eq!(p.verdict, PlateauVerdict::Inconclusive, "{p:?}");
        // Eight points end at 7480, past 7040: the window that would cover it.
        assert_eq!(p.points_needed, Some(8), "{p:?}");
        let longer = mb(10, &[100., 115., 130., 148., 140., 130., 120., 125.]);
        let q = plateau(&main, &longer, false, 0.0, MIN_BYTE_RISE);
        assert_eq!(q.verdict, PlateauVerdict::Plateau, "{q:?}");
        assert_eq!(q.points_needed, None);
    }

    #[test]
    fn a_stair_that_stays_at_or_under_its_maximum_for_longer_than_its_expected_gap_is_a_plateau() {
        // Ten points end at 7920, 4400 after the last step and over the 3520 the next was
        // expected after it, and rise clearly (a slow recovery to the old level) without a new
        // maximum: the growth stopped.
        let main = doubling_main();
        let extra = mb(
            9,
            &[90., 95., 100., 105., 110., 115., 120., 125., 128., 129.],
        );
        let p = plateau(&main, &extra, false, 0.0, MIN_BYTE_RISE);
        assert!(p.extra_rise.unwrap() > p.noise.unwrap(), "{p:?}");
        assert_eq!(p.verdict, PlateauVerdict::Plateau, "{p:?}");
        assert_eq!(p.points_needed, None);
    }

    /// The allocator-shim column of a measured run (a `Vec` that doubles its capacity, in bytes):
    /// nine points in the main window, eight in the extra window (its step at 7040).
    fn measured_vec() -> (Vec<Point>, Vec<Point>) {
        let at = |first: usize, ys: &[f64]| -> Vec<Point> {
            ys.iter()
                .enumerate()
                .map(|(i, y)| (440.0 * (first + i) as f64, *y))
                .collect()
        };
        let main = at(
            1,
            &[
                2_663_109.,
                3_646_149.,
                5_612_229.,
                5_612_229.,
                9_544_389.,
                9_544_389.,
                9_544_389.,
                17_408_709.,
                17_408_709.,
            ],
        );
        let extra = at(
            10,
            &[
                17_408_709.,
                17_408_709.,
                17_408_709.,
                17_408_709.,
                17_408_709.,
                17_408_709.,
                33_137_349.,
                33_137_525.,
            ],
        );
        (main, extra)
    }

    #[test]
    fn a_late_step_in_the_extra_window_is_growth_though_the_robust_slope_is_zero() {
        // The measured case: six flat points and two high ones give a Theil-Sen slope of zero, so
        // the extra rise is zero and the old guard (a clear rise) never read the steps. The window
        // ends 15.7 MB over the main window's maximum, the noise 1.5 MB.
        let (main, extra) = measured_vec();
        let p = plateau(&main, &extra, false, 0.0, MIN_BYTE_RISE);
        assert_eq!(p.extra_rise, Some(0.0), "{p:?}");
        assert_eq!(p.verdict, PlateauVerdict::Continues, "{p:?}");
        assert_eq!(p.points_needed, None);
        // A spike: the same point high, and back at the old level at the end of the window.
        let mut spike = extra.clone();
        spike[6].1 = 33_137_349.0;
        spike[7].1 = 17_408_709.0;
        let p = plateau(&main, &spike, false, 0.0, MIN_BYTE_RISE);
        assert_ne!(p.verdict, PlateauVerdict::Continues, "{p:?}");
        assert_ne!(stair(&main, &spike, p.noise.unwrap()), Some(Stair::NewMax));
        // A new maximum under the column's minimum rise is not growth.
        let small: Vec<Point> = extra
            .iter()
            .map(|&(x, y)| (x, if y > 2e7 { 17_408_709.0 + 2e6 } else { y }))
            .collect();
        let p = plateau(&main, &small, false, 0.0, 3e6);
        assert_ne!(p.verdict, PlateauVerdict::Continues, "{p:?}");
    }

    #[test]
    fn a_staircase_whose_gaps_grow_is_not_over_before_twice_its_last_gap() {
        // The same column cut after 6600: no step yet, 3080 since the last one (at 3520), where
        // the gaps (440, 880, 1320) call for 2 * (1320 + 440) = 3520. Not a plateau: the window
        // that would decide it runs past 7040.
        let (main, extra) = measured_vec();
        let cut = &extra[..6];
        assert_eq!(cut.last().unwrap().0, 6600.0);
        let p = plateau(&main, cut, false, 0.0, MIN_BYTE_RISE);
        assert_eq!(p.verdict, PlateauVerdict::Inconclusive, "{p:?}");
        let n = p.points_needed.unwrap();
        assert_eq!(n, 8, "{p:?}");
        assert!(4400.0 + 440.0 * f64::from(n - 1) > 7040.0);
        // The window that long holds the step.
        let whole = plateau(&main, &extra[..n as usize], false, 0.0, MIN_BYTE_RISE);
        assert_eq!(whole.verdict, PlateauVerdict::Continues, "{whole:?}");
    }

    #[test]
    fn a_staircase_that_stays_flat_past_twice_its_last_gap_is_a_plateau() {
        let (main, _) = measured_vec();
        // Ends at 7480: 3960 since the step at 3520, over 3520.
        let flat: Vec<Point> = (0..8)
            .map(|i| (4400.0 + 440.0 * f64::from(i), 17_408_709.0))
            .collect();
        assert_eq!(stair(&main, &flat, 1.5e6), Some(Stair::Stopped));
        let p = plateau(&main, &flat, false, 0.0, MIN_BYTE_RISE);
        assert_eq!(p.verdict, PlateauVerdict::Plateau, "{p:?}");
        // At exactly the expected end (7040) a step may still be arriving.
        assert!(matches!(
            stair(&main, &flat[..7], 1.5e6),
            Some(Stair::TooShort { .. })
        ));
    }

    #[test]
    fn a_ramp_keeps_its_ordinary_verdict_with_a_late_point_too() {
        // A rise at every point is no staircase (`stair` is None, see the test of that name): the
        // ordinary test reads it, a flat extra window is a plateau and a line goes on.
        let ramp = mb(1, &[10., 30., 50., 70., 90., 110., 130., 150.]);
        let flat = mb(9, &[150.0; 8]);
        assert_eq!(
            plateau(&ramp, &flat, false, 0.0, MIN_BYTE_RISE).verdict,
            PlateauVerdict::Plateau
        );
        let on = mb(9, &[170., 190., 210., 230., 250., 270., 290., 310.]);
        assert_eq!(
            plateau(&ramp, &on, false, 0.0, MIN_BYTE_RISE).verdict,
            PlateauVerdict::Continues
        );
    }

    #[test]
    fn a_column_that_rose_in_one_step_or_at_every_point_keeps_its_ordinary_verdict() {
        let extra = mb(
            9,
            &[
                130., 130., 130., 130., 130., 130., 130., 130., 184., 184., 184., 184., 184., 184.,
                184., 184.,
            ],
        );
        // One step is no staircase: there is no gap to measure the extra window against.
        let one = mb(1, &[10., 10., 10., 10., 10., 10., 10., 130.]);
        assert_eq!(stair(&one, &extra, 22e6), None);
        // A ramp rises at every point (the noise is small): the ordinary test serves.
        let ramp = mb(1, &[10., 30., 50., 70., 90., 110., 130., 150.]);
        assert_eq!(stair(&ramp, &extra, 5e6), None);
        // And the rule needs a clear rise of the extra window to apply at all.
        let flat = mb(9, &[130.0; 16]);
        assert_eq!(
            stair(&doubling_main(), &flat, 22e6),
            Some(Stair::Stopped),
            "a flat window is a plateau by the step reading too"
        );
    }

    #[test]
    fn a_retained_purge_holds_a_plateau_back_only_while_its_window_is_short() {
        let kept = purge("trim", Some(100.0), Some(99.0), Some(20.0), 0.0);
        let held = |e| combine(PlateauVerdict::Plateau, Some(&kept), None, e);
        assert_eq!(held(Some(2.9)), (Favors::Inconclusive, Vec::new()));
        assert_eq!(held(None), (Favors::Inconclusive, Vec::new()));
        let (f, s) = held(Some(MIN_EXPECTED_RISES));
        assert_eq!(f, Favors::NonLeak);
        assert_eq!(s, ["N-WARM", "N-HWM"]);
        // Growth that continues is still a leak, whatever the count says.
        assert_eq!(
            combine(PlateauVerdict::Continues, Some(&kept), None, Some(9.0)).0,
            Favors::Leak
        );
    }

    #[test]
    fn plateau_and_retained_growth_decide_the_rest() {
        let kept = purge("trim", Some(100.0), Some(99.0), Some(20.0), 0.0);
        // A plateau never overrides a purge that kept the growth: they disagree.
        assert_eq!(
            combine(PlateauVerdict::Plateau, Some(&kept), None, None),
            (Favors::Inconclusive, Vec::new())
        );
        assert_eq!(
            combine(PlateauVerdict::Plateau, None, None, None).0,
            Favors::NonLeak
        );
        assert_eq!(
            combine(PlateauVerdict::Continues, Some(&kept), None, None).0,
            Favors::Leak
        );
        assert_eq!(
            combine(PlateauVerdict::NotRun, Some(&kept), None, None).0,
            Favors::Leak
        );
        assert_eq!(
            combine(PlateauVerdict::Inconclusive, None, None, None).0,
            Favors::Inconclusive
        );
    }

    #[test]
    fn a_purge_that_does_not_apply_to_the_column_leaves_it_to_its_plateau() {
        // Retained, as the page cache of mapped files is by a heap trim: the reading says
        // nothing of the column, so an undecided plateau is not read as a leak.
        let kept = purge("trim", Some(100.0), Some(99.0), Some(20.0), 0.0).not_applicable();
        assert_eq!(kept.verdict, PurgeVerdict::Retained);
        for plateau in [PlateauVerdict::Inconclusive, PlateauVerdict::NotRun] {
            assert_eq!(
                combine(plateau, Some(&kept), None, None),
                (Favors::Inconclusive, Vec::new()),
                "{plateau:?}"
            );
        }
        // A plateau stands whatever the count of rises says (nothing is held against it).
        assert_eq!(
            combine(PlateauVerdict::Plateau, Some(&kept), None, Some(0.0)).0,
            Favors::NonLeak
        );
        // A release that cannot be the purge's does not turn growth into a non-leak.
        let released = purge("trim", Some(100.0), Some(40.0), Some(20.0), 0.0).not_applicable();
        assert_eq!(
            combine(PlateauVerdict::Continues, Some(&released), None, None).0,
            Favors::Leak
        );
        assert_eq!(
            combine(PlateauVerdict::Inconclusive, Some(&released), None, None).0,
            Favors::Inconclusive
        );
        // The reading stays in the evidence, marked.
        assert_eq!(kept.to_json()["applies"], false);
        assert_eq!(kept.to_json()["verdict"], "retained");
        assert!(
            purge("trim", Some(100.0), Some(99.0), Some(20.0), 0.0)
                .to_json()
                .get("applies")
                .is_none()
        );
        // The same reading of a column the purge does apply to is a leak, as before.
        let applies = purge("trim", Some(100.0), Some(99.0), Some(20.0), 0.0);
        assert_eq!(
            combine(PlateauVerdict::Inconclusive, Some(&applies), None, None),
            (Favors::Leak, vec!["leak".to_owned()])
        );
        assert_eq!(
            combine(PlateauVerdict::Continues, Some(&kept), None, None).0,
            Favors::Leak
        );
    }

    #[test]
    fn a_growing_pre_gc_twin_over_a_flat_column_is_headroom() {
        let pre = pts(&[10., 30., 20., 50., 40., 70., 60., 90.], 1000.0);
        let post = pts(&[5., 5., 5., 5., 5., 5., 5., 5.], 1000.0);
        let g = gc(&pre, &post, 0.0).unwrap();
        assert!(g.headroom, "{g:?}");
        let both = pts(&[10., 20., 30., 40., 50., 60., 70., 80.], 1000.0);
        assert!(!gc(&both, &both, 0.0).unwrap().headroom);
    }

    #[test]
    fn a_smooth_drift_far_below_one_percent_of_the_level_is_not_growth() {
        let main = pts(&[3000000., 3000100., 3000200., 3000300.], 1000.0);
        let drift = pts(&[3000400., 3000500., 3000601., 3000700., 3000800.], 5000.0);
        // The main window grows with confidence, so the floor alone cannot call a plateau.
        let p = plateau(&main, &drift, false, 0.0, 0.0);
        assert_eq!(p.verdict, PlateauVerdict::Inconclusive, "{p:?}");
        // With a main window that does not grow with confidence the floor still applies.
        let still = pts(&[3000000., 3000300., 3000100., 3000200.], 1000.0);
        let p = plateau(&still, &drift, false, 0.0, 0.0);
        assert_eq!(p.verdict, PlateauVerdict::Plateau, "{p:?}");
    }

    #[test]
    fn the_main_windows_scatter_bounds_the_extra_windows_noise() {
        // go-buffer-pool under load (a measured run): the live heap of a correct program
        // scatters by a few KB between points. The extra window alone reads a MAD noise of
        // 2.7 KB, so its rise of 13.4 KB passed three noise units.
        let at = |xs: &[f64], ys: &[f64]| -> Vec<(f64, f64)> {
            xs.iter().copied().zip(ys.iter().copied()).collect()
        };
        let main = at(
            &[440., 880., 1320., 1760., 2200., 2640., 3080., 3520., 3960.],
            &[
                167144., 173448., 173704., 178488., 175000., 178312., 170456., 175976., 172488.,
            ],
        );
        let extra = at(
            &[4400., 4840., 5280., 5720., 6160., 6600., 7040., 7480.],
            &[
                175784., 173880., 183128., 181960., 190376., 185256., 173240., 189640.,
            ],
        );
        let alone = plateau(&[], &extra, false, 0.0, 0.0);
        assert!(
            alone.extra_rise.unwrap() > NOISE_MULTIPLE * alone.noise.unwrap(),
            "{alone:?}"
        );
        assert_eq!(alone.verdict, PlateauVerdict::Continues, "{alone:?}");
        // With the main window's scatter the same rise is within the noise.
        let p = plateau(&main, &extra, false, 0.0, 0.0);
        assert!(p.noise.unwrap() > alone.noise.unwrap(), "{p:?}");
        assert_eq!(p.verdict, PlateauVerdict::Plateau, "{p:?}");
    }

    fn shift(points: &[(f64, f64)], x0: f64) -> Vec<(f64, f64)> {
        points.iter().map(|&(x, y)| (x0 + x, y)).collect()
    }

    #[test]
    fn a_byte_rise_under_the_minimum_is_a_plateau_however_clean() {
        // A smooth climb of 21 KB over the extra window: above its noise, above 1% of the
        // level, and under the minimum rise of a byte column.
        let main = pts(&[165000., 166000., 167000., 168000., 169000.], 440.0);
        let extra = pts(
            &[
                170000., 173000., 176000., 179000., 182000., 185000., 188000., 191000.,
            ],
            440.0,
        );
        let extra = shift(&extra, 3000.0);
        let free = plateau(&main, &extra, false, 0.0, 0.0);
        assert_eq!(free.verdict, PlateauVerdict::Continues, "{free:?}");
        let p = plateau(&main, &extra, false, 0.0, MIN_BYTE_RISE);
        assert_eq!(p.verdict, PlateauVerdict::Plateau, "{p:?}");
        // The same slope over a longer window is a rise over the minimum.
        let long = pts(
            &[
                170000., 173000., 176000., 179000., 182000., 185000., 188000., 191000., 194000.,
                197000., 200000., 203000.,
            ],
            440.0,
        );
        let long = shift(&long, 3000.0);
        let p = plateau(&main, &long, false, 0.0, MIN_BYTE_RISE);
        assert_eq!(p.verdict, PlateauVerdict::Continues, "{p:?}");
    }

    #[test]
    fn a_clean_leak_keeps_its_verdict_with_the_main_windows_scatter_counted() {
        // A straight climb has no scatter, so the larger-of rule changes nothing.
        let main = pts(&[165000., 166000., 167000., 168000., 169000.], 440.0);
        let extra = pts(
            &[
                170000., 173000., 176000., 179000., 182000., 185000., 188000., 191000.,
            ],
            3960.0,
        );
        let p = plateau(&main, &extra, false, 0.0, 0.0);
        assert_eq!(p.verdict, PlateauVerdict::Continues, "{p:?}");
    }

    #[test]
    fn a_main_window_shorter_than_three_points_adds_no_noise() {
        let main = pts(&[1000., 1100.], 440.0);
        let extra = pts(&[1200., 1300., 1400., 1500.], 3960.0);
        let with = plateau(&main, &extra, false, 0.0, 0.0);
        let without = plateau(&[], &extra, false, 0.0, 0.0);
        assert_eq!(with.noise, without.noise);
    }

    #[test]
    fn a_precise_leak_under_one_percent_of_rss_is_not_a_plateau() {
        // 500 MB process, a probe leaking 30 KB per step: clean, significant growth that
        // the RSS-relative floor used to call a plateau. With no reference (a non-os
        // column) it continues.
        let main = pts(&[1e6, 1.03e6, 1.06e6, 1.09e6, 1.12e6], 1000.0);
        let extra = pts(&[1.15e6, 1.18e6, 1.21e6, 1.24e6, 1.27e6], 6000.0);
        assert_eq!(
            plateau(&main, &extra, false, 0.0, 0.0).verdict,
            PlateauVerdict::Continues
        );
        // Even if a reference is supplied, the floor alone must not yield a plateau.
        assert_eq!(
            plateau(&main, &extra, false, 500e6, 0.0).verdict,
            PlateauVerdict::Inconclusive
        );
    }

    #[test]
    fn windows_split_by_x_not_by_count() {
        let all = pts(&[1., 2., 3., 4., 5., 6.], 1000.0);
        let (m, e) = split(&all, 4000.0);
        assert_eq!((m.len(), e.len()), (4, 2));
        // A column that only exists in the extra part has no main points.
        let late = &all[4..];
        let (m, e) = split(late, 4000.0);
        assert_eq!((m.len(), e.len()), (0, 2));
        // A missing main reading does not move the boundary.
        let gap: Vec<_> = all.iter().copied().filter(|p| p.0 != 2000.0).collect();
        let (m, e) = split(&gap, 4000.0);
        assert_eq!((m.len(), e.len()), (3, 2));
    }

    #[test]
    fn a_sawtooth_twin_over_a_creeping_column_is_headroom() {
        let pre = pts(&[5e6, 8e6, 3e6, 6e6, 9e6, 4e6, 7e6, 3.5e6], 1000.0);
        let post = pts(
            &[
                3000000., 3000100., 3000200., 3000300., 3000400., 3000500., 3000600., 3000700.,
            ],
            1000.0,
        );
        assert!(gc(&pre, &post, 0.0).unwrap().headroom);
    }

    #[test]
    fn a_rise_under_one_percent_of_the_process_is_drift_for_a_small_column() {
        let main = pts(&[1000., 1100., 1200., 1300.], 1000.0);
        let creep = pts(&[1400., 1500., 1600., 1700., 1800.], 5000.0);
        assert_eq!(
            plateau(&main, &creep, false, 0.0, 0.0).verdict,
            PlateauVerdict::Continues
        );
        assert_eq!(
            plateau(&main, &creep, false, 50e6, 0.0).verdict,
            PlateauVerdict::Inconclusive
        );
    }

    #[test]
    fn overall_needs_every_column_for_non_leak() {
        let n = (Favors::NonLeak, vec!["N-WARM".to_owned()]);
        let l = (Favors::Leak, vec!["leak".to_owned()]);
        let i = (Favors::Inconclusive, Vec::new());
        assert_eq!(overall(&[n.clone(), n.clone()]).0, Favors::NonLeak);
        assert_eq!(overall(&[n.clone(), i.clone()]).0, Favors::Inconclusive);
        assert_eq!(overall(&[l.clone(), i]).0, Favors::Leak);
        assert_eq!(overall(&[l, n]).0, Favors::Inconclusive);
        assert_eq!(overall(&[]).0, Favors::Inconclusive);
    }
}
