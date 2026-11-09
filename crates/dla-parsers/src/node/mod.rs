//! Node.js formats: V8 `.heapsnapshot` files (a streaming reader, class histogram,
//! retention paths, dominators and holders), `.heapprofile` sampling profiles, and
//! source maps.
//!
//! Everything is read with std and serde only. Arithmetic over values read from files
//! is checked, and memory use is bounded by a preflight before a snapshot is loaded.

use std::fmt;

pub mod graph;
pub mod heapprofile;
pub mod holders;
mod json;
pub mod snapshot;
pub mod sourcemap;

pub use holders::{Accumulation, Holder, HolderKind, HolderOptions, HolderSummary, Hop};
pub use snapshot::{Options, Snapshot, sniff};

/// Why a Node file could not be read.
#[derive(Debug)]
pub enum NodeError {
    /// The file is not in the format.
    NotThisFormat(String),
    /// The JSON is damaged.
    Malformed(String),
    /// The file ends early.
    Truncated(String),
    /// A limit was hit (size, memory, element count).
    TooLarge(String),
    /// Reading failed.
    Io(std::io::Error),
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotThisFormat(m)
            | Self::Malformed(m)
            | Self::Truncated(m)
            | Self::TooLarge(m) => f.write_str(m),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for NodeError {}

impl From<std::io::Error> for NodeError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// A script URL as a plain path: `file:///a/b.js` becomes `/a/b.js` (percent escapes
/// decoded); anything else is returned as it is.
pub fn url_to_path(url: &str) -> String {
    let Some(rest) = url.strip_prefix("file://") else {
        return url.to_owned();
    };
    // `file:///C:/x` on Windows keeps the drive without the leading slash.
    let rest = match rest.as_bytes() {
        [b'/', d, b':', ..] if d.is_ascii_alphabetic() => &rest[1..],
        _ => rest,
    };
    let b = rest.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let (Some(h), Some(l)) = (
                (b[i + 1] as char).to_digit(16),
                (b[i + 2] as char).to_digit(16),
            )
        {
            out.push((h * 16 + l) as u8);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_become_paths() {
        assert_eq!(url_to_path("file:///a/b%20c.js"), "/a/b c.js");
        assert_eq!(url_to_path("file:///C:/x/y.js"), "C:/x/y.js");
        assert_eq!(url_to_path("node:internal/timers"), "node:internal/timers");
        assert_eq!(url_to_path("/plain.js"), "/plain.js");
        assert_eq!(url_to_path("file:///a%2"), "/a%2");
    }
}
