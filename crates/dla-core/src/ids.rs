//! Validated identifier newtypes.
//!
//! Evidence `E<n>`, finding `F<n>`, sibling hypothesis `F<n>.n<k>`, open item `O<n>`.
//! A worker slice prefix `<worker>/` (lowercase letters, digits, `-`) is accepted
//! everywhere; this crate never generates one.

use std::{borrow::Cow, fmt, str::FromStr};

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};

/// An identifier failed validation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid {kind} id {input:?}: expected {expected}")]
pub struct IdParseError {
    /// Which kind of ID was being parsed.
    pub kind: &'static str,
    /// The rejected input.
    pub input: String,
    /// The accepted shape.
    pub expected: &'static str,
}

fn split_prefix(s: &str) -> Option<(Option<String>, &str)> {
    match s.split_once('/') {
        None => Some((None, s)),
        Some((prefix, rest)) => {
            let ok = !prefix.is_empty()
                && prefix
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
            ok.then(|| (Some(prefix.to_owned()), rest))
        }
    }
}

/// Parses a canonical positive decimal (no sign, no leading zeros).
fn parse_number(s: &str) -> Option<u32> {
    if s.is_empty() || s.starts_with('0') || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

fn parse_simple(letter: char, s: &str) -> Option<(Option<String>, u32)> {
    let (prefix, body) = split_prefix(s)?;
    let n = parse_number(body.strip_prefix(letter)?)?;
    Some((prefix, n))
}

macro_rules! id_serde {
    ($ty:ident, $pattern:literal) => {
        impl TryFrom<String> for $ty {
            type Error = IdParseError;
            fn try_from(s: String) -> Result<Self, Self::Error> {
                s.parse()
            }
        }

        impl From<$ty> for String {
            fn from(id: $ty) -> String {
                id.to_string()
            }
        }

        impl JsonSchema for $ty {
            fn schema_name() -> Cow<'static, str> {
                Cow::Borrowed(stringify!($ty))
            }

            fn json_schema(_: &mut SchemaGenerator) -> Schema {
                json_schema!({ "type": "string", "pattern": $pattern })
            }
        }
    };
}

/// Evidence identifier, `E<n>` with an optional `<worker>/` prefix.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct EvidenceId {
    prefix: Option<String>,
    n: u32,
}

impl EvidenceId {
    /// `E<n>` without a prefix. `n` must be at least 1.
    pub fn new(n: u32) -> Self {
        Self {
            prefix: None,
            n: n.max(1),
        }
    }
    /// The worker prefix, if any.
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
    /// The numeric part.
    pub fn number(&self) -> u32 {
        self.n
    }
}

impl FromStr for EvidenceId {
    type Err = IdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_simple('E', s)
            .map(|(prefix, n)| Self { prefix, n })
            .ok_or_else(|| IdParseError {
                kind: "evidence",
                input: s.to_owned(),
                expected: "E<n> such as E3, optionally prefixed like jvm/E3",
            })
    }
}

impl fmt::Display for EvidenceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(p) = &self.prefix {
            write!(f, "{p}/")?;
        }
        write!(f, "E{}", self.n)
    }
}
id_serde!(EvidenceId, "^([a-z0-9-]+/)?E[1-9][0-9]*$");

/// Open item identifier, `O<n>` with an optional `<worker>/` prefix.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct OpenItemId {
    prefix: Option<String>,
    n: u32,
}

impl OpenItemId {
    /// `O<n>` without a prefix. `n` must be at least 1.
    pub fn new(n: u32) -> Self {
        Self {
            prefix: None,
            n: n.max(1),
        }
    }
    /// The worker prefix, if any.
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
    /// The numeric part.
    pub fn number(&self) -> u32 {
        self.n
    }
}

impl FromStr for OpenItemId {
    type Err = IdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_simple('O', s)
            .map(|(prefix, n)| Self { prefix, n })
            .ok_or_else(|| IdParseError {
                kind: "open item",
                input: s.to_owned(),
                expected: "O<n> such as O2, optionally prefixed like jvm/O2",
            })
    }
}

impl fmt::Display for OpenItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(p) = &self.prefix {
            write!(f, "{p}/")?;
        }
        write!(f, "O{}", self.n)
    }
}
id_serde!(OpenItemId, "^([a-z0-9-]+/)?O[1-9][0-9]*$");

/// Finding identifier: `F<n>`, or `F<n>.n<k>` for a sibling hypothesis, with an
/// optional `<worker>/` prefix.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FindingId {
    prefix: Option<String>,
    n: u32,
    sibling: Option<u32>,
}

impl FindingId {
    /// `F<n>` without a prefix. `n` must be at least 1.
    pub fn new(n: u32) -> Self {
        Self {
            prefix: None,
            n: n.max(1),
            sibling: None,
        }
    }
    /// The sibling ID `F<n>.n<k>` under this finding (keeps the prefix).
    pub fn sibling(&self, k: u32) -> Self {
        Self {
            prefix: self.prefix.clone(),
            n: self.n,
            sibling: Some(k.max(1)),
        }
    }
    /// The worker prefix, if any.
    pub fn prefix(&self) -> Option<&str> {
        self.prefix.as_deref()
    }
    /// The numeric part after `F`.
    pub fn number(&self) -> u32 {
        self.n
    }
    /// The `k` of `F<n>.n<k>`, if this is a sibling.
    pub fn sibling_number(&self) -> Option<u32> {
        self.sibling
    }
    /// True for `F<n>.n<k>`.
    pub fn is_sibling(&self) -> bool {
        self.sibling.is_some()
    }
    /// The parent `F<n>` of a sibling, `None` for a top-level finding.
    pub fn parent(&self) -> Option<FindingId> {
        self.sibling.map(|_| Self {
            prefix: self.prefix.clone(),
            n: self.n,
            sibling: None,
        })
    }
}

impl FromStr for FindingId {
    type Err = IdParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || IdParseError {
            kind: "finding",
            input: s.to_owned(),
            expected: "F<n> such as F3, or F<n>.n<k> such as F3.n1, optionally prefixed like jvm/F3",
        };
        let (prefix, body) = split_prefix(s).ok_or_else(err)?;
        let body = body.strip_prefix('F').ok_or_else(err)?;
        let (num, sib) = match body.split_once(".n") {
            Some((a, b)) => (a, Some(parse_number(b).ok_or_else(err)?)),
            None => (body, None),
        };
        let n = parse_number(num).ok_or_else(err)?;
        Ok(Self {
            prefix,
            n,
            sibling: sib,
        })
    }
}

impl fmt::Display for FindingId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(p) = &self.prefix {
            write!(f, "{p}/")?;
        }
        write!(f, "F{}", self.n)?;
        if let Some(k) = self.sibling {
            write!(f, ".n{k}")?;
        }
        Ok(())
    }
}
id_serde!(FindingId, "^([a-z0-9-]+/)?F[1-9][0-9]*(\\.n[1-9][0-9]*)?$");
