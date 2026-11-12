//! `dla-bench dynamic` runs a fixture's `dla` commands on the tools its `[toolchain]` names,
//! as `dla-bench run` does. Stand-ins for the `dla` executable and for the Node release.
#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

fn write(p: &Path, text: &str) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn script(p: &Path, body: &str) {
    write(p, &format!("#!/bin/sh\n{body}\n"));
    fs::set_permissions(p, fs::Permissions::from_mode(0o755)).unwrap();
}

fn node_platform() -> Option<String> {
    dla_bench::fixture_toolchain::node_platform(std::env::consts::OS, std::env::consts::ARCH).ok()
}

/// A fixture the policy cannot drive (the stand-in `dla` fails), so only the environment
/// its commands saw is of interest.
fn fixture(root: &Path, id: &str, version: u32, toolchain: &str) {
    let d = root.join(id);
    write(&d.join("project/src/a.js"), "// a\n");
    write(
        &d.join("expected.toml"),
        &format!(
            "schema_version = {version}\nid = \"{id}\"\nruntime = \"node\"\ncategory = \"non-leak\"\n\
             dynamic_confirmation = true\n{toolchain}\n[task]\nprompt = \"p\"\n\
             [[non_leak]]\nid = \"N1\"\ncause = \"N-WARM\"\ndiscriminator = \"d\"\n"
        ),
    );
}

fn node_release(root: &Path, platform: &str, version: &str) {
    let stem = format!("node-v{version}-{platform}");
    script(
        &root.join("dist-src").join(&stem).join("bin/node"),
        &format!("echo v{version}"),
    );
    let dir = root.join(format!("dist/v{version}"));
    fs::create_dir_all(&dir).unwrap();
    let tarball = dir.join(format!("{stem}.tar.gz"));
    assert!(
        Command::new("tar")
            .arg("-czf")
            .arg(&tarball)
            .arg("-C")
            .arg(root.join("dist-src"))
            .arg(&stem)
            .status()
            .unwrap()
            .success()
    );
    let out = Command::new("sha256sum").arg(&tarball).output();
    let sum = match out {
        Ok(o) if o.status.success() => o.stdout,
        _ => {
            Command::new("shasum")
                .args(["-a", "256"])
                .arg(&tarball)
                .output()
                .unwrap()
                .stdout
        }
    };
    let sum = String::from_utf8_lossy(&sum)
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();
    write(
        &dir.join("SHASUMS256.txt"),
        &format!("{sum}  {stem}.tar.gz\n"),
    );
}

struct Env {
    _t: tempfile::TempDir,
    root: PathBuf,
}

impl Env {
    fn new() -> Env {
        let t = tempfile::Builder::new()
            .prefix("dynamic-toolchain-")
            .tempdir_in(env!("CARGO_TARGET_TMPDIR"))
            .unwrap();
        let root = t.path().to_path_buf();
        // Records what the first command of every fixture saw, then fails.
        script(
            &root.join("dla"),
            r#"id=$(basename "$PWD")
{ command -v node; node --version 2>&1; echo "gotoolchain=$GOTOOLCHAIN"; } > "$SEEN_DIR/$id.txt"
echo '{"ok":false}'
exit 1"#,
        );
        fs::create_dir_all(root.join("seen")).unwrap();
        fs::create_dir_all(root.join("repo/data")).unwrap();
        Env { _t: t, root }
    }

    fn run(&self, only: &str) -> std::process::Output {
        self.run_with(only, &[])
    }

    fn run_with(&self, only: &str, extra: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_dla-bench"))
            .arg("dynamic")
            .arg("--fixtures")
            .arg(self.root.join("fx"))
            .arg("--dla-bin")
            .arg(self.root.join("dla"))
            .arg("--repo-root")
            .arg(self.root.join("repo"))
            .arg("--log-dir")
            .arg(self.root.join("logs"))
            .args(["--only", only, "--parallel", "1"])
            .args(extra)
            .env("SEEN_DIR", self.root.join("seen"))
            .env(
                "DLA_BENCH_NODE_DIST",
                format!("file://{}", self.root.join("dist").display()),
            )
            .output()
            .unwrap()
    }

    /// Like [`Env::run`] with the repository root given as a relative path, from `self.root`.
    fn run_relative(&self, only: &str) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_dla-bench"))
            .current_dir(&self.root)
            .arg("dynamic")
            .arg("--fixtures")
            .arg(self.root.join("fx"))
            .arg("--dla-bin")
            .arg(self.root.join("dla"))
            .args(["--repo-root", "repo"])
            .arg("--log-dir")
            .arg(self.root.join("logs"))
            .args(["--only", only, "--parallel", "1"])
            .env("SEEN_DIR", self.root.join("seen"))
            .env(
                "DLA_BENCH_NODE_DIST",
                format!("file://{}", self.root.join("dist").display()),
            )
            .output()
            .unwrap()
    }

    fn seen(&self, id: &str) -> String {
        fs::read_to_string(self.root.join(format!("seen/{id}.txt"))).unwrap()
    }
}

#[test]
fn dynamic_runs_the_dla_commands_on_the_tool_the_toolchain_names() {
    let Some(platform) = node_platform() else {
        return;
    };
    let e = Env::new();
    node_release(&e.root, &platform, "22.12.0");
    fixture(
        &e.root.join("fx"),
        "pair-before",
        2,
        "[toolchain]\nnode = \"22.12.0\"",
    );
    let out = e.run("pair-before");
    // The stand-in fails every command, so the run fails; the environment is what counts.
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("pair-before:"), "{err}");
    let seen = e.seen("pair-before");
    assert!(
        seen.contains(&format!(
            "toolchains/node/node-v22.12.0-{platform}/bin/node"
        )),
        "{seen}"
    );
    assert!(seen.contains("v22.12.0"), "{seen}");
}

#[test]
fn dynamic_leaves_a_fixture_without_a_toolchain_on_the_host_tools() {
    let e = Env::new();
    fixture(&e.root.join("fx"), "plain", 1, "");
    e.run("plain");
    let seen = e.seen("plain");
    assert!(!seen.contains("toolchains/"), "{seen}");
    assert!(
        seen.contains("gotoolchain=\n") || seen.ends_with("gotoolchain="),
        "{seen}"
    );
}

#[test]
fn dynamic_stops_before_any_command_when_a_release_cannot_be_provided() {
    if node_platform().is_none() {
        return;
    }
    let e = Env::new();
    fixture(
        &e.root.join("fx"),
        "pair-after",
        2,
        "[toolchain]\nnode = \"24.13.1\"",
    );
    fixture(&e.root.join("fx"), "plain", 1, "");
    let out = e.run("p");
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("fixture pair-after") && err.contains("24.13.1"),
        "{err}"
    );
    assert_eq!(fs::read_dir(e.root.join("seen")).unwrap().count(), 0);
}

#[test]
fn without_refuses_a_fixture_whose_toolchain_provides_the_hidden_tool() {
    let Some(platform) = node_platform() else {
        return;
    };
    let e = Env::new();
    node_release(&e.root, &platform, "22.12.0");
    fixture(
        &e.root.join("fx"),
        "pair-before",
        2,
        "[toolchain]\nnode = \"22.12.0\"",
    );
    let out = e.run_with("pair-before", &["--without", "node"]);
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("--without node") && err.contains("fixture pair-before"),
        "{err}"
    );
    assert_eq!(fs::read_dir(e.root.join("seen")).unwrap().count(), 0);
    // A tool the toolchain does not provide can still be hidden.
    let out = e.run_with("pair-before", &["--without", "valgrind"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(!err.contains("provides it"), "{err}");
}

#[test]
fn a_second_dynamic_run_reuses_the_toolchains_of_the_first() {
    let Some(platform) = node_platform() else {
        return;
    };
    let e = Env::new();
    node_release(&e.root, &platform, "22.12.0");
    fixture(
        &e.root.join("fx"),
        "pair-before",
        2,
        "[toolchain]\nnode = \"22.12.0\"",
    );
    e.run("pair-before");
    assert!(
        e.root
            .join(format!(
                "repo/target/dla-bench-toolchains/node/node-v22.12.0-{platform}/bin/node"
            ))
            .is_file(),
        "the release is unpacked next to the repository's target directory"
    );
    // The mirror is gone: an on run followed by an off run must not fetch again.
    fs::remove_dir_all(e.root.join("dist")).unwrap();
    let out = e.run_with("pair-before", &["--archetypes", "off"]);
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains("could not") && !err.contains("download"),
        "{err}"
    );
    let seen = e.seen("pair-before");
    assert!(seen.contains("v22.12.0"), "{seen}");
}

#[test]
fn a_relative_repo_root_still_gives_the_fixture_its_toolchain() {
    let Some(platform) = node_platform() else {
        return;
    };
    let e = Env::new();
    node_release(&e.root, &platform, "22.12.0");
    fixture(
        &e.root.join("fx"),
        "pair-before",
        2,
        "[toolchain]\nnode = \"22.12.0\"",
    );
    e.run_relative("pair-before");
    // Every `dla` call runs in the project directory: a relative store would name nothing
    // there and the host's node would run.
    let seen = e.seen("pair-before");
    let first = seen.lines().next().unwrap();
    assert!(
        first.starts_with('/') && first.ends_with(&format!("node-v22.12.0-{platform}/bin/node")),
        "{seen}"
    );
    assert!(seen.contains("v22.12.0"), "{seen}");
}
