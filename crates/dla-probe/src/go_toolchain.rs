//! The Go toolchain facts DLA decides things by: the version of the `go` command that
//! builds the project, and whether the `goroutineleak` profile can be had.
//!
//! The goroutineleak profile is generally available from Go 1.27. Go 1.26 has it behind
//! `GOEXPERIMENT=goroutineleakprofile`. On an older toolchain a separate leak pass can use
//! a Go 1.27 or newer toolchain from the Go module cache (`GOTOOLCHAIN=go1.27.<n>`), which
//! `dla install go-toolchain-1.27` puts there. Without any of these the capability is
//! `missing`, with that command as the hint.
//!
//! Everything but [`query`] is a pure function of a version and a list of cached toolchains,
//! so the decision is testable without Go.

use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use dla_core::probe::GoProbe;

/// The install command named in a `missing` capability.
pub const INSTALL_HINT: &str = "dla install go-toolchain-1.27";
/// First minor version with the goroutineleak profile on by default.
const NATIVE_MINOR: u32 = 27;
/// The minor version that has it behind `GOEXPERIMENT`.
const EXPERIMENT_MINOR: u32 = 26;
/// The experiment name on Go 1.26.
pub const EXPERIMENT: &str = "goroutineleakprofile";

/// `(major, minor, patch)` of a Go release.
pub type Version = (u32, u32, u32);

/// How the goroutineleak profile is reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Leak {
    /// The project's own toolchain has it.
    Native,
    /// Go 1.26: set `GOEXPERIMENT=goroutineleakprofile` for the run.
    Experiment,
    /// A cached newer toolchain, named for `GOTOOLCHAIN` (`go1.27.0`): a separate pass.
    Toolchain(String),
    /// Not available.
    Missing,
}

impl Leak {
    /// `native`, `experiment`, `toolchain` or `missing`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Native => "native",
            Self::Experiment => "experiment",
            Self::Toolchain(_) => "toolchain",
            Self::Missing => "missing",
        }
    }
}

/// Parses `go1.24.7`, `go1.26`, `go1.27rc1` or a whole `go version go1.24.7 linux/amd64`
/// line. A missing patch is 0.
pub fn parse_version(text: &str) -> Option<Version> {
    let at = text.find("go1.").or_else(|| text.find("go2."))?;
    let rest = &text[at + 2..];
    let mut nums = [0u32; 3];
    let mut n = 0;
    let mut it = rest.chars().peekable();
    while n < 3 {
        let digits: String = std::iter::from_fn(|| it.next_if(char::is_ascii_digit)).collect();
        if digits.is_empty() {
            break;
        }
        nums[n] = digits.parse().ok()?;
        n += 1;
        if it.next_if(|c| *c == '.').is_none() {
            break;
        }
    }
    (n >= 2).then_some((nums[0], nums[1], nums[2]))
}

/// The toolchains in a Go module cache: `(version, GOTOOLCHAIN name)`, for the directories
/// `golang.org/toolchain@v0.0.1-go1.27.0.linux-amd64` that match this OS and architecture.
pub fn cached_toolchains(modcache: &Path) -> Vec<(Version, String)> {
    let dir = modcache.join("golang.org");
    let Ok(rd) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let suffix = format!(".{}-{}", go_os(), go_arch());
    let mut out = Vec::new();
    for e in rd.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let Some(rest) = name.strip_prefix("toolchain@v0.0.1-") else {
            continue;
        };
        let Some(go) = rest.strip_suffix(&suffix) else {
            continue;
        };
        // The unpacked tree has a `bin/go`; a partial download does not.
        if !e.path().join("bin").join(go_exe()).is_file() {
            continue;
        }
        if let Some(v) = parse_version(go) {
            out.push((v, go.to_owned()));
        }
    }
    // A prerelease ranks below the release of the same version, and rc10 above rc2.
    out.sort_by(|a, b| (a.0, pre_rank(&a.1), &a.1).cmp(&(b.0, pre_rank(&b.1), &b.1)));
    out
}

/// `(1, 0)` for a release; `(0, n)` for a `rcN` or `betaN` prerelease.
fn pre_rank(name: &str) -> (u8, u32) {
    let pre = name
        .find("rc")
        .map(|i| i + 2)
        .or_else(|| name.find("beta").map(|i| i + 4));
    match pre {
        Some(i) => {
            let n: String = name[i..].chars().take_while(char::is_ascii_digit).collect();
            (0, n.parse().unwrap_or(0))
        }
        None => (1, 0),
    }
}

fn go_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        o => o,
    }
}

fn go_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "x86" => "386",
        a => a,
    }
}

fn go_exe() -> &'static str {
    if cfg!(windows) { "go.exe" } else { "go" }
}

/// Decides how the goroutineleak profile is reached from the project's version and the
/// cached toolchains (sorted, oldest first).
pub fn decide(version: Option<Version>, cached: &[(Version, String)]) -> Leak {
    match version {
        Some((1, m, _)) if m >= NATIVE_MINOR => return Leak::Native,
        Some((a, _, _)) if a > 1 => return Leak::Native,
        Some((1, EXPERIMENT_MINOR, _)) => return Leak::Experiment,
        _ => {}
    }
    cached
        .iter()
        .rev()
        .find(|((a, m, _), _)| *a > 1 || (*a == 1 && *m >= NATIVE_MINOR))
        .map_or(Leak::Missing, |(_, name)| Leak::Toolchain(name.clone()))
}

/// The probe record for a version and cache.
pub fn probe_from(version_text: Option<&str>, cached: &[(Version, String)]) -> GoProbe {
    let version = version_text.and_then(parse_version);
    let leak = decide(version, cached);
    GoProbe {
        version: version.map(|(a, m, p)| format!("go{a}.{m}.{p}")),
        goroutineleak: leak.as_str().to_owned(),
        leak_toolchain: match &leak {
            Leak::Toolchain(n) => Some(n.clone()),
            _ => None,
        },
        install_hint: matches!(leak, Leak::Missing).then(|| INSTALL_HINT.to_owned()),
    }
}

/// What a `go` command run in `cwd` reports (its version line and module cache).
#[derive(Debug, Clone, Default)]
pub struct Query {
    /// The first line of `go version`, when it ran.
    pub version_text: Option<String>,
    /// `go env GOMODCACHE`, when it ran.
    pub modcache: Option<PathBuf>,
}

/// Runs `go version` and `go env GOMODCACHE` in `cwd`. Each has the time `timeout`; a
/// failure leaves its field empty. `GOTOOLCHAIN` is left as the person set it, so the
/// answer is the toolchain that would build the project.
pub fn query(cwd: &Path, go: &str, timeout: Duration) -> Query {
    Query {
        version_text: run(cwd, go, &["version"], timeout),
        modcache: run(cwd, go, &["env", "GOMODCACHE"], timeout)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from),
    }
}

fn run(cwd: &Path, go: &str, args: &[&str], timeout: Duration) -> Option<String> {
    let mut child = Command::new(go)
        .args(args)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(st)) if st.success() => break,
            Ok(Some(_)) | Err(_) => return None,
            Ok(None) if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
        }
    }
    let mut text = String::new();
    std::io::Read::read_to_string(&mut child.stdout.take()?, &mut text).ok()?;
    Some(text.lines().next().unwrap_or("").trim().to_owned())
}

/// The full probe of the Go toolchain for the project at `cwd`.
pub fn probe(cwd: &Path, timeout: Duration) -> GoProbe {
    let q = query(cwd, "go", timeout);
    let cached = q
        .modcache
        .as_deref()
        .map(cached_toolchains)
        .unwrap_or_default();
    probe_from(q.version_text.as_deref(), &cached)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_parse_from_lines_and_names() {
        assert_eq!(
            parse_version("go version go1.24.7 linux/amd64"),
            Some((1, 24, 7))
        );
        assert_eq!(parse_version("go1.26"), Some((1, 26, 0)));
        assert_eq!(parse_version("go1.27rc1"), Some((1, 27, 0)));
        assert_eq!(parse_version("go1.27.0.linux-amd64"), Some((1, 27, 0)));
        assert_eq!(parse_version("devel +abc"), None);
        assert_eq!(parse_version(""), None);
    }

    #[test]
    fn a_cached_release_outranks_its_prerelease() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("golang.org");
        let suffix = format!(".{}-{}", go_os(), go_arch());
        for n in ["go1.27rc1", "go1.27.0", "go1.27rc2", "go1.27rc10"] {
            let d = dir
                .join(format!("toolchain@v0.0.1-{n}{suffix}"))
                .join("bin");
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join(go_exe()), "").unwrap();
        }
        let c = cached_toolchains(tmp.path());
        assert_eq!(c.last().unwrap().1, "go1.27.0");
        assert_eq!(
            decide(Some((1, 24, 7)), &c),
            Leak::Toolchain("go1.27.0".to_owned())
        );
        let pre: Vec<_> = c.iter().map(|x| x.1.as_str()).collect();
        assert_eq!(pre, ["go1.27rc1", "go1.27rc2", "go1.27rc10", "go1.27.0"]);
    }

    #[test]
    fn the_capability_follows_the_version_then_the_cache() {
        let cache = vec![
            ((1, 26, 10), "go1.26.10".to_owned()),
            ((1, 27, 0), "go1.27.0".to_owned()),
        ];
        assert_eq!(decide(Some((1, 27, 0)), &[]), Leak::Native);
        assert_eq!(decide(Some((1, 28, 3)), &[]), Leak::Native);
        assert_eq!(decide(Some((1, 26, 2)), &[]), Leak::Experiment);
        assert_eq!(
            decide(Some((1, 24, 7)), &cache),
            Leak::Toolchain("go1.27.0".to_owned())
        );
        // A 1.26 in the cache is no help below 1.26: only 1.27 or newer counts.
        assert_eq!(decide(Some((1, 24, 7)), &cache[..1]), Leak::Missing);
        assert_eq!(decide(None, &[]), Leak::Missing);
        assert_eq!(
            decide(None, &cache),
            Leak::Toolchain("go1.27.0".to_owned()),
            "an unknown project toolchain still has the cached one"
        );
    }

    #[test]
    fn the_probe_record_names_the_fix_only_when_missing() {
        let p = probe_from(Some("go version go1.24.7 linux/amd64"), &[]);
        assert_eq!(p.version.as_deref(), Some("go1.24.7"));
        assert_eq!(p.goroutineleak, "missing");
        assert_eq!(
            p.install_hint.as_deref(),
            Some("dla install go-toolchain-1.27")
        );
        let c = vec![((1, 27, 1), "go1.27.1".to_owned())];
        let p = probe_from(Some("go1.24.7"), &c);
        assert_eq!(p.goroutineleak, "toolchain");
        assert_eq!(p.leak_toolchain.as_deref(), Some("go1.27.1"));
        assert!(p.install_hint.is_none());
        let p = probe_from(Some("go1.27.0"), &[]);
        assert_eq!((p.goroutineleak.as_str(), p.install_hint), ("native", None));
    }

    #[test]
    fn the_cache_lists_only_unpacked_toolchains_of_this_platform() {
        let d = tempfile::tempdir().unwrap();
        let tc = |name: &str, unpacked: bool| {
            let p = d.path().join("golang.org").join(name);
            std::fs::create_dir_all(p.join("bin")).unwrap();
            if unpacked {
                std::fs::write(p.join("bin").join(go_exe()), "").unwrap();
            }
        };
        let plat = format!("{}-{}", go_os(), go_arch());
        tc(&format!("toolchain@v0.0.1-go1.27.0.{plat}"), true);
        tc(&format!("toolchain@v0.0.1-go1.26.0.{plat}"), true);
        tc(&format!("toolchain@v0.0.1-go1.28.0.{plat}"), false);
        tc("toolchain@v0.0.1-go1.27.5.plan9-arm", true);
        tc("x", true);
        let got = cached_toolchains(d.path());
        assert_eq!(
            got.iter().map(|(_, n)| n.as_str()).collect::<Vec<_>>(),
            ["go1.26.0", "go1.27.0"]
        );
    }
}
