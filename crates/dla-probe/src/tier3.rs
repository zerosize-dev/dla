//! Coverage rows for kernel-level and device tools DLA has no driver for.
//!
//! Tier 3 names eBPF, perf and GPU counters. `dla attach bpftrace` drives one of them. The
//! others are listed here as coverage rows so a report states what was not looked at instead of
//! staying silent: bcc `memleak`, GPU memory through NVML, and Valgrind on a JVM. A row is
//! written once per run and never replaces a row a real run already wrote.

use dla_core::{
    coverage::{BackendCoverage, CoverageManifest},
    runtime::Runtime,
};

/// One row: worker runtime, backend name, reason.
struct Row {
    backend: &'static str,
    reason: &'static str,
}

const BCC: Row = Row {
    backend: "bcc-memleak",
    reason: "bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root)",
};
const GPU: Row = Row {
    backend: "gpu-nvml",
    reason: "GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot",
};
const JVM_VALGRIND: Row = Row {
    backend: "valgrind",
    reason: "Valgrind is not run on the JVM: the virtual machine's own allocations drown the project's; use the allocator shim, jcmd NMT or `dla attach bpftrace`",
};

/// The runtime whose worker carries the native-side rows: native, then rust, then the first.
pub fn native_worker(runtimes: &[Runtime]) -> Option<Runtime> {
    [Runtime::Native, Runtime::Rust]
        .into_iter()
        .find(|r| runtimes.contains(r))
        .or_else(|| runtimes.first().copied())
}

/// Adds the rows to `m` for the probed `runtimes`; an existing row stays.
pub fn note(m: &mut CoverageManifest, runtimes: &[Runtime]) {
    let mut put = |worker: Runtime, row: &Row| {
        let key = worker.as_str();
        if m.workers
            .get(key)
            .is_some_and(|w| w.backends.contains_key(row.backend))
        {
            return;
        }
        m.set_backend(key, row.backend, BackendCoverage::blocked(row.reason));
    };
    if runtimes.contains(&Runtime::Jvm) {
        put(Runtime::Jvm, &JVM_VALGRIND);
    }
    if let Some(w) = native_worker(runtimes) {
        put(w, &BCC);
        put(w, &GPU);
    }
}

#[cfg(test)]
mod tests {
    use dla_core::coverage::BackendStatus;

    use super::*;

    #[test]
    fn a_jvm_project_gets_the_valgrind_row_and_the_native_rows() {
        let mut m = CoverageManifest::new(None);
        note(&mut m, &[Runtime::Jvm, Runtime::Rust]);
        let jvm = &m.workers["jvm"].backends;
        assert_eq!(jvm["valgrind"].status, BackendStatus::Blocked);
        let rust = &m.workers["rust"].backends;
        assert!(rust.contains_key("bcc-memleak") && rust.contains_key("gpu-nvml"));
        assert!(!m.workers.contains_key("native"));
    }

    #[test]
    fn rows_follow_the_runtimes_found() {
        let mut m = CoverageManifest::new(None);
        note(&mut m, &[]);
        assert!(m.workers.is_empty());
        note(&mut m, &[Runtime::Python]);
        assert!(m.workers["python"].backends.contains_key("bcc-memleak"));
        assert!(!m.workers["python"].backends.contains_key("valgrind"));
        let mut m = CoverageManifest::new(None);
        note(&mut m, &[Runtime::Node, Runtime::Native]);
        assert!(m.workers["native"].backends.contains_key("gpu-nvml"));
        assert!(!m.workers.contains_key("node"));
    }

    #[test]
    fn an_existing_row_stays() {
        let mut m = CoverageManifest::new(None);
        m.set_backend("jvm", "valgrind", BackendCoverage::ran(None, 3));
        note(&mut m, &[Runtime::Jvm]);
        assert_eq!(
            m.workers["jvm"].backends["valgrind"].status,
            BackendStatus::Ran
        );
    }
}
