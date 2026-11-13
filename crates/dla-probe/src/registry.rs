//! The analysis backends DLA knows, where to find them and how to install them.

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::Duration,
};

use dla_core::{probe::ToolProbe, runtime::Runtime};

use crate::exec;

/// One analysis backend.
pub struct Backend {
    /// Name used in detector tables and `coverage.json`.
    pub name: &'static str,
    /// Runtimes it serves.
    pub runtimes: &'static [Runtime],
    /// Executable names, in order of preference.
    pub binaries: &'static [&'static str],
    /// `(prefix, suffix)` of a jar in `<tools>/jars` (version sits between them).
    pub jar: Option<(&'static str, &'static str)>,
    /// Arguments that print the version; `None` when the tool has no such flag.
    pub version_args: Option<&'static [&'static str]>,
    /// Extra explanation recorded with a missing backend (not part of the hint command).
    pub hint_note: &'static str,
}

/// Every backend of the registry.
pub const BACKENDS: &[Backend] = &[
    Backend {
        name: "dla",
        runtimes: Runtime::ALL,
        binaries: &[],
        jar: None,
        version_args: None,
        hint_note: "",
    },
    Backend {
        name: "staticcheck",
        runtimes: &[Runtime::Go],
        binaries: &["staticcheck"],
        jar: None,
        version_args: Some(&["-version"]),
        hint_note: "",
    },
    Backend {
        name: "golangci-lint",
        runtimes: &[Runtime::Go],
        binaries: &["golangci-lint"],
        jar: None,
        version_args: Some(&["--version"]),
        hint_note: "",
    },
    Backend {
        name: "govet",
        runtimes: &[Runtime::Go],
        binaries: &["go"],
        jar: None,
        version_args: Some(&["version"]),
        hint_note: "go vet ships with the Go toolchain",
    },
    Backend {
        name: "errorprone",
        runtimes: &[Runtime::Jvm],
        binaries: &["run-errorprone.sh", "errorprone"],
        jar: Some(("error_prone_core-", ".jar")),
        version_args: None,
        hint_note: "",
    },
    Backend {
        name: "spotbugs",
        runtimes: &[Runtime::Jvm],
        binaries: &["spotbugs"],
        jar: Some(("spotbugs-", ".jar")),
        version_args: Some(&["-version"]),
        hint_note: "",
    },
    Backend {
        name: "detekt",
        runtimes: &[Runtime::Jvm],
        binaries: &["detekt2", "detekt", "detekt-cli"],
        jar: Some(("detekt-cli-", "-all.jar")),
        version_args: Some(&["--version"]),
        hint_note: "",
    },
    Backend {
        name: "codeql",
        runtimes: &[Runtime::Jvm, Runtime::Python],
        binaries: &["codeql"],
        jar: None,
        version_args: Some(&["version", "--format=terse"]),
        hint_note: "",
    },
    Backend {
        name: "sonar",
        runtimes: &[Runtime::Jvm],
        binaries: &["sonar-scanner"],
        jar: None,
        version_args: Some(&["--version"]),
        hint_note: "server-only: the rules run on a SonarQube or SonarCloud server that DLA does not query",
    },
    Backend {
        name: "ruff",
        runtimes: &[Runtime::Python],
        binaries: &["ruff"],
        jar: None,
        version_args: Some(&["--version"]),
        hint_note: "",
    },
    Backend {
        name: "pylint",
        runtimes: &[Runtime::Python],
        binaries: &["pylint"],
        jar: None,
        version_args: Some(&["--version"]),
        hint_note: "",
    },
    Backend {
        name: "eslint",
        runtimes: &[Runtime::Node],
        binaries: &["eslint"],
        jar: None,
        version_args: Some(&["--version"]),
        hint_note: "",
    },
    Backend {
        name: "rapx",
        runtimes: &[Runtime::Rust],
        binaries: &["rapx", "cargo-rapx"],
        jar: None,
        version_args: Some(&["--version"]),
        hint_note: "",
    },
];

/// A dynamic tool `dla summarize` and `dla attach` may run: a profiler, a dump tool or a
/// leak checker. These are not static analyzers, so they are not in [`BACKENDS`] and not
/// in the detector contract.
#[derive(Debug)]
pub struct DynamicTool {
    /// Name used in `coverage.json` and `dla summarize --list`.
    pub name: &'static str,
    /// Executable names, in order of preference.
    pub binaries: &'static [&'static str],
    /// True for JDK tools, which are also looked for in `$JAVA_HOME/bin`.
    pub jdk: bool,
    /// Arguments that print the version; `None` when the tool has no such flag.
    pub version_args: Option<&'static [&'static str]>,
    /// Runtimes that use it.
    pub runtimes: &'static [Runtime],
    /// How to get it.
    pub install_hint: &'static str,
    /// True for a tool no cut-1 adapter runs yet (listed so coverage can name it).
    pub placeholder: bool,
}

/// Every dynamic tool DLA knows.
pub const DYNAMIC_TOOLS: &[DynamicTool] = &[
    DynamicTool {
        name: "jcmd",
        binaries: &["jcmd"],
        jdk: true,
        version_args: Some(&["-version"]),
        runtimes: &[Runtime::Jvm],
        install_hint: "install a JDK and set JAVA_HOME (jcmd lives in $JAVA_HOME/bin), or put jcmd on PATH",
        placeholder: false,
    },
    DynamicTool {
        name: "jfr",
        binaries: &["jfr"],
        jdk: true,
        version_args: Some(&["version"]),
        runtimes: &[Runtime::Jvm],
        install_hint: "install a JDK 11 or newer and set JAVA_HOME (jfr lives in $JAVA_HOME/bin), or put jfr on PATH",
        placeholder: false,
    },
    DynamicTool {
        name: "heaptrack",
        binaries: &["heaptrack"],
        jdk: false,
        version_args: Some(&["--version"]),
        runtimes: &[Runtime::Rust],
        install_hint: "install heaptrack (Linux): apt-get install heaptrack, or put it in $DLA_DATA_DIR/tools/bin",
        placeholder: false,
    },
    DynamicTool {
        name: "heaptrack_print",
        binaries: &["heaptrack_print"],
        jdk: false,
        version_args: Some(&["--version"]),
        runtimes: &[Runtime::Rust],
        install_hint: "install heaptrack (Linux): apt-get install heaptrack, or put heaptrack_print in $DLA_DATA_DIR/tools/bin",
        placeholder: false,
    },
    DynamicTool {
        name: "valgrind",
        binaries: &["valgrind"],
        jdk: false,
        version_args: Some(&["--version"]),
        runtimes: &[Runtime::Rust],
        install_hint: "install valgrind (Linux): apt-get install valgrind, or put it in $DLA_DATA_DIR/tools/bin",
        placeholder: false,
    },
    DynamicTool {
        name: "vgdb",
        binaries: &["vgdb"],
        jdk: false,
        version_args: Some(&["--version"]),
        runtimes: &[Runtime::Rust],
        install_hint: "install valgrind (Linux), which ships vgdb: apt-get install valgrind",
        placeholder: false,
    },
    DynamicTool {
        name: "node",
        binaries: &["node"],
        jdk: false,
        version_args: Some(&["--version"]),
        runtimes: &[Runtime::Node],
        install_hint: "install Node.js 18 or newer",
        placeholder: true,
    },
    DynamicTool {
        name: "go",
        binaries: &["go"],
        jdk: false,
        version_args: Some(&["version"]),
        runtimes: &[Runtime::Go],
        install_hint: "install the Go toolchain",
        placeholder: true,
    },
    DynamicTool {
        name: "memray",
        binaries: &["memray"],
        jdk: false,
        version_args: Some(&["--version"]),
        runtimes: &[Runtime::Python],
        install_hint: "pip install memray (Linux and macOS)",
        placeholder: true,
    },
    DynamicTool {
        name: "bpftrace",
        binaries: &["bpftrace"],
        jdk: false,
        version_args: Some(&["--version"]),
        runtimes: &[Runtime::Rust, Runtime::Go, Runtime::Python],
        install_hint: "install bpftrace (Linux; needs root, tracefs and BTF): apt-get install bpftrace",
        placeholder: true,
    },
];

/// Finds a dynamic tool by name.
pub fn dynamic_tool(name: &str) -> Option<&'static DynamicTool> {
    DYNAMIC_TOOLS.iter().find(|t| t.name == name)
}

/// Looks for a dynamic tool: the installed tools tree first (as for analyzers), then
/// `$JAVA_HOME/bin` for JDK tools, then `path_var`. `java_home` is passed in so a caller
/// (and a test) decides which environment it asks about.
pub fn locate_dynamic(
    tool: &DynamicTool,
    tools: Option<&Path>,
    java_home: Option<&std::ffi::OsStr>,
    path_var: Option<&std::ffi::OsStr>,
) -> Option<Found> {
    for bin in tool.binaries {
        if let Some(t) = tools {
            for d in tool_bin_dirs(t) {
                if let Some(c) = candidates(&d, bin).into_iter().find(|c| is_executable(c)) {
                    return Some(Found {
                        path: c,
                        source: "tools-dir",
                    });
                }
            }
        }
        if tool.jdk
            && let Some(home) = java_home.filter(|h| !h.is_empty())
        {
            let d = Path::new(home).join("bin");
            if let Some(c) = candidates(&d, bin).into_iter().find(|c| is_executable(c)) {
                return Some(Found {
                    path: c,
                    source: "java-home",
                });
            }
        }
        if let Some(f) = locate(bin, None, path_var) {
            return Some(f);
        }
    }
    None
}

/// Resolution of a dynamic tool: where it is and, when asked, its version.
#[derive(Debug, Clone)]
pub struct DynamicResolved {
    /// Where it was found.
    pub found: Option<Found>,
    /// First informative line of its version output.
    pub version: Option<String>,
}

/// Finds `tool` and, when `with_version`, asks it for its version (bounded by `timeout`).
pub fn resolve_dynamic(
    tool: &DynamicTool,
    tools: Option<&Path>,
    java_home: Option<&std::ffi::OsStr>,
    path_var: Option<&std::ffi::OsStr>,
    with_version: bool,
    timeout: Duration,
) -> DynamicResolved {
    let found = locate_dynamic(tool, tools, java_home, path_var);
    let mut version = None;
    if with_version && let (Some(f), Some(args)) = (&found, tool.version_args) {
        let o = exec::run(&f.path, args, None, &[], timeout);
        if o.success {
            version = version_line(&o.text);
        }
    }
    DynamicResolved { found, version }
}

impl Backend {
    /// Every dynamic tool (see [`DynamicTool`]).
    pub fn dynamic_tools() -> &'static [DynamicTool] {
        DYNAMIC_TOOLS
    }

    /// A dynamic tool by name.
    pub fn dynamic(name: &str) -> Option<&'static DynamicTool> {
        dynamic_tool(name)
    }

    /// Where `tool` is on this machine: the tools tree, `$JAVA_HOME/bin`, then PATH.
    pub fn locate_dynamic(
        tool: &DynamicTool,
        tools: Option<&Path>,
        java_home: Option<&std::ffi::OsStr>,
        path_var: Option<&std::ffi::OsStr>,
    ) -> Option<Found> {
        locate_dynamic(tool, tools, java_home, path_var)
    }

    /// [`resolve_dynamic`].
    pub fn resolve_dynamic(
        tool: &DynamicTool,
        tools: Option<&Path>,
        java_home: Option<&std::ffi::OsStr>,
        path_var: Option<&std::ffi::OsStr>,
        with_version: bool,
        timeout: Duration,
    ) -> DynamicResolved {
        resolve_dynamic(tool, tools, java_home, path_var, with_version, timeout)
    }
}

impl Backend {
    /// The install hint recorded when the backend is missing.
    pub fn install_hint(&self) -> String {
        if self.name == "sonar" {
            // A scanner alone leads to no run: the rules are evaluated on a server.
            return "nothing to install: Sonar rules are evaluated on a SonarQube or SonarCloud server, which DLA does not query".to_owned();
        }
        format!("dla install {}", self.name)
    }
}

/// Where a binary was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// Full path.
    pub path: PathBuf,
    /// `path` or `tools-dir`.
    pub source: &'static str,
}

fn is_executable(p: &Path) -> bool {
    let Ok(md) = fs::metadata(p) else {
        return false;
    };
    if !md.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        md.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// Directories of an installed tools tree searched for executables.
pub fn tool_bin_dirs(tools: &Path) -> Vec<PathBuf> {
    vec![
        tools.join("bin"),
        tools.join("venv/bin"),
        tools.join("venv/Scripts"),
        tools.join("node/node_modules/.bin"),
        tools.join("codeql"),
        tools.to_path_buf(),
    ]
}

/// Candidate file names for `bin` in one directory (on Windows also with the
/// executable extensions).
fn candidates(dir: &Path, bin: &str) -> Vec<PathBuf> {
    let mut v = vec![dir.join(bin)];
    if cfg!(windows) {
        v.extend(["exe", "cmd", "bat"].map(|ext| dir.join(format!("{bin}.{ext}"))));
    }
    v
}

/// Looks for an executable named `bin` in the tools dir, then on `path_var`.
pub fn locate(
    bin: &str,
    tools: Option<&Path>,
    path_var: Option<&std::ffi::OsStr>,
) -> Option<Found> {
    if let Some(t) = tools {
        for d in tool_bin_dirs(t) {
            if let Some(c) = candidates(&d, bin).into_iter().find(|c| is_executable(c)) {
                return Some(Found {
                    path: c,
                    source: "tools-dir",
                });
            }
        }
    }
    let pv = path_var?;
    for d in env::split_paths(pv) {
        if d.as_os_str().is_empty() {
            continue;
        }
        if let Some(c) = candidates(&d, bin).into_iter().find(|c| is_executable(c)) {
            return Some(Found {
                path: c,
                source: "path",
            });
        }
    }
    None
}

/// Compares dotted versions numerically.
fn version_key(v: &str) -> Vec<u64> {
    v.split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap_or(0))
        .collect()
}

/// The newest jar `<dir>/<prefix><digit...><suffix>`: path and version.
pub fn find_jar(dir: &Path, prefix: &str, suffix: &str) -> Option<(PathBuf, String)> {
    let mut best: Option<(PathBuf, String)> = None;
    for e in fs::read_dir(dir).ok()?.filter_map(Result::ok) {
        let name = e.file_name().to_string_lossy().into_owned();
        let Some(rest) = name.strip_prefix(prefix) else {
            continue;
        };
        let Some(ver) = rest.strip_suffix(suffix) else {
            continue;
        };
        if !ver.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            continue;
        }
        let better = best
            .as_ref()
            .is_none_or(|(_, v)| version_key(ver) > version_key(v));
        if better {
            best = Some((e.path(), ver.to_owned()));
        }
    }
    best
}

/// First informative line of a version command's output.
pub fn version_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|l| {
            !l.is_empty()
                && !l.starts_with("Picked up ")
                && !l.starts_with("SLF4J")
                && !l.starts_with("WARNING")
        })
        .map(|l| exec::shorten(l, 200))
}

/// Resolution of one backend on this machine.
#[derive(Debug, Clone)]
pub struct Resolved {
    /// Where it is, when found.
    pub found: Option<Found>,
    /// Its version, when known.
    pub version: Option<String>,
}

/// Finds `b` and asks it for its version.
pub fn resolve(
    b: &Backend,
    tools: Option<&Path>,
    path_var: Option<&std::ffi::OsStr>,
    timeout: Duration,
) -> Resolved {
    if b.name == "dla" {
        return Resolved {
            found: Some(Found {
                path: PathBuf::from("dla"),
                source: "builtin",
            }),
            version: Some(env!("CARGO_PKG_VERSION").to_owned()),
        };
    }
    let mut found = b.binaries.iter().find_map(|n| locate(n, tools, path_var));
    let mut version = None;
    if let (Some(f), Some(args)) = (&found, b.version_args) {
        let o = exec::run(&f.path, args, None, &[], timeout);
        if o.success {
            version = version_line(&o.text);
        }
    }
    if let Some((prefix, suffix)) = b.jar {
        if let Some(t) = tools
            && let Some((jar, v)) = find_jar(&t.join("jars"), prefix, suffix)
        {
            if found.is_none() {
                found = Some(Found {
                    path: jar,
                    source: "tools-dir",
                });
            }
            if version.is_none() {
                version = Some(v);
            }
        }
        // A wrapper script that sits next to its jars knows no version flag.
        if version.is_none()
            && let Some(f) = &found
            && let Some(real) = fs::canonicalize(&f.path)
                .ok()
                .or_else(|| Some(f.path.clone()))
            && let Some(dir) = real.parent()
            && let Some((_, v)) = find_jar(dir, prefix, suffix)
        {
            version = Some(v);
        }
    }
    Resolved { found, version }
}

/// Resolves every backend serving a detected runtime, in parallel. One [`ToolProbe`]
/// per `(backend, runtime)`.
pub fn probe_tools(
    runtimes: &[Runtime],
    tools: Option<&Path>,
    path_var: Option<&std::ffi::OsStr>,
    timeout: Duration,
) -> Vec<ToolProbe> {
    let wanted: Vec<&Backend> = BACKENDS
        .iter()
        .filter(|b| b.runtimes.iter().any(|r| runtimes.contains(r)))
        .collect();
    let resolved: Vec<Resolved> = std::thread::scope(|s| {
        let hs: Vec<_> = wanted
            .iter()
            .map(|b| s.spawn(move || resolve(b, tools, path_var, timeout)))
            .collect();
        hs.into_iter()
            .map(|h| {
                h.join().unwrap_or(Resolved {
                    found: None,
                    version: None,
                })
            })
            .collect()
    });
    let mut out = Vec::new();
    for (b, r) in wanted.iter().zip(resolved) {
        for rt in b.runtimes.iter().filter(|r| runtimes.contains(r)) {
            out.push(ToolProbe {
                name: b.name.to_owned(),
                runtime: *rt,
                available: r.found.is_some(),
                source: r.found.as_ref().map(|f| f.source.to_owned()),
                path: r.found.as_ref().map(|f| dla_core::paths::to_slash(&f.path)),
                version: r.version.clone(),
                install_hint: r.found.is_none().then(|| b.install_hint()),
            });
        }
    }
    out
}

// Unix only: fake tools are `#!/bin/sh` scripts made executable with mode bits.
#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn exe(dir: &Path, name: &str, body: &str) -> PathBuf {
        fs::create_dir_all(dir).unwrap();
        let p = dir.join(name);
        fs::write(&p, body).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
        p
    }

    #[test]
    fn sonar_has_no_install_hint_that_cannot_lead_to_a_run() {
        let sonar = BACKENDS.iter().find(|b| b.name == "sonar").unwrap();
        let hint = sonar.install_hint();
        assert!(hint.starts_with("nothing to install"), "{hint}");
        assert!(!hint.contains("dla install"), "{hint}");
        let ruff = BACKENDS.iter().find(|b| b.name == "ruff").unwrap();
        assert_eq!(ruff.install_hint(), "dla install ruff");
    }

    #[test]
    fn registry_names_match_the_detector_contract() {
        for b in BACKENDS {
            assert!(
                dla_core::detectors::KNOWN_BACKENDS.contains(&b.name),
                "{} is not a known backend",
                b.name
            );
        }
        for k in dla_core::detectors::KNOWN_BACKENDS {
            assert!(BACKENDS.iter().any(|b| b.name == k), "{k} not in registry");
        }
    }

    #[test]
    fn finds_on_path_and_in_the_tools_dir_with_a_version() {
        let t = tempfile::tempdir().unwrap();
        let bin = t.path().join("bin");
        exe(&bin, "ruff", "#!/bin/sh\necho 'ruff 9.9.9'\n");
        let tools = t.path().join("tools");
        exe(
            &tools.join("bin"),
            "staticcheck",
            "#!/bin/sh\necho 'staticcheck 1.2.3'\n",
        );
        let pv = std::ffi::OsString::from(bin.as_os_str());
        let ruff = BACKENDS.iter().find(|b| b.name == "ruff").unwrap();
        let r = resolve(ruff, Some(&tools), Some(&pv), Duration::from_secs(5));
        assert_eq!(r.found.as_ref().unwrap().source, "path");
        assert_eq!(r.version.as_deref(), Some("ruff 9.9.9"));
        let sc = BACKENDS.iter().find(|b| b.name == "staticcheck").unwrap();
        let r = resolve(sc, Some(&tools), Some(&pv), Duration::from_secs(5));
        assert_eq!(r.found.as_ref().unwrap().source, "tools-dir");
        assert_eq!(r.version.as_deref(), Some("staticcheck 1.2.3"));
    }

    #[test]
    fn a_missing_backend_carries_the_install_hint() {
        let t = tempfile::tempdir().unwrap();
        let pv = std::ffi::OsString::from(t.path().as_os_str());
        let tools = probe_tools(
            &[Runtime::Go],
            Some(t.path()),
            Some(&pv),
            Duration::from_secs(2),
        );
        let sc = tools.iter().find(|x| x.name == "staticcheck").unwrap();
        assert!(!sc.available);
        assert_eq!(sc.install_hint.as_deref(), Some("dla install staticcheck"));
        let d = tools.iter().find(|x| x.name == "dla").unwrap();
        assert!(d.available && d.version.is_some());
        // Only Go backends (plus dla) are listed for a Go-only project.
        assert!(tools.iter().all(|x| x.runtime == Runtime::Go));
    }

    #[test]
    fn jars_in_the_tools_dir_are_found_by_version() {
        let t = tempfile::tempdir().unwrap();
        let jars = t.path().join("jars");
        fs::create_dir_all(&jars).unwrap();
        for n in [
            "error_prone_core-2.9.0.jar",
            "error_prone_core-2.52.0.jar",
            "error_prone_check_api-2.52.0.jar",
        ] {
            fs::write(jars.join(n), "").unwrap();
        }
        let pv = std::ffi::OsString::new();
        let ep = BACKENDS.iter().find(|b| b.name == "errorprone").unwrap();
        let r = resolve(ep, Some(t.path()), Some(&pv), Duration::from_secs(2));
        assert_eq!(r.version.as_deref(), Some("2.52.0"));
        assert!(r.found.is_some());
    }

    #[test]
    fn a_wrapper_next_to_its_jars_takes_the_jar_version() {
        let t = tempfile::tempdir().unwrap();
        exe(t.path(), "run-errorprone.sh", "#!/bin/sh\n");
        fs::write(t.path().join("error_prone_core-2.40.0.jar"), "").unwrap();
        let pv = std::ffi::OsString::from(t.path().as_os_str());
        let ep = BACKENDS.iter().find(|b| b.name == "errorprone").unwrap();
        let r = resolve(ep, None, Some(&pv), Duration::from_secs(2));
        assert_eq!(r.version.as_deref(), Some("2.40.0"));
    }

    #[test]
    fn dynamic_tools_are_found_in_java_home_the_tools_dir_and_path() {
        let t = tempfile::tempdir().unwrap();
        let jh = t.path().join("jdk");
        exe(&jh.join("bin"), "jcmd", "#!/bin/sh\necho 'jcmd 21'\n");
        let tools = t.path().join("tools");
        exe(
            &tools.join("bin"),
            "valgrind",
            "#!/bin/sh\necho 'valgrind-3.22'\n",
        );
        let pbin = t.path().join("pbin");
        exe(
            &pbin,
            "heaptrack_print",
            "#!/bin/sh\necho 'heaptrack 1.5'\n",
        );
        let pv = std::ffi::OsString::from(pbin.as_os_str());
        let jh_os = std::ffi::OsString::from(jh.as_os_str());
        let d = |n: &str| dynamic_tool(n).unwrap();
        let f = locate_dynamic(d("jcmd"), None, Some(&jh_os), Some(&pv)).unwrap();
        assert_eq!(f.source, "java-home");
        let f = locate_dynamic(d("valgrind"), Some(&tools), None, Some(&pv)).unwrap();
        assert_eq!(f.source, "tools-dir");
        let r = resolve_dynamic(
            d("heaptrack_print"),
            None,
            None,
            Some(&pv),
            true,
            Duration::from_secs(5),
        );
        assert_eq!(r.found.unwrap().source, "path");
        assert_eq!(r.version.as_deref(), Some("heaptrack 1.5"));
        // Not a JDK tool: JAVA_HOME is not consulted for valgrind.
        exe(&jh.join("bin"), "valgrind", "#!/bin/sh\n");
        assert!(locate_dynamic(d("valgrind"), None, Some(&jh_os), None).is_none());
        // Nothing anywhere: missing.
        let empty = std::ffi::OsString::new();
        assert!(locate_dynamic(d("jfr"), None, None, Some(&empty)).is_none());
        assert!(DYNAMIC_TOOLS.iter().any(|t| t.name == "vgdb"));
    }

    #[test]
    fn version_lines_skip_launcher_noise() {
        assert_eq!(
            version_line("Picked up JAVA_TOOL_OPTIONS: -x\n\nspotbugs 4.10.5\n").as_deref(),
            Some("spotbugs 4.10.5")
        );
    }
}
