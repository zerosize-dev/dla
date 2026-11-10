//! The verification record: what an independent verifier concluded from a finding's
//! evidence bundle, and whether that agrees with the finding.
//!
//! The verifier sees only the bundle (see `dla bundle`), never the finding's
//! hypothesis or label, so agreement is evidence that the finding follows from the
//! data. `dla ledger verify` (core only) appends the record; a disagreement also opens
//! an open item naming both values.

use crate::{
    finding::{Finding, Holder, check_site},
    record::Verification,
};

/// Lines of slack when comparing the verifier's site with the finding's.
pub const SITE_TOLERANCE: u32 = 3;

/// Splits a `file:line` site.
fn parse_site(site: &str) -> Option<(&str, u32)> {
    let (file, line) = site.rsplit_once(':')?;
    Some((file, line.parse().ok()?))
}

/// Whether the verifier's site agrees with the finding's: the same file and a line
/// within [`SITE_TOLERANCE`] of it, or neither names a site.
pub fn agrees_site(finding: Option<&str>, verifier: Option<&str>) -> bool {
    match (finding, verifier) {
        (None, None) => true,
        (Some(a), Some(b)) => match (parse_site(a), parse_site(b)) {
            (Some((fa, la)), Some((fb, lb))) => fa == fb && la.abs_diff(lb) <= SITE_TOLERANCE,
            _ => a == b,
        },
        _ => false,
    }
}

/// Whether the verifier's holder is the finding's. A finding without a holder agrees
/// with nothing.
pub fn agrees_holder(finding: Option<Holder>, verifier: Holder) -> bool {
    finding == Some(verifier)
}

/// Parses the `--site` value of `dla ledger verify`: `none` or `file:line`.
pub fn parse_verifier_site(raw: &str) -> Result<Option<String>, String> {
    if raw.trim().eq_ignore_ascii_case("none") {
        return Ok(None);
    }
    check_site(raw.trim())?;
    Ok(Some(raw.trim().to_owned()))
}

/// True for a SHA-256 in lowercase hex.
pub fn is_sha256(s: &str) -> bool {
    s.len() == 64 && s.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// Whether the verifier's label is the finding's. `None` when archetypes are off (the
/// verifier could only answer `UNCLASSIFIED`, which says nothing).
pub fn agrees_label(finding: &Finding, verifier: &str, archetypes_on: bool) -> Option<bool> {
    archetypes_on.then(|| finding.archetype == verifier)
}

/// Builds the record for `finding` from the verifier's answer, computing the agreement.
/// `archetypes_on` says whether the label could be compared.
pub fn record(
    finding: &Finding,
    site: Option<String>,
    holder: Holder,
    mechanism: String,
    label: String,
    bundle: String,
    archetypes_on: bool,
) -> Verification {
    Verification {
        agrees_site: agrees_site(finding.site.as_deref(), site.as_deref()),
        agrees_holder: agrees_holder(finding.holder, holder),
        agrees_label: agrees_label(finding, &label, archetypes_on),
        finding: finding.id.clone(),
        site,
        holder,
        mechanism,
        label,
        bundle,
    }
}

/// Longest mechanism text copied into an open item.
const MECHANISM_CLIP: usize = 200;

/// True when the verification agrees with the finding on site, holder and (when it was
/// compared) label.
pub fn agrees(v: &Verification) -> bool {
    v.agrees_site && v.agrees_holder && v.agrees_label.unwrap_or(true)
}

/// The open item text for a disagreement, naming both values and the verifier's
/// mechanism.
pub fn disagreement_text(finding: &Finding, v: &Verification) -> String {
    let mut parts = Vec::new();
    if !v.agrees_site {
        parts.push(format!(
            "site: finding {} vs verifier {}",
            finding.site.as_deref().unwrap_or("none"),
            v.site.as_deref().unwrap_or("none")
        ));
    }
    if !v.agrees_holder {
        parts.push(format!(
            "holder: finding {} vs verifier {}",
            finding.holder.map_or("none", Holder::as_str),
            v.holder
        ));
    }
    if v.agrees_label == Some(false) {
        parts.push(format!(
            "label: finding {} vs verifier {}",
            finding.archetype, v.label
        ));
    }
    let mechanism: String = v.mechanism.split_whitespace().collect::<Vec<_>>().join(" ");
    let mechanism = if mechanism.chars().count() > MECHANISM_CLIP {
        let cut: String = mechanism.chars().take(MECHANISM_CLIP).collect();
        format!("{cut}...")
    } else {
        mechanism
    };
    format!(
        "verifier disagrees with {} ({}); verifier mechanism: {mechanism:?}; bundle {}",
        finding.id,
        parts.join("; "),
        &v.bundle[..12.min(v.bundle.len())]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_agreement_is_same_file_within_three_lines() {
        assert!(agrees_site(Some("a/B.java:40"), Some("a/B.java:43")));
        assert!(agrees_site(Some("a/B.java:40"), Some("a/B.java:37")));
        assert!(!agrees_site(Some("a/B.java:40"), Some("a/B.java:44")));
        assert!(!agrees_site(Some("a/B.java:40"), Some("a/C.java:40")));
        assert!(agrees_site(None, None));
        assert!(!agrees_site(Some("a/B.java:40"), None));
        assert!(!agrees_site(None, Some("a/B.java:40")));
    }

    #[test]
    fn holder_agreement_is_equal_kind() {
        assert!(agrees_holder(
            Some(Holder::StaticGlobal),
            Holder::StaticGlobal
        ));
        assert!(!agrees_holder(
            Some(Holder::StaticGlobal),
            Holder::ThreadLocal
        ));
        assert!(!agrees_holder(None, Holder::ThreadLocal));
    }

    #[test]
    fn verifier_site_parsing() {
        assert_eq!(parse_verifier_site("none").unwrap(), None);
        assert_eq!(
            parse_verifier_site("src/A.java:12").unwrap().as_deref(),
            Some("src/A.java:12")
        );
        assert!(parse_verifier_site("src/A.java").is_err());
    }

    #[test]
    fn sha256_shape() {
        assert!(is_sha256(&"a".repeat(64)));
        assert!(!is_sha256(&"A".repeat(64)));
        assert!(!is_sha256("abc"));
    }
}
