//! The dump-series rule and the plateau discriminator: pure functions over the retained
//! bytes and dominated counts of one holder in successive dumps.

use dla_core::discriminator::Favors;
use serde_json::{Value, json};

/// Fewest dumps that can confirm growth.
pub const MIN_DUMPS: usize = 3;
/// The last interval's byte increase must be at least this share of the first's.
pub const LAST_OVER_FIRST: f64 = 0.5;
/// An increase at most this share of the first interval's counts as stopped.
pub const STOPPED_SHARE: f64 = 0.05;

/// One holder's figures per dump, oldest first; `None` where the dump has no figure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Observed {
    /// Retained bytes.
    pub bytes: Vec<Option<u64>>,
    /// Dominated object count.
    pub counts: Vec<Option<u64>>,
}

/// Why a holder passes.
#[derive(Debug, Clone, PartialEq)]
pub struct Growth {
    /// Byte increase per interval.
    pub byte_deltas: Vec<u64>,
    /// Count increase per interval.
    pub count_deltas: Vec<u64>,
    /// Last interval's byte increase over the first's.
    pub last_over_first: f64,
    /// The count was the same positive number at every dump (one block, or a fixed set of
    /// blocks, whose bytes grew: a buffer that is resized, a `Vec` field, a `byte[]` behind a
    /// list of primitives): only the bytes were held to rise.
    pub resized: bool,
}

/// The same positive number at every dump: what a block that is resized, not added to, shows.
fn constant_count(counts: &[u64]) -> bool {
    counts.first().is_some_and(|&c| c > 0) && counts.iter().all(|&c| c == counts[0])
}

fn deltas(v: &[u64]) -> Vec<i128> {
    v.windows(2)
        .map(|w| i128::from(w[1]) - i128::from(w[0]))
        .collect()
}

fn all_present(v: &[Option<u64>]) -> Option<Vec<u64>> {
    v.iter().copied().collect()
}

/// Applies the dump-series rule: at least [`MIN_DUMPS`] dumps, retained bytes strictly
/// increasing across every consecutive pair and the dominated count either strictly increasing
/// too (a holder that keeps more objects) or the same positive number at every dump (one block,
/// or a fixed set of blocks, that grows: its [`Growth::resized`] is set), and the last
/// interval's byte increase at least [`LAST_OVER_FIRST`] of the first's. A count that rises at
/// some pairs and not at others fails, as does a count of zero.
pub fn gate(o: &Observed) -> Result<Growth, Vec<String>> {
    gate_at(o, None)
}

/// [`gate`] over points at the given `x` positions (strictly increasing, one per dump):
/// the last-over-first comparison is then between the rates (bytes per unit of x) of the
/// last and the first interval, so unevenly spaced dumps do not bias it. With `None` the
/// dumps are one unit apart.
pub fn gate_at(o: &Observed, xs: Option<&[f64]>) -> Result<Growth, Vec<String>> {
    let n = o.bytes.len();
    if n < MIN_DUMPS {
        return Err(vec![format!("dumps < {MIN_DUMPS}")]);
    }
    let (Some(bytes), Some(counts)) = (all_present(&o.bytes), all_present(&o.counts)) else {
        let missing: Vec<String> = (0..n)
            .filter(|&i| o.bytes[i].is_none() || o.counts[i].is_none())
            .map(|i| (i + 1).to_string())
            .collect();
        return Err(vec![format!(
            "holder absent from dump {}",
            missing.join(", ")
        )]);
    };
    let db = deltas(&bytes);
    let dc = deltas(&counts);
    let mut why = Vec::new();
    if let Some(i) = db.iter().position(|&d| d <= 0) {
        why.push(format!(
            "retained bytes do not increase from dump {} to {}",
            i + 1,
            i + 2
        ));
    }
    let resized = constant_count(&counts);
    if !resized && let Some(i) = dc.iter().position(|&d| d <= 0) {
        why.push(format!(
            "dominated count does not increase from dump {} to {} (and is not one constant number of blocks whose bytes grow)",
            i + 1,
            i + 2
        ));
    }
    if !why.is_empty() {
        return Err(why);
    }
    let mut first = db[0] as f64;
    let mut last = db[db.len() - 1] as f64;
    let rated = match xs {
        Some(xs) if xs.len() == n => {
            let (dx_first, dx_last) = (xs[1] - xs[0], xs[n - 1] - xs[n - 2]);
            if dx_first > 0.0 && dx_last > 0.0 {
                first /= dx_first;
                last /= dx_last;
                true
            } else {
                return Err(vec![
                    "the x positions of the dumps do not increase".to_owned(),
                ]);
            }
        }
        _ => false,
    };
    let ratio = last / first;
    if ratio < LAST_OVER_FIRST {
        return Err(vec![format!(
            "last interval's {} is {:.0}% of the first's (needs >= {:.0}%)",
            if rated {
                "rate of increase per unit of x"
            } else {
                "increase"
            },
            ratio * 100.0,
            LAST_OVER_FIRST * 100.0
        )]);
    }
    Ok(Growth {
        byte_deltas: db.iter().map(|&d| d as u64).collect(),
        count_deltas: dc.iter().map(|&d| d as u64).collect(),
        last_over_first: ratio,
        resized,
    })
}

/// What to do when [`stood_still`] is true for a series.
pub const STOOD_STILL_NEEDED: &str = "a holder keeps one block (the same count at every dump) whose bytes grew but stood still between two dumps: a buffer that is resized is flat between resizes, so take dumps whose work differs by a factor of two or more, so that a resize falls between every pair";

/// Whether the series is one block (or a fixed set of blocks: the same positive count at every
/// dump) whose bytes rose from the first dump to the last but did not rise at every pair: a
/// buffer that is resized stands still between resizes, and the dumps fell between two of them.
pub fn stood_still(o: &Observed) -> bool {
    let (Some(bytes), Some(counts)) = (all_present(&o.bytes), all_present(&o.counts)) else {
        return false;
    };
    bytes.len() >= MIN_DUMPS
        && constant_count(&counts)
        && bytes[bytes.len() - 1] > bytes[0]
        && bytes.windows(2).any(|w| w[1] <= w[0])
}

/// `steady` when the byte increases stay within a factor of two of each other, else `stair`
/// (the shapes of a trend: a result's `shape` is read back as one). A block that is resized
/// says so in its own field ([`Growth::resized`], `"resized": true` in the passed column).
pub fn shape(g: &Growth) -> &'static str {
    let min = g.byte_deltas.iter().copied().min().unwrap_or(0) as f64;
    let max = g.byte_deltas.iter().copied().max().unwrap_or(0) as f64;
    if max > 0.0 && min >= 0.5 * max {
        "steady"
    } else {
        "stair"
    }
}

/// The `dump-plateau` verdict for one holder.
#[derive(Debug, Clone, PartialEq)]
pub struct Plateau {
    /// Which side the figures favor.
    pub favors: Favors,
    /// `N-*` ids for a non-leak verdict, `["leak"]` for a leak verdict.
    pub supports: Vec<&'static str>,
    /// Retained bytes per dump.
    pub bytes: Vec<u64>,
    /// Dominated counts per dump.
    pub counts: Vec<u64>,
    /// Why, in a few words.
    pub reason: String,
}

/// Judges a holder across at least [`MIN_DUMPS`] dumps: growth over every interval favors
/// a leak (the count too, or a count that is one constant number of blocks: a block that is
/// resized); an increase that stopped (the last interval at most [`STOPPED_SHARE`] of the
/// first, or zero) favors a non-leak (warm-up fill, high-water mark), except for a constant
/// count that rose and then ends flat, which is inconclusive: one block that is resized is flat
/// between resizes, and the allocation test over a longer window says whether it stopped;
/// anything else is inconclusive. `None` when a dump has no figure or there are fewer than
/// three dumps.
pub fn plateau(o: &Observed) -> Option<Plateau> {
    if o.bytes.len() < MIN_DUMPS {
        return None;
    }
    let bytes = all_present(&o.bytes)?;
    let counts = all_present(&o.counts)?;
    let db = deltas(&bytes);
    let dc = deltas(&counts);
    let first = db[0];
    let last = db[db.len() - 1];
    let stopped = last <= 0 || (first > 0 && (last as f64) <= STOPPED_SHARE * first as f64);
    let resized = constant_count(&counts);
    let grew = db.iter().all(|&d| d > 0) && (resized || dc.iter().all(|&d| d > 0));
    let (favors, supports, reason) = if stopped && resized && first > 0 {
        (
            Favors::Inconclusive,
            Vec::new(),
            format!(
                "one block count with bytes that rose and ended flat: a block that is resized is flat between resizes (the last interval added {last} bytes against {first} in the first); a longer window says whether it stopped"
            ),
        )
    } else if stopped {
        (
            Favors::NonLeak,
            vec!["N-WARM", "N-HWM"],
            format!(
                "the increase stopped: the last interval added {last} bytes against {first} in the first"
            ),
        )
    } else if grew {
        (
            Favors::Leak,
            vec!["leak"],
            if resized {
                "retained bytes grew over every interval at a constant count of blocks (a block that is resized)"
            } else {
                "retained bytes and count grew over every interval"
            }
            .to_owned(),
        )
    } else {
        (
            Favors::Inconclusive,
            Vec::new(),
            "growth is neither steady over every interval nor stopped".to_owned(),
        )
    };
    Some(Plateau {
        favors,
        supports,
        bytes,
        counts,
        reason,
    })
}

impl Plateau {
    /// The test-specific fields of the discriminator result.
    pub fn extra(&self) -> Value {
        json!({"bytes": self.bytes, "counts": self.counts, "reason": self.reason})
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(b: &[u64], c: &[u64]) -> Observed {
        Observed {
            bytes: b.iter().map(|&x| Some(x)).collect(),
            counts: c.iter().map(|&x| Some(x)).collect(),
        }
    }

    #[test]
    fn needs_three_dumps() {
        for n in 0..3 {
            let b: Vec<u64> = (0..n).map(|i| 100 + 10 * i as u64).collect();
            assert_eq!(
                gate(&obs(&b, &b)).unwrap_err(),
                vec!["dumps < 3".to_owned()]
            );
            assert!(plateau(&obs(&b, &b)).is_none());
        }
    }

    #[test]
    fn linear_growth_passes() {
        let g = gate(&obs(&[738, 1478, 2226], &[13, 27, 40])).unwrap();
        assert!(g.last_over_first > 0.99);
        assert_eq!(shape(&g), "steady");
        let p = plateau(&obs(&[738, 1478, 2226], &[13, 27, 40])).unwrap();
        assert_eq!(p.favors, Favors::Leak);
        assert_eq!(p.supports, vec!["leak"]);
    }

    #[test]
    fn warm_up_fill_fails_and_favors_non_leak() {
        // Control series: +49% then flat.
        let o = obs(&[510, 762, 764], &[9002, 13502, 13502]);
        let why = gate(&o).unwrap_err();
        assert!(
            why.iter()
                .any(|w| w.contains("dominated count does not increase")),
            "{why:?}"
        );
        let p = plateau(&o).unwrap();
        assert_eq!(p.favors, Favors::NonLeak);
        assert_eq!(p.supports, vec!["N-WARM", "N-HWM"]);
    }

    #[test]
    fn unevenly_spaced_dumps_are_judged_by_their_rate_over_x() {
        // A linear leak (5 bytes per unit of x) read at x = 0.67, 3.5, 6.0, 7.4: the last
        // step is about half of the first in bytes, and the same rate in bytes per x.
        let xs = [670.0, 3500.0, 6000.0, 7400.0];
        let b: Vec<u64> = xs.iter().map(|x| (x * 5.0) as u64).collect();
        let c: Vec<u64> = b.iter().map(|x| x / 100).collect();
        let o = obs(&b, &c);
        let why = gate(&o).unwrap_err();
        assert!(why[0].contains("49%"), "{why:?}");
        let g = gate_at(&o, Some(&xs)).unwrap();
        assert!((g.last_over_first - 1.0).abs() < 0.01, "{g:?}");
        assert_eq!(shape(&g), "stair");
        // The same figures at evenly spaced x are the plain rule.
        assert!(gate_at(&o, Some(&[1.0, 2.0, 3.0, 4.0])).is_err());
        assert_eq!(
            gate_at(&obs(&[10, 20, 30], &[1, 2, 3]), None),
            gate_at(&obs(&[10, 20, 30], &[1, 2, 3]), Some(&[1.0, 2.0, 3.0]))
        );
    }

    #[test]
    fn a_flat_series_and_a_plateau_in_x_still_fail_at_any_spacing() {
        let xs = [670.0, 3500.0, 6000.0, 7400.0];
        // Flat.
        assert!(gate_at(&obs(&[500; 4], &[5; 4]), Some(&xs)).is_err());
        // Rises over the first two intervals, then stops: the rate of the last interval
        // is 2% of the first.
        let o = obs(&[3350, 17500, 30000, 30140], &[33, 175, 300, 301]);
        let why = gate_at(&o, Some(&xs)).unwrap_err();
        assert!(why[0].contains("rate of increase per unit of x"), "{why:?}");
        assert_eq!(plateau(&o).unwrap().favors, Favors::NonLeak);
    }

    #[test]
    fn shrinking_last_interval_fails_the_ratio() {
        // 1000 then 100: strictly increasing, but the last interval is 10% of the first.
        let o = obs(&[1000, 2000, 2100], &[10, 20, 21]);
        let why = gate(&o).unwrap_err();
        assert!(why[0].contains("10%"), "{why:?}");
        // Not stopped (10% > 5%) and growing: a leak by the discriminator.
        assert_eq!(plateau(&o).unwrap().favors, Favors::Leak);
        // 4% of the first: stopped.
        let o = obs(&[1000, 2000, 2040], &[10, 20, 21]);
        assert_eq!(plateau(&o).unwrap().favors, Favors::NonLeak);
    }

    #[test]
    fn missing_and_flat_holders() {
        let o = Observed {
            bytes: vec![Some(1), None, Some(3)],
            counts: vec![Some(1), None, Some(3)],
        };
        assert_eq!(
            gate(&o).unwrap_err(),
            vec!["holder absent from dump 2".to_owned()]
        );
        assert!(plateau(&o).is_none());
        let flat = obs(&[500, 500, 500], &[5, 5, 5]);
        assert!(gate(&flat).is_err());
        assert_eq!(plateau(&flat).unwrap().favors, Favors::NonLeak);
        // Bytes grow at a constant count (one block, or five, that is resized): it passes as
        // `resized` and the discriminator reads a leak. (Before the constant-count rule this
        // series failed the gate and read inconclusive.)
        let o = obs(&[100, 200, 300], &[5, 5, 5]);
        assert!(gate(&o).unwrap().resized);
        assert_eq!(plateau(&o).unwrap().favors, Favors::Leak);
        // Bytes grow, the count falls: no pass, and no leak verdict either.
        let o = obs(&[100, 200, 300], &[5, 4, 3]);
        assert!(gate(&o).is_err());
        assert_eq!(plateau(&o).unwrap().favors, Favors::Inconclusive);
    }

    #[test]
    fn one_block_whose_bytes_rise_at_a_constant_count_passes_as_resized() {
        let g = gate(&obs(&[1 << 20, 1 << 21, 1 << 22, 1 << 23], &[1, 1, 1, 1])).unwrap();
        assert!(g.resized);
        // The doubling steps are a stair; the kind is its own field.
        assert_eq!(shape(&g), "stair");
        // A fixed set of blocks counts as well, and the ratio test still applies.
        assert!(gate(&obs(&[100, 200, 400], &[3, 3, 3])).unwrap().resized);
        let why = gate(&obs(&[1000, 2000, 2100], &[1, 1, 1])).unwrap_err();
        assert!(why[0].contains("10%"), "{why:?}");
        // A holder that keeps more objects is not `resized`.
        let g = gate(&obs(&[100, 200, 300], &[1, 2, 3])).unwrap();
        assert!(!g.resized);
        assert_eq!(shape(&g), "steady");
    }

    #[test]
    fn a_constant_count_with_one_flat_pair_fails_and_so_does_a_count_that_rises_then_stops() {
        // Capacity 2^20, 2^21, 2^22, 2^22: a block that is resized is flat between resizes.
        let why = gate(&obs(&[1 << 20, 1 << 21, 1 << 22, 1 << 22], &[1; 4])).unwrap_err();
        assert!(
            why.iter()
                .any(|w| w.contains("retained bytes do not increase from dump 3 to 4")),
            "{why:?}"
        );
        // A count of zero is no block.
        assert!(gate(&obs(&[100, 200, 300], &[0, 0, 0])).is_err());
        // A count that rises, then stops (a warm-up fill of many objects), fails as before.
        let why = gate(&obs(&[100, 200, 300], &[5, 6, 6])).unwrap_err();
        assert!(
            why.iter()
                .any(|w| w.contains("dominated count does not increase from dump 2 to 3")),
            "{why:?}"
        );
        // A count that rises at one pair and holds at the other fails too.
        assert!(gate(&obs(&[100, 200, 300], &[5, 5, 6])).is_err());
    }

    #[test]
    fn a_resized_block_that_stood_still_between_two_dumps_is_named() {
        assert!(stood_still(&obs(
            &[1 << 20, 1 << 21, 1 << 22, 1 << 22],
            &[1; 4]
        )));
        // Rose at every pair (it passes), flat throughout, shrank overall, a count that moved.
        assert!(!stood_still(&obs(&[100, 200, 300], &[1; 3])));
        assert!(!stood_still(&obs(&[500; 3], &[1; 3])));
        assert!(!stood_still(&obs(&[300, 200, 100], &[1; 3])));
        assert!(!stood_still(&obs(&[100, 200, 200], &[1, 2, 2])));
    }

    #[test]
    fn plateau_reads_a_block_that_is_resized_as_a_leak_and_one_that_ends_flat_as_inconclusive() {
        // Bytes rose at every step at a constant count: a leak.
        let p = plateau(&obs(&[1 << 20, 1 << 21, 1 << 22, 1 << 23], &[1; 4])).unwrap();
        assert_eq!((p.favors, p.supports), (Favors::Leak, vec!["leak"]));
        assert!(p.reason.contains("constant count"), "{}", p.reason);
        // Rose, then flat (capacity 2^22 held over the last pair): not a non-leak, the
        // allocation test decides whether it stopped.
        let p = plateau(&obs(&[1 << 20, 1 << 21, 1 << 22, 1 << 22], &[1; 4])).unwrap();
        assert_eq!((p.favors, p.supports), (Favors::Inconclusive, Vec::new()));
        // Flat from the first dump: still a non-leak.
        let p = plateau(&obs(&[500, 500, 500, 500], &[1; 4])).unwrap();
        assert_eq!(p.favors, Favors::NonLeak);
        // A count that rose and then stopped (a warm-up fill of many objects): still a non-leak.
        let p = plateau(&obs(&[510, 762, 764], &[9002, 13502, 13502])).unwrap();
        assert_eq!(p.favors, Favors::NonLeak);
        let p = plateau(&obs(&[1000, 2000, 2000], &[10, 20, 20])).unwrap();
        assert_eq!(p.favors, Favors::NonLeak);
    }

    #[test]
    fn uneven_growth_is_a_stair() {
        let g = gate(&obs(&[100, 200, 500], &[1, 2, 5])).unwrap();
        assert_eq!(shape(&g), "stair");
    }
}
