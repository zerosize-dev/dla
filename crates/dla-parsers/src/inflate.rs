//! A small pure-Rust gzip reader: inflate (stored, fixed and dynamic Huffman blocks),
//! CRC-32 and length checks, and concatenated members. It streams: the input is read
//! through a fixed buffer and the output goes to a [`Write`] after a 32 KiB window.

use std::{
    fmt,
    io::{self, Read, Write},
};

/// Why a gzip stream could not be inflated.
#[derive(Debug)]
pub enum GzipError {
    /// Reading the input or writing the output failed.
    Io(io::Error),
    /// The stream ends before the final block or the trailer is complete.
    Truncated,
    /// The member header is not a gzip header this reader supports.
    BadHeader(&'static str),
    /// A deflate block is malformed.
    BadData(&'static str),
    /// The CRC-32 of the output differs from the trailer's.
    BadChecksum {
        /// The value in the trailer.
        expected: u32,
        /// The value computed over the output.
        actual: u32,
    },
    /// The output length (mod 2^32) differs from the trailer's.
    BadLength {
        /// The value in the trailer.
        expected: u32,
        /// The value computed over the output.
        actual: u32,
    },
    /// Non-zero bytes follow the last member.
    TrailingGarbage,
}

impl fmt::Display for GzipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "i/o error: {e}"),
            Self::Truncated => f.write_str("gzip stream is truncated"),
            Self::BadHeader(m) => write!(f, "bad gzip header: {m}"),
            Self::BadData(m) => write!(f, "corrupt deflate data: {m}"),
            Self::BadChecksum { expected, actual } => {
                write!(
                    f,
                    "gzip CRC-32 mismatch (trailer {expected:08x}, data {actual:08x})"
                )
            }
            Self::BadLength { expected, actual } => {
                write!(
                    f,
                    "gzip length mismatch (trailer {expected}, data {actual})"
                )
            }
            Self::TrailingGarbage => f.write_str("unexpected bytes after the last gzip member"),
        }
    }
}

impl std::error::Error for GzipError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for GzipError {
    fn from(e: io::Error) -> Self {
        if e.kind() == io::ErrorKind::UnexpectedEof {
            Self::Truncated
        } else {
            Self::Io(e)
        }
    }
}

/// True when `prefix` starts with the gzip magic bytes.
pub fn is_gzip(prefix: &[u8]) -> bool {
    prefix.len() >= 2 && prefix[0] == 0x1f && prefix[1] == 0x8b
}

const fn crc_table() -> [u32; 256] {
    let mut t = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
            k += 1;
        }
        t[i] = c;
        i += 1;
    }
    t
}

static CRC_TABLE: [u32; 256] = crc_table();

/// Running CRC-32 (IEEE) state.
#[derive(Debug, Clone, Copy)]
pub struct Crc32(u32);

impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}

impl Crc32 {
    /// A fresh state.
    pub fn new() -> Self {
        Self(0xFFFF_FFFF)
    }
    /// Folds `data` in.
    pub fn update(&mut self, data: &[u8]) {
        let mut c = self.0;
        for &b in data {
            c = CRC_TABLE[((c ^ u32::from(b)) & 0xFF) as usize] ^ (c >> 8);
        }
        self.0 = c;
    }
    /// The CRC of everything folded in.
    pub fn finish(&self) -> u32 {
        !self.0
    }
}

/// CRC-32 of `data`.
pub fn crc32(data: &[u8]) -> u32 {
    let mut c = Crc32::new();
    c.update(data);
    c.finish()
}

const WINDOW: usize = 32768;
const OUT_BUF: usize = 1 << 16;

const LEN_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LEN_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];
const CL_ORDER: [usize; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];

/// A canonical Huffman decoding table indexed by the next `bits` input bits.
struct Huff {
    table: Vec<u16>,
    bits: u32,
}

impl Huff {
    /// Builds the table for code `lengths`; entries are `(symbol << 4) | length`.
    fn new(lengths: &[u8], allow_incomplete: bool) -> Result<Self, GzipError> {
        let max = lengths.iter().copied().max().unwrap_or(0);
        if max == 0 {
            // No codes at all: legal only for an unused distance tree.
            return if allow_incomplete {
                Ok(Self {
                    table: vec![0; 2],
                    bits: 1,
                })
            } else {
                Err(GzipError::BadData("empty Huffman code"))
            };
        }
        let mut count = [0u32; 16];
        for &l in lengths {
            count[l as usize] += 1;
        }
        count[0] = 0;
        let mut left: i32 = 1;
        for &c in &count[1..] {
            left <<= 1;
            left -= c as i32;
            if left < 0 {
                return Err(GzipError::BadData("over-subscribed Huffman code"));
            }
        }
        if left > 0 && !(allow_incomplete && max == 1 && count[1] == 1) {
            return Err(GzipError::BadData("incomplete Huffman code"));
        }
        let mut next = [0u32; 16];
        let mut code = 0u32;
        for l in 1..16 {
            code = (code + count[l - 1]) << 1;
            next[l] = code;
        }
        let bits = u32::from(max);
        let mut table = vec![0u16; 1 << bits];
        for (sym, &l) in lengths.iter().enumerate() {
            if l == 0 {
                continue;
            }
            let c = next[l as usize];
            next[l as usize] += 1;
            let rev = c.reverse_bits() >> (32 - u32::from(l));
            let entry = ((sym as u16) << 4) | u16::from(l);
            let mut k = rev as usize;
            while k < table.len() {
                table[k] = entry;
                k += 1 << l;
            }
        }
        Ok(Self { table, bits })
    }
}

struct Inflater<R: Read, W: Write> {
    input: R,
    inbuf: Vec<u8>,
    inpos: usize,
    inlen: usize,
    bitbuf: u64,
    bitcnt: u32,
    out: W,
    outbuf: Vec<u8>,
    win: Vec<u8>,
    wpos: usize,
    member_out: u64,
    crc: Crc32,
    total: u64,
}

impl<R: Read, W: Write> Inflater<R, W> {
    fn new(input: R, out: W) -> Self {
        Self {
            input,
            inbuf: vec![0; 1 << 16],
            inpos: 0,
            inlen: 0,
            bitbuf: 0,
            bitcnt: 0,
            out,
            outbuf: Vec::with_capacity(OUT_BUF + 300),
            win: vec![0; WINDOW],
            wpos: 0,
            member_out: 0,
            crc: Crc32::new(),
            total: 0,
        }
    }

    /// The next input byte, or `None` at the end of the input.
    fn next_byte(&mut self) -> Result<Option<u8>, GzipError> {
        if self.inpos == self.inlen {
            loop {
                match self.input.read(&mut self.inbuf) {
                    Ok(0) => return Ok(None),
                    Ok(n) => {
                        self.inlen = n;
                        self.inpos = 0;
                        break;
                    }
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => {}
                    Err(e) => return Err(GzipError::Io(e)),
                }
            }
        }
        let b = self.inbuf[self.inpos];
        self.inpos += 1;
        Ok(Some(b))
    }

    fn byte(&mut self) -> Result<u8, GzipError> {
        self.next_byte()?.ok_or(GzipError::Truncated)
    }

    /// Ensures at least `n` (at most 32) bits are buffered.
    fn need(&mut self, n: u32) -> Result<(), GzipError> {
        while self.bitcnt < n {
            let b = self.byte()?;
            self.bitbuf |= u64::from(b) << self.bitcnt;
            self.bitcnt += 8;
        }
        Ok(())
    }

    fn bits(&mut self, n: u32) -> Result<u32, GzipError> {
        if n == 0 {
            return Ok(0);
        }
        self.need(n)?;
        let v = (self.bitbuf & ((1u64 << n) - 1)) as u32;
        self.bitbuf >>= n;
        self.bitcnt -= n;
        Ok(v)
    }

    fn decode(&mut self, h: &Huff) -> Result<u16, GzipError> {
        // Fill as far as the input allows; at the very end fewer bits may remain.
        while self.bitcnt < h.bits {
            match self.next_byte()? {
                Some(b) => {
                    self.bitbuf |= u64::from(b) << self.bitcnt;
                    self.bitcnt += 8;
                }
                None => break,
            }
        }
        let mask = (1u64 << h.bits) - 1;
        let e = h.table[(self.bitbuf & mask) as usize];
        let len = u32::from(e & 15);
        if len == 0 {
            return if self.bitcnt < h.bits {
                Err(GzipError::Truncated)
            } else {
                Err(GzipError::BadData("invalid Huffman code"))
            };
        }
        if len > self.bitcnt {
            return Err(GzipError::Truncated);
        }
        self.bitbuf >>= len;
        self.bitcnt -= len;
        Ok(e >> 4)
    }

    fn flush(&mut self) -> Result<(), GzipError> {
        self.crc.update(&self.outbuf);
        self.out.write_all(&self.outbuf)?;
        self.outbuf.clear();
        Ok(())
    }

    #[inline]
    fn put(&mut self, b: u8) {
        self.win[self.wpos] = b;
        self.wpos = (self.wpos + 1) & (WINDOW - 1);
        self.outbuf.push(b);
    }

    fn put_checked(&mut self, b: u8) -> Result<(), GzipError> {
        self.put(b);
        self.member_out += 1;
        if self.outbuf.len() >= OUT_BUF {
            self.flush()?;
        }
        Ok(())
    }

    fn copy_match(&mut self, len: usize, dist: usize) -> Result<(), GzipError> {
        if dist as u64 > self.member_out || dist > WINDOW {
            return Err(GzipError::BadData("distance too far back"));
        }
        let mut src = (self.wpos + WINDOW - dist) & (WINDOW - 1);
        for _ in 0..len {
            let b = self.win[src];
            src = (src + 1) & (WINDOW - 1);
            self.put(b);
        }
        self.member_out += len as u64;
        if self.outbuf.len() >= OUT_BUF {
            self.flush()?;
        }
        Ok(())
    }

    fn stored(&mut self) -> Result<(), GzipError> {
        let drop = self.bitcnt % 8;
        self.bitbuf >>= drop;
        self.bitcnt -= drop;
        let len = self.bits(16)?;
        let nlen = self.bits(16)?;
        if len != (!nlen & 0xFFFF) {
            return Err(GzipError::BadData("stored block length check"));
        }
        for _ in 0..len {
            let b = self.bits(8)? as u8;
            self.put_checked(b)?;
        }
        Ok(())
    }

    fn codes(&mut self, lit: &Huff, dist: &Huff) -> Result<(), GzipError> {
        loop {
            let sym = self.decode(lit)?;
            if sym < 256 {
                self.put_checked(sym as u8)?;
            } else if sym == 256 {
                return Ok(());
            } else {
                let i = usize::from(sym) - 257;
                if i >= 29 {
                    return Err(GzipError::BadData("invalid length symbol"));
                }
                let len = usize::from(LEN_BASE[i]) + self.bits(u32::from(LEN_EXTRA[i]))? as usize;
                let ds = usize::from(self.decode(dist)?);
                if ds >= 30 {
                    return Err(GzipError::BadData("invalid distance symbol"));
                }
                let d = usize::from(DIST_BASE[ds]) + self.bits(u32::from(DIST_EXTRA[ds]))? as usize;
                self.copy_match(len, d)?;
            }
        }
    }

    fn dynamic(&mut self) -> Result<(), GzipError> {
        let nlen = self.bits(5)? as usize + 257;
        let ndist = self.bits(5)? as usize + 1;
        let ncode = self.bits(4)? as usize + 4;
        if nlen > 286 || ndist > 30 {
            return Err(GzipError::BadData("too many length or distance codes"));
        }
        let mut cl = [0u8; 19];
        for &o in CL_ORDER.iter().take(ncode) {
            cl[o] = self.bits(3)? as u8;
        }
        let clh = Huff::new(&cl, false)?;
        let mut lengths = vec![0u8; nlen + ndist];
        let mut i = 0;
        while i < nlen + ndist {
            let sym = self.decode(&clh)?;
            match sym {
                0..=15 => {
                    lengths[i] = sym as u8;
                    i += 1;
                }
                16 => {
                    if i == 0 {
                        return Err(GzipError::BadData("repeat with no previous length"));
                    }
                    let prev = lengths[i - 1];
                    let n = 3 + self.bits(2)? as usize;
                    if i + n > lengths.len() {
                        return Err(GzipError::BadData("code lengths overflow"));
                    }
                    lengths[i..i + n].fill(prev);
                    i += n;
                }
                17 | 18 => {
                    let n = if sym == 17 {
                        3 + self.bits(3)? as usize
                    } else {
                        11 + self.bits(7)? as usize
                    };
                    if i + n > lengths.len() {
                        return Err(GzipError::BadData("code lengths overflow"));
                    }
                    i += n;
                }
                _ => return Err(GzipError::BadData("invalid code length symbol")),
            }
        }
        if lengths[256] == 0 {
            return Err(GzipError::BadData("no end-of-block code"));
        }
        let lit = Huff::new(&lengths[..nlen], false)?;
        let dist = Huff::new(&lengths[nlen..], true)?;
        self.codes(&lit, &dist)
    }

    fn fixed(&mut self) -> Result<(), GzipError> {
        let mut l = [0u8; 288];
        l[..144].fill(8);
        l[144..256].fill(9);
        l[256..280].fill(7);
        l[280..].fill(8);
        let lit = Huff::new(&l, false)?;
        let dist = Huff::new(&[5u8; 32], true)?;
        self.codes(&lit, &dist)
    }

    fn deflate(&mut self) -> Result<(), GzipError> {
        loop {
            let last = self.bits(1)?;
            match self.bits(2)? {
                0 => self.stored()?,
                1 => self.fixed()?,
                2 => self.dynamic()?,
                _ => return Err(GzipError::BadData("reserved block type")),
            }
            if last == 1 {
                return Ok(());
            }
        }
    }

    /// Reads a 32-bit little-endian trailer field from the byte-aligned input.
    fn le32(&mut self) -> Result<u32, GzipError> {
        let mut v = 0u32;
        for k in 0..4 {
            v |= u32::from(self.bits(8)? as u8) << (8 * k);
        }
        Ok(v)
    }

    /// Reads the header of a member whose first byte is `b0`.
    fn header(&mut self, b0: u8) -> Result<(), GzipError> {
        if b0 != 0x1f || self.byte()? != 0x8b {
            return Err(GzipError::BadHeader("missing gzip magic bytes"));
        }
        if self.byte()? != 8 {
            return Err(GzipError::BadHeader("compression method is not deflate"));
        }
        let flg = self.byte()?;
        if flg & 0xE0 != 0 {
            return Err(GzipError::BadHeader("reserved flag bits are set"));
        }
        for _ in 0..6 {
            self.byte()?; // mtime, xfl, os
        }
        if flg & 4 != 0 {
            let n = usize::from(self.byte()?) | usize::from(self.byte()?) << 8;
            for _ in 0..n {
                self.byte()?;
            }
        }
        for bit in [8u8, 16] {
            if flg & bit != 0 {
                while self.byte()? != 0 {}
            }
        }
        if flg & 2 != 0 {
            self.byte()?;
            self.byte()?;
        }
        Ok(())
    }

    fn run(&mut self) -> Result<u64, GzipError> {
        let Some(mut b0) = self.next_byte()? else {
            return Err(GzipError::Truncated);
        };
        loop {
            self.header(b0)?;
            self.member_out = 0;
            self.crc = Crc32::new();
            self.bitbuf = 0;
            self.bitcnt = 0;
            self.deflate()?;
            self.flush()?;
            // The trailer starts at the next byte boundary.
            let drop = self.bitcnt % 8;
            self.bitbuf >>= drop;
            self.bitcnt -= drop;
            let crc = self.le32()?;
            let isize_ = self.le32()?;
            let actual = self.crc.finish();
            if crc != actual {
                return Err(GzipError::BadChecksum {
                    expected: crc,
                    actual,
                });
            }
            let len = self.member_out as u32;
            if isize_ != len {
                return Err(GzipError::BadLength {
                    expected: isize_,
                    actual: len,
                });
            }
            self.total += self.member_out;
            // Bytes buffered but not consumed belong to the next member.
            debug_assert_eq!(self.bitcnt, 0);
            match self.next_byte()? {
                None => break,
                Some(0) => {
                    // Zero padding after the last member is tolerated.
                    while let Some(z) = self.next_byte()? {
                        if z != 0 {
                            return Err(GzipError::TrailingGarbage);
                        }
                    }
                    break;
                }
                Some(b) => b0 = b,
            }
        }
        self.out.flush()?;
        Ok(self.total)
    }
}

/// Inflates every gzip member of `input` into `output` and returns the number of bytes
/// written. The CRC-32 and length of each member are checked; a stream that ends early
/// is [`GzipError::Truncated`]. Zero bytes after the last member are ignored.
pub fn gunzip<R: Read, W: Write>(input: R, output: W) -> Result<u64, GzipError> {
    Inflater::new(input, output).run()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gz_of_deflate(raw: &[u8], data: &[u8]) -> Vec<u8> {
        let mut v = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 3];
        v.extend_from_slice(raw);
        v.extend_from_slice(&crc32(data).to_le_bytes());
        v.extend_from_slice(&(data.len() as u32).to_le_bytes());
        v
    }

    fn un(bytes: &[u8]) -> Result<Vec<u8>, GzipError> {
        let mut out = Vec::new();
        gunzip(bytes, &mut out)?;
        Ok(out)
    }

    #[test]
    fn crc_known_value() {
        assert_eq!(crc32(b"a"), 0xe8b7_be43);
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
    }

    #[test]
    fn empty_member() {
        let gz = gz_of_deflate(&[0x03, 0x00], b"");
        assert_eq!(un(&gz).unwrap(), b"");
    }

    #[test]
    fn fixed_block_literal() {
        let gz = gz_of_deflate(&[0x4b, 0x04, 0x00], b"a");
        assert_eq!(un(&gz).unwrap(), b"a");
    }

    #[test]
    fn stored_block() {
        let data = b"stored bytes";
        let mut raw = vec![0x01, data.len() as u8, 0, !(data.len() as u8), 0xff];
        raw.extend_from_slice(data);
        assert_eq!(un(&gz_of_deflate(&raw, data)).unwrap(), data);
    }

    #[test]
    fn two_stored_blocks_and_two_members() {
        let mut raw = vec![0x00, 2, 0, 0xfd, 0xff, b'a', b'b'];
        raw.extend_from_slice(&[0x01, 1, 0, 0xfe, 0xff, b'c']);
        let one = gz_of_deflate(&raw, b"abc");
        assert_eq!(un(&one).unwrap(), b"abc");
        let mut two = one.clone();
        two.extend_from_slice(&one);
        assert_eq!(un(&two).unwrap(), b"abcabc");
        two.extend_from_slice(&[0, 0, 0]);
        assert_eq!(un(&two).unwrap(), b"abcabc");
        two.push(7);
        assert!(matches!(un(&two), Err(GzipError::TrailingGarbage)));
    }

    #[test]
    fn truncation_and_corruption_are_errors() {
        let gz = gz_of_deflate(&[0x4b, 0x04, 0x00], b"a");
        for cut in 0..gz.len() {
            assert!(un(&gz[..cut]).is_err(), "cut at {cut}");
        }
        let mut bad = gz.clone();
        let n = bad.len();
        bad[n - 5] ^= 1;
        assert!(matches!(un(&bad), Err(GzipError::BadChecksum { .. })));
        let mut bad = gz.clone();
        let n = bad.len();
        bad[n - 4] ^= 1;
        assert!(matches!(un(&bad), Err(GzipError::BadLength { .. })));
        assert!(matches!(un(b"plain"), Err(GzipError::BadHeader(_))));
        assert!(matches!(un(b""), Err(GzipError::Truncated)));
    }

    #[test]
    fn reserved_block_type_and_bad_distance() {
        // BFINAL=1, BTYPE=3.
        assert!(matches!(
            un(&gz_of_deflate(&[0x07], b"")),
            Err(GzipError::BadData(_))
        ));
        // Fixed block starting with a back reference: literal 'a' omitted, so the
        // distance points before the start. Length symbol 257 (7 bits 0000001),
        // distance code 0 (5 bits).
        assert!(matches!(
            un(&gz_of_deflate(&[0x03, 0x02, 0x00], b"")),
            Err(GzipError::BadData(_) | GzipError::Truncated)
        ));
    }
}
