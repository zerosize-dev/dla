//! `jfr print --events jdk.OldObjectSample <file.jfr>` text to an [`AllocProfile`].
//!
//! Each event is one sampled live object with its allocation stack and, when the
//! recording used `path-to-gc-roots=true`, the reference chain to a GC root. Samples are
//! grouped by (allocation stack, root path, sampled type). The root path starts with a
//! holder key in the same form as the HPROF reader's (`static:<pkg.Class>.<field>`), then
//! the class names of the chain from the holder down to the sampled object.
//!
//! The text form is parsed rather than JSON: it is about 25 times smaller for the same
//! events. Only class names, field names and sizes are kept; the descriptions JFR prints
//! after a type (`Size: 60000`) and any string contents are dropped.

use std::collections::BTreeMap;

use super::{ParseError, deproxy};
use crate::model::{AllocGroup, AllocProfile, Frame};

/// The `source` of the profiles this module returns.
pub const SOURCE: &str = "jfr-old-object-sample";

/// Chain entries kept in a root path after the holder key.
const MAX_CHAIN: usize = 12;

/// One sampled object.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Sample {
    /// Size in bytes as JFR printed it (`2.0 kB` is 2048).
    pub size: u64,
    /// The sampled object's type, array lengths removed (`byte[]`).
    pub kind: String,
    /// Allocation stack, innermost first. JFR truncates it (`...`), see [`Sample::stack_truncated`].
    pub frames: Vec<Frame>,
    /// True when JFR printed `...` after the frames.
    pub stack_truncated: bool,
    /// Holder key, `None` when the chain has no static field (a thread or handle root).
    pub holder: Option<String>,
    /// Holder key (or `root:<system>` when there is none), then the chain's class names
    /// from the holder down to the sampled object.
    pub root_path: Vec<String>,
}

/// Parses the printed events into samples, in file order.
pub fn parse_samples(text: &str) -> Result<Vec<Sample>, ParseError> {
    let mut out = Vec::new();
    let mut ev: Option<Event> = None;
    let mut saw_other_event = false;
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim_end();
        let lno = i + 1;
        let Some(e) = ev.as_mut() else {
            if line.starts_with("jdk.OldObjectSample") && line.trim_end().ends_with('{') {
                ev = Some(Event::default());
            } else if !line.trim().is_empty() && !line.starts_with(' ') {
                saw_other_event = true;
            }
            continue;
        };
        if line == "}" {
            let done = ev.take().unwrap_or_default();
            out.push(done.finish(lno)?);
            continue;
        }
        e.line(line);
    }
    if ev.is_some() {
        return Err(ParseError::new(format!(
            "the jdk.OldObjectSample event starting before line {} is not closed: the output is truncated",
            text.lines().count() + 1
        )));
    }
    if out.is_empty() && saw_other_event {
        return Err(ParseError::new(
            "no jdk.OldObjectSample events found; expected the output of `jfr print --events jdk.OldObjectSample <file.jfr>` (text, not --json)",
        ));
    }
    Ok(out)
}

/// Parses the printed events and groups them into an [`AllocProfile`] (`sampled`).
pub fn parse(text: &str) -> Result<AllocProfile, ParseError> {
    Ok(group(&parse_samples(text)?))
}

/// Groups samples by (stack, root path, type). Groups come out largest first by bytes,
/// then by key, so the order is stable.
pub fn group(samples: &[Sample]) -> AllocProfile {
    // (stack, root path, type) to (first sample, bytes, samples).
    type GroupKey = (Vec<String>, Vec<String>, String);
    let mut by: BTreeMap<GroupKey, (usize, u64, u64)> = BTreeMap::new();
    for (idx, s) in samples.iter().enumerate() {
        let stack_key: Vec<String> = s
            .frames
            .iter()
            .map(|f| format!("{}:{}", f.function, f.line.map_or(-1, i64::from)))
            .collect();
        let e = by
            .entry((stack_key, s.root_path.clone(), s.kind.clone()))
            .or_insert((idx, 0, 0));
        e.1 += s.size;
        e.2 += 1;
    }
    let mut groups: Vec<AllocGroup> = by
        .into_values()
        .map(|(idx, bytes, blocks)| {
            let s = &samples[idx];
            AllocGroup {
                frames: s.frames.clone(),
                live_bytes: bytes,
                live_blocks: blocks,
                total_bytes: None,
                root_path: s.root_path.clone(),
                kind: Some(s.kind.clone()),
            }
        })
        .collect();
    groups.sort_by(|a, b| {
        b.live_bytes
            .cmp(&a.live_bytes)
            .then_with(|| b.live_blocks.cmp(&a.live_blocks))
    });
    AllocProfile {
        source: SOURCE.to_owned(),
        timestamp_ms: None,
        sampled: true,
        groups,
    }
}

#[derive(PartialEq, Clone, Copy, Default)]
enum Section {
    #[default]
    Top,
    Object,
    Root,
    Stack,
}

#[derive(Default)]
struct Event {
    section: Section,
    size: u64,
    /// Raw chain lines, leaf first.
    chain: Vec<String>,
    root_system: Option<String>,
    frames: Vec<Frame>,
    truncated: bool,
}

impl Event {
    fn line(&mut self, line: &str) {
        let t = line.trim();
        match self.section {
            Section::Top => {
                if let Some(v) = t.strip_prefix("objectSize = ") {
                    self.size = parse_size(v);
                } else if t.starts_with("object = ") && t.ends_with('[') {
                    self.section = Section::Object;
                } else if t.starts_with("root = ") && t.ends_with('{') {
                    self.section = Section::Root;
                } else if t.starts_with("stackTrace = ") && t.ends_with('[') {
                    self.section = Section::Stack;
                }
            }
            Section::Object => {
                if t == "]" {
                    self.section = Section::Top;
                } else if !t.is_empty() {
                    self.chain.push(t.to_owned());
                }
            }
            Section::Root => {
                if t == "}" {
                    self.section = Section::Top;
                } else if let Some(v) = t.strip_prefix("system = ") {
                    self.root_system = Some(v.trim_matches('"').to_owned());
                }
            }
            Section::Stack => {
                if t == "]" {
                    self.section = Section::Top;
                } else if t == "..." {
                    self.truncated = true;
                } else if !t.is_empty() {
                    self.frames.push(parse_frame(t));
                }
            }
        }
    }

    fn finish(self, lno: usize) -> Result<Sample, ParseError> {
        if self.section != Section::Top {
            return Err(ParseError::new(format!(
                "line {lno}: a jdk.OldObjectSample event ends inside a bracketed block"
            )));
        }
        let Some(leaf) = self.chain.first() else {
            return Err(ParseError::new(format!(
                "line {lno}: a jdk.OldObjectSample event has no `object = [ ... ]` block"
            )));
        };
        let kind = clean_type(leaf.split_whitespace().next().unwrap_or(""));
        // Line i reads `field : Type desc`: Type has `field` pointing at line i-1.
        let mut holder = None;
        let mut holder_at = None;
        for (i, l) in self.chain.iter().enumerate().skip(1) {
            let Some((field, rest)) = l.split_once(" : ") else {
                continue;
            };
            let mut it = rest.splitn(2, ' ');
            let ty = it.next().unwrap_or("");
            let desc = it.next().unwrap_or("");
            if ty == "java.lang.Class"
                && let Some(class) = desc.strip_prefix("Class Name: ")
            {
                let class = deproxy::normalize(class.trim());
                holder = Some(format!("static:{class}.{field}"));
                holder_at = Some(i);
                break;
            }
        }
        let mut root_path = Vec::new();
        match (&holder, holder_at) {
            (Some(h), Some(at)) => {
                root_path.push(h.clone());
                // From the holder down to the leaf: lines at-1 .. 0.
                for l in self.chain[..at].iter().rev() {
                    root_path.push(chain_type(l));
                }
            }
            _ => {
                root_path.push(format!(
                    "root:{}",
                    self.root_system.as_deref().unwrap_or("unknown")
                ));
                for l in self.chain.iter().rev() {
                    root_path.push(chain_type(l));
                }
            }
        }
        if root_path.len() > MAX_CHAIN + 1 {
            // Keep the holder (or root) and the entries nearest the sampled object.
            let keep_tail = MAX_CHAIN;
            let tail: Vec<String> = root_path[root_path.len() - keep_tail..].to_vec();
            root_path.truncate(1);
            root_path.extend(tail);
        }
        Ok(Sample {
            size: self.size,
            kind,
            frames: self.frames,
            stack_truncated: self.truncated,
            holder,
            root_path,
        })
    }
}

/// The class name of one chain line, normalized and without array lengths.
fn chain_type(line: &str) -> String {
    let ty = match line.split_once(" : ") {
        Some((_, rest)) => rest.split_whitespace().next().unwrap_or(""),
        None => line.split_whitespace().next().unwrap_or(""),
    };
    clean_type(ty)
}

/// `byte[2048]` to `byte[]`; proxy suffixes removed.
fn clean_type(ty: &str) -> String {
    let mut s = String::with_capacity(ty.len());
    let mut in_len = false;
    for c in ty.chars() {
        match c {
            '[' => {
                in_len = true;
                s.push('[');
            }
            ']' => {
                in_len = false;
                s.push(']');
            }
            c if in_len && c.is_ascii_digit() => {}
            c => s.push(c),
        }
    }
    deproxy::normalize(&s).into_owned()
}

/// `com.acme.Session.<init>(String, int, String) line: 9`.
fn parse_frame(t: &str) -> Frame {
    let (head, line) = match t.rsplit_once(" line: ") {
        Some((h, l)) => (h, l.trim().parse::<i64>().ok().filter(|n| *n >= 0)),
        None => (t, None),
    };
    let function = head.split('(').next().unwrap_or(head).trim();
    Frame {
        function: function.to_owned(),
        file: None,
        line: line.map(|n| n as u32),
        module: None,
    }
}

/// `2.0 kB`, `512.0 kB`, `32 bytes`, `1.5 MB`, using powers of 1024 as JFR does.
fn parse_size(v: &str) -> u64 {
    let v = v.trim();
    let mut it = v.split_whitespace();
    let num: f64 = it.next().and_then(|n| n.parse().ok()).unwrap_or(0.0);
    let mult = match it.next().unwrap_or("bytes") {
        "kB" | "KB" => 1024.0,
        "MB" => 1024.0 * 1024.0,
        "GB" => 1024.0 * 1024.0 * 1024.0,
        _ => 1.0,
    };
    (num * mult).round() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVENT: &str = "jdk.OldObjectSample {
  startTime = 07:25:29.099 (2026-11-09)
  objectSize = 2.0 kB
  object =  [
    byte[2048]
    payload : com.acme.Session
    value : java.util.HashMap$Node
    [35257] : java.util.HashMap$Node[131072]
    table : java.util.HashMap Size: 60000
    SESSIONS : java.lang.Class Class Name: com.acme.SessionCache
    [4] : java.lang.Object[10]
  ]
  root = {
    description = N/A
    system = \"VM Global\"
    type = \"Global Object Handle\"
  }
  stackTrace = [
    com.acme.Session.<init>(String, int, String) line: 9
    com.acme.SessionService$$SpringCGLIB$$0.open(int) line: 4
    ...
  ]
}
";

    #[test]
    fn one_event() {
        let s = parse_samples(EVENT).unwrap();
        assert_eq!(s.len(), 1);
        let s = &s[0];
        assert_eq!(s.size, 2048);
        assert_eq!(s.kind, "byte[]");
        assert_eq!(
            s.holder.as_deref(),
            Some("static:com.acme.SessionCache.SESSIONS")
        );
        assert_eq!(
            s.root_path,
            [
                "static:com.acme.SessionCache.SESSIONS",
                "java.util.HashMap",
                "java.util.HashMap$Node[]",
                "java.util.HashMap$Node",
                "com.acme.Session",
                "byte[]"
            ]
        );
        assert_eq!(s.frames[0].function, "com.acme.Session.<init>");
        assert_eq!(s.frames[0].line, Some(9));
        assert!(s.stack_truncated);
    }

    #[test]
    fn empty_input_is_an_empty_profile() {
        let p = parse("").unwrap();
        assert!(p.groups.is_empty());
        assert!(p.sampled);
    }

    #[test]
    fn rejects_other_text() {
        let e = parse("Total = 3\nsomething else\n").unwrap_err();
        assert!(e.message().contains("jfr print"), "{e}");
    }

    #[test]
    fn rejects_truncated_event() {
        let cut = &EVENT[..EVENT.len() - 40];
        let e = parse(cut).unwrap_err();
        assert!(e.message().contains("truncated"), "{e}");
    }

    #[test]
    fn sizes() {
        assert_eq!(parse_size("32 bytes"), 32);
        assert_eq!(parse_size("1.5 MB"), 1_572_864);
        assert_eq!(parse_size("garbage"), 0);
    }
}
