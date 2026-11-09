//! DLA parsers: a pure-Rust gzip reader, the HPROF heap dump summarizer, and the JVM and
//! native tool formats behind `dla summarize`.
//!
//! The crate uses std, serde and serde_json only and contains no unsafe code.

pub mod hprof;
pub mod inflate;
pub mod jvm;
pub mod model;
pub mod native;
