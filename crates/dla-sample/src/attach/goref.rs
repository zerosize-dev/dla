//! `grf attach`: a heap profile of a running Go process (goref).
//!
//! Written from goref's README and source. It could not be executed where this was written
//! (a permission check refused running a `go install`ed binary), so every result is marked
//! unverified.

use std::{
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use serde_json::json;

use super::{
    Backend, Detail, Done, Plan, Request, Tracker,
    resources::Estimate,
    util::{AttachError, Cleanup, clip, first_error_line, output_name},
};
use crate::exec;

const MIB: u64 = 1 << 20;
/// `grf attach` suspends the target; it is given this long before DLA stops waiting.
const RUN_LIMIT: Duration = Duration::from_secs(600);

pub(crate) struct Prep {
    exe: PathBuf,
}

fn find_grf(req: &Request) -> Option<PathBuf> {
    let mut extra = req.tool_dirs.clone();
    if let Some(g) = std::env::var_os("GOBIN") {
        extra.push(PathBuf::from(g));
    }
    if let Some(g) = std::env::var_os("GOPATH") {
        extra.push(PathBuf::from(g).join("bin"));
    }
    if let Some(h) = std::env::var_os("HOME") {
        extra.push(PathBuf::from(h).join("go/bin"));
    }
    exec::find_exe("grf", &extra)
}

pub(crate) fn plan(req: &Request) -> Result<Plan, AttachError> {
    let pid = req.target.pid;
    let Some(exe) = find_grf(req) else {
        return Err(AttachError::tool_missing(
            "grf (goref) is not installed",
            "grf on PATH",
            "go install github.com/cloudwego/goref/cmd/grf@latest   (needs Go 1.21 or later; the target must be built with Go 1.19 to 1.26)",
        ));
    };
    let file = match &req.out_name {
        Some(n) => output_name(n, "out").map_err(|m| {
            AttachError::not_ready(
                m,
                "--out as a plain file name",
                "dla attach ... --out heap.out",
            )
        })?,
        None => "grf.out".to_owned(),
    };
    let rss = req.target.rss.unwrap_or(256 * MIB);
    Ok(Plan {
        backend: Backend::Goref,
        op: req.op,
        tool: format!("grf {}", exe.display()),
        expected_signal: "a gzipped pprof profile (inuse space and inuse objects) of the Go heap, read with go tool pprof".into(),
        stopping: "grf exits after writing the profile; DLA stops waiting after 10 minutes".into(),
        effects: vec![
            format!("grf attaches to {pid} with ptrace (delve) and SUSPENDS the process for the whole run"),
            "grf loads the heap into its own memory; its documentation warns that attach mode can run a leaking process out of memory (the alternative is gcore, then grf core on another machine)".into(),
            "the profile holds type and size information of live objects".into(),
        ],
        cleanup: vec![
            "note the target resumes when grf exits; if grf is killed, check that the process is not left stopped (kill -CONT)".into(),
        ],
        estimate: Estimate {
            dump_bytes: 64 * MIB,
            peak_bytes: rss.saturating_mul(2).max(256 * MIB),
            basis: format!("not measured: twice the resident size ({rss} bytes) assumed for delve's heap walk; the profile is small"),
        },
        notes: vec!["goref has never been executed by DLA: treat the result as unverified".to_owned()],
        unverified: true,
        file,
        detail: Detail::Goref(Prep { exe }),
    })
}

pub(crate) fn run(
    p: &Prep,
    plan: &Plan,
    req: &Request,
    out_dir: &Path,
    t: &mut Tracker<'_>,
) -> Result<Done, AttachError> {
    let pid = req.target.pid;
    let path = out_dir.join(&plan.file);
    t.need(Cleanup::Rm(path.clone()));
    t.need(Cleanup::Note(format!(
        "grf suspends process {pid} while it runs; if the process is left stopped, run kill -CONT {pid}"
    )));
    let mut c = Command::new(&p.exe);
    c.args(["attach", &pid.to_string(), "-o"]).arg(&path);
    let r = exec::run(c, RUN_LIMIT.max(req.duration)).map_err(|e| {
        AttachError::failed(
            format!("grf attach: {e}"),
            "check the process is not left stopped (kill -CONT)",
        )
    })?;
    if !r.success {
        return Err(AttachError::failed(
            format!(
                "grf attach failed: {}",
                clip(&first_error_line(&r.stderr, &r.stdout), 200)
            ),
            "check the target is a Go program built with Go 1.19 to 1.26 and that ptrace is allowed",
        ));
    }
    let bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    if bytes == 0 {
        return Err(AttachError::failed(
            "grf attach reported success but wrote no profile",
            "repeat the command",
        ));
    }
    t.keep(&Cleanup::Rm(path.clone()));
    let mut done = Done::default();
    done.files.push(plan.file.clone());
    done.summary = json!({
        "bytes": bytes,
        "unverified": true,
        "read_with": format!("go tool pprof {}", plan.file),
    });
    done.limitations.push("goref was never executed by DLA before this run: the profile format is taken from its documentation".into());
    Ok(done)
}
