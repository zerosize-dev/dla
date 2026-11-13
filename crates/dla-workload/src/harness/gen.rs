//! `dla harness gen` (spec 4.2).
//!
//! Writes a harness into `.dla/<run>/harness/H<n>/`: a scratch copy of the project, the
//! per-runtime agent (compiled here when it needs a compiler), the probe list, in driver
//! mode the loop-test template, and the manifest `dla harness run` executes. It runs no
//! project code and needs no consent; compiling DLA's own agent and shim is not running the
//! project.
//!
//! Refusals: `workload-required` (no main or script workload with an iteration argument; the
//! message lists the candidates), `iteration-arg-unknown` (the chosen workload shows none and
//! `--iter-arg` was not given), `tool-missing` (no `javac`/`jar`/`cc`; the fix names the
//! install), `driver-unavailable` (Rust bin-only crate in driver mode), `tool-failed` (the
//! agent did not compile).

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};

use dla_core::{
    FindingId, Refusal,
    error::{Error, codes},
    evidence::EvidenceKind,
    harness::{CommandSpec, FfiHarness, FfiMethod, HarnessManifest, HarnessMode},
    probe::{FfiBoundary, LibraryProbe},
    runtime::Runtime,
    series::KeyMode,
    toolctx::{EvidenceSpec, StageOutcome, ToolCtx},
    workload::{Workload, WorkloadKind, WorkloadReport},
};
use regex::Regex;
use serde_json::{Value, json};

use super::{
    AGENT, COV, DRIVER, LOGS, OUT, PROBES, WORK, commands, harness_dir, probes, scratch, templates,
};

/// Refusal code: a build tool or compiler is not installed.
pub const TOOL_MISSING: &str = "tool-missing";
/// Refusal code: no usable main or script workload.
pub const WORKLOAD_REQUIRED: &str = "workload-required";
/// Refusal code: the workload shows no iteration argument.
pub const ITERATION_ARG_UNKNOWN: &str = "iteration-arg-unknown";
/// Refusal code: driver mode is not possible for this project.
pub const DRIVER_UNAVAILABLE: &str = "driver-unavailable";

/// Default timeout of one measured process.
const TIMEOUT_SECS: u64 = 300;
/// Default size of the repeated key space of a driver.
const DEFAULT_REPEAT: u64 = 64;
/// Name of the JVM driver class.
const JVM_DRIVER_CLASS: &str = "DlaLoop";
/// Default units (runs of the selected tests) per round of a JUnit repeat harness.
const JUNIT_STEP: u64 = 2;
/// Default timeout of one measured JUnit repeat process: a Spring context start is paid once
/// and every later unit is a repeat.
const JUNIT_TIMEOUT_SECS: u64 = 1800;

/// Options of `dla harness gen`.
#[derive(Debug, Clone)]
pub struct GenOptions {
    /// Runtime of the harness.
    pub lang: Runtime,
    /// Workload ID from discovery (default: the best-ranked main or script workload with an
    /// iteration argument).
    pub workload: Option<String>,
    /// Driver mode instead of command mode.
    pub driver: bool,
    /// Iteration argument template with `{n}`, overriding detection.
    pub iter_arg: Option<String>,
    /// Units per point (command mode: n_k = step * k).
    pub step: Option<u64>,
    /// Post-warm-up points (default 8).
    pub iterations: u32,
    /// Warm-up points (default 1).
    pub warmup: u32,
    /// Findings to add a probe for.
    pub probe_findings: Vec<FindingId>,
    /// Derive probes from census candidates.
    pub auto_probes: bool,
    /// Go/JVM package (directory or name) for driver mode.
    pub package: Option<String>,
    /// JVM: run the project's own JUnit 5 tests in a loop (implies driver mode; no hooks).
    pub junit: bool,
    /// JVM, JUnit repeat: classes (`pkg.Class`) or methods (`pkg.Class#method`) to run, instead
    /// of the workload's test class.
    pub tests: Vec<String>,
}

fn refuse(
    code: &str,
    msg: impl Into<String>,
    missing: impl Into<String>,
    fix: impl Into<String>,
) -> Refusal {
    Refusal::new(code, msg, missing, fix)
}

fn io_refusal(path: &Path, e: std::io::Error) -> Refusal {
    Error::io(path, e).to_refusal()
}

fn write(path: &Path, text: &str) -> Result<(), Refusal> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| io_refusal(dir, e))?;
    }
    fs::write(path, text).map_err(|e| io_refusal(path, e))
}

/// True when `name` is an executable on `PATH`.
pub fn on_path(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        let p = dir.join(name);
        p.is_file() || (cfg!(windows) && dir.join(format!("{name}.exe")).is_file())
    })
}

/// Runs a tool to completion; `Err` carries the tail of its output.
fn run_tool(program: &str, args: &[&str], cwd: &Path) -> Result<(), String> {
    let out = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .output()
        .map_err(|e| format!("{program}: {e}"))?;
    if out.status.success() {
        return Ok(());
    }
    let mut text = String::from_utf8_lossy(&out.stderr).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stdout));
    let tail: Vec<&str> = text.lines().rev().take(8).collect();
    Err(tail.into_iter().rev().collect::<Vec<_>>().join(" | "))
}

fn tool_exit_ok(program: &str, args: &[&str], cwd: &Path, env: &[(&str, &str)]) -> bool {
    Command::new(program)
        .args(args)
        .current_dir(cwd)
        .envs(env.iter().copied())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

// ---------------------------------------------------------------------------------------
// Workload choice
// ---------------------------------------------------------------------------------------

fn load_workloads(ctx: &ToolCtx) -> Result<Option<WorkloadReport>, Refusal> {
    let path = ctx
        .run_dir
        .join("workload")
        .join(crate::report_file(ctx.worker.as_deref()));
    let Ok(text) = fs::read_to_string(&path) else {
        return Ok(None);
    };
    serde_json::from_str(&text).map(Some).map_err(|e| {
        Error::Malformed {
            path,
            message: e.to_string(),
        }
        .to_refusal()
    })
}

fn candidates_text(report: &WorkloadReport, rt: Runtime) -> String {
    let c: Vec<String> = report
        .workloads
        .iter()
        .filter(|w| w.runtime == rt)
        .map(|w| {
            format!(
                "{} ({}{})",
                w.id,
                w.kind.as_str(),
                w.iteration_arg
                    .as_deref()
                    .map_or(", no iteration argument".to_owned(), |a| format!(", {a}"))
            )
        })
        .collect();
    if c.is_empty() {
        format!("no {} workloads", rt.as_str())
    } else {
        c.join("; ")
    }
}

/// Picks the workload: the named one, else the best-ranked main or script of `rt` with an
/// iteration argument (or any main or script when `need_iter` is false).
fn choose_workload(
    ctx: &ToolCtx,
    report: Option<&WorkloadReport>,
    opts: &GenOptions,
    need_iter: bool,
) -> Result<Option<Workload>, Refusal> {
    let rt = opts.lang;
    let lang = rt.as_str();
    if opts.junit {
        junit_lang_check(opts)?;
    }
    let Some(report) = report else {
        if opts.driver && opts.workload.is_none() && !opts.junit {
            return Ok(None);
        }
        return Err(refuse(
            codes::MISSING_EVIDENCE,
            "no workload discovery result in this run",
            format!("workload/{}", crate::report_file(ctx.worker.as_deref())),
            format!("dla workload discover --lang {lang}"),
        ));
    };
    if let Some(id) = &opts.workload {
        let w = report
            .workloads
            .iter()
            .find(|w| &w.id == id)
            .ok_or_else(|| {
                refuse(
                    codes::INVALID_VALUE,
                    format!("workload {id} is not in the discovery result"),
                    format!("workload {id}"),
                    format!(
                        "dla harness gen --lang {lang} --workload <one of: {}>",
                        candidates_text(report, rt)
                    ),
                )
            })?;
        if w.runtime != rt {
            return Err(refuse(
                codes::INVALID_VALUE,
                format!(
                    "workload {id} is a {} workload, not {lang}",
                    w.runtime.as_str()
                ),
                format!("a {lang} workload"),
                format!(
                    "dla harness gen --lang {} --workload {id}",
                    w.runtime.as_str()
                ),
            ));
        }
        return Ok(Some(w.clone()));
    }
    if opts.junit {
        return pick_junit_workload(ctx, report, opts).map(Some);
    }
    let pick = report.workloads.iter().find(|w| {
        w.runtime == rt
            && matches!(w.kind, WorkloadKind::Main | WorkloadKind::Script)
            && (!need_iter || w.iteration_arg.is_some() || opts.iter_arg.is_some())
    });
    match pick {
        Some(w) => Ok(Some(w.clone())),
        None if opts.driver => Ok(report
            .workloads
            .iter()
            .find(|w| {
                w.runtime == rt && matches!(w.kind, WorkloadKind::Main | WorkloadKind::Script)
            })
            .cloned()),
        None => {
            // A project of JUnit 5 tests is the case --junit is for: name it first.
            let junit = rt == Runtime::Jvm
                && report
                    .workloads
                    .iter()
                    .any(|w| w.runtime == rt && junit_problem(&ctx.project, w).is_none());
            let junit_fix = if junit {
                "dla harness gen --lang jvm --junit   (runs your JUnit 5 tests in a loop; nothing to fill)   or   "
            } else {
                ""
            };
            Err(refuse(
                WORKLOAD_REQUIRED,
                format!(
                    "no {lang} main or script workload with an iteration argument; candidates: {}",
                    candidates_text(report, rt)
                ),
                format!("a {lang} workload that takes an iteration count"),
                format!(
                    "{junit_fix}dla harness gen --lang {lang} --workload <id> --iter-arg \"--iterations {{n}}\"   or   dla harness gen --lang {lang} --driver"
                ),
            ))
        }
    }
}

// ---------------------------------------------------------------------------------------
// JUnit repeat
// ---------------------------------------------------------------------------------------

/// The build tool a discovered JVM test workload runs through (`mvn ...`, `./gradlew ...`).
fn junit_tool(w: &Workload) -> Option<commands::JunitTool> {
    let first = w.command.split_whitespace().next()?;
    let name = first.rsplit('/').next().unwrap_or(first);
    if name.starts_with("mvn") {
        Some(commands::JunitTool::Maven)
    } else if name.starts_with("gradle") {
        Some(commands::JunitTool::Gradle)
    } else {
        None
    }
}

/// Why workload `w` cannot be run by the JUnit repeat driver; `None` when it can: a JVM test
/// of a Gradle or Maven project whose file uses the JUnit 5 (`org.junit.jupiter`) API.
fn junit_problem(project: &Path, w: &Workload) -> Option<String> {
    if w.runtime != Runtime::Jvm || w.kind != WorkloadKind::Test {
        return Some(format!(
            "workload {} is a {} {} workload, not a JVM test",
            w.id,
            w.runtime.as_str(),
            w.kind.as_str()
        ));
    }
    if junit_tool(w).is_none() {
        return Some(format!(
            "workload {} does not run through Maven or Gradle (command: {}), so there is no build tool to resolve the test class path",
            w.id, w.command
        ));
    }
    let Ok(text) = fs::read_to_string(project.join(&w.file)) else {
        return Some(format!("{} cannot be read", w.file));
    };
    // `kotlin.test` runs on JUnit 5 when the project uses the kotlin-test-junit5 backend; the
    // build step checks that a junit-platform-engine is on the class path.
    if text.contains("org.junit.jupiter")
        || (text.contains("kotlin.test") && !text.contains("org.junit.Test"))
    {
        return None;
    }
    if text.contains("org.junit.Test") || text.contains("org.testng") {
        Some(format!(
            "{} uses JUnit 4 or TestNG, which the Jupiter launcher runs only with the vintage engine (junit-vintage-engine) or not at all",
            w.file
        ))
    } else {
        Some(format!(
            "{} imports neither org.junit.jupiter nor kotlin.test, so it is not a JUnit 5 test",
            w.file
        ))
    }
}

/// `--junit` without `--workload`: the JUnit 5 test workload that holds the first `--tests`
/// class, else the best-ranked one.
fn pick_junit_workload(
    ctx: &ToolCtx,
    report: &WorkloadReport,
    opts: &GenOptions,
) -> Result<Workload, Refusal> {
    let tests: Vec<&Workload> = report
        .workloads
        .iter()
        .filter(|w| w.runtime == Runtime::Jvm && w.kind == WorkloadKind::Test)
        .collect();
    let ok: Vec<&Workload> = tests
        .iter()
        .copied()
        .filter(|w| junit_problem(&ctx.project, w).is_none())
        .collect();
    let wanted = opts.tests.first().map(|t| t.split('#').next().unwrap_or(t));
    let by_class = wanted.and_then(|class| {
        ok.iter().copied().find(|w| {
            fs::read_to_string(ctx.project.join(&w.file))
                .is_ok_and(|text| crate::jvm::fqcn_of(&w.file, &text) == class)
        })
    });
    if let Some(w) = by_class.or_else(|| ok.first().copied()) {
        return Ok(w.clone());
    }
    match tests.first() {
        Some(w) => Err(refuse(
            DRIVER_UNAVAILABLE,
            format!(
                "no JVM test workload can run as a JUnit repeat: {}",
                junit_problem(&ctx.project, w).unwrap_or_default()
            ),
            "a JUnit 5 test in a Gradle or Maven project",
            "dla harness gen --lang jvm --driver   (hooks to fill; the driver runs your own code), or add a JUnit 5 test",
        )),
        None => Err(refuse(
            WORKLOAD_REQUIRED,
            format!(
                "no JVM test workload in this project; candidates: {}",
                candidates_text(report, Runtime::Jvm)
            ),
            "a JUnit 5 test in a Gradle or Maven project",
            "dla workload discover --lang jvm   or   dla harness gen --lang jvm --driver",
        )),
    }
}

/// What the JUnit repeat build and driver are made from.
struct JunitPick {
    tool: commands::JunitTool,
    /// Wrapper script (`gradlew`, `mvnw`) path relative to the project root, when the project
    /// has one at or above the workload's directory.
    wrapper: Option<String>,
    /// Selectors: `pkg.Class` or `pkg.Class#method`.
    select: Vec<String>,
    /// Notes for the manifest.
    gaps: Vec<String>,
}

/// True for a class (`pkg.Class`) or method (`pkg.Class#method`) selector the driver can embed.
fn valid_selector(t: &str) -> bool {
    let (class, method) = t.split_once('#').map_or((t, None), |(c, m)| (c, Some(m)));
    let word = |s: &str, dots: bool| {
        !s.is_empty()
            && !s.starts_with(|c: char| c.is_ascii_digit() || c == '.')
            && !s.ends_with('.')
            && s.chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '$' || (dots && c == '.'))
    };
    // A method name may hold spaces and punctuation (a Kotlin backtick test,
    // fun `publishes to the broker`()): the JVM accepts them, and they are safe inside the
    // Java string literal the selector is embedded in. Quotes, backslashes and control
    // characters are not.
    let name_ok = |s: &str| {
        !s.is_empty()
            && !s.starts_with(char::is_whitespace)
            && !s.ends_with(char::is_whitespace)
            && !s
                .chars()
                .any(|c| c.is_control() || matches!(c, '"' | '\\' | '#' | '('))
    };
    // `method(java.lang.String, int)`: the parameter types of a parameterized test, which is
    // selected by its full signature.
    let method_ok = |m: &str| match m.split_once('(') {
        None => name_ok(m),
        Some((name, rest)) => {
            name_ok(name)
                && rest.strip_suffix(')').is_some_and(|params| {
                    params.chars().all(|c| {
                        c.is_alphanumeric() || matches!(c, '_' | '$' | '.' | ',' | '[' | ']' | ' ')
                    })
                })
        }
    };
    word(class, true) && method.is_none_or(method_ok)
}

/// `--junit` is for JVM projects.
fn junit_lang_check(opts: &GenOptions) -> Result<(), Refusal> {
    if opts.lang == Runtime::Jvm {
        return Ok(());
    }
    Err(refuse(
        codes::INVALID_VALUE,
        format!("--junit is for JVM projects, not {}", opts.lang.as_str()),
        "--lang jvm",
        "dla harness gen --lang jvm --junit",
    ))
}

/// Decides whether this harness is a JUnit repeat and, if so, what it runs. `Ok(None)` for
/// every other harness.
fn junit_pick(
    ctx: &ToolCtx,
    opts: &GenOptions,
    w: Option<&Workload>,
) -> Result<Option<JunitPick>, Refusal> {
    // Only `--junit` makes a JUnit repeat: an explicit `--driver` is always the plain driver
    // with hooks, also when its workload is a JUnit 5 test file.
    if !opts.junit {
        if let Some(t) = opts.tests.first() {
            return Err(refuse(
                codes::INVALID_VALUE,
                format!("--tests {t} applies to the JUnit repeat driver only"),
                "--junit",
                format!("dla harness gen --lang jvm --junit --tests {t}"),
            ));
        }
        return Ok(None);
    }
    junit_lang_check(opts)?;
    let Some(w) = w else {
        return Err(refuse(
            WORKLOAD_REQUIRED,
            "no JVM test workload to run as a JUnit repeat",
            "a JUnit 5 test workload",
            "dla workload discover --lang jvm",
        ));
    };
    if let Some(why) = junit_problem(&ctx.project, w) {
        return Err(refuse(
            DRIVER_UNAVAILABLE,
            format!("cannot run {} as a JUnit repeat: {why}", w.id),
            "a JUnit 5 test in a Gradle or Maven project",
            "dla harness gen --lang jvm --driver --workload <a Main or Script workload>   (hooks to fill)",
        ));
    }
    let tool = junit_tool(w).ok_or_else(|| {
        refuse(
            DRIVER_UNAVAILABLE,
            format!("{} has no Maven or Gradle build", w.id),
            "pom.xml or build.gradle",
            "dla harness gen --lang jvm --driver",
        )
    })?;
    for t in &opts.tests {
        if !valid_selector(t) {
            return Err(refuse(
                codes::INVALID_VALUE,
                format!("--tests {t:?} is not a class name or Class#method"),
                "fully qualified class names, optionally with #method or #method(parameter types)",
                "dla harness gen --lang jvm --junit --tests pkg.FooTest,pkg.BarTest#touches",
            ));
        }
    }
    let text = fs::read_to_string(ctx.project.join(&w.file)).unwrap_or_default();
    let select = if opts.tests.is_empty() {
        vec![crate::jvm::fqcn_of(&w.file, &text)]
    } else {
        opts.tests.clone()
    };
    let wrapper_name = match tool {
        commands::JunitTool::Gradle => "gradlew",
        commands::JunitTool::Maven => "mvnw",
    };
    let wrapper = find_wrapper(&ctx.project, &w.cwd, wrapper_name);
    let program = match tool {
        commands::JunitTool::Gradle => "gradle",
        commands::JunitTool::Maven => "mvn",
    };
    if wrapper.is_none() && !on_path(program) {
        return Err(refuse(
            TOOL_MISSING,
            format!(
                "{program} is not installed and the project has no {wrapper_name}, so the tests' class path cannot be resolved"
            ),
            program,
            format!(
                "install {program} (or add the project's {wrapper_name}), then rerun dla harness gen --lang jvm {}",
                if opts.junit { "--junit" } else { "--driver" }
            ),
        ));
    }
    let mut gaps = vec![
        format!(
            "JUnit repeat: each unit runs {} through the JUnit Platform Launcher in one JVM; the build step resolves the test class path with {program} and adds junit-platform-launcher of the project's junit-platform-engine version when the project does not declare it",
            select.join(", ")
        ),
        "the tests start whatever the project's tests start: a unit that needs a service that is not running (database, broker, cache) fails, and the driver's last stderr lines name the first failure".to_owned(),
        "a framework that caches state between units (the Spring test context) keeps it across units, so growth comes from the code under test; a test that discards the cache (@DirtiesContext) rebuilds it every unit".to_owned(),
    ];
    if cfg!(windows) {
        gaps.push("the build step is a sh script (Git Bash)".to_owned());
    }
    if text.contains("@DirtiesContext") {
        gaps.push(format!(
            "{} uses @DirtiesContext: its Spring context is rebuilt every unit, so the series includes that cost",
            w.file
        ));
    }
    Ok(Some(JunitPick {
        tool,
        wrapper,
        select,
        gaps,
    }))
}

/// The build tool and wrapper of a plain JVM driver (`--driver`, no `--junit`) on a Gradle or
/// Maven project, whose build resolves the driver's class path (the main runtime class path:
/// the project's classes of every language, resources and the dependency jars). `None` for a
/// project built by a script (`build.sh`) or without a build tool, and, with a note pushed to
/// `gaps`, when the build tool is not installed and the project has no wrapper: the driver then
/// keeps the workload's classes directory.
fn driver_build_pick(
    project: &Path,
    w: Option<&Workload>,
    gaps: &mut Vec<String>,
) -> Option<JunitPick> {
    let (tool, cwd) = match w {
        Some(w) if w.command.contains("build.sh") => return None,
        Some(w) => (junit_tool(w)?, w.cwd.as_str()),
        None => {
            let has = |n: &str| project.join(n).is_file();
            let tool = if has("pom.xml") {
                commands::JunitTool::Maven
            } else if has("build.gradle")
                || has("build.gradle.kts")
                || has("settings.gradle")
                || has("settings.gradle.kts")
            {
                commands::JunitTool::Gradle
            } else {
                return None;
            };
            (tool, ".")
        }
    };
    let (wrapper_name, program) = match tool {
        commands::JunitTool::Gradle => ("gradlew", "gradle"),
        commands::JunitTool::Maven => ("mvnw", "mvn"),
    };
    let wrapper = find_wrapper(project, cwd, wrapper_name);
    if wrapper.is_none() && !on_path(program) {
        gaps.push(format!(
            "{program} is not installed and the project has no {wrapper_name}: the driver's class path is the workload's classes directory, not the project's resolved runtime class path"
        ));
        return None;
    }
    Some(JunitPick {
        tool,
        wrapper,
        select: Vec::new(),
        gaps: vec![format!(
            "the driver's class path is resolved by the project's build ({program}: the main runtime class path, its classes of every language and the dependency jars) when dla harness run builds the harness; the build step tries offline first and retries online, downloading dependencies into the build tool's own cache"
        )],
    })
}

/// The build tool executable or wrapper script of `j` as the build step takes it.
fn launcher_of(j: &JunitPick, work: &Path) -> String {
    j.wrapper.as_ref().map_or_else(
        || match j.tool {
            commands::JunitTool::Gradle => "gradle".to_owned(),
            commands::JunitTool::Maven => "mvn".to_owned(),
        },
        |rel| dla_core::paths::to_slash(&work.join(rel)),
    )
}

/// The wrapper script `name` in the workload's directory or the nearest parent, as a path
/// relative to the project root.
fn find_wrapper(project: &Path, cwd: &str, name: &str) -> Option<String> {
    let mut dir: Vec<&str> = cwd
        .split('/')
        .filter(|c| !c.is_empty() && *c != ".")
        .collect();
    loop {
        let rel = if dir.is_empty() {
            name.to_owned()
        } else {
            format!("{}/{name}", dir.join("/"))
        };
        if project.join(&rel).is_file() {
            return Some(rel);
        }
        dir.pop()?;
    }
}

// ---------------------------------------------------------------------------------------
// Libraries and detector flags
// ---------------------------------------------------------------------------------------

/// What the probe's library flags turn into.
#[derive(Debug, Default)]
struct Detectors {
    jvm_options: Vec<String>,
    env: Vec<(String, String)>,
    /// Flags recorded in the manifest as switched on.
    on: Vec<String>,
    /// Notes for flags that cannot be passed from outside.
    notes: Vec<String>,
    goleak: bool,
}

fn detectors(ctx: &ToolCtx, rt: Runtime) -> Detectors {
    let mut d = Detectors::default();
    // Only the libraries are read: a probe report of another shape must not lose them.
    let libraries: Vec<LibraryProbe> = fs::read_to_string(crate::probe_path(ctx))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|doc| doc.get("libraries").cloned())
        .and_then(|l| serde_json::from_value(l).ok())
        .unwrap_or_default();
    for lib in libraries.iter().filter(|l| l.runtime == rt) {
        for flag in &lib.harness_flags {
            match flag.as_str() {
                f if f.starts_with("-D") && rt == Runtime::Jvm => {
                    d.jvm_options.push(f.to_owned());
                    d.on.push(f.to_owned());
                }
                "PYTHONASYNCIODEBUG=1" => {
                    d.env.push(("PYTHONASYNCIODEBUG".into(), "1".into()));
                    d.on.push(flag.clone());
                }
                "-X dev" => {
                    d.env.push(("PYTHONDEVMODE".into(), "1".into()));
                    d.on.push(flag.clone());
                }
                f if f.starts_with("-W ") => d.on.push(f.to_owned()),
                f if f.starts_with("goleak.") => d.goleak = true,
                f => d.notes.push(format!(
                    "note: {} flag {f:?} cannot be switched on from outside the process (set it in the project's own configuration)",
                    lib.library
                )),
            }
        }
    }
    if rt == Runtime::Node {
        d.on.push("MaxListenersExceededWarning".to_owned());
    }
    d.on.dedup();
    d
}

// ---------------------------------------------------------------------------------------
// Agents
// ---------------------------------------------------------------------------------------

fn compile_jvm_agent(agent: &Path) -> Result<(), Refusal> {
    for tool in ["javac", "jar"] {
        if !on_path(tool) {
            return Err(refuse(
                TOOL_MISSING,
                format!("{tool} is not installed, so the JVM agent cannot be built"),
                tool,
                "install a JDK 11 or newer (for example `apt-get install default-jdk-headless`) so javac and jar are on PATH, then rerun dla harness gen",
            ));
        }
    }
    let src = agent.join("src");
    write(&src.join("DlaAgent.java"), templates::JVM_AGENT)?;
    write(&src.join("DlaDump.java"), templates::JVM_DUMP)?;
    write(&src.join("DlaCapture.java"), templates::JVM_CAPTURE)?;
    write(&agent.join("MANIFEST.MF"), templates::JVM_MANIFEST)?;
    let classes = agent.join("classes");
    fs::create_dir_all(&classes).map_err(|e| io_refusal(&classes, e))?;
    let failed = |what: &str, why: String| {
        refuse(
            codes::TOOL_FAILED,
            format!("{what} failed: {why}"),
            "the JVM agent build",
            "dla harness gen needs a JDK 11 or newer; check `javac -version`",
        )
    };
    run_tool(
        "javac",
        &[
            "-nowarn",
            "-d",
            "classes",
            "src/DlaAgent.java",
            "src/DlaDump.java",
            "src/DlaCapture.java",
        ],
        agent,
    )
    .map_err(|e| failed("javac", e))?;
    run_tool(
        "jar",
        &[
            "--create",
            "--file",
            "dla-agent.jar",
            "--manifest",
            "MANIFEST.MF",
            "-C",
            "classes",
            ".",
        ],
        agent,
    )
    .map_err(|e| failed("jar", e))
}

fn compile_shim(agent: &Path) -> Result<(), Refusal> {
    if !on_path("cc") {
        return Err(refuse(
            TOOL_MISSING,
            "cc is not installed, so the allocator shim cannot be built",
            "cc (a C compiler)",
            "install a C compiler (for example `apt-get install build-essential`) so cc is on PATH, then rerun dla harness gen",
        ));
    }
    write(&agent.join("dla_alloc_shim.c"), templates::NATIVE_SHIM)?;
    let out = commands::shim_file();
    let args: Vec<&str> = if cfg!(target_os = "macos") {
        vec![
            "-std=gnu11",
            "-O2",
            "-g",
            "-dynamiclib",
            "-o",
            out,
            "dla_alloc_shim.c",
        ]
    } else {
        vec![
            "-std=gnu11",
            "-O2",
            "-g",
            "-fPIC",
            "-shared",
            "-fno-optimize-sibling-calls",
            "-o",
            out,
            "dla_alloc_shim.c",
            "-ldl",
            "-lpthread",
        ]
    };
    run_tool("cc", &args, agent).map_err(|e| {
        refuse(
            codes::TOOL_FAILED,
            format!("cc failed to build the allocator shim: {e}"),
            "the allocator shim build",
            "check `cc --version`; the shim needs a C11 compiler with dlsym",
        )
    })
}

/// The JNI boundary `dla probe` recorded in the probe report (`boundaries`), if any.
fn jni_boundary(ctx: &ToolCtx) -> Option<FfiBoundary> {
    let doc: Value =
        serde_json::from_str(&fs::read_to_string(crate::probe_path(ctx)).ok()?).ok()?;
    let all: Vec<FfiBoundary> = serde_json::from_value(doc.get("boundaries")?.clone()).ok()?;
    all.into_iter()
        .find(|b| b.kind == "jni" && b.managed == Runtime::Jvm)
}

/// Prepares the JVM harness to measure the native side of a JNI boundary: builds the
/// allocator shim and works out where the built library will be. `Err` is a gap note (the
/// harness still works without the native columns).
fn ffi_setup(
    b: &FfiBoundary,
    agent: &Path,
    work: &Path,
) -> Result<(FfiHarness, commands::FfiSwitch), String> {
    if !cfg!(target_os = "linux") {
        return Err(
            "the project calls a native library through JNI, but the allocator shim is preloaded on Linux only: native growth shows as os.rss only"
                .to_owned(),
        );
    }
    compile_shim(agent).map_err(|r| {
        format!(
            "the project calls a native library through JNI, but the allocator shim could not be built ({}): native growth shows as os.rss only",
            r.message
        )
    })?;
    let mut lib_dirs: Vec<String> = Vec::new();
    if b.cdylib
        && let Some(dir) = &b.native_dir
    {
        lib_dirs = cdylib_target_dirs(work, dir);
    }
    let harness = FfiHarness {
        kind: b.kind.clone(),
        library: b.library.clone(),
        native_dir: b.native_dir.clone(),
        methods: b
            .methods
            .iter()
            .map(|m| FfiMethod {
                export: m.export.clone(),
                managed: m.managed.clone(),
                declared_at: m.declared_at.clone(),
            })
            .collect(),
    };
    Ok((harness, commands::FfiSwitch { lib_dirs }))
}

/// Candidate directories of a built Rust cdylib: the `target` of the crate and of each parent
/// directory up to the work root (a Cargo workspace keeps one `target` at its root), release
/// and debug. They go on `LD_LIBRARY_PATH`; a directory that does not exist is harmless.
fn cdylib_target_dirs(work: &Path, native_dir: &str) -> Vec<String> {
    let mut dir = if native_dir == "." {
        work.to_path_buf()
    } else {
        work.join(native_dir)
    };
    let mut out = Vec::new();
    loop {
        for profile in ["release", "debug"] {
            out.push(
                dir.join("target")
                    .join(profile)
                    .to_string_lossy()
                    .into_owned(),
            );
        }
        if dir == work || !dir.pop() || !dir.starts_with(work) {
            break;
        }
    }
    out
}

/// Writes the agent files of `rt` into `agent` (compiling what needs it).
fn write_agent(rt: Runtime, driver: bool, agent: &Path) -> Result<(), Refusal> {
    match rt {
        Runtime::Jvm => compile_jvm_agent(agent),
        Runtime::Python => {
            write(&agent.join("sitecustomize.py"), templates::PY_AGENT)?;
            write(&agent.join("dla_memray_leaks.py"), templates::PY_MEMRAY)
        }
        Runtime::Node => write(&agent.join("dla-preload.js"), templates::NODE_AGENT),
        Runtime::Go => Ok(()),
        Runtime::Rust | Runtime::Native => {
            if driver || cfg!(windows) {
                Ok(())
            } else {
                compile_shim(agent)
            }
        }
    }
}

// ---------------------------------------------------------------------------------------
// Steps
// ---------------------------------------------------------------------------------------

/// Units per point so that the largest `n` is about twice the workload's default
/// (4000 when the default is unknown), at least 1. Twice the default keeps a point
/// within the workload's own time budget under an agent (tracemalloc and the
/// allocator shim slow a workload several times) while leaving eight post-warm-up
/// points for the gate.
pub fn choose_step(default: Option<u64>, warmup: u32, iterations: u32) -> u64 {
    let points = u64::from(warmup + iterations).max(1);
    let target = default.map_or(4000, |d| d.saturating_mul(2));
    let raw = (target / points).max(1);
    if raw >= 50 { (raw + 5) / 10 * 10 } else { raw }
}

// ---------------------------------------------------------------------------------------
// Driver mode
// ---------------------------------------------------------------------------------------

static PACKAGE_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*package\s+([\w.]+)").expect("static regex"));
static GO_PACKAGE_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*package\s+(\w+)").expect("static regex"));

/// A driver-mode instance: files to write and the plan inputs.
struct DriverBuild {
    /// File the model edits, relative to the harness directory.
    edit: String,
    /// Hooks in it.
    hooks: Vec<String>,
    /// `(driver file, destination relative to the scratch copy)` copied into place before
    /// the build.
    sync: Vec<(PathBuf, String)>,
    /// Plan inputs.
    spec: commands::DriverSpec,
    /// Extra files in the driver directory (relative to the harness directory).
    extra_files: Vec<String>,
    tools: Vec<String>,
    gaps: Vec<String>,
    /// Go: limit GoVar probes to this directory.
    go_dir: Option<String>,
}

fn jvm_package_for(
    ctx: &ToolCtx,
    work: &Path,
    w: Option<&Workload>,
    pkg_opt: Option<&str>,
) -> String {
    if let Some(p) = pkg_opt {
        if p.contains('/') {
            let dir = work.join(p);
            if let Ok(rd) = fs::read_dir(&dir) {
                for e in rd.flatten() {
                    if let Ok(t) = fs::read_to_string(e.path())
                        && let Some(c) = PACKAGE_LINE.captures(&t)
                    {
                        return c[1].to_owned();
                    }
                }
            }
        }
        return p.replace('/', ".");
    }
    if let Some(w) = w
        && let Ok(t) = fs::read_to_string(work.join(&w.file))
        && let Some(c) = PACKAGE_LINE.captures(&t)
    {
        return c[1].to_owned();
    }
    probes::census_entries(ctx, Runtime::Jvm)
        .iter()
        .find_map(|e| {
            let segs: Vec<&str> = e.owner.split('.').collect();
            (segs.len() > 2).then(|| segs[..segs.len() - 2].join("."))
        })
        .unwrap_or_default()
}

fn join_rel(a: &str, b: &str) -> String {
    match (a.is_empty() || a == ".", b.is_empty()) {
        (true, _) => b.to_owned(),
        (_, true) => a.to_owned(),
        _ => format!("{a}/{b}"),
    }
}

fn cargo_lib(work: &Path, cwd: &str) -> Option<String> {
    let dir = work.join(if cwd == "." { "" } else { cwd });
    let manifest: toml::Table = fs::read_to_string(dir.join("Cargo.toml"))
        .ok()?
        .parse()
        .ok()?;
    let has_lib = dir.join("src/lib.rs").is_file() || manifest.contains_key("lib");
    if !has_lib {
        return None;
    }
    let name = manifest
        .get("lib")
        .and_then(|l| l.get("name"))
        .and_then(toml::Value::as_str)
        .or_else(|| {
            manifest
                .get("package")
                .and_then(|p| p.get("name"))
                .and_then(toml::Value::as_str)
        })?;
    Some(name.replace('-', "_"))
}

fn go_package_dir_for(ctx: &ToolCtx, w: Option<&Workload>, pkg_opt: Option<&str>) -> String {
    if let Some(p) = pkg_opt {
        return p.trim_start_matches("./").trim_end_matches('/').to_owned();
    }
    let entries = probes::census_entries(ctx, Runtime::Go);
    let mut counts: std::collections::BTreeMap<String, (u32, u32)> = Default::default();
    for e in &entries {
        let dir = e.site.rsplit_once('/').map_or("", |(d, _)| d);
        let dir = dir.rsplit_once(':').map_or(dir, |(d, _)| d).to_owned();
        let c = counts.entry(dir).or_default();
        c.0 += u32::from(e.kind == "package-var");
        c.1 += 1;
    }
    counts
        .into_iter()
        .max_by_key(|(_, c)| *c)
        .map(|(d, _)| d)
        .or_else(|| w.and_then(|w| commands::go_package_dir(&w.command)))
        .unwrap_or_default()
}

fn pytest_memray_available() -> bool {
    tool_exit_ok(
        "python3",
        &["-c", "import pytest_memray"],
        Path::new("."),
        &[],
    )
}

fn rust_dhat_available() -> bool {
    let home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cargo")));
    let Some(src) = home.map(|h| h.join("registry/src")) else {
        return false;
    };
    fs::read_dir(src).is_ok_and(|rd| {
        rd.flatten().any(|idx| {
            fs::read_dir(idx.path()).is_ok_and(|inner| {
                inner
                    .flatten()
                    .any(|c| c.file_name().to_string_lossy().starts_with("dhat-"))
            })
        })
    })
}

struct DriverCtx<'a> {
    ctx: &'a ToolCtx,
    opts: &'a GenOptions,
    work: &'a Path,
    driver: &'a Path,
    agent: &'a Path,
    workload: Option<&'a Workload>,
    cwd: &'a str,
    step: u64,
    junit: Option<&'a JunitPick>,
    /// A plain JVM driver whose class path the project's build resolves.
    main_build: Option<&'a JunitPick>,
}

fn driver_vars(d: &DriverCtx<'_>) -> [(String, String); 4] {
    [
        ("STEP".into(), d.step.to_string()),
        ("WARMUP".into(), d.opts.warmup.to_string()),
        ("ROUNDS".into(), d.opts.iterations.to_string()),
        ("REPEAT".into(), DEFAULT_REPEAT.to_string()),
    ]
}

fn fill_with(template: &str, base: &[(String, String)]) -> Result<String, Refusal> {
    let vars: Vec<(&str, &str)> = base.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
    templates::fill(template, &vars).map_err(|e| {
        refuse(
            codes::TOOL_FAILED,
            format!("driver template: {e}"),
            "a template without leftover placeholders",
            "report this as a dla bug",
        )
    })
}

/// Writes the driver template of the runtime and returns what the caller needs.
fn build_driver(d: &DriverCtx<'_>) -> Result<DriverBuild, Refusal> {
    let rt = d.opts.lang;
    let mut vars: Vec<(String, String)> = driver_vars(d).into();
    let mut out = DriverBuild {
        edit: String::new(),
        hooks: Vec::new(),
        sync: Vec::new(),
        spec: commands::DriverSpec::default(),
        extra_files: Vec::new(),
        tools: Vec::new(),
        gaps: Vec::new(),
        go_dir: None,
    };
    let driver_rel = |name: &str| format!("{DRIVER}/{name}");
    match rt {
        Runtime::Jvm if d.junit.is_some() => {
            let j = d.junit.ok_or_else(|| {
                refuse(
                    codes::TOOL_FAILED,
                    "JUnit repeat without a selection",
                    "a JUnit selection",
                    "report this as a dla bug",
                )
            })?;
            let select = j
                .select
                .iter()
                .map(|t| format!("\"{t}\""))
                .collect::<Vec<_>>()
                .join(", ");
            vars.push(("SELECT".into(), select));
            let text = fill_with(templates::JVM_JUNIT_DRIVER, &vars)?;
            let file = format!("{}.java", commands::JUNIT_DRIVER_CLASS);
            write(&d.driver.join(&file), &text)?;
            write(
                &d.driver.join(commands::JUNIT_INIT_SCRIPT),
                templates::JVM_JUNIT_INIT,
            )?;
            write(
                &d.driver.join(commands::JUNIT_BUILD_SCRIPT),
                templates::JVM_JUNIT_BUILD,
            )?;
            out.hooks = templates::hooks(&text);
            // Nothing to edit: the driver runs the project's own tests.
            let launcher = launcher_of(j, d.work);
            out.spec.junit = Some(commands::JunitSpec {
                tool: j.tool,
                launcher,
            });
            out.tools.push("junit-platform-launcher".to_owned());
            out.tools.push(
                match j.tool {
                    commands::JunitTool::Gradle => "gradle",
                    commands::JunitTool::Maven => "maven",
                }
                .to_owned(),
            );
            out.gaps.extend(j.gaps.iter().cloned());
        }
        Runtime::Jvm => {
            let package = jvm_package_for(d.ctx, d.work, d.workload, d.opts.package.as_deref());
            vars.push(("PACKAGE".into(), package.clone()));
            vars.push(("CLASS".into(), JVM_DRIVER_CLASS.into()));
            let text = fill_with(templates::JVM_DRIVER, &vars)?;
            let file = format!("{JVM_DRIVER_CLASS}.java");
            write(&d.driver.join(&file), &text)?;
            out.hooks = templates::hooks(&text);
            out.edit = driver_rel(&file);
            // Compiled from the driver directory against the project's classes: the
            // project's own build never sees it, and package-private API is still usable
            // because the class is declared in the workload's package.
            out.spec.work_rel = dla_core::paths::to_slash(&d.driver.join(&file));
            out.spec.target = if package.is_empty() {
                JVM_DRIVER_CLASS.to_owned()
            } else {
                format!("{package}.{JVM_DRIVER_CLASS}")
            };
            let _ = d.agent;
            if let Some(m) = d.main_build {
                // The project's build resolves the class path: the same build step as the
                // JUnit repeat, in its main mode.
                write(
                    &d.driver.join(commands::JUNIT_INIT_SCRIPT),
                    templates::JVM_JUNIT_INIT,
                )?;
                write(
                    &d.driver.join(commands::JUNIT_BUILD_SCRIPT),
                    templates::JVM_JUNIT_BUILD,
                )?;
                out.spec.main_build = Some(commands::JunitSpec {
                    tool: m.tool,
                    launcher: launcher_of(m, d.work),
                });
                out.tools.push(
                    match m.tool {
                        commands::JunitTool::Gradle => "gradle",
                        commands::JunitTool::Maven => "maven",
                    }
                    .to_owned(),
                );
                out.gaps.extend(m.gaps.iter().cloned());
            }
        }
        Runtime::Python => {
            let text = fill_with(templates::PY_DRIVER, &vars)?;
            write(&d.driver.join("dla_harness.py"), &text)?;
            out.hooks = templates::hooks(&text);
            out.edit = driver_rel("dla_harness.py");
            if pytest_memray_available() {
                let t = fill_with(templates::PY_PYTEST, &vars)?;
                write(&d.driver.join("test_dla_harness.py"), &t)?;
                out.extra_files.push(driver_rel("test_dla_harness.py"));
                out.tools.push("pytest-memray".to_owned());
            } else {
                out.gaps
                    .push("pytest-memray not installed: standard-library driver used".to_owned());
            }
        }
        Runtime::Node => {
            let text = fill_with(templates::NODE_DRIVER, &vars)?;
            write(&d.driver.join("dla-harness.js"), &text)?;
            out.hooks = templates::hooks(&text);
            out.edit = driver_rel("dla-harness.js");
        }
        Runtime::Go => {
            let dir = go_package_dir_for(d.ctx, d.workload, d.opts.package.as_deref());
            let abs_dir = d.work.join(join_rel(d.cwd, &dir));
            let package = fs::read_dir(&abs_dir)
                .ok()
                .into_iter()
                .flat_map(|rd| rd.flatten())
                .filter(|e| {
                    let n = e.file_name().to_string_lossy().into_owned();
                    n.ends_with(".go") && !n.ends_with("_test.go")
                })
                .find_map(|e| {
                    fs::read_to_string(e.path())
                        .ok()
                        .and_then(|t| GO_PACKAGE_LINE.captures(&t).map(|c| c[1].to_owned()))
                })
                .ok_or_else(|| {
                    refuse(
                        codes::INVALID_VALUE,
                        format!("{dir:?} holds no Go package to drive"),
                        "a package directory with .go files",
                        "dla harness gen --lang go --driver --package <dir>",
                    )
                })?;
            vars.push(("PACKAGE".into(), package));
            let text = fill_with(templates::GO_DRIVER, &vars)?;
            write(&d.driver.join(templates::GO_DRIVER_FILE), &text)?;
            out.hooks = templates::hooks(&text);
            out.edit = driver_rel(templates::GO_DRIVER_FILE);
            let dest_dir = join_rel(d.cwd, &dir);
            out.sync.push((
                d.driver.join(templates::GO_DRIVER_FILE),
                join_rel(&dest_dir, templates::GO_DRIVER_FILE),
            ));
            out.spec.target = dir.clone();
            out.go_dir = Some(dir);
            let goleak = tool_exit_ok(
                "go",
                &["list", "-m", "go.uber.org/goleak"],
                &d.work.join(if d.cwd == "." { "" } else { d.cwd }),
                &[("GOFLAGS", "-mod=mod"), ("GOPROXY", "off")],
            );
            if goleak {
                let t = fill_with(templates::GO_GOLEAK, &vars)?;
                write(&d.driver.join(templates::GO_GOLEAK_FILE), &t)?;
                out.sync.push((
                    d.driver.join(templates::GO_GOLEAK_FILE),
                    join_rel(&dest_dir, templates::GO_GOLEAK_FILE),
                ));
                out.spec.goleak = true;
                out.tools.push("goleak".to_owned());
            } else {
                out.gaps
                    .push("goleak not available offline: the goleak check is skipped".to_owned());
            }
        }
        Runtime::Rust => {
            let cwd = d.cwd;
            let Some(krate) = cargo_lib(d.work, cwd) else {
                return Err(refuse(
                    DRIVER_UNAVAILABLE,
                    "driver mode needs a crate with a library target (the driver imports it), and this crate has only binaries",
                    "a [lib] target or src/lib.rs",
                    "dla harness gen --lang rust   (command mode, which measures the binary with the allocator shim)",
                ));
            };
            vars.push(("CRATE".into(), krate));
            let text = fill_with(templates::RUST_DRIVER, &vars)?;
            write(&d.driver.join("dla_harness.rs"), &text)?;
            out.hooks = templates::hooks(&text);
            out.edit = driver_rel("dla_harness.rs");
            out.sync.push((
                d.driver.join("dla_harness.rs"),
                join_rel(&join_rel(cwd, "src/bin"), "dla_harness.rs"),
            ));
            out.spec.crate_dir = cwd.to_owned();
            if !rust_dhat_available() {
                out.gaps.push(
                    "dhat-rs not available offline: counting-allocator driver used".to_owned(),
                );
            }
        }
        Runtime::Native => {
            let text = fill_with(templates::NATIVE_DRIVER, &vars)?;
            write(&d.driver.join("dla_harness.c"), &text)?;
            out.hooks = templates::hooks(&text);
            out.edit = driver_rel("dla_harness.c");
        }
    }
    // Place copies where the build needs them now too, so the scratch copy is complete.
    for (src, dest) in &out.sync {
        let to = d.work.join(dest);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|e| io_refusal(parent, e))?;
        }
        fs::copy(src, &to).map_err(|e| io_refusal(&to, e))?;
    }
    Ok(out)
}

fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "/._-=:@%+,".contains(c))
    {
        s.to_owned()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}

/// `cp` of the driver files into the scratch copy followed by the original build.
fn with_sync(
    build: Option<CommandSpec>,
    sync: &[(PathBuf, String)],
    work: &Path,
) -> Option<CommandSpec> {
    if sync.is_empty() {
        return build;
    }
    let copies: Vec<String> = sync
        .iter()
        .map(|(src, dest)| {
            format!(
                "cp -f {} {}",
                shell_quote(&dla_core::paths::to_slash(src)),
                shell_quote(&dla_core::paths::to_slash(&work.join(dest)))
            )
        })
        .collect();
    let mut b = build?;
    let original = match b.argv.as_slice() {
        [sh, c, body] if sh == "sh" && c == "-c" => body.clone(),
        argv => argv
            .iter()
            .map(|a| shell_quote(a))
            .collect::<Vec<_>>()
            .join(" "),
    };
    b.argv = vec![
        "sh".into(),
        "-c".into(),
        format!("{} && {original}", copies.join(" && ")),
    ];
    Some(b)
}

// ---------------------------------------------------------------------------------------
// generate
// ---------------------------------------------------------------------------------------

fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_owned()
    } else {
        let mut t: String = s.chars().take(n).collect();
        t.push_str("...");
        t
    }
}

/// A command line for the evidence and the CLI output, with the harness directory shortened
/// to `<H>`.
fn cmd_text(c: &CommandSpec, hdir: &Path) -> String {
    let h = dla_core::paths::to_slash(hdir);
    clip(&c.argv.join(" ").replace(&h, "<H>"), 160)
}

/// Removes a half-written harness directory when generation fails.
struct Cleanup {
    dir: PathBuf,
    armed: bool,
}

impl Drop for Cleanup {
    fn drop(&mut self) {
        if self.armed {
            let _ = fs::remove_dir_all(&self.dir);
        }
    }
}

/// Writes a harness and its manifest; see the module docs.
pub fn generate(ctx: &ToolCtx, opts: &GenOptions) -> Result<StageOutcome, Refusal> {
    let junit_opts;
    let opts = if opts.junit && !opts.driver {
        junit_opts = GenOptions {
            driver: true,
            ..opts.clone()
        };
        &junit_opts
    } else {
        opts
    };
    let rt = opts.lang;
    let lang = rt.as_str();
    if !opts.junit
        && !opts.driver
        && let Some(t) = opts.tests.first()
    {
        return Err(refuse(
            codes::INVALID_VALUE,
            format!("--tests {t} applies to the JUnit repeat driver only"),
            "--junit",
            format!("dla harness gen --lang jvm --junit --tests {t}"),
        ));
    }
    if opts.iterations == 0 {
        return Err(refuse(
            codes::INVALID_VALUE,
            "--iterations must be at least 1",
            "a positive --iterations",
            format!("dla harness gen --lang {lang} --iterations 8"),
        ));
    }
    if let Some(t) = &opts.iter_arg
        && !t.contains("{n}")
    {
        return Err(refuse(
            codes::INVALID_VALUE,
            format!("--iter-arg {t:?} has no {{n}} placeholder"),
            "{n} in --iter-arg",
            format!("dla harness gen --lang {lang} --iter-arg \"--iterations {{n}}\""),
        ));
    }
    let report = load_workloads(ctx)?;
    let workload = choose_workload(ctx, report.as_ref(), opts, !opts.driver)?;
    let junit = junit_pick(ctx, opts, workload.as_ref())?;
    // A hook driver on a project that has JUnit 5 tests: name the variant with nothing to fill.
    let junit_hint = (junit.is_none()
        && rt == Runtime::Jvm
        && report.as_ref().is_some_and(|r| {
            r.workloads
                .iter()
                .any(|w| w.runtime == rt && junit_problem(&ctx.project, w).is_none())
        }))
    .then_some(
        "this project has JUnit 5 tests: dla harness gen --lang jvm --junit runs them in a loop with nothing to fill",
    );
    let iter_arg = opts
        .iter_arg
        .clone()
        .or_else(|| workload.as_ref().and_then(|w| w.iteration_arg.clone()));
    if !opts.driver && iter_arg.is_none() {
        let id = workload.as_ref().map_or("<id>", |w| w.id.as_str());
        return Err(refuse(
            ITERATION_ARG_UNKNOWN,
            format!("workload {id} shows no way to pass an iteration count"),
            "an iteration argument",
            format!(
                "dla harness gen --lang {lang} --workload {id} --iter-arg \"--iterations {{n}}\"   (the argument your program takes, with {{n}} for the count), or --driver"
            ),
        ));
    }
    if !opts.driver && workload.is_none() {
        return Err(refuse(
            WORKLOAD_REQUIRED,
            format!("no {lang} workload to run"),
            format!("a {lang} workload"),
            format!("dla workload discover --lang {lang}"),
        ));
    }

    // Tools first: nothing is created when one is missing.
    if rt == Runtime::Jvm {
        for tool in ["javac", "jar"] {
            if !on_path(tool) {
                return Err(refuse(
                    TOOL_MISSING,
                    format!("{tool} is not installed, so the JVM agent cannot be built"),
                    tool,
                    "install a JDK 11 or newer (for example `apt-get install default-jdk-headless`) so javac and jar are on PATH, then rerun dla harness gen",
                ));
            }
        }
    }
    if matches!(rt, Runtime::Rust | Runtime::Native)
        && !opts.driver
        && !cfg!(windows)
        && !on_path("cc")
    {
        return Err(refuse(
            TOOL_MISSING,
            "cc is not installed, so the allocator shim cannot be built",
            "cc (a C compiler)",
            "install a C compiler (for example `apt-get install build-essential`) so cc is on PATH, then rerun dla harness gen",
        ));
    }

    let id = ctx.next_harness_id();
    let hdir = harness_dir(&ctx.run_dir, &id);
    let mut cleanup = Cleanup {
        dir: hdir.clone(),
        armed: true,
    };
    let (work, agent, driver) = (hdir.join(WORK), hdir.join(AGENT), hdir.join(DRIVER));
    for d in [
        &work,
        &agent,
        &driver,
        &hdir.join(OUT),
        &hdir.join(LOGS),
        &hdir.join(COV),
    ] {
        fs::create_dir_all(d).map_err(|e| io_refusal(d, e))?;
    }

    let scope = ctx.scope()?;
    let copied = scratch::copy_project_scoped(&ctx.project, &work, scope.as_ref())
        .map_err(|e| io_refusal(&work, e))?;
    for (r, n) in &copied.out_of_scope_sources {
        dla_census::record_out_of_scope(ctx, *r, *n)?;
    }
    write_agent(rt, opts.driver, &agent)?;

    let det = detectors(ctx, rt);
    let mut gaps: Vec<String> = copied.gaps.clone();
    if let Some(w) = &workload
        && let Some(reason) = &w.not_exercisable
    {
        gaps.push(format!(
            "note: discovery marked the workload not exercisable ({reason}): the harness build is likely to fail"
        ));
    }
    if cfg!(windows) && matches!(rt, Runtime::Rust | Runtime::Native) && !opts.driver {
        gaps.push("no allocator shim on Windows: only OS-level columns are measured".to_owned());
    }
    gaps.extend(det.notes.iter().cloned());

    let cwd_owned = workload.as_ref().map_or(".".to_owned(), |w| w.cwd.clone());
    let cwd = cwd_owned.as_str();
    let default_count = workload.as_ref().and_then(|w| w.iteration_default);
    let step = opts
        .step
        .unwrap_or_else(|| {
            if junit.is_some() {
                JUNIT_STEP
            } else {
                choose_step(default_count, opts.warmup, opts.iterations)
            }
        })
        .max(1);
    let layout = commands::Layout {
        work: work.clone(),
        agent: agent.clone(),
        driver: driver.clone(),
    };
    let mut ffi_info: Option<FfiHarness> = None;
    let mut ffi_switch: Option<commands::FfiSwitch> = None;
    if rt == Runtime::Jvm
        && let Some(b) = jni_boundary(ctx)
    {
        match ffi_setup(&b, &agent, &work) {
            Ok((h, sw)) => {
                gaps.push(format!(
                    "note: JNI library {} is measured: the allocator shim is preloaded into java (DLA_SHIM_ONLY=java) and the cdylib is built with line tables, so native stack groups resolve to source lines",
                    h.library.as_deref().unwrap_or("?")
                ));
                ffi_info = Some(h);
                ffi_switch = Some(sw);
            }
            Err(gap) => gaps.push(gap),
        }
    }
    let main_build = if rt == Runtime::Jvm && opts.driver && junit.is_none() {
        driver_build_pick(&work, workload.as_ref(), &mut gaps)
    } else {
        None
    };
    let mut kotlin_stdlib: Option<String> = None;
    // The build-resolved class path holds the project's own Kotlin standard library: the newest
    // one in ~/.m2 is looked for only when a script or a bare project builds the classes.
    if rt == Runtime::Jvm
        && opts.driver
        && junit.is_none()
        && main_build.is_none()
        && super::kotlin::has_kotlin_sources(&work)
    {
        match super::kotlin::find_stdlib(&work, &super::kotlin::Lookup::current()) {
            Ok(jar) => {
                gaps.push(format!(
                    "note: the project is Kotlin: {} is on the driver's class path",
                    jar.display()
                ));
                kotlin_stdlib = Some(dla_core::paths::to_slash(&jar));
            }
            Err(why) => gaps.push(why),
        }
    }
    let switches = commands::Switches {
        jvm_options: det.jvm_options.clone(),
        env: det.env.clone(),
        jfr: false,
        ffi: ffi_switch,
        kotlin_stdlib,
    };

    let mut tools: Vec<String> = Vec::new();
    let mut driver_info: Option<DriverBuild> = None;
    let (build, run, agent_kind, iter_for_manifest);
    let mut go_dir: Option<String> = None;
    if opts.driver {
        let dctx = DriverCtx {
            ctx,
            opts,
            work: &work,
            driver: &driver,
            agent: &agent,
            workload: workload.as_ref(),
            cwd,
            step,
            junit: junit.as_ref(),
            main_build: main_build.as_ref(),
        };
        let info = build_driver(&dctx)?;
        let mut sw = switches.clone();
        sw.jfr = rt == Runtime::Jvm && jfr_available();
        if rt == Runtime::Jvm && !sw.jfr {
            gaps.push("jdk.jfr is not available: no flight recording in driver mode".to_owned());
        }
        let plan = commands::plan_driver(rt, workload.as_ref(), cwd, &layout, &sw, &info.spec)
            .map_err(|e| {
                refuse(
                    DRIVER_UNAVAILABLE,
                    format!("cannot plan the driver: {e}"),
                    "a supported project layout",
                    format!("dla harness gen --lang {lang}   (command mode)"),
                )
            })?;
        build = with_sync(plan.build, &info.sync, &work);
        run = plan.run;
        agent_kind = plan.agent;
        gaps.extend(plan.gaps);
        gaps.extend(info.gaps.iter().cloned());
        tools.extend(info.tools.iter().cloned());
        go_dir = info.go_dir.clone();
        iter_for_manifest = None;
        driver_info = Some(info);
        if det.goleak && rt == Runtime::Go && !tools.contains(&"goleak".to_owned()) {
            // already recorded as a gap by the driver builder
        }
    } else {
        let w = workload.as_ref().ok_or_else(|| {
            refuse(
                WORKLOAD_REQUIRED,
                format!("no {lang} workload to run"),
                format!("a {lang} workload"),
                format!("dla workload discover --lang {lang}"),
            )
        })?;
        let package_script = (rt == Runtime::Node)
            .then(|| fs::read_to_string(work.join(join_rel(&w.cwd, "package.json"))).ok())
            .flatten()
            .and_then(|text| crate::iteration::npm_script_in(&text, &w.command));
        let plan = commands::plan(
            w,
            iter_arg.as_deref(),
            &layout,
            &switches,
            package_script.as_deref(),
        )
        .map_err(|e| {
            refuse(
                codes::INVALID_VALUE,
                format!("cannot turn workload {} into a harness command: {e}", w.id),
                "a supported command shape",
                format!("dla harness gen --lang {lang} --driver"),
            )
        })?;
        for f in &plan.files {
            write(&work.join(&f.rel), &f.content)?;
            if rt == Runtime::Go {
                write(&agent.join(templates::GO_WRAPPER_FILE), &f.content)?;
            }
        }
        build = plan.build;
        run = plan.run;
        agent_kind = plan.agent;
        gaps.extend(plan.gaps);
        iter_for_manifest = iter_arg.clone();
        if rt == Runtime::Go && det.goleak {
            gaps.push("goleak is used in driver mode only (dla harness gen --driver)".to_owned());
        }
    }

    // Probes.
    let derived = probes::derive(
        ctx,
        rt,
        opts.driver,
        opts.auto_probes,
        &opts.probe_findings,
        go_dir.as_deref(),
    )?;
    gaps.extend(derived.notes.iter().cloned());
    write(
        &hdir.join(PROBES),
        &probes::probes_json(&ctx.project, &derived.probes),
    )?;

    let manifest = HarnessManifest {
        schema_version: dla_core::harness::HARNESS_SCHEMA_VERSION,
        id: id.clone(),
        runtime: rt,
        mode: if opts.driver {
            HarnessMode::Driver
        } else {
            HarnessMode::Command
        },
        workload: workload.as_ref().map(|w| w.id.clone()),
        scratch: format!("harness/{id}/{WORK}"),
        build,
        run,
        iter_arg: iter_for_manifest,
        warmup: opts.warmup,
        iterations: opts.iterations,
        step,
        key_modes: if junit.is_some() {
            // A JUnit repeat has no keys: one series, labelled like the first key mode of a
            // driver so `dla alloc-test` and `dla trend` read it the same way.
            vec![KeyMode::Fresh]
        } else if opts.driver {
            vec![KeyMode::Fresh, KeyMode::Repeated]
        } else {
            vec![KeyMode::Native]
        },
        probes: derived.probes.clone(),
        agent: agent_kind,
        detectors: det.on.clone(),
        tools,
        gaps: gaps.clone(),
        timeout_secs: if junit.is_some() {
            JUNIT_TIMEOUT_SECS
        } else {
            TIMEOUT_SECS
        },
        ffi: ffi_info.clone(),
        captures: vec![],
    };
    manifest.save(&ctx.run_dir).map_err(|e| e.to_refusal())?;

    let points = opts.warmup + opts.iterations;
    let mut result = json!({
        "harness": id,
        "mode": manifest.mode.as_str(),
        "runtime": lang,
        "workload": manifest.workload,
        "build": manifest.build.as_ref().map(|b| cmd_text(b, &hdir)),
        "run": cmd_text(&manifest.run, &hdir),
        "iter_arg": manifest.iter_arg,
        "step": step,
        "points": points,
        "probes": manifest.probes.len(),
        "agent": manifest.agent.as_str(),
        "detectors": manifest.detectors,
        "tools": manifest.tools,
        "gaps": manifest.gaps.len(),
        "files": copied.files,
        "ffi": ffi_info.as_ref().map(|f| format!("{}:{}", f.kind, f.library.as_deref().unwrap_or("?"))),
    });
    let skipped_sources: u64 = copied.out_of_scope_sources.values().sum();
    if let Value::Object(m) = &mut result {
        if skipped_sources > 0 {
            m.insert("out_of_scope".into(), json!(skipped_sources));
        }
        m.retain(|_, v| !v.is_null());
    }
    let mut ledger = ctx.ledger()?;
    let mut spec = EvidenceSpec::new(
        EvidenceKind::Workload,
        "dla",
        env!("CARGO_PKG_VERSION"),
        result.clone(),
    );
    spec.raw = Some(format!("harness/{id}/{}", super::MANIFEST));
    spec.limitations =
        vec!["harness written; nothing was run (dla harness run measures it)".to_owned()];
    spec.limitations
        .extend(gaps.iter().take(3).map(|g| clip(g, 200)));
    let ev = ctx.add_evidence(&mut ledger, spec)?;
    cleanup.armed = false;

    let mut artifacts = vec![
        ctx.pointer(&format!("harness/{id}/{}", super::MANIFEST)),
        ctx.pointer(&format!("harness/{id}/{PROBES}")),
    ];
    let mut summary = result;
    if let (Some(info), Value::Object(m)) = (&driver_info, &mut summary) {
        if info.spec.junit.is_some() {
            let driver_file = ctx.pointer(&format!(
                "harness/{id}/{DRIVER}/{}.java",
                commands::JUNIT_DRIVER_CLASS
            ));
            artifacts.push(driver_file);
            m.insert("hooks".into(), json!(info.hooks));
            m.insert("junit".into(), json!(true));
            m.insert(
                "instructions".into(),
                json!("nothing to fill: the driver runs the project's own JUnit 5 tests in a loop; run it"),
            );
        } else {
            let edit = ctx.pointer(&format!("harness/{id}/{}", info.edit));
            artifacts.push(edit.clone());
            m.insert("edit".into(), json!(edit));
            m.insert("hooks".into(), json!(info.hooks));
            m.insert(
                "instructions".into(),
                json!("replace the body of each DLA-HOOK (keep the comment), then run the harness"),
            );
            if let Some(hint) = junit_hint {
                m.insert("junit_hint".into(), json!(hint));
            }
        }
        if !info.extra_files.is_empty() {
            m.insert("also".into(), json!(info.extra_files));
        }
    }
    if let Value::Object(m) = &mut summary {
        m.insert(
            "next".into(),
            json!(if junit.is_some() {
                format!("dla harness run {id}")
            } else if opts.driver {
                format!("fill the hooks, then dla harness run {id} --mode both")
            } else {
                format!("dla harness run {id}")
            }),
        );
    }
    Ok(StageOutcome {
        evidence: vec![ev.to_string()],
        candidates: 0,
        skipped_duplicates: 0,
        artifacts,
        summary,
    })
}

/// True when `java` lists the `jdk.jfr` module.
fn jfr_available() -> bool {
    Command::new("java")
        .args(["--list-modules"])
        .output()
        .is_ok_and(|o| String::from_utf8_lossy(&o.stdout).contains("jdk.jfr"))
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    #[test]
    fn cdylib_dirs_cover_crate_workspace_and_both_profiles() {
        let w = std::path::Path::new("/w");
        assert_eq!(
            super::cdylib_target_dirs(w, "native/codec"),
            [
                "/w/native/codec/target/release",
                "/w/native/codec/target/debug",
                "/w/native/target/release",
                "/w/native/target/debug",
                "/w/target/release",
                "/w/target/debug",
            ]
        );
        assert_eq!(super::cdylib_target_dirs(w, ".").len(), 2);
    }

    use super::*;

    #[test]
    fn step_targets_twice_the_default() {
        // 9 points, default 2000: largest n = 9 * step ~ 4000.
        assert_eq!(choose_step(Some(2000), 1, 8), 440);
        assert_eq!(choose_step(None, 1, 8), 440);
        assert_eq!(choose_step(Some(5000), 1, 8), 1110);
        assert_eq!(choose_step(Some(1), 1, 8), 1);
        assert_eq!(choose_step(Some(10), 1, 8), 2);
    }

    #[test]
    fn pretty_shell_quoting() {
        assert_eq!(shell_quote("a/b.c"), "a/b.c");
        assert_eq!(shell_quote("a b"), "'a b'");
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
    }

    #[test]
    fn sync_prefixes_the_build() {
        let b = CommandSpec {
            argv: vec!["go".into(), "test".into(), "./x".into()],
            cwd: ".".into(),
            env: Vec::new(),
        };
        let out = with_sync(
            Some(b),
            &[(PathBuf::from("/h/driver/d.go"), "pkg/d.go".into())],
            Path::new("/h/work"),
        )
        .unwrap();
        assert_eq!(
            out.argv[2],
            "cp -f /h/driver/d.go /h/work/pkg/d.go && go test ./x"
        );
    }
}

#[cfg(test)]
mod detector_tests {
    use super::*;
    use dla_core::{Ledger, RunState, state::now_utc};

    fn ctx_with_probe(libs: serde_json::Value) -> (tempfile::TempDir, ToolCtx) {
        let d = tempfile::tempdir().unwrap();
        let run_dir = d.path().join(".dla/r1");
        fs::create_dir_all(run_dir.join("probe")).unwrap();
        let mut st =
            RunState::init(d.path().to_str().unwrap(), Some("s"), 3600, None, now_utc()).unwrap();
        st.run_id = "r1".into();
        st.save(&run_dir).unwrap();
        Ledger::create(&run_dir).unwrap();
        let report = json!({
            "schema_version": 1, "project": "/p", "runtimes": [], "tools": [],
            "privilege": {"container": false, "ptrace_scope": null, "root": false},
            "libraries": libs,
        });
        fs::write(run_dir.join("probe/probe.json"), report.to_string()).unwrap();
        let ctx = ToolCtx {
            project: d.path().to_path_buf(),
            run_dir,
            run_id: "r1".into(),
            source_commit: "abc".into(),
            data_flag: None,
            worker: None,
        };
        (d, ctx)
    }

    #[test]
    fn library_flags_become_options_environment_or_notes() {
        let (_d, ctx) = ctx_with_probe(json!([
            {"runtime": "jvm", "library": "netty", "evidence": "pom.xml",
             "harness_flags": ["-Dio.netty.leakDetection.level=paranoid"]},
            {"runtime": "jvm", "library": "hikaricp", "evidence": "pom.xml",
             "harness_flags": ["HikariConfig.leakDetectionThreshold=2000 (ms)"]},
            {"runtime": "python", "library": "asyncio", "evidence": "a.py",
             "harness_flags": ["PYTHONASYNCIODEBUG=1", "-X dev"]},
            {"runtime": "python", "library": "python-resourcewarning", "evidence": "a.py",
             "harness_flags": ["-W error::ResourceWarning"]},
        ]));
        let jvm = detectors(&ctx, Runtime::Jvm);
        assert_eq!(jvm.jvm_options, ["-Dio.netty.leakDetection.level=paranoid"]);
        assert_eq!(jvm.on, ["-Dio.netty.leakDetection.level=paranoid"]);
        assert!(jvm.notes[0].contains("hikaricp") && jvm.notes[0].starts_with("note:"));
        let py = detectors(&ctx, Runtime::Python);
        assert!(py.env.contains(&("PYTHONASYNCIODEBUG".into(), "1".into())));
        assert!(py.env.contains(&("PYTHONDEVMODE".into(), "1".into())));
        assert!(py.on.contains(&"-W error::ResourceWarning".to_owned()));
        assert_eq!(
            detectors(&ctx, Runtime::Node).on,
            ["MaxListenersExceededWarning"]
        );
    }
}
