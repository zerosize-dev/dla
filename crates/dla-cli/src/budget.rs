//! Budget parsing: `90s`, `45m`, `2h` or `auto`.

use crate::fail::Failure;
use dla_core::error::codes;

/// Default budget when neither flag nor environment gives one: `auto`.
pub const DEFAULT: &str = "auto";

/// A budget as asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Spec {
    /// Sized to the project once the sweep has found its candidates (`dla run init` starts
    /// it at [`dla_core::state::AUTO_START_SECS`]).
    Auto,
    /// A fixed length in seconds; it never changes except by `dla run extend`.
    Fixed(u64),
}

fn bad(text: &str, why: &str) -> Failure {
    Failure::refuse(
        codes::INVALID_VALUE,
        format!("invalid budget {text:?}: {why}"),
        "auto, or a budget such as 90s, 45m or 2h",
        "dla run init --budget 60m",
    )
}

/// Parses `auto` (any case) or `<digits><s|m|h>` into seconds. Zero is refused.
pub fn parse(text: &str) -> Result<Spec, Failure> {
    if text.trim().eq_ignore_ascii_case("auto") {
        return Ok(Spec::Auto);
    }
    parse_fixed(text).map(Spec::Fixed)
}

/// Parses `<digits><s|m|h>` into seconds; `auto` is refused. Zero is refused.
pub fn parse_fixed(text: &str) -> Result<u64, Failure> {
    let t = text.trim();
    let (digits, mult) = match t.char_indices().last() {
        Some((i, 's')) => (&t[..i], 1),
        Some((i, 'm')) => (&t[..i], 60),
        Some((i, 'h')) => (&t[..i], 3600),
        _ => return Err(bad(text, "the unit must be s, m or h")),
    };
    let n: u64 = digits
        .parse()
        .map_err(|_| bad(text, "expected digits before the unit"))?;
    if n == 0 {
        return Err(bad(text, "must be greater than zero"));
    }
    n.checked_mul(mult).ok_or_else(|| bad(text, "too large"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_units() {
        assert_eq!(parse("90s").unwrap(), Spec::Fixed(90));
        assert_eq!(parse("45m").unwrap(), Spec::Fixed(2700));
        assert_eq!(parse("2h").unwrap(), Spec::Fixed(7200));
        assert_eq!(parse(" 5m ").unwrap(), Spec::Fixed(300));
    }

    #[test]
    fn auto_is_accepted_in_any_case() {
        for s in ["auto", "AUTO", "Auto", " auto "] {
            assert_eq!(parse(s).unwrap(), Spec::Auto, "{s}");
        }
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
            "autos",
            "auto m",
            "99999999999999999999h",
        ] {
            assert!(parse(s).is_err(), "{s}");
        }
    }

    #[test]
    fn a_fixed_parse_refuses_auto() {
        let e = parse_fixed("auto").unwrap_err();
        assert!(format!("{e:?}").contains("invalid budget"), "{e:?}");
        assert_eq!(parse_fixed("30m").unwrap(), 1800);
    }
}
