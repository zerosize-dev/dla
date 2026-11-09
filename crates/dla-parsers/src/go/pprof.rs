//! A hand-rolled reader for the pprof profile protobuf (`profile.proto`) that Go's
//! `runtime/pprof` writes, gzipped. It decodes varints and length-delimited fields and
//! nothing else: the messages Profile, ValueType, Sample, Label, Mapping (skipped),
//! Location, Line and Function, and the string table.
//!
//! The string table is the last field of a profile, so every name is kept as an index
//! while reading and resolved at the end. Counts and lengths read from the file are
//! checked against the input, and the inflated size is capped.

use std::{
    collections::HashMap,
    io::{self, Write},
};

use super::ParseError;
use crate::inflate::{self, GzipError};

/// Most bytes a profile may inflate to.
pub const MAX_INFLATED: usize = 256 << 20;
/// Most samples read.
const MAX_SAMPLES: usize = 4_000_000;
/// Most locations and functions read.
const MAX_TABLE: usize = 4_000_000;
/// Most strings in the string table.
const MAX_STRINGS: usize = 4_000_000;

/// A sample type or period type: `inuse_space` and `bytes`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValueType {
    /// What is counted (`inuse_space`, `goroutine`).
    pub kind: String,
    /// The unit (`bytes`, `count`).
    pub unit: String,
}

/// One label of a sample (`pprof.Do` labels, or the `bytes` size class of a heap sample).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Label {
    /// The key.
    pub key: String,
    /// The string value, when the label has one.
    pub str: Option<String>,
    /// The number, for a numeric label.
    pub num: i64,
}

/// One sample: a stack (innermost location first) and its values, one per sample type.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Sample {
    /// Location ids, innermost first.
    pub locations: Vec<u64>,
    /// Values, in the order of [`Profile::sample_types`].
    pub values: Vec<i64>,
    /// Labels.
    pub labels: Vec<Label>,
}

/// One line of a location: a function and the line in it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Line {
    /// Function id.
    pub function: u64,
    /// Line number, 0 when unknown.
    pub line: i64,
}

/// A code location. More than one line means inlined functions: the first line is the
/// innermost, the last the function the others were inlined into.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Location {
    /// The lines.
    pub lines: Vec<Line>,
}

/// A function.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Function {
    /// Its name, as Go prints it (`example.com/p.(*T).m`).
    pub name: String,
    /// The source file as the compiler recorded it.
    pub file: String,
}

/// A decoded profile.
#[derive(Debug, Clone, Default)]
pub struct Profile {
    /// Sample types, in value order.
    pub sample_types: Vec<ValueType>,
    /// Samples.
    pub samples: Vec<Sample>,
    /// Locations by id.
    pub locations: HashMap<u64, Location>,
    /// Functions by id.
    pub functions: HashMap<u64, Function>,
    /// Collection time, nanoseconds since the Unix epoch (0 when absent).
    pub time_nanos: i64,
    /// Collection duration, nanoseconds.
    pub duration_nanos: i64,
    /// The period type (`space`/`bytes` for a heap profile).
    pub period_type: ValueType,
    /// The sampling period (the mean bytes between heap samples).
    pub period: i64,
}

impl Profile {
    /// The index of the sample type `kind`, when the profile has it.
    pub fn value_index(&self, kind: &str) -> Option<usize> {
        self.sample_types.iter().position(|t| t.kind == kind)
    }

    /// The frames of a sample, innermost first, as `(function, file, line)`.
    pub fn frames(&self, s: &Sample) -> Vec<(String, String, i64)> {
        let mut out = Vec::new();
        for id in &s.locations {
            let Some(loc) = self.locations.get(id) else {
                continue;
            };
            for l in &loc.lines {
                match self.functions.get(&l.function) {
                    Some(f) => out.push((f.name.clone(), f.file.clone(), l.line)),
                    None => out.push((String::new(), String::new(), l.line)),
                }
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------------------
// Protobuf reading
// ---------------------------------------------------------------------------------------

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

/// A decoded field value.
enum Value<'a> {
    Varint(u64),
    Bytes(&'a [u8]),
    /// Fixed 32 or 64 bit values and groups are not used by profile.proto.
    Skipped,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }

    fn done(&self) -> bool {
        self.pos >= self.buf.len()
    }

    fn varint(&mut self) -> Result<u64, ParseError> {
        let mut v = 0u64;
        for shift in (0..70).step_by(7) {
            let Some(&b) = self.buf.get(self.pos) else {
                return Err(ParseError::new("the profile ends inside a number"));
            };
            self.pos += 1;
            if shift < 64 {
                v |= u64::from(b & 0x7f) << shift;
            }
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err(ParseError::new(
            "a number in the profile is longer than 10 bytes",
        ))
    }

    /// The next `(field number, value)`.
    fn field(&mut self) -> Result<(u32, Value<'a>), ParseError> {
        let key = self.varint()?;
        let num = u32::try_from(key >> 3)
            .map_err(|_| ParseError::new("a field number in the profile is out of range"))?;
        match key & 7 {
            0 => Ok((num, Value::Varint(self.varint()?))),
            1 => self.skip(8, num),
            5 => self.skip(4, num),
            2 => {
                let len = usize::try_from(self.varint()?)
                    .map_err(|_| ParseError::new("a length in the profile is out of range"))?;
                let end = self
                    .pos
                    .checked_add(len)
                    .filter(|e| *e <= self.buf.len())
                    .ok_or_else(|| {
                        ParseError::new("a field in the profile is longer than the file")
                    })?;
                let bytes = &self.buf[self.pos..end];
                self.pos = end;
                Ok((num, Value::Bytes(bytes)))
            }
            w => Err(ParseError::new(format!(
                "wire type {w} in the profile: not a pprof profile"
            ))),
        }
    }

    fn skip(&mut self, n: usize, num: u32) -> Result<(u32, Value<'a>), ParseError> {
        let end = self
            .pos
            .checked_add(n)
            .filter(|e| *e <= self.buf.len())
            .ok_or_else(|| ParseError::new("the profile ends inside a fixed-size field"))?;
        self.pos = end;
        Ok((num, Value::Skipped))
    }
}

/// Repeated integers: a packed run (`Bytes`) or one varint.
fn push_ints(v: Value<'_>, out: &mut Vec<u64>, cap: usize) -> Result<(), ParseError> {
    match v {
        Value::Varint(x) => out.push(x),
        Value::Bytes(b) => {
            let mut r = Reader::new(b);
            while !r.done() {
                if out.len() >= cap {
                    return Err(ParseError::new(
                        "a sample in the profile has too many values",
                    ));
                }
                out.push(r.varint()?);
            }
        }
        Value::Skipped => {}
    }
    Ok(())
}

fn as_i64(v: &Value<'_>) -> i64 {
    match v {
        Value::Varint(x) => *x as i64,
        _ => 0,
    }
}

fn as_u64(v: &Value<'_>) -> u64 {
    match v {
        Value::Varint(x) => *x,
        _ => 0,
    }
}

/// A message with string indexes still to resolve.
struct RawType {
    kind: i64,
    unit: i64,
}

struct RawLabel {
    key: i64,
    str: i64,
    num: i64,
}

struct RawSample {
    locations: Vec<u64>,
    values: Vec<u64>,
    labels: Vec<RawLabel>,
}

fn read_type(b: &[u8]) -> Result<RawType, ParseError> {
    let mut t = RawType { kind: 0, unit: 0 };
    let mut r = Reader::new(b);
    while !r.done() {
        let (n, v) = r.field()?;
        match n {
            1 => t.kind = as_i64(&v),
            2 => t.unit = as_i64(&v),
            _ => {}
        }
    }
    Ok(t)
}

fn read_sample(b: &[u8]) -> Result<RawSample, ParseError> {
    let mut s = RawSample {
        locations: Vec::new(),
        values: Vec::new(),
        labels: Vec::new(),
    };
    let mut r = Reader::new(b);
    while !r.done() {
        let (n, v) = r.field()?;
        match n {
            1 => push_ints(v, &mut s.locations, 4096)?,
            2 => push_ints(v, &mut s.values, 64)?,
            3 => {
                if let Value::Bytes(lb) = v {
                    let mut l = RawLabel {
                        key: 0,
                        str: 0,
                        num: 0,
                    };
                    let mut lr = Reader::new(lb);
                    while !lr.done() {
                        let (ln, lv) = lr.field()?;
                        match ln {
                            1 => l.key = as_i64(&lv),
                            2 => l.str = as_i64(&lv),
                            3 => l.num = as_i64(&lv),
                            _ => {}
                        }
                    }
                    if s.labels.len() < 64 {
                        s.labels.push(l);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(s)
}

fn read_location(b: &[u8]) -> Result<(u64, Location), ParseError> {
    let mut id = 0;
    let mut loc = Location::default();
    let mut r = Reader::new(b);
    while !r.done() {
        let (n, v) = r.field()?;
        match (n, v) {
            (1, v) => id = as_u64(&v),
            (4, Value::Bytes(lb)) => {
                let mut line = Line::default();
                let mut lr = Reader::new(lb);
                while !lr.done() {
                    let (ln, lv) = lr.field()?;
                    match ln {
                        1 => line.function = as_u64(&lv),
                        2 => line.line = as_i64(&lv),
                        _ => {}
                    }
                }
                if loc.lines.len() < 256 {
                    loc.lines.push(line);
                }
            }
            _ => {}
        }
    }
    Ok((id, loc))
}

fn read_function(b: &[u8]) -> Result<(u64, i64, i64), ParseError> {
    let (mut id, mut name, mut file) = (0, 0, 0);
    let mut r = Reader::new(b);
    while !r.done() {
        let (n, v) = r.field()?;
        match n {
            1 => id = as_u64(&v),
            2 => name = as_i64(&v),
            4 => file = as_i64(&v),
            _ => {}
        }
    }
    Ok((id, name, file))
}

/// Decodes an uncompressed profile.
pub fn decode(buf: &[u8]) -> Result<Profile, ParseError> {
    let mut strings: Vec<String> = Vec::new();
    let mut types: Vec<RawType> = Vec::new();
    let mut period_type = RawType { kind: 0, unit: 0 };
    let mut samples: Vec<RawSample> = Vec::new();
    let mut funcs: Vec<(u64, i64, i64)> = Vec::new();
    let mut p = Profile::default();
    let mut r = Reader::new(buf);
    while !r.done() {
        let (n, v) = r.field()?;
        match (n, v) {
            (1, Value::Bytes(b)) => types.push(read_type(b)?),
            (2, Value::Bytes(b)) => {
                if samples.len() >= MAX_SAMPLES {
                    return Err(ParseError::new(format!(
                        "the profile has more than {MAX_SAMPLES} samples"
                    )));
                }
                samples.push(read_sample(b)?);
            }
            (4, Value::Bytes(b)) => {
                if p.locations.len() >= MAX_TABLE {
                    return Err(ParseError::new("the profile has too many locations"));
                }
                let (id, loc) = read_location(b)?;
                p.locations.insert(id, loc);
            }
            (5, Value::Bytes(b)) => {
                if funcs.len() >= MAX_TABLE {
                    return Err(ParseError::new("the profile has too many functions"));
                }
                funcs.push(read_function(b)?);
            }
            (6, Value::Bytes(b)) => {
                if strings.len() >= MAX_STRINGS {
                    return Err(ParseError::new("the profile has too many strings"));
                }
                strings.push(String::from_utf8_lossy(b).into_owned());
            }
            (9, v) => p.time_nanos = as_i64(&v),
            (10, v) => p.duration_nanos = as_i64(&v),
            (11, Value::Bytes(b)) => period_type = read_type(b)?,
            (12, v) => p.period = as_i64(&v),
            _ => {}
        }
    }
    if strings.first().is_none_or(|s| !s.is_empty()) {
        return Err(ParseError::new(
            "the profile has no string table starting with the empty string: not a pprof profile",
        ));
    }
    let s = |i: i64| -> String {
        usize::try_from(i)
            .ok()
            .and_then(|i| strings.get(i))
            .cloned()
            .unwrap_or_default()
    };
    p.sample_types = types
        .iter()
        .map(|t| ValueType {
            kind: s(t.kind),
            unit: s(t.unit),
        })
        .collect();
    p.period_type = ValueType {
        kind: s(period_type.kind),
        unit: s(period_type.unit),
    };
    for (id, name, file) in funcs {
        p.functions.insert(
            id,
            Function {
                name: s(name),
                file: s(file),
            },
        );
    }
    let nvals = p.sample_types.len();
    for rs in samples {
        if rs.values.len() != nvals {
            return Err(ParseError::new(format!(
                "a sample has {} values but the profile has {nvals} sample types",
                rs.values.len()
            )));
        }
        p.samples.push(Sample {
            locations: rs.locations,
            values: rs.values.iter().map(|v| *v as i64).collect(),
            labels: rs
                .labels
                .iter()
                .map(|l| Label {
                    key: s(l.key),
                    str: (l.str != 0).then(|| s(l.str)),
                    num: l.num,
                })
                .collect(),
        });
    }
    if p.sample_types.is_empty() {
        return Err(ParseError::new("the profile has no sample type"));
    }
    Ok(p)
}

/// A writer that fails once more than `cap` bytes were written.
struct Capped {
    out: Vec<u8>,
    cap: usize,
}

impl Write for Capped {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        if self.out.len().saturating_add(data.len()) > self.cap {
            return Err(io::Error::other("inflated profile over the size cap"));
        }
        self.out.extend_from_slice(data);
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A writer that keeps the first bytes and then stops the inflater.
struct Prefix {
    out: Vec<u8>,
    want: usize,
}

impl Write for Prefix {
    fn write(&mut self, data: &[u8]) -> io::Result<usize> {
        let room = self.want.saturating_sub(self.out.len());
        self.out.extend_from_slice(&data[..data.len().min(room)]);
        if self.out.len() >= self.want {
            return Err(io::Error::other("enough"));
        }
        Ok(data.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// True when the start of `bytes` (gzip or bare protobuf) can be the start of a profile: a
/// tag of one of the Profile fields (Go writes `period_type` first, other writers
/// `sample_type`), then a plausible length or number. A cheap check that reads a few
/// bytes; the decode that follows decides.
pub fn looks_like_profile(bytes: &[u8]) -> bool {
    let head: Vec<u8> = if inflate::is_gzip(bytes) {
        let mut sink = Prefix {
            out: Vec::new(),
            want: 16,
        };
        let _ = inflate::gunzip(bytes, &mut sink);
        sink.out
    } else {
        bytes.iter().copied().take(16).collect()
    };
    let Some(&tag) = head.first() else {
        return false;
    };
    // Fields 1-6, 9-14 with their wire types: 0x0a 0x12 0x1a 0x22 0x2a 0x32 (LEN), 0x48
    // 0x50 0x60 0x70 (varint), 0x5a 0x6a (LEN).
    if !matches!(
        tag,
        0x0a | 0x12 | 0x1a | 0x22 | 0x2a | 0x32 | 0x48 | 0x50 | 0x5a | 0x60 | 0x6a | 0x70
    ) {
        return false;
    }
    let mut r = Reader::new(&head);
    r.pos = 1;
    match r.varint() {
        Ok(n) => n > 0 || tag & 7 != 2,
        Err(_) => false,
    }
}

/// Reads a profile: gzip (what `pprof.Lookup(...).WriteTo(w, 0)` writes) or the bare
/// protobuf.
pub fn read(bytes: &[u8]) -> Result<Profile, ParseError> {
    if inflate::is_gzip(bytes) {
        let mut sink = Capped {
            out: Vec::new(),
            cap: MAX_INFLATED,
        };
        match inflate::gunzip(bytes, &mut sink) {
            Ok(_) => decode(&sink.out),
            Err(GzipError::Io(e)) if e.to_string().contains("size cap") => {
                Err(ParseError::new(format!(
                    "the profile inflates to more than {} MB",
                    MAX_INFLATED >> 20
                )))
            }
            Err(e) => Err(ParseError::new(format!(
                "the profile is not a readable gzip file ({e})"
            ))),
        }
    } else {
        decode(bytes)
    }
}

#[cfg(test)]
pub(crate) mod build {
    //! A tiny profile writer for tests.

    pub fn varint(mut v: u64, out: &mut Vec<u8>) {
        loop {
            let b = (v & 0x7f) as u8;
            v >>= 7;
            if v == 0 {
                out.push(b);
                return;
            }
            out.push(b | 0x80);
        }
    }

    pub fn field_varint(num: u32, v: u64, out: &mut Vec<u8>) {
        varint(u64::from(num) << 3, out);
        varint(v, out);
    }

    pub fn field_bytes(num: u32, b: &[u8], out: &mut Vec<u8>) {
        varint(u64::from(num) << 3 | 2, out);
        varint(b.len() as u64, out);
        out.extend_from_slice(b);
    }

    pub struct P {
        pub strings: Vec<String>,
        pub body: Vec<u8>,
    }

    impl P {
        pub fn new() -> Self {
            Self {
                strings: vec![String::new()],
                body: Vec::new(),
            }
        }

        pub fn s(&mut self, t: &str) -> u64 {
            if let Some(i) = self.strings.iter().position(|x| x == t) {
                return i as u64;
            }
            self.strings.push(t.to_owned());
            (self.strings.len() - 1) as u64
        }

        pub fn sample_type(&mut self, kind: &str, unit: &str) {
            let (k, u) = (self.s(kind), self.s(unit));
            let mut m = Vec::new();
            field_varint(1, k, &mut m);
            field_varint(2, u, &mut m);
            field_bytes(1, &m, &mut self.body);
        }

        /// A sample with packed locations and values.
        pub fn sample(&mut self, locs: &[u64], vals: &[i64]) {
            let mut m = Vec::new();
            let mut l = Vec::new();
            locs.iter().for_each(|x| varint(*x, &mut l));
            field_bytes(1, &l, &mut m);
            let mut v = Vec::new();
            vals.iter().for_each(|x| varint(*x as u64, &mut v));
            field_bytes(2, &v, &mut m);
            field_bytes(2, &m, &mut self.body);
        }

        pub fn function(&mut self, id: u64, name: &str, file: &str) {
            let (n, f) = (self.s(name), self.s(file));
            let mut m = Vec::new();
            field_varint(1, id, &mut m);
            field_varint(2, n, &mut m);
            field_varint(4, f, &mut m);
            field_bytes(5, &m, &mut self.body);
        }

        pub fn location(&mut self, id: u64, lines: &[(u64, i64)]) {
            let mut m = Vec::new();
            field_varint(1, id, &mut m);
            for (f, l) in lines {
                let mut lm = Vec::new();
                field_varint(1, *f, &mut lm);
                field_varint(2, *l as u64, &mut lm);
                field_bytes(4, &lm, &mut m);
            }
            field_bytes(4, &m, &mut self.body);
        }

        pub fn time(&mut self, nanos: i64) {
            field_varint(9, nanos as u64, &mut self.body);
        }

        pub fn finish(mut self) -> Vec<u8> {
            let mut out = std::mem::take(&mut self.body);
            for s in &self.strings {
                field_bytes(6, s.as_bytes(), &mut out);
            }
            out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{build::*, *};

    fn sample_profile() -> Vec<u8> {
        let mut p = P::new();
        p.sample_type("inuse_space", "bytes");
        p.function(1, "example.com/p.f", "/w/p/f.go");
        p.function(2, "runtime.main", "/goroot/src/runtime/proc.go");
        p.location(10, &[(1, 12)]);
        p.location(11, &[(2, 300)]);
        p.sample(&[10, 11], &[4096]);
        p.time(1_700_000_000_000_000_000);
        p.finish()
    }

    #[test]
    fn decodes_a_small_profile() {
        let prof = decode(&sample_profile()).unwrap();
        assert_eq!(prof.sample_types[0].kind, "inuse_space");
        assert_eq!(prof.samples.len(), 1);
        assert_eq!(prof.samples[0].values, [4096]);
        let frames = prof.frames(&prof.samples[0]);
        assert_eq!(
            frames[0],
            ("example.com/p.f".into(), "/w/p/f.go".into(), 12)
        );
        assert_eq!(frames[1].0, "runtime.main");
        assert_eq!(prof.time_nanos, 1_700_000_000_000_000_000);
        assert_eq!(prof.value_index("inuse_space"), Some(0));
    }

    #[test]
    fn unpacked_and_negative_values_decode() {
        let mut p = P::new();
        p.sample_type("delta", "count");
        // One unpacked location and a negative value (ten-byte varint).
        let mut s = Vec::new();
        field_varint(1, 7, &mut s);
        field_varint(2, (-5i64) as u64, &mut s);
        field_bytes(2, &s, &mut p.body);
        let prof = decode(&p.finish()).unwrap();
        assert_eq!(prof.samples[0].locations, [7]);
        assert_eq!(prof.samples[0].values, [-5]);
    }

    #[test]
    fn damaged_input_is_refused_without_panicking() {
        let good = sample_profile();
        for cut in [1, 5, good.len() / 2, good.len() - 3] {
            let _ = decode(&good[..cut]);
        }
        assert!(decode(b"").is_err());
        assert!(decode(b"hello world, not a profile").is_err());
        // A length that runs past the end.
        assert!(decode(&[0x0a, 0x7f, 0x01]).is_err());
        // A varint that never ends.
        assert!(
            decode(&[
                0x08, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff
            ])
            .is_err()
        );
    }

    #[test]
    fn profiles_are_told_from_other_files_by_their_first_bytes() {
        assert!(looks_like_profile(&sample_profile()));
        assert!(!looks_like_profile(b"JAVA PROFILE 1.0.2\0"));
        assert!(!looks_like_profile(b"goroutine 1 [running]:\n"));
        assert!(!looks_like_profile(b"\x1f\x8b\x08\x00"));
        assert!(!looks_like_profile(b"hprof"));
        assert!(!looks_like_profile(b""));
    }

    #[test]
    fn a_sample_with_the_wrong_value_count_is_refused() {
        let mut p = P::new();
        p.sample_type("a", "count");
        p.sample(&[1], &[1, 2]);
        let e = decode(&p.finish()).unwrap_err();
        assert!(e.message().contains("2 values"), "{e}");
    }
}
