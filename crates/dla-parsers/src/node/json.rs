//! A small streaming JSON reader for the Node formats.
//!
//! A V8 heap snapshot is one JSON object whose arrays hold millions of integers, so it
//! is read token by token into `Vec<u32>` instead of building a value tree. The reader
//! is bounded: strings, raw values and arrays have limits, and nesting is counted.

use std::io::{self, Read};

use super::NodeError;

/// Bytes read from the source at a time.
const CHUNK: usize = 64 << 10;
/// Deepest nesting `skip_value` and `raw_value` follow: a depth counter, not recursion, so
/// the trace tree of a deep call stack is skipped; only a damaged file runs into it.
const MAX_DEPTH: usize = 4 << 20;

pub(crate) struct Reader<R: Read> {
    src: R,
    buf: Vec<u8>,
    pos: usize,
    len: usize,
    /// Bytes consumed so far, for messages.
    pub(crate) offset: u64,
}

impl<R: Read> Reader<R> {
    pub(crate) fn new(src: R) -> Self {
        Self {
            src,
            buf: vec![0; CHUNK],
            pos: 0,
            len: 0,
            offset: 0,
        }
    }

    fn fill(&mut self) -> Result<bool, NodeError> {
        if self.pos < self.len {
            return Ok(true);
        }
        loop {
            match self.src.read(&mut self.buf) {
                Ok(0) => return Ok(false),
                Ok(n) => {
                    self.pos = 0;
                    self.len = n;
                    return Ok(true);
                }
                Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                Err(e) => return Err(NodeError::Io(e)),
            }
        }
    }

    pub(crate) fn peek(&mut self) -> Result<Option<u8>, NodeError> {
        Ok(self.fill()?.then(|| self.buf[self.pos]))
    }

    fn bump(&mut self) -> Result<Option<u8>, NodeError> {
        let b = self.peek()?;
        if b.is_some() {
            self.pos += 1;
            self.offset += 1;
        }
        Ok(b)
    }

    fn bad(&self, what: &str) -> NodeError {
        NodeError::Malformed(format!("{what} at byte {}", self.offset))
    }

    pub(crate) fn skip_ws(&mut self) -> Result<(), NodeError> {
        while let Some(b) = self.peek()? {
            if matches!(b, b' ' | b'\n' | b'\r' | b'\t') {
                self.pos += 1;
                self.offset += 1;
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Consumes `want` after whitespace.
    pub(crate) fn expect(&mut self, want: u8) -> Result<(), NodeError> {
        self.skip_ws()?;
        match self.bump()? {
            Some(b) if b == want => Ok(()),
            Some(b) => Err(self.bad(&format!(
                "expected `{}`, found `{}`",
                want as char, b as char
            ))),
            None => Err(NodeError::Truncated(format!(
                "the file ends where `{}` was expected",
                want as char
            ))),
        }
    }

    /// Consumes `want` when it is next.
    pub(crate) fn eat(&mut self, want: u8) -> Result<bool, NodeError> {
        self.skip_ws()?;
        if self.peek()? == Some(want) {
            self.pos += 1;
            self.offset += 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// A string, up to `limit` bytes of decoded text.
    pub(crate) fn string(&mut self, limit: usize) -> Result<String, NodeError> {
        self.expect(b'"')?;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let Some(b) = self.bump()? else {
                return Err(NodeError::Truncated(
                    "the file ends inside a string".to_owned(),
                ));
            };
            match b {
                b'"' => break,
                b'\\' => {
                    let Some(e) = self.bump()? else {
                        return Err(NodeError::Truncated(
                            "the file ends inside a string".to_owned(),
                        ));
                    };
                    let c = match e {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{8}',
                        b'f' => '\u{c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => self.unicode()?,
                        _ => return Err(self.bad("bad escape")),
                    };
                    let mut tmp = [0u8; 4];
                    out.extend_from_slice(c.encode_utf8(&mut tmp).as_bytes());
                }
                _ => out.push(b),
            }
            if out.len() > limit {
                return Err(NodeError::Malformed(format!(
                    "a string is longer than {limit} bytes at byte {}",
                    self.offset
                )));
            }
        }
        Ok(String::from_utf8(out)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned()))
    }

    fn hex4(&mut self) -> Result<u32, NodeError> {
        let mut v = 0u32;
        for _ in 0..4 {
            let Some(b) = self.bump()? else {
                return Err(NodeError::Truncated(
                    "the file ends inside an escape".to_owned(),
                ));
            };
            let d = (b as char)
                .to_digit(16)
                .ok_or_else(|| self.bad("bad \\u escape"))?;
            v = v * 16 + d;
        }
        Ok(v)
    }

    fn unicode(&mut self) -> Result<char, NodeError> {
        let hi = self.hex4()?;
        if (0xD800..0xDC00).contains(&hi) {
            // A high surrogate needs its low half.
            if self.peek()? == Some(b'\\') {
                self.bump()?;
                if self.bump()? == Some(b'u') {
                    let lo = self.hex4()?;
                    if (0xDC00..0xE000).contains(&lo) {
                        let c = 0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
                        return Ok(char::from_u32(c).unwrap_or('\u{fffd}'));
                    }
                }
            }
            return Ok('\u{fffd}');
        }
        Ok(char::from_u32(hi).unwrap_or('\u{fffd}'))
    }

    /// A non-negative integer, or a number written as a float that is whole.
    pub(crate) fn uint(&mut self) -> Result<u64, NodeError> {
        self.skip_ws()?;
        let mut v: u64 = 0;
        let mut digits = 0;
        while let Some(b) = self.peek()? {
            if b.is_ascii_digit() {
                v = v
                    .checked_mul(10)
                    .and_then(|x| x.checked_add(u64::from(b - b'0')))
                    .ok_or_else(|| self.bad("integer too large"))?;
                digits += 1;
                self.pos += 1;
                self.offset += 1;
            } else {
                break;
            }
        }
        if digits == 0 {
            return Err(self.bad("expected an unsigned integer"));
        }
        // A fraction or exponent of a whole number (`12.0`, `1e3`) is not used by the
        // formats; refuse it instead of misreading it.
        if matches!(self.peek()?, Some(b'.' | b'e' | b'E')) {
            return Err(self.bad("expected an integer"));
        }
        Ok(v)
    }

    /// A signed integer.
    pub(crate) fn int(&mut self) -> Result<i64, NodeError> {
        self.skip_ws()?;
        let neg = self.eat(b'-')?;
        let v = i64::try_from(self.uint()?).map_err(|_| self.bad("integer too large"))?;
        Ok(if neg { -v } else { v })
    }

    /// An array of unsigned integers appended to `out`; at most `max` elements.
    pub(crate) fn uint_array(&mut self, out: &mut Vec<u32>, max: usize) -> Result<(), NodeError> {
        self.expect(b'[')?;
        if self.eat(b']')? {
            return Ok(());
        }
        loop {
            let v = self.uint()?;
            let v = u32::try_from(v).map_err(|_| self.bad("integer over 32 bits"))?;
            if out.len() >= max {
                return Err(NodeError::TooLarge(format!(
                    "an array holds more than {max} numbers"
                )));
            }
            out.push(v);
            if self.eat(b',')? {
                continue;
            }
            self.expect(b']')?;
            return Ok(());
        }
    }

    /// An array of strings appended to `out`; at most `max` strings of `limit` bytes.
    pub(crate) fn string_array(
        &mut self,
        out: &mut Vec<String>,
        max: usize,
        limit: usize,
    ) -> Result<(), NodeError> {
        self.expect(b'[')?;
        if self.eat(b']')? {
            return Ok(());
        }
        loop {
            if out.len() >= max {
                return Err(NodeError::TooLarge(format!(
                    "the string table holds more than {max} strings"
                )));
            }
            out.push(self.string(limit)?);
            if self.eat(b',')? {
                continue;
            }
            self.expect(b']')?;
            return Ok(());
        }
    }

    /// Skips one value of any kind.
    pub(crate) fn skip_value(&mut self) -> Result<(), NodeError> {
        self.raw_value(None).map(|_| ())
    }

    /// The raw bytes of one value, at most `limit` of them; `None` skips without keeping.
    pub(crate) fn raw_value(&mut self, keep: Option<usize>) -> Result<Vec<u8>, NodeError> {
        self.skip_ws()?;
        let mut out = Vec::new();
        let mut depth = 0usize;
        let mut in_str = false;
        let mut esc = false;
        let mut started = false;
        loop {
            let Some(b) = self.peek()? else {
                if started && depth == 0 && !in_str {
                    return Ok(out);
                }
                return Err(NodeError::Truncated(
                    "the file ends inside a value".to_owned(),
                ));
            };
            if !started {
                started = true;
            } else if depth == 0 && !in_str && matches!(b, b',' | b'}' | b']' | b' ' | b'\n') {
                return Ok(out);
            }
            self.pos += 1;
            self.offset += 1;
            if let Some(limit) = keep {
                if out.len() >= limit {
                    return Err(NodeError::TooLarge(format!(
                        "a value is longer than {limit} bytes"
                    )));
                }
                out.push(b);
            }
            if in_str {
                if esc {
                    esc = false;
                } else if b == b'\\' {
                    esc = true;
                } else if b == b'"' {
                    in_str = false;
                    if depth == 0 {
                        return Ok(out);
                    }
                }
                continue;
            }
            match b {
                b'"' => in_str = true,
                b'{' | b'[' => {
                    depth += 1;
                    if depth > MAX_DEPTH {
                        return Err(NodeError::Malformed(format!(
                            "nesting deeper than {MAX_DEPTH} at byte {}",
                            self.offset
                        )));
                    }
                }
                b'}' | b']' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Ok(out);
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rd(s: &str) -> Reader<&[u8]> {
        Reader::new(s.as_bytes())
    }

    #[test]
    fn strings_decode_escapes_and_surrogates() {
        let mut r = rd(r#""a\né😀\"x""#);
        assert_eq!(r.string(100).unwrap(), "a\n\u{e9}\u{1f600}\"x");
    }

    #[test]
    fn arrays_of_integers_are_bounded() {
        let mut v = Vec::new();
        rd("[1, 2,3\n,4]").uint_array(&mut v, 10).unwrap();
        assert_eq!(v, [1, 2, 3, 4]);
        let mut w = Vec::new();
        assert!(rd("[1,2,3]").uint_array(&mut w, 2).is_err());
        assert!(rd("[1,-2]").uint_array(&mut Vec::new(), 9).is_err());
        assert!(rd("[99999999999]").uint_array(&mut Vec::new(), 9).is_err());
        assert!(rd("[1,2").uint_array(&mut Vec::new(), 9).is_err());
    }

    #[test]
    fn values_are_skipped_or_kept_whole() {
        let mut r = rd(r#"{"a":[1,{"b":"x]}"}],"c":2} , 7"#);
        let raw = r.raw_value(Some(100)).unwrap();
        assert_eq!(raw, br#"{"a":[1,{"b":"x]}"}],"c":2}"#);
        assert!(r.eat(b',').unwrap());
        assert_eq!(r.uint().unwrap(), 7);
        assert!(rd(r#"{"a":"long"}"#).raw_value(Some(4)).is_err());
        let open = "[".repeat((4 << 20) + 2);
        let mut deep = rd(&open);
        assert!(deep.skip_value().is_err());
    }
}
