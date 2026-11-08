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
}

fn deltas(v: &[u64]) -> Vec<i128> {
    v.windows(2)
        .map(|w| i128::from(w[1]) - i128::from(w[0]))
        .collect()
}

fn all_present(v: &[Option<u64>]) -> Option<Vec<u64>> {
    v.iter().copied().collect()
}

/// Applies the dump-series rule: at least [`MIN_DUMPS`] dumps, retained bytes and
/// dominated count strictly increasing across every consecutive pair, and the last
/// interval's byte increase at least [`LAST_OVER_FIRST`] of the first's.
pub fn gate(o: &Observed) -> Result<Growth, Vec<String>> {
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
    if let Some(i) = dc.iter().position(|&d| d <= 0) {
        why.push(format!(
            "dominated count does not increase from dump {} to {}",
            i + 1,
            i + 2
        ));
    }
    if !why.is_empty() {
        return Err(why);
    }
    let first = db[0] as f64;
    let last = db[db.len() - 1] as f64;
    let ratio = last / first;
    if ratio < LAST_OVER_FIRST {
        return Err(vec![format!(
            "last interval's increase is {:.0}% of the first's (needs >= {:.0}%)",
            ratio * 100.0,
            LAST_OVER_FIRST * 100.0
        )]);
    }
    Ok(Growth {
        byte_deltas: db.iter().map(|&d| d as u64).collect(),
        count_deltas: dc.iter().map(|&d| d as u64).collect(),
        last_over_first: ratio,
    })
}

/// `steady` when the byte increases stay within a factor of two of each other, else
/// `stair`.
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
/// a leak; an increase that stopped (the last interval at most [`STOPPED_SHARE`] of the
/// first, or zero) favors a non-leak (warm-up fill, high-water mark); anything else is
/// inconclusive. `None` when a dump has no figure or there are fewer than three dumps.
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
    let grew = db.iter().all(|&d| d > 0) && dc.iter().all(|&d| d > 0);
    let (favors, supports, reason) = if stopped {
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
            "retained bytes and count grew over every interval".to_owned(),
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
        // Bytes grow, the count does not: no pass, and no leak verdict either.
        let o = obs(&[100, 200, 300], &[5, 5, 5]);
        assert!(gate(&o).is_err());
        assert_eq!(plateau(&o).unwrap().favors, Favors::Inconclusive);
    }

    #[test]
    fn uneven_growth_is_a_stair() {
        let g = gate(&obs(&[100, 200, 500], &[1, 2, 5])).unwrap();
        assert_eq!(shape(&g), "stair");
    }
}
