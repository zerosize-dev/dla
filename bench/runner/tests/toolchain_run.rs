//! Fixture toolchains: the validator rules, and `dla-bench run` giving the child the named
//! tool and recording the versions it saw. Stand-ins for `claude` and for the Node release.
#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

use dla_bench::validate::validate_dir;

fn write(p: &Path, text: &str) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn script(p: &Path, body: &str) {
    write(p, &format!("#!/bin/sh\n{body}\n"));
    fs::set_permissions(p, fs::Permissions::from_mode(0o755)).unwrap();
}

fn lines(prefix: &str, n: usize) -> String {
    (0..n).map(|i| format!("{prefix}_{i} = {i}\n")).collect()
}

/// A valid fixture of three Python files; `toolchain` is the `[toolchain]` text and
/// `version` the schema version.
fn fixture(root: &Path, id: &str, version: u32, runtime: &str, toolchain: &str) {
    let d = root.join(id);
    write(&d.join("README.md"), "readme\n");
    let (ext, fill) = match runtime {
        "go" => ("go", false),
        "node" => ("js", false),
        _ => ("py", true),
    };
    for f in ["a", "b", "c"] {
        let body = if fill {
            lines(f, 60)
        } else {
            format!("// {f}\n").repeat(60)
        };
        write(&d.join(format!("project/src/{f}.{ext}")), &body);
    }
    write(
        &d.join("expected.toml"),
        &format!(
            "schema_version = {version}\nid = \"{id}\"\nruntime = \"{runtime}\"\ncategory = \"non-leak\"\n\
             dynamic_confirmation = true\n{toolchain}\n[task]\nprompt = \"The service grows.\"\n\
             [[non_leak]]\nid = \"N1\"\ncause = \"N-WARM\"\ndiscriminator = \"d\"\n"
        ),
    );
}

fn problems(root: &Path) -> Vec<String> {
    validate_dir(root, &root.join("no-schemas")).unwrap().1
}

fn has(p: &[String], needle: &str) -> bool {
    p.iter().any(|l| l.contains(needle))
}

#[test]
fn the_validator_accepts_a_toolchain_on_schema_version_2_and_refuses_the_rest() {
    let t = tempfile::tempdir().unwrap();
    fixture(t.path(), "ok", 2, "node", "[toolchain]\nnode = \"22.12.0\"");
    assert_eq!(problems(t.path()), Vec::<String>::new());

    fixture(
        t.path(),
        "old-version",
        1,
        "node",
        "[toolchain]\nnode = \"22.12.0\"",
    );
    fixture(t.path(), "no-table", 2, "node", "");
    fixture(t.path(), "range", 2, "node", "[toolchain]\nnode = \"22\"");
    fixture(
        t.path(),
        "wrong-runtime",
        2,
        "node",
        "[toolchain]\ngo = \"1.22.12\"",
    );
    fixture(
        t.path(),
        "unknown-tool",
        2,
        "node",
        "[toolchain]\nruby = \"3.3.0\"",
    );
    fixture(t.path(), "future", 3, "node", "");
    let p = problems(t.path());
    assert!(
        has(&p, "old-version: [toolchain] needs schema_version = 2"),
        "{p:?}"
    );
    assert!(
        has(
            &p,
            "no-table: schema_version 2 is for fixtures with a [toolchain]"
        ),
        "{p:?}"
    );
    assert!(
        has(
            &p,
            "range: [toolchain] node = \"22\" must be an exact version"
        ),
        "{p:?}"
    );
    assert!(
        has(
            &p,
            "wrong-runtime: [toolchain] go is set but the fixture has no Go runtime"
        ),
        "{p:?}"
    );
    assert!(has(&p, "unknown-tool: expected.toml schema"), "{p:?}");
    assert!(has(&p, "future: expected.toml schema"), "{p:?}");
    assert!(!has(&p, "ok:"), "{p:?}");
}

#[test]
fn go_mod_may_not_need_a_newer_go_than_the_fixture_names() {
    let t = tempfile::tempdir().unwrap();
    fixture(t.path(), "before", 2, "go", "[toolchain]\ngo = \"1.22.12\"");
    write(
        &t.path().join("before/project/go.mod"),
        "module m\n\ngo 1.22\n",
    );
    fixture(t.path(), "after", 2, "go", "[toolchain]\ngo = \"1.23.12\"");
    write(
        &t.path().join("after/project/go.mod"),
        "module m\n\ngo 1.23\n",
    );
    fixture(t.path(), "broken", 2, "go", "[toolchain]\ngo = \"1.22.12\"");
    write(
        &t.path().join("broken/project/go.mod"),
        "module m\n\ngo 1.23\n",
    );
    fixture(
        t.path(),
        "patched",
        2,
        "go",
        "[toolchain]\ngo = \"1.22.12\"",
    );
    write(
        &t.path().join("patched/project/go.mod"),
        "module m\n\ngo 1.22.13\n",
    );
    let p = problems(t.path());
    assert!(
        has(
            &p,
            "broken: go.mod asks for go 1.23 but [toolchain] go is 1.22.12"
        ),
        "{p:?}"
    );
    assert!(
        has(
            &p,
            "patched: go.mod asks for go 1.22.13 but [toolchain] go is 1.22.12"
        ),
        "{p:?}"
    );
    assert!(!has(&p, "before:") && !has(&p, "after:"), "{p:?}");
}

#[test]
fn the_generated_schema_names_the_toolchain_table() {
    let schema: serde_json::Value =
        serde_json::from_str(&dla_bench::schema_cmd::expected_schema()).unwrap();
    assert!(schema["properties"]["toolchain"].is_object());
    assert_eq!(schema["properties"]["schema_version"]["maximum"], 2);
    let run: serde_json::Value = serde_json::from_str(
        &dla_bench::schema_cmd::all()
            .into_iter()
            .find(|(n, _)| n == "run.schema.json")
            .unwrap()
            .1,
    )
    .unwrap();
    assert!(run["properties"]["toolchain"].is_object());
}

// ------------------------------------------------------------------ run

const FAKE: &str = r#"
echo called >> "$FAKE_LOG"
cfg="$CLAUDE_CONFIG_DIR"
command -v node > "$cfg/node-path.txt"
node --version > "$cfg/node-version.txt" 2>&1
echo "$GOTOOLCHAIN" > "$cfg/gotoolchain.txt"
echo '{"type":"system","subtype":"init","model":"fake-model"}'
echo '{"findings":[]}' > findings.json
echo '{"type":"result","total_cost_usd":0.1,"num_turns":1,"usage":{"input_tokens":1,"output_tokens":2}}'
"#;

struct Env {
    _t: tempfile::TempDir,
    root: PathBuf,
}

impl Env {
    fn new() -> Env {
        let t = tempfile::Builder::new()
            .prefix("toolchain-run-")
            .tempdir_in(env!("CARGO_TARGET_TMPDIR"))
            .unwrap();
        let root = t.path().to_path_buf();
        script(&root.join("claude"), FAKE);
        fs::create_dir_all(root.join("repo")).unwrap();
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(&root)
            .status()
            .unwrap();
        Env { _t: t, root }
    }

    /// A Node release `v<version>` whose `node` prints its version.
    fn node_release(&self, version: &str) {
        let stem = format!("node-v{version}-{}", node_platform());
        script(
            &self.root.join("dist-src").join(&stem).join("bin/node"),
            &format!("echo v{version}"),
        );
        let dir = self.root.join(format!("dist/v{version}"));
        fs::create_dir_all(&dir).unwrap();
        let tarball = dir.join(format!("{stem}.tar.gz"));
        assert!(
            Command::new("tar")
                .arg("-czf")
                .arg(&tarball)
                .arg("-C")
                .arg(self.root.join("dist-src"))
                .arg(&stem)
                .status()
                .unwrap()
                .success()
        );
        // The checksum of the tarball, as `sha256sum` prints it.
        let out = Command::new("sha256sum").arg(&tarball).output();
        let sum = match out {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
                .split_whitespace()
                .next()
                .unwrap()
                .to_owned(),
            _ => {
                let o = Command::new("shasum")
                    .args(["-a", "256"])
                    .arg(&tarball)
                    .output()
                    .unwrap();
                String::from_utf8_lossy(&o.stdout)
                    .split_whitespace()
                    .next()
                    .unwrap()
                    .to_owned()
            }
        };
        write(
            &dir.join("SHASUMS256.txt"),
            &format!("{sum}  {stem}.tar.gz\n"),
        );
    }

    fn run(&self, fixtures: &str) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_dla-bench"))
            .arg("run")
            .arg("--fixtures-dir")
            .arg(self.root.join("fx"))
            .arg("--out")
            .arg(self.root.join("out/res"))
            .arg("--work-root")
            .arg(self.root.join("work"))
            .arg("--claude-bin")
            .arg(self.root.join("claude"))
            .arg("--repo-root")
            .arg(self.root.join("repo"))
            .args([
                "--ablation",
                "no-skill",
                "--runs",
                "1",
                "--keep-work",
                "--fixtures",
                fixtures,
            ])
            .env("FAKE_LOG", self.root.join("calls.log"))
            .env(
                "DLA_BENCH_NODE_DIST",
                format!("file://{}", self.root.join("dist").display()),
            )
            .env("CLAUDE_CODE_SESSION_ID", "parent-session")
            .output()
            .unwrap()
    }

    fn run_json(&self, fx: &str) -> serde_json::Value {
        let p = self.root.join(format!(
            "out/res/runs/no-skill/archetypes-on/{fx}/1/run.json"
        ));
        serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
    }

    fn config(&self, fx: &str) -> PathBuf {
        self.root
            .join("work")
            .join(dla_bench::run::work_key("res", "", "", "", 0))
            .join(dla_bench::run::work_key("res", "no-skill", "on", fx, 1))
            .join("config")
    }
}

/// The platform part of the tarball name the runner asks for on this host (`linux-x64`,
/// `darwin-arm64`).
fn node_platform() -> String {
    dla_bench::fixture_toolchain::node_platform(std::env::consts::OS, std::env::consts::ARCH)
        .expect("a host with a Node toolchain")
}

/// True on a host the runner can unpack a Node tarball for; the Node tests skip elsewhere.
fn node_host() -> bool {
    dla_bench::fixture_toolchain::node_platform(std::env::consts::OS, std::env::consts::ARCH)
        .is_ok()
}

#[test]
fn the_child_gets_the_named_node_first_on_path_and_run_json_records_it() {
    if !node_host() {
        return;
    }
    let e = Env::new();
    e.node_release("22.12.0");
    fixture(
        &e.root.join("fx"),
        "pair-before",
        2,
        "node",
        "[toolchain]\nnode = \"22.12.0\"",
    );
    let out = e.run("pair-before");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let cfg = e.config("pair-before");
    let seen = fs::read_to_string(cfg.join("node-path.txt")).unwrap();
    assert!(
        seen.trim().ends_with(&format!(
            "toolchains/node/node-v22.12.0-{}/bin/node",
            node_platform()
        )),
        "{seen}"
    );
    assert_eq!(
        fs::read_to_string(cfg.join("node-version.txt"))
            .unwrap()
            .trim(),
        "v22.12.0"
    );
    let rj = e.run_json("pair-before");
    assert_eq!(rj["toolchain"]["node"], "22.12.0", "{rj}");
}

#[test]
fn a_fixture_without_a_toolchain_runs_on_the_host_tools_and_still_records_them() {
    let e = Env::new();
    fixture(&e.root.join("fx"), "plain", 1, "node", "");
    let out = e.run("plain");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let seen = fs::read_to_string(e.config("plain").join("node-path.txt")).unwrap();
    assert!(!seen.contains("toolchains/"), "{seen}");
    // No other change to the child's environment.
    assert_eq!(
        fs::read_to_string(e.config("plain").join("gotoolchain.txt"))
            .unwrap()
            .trim(),
        "auto"
    );
    // The host's node, when it has one, is what run.json says.
    let rj = e.run_json("plain");
    let host = Command::new("node").arg("--version").output();
    match host {
        Ok(o) if o.status.success() => {
            let v = String::from_utf8_lossy(&o.stdout)
                .trim()
                .trim_start_matches('v')
                .to_owned();
            assert_eq!(rj["toolchain"]["node"], v.as_str(), "{rj}");
        }
        _ => assert!(rj.get("toolchain").is_none(), "{rj}"),
    }
}

#[test]
fn an_unavailable_release_stops_the_run_before_any_child_starts() {
    if !node_host() {
        return;
    }
    let e = Env::new();
    fixture(
        &e.root.join("fx"),
        "pair-after",
        2,
        "node",
        "[toolchain]\nnode = \"24.13.1\"",
    );
    fixture(&e.root.join("fx"), "plain", 1, "node", "");
    let out = e.run("plain,pair-after");
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("fixture pair-after") && err.contains("24.13.1"),
        "{err}"
    );
    assert!(
        !e.root.join("calls.log").exists(),
        "no child may start on the wrong tool"
    );
}
