//! Consent for intrusive steps: parsing the tier list and building the record.
//!
//! One consent is recorded per run. The tiers are the privilege tiers 1 to 4 of the
//! safety reference; a ceiling caps which tiers may be granted.

use chrono::{DateTime, Utc};

use crate::{
    Refusal,
    error::codes,
    state::{Consent, ConsentGiver},
};

/// Lowest privilege tier.
pub const MIN_TIER: u8 = 1;
/// Highest privilege tier.
pub const MAX_TIER: u8 = 4;

fn invalid(message: String, fix: &str) -> Refusal {
    Refusal::new(
        codes::INVALID_VALUE,
        message,
        "tiers written as 1-4, 1,2,3 or 1-2,4 (each from 1 to 4)",
        fix,
    )
}

/// Parses a tier list such as `1-4`, `1,2,3` or `1-2,4` into sorted, distinct tiers.
pub fn parse_tiers(text: &str) -> Result<Vec<u8>, Refusal> {
    const FIX: &str = "dla consent --tiers 1-3";
    let tier = |s: &str| -> Result<u8, Refusal> {
        let n: u8 = s.trim().parse().map_err(|_| {
            invalid(
                format!("{s:?} in the tier list {text:?} is not a number"),
                FIX,
            )
        })?;
        if (MIN_TIER..=MAX_TIER).contains(&n) {
            Ok(n)
        } else {
            Err(invalid(
                format!("tier {n} is outside {MIN_TIER}..{MAX_TIER}"),
                FIX,
            ))
        }
    };
    let mut tiers: Vec<u8> = Vec::new();
    for part in text.split(',') {
        match part.split_once('-') {
            Some((a, b)) => {
                let (a, b) = (tier(a)?, tier(b)?);
                if a > b {
                    return Err(invalid(format!("the range {part:?} runs backwards"), FIX));
                }
                tiers.extend(a..=b);
            }
            None => tiers.push(tier(part)?),
        }
    }
    tiers.sort_unstable();
    tiers.dedup();
    Ok(tiers)
}

/// Parses a ceiling: a single tier from 1 to 4.
pub fn parse_ceiling(text: &str) -> Result<u8, Refusal> {
    match text.trim().parse::<u8>() {
        Ok(n) if (MIN_TIER..=MAX_TIER).contains(&n) => Ok(n),
        _ => Err(invalid(
            format!("the ceiling {text:?} is not a tier from {MIN_TIER} to {MAX_TIER}"),
            "dla consent --tiers 1-3 --ceiling 3",
        )),
    }
}

/// Reads a boolean switch the way environment variables are written: `1` or `true`
/// (any case) is on; anything else, including empty, is off.
pub fn flag_on(text: &str) -> bool {
    let t = text.trim();
    t == "1" || t.eq_ignore_ascii_case("true")
}

/// Checks the tiers against the ceiling and builds the consent record.
pub fn build(
    tiers: Vec<u8>,
    raw_values: bool,
    given_by: ConsentGiver,
    ceiling: Option<u8>,
    now: DateTime<Utc>,
) -> Result<Consent, Refusal> {
    if let Some(c) = ceiling
        && let Some(over) = tiers.iter().copied().filter(|t| *t > c).max()
    {
        return Err(Refusal::new(
            codes::INVALID_VALUE,
            format!("tier {over} is above the privilege ceiling {c}"),
            format!("tiers no higher than {c}"),
            format!("dla consent --tiers 1-{c} --ceiling {c}"),
        ));
    }
    if tiers.is_empty() {
        return Err(invalid(
            "the tier list is empty".to_owned(),
            "dla consent --tiers 1-3",
        ));
    }
    Ok(Consent {
        tiers,
        raw_values,
        given_by,
        recorded_at: now,
    })
}

/// The refusal for a second consent in one run.
pub fn already_given(existing: &Consent) -> Refusal {
    Refusal::new(
        codes::ALREADY_RESOLVED,
        format!(
            "consent was already recorded for this run (given by {}, tiers {:?}); there is one consent question per run",
            existing.given_by.as_str(),
            existing.tiers
        ),
        "a new run for a different consent",
        "dla run init",
    )
}

/// The `result` of the `run-config` evidence record that describes a consent.
pub fn evidence_result(c: &Consent) -> serde_json::Value {
    serde_json::json!({
        "consent": {
            "tiers": c.tiers,
            "raw_values": c.raw_values,
            "given_by": c.given_by.as_str(),
            "recorded_at": c.recorded_at,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_lists_parse() {
        assert_eq!(parse_tiers("1-4").unwrap(), vec![1, 2, 3, 4]);
        assert_eq!(parse_tiers("3,1,1").unwrap(), vec![1, 3]);
        assert_eq!(parse_tiers("1-2, 4").unwrap(), vec![1, 2, 4]);
        for bad in ["", "0", "5", "1-5", "4-1", "a", "1,,2", "1-"] {
            assert!(parse_tiers(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn ceiling_caps_tiers() {
        let now = Utc::now();
        assert!(build(vec![1, 2], false, ConsentGiver::User, Some(2), now).is_ok());
        let r = build(vec![1, 3], false, ConsentGiver::User, Some(2), now).unwrap_err();
        assert!(r.message.contains("tier 3"));
        assert_eq!(parse_ceiling("3").unwrap(), 3);
        assert!(parse_ceiling("0").is_err() && parse_ceiling("x").is_err());
    }

    #[test]
    fn flags() {
        assert!(flag_on("1") && flag_on("TRUE") && !flag_on("0") && !flag_on(""));
    }
}
