//! `dla bundle <F>`: a finding's label-free evidence bundle for an independent verifier.
//!
//! The bundle holds copies of the raw files behind the finding's growth, retention,
//! allocation and discriminator evidence, an index of those evidence records, and code
//! excerpts around up to four project frames that the bundled evidence names (allocation
//! stacks, retention paths, column sites). It never holds the finding's own site, the
//! hypothesis, the refutation condition, the archetype or suggestion, the label, the
//! explanation, the origin, or annotation notes: those keys are removed from every JSON
//! file it contains, and the excerpts are never chosen from the finding's site field.

use std::{
    fs,
    path::{Path, PathBuf},
};

use clap::Args;
use dla_core::{
    EvidenceId, FindingId,
    error::codes as core_codes,
    evidence::EvidenceClass,
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
    let ids = evidence_ids(&ledger, &id);
    let dir = tc.next_bundle_dir(&dir_name(&id))?;
    let built = build(&tc, &ledger, &dir, project, &id, owner.as_deref(), &ids);
    let (manifest_sha, files, copied, omitted, frames) = match built {
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
        "excerpt": !frames.is_empty(),
        "excerpt_frames": frames,
        "next": format!(
            "give the verifier only {} ; then: dla ledger verify {id} --site <file:line|none> --holder <h> --mechanism \"<text>\" --label <ID|UNCLASSIFIED> --bundle {manifest_sha}",
            ctx.pointer(&rel)
        ),
    }))
}

type Built = (String, usize, usize, Vec<String>, Vec<String>);

fn build(
    tc: &ToolCtx,
    ledger: &dla_core::Ledger,
    dir: &Path,
    project: &Path,
    id: &FindingId,
    owner: Option<&str>,
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
    // Frames the evidence names, each with the class rank of its record.
    let mut named: Vec<(u8, Vec<(String, usize)>)> = Vec::new();
    for eid in ids {
        let Some(e) = ledger.evidence_by_id(eid) else {
            continue;
        };
        let mut result = e.result.clone();
        strip_forbidden(&mut result);
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
                let bytes = fs::read(&src).map_err(|e| io_fail(&src, e))?;
                let kept = bundle_bytes(&bytes, raw.ends_with(".json"));
                frames_in_bytes(&kept, &mut here);
                write(&name, &kept)?;
                rec.insert("file".into(), json!(name));
                copied += 1;
            }
        }
        records.push(Value::Object(rec));
        let rank = match e.kind.class() {
            EvidenceClass::Allocation => 0,
            EvidenceClass::Retention => 1,
            EvidenceClass::Growth => 2,
            _ => 3,
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
        "finding": id.to_string(),
        "excerpt": (!frames.is_empty()).then_some("excerpt.txt"),
        "frames": frames,
        "evidence": records,
    });
    if let Some(o) = owner {
        index["owner"] = json!(o);
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
}
