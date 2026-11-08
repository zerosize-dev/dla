//! Sampling for DLA: periodic collection of memory and resource measurements from a running target into series.
//!
//! The [`os`] module reads operating-system figures (resident memory, file descriptors, threads,
//! container memory) for a process id on Linux, macOS and Windows.

pub mod os;
