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
    AGENT, COV, DRIVER, LOGS, OUT, PROBES, WORK, commands, harness_dir, probes, scratch, targets,
    templates,
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
/// The time limit of a test-loop batch: the batch is cut to about two minutes of tests, the
/// loop fits its rounds into this (`DLA_TIME_LIMIT_SECS`), so a slow class cannot hold the
/// run for half an hour.
const TEST_LOOP_TIMEOUT_SECS: u64 = 600;

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
    /// Findings to add a probe for; each is also declared a target of the harness.
    pub probe_findings: Vec<FindingId>,
    /// Declare this many of the next pending queue entries of the runtime as targets
    /// (`--next`); `None` declares none.
    pub next: Option<usize>,
    /// `--entry`: the findings an entry-path harness is generated for (at most 8, each a
    /// leak-side finding that grew in some harness). Excludes `--probe` and `--next`.
    pub entry: Vec<FindingId>,
    /// Derive probes from census candidates.
    pub auto_probes: bool,
    /// Go/JVM package (directory or name) for driver mode.
    pub package: Option<String>,
    /// JVM: run the project's own JUnit 5 tests in a loop (implies driver mode; no hooks).
    pub junit: bool,
    /// JVM, JUnit repeat: classes (`pkg.Class`) or methods (`pkg.Class#method`) to run, instead
    /// of the workload's test class.
    pub tests: Vec<String>,
    /// Rust driver: crate features the driver build enables (`--features a,b`).
    pub features: Vec<String>,
    /// Rust driver: the driver build leaves the crate's default features out.
    pub no_default_features: bool,
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
    dla_core::testloop::on_path(name)
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
    match dla_core::testloop::build_tool(w)? {
        "maven" => Some(commands::JunitTool::Maven),
        _ => Some(commands::JunitTool::Gradle),
    }
}

/// Why workload `w` cannot be run by the JUnit repeat driver; `None` when it can: a JVM test
/// of a Gradle or Maven project whose file uses the JUnit 5 (`org.junit.jupiter`) API.
fn junit_problem(project: &Path, w: &Workload) -> Option<String> {
    dla_core::testloop::junit_problem(project, w)
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
    dla_core::testloop::find_wrapper(project, cwd, name)
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
        Runtime::Node => write(&agent.join("dla-preload.cjs"), templates::NODE_AGENT),
        Runtime::Go => Ok(()),
        Runtime::Rust | Runtime::Native => {
            if cfg!(windows) || (driver && rt == Runtime::Native) {
                Ok(())
            } else if driver {
                // A Rust driver counts allocations itself; the shim is built for
                // `dla harness run --capture alloc` only (allocation stacks), so a missing
                // compiler is not a refusal here: that run says so.
                let _ = compile_shim(agent);
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
    /// Go: limit GoVar and GoField probes to this directory.
    go_dir: Option<String>,
    /// Lines about the driver itself, written under the first line of the target block
    /// (Go: the package the driver is in).
    block_notes: Vec<String>,
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

/// True when the nearest `package.json` at or above `work/cwd` (not above `work`) says
/// `"type": "module"`: every `.js` file under it is then an ES module.
fn node_module_package(work: &Path, cwd: &str) -> bool {
    let mut dir = work.join(if cwd == "." { "" } else { cwd });
    loop {
        if let Ok(text) = fs::read_to_string(dir.join("package.json")) {
            return serde_json::from_str::<Value>(&text)
                .ok()
                .is_some_and(|v| v.get("type").and_then(Value::as_str) == Some("module"));
        }
        if dir == work || !dir.pop() || !dir.starts_with(work) {
            return false;
        }
    }
}

/// The block line that tells the model to import (not require) ES modules in a Node driver: the
/// project is a `"type": "module"` package, or the workload's file is a `.mjs`.
fn node_module_note(work: &Path, cwd: &str, w: Option<&Workload>) -> Option<String> {
    let mjs = w.is_some_and(|w| w.file.ends_with(".mjs"));
    if node_module_package(work, cwd) {
        Some(
            "This project's package.json says \"type\": \"module\": its .js files are ES modules. Load them in the hooks with `await projectImport('<file>')` (a dynamic import(): the namespace object); `projectRequire` is for CommonJS files (.cjs, or a package without that type) and throws on an ES module."
                .to_owned(),
        )
    } else if mjs {
        Some(
            "The workload's file is an .mjs ES module: load it in the hooks with `await projectImport('<file>')`; `projectRequire` is for CommonJS files and throws on an ES module."
                .to_owned(),
        )
    } else {
        None
    }
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
        block_notes: Vec::new(),
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
            write(&d.driver.join("dla-harness.cjs"), &text)?;
            out.hooks = templates::hooks(&text);
            out.edit = driver_rel("dla-harness.cjs");
            if let Some(note) = node_module_note(d.work, d.cwd, d.workload) {
                out.block_notes.push(note);
            }
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
            out.block_notes.push(format!(
                "The driver is in package {package} (directory {}): the hooks can read its unexported names and package variables, and no other package's.",
                dla_core::queue::package_label(&dir)
            ));
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
            out.spec.features.clone_from(&d.opts.features);
            out.spec.no_default_features = d.opts.no_default_features;
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

/// The targets of a harness, what `--next` found nothing to declare says, and the targets
/// `--next` added because they share an owner with another (added finding, finding it came with).
type DeclaredTargets = (
    Vec<FindingId>,
    Option<String>,
    std::collections::BTreeMap<FindingId, FindingId>,
    Option<String>,
    FocusFacts,
);

/// What a focus batch records: for each target the earlier run that missed it, and the text
/// that says the batch is a focus batch and why (empty for a broad batch).
#[derive(Debug, Default, Clone)]
struct FocusFacts {
    missed: std::collections::BTreeMap<FindingId, dla_core::queue::Miss>,
    why: Option<String>,
    /// A second look: the targets read `executed` flat in the earlier harness.
    second: bool,
}

/// The findings a harness is generated to reach: the `--probe` findings, then (with
/// `--next`) the next pending entries of the runtime's queue, at most [`MAX_TARGETS`] in all.
/// The second value says so when `--next` found nothing to declare.
fn resolve_targets(ctx: &ToolCtx, opts: &GenOptions) -> Result<DeclaredTargets, Refusal> {
    use dla_core::harness::{MAX_TARGETS, MAX_TARGETS_WITH_SIBLINGS};
    let rt = opts.lang;
    // A hookless test-loop harness declares more: nothing is written per target.
    let limit = if opts.junit {
        dla_core::testloop::MAX_TARGETS
    } else {
        MAX_TARGETS
    };
    let mut ids: Vec<FindingId> = Vec::new();
    for f in &opts.probe_findings {
        if !ids.contains(f) {
            ids.push(f.clone());
        }
    }
    let mut note = None;
    let mut siblings = std::collections::BTreeMap::new();
    // Go: the package directory every declared candidate of `--next` is in, and so the one
    // the driver is generated in.
    let mut go_package: Option<String> = None;
    let mut focus = FocusFacts::default();
    if let Some(n) = opts.next {
        if n > MAX_TARGETS || ids.len() + n > MAX_TARGETS {
            return Err(refuse(
                codes::TOO_MANY_TARGETS,
                format!(
                    "{} target(s) were asked for (--probe {} and --next {n}); one harness declares at most {MAX_TARGETS}",
                    ids.len() + n,
                    ids.len()
                ),
                format!("at most {MAX_TARGETS} targets per harness"),
                format!(
                    "dla harness gen --lang {} --driver --next {}",
                    rt.as_str(),
                    MAX_TARGETS.saturating_sub(ids.len()).max(1)
                ),
            ));
        }
        let view = dla_core::RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())
            .map_err(|e| e.to_refusal())?;
        let queue = dla_core::queue::queues(&view, ctx.worker.as_deref())
            .into_iter()
            .find(|q| q.runtime == rt.as_str());
        let mut skipped: Vec<String> = Vec::new();
        let chosen = match &queue {
            Some(q) if rt == Runtime::Go => go_batch(ctx, opts, &view, q, n, &ids, &mut skipped)?,
            // `--junit --tests ... --next`: the tests are the caller's, so the candidates are the
            // next pending ones in rank order, test-loop batches included.
            Some(q) if opts.junit => dla_core::queue::Batch {
                ids: q
                    .test_loop
                    .batches
                    .iter()
                    .flat_map(|b| b.ids.iter())
                    .chain(q.declarable.iter())
                    .take(n)
                    .cloned()
                    .collect(),
                ..Default::default()
            },
            Some(q) => q.batch_for(&view, n),
            None => Default::default(),
        };
        go_package = chosen.package.clone();
        if chosen.focus
            && let Some(q) = &queue
        {
            for f in &chosen.ids {
                if let Some(m) = q.missed.get(f).or_else(|| q.second.get(f)) {
                    focus.missed.insert(f.clone(), m.clone());
                }
            }
            focus.second = chosen.second;
            let first = chosen
                .ids
                .first()
                .map(ToString::to_string)
                .unwrap_or_default();
            let cap = if n > dla_core::queue::FOCUS_MAX {
                format!(
                    " --next {n} is capped at {} for a focus batch.",
                    dla_core::queue::FOCUS_MAX
                )
            } else {
                String::new()
            };
            focus.why = Some(if chosen.second {
                format!(
                    "second look: one flat run of one driver settled {first} and budget is left, so this harness holds it and at most {} more such candidates, alone, with a different driver (a VARIES.txt line per target is required).{cap}",
                    dla_core::queue::FOCUS_MAX - 1
                )
            } else {
                format!(
                    "focus batch: the best-ranked pending candidate {first} was declared in an earlier harness and its code did not run there, so this harness holds it and at most {} more candidates that a harness missed, alone (no siblings, no fresh candidates).{cap}",
                    dla_core::queue::FOCUS_MAX - 1
                )
            });
        }
        let (batch, sib) = (chosen.ids, chosen.siblings);
        let before = ids.len();
        // The first `n` by rank, then the siblings of their owners, up to the limit.
        for (k, f) in batch.into_iter().enumerate() {
            if ids.contains(&f) {
                continue;
            }
            if k >= n && ids.len() >= MAX_TARGETS_WITH_SIBLINGS {
                break;
            }
            if let Some(with) = sib.get(&f) {
                siblings.insert(f.clone(), with.clone());
            }
            ids.push(f);
        }
        if ids.len() == before {
            note = Some(format!(
                "--next: no pending candidate of {} is waiting for a measurement, so none was declared",
                rt.as_str()
            ));
        }
        if !skipped.is_empty() {
            let mut text = skipped.join("; ");
            if let Some(n) = note.take() {
                text = format!("{n}; {text}");
            }
            note = Some(text);
        }
    }
    if ids.len() - siblings.len() > limit {
        return Err(refuse(
            codes::TOO_MANY_TARGETS,
            format!(
                "{} targets were asked for with --probe; one harness declares at most {limit}",
                ids.len() - siblings.len()
            ),
            format!("at most {limit} targets per harness"),
            format!(
                "dla harness gen --lang {} --{} --probe <F> ...   (at most {limit}), then again for the rest",
                rt.as_str(),
                if opts.junit { "junit" } else { "driver" }
            ),
        ));
    }
    Ok((ids, note, siblings, go_package, focus))
}

/// What a hook driver's output says about the project's own tests: the test loop, with the
/// queue's count when it has batches left.
fn junit_hint_text(ctx: &ToolCtx) -> String {
    let sentence = dla_core::RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())
        .ok()
        .and_then(|view| {
            dla_core::queue::queues(&view, ctx.worker.as_deref())
                .into_iter()
                .find(|q| q.runtime == "jvm")
        })
        .map(|q| q.test_loop.sentence())
        .filter(|s| !s.is_empty());
    match sentence {
        Some(s) => format!(
            "this project has JUnit 5 tests: {s}: dla harness gen --lang jvm --junit --next runs them in a loop with nothing to fill"
        ),
        None => "this project has JUnit 5 tests: dla harness gen --lang jvm --junit runs them in a loop with nothing to fill".to_owned(),
    }
}

/// A chosen test class: fully qualified name, file, tests, and the candidates it was taken for.
type TestLoopClass = (
    String,
    String,
    usize,
    Vec<(FindingId, dla_core::testloop::Via)>,
);

/// What the next test-loop batch chose, for the summary of `dla harness gen --junit --next`.
#[derive(Debug)]
struct TestLoopFacts {
    /// The test classes, in the order they run: fully qualified name, file, tests, and the
    /// candidates each was taken for.
    classes: Vec<TestLoopClass>,
    /// What the queue says about the rest.
    sentence: String,
    /// Targets the batch had that `--next N` left for the next one.
    left: usize,
}

/// `--junit --next`: the targets and the tests of the queue's next test-loop batch, as the
/// options of a plain `--junit --probe ... --tests ...` generation. The batch is what the queue
/// planned ([`dla_core::testloop`]); `--next N` takes the first N candidates of it and the
/// classes they need.
fn test_loop_options(
    ctx: &ToolCtx,
    opts: &GenOptions,
) -> Result<(GenOptions, TestLoopFacts), Refusal> {
    use dla_core::testloop;
    junit_lang_check(opts)?;
    if !opts.probe_findings.is_empty() {
        return Err(refuse(
            codes::INVALID_VALUE,
            "--next with --junit chooses the targets and the tests itself; a --probe finding would be a target no chosen test is known to reach",
            "--next alone, or --probe with --tests",
            "dla harness gen --lang jvm --junit --next   or   dla harness gen --lang jvm --junit --probe F1 --tests pkg.FooTest",
        ));
    }
    let view = dla_core::RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())
        .map_err(|e| e.to_refusal())?;
    let queue = dla_core::queue::queues(&view, ctx.worker.as_deref())
        .into_iter()
        .find(|q| q.runtime == "jvm");
    let Some((q, batch)) = queue
        .as_ref()
        .and_then(|q| Some((q, q.test_loop.batches.first()?)))
    else {
        return Err(refuse(
            DRIVER_UNAVAILABLE,
            "no pending jvm candidate has a test class the test loop can run (a JUnit 5 test of a Maven or Gradle project whose build tool is installed, whose name or source uses the candidate's owner class), or the queue is closed",
            "a pending candidate with a JUnit 5 test class",
            "dla harness gen --lang jvm --driver --next   (hooks to fill; the driver runs your own code)",
        ));
    };
    let n = opts
        .next
        .unwrap_or(testloop::MAX_TARGETS)
        .clamp(1, testloop::MAX_TARGETS);
    let ids: Vec<FindingId> = batch.ids.iter().take(n).cloned().collect();
    let mut classes = Vec::new();
    let mut tests = Vec::new();
    for c in &batch.classes {
        let for_ids: Vec<(FindingId, testloop::Via)> = c
            .for_ids
            .iter()
            .filter(|(f, _)| ids.contains(f))
            .cloned()
            .collect();
        if for_ids.is_empty() {
            continue;
        }
        let text = fs::read_to_string(ctx.project.join(&c.file)).unwrap_or_default();
        let fqcn = crate::jvm::fqcn_of(&c.file, &text);
        tests.push(fqcn.clone());
        classes.push((fqcn, c.file.clone(), c.tests, for_ids));
    }
    let facts = TestLoopFacts {
        classes,
        sentence: q.test_loop.sentence(),
        left: batch.ids.len() - ids.len(),
    };
    Ok((
        GenOptions {
            probe_findings: ids,
            next: None,
            tests,
            ..opts.clone()
        },
        facts,
    ))
}

/// A reason a Go package directory cannot hold the driver, or `None` when it can (or when
/// `go` cannot be asked): a driver is a test file of the package, so the directory needs a
/// non-test Go file that builds on this platform, and `go list` must read it.
fn go_package_problem(project: &Path, dir: &str) -> Option<String> {
    let abs = project.join(dir);
    let has_file = fs::read_dir(&abs).ok().is_some_and(|rd| {
        rd.flatten().any(|e| {
            let n = e.file_name().to_string_lossy().into_owned();
            n.ends_with(".go") && !n.ends_with("_test.go")
        })
    });
    if !has_file {
        return Some("the directory has no non-test Go file".to_owned());
    }
    let out = Command::new("go")
        .args(["list", "-f", "{{len .GoFiles}}", "."])
        .current_dir(&abs)
        .env("GOPROXY", "off")
        .output()
        .ok()?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        let first = err.lines().find(|l| !l.trim().is_empty()).unwrap_or("");
        return Some(format!("go list fails there: {}", clip(first.trim(), 160)));
    }
    let n = String::from_utf8_lossy(&out.stdout);
    (n.trim() == "0").then(|| "no Go file of it builds on this platform".to_owned())
}

/// The batch of a Go `--next`: one package. `--package <dir>` names it; with `--probe` and no
/// `--package` it is the package of the first `--probe` finding; otherwise the package of the
/// best-ranked pending candidate that can hold the driver, the ones before it being named in
/// `skipped` (their candidates stay pending). Returns the package directory with the batch.
fn go_batch(
    ctx: &ToolCtx,
    opts: &GenOptions,
    view: &dla_core::RunView,
    q: &dla_core::queue::RuntimeQueue,
    n: usize,
    probed: &[FindingId],
    skipped: &mut Vec<String>,
) -> Result<dla_core::queue::Batch, Refusal> {
    use dla_core::queue::{go_package_dir, package_label};
    let packages = q.packages(view);
    let named = opts.package.as_deref().map(|p| {
        let p = p.trim_start_matches("./").trim_end_matches('/');
        if p == "." { "" } else { p }.to_owned()
    });
    let from_probe = || {
        probed
            .first()
            .and_then(|f| view.ledger.finding(f))
            .and_then(|f| f.site.as_deref())
            .map(go_package_dir)
    };
    if let Some(p) = named.clone().or_else(from_probe) {
        if !packages.contains(&p) {
            if named.is_some() {
                let list: Vec<&str> = packages.iter().take(6).map(|p| package_label(p)).collect();
                return Err(refuse(
                    codes::INVALID_VALUE,
                    format!(
                        "--next --package {}: no pending candidate of go has its site in that package",
                        package_label(&p)
                    ),
                    "a package directory with a pending candidate",
                    if list.is_empty() {
                        "dla run status   (no candidate of go is waiting for a measurement)"
                            .to_owned()
                    } else {
                        format!(
                            "dla harness gen --lang go --driver --next --package <dir>   (packages with a pending candidate, best-ranked first: {}), or --next alone for the best-ranked one",
                            list.join(", ")
                        )
                    },
                ));
            }
            return Ok(dla_core::queue::Batch {
                package: Some(p),
                ..Default::default()
            });
        }
        let mut b = q.batch_in_package(view, n, &p);
        b.package = Some(p);
        return Ok(b);
    }
    for p in &packages {
        if let Some(why) = go_package_problem(&ctx.project, p) {
            let b = q.batch_in_package(view, n, p);
            skipped.push(format!(
                "package {} cannot hold the driver ({why}): its {} best-ranked candidate(s) stay pending and are not targeted, and the next package by rank was taken",
                package_label(p),
                b.ids.len()
            ));
            continue;
        }
        let mut b = q.batch_in_package(view, n, p);
        b.package = Some(p.clone());
        return Ok(b);
    }
    Ok(Default::default())
}

/// Writes the second-look unit skeleton (see [`dla_core::harness::unit_skeleton`]) above the
/// line of `file` that says the unit hook is not filled in.
fn insert_unit_skeleton(file: &Path, rt: Runtime) -> std::io::Result<()> {
    let text = fs::read_to_string(file)?;
    let mut out = String::with_capacity(text.len() + 1200);
    let mut done = false;
    for line in text.split_inclusive('\n') {
        if !done && line.contains("DLA-HOOK unit has not been filled in") {
            out.push_str(&dla_core::harness::unit_skeleton(rt));
            done = true;
        }
        out.push_str(line);
    }
    fs::write(file, out)
}

/// The findings of `--entry`: refused before anything is created unless the harness is a hook
/// driver, no `--probe` or `--next` is given, there are at most [`MAX_TARGETS`] findings and each
/// is a leak-side finding that grew in some harness (the trigger of a holder that never grew is
/// not a question yet).
fn entry_targets(ctx: &ToolCtx, opts: &GenOptions) -> Result<Vec<FindingId>, Refusal> {
    use dla_core::harness::MAX_TARGETS;
    let lang = opts.lang.as_str();
    if !opts.driver || opts.junit {
        return Err(refuse(
            codes::INVALID_VALUE,
            "--entry writes a hook driver whose units call the project's entry points",
            "--driver (and no --junit)",
            format!("dla harness gen --lang {lang} --driver --entry <F>"),
        ));
    }
    if !opts.probe_findings.is_empty() || opts.next.is_some() {
        return Err(refuse(
            codes::INVALID_VALUE,
            "--entry cannot be combined with --probe or --next: an entry-path harness declares exactly the findings of --entry",
            "--entry alone",
            format!(
                "dla harness gen --lang {lang} --driver --entry {}",
                opts.entry
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ));
    }
    let mut ids: Vec<FindingId> = Vec::new();
    for f in &opts.entry {
        if !ids.contains(f) {
            ids.push(f.clone());
        }
    }
    if ids.len() > MAX_TARGETS {
        return Err(refuse(
            codes::TOO_MANY_TARGETS,
            format!(
                "{} findings were given to --entry; one harness declares at most {MAX_TARGETS}",
                ids.len()
            ),
            format!("at most {MAX_TARGETS} findings per entry-path harness"),
            format!(
                "dla harness gen --lang {lang} --driver --entry {}   (then again for the rest)",
                ids.iter()
                    .take(MAX_TARGETS)
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        ));
    }
    let view = dla_core::RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())
        .map_err(|e| e.to_refusal())?;
    let triggers = dla_core::trigger::Triggers::of(&view);
    for id in &ids {
        let measure = format!(
            "dla harness gen --lang {lang} --driver --probe {id}, fill the hooks so a unit reaches it with a new key, dla harness run <H> --mode both, dla trend S<n>; then --entry {id}"
        );
        let Some(f) = view.ledger.finding(id) else {
            return Err(refuse(
                codes::UNKNOWN_FINDING,
                format!("--entry {id}: the ledger has no such finding"),
                format!("finding {id}"),
                "dla ledger list   (lists the findings)",
            ));
        };
        if let Some(canonical) = view.ledger.merged_into(id) {
            return Err(refuse(
                codes::INVALID_VALUE,
                format!("--entry {id}: {id} is a duplicate of {canonical} (one site, one finding)"),
                format!("the finding that stands for the site: {canonical}"),
                format!("dla harness gen --lang {lang} --driver --entry {canonical}"),
            ));
        }
        if f.polarity() != dla_core::finding::Polarity::Leak {
            return Err(refuse(
                codes::ENTRY_NOT_GROWN,
                format!(
                    "--entry {id}: it is a non-leak explanation; only a leak-side finding has a trigger"
                ),
                format!("a leak-side finding that grew in a harness, not {id}"),
                "dla ledger list   (the leak-side findings, with their harnesses)",
            ));
        }
        if !triggers.grew_in_a_harness(id) {
            return Err(refuse(
                codes::ENTRY_NOT_GROWN,
                format!(
                    "--entry {id}: it has not grown in any harness yet; the trigger is asked for a holder that grows when a harness feeds it keys"
                ),
                format!("a trend of a harness in which {id} has the verdict grew"),
                measure,
            ));
        }
    }
    Ok(ids)
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
    // `--junit --next`: the queue's next test-loop batch chooses the targets and the tests.
    let test_loop_opts;
    let mut test_loop_facts: Option<TestLoopFacts> = None;
    let opts = if opts.junit && opts.next.is_some() && opts.tests.is_empty() {
        let (o, facts) = test_loop_options(ctx, opts)?;
        test_loop_opts = o;
        test_loop_facts = Some(facts);
        &test_loop_opts
    } else {
        opts
    };
    let rt = opts.lang;
    let lang = rt.as_str();
    // `--entry`: its findings and options are checked before anything else is read.
    let entry_ids = if opts.entry.is_empty() {
        None
    } else {
        Some(entry_targets(ctx, opts)?)
    };
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
    if (!opts.features.is_empty() || opts.no_default_features)
        && !(rt == Runtime::Rust && opts.driver)
    {
        return Err(refuse(
            codes::INVALID_VALUE,
            "--features and --no-default-features apply to the build of a Rust driver",
            "--lang rust --driver",
            "dla harness gen --lang rust --driver --features a,b",
        ));
    }
    if let Some(f) = opts.features.iter().find(|f| {
        f.is_empty()
            || !f
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "_-/:+".contains(c))
    }) {
        return Err(refuse(
            codes::INVALID_VALUE,
            format!("--features {f:?} is not a crate feature name"),
            "feature names of the crate, comma-separated",
            "dla harness gen --lang rust --driver --features a,b",
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
    .then(|| junit_hint_text(ctx));
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

    // The targets: `--probe` findings, then the next pending queue entries. Refused before
    // anything is created. An entry-path harness declares its `--entry` findings.
    let (asked_ids, next_note, siblings, go_package, focus) = match entry_ids {
        None => resolve_targets(ctx, opts)?,
        Some(ids) => (
            ids,
            None,
            std::collections::BTreeMap::new(),
            None,
            FocusFacts::default(),
        ),
    };
    // A Go `--next` batch is one package: the driver is generated in it, as `--package` does.
    let package_opts;
    let opts = match go_package {
        Some(p) if opts.package.is_none() && rt == Runtime::Go => {
            package_opts = GenOptions {
                package: Some(p),
                ..opts.clone()
            };
            &package_opts
        }
        _ => opts,
    };

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
    if let Some(f) = &test_loop_facts {
        gaps.push(format!(
            "test loop (--junit --next): {} test class(es) with {} tests, chosen for {} target(s): {}",
            f.classes.len(),
            f.classes.iter().map(|c| c.2).sum::<usize>(),
            opts.probe_findings.len(),
            f.classes
                .iter()
                .map(|(class, _, _, for_ids)| format!(
                    "{class} for {}",
                    for_ids
                        .iter()
                        .map(|(id, via)| format!("{id} by {}", via.as_str()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }

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
        &asked_ids,
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
        sibling_of: siblings.clone(),
        purpose: if !opts.entry.is_empty() {
            Some(dla_core::harness::HarnessPurpose::EntryPath)
        } else if !focus.missed.is_empty() && focus.second {
            Some(dla_core::harness::HarnessPurpose::SecondLook)
        } else if !focus.missed.is_empty() {
            Some(dla_core::harness::HarnessPurpose::Focus)
        } else {
            None
        },
        targets: derived.targets.iter().map(|a| a.id.clone()).collect(),
        agent: agent_kind,
        detectors: det.on.clone(),
        tools,
        gaps: gaps.clone(),
        timeout_secs: if junit.is_some() && test_loop_facts.is_some() {
            TEST_LOOP_TIMEOUT_SECS
        } else if junit.is_some() {
            JUNIT_TIMEOUT_SECS
        } else {
            TIMEOUT_SECS
        },
        ffi: ffi_info.clone(),
        captures: vec![],
    };
    manifest.save(&ctx.run_dir).map_err(|e| e.to_refusal())?;
    if manifest.is_focus() || manifest.is_second_look() {
        // Which earlier harness missed (or read flat) each target, and what its trend said.
        let doc = json!({
            "focus": true,
            "missed_in": focus.missed.iter().map(|(f, x)| (f.to_string(), json!({
                "harness": x.harness, "verdict": x.verdict.as_str(), "detail": x.detail,
            }))).collect::<serde_json::Map<_, _>>(),
        });
        write(
            &hdir.join("focus.json"),
            &(serde_json::to_string_pretty(&doc).unwrap_or_default() + "\n"),
        )?;
    }

    // What each declared target needs, printed and written above the driver's first hook.
    let mut described = if derived.targets.is_empty() {
        Vec::new()
    } else {
        targets::describe(
            ctx,
            &ctx.ledger()?,
            &derived.targets,
            &derived.probes,
            &siblings,
            rt,
        )
    };
    if rt == Runtime::Rust {
        targets::add_feature_gates(&ctx.project, &mut described, &opts.features);
    }
    if manifest.is_entry_path() {
        targets::add_entry_facts(&ctx.project, rt, &mut described);
        write(
            &hdir.join(DRIVER).join(dla_core::harness::VARIES_FILE),
            &dla_core::harness::varies_template(),
        )?;
    }
    if manifest.is_second_look() {
        write(
            &hdir.join(DRIVER).join(dla_core::harness::VARIES_FILE),
            &dla_core::harness::second_look_varies_template(&manifest.targets),
        )?;
    }
    let mut block_in: Option<String> = None;
    if !described.is_empty() {
        write(
            &hdir.join("targets.json"),
            &(serde_json::to_string_pretty(&json!({
                "harness": id,
                "targets": described.iter().map(targets::Target::json).collect::<Vec<_>>(),
                "general": targets::GENERAL,
            }))
            .unwrap_or_default()
                + "\n"),
        )?;
        if let Some(info) = driver_info.as_ref().filter(|i| !i.edit.is_empty()) {
            let file = hdir.join(&info.edit);
            let mut notes = info.block_notes.clone();
            if manifest.is_second_look() {
                // The unit hook is a skeleton: one variable per choice, to be used.
                insert_unit_skeleton(&file, rt).map_err(|e| io_refusal(&file, e))?;
            }
            if manifest.is_second_look() {
                notes.extend(targets::second_look_notes(
                    &focus.missed,
                    &derived.targets,
                    dla_core::harness::choice_helper(rt),
                ));
            } else if manifest.is_focus() {
                notes.extend(targets::focus_notes(&focus.missed, &derived.targets));
            } else if opts.next.is_some() && !manifest.is_entry_path() {
                notes.push(targets::broad_note(dla_core::harness::choice_helper(rt)));
            }
            if targets::write_block_with(&file, &id, &described, &notes)
                .map_err(|e| io_refusal(&file, e))?
            {
                block_in = Some(info.edit.clone());
                // The copies placed in the scratch project now carry the block too.
                for (src, dest) in &info.sync {
                    let to = work.join(dest);
                    fs::copy(src, &to).map_err(|e| io_refusal(&to, e))?;
                }
            }
        }
    }

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
        "purpose": manifest.purpose.map(|p| p.as_str()),
        "features": (!opts.features.is_empty()).then(|| opts.features.join(",")),
        "no_default_features": opts.no_default_features.then_some(true),
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
    if let Value::Object(m) = &mut summary {
        if !described.is_empty() {
            m.insert(
                "targets".into(),
                json!(
                    described
                        .iter()
                        .map(|t| {
                            let mut j = json!({
                                "finding": t.id,
                                "site": t.site,
                                "holder": t.holder,
                                "owner": t.owner,
                                "probe": t.probe.is_some(),
                                "sibling_of": t.sibling_of,
                                "to_do": clip(&t.instruction, 90),
                            });
                            if let (Some(g), Value::Object(o)) = (&t.feature_gate, &mut j) {
                                o.insert("feature_gate".into(), json!(g.text()));
                            }
                            j
                        })
                        .collect::<Vec<_>>()
                ),
            );
            // One short note per set of features: the ids behind them and what to generate with.
            let mut gated: std::collections::BTreeMap<Vec<String>, Vec<String>> =
                std::collections::BTreeMap::new();
            for t in &described {
                if let Some(g) = t.feature_gate.as_ref().filter(|g| !g.enabled) {
                    gated
                        .entry(g.features.clone())
                        .or_default()
                        .push(t.id.to_string());
                }
            }
            if !gated.is_empty() {
                let notes: Vec<String> = gated
                    .iter()
                    .map(|(f, ids)| {
                        format!(
                            "{} behind feature {}: generate with --features {}",
                            ids.join(","),
                            f.join(", "),
                            f.join(",")
                        )
                    })
                    .collect();
                m.insert("feature_note".into(), json!(notes.join("; ")));
            }
            m.insert(
                "targets_full".into(),
                json!(match &block_in {
                    Some(f) => format!(
                        "the comment block at the top of {}",
                        ctx.pointer(&format!("harness/{id}/{f}"))
                    ),
                    None => ctx.pointer(&format!("harness/{id}/targets.json")),
                }),
            );
            artifacts.push(ctx.pointer(&format!("harness/{id}/targets.json")));
        }
        if let Some(n) = &next_note {
            m.insert("next_note".into(), json!(n));
        }
        if let Some(why) = &focus.why {
            m.insert("focus".into(), json!(why));
            m.insert(
                "missed_in".into(),
                json!(
                    focus
                        .missed
                        .iter()
                        .map(|(f, x)| (
                            f.to_string(),
                            json!(format!("{} {}", x.harness, x.verdict.as_str()))
                        ))
                        .collect::<serde_json::Map<_, _>>()
                ),
            );
        }
    }
    if let (Some(info), Value::Object(m)) = (&driver_info, &mut summary) {
        if let Some(dir) = &info.go_dir {
            // Go: the driver is a test file of this package.
            m.insert("package".into(), json!(dla_core::queue::package_label(dir)));
        }
        if info.spec.junit.is_some() {
            let driver_file = ctx.pointer(&format!(
                "harness/{id}/{DRIVER}/{}.java",
                commands::JUNIT_DRIVER_CLASS
            ));
            artifacts.push(driver_file);
            m.insert("hooks".into(), json!(info.hooks));
            m.insert("junit".into(), json!(true));
            if let Some(f) = &test_loop_facts {
                // The output is bounded (2 KB): the tests and why, one line each; the rest is in
                // `tests.json`.
                let simple = |class: &str| class.rsplit('.').next().unwrap_or(class).to_owned();
                m.insert(
                    "tests_chosen".into(),
                    json!(
                        f.classes
                            .iter()
                            .map(|(class, _, tests, for_ids)| format!(
                                "{} ({tests} tests) for {} by {}",
                                simple(class),
                                for_ids
                                    .iter()
                                    .map(|(id, _)| id.to_string())
                                    .collect::<Vec<_>>()
                                    .join(","),
                                if for_ids
                                    .iter()
                                    .all(|(_, v)| *v == dla_core::testloop::Via::Name)
                                {
                                    "name"
                                } else if for_ids
                                    .iter()
                                    .all(|(_, v)| *v == dla_core::testloop::Via::Mention)
                                {
                                    "mention"
                                } else {
                                    "name or mention"
                                }
                            ))
                            .collect::<Vec<_>>()
                    ),
                );
                m.insert(
                    "test_loop".into(),
                    json!({
                        "queue": f.sentence,
                        "targets": opts.probe_findings.len(),
                        "tests": f.classes.iter().map(|c| c.2).sum::<usize>(),
                        "left_for_the_next_batch": f.left,
                    }),
                );
                let doc = json!({
                    "rule": dla_core::testloop::rule(),
                    "queue": f.sentence,
                    "bounds": {
                        "targets": dla_core::testloop::MAX_TARGETS,
                        "classes": dla_core::testloop::MAX_CLASSES,
                        "tests": dla_core::testloop::MAX_TESTS,
                        "time_limit_secs": TEST_LOOP_TIMEOUT_SECS,
                    },
                    "targets": opts.probe_findings,
                    "left_for_the_next_batch": f.left,
                    "classes": f.classes.iter().map(|(class, file, tests, for_ids)| json!({
                        "class": class,
                        "file": file,
                        "tests": tests,
                        "for": for_ids.iter().map(|(id, via)| json!({"finding": id, "by": via.as_str()})).collect::<Vec<_>>(),
                    })).collect::<Vec<_>>(),
                    "if_it_fails": "the run prints the driver command for the same targets; they are not counted as measured",
                });
                write(
                    &hdir.join("tests.json"),
                    &(serde_json::to_string_pretty(&doc).unwrap_or_default() + "\n"),
                )?;
                m.insert(
                    "tests_full".into(),
                    json!(ctx.pointer(&format!("harness/{id}/tests.json"))),
                );
            }
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
            } else if manifest.is_second_look() {
                format!(
                    "write a driver that differs from the one of the earlier harness (read its unit; follow the to-dos in the block), write one line per target in {} (what this driver does that the first did not), then dla harness run {id} --mode both",
                    ctx.pointer(&format!("harness/{id}/{DRIVER}/{}", dla_core::harness::VARIES_FILE))
                )
            } else if manifest.is_entry_path() {
                format!(
                    "fill the hooks so that each unit calls only the project's public entry points with data a caller controls, write the one line of {} (what a caller varies per unit), then dla harness run {id} --mode both",
                    ctx.pointer(&format!("harness/{id}/{DRIVER}/{}", dla_core::harness::VARIES_FILE))
                )
            } else if opts.driver && !described.is_empty() {
                format!(
                    "fill the hooks so that each unit reaches the targets listed in the driver, then dla harness run {id} --mode both"
                )
            } else if opts.driver {
                format!("fill the hooks, then dla harness run {id} --mode both")
            } else {
                format!("dla harness run {id}")
            }),
        );
    }
    // A test-loop batch has up to 16 targets and the output is bounded (2 KB): the ids, the tests
    // chosen and why, the count of what is left; the per-target blocks are in `targets.json` and
    // `tests.json`.
    if test_loop_facts.is_some()
        && let Value::Object(m) = &mut summary
    {
        for k in [
            "agent",
            "build",
            "detectors",
            "files",
            "gaps",
            "hooks",
            "instructions",
            "junit",
            "probes",
            "run",
            "tools",
            "workload",
        ] {
            m.remove(k);
        }
        let ids: Vec<Value> = opts
            .probe_findings
            .iter()
            .map(|f| json!(f.to_string()))
            .collect();
        m.insert("targets".into(), Value::Array(ids));
        artifacts.retain(|a| !a.ends_with(PROBES) && !a.ends_with(".java"));
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

#[cfg(test)]
mod node_module_tests {
    use super::*;

    #[test]
    fn a_module_package_is_found_from_the_workload_directory_upwards() {
        let t = tempfile::tempdir().unwrap();
        let work = t.path();
        fs::create_dir_all(work.join("sub/deep")).unwrap();
        assert!(!node_module_package(work, "sub/deep"), "no package.json");
        assert_eq!(node_module_note(work, ".", None), None);
        fs::write(work.join("package.json"), "{\"type\": \"module\"}").unwrap();
        assert!(node_module_package(work, "."));
        assert!(node_module_package(work, "sub/deep"));
        let note = node_module_note(work, "sub", None).unwrap();
        assert!(note.contains("projectImport"), "{note}");
        // The nearest package.json decides: a CommonJS sub-package inside a module package.
        fs::write(work.join("sub/package.json"), "{\"type\": \"commonjs\"}").unwrap();
        assert!(!node_module_package(work, "sub/deep"));
        assert!(node_module_package(work, "."));
    }
}
