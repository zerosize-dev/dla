//! JVM text and JSON formats: JFR `jdk.OldObjectSample` (`jfr print` text), thread dumps
//! (`Thread.dump_to_file -format=json`), `VM.classloader_stats`, `VM.native_memory`
//! summaries and diffs, and generated-class name normalization.
//!
//! Every parser is a pure function from the tool's text to a [`crate::model`] type, and
//! fails with a [`ParseError`] that says what was expected instead of panicking.

use std::fmt;

use crate::model::Frame;

pub mod classloader;
pub mod deproxy;
pub mod jfr;
pub mod nmt;
pub mod thread_dump;

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

/// Class name prefixes that are never project code: the JDK, language runtimes and
/// framework internals that sit between a project frame and the allocation.
const LIBRARY_PREFIXES: [&str; 17] = [
    "java.",
    "javax.",
    "jdk.",
    "sun.",
    "com.sun.",
    "org.w3c.",
    "org.xml.",
    "kotlin.",
    "kotlinx.",
    "scala.",
    "org.springframework.",
    "org.apache.",
    "org.hibernate.",
    "io.netty.",
    "com.fasterxml.",
    "reactor.",
    "org.jboss.",
];

/// True when `name` (a class or a `Class.method` function name) looks like JDK or common
/// library code rather than project code.
pub fn is_library_name(name: &str) -> bool {
    LIBRARY_PREFIXES.iter().any(|p| name.starts_with(p))
}

/// True when the frame is not JDK or library code and not a generated proxy or lambda
/// class, so it can name a project site.
pub fn is_project_frame(f: &Frame) -> bool {
    !f.function.is_empty() && !is_library_name(&f.function) && !f.function.starts_with("<")
}

/// The first project frame of a stack (innermost first), skipping generated proxy
/// classes (their lines are not in any source file).
pub fn first_project_frame(frames: &[Frame]) -> Option<&Frame> {
    frames
        .iter()
        .find(|f| is_project_frame(f) && !deproxy::is_generated(class_of_function(&f.function)))
}

/// `pkg.Class.method` to `pkg.Class`; the name itself when it has no dot.
pub fn class_of_function(function: &str) -> &str {
    function.rsplit_once('.').map_or(function, |(c, _)| c)
}

/// Parses `pkg.Class.method(File.java:12)` (the form of a thread-dump line, with an
/// optional `module/` prefix) into a frame.
pub(crate) fn parse_java_frame(text: &str) -> Frame {
    let text = text.trim();
    let (head, tail) = match text.find('(') {
        Some(p) => (&text[..p], text[p + 1..].trim_end_matches(')')),
        None => (text, ""),
    };
    let (module, function) = match head.split_once('/') {
        Some((m, f)) => (Some(m.to_owned()), f),
        None => (None, head),
    };
    let (file, line) = match tail.rsplit_once(':') {
        Some((f, l)) => (
            Some(f.to_owned()).filter(|f| !f.is_empty()),
            l.trim().parse::<u32>().ok(),
        ),
        None => (None, None),
    };
    Frame {
        function: function.to_owned(),
        file,
        line,
        module,
    }
}
