//! The probe report: `.dla/<run>/probe/probe.json`, written by `dla probe`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::runtime::Runtime;

string_enum! {
    /// Result of one build attempt.
    pub enum BuildStatus {
        /// The build command succeeded.
        Ok = "ok",
        /// The build command failed.
        Failed = "failed",
        /// The build command ran out of time.
        Timeout = "timeout",
        /// No build was attempted (not requested, no consent, or no command found).
        Skipped = "skipped",
    }
}

/// A tool name with the version it reported.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ToolVersion {
    /// Tool name, for example `java`, `go`, `rustc`.
    pub name: String,
    /// The version it reported, when it ran.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Why no version is known (not found, timed out, failed).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// One runtime found in the project.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct RuntimeProbe {
    /// The runtime.
    pub runtime: Runtime,
    /// Project-relative files that identified it (build files first), at most a few.
    pub detected_by: Vec<String>,
    /// Number of source files seen for the runtime.
    pub source_files: u64,
    /// Toolchain versions (`java -version`, `go version`, ...).
    #[serde(default)]
    pub toolchain: Vec<ToolVersion>,
    /// Version the project declares (the `go` line of go.mod, `<release>` in a pom, ...).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub declared_version: Option<String>,
}

/// A backend binary or other analysis tool.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ToolProbe {
    /// Tool or backend name from the registry.
    pub name: String,
    /// Runtime it serves.
    pub runtime: Runtime,
    /// True when found.
    pub available: bool,
    /// Where it was found: `path` or `tools-dir`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Full path of the binary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Version it reported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// How to install it when missing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub install_hint: Option<String>,
}

/// Facts that decide which intrusive steps are possible.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Default)]
pub struct PrivilegeFacts {
    /// Operating system (`linux`, `macos`, `windows`).
    pub os: String,
    /// Numeric user id on Unix.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uid: Option<u32>,
    /// True when running as root (uid 0) or an elevated administrator.
    pub root: bool,
    /// Linux `kernel.yama.ptrace_scope`, when readable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ptrace_scope: Option<i32>,
    /// Hints that the process runs inside a container.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub container: Vec<String>,
    /// Output of `csrutil status` on macOS.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub macos_sip: Option<String>,
}

/// A test runner found from build files or sources.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct TestRunnerProbe {
    /// Runtime.
    pub runtime: Runtime,
    /// Runner name (`junit`, `testng`, `pytest`, `unittest`, `node --test`, `jest`,
    /// `vitest`, `go test`, `cargo test`, `make test`).
    pub name: String,
    /// Project-relative file that shows it.
    pub evidence: String,
}

/// A library whose presence changes how the harness is configured.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct LibraryProbe {
    /// Runtime.
    pub runtime: Runtime,
    /// Library (`netty`, `hikaricp`, `micrometer`, `tomcat`, `asyncio`, `goleak`, ...).
    pub library: String,
    /// Project-relative file that shows it.
    pub evidence: String,
    /// Flags or settings the M3 harness switches on for it.
    pub harness_flags: Vec<String>,
}

/// One build attempt (`dla probe --build`).
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct BuildProbe {
    /// Runtime built.
    pub runtime: Runtime,
    /// The command line that was (or would be) run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    /// Outcome.
    pub status: BuildStatus,
    /// Exit code, when the command ended.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    /// Wall time in milliseconds.
    #[serde(default)]
    pub duration_ms: u64,
    /// Last lines of the combined output, or the reason for `skipped`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_tail: Option<String>,
}

/// The full probe report.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct ProbeReport {
    /// Schema version.
    pub schema_version: u32,
    /// Absolute project root that was probed.
    pub project: String,
    /// Runtimes detected.
    #[serde(default)]
    pub runtimes: Vec<RuntimeProbe>,
    /// Backend and analysis tools, found or missing, for each detected runtime.
    #[serde(default)]
    pub tools: Vec<ToolProbe>,
    /// Privilege facts.
    pub privilege: PrivilegeFacts,
    /// Test runners.
    #[serde(default)]
    pub test_runners: Vec<TestRunnerProbe>,
    /// Libraries and the harness flags they imply.
    #[serde(default)]
    pub libraries: Vec<LibraryProbe>,
    /// Build attempts (empty without `--build`).
    #[serde(default)]
    pub builds: Vec<BuildProbe>,
}
