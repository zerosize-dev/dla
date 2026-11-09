//! `dla attach <backend> --pid <P> --op <op>`: attach a tool to a process DLA did not start.
//!
//! The order is fixed: the run's time budget, the target's identity, the consent tier, a
//! read-only plan, the preflight (refused with `insufficient-resources` when the dump or the
//! peak memory does not fit), and only then the registry record and the attach itself. The
//! artifacts land in `attach/P<n>/` and one `artifact` evidence describes them.
//!
//! Two environment variables replace the measured figures of the preflight, for tests and for
//! a box whose figures are wrong: `DLA_ATTACH_FREE_DISK` and `DLA_ATTACH_AVAILABLE_MEMORY`
//! (bytes).

use std::path::{Path, PathBuf};

use clap::Args;
use dla_core::{
    CoverageManifest, RunState, coverage::BackendCoverage, error::codes as core_codes,
    toolctx::ToolCtx,
};
use dla_sample::attach::{
    self, AttachError, Backend, Cleanup, DiskFigures, ErrorKind, MemoryAvailable, Request,
    free_disk, inspect_target, preflight, read_memory_available, resolve, tier_for,
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    stage,
};

#[path = "attach/mod.rs"]
mod parts;

/// Default recording time.
const DEFAULT_DURATION: &str = "30s";

/// Flags of `dla attach`.
#[derive(Args, Debug)]
pub struct AttachArgs {
    /// Backend: jcmd, node-inspector, memray, heaptrack, goref.
    pub backend: String,
    /// Target process ID.
    #[arg(long)]
    pub pid: u32,
    /// Operation: jcmd histogram|heap-dump|thread-dump|nmt|jfr, node-inspector
    /// memory|heap-snapshot, memray|heaptrack record, goref heap-profile.
    #[arg(long)]
    pub op: String,
    /// How long recording operations run (jfr, memray, heaptrack).
    #[arg(long, default_value = DEFAULT_DURATION)]
    pub duration: String,
    /// Name of the main artifact inside `attach/P<n>/` (default depends on the operation).
    #[arg(long)]
    pub out: Option<String>,
}

fn env_bytes(name: &str) -> Option<u64> {
    std::env::var(name).ok()?.trim().parse().ok()
}

/// The command text stored in the registry: unique per attach, no raw command line.
fn record_command(id: &str, backend: Backend, op: attach::Op) -> String {
    format!("attach {id}: {} {}", backend.name(), op.name())
}

/// The coverage worker key (the runtime) of an attach backend.
fn coverage_worker(b: Backend) -> &'static str {
    match b {
        Backend::Jcmd => "jvm",
        Backend::NodeInspector => "node",
        Backend::Memray => "python",
        Backend::Heaptrack => "rust",
        Backend::Goref => "go",
    }
}

/// Writes the backend's coverage row: `ran`, `missing` (with the fix as the install
/// hint) or `blocked`. A failure to write it never hides the attach result.
fn note_coverage(tc: &ToolCtx, backend: Backend, cov: BackendCoverage) {
    let worker = coverage_worker(backend);
    let r = CoverageManifest::update(&tc.run_dir, |m| {
        m.note_backend(worker, backend.name(), cov);
    });
    if let Err(e) = r {
        eprintln!("dla attach: coverage.json not updated: {e}");
    }
}

/// The coverage row for an attach error.
fn error_coverage(e: &AttachError) -> BackendCoverage {
    match e.kind {
        ErrorKind::ToolMissing => BackendCoverage::missing(e.message.clone(), e.fix.clone()),
        ErrorKind::NotReady | ErrorKind::Failed => BackendCoverage::blocked(e.message.clone()),
    }
}

/// Runs `dla attach`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: AttachArgs) -> Res<Value> {
    let (backend, op) = resolve(&a.backend, &a.op).map_err(|m| {
        let unknown_backend = Backend::parse(&a.backend).is_none();
        Failure::refuse(
            if unknown_backend {
                core_codes::UNKNOWN_BACKEND
            } else {
                core_codes::INVALID_VALUE
            },
            m,
            "--backend and --op from the list above",
            "dla attach jcmd --pid <pid> --op histogram",
        )
    })?;
    let duration = parts::parse_duration(&a.duration).ok_or_else(|| {
        Failure::refuse(
            core_codes::INVALID_VALUE,
            format!("invalid --duration {:?}", a.duration),
            "--duration such as 500ms, 30s or 2m",
            "dla attach ... --duration 30s",
        )
    })?;

    let tc = stage::tool_ctx(project, ctx, data);
    tc.require_time(&format!("dla attach {} {}", backend.name(), op.name()))?;
    let state = RunState::load(&tc.run_dir)?;
    let Some(target) = inspect_target(a.pid) else {
        return Err(Failure::refuse(
            core_codes::INVALID_VALUE,
            format!("process {} is not running", a.pid),
            "a running process to attach to",
            "start the process, then dla attach <backend> --pid <pid> --op <op>",
        ));
    };

    // Consent first, before anything reads the target beyond its identity.
    let need = tier_for(backend, op);
    let by_registry = parts::registry::started_by_dla(&state, a.pid, target.start_time.as_deref());
    let tier = if by_registry { 1 } else { need.tier };
    tc.require_tier(
        tier,
        &format!(
            "attaching {} {} to process {} ({})",
            backend.name(),
            op.name(),
            a.pid,
            need.why
        ),
    )?;

    // Plan: reads only (the Node inspector may be opened with SIGUSR1; the plan says so).
    let dump_root = tc.run_dir.join("attach");
    std::fs::create_dir_all(&dump_root)
        .map_err(|e| Failure::from(dla_core::Error::io(&dump_root, e)))?;
    let req = Request {
        target: target.clone(),
        backend,
        op,
        duration,
        out_name: a.out.clone(),
        dump_root: dump_root.clone(),
        tool_dirs: tc.tools_dir().into_iter().collect(),
    };
    let plan = match attach::plan(&req) {
        Ok(p) => p,
        Err(e) => {
            note_coverage(&tc, backend, error_coverage(&e));
            return Err(parts::refusal(e));
        }
    };

    // Preflight and its gate.
    let disk = DiskFigures {
        run_dir_free: env_bytes("DLA_ATTACH_FREE_DISK").or_else(|| free_disk(&tc.run_dir)),
        dump_dir_free: env_bytes("DLA_ATTACH_FREE_DISK").or_else(|| free_disk(&dump_root)),
    };
    let memory = match env_bytes("DLA_ATTACH_AVAILABLE_MEMORY") {
        Some(m) => MemoryAvailable {
            host: Some(m),
            cgroup: None,
        },
        None => read_memory_available(Some(a.pid)),
    };
    let pre = preflight(&target, &plan, disk, memory);
    eprintln!(
        "dla attach: {} {} on {}: {}; effects: {}",
        backend.name(),
        op.name(),
        a.pid,
        pre.expected_signal,
        pre.effects.join("; ")
    );
    if !pre.reasons.is_empty() {
        let fix = match (backend, op) {
            (Backend::Jcmd, attach::Op::HeapDump) => {
                "free disk space or memory, or take the lighter view first: dla attach jcmd --pid <pid> --op histogram"
            }
            (Backend::NodeInspector, attach::Op::HeapSnapshot) => {
                "free disk space or memory, or take the lighter view first: dla attach node-inspector --pid <pid> --op memory"
            }
            _ => "free disk space or memory, then repeat the command",
        };
        return Err(Failure::refuse(
            parts::INSUFFICIENT_RESOURCES,
            format!(
                "attach {} {} refused before touching process {}: {}",
                backend.name(),
                op.name(),
                a.pid,
                pre.reasons.join("; ")
            ),
            "enough free disk for the dump (x1.2) and 20% memory headroom for the peak",
            fix,
        ));
    }

    // Go: the attach directory, the preflight file, the registry record.
    let id = tc.next_attach_id();
    let rel_dir = format!("attach/{id}");
    let out_dir = tc.run_dir.join(&rel_dir);
    std::fs::create_dir_all(&out_dir)
        .map_err(|e| Failure::from(dla_core::Error::io(&out_dir, e)))?;
    let pre_rel = format!("{rel_dir}/preflight.json");
    std::fs::write(
        tc.run_dir.join(&pre_rel),
        serde_json::to_string_pretty(&pre).unwrap_or_default(),
    )
    .map_err(|e| Failure::from(dla_core::Error::io(&out_dir, e)))?;
    let command = record_command(&id, backend, op);
    tc.register_process(parts::registry::new_record(
        a.pid,
        target.start_time.as_deref(),
        &command,
        ctx.worker.as_deref(),
    ))?;

    let run_dir = tc.run_dir.clone();
    let pid = a.pid;
    let cmd2 = command.clone();
    let mut register = move |c: &Cleanup| parts::registry::add_cleanup(&run_dir, pid, &cmd2, c);
    let fin = attach::run(plan, &req, &out_dir, &mut register);
    let left: Vec<String> = fin.cleanup_left.iter().map(Cleanup::render).collect();
    parts::registry::finish(&tc.run_dir, pid, &command, &fin.cleanup_left)?;

    let done = match &fin.result {
        Ok(d) => d,
        Err(e) => {
            note_coverage(&tc, backend, error_coverage(e));
            let mut f = parts::refusal(e.clone());
            let pending: Vec<String> = fin
                .cleanup_left
                .iter()
                .filter(|c| c.executable())
                .map(Cleanup::render)
                .collect();
            if !pending.is_empty() {
                f.refusal.message = format!(
                    "{} (the target may still be traced or recorded; left to do, `dla run abort` runs it: {})",
                    f.refusal.message,
                    pending.join("; ")
                );
            }
            if !fin.cleanup_done.is_empty() {
                f.refusal.message = format!(
                    "{} (cleaned up: {})",
                    f.refusal.message,
                    fin.cleanup_done.join(", ")
                );
            }
            return Err(f);
        }
    };

    let files = parts::evidence::file_entries(&tc.run_dir, &rel_dir, done);
    let ev_parts = parts::evidence::Parts {
        backend,
        op,
        files: &files,
        preflight: &pre,
        fin: &fin,
        done,
        left: &left,
    };
    let evidence = parts::evidence::add(&tc, &pre.tool, &ev_parts, &target, &pre_rel)?;
    note_coverage(
        &tc,
        backend,
        BackendCoverage::ran(None, u64::try_from(files.len()).unwrap_or(u64::MAX)),
    );

    let mut out = json!({
        "ok": true,
        "command": "attach",
        "backend": backend.name(),
        "op": op.name(),
        "pid": a.pid,
        "consent_tier": tier,
        "started_by_dla": by_registry,
        "preflight": pre.compact(),
        "dir": tc.pointer(&rel_dir),
        "files": files.iter().map(|f| json!({
            "path": tc.pointer(&f.path), "bytes": f.bytes, "sha256": f.sha256,
        })).collect::<Vec<_>>(),
        "evidence": evidence,
        "summary": done.summary,
        "durations": done.durations,
        "cleanup_done": fin.cleanup_done,
        "cleanup_left": left,
    });
    if pre.unverified {
        out["unverified"] = Value::Bool(true);
    }
    if let Some(f) = files.iter().find(|f| f.path.ends_with(".hprof")) {
        out["next"] = json!(format!("dla summarize heap-graph {}", tc.pointer(&f.path)));
    }
    Ok(crate::out::bound(out))
}
