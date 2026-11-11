//! VM helpers over rightsize (microsandbox backend). Needs the `sandbox-it` feature.
//!
//! Limits of the backend that shape these helpers: `exec` has no environment,
//! working directory, stdin or timeout argument and stops after 120 seconds, so
//! every command goes through `sh -c` and must be short. Longer work belongs in
//! a background job that later tests will poll.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rightsize::backend::BackendProvider;
use rightsize::{Container, ContainerGuard, ExecResult, RightsizeError, Wait};
use rightsize_msb::MsbBackendProvider;
use serde_json::Value;

use crate::{json_line, shell_quote};

/// Where the plugin tree lives in the guest.
pub const GUEST_ROOT: &str = "/opt/dla";
/// The installed binary in the guest.
pub const GUEST_DLA: &str = "/opt/dla/bin/dla";
/// The plugin data directory in the guest (analyzers install here).
pub const GUEST_DATA_DIR: &str = "/var/dla-data";

/// The stock image used when a test needs no runtime.
pub const DEBIAN: &str = "debian:bookworm-slim";

fn env_set(name: &str) -> bool {
    std::env::var(name).is_ok_and(|v| !v.is_empty() && v != "0")
}

/// Why a test cannot run its VM part, or `None` when it can.
/// Registers only the msb provider, so there is no container fallback.
pub fn skip_reason() -> Option<String> {
    if env_set("DLA_IT_SKIP_VM") {
        return Some("DLA_IT_SKIP_VM is set".into());
    }
    rightsize::backends::register_provider(Box::new(MsbBackendProvider));
    let provider = MsbBackendProvider;
    if provider.is_supported() {
        return None;
    }
    let reason = provider.unsupported_reason();
    if env_set("DLA_IT_REQUIRE_VM") {
        panic!("DLA_IT_REQUIRE_VM is set but this host cannot boot a microVM: {reason}");
    }
    Some(reason)
}

/// Skips the calling test (prints a notice, returns) when no VM can be booted.
#[macro_export]
macro_rules! require_vm {
    () => {
        if let Some(reason) = $crate::vm::skip_reason() {
            eprintln!("skipping: {reason}");
            return;
        }
    };
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .components()
        .collect()
}

/// The static `dla` binary to copy into guests.
pub fn dla_binary() -> PathBuf {
    let dir = std::env::var_os("DLA_IT_BIN_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("target/x86_64-unknown-linux-musl/release"));
    let bin = dir.join("dla");
    assert!(
        bin.is_file(),
        "{} not found: build it with `cargo build --release --locked -p dla-cli --target x86_64-unknown-linux-musl` or set DLA_IT_BIN_DIR",
        bin.display()
    );
    bin
}

/// Writes `value` as pretty JSON to `$DLA_IT_OUT/<name>`. No-op when unset.
pub fn write_out(name: &str, value: &Value) {
    let Some(dir) = std::env::var_os("DLA_IT_OUT") else {
        return;
    };
    let dir = PathBuf::from(dir);
    std::fs::create_dir_all(&dir).expect("create DLA_IT_OUT");
    let text = serde_json::to_string_pretty(value).expect("serialize");
    std::fs::write(dir.join(name), text + "\n").expect("write fact file");
}

/// A running guest. Isolation is always required: a backend without hardware
/// isolation makes `start` fail before anything is created.
pub struct Guest {
    pub guard: ContainerGuard,
}

impl Guest {
    /// Boots `image` and keeps it alive with `sleep`. `memory_mb` sizes the guest.
    pub async fn boot(image: &str, memory_mb: u64) -> Result<Guest, RightsizeError> {
        let guard = Container::new(image)
            .with_command(&["sleep", "86400"])
            .with_memory_limit(memory_mb)
            .require_isolation(true)
            // No port to wait for: any log line (or none, after the grace period).
            .waiting_for(
                Wait::for_log_message(".*", 0).with_startup_timeout(Duration::from_secs(600)),
            )
            .start()
            .await?;
        Ok(Guest { guard })
    }

    /// Runs `script` with `sh -c`. Does not check the exit code.
    pub async fn sh(&self, script: &str) -> ExecResult {
        self.guard
            .exec(&["sh", "-c", script])
            .await
            .unwrap_or_else(|e| panic!("exec failed for {script:?}: {e}"))
    }

    /// Runs `script` and asserts exit code 0. Returns stdout.
    pub async fn sh_ok(&self, script: &str) -> String {
        let r = self.sh(script).await;
        assert_eq!(
            r.exit_code, 0,
            "command failed: {script}\nstdout:\n{}\nstderr:\n{}",
            r.stdout, r.stderr
        );
        r.stdout
    }

    /// Copies the static `dla` and the plugin `data/` directory into the guest.
    pub async fn install_dla(&self) {
        let root = workspace_root();
        self.guard
            .copy_file_to_container(dla_binary(), GUEST_DLA)
            .await
            .expect("copy dla");
        self.guard
            .copy_file_to_container(root.join("data"), &format!("{GUEST_ROOT}/data"))
            .await
            .expect("copy data");
        self.sh_ok(&format!(
            "chmod +x {GUEST_DLA} && mkdir -p {GUEST_DATA_DIR}"
        ))
        .await;
    }

    /// Runs `dla <args>` in `cwd` with the guest environment and returns the result.
    pub async fn dla(&self, cwd: &str, args: &str) -> ExecResult {
        self.sh(&format!(
            "cd {cwd} && DLA_PLUGIN_ROOT={root} DLA_DATA_DIR={data} {bin} {args}",
            cwd = shell_quote(cwd),
            root = GUEST_ROOT,
            data = GUEST_DATA_DIR,
            bin = GUEST_DLA,
        ))
        .await
    }

    /// Runs `dla <args>`, asserts exit code 0 and returns the last JSON line.
    pub async fn dla_json(&self, cwd: &str, args: &str) -> Value {
        let r = self.dla(cwd, args).await;
        assert_eq!(
            r.exit_code, 0,
            "dla {args} failed\nstdout:\n{}\nstderr:\n{}",
            r.stdout, r.stderr
        );
        json_line(&r.stdout).unwrap_or_else(|e| panic!("dla {args}: {e}"))
    }
}
