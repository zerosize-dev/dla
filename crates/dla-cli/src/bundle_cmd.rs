//! `dla bundle <F>`: a finding's label-free evidence bundle for an independent verifier.
//!
//! The bundle holds copies of the raw files behind the finding's growth, retention,
//! allocation and discriminator evidence, an index of those evidence records, and code
//! excerpts around up to four project frames that the bundled allocation and retention
//! evidence names. It never holds the finding's own site, the finding id, the
//! hypothesis, the refutation condition, the archetype or suggestion, the label, the
//! explanation, the origin, or annotation notes: those keys are removed from every JSON
//! file it contains.
//!
//! Growth columns are tied to a finding by a site, a finding id or a candidate list, and
//! their names can embed a site (`alloc@src/a.js:7`). Every record and raw copy is
//! scrubbed alike: the `finding`, `candidates`, `ties`, `tie`, `tied` and `basis` keys go
//! at every depth, a column object loses its tie keys, a column name is cut at `@`, and the
//! finding id (with and without the worker prefix), its site and its recorded writers are
//! replaced in every string, except in the structured frame fields of allocation stacks and
//! retention paths, which are raw evidence the verifier reads. Excerpts come from those
//! frames only: the verifier has to find the site from the stacks and paths.
//!
//! The records a transition cites are not the only ones: the newest two allocation-stack and
//! two retention-path records that are about the finding (`labels::about_finding`: the rule
//! `dla ledger label` applies, `names_finding`, or an allocation stack with an untied growing row
//! at the finding's line) are bundled too. A confirmed or strong finding whose growth is a
//! count of blocks allocated at one line (`py.blocks@<file:line>`) would otherwise reach the
//! verifier with no stack and no excerpt, so the bundle is refused with `stacks-not-captured`
//! when the runtime can capture the stacks and no capture run of the harness that grew the
//! column is recorded.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use dla_core::{
    EvidenceId, FindingId,
    error::codes as core_codes,
    evidence::{EvidenceClass, EvidenceKind},
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

/// Lines of source shown on each side of a frame.
const EXCERPT_RADIUS: usize = 20;
/// Most frames that get an excerpt.
const MAX_EXCERPTS: usize = 4;
/// Most candidate frames tried before giving up.
const MAX_CANDIDATES: usize = 400;
/// Most files indexed when a frame names a bare file name.
const MAX_INDEXED_FILES: usize = 50_000;
/// Largest raw text file scanned for frames.
const MAX_SCAN_BYTES: usize = 1024 * 1024;
/// Directories whose files are never project frames.
const SKIP_DIRS: [&str; 10] = [
    ".git",
    ".dla",
    "target",
    "node_modules",
    "vendor",
    "site-packages",
    ".venv",
    "venv",
    "__pycache__",
    "build",
];
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

/// Keys of a column object that tie it to a finding.
const TIE_KEYS: [&str; 6] = ["site", "finding", "candidates", "tie", "tied", "ambiguous"];

/// Keys dropped from every object of every bundled record and raw copy: they state which
/// finding a record belongs to or why it was tied to it.
const DROP_KEYS: [&str; 6] = ["finding", "candidates", "ties", "tie", "tied", "basis"];

/// Keys whose subtree is the structured frame evidence of an allocation stack or a
/// retention path: raw evidence the verifier must read, so the finding's site and writers
/// are not replaced inside it (the finding id still is).
const FRAME_KEYS: [&str; 8] = [
    "frames",
    "root_path",
    "path",
    "paths",
    "retention_path",
    "retention_paths",
    "chain",
    "via",
];

/// What a bundled record must not show: the finding's id (with and without the worker
/// prefix) and its site and recorded writers.
#[derive(Debug, Clone, Default)]
pub struct Needles {
    /// Finding ids, replaced as whole tokens.
    pub ids: Vec<String>,
    /// Sites and writers (`file:line`), replaced wherever they occur.
    pub sites: Vec<String>,
}

/// The replacement text.
const REDACTED: &str = "<redacted>";

/// Cuts a column name at `@`: `alloc@src/a.js:7` -> `alloc`.
fn bare_name(name: &str) -> &str {
    name.split_once('@').map_or(name, |(base, _)| base)
}

/// Replaces `id` where it is a whole token (no letter or digit on either side).
fn replace_token(s: &str, id: &str) -> String {
    if id.is_empty() || !s.contains(id) {
        return s.to_owned();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(at) = rest.find(id) {
        let before = rest[..at].chars().next_back();
        let after = rest[at + id.len()..].chars().next();
        let edge = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric());
        out.push_str(&rest[..at]);
        if edge(before) && edge(after) {
            out.push_str(REDACTED);
        } else {
            out.push_str(id);
        }
        rest = &rest[at + id.len()..];
    }
    out.push_str(rest);
    out
}

/// `s` without the finding id and, unless `keep_sites`, without the site and writers.
fn redact_text(s: &str, n: &Needles, keep_sites: bool) -> String {
    let mut out = s.to_owned();
    for id in &n.ids {
        out = replace_token(&out, id);
    }
    if !keep_sites {
        for site in n.sites.iter().filter(|x| !x.is_empty()) {
            if out.contains(site.as_str()) {
                out = out.replace(site.as_str(), REDACTED);
            }
        }
    }
    out
}

/// Scrubs one bundled JSON value:
///
/// * the [`DROP_KEYS`] go at every depth, and the tie keys of every object that names a
///   `column`;
/// * site-bearing column names and object keys are cut at `@`;
/// * the finding id is replaced in every string and key;
/// * the site and the recorded writers are replaced in every string and key, except under
///   a [`FRAME_KEYS`] key when `frames_exempt` (allocation and retention records).
pub fn scrub_ties(v: &mut Value, needles: &Needles, frames_exempt: bool) {
    scrub_in(v, needles, frames_exempt, false);
}

fn scrub_in(v: &mut Value, n: &Needles, frames_exempt: bool, in_frames: bool) {
    match v {
        Value::Object(o) => {
            o.retain(|k, _| !DROP_KEYS.contains(&k.as_str()));
            if o.contains_key("column") {
                o.retain(|k, _| !TIE_KEYS.contains(&k.as_str()));
            }
            for key in ["column", "name"] {
                if let Some(Value::String(s)) = o.get_mut(key)
                    && s.contains('@')
                {
                    *s = bare_name(s).to_owned();
                }
            }
            let renamed = o
                .keys()
                .any(|k| k.contains('@') || redact_text(k, n, in_frames) != *k);
            if renamed {
                let old = std::mem::take(o);
                for (k, x) in old {
                    let base = redact_text(bare_name(&k), n, in_frames);
                    let mut key = base.clone();
                    let mut i = 1;
                    while o.contains_key(&key) {
                        i += 1;
                        key = format!("{base}#{i}");
                    }
                    o.insert(key, x);
                }
            }
            for (k, x) in o.iter_mut() {
                let frames = in_frames || (frames_exempt && FRAME_KEYS.contains(&k.as_str()));
                scrub_in(x, n, frames_exempt, frames);
            }
        }
        Value::Array(a) => a
            .iter_mut()
            .for_each(|x| scrub_in(x, n, frames_exempt, in_frames)),
        Value::String(s) => {
            let t = redact_text(s, n, in_frames);
            if t != *s {
                *s = t;
            }
        }
        _ => {}
    }
}

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

/// The resolved path of `rel` under `base`, or `None` when `rel` is not a plain
/// relative path (an absolute path, a Windows drive prefix, `.` or `..` parts) or when
/// the file resolves, through a symlink, to somewhere outside `base`.
fn contained(base: &Path, rel: &str) -> Option<std::path::PathBuf> {
    use std::path::Component;
    let rel_path = Path::new(rel);
    if rel.is_empty()
        || rel.starts_with(['/', '\\'])
        || !rel_path
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
        || rel.split(['/', '\\']).any(|c| c == "..")
        || rel.as_bytes().get(1) == Some(&b':')
    {
        return None;
    }
    let canon_base = fs::canonicalize(base).ok()?;
    let canon = fs::canonicalize(base.join(rel_path)).ok()?;
    canon.starts_with(&canon_base).then_some(canon)
}

/// Numbered excerpt of `file` around `line`, when the file is a readable source file
/// inside the project.
fn excerpt(project: &Path, site: &str) -> Option<String> {
    let (file, line) = site.rsplit_once(':')?;
    let line: usize = line.parse().ok()?;
    let path = contained(project, file)?;
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

/// Characters that end a `file:line` token inside free text.
const TOKEN_BREAKS: &str = "()[]{}<>,;\"'`|=";

/// Appends `file:line` when `file` looks like a source file name.
fn push_frame(file: &str, line: usize, out: &mut Vec<(String, usize)>) {
    let name = file.rsplit(['/', '\\']).next().unwrap_or("");
    let ext_ok = name.rsplit_once('.').is_some_and(|(stem, ext)| {
        !stem.is_empty()
            && !ext.is_empty()
            && ext.len() <= 8
            && ext.bytes().all(|b| b.is_ascii_alphanumeric())
            && ext.bytes().any(|b| b.is_ascii_alphabetic())
    });
    if ext_ok && line > 0 && out.len() < MAX_CANDIDATES * 10 {
        out.push((file.to_owned(), line));
    }
}

/// Appends every `file:line` (or `file:line:column`) token found in `s`.
fn push_text_frames(s: &str, out: &mut Vec<(String, usize)>) {
    for tok in s.split(|c: char| c.is_whitespace() || TOKEN_BREAKS.contains(c)) {
        let tok = tok.trim_end_matches(['.', ':']);
        if tok.len() > 400 || !tok.contains(':') {
            continue;
        }
        let parts: Vec<&str> = tok.split(':').collect();
        for i in 1..parts.len() {
            let n = parts[i];
            if !n.is_empty()
                && n.bytes().all(|b| b.is_ascii_digit())
                && let Ok(line) = n.parse::<usize>()
            {
                push_frame(&parts[..i].join(":"), line, out);
                break;
            }
        }
    }
}

/// The frames a JSON value names: `{"file", "line"}` objects, `[file, line]` pairs and
/// `file:line` tokens in any string or object key, in document order.
fn frames_in_value(v: &Value, out: &mut Vec<(String, usize)>) {
    match v {
        Value::String(s) => push_text_frames(s, out),
        Value::Array(a) => {
            if let [Value::String(f), Value::Number(n)] = a.as_slice()
                && let Some(line) = n.as_u64()
            {
                push_frame(f, line as usize, out);
            }
            a.iter().for_each(|x| frames_in_value(x, out));
        }
        Value::Object(o) => {
            if let (Some(Value::String(f)), Some(line)) =
                (o.get("file"), o.get("line").and_then(Value::as_u64))
            {
                push_frame(f, line as usize, out);
            }
            for (k, x) in o {
                push_text_frames(k, out);
                frames_in_value(x, out);
            }
        }
        _ => {}
    }
}

/// The frames named by a bundled raw file: its JSON strings, else its text.
fn frames_in_bytes(bytes: &[u8], out: &mut Vec<(String, usize)>) {
    if let Ok(v) = serde_json::from_slice::<Value>(bytes) {
        frames_in_value(&v, out);
    } else {
        let head = &bytes[..bytes.len().min(MAX_SCAN_BYTES)];
        push_text_frames(&String::from_utf8_lossy(head), out);
    }
}

/// Turns the file part of a frame into a path relative to the project.
struct Resolver<'a> {
    project: &'a Path,
    canon: Option<PathBuf>,
    index: Option<std::collections::HashMap<String, Vec<String>>>,
}

impl<'a> Resolver<'a> {
    fn new(project: &'a Path) -> Self {
        Resolver {
            project,
            canon: fs::canonicalize(project).ok(),
            index: None,
        }
    }

    /// Project files by base name, built on first use; vendored and build directories,
    /// hidden directories and symlinks are left out.
    fn index(&mut self) -> &std::collections::HashMap<String, Vec<String>> {
        let project = self.project;
        self.index.get_or_insert_with(|| {
            let mut map: std::collections::HashMap<String, Vec<String>> = Default::default();
            let mut stack = vec![project.to_path_buf()];
            let mut seen = 0usize;
            while let Some(dir) = stack.pop() {
                let Ok(rd) = fs::read_dir(&dir) else { continue };
                for e in rd.flatten() {
                    let name = e.file_name().to_string_lossy().into_owned();
                    let Ok(ft) = e.file_type() else { continue };
                    if ft.is_symlink() {
                        continue;
                    }
                    if ft.is_dir() {
                        if !name.starts_with('.') && !SKIP_DIRS.contains(&name.as_str()) {
                            stack.push(e.path());
                        }
                    } else if ft.is_file() && name.contains('.') {
                        seen += 1;
                        if seen > MAX_INDEXED_FILES {
                            return map;
                        }
                        if let Some(rel) = paths::rel_slash(project, &e.path()) {
                            map.entry(name).or_default().push(rel);
                        }
                    }
                }
            }
            map
        })
    }

    fn resolve(&mut self, file: &str) -> Option<String> {
        let norm = file.replace('\\', "/");
        let mut rel = norm.trim_start_matches("./").to_owned();
        if Path::new(&rel).is_absolute() {
            let p = Path::new(&rel);
            let stripped = self
                .canon
                .as_deref()
                .and_then(|c| p.strip_prefix(c).ok())
                .or_else(|| p.strip_prefix(self.project).ok())?;
            rel = stripped.to_string_lossy().replace('\\', "/");
        }
        let skipped = |r: &str| {
            r.split('/')
                .any(|c| SKIP_DIRS.contains(&c) || (c.starts_with('.') && c.len() > 1))
        };
        if skipped(&rel) {
            return None;
        }
        if contained(self.project, &rel).is_some_and(|p| p.is_file()) {
            return Some(rel);
        }
        let base = rel.rsplit('/').next()?.to_owned();
        let suffix = format!("/{rel}");
        let hits: Vec<&String> = self
            .index()
            .get(&base)?
            .iter()
            .filter(|c| **c == rel || c.ends_with(&suffix))
            .collect();
        match hits.as_slice() {
            [one] => Some((*one).clone()),
            _ => None,
        }
    }
}

/// Up to [`MAX_EXCERPTS`] excerpts of distinct project frames, in candidate order, and
/// the frames they are centred on.
fn excerpts_for(project: &Path, candidates: &[(String, usize)]) -> (Vec<String>, Vec<String>) {
    let mut resolver = Resolver::new(project);
    let mut seen: Vec<(String, usize)> = Vec::new();
    let (mut texts, mut frames) = (Vec::new(), Vec::new());
    for (file, line) in candidates.iter().take(MAX_CANDIDATES) {
        if texts.len() >= MAX_EXCERPTS {
            break;
        }
        let Some(rel) = resolver.resolve(file) else {
            continue;
        };
        let key = (rel, *line);
        if seen.contains(&key) {
            continue;
        }
        seen.push(key.clone());
        if let Some(t) = excerpt(project, &format!("{}:{}", key.0, key.1)) {
            texts.push(t);
            frames.push(format!("{}:{}", key.0, key.1));
        }
    }
    (texts, frames)
}

/// A raw file's bytes as the bundle keeps them: JSON is stripped of forbidden keys and
/// scrubbed; other text loses the finding id (and the site and writers unless the record is
/// frame evidence, whose text cannot be split into frame and other fields); bytes that are
/// not text are kept.
fn bundle_bytes(raw: &[u8], is_json_name: bool, needles: &Needles, frames_exempt: bool) -> Vec<u8> {
    if is_json_name && let Ok(mut v) = serde_json::from_slice::<Value>(raw) {
        strip_forbidden(&mut v);
        scrub_ties(&mut v, needles, frames_exempt);
        return to_pretty_json(&v).into_bytes();
    }
    match std::str::from_utf8(raw) {
        Ok(text) => redact_text(text, needles, frames_exempt).into_bytes(),
        Err(_) => raw.to_vec(),
    }
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
                "dla run status   (the finding ids it lists, such as F3 or jvm/F3)",
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
    let owner = finding.owner.clone();
    let mut ids = evidence_ids(&ledger, &id);
    ids.extend(naming_records(&ledger, finding));
    ids.sort();
    ids.dedup();
    let stacks_note = stacks_check(&tc, &id, &ids)?;
    let dir = tc.next_bundle_dir(&dir_name(&id))?;
    let built = build(
        &tc,
        &ledger,
        &dir,
        project,
        &id,
        owner.as_deref(),
        &ids,
        stacks_note.as_deref(),
    );
    let (manifest_sha, files, copied, omitted, frames) = match built {
        Ok(b) => b,
        Err(f) => {
            let _ = fs::remove_dir_all(&dir);
            return Err(f);
        }
    };
    let rel = paths::rel_slash(&ctx.run_dir, &dir).unwrap_or_default();
    let mut result = json!({
        "ok": true,
        "finding": id.to_string(),
        "dir": ctx.pointer(&rel),
        "manifest_sha256": manifest_sha,
        "files": files,
        "evidence": ids.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "raw_copied": copied,
        "raw_omitted": omitted,
        "excerpt": !frames.is_empty(),
        "excerpt_frames": frames,
        "next": format!(
            "give the verifier only {} ; then: dla ledger verify {id} --site <file:line|none> --holder <h> --mechanism \"<text>\" --label <ID|UNCLASSIFIED> --bundle {manifest_sha}",
            ctx.pointer(&rel)
        ),
    });
    if let Some(note) = &stacks_note {
        result["stacks_note"] = json!(note);
    }
    Ok(result)
}

/// At most this many allocation-stack records and this many retention-path records that name
/// the finding are added to a bundle.
const MAX_NAMING_PER_KIND: usize = 2;

/// The newest allocation-stack and retention-path records that name the finding or its root by
/// the rule the label uses, at most [`MAX_NAMING_PER_KIND`] of each kind.
fn naming_records(ledger: &dla_core::Ledger, f: &dla_core::Finding) -> Vec<EvidenceId> {
    let mut subjects = vec![f];
    if let Some(root) = f.parent.as_ref().and_then(|p| ledger.finding(p)) {
        subjects.push(root);
    }
    let mut out = Vec::new();
    for kind in [EvidenceKind::AllocStack, EvidenceKind::RetentionPath] {
        let mut named: Vec<EvidenceId> = ledger
            .evidence()
            .filter(|e| e.kind == kind && dla_core::labels::about_finding(e, &subjects))
            .map(|e| e.evidence_id.clone())
            .collect();
        named.reverse();
        named.truncate(MAX_NAMING_PER_KIND);
        out.extend(named);
    }
    out
}

/// The files `harness/<H>/captures.json` lists that lead to the growing line (kind `alloc`, and,
/// for a runtime whose heap capture is the holder chains of the objects, the chains file), with
/// the time the list was written; `None` when there is no such list or it names no allocation
/// capture.
fn alloc_captures(
    run_dir: &Path,
    harness: &str,
) -> Option<(Vec<dla_core::harness::CaptureFile>, std::time::SystemTime)> {
    use dla_core::harness::CaptureKind;
    let path = dla_core::HarnessManifest::dir_in(run_dir, harness).join("captures.json");
    let v: Value = serde_json::from_str(&fs::read_to_string(&path).ok()?).ok()?;
    let files: Vec<dla_core::harness::CaptureFile> =
        serde_json::from_value(v.get("files")?.clone()).ok()?;
    let chains = |f: &dla_core::harness::CaptureFile| {
        f.kind == CaptureKind::Heap && f.path.ends_with(".retain.json")
    };
    let has_alloc = files.iter().any(|f| f.kind == CaptureKind::Alloc);
    let files: Vec<_> = files
        .into_iter()
        .filter(|f| f.kind == CaptureKind::Alloc || chains(f))
        .collect();
    let at = fs::metadata(&path).and_then(|m| m.modified()).ok()?;
    (has_alloc && !files.is_empty()).then_some((files, at))
}

/// The SHA-256 of the capture file at run-relative `rel`: the digest its entry in `files` stores
/// (a file deleted to keep the run under its size limit keeps it), else the digest of the file.
fn digest_of(
    run_dir: &Path,
    files: &[dla_core::harness::CaptureFile],
    rel: &str,
) -> Option<String> {
    files
        .iter()
        .find(|f| f.path == rel)
        .and_then(|f| f.sha256.clone())
        .or_else(|| fs::read(run_dir.join(rel)).ok().map(|b| sha(&b)))
}

/// The summarize commands of the capture `files` that were not run: an `alloc-stack-diff`
/// command is run when an allocation-stack record was made from one of its files, a `heap-graph`
/// command when a retention-path record was. The raw file of a summary lists the sha256 of each
/// input in its `captures` array (`alloc/A<n>.json`, `retain/R<n>.json`), and the file of the
/// capture has that digest.
fn unrun_commands(
    tc: &ToolCtx,
    ledger: &dla_core::Ledger,
    files: &[dla_core::harness::CaptureFile],
) -> (Vec<String>, usize) {
    let base = tc.pointer("");
    let all: Vec<String> =
        dla_workload::harness::capture::summarize_commands(files, &|p| tc.pointer(p), false)
            .into_iter()
            .filter(|c| {
                c.starts_with("dla summarize alloc-stack-diff ")
                    || c.starts_with("dla summarize heap-graph ")
            })
            .collect();
    let total = all.len();
    let unrun = all
        .into_iter()
        .filter(|c| {
            let kind = if c.starts_with("dla summarize alloc-stack-diff ") {
                EvidenceKind::AllocStack
            } else {
                EvidenceKind::RetentionPath
            };
            let digests: Vec<String> = c
                .split_whitespace()
                .skip(3)
                .filter_map(|p| p.strip_prefix(&base))
                .filter_map(|rel| digest_of(&tc.run_dir, files, rel))
                .collect();
            !made_from(&tc.run_dir, ledger, kind, &digests)
        })
        .collect();
    (unrun, total)
}

/// The literal `dla summarize alloc-stack-diff --finding <F> <files>` command of the capture that
/// an allocation-stack record about `finding` was made from, when the record states no tie to it
/// (the summary was written before the finding existed): the record's raw file lists the sha256
/// of each input, and the command is rebuilt from `harness/<H>/captures.json` by those digests.
pub(crate) fn literal_summarize(
    run_dir: &Path,
    pointer: &dyn Fn(&str) -> String,
    ledger: &dla_core::Ledger,
    finding: &FindingId,
) -> Option<String> {
    let f = ledger.finding(finding)?;
    let subjects = subjects_of(ledger, f);
    let base = pointer("");
    let heads = fs::read_dir(run_dir.join("harness")).ok()?;
    let mut harness_files: Vec<Vec<dla_core::harness::CaptureFile>> = Vec::new();
    for h in heads.flatten() {
        let text = fs::read_to_string(h.path().join("captures.json")).ok();
        let files = text
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            .and_then(|v| v.get("files").cloned())
            .and_then(|v| serde_json::from_value(v).ok());
        harness_files.extend(files);
    }
    for rec in ledger
        .evidence()
        .filter(|e| {
            e.kind == EvidenceKind::AllocStack
                && !dla_core::labels::names_finding(e, &subjects)
                && dla_core::labels::about_finding(e, &subjects)
        })
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
    {
        let Some(raw) = e_raw(run_dir, rec) else {
            continue;
        };
        let used: Vec<&str> = raw
            .get("captures")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|c| c.get("sha256").and_then(Value::as_str))
            .collect();
        for files in &harness_files {
            for cmd in dla_workload::harness::capture::summarize_commands(files, pointer, false)
                .into_iter()
                .filter(|c| c.starts_with("dla summarize alloc-stack-diff "))
            {
                let hit = cmd
                    .split_whitespace()
                    .skip(3)
                    .filter_map(|p| p.strip_prefix(&base))
                    .filter_map(|rel| digest_of(run_dir, files, rel))
                    .any(|d| used.contains(&d.as_str()));
                if hit {
                    return Some(cmd.replacen(
                        "dla summarize alloc-stack-diff ",
                        &format!("dla summarize alloc-stack-diff --finding {finding} "),
                        1,
                    ));
                }
            }
        }
    }
    None
}

/// The parsed raw file of a record, when it lies inside the run directory.
fn e_raw(run_dir: &Path, rec: &dla_core::evidence::EvidenceRecord) -> Option<Value> {
    let p = contained(run_dir, rec.raw.as_deref()?)?;
    serde_json::from_slice(&fs::read(p).ok()?).ok()
}

/// True when a record of `kind` was made from a file with one of `digests`.
fn made_from(
    run_dir: &Path,
    ledger: &dla_core::Ledger,
    kind: EvidenceKind,
    digests: &[String],
) -> bool {
    ledger
        .evidence()
        .filter(|e| e.kind == kind)
        .filter_map(|e| contained(run_dir, e.raw.as_deref()?))
        .filter_map(|p| serde_json::from_slice::<Value>(&fs::read(p).ok()?).ok())
        .any(|raw| {
            raw.get("captures")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|c| c.get("sha256").and_then(Value::as_str))
                .any(|d| digests.iter().any(|x| x == d))
        })
}

/// The finding and, for a sibling, its root: the findings a record may name.
fn subjects_of<'a>(
    ledger: &'a dla_core::Ledger,
    f: &'a dla_core::Finding,
) -> Vec<&'a dla_core::Finding> {
    let mut v = vec![f];
    if let Some(root) = f.parent.as_ref().and_then(|p| ledger.finding(p)) {
        v.push(root);
    }
    v
}

/// True for a holder entry of the Python holder chains (it lists `sites`) that holds a line of
/// `subjects` or is owned by one of them.
fn entry_is_mine(e: &Value, subjects: &[&dla_core::Finding]) -> bool {
    let held = dla_core::holders::entry_sites(e);
    subjects.iter().any(|f| {
        f.site.as_deref().is_some_and(|s| held.contains(&s.trim()))
            || f.owner
                .as_deref()
                .is_some_and(|o| dla_core::holders::entry_is_owner(e, o))
    })
}

/// Whether a retention record stands in for the stack: a record that is not of the Python holder
/// chains does; one that is does only when a holder entry that holds the finding's line (or is
/// owned by its owner) has a `path` and is not `unreferenced`: an entry of objects that nothing refers
/// to shows the type and no way to the line.
fn retention_lifts(result: &Value, subjects: &[&dla_core::Finding]) -> bool {
    let Some(list) = result.get("holders").and_then(Value::as_array) else {
        return true;
    };
    if !list.iter().any(|e| e.get("sites").is_some()) {
        return true;
    }
    list.iter().any(|e| {
        entry_is_mine(e, subjects)
            && e.get("unreferenced").is_none()
            && e.get("path")
                .or_else(|| e.get("paths"))
                .and_then(Value::as_array)
                .is_some_and(|p| !p.is_empty())
    })
}

/// Cuts a copy of a retention record (its `result` or its raw file) of the Python holder chains to
/// the holder entries that hold a line of `subjects` or are owned by them, before anything is
/// scrubbed or any frame is collected, and says how many were left out (`other_holders`). The
/// lines of the raw file's site list that no remaining entry holds go too. A record that names no
/// entry of the finding is left as it is.
fn focus_retention(v: &mut Value, subjects: &[&dla_core::Finding]) {
    let Some(list) = v.get("holders").and_then(Value::as_array) else {
        return;
    };
    if !list.iter().any(|e| e.get("sites").is_some())
        || !list.iter().any(|e| entry_is_mine(e, subjects))
    {
        return;
    }
    let (mine, other): (Vec<Value>, Vec<Value>) = list
        .iter()
        .cloned()
        .partition(|e| entry_is_mine(e, subjects));
    let mut keep: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for e in &mine {
        for key in ["sites", "minor_sites"] {
            for s in e.get(key).and_then(Value::as_array).into_iter().flatten() {
                let name = s.as_str().or_else(|| s.get("site").and_then(Value::as_str));
                keep.extend(name.map(str::to_owned));
            }
        }
    }
    let kept_total: u64 = mine
        .iter()
        .filter_map(|e| e.get("kept").and_then(Value::as_u64))
        .sum();
    let Some(o) = v.as_object_mut() else { return };
    // The totals are those of the entries left.
    if o.get("kept").is_some_and(Value::is_u64) {
        o.insert("kept".into(), json!(kept_total));
    }
    if o.get("sites").is_some_and(Value::is_u64) {
        o.insert("sites".into(), json!(keep.len()));
    }
    o.insert("holders".into(), Value::Array(mine));
    if !other.is_empty() {
        o.insert("other_holders".into(), json!(other.len()));
    }
    if let Some(Value::Array(sites)) = o.get_mut("sites") {
        sites.retain(|s| {
            s.get("site")
                .and_then(Value::as_str)
                .is_none_or(|n| keep.contains(n))
        });
    }
}

/// Cuts a copy of an allocation-stack record (its `result`, with `top`, or its raw file, with
/// `stacks`) to the stacks whose first project frame is the line of `subjects`, before anything
/// is scrubbed or any frame is collected, and says how many were left out (`other_stacks`). A
/// finding with no line, and a record with no stack at that line, leave the record whole.
fn focus_stacks(v: &mut Value, subjects: &[&dla_core::Finding]) {
    let sites: Vec<&str> = subjects
        .iter()
        .filter_map(|f| f.site.as_deref())
        .map(str::trim)
        .collect();
    let mine = |row: &Value| {
        row.get("site")
            .and_then(Value::as_str)
            .is_some_and(|s| sites.contains(&s.trim()))
    };
    let matches = ["top", "stacks"]
        .iter()
        .filter_map(|k| v.get(*k).and_then(Value::as_array))
        .any(|rows| rows.iter().any(mine));
    if sites.is_empty() || !matches {
        return;
    }
    let Some(o) = v.as_object_mut() else { return };
    let mut left_out = 0usize;
    for key in ["top", "stacks"] {
        if let Some(Value::Array(rows)) = o.get_mut(key) {
            let before = rows.len();
            rows.retain(|r| mine(r));
            left_out = left_out.max(before - rows.len());
        }
    }
    if left_out > 0 {
        o.insert("other_stacks".into(), json!(left_out));
    }
}

/// The check of a bundle with no stack: `Err(stacks-not-captured)` when the growth is a count of
/// blocks at one line, the bundle would hold no allocation or retention evidence, the runtime
/// captures stacks and either no capture run of a harness that grew the column is recorded or
/// the newest one was never summarized; the sentence to carry when a capture was summarized and
/// no stack names the finding; `None` when the check does not apply.
fn stacks_check(
    tc: &ToolCtx,
    id: &FindingId,
    ids: &[EvidenceId],
) -> Result<Option<String>, Failure> {
    use dla_core::{FindingState, finding::Polarity};
    let Ok(view) = dla_core::RunView::load_for(&tc.run_dir, tc.worker.as_deref()) else {
        return Ok(None);
    };
    let Some(f) = view.ledger.finding(id) else {
        return Ok(None);
    };
    if f.polarity() != Polarity::Leak
        || !matches!(f.state, FindingState::Confirmed | FindingState::Strong)
        || ids.iter().any(|e| {
            view.ledger.evidence_by_id(e).is_some_and(|r| {
                match r.kind.class() {
                    EvidenceClass::Allocation => true,
                    // A retention record shows the verifier the line and the call into an
                    // extension only when the holder of the finding's line has a chain.
                    EvidenceClass::Retention => {
                        retention_lifts(&r.result, &subjects_of(&view.ledger, f))
                    }
                    _ => false,
                }
            })
        })
    {
        return Ok(None);
    }
    // The harnesses that grew the column, in series order, whose runtime captures stacks.
    let harnesses: Vec<(String, dla_core::HarnessManifest)> =
        dla_core::grown::site_column_harnesses(&view, f)
            .into_iter()
            .filter_map(|h| {
                let m = dla_core::HarnessManifest::load(&tc.run_dir, &h).ok()?;
                dla_workload::harness::capture::supported(m.runtime)
                    .alloc
                    .then_some((h, m))
            })
            .collect();
    if harnesses.is_empty() {
        return Ok(None);
    }
    let captured: Vec<_> = harnesses
        .iter()
        .filter_map(|(h, _)| alloc_captures(&tc.run_dir, h).map(|(files, at)| (h, files, at)))
        .collect();
    // Per capture: the summarize commands of it that were not run, and how many it has.
    let pending: Vec<_> = captured
        .iter()
        .map(|(h, files, at)| (h, unrun_commands(tc, &view.ledger, files), *at))
        .collect();
    if let Some((h, _, _)) = pending
        .iter()
        .find(|(_, (unrun, total), _)| unrun.is_empty() && *total > 0)
    {
        return Ok(Some(format!(
            "an allocation capture of {h} was summarized and no growing stack ends at the finding's line, so the bundle holds no stack and no excerpt"
        )));
    }
    // Captured and not summarized: every command of the newest capture that was not run is the
    // fix.
    if let Some((h, (cmds, total), _)) = pending
        .iter()
        .filter(|(_, (unrun, _), _)| !unrun.is_empty())
        .max_by_key(|(_, _, at)| *at)
    {
        let what = if cmds.len() == *total {
            "was not summarized"
        } else {
            "was only partly summarized"
        };
        return Err(Failure::refuse(
            core_codes::STACKS_NOT_CAPTURED,
            format!(
                "an allocation capture of {h} exists and {what}, so no stack is tied to {id}; the bundle would give the verifier no stack and no source excerpt"
            ),
            format!(
                "{} made from the capture of {h}",
                if *total > 1 {
                    "an alloc-stack record and a retention record"
                } else {
                    "an alloc-stack record"
                }
            ),
            format!("{} ; then dla bundle {id} again", cmds.join(" ; ")),
        ));
    }
    // A hook driver (entry-path, probe or batch) first, else the test loop, else the first.
    let (h, m) = harnesses
        .iter()
        .find(|(_, m)| m.is_hook_driver())
        .or_else(|| harnesses.iter().find(|(_, m)| m.is_test_loop()))
        .unwrap_or(&harnesses[0]);
    let mode = crate::alloc_cmd::key_mode(m).as_str();
    let commands =
        crate::alloc_cmd::stacks_commands(m.runtime, h, mode, &id.to_string(), "the line");
    let loop_note = if m.is_test_loop() {
        format!(
            " A capture slows a test loop; a probe driver is the cheaper route: dla harness gen --lang {} --driver --probe {id}, then run it the same way.",
            m.runtime.as_str()
        )
    } else {
        String::new()
    };
    let then = if dla_workload::harness::capture::heap_is_holder_chains(m.runtime) {
        format!(
            "the two dla summarize commands that run prints (alloc-stack-diff ties the stacks at the line to {id}, heap-graph names who holds the objects of the line)"
        )
    } else {
        format!(
            "the dla summarize alloc-stack-diff command that run prints (it ties the stacks at the line to {id})"
        )
    };
    Err(Failure::refuse(
        core_codes::STACKS_NOT_CAPTURED,
        format!(
            "the bundle of {id} would give the verifier no stack and no source excerpt: its growth is a count of blocks allocated at one line, and the stacks that lead to the line were not captured"
        ),
        format!("an allocation capture of {h} (harness/{h}/captures.json)"),
        format!(
            "{} ; then {then} ; then dla bundle {id} again.{loop_note}",
            commands[0]
        ),
    ))
}

type Built = (String, usize, usize, Vec<String>, Vec<String>);

/// The result of writing one file of the bundle.
type Written = Result<(), Failure>;

/// At most this many entry-path harnesses go into a bundle.
const MAX_ENTRY_HARNESSES: usize = 2;
/// Largest driver file copied for an entry-path harness.
const MAX_DRIVER_BYTES: u64 = 256 * 1024;

/// Removes the generated target block from a driver: the run of comment lines that starts at
/// the line that names the harness (`Entry-path harness H5:` or `Targets of harness H5:`). It
/// names the finding, its site, the rule that raised it and its writers, which the verifier
/// must find from the evidence.
fn without_target_block(text: &str) -> String {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let comment = |l: &str| {
        let t = l.trim_start();
        t.starts_with('#') || t.starts_with("//") || t.starts_with("/*")
    };
    let Some(start) = lines.iter().position(|l| {
        comment(l) && (l.contains("Entry-path harness H") || l.contains("Targets of harness H"))
    }) else {
        return text.to_owned();
    };
    let mut end = start;
    while end < lines.len() && comment(lines[end]) && !lines[end].contains("DLA-HOOK") {
        end += 1;
    }
    let first = lines[start];
    let pad = &first[..first.len() - first.trim_start().len()];
    let marker = if first.trim_start().starts_with('#') {
        "#"
    } else {
        "//"
    };
    let mut out: String = lines[..start].concat();
    out.push_str(&format!(
        "{pad}{marker} (the generated target block is not part of the bundle)\n"
    ));
    out.push_str(&lines[end..].concat());
    out
}

/// The driver, `VARIES.txt` and trend verdict of the entry-path harnesses that targeted the
/// finding (the one in which it grew, else up to two), copied into `entry-path/<H>/`. The
/// verifier judges the trigger from them: whether the unit calls public entry points only and
/// whether what it varies is under a caller's control. The finding's id, site and writers are
/// scrubbed from the copies like those of every other record.
fn entry_path_files(
    tc: &ToolCtx,
    id: &FindingId,
    needles: &Needles,
    write: &dyn Fn(&str, &[u8]) -> Written,
) -> Result<Vec<Value>, Failure> {
    let Ok(view) = dla_core::RunView::load(&tc.run_dir) else {
        return Ok(Vec::new());
    };
    let triggers = dla_core::trigger::Triggers::of(&view);
    let mut runs = triggers.entry_runs(id);
    let mut seen = std::collections::BTreeSet::new();
    runs.retain(|r| seen.insert(r.harness.clone()));
    runs.truncate(MAX_ENTRY_HARNESSES);
    let mut out = Vec::new();
    for r in runs {
        let driver = dla_core::HarnessManifest::dir_in(&tc.run_dir, &r.harness).join("driver");
        let mut listing = Vec::new();
        let mut names: Vec<(String, PathBuf)> = Vec::new();
        files_under(&driver, &driver, &mut names).map_err(|e| io_fail(&driver, e))?;
        names.sort();
        for (rel, path) in names {
            if fs::metadata(&path).is_ok_and(|m| m.len() > MAX_DRIVER_BYTES) {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else {
                continue;
            };
            let cleaned = without_target_block(&text);
            let kept = bundle_bytes(cleaned.as_bytes(), false, needles, false);
            let name = format!("entry-path/{}/{rel}", r.harness);
            write(&name, &kept)?;
            listing.push(name);
        }
        let verdict = json!({
            "harness": r.harness,
            "series": r.series,
            "driver_digest": r.digest,
            "varies": r.varies,
            "verdict": r.verdict.map(|v| v.as_str()),
            "meaning": "grew: the holder grew when this driver ran; constant: the holder was measured and stayed flat; any other verdict: nothing measured the holder in this run, which is no evidence either way",
        });
        let vname = format!("entry-path/{}/verdict.json", r.harness);
        let mut vv = verdict.clone();
        scrub_ties(&mut vv, needles, false);
        write(&vname, to_pretty_json(&vv).as_bytes())?;
        listing.push(vname);
        let mut entry = json!({
            "harness": r.harness,
            "verdict": r.verdict.map(|v| v.as_str()),
            "varies": r.varies,
            "files": listing,
        });
        scrub_ties(&mut entry, needles, false);
        out.push(entry);
    }
    Ok(out)
}

/// What a harness is, in plain words: the project's own tests in a loop, a driver made for the
/// finding or for other targets, or a command of the project.
fn harness_words(
    m: &dla_core::HarnessManifest,
    about: &dla_core::grown::HarnessAbout,
) -> &'static str {
    use dla_core::grown::HarnessAbout;
    if m.is_test_loop() {
        "the project's own tests in a loop"
    } else if m.mode == dla_core::harness::HarnessMode::Driver {
        if matches!(
            about,
            HarnessAbout::Target | HarnessAbout::EntryPath | HarnessAbout::Probe
        ) {
            "a driver made for this finding"
        } else if m.targets.is_empty() && m.probes.is_empty() {
            "a driver with no target"
        } else {
            "a driver for other targets"
        }
    } else {
        "a command of the project"
    }
}

/// The `harnesses` of `index.json`: the harnesses the evidence of the bundle comes from, each
/// with its id, what it is in plain words, the units it runs per round and its key modes. It
/// names no finding and no site: the verifier learns from it whether the growth was reproduced
/// by a driver made for the line or only by a loop over the project's tests.
fn harness_list(tc: &ToolCtx, id: &FindingId) -> Vec<Value> {
    let Ok(view) = dla_core::RunView::load_for(&tc.run_dir, tc.worker.as_deref()) else {
        return Vec::new();
    };
    dla_core::grown::harnesses_about(&view, id)
        .into_iter()
        .map(|(m, about)| {
            json!({
                "id": m.id,
                "is": harness_words(&m, &about),
                "units_per_round": m.step,
                "key_modes": m.key_modes.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
            })
        })
        .collect()
}

/// At most this many test-loop harnesses go into a bundle.
const MAX_TEST_LOOP_HARNESSES: usize = 2;

/// `test-loop/<H>/tests.json` for each test-loop harness that targeted the finding (a pytest
/// loop declares none: it counts when the finding grew in it) and in whose trend it grew. With the attribution `dla alloc-test <H> --finding <F>` stored, it names the
/// test classes that fed the holder (with the entries each added per run) and the classes that
/// ran and added nothing; without it, the classes that ran, and says which of them feeds the
/// holder was not measured. A finding with no such harness gets nothing.
fn test_loop_files(
    tc: &ToolCtx,
    id: &FindingId,
    needles: &Needles,
    write: &dyn Fn(&str, &[u8]) -> Written,
) -> Result<Vec<Value>, Failure> {
    let Ok(view) = dla_core::RunView::load(&tc.run_dir) else {
        return Ok(Vec::new());
    };
    let triggers = dla_core::trigger::Triggers::of(&view);
    let mut out = Vec::new();
    for h in triggers
        .test_loop_harnesses(id)
        .into_iter()
        .take(MAX_TEST_LOOP_HARNESSES)
    {
        // The pytest loop: the test files it ran, no per-file figures (nothing measures which
        // file feeds a holder).
        if dla_core::HarnessManifest::load(&tc.run_dir, &h).is_ok_and(|m| m.is_pytest_loop()) {
            let (ran, dropped) =
                dla_core::testloop::pytest_files_of(&tc.run_dir, &h).unwrap_or_default();
            let mut doc = json!({
                "harness": h,
                "test_files_run": ran,
                "note": if ran.is_empty() {
                    "these tests ran in a loop (pytest, one process) while the holder grew; the files that ran were not recorded, and which of them feeds the holder was not measured"
                } else {
                    "these test files ran in a loop (pytest, one process) while the holder grew; which of them feeds the holder was not measured"
                },
            });
            if !dropped.is_empty() {
                doc["dropped_files"] = json!(dropped);
            }
            scrub_ties(&mut doc, needles, false);
            let name = format!("test-loop/{h}/tests.json");
            write(&name, to_pretty_json(&doc).as_bytes())?;
            out.push(json!({"harness": h, "files": [name]}));
            continue;
        }
        let files = dla_core::testloop::classes_of(&tc.run_dir, &h);
        let with_file = |class: &str| -> Value {
            let mut v = json!({"class": class});
            if let Some((_, f)) = files.iter().find(|(c, f)| c == class && !f.is_empty()) {
                v["file"] = json!(f);
            }
            v
        };
        let mut doc = match dla_core::testloop::read_attribution(&tc.run_dir, &h, id) {
            Some(a) => {
                let fed: Vec<Value> = a
                    .fed_by()
                    .iter()
                    .map(|f| {
                        let mut v = with_file(&f.class);
                        v["per_run"] = json!((f.per_run * 10.0).round() / 10.0);
                        v
                    })
                    .collect();
                let flat: Vec<Value> = a.ran_flat().iter().map(|c| with_file(&c.class)).collect();
                let stepped: Vec<Value> = a
                    .stepped()
                    .iter()
                    .map(|c| {
                        let mut v = with_file(&c.class);
                        v["entries"] = json!(c.later_runs);
                        v["in_runs"] = json!(c.grew_in);
                        v
                    })
                    .collect();
                let mut doc = json!({
                    "harness": h,
                    "holder": a.probe,
                    "later_runs": a.later_runs,
                    "fed_by": fed,
                    "ran_and_added_nothing": flat,
                    "note": "the figures were measured per test class while the tests ran in a loop: the entries a class added to the holder, per run of the tests after the first",
                });
                if !stepped.is_empty() {
                    // A late one-time fill, not a feed: entries added in a few runs only.
                    doc["added_in_a_few_runs_only"] = Value::Array(stepped);
                }
                if a.parallel {
                    doc["parallel"] =
                        json!("the tests ran in parallel: the figures of the classes overlap");
                }
                doc
            }
            None => json!({
                "harness": h,
                "classes_run": files
                    .iter()
                    .map(|(c, _)| with_file(c))
                    .collect::<Vec<_>>(),
                "note": "these test classes ran in a loop while the holder grew; which of them feeds the holder was not measured",
            }),
        };
        scrub_ties(&mut doc, needles, false);
        let name = format!("test-loop/{h}/tests.json");
        write(&name, to_pretty_json(&doc).as_bytes())?;
        out.push(json!({"harness": h, "files": [name]}));
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn build(
    tc: &ToolCtx,
    ledger: &dla_core::Ledger,
    dir: &Path,
    project: &Path,
    id: &FindingId,
    owner: Option<&str>,
    ids: &[EvidenceId],
    stacks_note: Option<&str>,
) -> Result<Built, Failure> {
    let write = |name: &str, bytes: &[u8]| -> Result<(), Failure> {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| io_fail(parent, e))?;
        }
        write_atomic(&path, bytes).map_err(Failure::from)
    };
    // What no bundled record may show: the finding's id (with and without the worker
    // prefix), its site and its recorded writers.
    let mut needles = Needles {
        ids: vec![id.to_string(), id.reprefixed(None).to_string()],
        sites: Vec::new(),
    };
    if let Some(f) = ledger.finding(id) {
        needles.sites.extend(f.site.clone());
    }
    needles.sites.extend(ledger.recorded_sites(id).0);
    for list in [&mut needles.ids, &mut needles.sites] {
        list.sort_by_key(|s| std::cmp::Reverse(s.len()));
        list.dedup();
    }
    let mut copied = 0usize;
    let mut omitted: Vec<String> = Vec::new();
    let mut records: Vec<Value> = Vec::new();
    // Frames the evidence names, each with the class rank of its record.
    let mut named: Vec<(u8, Vec<(String, usize)>)> = Vec::new();
    for eid in ids {
        let Some(e) = ledger.evidence_by_id(eid) else {
            continue;
        };
        let mut result = e.result.clone();
        let subjects = ledger
            .finding(id)
            .map(|f| subjects_of(ledger, f))
            .unwrap_or_default();
        if e.kind == EvidenceKind::RetentionPath {
            focus_retention(&mut result, &subjects);
        }
        if e.kind == EvidenceKind::AllocStack {
            focus_stacks(&mut result, &subjects);
        }
        strip_forbidden(&mut result);
        // Allocation and retention records keep the site in their structured frame fields.
        let frames_exempt = matches!(
            e.kind.class(),
            EvidenceClass::Allocation | EvidenceClass::Retention
        );
        scrub_ties(&mut result, &needles, frames_exempt);
        let mut here: Vec<(String, usize)> = Vec::new();
        frames_in_value(&result, &mut here);
        let mut rec = Map::new();
        rec.insert("id".into(), json!(e.evidence_id.to_string()));
        rec.insert("kind".into(), json!(e.kind.as_str()));
        rec.insert("authority".into(), json!(e.authority.as_str()));
        rec.insert("result".into(), result);
        if let Some(raw) = &e.raw {
            let contained_src = contained(&tc.run_dir, raw);
            let safe = contained_src.is_some();
            let src = contained_src.unwrap_or_default();
            let name = format!(
                "evidence/{}-{}",
                e.evidence_id.to_string().replace('/', "_"),
                raw.rsplit('/').next().unwrap_or("raw")
            );
            if !safe || !src.is_file() || !is_written(&src) {
                omitted.push(if safe {
                    format!("{eid}: {raw} is missing")
                } else {
                    format!("{eid}: {raw} is not a file inside the run directory")
                });
            } else if fs::metadata(&src).is_ok_and(|m| m.len() > MAX_COPY_BYTES) {
                omitted.push(format!("{eid}: {raw} is over {MAX_COPY_BYTES} bytes"));
            } else {
                let mut bytes = fs::read(&src).map_err(|e| io_fail(&src, e))?;
                if e.kind == EvidenceKind::RetentionPath
                    && let Ok(mut doc) = serde_json::from_slice::<Value>(&bytes)
                {
                    focus_retention(&mut doc, &subjects);
                    bytes = to_pretty_json(&doc).into_bytes();
                } else if e.kind == EvidenceKind::AllocStack
                    && let Ok(mut doc) = serde_json::from_slice::<Value>(&bytes)
                {
                    focus_stacks(&mut doc, &subjects);
                    bytes = to_pretty_json(&doc).into_bytes();
                }
                let kept = bundle_bytes(&bytes, raw.ends_with(".json"), &needles, frames_exempt);
                frames_in_bytes(&kept, &mut here);
                write(&name, &kept)?;
                rec.insert("file".into(), json!(name));
                copied += 1;
            }
        }
        records.push(Value::Object(rec));
        // Growth and discriminator frames are the tied site: no excerpt from them.
        let rank = match e.kind.class() {
            EvidenceClass::Allocation => 0,
            EvidenceClass::Retention => 1,
            _ => continue,
        };
        named.push((rank, here));
    }
    named.sort_by_key(|(rank, _)| *rank);
    let candidates: Vec<(String, usize)> = named.into_iter().flat_map(|(_, f)| f).collect();
    let (texts, frames) = excerpts_for(project, &candidates);
    if !texts.is_empty() {
        write("excerpt.txt", texts.join("\n").as_bytes())?;
    }
    let mut index = json!({
        "excerpt": (!frames.is_empty()).then_some("excerpt.txt"),
        "frames": frames,
        "evidence": records,
    });
    let entry = entry_path_files(tc, id, &needles, &write)?;
    if !entry.is_empty() {
        index["entry_path"] = Value::Array(entry);
    }
    let tests = test_loop_files(tc, id, &needles, &write)?;
    if !tests.is_empty() {
        index["test_loop"] = Value::Array(tests);
    }
    let behind = harness_list(tc, id);
    if !behind.is_empty() {
        index["harnesses"] = Value::Array(behind);
    }
    if let Some(o) = owner {
        index["owner"] = json!(o);
    }
    if let Some(n) = stacks_note {
        index["stacks_note"] = json!(n);
    }
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
        frames,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding_at(owner: Option<&str>, site: Option<&str>) -> dla_core::Finding {
        let mut f = dla_core::Finding::candidate(
            FindingId::new(1),
            dla_core::Origin::parse("growth-series").unwrap(),
            vec![EvidenceId::new(1)],
        );
        f.owner = owner.map(str::to_owned);
        f.site = site.map(str::to_owned);
        f
    }

    /// A record of two holders: a module global with a chain that keeps lines a.py:1 and a.py:2,
    /// and the native entry of a.py:9, which has no chain.
    fn two_holders() -> Value {
        json!({"file": "x", "sites": [
            {"site": "a.py:1"}, {"site": "a.py:2"}, {"site": "a.py:9"}, {"site": "a.py:7"}],
            "holders": [
            {"key": "static:m.g", "kept": 5,
             "sites": [{"site": "a.py:1", "first": 1, "last": 9}, {"site": "a.py:2", "first": 1, "last": 9}],
             "minor_sites": [{"site": "a.py:7"}],
             "paths": [{"site": "a.py:1", "count": 5, "path": [{"type": "list", "at": "a.py:3"}]}],
             "path": [{"type": "list", "at": "a.py:3"}]},
            {"key": "a.py:9", "kept": 0, "unreferenced": "x",
             "sites": [{"site": "a.py:9", "first": 1, "last": 9}]}]})
    }

    #[test]
    fn a_retention_record_lifts_the_stack_only_when_the_holder_of_the_line_has_a_chain() {
        let rec = two_holders();
        let native = finding_at(None, Some("a.py:9"));
        let global = finding_at(Some("m.g"), Some("a.py:1"));
        assert!(
            !retention_lifts(&rec, &[&native]),
            "an unreferenced entry has no path"
        );
        // An entry of objects nothing refers to has the type as a one-step path and still no way
        // to the line.
        let with_path = json!({"holders": [{"key": "a.py:9", "unreferenced": "x",
            "sites": [{"site": "a.py:9"}], "path": [{"type": "list"}]}]});
        assert!(!retention_lifts(&with_path, &[&native]));
        assert!(retention_lifts(&rec, &[&global]));
        assert!(retention_lifts(&rec, &[&finding_at(None, Some("a.py:2"))]));
        // Another adapter's record (no `sites` in its entries) counts as before.
        assert!(retention_lifts(
            &json!({"holders": [{"key": "static:a.B.c"}]}),
            &[&native]
        ));
        // A record that names no entry of the finding does not lift the requirement.
        assert!(!retention_lifts(
            &rec,
            &[&finding_at(None, Some("a.py:40"))]
        ));
    }

    #[test]
    fn each_finding_sees_only_its_own_holder_of_a_record_with_two() {
        let native = finding_at(None, Some("a.py:9"));
        let mut v = two_holders();
        focus_retention(&mut v, &[&native]);
        let keys: Vec<&str> = v["holders"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["key"].as_str().unwrap())
            .collect();
        assert_eq!(keys, ["a.py:9"]);
        assert_eq!(v["other_holders"], 1);
        let sites: Vec<&str> = v["sites"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["site"].as_str().unwrap())
            .collect();
        assert_eq!(sites, ["a.py:9"], "no line of the other holder is left");
        assert!(!v.to_string().contains("a.py:3"), "{v}");
        let mut v = two_holders();
        focus_retention(&mut v, &[&finding_at(Some("m.g"), Some("a.py:1"))]);
        assert_eq!(v["holders"].as_array().unwrap().len(), 1);
        assert_eq!(v["holders"][0]["key"], "static:m.g");
        let sites: Vec<&str> = v["sites"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["site"].as_str().unwrap())
            .collect();
        assert_eq!(sites, ["a.py:1", "a.py:2", "a.py:7"]);
        assert!(!v.to_string().contains("a.py:9"), "{v}");
        // A record that names no entry of the finding, or an entry of no chains, is left alone.
        let mut v = two_holders();
        focus_retention(&mut v, &[&finding_at(None, Some("a.py:40"))]);
        assert_eq!(v, two_holders());
        let mut v = json!({"holders": [{"key": "static:a.B.c"}]});
        focus_retention(&mut v, &[&native]);
        assert_eq!(v, json!({"holders": [{"key": "static:a.B.c"}]}));
    }

    #[test]
    fn each_finding_sees_only_its_own_stacks_of_a_record_with_stacks_of_two_lines() {
        let rec = json!({"top": [
            {"at": "m", "site": "a.py:15", "d_blocks": 880},
            {"at": "m", "site": "b.py:6", "d_blocks": 880},
            {"at": "m", "site": "a.py:15", "d_blocks": 2},
            {"at": "d"}],
            "stacks": [
            {"site": "a.py:15", "frames": [{"function": "f", "file": "a.py", "line": 15}]},
            {"site": "b.py:6", "frames": [{"function": "g", "file": "b.py", "line": 6}]}]});
        let a = finding_at(None, Some("a.py:15"));
        let b = finding_at(None, Some(" b.py:6 "));
        let mut v = rec.clone();
        focus_stacks(&mut v, &[&a]);
        assert_eq!(v["top"].as_array().unwrap().len(), 2);
        assert_eq!(v["stacks"].as_array().unwrap().len(), 1);
        assert_eq!(v["other_stacks"], 2);
        let mut frames = Vec::new();
        frames_in_value(&v, &mut frames);
        assert!(frames.iter().all(|(f, _)| f == "a.py"), "{frames:?}");
        assert!(!v.to_string().contains("b.py"), "{v}");
        let mut v = rec.clone();
        focus_stacks(&mut v, &[&b]);
        assert_eq!(v["top"].as_array().unwrap().len(), 1);
        assert!(!v.to_string().contains("a.py"), "{v}");
        // No stack at the finding's line, or no line: the record stays whole.
        let mut v = rec.clone();
        focus_stacks(&mut v, &[&finding_at(None, Some("c.py:1"))]);
        assert_eq!(v, rec);
        focus_stacks(&mut v, &[&finding_at(Some("m.g"), None)]);
        assert_eq!(v, rec);
    }

    #[test]
    fn the_totals_of_a_focused_retention_record_are_those_of_the_entries_left() {
        let mut v = two_holders();
        v["kept"] = json!(5);
        v["sites"] = json!(4);
        focus_retention(&mut v, &[&finding_at(None, Some("a.py:9"))]);
        assert_eq!(v["kept"], 0);
        assert_eq!(v["sites"], 1);
        let mut v = two_holders();
        v["kept"] = json!(5);
        v["sites"] = json!(4);
        focus_retention(&mut v, &[&finding_at(Some("m.g"), Some("a.py:1"))]);
        assert_eq!(v["kept"], 5);
        assert_eq!(v["sites"], 3, "two lines kept and one minor");
    }

    #[test]
    fn a_harness_is_told_in_plain_words_without_a_finding_or_a_site() {
        use dla_core::grown::HarnessAbout;
        let m = |mode: &str, tools: &[&str]| -> dla_core::HarnessManifest {
            serde_json::from_value(json!({
                "id": "H3", "runtime": "python", "mode": mode, "scratch": "x",
                "run": {"argv": ["x"], "cwd": "."}, "warmup": 1, "iterations": 8, "step": 50,
                "key_modes": ["fresh", "repeated"], "agent": "driver-python", "timeout_secs": 60,
                "tools": tools,
            }))
            .unwrap()
        };
        let probe = HarnessAbout::Probe;
        let column = HarnessAbout::Column("py.blocks@a.py:1".into());
        assert_eq!(
            harness_words(&m("driver", &[]), &probe),
            "a driver made for this finding"
        );
        assert_eq!(
            harness_words(&m("driver", &[]), &HarnessAbout::Target),
            "a driver made for this finding"
        );
        assert_eq!(
            harness_words(&m("driver", &[]), &column),
            "a driver with no target"
        );
        let mut other = m("driver", &[]);
        other.targets = vec!["F9".parse().unwrap()];
        assert_eq!(harness_words(&other, &column), "a driver for other targets");
        assert_eq!(
            harness_words(&m("driver", &["pytest-loop"]), &column),
            "the project's own tests in a loop"
        );
        assert_eq!(
            harness_words(&m("command", &[]), &HarnessAbout::OwnWorkload),
            "a command of the project"
        );
    }

    #[test]
    fn the_newest_two_records_of_each_kind_that_name_the_finding_are_taken() {
        use dla_core::{
            Ledger, NewEvidence, NewFinding, Origin,
            evidence::{Authority, EvidenceCoverage, Source, Tool},
            state::{RunState, now_utc},
        };
        let dir = tempfile::tempdir().unwrap();
        RunState::init("/p", None, 3600, None, now_utc())
            .unwrap()
            .save(dir.path())
            .unwrap();
        let mut l = Ledger::create(dir.path()).unwrap();
        let mut add = |kind: EvidenceKind, authority: Authority, result: Value| {
            l.add_evidence(NewEvidence {
                source: Source::Repo {
                    commit: "c".into(),
                    path: None,
                },
                tool: Tool {
                    name: "t".into(),
                    version: "1".into(),
                },
                authority,
                kind,
                result,
                raw: None,
                coverage: EvidenceCoverage::default(),
                limitations: vec![],
                unknowns: vec![],
            })
            .unwrap();
        };
        add(EvidenceKind::GrowthSeries, Authority::Test, json!({}));
        let named = || json!({"finding": "F1"});
        // E2..E4 name F1, E5 names another finding, E6 and E7 are retention paths naming F1, E8
        // is a static stack record that names it.
        for _ in 0..3 {
            add(EvidenceKind::AllocStack, Authority::Test, named());
        }
        add(
            EvidenceKind::AllocStack,
            Authority::Test,
            json!({"finding": "F2"}),
        );
        add(EvidenceKind::RetentionPath, Authority::Test, named());
        add(EvidenceKind::RetentionPath, Authority::Test, named());
        add(EvidenceKind::AllocStack, Authority::Static, named());
        let mut nf = NewFinding::new(
            Origin::parse("census").unwrap(),
            vec!["E1".parse().unwrap()],
        );
        nf.site = Some("a.py:1".into());
        l.add_finding(nf).unwrap();
        let f = l.finding(&"F1".parse().unwrap()).unwrap();
        let got: Vec<String> = naming_records(&l, f)
            .iter()
            .map(ToString::to_string)
            .collect();
        assert_eq!(got, ["E4", "E3", "E7", "E6"]);
    }

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
    fn tie_keys_and_site_bearing_names_are_scrubbed() {
        let mut v = json!({
            "passed": [{"column": "alloc@src/a.js:7", "site": "src/a.js:7", "finding": "F1",
                        "tie": {"rule": "site", "key": "src/a.js:7"}}],
            "values": {"alloc@src/a.js:7": [1, 2], "alloc@src/b.js:9": [3]},
            "needed": "tie src/a.js:7 to jvm/F1",
            "alloc": {"site": "kept.js:3"},
        });
        scrub_ties(
            &mut v,
            &Needles {
                ids: vec!["jvm/F1".to_owned()],
                sites: vec!["src/a.js:7".to_owned()],
            },
            false,
        );
        let t = v.to_string();
        assert!(
            !t.contains("a.js") && !t.contains("jvm/F1") && !t.contains('@'),
            "{t}"
        );
        assert_eq!(v["values"].as_object().unwrap().len(), 2, "{t}");
        assert_eq!(v["passed"], json!([{"column": "alloc"}]));
        // A site key outside a column object is allocation evidence and stays.
        assert_eq!(v["alloc"]["site"], "kept.js:3");
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

    fn frames(v: Value) -> Vec<String> {
        let mut out = Vec::new();
        frames_in_value(&v, &mut out);
        out.into_iter().map(|(f, l)| format!("{f}:{l}")).collect()
    }

    #[test]
    fn frames_come_from_structured_pairs_and_tokens() {
        let got = frames(json!({
            "frames": [
                {"function": "grow", "file": "src/a.rs", "line": 7},
                ["pkg/b.py", 12],
                "com.acme.Cache.put(Cache.java:44)",
                "at /srv/app/c.js:9:13",
            ],
            "alloc": "grow (d.go:2)",
            "sites": {"e.c:5": 3},
            "time": "12:30:15",
            "not_a_file": "key:5",
        }));
        for want in [
            "src/a.rs:7",
            "pkg/b.py:12",
            "Cache.java:44",
            "/srv/app/c.js:9",
            "d.go:2",
            "e.c:5",
        ] {
            assert!(got.iter().any(|g| g == want), "{want} in {got:?}");
        }
        assert!(
            !got.iter()
                .any(|g| g.contains("12:30") || g.contains("key:"))
        );
    }

    #[test]
    fn excerpts_are_capped_deduped_and_stay_inside_the_project() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join("src/a")).unwrap();
        fs::create_dir_all(root.join("src/b")).unwrap();
        fs::create_dir_all(root.join("node_modules/x")).unwrap();
        let body: String = (1..=100).map(|n| format!("line {n}\n")).collect();
        for f in [
            "src/a/Cache.java",
            "src/a/Only.java",
            "src/b/Cache.java",
            "node_modules/x/lib.js",
        ] {
            fs::write(root.join(f), &body).unwrap();
        }
        for n in 1..=5 {
            fs::write(root.join(format!("m{n}.py")), &body).unwrap();
        }
        let cands = |v: &[(&str, usize)]| -> Vec<(String, usize)> {
            v.iter().map(|(f, l)| ((*f).to_owned(), *l)).collect()
        };
        // A bare name that two files share is skipped; a unique one resolves.
        let (texts, got) = excerpts_for(
            root,
            &cands(&[
                ("Cache.java", 10),
                ("Only.java", 20),
                ("a/Cache.java", 30),
                ("node_modules/x/lib.js", 1),
                ("lib.js", 1),
                ("../outside.py", 1),
                ("missing.py", 1),
            ]),
        );
        assert_eq!(got, ["src/a/Only.java:20", "src/a/Cache.java:30"]);
        assert_eq!(texts.len(), 2);
        assert!(
            texts[0].starts_with("src/a/Only.java:20 (lines 1-40)"),
            "{}",
            texts[0]
        );
        // An absolute path inside the project resolves; the cap is four; repeats count once.
        let abs = format!("{}/m1.py", root.display());
        let (_, got) = excerpts_for(
            root,
            &cands(&[
                (&abs, 5),
                ("m1.py", 5),
                ("m2.py", 5),
                ("m3.py", 5),
                ("m4.py", 5),
                ("m5.py", 5),
            ]),
        );
        assert_eq!(got, ["m1.py:5", "m2.py:5", "m3.py:5", "m4.py:5"]);
        let (texts, got) = excerpts_for(root, &[]);
        assert!(texts.is_empty() && got.is_empty());
    }

    fn needles() -> Needles {
        Needles {
            ids: vec!["jvm/F3".to_owned(), "F3".to_owned()],
            sites: vec![
                "src/Cache.java:41".to_owned(),
                "src/Cache.java:57".to_owned(),
            ],
        }
    }

    #[test]
    fn every_record_loses_ids_sites_and_the_drop_keys() {
        let mut disc = json!({
            "test": "plateau", "finding": "jvm/F3", "basis": "site src/Cache.java:41",
            "limitations": ["F3 grows at src/Cache.java:41", "jvm/F3.n1 unchanged"],
            "supports": ["leak"],
        });
        let mut hist = json!({
            "ties": [{"stack": "x", "finding": "F3"}], "candidates": ["F3"],
            "top": [{"class": "Item", "site": "src/Cache.java:57"}],
        });
        for v in [&mut disc, &mut hist] {
            strip_forbidden(v);
            scrub_ties(v, &needles(), false);
            let t = v.to_string();
            for bad in [
                "F3",
                "Cache.java:41",
                "Cache.java:57",
                "basis",
                "candidates",
            ] {
                assert!(!t.contains(bad), "{bad} in {t}");
            }
        }
        // A bigger number is not the id.
        let mut v = json!({"note": "F31 and AF3 and F3."});
        scrub_ties(&mut v, &needles(), false);
        assert_eq!(v["note"], "F31 and AF3 and <redacted>.");
    }

    #[test]
    fn frame_fields_of_allocation_and_retention_keep_the_site() {
        let mut alloc = json!({
            "finding": "F3", "candidates": ["F3"], "ties": [{"stack": "s"}],
            "top": [{"at": "put", "site": "src/Cache.java:41", "root": "static"}],
            "stacks": [{"label": "L", "site": "src/Cache.java:41",
                "frames": [{"function": "put", "file": "src/Cache.java", "line": 41}],
                "root_path": ["static src/Cache.java:41"]}],
            "needed": "tie src/Cache.java:41 to F3",
        });
        scrub_ties(&mut alloc, &needles(), true);
        let t = alloc.to_string();
        assert!(
            !t.contains("F3") && !t.contains("candidates") && !t.contains("ties"),
            "{t}"
        );
        assert_eq!(alloc["top"][0]["site"], "<redacted>");
        assert_eq!(alloc["needed"], "tie <redacted> to <redacted>");
        assert_eq!(alloc["stacks"][0]["site"], "<redacted>");
        assert_eq!(alloc["stacks"][0]["frames"][0]["line"], 41);
        assert_eq!(alloc["stacks"][0]["frames"][0]["file"], "src/Cache.java");
        assert_eq!(
            alloc["stacks"][0]["root_path"][0],
            "static src/Cache.java:41"
        );
    }

    #[test]
    fn raw_text_copies_lose_the_finding_id() {
        let raw = b"F3 leaks at src/Cache.java:41\n";
        let plain = bundle_bytes(raw, false, &needles(), false);
        assert_eq!(
            String::from_utf8(plain).unwrap(),
            "<redacted> leaks at <redacted>\n"
        );
        let frames = bundle_bytes(raw, false, &needles(), true);
        assert_eq!(
            String::from_utf8(frames).unwrap(),
            "<redacted> leaks at src/Cache.java:41\n"
        );
        assert_eq!(
            bundle_bytes(&[0xff, 0x46, 0x33], false, &needles(), false),
            [0xff, 0x46, 0x33]
        );
    }
    #[test]
    fn the_generated_target_block_is_cut_out_of_a_driver() {
        let text = "import os\n\ndef setup():\n    # Entry-path harness H5: show whether\n    # F1 at a.py:3: holder static/global\n    #   writers: a.py:9 (put)\n    # DLA-HOOK setup: build\n    x = 1\n";
        let out = without_target_block(text);
        assert!(
            out.contains("    # (the generated target block is not part of the bundle)\n"),
            "{out}"
        );
        assert!(
            !out.contains("F1 at a.py:3") && !out.contains("writers:"),
            "{out}"
        );
        // The hook comment, which is not part of the block's run, stays.
        assert!(
            out.contains("# DLA-HOOK setup: build") && out.contains("x = 1"),
            "{out}"
        );
        // No block: unchanged.
        assert_eq!(without_target_block("a = 1\n"), "a = 1\n");
    }
}
