//! Source map v3: a base64 VLQ `mappings` decoder, index maps (`sections`), and a
//! lookup from a generated line and column to the original file and line.
//!
//! The decoded size is capped, and every delta is checked, so a hostile map cannot
//! exhaust memory or overflow.

use serde_json::Value;

use super::NodeError;

/// A decoded position in an original source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Seg {
    gen_col: u32,
    /// Index into `sources`, `u32::MAX` for a segment with no source.
    src: u32,
    line: u32,
    col: u32,
}

/// A decoded map.
#[derive(Debug, Default, Clone)]
pub struct SourceMap {
    sources: Vec<String>,
    /// Segments of each generated line, sorted by generated column.
    lines: Vec<Vec<Seg>>,
    segments: usize,
}

/// An original position, zero-based.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Original<'a> {
    pub source: &'a str,
    pub line: u32,
    pub column: u32,
}

fn b64(c: u8) -> Option<u32> {
    match c {
        b'A'..=b'Z' => Some(u32::from(c - b'A')),
        b'a'..=b'z' => Some(u32::from(c - b'a') + 26),
        b'0'..=b'9' => Some(u32::from(c - b'0') + 52),
        b'+' | b'-' => Some(62),
        b'/' | b'_' => Some(63),
        _ => None,
    }
}

/// Reads one VLQ number starting at `*at`.
fn vlq(bytes: &[u8], at: &mut usize) -> Result<i64, NodeError> {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    loop {
        let Some(&c) = bytes.get(*at) else {
            return Err(NodeError::Malformed(
                "a source map segment ends inside a number".to_owned(),
            ));
        };
        let d = b64(c).ok_or_else(|| {
            NodeError::Malformed(format!("`{}` is not a source map digit", c as char))
        })?;
        *at += 1;
        if shift > 55 {
            return Err(NodeError::Malformed(
                "a source map number is too long".to_owned(),
            ));
        }
        result |= u64::from(d & 31) << shift;
        shift += 5;
        if d & 32 == 0 {
            break;
        }
    }
    let magnitude = (result >> 1) as i64;
    Ok(if result & 1 == 1 {
        -magnitude
    } else {
        magnitude
    })
}

fn add(base: i64, delta: i64, what: &str) -> Result<i64, NodeError> {
    base.checked_add(delta)
        .filter(|v| (0..=i64::from(u32::MAX)).contains(v))
        .ok_or_else(|| NodeError::Malformed(format!("the source map {what} is out of range")))
}

impl SourceMap {
    /// Parses a map from its JSON text; at most `max_segments` segments are decoded.
    pub fn parse(json: &str, max_segments: usize) -> Result<Self, NodeError> {
        let v: Value = serde_json::from_str(json)
            .map_err(|e| NodeError::Malformed(format!("the source map is not JSON: {e}")))?;
        let mut m = SourceMap::default();
        m.add_map(&v, 0, 0, max_segments)?;
        Ok(m)
    }

    fn add_map(
        &mut self,
        v: &Value,
        line_off: usize,
        col_off: u32,
        max: usize,
    ) -> Result<(), NodeError> {
        if let Some(sections) = v.get("sections").and_then(Value::as_array) {
            for s in sections {
                let off = s.get("offset");
                let l = off
                    .and_then(|o| o.get("line"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let c = off
                    .and_then(|o| o.get("column"))
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let Some(map) = s.get("map") else { continue };
                let l = usize::try_from(l).unwrap_or(usize::MAX);
                if l > 50_000_000 {
                    return Err(NodeError::TooLarge(
                        "a source map section starts past line 50 million".to_owned(),
                    ));
                }
                let c = u32::try_from(c).unwrap_or(u32::MAX);
                self.add_map(map, line_off + l, c, max)?;
            }
            return Ok(());
        }
        let mappings = v.get("mappings").and_then(Value::as_str).ok_or_else(|| {
            NodeError::NotThisFormat("the source map has no `mappings`".to_owned())
        })?;
        let base = u32::try_from(self.sources.len()).unwrap_or(u32::MAX);
        let root = v.get("sourceRoot").and_then(Value::as_str).unwrap_or("");
        if let Some(srcs) = v.get("sources").and_then(Value::as_array) {
            for s in srcs {
                let s = s.as_str().unwrap_or_default();
                self.sources.push(if root.is_empty() || s.starts_with('/') {
                    s.to_owned()
                } else {
                    format!("{}/{s}", root.trim_end_matches('/'))
                });
            }
        }
        let bytes = mappings.as_bytes();
        let (mut src, mut line, mut col) = (0i64, 0i64, 0i64);
        let mut at = 0usize;
        let mut gen_line = line_off;
        let mut gen_col: i64 = 0;
        while at <= bytes.len() {
            // Segments of one generated line run to `;`.
            let end = bytes[at..]
                .iter()
                .position(|&b| b == b';')
                .map_or(bytes.len(), |p| at + p);
            gen_col = 0;
            while at < end {
                if bytes[at] == b',' {
                    at += 1;
                    continue;
                }
                gen_col = add(gen_col, vlq(bytes, &mut at)?, "column")?;
                let mut seg = Seg {
                    gen_col: u32::try_from(gen_col).unwrap_or(u32::MAX),
                    src: u32::MAX,
                    line: 0,
                    col: 0,
                };
                if at < end && bytes[at] != b',' {
                    src = add(src, vlq(bytes, &mut at)?, "source index")?;
                    line = add(line, vlq(bytes, &mut at)?, "line")?;
                    col = add(col, vlq(bytes, &mut at)?, "column")?;
                    if at < end && bytes[at] != b',' {
                        // The name index is read and dropped.
                        vlq(bytes, &mut at)?;
                    }
                    let s = base.saturating_add(u32::try_from(src).unwrap_or(u32::MAX));
                    if (s as usize) < self.sources.len() {
                        seg.src = s;
                        seg.line = u32::try_from(line).unwrap_or(u32::MAX);
                        seg.col = u32::try_from(col).unwrap_or(u32::MAX);
                    }
                }
                self.segments += 1;
                if self.segments > max {
                    return Err(NodeError::TooLarge(format!(
                        "the source map has more than {max} segments"
                    )));
                }
                if self.lines.len() <= gen_line {
                    if gen_line > 50_000_000 {
                        return Err(NodeError::TooLarge(
                            "the source map has more than 50 million lines".to_owned(),
                        ));
                    }
                    self.lines.resize(gen_line + 1, Vec::new());
                }
                // An index map's column offset applies to the section's first line only.
                let off = if gen_line == line_off { col_off } else { 0 };
                seg.gen_col = seg.gen_col.saturating_add(off);
                self.lines[gen_line].push(seg);
            }
            at = end + 1;
            gen_line += 1;
        }
        let _ = gen_col;
        for l in &mut self.lines {
            l.sort_by_key(|s| s.gen_col);
        }
        Ok(())
    }

    /// Number of decoded segments.
    pub fn segments(&self) -> usize {
        self.segments
    }

    /// The original position of a generated position (zero-based line and column): the
    /// last segment on the line that starts at or before the column.
    pub fn lookup(&self, line: u32, column: u32) -> Option<Original<'_>> {
        let segs = self.lines.get(line as usize)?;
        let i = segs.partition_point(|s| s.gen_col <= column);
        let s = segs.get(i.checked_sub(1)?)?;
        (s.src != u32::MAX).then(|| Original {
            source: &self.sources[s.src as usize],
            line: s.line,
            column: s.col,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vlq_numbers_decode_with_signs_and_continuations() {
        let one = |s: &str| vlq(s.as_bytes(), &mut 0).unwrap();
        assert_eq!(one("A"), 0);
        assert_eq!(one("C"), 1);
        assert_eq!(one("D"), -1);
        assert_eq!(one("gB"), 16);
        assert_eq!(one("2H"), 123);
        assert_eq!(one("3H"), -123);
        assert!(vlq(b"g", &mut 0).is_err());
        assert!(vlq(b"!", &mut 0).is_err());
        assert!(vlq(&[b'g'; 20], &mut 0).is_err());
    }

    #[test]
    fn a_map_resolves_a_column_to_its_source_line() {
        // line 0: col 0 -> a.ts 0:0; col 23 -> a.ts 1:15 (the esbuild-style case)
        // line 1: col 4 -> b.ts 4:2
        let json = r#"{"version":3,"sources":["../src/a.ts","../src/b.ts"],"mappings":"AAAA,uBACeA;IAIAC"}"#;
        let m = SourceMap::parse(json, 1000).unwrap();
        let _ = m.segments();
        let o = m.lookup(0, 30).unwrap();
        assert_eq!(o.source, "../src/a.ts");
        let first = m.lookup(0, 0).unwrap();
        assert_eq!((first.line, first.column), (0, 0));
        assert!(m.lookup(0, 0).is_some());
        assert!(m.lookup(5, 0).is_none());
        // Before the first segment of a line there is nothing.
        assert!(m.lookup(1, 1).is_none());
    }

    #[test]
    fn index_maps_offset_their_sections() {
        let json = r#"{"version":3,"sections":[
          {"offset":{"line":0,"column":0},"map":{"version":3,"sources":["a.js"],"mappings":"AAAA"}},
          {"offset":{"line":2,"column":10},"map":{"version":3,"sources":["b.js"],"mappings":"AAEA;AACA"}}]}"#;
        let m = SourceMap::parse(json, 100).unwrap();
        assert_eq!(m.lookup(0, 0).unwrap().source, "a.js");
        let b = m.lookup(2, 10).unwrap();
        assert_eq!((b.source, b.line), ("b.js", 2));
        assert!(m.lookup(2, 5).is_none());
        // The second line of the section is not shifted by the column offset.
        let b2 = m.lookup(3, 0).unwrap();
        assert_eq!((b2.source, b2.line), ("b.js", 3));
    }

    #[test]
    fn hostile_maps_are_bounded() {
        let many = format!(
            "{{\"version\":3,\"sources\":[\"a\"],\"mappings\":\"{}\"}}",
            "AAAA,".repeat(500)
        );
        assert!(SourceMap::parse(&many, 100).is_err());
        let wild = r#"{"version":3,"sources":["a"],"mappings":"+/////////////////D"}"#;
        assert!(SourceMap::parse(wild, 100).is_err());
        assert!(SourceMap::parse("{\"version\":3}", 100).is_err());
        assert!(SourceMap::parse("nope", 100).is_err());
    }
}
