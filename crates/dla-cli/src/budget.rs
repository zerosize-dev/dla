//! Budget parsing: `90s`, `45m`, `2h`.

use crate::fail::Failure;
use dla_core::error::codes;

/// Default budget when neither flag nor environment gives one.
pub const DEFAULT_SECONDS: u64 = 60 * 60;

/// Parses `<digits><s|m|h>` into seconds. Zero is refused.
pub fn parse(text: &str) -> Result<u64, Failure> {
    let bad = |why: &str| {
        Failure::refuse(
            codes::INVALID_VALUE,
            format!("invalid budget {text:?}: {why}"),
            "a budget such as 90s, 45m or 2h",
            "dla run init --budget 60m",
        )
    };
    let t = text.trim();
    let (digits, mult) = match t.char_indices().last() {
        Some((i, 's')) => (&t[..i], 1),
        Some((i, 'm')) => (&t[..i], 60),
        Some((i, 'h')) => (&t[..i], 3600),
        _ => return Err(bad("the unit must be s, m or h")),
    };
    let n: u64 = digits
        .parse()
        .map_err(|_| bad("expected digits before the unit"))?;
    if n == 0 {
        return Err(bad("must be greater than zero"));
    }
    n.checked_mul(mult).ok_or_else(|| bad("too large"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_units() {
        assert_eq!(parse("90s").unwrap(), 90);
        assert_eq!(parse("45m").unwrap(), 2700);
        assert_eq!(parse("2h").unwrap(), 7200);
        assert_eq!(parse(" 5m ").unwrap(), 300);
    }

    #[test]
    fn rejects_bad_input() {
        for s in [
            "",
            "m",
            "10",
            "0m",
            "-5m",
            "1.5h",
            "5d",
            "99999999999999999999h",
        ] {
            assert!(parse(s).is_err(), "{s}");
        }
    }
}
