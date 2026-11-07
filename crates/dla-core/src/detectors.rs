//! Detector tables: `data/detectors/<lang>.toml`.
//!
//! Each file lists the static rules DLA turns into candidates:
//!
//! ```toml
//! [[detector]]
//! id = "staticcheck/SA5003"
//! backend = "staticcheck"
//! rule = "SA5003"
//! suggested_archetype = "L-HANDLE"   # "" for none
//! verified = true
//! precision_prior = 0.6
//! default_enabled = true
//! note = "defer in an infinite loop"
//! ```
//!
//! [`DetectorFile::parse`] validates a file and refuses with the file name and line of
//! the first problem: unknown backend, duplicate id, an id that is not
//! `<backend>/<rule>`, a bad archetype id, a prior outside 0..1, a malformed table.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{Refusal, codes};

/// Every backend name a detector may use. The backends crate owns the adapters; this
/// list is the contract the tables are validated against.
pub const KNOWN_BACKENDS: [&str; 13] = [
    "dla",
    "staticcheck",
    "golangci-lint",
    "govet",
    "errorprone",
    "spotbugs",
    "detekt",
    "codeql",
    "sonar",
    "ruff",
    "pylint",
    "eslint",
    "rapx",
];

/// The archetype IDs a detector may suggest (leak and non-leak archetypes). The empty
/// string means "no suggestion". Archetypes are tags only: nothing filters on them.
pub const ARCHETYPE_IDS: [&str; 17] = [
    "L-HANDLE",
    "L-COLLECT",
    "L-REGISTER",
    "L-WORKER",
    "L-TLS",
    "L-STALE",
    "L-CAPTURE",
    "L-CYCLE",
    "L-OWNERSHIP",
    "L-UNLOAD",
    "N-ALLOC",
    "N-HEADROOM",
    "N-WARM",
    "N-HWM",
    "N-SCALE",
    "N-INTENT",
    "N-OBSERVER",
];

fn default_true() -> bool {
    true
}

fn default_prior() -> f64 {
    0.5
}

/// One rule of one backend.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Detector {
    /// `<backend>/<rule>`.
    pub id: String,
    /// Backend name, one of [`KNOWN_BACKENDS`].
    pub backend: String,
    /// The backend's own rule identifier.
    pub rule: String,
    /// Archetype tag to suggest, or the empty string.
    #[serde(default)]
    pub suggested_archetype: String,
    /// True when the rule id was checked against the installed tool or its documentation.
    #[serde(default)]
    pub verified: bool,
    /// Prior probability that a hit is a real leak, 0 to 1.
    #[serde(default = "default_prior")]
    pub precision_prior: f64,
    /// Whether the rule is enabled in the tool by default. DLA enables every listed
    /// rule regardless; this records what the tool does on its own.
    #[serde(default = "default_true")]
    pub default_enabled: bool,
    /// Free-text note.
    #[serde(default)]
    pub note: String,
}

/// A detector table.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct DetectorFile {
    /// The `[[detector]]` entries, in file order.
    #[serde(default)]
    pub detector: Vec<Detector>,
}

fn invalid(file: &str, line: usize, message: impl std::fmt::Display) -> Refusal {
    Refusal::new(
        codes::INVALID_DATA_FILE,
        format!("{file}:{line}: {message}"),
        format!("a valid detector table at {file}:{line}"),
        format!("edit {file} line {line}"),
    )
}

fn line_of_offset(text: &str, offset: usize) -> usize {
    let end = offset.min(text.len());
    text.as_bytes()[..end]
        .iter()
        .filter(|b| **b == b'\n')
        .count()
        + 1
}

/// 1-based line of the `[[detector]]` header of each entry, in order.
fn entry_lines(text: &str) -> Vec<usize> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| l.trim() == "[[detector]]")
        .map(|(i, _)| i + 1)
        .collect()
}

/// Line of `key =` inside the entry that starts at `start` (falls back to the header).
fn key_line(text: &str, start: usize, end: usize, key: &str) -> usize {
    text.lines()
        .enumerate()
        .skip(start - 1)
        .take(end.saturating_sub(start) + 1)
        .find(|(_, l)| {
            let t = l.trim_start();
            t.strip_prefix(key)
                .is_some_and(|r| r.trim_start().starts_with('='))
        })
        .map_or(start, |(i, _)| i + 1)
}

impl DetectorFile {
    /// Parses and validates `text`. `file` is the name used in refusals (for example
    /// `data/detectors/go.toml`).
    pub fn parse(text: &str, file: &str) -> Result<Self, Refusal> {
        let parsed: DetectorFile = toml::from_str(text).map_err(|e| {
            let line = e.span().map_or(1, |s| line_of_offset(text, s.start));
            invalid(file, line, e.message())
        })?;
        parsed.validate(text, file)?;
        Ok(parsed)
    }

    fn validate(&self, text: &str, file: &str) -> Result<(), Refusal> {
        let starts = entry_lines(text);
        let total = text.lines().count().max(1);
        let mut seen: Vec<&str> = Vec::new();
        for (i, d) in self.detector.iter().enumerate() {
            let start = starts.get(i).copied().unwrap_or(1);
            let end = starts.get(i + 1).map_or(total, |n| n - 1);
            let at = |key: &str| key_line(text, start, end, key);
            if !KNOWN_BACKENDS.contains(&d.backend.as_str()) {
                return Err(invalid(
                    file,
                    at("backend"),
                    format!(
                        "unknown backend {:?} (known: {})",
                        d.backend,
                        KNOWN_BACKENDS.join(", ")
                    ),
                ));
            }
            if d.rule.trim().is_empty() || d.rule.chars().any(char::is_whitespace) {
                return Err(invalid(file, at("rule"), "rule must be a non-empty token"));
            }
            let want = format!("{}/{}", d.backend, d.rule);
            if d.id != want {
                return Err(invalid(
                    file,
                    at("id"),
                    format!("id {:?} must be \"<backend>/<rule>\", here {want:?}", d.id),
                ));
            }
            if seen.contains(&d.id.as_str()) {
                return Err(invalid(
                    file,
                    at("id"),
                    format!("duplicate detector id {:?}", d.id),
                ));
            }
            seen.push(&d.id);
            if !d.suggested_archetype.is_empty()
                && !ARCHETYPE_IDS.contains(&d.suggested_archetype.as_str())
            {
                return Err(invalid(
                    file,
                    at("suggested_archetype"),
                    format!(
                        "unknown archetype {:?} (use one of {} or \"\")",
                        d.suggested_archetype,
                        ARCHETYPE_IDS.join(", ")
                    ),
                ));
            }
            if !(0.0..=1.0).contains(&d.precision_prior) || d.precision_prior.is_nan() {
                return Err(invalid(
                    file,
                    at("precision_prior"),
                    format!("precision_prior {} is outside 0..1", d.precision_prior),
                ));
            }
        }
        Ok(())
    }

    /// Detectors of one backend, in file order.
    pub fn for_backend<'a>(&'a self, backend: &'a str) -> impl Iterator<Item = &'a Detector> {
        self.detector.iter().filter(move |d| d.backend == backend)
    }

    /// The distinct backends referenced, in first-appearance order.
    pub fn backends(&self) -> Vec<&str> {
        let mut out: Vec<&str> = Vec::new();
        for d in &self.detector {
            if !out.contains(&d.backend.as_str()) {
                out.push(&d.backend);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
[[detector]]
id = "staticcheck/SA5003"
backend = "staticcheck"
rule = "SA5003"
suggested_archetype = "L-HANDLE"
verified = true
precision_prior = 0.6
default_enabled = true
note = "defer in loop"

[[detector]]
id = "govet/lostcancel"
backend = "govet"
rule = "lostcancel"
"#;

    #[test]
    fn a_good_table_parses_with_defaults() {
        let f = DetectorFile::parse(GOOD, "go.toml").unwrap();
        assert_eq!(f.detector.len(), 2);
        let d = &f.detector[1];
        assert_eq!(d.suggested_archetype, "");
        assert!(!d.verified);
        assert!(d.default_enabled);
        assert_eq!(d.precision_prior, 0.5);
        assert_eq!(f.backends(), ["staticcheck", "govet"]);
        assert_eq!(f.for_backend("govet").count(), 1);
    }

    #[test]
    fn an_empty_file_is_an_empty_table() {
        assert!(
            DetectorFile::parse("", "x.toml")
                .unwrap()
                .detector
                .is_empty()
        );
    }

    #[test]
    fn unknown_backend_names_file_and_line() {
        let text = GOOD
            .replace("\"govet\"", "\"nosuch\"")
            .replace("govet/", "nosuch/");
        let e = DetectorFile::parse(&text, "data/detectors/go.toml").unwrap_err();
        assert_eq!(e.code, codes::INVALID_DATA_FILE);
        assert!(
            e.message.starts_with("data/detectors/go.toml:14:"),
            "{}",
            e.message
        );
        assert!(e.message.contains("unknown backend"));
    }

    #[test]
    fn duplicate_id_is_refused_at_the_second_entry() {
        let text = format!(
            "{GOOD}\n[[detector]]\nid = \"govet/lostcancel\"\nbackend = \"govet\"\nrule = \"lostcancel\"\n"
        );
        let e = DetectorFile::parse(&text, "go.toml").unwrap_err();
        assert!(e.message.contains("duplicate"), "{}", e.message);
        assert!(e.message.starts_with("go.toml:18:"), "{}", e.message);
    }

    #[test]
    fn bad_archetype_prior_and_id_shape_are_refused() {
        let e = DetectorFile::parse(&GOOD.replace("L-HANDLE", "L-NOPE"), "f").unwrap_err();
        assert!(e.message.contains("unknown archetype"));
        let e = DetectorFile::parse(&GOOD.replace("0.6", "1.6"), "f").unwrap_err();
        assert!(e.message.contains("outside 0..1"));
        let e = DetectorFile::parse(
            &GOOD.replace("id = \"govet/lostcancel\"", "id = \"lostcancel\""),
            "f",
        )
        .unwrap_err();
        assert!(e.message.contains("<backend>/<rule>"));
    }

    #[test]
    fn syntax_errors_and_unknown_keys_carry_a_line() {
        let e = DetectorFile::parse("[[detector]]\nid = \n", "f.toml").unwrap_err();
        assert!(e.message.starts_with("f.toml:"), "{}", e.message);
        let e = DetectorFile::parse(
            "[[detector]]\nid = \"a/b\"\nbackend = \"dla\"\nrule = \"b\"\nbogus = 1\n",
            "f.toml",
        )
        .unwrap_err();
        assert!(e.message.starts_with("f.toml:5:"), "{}", e.message);
    }
}
