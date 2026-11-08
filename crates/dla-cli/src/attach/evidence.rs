//! The artifact evidence of an attach: file hashes and the bounded result.

use std::{fs::File, io::Read, path::Path};

use dla_core::{
    MAX_RESULT_BYTES,
    evidence::{Authority, CoverageStatus, EvidenceCoverage, EvidenceKind},
    toolctx::{EvidenceSpec, ToolCtx},
};
use dla_sample::attach::{Backend, Done, Finished, Op, Preflight, TargetInfo};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

/// One artifact file.
#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    /// Path relative to the run directory (`attach/P1/heap.hprof`).
    pub path: String,
    /// Size in bytes.
    pub bytes: u64,
    /// SHA-256 of the contents.
    pub sha256: String,
}

/// Size and hash of `path`, read in 1 MiB steps (a heap dump is hundreds of MiB).
pub fn hash_file(path: &Path) -> std::io::Result<(u64, String)> {
    let mut f = File::open(path)?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    let mut total = 0u64;
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
        total += n as u64;
    }
    Ok((total, hex::encode(h.finalize())))
}

/// The entries of the files `done` lists that exist in `run_dir/<rel_dir>`.
pub fn file_entries(run_dir: &Path, rel_dir: &str, done: &Done) -> Vec<FileEntry> {
    done.files
        .iter()
        .filter_map(|name| {
            let rel = format!("{rel_dir}/{name}");
            let (bytes, sha256) = hash_file(&run_dir.join(&rel)).ok()?;
            Some(FileEntry {
                path: rel,
                bytes,
                sha256,
            })
        })
        .collect()
}

/// How much of the subject the backend's result covers.
pub fn coverage(backend: Backend, op: Op) -> CoverageStatus {
    match (backend, op) {
        // Allocations after the attach only, or a sampling profile.
        (Backend::Memray | Backend::Heaptrack, _) | (Backend::Jcmd, Op::Jfr) => {
            CoverageStatus::Partial
        }
        (Backend::Goref, _) => CoverageStatus::Unknown,
        _ => CoverageStatus::Complete,
    }
}

/// Everything the evidence result holds, before it is cut to size.
pub struct Parts<'a> {
    /// Backend.
    pub backend: Backend,
    /// Operation.
    pub op: Op,
    /// Files with hashes.
    pub files: &'a [FileEntry],
    /// The preflight.
    pub preflight: &'a Preflight,
    /// The finished attach.
    pub fin: &'a Finished,
    /// What the backend produced.
    pub done: &'a Done,
    /// Cleanup left as text.
    pub left: &'a [String],
}

/// `{backend, op, files, preflight, durations, cleanup_done, unverified?, summary}` under the
/// evidence size bound: the summary and the preflight are cut before the files are.
pub fn result(p: &Parts<'_>) -> Value {
    let mut v = json!({
        "backend": p.backend.name(),
        "op": p.op.name(),
        "files": p.files,
        "preflight": p.preflight.compact(),
        "durations": p.done.durations,
        "cleanup_done": p.fin.cleanup_done,
        "cleanup_left": p.left.len(),
        "summary": p.done.summary,
    });
    if p.preflight.unverified {
        v["unverified"] = Value::Bool(true);
    }
    let size = |v: &Value| serde_json::to_string(v).map_or(usize::MAX, |s| s.len());
    let limit = MAX_RESULT_BYTES - 64;
    if size(&v) > limit {
        v["summary"] = shrink_summary(&p.done.summary);
    }
    if size(&v) > limit {
        v["preflight"] = json!({
            "pid": p.preflight.target.pid,
            "decision": p.preflight.decision,
            "est_dump": p.preflight.estimate.dump_bytes,
            "est_peak": p.preflight.estimate.peak_bytes,
        });
    }
    if size(&v) > limit {
        v["summary"] = Value::Null;
        v["cleanup_done"] = json!(p.fin.cleanup_done.len());
    }
    v
}

/// Keeps the scalar fields of a summary object.
fn shrink_summary(s: &Value) -> Value {
    match s {
        Value::Object(o) => Value::Object(
            o.iter()
                .filter(|(_, v)| !v.is_array() && !v.is_object())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        ),
        other => other.clone(),
    }
}

fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    text.chars().take(max).collect()
}

/// Adds the evidence record of a finished attach and returns its ID.
pub fn add(
    tc: &ToolCtx,
    plan_tool: &str,
    parts: &Parts<'_>,
    target: &TargetInfo,
    raw: &str,
) -> Result<String, dla_core::Refusal> {
    let mut spec = EvidenceSpec::new(
        EvidenceKind::Artifact,
        parts.backend.name(),
        &clip(plan_tool, 80),
        result(parts),
    );
    spec.authority = Authority::Live;
    spec.raw = Some(raw.to_owned());
    spec.source = Some(ToolCtx::process_source(
        target.pid,
        target.start_time.as_deref().unwrap_or("unknown"),
    ));
    spec.coverage = EvidenceCoverage {
        status: coverage(parts.backend, parts.op),
        examined: Some(parts.files.len() as u64),
        total: None,
    };
    spec.limitations = parts.done.limitations.iter().take(8).cloned().collect();
    if parts.preflight.unverified {
        spec.limitations.insert(
            0,
            "unverified: the backend was written from documentation and never executed".into(),
        );
    }
    let mut ledger = tc.ledger()?;
    Ok(tc.add_evidence(&mut ledger, spec)?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hashes_match_sha256_of_the_bytes() {
        let d = tempfile::tempdir().unwrap();
        let p = d.path().join("f");
        std::fs::write(&p, b"abc").unwrap();
        let (n, h) = hash_file(&p).unwrap();
        assert_eq!(n, 3);
        assert_eq!(
            h,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert!(hash_file(&d.path().join("missing")).is_err());
    }

    #[test]
    fn coverage_is_partial_for_recordings_and_unknown_for_goref() {
        assert_eq!(
            coverage(Backend::Jcmd, Op::HeapDump),
            CoverageStatus::Complete
        );
        assert_eq!(coverage(Backend::Jcmd, Op::Jfr), CoverageStatus::Partial);
        assert_eq!(
            coverage(Backend::Memray, Op::Record),
            CoverageStatus::Partial
        );
        assert_eq!(
            coverage(Backend::Heaptrack, Op::Record),
            CoverageStatus::Partial
        );
        assert_eq!(
            coverage(Backend::Goref, Op::HeapProfile),
            CoverageStatus::Unknown
        );
        assert_eq!(
            coverage(Backend::NodeInspector, Op::Memory),
            CoverageStatus::Complete
        );
    }

    #[test]
    fn shrinking_a_summary_keeps_scalars() {
        let s = json!({"bytes": 5, "top": [1,2,3], "nested": {"a": 1}, "ok": true});
        assert_eq!(shrink_summary(&s), json!({"bytes": 5, "ok": true}));
    }
}
