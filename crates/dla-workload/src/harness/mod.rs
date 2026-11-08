//! Loop harnesses: `dla harness gen`, `dla harness run` and the machinery `dla alloc-test`
//! reuses.
//!
//! # Layout of a harness directory (`.dla/<run>/harness/H<n>/`)
//!
//! | Path | Written by | Holds |
//! |---|---|---|
//! | `harness.json` | gen | the [`dla_core::harness::HarnessManifest`] |
//! | `probes.json` | gen | `{"probes": [ProbeSpec...]}` as serialised by dla-core |
//! | `work/` | gen | the scratch copy of the project; build outputs land here |
//! | `agent/` | gen | per-runtime agent files: `dla-agent.jar`, `sitecustomize.py`, `dla-preload.js`, `libdlashim.so`, Go test files to drop into `work/` |
//! | `driver/` | gen | driver-mode template instance the model edits |
//! | `out/<label>/` | run | dump records (`<pid>.jsonl`) of one measured process, label `p<k>-<mode>` |
//! | `logs/` | run | build and run stdout/stderr |
//! | `cov/` | run | coverage data |
//!
//! Generation lives in [`gen`] (with [`probes`], [`commands`], [`templates`] and
//! [`scratch`]); execution in [`run`] (with [`dumps`], [`build`], [`symbolize`] and
//! [`cover`]).

pub mod build;
pub mod commands;
pub mod cover;
pub mod dumps;
pub mod r#gen;
pub mod probes;
pub mod run;
pub mod scratch;
pub mod symbolize;
pub mod templates;

use std::path::{Path, PathBuf};

/// Manifest file name inside a harness directory.
pub const MANIFEST: &str = "harness.json";
/// Probe list file name.
pub const PROBES: &str = "probes.json";
/// Scratch copy of the project.
pub const WORK: &str = "work";
/// Agent files.
pub const AGENT: &str = "agent";
/// Driver template instance.
pub const DRIVER: &str = "driver";
/// Dump records.
pub const OUT: &str = "out";
/// Logs.
pub const LOGS: &str = "logs";
/// Coverage data.
pub const COV: &str = "cov";

/// `.dla/<run>/harness/<id>`.
pub fn harness_dir(run_dir: &Path, id: &str) -> PathBuf {
    run_dir.join("harness").join(id)
}
