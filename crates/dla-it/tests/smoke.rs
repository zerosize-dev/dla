//! Smoke test: the static musl `dla` runs in a stock image.
//!
//! It copies the binary and the plugin `data/` directory into
//! `debian:bookworm-slim`, which has no Rust, no Git and no analyzers, then runs
//! `dla run init` and `dla probe` in an empty project directory.
//!
//! It asserts that:
//! - the binary is statically linked (`ldd` says "not a dynamic executable", or
//!   "statically linked" for a static-PIE binary, which the musl build makes) and
//!   reports its version;
//! - `dla run init` creates a run, its state file and an empty ledger;
//! - `dla probe` completes, finds no runtime in an empty project, and reports a
//!   privilege fact equal to what the guest says (`id -u` is 0 exactly when
//!   `privilege.root` is true).

#![cfg(feature = "sandbox-it")]

use dla_it::require_vm;
use dla_it::vm::{DEBIAN, GUEST_DLA, Guest};

#[tokio::test]
async fn static_dla_runs_init_and_probe_in_a_stock_image() {
    require_vm!();
    let guest = Guest::boot(DEBIAN, 512).await.expect("boot");
    guest.install_dla().await;
    guest.sh_ok("mkdir -p /srv/proj").await;

    let linkage = guest.sh(&format!("ldd {GUEST_DLA} 2>&1")).await.stdout;
    // glibc's ldd says "not a dynamic executable" for a plain static binary and
    // "statically linked" for a static-PIE one (glibc 2.34 and later).
    assert!(
        linkage.contains("not a dynamic executable") || linkage.contains("statically linked"),
        "the guest binary must be static: {linkage:?}"
    );

    let version = guest.sh_ok(&format!("{GUEST_DLA} --version")).await;
    assert!(version.starts_with("dla "), "version output: {version:?}");

    let init = guest.dla_json("/srv/proj", "run init").await;
    assert_eq!(init["ok"], true, "{init}");
    assert_eq!(init["stage"], "intake", "{init}");
    let run_id = init["run_id"].as_str().expect("run_id").to_string();
    let state = guest
        .sh_ok(&format!("test -s /srv/proj/.dla/{run_id}/state.json && test -f /srv/proj/.dla/{run_id}/ledger.jsonl && echo present"))
        .await;
    assert_eq!(state.trim(), "present");

    let probe = guest.dla_json("/srv/proj", "probe").await;
    assert_eq!(probe["ok"], true, "{probe}");
    let summary = &probe["summary"];
    assert_eq!(
        summary["runtimes"].as_array().map(Vec::len),
        Some(0),
        "an empty project has no runtime: {probe}"
    );
    let uid = guest.sh_ok("id -u").await;
    assert_eq!(
        summary["privilege"]["root"],
        uid.trim() == "0",
        "probe privilege must match the guest: uid {uid:?}, probe {probe}"
    );
}
