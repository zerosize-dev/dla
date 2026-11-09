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

use super::{ExecSpec, Paths, PointEnv, RunOptions, RunOutput, exec, failed, subst};
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
        // A relative program is resolved against the working directory by the tool.
        let mut argv = vec![tool_path.to_string_lossy().into_owned()];
        let report: PathBuf;
        match tool {
            NativeTool::Valgrind => {
                report = dir.join("valgrind.xml");
                let _ = fs::remove_file(&report);
                argv.extend([
                    "--tool=memcheck".to_owned(),
                    "--leak-check=full".to_owned(),
                    "--show-reachable=yes".to_owned(),
                    "--num-callers=32".to_owned(),
                    "--xml=yes".to_owned(),
                    format!("--xml-file={}", report.display()),
                ]);
                if is_shell(&program) {
                    argv.push("--trace-children=yes".to_owned());
                    out.limitations.push(format!(
                        "{label}: the workload starts through {program}; valgrind follows its children, so the report may include other programs"
                    ));
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
                let slot = Slot {
                    label: &label,
                    seq: 1,
                    x: n as f64,
                    ms: Some(res.wall_ms),
                };
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
