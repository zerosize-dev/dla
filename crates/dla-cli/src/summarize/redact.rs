//! Redaction for summarizer output.
//!
//! Names of classes, functions, files, categories and states are program structure and
//! stay as they are, with counts and sizes. String values that are data do not: the
//! value of an assignment (`API_KEY=...`, `token: ...`), a URL query (`?a=b&c=d`), a quoted
//! string literal and a token that looks like a key. Each becomes a shape
//! (`dla_core::redact::shape`: length, character-class pattern, a salted hash) unless
//! the run's consent allows raw values.

use dla_core::{RunState, redact::shape};
use serde_json::{Value, json};

/// Longest raw value kept when raw values are allowed.
const RAW_CLIP: usize = 120;

/// Redacts string values for one run.
#[derive(Debug, Clone)]
pub struct Redactor {
    salt: Vec<u8>,
    raw: bool,
}

impl Redactor {
    /// From the run state: the salt, and whether consent allows raw values.
    pub fn from_state(state: &RunState) -> Self {
        Self {
            salt: state.salt_bytes(),
            raw: state.consent.as_ref().is_some_and(|c| c.raw_values),
        }
    }

    /// A redactor with an explicit salt and mode (tests).
    #[cfg(test)]
    pub fn new(salt: &[u8], raw: bool) -> Self {
        Self {
            salt: salt.to_vec(),
            raw,
        }
    }

    /// `"shapes"` or `"raw"`, for the `redaction` stanza of an output.
    pub fn mode(&self) -> &'static str {
        if self.raw { "raw" } else { "shapes" }
    }

    /// One value as a string: its shape (`<9:a+_9+:3f2a9c0d11ab>`), or the value itself
    /// (clipped) with raw consent.
    pub fn value(&self, v: &str) -> String {
        if self.raw {
            return v.chars().take(RAW_CLIP).collect();
        }
        let s = shape(v, &self.salt);
        format!("<{}:{}:{}>", s.len, s.pattern, s.hash)
    }

    /// One value as JSON: `{len, pattern, hash}`, or the string with raw consent. For
    /// adapters that show a field value.
    #[allow(dead_code)]
    pub fn value_json(&self, v: &str) -> Value {
        if self.raw {
            return json!(self.value(v));
        }
        let s = shape(v, &self.salt);
        json!({"len": s.len, "pattern": s.pattern, "hash": s.hash})
    }

    /// A name that may carry data: class, function, file, key and category names pass
    /// unchanged, but the values inside them do not. Replaces, in order, the query of a
    /// URL, the value after `=` or `: ` in an assignment, quoted literals, and any token
    /// of 20 or more key-like characters (letters and digits mixed, no `.`, `/` or `$`).
    pub fn name(&self, s: &str) -> String {
        if self.raw {
            return s.to_owned();
        }
        let s1 = self.queries(s);
        let s2 = self.assignments(&s1);
        let s3 = self.quoted(&s2);
        self.key_like(&s3)
    }

    fn queries(&self, s: &str) -> String {
        let Some(q) = s.find('?') else {
            return s.to_owned();
        };
        let rest = &s[q + 1..];
        // A query has at least one key=value pair; a lone `?` (a generic wildcard) stays.
        if !rest.contains('=') {
            return s.to_owned();
        }
        let end = rest
            .find(|c: char| c.is_whitespace() || matches!(c, ')' | '"' | '\'' | '>'))
            .unwrap_or(rest.len());
        format!("{}?{}{}", &s[..q], self.value(&rest[..end]), &rest[end..])
    }

    fn assignments(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut chars = s.char_indices().peekable();
        let mut last = 0;
        while let Some((i, c)) = chars.next() {
            let sep = match c {
                '=' => Some(1),
                ':' if s[i + 1..].starts_with(' ') && is_key_before(s, i) => Some(2),
                _ => None,
            };
            let Some(skip) = sep else { continue };
            if c == '=' && !is_key_before(s, i) {
                continue;
            }
            let start = i + skip;
            if s[start..].starts_with('<') {
                continue;
            }
            let rest = &s[start..];
            let len = rest
                .find(|c: char| {
                    c.is_whitespace() || matches!(c, ',' | ';' | ')' | '&' | '"' | '\'')
                })
                .unwrap_or(rest.len());
            if len == 0 {
                continue;
            }
            out.push_str(&s[last..start]);
            out.push_str(&self.value(&rest[..len]));
            last = start + len;
            while chars.peek().is_some_and(|(j, _)| *j < last) {
                chars.next();
            }
        }
        out.push_str(&s[last..]);
        out
    }

    fn quoted(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut rest = s;
        while let Some(i) = rest.find('"') {
            let after = &rest[i + 1..];
            let Some(j) = after.find('"') else { break };
            out.push_str(&rest[..i + 1]);
            if j > 0 {
                out.push_str(&self.value(&after[..j]));
            }
            out.push('"');
            rest = &after[j + 1..];
        }
        out.push_str(rest);
        out
    }

    fn key_like(&self, s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let mut token = String::new();
        let flush = |token: &mut String, out: &mut String, this: &Self| {
            if is_key_like(token) {
                out.push_str(&this.value(token));
            } else {
                out.push_str(token);
            }
            token.clear();
        };
        for c in s.chars() {
            if c.is_ascii_alphanumeric() {
                token.push(c);
            } else {
                flush(&mut token, &mut out, self);
                out.push(c);
            }
        }
        flush(&mut token, &mut out, self);
        out
    }
}

/// True when the text before byte `i` ends in an identifier (the left side of an
/// assignment).
fn is_key_before(s: &str, i: usize) -> bool {
    s[..i]
        .chars()
        .next_back()
        .is_some_and(|c| c.is_alphanumeric() || matches!(c, '_' | '-'))
}

/// A token that looks like a credential: long, with digits and letters mixed (or an
/// all-caps/digit key id).
fn is_key_like(t: &str) -> bool {
    if t.len() < 20 {
        return false;
    }
    let digits = t.chars().filter(char::is_ascii_digit).count();
    let upper = t.chars().filter(char::is_ascii_uppercase).count();
    let lower = t.chars().filter(char::is_ascii_lowercase).count();
    let letters = upper + lower;
    if digits == 0 || letters == 0 {
        return false;
    }
    // A readable identifier (camel case or snake case) has far more letters than digits
    // and, if snake case, short words; hex and base64 runs mix them evenly.
    let mixed = digits * 6 >= t.len();
    let key_id = upper + digits >= t.len() - 1 && t.len() >= 16;
    mixed || key_id
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r() -> Redactor {
        Redactor::new(b"salt", false)
    }

    #[test]
    fn structure_passes_and_values_become_shapes() {
        let r = r();
        for keep in [
            "shop.codec.Audit.trail",
            "pkg.Svc$$Lambda.run(Svc.java:42)",
            "java.util.concurrent.ConcurrentHashMap$Node[]",
            "src/lib.rs:40",
            "main::{{closure}}",
            "thread-state:RUNNABLE",
            "fd.socket",
        ] {
            assert_eq!(r.name(keep), keep, "{keep}");
        }
        let n = r.name("env:API_KEY=AKIAIOSFODNN7EXAMPLE");
        assert!(!n.contains("AKIA"), "{n}");
        assert!(n.starts_with("env:API_KEY=<"), "{n}");
        let n = r.name("GET /pay?token=s3cr3t&user=alice");
        assert!(!n.contains("s3cr3t") && !n.contains("alice"), "{n}");
        assert!(n.starts_with("GET /pay?<"), "{n}");
        let n = r.name("failed for \"hunter2-password\" now");
        assert!(!n.contains("hunter2"), "{n}");
        let n = r.name("worker-AKIAIOSFODNN7EXAMPLE1");
        assert!(!n.contains("AKIA"), "{n}");
    }

    #[test]
    fn values_are_stable_within_a_run_and_differ_by_salt() {
        let a = Redactor::new(b"one", false);
        let b = Redactor::new(b"two", false);
        assert_eq!(a.value("secret"), a.value("secret"));
        assert_ne!(a.value("secret"), b.value("secret"));
        assert_ne!(a.value("secret"), a.value("secret2"));
        assert!(a.value("user_42@x.io").contains("a+_9+@a.a+"));
    }

    #[test]
    fn raw_consent_keeps_values_clipped() {
        let r = Redactor::new(b"s", true);
        assert_eq!(r.name("K=AKIA123"), "K=AKIA123");
        assert_eq!(r.value("abc"), "abc");
        assert_eq!(r.value(&"x".repeat(500)).len(), RAW_CLIP);
        assert_eq!(r.mode(), "raw");
    }
}
