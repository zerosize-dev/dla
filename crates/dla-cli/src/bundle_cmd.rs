//! `dla bundle <F>`: a finding's label-free evidence bundle for an independent verifier.
//!
//! The bundle holds the finding's site with a code excerpt, copies of the raw files
//! behind its growth, retention, allocation and discriminator evidence, and an index
//! of those evidence records. It never holds the hypothesis, the refutation condition,
//! the archetype or suggestion, the explanation, the origin, or annotation notes and
//! labels: those keys are removed from every JSON file it contains.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use dla_core::{
    EvidenceId, FindingId,
    error::codes as core_codes,
    evidence::EvidenceClass,
    finding::check_site,
    fsutil::{is_written, to_pretty_json, write_atomic},
    paths,
    toolctx::ToolCtx,
};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    stage,
};

/// Flags of `dla bundle`.
#[derive(Args, Debug)]
pub struct BundleArgs {
    /// The finding (F3, jvm/F3).
    pub finding: String,
}

/// Keys that would tell the verifier what the investigation concluded; removed
/// everywhere in the bundle.
pub const FORBIDDEN_KEYS: [&str; 13] = [
    "hypothesis",
    "refute_if",
    "archetype",
    "suggested_archetype",
    "explanation",
    "origin",
    "annotation",
    "annotations",
    "note",
    "notes",
    "label",
    "labels",
    "remedy",
];

/// Lines of source shown on each side of the site.
const EXCERPT_RADIUS: usize = 20;
/// Largest raw file copied into a bundle.
const MAX_COPY_BYTES: u64 = 8 * 1024 * 1024;
/// Largest source file read for the excerpt.
const MAX_SOURCE_BYTES: u64 = 4 * 1024 * 1024;
/// Evidence classes a bundle carries.
const CLASSES: [EvidenceClass; 4] = [
    EvidenceClass::Growth,
    EvidenceClass::Retention,
    EvidenceClass::Allocation,
    EvidenceClass::Discriminator,
];

/// Removes every [`FORBIDDEN_KEYS`] entry from `v`, at any depth.
pub fn strip_forbidden(v: &mut Value) {
    match v {
        Value::Object(o) => {
            o.retain(|k, _| !FORBIDDEN_KEYS.contains(&k.as_str()));
            o.values_mut().for_each(strip_forbidden);
        }
        Value::Array(a) => a.iter_mut().for_each(strip_forbidden),
        _ => {}
    }
}

fn io_fail(path: &Path, e: std::io::Error) -> Failure {
    dla_core::Error::io(path, e).into()
}

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

/// `jvm/F3` -> `jvm_F3`.
fn dir_name(id: &FindingId) -> String {
    id.to_string().replace('/', "_")
}

/// The evidence of the bundle: the finding's and its siblings' supporting and
/// contradicting records of the four classes, in ID order, without duplicates.
fn evidence_ids(ledger: &dla_core::Ledger, id: &FindingId) -> Vec<EvidenceId> {
    let mut findings = vec![id.clone()];
    if let Some(f) = ledger.finding(id) {
        findings.extend(f.siblings.iter().cloned());
        if let Some(p) = f.parent.clone() {
            findings.push(p);
        }
    }
    let mut ids: Vec<EvidenceId> = findings
        .iter()
        .filter_map(|f| ledger.finding(f))
        .flat_map(|f| f.supports.iter().chain(f.contradicts.iter()).cloned())
        .collect();
    ids.sort();
    ids.dedup();
    ids.retain(|e| {
        ledger
            .evidence_by_id(e)
            .is_some_and(|r| CLASSES.contains(&r.kind.class()))
    });
    ids
}

/// Numbered excerpt of `file` around `line`, when the file is a readable source file
/// inside the project.
fn excerpt(project: &Path, site: &str) -> Option<String> {
    let (file, line) = site.rsplit_once(':')?;
    let line: usize = line.parse().ok()?;
    if file.starts_with('/') || file.split(['/', '\\']).any(|c| c == "..") {
        return None;
    }
    let path = project.join(file);
    let meta = fs::metadata(&path).ok()?;
    if !meta.is_file() || meta.len() > MAX_SOURCE_BYTES {
        return None;
    }
    let text = String::from_utf8_lossy(&fs::read(&path).ok()?).into_owned();
    let lines: Vec<&str> = text.lines().collect();
    if line == 0 || line > lines.len() {
        return None;
    }
    let from = line.saturating_sub(EXCERPT_RADIUS).max(1);
    let to = (line + EXCERPT_RADIUS).min(lines.len());
    let mut out = format!("{file}:{line} (lines {from}-{to})\n");
    for n in from..=to {
        let mark = if n == line { ">" } else { " " };
        out.push_str(&format!("{mark}{n:>6}  {}\n", lines[n - 1]));
    }
    Some(out)
}

/// A raw file's bytes as the bundle keeps them: JSON is stripped of forbidden keys.
fn bundle_bytes(raw: &[u8], is_json_name: bool) -> Vec<u8> {
    if is_json_name && let Ok(mut v) = serde_json::from_slice::<Value>(raw) {
        strip_forbidden(&mut v);
        return to_pretty_json(&v).into_bytes();
    }
    raw.to_vec()
}

fn files_under(dir: &Path, base: &Path, out: &mut Vec<(String, PathBuf)>) -> std::io::Result<()> {
    for e in fs::read_dir(dir)? {
        let p = e?.path();
        if p.is_dir() {
            files_under(&p, base, out)?;
        } else if let Some(rel) = paths::rel_slash(base, &p) {
            out.push((rel, p));
        }
    }
    Ok(())
}

/// Runs `dla bundle`.
pub fn run(ctx: &Ctx, project: &Path, data: Option<&PathBuf>, a: BundleArgs) -> Res<Value> {
    let tc = stage::tool_ctx(project, ctx, data);
    let mut id: FindingId = a
        .finding
        .parse()
        .map_err(|e: dla_core::ids::IdParseError| {
            Failure::refuse(
                core_codes::INVALID_VALUE,
                e.to_string(),
                "a finding id such as F3 or jvm/F3",
                "dla ledger resolve --help   (finding ids are listed by dla run status)",
            )
        })?;
    if id.prefix().is_none()
        && let Some(w) = &ctx.worker
    {
        id = id.reprefixed(Some(w));
    }
    let ledger = tc.ledger()?;
    let finding = ledger.finding(&id).ok_or_else(|| {
        Failure::refuse(
            core_codes::UNKNOWN_FINDING,
            format!("no finding {id} in this run's ledger"),
            format!("finding {id}"),
            "dla run status   (after dla ledger merge when the finding is a worker's)",
        )
    })?;
    let site = finding.site.clone().filter(|s| check_site(s).is_ok());
    let ids = evidence_ids(&ledger, &id);
    let dir = tc.next_bundle_dir(&dir_name(&id))?;
    let built = build(&tc, &ledger, &dir, project, &id, site.as_deref(), &ids);
    let (manifest_sha, files, copied, omitted, has_excerpt) = match built {
        Ok(b) => b,
        Err(f) => {
            let _ = fs::remove_dir_all(&dir);
            return Err(f);
        }
    };
    let rel = paths::rel_slash(&ctx.run_dir, &dir).unwrap_or_default();
    Ok(json!({
        "ok": true,
        "finding": id.to_string(),
        "dir": ctx.pointer(&rel),
        "manifest_sha256": manifest_sha,
        "files": files,
        "evidence": ids.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "raw_copied": copied,
        "raw_omitted": omitted,
        "excerpt": has_excerpt,
        "next": format!(
            "give the verifier only {} ; then: dla ledger verify {id} --site <file:line|none> --holder <h> --mechanism \"<text>\" --bundle {manifest_sha}",
            ctx.pointer(&rel)
        ),
    }))
}

type Built = (String, usize, usize, Vec<String>, bool);

fn build(
    tc: &ToolCtx,
    ledger: &dla_core::Ledger,
    dir: &Path,
    project: &Path,
    id: &FindingId,
    site: Option<&str>,
    ids: &[EvidenceId],
) -> Result<Built, Failure> {
    let write = |name: &str, bytes: &[u8]| -> Result<(), Failure> {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| io_fail(parent, e))?;
        }
        write_atomic(&path, bytes).map_err(Failure::from)
    };
    let mut copied = 0usize;
    let mut omitted: Vec<String> = Vec::new();
    let mut records: Vec<Value> = Vec::new();
    for eid in ids {
        let Some(e) = ledger.evidence_by_id(eid) else {
            continue;
        };
        let mut result = e.result.clone();
        strip_forbidden(&mut result);
        let mut rec = Map::new();
        rec.insert("id".into(), json!(e.evidence_id.to_string()));
        rec.insert("kind".into(), json!(e.kind.as_str()));
        rec.insert("authority".into(), json!(e.authority.as_str()));
        rec.insert("result".into(), result);
        if let Some(raw) = &e.raw {
            let src = tc.run_dir.join(raw);
            let safe = !raw.starts_with('/') && !raw.split(['/', '\\']).any(|c| c == "..");
            let name = format!(
                "evidence/{}-{}",
                e.evidence_id.to_string().replace('/', "_"),
                raw.rsplit('/').next().unwrap_or("raw")
            );
            if !safe || !src.is_file() || !is_written(&src) {
                omitted.push(format!("{eid}: {raw} is missing"));
            } else if fs::metadata(&src).is_ok_and(|m| m.len() > MAX_COPY_BYTES) {
                omitted.push(format!("{eid}: {raw} is over {MAX_COPY_BYTES} bytes"));
            } else {
                let bytes = fs::read(&src).map_err(|e| io_fail(&src, e))?;
                write(&name, &bundle_bytes(&bytes, raw.ends_with(".json")))?;
                rec.insert("file".into(), json!(name));
                copied += 1;
            }
        }
        records.push(Value::Object(rec));
    }
    let excerpt_text = site.and_then(|s| excerpt(project, s));
    let has_excerpt = excerpt_text.is_some();
    if let Some(text) = &excerpt_text {
        write("excerpt.txt", text.as_bytes())?;
    }
    let mut index = json!({
        "finding": id.to_string(),
        "site": site,
        "excerpt": has_excerpt.then_some("excerpt.txt"),
        "evidence": records,
    });
    if !omitted.is_empty() {
        index["omitted"] = json!(omitted);
    }
    write("index.json", to_pretty_json(&index).as_bytes())?;

    let mut files = Vec::new();
    files_under(dir, dir, &mut files).map_err(|e| io_fail(dir, e))?;
    files.sort();
    let mut listing = Vec::new();
    for (rel, path) in &files {
        let bytes = fs::read(path).map_err(|e| io_fail(path, e))?;
        listing.push(json!({"path": rel, "sha256": sha(&bytes)}));
    }
    let manifest = to_pretty_json(&json!({"files": listing}));
    write("manifest.json", manifest.as_bytes())?;
    Ok((
        sha(manifest.as_bytes()),
        listing.len() + 1,
        copied,
        omitted,
        has_excerpt,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forbidden_keys_go_at_every_depth() {
        let mut v = json!({
            "a": 1,
            "hypothesis": "x",
            "nested": {"origin": "census", "keep": [{"label": "L-X", "n": 2}], "notes": ["n"]},
            "archetype": "A",
        });
        strip_forbidden(&mut v);
        assert_eq!(v, json!({"a": 1, "nested": {"keep": [{"n": 2}]}}));
    }

    #[test]
    fn excerpt_marks_the_site_and_clamps_at_the_ends() {
        let dir = tempfile::tempdir().unwrap();
        let body: String = (1..=50).map(|n| format!("line {n}\n")).collect();
        fs::write(dir.path().join("a.txt"), body).unwrap();
        let t = excerpt(dir.path(), "a.txt:25").unwrap();
        assert!(t.starts_with("a.txt:25 (lines 5-45)\n"), "{t}");
        assert!(t.contains(">    25  line 25\n"), "{t}");
        let t = excerpt(dir.path(), "a.txt:2").unwrap();
        assert!(t.starts_with("a.txt:2 (lines 1-22)"), "{t}");
        assert!(excerpt(dir.path(), "a.txt:51").is_none());
        assert!(excerpt(dir.path(), "../a.txt:1").is_none());
        assert!(excerpt(dir.path(), "missing.txt:1").is_none());
    }
}
