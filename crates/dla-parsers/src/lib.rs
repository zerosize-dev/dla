//! DLA parsers: a pure-Rust gzip reader and the HPROF heap dump summarizer.
//!
//! The crate uses std, serde and serde_json only and contains no unsafe code.

pub mod hprof;
pub mod inflate;
