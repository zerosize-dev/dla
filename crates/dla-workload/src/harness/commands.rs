//! Build and run command resolution for command mode (spec 4.2) and the build and run
//! plan of driver mode.
//!
//! A workload's `command` is a POSIX shell line (`a && b`, `sh ./build.sh run`,
//! `cargo run -q --bin x`, `go run ./cmd/x`, `npm run load`, `make && ./bin`). [`plan`]
//! turns it into what the harness runs:
//!
//! | Workload command | Build | Measured command |
//! |---|---|---|
//! | `a && b` | `a` (several steps: one `sh -c`) | `b` plus the iteration argument |
//! | `sh ./build.sh run` | none | `sh ./build.sh run {n}` (kept whole) |
//! | `cargo run -q --bin x` | `cargo build --release --bin x` (line-table debug info, target dir in the scratch copy) | the built binary |
//! | `go run ./cmd/x -n 2000` | `go test -run ^$ ...` compile check | `go test ... -run ^TestDLAHarness$` with `DLA_ARGS` |
//! | `npm run load` | none | the script's command line (iteration value replaced) |
//! | `python3 -m pkg.mod` | none | same, with `PYTHONPATH=<agent>` |
//! | `make && ./bin` | `make` | `./bin` with `LD_PRELOAD=<agent>/libdlashim.so` |
//!
//! Tokens `{n}`, `{label}`, `{mode}`, `{out}` and `{cov}` stay in the manifest; the run
//! replaces them per point.

use std::path::{Path, PathBuf};

use dla_core::{
    harness::{AgentKind, CommandSpec},
    runtime::Runtime,
    workload::Workload,
};
use serde_json::json;

use super::templates;

/// Where the pieces of a harness live (absolute paths).
#[derive(Debug, Clone)]
pub struct Layout {
    /// The scratch copy of the project.
    pub work: PathBuf,
    /// Agent files.
    pub agent: PathBuf,
    /// Driver template instance.
    pub driver: PathBuf,
}

/// A file generation writes into the scratch copy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedFile {
    /// Path relative to the scratch copy.
    pub rel: String,
    /// Content.
    pub content: String,
}

/// What command resolution decided.
#[derive(Debug, Clone)]
pub struct Plan {
    /// Built once before measuring.
    pub build: Option<CommandSpec>,
    /// The measured command.
    pub run: CommandSpec,
    /// What measures the process.
    pub agent: AgentKind,
    /// Notes the manifest records as gaps.
    pub gaps: Vec<String>,
    /// Files to write into the scratch copy.
    pub files: Vec<PlannedFile>,
}

/// Extra environment and options the caller decided on.
#[derive(Debug, Clone, Default)]
pub struct Switches {
    /// JVM options appended to `JAVA_TOOL_OPTIONS` after the agent (detector flags).
    pub jvm_options: Vec<String>,
    /// Extra environment for the measured process (`PYTHONASYNCIODEBUG=1`).
    pub env: Vec<(String, String)>,
    /// Add `-XX:StartFlightRecording` (driver mode, JVM).
    pub jfr: bool,
    /// The JVM calls a native library through JNI: preload the allocator shim into it.
    pub ffi: Option<FfiSwitch>,
    /// Driver mode, Kotlin project: the `kotlin-stdlib` jar added to the driver's build and
    /// run class paths.
    pub kotlin_stdlib: Option<String>,
}

/// What a JVM harness needs to measure the native side of a JNI boundary.
#[derive(Debug, Clone, Default)]
pub struct FfiSwitch {
    /// Absolute directories that may hold the built native library. They are put first on
    /// `LD_LIBRARY_PATH`, so the JVM keeps its own and the system library directories.
    /// A directory that does not exist is harmless.
    pub lib_dirs: Vec<String>,
}

/// Path separator of `PYTHONPATH`, class paths and `PATH`.
pub fn path_sep() -> &'static str {
    if cfg!(windows) { ";" } else { ":" }
}

/// File name of the allocator shim on this OS.
pub fn shim_file() -> &'static str {
    if cfg!(target_os = "macos") {
        "libdlashim.dylib"
    } else {
        "libdlashim.so"
    }
}

/// Environment variable that preloads the shim on this OS.
pub fn preload_var() -> &'static str {
    if cfg!(target_os = "macos") {
        "DYLD_INSERT_LIBRARIES"
    } else {
        "LD_PRELOAD"
    }
}

fn abs(p: &Path) -> String {
    dla_core::paths::to_slash(p)
}

/// Splits a shell line on `&&` outside quotes.
pub fn split_chain(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match quote {
            Some(q) => {
                cur.push(c);
                if c == q {
                    quote = None;
                }
            }
            None => {
                if c == '\'' || c == '"' {
                    quote = Some(c);
                    cur.push(c);
                } else if c == '&' && chars.get(i + 1) == Some(&'&') {
                    parts.push(cur.trim().to_owned());
                    cur.clear();
                    i += 1;
                } else {
                    cur.push(c);
                }
            }
        }
        i += 1;
    }
    parts.push(cur.trim().to_owned());
    parts.retain(|p| !p.is_empty());
    parts
}

/// Splits a simple command line into words (quotes honoured). `None` when the line uses
/// shell syntax that a direct `exec` would not interpret (`$`, pipes, redirects, globs...).
pub fn shell_words(s: &str) -> Option<Vec<String>> {
    let mut words = Vec::new();
    let mut cur = String::new();
    let mut in_word = false;
    let mut quote: Option<char> = None;
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        match quote {
            Some('\'') => {
                if c == '\'' {
                    quote = None;
                } else {
                    cur.push(c);
                }
            }
            Some(_) => match c {
                '"' => quote = None,
                '$' | '`' => return None,
                '\\' => cur.push(chars.next()?),
                _ => cur.push(c),
            },
            None => match c {
                '\'' | '"' => {
                    quote = Some(c);
                    in_word = true;
                }
                '\\' => {
                    cur.push(chars.next()?);
                    in_word = true;
                }
                c if c.is_whitespace() => {
                    if in_word {
                        words.push(std::mem::take(&mut cur));
                        in_word = false;
                    }
                }
                '$' | '`' | '|' | ';' | '<' | '>' | '(' | ')' | '&' | '*' | '?' | '{' | '}'
                | '~' | '#' | '!' => return None,
                _ => {
                    cur.push(c);
                    in_word = true;
                }
            },
        }
    }
    if quote.is_some() {
        return None;
    }
    if in_word {
        words.push(cur);
    }
    Some(words)
}

/// Leading `NAME=value` words of a command line as environment pairs.
pub(crate) fn split_env(words: Vec<String>) -> (Vec<(String, String)>, Vec<String>) {
    let mut env = Vec::new();
    let mut rest = words.into_iter().peekable();
    while let Some(w) = rest.peek() {
        let valid = w.split_once('=').is_some_and(|(k, _)| {
            !k.is_empty()
                && k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                && !k.starts_with(|c: char| c.is_ascii_digit())
        });
        if !valid {
            break;
        }
        let w = rest.next().unwrap_or_default();
        if let Some((k, v)) = w.split_once('=') {
            env.push((k.to_owned(), v.to_owned()));
        }
    }
    (env, rest.collect())
}

fn is_number(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit() || c == '_')
}

/// Puts the iteration count into `argv`: replaces the value of the template's option when
/// the command already passes it, else appends the template's tokens.
pub fn apply_iter(mut argv: Vec<String>, template: Option<&str>) -> Vec<String> {
    let Some(template) = template else {
        return argv;
    };
    let tokens: Vec<&str> = template.split_whitespace().collect();
    match tokens.as_slice() {
        [flag, _] if flag.starts_with('-') => {
            for i in 0..argv.len() {
                if argv[i] == *flag && argv.get(i + 1).is_some_and(|v| is_number(v) || v == "{n}") {
                    argv[i + 1] = "{n}".into();
                    return argv;
                }
                if let Some(v) = argv[i].strip_prefix(&format!("{flag}="))
                    && (is_number(v) || v == "{n}")
                {
                    argv[i] = format!("{flag}={{n}}");
                    return argv;
                }
            }
            argv.push((*flag).to_owned());
            argv.push("{n}".into());
        }
        _ => {
            let last_is_count = argv.len() >= 2
                && argv.last().is_some_and(|l| is_number(l))
                && !argv[argv.len() - 2].starts_with('-');
            if last_is_count {
                let n = argv.len() - 1;
                argv[n] = "{n}".into();
            } else {
                argv.extend(tokens.iter().map(|t| (*t).to_owned()));
            }
        }
    }
    argv
}

/// One shell piece as a command: direct argv when it is simple, else `sh -c`.
fn spec_for(piece: &str, cwd: &str, env: Vec<(String, String)>) -> CommandSpec {
    match shell_words(piece) {
        Some(words) if !words.is_empty() => {
            let (mut pairs, argv) = split_env(words);
            pairs.extend(env);
            CommandSpec {
                argv,
                cwd: cwd.to_owned(),
                env: pairs,
            }
        }
        _ => CommandSpec {
            argv: vec!["sh".into(), "-c".into(), piece.to_owned()],
            cwd: cwd.to_owned(),
            env,
        },
    }
}

/// All pieces but the last as one build command.
fn build_spec(pieces: &[String], cwd: &str) -> Option<CommandSpec> {
    match pieces {
        [] => None,
        [one] => Some(spec_for(one, cwd, Vec::new())),
        many => Some(CommandSpec {
            argv: vec!["sh".into(), "-c".into(), many.join(" && ")],
            cwd: cwd.to_owned(),
            env: Vec::new(),
        }),
    }
}

/// Run command of a piece with the iteration count applied.
fn run_spec(piece: &str, cwd: &str, template: Option<&str>) -> CommandSpec {
    let mut spec = spec_for(piece, cwd, Vec::new());
    if spec.argv.first().map(String::as_str) == Some("sh")
        && spec.argv.get(1).map(String::as_str) == Some("-c")
    {
        if let (Some(t), Some(body)) = (template, spec.argv.get_mut(2)) {
            body.push(' ');
            body.push_str(t);
        }
    } else {
        spec.argv = apply_iter(spec.argv, template);
    }
    spec
}

/// Sets `key` in `spec.env`. A value the command already carries inline
/// (`PYTHONPATH=src python3 ...`) is combined with the injected one, never replaced:
/// `PYTHONPATH`, `LD_PRELOAD`, `LD_LIBRARY_PATH`, `DYLD_LIBRARY_PATH` and `DYLD_INSERT_LIBRARIES` get the injected value first
/// (path separator), `JAVA_TOOL_OPTIONS`, `NODE_OPTIONS` and `GRADLE_OPTS` get it last (space).
fn set_env(spec: &mut CommandSpec, key: &str, value: String) {
    if let Some(e) = spec.env.iter_mut().find(|(k, _)| k == key) {
        let old = std::mem::take(&mut e.1);
        e.1 = if old.is_empty() {
            value
        } else {
            match key {
                "PYTHONPATH" => format!("{value}{}{old}", path_sep()),
                "LD_PRELOAD"
                | "DYLD_INSERT_LIBRARIES"
                | "LD_LIBRARY_PATH"
                | "DYLD_LIBRARY_PATH" => {
                    format!("{value}:{old}")
                }
                "JAVA_TOOL_OPTIONS" | "NODE_OPTIONS" | "GRADLE_OPTS" => format!("{old} {value}"),
                _ => value,
            }
        };
    } else {
        spec.env.push((key.to_owned(), value));
    }
}

/// The variable the JVM builds its default `java.library.path` from.
fn library_path_var() -> &'static str {
    if cfg!(target_os = "macos") {
        "DYLD_LIBRARY_PATH"
    } else {
        "LD_LIBRARY_PATH"
    }
}

fn quote_opt(path: &str) -> String {
    if path.contains(char::is_whitespace) {
        format!("\"{path}\"")
    } else {
        path.to_owned()
    }
}

/// `JAVA_TOOL_OPTIONS` value injecting the agent.
pub fn jvm_options(agent_dir: &Path, sw: &Switches) -> String {
    let jar = abs(&agent_dir.join("dla-agent.jar"));
    let mut opts = vec![
        format!("-javaagent:{}", quote_opt(&jar)),
        "-XX:NativeMemoryTracking=summary".to_owned(),
    ];
    if sw.jfr {
        opts.push("-XX:StartFlightRecording=filename={out}.jfr,settings=profile".to_owned());
    }
    opts.extend(sw.jvm_options.iter().cloned());
    opts.join(" ")
}

/// Preloads the allocator shim into the JVM of `run` (narrowed to `java` processes) and makes
/// the Rust cdylib carry line tables, so the native stacks of a JNI library resolve to
/// project lines. The build step gets the same cargo setting.
pub fn attach_ffi(
    run: &mut CommandSpec,
    build: Option<&mut CommandSpec>,
    layout: &Layout,
    lib_dirs: &[String],
    gaps: &mut Vec<String>,
) {
    if cfg!(windows) {
        gaps.push("no allocator shim on Windows: native allocations are not attributed".to_owned());
        return;
    }
    set_env(run, preload_var(), abs(&layout.agent.join(shim_file())));
    set_env(run, "DLA_SHIM_ONLY", "java".to_owned());
    if !lib_dirs.is_empty() {
        // Not `-Djava.library.path`: that would replace the JVM's default path. The JVM builds
        // its default from `LD_LIBRARY_PATH` (`DYLD_LIBRARY_PATH` on macOS) and keeps the
        // system directories.
        set_env(run, library_path_var(), lib_dirs.join(":"));
    }
    // A Gradle daemon is detached and would outlive the run with the shim loaded.
    let gradle = |a: &String| {
        matches!(
            a.rsplit('/').next().unwrap_or(a),
            "gradle" | "gradlew" | "gradle.bat" | "gradlew.bat"
        )
    };
    if run.argv.iter().any(gradle) {
        set_env(run, "GRADLE_OPTS", "-Dorg.gradle.daemon=false".to_owned());
        gaps.push(
            "note: Gradle runs without its daemon so no instrumented JVM outlives the run"
                .to_owned(),
        );
    }
    set_env(
        run,
        "CARGO_PROFILE_RELEASE_DEBUG",
        "line-tables-only".to_owned(),
    );
    if let Some(b) = build {
        set_env(
            b,
            "CARGO_PROFILE_RELEASE_DEBUG",
            "line-tables-only".to_owned(),
        );
    }
}

/// Directory of the first entry of the `-cp` option of a `java` command.
fn classes_dir(run_piece: &str) -> Option<String> {
    let words = shell_words(run_piece)?;
    let i = words
        .iter()
        .position(|w| matches!(w.as_str(), "-cp" | "-classpath" | "--class-path"))?;
    words
        .get(i + 1)?
        .split([':', ';'])
        .next()
        .map(str::to_owned)
}

/// Resolves the commands of a command-mode harness.
///
/// `iter_arg` is the template with `{n}` (detected or given). Errors carry the reason the
/// command shape is not supported.
pub fn plan(
    w: &Workload,
    iter_arg: Option<&str>,
    layout: &Layout,
    sw: &Switches,
    package_script: Option<&str>,
) -> Result<Plan, String> {
    let cwd = w.cwd.as_str();
    let mut p = match w.runtime {
        Runtime::Jvm => plan_jvm(w, iter_arg, layout, sw)?,
        Runtime::Python => plan_python(w, iter_arg, layout),
        Runtime::Node => plan_node(w, iter_arg, layout, package_script),
        Runtime::Go => plan_go(w, iter_arg, layout)?,
        Runtime::Rust => plan_rust(w, iter_arg, layout)?,
        Runtime::Native => plan_native(w, iter_arg, layout),
    };
    if p.run.cwd.is_empty() {
        p.run.cwd = cwd.to_owned();
    }
    for (k, v) in &sw.env {
        if !p.run.env.iter().any(|(ek, _)| ek == k) {
            p.run.env.push((k.clone(), v.clone()));
        }
    }
    Ok(p)
}

fn plan_jvm(
    w: &Workload,
    iter_arg: Option<&str>,
    layout: &Layout,
    sw: &Switches,
) -> Result<Plan, String> {
    let pieces = split_chain(&w.command);
    let (build, run_piece): (Option<CommandSpec>, &String) = match pieces.split_last() {
        Some((last, before)) => (build_spec(before, &w.cwd), last),
        None => return Err("the workload has an empty command".into()),
    };
    let mut gaps = Vec::new();
    if w.command.contains("build.sh") {
        gaps.push(
            "note: the build script compiles and runs the unit tests again at every point; the measured program is the last JVM it starts".to_owned(),
        );
    }
    let mut run = run_spec(run_piece, &w.cwd, iter_arg);
    set_env(
        &mut run,
        "JAVA_TOOL_OPTIONS",
        jvm_options(&layout.agent, sw),
    );
    let mut build = build;
    if let Some(f) = &sw.ffi {
        attach_ffi(&mut run, build.as_mut(), layout, &f.lib_dirs, &mut gaps);
    }
    Ok(Plan {
        build,
        run,
        agent: AgentKind::JvmAgent,
        gaps,
        files: Vec::new(),
    })
}

fn plan_python(w: &Workload, iter_arg: Option<&str>, layout: &Layout) -> Plan {
    let pieces = split_chain(&w.command);
    let (last, before) = pieces
        .split_last()
        .map_or((w.command.as_str(), &[][..]), |(l, b)| (l.as_str(), b));
    let mut run = run_spec(last, &w.cwd, iter_arg);
    set_env(&mut run, "PYTHONPATH", abs(&layout.agent));
    Plan {
        build: build_spec(before, &w.cwd),
        run,
        agent: AgentKind::PySitecustomize,
        gaps: Vec::new(),
        files: Vec::new(),
    }
}

fn node_options(layout: &Layout) -> String {
    format!(
        "--require {}",
        quote_opt(&abs(&layout.agent.join("dla-preload.js")))
    )
}

fn plan_node(
    w: &Workload,
    iter_arg: Option<&str>,
    layout: &Layout,
    package_script: Option<&str>,
) -> Plan {
    let mut run = if let Some(script) = package_script {
        match shell_words(script).map(split_env) {
            Some((env, argv)) if !argv.is_empty() => CommandSpec {
                argv: apply_iter(argv, iter_arg),
                cwd: w.cwd.clone(),
                env,
            },
            _ => npm_fallback(w, iter_arg),
        }
    } else {
        run_spec(&w.command, &w.cwd, iter_arg)
    };
    set_env(&mut run, "NODE_OPTIONS", node_options(layout));
    set_env(&mut run, "NODE_V8_COVERAGE", "{cov}".to_owned());
    Plan {
        build: None,
        run,
        agent: AgentKind::NodePreload,
        gaps: Vec::new(),
        files: Vec::new(),
    }
}

/// `npm run x -- --iterations {n}` for a script that needs a shell.
fn npm_fallback(w: &Workload, iter_arg: Option<&str>) -> CommandSpec {
    let mut argv: Vec<String> = w.command.split_whitespace().map(str::to_owned).collect();
    if let Some(t) = iter_arg {
        argv.push("--".into());
        argv.extend(t.split_whitespace().map(str::to_owned));
    }
    CommandSpec {
        argv,
        cwd: w.cwd.clone(),
        env: Vec::new(),
    }
}

/// Words after `go run` (flags before the package skipped): the package and its arguments.
fn go_package_and_args(command: &str) -> Option<(String, Vec<String>)> {
    let words = shell_words(command)?;
    let run = words.iter().position(|w| w == "run")?;
    let mut rest = words[run + 1..].iter().peekable();
    while rest.peek().is_some_and(|w| w.starts_with('-')) {
        let flag = rest.next()?;
        // `-tags x` and friends take a value.
        if matches!(
            flag.as_str(),
            "-tags" | "-ldflags" | "-gcflags" | "-mod" | "-o"
        ) {
            rest.next();
        }
    }
    let pkg = rest.next()?.clone();
    Some((pkg, rest.cloned().collect()))
}

/// The package directory (relative to the workload's cwd) of a `go run` command.
pub fn go_package_dir(command: &str) -> Option<String> {
    go_package_and_args(command).map(|(p, _)| p.trim_start_matches("./").to_owned())
}

fn plan_go(w: &Workload, iter_arg: Option<&str>, layout: &Layout) -> Result<Plan, String> {
    let (pkg, args) = go_package_and_args(&w.command)
        .ok_or_else(|| format!("{:?} is not a `go run <package> [args]` command", w.command))?;
    let args = apply_iter(args, iter_arg);
    let dir = pkg.trim_start_matches("./").to_owned();
    let pkg_arg = if pkg.starts_with('.') {
        pkg.clone()
    } else {
        format!("./{pkg}")
    };
    let test_dir = if w.cwd == "." || w.cwd.is_empty() {
        dir.clone()
    } else {
        format!("{}/{dir}", w.cwd)
    };
    let wrapper = templates::fill(templates::GO_WRAPPER, &[("PACKAGE", "main")])?;
    let _ = layout;
    let build = CommandSpec {
        argv: [
            "go",
            "test",
            "-vet=off",
            "-count=1",
            "-run",
            "^$",
            "-coverpkg=./...",
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .chain([pkg_arg.clone()])
        .collect(),
        cwd: w.cwd.clone(),
        env: Vec::new(),
    };
    let run = CommandSpec {
        argv: [
            "go",
            "test",
            "-vet=off",
            "-count=1",
            "-timeout",
            "0",
            "-run",
            "^TestDLAHarness$",
            "-coverprofile",
            "{cov}/{label}.out",
            "-coverpkg=./...",
        ]
        .iter()
        .map(|s| (*s).to_owned())
        .chain([pkg_arg])
        .collect(),
        cwd: w.cwd.clone(),
        env: vec![("DLA_ARGS".into(), json!(args).to_string())],
    };
    Ok(Plan {
        build: Some(build),
        run,
        agent: AgentKind::GoTestWrapper,
        gaps: Vec::new(),
        files: vec![PlannedFile {
            rel: format!("{test_dir}/{}", templates::GO_WRAPPER_FILE),
            content: wrapper,
        }],
    })
}

fn plan_rust(w: &Workload, iter_arg: Option<&str>, layout: &Layout) -> Result<Plan, String> {
    let words = shell_words(&w.command)
        .ok_or_else(|| format!("{:?} is not a plain `cargo run` command", w.command))?;
    let run_at = words
        .iter()
        .position(|x| x == "run")
        .ok_or_else(|| format!("{:?} is not a `cargo run` command", w.command))?;
    let (cargo_args, prog_args) = match words.iter().position(|x| x == "--") {
        Some(i) => (&words[run_at + 1..i], words[i + 1..].to_vec()),
        None => (&words[run_at + 1..], Vec::new()),
    };
    let mut bin: Option<String> = None;
    let mut keep: Vec<String> = Vec::new();
    let mut it = cargo_args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-q" | "--quiet" | "--release" | "-r" => {}
            "--bin" => bin = it.next().cloned(),
            "-p" | "--package" | "--manifest-path" | "--features" | "-F" | "--profile" => {
                keep.push(a.clone());
                keep.extend(it.next().cloned());
            }
            other if other.starts_with('-') => keep.push(other.to_owned()),
            _ => {}
        }
    }
    let bin = bin.ok_or_else(|| format!("{:?} names no --bin", w.command))?;
    let target = layout.work.join("target");
    let mut build_argv = vec![
        "cargo".to_owned(),
        "build".into(),
        "--release".into(),
        "--bin".into(),
        bin.clone(),
    ];
    build_argv.extend(keep.into_iter().filter(|k| k != "--profile"));
    let exe = format!("{bin}{}", if cfg!(windows) { ".exe" } else { "" });
    let mut run_argv = vec![abs(&target.join("release").join(exe))];
    run_argv.extend(apply_iter(prog_args, iter_arg));
    let mut run = CommandSpec {
        argv: run_argv,
        cwd: w.cwd.clone(),
        env: Vec::new(),
    };
    let mut gaps = Vec::new();
    let agent = attach_shim(&mut run, layout, &mut gaps);
    Ok(Plan {
        build: Some(CommandSpec {
            argv: build_argv,
            cwd: w.cwd.clone(),
            env: vec![
                (
                    "CARGO_PROFILE_RELEASE_DEBUG".into(),
                    "line-tables-only".into(),
                ),
                ("CARGO_TARGET_DIR".into(), abs(&target)),
            ],
        }),
        run,
        agent,
        gaps,
        files: Vec::new(),
    })
}

/// Preloads the allocator shim into `run`; on Windows records the gap instead.
fn attach_shim(run: &mut CommandSpec, layout: &Layout, gaps: &mut Vec<String>) -> AgentKind {
    if cfg!(windows) {
        gaps.push("no allocator shim on Windows: only OS-level columns are measured".to_owned());
        return AgentKind::None;
    }
    set_env(run, preload_var(), abs(&layout.agent.join(shim_file())));
    AgentKind::AllocShim
}

/// Basename of the program `piece` starts when it is a native executable (ELF or Mach-O)
/// found in the scratch copy; `None` for shells, scripts and anything not found.
fn native_executable(work: &Path, cwd: &str, piece: &str) -> Option<String> {
    let words: Vec<String> = piece.split_whitespace().map(str::to_owned).collect();
    let (_, rest) = split_env(words);
    let tok = rest.first()?;
    if matches!(
        tok.as_str(),
        "sh" | "bash" | "dash" | "zsh" | "exec" | "env" | "make"
    ) {
        return None;
    }
    let tok = tok.trim_matches(|c| c == '"' || c == '\'');
    let base = tok.rsplit('/').next().unwrap_or(tok);
    if base.is_empty() || base.ends_with(".sh") {
        return None;
    }
    let path = work.join(cwd).join(tok);
    is_native_binary(&path).then(|| base.to_owned())
}

/// Basename of the program `piece` starts when it is an explicit relative path (`./prog`,
/// `bin/prog`) without an extension that does not exist in the scratch copy yet: the name a
/// build step before it is expected to produce. `None` for anything that exists (that case
/// is [`native_executable`]'s), for shells, interpreters and scripts, and for bare names.
fn to_be_built_executable(work: &Path, cwd: &str, piece: &str) -> Option<String> {
    let words: Vec<String> = piece.split_whitespace().map(str::to_owned).collect();
    let (_, rest) = split_env(words);
    let tok = rest.first()?;
    let tok = tok.trim_matches(|c| c == '"' || c == '\'');
    if !tok.contains('/') || tok.starts_with('/') || tok.contains("..") {
        return None;
    }
    let base = tok.rsplit('/').next().unwrap_or(tok);
    if base.is_empty() || base.contains('.') || work.join(cwd).join(tok).exists() {
        return None;
    }
    Some(base.to_owned())
}

/// Whether `path` is an ELF or Mach-O executable (by its first four bytes), not a script.
pub(crate) fn is_native_binary(path: &Path) -> bool {
    let mut head = [0u8; 4];
    let Ok(mut f) = std::fs::File::open(path) else {
        return false;
    };
    if std::io::Read::read_exact(&mut f, &mut head).is_err() {
        return false;
    }
    let elf = head == [0x7f, b'E', b'L', b'F'];
    let macho = matches!(
        u32::from_le_bytes(head),
        0xfeed_face | 0xfeed_facf | 0xcefa_edfe | 0xcffa_edfe | 0xcafe_babe | 0xbeba_feca
    );
    elf || macho
}

fn plan_native(w: &Workload, iter_arg: Option<&str>, layout: &Layout) -> Plan {
    let pieces = split_chain(&w.command);
    let (last, before) = pieces
        .split_last()
        .map_or((w.command.as_str(), &[][..]), |(l, b)| (l.as_str(), b));
    let mut run = run_spec(last, &w.cwd, iter_arg);
    let mut gaps = Vec::new();
    let agent = attach_shim(&mut run, layout, &mut gaps);
    if agent == AgentKind::AllocShim {
        // Narrow the shim to the program itself only when the command starts a real
        // executable of the scratch copy. A shell, a script or anything else leaves the
        // shim loaded in every process; `select_measured` drops the shell's record.
        // A build step in front of the command (`make && ./prog`) produces the executable
        // after this plan is made (the probe's build no longer leaves it in the project),
        // so a path that does not exist yet is narrowed by its name too.
        let found = native_executable(&layout.work, &w.cwd, last).or_else(|| {
            (!before.is_empty())
                .then(|| to_be_built_executable(&layout.work, &w.cwd, last))
                .flatten()
        });
        if let Some(base) = found {
            set_env(&mut run, "DLA_SHIM_ONLY", base);
        }
    }
    Plan {
        build: build_spec(before, &w.cwd),
        run,
        agent,
        gaps,
        files: Vec::new(),
    }
}

/// What a driver-mode harness needs from the caller beyond the workload.
#[derive(Debug, Clone, Default)]
pub struct DriverSpec {
    /// Where the driver source lives inside the scratch copy (relative).
    pub work_rel: String,
    /// JVM: fully qualified driver class. Go: package directory (relative to cwd).
    pub target: String,
    /// Build with the goleak tag (Go).
    pub goleak: bool,
    /// Rust: crate directory, relative to the scratch copy (the workload's cwd).
    pub crate_dir: String,
    /// JVM: run the project's own JUnit 5 tests in a loop (no hooks, see [`JunitSpec`]).
    pub junit: Option<JunitSpec>,
    /// JVM: the project's Gradle or Maven build resolves the driver's class path (the main
    /// runtime class path, see [`ClasspathSet::Main`]) instead of the workload's classes
    /// directory; the hooks driver is compiled and run against it.
    pub main_build: Option<JunitSpec>,
}

/// Build tool of a JUnit repeat harness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JunitTool {
    /// Gradle (or its wrapper).
    Gradle,
    /// Maven (or its wrapper).
    Maven,
}

impl JunitTool {
    /// The word `dla-junit-build.sh` takes.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gradle => "gradle",
            Self::Maven => "maven",
        }
    }
}

/// Which class path the build step resolves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClasspathSet {
    /// The tests' runtime class path (JUnit repeat harness).
    Test,
    /// The main runtime class path: the project's main classes of every language (Gradle:
    /// `build/classes/java/main` and `build/classes/kotlin/main`), resources and the runtime
    /// dependency jars (driver harness).
    Main,
}

impl ClasspathSet {
    /// The word `dla-junit-build.sh` takes as its mode.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::Main => "main",
        }
    }
}

/// What a JUnit repeat harness, or a build-resolved driver harness, is built from.
#[derive(Debug, Clone)]
pub struct JunitSpec {
    /// Which build tool resolves the test class path.
    pub tool: JunitTool,
    /// The tool executable (`gradle`, `mvn`) or the project's wrapper script (absolute).
    pub launcher: String,
}

/// File name of the JUnit repeat driver class (default package).
pub const JUNIT_DRIVER_CLASS: &str = "DlaJUnitLoop";
/// File names the JUnit repeat build step reads or writes in the driver directory.
pub const JUNIT_BUILD_SCRIPT: &str = "dla-junit-build.sh";
/// Gradle init script file name.
pub const JUNIT_INIT_SCRIPT: &str = "dla-junit.init.gradle";
/// The Java argument file the build step writes (`java @run.args`).
pub const JUNIT_ARGS_FILE: &str = "run.args";

/// Build and run of a JUnit repeat harness: the build step resolves the class path of the
/// project's tests and compiles the driver against it, the run is `java @run.args DlaJUnitLoop`.
fn plan_junit(cwd: &str, layout: &Layout, sw: &Switches, j: &JunitSpec) -> Plan {
    plan_resolved(
        cwd,
        layout,
        sw,
        j,
        ClasspathSet::Test,
        &format!("{JUNIT_DRIVER_CLASS}.java"),
        JUNIT_DRIVER_CLASS,
    )
}

/// Build and run of a driver whose class path the build tool resolves: the build step writes
/// the class path, compiles `source` (a file of the driver directory) against it and writes the
/// argument file; the run is `java @run.args <class>`.
fn plan_resolved(
    cwd: &str,
    layout: &Layout,
    sw: &Switches,
    j: &JunitSpec,
    set: ClasspathSet,
    source: &str,
    class: &str,
) -> Plan {
    let driver = |file: &str| abs(&layout.driver.join(file));
    let jar = abs(&layout.agent.join("dla-agent.jar"));
    let mut build = CommandSpec {
        argv: vec![
            "sh".into(),
            driver(JUNIT_BUILD_SCRIPT),
            j.tool.as_str().into(),
            j.launcher.clone(),
            abs(&layout.driver),
            jar,
            path_sep().into(),
            set.as_str().into(),
            driver(source),
        ],
        cwd: cwd.to_owned(),
        env: Vec::new(),
    };
    let mut sw = sw.clone();
    sw.jfr = true;
    let mut run = CommandSpec {
        argv: vec![
            "java".into(),
            format!("@{}", driver(JUNIT_ARGS_FILE)),
            class.into(),
        ],
        cwd: cwd.to_owned(),
        env: Vec::new(),
    };
    set_env(
        &mut run,
        "JAVA_TOOL_OPTIONS",
        jvm_options(&layout.agent, &sw),
    );
    let mut gaps = Vec::new();
    if let Some(f) = &sw.ffi {
        attach_ffi(&mut run, Some(&mut build), layout, &f.lib_dirs, &mut gaps);
    }
    Plan {
        build: Some(build),
        run,
        agent: AgentKind::DriverJvm,
        gaps,
        files: Vec::new(),
    }
}

/// Plans the build and run of a driver-mode harness for runtime `rt`.
pub fn plan_driver(
    rt: Runtime,
    w: Option<&Workload>,
    cwd: &str,
    layout: &Layout,
    sw: &Switches,
    spec: &DriverSpec,
) -> Result<Plan, String> {
    let driver = |file: &str| abs(&layout.driver.join(file));
    let mut plan = match rt {
        Runtime::Jvm if spec.junit.is_some() => {
            let j = spec.junit.as_ref().ok_or("no JUnit spec")?;
            plan_junit(cwd, layout, sw, j)
        }
        Runtime::Jvm if spec.main_build.is_some() => {
            let j = spec.main_build.as_ref().ok_or("no build spec")?;
            let file = spec
                .work_rel
                .rsplit('/')
                .next()
                .filter(|f| !f.is_empty())
                .ok_or("no driver source")?;
            plan_resolved(cwd, layout, sw, j, ClasspathSet::Main, file, &spec.target)
        }
        Runtime::Jvm => {
            let (pre, run_piece): (Vec<String>, String) = match w {
                Some(w) => {
                    let mut pieces = split_chain(&w.command);
                    let last = pieces.pop().unwrap_or_default();
                    if let Some(l) = pieces.last()
                        && l.is_empty()
                    {
                        pieces.pop();
                    }
                    (pieces, last)
                }
                None => (Vec::new(), String::new()),
            };
            let mut classes = classes_dir(&run_piece).unwrap_or_else(|| "out".to_owned());
            let mut pre = pre;
            if run_piece.contains("build.sh") {
                // `sh ./build.sh run` -> build with `sh ./build.sh` (no run argument).
                pre.push(
                    run_piece
                        .trim_end_matches(" run")
                        .trim_end_matches(" 'run'")
                        .to_owned(),
                );
                classes = "out".to_owned();
            }
            let jar = abs(&layout.agent.join("dla-agent.jar"));
            let sep = path_sep();
            let std_cp = sw
                .kotlin_stdlib
                .as_deref()
                .map_or(String::new(), |j| format!("{sep}{}", quote_opt(j)));
            let javac = format!(
                "javac -cp {classes}{sep}{}{std_cp} -d {classes} {}",
                quote_opt(&jar),
                spec.work_rel
            );
            let mut all = pre;
            all.push(javac);
            let build = CommandSpec {
                argv: vec!["sh".into(), "-c".into(), all.join(" && ")],
                cwd: cwd.to_owned(),
                env: Vec::new(),
            };
            let mut sw = sw.clone();
            sw.jfr = true;
            let mut run = CommandSpec {
                argv: vec![
                    "java".into(),
                    "-cp".into(),
                    format!(
                        "{classes}{sep}{jar}{}",
                        sw.kotlin_stdlib
                            .as_deref()
                            .map_or(String::new(), |j| format!("{sep}{j}"))
                    ),
                    spec.target.clone(),
                ],
                cwd: cwd.to_owned(),
                env: Vec::new(),
            };
            set_env(
                &mut run,
                "JAVA_TOOL_OPTIONS",
                jvm_options(&layout.agent, &sw),
            );
            let mut build = build;
            let mut gaps = Vec::new();
            if let Some(f) = &sw.ffi {
                attach_ffi(&mut run, Some(&mut build), layout, &f.lib_dirs, &mut gaps);
            }
            Plan {
                build: Some(build),
                run,
                agent: AgentKind::DriverJvm,
                gaps,
                files: Vec::new(),
            }
        }
        Runtime::Python => {
            let mut run = CommandSpec {
                argv: vec!["python3".into(), driver("dla_harness.py")],
                cwd: cwd.to_owned(),
                env: Vec::new(),
            };
            set_env(
                &mut run,
                "PYTHONPATH",
                format!("{}{}.", abs(&layout.agent), path_sep()),
            );
            Plan {
                build: None,
                run,
                agent: AgentKind::DriverPython,
                gaps: Vec::new(),
                files: Vec::new(),
            }
        }
        Runtime::Node => {
            let mut run = CommandSpec {
                argv: vec!["node".into(), driver("dla-harness.js")],
                cwd: cwd.to_owned(),
                env: Vec::new(),
            };
            set_env(&mut run, "NODE_OPTIONS", node_options(layout));
            set_env(
                &mut run,
                "DLA_AGENT",
                abs(&layout.agent.join("dla-preload.js")),
            );
            set_env(&mut run, "NODE_V8_COVERAGE", "{cov}".to_owned());
            Plan {
                build: None,
                run,
                agent: AgentKind::DriverNode,
                gaps: Vec::new(),
                files: Vec::new(),
            }
        }
        Runtime::Go => {
            let pkg = format!("./{}", spec.target.trim_start_matches("./"));
            let tags: Vec<String> = if spec.goleak {
                vec!["-tags".into(), "dlagoleak".into()]
            } else {
                Vec::new()
            };
            let mut build_argv: Vec<String> = ["go", "test", "-vet=off", "-count=1", "-run", "^$"]
                .iter()
                .map(|s| (*s).to_owned())
                .collect();
            build_argv.extend(tags.clone());
            build_argv.push(pkg.clone());
            let mut run_argv: Vec<String> = [
                "go",
                "test",
                "-vet=off",
                "-count=1",
                "-timeout",
                "0",
                "-run",
                "^TestDLADriver$",
            ]
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
            run_argv.extend(tags);
            run_argv.push(pkg);
            Plan {
                build: Some(CommandSpec {
                    argv: build_argv,
                    cwd: cwd.to_owned(),
                    env: Vec::new(),
                }),
                run: CommandSpec {
                    argv: run_argv,
                    cwd: cwd.to_owned(),
                    env: Vec::new(),
                },
                agent: AgentKind::DriverGo,
                gaps: Vec::new(),
                files: Vec::new(),
            }
        }
        Runtime::Rust => {
            let target = layout.work.join("target");
            let exe = format!("dla_harness{}", if cfg!(windows) { ".exe" } else { "" });
            Plan {
                build: Some(CommandSpec {
                    argv: ["cargo", "build", "--release", "--bin", "dla_harness"]
                        .iter()
                        .map(|s| (*s).to_owned())
                        .collect(),
                    cwd: cwd.to_owned(),
                    env: vec![
                        (
                            "CARGO_PROFILE_RELEASE_DEBUG".into(),
                            "line-tables-only".into(),
                        ),
                        ("CARGO_TARGET_DIR".into(), abs(&target)),
                    ],
                }),
                run: CommandSpec {
                    argv: vec![abs(&target.join("release").join(exe))],
                    cwd: cwd.to_owned(),
                    env: Vec::new(),
                },
                agent: AgentKind::DriverRust,
                gaps: Vec::new(),
                files: Vec::new(),
            }
        }
        Runtime::Native => {
            let src = driver("dla_harness.c");
            let find = "$(find . -name '*.c' -not -name 'main.c' -not -path './tests/*' -not -path './test/*' -not -path './build/*' -not -name 'dla_harness.c')";
            Plan {
                build: Some(CommandSpec {
                    argv: vec![
                        "sh".into(),
                        "-c".into(),
                        format!("cc -std=gnu11 -O2 -g -I. -Isrc -o dla_harness {src} {find}"),
                    ],
                    cwd: cwd.to_owned(),
                    env: Vec::new(),
                }),
                run: CommandSpec {
                    argv: vec!["./dla_harness".into()],
                    cwd: cwd.to_owned(),
                    env: Vec::new(),
                },
                agent: AgentKind::DriverNative,
                gaps: Vec::new(),
                files: Vec::new(),
            }
        }
    };
    for (k, v) in &sw.env {
        if !plan.run.env.iter().any(|(ek, _)| ek == k) {
            plan.run.env.push((k.clone(), v.clone()));
        }
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_core::workload::WorkloadKind;

    fn layout() -> Layout {
        Layout {
            work: "/r/harness/H1/work".into(),
            agent: "/r/harness/H1/agent".into(),
            driver: "/r/harness/H1/driver".into(),
        }
    }

    fn wl(rt: Runtime, command: &str) -> Workload {
        Workload {
            id: "x".into(),
            kind: WorkloadKind::Main,
            runtime: rt,
            command: command.into(),
            cwd: ".".into(),
            file: "f".into(),
            entry_points: Vec::new(),
            not_exercisable: None,
            rank: 0,
            iteration_arg: None,
            iteration_default: None,
        }
    }

    fn env<'a>(c: &'a CommandSpec, k: &str) -> Option<&'a str> {
        c.env.iter().find(|(e, _)| e == k).map(|(_, v)| v.as_str())
    }

    #[test]
    fn chain_and_words() {
        assert_eq!(split_chain("a && b -c 'x && y'"), ["a", "b -c 'x && y'"]);
        assert_eq!(shell_words("a 'b c' \"d\"").unwrap(), ["a", "b c", "d"]);
        assert!(shell_words("a $(b)").is_none());
        assert!(shell_words("a | b").is_none());
        assert!(shell_words("find . -name '*.c'").is_some());
        assert_eq!(
            apply_iter(
                vec!["x".into(), "--iterations".into(), "2000".into()],
                Some("--iterations {n}")
            ),
            ["x", "--iterations", "{n}"]
        );
        assert_eq!(
            apply_iter(
                vec!["x".into(), "--iterations=7".into()],
                Some("--iterations {n}")
            ),
            ["x", "--iterations={n}"]
        );
        assert_eq!(
            apply_iter(vec!["x".into()], Some("-n {n}")),
            ["x", "-n", "{n}"]
        );
        assert_eq!(apply_iter(vec!["x".into()], Some("{n}")), ["x", "{n}"]);
        assert_eq!(
            apply_iter(vec!["x".into(), "--w".into(), "4".into()], Some("{n}")),
            ["x", "--w", "4", "{n}"]
        );
    }

    #[test]
    fn jvm_script_stays_whole() {
        let w = wl(Runtime::Jvm, "sh ./build.sh run");
        let p = plan(&w, Some("{n}"), &layout(), &Switches::default(), None).unwrap();
        assert!(p.build.is_none());
        assert_eq!(p.run.argv, ["sh", "./build.sh", "run", "{n}"]);
        let jto = env(&p.run, "JAVA_TOOL_OPTIONS").unwrap();
        assert!(
            jto.contains("-javaagent:/r/harness/H1/agent/dla-agent.jar"),
            "{jto}"
        );
        assert!(jto.contains("NativeMemoryTracking=summary"));
        assert_eq!(p.agent, AgentKind::JvmAgent);
        assert!(p.gaps[0].starts_with("note:"));
    }

    #[test]
    fn a_kotlin_stdlib_goes_on_the_driver_build_and_run_class_paths() {
        let w = wl(Runtime::Jvm, "sh ./build.sh && sh ./build.sh run");
        let spec = DriverSpec {
            work_rel: "driver/Driver.java".into(),
            target: "dla.Driver".into(),
            ..DriverSpec::default()
        };
        let with = Switches {
            kotlin_stdlib: Some("/k/lib/kotlin-stdlib.jar".into()),
            ..Switches::default()
        };
        let p = plan_driver(Runtime::Jvm, Some(&w), ".", &layout(), &with, &spec).unwrap();
        let cp = &p.run.argv[2];
        assert!(
            cp.starts_with("out") && cp.ends_with("/k/lib/kotlin-stdlib.jar"),
            "{cp}"
        );
        assert!(cp.contains("dla-agent.jar"), "{cp}");
        let build = p.build.unwrap().argv[2].clone();
        assert!(
            build.contains("javac -cp out") && build.contains("/k/lib/kotlin-stdlib.jar"),
            "{build}"
        );
        let without = plan_driver(
            Runtime::Jvm,
            Some(&w),
            ".",
            &layout(),
            &Switches::default(),
            &spec,
        )
        .unwrap();
        assert!(!without.run.argv[2].contains("kotlin"));
    }

    // The allocator shim does not exist on Windows (attach_ffi records a gap there).
    #[cfg(not(windows))]
    #[test]
    fn a_jni_boundary_preloads_the_shim_and_builds_with_line_tables() {
        let w = wl(Runtime::Jvm, "sh ./build.sh && sh ./build.sh run");
        let sw = Switches {
            ffi: Some(FfiSwitch {
                lib_dirs: vec!["/r/harness/H1/work/native/target/release".into()],
            }),
            ..Switches::default()
        };
        let p = plan(&w, Some("{n}"), &layout(), &sw, None).unwrap();
        let preload = env(&p.run, preload_var()).unwrap();
        assert!(preload.ends_with(shim_file()), "{preload}");
        assert_eq!(env(&p.run, "DLA_SHIM_ONLY"), Some("java"));
        assert_eq!(
            env(&p.run, "CARGO_PROFILE_RELEASE_DEBUG"),
            Some("line-tables-only")
        );
        assert_eq!(
            env(p.build.as_ref().unwrap(), "CARGO_PROFILE_RELEASE_DEBUG"),
            Some("line-tables-only")
        );
        let jto = env(&p.run, "JAVA_TOOL_OPTIONS").unwrap();
        assert!(!jto.contains("library.path"), "{jto}");
        assert_eq!(
            env(&p.run, library_path_var()),
            Some("/r/harness/H1/work/native/target/release")
        );
        if cfg!(target_os = "macos") {
            assert_eq!(library_path_var(), "DYLD_LIBRARY_PATH");
            assert!(env(&p.run, "LD_LIBRARY_PATH").is_none());
        } else {
            assert_eq!(library_path_var(), "LD_LIBRARY_PATH");
            assert!(env(&p.run, "DYLD_LIBRARY_PATH").is_none());
        }
        // Without the switch none of this appears.
        let plain = plan(&w, Some("{n}"), &layout(), &Switches::default(), None).unwrap();
        assert!(env(&plain.run, "DLA_SHIM_ONLY").is_none());
        assert!(
            !env(&plain.run, "JAVA_TOOL_OPTIONS")
                .unwrap()
                .contains("library.path")
        );
    }

    // The allocator shim does not exist on Windows (attach_ffi records a gap there).
    #[cfg(not(windows))]
    #[test]
    fn a_jni_boundary_keeps_an_inline_library_path_and_stops_the_gradle_daemon() {
        let sw = Switches {
            ffi: Some(FfiSwitch {
                lib_dirs: vec!["/h/work/native/target/release".into()],
            }),
            ..Switches::default()
        };
        let var = library_path_var();
        let w = wl(
            Runtime::Jvm,
            &format!("{var}=native/target/debug java -cp out app.Main"),
        );
        let p = plan(&w, Some("{n}"), &layout(), &sw, None).unwrap();
        assert_eq!(
            env(&p.run, var),
            Some("/h/work/native/target/release:native/target/debug")
        );
        assert!(
            !env(&p.run, "JAVA_TOOL_OPTIONS")
                .unwrap()
                .contains("library.path")
        );
        let w = wl(Runtime::Jvm, "gradle -q run");
        let p = plan(&w, Some("{n}"), &layout(), &sw, None).unwrap();
        assert_eq!(
            env(&p.run, "GRADLE_OPTS"),
            Some("-Dorg.gradle.daemon=false")
        );
        assert!(p.gaps.iter().any(|g| g.contains("daemon")));
        // A value the command carries inline stays, the injected option follows it.
        let w = wl(Runtime::Jvm, "GRADLE_OPTS=-Xmx2g gradle -q run");
        let p = plan(&w, Some("{n}"), &layout(), &sw, None).unwrap();
        assert_eq!(
            env(&p.run, "GRADLE_OPTS"),
            Some("-Xmx2g -Dorg.gradle.daemon=false")
        );
    }

    #[test]
    fn set_env_merges_the_library_path_and_gradle_options_with_inline_values() {
        let mut spec = CommandSpec {
            argv: vec!["java".into()],
            cwd: ".".into(),
            env: Vec::new(),
        };
        spec.env.push(("DYLD_LIBRARY_PATH".into(), "/old".into()));
        set_env(&mut spec, "DYLD_LIBRARY_PATH", "/new".into());
        assert_eq!(spec.env[0].1, "/new:/old");
    }

    #[test]
    fn jvm_chain_splits_build_and_run() {
        let w = wl(
            Runtime::Jvm,
            "mvn -q -o compile && java -cp target/classes billing.Main",
        );
        let sw = Switches {
            jvm_options: vec!["-Dio.netty.leakDetection.level=paranoid".into()],
            ..Switches::default()
        };
        let p = plan(&w, Some("{n}"), &layout(), &sw, None).unwrap();
        assert_eq!(p.build.unwrap().argv, ["mvn", "-q", "-o", "compile"]);
        assert_eq!(
            p.run.argv,
            ["java", "-cp", "target/classes", "billing.Main", "{n}"]
        );
        assert!(
            env(&p.run, "JAVA_TOOL_OPTIONS")
                .unwrap()
                .ends_with("paranoid")
        );
    }

    #[test]
    fn python_gets_the_agent_on_pythonpath() {
        let w = wl(Runtime::Python, "python3 -m outlines.load");
        let p = plan(
            &w,
            Some("--iterations {n}"),
            &layout(),
            &Switches::default(),
            None,
        )
        .unwrap();
        assert_eq!(
            p.run.argv,
            ["python3", "-m", "outlines.load", "--iterations", "{n}"]
        );
        assert_eq!(env(&p.run, "PYTHONPATH"), Some("/r/harness/H1/agent"));
        assert_eq!(p.agent, AgentKind::PySitecustomize);
    }

    #[test]
    fn node_script_is_resolved_from_package_json() {
        let w = wl(Runtime::Node, "npm run load");
        let p = plan(
            &w,
            Some("--iterations {n}"),
            &layout(),
            &Switches::default(),
            Some("node load.js --iterations 2000"),
        )
        .unwrap();
        assert_eq!(p.run.argv, ["node", "load.js", "--iterations", "{n}"]);
        assert_eq!(
            env(&p.run, "NODE_OPTIONS"),
            Some("--require /r/harness/H1/agent/dla-preload.js")
        );
        assert_eq!(env(&p.run, "NODE_V8_COVERAGE"), Some("{cov}"));
        let shell = plan(
            &w,
            Some("--iterations {n}"),
            &layout(),
            &Switches::default(),
            Some("node a.js && node b.js"),
        )
        .unwrap();
        assert_eq!(
            shell.run.argv,
            ["npm", "run", "load", "--", "--iterations", "{n}"]
        );
    }

    #[test]
    fn go_run_becomes_the_test_wrapper() {
        let w = wl(Runtime::Go, "go run ./cmd/invoiced");
        let p = plan(&w, Some("-n {n}"), &layout(), &Switches::default(), None).unwrap();
        let run = p.run.argv.join(" ");
        assert!(run.contains("-run ^TestDLAHarness$"), "{run}");
        assert!(
            run.contains("-coverprofile {cov}/{label}.out -coverpkg=./... ./cmd/invoiced"),
            "{run}"
        );
        assert_eq!(env(&p.run, "DLA_ARGS"), Some(r#"["-n","{n}"]"#));
        assert_eq!(p.files[0].rel, "cmd/invoiced/zz_dla_harness_test.go");
        assert!(p.files[0].content.contains("package main"));
        assert_eq!(p.agent, AgentKind::GoTestWrapper);
    }

    #[test]
    fn cargo_run_becomes_build_and_binary() {
        let w = wl(Runtime::Rust, "cargo run -q --bin rollup-agent");
        let p = plan(
            &w,
            Some("--iterations {n}"),
            &layout(),
            &Switches::default(),
            None,
        )
        .unwrap();
        let b = p.build.unwrap();
        assert_eq!(
            b.argv,
            ["cargo", "build", "--release", "--bin", "rollup-agent"]
        );
        assert_eq!(
            env(&b, "CARGO_PROFILE_RELEASE_DEBUG"),
            Some("line-tables-only")
        );
        assert_eq!(
            env(&b, "CARGO_TARGET_DIR"),
            Some("/r/harness/H1/work/target")
        );
        assert_eq!(
            p.run.argv,
            [
                format!(
                    "/r/harness/H1/work/target/release/rollup-agent{}",
                    std::env::consts::EXE_SUFFIX
                )
                .as_str(),
                "--iterations",
                "{n}"
            ]
        );
        if cfg!(unix) {
            assert_eq!(
                env(&p.run, preload_var()),
                Some(format!("/r/harness/H1/agent/{}", shim_file()).as_str())
            );
            assert_eq!(p.agent, AgentKind::AllocShim);
        }
    }

    #[test]
    fn make_and_binary() {
        let w = wl(Runtime::Native, "make && ./framedec");
        let p = plan(
            &w,
            Some("--iterations {n}"),
            &layout(),
            &Switches::default(),
            None,
        )
        .unwrap();
        assert_eq!(p.build.unwrap().argv, ["make"]);
        assert_eq!(p.run.argv, ["./framedec", "--iterations", "{n}"]);
        let cm = wl(
            Runtime::Native,
            "cmake -S . -B build && cmake --build build && ./build/app",
        );
        let p = plan(&cm, Some("-n {n}"), &layout(), &Switches::default(), None).unwrap();
        assert_eq!(p.build.unwrap().argv[..2], ["sh", "-c"]);
        assert_eq!(p.run.argv, ["./build/app", "-n", "{n}"]);
    }

    fn plan_of(rt: Runtime, cmd: &str, layout: &Layout) -> Plan {
        plan(
            &wl(rt, cmd),
            Some("--iterations {n}"),
            layout,
            &Switches::default(),
            None,
        )
        .unwrap()
    }

    #[test]
    fn inline_assignments_are_combined_with_the_injection_not_replaced() {
        let sep = path_sep();
        let p = plan_of(
            Runtime::Python,
            "PYTHONPATH=src python3 -m pkg.main",
            &layout(),
        );
        assert_eq!(
            env(&p.run, "PYTHONPATH"),
            Some(format!("/r/harness/H1/agent{sep}src").as_str())
        );
        let p = plan_of(
            Runtime::Node,
            "NODE_OPTIONS=--max-old-space-size=64 node app.js",
            &layout(),
        );
        let node = env(&p.run, "NODE_OPTIONS").unwrap();
        assert!(node.starts_with("--max-old-space-size=64 "), "{node}");
        assert!(node.contains("dla-preload"), "{node}");
        let mut spec = CommandSpec {
            argv: vec![],
            cwd: ".".into(),
            env: vec![
                ("JAVA_TOOL_OPTIONS".into(), "-Xmx64m".into()),
                ("LD_PRELOAD".into(), "libmine.so".into()),
            ],
        };
        set_env(&mut spec, "JAVA_TOOL_OPTIONS", "-javaagent:a.jar".into());
        set_env(&mut spec, "LD_PRELOAD", "/agent/shim.so".into());
        assert_eq!(
            env(&spec, "JAVA_TOOL_OPTIONS"),
            Some("-Xmx64m -javaagent:a.jar")
        );
        assert_eq!(env(&spec, "LD_PRELOAD"), Some("/agent/shim.so:libmine.so"));
    }

    #[test]
    fn shim_only_is_set_for_a_real_executable_not_a_script() {
        if cfg!(windows) {
            return;
        }
        let d = tempfile::tempdir().unwrap();
        let l = Layout {
            work: d.path().join("work"),
            agent: d.path().join("agent"),
            driver: d.path().join("driver"),
        };
        std::fs::create_dir_all(&l.work).unwrap();
        std::fs::write(l.work.join("run.sh"), "#!/bin/sh\nexec ./prog\n").unwrap();
        std::fs::write(l.work.join("prog"), b"\x7fELF\x02\x01\x01\0rest").unwrap();
        for cmd in [
            "sh ./run.sh",
            "bash run.sh",
            "./run.sh",
            "make && sh ./run.sh",
        ] {
            let p = plan_of(Runtime::Native, cmd, &l);
            assert_eq!(p.agent, AgentKind::AllocShim, "{cmd}");
            assert_eq!(env(&p.run, "DLA_SHIM_ONLY"), None, "{cmd}");
        }
        let p = plan_of(Runtime::Native, "make && ./prog", &l);
        assert_eq!(env(&p.run, "DLA_SHIM_ONLY"), Some("prog"));
    }

    #[test]
    fn shim_only_names_an_executable_the_build_step_has_yet_to_produce() {
        if cfg!(windows) {
            return;
        }
        let d = tempfile::tempdir().unwrap();
        let l = Layout {
            work: d.path().join("work"),
            agent: d.path().join("agent"),
            driver: d.path().join("driver"),
        };
        std::fs::create_dir_all(&l.work).unwrap();
        // Not built yet: the build step in front of it names the program.
        let p = plan_of(Runtime::Native, "make && ./leasebroker --iterations 5", &l);
        assert_eq!(env(&p.run, "DLA_SHIM_ONLY"), Some("leasebroker"));
        // Nothing builds it, a script name, a bare name: no narrowing.
        for cmd in ["./leasebroker", "make && ./run.sh", "make && prog"] {
            let p = plan_of(Runtime::Native, cmd, &l);
            assert_eq!(env(&p.run, "DLA_SHIM_ONLY"), None, "{cmd}");
        }
        // An absolute path or one that leaves the scratch copy is not the build's output.
        for cmd in [
            "make && /usr/local/bin/leasebroker",
            "make && ../leasebroker",
            "make && ./bin/../../leasebroker",
        ] {
            let p = plan_of(Runtime::Native, cmd, &l);
            assert_eq!(env(&p.run, "DLA_SHIM_ONLY"), None, "{cmd}");
        }
        // A relative path below the copy is.
        let p = plan_of(Runtime::Native, "make && bin/leasebroker", &l);
        assert_eq!(env(&p.run, "DLA_SHIM_ONLY"), Some("leasebroker"));
    }

    #[test]
    fn junit_plan_builds_with_the_script_and_runs_java_with_an_argument_file() {
        let l = layout();
        let spec = DriverSpec {
            junit: Some(JunitSpec {
                tool: JunitTool::Gradle,
                launcher: "/w/gradlew".into(),
            }),
            ..DriverSpec::default()
        };
        let p = plan_driver(Runtime::Jvm, None, "svc", &l, &Switches::default(), &spec).unwrap();
        assert_eq!(p.agent, AgentKind::DriverJvm);
        let build = p.build.unwrap();
        assert_eq!(
            &build.argv[..4],
            [
                "sh",
                &format!("{}/dla-junit-build.sh", abs(&l.driver)),
                "gradle",
                "/w/gradlew"
            ]
        );
        assert_eq!(build.argv[4], abs(&l.driver));
        assert!(build.argv[5].ends_with("dla-agent.jar"));
        assert_eq!(build.argv[6], path_sep());
        assert_eq!(build.argv[7], "test");
        assert_eq!(
            build.argv[8],
            format!("{}/DlaJUnitLoop.java", abs(&l.driver))
        );
        assert_eq!(build.cwd, "svc");
        assert_eq!(
            p.run.argv,
            [
                "java".to_owned(),
                format!("@{}/run.args", abs(&l.driver)),
                "DlaJUnitLoop".to_owned()
            ]
        );
        assert_eq!(p.run.cwd, "svc");
        let jto = env(&p.run, "JAVA_TOOL_OPTIONS").unwrap();
        assert!(
            jto.contains("-javaagent:") && jto.contains("StartFlightRecording"),
            "{jto}"
        );
    }

    #[test]
    fn a_gradle_driver_resolves_the_main_runtime_class_path() {
        let l = layout();
        let spec = DriverSpec {
            work_rel: format!("{}/DlaLoop.java", abs(&l.driver)),
            target: "app.DlaLoop".into(),
            main_build: Some(JunitSpec {
                tool: JunitTool::Gradle,
                launcher: "/w/gradlew".into(),
            }),
            ..DriverSpec::default()
        };
        // A Spring Boot dev-server workload: its command names no classes directory.
        let w = Workload {
            id: "jvm:dev-server:src/main/kotlin/app/App.kt".into(),
            kind: WorkloadKind::DevServer,
            file: "src/main/kotlin/app/App.kt".into(),
            cwd: "svc".into(),
            ..wl(Runtime::Jvm, "gradle -q --offline bootRun")
        };
        let p = plan_driver(
            Runtime::Jvm,
            Some(&w),
            "svc",
            &l,
            &Switches {
                kotlin_stdlib: Some("/m2/kotlin-stdlib-2.2.0.jar".into()),
                ..Switches::default()
            },
            &spec,
        )
        .unwrap();
        let build = p.build.unwrap();
        assert_eq!(
            &build.argv[..4],
            [
                "sh",
                &format!("{}/dla-junit-build.sh", abs(&l.driver)),
                "gradle",
                "/w/gradlew"
            ]
        );
        assert_eq!(build.argv[7], "main");
        assert_eq!(build.argv[8], format!("{}/DlaLoop.java", abs(&l.driver)));
        let all = build.argv.join(" ");
        assert!(
            !all.contains("javac -cp out") && !all.contains("build/classes/java/main"),
            "the build step, not a guessed classes directory: {all}"
        );
        assert!(
            !all.contains("kotlin-stdlib"),
            "the project's own stdlib: {all}"
        );
        assert_eq!(
            p.run.argv,
            [
                "java".to_owned(),
                format!("@{}/run.args", abs(&l.driver)),
                "app.DlaLoop".to_owned()
            ]
        );
        assert_eq!(p.agent, AgentKind::DriverJvm);
        let jto = env(&p.run, "JAVA_TOOL_OPTIONS").unwrap();
        assert!(jto.contains("-javaagent:"), "{jto}");
    }
}
