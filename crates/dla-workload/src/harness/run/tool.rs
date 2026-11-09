//! `dla harness run --tool valgrind|heaptrack`: the points of a command-mode harness run
//! under a native tool, which leaves its report in `out/<label>/`.
//!
//! These runs measure nothing for the series (the tool slows the process by an order of
//! magnitude), so no series is written. The points are the end of the warm-up, the last
//! point and one in between ([`capture::TOOL_POINTS`]). The allocator shim is switched off
//! for them: the tool replaces the allocator itself.
//!
//! * valgrind: `valgrind --tool=memcheck --leak-check=full --show-reachable=yes --xml=yes
//!   --xml-file=out/<label>/valgrind.xml <program>`.
//! * heaptrack: `heaptrack -o out/<label>/heaptrack <program>`, then `heaptrack_print` writes
//!   `heaptrack.txt` (leaks) and, for the last point, `heaptrack-diff.txt` (against the
//!   first).

use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use dla_core::{
    Refusal,
    coverage::{BackendCoverage, CoverageManifest},
    harness::{CaptureFile, HarnessManifest, HarnessMode},
    runtime::Runtime,
    toolctx::ToolCtx,
};

use super::{ExecSpec, Paths, PointEnv, RunOptions, RunOutput, exec, failed, subst, vgdb};
use crate::harness::{
    capture::{self, NativeTool},
    r#gen::TOOL_MISSING,
};
use dla_core::series::KeyMode;

/// Timeout multiplier of a tool run over the manifest's per-process timeout.
fn slowdown(tool: NativeTool) -> u64 {
    match tool {
        NativeTool::Valgrind => 6,
        NativeTool::Heaptrack => 2,
    }
}

/// Finds `name` in the tools directory, then on PATH.
pub(super) fn find_tool(ctx: &ToolCtx, name: &str) -> Option<PathBuf> {
    let exe = |dir: &Path| {
        let p = dir.join(name);
        p.is_file().then_some(p)
    };
    if let Some(dir) = ctx.tools_dir() {
        for d in [dir.join("bin"), dir] {
            if let Some(p) = exe(&d) {
                return Some(p);
            }
        }
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).find_map(|d| exe(&d))
}

fn refuse_missing(ctx: &ToolCtx, runtime: Runtime, tool: NativeTool, what: &str) -> Refusal {
    // The row is the coverage gap; the run goes no further.
    let _ = CoverageManifest::update(&ctx.run_dir, |m| {
        m.set_backend(
            runtime.as_str(),
            tool.as_str(),
            BackendCoverage::missing(
                format!("{what} not found on PATH or in the tools directory"),
                tool.install_hint(),
            ),
        );
    });
    Refusal::new(
        TOOL_MISSING,
        format!(
            "{what} is not installed, so the points cannot run under {}",
            tool.as_str()
        ),
        what.to_owned(),
        format!(
            "{}, then dla harness run <H> --tool {}",
            tool.install_hint(),
            tool.as_str()
        ),
    )
}

/// Drops the allocator shim from an environment: the tool replaces the allocator, and a
/// second interposer would hide the allocations from it.
fn without_shim(env: Vec<(String, String)>) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for (k, v) in env {
        if k == "LD_PRELOAD" || k == "DYLD_INSERT_LIBRARIES" {
            let kept: Vec<&str> = v.split(':').filter(|p| !p.contains("libdlashim")).collect();
            if kept.is_empty() {
                continue;
            }
            out.push((k, kept.join(":")));
        } else if k != "DLA_SHIM_ONLY" {
            out.push((k, v));
        }
    }
    out.push(("DLA_SHIM_DISABLE".to_owned(), "1".to_owned()));
    out
}

fn is_shell(program: &str) -> bool {
    let base = program.rsplit('/').next().unwrap_or(program);
    matches!(base, "sh" | "bash" | "dash" | "zsh" | "make" | "env")
}

/// The file `program` names: relative to `cwd`, or found on `PATH` when it has no slash.
fn resolve_program(cwd: &Path, program: &str) -> Option<PathBuf> {
    if program.contains('/') {
        let p = cwd.join(program);
        return p.is_file().then_some(p);
    }
    let local = cwd.join(program);
    if local.is_file() && !is_shell(program) {
        return Some(local);
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|d| d.join(program))
        .find(|p| p.is_file())
}

/// Whether `program` is a wrapper (a shell, `make`, a `#!` script) rather than the native
/// executable under test. A program that cannot be found counts as native: the spawn error
/// then names it.
fn is_wrapper(cwd: &Path, program: &str) -> bool {
    is_shell(program)
        || resolve_program(cwd, program)
            .is_some_and(|p| !super::super::commands::is_native_binary(&p))
}

/// The `<exe>` of the traced program (`<argv>`, not valgrind's own `<vargv>`) in a valgrind
/// XML file, and its first `<arg>` values.
fn traced_program(xml: &str) -> Option<(String, Vec<String>)> {
    let at = xml.find("<argv>")?;
    let rest = &xml[at..];
    let end = rest.find("</argv>").unwrap_or(rest.len());
    let block = &rest[..end];
    let exe = block
        .split_once("<exe>")?
        .1
        .split_once("</exe>")?
        .0
        .to_owned();
    let args = block
        .split("<arg>")
        .skip(1)
        .filter_map(|a| a.split_once("</arg>").map(|(v, _)| v.to_owned()))
        .collect();
    Some((exe, args))
}

/// Of the per-process XML files `valgrind.<pid>.xml` that a traced wrapper left in `dir`,
/// the one of the program itself: not a shell, `make` or script, and preferably the one that
/// got `n` as an argument. Returns the file and how many candidates there were.
fn pick_program_xml(dir: &Path, cwd: &Path, n: u64) -> Option<(PathBuf, usize)> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.file_name().and_then(|f| f.to_str()).is_some_and(|f| {
                f.starts_with("valgrind.") && f.ends_with(".xml") && f != "valgrind.xml"
            })
        })
        .collect();
    files.sort();
    let mut cands: Vec<(PathBuf, bool)> = Vec::new();
    for f in files {
        let Ok(text) = fs::read_to_string(&f) else {
            continue;
        };
        let Some((exe, args)) = traced_program(&text) else {
            continue;
        };
        if is_wrapper(cwd, &exe) {
            continue;
        }
        let n_text = n.to_string();
        let has_n = args
            .iter()
            .any(|a| *a == n_text || a.ends_with(&format!("={n_text}")));
        cands.push((f, has_n));
    }
    let count = cands.len();
    let pick = cands
        .iter()
        .find(|(_, has_n)| *has_n)
        .or_else(|| cands.first())
        .map(|(f, _)| f.clone())?;
    Some((pick, count))
}

/// The vgdb chain for a valgrind run of a program that is started directly: the `vgdb`
/// executable (next to `valgrind`, else on the tools path) and a private pipe prefix under
/// the run's `tmp/`. A wrapper command is left out: each traced child would stop at exit.
fn vgdb_plan(
    ctx: &ToolCtx,
    valgrind: &Path,
    dir: &Path,
    log: &Path,
) -> Result<vgdb::VgdbPlan, String> {
    let beside = valgrind
        .parent()
        .map(|d| d.join("vgdb"))
        .filter(|p| p.is_file());
    let exe = beside
        .or_else(|| find_tool(ctx, "vgdb"))
        .ok_or("vgdb was not found next to valgrind or on PATH, so no retention chain was taken (install the full valgrind package)")?;
    let label = dir
        .file_name()
        .map_or_else(|| "point".to_owned(), |n| n.to_string_lossy().into_owned());
    let base = ctx
        .tmp_dir()
        .map_err(|e| format!("no scratch directory for the vgdb pipes: {}", e.message))?
        .join(format!("vgdb-{label}"));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).map_err(|e| format!("cannot create {}: {e}", base.display()))?;
    Ok(vgdb::VgdbPlan {
        vgdb: exe,
        prefix: base.join("vg"),
        log: log.to_path_buf(),
        dir: dir.to_path_buf(),
    })
}

/// Where a tool report belongs in the capture list.
struct Slot<'a> {
    label: &'a str,
    seq: u32,
    x: f64,
    ms: Option<u64>,
}

fn file_entry(
    ctx: &ToolCtx,
    paths: &Paths,
    path: &Path,
    tool: NativeTool,
    slot: &Slot<'_>,
) -> Option<CaptureFile> {
    let md = fs::metadata(path)
        .ok()
        .filter(|m| m.is_file() && m.len() > 0)?;
    let name = path.file_name()?.to_string_lossy().into_owned();
    let role = capture::classify(&name).map_or("tool output", |c| c.2);
    Some(CaptureFile {
        kind: tool.kind(),
        path: paths.rel(ctx, path),
        label: slot.label.to_owned(),
        seq: slot.seq,
        x: Some(slot.x),
        role: Some(role.to_owned()),
        bytes: md.len(),
        ms: slot.ms,
        captured_at: capture::rfc3339(md.modified().unwrap_or(std::time::UNIX_EPOCH)),
    })
}

/// Runs the harness points under `tool`; fills `out.captures`, `out.processes`, `out.logs`
/// and `out.limitations`.
pub(super) fn run_tool_points(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    opts: &RunOptions,
    paths: &Paths,
    tool: NativeTool,
    out: &mut RunOutput,
) -> Result<(), Refusal> {
    if manifest.mode != HarnessMode::Command
        || !matches!(manifest.runtime, Runtime::Rust | Runtime::Native)
    {
        return Err(Refusal::new(
            dla_core::error::codes::INVALID_VALUE,
            format!(
                "--tool {} runs the points of a rust or native command-mode harness; {} is a {} {} harness",
                tool.as_str(),
                manifest.id,
                manifest.runtime.as_str(),
                manifest.mode.as_str()
            ),
            "a command-mode harness for rust or native code",
            "dla harness gen --lang rust (or native) without --driver, then dla harness run <H> --tool valgrind",
        ));
    }
    let tool_path = find_tool(ctx, tool.binary())
        .ok_or_else(|| refuse_missing(ctx, manifest.runtime, tool, tool.binary()))?;
    let print_path = if tool == NativeTool::Heaptrack {
        Some(
            find_tool(ctx, "heaptrack_print")
                .ok_or_else(|| refuse_missing(ctx, manifest.runtime, tool, "heaptrack_print"))?,
        )
    } else {
        None
    };
    let secs = opts.timeout_secs.unwrap_or(manifest.timeout_secs).max(1);
    let timeout = Duration::from_secs(secs * slowdown(tool));
    let iterations = opts.iterations.unwrap_or(manifest.iterations);
    let total = manifest.warmup + iterations + opts.extra;
    let points = capture::select_points(total, manifest.warmup, capture::TOOL_POINTS);
    out.tool = Some(tool);
    out.limitations.push(format!(
        "{} slows the program down; points {} of {total} ran under it and no series was written (dla harness run without --tool measures the series)",
        tool.as_str(),
        points.iter().map(u32::to_string).collect::<Vec<_>>().join(", ")
    ));
    let mut first_data: Option<(PathBuf, String, f64)> = None;
    let mut last_data: Option<(PathBuf, String, u32, f64)> = None;
    let mut ran = 0u64;
    for k in points {
        let n = manifest.step.saturating_mul(u64::from(k));
        let label = format!("p{k}-native");
        let pe = PointEnv {
            manifest,
            paths,
            label: label.clone(),
            mode: KeyMode::Native,
            n,
            purge: None,
            rounds: 0,
            capture: None,
            dumps: 0,
        };
        let dir = paths.out.join(&label);
        fs::create_dir_all(&dir).map_err(|e| super::io_refusal(&dir, e))?;
        let command = subst(&manifest.run.argv.join(" "), &[("n", n.to_string())]);
        let mut spec = pe.exec_spec(timeout, &command, &[]);
        spec.env = without_shim(std::mem::take(&mut spec.env));
        spec.sample = false;
        let Some(program) = spec.argv.first().cloned() else {
            continue;
        };
        let target = std::mem::take(&mut spec.argv);
        let wrapper = is_wrapper(&spec.cwd, &program);
        if wrapper && tool == NativeTool::Heaptrack {
            // heaptrack's preload is cleared in the processes the wrapper starts, so it would
            // report the wrapper only.
            let _ = CoverageManifest::update(&ctx.run_dir, |m| {
                m.set_backend(
                    manifest.runtime.as_str(),
                    tool.as_str(),
                    BackendCoverage::blocked(format!(
                        "the run command starts {program}, a wrapper: heaptrack cannot follow its children"
                    )),
                );
            });
            return Err(Refusal::new(
                dla_core::error::codes::INVALID_VALUE,
                format!(
                    "heaptrack cannot follow the children of {program}, so it would report the wrapper instead of the program"
                ),
                "a run command that starts the native executable directly",
                format!(
                    "dla harness gen --lang native --workload <main id> --iter-arg <template> (or point run.argv at the binary), then dla harness run {} --tool heaptrack",
                    manifest.id
                ),
            ));
        }
        // A relative program is resolved against the working directory by the tool.
        let mut argv = vec![tool_path.to_string_lossy().into_owned()];
        let report: PathBuf;
        let mut vgdb_dir: Option<PathBuf> = None;
        match tool {
            NativeTool::Valgrind => {
                report = dir.join("valgrind.xml");
                let _ = fs::remove_file(&report);
                if let Ok(rd) = fs::read_dir(&dir) {
                    for e in rd.flatten() {
                        let f = e.file_name().to_string_lossy().into_owned();
                        if f.starts_with("valgrind.") && f.ends_with(".xml") {
                            let _ = fs::remove_file(e.path());
                        }
                    }
                }
                // Every traced process writes its own file when a wrapper starts the program:
                // processes that share one file overwrite each other.
                let xml_file = if wrapper {
                    dir.join("valgrind.%p.xml")
                } else {
                    report.clone()
                };
                argv.extend([
                    "--tool=memcheck".to_owned(),
                    "--leak-check=full".to_owned(),
                    "--show-reachable=yes".to_owned(),
                    "--num-callers=32".to_owned(),
                    "--xml=yes".to_owned(),
                    format!("--xml-file={}", xml_file.display()),
                ]);
                if wrapper {
                    argv.push("--trace-children=yes".to_owned());
                } else {
                    match vgdb_plan(ctx, &tool_path, &dir, &spec.log) {
                        Ok(plan) => {
                            argv.extend(vgdb::valgrind_flags(&plan.prefix));
                            vgdb_dir = Some(plan.prefix.clone());
                            spec.vgdb = Some(plan);
                        }
                        Err(why) => out.limitations.push(format!("{label}: {why}")),
                    }
                }
            }
            NativeTool::Heaptrack => {
                report = dir.join("heaptrack.gz");
                let _ = fs::remove_file(&report);
                argv.extend([
                    "-o".to_owned(),
                    dir.join("heaptrack").to_string_lossy().into_owned(),
                ]);
            }
        }
        argv.extend(target);
        spec.argv = argv;
        let res = exec(ctx, &spec)?;
        out.processes += 1;
        if let Some(prefix) = &vgdb_dir {
            // Valgrind removes its pipes at exit; a killed run leaves them.
            if let Some(d) = prefix.parent() {
                let _ = fs::remove_dir_all(d);
            }
        }
        let log_rel = paths.rel(ctx, &spec.log);
        out.logs.push(log_rel.clone());
        out.exits.push((label.clone(), res.code));
        if let Some(e) = &res.spawn_error {
            out.limitations.push(format!("{label}: {e}"));
            continue;
        }
        if res.timed_out {
            out.limitations.push(format!(
                "{label} (n={n}) did not finish in {} s under {} and was killed (log {log_rel}); its report is incomplete",
                timeout.as_secs(),
                tool.as_str()
            ));
        } else if let Some(c) = res.code.filter(|c| *c != 0) {
            out.limitations.push(format!(
                "{label} exited with code {c} under {} (log {log_rel})",
                tool.as_str()
            ));
        }
        match tool {
            NativeTool::Valgrind => {
                if wrapper {
                    match pick_program_xml(&dir, &spec.cwd, n) {
                        Some((f, count)) => {
                            let _ = fs::rename(&f, &report);
                            if count > 1 {
                                out.limitations.push(format!(
                                    "{label}: {count} traced programs started through {program}; the report is the one that got n={n}, or the first"
                                ));
                            }
                        }
                        None => out.limitations.push(format!(
                            "{label}: the run command starts {program}, a wrapper, and no traced process was the program itself, so there is no report (log {log_rel})"
                        )),
                    }
                }
                let slot = Slot {
                    label: &label,
                    seq: 1,
                    x: n as f64,
                    ms: Some(res.wall_ms),
                };
                if let Some(chain) = &res.vgdb {
                    for note in &chain.notes {
                        out.limitations.push(format!("{label}: {note}"));
                    }
                    for f in &chain.files {
                        let seq = f
                            .file_name()
                            .and_then(|n| capture::classify(&n.to_string_lossy()))
                            .map_or(1, |c| c.1);
                        let slot = Slot {
                            label: &label,
                            seq,
                            x: n as f64,
                            ms: None,
                        };
                        if let Some(e) = file_entry(ctx, paths, f, tool, &slot) {
                            out.captures.push(e);
                        }
                    }
                }
                match file_entry(ctx, paths, &report, tool, &slot) {
                    Some(f) => {
                        ran += 1;
                        out.captures.push(f);
                    }
                    None => out
                        .limitations
                        .push(format!("{label}: valgrind wrote no XML (log {log_rel})")),
                }
            }
            NativeTool::Heaptrack => {
                // heaptrack names the file `<prefix>.gz` or `<prefix>.zst`.
                let data = ["heaptrack.gz", "heaptrack.zst"]
                    .iter()
                    .map(|n| dir.join(n))
                    .find(|p| p.is_file());
                let Some(data) = data else {
                    out.limitations.push(format!(
                        "{label}: heaptrack wrote no data file (log {log_rel})"
                    ));
                    continue;
                };
                let text = dir.join("heaptrack.txt");
                let print = print_path.as_ref().expect("checked above");
                if print_report(ctx, print, &data, None, &text, &dir, timeout, out)? {
                    let slot = Slot {
                        label: &label,
                        seq: 1,
                        x: n as f64,
                        ms: Some(res.wall_ms),
                    };
                    if let Some(f) = file_entry(ctx, paths, &text, tool, &slot) {
                        ran += 1;
                        out.captures.push(f);
                    }
                    if first_data.is_none() {
                        first_data = Some((data.clone(), label.clone(), n as f64));
                    }
                    last_data = Some((data, label.clone(), k, n as f64));
                }
            }
        }
    }
    if let (Some((first, first_label, _)), Some((last, last_label, _, x)), Some(print)) =
        (&first_data, &last_data, &print_path)
        && first_label != last_label
    {
        let diff = paths.out.join(last_label).join("heaptrack-diff.txt");
        let dir = paths.out.join(last_label);
        if print_report(ctx, print, last, Some(first), &diff, &dir, timeout, out)?
            && let Some(f) = file_entry(
                ctx,
                paths,
                &diff,
                tool,
                &Slot {
                    label: last_label,
                    seq: 2,
                    x: *x,
                    ms: None,
                },
            )
        {
            out.captures.push(f);
        }
    }
    let _ = CoverageManifest::update(&ctx.run_dir, |m| {
        m.note_backend(
            manifest.runtime.as_str(),
            tool.as_str(),
            if ran > 0 {
                BackendCoverage::ran(None, ran)
            } else {
                BackendCoverage::blocked(format!("{} produced no report", tool.as_str()))
            },
        );
    });
    if ran == 0 {
        return Err(failed(
            format!(
                "{} produced no report for harness {}: {}",
                tool.as_str(),
                manifest.id,
                out.limitations.last().cloned().unwrap_or_default()
            ),
            format!("at least one {} report; see the logs", tool.as_str()),
            format!(
                "read the log, fix the harness, then dla harness run {} --tool {}",
                manifest.id,
                tool.as_str()
            ),
        ));
    }
    Ok(())
}

/// Runs `heaptrack_print` on `data` (against `base` for a diff) with the report in `text`.
#[allow(clippy::too_many_arguments)]
fn print_report(
    ctx: &ToolCtx,
    print: &Path,
    data: &Path,
    base: Option<&Path>,
    text: &Path,
    dir: &Path,
    timeout: Duration,
    out: &mut RunOutput,
) -> Result<bool, Refusal> {
    let mut argv = vec![
        print.to_string_lossy().into_owned(),
        "-f".to_owned(),
        data.to_string_lossy().into_owned(),
    ];
    if let Some(b) = base {
        argv.extend(["-d".to_owned(), b.to_string_lossy().into_owned()]);
    }
    argv.extend(
        ["-l", "-p0", "-a0", "-T0", "-n", "200"]
            .iter()
            .map(|s| (*s).to_owned()),
    );
    let command = argv.join(" ");
    let res = exec(
        ctx,
        &ExecSpec {
            argv,
            cwd: dir.to_path_buf(),
            env: Vec::new(),
            log: text.to_path_buf(),
            stderr_log: Some(dir.join("heaptrack-print.err")),
            timeout: timeout.min(Duration::from_secs(300)),
            sample: false,
            command: &command,
            vgdb: None,
        },
    )?;
    out.processes += 1;
    // An empty error file is noise in the harness directory.
    let err = dir.join("heaptrack-print.err");
    if fs::metadata(&err).is_ok_and(|m| m.len() == 0) {
        let _ = fs::remove_file(&err);
    }
    let ok = res.code == Some(0) && !res.timed_out && res.spawn_error.is_none();
    if !ok {
        out.limitations.push(format!(
            "heaptrack_print failed for {}: {}",
            data.display(),
            res.spawn_error
                .clone()
                .unwrap_or_else(|| format!("exit {:?}", res.code))
        ));
    }
    Ok(ok)
}
