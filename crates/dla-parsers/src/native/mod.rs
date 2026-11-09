//! Native formats: Valgrind memcheck XML and vgdb monitor output, LeakSanitizer text,
//! `heaptrack_print` text, dhat JSON, `/proc/<pid>/fd` classification and Rust symbol
//! demangling.
//!
//! Every parser is a pure function from the tool's text to a [`crate::model`] type (or a small
//! report holding one), and fails with a [`ParseError`] that says what was expected instead
//! of panicking.

use std::fmt;

pub mod demangle;
pub mod dhat;
pub mod fd;
pub mod frames;
pub mod heaptrack;
pub mod lsan;
pub mod valgrind;
pub mod vgdb;
mod xml;

/// Why a tool output could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(String);

impl ParseError {
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }

    /// The message: what was found and what was expected.
    pub fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}
