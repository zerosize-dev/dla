//! Streaming HPROF record scanner: a seekable buffered reader and a visitor-driven walk
//! over the top-level records and the heap-dump sub-records.

use std::io::{self, Read, Seek, SeekFrom};

use super::HprofError;

/// Buffered big-endian reader over a seekable source that tracks the file offset.
pub(crate) struct Rd<R: Read + Seek> {
    inner: R,
    buf: Vec<u8>,
    pos: usize,
    len: usize,
    base: u64,
    file_len: u64,
    /// Width of an object id in bytes (4 or 8).
    pub id_size: usize,
}

const BUF: usize = 1 << 20;

impl<R: Read + Seek> Rd<R> {
    pub fn new(mut inner: R, id_size: usize) -> io::Result<Self> {
        let file_len = inner.seek(SeekFrom::End(0))?;
        inner.seek(SeekFrom::Start(0))?;
        Ok(Self {
            inner,
            buf: vec![0; BUF],
            pos: 0,
            len: 0,
            base: 0,
            file_len,
            id_size,
        })
    }

    pub fn file_len(&self) -> u64 {
        self.file_len
    }

    pub fn tell(&self) -> u64 {
        self.base + self.pos as u64
    }

    fn eof() -> io::Error {
        io::Error::from(io::ErrorKind::UnexpectedEof)
    }

    /// Makes at least `n` bytes (`n` at most half the buffer) available.
    fn fill(&mut self, n: usize) -> io::Result<()> {
        if self.len - self.pos >= n {
            return Ok(());
        }
        self.buf.copy_within(self.pos..self.len, 0);
        self.base += self.pos as u64;
        self.len -= self.pos;
        self.pos = 0;
        while self.len < n {
            let k = self.inner.read(&mut self.buf[self.len..])?;
            if k == 0 {
                return Err(Self::eof());
            }
            self.len += k;
        }
        Ok(())
    }

    pub fn seek_to(&mut self, off: u64) -> io::Result<()> {
        let cur = self.tell();
        if off >= cur && off - cur <= (self.len - self.pos) as u64 {
            self.pos += (off - cur) as usize;
            return Ok(());
        }
        if off < cur && cur - off <= self.pos as u64 {
            self.pos -= (cur - off) as usize;
            return Ok(());
        }
        self.inner.seek(SeekFrom::Start(off))?;
        self.base = off;
        self.pos = 0;
        self.len = 0;
        Ok(())
    }

    pub fn skip(&mut self, n: u64) -> io::Result<()> {
        let to = self.tell().checked_add(n).ok_or_else(Self::eof)?;
        self.seek_to(to)
    }

    pub fn u1(&mut self) -> io::Result<u8> {
        self.fill(1)?;
        let v = self.buf[self.pos];
        self.pos += 1;
        Ok(v)
    }

    pub fn u2(&mut self) -> io::Result<u16> {
        self.fill(2)?;
        let v = u16::from_be_bytes([self.buf[self.pos], self.buf[self.pos + 1]]);
        self.pos += 2;
        Ok(v)
    }

    pub fn u4(&mut self) -> io::Result<u32> {
        self.fill(4)?;
        let p = self.pos;
        let v = u32::from_be_bytes([
            self.buf[p],
            self.buf[p + 1],
            self.buf[p + 2],
            self.buf[p + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    pub fn u8(&mut self) -> io::Result<u64> {
        self.fill(8)?;
        let mut a = [0u8; 8];
        a.copy_from_slice(&self.buf[self.pos..self.pos + 8]);
        self.pos += 8;
        Ok(u64::from_be_bytes(a))
    }

    pub fn id(&mut self) -> io::Result<u64> {
        if self.id_size == 8 {
            self.u8()
        } else {
            self.u4().map(u64::from)
        }
    }

    /// Reads `n` bytes into `out` (replacing its contents).
    pub fn read_vec(&mut self, n: usize, out: &mut Vec<u8>) -> io::Result<()> {
        out.clear();
        let mut left = n;
        while left > 0 {
            let chunk = left.min(BUF / 2);
            self.fill(chunk)?;
            out.extend_from_slice(&self.buf[self.pos..self.pos + chunk]);
            self.pos += chunk;
            left -= chunk;
        }
        Ok(())
    }
}

/// The type sizes of HPROF basic types (2 = object).
pub(crate) fn type_size(t: u8, id_size: usize) -> Option<usize> {
    Some(match t {
        2 => id_size,
        4 | 8 => 1,
        5 | 9 => 2,
        6 | 10 => 4,
        7 | 11 => 8,
        _ => return None,
    })
}

/// A parsed CLASS_DUMP.
#[derive(Debug, Default, Clone)]
pub(crate) struct ClassDump {
    pub id: u64,
    pub super_id: u64,
    pub loader_id: u64,
    /// Object-typed statics: `(name string id, value id)`.
    pub statics: Vec<(u64, u64)>,
    /// Instance fields in declaration order: `(name string id, type)`.
    pub fields: Vec<(u64, u8)>,
}

/// What the scanner reports; every method has an empty default.
pub(crate) trait Visitor {
    fn utf8(&mut self, _id: u64, _off: u64, _len: u32) {}
    fn load_class(&mut self, _serial: u32, _class_id: u64, _name_id: u64) {}
    fn frame(&mut self, _id: u64, _method: u64, _file: u64, _class_serial: u32, _line: i32) {}
    fn trace(&mut self, _serial: u32, _thread: u32, _frames: &[u64]) {}
    /// A GC root: `kind` is the sub-record tag; `a` and `b` are the record's two `u4`
    /// fields (thread serial and frame or stack-trace number) when it has them.
    fn root(&mut self, _kind: u8, _id: u64, _a: u32, _b: u32) {}
    fn class_dump(&mut self, _off: u64, _c: ClassDump) {}
    /// An instance; the visitor must consume `len` bytes of `rd` or leave them (the
    /// scanner seeks past).
    fn instance(
        &mut self,
        _off: u64,
        _id: u64,
        _class_id: u64,
        _len: u32,
        _rd: &mut Rd<impl Read + Seek>,
    ) -> io::Result<()> {
        Ok(())
    }
    fn obj_array(
        &mut self,
        _off: u64,
        _id: u64,
        _class_id: u64,
        _n: u32,
        _rd: &mut Rd<impl Read + Seek>,
    ) -> io::Result<()> {
        Ok(())
    }
    fn prim_array(&mut self, _off: u64, _id: u64, _elem: u8, _n: u32) {}
    /// True once the visitor wants the scan to end early.
    fn stopped(&self) -> bool {
        false
    }
}

/// Header of an HPROF file.
#[derive(Debug, Clone)]
pub(crate) struct RawHeader {
    pub version: String,
    pub id_size: u32,
    pub timestamp_ms: u64,
    /// Offset of the first record.
    pub body: u64,
}

/// Reads the header from the start of `r` (a plain reader positioned at 0).
pub(crate) fn read_header<R: Read>(r: &mut R) -> Result<RawHeader, HprofError> {
    let mut head = Vec::with_capacity(32);
    let mut b = [0u8; 1];
    loop {
        r.read_exact(&mut b).map_err(|e| {
            if e.kind() == io::ErrorKind::UnexpectedEof {
                HprofError::NotHprof("file ends before a NUL-terminated header".into())
            } else {
                HprofError::Io(e)
            }
        })?;
        if b[0] == 0 {
            break;
        }
        head.push(b[0]);
        if head.len() > 40 {
            return Err(HprofError::NotHprof(
                "no HPROF header (no NUL within 40 bytes)".into(),
            ));
        }
    }
    let version = String::from_utf8_lossy(&head).into_owned();
    if !version.starts_with("JAVA PROFILE 1.0") {
        return Err(HprofError::NotHprof(format!(
            "header {version:?} is not \"JAVA PROFILE 1.0.x\""
        )));
    }
    let mut w = [0u8; 12];
    r.read_exact(&mut w)
        .map_err(|_| HprofError::NotHprof("header is cut short".into()))?;
    let id_size = u32::from_be_bytes([w[0], w[1], w[2], w[3]]);
    if id_size != 4 && id_size != 8 {
        return Err(HprofError::NotHprof(format!(
            "identifier size {id_size} is not 4 or 8"
        )));
    }
    let mut ts = [0u8; 8];
    ts.copy_from_slice(&w[4..12]);
    Ok(RawHeader {
        body: head.len() as u64 + 1 + 12,
        version,
        id_size,
        timestamp_ms: u64::from_be_bytes(ts),
    })
}

fn malformed(rd_pos: u64, what: &str) -> HprofError {
    HprofError::Malformed(format!("{what} at offset {rd_pos}"))
}

fn io_err(e: io::Error, pos: u64) -> HprofError {
    if e.kind() == io::ErrorKind::UnexpectedEof {
        HprofError::Truncated(pos)
    } else {
        HprofError::Io(e)
    }
}

/// Walks every record from `header.body` to the end, reporting to `v`. Problems that
/// leave the rest readable (an unknown sub-record) become `warnings`.
pub(crate) fn scan<R: Read + Seek, V: Visitor>(
    rd: &mut Rd<R>,
    header: &RawHeader,
    v: &mut V,
    warnings: &mut Vec<String>,
) -> Result<(), HprofError> {
    rd.seek_to(header.body)
        .map_err(|e| io_err(e, header.body))?;
    let ids = rd.id_size as u64;
    let mut frames_scratch: Vec<u64> = Vec::new();
    // A segmented dump ends with HEAP_DUMP_END (0x2c). Segments with no end record are a
    // dump cut short, for example by a kill while the JVM was writing it: a record
    // boundary can fall at the cut, so the walk alone would not notice.
    let (mut segments, mut ended) = (false, false);
    loop {
        let start = rd.tell();
        if v.stopped() {
            return Ok(());
        }
        if start >= rd.file_len() {
            return if segments && !ended {
                Err(HprofError::Truncated(start))
            } else {
                Ok(())
            };
        }
        let tag = scan_record(rd, v, warnings, ids, &mut frames_scratch).map_err(|e| match e {
            ScanErr::Io(e) => io_err(e, rd.tell()),
            ScanErr::Bad(m) => malformed(start, &m),
        })?;
        match tag {
            0x1c => segments = true,
            0x2c => ended = true,
            _ => {}
        }
    }
}

enum ScanErr {
    Io(io::Error),
    Bad(String),
}

impl From<io::Error> for ScanErr {
    fn from(e: io::Error) -> Self {
        ScanErr::Io(e)
    }
}

fn scan_record<R: Read + Seek, V: Visitor>(
    rd: &mut Rd<R>,
    v: &mut V,
    warnings: &mut Vec<String>,
    ids: u64,
    frames: &mut Vec<u64>,
) -> Result<u8, ScanErr> {
    let start = rd.tell();
    let tag = rd.u1()?;
    let _time = rd.u4()?;
    let len = u64::from(rd.u4()?);
    let body = rd.tell();
    let end = body + len;
    if end > rd.file_len() {
        return Err(ScanErr::Io(io::Error::from(io::ErrorKind::UnexpectedEof)));
    }
    match tag {
        0x01 if len >= ids => {
            let id = rd.id()?;
            v.utf8(id, rd.tell(), (len - ids) as u32);
        }
        0x02 if len >= 8 + 2 * ids => {
            let serial = rd.u4()?;
            let class_id = rd.id()?;
            let _stack = rd.u4()?;
            let name = rd.id()?;
            v.load_class(serial, class_id, name);
        }
        0x04 if len >= 4 * ids + 8 => {
            let id = rd.id()?;
            let method = rd.id()?;
            let _sig = rd.id()?;
            let file = rd.id()?;
            let class_serial = rd.u4()?;
            let line = rd.u4()? as i32;
            v.frame(id, method, file, class_serial, line);
        }
        0x05 if len >= 12 => {
            let serial = rd.u4()?;
            let thread = rd.u4()?;
            let n = rd.u4()? as u64;
            if n * ids + 12 <= len {
                frames.clear();
                for _ in 0..n.min(64) {
                    frames.push(rd.id()?);
                }
                v.trace(serial, thread, frames);
            }
        }
        0x0c | 0x1c => heap_records(rd, v, warnings, end)?,
        _ => {}
    }
    if rd.tell() > end {
        return Err(ScanErr::Bad(format!(
            "record 0x{tag:02x} at offset {start} overruns its length"
        )));
    }
    rd.seek_to(end)?;
    Ok(tag)
}

fn heap_records<R: Read + Seek, V: Visitor>(
    rd: &mut Rd<R>,
    v: &mut V,
    warnings: &mut Vec<String>,
    end: u64,
) -> Result<(), ScanErr> {
    let ids = rd.id_size;
    while rd.tell() < end {
        if v.stopped() {
            return Ok(());
        }
        let off = rd.tell();
        let sub = rd.u1()?;
        match sub {
            0xff | 0x05 | 0x07 | 0x89 | 0x8a | 0x8b | 0x8c | 0x8d | 0x8f | 0x90 => {
                let id = rd.id()?;
                v.root(sub, id, 0, 0);
            }
            0x01 => {
                let id = rd.id()?;
                rd.id()?;
                v.root(sub, id, 0, 0);
            }
            0x02 | 0x03 | 0x08 | 0x8e => {
                let id = rd.id()?;
                let a = rd.u4()?;
                let b = rd.u4()?;
                v.root(sub, id, a, b);
            }
            0x04 | 0x06 => {
                let id = rd.id()?;
                let a = rd.u4()?;
                v.root(sub, id, a, 0);
            }
            0xfe => {
                rd.u4()?;
                rd.id()?;
            }
            0x20 => {
                let c = class_dump(rd)?;
                v.class_dump(off, c);
            }
            0x21 => {
                let id = rd.id()?;
                rd.u4()?;
                let class_id = rd.id()?;
                let n = rd.u4()?;
                let data = rd.tell();
                v.instance(off, id, class_id, n, rd)?;
                rd.seek_to(data + u64::from(n))?;
            }
            0x22 => {
                let id = rd.id()?;
                rd.u4()?;
                let n = rd.u4()?;
                let class_id = rd.id()?;
                let data = rd.tell();
                v.obj_array(off, id, class_id, n, rd)?;
                rd.seek_to(data + u64::from(n) * ids as u64)?;
            }
            0x23 => {
                let id = rd.id()?;
                rd.u4()?;
                let n = rd.u4()?;
                let t = rd.u1()?;
                let Some(sz) = type_size(t, ids).filter(|_| t != 2) else {
                    return Err(ScanErr::Bad(format!(
                        "primitive array with element type {t}"
                    )));
                };
                v.prim_array(off, id, t, n);
                rd.skip(u64::from(n) * sz as u64)?;
            }
            0xc3 => {
                let id = rd.id()?;
                rd.u4()?;
                let n = rd.u4()?;
                let t = rd.u1()?;
                v.prim_array(off, id, t, n);
            }
            other => {
                warnings.push(format!(
                    "unknown heap sub-record 0x{other:02x} at offset {off}: the rest of that record was skipped"
                ));
                return Ok(());
            }
        }
    }
    Ok(())
}

fn class_dump<R: Read + Seek>(rd: &mut Rd<R>) -> Result<ClassDump, ScanErr> {
    let ids = rd.id_size;
    let id = rd.id()?;
    rd.u4()?;
    let super_id = rd.id()?;
    let loader_id = rd.id()?;
    for _ in 0..4 {
        rd.id()?; // signers, protection domain, two reserved
    }
    rd.u4()?; // instance size: data only on HotSpot, so not used
    let ncp = rd.u2()?;
    for _ in 0..ncp {
        rd.u2()?;
        let t = rd.u1()?;
        let sz = type_size(t, ids).ok_or_else(|| ScanErr::Bad(format!("constant of type {t}")))?;
        rd.skip(sz as u64)?;
    }
    let ns = rd.u2()?;
    let mut statics = Vec::new();
    for _ in 0..ns {
        let name = rd.id()?;
        let t = rd.u1()?;
        let sz = type_size(t, ids).ok_or_else(|| ScanErr::Bad(format!("static of type {t}")))?;
        if t == 2 {
            let value = rd.id()?;
            if value != 0 {
                statics.push((name, value));
            }
        } else {
            rd.skip(sz as u64)?;
        }
    }
    let nf = rd.u2()?;
    let mut fields = Vec::with_capacity(nf as usize);
    for _ in 0..nf {
        let name = rd.id()?;
        let t = rd.u1()?;
        if type_size(t, ids).is_none() {
            return Err(ScanErr::Bad(format!("instance field of type {t}")));
        }
        fields.push((name, t));
    }
    Ok(ClassDump {
        id,
        super_id,
        loader_id,
        statics,
        fields,
    })
}
