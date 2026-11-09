//! The Go goroutineleak pass of `dla harness run --capture nonheap`.
//!
//! The goroutineleak profile is a second signal next to the goroutine growth column: the
//! runtime's collector proves a goroutine can never wake. How it is reached depends on the
//! toolchain that builds the project (decided by [`dla_probe::go_toolchain`], the same
//! answer `dla probe` records):
//!
//! * Go 1.27 or newer: the agent writes the profile in the main run, nothing else to do.
//! * Go 1.26: the main run gets `GOEXPERIMENT=goroutineleakprofile`.
//! * An older Go with a Go 1.27 or newer toolchain in the module cache (`dla install
//!   go-toolchain-1.27`): after the main run, one more process runs the same loop with
//!   `GOTOOLCHAIN=<that toolchain>` and `DLA_GO_LEAK_ONLY=1`, so it writes only the leak
//!   profile. Command mode runs the last point only; driver mode runs the whole loop (the
//!   checkpoints are the same ones). Everything else that process measures is discarded:
//!   the project's own toolchain stays the source of the growth series.
//! * Otherwise the coverage row `go/goroutineleak` is `missing` with the install command,
//!   and the run goes on.
//!
//! The leak files are listed with the label of the main run, so `dla harness run` prints
//! one `dla summarize non-heap` command for them next to the goroutine dumps.

use dla_core::coverage::{BackendCoverage, CoverageManifest};
use dla_probe::go_toolchain::{self, EXPERIMENT, INSTALL_HINT};

use super::*;

/// Time allowed for `go version` and `go env`.
const QUERY_TIMEOUT: Duration = Duration::from_secs(20);

/// How this run reaches the goroutineleak profile.
pub(super) struct Plan {
    /// `GOEXPERIMENT` is set for the main run.
    experiment: bool,
    /// `GOTOOLCHAIN` of the separate leak pass.
    toolchain: Option<String>,
}

impl Plan {
    /// The manifest the main run uses: with the experiment in its environment on Go 1.26.
    pub(super) fn patched(&self, manifest: &HarnessManifest) -> Option<HarnessManifest> {
        if !self.experiment {
            return None;
        }
        let mut m = manifest.clone();
        m.run
            .env
            .push(("GOEXPERIMENT".to_owned(), EXPERIMENT.to_owned()));
        Some(m)
    }
}

/// Decides how the profile is reached, records the coverage row and returns the plan; `None`
/// when this run does not ask for goroutine captures of a Go harness.
pub(super) fn plan(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    paths: &Paths,
    opts: &RunOptions,
    limitations: &mut Vec<String>,
) -> Result<Option<Plan>, Refusal> {
    if manifest.runtime != dla_core::runtime::Runtime::Go || !opts.capture.nonheap {
        return Ok(None);
    }
    let go = go_toolchain::probe(&cwd_of(paths, &manifest.run), QUERY_TIMEOUT);
    let (row, plan) = match go.goroutineleak.as_str() {
        "native" => (
            BackendCoverage::ran(go.version.clone(), 0),
            Plan {
                experiment: false,
                toolchain: None,
            },
        ),
        "experiment" => (
            BackendCoverage::ran(go.version.clone(), 0),
            Plan {
                experiment: true,
                toolchain: None,
            },
        ),
        "toolchain" => {
            limitations.push(format!(
                "the goroutineleak profile comes from a separate pass with GOTOOLCHAIN={} (the project builds with {}); that pass measures nothing else",
                go.leak_toolchain.as_deref().unwrap_or("?"),
                go.version.as_deref().unwrap_or("an unknown Go")
            ));
            (
                BackendCoverage::ran(go.leak_toolchain.clone(), 0),
                Plan {
                    experiment: false,
                    toolchain: go.leak_toolchain.clone(),
                },
            )
        }
        _ => {
            limitations.push(format!(
                "goroutineleak: missing (the project builds with {}, and the profile needs Go 1.27 or newer); {}",
                go.version.as_deref().unwrap_or("an unknown Go"),
                go.install_hint.as_deref().unwrap_or(INSTALL_HINT)
            ));
            (
                BackendCoverage::missing(
                    format!(
                        "the goroutineleak profile needs Go 1.27 or newer ({} here)",
                        go.version.as_deref().unwrap_or("no go command found")
                    ),
                    go.install_hint
                        .clone()
                        .unwrap_or_else(|| INSTALL_HINT.to_owned()),
                ),
                Plan {
                    experiment: false,
                    toolchain: None,
                },
            )
        }
    };
    CoverageManifest::update(&ctx.run_dir, |m| {
        let ran = m
            .workers
            .get("go")
            .and_then(|w| w.backends.get("goroutineleak"))
            .is_some_and(|b| {
                b.status == dla_core::coverage::BackendStatus::Ran && b.enumerated > 0
            });
        if !ran {
            m.set_backend("go", "goroutineleak", row);
        }
    })
    .map_err(|e| e.to_refusal())?;
    Ok(Some(plan))
}

/// Runs the separate leak pass for one key mode, when the plan asks for it, and adds its
/// capture files to `run` under the main run's label.
#[allow(clippy::too_many_arguments)]
pub(super) fn leak_pass(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    opts: &RunOptions,
    paths: &Paths,
    mode: KeyMode,
    timeout: Duration,
    plan: Option<&Plan>,
    run: &mut ModeRun,
) -> Result<(), Refusal> {
    let Some(toolchain) = plan.and_then(|p| p.toolchain.as_deref()) else {
        return Ok(());
    };
    if run.points.is_empty() {
        return Ok(());
    }
    let command_mode = manifest.mode == HarnessMode::Command;
    let iterations = opts.iterations.unwrap_or(manifest.iterations);
    let rounds = iterations + opts.extra;
    let total = manifest.warmup + rounds;
    let dumps_wanted = opts.capture_dumps.unwrap_or(capture::DEFAULT_DUMPS);
    let (main_label, n) = if command_mode {
        (
            format!("p{total}-{}", mode.as_str()),
            manifest.step.saturating_mul(u64::from(total)),
        )
    } else {
        (
            format!("p1-{}", mode.as_str()),
            manifest.step.saturating_mul(u64::from(total)),
        )
    };
    let label = format!("{main_label}-leak");
    let only_leaks = CaptureSet {
        nonheap: true,
        ..CaptureSet::default()
    };
    let pe = PointEnv {
        manifest,
        paths,
        label: label.clone(),
        mode,
        n,
        purge: None,
        rounds: if command_mode { 0 } else { rounds },
        capture: Some((only_leaks, true)),
        dumps: dumps_wanted,
    };
    let dump = pe.dump_path();
    if let Some(d) = dump.parent() {
        fs::create_dir_all(d).map_err(|e| io_refusal(d, e))?;
    }
    let _ = fs::remove_file(&dump);
    let _ = fs::remove_dir_all(pe.capture_dir());
    let command = format!("GOTOOLCHAIN={toolchain} {}", manifest.run.argv.join(" "));
    let spec = pe.exec_spec(
        timeout,
        &command,
        &[
            ("GOTOOLCHAIN", toolchain.to_owned()),
            ("DLA_GO_LEAK_ONLY", "1".to_owned()),
        ],
    );
    let res = exec(ctx, &spec)?;
    run.processes += 1;
    let log_rel = paths.rel(ctx, &spec.log);
    run.logs.push(log_rel.clone());
    run.exits.push((label.clone(), res.code));
    let blocked = |why: String| -> Result<(), Refusal> {
        CoverageManifest::update(&ctx.run_dir, |m| {
            m.set_backend("go", "goroutineleak", BackendCoverage::blocked(why));
        })
        .map(|_| ())
        .map_err(|e| e.to_refusal())
    };
    if res.spawn_error.is_some() || res.timed_out || res.code != Some(0) {
        let why = format!(
            "the goroutineleak pass with GOTOOLCHAIN={toolchain} did not complete ({}; log {log_rel})",
            if res.timed_out {
                "timed out".to_owned()
            } else {
                exit_note(&label, &res).unwrap_or_else(|| "it could not start".to_owned())
            }
        );
        run.limitations.push(why.clone());
        return blocked(why);
    }
    let file = dumps::read_dumps(&dump);
    let pid = file.records.iter().find_map(|r| r.dump.pid);
    let mut notes = AgentNotes::default();
    for r in file.records.iter().filter(|r| r.dump.pid == pid) {
        if let Some(c) = &r.dump.captures {
            notes.records.push((r.dump.x, c.clone()));
        }
    }
    let label_dir = paths.out.join(&label);
    let rel = paths.rel(ctx, &label_dir);
    let (mut files, left_out) = capture::gather(
        &label_dir,
        &rel,
        &label,
        pid,
        &notes,
        command_mode.then_some(n as f64),
        false,
    );
    for f in &mut files {
        f.label = main_label.clone();
    }
    for (kind, error) in notes.errors() {
        run.limitations.push(format!(
            "{label}: {kind} capture failed: {}",
            clip(&error, 160)
        ));
    }
    if !left_out.is_empty() {
        run.limitations.push(format!(
            "{label}: {} leak file(s) were not listed",
            left_out.len()
        ));
    }
    if files.is_empty() {
        let why = format!(
            "the goroutineleak pass with GOTOOLCHAIN={toolchain} wrote no profile (log {log_rel})"
        );
        run.limitations.push(why.clone());
        return blocked(why);
    }
    run.captures.extend(files);
    Ok(())
}
