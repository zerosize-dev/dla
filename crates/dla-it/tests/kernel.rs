//! A1: what the microVM guest kernel offers.
//!
//! The guest kernel comes from the microsandbox backend, not from the host, and
//! nothing documents its configuration. This test reads it and records the facts
//! (BTF, tracefs, kprobes and uprobes, cgroup v2, ptrace scope, privileges).
//!
//! It asserts only what must hold on any Linux guest: the kernel is Linux, the
//! probe saw a user id and `/proc` is mounted. It does not assert that BTF or
//! tracefs exist. The recorded facts decide what the later tier 3 tests expect;
//! a guest without BTF is a result, not a failure.

#![cfg(feature = "sandbox-it")]

use dla_it::vm::{DEBIAN, Guest, write_out};
use dla_it::{KERNEL_PROBE_SCRIPT, KernelFacts, require_vm};

#[tokio::test]
async fn kernel_probe_records_facts() {
    require_vm!();
    let guest = Guest::boot(DEBIAN, 512).await.expect("boot");

    let out = guest.sh_ok(KERNEL_PROBE_SCRIPT).await;
    let facts = KernelFacts::parse(&out);
    eprintln!(
        "kernel facts:\n{}",
        serde_json::to_string_pretty(&facts.to_json()).unwrap()
    );
    write_out("kernel-facts.json", &facts.to_json());

    assert_eq!(facts.get("uname_s"), Some("Linux"), "facts: {out}");
    assert_eq!(facts.flag("proc_mounted"), Some(true), "facts: {out}");
    assert!(
        facts.get("uid").is_some_and(|u| u.parse::<u32>().is_ok()),
        "the probe must report a numeric user id: {out}"
    );
    // The booleans the later tests branch on must at least be readable.
    for key in [
        "btf",
        "tracefs",
        "tracefs_syscalls",
        "kprobe_events",
        "uprobe_events",
        "cgroup2",
    ] {
        assert!(facts.flag(key).is_some(), "fact {key} missing: {out}");
    }
}
