//! `dla sample --pid <P>`: sample a running process into a series (spec 5).

use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use clap::Args;
use dla_core::{
    EvidenceKind,
    coverage::{BackendCoverage, CoverageManifest},
    error::codes as core_codes,
    evidence::{Authority, CoverageStatus, EvidenceCoverage},
    runtime::Runtime,
    series::{
        Axis, Column, MAX_COLUMNS, Point, SeriesFile, SeriesSource, SeriesSourceKind, column_meta,
    },
    state::{ProcessRecord, RunState},
    toolctx::{EvidenceSpec, ToolCtx},
};
use dla_sample::{
    BackendReport, BackendSpec, BackendState, CountersSpec, OpenCtx, SessionConfig, SessionOutput,
    auto_specs, open_backend, parse_counters, proc, run_session,
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res, codes},
    stage,
};

/// Default time between samples.
const DEFAULT_INTERVAL: &str = "200ms";
/// Default sampling time.
const DEFAULT_DURATION: &str = "60s";
/// Shortest accepted interval.
const MIN_INTERVAL: Duration = Duration::from_millis(10);

/// Flags of `dla sample`.
#[derive(Args, Debug)]
pub struct SampleArgs {
    /// The process to sample. Tier 1 when DLA started it (it is in the run's process
    /// registry), tier 2 otherwise.
    #[arg(long)]
    pid: Option<u32>,
    /// Time between samples, for example 200ms or 1s.
    #[arg(long, default_value = DEFAULT_INTERVAL)]
    interval: String,
    /// Longest sampling time, for example 60s or 5m (capped by the run budget). Sampling
    /// also ends when the process exits.
    #[arg(long, default_value = DEFAULT_DURATION)]
    duration: String,
    /// Runtime counters: auto | none | a comma list of jcmd, go-pprof=<url>,
    /// node-inspect[=<port>], python-remote, tokio.
    #[arg(long, default_value = "auto")]
    counters: String,
    /// Sum the figures of the process and all its descendants.
    #[arg(long)]
    tree: bool,
    /// Number of leading points to treat as warm-up in `dla trend`.
    #[arg(long, default_value_t = 0)]
    warmup: u32,
}

/// Parses `<digits><ms|s|m|h>`.
fn parse_time(flag: &str, text: &str) -> Result<Duration, Failure> {
    let bad = |why: &str| {
        Failure::refuse(
            core_codes::INVALID_VALUE,
            format!("invalid {flag} {text:?}: {why}"),
            format!("{flag} such as 200ms, 5s or 2m"),
            format!("dla sample --pid <pid> {flag} 1s"),
        )
    };
    let t = text.trim();
    let (digits, unit_ms) = if let Some(d) = t.strip_suffix("ms") {
        (d, 1)
    } else if let Some(d) = t.strip_suffix('s') {
        (d, 1000)
    } else if let Some(d) = t.strip_suffix('m') {
        (d, 60_000)
    } else if let Some(d) = t.strip_suffix('h') {
        (d, 3_600_000)
    } else {
        return Err(bad("the unit must be ms, s, m or h"));
    };
    let n: u64 = digits
        .parse()
        .map_err(|_| bad("expected digits before the unit"))?;
    n.checked_mul(unit_ms)
        .filter(|ms| *ms > 0)
        .map(Duration::from_millis)
        .ok_or_else(|| bad("must be greater than zero and not overflow"))
}

/// The registered process matching `pid` (and its start time), if DLA started it and it has
/// not been marked stopped.
fn registered<'a>(state: &'a RunState, pid: u32, start: Option<&str>) -> Option<&'a ProcessRecord> {
    state.processes.iter().rev().find(|p| {
        p.pid == pid
            && p.stopped_at.is_none()
            // The pid alone proves nothing (pids are reused): both start times must be known
            // and agree.
            && match (p.start_time.as_deref(), start) {
                (Some(a), Some(b)) => same_start(a, b),
                _ => false,
            }
    })
}

/// Whether two process start times (RFC 3339) are the same instant within 2 s. The Linux
/// figure is derived from the boot time, which a clock step can move by a second.
fn same_start(a: &str, b: &str) -> bool {
    match (
        chrono::DateTime::parse_from_rfc3339(a),
        chrono::DateTime::parse_from_rfc3339(b),
    ) {
        (Ok(x), Ok(y)) => (x - y).num_milliseconds().abs() <= 2000,
        _ => a == b,
    }
}

/// How the sampled process is named in the series and the effects statement: without raw-value
/// consent only the executable's base name and the argument count (a command line can hold
/// secrets), with it the full line.
fn describe_command(args: &[String], raw_values: bool) -> String {
    let Some(first) = args.first() else {
        return String::new();
    };
    if raw_values {
        return args.join(" ");
    }
    let n = args.len() - 1;
    format!(
        "{} ({n} argument{})",
        proc::exe_name(first),
        if n == 1 { "" } else { "s" }
    )
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_owned();
    }
    let mut t: String = s.chars().take(max.saturating_sub(3)).collect();
    t.push_str("...");
    t
}

/// The effects statement (spec 9): what is observed, what changes, how it ends.
fn effects(pid: u32, command: &str, tier: u8, specs: &[BackendSpec], a: &SampleArgs) -> Value {
    let mut signals = Vec::new();
    for s in specs {
        signals.push(match s {
            BackendSpec::Jcmd => "JVM heap, committed, NMT and threads",
            BackendSpec::GoPprof(_) => "Go MemStats and goroutines by creation site",
            BackendSpec::NodeInspect(_) => "Node heap, external and active resources",
            BackendSpec::PythonRemote => "Python traced memory and gc objects",
            BackendSpec::Tokio => "tokio metrics (not readable from outside)",
        });
    }
    let mut effects = vec!["read-only observation of /proc or the platform's process API"];
    let mut cleanup = vec!["none: the process is left running"];
    for s in specs {
        match s {
            BackendSpec::Jcmd => {
                effects.push("jcmd attaches to the JVM and runs diagnostic commands")
            }
            BackendSpec::GoPprof(_) => effects.push("HTTP GETs to the target's pprof endpoint"),
            BackendSpec::NodeInspect(None) => {
                effects.push("the target runs a small read-only script through its inspector; if it has no inspector, SIGUSR1 starts one on port 9229");
                cleanup.push("an inspector started by SIGUSR1 stays open until the process exits");
            }
            BackendSpec::NodeInspect(Some(_)) => {
                effects.push("the target runs a small read-only script through its inspector");
            }
            BackendSpec::PythonRemote => {
                effects.push("the target runs a small generated script (sys.remote_exec) that appends JSON to a file in the run directory");
            }
            BackendSpec::Tokio => {}
        }
    }
    json!({
        "target": format!("pid {pid} ({}), tier {tier}", clip(command, 80)),
        "expected_signal": if signals.is_empty() {
            "OS memory, threads and descriptors over time".to_owned()
        } else {
            format!("OS memory, threads and descriptors, plus {}", signals.join("; "))
        },
        "stopping": format!("when the process exits or after {} (and never past the run budget)", a.duration),
        "effects": effects.join("; "),
        "cleanup": cleanup.join("; "),
    })
}

fn source_label(name: &str) -> &'static str {
    match name {
        "os.pss" | "os.pss_anon" | "os.private_dirty" => "smaps-rollup",
        "os.fds" | "os.handles" => "proc-fd",
        n if n.starts_with("os.") => {
            if cfg!(target_os = "linux") {
                "proc-status"
            } else if cfg!(target_os = "macos") {
                "libproc"
            } else {
                "psapi"
            }
        }
        n if n.starts_with("cgroup.") => "cgroup",
        n if n.starts_with("jvm.") => "jcmd",
        n if n.starts_with("go.") => "go-pprof",
        n if n.starts_with("node.") => "node-inspect",
        n if n.starts_with("py.") => "python-remote",
        _ => "sample",
    }
}

fn is_grouped(name: &str) -> bool {
    name.contains('@') || name.starts_with("node.active.") || name.starts_with("jvm.class:")
}

/// Column names in series order: `os.rss` first, then `os.*`, `cgroup.*`, the rest.
fn ordered_columns(out: &SessionOutput) -> Vec<String> {
    let names: BTreeSet<&String> = out.points.iter().flat_map(|p| p.values.keys()).collect();
    let rank = |n: &str| -> (u8, bool) {
        (
            if n.starts_with("os.") {
                0
            } else if n.starts_with("cgroup.") {
                1
            } else {
                2
            },
            n != "os.rss",
        )
    };
    let mut v: Vec<String> = names.into_iter().cloned().collect();
    v.sort_by(|a, b| rank(a).cmp(&rank(b)).then_with(|| a.cmp(b)));
    v
}

/// Drops grouped columns, lowest growth first, until at most [`MAX_COLUMNS`] remain.
/// Returns the notes to record in the series limitations.
fn cap_columns(names: &mut Vec<String>, out: &SessionOutput) -> Vec<String> {
    let mut notes = Vec::new();
    if names.len() <= MAX_COLUMNS {
        return notes;
    }
    let growth = |name: &str| -> f64 {
        let vals: Vec<f64> = out
            .points
            .iter()
            .filter_map(|p| p.values.get(name).copied())
            .collect();
        match (vals.first(), vals.last()) {
            (Some(a), Some(b)) => b - a,
            _ => f64::NEG_INFINITY,
        }
    };
    let mut grouped: Vec<String> = names.iter().filter(|n| is_grouped(n)).cloned().collect();
    grouped.sort_by(|a, b| growth(a).total_cmp(&growth(b)).then_with(|| b.cmp(a)));
    let excess = names.len() - MAX_COLUMNS;
    let drop: BTreeSet<String> = grouped.into_iter().take(excess).collect();
    if !drop.is_empty() {
        notes.push(format!(
            "dropped {} grouped columns (lowest growth first)",
            drop.len()
        ));
        names.retain(|n| !drop.contains(n));
    }
    if names.len() > MAX_COLUMNS {
        let extra = names.len() - MAX_COLUMNS;
        notes.push(format!(
            "dropped {extra} further columns beyond the {MAX_COLUMNS}-column cap"
        ));
        names.truncate(MAX_COLUMNS);
    }
    notes
}

/// Builds the series file from a finished session.
fn build_series(
    id: &str,
    out: &SessionOutput,
    source: SeriesSource,
    warmup: u32,
    extra_limitations: Vec<String>,
) -> SeriesFile {
    let mut names = ordered_columns(out);
    let mut limitations = extra_limitations;
    limitations.extend(out.limitations.iter().cloned());
    limitations.extend(cap_columns(&mut names, out));
    let columns: Vec<Column> = names
        .iter()
        .map(|n| {
            let m = column_meta(n);
            Column {
                name: n.clone(),
                unit: m.unit,
                kind: m.kind,
                region: m.region,
                baseline: m.baseline,
                // Nothing here forces a collection, so no figure is an after-GC figure.
                after_gc: false,
                scope: m.scope,
                site: n
                    .split_once('@')
                    .map(|(_, s)| s)
                    .filter(|s| s.contains(':'))
                    .map(str::to_owned),
                finding: None,
                source: source_label(n).to_owned(),
            }
        })
        .collect();
    let points = out
        .points
        .iter()
        .map(|p| Point {
            x: p.t_ms as f64 / 1000.0,
            t_ms: p.t_ms,
            values: names.iter().map(|n| p.values.get(n).copied()).collect(),
        })
        .collect::<Vec<_>>();
    let warmup = warmup.min(points.len() as u32);
    SeriesFile {
        schema_version: 1,
        id: id.to_owned(),
        source,
        axis: Axis::Seconds,
        warmup,
        columns,
        points,
        events: Vec::new(),
        limitations,
    }
}

fn state_name(r: &BackendReport) -> &'static str {
    match r.state {
        BackendState::Ran => "ran",
        BackendState::Blocked => "blocked",
    }
}

/// Runs `dla sample`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: SampleArgs) -> Res<Value> {
    let Some(pid) = a.pid else {
        return Err(Failure::refuse(
            codes::MISSING_ARGUMENT,
            "dla sample needs a process to sample",
            "--pid <pid>",
            "dla sample --pid <pid>   (to run a loop harness instead: dla harness run)",
        ));
    };
    let interval = parse_time("--interval", &a.interval)?;
    let duration = parse_time("--duration", &a.duration)?;
    if interval < MIN_INTERVAL {
        return Err(Failure::refuse(
            core_codes::INVALID_VALUE,
            format!("--interval {} is below the 10ms minimum", a.interval),
            "--interval of at least 10ms",
            "dla sample --pid <pid> --interval 200ms",
        ));
    }
    let counters = parse_counters(&a.counters).map_err(|e| {
        Failure::refuse(
            core_codes::INVALID_VALUE,
            e,
            "--counters auto, none or a list such as jcmd,go-pprof=http://127.0.0.1:6060",
            "dla sample --pid <pid> --counters auto",
        )
    })?;

    let started = Instant::now();
    let tc = stage::tool_ctx(project, ctx, data);
    tc.require_time("sampling")?;
    let state = RunState::load(&tc.run_dir)?;
    let start_time = dla_sample::os::process_start_time(pid);
    if !dla_sample::os::is_alive(pid) {
        return Err(Failure::refuse(
            core_codes::INVALID_VALUE,
            format!("process {pid} is not running"),
            "a running process to sample",
            "start the process, then dla sample --pid <pid>",
        ));
    }
    let tier = if registered(&state, pid, start_time.as_deref()).is_some() {
        1
    } else {
        2
    };
    tc.require_tier(tier, &format!("sampling process {pid}"))?;

    // How long may it run: never past the run budget.
    let mut notes: Vec<String> = Vec::new();
    let deadline = ctx
        .worker
        .as_deref()
        .and_then(|w| state.worker(w))
        .map_or(state.budget.deadline, |w| {
            w.deadline.min(state.budget.deadline)
        });
    let remaining = (deadline - chrono::Utc::now())
        .to_std()
        .unwrap_or(Duration::ZERO);
    if remaining < interval * 2 {
        return Err(Failure::refuse(
            core_codes::RUN_NOT_ACTIVE,
            "the run budget is used up",
            "time left in the run budget",
            "dla run init --budget 60m",
        ));
    }
    let duration = if duration > remaining {
        notes.push(format!(
            "duration cut to {}s by the run budget",
            remaining.as_secs()
        ));
        remaining
    } else {
        duration
    };

    let args = proc::command_line(pid).unwrap_or_default();
    let raw_values = state.consent.as_ref().is_some_and(|c| c.raw_values);
    let command = describe_command(&args, raw_values);
    let specs = match counters {
        CountersSpec::None => Vec::new(),
        CountersSpec::Auto => auto_specs(&args),
        CountersSpec::List(l) => l,
    };
    let effects = effects(pid, &command, tier, &specs, &a);
    eprintln!("dla sample: {effects}");

    // Open the backends. A failure becomes a limitation and a coverage entry.
    let mut roots = vec![dla_core::paths::to_slash(project)];
    if let Ok(cwd) = std::fs::read_link(format!("/proc/{pid}/cwd")) {
        roots.push(dla_core::paths::to_slash(&cwd));
    }
    let open_ctx = OpenCtx {
        pid,
        interval,
        args: args.clone(),
        roots,
        scratch: tc.tmp_dir()?,
        deadline: started + duration,
    };
    let mut backends = Vec::new();
    let mut failed: Vec<(BackendSpec, dla_sample::BackendError)> = Vec::new();
    for spec in &specs {
        match open_backend(spec, &open_ctx) {
            Ok(b) => backends.push(b),
            Err(e) => {
                notes.push(e.message.clone());
                failed.push((spec.clone(), e));
            }
        }
    }

    // Opening the backends counts against --duration.
    let duration = duration.saturating_sub(started.elapsed()).max(interval);
    let out = run_session(
        SessionConfig {
            pid,
            interval,
            duration,
            tree: a.tree,
        },
        backends,
    )
    .map_err(|e| {
        Failure::refuse(
            core_codes::INVALID_VALUE,
            format!("cannot sample process {pid}: {e}"),
            "a running process to sample",
            "dla sample --pid <pid>",
        )
    })?;

    let mut extra = notes;
    if let Some(e) = &out.error {
        extra.push(format!("sampling ended early: {e}"));
    }
    if a.tree
        && let Some(l) = out.largest_descendant
    {
        extra.push(format!(
            "tree mode: the largest descendant was process {} with {} bytes resident",
            l.pid, l.rss
        ));
    }
    let mut source = SeriesSource::new(SeriesSourceKind::Sample);
    source.pid = Some(pid);
    if !command.is_empty() {
        source.command = Some(clip(&command, 500));
    }
    let ran: Vec<&str> = out
        .backends
        .iter()
        .filter(|b| b.state == BackendState::Ran)
        .map(|b| b.runtime.as_str())
        .collect();
    if let Some(first) = ran.first()
        && ran.iter().all(|r| r == first)
    {
        source.runtime = first.parse::<Runtime>().ok();
    }
    let id = tc.next_series_id();
    let series = build_series(&id, &out, source, a.warmup, extra);
    series.save(&tc.run_dir)?;

    // Coverage: one entry per requested backend, under its runtime worker.
    CoverageManifest::update(&tc.run_dir, |m| {
        for r in &out.backends {
            let cov = match r.state {
                BackendState::Ran => BackendCoverage::ran(None, r.readings),
                BackendState::Blocked => BackendCoverage::blocked(
                    r.reason.clone().unwrap_or_else(|| "no reading".to_owned()),
                ),
            };
            m.set_backend(&r.runtime, &r.name, cov);
        }
        for (spec, e) in &failed {
            let cov = if e.missing {
                BackendCoverage::missing(e.message.clone(), e.hint.clone().unwrap_or_default())
            } else {
                BackendCoverage::blocked(e.message.clone())
            };
            m.set_backend(spec.runtime(), spec.name(), cov);
        }
    })?;

    let mut statuses = serde_json::Map::new();
    for r in &out.backends {
        statuses.insert(r.name.clone(), json!(state_name(r)));
    }
    for (spec, e) in &failed {
        statuses.insert(
            spec.name().to_owned(),
            json!(if e.missing { "missing" } else { "blocked" }),
        );
    }
    // Runtime columns first: they are what the operating-system baseline does not say.
    let mut names: Vec<&str> = series.columns.iter().map(|c| c.name.as_str()).collect();
    names.sort_by_key(|n| n.starts_with("os.") || n.starts_with("cgroup."));
    let evidence = add_evidence(
        &tc,
        &series,
        pid,
        start_time.as_deref(),
        &out,
        &names,
        &statuses,
        !failed.is_empty()
            || out
                .backends
                .iter()
                .any(|b| b.state == BackendState::Blocked),
    )?;

    Ok(json!({
        "ok": true,
        "command": "sample",
        "series": id,
        "evidence": evidence,
        "ticks": series.points.len(),
        "exited": out.exited,
        "columns": names.iter().take(16).collect::<Vec<_>>(),
        "columns_total": names.len(),
        "backends": statuses,
        "limitations": series.limitations.iter().take(5).map(|l| clip(l, 160)).collect::<Vec<_>>(),
        "limitations_total": series.limitations.len(),
        "effects": effects,
        "next": format!("dla trend {id}"),
    }))
}

#[allow(clippy::too_many_arguments)]
fn add_evidence(
    tc: &ToolCtx,
    series: &SeriesFile,
    pid: u32,
    start_time: Option<&str>,
    out: &SessionOutput,
    names: &[&str],
    statuses: &serde_json::Map<String, Value>,
    degraded: bool,
) -> Res<String> {
    let mut result = json!({
        "series": series.id,
        "pid": pid,
        "duration_s": (out.elapsed_ms as f64 / 100.0).round() / 10.0,
        "ticks": series.points.len(),
        "columns": names.iter().take(24).map(|n| clip(n, 60)).collect::<Vec<_>>(),
        "columns_total": names.len(),
        "backends": statuses,
        "limitations": series.limitations.len(),
    });
    if let Some(l) = out.largest_descendant {
        result["largest_descendant"] = json!({"pid": l.pid, "rss": l.rss});
    }
    // Keep the result under the evidence bound whatever the column names look like.
    while result.to_string().len() > 1900 {
        let cols = result["columns"].as_array_mut().filter(|c| !c.is_empty());
        match cols {
            Some(c) => c.truncate(c.len() / 2),
            None => break,
        }
    }
    let mut spec = EvidenceSpec::new(
        EvidenceKind::GrowthSeries,
        "dla-sample",
        env!("CARGO_PKG_VERSION"),
        result,
    );
    spec.authority = Authority::Live;
    spec.raw = Some(format!("series/{}.json", series.id));
    spec.source = Some(ToolCtx::process_source(
        pid,
        start_time.unwrap_or("unknown"),
    ));
    spec.coverage = EvidenceCoverage {
        status: if degraded {
            CoverageStatus::Partial
        } else {
            CoverageStatus::Complete
        },
        examined: Some(series.points.len() as u64),
        total: None,
    };
    spec.limitations = series
        .limitations
        .iter()
        .take(12)
        .map(|l| clip(l, 300))
        .collect();
    let mut ledger = tc.ledger()?;
    let id = tc.add_evidence(&mut ledger, spec)?;
    Ok(id.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_sample::TickPoint;

    #[test]
    fn times_need_a_unit() {
        assert_eq!(
            parse_time("--interval", "200ms").unwrap(),
            Duration::from_millis(200)
        );
        assert_eq!(
            parse_time("--duration", "5s").unwrap(),
            Duration::from_secs(5)
        );
        assert_eq!(
            parse_time("--duration", "2m").unwrap(),
            Duration::from_secs(120)
        );
        assert_eq!(
            parse_time("--duration", "1h").unwrap(),
            Duration::from_secs(3600)
        );
        for bad in ["200", "ms", "0s", "-1s", "1.5s", ""] {
            assert!(parse_time("--interval", bad).is_err(), "{bad}");
        }
    }

    fn output(columns: &[(&str, f64)], points: usize) -> SessionOutput {
        SessionOutput {
            points: (0..points)
                .map(|i| TickPoint {
                    t_ms: i as u64 * 100,
                    values: columns
                        .iter()
                        .map(|(n, step)| ((*n).to_owned(), 10.0 + step * i as f64))
                        .collect(),
                })
                .collect(),
            ..SessionOutput::default()
        }
    }

    #[test]
    fn series_uses_the_catalog_and_never_claims_after_gc() {
        let out = output(
            &[
                ("os.rss", 5.0),
                ("os.fds", 0.0),
                ("jvm.heap_used", 1.0),
                ("go.goroutines@pkg/a.go:33", 1.0),
                ("cgroup.current", 0.0),
            ],
            4,
        );
        let mut src = SeriesSource::new(SeriesSourceKind::Sample);
        src.pid = Some(7);
        let s = build_series("S1", &out, src, 9, vec!["note".into()]);
        s.validate().unwrap();
        let names: Vec<&str> = s.columns.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            names,
            [
                "os.rss",
                "os.fds",
                "cgroup.current",
                "go.goroutines@pkg/a.go:33",
                "jvm.heap_used"
            ]
        );
        assert!(s.columns.iter().all(|c| !c.after_gc));
        let g = &s.columns[3];
        assert_eq!(g.site.as_deref(), Some("pkg/a.go:33"));
        assert_eq!(s.columns[2].scope, dla_core::series::ColumnScope::Container);
        assert_eq!(s.warmup, 4, "warm-up is clamped to the points");
        assert_eq!(s.axis, Axis::Seconds);
        assert_eq!(s.points[3].x, 0.3);
        assert_eq!(s.limitations, ["note"]);
    }

    #[test]
    fn extra_grouped_columns_are_dropped_by_growth() {
        let mut cols: Vec<(String, f64)> = vec![("os.rss".into(), 1.0)];
        for i in 0..70 {
            cols.push((format!("go.goroutines@f.go:{i}"), f64::from(i)));
        }
        let refs: Vec<(&str, f64)> = cols.iter().map(|(n, g)| (n.as_str(), *g)).collect();
        let out = output(&refs, 3);
        let s = build_series(
            "S2",
            &out,
            SeriesSource::new(SeriesSourceKind::Sample),
            0,
            vec![],
        );
        s.validate().unwrap();
        assert_eq!(s.columns.len(), MAX_COLUMNS);
        assert!(s.columns.iter().any(|c| c.name == "os.rss"));
        // The fastest growers survive; the flat ones go.
        assert!(s.columns.iter().any(|c| c.name == "go.goroutines@f.go:69"));
        assert!(!s.columns.iter().any(|c| c.name == "go.goroutines@f.go:0"));
        assert!(
            s.limitations
                .iter()
                .any(|l| l.starts_with("dropped 7 grouped columns"))
        );
    }

    #[test]
    fn registry_match_needs_known_start_times_within_two_seconds() {
        assert!(same_start("2026-01-01T00:00:10Z", "2026-01-01T00:00:11Z"));
        assert!(same_start("2026-01-01T00:00:10Z", "2026-01-01T00:00:08Z"));
        assert!(!same_start("2026-01-01T00:00:10Z", "2026-01-01T00:00:13Z"));
        assert!(same_start("odd", "odd"));
        assert!(!same_start("odd", "2026-01-01T00:00:10Z"));
    }

    #[test]
    fn registry_pid_alone_is_tier_two() {
        use dla_core::state::{ProcessKind, ProcessRecord};
        let rec = |start: Option<&str>| ProcessRecord {
            pid: 77,
            start_time: start.map(str::to_owned),
            kind: ProcessKind::Profiler,
            command: "x".into(),
            started_at: chrono::Utc::now(),
            stopped_at: None,
            role: dla_core::state::ProcessRole::Started,
            worker: None,
            cleanup: Vec::new(),
        };
        let mut state = RunState::init("/p", None, 60, None, chrono::Utc::now()).unwrap();
        state.processes.push(rec(Some("2026-01-01T00:00:10Z")));
        assert!(registered(&state, 77, Some("2026-01-01T00:00:11Z")).is_some());
        assert!(registered(&state, 77, None).is_none(), "live start unknown");
        assert!(registered(&state, 77, Some("2026-01-01T00:05:00Z")).is_none());
        state.processes[0] = rec(None);
        assert!(registered(&state, 77, Some("2026-01-01T00:00:11Z")).is_none());
        assert!(registered(&state, 77, None).is_none());
    }

    #[test]
    fn command_lines_are_redacted_without_raw_value_consent() {
        let args: Vec<String> = ["/usr/bin/java", "--password=hunter2", "-Dsecret=x"]
            .iter()
            .map(|s| (*s).to_owned())
            .collect();
        let d = describe_command(&args, false);
        assert_eq!(d, "java (2 arguments)");
        assert!(!d.contains("hunter2"));
        assert_eq!(
            describe_command(&args, true),
            "/usr/bin/java --password=hunter2 -Dsecret=x"
        );
        assert_eq!(describe_command(&args[..1], false), "java (0 arguments)");
        assert_eq!(describe_command(&[], false), "");
    }

    #[test]
    fn effects_name_the_signal_script_and_cleanup() {
        let a = SampleArgs {
            pid: Some(9),
            interval: "200ms".into(),
            duration: "30s".into(),
            counters: "node-inspect".into(),
            tree: false,
            warmup: 0,
        };
        let e = effects(9, "node app.js", 2, &[BackendSpec::NodeInspect(None)], &a);
        for k in [
            "target",
            "expected_signal",
            "stopping",
            "effects",
            "cleanup",
        ] {
            assert!(e[k].as_str().is_some_and(|s| !s.is_empty()), "{k}");
        }
        assert!(e["effects"].as_str().unwrap().contains("SIGUSR1"));
        assert!(e["target"].as_str().unwrap().contains("tier 2"));
        assert!(e["stopping"].as_str().unwrap().contains("30s"));
    }
}
