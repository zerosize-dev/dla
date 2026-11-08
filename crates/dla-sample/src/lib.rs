//! Sampling for DLA: periodic collection of memory and resource measurements from a running target into series.
//!
//! The [`os`] module reads operating-system figures (resident memory, file descriptors, threads,
//! container memory) for a process id on Linux, macOS and Windows. The [`backend`] module adds
//! runtime counters (JVM through `jcmd`, Go through `net/http/pprof`, Node.js through the
//! inspector, Python 3.14 through remote exec) and [`session`] runs both on a fixed cadence.

pub mod backend;
mod exec;
mod gopprof;
mod http;
mod jcmd;
mod node;
pub mod os;
pub mod proc;
mod pyremote;
pub mod session;

pub use backend::{
    Backend, BackendError, BackendKind, BackendSpec, CountersSpec, OpenCtx, Reading, auto_specs,
    open_backend, parse_counters,
};
pub use session::{
    BackendReport, BackendState, SessionConfig, SessionOutput, TickPoint, run as run_session,
};
