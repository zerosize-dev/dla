//! Known-leak table: `data/known-leaks.toml`.
//!
//! Each entry names a documented behavior of a runtime or library that looks like a
//! leak or hides one, with the public source that states it:
//!
//! ```toml
//! [[leak]]
//! id = "go-timer-unstopped-before-1.23"
//! runtime = "go"
//! library = "go"                 # a runtime name means the runtime version
//! versions = "<1.23"
//! when = { source_regex = 'time\.NewTicker\(' }
//! effect = "..."
//! holder = "registration-table"
//! archetype = "L-REGISTER"       # the leak archetype the behavior belongs to
//! evidence = "https://go.dev/doc/go1.23"
//! remedy = "..."
//! ```
//!
//! `dla probe` evaluates each entry for the runtimes it finds and writes the verdict
//! to probe evidence. An entry is context, never a finding.
//!
//! [`KnownLeaksFile::parse`] validates a file and refuses with the file name and line
//! of the first problem.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    error::{Refusal, codes},
    finding::Holder,
    runtime::Runtime,
};

/// A condition over the project. Evaluated three-valued by `dla probe`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Predicate {
    /// True when every part is true.
    All(Vec<Predicate>),
    /// True when any part is true.
    Any(Vec<Predicate>),
    /// Negation. `maybe` stays `maybe`.
    Not(Box<Predicate>),
    /// The runtime version (the `go` line of go.mod, the declared or installed
    /// language version) is in this range.
    RuntimeVersion(String),
    /// A library from the manifests is present with a version in the range.
    Library(LibraryReq),
    /// A configuration key is set in the project's config files, build files or
    /// scripts, optionally to a value (compared without case).
    Config(ConfigReq),
    /// A regular expression matches a source file of the entry's runtime.
    SourceRegex(String),
}

/// A library requirement.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LibraryReq {
    /// Library name as the manifests spell it (`artifactId`, module path, crate or
    /// package name); `group:*` matches a whole group.
    pub name: String,
    /// Version range, for example `>=4.1.0, <4.2`. `*` for any.
    #[serde(default = "any_version")]
    pub versions: String,
}

/// A configuration requirement.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ConfigReq {
    /// Key, for example `spring.cache.type` or `io.netty.leakDetection.level`.
    pub key: String,
    /// Required value, compared without case. Absent means "set to anything".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

fn any_version() -> String {
    "*".to_owned()
}

/// One known leak.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct KnownLeak {
    /// Stable ID, `<runtime>-<slug>`.
    pub id: String,
    /// Runtime the entry applies to.
    pub runtime: Runtime,
    /// Library name, or a runtime name (`go`) for the runtime itself.
    pub library: String,
    /// Other manifest names of the same library (`io.netty:*`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// Affected version range of the library.
    pub versions: String,
    /// Extra condition (config keys, source patterns, other libraries).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<Predicate>,
    /// What happens, in one or two sentences.
    pub effect: String,
    /// What holds the memory.
    pub holder: Holder,
    /// The leak archetype (`L-*`) the behavior belongs to. A hint for the reader: a
    /// verdict never labels a finding.
    pub archetype: String,
    /// Public source URL that states the behavior.
    pub evidence: String,
    /// How to fix or avoid it.
    pub remedy: String,
}

/// The known-leak table.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct KnownLeaksFile {
    /// The `[[leak]]` entries, in file order.
    #[serde(default)]
    pub leak: Vec<KnownLeak>,
}

/// A parsed version: its numeric parts.
pub type Version = Vec<u64>;

/// Parses the leading dotted numbers of `s` (`go1.24.7`, `v2.0.1-rc1`, `4.1.100.Final`
/// give their numbers). `None` when there is no number. Parts too large for u64 stop
/// the parse.
pub fn parse_version(s: &str) -> Option<Version> {
    let t = s.trim().trim_start_matches(|c: char| !c.is_ascii_digit());
    let mut out = Vec::new();
    for part in t.split('.') {
        let digits: String = part.chars().take_while(char::is_ascii_digit).collect();
        if digits.is_empty() {
            break;
        }
        match digits.parse::<u64>() {
            Ok(n) => out.push(n),
            Err(_) => break,
        }
        if digits.len() != part.len() {
            // A PEP 440 epoch (`1!2.0`) is not a leading release number.
            if part[digits.len()..].starts_with('!') {
                return None;
            }
            break;
        }
    }
    (!out.is_empty()).then_some(out)
}

/// Compares two versions, padding the shorter with zeros.
pub fn cmp_version(a: &[u64], b: &[u64]) -> std::cmp::Ordering {
    let n = a.len().max(b.len());
    for i in 0..n {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            std::cmp::Ordering::Equal => {}
            o => return o,
        }
    }
    std::cmp::Ordering::Equal
}

/// A version range: comparators joined by commas, all of which must hold.
#[derive(Debug, Clone, PartialEq)]
pub struct VersionRange {
    cmps: Vec<(Op, Version)>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Lt,
    Le,
    Gt,
    Ge,
    Eq,
    Ne,
}

impl VersionRange {
    /// Parses `*` or comparators such as `>=4.1.0, <4.2`. A bare version means `=`.
    pub fn parse(s: &str) -> Result<Self, String> {
        let s = s.trim();
        if s == "*" {
            return Ok(Self { cmps: Vec::new() });
        }
        if s.is_empty() {
            return Err("empty version range".to_owned());
        }
        let mut cmps = Vec::new();
        for part in s.split(',') {
            let p = part.trim();
            let (op, rest) = if let Some(r) = p.strip_prefix(">=") {
                (Op::Ge, r)
            } else if let Some(r) = p.strip_prefix("<=") {
                (Op::Le, r)
            } else if let Some(r) = p.strip_prefix("!=") {
                (Op::Ne, r)
            } else if let Some(r) = p.strip_prefix("==") {
                (Op::Eq, r)
            } else if let Some(r) = p.strip_prefix('>') {
                (Op::Gt, r)
            } else if let Some(r) = p.strip_prefix('<') {
                (Op::Lt, r)
            } else if let Some(r) = p.strip_prefix('=') {
                (Op::Eq, r)
            } else {
                (Op::Eq, p)
            };
            let rest = rest.trim();
            let body = rest
                .strip_prefix("go")
                .or_else(|| rest.strip_prefix('v'))
                .unwrap_or(rest);
            if body.contains(char::is_whitespace)
                || !body.starts_with(|c: char| c.is_ascii_digit())
                || body.contains(['!', '^', '~', '*', '+'])
            {
                return Err(format!(
                    "unsupported version {rest:?} in range {s:?} (use comma-separated comparators)"
                ));
            }
            let v = parse_version(rest)
                .ok_or_else(|| format!("bad version {:?} in range {s:?}", rest.trim()))?;
            cmps.push((op, v));
        }
        Ok(Self { cmps })
    }

    /// True for `*`: every version, and an unknown one, is inside the range.
    pub fn is_any(&self) -> bool {
        self.cmps.is_empty()
    }

    /// True when `v` is inside the range.
    pub fn contains(&self, v: &[u64]) -> bool {
        use std::cmp::Ordering::*;
        self.cmps.iter().all(|(op, b)| {
            let o = cmp_version(v, b);
            match op {
                Op::Lt => o == Less,
                Op::Le => o != Greater,
                Op::Gt => o == Greater,
                Op::Ge => o != Less,
                Op::Eq => o == Equal,
                Op::Ne => o != Equal,
            }
        })
    }
}

fn invalid(file: &str, line: usize, message: impl std::fmt::Display) -> Refusal {
    Refusal::new(
        codes::INVALID_DATA_FILE,
        format!("{file}:{line}: {message}"),
        format!("a valid known-leaks table at {file}:{line}"),
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

fn entry_lines(text: &str) -> Vec<usize> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| l.trim() == "[[leak]]")
        .map(|(i, _)| i + 1)
        .collect()
}

fn check_predicate(p: &Predicate) -> Result<(), String> {
    match p {
        Predicate::All(v) | Predicate::Any(v) => {
            if v.is_empty() {
                return Err("all/any needs at least one condition".to_owned());
            }
            v.iter().try_for_each(check_predicate)
        }
        Predicate::Not(b) => check_predicate(b),
        Predicate::RuntimeVersion(r) => VersionRange::parse(r).map(|_| ()),
        Predicate::Library(l) => {
            if l.name.trim().is_empty() {
                return Err("library name is empty".to_owned());
            }
            VersionRange::parse(&l.versions).map(|_| ())
        }
        Predicate::Config(c) => {
            if c.key.trim().is_empty() {
                Err("config key is empty".to_owned())
            } else {
                Ok(())
            }
        }
        Predicate::SourceRegex(r) => {
            if r.is_empty() {
                Err("source_regex is empty".to_owned())
            } else {
                Ok(())
            }
        }
    }
}

impl KnownLeaksFile {
    /// Parses and validates `text`. `file` names the file in refusals.
    pub fn parse(text: &str, file: &str) -> Result<Self, Refusal> {
        let parsed: KnownLeaksFile = toml::from_str(text).map_err(|e| {
            let line = e.span().map_or(1, |s| line_of_offset(text, s.start));
            invalid(file, line, e.message())
        })?;
        let starts = entry_lines(text);
        let mut seen: Vec<&str> = Vec::new();
        for (i, e) in parsed.leak.iter().enumerate() {
            let at = starts.get(i).copied().unwrap_or(1);
            let slug = e.id.strip_prefix(&format!("{}-", e.runtime.as_str()));
            if slug.is_none_or(|s| s.is_empty() || s.chars().any(char::is_whitespace)) {
                return Err(invalid(
                    file,
                    at,
                    format!("id {:?} must be \"<runtime>-<slug>\"", e.id),
                ));
            }
            if seen.contains(&e.id.as_str()) {
                return Err(invalid(file, at, format!("duplicate id {:?}", e.id)));
            }
            seen.push(&e.id);
            if e.library.trim().is_empty() {
                return Err(invalid(file, at, "library is empty"));
            }
            VersionRange::parse(&e.versions).map_err(|m| invalid(file, at, m))?;
            if !e.archetype.starts_with("L-")
                || !crate::detectors::ARCHETYPE_IDS.contains(&e.archetype.as_str())
            {
                return Err(invalid(
                    file,
                    at,
                    format!(
                        "archetype {:?} is not a leak archetype ID (L-*)",
                        e.archetype
                    ),
                ));
            }
            if !e.evidence.starts_with("https://") {
                return Err(invalid(
                    file,
                    at,
                    "evidence must be a public https URL of the source",
                ));
            }
            for (name, v) in [("effect", &e.effect), ("remedy", &e.remedy)] {
                if v.trim().is_empty() {
                    return Err(invalid(file, at, format!("{name} is empty")));
                }
            }
            if let Some(w) = &e.when {
                check_predicate(w).map_err(|m| invalid(file, at, m))?;
            }
        }
        Ok(parsed)
    }
}

/// Opening sentence of the generated reference.
pub const REFERENCE_OPENING: &str = "A row match opens a hypothesis and carries no verdict.";

/// Renders `skills/detect-leak-anywhere/references/known-leaks.md` from the table. The
/// text is a pure function of the entries, so a test can compare it with the committed
/// file.
pub fn render_reference(file: &KnownLeaksFile) -> String {
    use std::fmt::Write;
    let cell = |s: &str| s.replace('|', "\\|").replace('\n', " ");
    let mut out = String::new();
    out.push_str("# Known leaks\n\n");
    out.push_str(REFERENCE_OPENING);
    out.push_str(
        "\nEach row is documented behavior of a runtime or library that looks like a leak or hides one.\n\
         `dla probe` checks the rows against the project and writes a `true`, `false` or `maybe` verdict\n\
         to probe evidence. Cite a verdict when choosing what to test; it never confirms or refutes a finding.\n\
         This file is generated from `data/known-leaks.toml`; edit the table, not this file.\n\n",
    );
    out.push_str(
        "| Id | Runtime | Library | Versions | Archetype | Holder | Effect | Remedy | Source |\n",
    );
    out.push_str("|---|---|---|---|---|---|---|---|---|\n");
    for e in &file.leak {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | `{}` | {} | {} | {} | {} | {} |",
            e.id,
            e.runtime.as_str(),
            cell(&e.library),
            cell(&e.versions),
            e.archetype,
            e.holder.as_str(),
            cell(&e.effect),
            cell(&e.remedy),
            e.evidence
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_parse_and_compare() {
        assert_eq!(parse_version("go1.24.7"), Some(vec![1, 24, 7]));
        assert_eq!(parse_version("v2.0.1-rc1"), Some(vec![2, 0, 1]));
        assert_eq!(parse_version("4.1.100.Final"), Some(vec![4, 1, 100]));
        assert_eq!(parse_version("abc"), None);
        assert_eq!(parse_version("99999999999999999999999.1"), None);
        let r = VersionRange::parse(">=4.1.0, <4.2").unwrap();
        assert!(r.contains(&[4, 1, 100]));
        assert!(!r.contains(&[4, 2]));
        assert!(!r.contains(&[4, 0, 9]));
        let go = VersionRange::parse("<1.23").unwrap();
        assert!(go.contains(&[1, 21]) && go.contains(&[1, 22, 9]));
        assert!(!go.contains(&[1, 23]) && !go.contains(&[1, 23, 0]));
        assert!(VersionRange::parse("*").unwrap().contains(&[0]));
        assert!(VersionRange::parse("").is_err());
        assert!(VersionRange::parse(">=x").is_err());
        assert!(VersionRange::parse("*").unwrap().is_any());
        assert!(!VersionRange::parse("<2").unwrap().is_any());
    }

    #[test]
    fn unsupported_range_syntax_is_refused_not_misread() {
        for bad in [
            ">=1.0 <2.0",
            "^1.2",
            "~1.2",
            "~=1.2",
            "1!2.0",
            ">=1.0, <2.0 x",
            "4.+",
        ] {
            assert!(VersionRange::parse(bad).is_err(), "{bad}");
        }
        assert!(VersionRange::parse(">=go1.23, <v2").is_ok());
        assert_eq!(parse_version("1!2.0"), None);
    }

    #[test]
    fn parse_refuses_with_a_line() {
        let ok = r#"
[[leak]]
id = "go-x"
runtime = "go"
library = "go"
versions = "<1.23"
effect = "e"
holder = "registration-table"
archetype = "L-REGISTER"
evidence = "https://go.dev/doc/go1.23"
remedy = "r"
"#;
        assert_eq!(KnownLeaksFile::parse(ok, "k.toml").unwrap().leak.len(), 1);
        let bad_id = ok.replace("go-x", "x");
        let e = KnownLeaksFile::parse(&bad_id, "k.toml").unwrap_err();
        assert!(e.to_string().contains("k.toml:2"), "{e}");
        let bad_url = ok.replace("https://", "http://");
        assert!(KnownLeaksFile::parse(&bad_url, "k.toml").is_err());
        let bad_range = ok.replace("<1.23", "<>");
        assert!(KnownLeaksFile::parse(&bad_range, "k.toml").is_err());
        let dup = format!("{ok}{ok}");
        assert!(KnownLeaksFile::parse(&dup, "k.toml").is_err());
        let unknown = ok.replace("remedy", "remedies");
        assert!(KnownLeaksFile::parse(&unknown, "k.toml").is_err());
        for bad in ["N-WARM", "L-NOPE", ""] {
            let t = ok.replace("L-REGISTER", bad);
            let e = KnownLeaksFile::parse(&t, "k.toml").unwrap_err();
            assert!(e.to_string().contains("k.toml:2"), "{e}");
        }
        let missing = ok.replace("archetype = \"L-REGISTER\"\n", "");
        assert!(KnownLeaksFile::parse(&missing, "k.toml").is_err());
    }
}
