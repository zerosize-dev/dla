//! Coverage gap with really absent tools.
//!
//! The host-side benchmark hides tools with `--without`. Here a stock image
//! lacks them for real: `debian:bookworm-slim` has no Go toolchain, no
//! staticcheck and no golangci-lint.
//!
//! The test puts a small Go module in an empty project and runs `dla probe`. It
//! asserts that:
//! - the Go runtime is detected from `go.mod` and `main.go`;
//! - `go:staticcheck`, `go:golangci-lint` and `go:govet` are listed as missing;
//! - the Go worker's `coverage.json` records `staticcheck` and `golangci-lint`
//!   as `missing`, each with the install hint `dla install <name>`, and the
//!   tier 3 row `bcc-memleak` as `blocked`.
//!
//! A report built from this run must therefore say what was not looked at.

#![cfg(feature = "sandbox-it")]

use dla_it::vm::{DEBIAN, Guest};
use dla_it::{coverage_hint, coverage_status, require_vm};
use serde_json::Value;

#[tokio::test]
async fn missing_go_tools_become_coverage_rows() {
    require_vm!();
    let guest = Guest::boot(DEBIAN, 512).await.expect("boot");
    guest.install_dla().await;
    guest
        .sh_ok(
            "mkdir -p /srv/gap && cd /srv/gap \
             && printf 'module gap\\n\\ngo 1.22\\n' > go.mod \
             && printf 'package main\\n\\nfunc main() {}\\n' > main.go",
        )
        .await;

    let init = guest.dla_json("/srv/gap", "run init").await;
    let run_id = init["run_id"].as_str().expect("run_id").to_string();
    let probe = guest.dla_json("/srv/gap", "probe").await;
    let summary = &probe["summary"];

    let runtimes: Vec<&str> = summary["runtimes"]
        .as_array()
        .expect("runtimes")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert_eq!(runtimes, ["go"], "{probe}");
    let missing: Vec<&str> = summary["tools_missing"]
        .as_array()
        .expect("tools_missing")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for tool in ["go:staticcheck", "go:golangci-lint", "go:govet"] {
        assert!(missing.contains(&tool), "{tool} must be missing: {probe}");
    }

    let text = guest
        .sh_ok(&format!("cat /srv/gap/.dla/{run_id}/coverage.json"))
        .await;
    let coverage: Value = serde_json::from_str(&text).expect("coverage.json");
    for backend in ["staticcheck", "golangci-lint"] {
        assert_eq!(
            coverage_status(&coverage, "go", backend).as_deref(),
            Some("missing"),
            "{backend}: {text}"
        );
        assert_eq!(
            coverage_hint(&coverage, "go", backend),
            Some(format!("dla install {backend}")),
            "{backend}: {text}"
        );
    }
    assert_eq!(
        coverage_status(&coverage, "go", "bcc-memleak").as_deref(),
        Some("blocked"),
        "{text}"
    );
}
