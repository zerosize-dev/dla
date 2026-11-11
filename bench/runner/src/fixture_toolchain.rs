//! The `[toolchain]` table of a fixture: the exact tool versions its children run on.
//!
//! A fixture that depends on a version of a tool (the Go timer change of 1.23, a Node or a
//! Python release) names it in `expected.toml`; `dla-bench run` fetches it once on the host
//! (the children have no network), puts it first on the child's `PATH` and records the
//! versions the child really saw in `run.json`.

use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    Res,
    model::{Expected, Runtime},
    run::{host_go_modcache, link_toolchain, prefetch_go_toolchain_with},
};

/// `schema_version` of an `expected.toml` with a `[toolchain]` table.
pub const TOOLCHAIN_SCHEMA_VERSION: u32 = 2;

/// Where the Node release tarballs come from.
pub const NODE_DIST: &str = "https://nodejs.org/dist";

/// `[toolchain]`: exact versions, such as `go = "1.22.12"` or `python = "3.14.4"`.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Toolchain {
    /// Go release, `major.minor.patch` (`1.22.12`). The child runs `GOTOOLCHAIN=go<version>`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub go: Option<String>,
    /// Node release, `major.minor.patch` (`22.12.0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    /// CPython release, `major.minor.patch` (`3.14.4`), installed with `uv python install`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python: Option<String>,
    /// JDK release, `17` or `21.0.4`; a JDK of that version must be installed on the host.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java: Option<String>,
}

impl Toolchain {
    /// True when no tool is named.
    pub fn is_empty(&self) -> bool {
        self.go.is_none() && self.node.is_none() && self.python.is_none() && self.java.is_none()
    }
}

/// The `toolchain` block of `run.json`: the exact versions the child saw.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Default, PartialEq, Eq)]
pub struct RunToolchain {
    /// Go, from `go env GOVERSION` without the `go` prefix (`1.22.12`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub go: Option<String>,
    /// Node, from `node --version` without the `v` (`22.12.0`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    /// Python, from `python3 --version` (`3.14.4`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub python: Option<String>,
    /// Java, from `java -version` (`21.0.12`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub java: Option<String>,
    /// Rust, from `rustc --version` (`1.99.1`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rust: Option<String>,
}

impl RunToolchain {
    /// True when no version could be read.
    pub fn is_empty(&self) -> bool {
        self == &RunToolchain::default()
    }
}

// ------------------------------------------------------------------ validation

fn three_part(v: &str) -> bool {
    let parts: Vec<&str> = v.split('.').collect();
    parts.len() == 3 && parts.iter().all(|p| is_number(p))
}

fn is_number(p: &str) -> bool {
    !p.is_empty() && p.len() <= 6 && p.bytes().all(|b| b.is_ascii_digit())
}

fn java_version(v: &str) -> bool {
    let parts: Vec<&str> = v.split('.').collect();
    (1..=4).contains(&parts.len()) && parts.iter().all(|p| is_number(p))
}

/// The runtimes a fixture holds code for.
fn runtimes_of(e: &Expected) -> Vec<Runtime> {
    let mut rt = vec![e.runtime];
    if let Some(more) = &e.runtimes {
        rt.extend(more.iter().copied());
    }
    rt
}

/// The release named by the `go` line of a `go.mod`, with its patch level (`go 1.22` and
/// `go 1.22.0` both give `(1, 22, 0)`). Go refuses a toolchain below it.
pub fn go_mod_release(go_mod: &str) -> Option<(u64, u64, u64)> {
    for line in go_mod.lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        let Some(rest) = line
            .strip_prefix("go ")
            .or_else(|| line.strip_prefix("go\t"))
        else {
            continue;
        };
        let mut it = rest.trim().split('.');
        let major = it.next()?.parse().ok()?;
        let minor = it.next()?.parse().ok()?;
        let patch = it.next().map_or(Some(0), |p| p.parse().ok())?;
        return Some((major, minor, patch));
    }
    None
}

/// Rules of the `[toolchain]` table and `schema_version`; the problems are appended to `out`.
/// `go_mod` is the text of the fixture's `project/go.mod`, when it has one.
pub fn check(e: &Expected, go_mod: Option<&str>, out: &mut Vec<String>) {
    let Some(tc) = &e.toolchain else {
        if e.schema_version == TOOLCHAIN_SCHEMA_VERSION {
            out.push(format!(
                "schema_version {TOOLCHAIN_SCHEMA_VERSION} is for fixtures with a [toolchain] table; this one has none, use schema_version 1"
            ));
        }
        return;
    };
    if e.schema_version != TOOLCHAIN_SCHEMA_VERSION {
        out.push(format!(
            "[toolchain] needs schema_version = {TOOLCHAIN_SCHEMA_VERSION} (found {})",
            e.schema_version
        ));
    }
    if tc.is_empty() {
        out.push(
            "[toolchain] names no tool; give go, node, python or java, or drop the table"
                .to_owned(),
        );
    }
    let runtimes = runtimes_of(e);
    let mut need =
        |field: &str, version: &Option<String>, ok: fn(&str) -> bool, shape: &str, rt: Runtime| {
            let Some(v) = version else { return };
            if !ok(v) {
                out.push(format!(
                    "[toolchain] {field} = {v:?} must be an exact version such as {shape}"
                ));
            }
            if !runtimes.contains(&rt) {
                out.push(format!(
                    "[toolchain] {field} is set but the fixture has no {rt:?} runtime"
                ));
            }
        };
    need("go", &tc.go, three_part, "\"1.22.12\"", Runtime::Go);
    need("node", &tc.node, three_part, "\"22.12.0\"", Runtime::Node);
    need(
        "python",
        &tc.python,
        three_part,
        "\"3.14.4\"",
        Runtime::Python,
    );
    need(
        "java",
        &tc.java,
        java_version,
        "\"17\" or \"21.0.4\"",
        Runtime::Jvm,
    );
    if let (Some(go), Some(text)) = (&tc.go, go_mod)
        && three_part(go)
        && let Some(want) = go_mod_release(text)
    {
        let mut it = go.split('.').filter_map(|p| p.parse::<u64>().ok());
        let have = (
            it.next().unwrap_or(0),
            it.next().unwrap_or(0),
            it.next().unwrap_or(0),
        );
        if want > have {
            let patch = if want.2 > 0 {
                format!(".{}", want.2)
            } else {
                String::new()
            };
            out.push(format!(
                "go.mod asks for go {}.{}{patch} but [toolchain] go is {go}; the build would stop",
                want.0, want.1
            ));
        }
    }
}

// ------------------------------------------------------------------ providing

/// The commands and sources the runner uses to provide a toolchain. [`Host::system`] is the
/// real host; tests put fakes here.
#[derive(Debug, Clone)]
pub struct Host {
    /// The `go` command.
    pub go: PathBuf,
    /// The `uv` command.
    pub uv: PathBuf,
    /// The `curl` command.
    pub curl: PathBuf,
    /// The `tar` command.
    pub tar: PathBuf,
    /// Base URL of the Node distribution; `file://` works.
    pub node_dist: String,
    /// The host Go module cache; `None` asks `go env GOMODCACHE`.
    pub go_modcache: Option<PathBuf>,
    /// Directories whose children are JDK homes (`/usr/lib/jvm`).
    pub jvm_dirs: Vec<PathBuf>,
    /// `std::env::consts::OS`.
    pub os: String,
    /// `std::env::consts::ARCH`.
    pub arch: String,
}

impl Host {
    /// The real host. `DLA_BENCH_NODE_DIST` replaces the Node distribution URL (a mirror,
    /// or a `file://` directory in tests).
    pub fn system() -> Host {
        Host {
            go: "go".into(),
            uv: "uv".into(),
            curl: "curl".into(),
            tar: "tar".into(),
            node_dist: std::env::var("DLA_BENCH_NODE_DIST")
                .ok()
                .filter(|v| !v.is_empty())
                .unwrap_or_else(|| NODE_DIST.to_owned()),
            go_modcache: None,
            jvm_dirs: vec!["/usr/lib/jvm".into(), "/opt/java".into(), "/opt/jdk".into()],
            os: std::env::consts::OS.to_owned(),
            arch: std::env::consts::ARCH.to_owned(),
        }
    }
}

/// What the child needs for its toolchain.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Provided {
    /// Directories put first on the child's `PATH`, in order.
    pub path_first: Vec<PathBuf>,
    /// Environment variables set for the child.
    pub env: Vec<(String, String)>,
    /// Unpacked Go toolchains to link into the child's module cache.
    pub go_toolchain_dirs: Vec<PathBuf>,
}

impl Provided {
    /// Sets the variables and puts the directories first on `PATH`.
    pub fn apply(&self, env: &mut Vec<(String, String)>) {
        for (k, v) in &self.env {
            env.retain(|(n, _)| n != k);
            env.push((k.clone(), v.clone()));
        }
        if self.path_first.is_empty() {
            return;
        }
        let old = env
            .iter()
            .find(|(n, _)| n == "PATH")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let mut dirs: Vec<PathBuf> = self.path_first.clone();
        dirs.extend(std::env::split_paths(&old));
        let joined = std::env::join_paths(dirs)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or(old);
        env.retain(|(n, _)| n != "PATH");
        env.push(("PATH".to_owned(), joined));
    }

    /// Links the Go toolchains into a child's cache (`<cache>/gopath/pkg/mod`).
    pub fn link_go(&self, cache: &Path) {
        for d in &self.go_toolchain_dirs {
            link_toolchain(d, cache);
        }
    }
}

/// Fetches what every fixture's `[toolchain]` names, once per distinct request. An error
/// names the fixture: a version pair must never run on the wrong tool.
pub fn provide_all(
    host: &Host,
    store: &Path,
    fixtures: &[(String, Toolchain)],
) -> Res<BTreeMap<String, Provided>> {
    let mut by_request: Vec<(Toolchain, Provided)> = Vec::new();
    let mut out = BTreeMap::new();
    for (id, tc) in fixtures {
        if tc.is_empty() {
            continue;
        }
        let provided = match by_request.iter().find(|(t, _)| t == tc) {
            Some((_, p)) => p.clone(),
            None => {
                let p = provide(host, store, tc).map_err(|e| format!("fixture {id}: {e}"))?;
                by_request.push((tc.clone(), p.clone()));
                p
            }
        };
        out.insert(id.clone(), provided);
    }
    Ok(out)
}

/// Fetches one `[toolchain]` table into `store`.
pub fn provide(host: &Host, store: &Path, tc: &Toolchain) -> Res<Provided> {
    let mut p = Provided::default();
    if let Some(v) = &tc.go {
        provide_go(host, v, &mut p)?;
    }
    if let Some(v) = &tc.node {
        provide_node(host, store, v, &mut p)?;
    }
    if let Some(v) = &tc.python {
        provide_python(host, store, v, &mut p)?;
    }
    if let Some(v) = &tc.java {
        provide_java(host, v, &mut p)?;
    }
    Ok(p)
}

fn provide_go(host: &Host, version: &str, p: &mut Provided) -> Res<()> {
    let name = format!("go{version}");
    let cache = host
        .go_modcache
        .clone()
        .or_else(host_go_modcache)
        .ok_or("the Go module cache of the host is unknown (no go on PATH and no HOME)")?;
    let dir = prefetch_go_toolchain_with(&host.go, &cache, &name).ok_or_else(|| {
        format!("{name} could not be fetched with `GOTOOLCHAIN={name} go version`; the host needs network access to proxy.golang.org")
    })?;
    p.go_toolchain_dirs.push(dir);
    // The exact release, not `auto`: a go.mod line must not pick another one.
    p.env.push(("GOTOOLCHAIN".to_owned(), name));
    Ok(())
}

/// The platform part of a Node tarball name, `linux-x64`.
pub fn node_platform(os: &str, arch: &str) -> Res<String> {
    let os = match os {
        "linux" => "linux",
        "macos" => "darwin",
        o => return Err(format!("no Node toolchain for the operating system {o}")),
    };
    let arch = match arch {
        "x86_64" => "x64",
        "aarch64" => "arm64",
        a => return Err(format!("no Node toolchain for the architecture {a}")),
    };
    Ok(format!("{os}-{arch}"))
}

fn run_tool(cmd: &mut Command, what: &str) -> Res<std::process::Output> {
    let out = cmd
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("{what}: cannot start: {e}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(format!(
            "{what} failed ({}): {}",
            out.status,
            err.trim().lines().last().unwrap_or("")
        ));
    }
    Ok(out)
}

fn sha256_hex(path: &Path) -> Res<String> {
    let mut f = fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 16];
    loop {
        let n = f
            .read(&mut buf)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

fn provide_node(host: &Host, store: &Path, version: &str, p: &mut Provided) -> Res<()> {
    let platform = node_platform(&host.os, &host.arch)?;
    let stem = format!("node-v{version}-{platform}");
    let root = store.join("node");
    let bin = root.join(&stem).join("bin");
    if !bin.join("node").is_file() {
        let tarball = format!("{stem}.tar.gz");
        let base = format!("{}/v{version}", host.node_dist.trim_end_matches('/'));
        fs::create_dir_all(&root).map_err(|e| format!("{}: {e}", root.display()))?;
        let part = root.join(format!("{tarball}.part"));
        let sums = root.join(format!("SHASUMS256-{version}.txt"));
        for (url, to) in [
            (format!("{base}/{tarball}"), &part),
            (format!("{base}/SHASUMS256.txt"), &sums),
        ] {
            run_tool(
                Command::new(&host.curl)
                    .args(["-fsSL", "--retry", "2", "-o"])
                    .arg(to)
                    .arg(&url),
                &format!("download of {url}"),
            )?;
        }
        let listing = fs::read_to_string(&sums).map_err(|e| format!("{}: {e}", sums.display()))?;
        let want = listing
            .lines()
            .filter_map(|l| l.split_once("  "))
            .find(|(_, f)| f.trim() == tarball)
            .map(|(h, _)| h.trim().to_ascii_lowercase())
            .ok_or_else(|| {
                format!("SHASUMS256.txt of node v{version} has no line for {tarball}")
            })?;
        let have = sha256_hex(&part)?;
        if have != want {
            let _ = fs::remove_file(&part);
            return Err(format!(
                "{tarball}: sha256 {have} does not match the published {want}"
            ));
        }
        let tmp = root.join(format!(".unpack-{stem}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        fs::create_dir_all(&tmp).map_err(|e| format!("{}: {e}", tmp.display()))?;
        run_tool(
            Command::new(&host.tar)
                .arg("-xzf")
                .arg(&part)
                .arg("-C")
                .arg(&tmp),
            &format!("unpacking {tarball}"),
        )?;
        let _ = fs::remove_file(&part);
        let target = root.join(&stem);
        let _ = fs::remove_dir_all(&target);
        fs::rename(tmp.join(&stem), &target)
            .map_err(|e| format!("{tarball} does not unpack to {stem}/: {e}"))?;
        let _ = fs::remove_dir_all(&tmp);
        if !bin.join("node").is_file() {
            return Err(format!("{tarball} has no bin/node"));
        }
    }
    p.path_first.push(bin);
    Ok(())
}

fn provide_python(host: &Host, store: &Path, version: &str, p: &mut Provided) -> Res<()> {
    let install = store.join("python");
    fs::create_dir_all(&install).map_err(|e| format!("{}: {e}", install.display()))?;
    let found = |install: &Path| -> Option<PathBuf> {
        let prefix = format!("cpython-{version}-");
        let mut dirs: Vec<PathBuf> = fs::read_dir(install)
            .ok()?
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
            .map(|e| e.path().join("bin"))
            .filter(|b| b.join("python3").exists() || b.join("python").exists())
            .collect();
        dirs.sort();
        dirs.into_iter().next()
    };
    if found(&install).is_none() {
        // `--no-bin`: no `python3.x` link in the host's `~/.local/bin`.
        run_tool(
            Command::new(&host.uv)
                .args(["python", "install", "--no-bin"])
                .arg(version)
                .env("UV_PYTHON_INSTALL_DIR", &install),
            &format!("`uv python install {version}`"),
        )?;
    }
    let bin = found(&install).ok_or_else(|| {
        format!(
            "`uv python install {version}` left no cpython-{version}-* directory in {}",
            install.display()
        )
    })?;
    p.path_first.push(bin);
    Ok(())
}

fn provide_java(host: &Host, version: &str, p: &mut Provided) -> Res<()> {
    for dir in &host.jvm_dirs {
        let Ok(rd) = fs::read_dir(dir) else { continue };
        let mut homes: Vec<PathBuf> = rd.flatten().map(|e| e.path()).collect();
        homes.sort();
        for home in homes {
            if !home.join("bin/java").exists() {
                continue;
            }
            if let Some(v) = jdk_version(&home)
                && (v == version || v.starts_with(&format!("{version}.")))
            {
                p.path_first.push(home.join("bin"));
                p.env
                    .push(("JAVA_HOME".to_owned(), home.display().to_string()));
                return Ok(());
            }
        }
    }
    Err(format!(
        "no JDK {version} found under {}; install one there (the runner does not download JDKs)",
        host.jvm_dirs
            .iter()
            .map(|d| d.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

/// The version a JDK home reports: the `JAVA_VERSION` of its `release` file, else the
/// output of `bin/java -version`.
fn jdk_version(home: &Path) -> Option<String> {
    if let Ok(t) = fs::read_to_string(home.join("release"))
        && let Some(v) = t
            .lines()
            .find_map(|l| l.strip_prefix("JAVA_VERSION="))
            .map(|v| v.trim().trim_matches('"').to_owned())
    {
        return Some(v);
    }
    let out = Command::new(home.join("bin/java"))
        .arg("-version")
        .output()
        .ok()?;
    parse_java_version(&String::from_utf8_lossy(&out.stderr))
}

// ------------------------------------------------------------------ observing

/// The first `"..."` of `java -version` (`openjdk version "21.0.12" 2025-07-15`).
pub fn parse_java_version(text: &str) -> Option<String> {
    let first = text.lines().find(|l| l.contains("version"))?;
    let start = first.find('"')? + 1;
    let end = first[start..].find('"')? + start;
    Some(first[start..end].to_owned())
}

/// `rustc 1.99.1 (abc 2026-01-01)` gives `1.99.1`.
pub fn parse_rustc_version(text: &str) -> Option<String> {
    text.split_whitespace().nth(1).map(str::to_owned)
}

/// The first `x.y.z`-looking word of a tool's version line, without `v` or `go` prefixes.
pub fn first_version(text: &str) -> Option<String> {
    text.split_whitespace().find_map(|w| {
        let w = w.trim_start_matches(['v', 'V']);
        let w = w.strip_prefix("go").unwrap_or(w);
        let ok = w.split('.').count() >= 2
            && w.split('.')
                .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()));
        ok.then(|| w.to_owned())
    })
}

/// Finds an executable on the `PATH` value of the child's environment.
pub fn find_on_path(path: &str, name: &str) -> Option<PathBuf> {
    std::env::split_paths(&OsString::from(path))
        .map(|d| d.join(name))
        .find(|p| p.is_file())
}

fn capture(program: &Path, args: &[&str], env: &[(String, String)], cwd: &Path) -> Option<String> {
    let mut child = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .env_clear()
        .envs(env.iter().map(|(k, v)| (k, v)))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;
    let start = Instant::now();
    let status = loop {
        match child.try_wait().ok()? {
            Some(s) => break s,
            None if start.elapsed() > Duration::from_secs(30) => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            None => std::thread::sleep(Duration::from_millis(20)),
        }
    };
    let mut text = String::new();
    if let Some(mut o) = child.stdout.take() {
        let _ = o.read_to_string(&mut text);
    }
    if let Some(mut e) = child.stderr.take() {
        let _ = e.read_to_string(&mut text);
    }
    status.success().then_some(text)
}

/// The tool versions the child will see: the commands run with the child's own environment
/// (and `cwd`, the project, because a `go.mod` can choose the Go release). Only the tools of
/// the fixture's runtimes are asked. A tool that does not answer leaves its field empty.
pub fn observe(env: &[(String, String)], cwd: &Path, e: &Expected) -> RunToolchain {
    let path = env
        .iter()
        .find(|(k, _)| k == "PATH")
        .map(|(_, v)| v.as_str())
        .unwrap_or("");
    let ask = |tool: &str, args: &[&str]| -> Option<String> {
        let exe = find_on_path(path, tool)?;
        capture(&exe, args, env, cwd)
    };
    let mut rt = RunToolchain::default();
    for r in runtimes_of(e) {
        match r {
            Runtime::Go => rt.go = ask("go", &["env", "GOVERSION"]).and_then(|t| first_version(&t)),
            Runtime::Node => rt.node = ask("node", &["--version"]).and_then(|t| first_version(&t)),
            Runtime::Python => {
                rt.python = ask("python3", &["--version"])
                    .or_else(|| ask("python", &["--version"]))
                    .and_then(|t| first_version(&t));
            }
            Runtime::Jvm => {
                rt.java = ask("java", &["-version"]).and_then(|t| parse_java_version(&t))
            }
            Runtime::Rust => {
                rt.rust = ask("rustc", &["--version"]).and_then(|t| parse_rustc_version(&t));
            }
            Runtime::Native => {}
        }
    }
    rt
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exp(runtime: &str, version: u32, toolchain: &str) -> Expected {
        toml::from_str(&format!(
            "schema_version = {version}\nid = \"fx\"\nruntime = \"{runtime}\"\ncategory = \"non-leak\"\n\
             dynamic_confirmation = true\n{toolchain}\n[task]\nprompt = \"p\"\n\
             [[non_leak]]\nid = \"N1\"\ncause = \"N-WARM\"\ndiscriminator = \"d\"\n"
        ))
        .unwrap()
    }

    fn problems(e: &Expected, go_mod: Option<&str>) -> Vec<String> {
        let mut out = Vec::new();
        check(e, go_mod, &mut out);
        out
    }

    #[test]
    fn a_fixture_without_a_toolchain_stays_on_schema_version_1() {
        assert!(problems(&exp("go", 1, ""), None).is_empty());
        let v2 = problems(&exp("go", 2, ""), None);
        assert!(
            v2.iter()
                .any(|p| p.contains("schema_version 2 is for fixtures with a [toolchain]")),
            "{v2:?}"
        );
    }

    #[test]
    fn a_toolchain_table_needs_schema_version_2() {
        let e = exp("go", 1, "[toolchain]\ngo = \"1.22.12\"");
        let p = problems(&e, None);
        assert!(
            p.iter().any(|m| m.contains("needs schema_version = 2")),
            "{p:?}"
        );
        assert!(problems(&exp("go", 2, "[toolchain]\ngo = \"1.22.12\""), None).is_empty());
    }

    #[test]
    fn an_empty_toolchain_table_is_refused() {
        let p = problems(&exp("go", 2, "[toolchain]"), None);
        assert!(p.iter().any(|m| m.contains("names no tool")), "{p:?}");
    }

    #[test]
    fn versions_must_be_exact() {
        for bad in ["1.22", "go1.22.12", "1.22.x", "1.22.12.1", "", "v1.22.12"] {
            let p = problems(&exp("go", 2, &format!("[toolchain]\ngo = \"{bad}\"")), None);
            assert!(
                p.iter().any(|m| m.contains("exact version")),
                "{bad:?}: {p:?}"
            );
        }
        for bad in ["22", "22.12", "latest"] {
            let p = problems(
                &exp("node", 2, &format!("[toolchain]\nnode = \"{bad}\"")),
                None,
            );
            assert!(
                p.iter().any(|m| m.contains("exact version")),
                "{bad:?}: {p:?}"
            );
        }
        assert!(problems(&exp("node", 2, "[toolchain]\nnode = \"22.12.0\""), None).is_empty());
        assert!(problems(&exp("python", 2, "[toolchain]\npython = \"3.14.4\""), None).is_empty());
        // A JDK may be named by its feature release or in full.
        assert!(problems(&exp("jvm", 2, "[toolchain]\njava = \"17\""), None).is_empty());
        assert!(problems(&exp("jvm", 2, "[toolchain]\njava = \"21.0.4\""), None).is_empty());
        let p = problems(&exp("jvm", 2, "[toolchain]\njava = \"17.x\""), None);
        assert!(p.iter().any(|m| m.contains("exact version")), "{p:?}");
    }

    #[test]
    fn a_tool_needs_a_runtime_of_the_fixture() {
        let p = problems(&exp("python", 2, "[toolchain]\ngo = \"1.22.12\""), None);
        assert!(p.iter().any(|m| m.contains("no Go runtime")), "{p:?}");
        let multi = toml::from_str::<Expected>(
            "schema_version = 2\nid = \"fx\"\nruntime = \"jvm\"\nruntimes = [\"jvm\", \"go\"]\n\
             category = \"non-leak\"\ndynamic_confirmation = true\n[toolchain]\ngo = \"1.23.12\"\n\
             [task]\nprompt = \"p\"\n[[non_leak]]\nid = \"N1\"\ncause = \"N-WARM\"\ndiscriminator = \"d\"\n",
        )
        .unwrap();
        assert!(problems(&multi, None).is_empty());
    }

    #[test]
    fn go_mod_may_not_ask_for_a_newer_go_than_the_toolchain() {
        let e = exp("go", 2, "[toolchain]\ngo = \"1.22.12\"");
        let p = problems(&e, Some("module m\n\ngo 1.23\n"));
        assert!(p.iter().any(|m| m.contains("asks for go 1.23")), "{p:?}");
        assert!(problems(&e, Some("module m\n\ngo 1.22\n")).is_empty());
        assert!(problems(&e, Some("module m\n\ngo 1.22.0 // pinned\n")).is_empty());
        // The "after" side of a pair: a newer toolchain runs an older go line.
        let after = exp("go", 2, "[toolchain]\ngo = \"1.23.12\"");
        assert!(problems(&after, Some("module m\n\ngo 1.22\n")).is_empty());
        assert!(problems(&after, Some("module m\n\ngo 1.23\n")).is_empty());
        // The patch level counts: Go refuses `go 1.22.13` on go1.22.12.
        let before = exp("go", 2, "[toolchain]\ngo = \"1.22.12\"");
        let p = problems(&before, Some("module m\n\ngo 1.22.13\n"));
        assert!(p.iter().any(|m| m.contains("asks for go 1.22.13")), "{p:?}");
        assert!(problems(&before, Some("module m\n\ngo 1.22.12\n")).is_empty());
        assert!(problems(&before, Some("module m\n\ngo 1.22\n")).is_empty());
    }

    #[test]
    fn go_mod_release_reads_the_go_line() {
        assert_eq!(go_mod_release("module m\ngo 1.22\n"), Some((1, 22, 0)));
        assert_eq!(
            go_mod_release("module m\n\ngo 1.23.4\ntoolchain go1.24.1\n"),
            Some((1, 23, 4))
        );
        assert_eq!(go_mod_release("module m\n"), None);
        assert_eq!(go_mod_release("module go-things\n"), None);
    }

    #[test]
    fn an_unknown_tool_is_refused_when_the_file_is_read() {
        let r = toml::from_str::<Expected>(
            "schema_version = 2\nid = \"fx\"\nruntime = \"go\"\ncategory = \"non-leak\"\n\
             dynamic_confirmation = true\n[toolchain]\nruby = \"3.3.0\"\n[task]\nprompt = \"p\"\n",
        );
        assert!(r.is_err());
    }

    #[test]
    fn apply_sets_variables_and_puts_directories_first_on_path() {
        let p = Provided {
            path_first: vec!["/tc/a".into(), "/tc/b".into()],
            env: vec![("GOTOOLCHAIN".into(), "go1.22.12".into())],
            go_toolchain_dirs: vec![],
        };
        let mut env = vec![
            ("PATH".to_owned(), "/usr/bin:/bin".to_owned()),
            ("GOTOOLCHAIN".to_owned(), "auto".to_owned()),
            ("HOME".to_owned(), "/h".to_owned()),
        ];
        p.apply(&mut env);
        let get = |k: &str| {
            env.iter()
                .filter(|(n, _)| n == k)
                .map(|(_, v)| v.clone())
                .collect::<Vec<_>>()
        };
        // The separator is the platform's (`;` on Windows), and the old value is kept as it was.
        let sep = if cfg!(windows) { ";" } else { ":" };
        assert_eq!(
            get("PATH"),
            vec![format!("/tc/a{sep}/tc/b{sep}/usr/bin:/bin")]
        );
        assert_eq!(get("GOTOOLCHAIN"), vec!["go1.22.12".to_owned()]);
        assert_eq!(get("HOME"), vec!["/h".to_owned()]);
    }

    #[test]
    fn version_lines_of_the_tools_are_read() {
        assert_eq!(first_version("go1.22.12\n").as_deref(), Some("1.22.12"));
        assert_eq!(first_version("v22.12.0\n").as_deref(), Some("22.12.0"));
        assert_eq!(first_version("Python 3.14.4\n").as_deref(), Some("3.14.4"));
        assert_eq!(first_version("devel +abc\n"), None);
        assert_eq!(
            parse_java_version(
                "openjdk version \"21.0.12\" 2025-07-15\nOpenJDK Runtime Environment\n"
            )
            .as_deref(),
            Some("21.0.12")
        );
        assert_eq!(parse_java_version("nothing"), None);
        assert_eq!(
            parse_rustc_version("rustc 1.99.1 (abc 2026-01-01)").as_deref(),
            Some("1.99.1")
        );
        assert_eq!(node_platform("linux", "x86_64").unwrap(), "linux-x64");
        assert_eq!(node_platform("macos", "aarch64").unwrap(), "darwin-arm64");
        assert!(node_platform("windows", "x86_64").is_err());
    }
}

#[cfg(all(test, unix))]
mod provide_tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn script(p: &Path, body: &str) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, format!("#!/bin/sh\n{body}\n")).unwrap();
        fs::set_permissions(p, fs::Permissions::from_mode(0o755)).unwrap();
    }

    fn host(root: &Path) -> Host {
        Host {
            go: root.join("fake/go"),
            uv: root.join("fake/uv"),
            curl: "curl".into(),
            tar: "tar".into(),
            node_dist: format!("file://{}", root.join("dist").display()),
            go_modcache: Some(root.join("modcache")),
            jvm_dirs: vec![root.join("jvm")],
            os: "linux".into(),
            arch: "x86_64".into(),
        }
    }

    fn tc(go: &str, node: &str, python: &str, java: &str) -> Toolchain {
        let o = |s: &str| (!s.is_empty()).then(|| s.to_owned());
        Toolchain {
            go: o(go),
            node: o(node),
            python: o(python),
            java: o(java),
        }
    }

    /// A Node distribution under `dist/v<version>/` whose `node` prints the version.
    fn node_dist(root: &Path, version: &str, corrupt: bool) {
        let stem = format!("node-v{version}-linux-x64");
        let src = root.join("dist-src").join(&stem);
        script(&src.join("bin/node"), &format!("echo v{version}"));
        let dir = root.join("dist").join(format!("v{version}"));
        fs::create_dir_all(&dir).unwrap();
        let tarball = dir.join(format!("{stem}.tar.gz"));
        let st = Command::new("tar")
            .arg("-czf")
            .arg(&tarball)
            .arg("-C")
            .arg(root.join("dist-src"))
            .arg(&stem)
            .status()
            .unwrap();
        assert!(st.success());
        let sum = if corrupt {
            "0".repeat(64)
        } else {
            sha256_hex(&tarball).unwrap()
        };
        fs::write(
            dir.join("SHASUMS256.txt"),
            format!(
                "{}  node-v{version}-darwin-arm64.tar.gz\n{sum}  {stem}.tar.gz\n",
                "1".repeat(64)
            ),
        )
        .unwrap();
    }

    fn root() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn go_is_fetched_with_the_named_release_and_pinned() {
        let t = root();
        let r = t.path();
        // The stand-in `go` unpacks the toolchain the way the real one does.
        let name = crate::run::go_toolchain_dir_name(
            "go1.22.12",
            std::env::consts::OS,
            std::env::consts::ARCH,
        );
        script(
            &r.join("fake/go"),
            &format!(
                "echo \"$GOTOOLCHAIN\" >> '{log}'\nmkdir -p \"$GOMODCACHE/golang.org/{name}/bin\"",
                log = r.join("go.log").display()
            ),
        );
        let p = provide(&host(r), &r.join("store"), &tc("1.22.12", "", "", "")).unwrap();
        assert_eq!(
            p.env,
            vec![("GOTOOLCHAIN".to_owned(), "go1.22.12".to_owned())]
        );
        assert_eq!(
            p.go_toolchain_dirs,
            vec![r.join("modcache/golang.org").join(&name)]
        );
        assert_eq!(fs::read_to_string(r.join("go.log")).unwrap(), "go1.22.12\n");
        // Cached: the host is not asked again.
        provide(&host(r), &r.join("store"), &tc("1.22.12", "", "", "")).unwrap();
        assert_eq!(
            fs::read_to_string(r.join("go.log"))
                .unwrap()
                .lines()
                .count(),
            1
        );
    }

    #[test]
    fn a_go_release_that_cannot_be_fetched_is_an_error_naming_it() {
        let t = root();
        script(&t.path().join("fake/go"), "exit 1");
        let e = provide(
            &host(t.path()),
            &t.path().join("store"),
            &tc("1.22.12", "", "", ""),
        )
        .unwrap_err();
        assert!(e.contains("go1.22.12 could not be fetched"), "{e}");
    }

    #[test]
    fn go_toolchains_are_linked_into_the_child_cache() {
        let t = root();
        let r = t.path();
        let host_mod = r.join("modcache");
        let name = crate::run::go_toolchain_dir_name(
            "go1.22.12",
            std::env::consts::OS,
            std::env::consts::ARCH,
        );
        let dir = host_mod.join("golang.org").join(&name);
        fs::create_dir_all(dir.join("bin")).unwrap();
        let v = name.strip_prefix("toolchain@").unwrap();
        let dl = host_mod.join("cache/download/golang.org/toolchain/@v");
        fs::create_dir_all(&dl).unwrap();
        fs::write(dl.join(format!("{v}.zip")), "z").unwrap();
        fs::write(dl.join(format!("{v}.ziphash")), "h").unwrap();
        let p = Provided {
            go_toolchain_dirs: vec![dir.clone()],
            ..Provided::default()
        };
        let cache = r.join("child/cache");
        p.link_go(&cache);
        assert!(
            cache
                .join("gopath/pkg/mod/golang.org")
                .join(&name)
                .join("bin")
                .is_dir()
        );
        assert!(
            cache
                .join("gopath/pkg/mod/cache/download/golang.org/toolchain/@v")
                .join(format!("{v}.ziphash"))
                .is_file()
        );
    }

    #[test]
    fn node_is_downloaded_checked_unpacked_and_cached() {
        let t = root();
        let r = t.path();
        node_dist(r, "22.12.0", false);
        let store = r.join("store");
        let p = provide(&host(r), &store, &tc("", "22.12.0", "", "")).unwrap();
        let bin = store.join("node/node-v22.12.0-linux-x64/bin");
        assert_eq!(p.path_first, vec![bin.clone()]);
        let out = Command::new(bin.join("node")).output().unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "v22.12.0");
        // Second time: no download (the distribution is gone).
        fs::remove_dir_all(r.join("dist")).unwrap();
        assert_eq!(
            provide(&host(r), &store, &tc("", "22.12.0", "", ""))
                .unwrap()
                .path_first,
            vec![bin]
        );
    }

    #[test]
    fn a_node_tarball_with_the_wrong_checksum_is_refused() {
        let t = root();
        node_dist(t.path(), "24.13.1", true);
        let e = provide(
            &host(t.path()),
            &t.path().join("store"),
            &tc("", "24.13.1", "", ""),
        )
        .unwrap_err();
        assert!(e.contains("does not match the published"), "{e}");
        assert!(!t.path().join("store/node/node-v24.13.1-linux-x64").exists());
    }

    #[test]
    fn a_node_release_that_does_not_exist_is_an_error() {
        let t = root();
        let e = provide(
            &host(t.path()),
            &t.path().join("store"),
            &tc("", "22.99.0", "", ""),
        )
        .unwrap_err();
        assert!(e.contains("download of"), "{e}");
    }

    #[test]
    fn python_is_installed_with_uv_without_a_bin_link() {
        let t = root();
        let r = t.path();
        script(
            &r.join("fake/uv"),
            &format!(
                "echo \"$@\" >> '{log}'\nmkdir -p \"$UV_PYTHON_INSTALL_DIR/cpython-3.14.4-linux-x86_64-gnu/bin\" \"$UV_PYTHON_INSTALL_DIR/cpython-3.14-linux-x86_64-gnu\"\n\
                 printf '#!/bin/sh\\necho Python 3.14.4\\n' > \"$UV_PYTHON_INSTALL_DIR/cpython-3.14.4-linux-x86_64-gnu/bin/python3\"\nchmod +x \"$UV_PYTHON_INSTALL_DIR/cpython-3.14.4-linux-x86_64-gnu/bin/python3\"",
                log = r.join("uv.log").display()
            ),
        );
        let store = r.join("store");
        let p = provide(&host(r), &store, &tc("", "", "3.14.4", "")).unwrap();
        assert_eq!(
            p.path_first,
            vec![store.join("python/cpython-3.14.4-linux-x86_64-gnu/bin")]
        );
        assert_eq!(
            fs::read_to_string(r.join("uv.log")).unwrap().trim(),
            "python install --no-bin 3.14.4"
        );
        // Asking for 3.14.5 does not pick the 3.14.4 directory.
        let e = provide(&host(r), &store, &tc("", "", "3.14.5", "")).unwrap_err();
        assert!(e.contains("cpython-3.14.5-"), "{e}");
        // Cached: uv is not run again for 3.14.4.
        provide(&host(r), &store, &tc("", "", "3.14.4", "")).unwrap();
        assert_eq!(
            fs::read_to_string(r.join("uv.log"))
                .unwrap()
                .lines()
                .count(),
            2
        );
    }

    #[test]
    fn a_jdk_is_found_by_its_release_file() {
        let t = root();
        let r = t.path();
        for (dir, v) in [("jdk-17", "17.0.20"), ("jdk-21", "21.0.12")] {
            script(&r.join("jvm").join(dir).join("bin/java"), "exit 0");
            fs::write(
                r.join("jvm").join(dir).join("release"),
                format!("JAVA_VERSION=\"{v}\"\n"),
            )
            .unwrap();
        }
        let p = provide(&host(r), &r.join("store"), &tc("", "", "", "17")).unwrap();
        assert_eq!(p.path_first, vec![r.join("jvm/jdk-17/bin")]);
        assert_eq!(
            p.env,
            vec![(
                "JAVA_HOME".to_owned(),
                r.join("jvm/jdk-17").display().to_string()
            )]
        );
        let p = provide(&host(r), &r.join("store"), &tc("", "", "", "21.0.12")).unwrap();
        assert_eq!(p.path_first, vec![r.join("jvm/jdk-21/bin")]);
        // A release no JDK on the host has.
        let e = provide(&host(r), &r.join("store"), &tc("", "", "", "25")).unwrap_err();
        assert!(e.contains("no JDK 25"), "{e}");
    }

    #[test]
    fn identical_requests_are_fetched_once_and_a_failure_names_the_fixture() {
        let t = root();
        let r = t.path();
        node_dist(r, "22.12.0", false);
        let reqs = vec![
            ("a".to_owned(), tc("", "22.12.0", "", "")),
            ("b".to_owned(), tc("", "22.12.0", "", "")),
            ("plain".to_owned(), Toolchain::default()),
        ];
        let got = provide_all(&host(r), &r.join("store"), &reqs).unwrap();
        assert_eq!(
            got.keys().map(String::as_str).collect::<Vec<_>>(),
            ["a", "b"]
        );
        assert_eq!(got["a"], got["b"]);
        let bad = vec![("pair-after".to_owned(), tc("", "22.99.0", "", ""))];
        let e = provide_all(&host(r), &r.join("store"), &bad).unwrap_err();
        assert!(e.starts_with("fixture pair-after: "), "{e}");
    }

    #[test]
    fn observe_asks_the_tools_the_child_will_run() {
        let t = root();
        let r = t.path();
        script(&r.join("tc/node"), "echo v22.12.0");
        script(&r.join("tc/python3"), "echo Python 3.14.4");
        script(&r.join("tc/go"), "echo go1.22.12");
        script(&r.join("other/node"), "echo v9.9.9");
        let env = vec![(
            "PATH".to_owned(),
            format!("{}:{}", r.join("tc").display(), r.join("other").display()),
        )];
        let e = |rt: &str| exp_for(rt);
        let node = observe(&env, r, &e("node"));
        assert_eq!(
            node,
            RunToolchain {
                node: Some("22.12.0".into()),
                ..RunToolchain::default()
            }
        );
        let py = observe(&env, r, &e("python"));
        assert_eq!(py.python.as_deref(), Some("3.14.4"));
        let go = observe(&env, r, &e("go"));
        assert_eq!(go.go.as_deref(), Some("1.22.12"));
        // A runtime whose tool is not on PATH leaves the field out.
        let rs = observe(&env, r, &e("rust"));
        assert!(rs.is_empty());
        let native = observe(&env, r, &e("native"));
        assert!(native.is_empty());
    }

    fn exp_for(runtime: &str) -> Expected {
        toml::from_str(&format!(
            "schema_version = 1\nid = \"fx\"\nruntime = \"{runtime}\"\ncategory = \"non-leak\"\n\
             dynamic_confirmation = true\n[task]\nprompt = \"p\"\n[[non_leak]]\nid = \"N1\"\n\
             cause = \"N-WARM\"\ndiscriminator = \"d\"\n"
        ))
        .unwrap()
    }
}
