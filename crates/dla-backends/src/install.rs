//! `dla install <backend>`: one recipe per backend, everything under
//! `$DLA_DATA_DIR/tools`, never into the project.
//!
//! A recipe is a list of [`Step`]s plus a way to read the installed version. The same
//! list produces the command printed to the user when a step fails, so the fix is
//! exactly what DLA tried.

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use dla_core::{
    Refusal,
    coverage::{BackendCoverage, BackendStatus, CoverageManifest},
    error::{Error, codes},
    toolctx::{StageOutcome, ToolCtx},
};
use serde_json::json;

use crate::{
    exec::{self, Cmd},
    fsx::ToolEnv,
    registry,
};

/// Pinned versions (verified against the installed tools, see docs/VERIFY.md).
pub mod pins {
    /// staticcheck module version.
    pub const STATICCHECK: &str = "2025.1.1";
    /// golangci-lint v2 module version.
    pub const GOLANGCI_LINT: &str = "v2.5.0";
    /// ruff wheel version.
    pub const RUFF: &str = "0.17.4";
    /// pylint wheel version.
    pub const PYLINT: &str = "4.1.3";
    /// eslint and its plugins (npm).
    pub const ESLINT: &[(&str, &str)] = &[
        ("eslint", "10.14.0"),
        ("typescript-eslint", "8.72.1"),
        ("eslint-plugin-rxjs-x", "1.0.6"),
        ("typescript", "6.0.3"),
    ];
    /// Error Prone.
    pub const ERRORPRONE: &str = "2.52.0";
    /// SpotBugs.
    pub const SPOTBUGS: &str = "4.10.5";
    /// detekt 2.x (the only line with `SuspendFunInFinallySection`).
    pub const DETEKT: &str = "2.0.0-alpha.6";
    /// Kotlin standard library for detekt's full analysis classpath.
    pub const KOTLIN_STDLIB: &str = "2.2.0";
    /// kotlinx-coroutines for the same classpath.
    pub const COROUTINES: &str = "1.10.2";
    /// CodeQL bundle release (`codeql-bundle-<tag>` of github/codeql-action). Pinned so the
    /// Kotlin versions its extractor supports do not change between installs; a HEAD
    /// request answered 302 for the linux64, osx64 and win64 assets (nothing was
    /// downloaded). Never run against a real bundle here, see docs/VERIFY.md.
    pub const CODEQL_BUNDLE: &str = "codeql-bundle-v2.27.0";
    /// RAPx git revision (crates.io 0.7.71 no longer builds on current nightly).
    pub const RAPX_REV: &str = "385907a5";
    /// RAPx repository.
    pub const RAPX_GIT: &str = "https://github.com/safer-rust/RAPx.git";
    /// The Go toolchain `dla install go-toolchain-1.27` puts into the Go module cache
    /// (`GOTOOLCHAIN` name). Go 1.27 has the goroutineleak profile on by default.
    pub const GO_TOOLCHAIN: &str = "go1.27.0";
}

/// The name of the Go toolchain entry of `dla install`: not a static analyzer, so it is not
/// in `KNOWN_BACKENDS`. It fetches a newer toolchain into the Go module cache, for the
/// separate goroutineleak pass of a project that builds with an older Go.
pub const GO_TOOLCHAIN_BACKEND: &str = "go-toolchain-1.27";

/// One command of a recipe.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    /// Program (looked up on `PATH` unless absolute).
    pub program: String,
    /// Arguments.
    pub args: Vec<String>,
    /// Environment variables set for this command.
    pub env: Vec<(String, String)>,
}

impl Run {
    fn new(program: impl Into<String>, args: &[&str]) -> Self {
        Self {
            program: program.into(),
            args: args.iter().map(|s| (*s).to_owned()).collect(),
            env: Vec::new(),
        }
    }

    fn env(mut self, k: &str, v: impl Into<String>) -> Self {
        self.env.push((k.to_owned(), v.into()));
        self
    }

    fn arg(mut self, a: impl Into<String>) -> Self {
        self.args.push(a.into());
        self
    }

    /// The program and its first two arguments, enough to name the step in a message.
    pub fn short(&self) -> String {
        let name = Path::new(&self.program).file_name().map_or_else(
            || self.program.clone(),
            |n| n.to_string_lossy().into_owned(),
        );
        let mut parts = vec![name];
        parts.extend(self.args.iter().take(2).cloned());
        parts.join(" ")
    }

    /// The command as a shell line (`K=v prog arg ...`, arguments quoted when needed).
    pub fn line(&self) -> String {
        let mut parts: Vec<String> = self
            .env
            .iter()
            .map(|(k, v)| format!("{k}={}", quote(v)))
            .collect();
        parts.push(quote(&self.program));
        parts.extend(self.args.iter().map(|a| quote(a)));
        parts.join(" ")
    }
}

fn quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./:=@+,%".contains(c))
    {
        s.to_owned()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

/// One step of a recipe.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// Create a directory.
    MkDir(PathBuf),
    /// Write a file (a generated build descriptor).
    Write {
        /// Where.
        path: PathBuf,
        /// Content.
        content: String,
    },
    /// Run a command.
    Run(Run),
}

/// How the installed version is read.
#[derive(Debug, Clone, PartialEq)]
pub enum Probe {
    /// Run `program args` and take the first version number.
    Program(PathBuf, Vec<String>),
    /// The version in the name of a jar in `dir` that starts with `prefix`.
    Jar(PathBuf, String),
    /// A fixed answer (nothing to install).
    Fixed(String),
    /// `version` when the file exists (a tool that cannot be run on its own).
    File(PathBuf, String),
}

/// A backend's install recipe.
#[derive(Debug, Clone, PartialEq)]
pub struct Recipe {
    /// Backend.
    pub backend: String,
    /// Programs that must exist before the steps run.
    pub requires: Vec<String>,
    /// Steps, in order.
    pub steps: Vec<Step>,
    /// Version probe after the steps.
    pub probe: Probe,
    /// Extra advice appended to a failure message.
    pub advice: String,
}

impl Recipe {
    /// The commands a person would type to do what the steps do, one per line.
    pub fn user_command(&self) -> String {
        let mut lines = Vec::new();
        for s in &self.steps {
            match s {
                Step::MkDir(d) => lines.push(format!("mkdir -p {}", quote(&d.to_string_lossy()))),
                Step::Write { path, .. } => lines.push(format!(
                    "# DLA writes {} first (a generated build file)",
                    path.display()
                )),
                Step::Run(r) => lines.push(r.line()),
            }
        }
        lines.join(" && ")
    }
}

/// Operating-system facts a recipe depends on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Platform {
    /// Windows.
    pub windows: bool,
    /// macOS.
    pub macos: bool,
}

impl Platform {
    /// The platform this binary runs on.
    pub fn current() -> Self {
        Self {
            windows: cfg!(windows),
            macos: cfg!(target_os = "macos"),
        }
    }

    fn python(self) -> &'static str {
        if self.windows { "python" } else { "python3" }
    }

    fn venv_bin(self, tools: &Path) -> PathBuf {
        tools
            .join("venv")
            .join(if self.windows { "Scripts" } else { "bin" })
    }

    fn exe(self, name: &str) -> String {
        if self.windows {
            format!("{name}.exe")
        } else {
            name.to_owned()
        }
    }

    fn codeql_asset(self) -> &'static str {
        if self.windows {
            "codeql-bundle-win64.tar.gz"
        } else if self.macos {
            "codeql-bundle-osx64.tar.gz"
        } else {
            "codeql-bundle-linux64.tar.gz"
        }
    }
}

fn pom(group: &str, artifact: &str, version: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<project xmlns=\"http://maven.apache.org/POM/4.0.0\">\n  <modelVersion>4.0.0</modelVersion>\n  <groupId>dla.install</groupId>\n  <artifactId>{artifact}-install</artifactId>\n  <version>1</version>\n  <dependencies>\n    <dependency>\n      <groupId>{group}</groupId>\n      <artifactId>{artifact}</artifactId>\n      <version>{version}</version>\n    </dependency>\n  </dependencies>\n</project>\n"
    )
}

/// Copies a dependency tree into `<tools>/jars/<name>` through a generated pom.
fn maven_tree(tools: &Path, name: &str, group: &str, artifact: &str, version: &str) -> Vec<Step> {
    let out = tools.join("jars").join(name);
    let build = tools.join("jars/.build").join(name);
    let pom_path = build.join("pom.xml");
    vec![
        Step::MkDir(out.clone()),
        Step::Write {
            path: pom_path.clone(),
            content: pom(group, artifact, version),
        },
        Step::Run(
            Run::new(
                "mvn",
                &[
                    "-B",
                    "-q",
                    "-f",
                    &pom_path.to_string_lossy(),
                    "dependency:copy-dependencies",
                ],
            )
            .arg(format!("-DoutputDirectory={}", out.display())),
        ),
    ]
}

/// Copies one artifact (no transitive dependencies) into `dir`.
fn maven_copy(artifact: &str, dir: &Path) -> Step {
    Step::Run(
        Run::new("mvn", &["-B", "-q", "dependency:copy"])
            .arg(format!("-Dartifact={artifact}"))
            .arg(format!("-DoutputDirectory={}", dir.display())),
    )
}

/// The recipe for `backend`, or why there is none.
pub fn recipe(backend: &str, tools: &Path, p: Platform) -> Result<Recipe, String> {
    let tools_s = tools.to_string_lossy().into_owned();
    let bin = tools.join("bin");
    let go_install = |module: &str| {
        Step::Run(Run::new("go", &["install", module]).env("GOBIN", bin.to_string_lossy()))
    };
    let r = |requires: &[&str], steps: Vec<Step>, probe: Probe, advice: &str| Recipe {
        backend: backend.to_owned(),
        requires: requires.iter().map(|s| (*s).to_owned()).collect(),
        steps,
        probe,
        advice: advice.to_owned(),
    };
    Ok(match backend {
        "dla" => r(
            &[],
            vec![],
            Probe::Fixed(env!("CARGO_PKG_VERSION").to_owned()),
            "",
        ),
        "staticcheck" => r(
            &["go"],
            vec![
                Step::MkDir(bin.clone()),
                go_install(&format!(
                    "honnef.co/go/tools/cmd/staticcheck@{}",
                    pins::STATICCHECK
                )),
            ],
            Probe::Program(bin.join(p.exe("staticcheck")), vec!["-version".into()]),
            "Go modules are fetched through GOPROXY; set GOPROXY if the default is not reachable.",
        ),
        "golangci-lint" => r(
            &["go"],
            vec![
                Step::MkDir(bin.clone()),
                go_install(&format!(
                    "github.com/golangci/golangci-lint/v2/cmd/golangci-lint@{}",
                    pins::GOLANGCI_LINT
                )),
            ],
            Probe::Program(bin.join(p.exe("golangci-lint")), vec!["--version".into()]),
            "Go modules are fetched through GOPROXY; set GOPROXY if the default is not reachable.",
        ),
        // `GOTOOLCHAIN=go1.27.0 go version` makes the go command download that toolchain
        // into the module cache (GOMODCACHE, outside the project) and run it. It needs
        // the network once; later runs find it in the cache.
        GO_TOOLCHAIN_BACKEND => r(
            &["go"],
            vec![Step::Run(
                Run::new("go", &["version"]).env("GOTOOLCHAIN", pins::GO_TOOLCHAIN),
            )],
            Probe::Fixed(pins::GO_TOOLCHAIN.trim_start_matches("go").to_owned()),
            "the Go toolchain is fetched through GOPROXY into the module cache; set GOPROXY if the default is not reachable, and GOTOOLCHAIN must not be `local`.",
        ),
        "govet" => r(
            &["go"],
            vec![],
            Probe::Program(PathBuf::from("go"), vec!["version".into()]),
            "go vet ships with the Go toolchain: install Go from https://go.dev/dl/",
        ),
        "ruff" | "pylint" => {
            let venv = tools.join("venv");
            let py = p.venv_bin(tools).join(p.exe("python"));
            let (pkg, exe) = if backend == "ruff" {
                (format!("ruff=={}", pins::RUFF), "ruff")
            } else {
                (format!("pylint=={}", pins::PYLINT), "pylint")
            };
            r(
                &[p.python()],
                vec![
                    Step::Run(Run::new(
                        p.python(),
                        &["-m", "venv", &venv.to_string_lossy()],
                    )),
                    Step::Run(
                        Run::new(
                            py.to_string_lossy(),
                            &[
                                "-m",
                                "pip",
                                "install",
                                "--disable-pip-version-check",
                                "--quiet",
                            ],
                        )
                        .arg(pkg),
                    ),
                ],
                Probe::Program(p.venv_bin(tools).join(p.exe(exe)), vec!["--version".into()]),
                "",
            )
        }
        "eslint" => {
            let node = tools.join("node");
            let mut args: Vec<String> = vec![
                "install".into(),
                "--prefix".into(),
                node.to_string_lossy().into_owned(),
                "--no-audit".into(),
                "--no-fund".into(),
                "--loglevel=error".into(),
            ];
            args.extend(pins::ESLINT.iter().map(|(n, v)| format!("{n}@{v}")));
            r(
                &["npm"],
                vec![
                    Step::MkDir(node.clone()),
                    Step::Run(Run {
                        program: "npm".into(),
                        args,
                        env: Vec::new(),
                    }),
                ],
                Probe::Program(
                    node.join("node_modules/.bin").join(if p.windows {
                        "eslint.cmd"
                    } else {
                        "eslint"
                    }),
                    vec!["--version".into()],
                ),
                "",
            )
        }
        "errorprone" => r(
            &["mvn", "javac"],
            maven_tree(
                tools,
                "errorprone",
                "com.google.errorprone",
                "error_prone_core",
                pins::ERRORPRONE,
            ),
            Probe::Jar(tools.join("jars/errorprone"), "error_prone_core".into()),
            "Maven Central must be reachable (set the proxy in JAVA_TOOL_OPTIONS or ~/.m2/settings.xml).",
        ),
        "spotbugs" => r(
            &["mvn", "java"],
            maven_tree(
                tools,
                "spotbugs",
                "com.github.spotbugs",
                "spotbugs",
                pins::SPOTBUGS,
            ),
            Probe::Jar(tools.join("jars/spotbugs"), "spotbugs".into()),
            "Maven Central must be reachable (set the proxy in JAVA_TOOL_OPTIONS or ~/.m2/settings.xml).",
        ),
        "detekt" => {
            let dir = tools.join("jars/detekt");
            let cp = tools.join("jars/kotlin-cp");
            r(
                &["mvn", "java"],
                vec![
                    Step::MkDir(dir.clone()),
                    Step::MkDir(cp.clone()),
                    maven_copy(
                        &format!("dev.detekt:detekt-cli:{}:jar:all", pins::DETEKT),
                        &dir,
                    ),
                    maven_copy(
                        &format!(
                            "org.jetbrains.kotlin:kotlin-stdlib:{}:jar",
                            pins::KOTLIN_STDLIB
                        ),
                        &cp,
                    ),
                    maven_copy(
                        &format!(
                            "org.jetbrains.kotlinx:kotlinx-coroutines-core-jvm:{}:jar",
                            pins::COROUTINES
                        ),
                        &cp,
                    ),
                ],
                Probe::Jar(dir, "detekt-cli".into()),
                "Maven Central must be reachable (set the proxy in JAVA_TOOL_OPTIONS or ~/.m2/settings.xml).",
            )
        }
        "codeql" => {
            let dir = tools.join("codeql");
            let tarball = dir.join("bundle.tar.gz");
            let url = format!(
                "https://github.com/github/codeql-action/releases/download/{}/{}",
                pins::CODEQL_BUNDLE,
                p.codeql_asset()
            );
            r(
                &["curl", "tar"],
                vec![
                    Step::MkDir(dir.clone()),
                    Step::Run(Run::new(
                        "curl",
                        &[
                            "-fL",
                            "--retry",
                            "3",
                            "-o",
                            &tarball.to_string_lossy(),
                            &url,
                        ],
                    )),
                    Step::Run(Run::new(
                        "tar",
                        &[
                            "-xzf",
                            &tarball.to_string_lossy(),
                            "-C",
                            &dir.to_string_lossy(),
                        ],
                    )),
                ],
                Probe::Program(
                    dir.join("codeql").join(p.exe("codeql")),
                    vec!["version".into(), "--format=terse".into()],
                ),
                "The bundle (pinned release) is about 1 GB; GitHub release downloads must be reachable. The CodeQL license restricts use to open source code and research, check it applies.",
            )
        }
        "rapx" => {
            let root = tools.join("rapx");
            r(
                &["rustup", "cargo"],
                vec![
                    Step::Run(Run::new(
                        "rustup",
                        &[
                            "toolchain",
                            "install",
                            "nightly",
                            "--profile",
                            "minimal",
                            "--component",
                            "rustc-dev,rust-src,llvm-tools-preview",
                        ],
                    )),
                    Step::Run(
                        Run::new(
                            "cargo",
                            &[
                                "+nightly",
                                "install",
                                "rapx",
                                "--git",
                                pins::RAPX_GIT,
                                "--rev",
                                pins::RAPX_REV,
                                "--root",
                            ],
                        )
                        .arg(root.to_string_lossy()),
                    ),
                ],
                Probe::File(
                    root.join("bin").join(p.exe("cargo-rapx")),
                    format!("0.7.71+git.{}", pins::RAPX_REV),
                ),
                "RAPx links Z3: install its development files first (Debian/Ubuntu: apt-get install libz3-dev z3). It builds only on a matching nightly, so the revision is pinned.",
            )
        }
        "sonar" => {
            return Err(
                "Sonar analyses on a SonarQube or SonarCloud server and DLA does not query one: there is nothing to install, and the sonar row stays blocked as server-only"
                    .to_owned(),
            );
        }
        other => {
            return Err(format!(
                "no install recipe for {other:?} (tools dir {tools_s})"
            ));
        }
    })
}

/// Reads the version a probe describes.
fn read_version(probe: &Probe, env: &ToolEnv) -> Option<String> {
    match probe {
        Probe::Fixed(v) => Some(v.clone()),
        Probe::File(path, v) => path.exists().then(|| v.clone()),
        Probe::Program(path, args) => {
            let program = if path.is_absolute() {
                if !path.exists() {
                    return None;
                }
                path.clone()
            } else {
                env.find_program(&path.to_string_lossy())?
            };
            let v = crate::adapters::version_of(
                Cmd::new(program)
                    .args(args.clone())
                    .env_remove("JAVA_TOOL_OPTIONS"),
            );
            Some(v)
        }
        Probe::Jar(dir, prefix) => {
            let jars = crate::fsx::jars_in(dir);
            jars.iter().find_map(|j| {
                let n = j.file_name()?.to_str()?;
                let rest = n.strip_prefix(prefix.as_str())?.strip_prefix('-')?;
                let v = rest
                    .strip_suffix("-all.jar")
                    .or_else(|| rest.strip_suffix(".jar"))?;
                v.starts_with(|c: char| c.is_ascii_digit())
                    .then(|| v.to_owned())
            })
        }
    }
}

fn step_timeout() -> Duration {
    Duration::from_secs(
        std::env::var("DLA_INSTALL_STEP_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3600),
    )
}

fn failure(rec: &Recipe, why: &str) -> Refusal {
    let mut msg = format!(
        "could not install {}: {}",
        rec.backend,
        why.replace('\n', " | ")
    );
    if !rec.advice.is_empty() {
        msg.push_str(&format!(" ({})", rec.advice));
    }
    let fix = if rec.steps.is_empty() {
        rec.advice.clone()
    } else {
        rec.user_command()
    };
    Refusal::new(
        codes::TOOL_FAILED,
        msg,
        format!("{} installed under the DLA tools directory", rec.backend),
        fix,
    )
}

/// Marks the backend `missing` (with the hint) for every runtime it serves, unless a
/// sweep already ran it.
fn keep_missing(ctx: &ToolCtx, backend: &str, reason: &str) -> Result<(), Refusal> {
    let Some(b) = registry::backend(backend) else {
        return Ok(());
    };
    // Sonar is a server-only row (`blocked`, no install hint); an install attempt that
    // cannot lead to a run must not turn it into a gap with a hint.
    if backend == "sonar" {
        return Ok(());
    }
    CoverageManifest::update(&ctx.run_dir, |m| {
        for rt in b.runtimes {
            let worker = m.worker_mut(rt.as_str());
            let ran = worker
                .backends
                .get(backend)
                .is_some_and(|c| c.status == BackendStatus::Ran);
            if !ran && *rt != dla_core::runtime::Runtime::Native {
                worker.backends.insert(
                    backend.to_owned(),
                    BackendCoverage::missing(reason, registry::install_hint(backend)),
                );
            }
        }
    })
    .map_err(|e| e.to_refusal())?;
    Ok(())
}

/// Runs `rec` with `tools` as the install root and returns the installed version.
pub fn execute(ctx: &ToolCtx, rec: &Recipe, env: &ToolEnv) -> Result<String, Refusal> {
    for prog in &rec.requires {
        if env.find_program(prog).is_none() && env.find_jdk_tool(prog).is_none() {
            return Err(failure(rec, &format!("{prog} was not found on PATH")));
        }
    }
    let scratch = ctx.tmp_dir()?;
    for step in &rec.steps {
        match step {
            Step::MkDir(d) => {
                fs::create_dir_all(d).map_err(|e| Error::io(d, e).to_refusal())?;
            }
            Step::Write { path, content } => {
                if let Some(p) = path.parent() {
                    fs::create_dir_all(p).map_err(|e| Error::io(p, e).to_refusal())?;
                }
                fs::write(path, content).map_err(|e| Error::io(path, e).to_refusal())?;
            }
            Step::Run(r) => {
                let program = env
                    .find_program(&r.program)
                    .or_else(|| env.find_jdk_tool(&r.program))
                    .unwrap_or_else(|| PathBuf::from(&r.program));
                let mut cmd = Cmd::new(program)
                    .args(r.args.clone())
                    .cwd(&scratch)
                    .timeout(step_timeout());
                for (k, v) in &r.env {
                    cmd = cmd.env(k, v);
                }
                let out = exec::run(&cmd);
                if let Some(why) = out.failure_reason() {
                    return Err(failure(rec, &format!("`{}` {why}", r.short())));
                }
                if out.exit != Some(0) {
                    return Err(failure(
                        rec,
                        &format!(
                            "`{}` exited with {}: {}",
                            r.short(),
                            out.exit
                                .map_or_else(|| "a signal".to_owned(), |c| c.to_string()),
                            out.tail(2)
                        ),
                    ));
                }
            }
        }
    }
    // The tools directory changed: look again.
    let env = ToolEnv::from_process(env.tools_dir.clone());
    read_version(&rec.probe, &env)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| failure(rec, "the installed tool was not found afterwards"))
}

/// Name of the lock file in the tools directory.
const LOCK_FILE: &str = ".install.lock";

/// Takes the exclusive lock of the tools directory, waiting for another install to
/// finish. The lock is released when the returned file is dropped (or the process
/// ends), so two workers installing at once cannot interleave their writes.
fn lock_tools(tools: &Path) -> Result<fs::File, Refusal> {
    let fail = |why: String| {
        Refusal::new(
            codes::TOOL_FAILED,
            format!("cannot lock the tools directory: {why}"),
            "an exclusive lock on the tools directory",
            "check that the tools directory is writable, then run `dla install` again",
        )
    };
    fs::create_dir_all(tools).map_err(|e| fail(format!("{}: {e}", tools.display())))?;
    let path = tools.join(LOCK_FILE);
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .map_err(|e| fail(format!("{}: {e}", path.display())))?;
    file.lock()
        .map_err(|e| fail(format!("{}: {e}", path.display())))?;
    Ok(file)
}

/// Installs one backend.
pub fn install(ctx: &ToolCtx, backend: &str) -> Result<StageOutcome, Refusal> {
    ctx.require_tier(1, "dla install")?;
    let Some(tools) = ctx.tools_dir() else {
        return Err(Refusal::new(
            codes::NO_DATA_DIR,
            "DLA_DATA_DIR is not set, so there is no tools directory to install into",
            "the plugin data directory (DLA_DATA_DIR)",
            "run through bin/dla, or export DLA_DATA_DIR=<writable directory>",
        ));
    };
    // Held until this function returns.
    let _lock = lock_tools(&tools)?;
    let rec = match recipe(backend, &tools, Platform::current()) {
        Ok(r) => r,
        Err(why) => {
            let first: String = why.chars().take(200).collect();
            keep_missing(ctx, backend, &first)?;
            return Err(Refusal::new(
                codes::TOOL_FAILED,
                format!("cannot install {backend}: {why}"),
                format!("{backend} installed"),
                registry::install_hint(backend),
            ));
        }
    };
    let env = ToolEnv::from_process(Some(tools.clone()));
    match execute(ctx, &rec, &env) {
        Ok(version) if backend == GO_TOOLCHAIN_BACKEND => {
            // The goroutineleak profile is now available through the cached toolchain.
            CoverageManifest::update(&ctx.run_dir, |m| {
                m.set_backend(
                    "go",
                    "goroutineleak",
                    BackendCoverage::ran(Some(format!("go{version}")), 0),
                );
            })
            .map_err(|e| e.to_refusal())?;
            Ok(StageOutcome {
                evidence: Vec::new(),
                candidates: 0,
                skipped_duplicates: 0,
                artifacts: Vec::new(),
                summary: json!({
                    "backend": backend,
                    "version": version,
                    "tools_dir": dla_core::paths::to_slash(&tools),
                    "toolchain": pins::GO_TOOLCHAIN,
                    "note": "stored in the Go module cache (GOMODCACHE), outside the project; dla harness run uses it for the goroutineleak pass",
                }),
            })
        }
        Ok(version) => Ok(StageOutcome {
            evidence: Vec::new(),
            candidates: 0,
            skipped_duplicates: 0,
            artifacts: Vec::new(),
            summary: json!({
                "backend": backend,
                "version": version,
                "tools_dir": dla_core::paths::to_slash(&tools),
            }),
        }),
        Err(e) => {
            let first: String = e
                .message
                .lines()
                .next()
                .unwrap_or("")
                .chars()
                .take(200)
                .collect();
            keep_missing(ctx, backend, &format!("install failed: {first}"))?;
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINUX: Platform = Platform {
        windows: false,
        macos: false,
    };

    #[test]
    fn the_tools_directory_lock_is_exclusive_until_dropped() {
        let d = tempfile::tempdir().unwrap();
        let tools = d.path().join("tools");
        let held = lock_tools(&tools).unwrap();
        let other = fs::OpenOptions::new()
            .write(true)
            .open(tools.join(LOCK_FILE))
            .unwrap();
        assert!(matches!(
            other.try_lock(),
            Err(fs::TryLockError::WouldBlock)
        ));
        drop(held);
        assert!(other.try_lock().is_ok());
    }

    fn tools() -> PathBuf {
        PathBuf::from("/data/tools")
    }

    /// `/data/tools/<rel>` built with the platform's separators, as the recipes build it.
    fn under_tools(rel: &str) -> String {
        let mut p = tools();
        for part in rel.split('/') {
            p.push(part);
        }
        p.to_string_lossy().into_owned()
    }

    fn runs(r: &Recipe) -> Vec<&Run> {
        r.steps
            .iter()
            .filter_map(|s| if let Step::Run(r) = s { Some(r) } else { None })
            .collect()
    }

    #[test]
    fn go_tools_install_into_the_tools_bin_directory() {
        let r = recipe("staticcheck", &tools(), LINUX).unwrap();
        let run = runs(&r)[0];
        assert_eq!(run.program, "go");
        assert_eq!(
            run.args,
            ["install", "honnef.co/go/tools/cmd/staticcheck@2025.1.1"]
        );
        assert_eq!(run.env, [("GOBIN".to_owned(), under_tools("bin"))]);
        let bin = quote(&under_tools("bin"));
        assert_eq!(
            r.user_command(),
            format!(
                "mkdir -p {bin} && GOBIN={bin} go install honnef.co/go/tools/cmd/staticcheck@2025.1.1"
            )
        );
        let g = recipe("golangci-lint", &tools(), LINUX).unwrap();
        assert!(
            g.user_command()
                .contains("golangci-lint/v2/cmd/golangci-lint@v2.5.0")
        );
    }

    #[test]
    fn python_tools_share_a_pinned_venv() {
        let r = recipe("ruff", &tools(), LINUX).unwrap();
        let rs = runs(&r);
        assert_eq!(rs[0].args, ["-m", "venv", &under_tools("venv")]);
        assert_eq!(rs[1].program, under_tools("venv/bin/python"));
        assert!(rs[1].args.contains(&"ruff==0.17.4".to_owned()));
        let p = recipe("pylint", &tools(), LINUX).unwrap();
        assert!(runs(&p)[1].args.contains(&"pylint==4.1.3".to_owned()));
        let w = recipe(
            "ruff",
            &tools(),
            Platform {
                windows: true,
                macos: false,
            },
        )
        .unwrap();
        assert_eq!(runs(&w)[0].program, "python");
        assert!(Path::new(&runs(&w)[1].program).ends_with("Scripts/python.exe"));
    }

    #[test]
    fn eslint_uses_npm_prefix() {
        let r = recipe("eslint", &tools(), LINUX).unwrap();
        let run = runs(&r)[0];
        assert_eq!(run.program, "npm");
        assert_eq!(
            &run.args[..3],
            ["install", "--prefix", under_tools("node").as_str()]
        );
        for pkg in [
            "eslint@10.14.0",
            "typescript-eslint@8.72.1",
            "eslint-plugin-rxjs-x@1.0.6",
        ] {
            assert!(run.args.contains(&pkg.to_owned()), "{pkg}");
        }
    }

    #[test]
    fn jvm_tools_come_from_maven_into_the_jars_directory() {
        let r = recipe("errorprone", &tools(), LINUX).unwrap();
        let Step::Write { path, content } = &r.steps[1] else {
            panic!("pom step first");
        };
        assert_eq!(
            path,
            &PathBuf::from(under_tools("jars/.build/errorprone/pom.xml"))
        );
        assert!(content.contains("<artifactId>error_prone_core</artifactId>"));
        assert!(content.contains("<version>2.52.0</version>"));
        let run = runs(&r)[0];
        assert_eq!(run.program, "mvn");
        assert!(run.args.contains(&format!(
            "-DoutputDirectory={}",
            under_tools("jars/errorprone")
        )));
        let d = recipe("detekt", &tools(), LINUX).unwrap();
        let cmds = runs(&d);
        assert_eq!(cmds.len(), 3);
        assert!(
            cmds[0]
                .args
                .contains(&"-Dartifact=dev.detekt:detekt-cli:2.0.0-alpha.6:jar:all".to_owned())
        );
        assert!(
            cmds[1]
                .args
                .iter()
                .any(|a| a.contains("kotlin-stdlib:2.2.0"))
        );
        assert!(
            cmds[2]
                .args
                .iter()
                .any(|a| a.contains("kotlinx-coroutines-core-jvm:1.10.2"))
        );
        let s = recipe("spotbugs", &tools(), LINUX).unwrap();
        assert!(
            dla_core::paths::slash(&s.user_command()).contains("jars/spotbugs"),
            "{}",
            s.user_command()
        );
    }

    #[test]
    fn rapx_pins_the_revision_and_roots_under_tools() {
        let r = recipe("rapx", &tools(), LINUX).unwrap();
        let rs = runs(&r);
        assert_eq!(rs[0].program, "rustup");
        assert!(
            rs[0]
                .args
                .contains(&"rustc-dev,rust-src,llvm-tools-preview".to_owned())
        );
        assert_eq!(rs[1].args[..3], ["+nightly", "install", "rapx"]);
        assert!(rs[1].args.contains(&"385907a5".to_owned()));
        assert_eq!(rs[1].args.last().unwrap(), &under_tools("rapx"));
        assert!(r.advice.contains("libz3-dev"));
    }

    #[test]
    fn codeql_downloads_the_platform_bundle() {
        let r = recipe("codeql", &tools(), LINUX).unwrap();
        let rs = runs(&r);
        assert_eq!(rs[0].program, "curl");
        let url = rs[0].args.last().unwrap();
        assert!(url.ends_with("/codeql-bundle-linux64.tar.gz"));
        assert!(
            url.contains(&format!("/releases/download/{}/", pins::CODEQL_BUNDLE))
                && !url.contains("/latest/"),
            "the bundle is pinned: {url}"
        );
        assert_eq!(rs[1].program, "tar");
        let m = recipe(
            "codeql",
            &tools(),
            Platform {
                windows: false,
                macos: true,
            },
        )
        .unwrap();
        assert!(
            runs(&m)[0]
                .args
                .last()
                .unwrap()
                .ends_with("codeql-bundle-osx64.tar.gz")
        );
    }

    #[test]
    fn the_go_toolchain_is_fetched_by_the_go_command_into_the_module_cache() {
        let r = recipe(GO_TOOLCHAIN_BACKEND, &tools(), LINUX).unwrap();
        assert_eq!(r.requires, ["go"]);
        let run = runs(&r)[0];
        assert_eq!(run.program, "go");
        assert_eq!(run.args, ["version"]);
        assert_eq!(run.env, [("GOTOOLCHAIN".to_owned(), "go1.27.0".to_owned())]);
        assert_eq!(r.user_command(), "GOTOOLCHAIN=go1.27.0 go version");
        assert_eq!(r.probe, Probe::Fixed("1.27.0".to_owned()));
        // Nothing is written into the tools directory: the module cache holds it.
        assert!(r.steps.iter().all(|s| matches!(s, Step::Run(_))));
    }

    #[test]
    fn sonar_has_no_recipe_and_govet_and_dla_need_nothing() {
        let e = recipe("sonar", &tools(), LINUX).unwrap_err();
        assert!(e.contains("nothing to install"), "{e}");
        assert!(recipe("govet", &tools(), LINUX).unwrap().steps.is_empty());
        assert!(recipe("dla", &tools(), LINUX).unwrap().steps.is_empty());
        assert!(recipe("nope", &tools(), LINUX).is_err());
    }

    #[test]
    fn every_recipe_stays_inside_the_tools_directory() {
        for b in dla_core::detectors::KNOWN_BACKENDS {
            let Ok(r) = recipe(b, &tools(), LINUX) else {
                continue;
            };
            for s in &r.steps {
                match s {
                    Step::MkDir(p) | Step::Write { path: p, .. } => {
                        assert!(p.starts_with("/data/tools"), "{b}: {p:?}");
                    }
                    Step::Run(run) => {
                        let hay = format!("{} {}", run.line(), run.env.len());
                        assert!(!hay.contains("/home/") && !hay.contains("./"), "{b}: {hay}");
                    }
                }
            }
        }
    }

    #[test]
    fn quoting_protects_spaces_and_quotes() {
        assert_eq!(quote("a b"), "'a b'");
        assert_eq!(quote("it's"), "'it'\\''s'");
        assert_eq!(quote("-DoutputDirectory=/x/y"), "-DoutputDirectory=/x/y");
    }

    #[test]
    fn jar_probe_reads_the_version_from_the_file_name() {
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("detekt-cli-2.0.0-alpha.6-all.jar"), "").unwrap();
        let v = read_version(
            &Probe::Jar(d.path().to_path_buf(), "detekt-cli".into()),
            &ToolEnv::default(),
        );
        assert_eq!(v.as_deref(), Some("2.0.0-alpha.6"));
        let d = tempfile::tempdir().unwrap();
        fs::write(d.path().join("error_prone_core-2.52.0.jar"), "").unwrap();
        let v = read_version(
            &Probe::Jar(d.path().to_path_buf(), "error_prone_core".into()),
            &ToolEnv::default(),
        );
        assert_eq!(v.as_deref(), Some("2.52.0"));
    }

    #[test]
    fn a_failed_install_leaves_the_sonar_row_blocked_and_marks_other_backends_missing() {
        use dla_core::coverage::{BackendCoverage, BackendStatus, CoverageManifest};
        let d = tempfile::tempdir().unwrap();
        let ctx = ToolCtx {
            project: d.path().to_path_buf(),
            run_dir: d.path().to_path_buf(),
            run_id: "r1".into(),
            source_commit: "c".into(),
            data_flag: None,
            worker: None,
        };
        CoverageManifest::update(&ctx.run_dir, |m| {
            m.set_backend("jvm", "sonar", BackendCoverage::blocked("server-only: x"));
        })
        .unwrap();
        keep_missing(&ctx, "sonar", "nothing to install").unwrap();
        let m = CoverageManifest::load(&ctx.run_dir).unwrap().unwrap();
        let row = &m.workers["jvm"].backends["sonar"];
        assert_eq!(row.status, BackendStatus::Blocked, "{row:?}");
        assert_eq!(row.install_hint, None);
        // Any other backend that is not installed becomes a gap with its hint.
        keep_missing(&ctx, "spotbugs", "install failed").unwrap();
        let m = CoverageManifest::load(&ctx.run_dir).unwrap().unwrap();
        assert_eq!(
            m.workers["jvm"].backends["spotbugs"].status,
            BackendStatus::Missing
        );
    }
}
