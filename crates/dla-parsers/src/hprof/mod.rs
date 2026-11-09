//! HPROF heap dump reader and summarizer.
//!
//! [`summarize`] reads a `.hprof` or `.hprof.gz` file in two streaming passes (classes,
//! names, layouts and roots; then edges), builds the strong reference graph with every
//! class object as an artificial root, computes dominators and retained sizes, and
//! reports the holders that retain the most memory.

mod analyze;
mod graph;
mod ids;
mod reader;
mod summary;

use std::{
    fmt,
    fs::{self, File},
    io::{self, BufWriter, Read},
    path::{Path, PathBuf},
    time::Instant,
};

pub use analyze::pretty_class;
pub use summary::{
    Accumulation, ClassStat, Counts, HeaderInfo, HeapSummary, Holder, HolderKind, SIZE_NOTES,
    Stats, ViaHolder, name_hash,
};

use crate::inflate::{GzipError, gunzip, is_gzip};

/// Default cap on the number of objects analyzed.
pub const DEFAULT_MAX_OBJECTS: usize = 20_000_000;
/// Default cap on dominator iterations.
pub const DEFAULT_MAX_PASSES: u32 = 100;

/// How many times its compressed size a gzip dump may inflate to (real dumps compress
/// about 10x; deflate cannot exceed about 1030x).
pub const MAX_INFLATE_RATIO: u64 = 100;
/// Smallest inflate limit, so tiny files are not refused on ratio alone.
pub const MIN_INFLATE_LIMIT: u64 = 64 << 20;

/// Why a dump could not be summarized.
#[derive(Debug)]
pub enum HprofError {
    /// A file operation failed.
    Io(io::Error),
    /// The file is not an HPROF dump (reason).
    NotHprof(String),
    /// The file ends inside a record (offset).
    Truncated(u64),
    /// A record is malformed.
    Malformed(String),
    /// A gzip dump could not be inflated.
    Gzip(GzipError),
    /// A gzip dump would inflate past the allowed size (limit in bytes).
    InflateTooLarge(u64),
    /// The disk filled up while a gzip dump was being inflated.
    DiskFull,
}

impl fmt::Display for HprofError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "i/o error: {e}"),
            Self::NotHprof(m) => write!(f, "not an HPROF file: {m}"),
            Self::Truncated(o) => write!(
                f,
                "the dump is truncated (ends inside a record near offset {o})"
            ),
            Self::Malformed(m) => write!(f, "malformed dump: {m}"),
            Self::Gzip(e) => write!(f, "cannot inflate the dump: {e}"),
            Self::InflateTooLarge(l) => write!(
                f,
                "the gzip dump inflates to more than {l} bytes ({}x its size at most is allowed)",
                MAX_INFLATE_RATIO
            ),
            Self::DiskFull => f.write_str("no space left on the device while inflating the dump"),
        }
    }
}

impl std::error::Error for HprofError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Gzip(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for HprofError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<GzipError> for HprofError {
    fn from(e: GzipError) -> Self {
        Self::Gzip(e)
    }
}

/// Limits and scratch space of one summary.
#[derive(Debug, Clone)]
pub struct Options {
    /// Where a gzip dump is inflated to (a file that is removed afterwards). Required for
    /// `.hprof.gz` input; the system temp directory is used when `None`.
    pub tmp_dir: Option<PathBuf>,
    /// Objects analyzed at most; above it the result is `truncated` and has no holders.
    pub max_objects: usize,
    /// Dominator iterations at most.
    pub max_passes: u32,
    /// Whether 64-bit references are taken to be 4 bytes (compressed oops) when sizing
    /// objects; `None` assumes so for dumps under 24 GiB.
    pub compressed_oops: Option<bool>,
    /// Bytes a gzip dump may inflate to at most; `None` allows [`MAX_INFLATE_RATIO`] times
    /// the compressed size (at least [`MIN_INFLATE_LIMIT`]).
    pub max_inflated: Option<u64>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            tmp_dir: None,
            max_objects: DEFAULT_MAX_OBJECTS,
            max_passes: DEFAULT_MAX_PASSES,
            compressed_oops: None,
            max_inflated: None,
        }
    }
}

/// How a file starts, by its magic bytes (not its name).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sniff {
    /// `JAVA PROFILE 1.0.x`.
    Hprof,
    /// A gzip stream (the content is checked when inflated).
    Gzip,
    /// Anything else.
    Other,
}

/// Reads the first bytes of `path` and classifies them.
pub fn sniff(path: &Path) -> io::Result<Sniff> {
    let mut head = [0u8; 16];
    let mut f = File::open(path)?;
    let mut n = 0;
    while n < head.len() {
        let k = f.read(&mut head[n..])?;
        if k == 0 {
            break;
        }
        n += k;
    }
    let head = &head[..n];
    Ok(if is_gzip(head) {
        Sniff::Gzip
    } else if head.starts_with(b"JAVA PROFILE 1.0") {
        Sniff::Hprof
    } else {
        Sniff::Other
    })
}

fn unique_tmp(dir: &Path, stem: &str) -> PathBuf {
    let mut i = 0u32;
    loop {
        let p = dir.join(format!("{stem}.{}.{i}.hprof.tmp", std::process::id()));
        if !p.exists() {
            return p;
        }
        i += 1;
    }
}

/// Passes writes through until `limit` bytes were written, then fails and records it.
struct Limited<'a, W: io::Write> {
    inner: W,
    left: u64,
    exceeded: &'a std::cell::Cell<bool>,
}

impl<W: io::Write> io::Write for Limited<'_, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if buf.len() as u64 > self.left {
            self.exceeded.set(true);
            return Err(io::Error::other("inflate limit exceeded"));
        }
        self.left -= buf.len() as u64;
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

fn is_storage_full(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::StorageFull || e.raw_os_error() == Some(28)
}

/// Removes the inflated copy when dropped.
struct TempFile(PathBuf);

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// Summarizes the dump at `path` (`.hprof`, or gzip detected by magic bytes).
pub fn summarize(path: &Path, opts: &Options) -> Result<HeapSummary, HprofError> {
    let kind = sniff(path)?;
    let params = analyze::Params {
        max_objects: opts.max_objects,
        max_passes: opts.max_passes,
        compressed_oops: opts.compressed_oops,
    };
    match kind {
        Sniff::Gzip => {
            let dir = opts.tmp_dir.clone().unwrap_or_else(std::env::temp_dir);
            fs::create_dir_all(&dir)?;
            let stem = path
                .file_name()
                .map_or_else(|| "dump".to_owned(), |s| s.to_string_lossy().into_owned());
            let tmp = TempFile(unique_tmp(&dir, &stem));
            let t0 = Instant::now();
            {
                let input = File::open(path)?;
                let limit = opts.max_inflated.unwrap_or_else(|| {
                    input
                        .metadata()
                        .map_or(0, |m| m.len())
                        .saturating_mul(MAX_INFLATE_RATIO)
                        .max(MIN_INFLATE_LIMIT)
                });
                let exceeded = std::cell::Cell::new(false);
                let mut out = Limited {
                    inner: BufWriter::with_capacity(1 << 20, File::create(&tmp.0)?),
                    left: limit,
                    exceeded: &exceeded,
                };
                let done = gunzip(io::BufReader::new(input), &mut out)
                    .map_err(|e| (exceeded.get(), e))
                    .and_then(|_| {
                        io::Write::flush(&mut out).map_err(|e| (false, GzipError::Io(e)))
                    });
                if let Err((over, e)) = done {
                    return Err(if over {
                        HprofError::InflateTooLarge(limit)
                    } else if matches!(&e, GzipError::Io(io) if is_storage_full(io)) {
                        HprofError::DiskFull
                    } else {
                        e.into()
                    });
                }
            }
            let inflate_ms = t0.elapsed().as_millis() as u64;
            let mut s = analyze::analyze(&tmp.0, &params)?;
            s.stats.gzip = true;
            s.stats.inflate_ms = inflate_ms;
            Ok(s)
        }
        _ => analyze::analyze(path, &params),
    }
}

/// Test support: the number of bytes a gzip dump inflates to is not needed by callers, but
/// the header of a plain dump is: reads `(version, id size, timestamp)` quickly.
pub fn read_header_info(path: &Path) -> Result<HeaderInfo, HprofError> {
    let h = reader::read_header(&mut io::BufReader::new(File::open(path)?))?;
    Ok(HeaderInfo {
        version: h.version,
        id_size: h.id_size,
        timestamp_ms: h.timestamp_ms,
    })
}
