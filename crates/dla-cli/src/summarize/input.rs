//! Input files of `dla summarize`: resolved, hashed and ready to sniff.

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use dla_core::{Error, paths::to_slash, state::DLA_DIR, toolctx::ToolCtx};
use sha2::{Digest, Sha256};

use super::Op;
use crate::fail::{Failure, Res};

/// Bytes of the file start kept for sniffing.
const HEAD_BYTES: usize = 8192;

/// One input file.
#[derive(Debug, Clone)]
pub struct Input {
    /// Project-relative path with `/` separators; the absolute path when the file is
    /// outside the project.
    pub rel: String,
    /// Absolute path.
    pub abs: PathBuf,
    /// SHA-256 of the file's bytes, lowercase hex.
    pub sha256: String,
    /// Size in bytes.
    pub size: u64,
    /// The first bytes of the file.
    pub head: Vec<u8>,
}

impl Input {
    /// The path as shown in output.
    pub fn shown(&self) -> &str {
        &self.rel
    }

    /// The file name, lowercase.
    pub fn name_lower(&self) -> String {
        self.abs
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default()
    }

    /// True when the lowercase file name ends with `suffix`.
    pub fn has_suffix(&self, suffix: &str) -> bool {
        self.name_lower().ends_with(suffix)
    }

    /// The start of the file as text (invalid UTF-8 replaced).
    pub fn head_text(&self) -> String {
        String::from_utf8_lossy(&self.head).into_owned()
    }

    /// The whole file as text, refusing a file over `limit` bytes.
    pub fn read_text(&self, limit: u64) -> Result<String, Failure> {
        if self.size > limit {
            return Err(Failure::refuse(
                "insufficient-resources",
                format!(
                    "{} is {} bytes, over the {limit}-byte limit for this format",
                    self.shown(),
                    self.size
                ),
                "an input of a plausible size",
                "trim the file to the part that matters, or capture a shorter recording",
            ));
        }
        let bytes = fs::read(&self.abs).map_err(|e| Failure::from(Error::io(&self.abs, e)))?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
}

/// SHA-256 of a file, streamed.
pub fn sha256_of(path: &Path) -> Result<String, Error> {
    let mut f = fs::File::open(path).map_err(|e| Error::io(path, e))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf).map_err(|e| Error::io(path, e))?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}

fn head_of(path: &Path) -> Result<Vec<u8>, Error> {
    let mut f = fs::File::open(path).map_err(|e| Error::io(path, e))?;
    let mut buf = vec![0u8; HEAD_BYTES];
    let mut n = 0;
    while n < HEAD_BYTES {
        let k = f.read(&mut buf[n..]).map_err(|e| Error::io(path, e))?;
        if k == 0 {
            break;
        }
        n += k;
    }
    buf.truncate(n);
    Ok(buf)
}

/// Largest dump file read to look for its exit record.
const EXIT_SCAN_BYTES: u64 = 64 << 20;

/// The label of a capture: the directory of its dump file (`p4-alloc`), else the path as shown.
fn capture_label(i: &Input) -> String {
    i.abs
        .parent()
        .and_then(|d| d.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| i.shown().to_owned())
}

/// True for the dump of an allocator-shim capture that holds no record of the end of its process:
/// the process still writes it, or was cut before it did. A dump is a shim capture when the shim
/// wrote in it (a record with `raw_sites`), when it is a `shim-dump.jsonl`, or when it is the
/// `dump.jsonl` of an allocation point (`p<k>-alloc/`) of a Rust or native driver, whose process
/// runs with the shim preloaded and gets its exit record from it. The `dump.jsonl` of any other
/// point of those drivers holds checkpoint records only, complete as it is, and is never this;
/// nor is a dump of a managed runtime, a file that is no harness dump, or one over the size bound.
fn shim_dump_without_exit(i: &Input) -> bool {
    let name = i.name_lower();
    if !matches!(name.as_str(), "dump.jsonl" | "shim-dump.jsonl") || i.size > EXIT_SCAN_BYTES {
        return false;
    }
    let Ok(bytes) = fs::read(&i.abs) else {
        return false;
    };
    let text = String::from_utf8_lossy(&bytes);
    let (dumps, _bad) = dla_core::HarnessDump::parse_lines(&text);
    let alloc_point = i
        .abs
        .parent()
        .and_then(|d| d.file_name())
        .is_some_and(|n| n.to_string_lossy().ends_with("-alloc"));
    let driver = dumps.iter().any(|d| {
        d.runtime
            .as_deref()
            .is_some_and(|r| r.eq_ignore_ascii_case("native") || r.eq_ignore_ascii_case("rust"))
    });
    let shim = name == "shim-dump.jsonl"
        || dumps.iter().any(|d| d.raw_sites.is_some())
        || (alloc_point && driver);
    shim && !dumps
        .iter()
        .any(|d| d.phase == dla_core::harness::DumpPhase::Exit)
}

/// The sentence for a capture that is not complete: why it is not (`why`, as a verb phrase), and
/// what to do. The harness is the one `harness/<H>/` of the path (`rel`, else `abs`).
pub fn unfinished_text(label: &str, rel: &str, abs: &str, why: &str) -> String {
    let h = dla_core::measured_holders::harness_of_path(rel)
        .or_else(|| dla_core::measured_holders::harness_of_path(abs))
        .unwrap_or("<H>");
    format!(
        "capture {label} {why}: it is still being written or its process was cut; wait for `dla harness run {h}` to end (dla run status), then summarize every capture it lists"
    )
}

/// The refusal for a heap dump or snapshot that is cut short (the parser read it up to its end and
/// found the file incomplete): the capture is still being written, or its process was cut. The
/// code stays `malformed-artifact`; the message says to wait for the harness, and `retake` is what
/// to do when it has ended and the file is still cut.
pub fn truncated_refusal(rel: &str, why: &str, retake: &str) -> Failure {
    let label = rel.rsplit('/').next().unwrap_or(rel);
    Failure::refuse(
        "malformed-artifact",
        unfinished_text(label, rel, rel, &format!("is truncated ({why})")),
        "a complete capture: the harness that writes it has ended",
        format!(
            "dla run status   (wait for dla harness run <H> to end, then summarize every capture file its captures.json lists); if it has ended and the file is still cut: {retake}"
        ),
    )
}

/// The inputs without the shim dumps that have no exit record (a capture still being written, or
/// whose process was cut), and one sentence per capture left out. When no input is left, the call
/// is refused with the same sentence: a summary of a capture that is still being written says
/// nothing about the run, and the harness must end first (`dla run status`).
pub fn without_unfinished(inputs: Vec<Input>) -> Res<(Vec<Input>, Vec<String>)> {
    let (kept, left): (Vec<Input>, Vec<Input>) =
        inputs.into_iter().partition(|i| !shim_dump_without_exit(i));
    if left.is_empty() {
        return Ok((kept, Vec::new()));
    }
    let said: Vec<String> = left
        .iter()
        .map(|i| {
            unfinished_text(
                &capture_label(i),
                &i.rel,
                &to_slash(&i.abs),
                "has no exit record",
            )
        })
        .collect();
    if kept.is_empty() {
        return Err(Failure::refuse(
            "capture-incomplete",
            said.join("; "),
            "dump files whose processes ended (each has an exit record)",
            "dla run status   (wait for dla harness run <H> to end), then dla summarize <every dump file its captures.json lists>",
        ));
    }
    Ok((kept, said))
}

/// Resolves one file: a path relative to the project, an absolute path, or a path relative
/// to the run directory (`attach/P1/histo.txt`, `harness/H1/out/1234.jsonl`).
fn locate(tc: &ToolCtx, given: &Path) -> Option<PathBuf> {
    if given.is_absolute() {
        return given.is_file().then(|| given.to_path_buf());
    }
    [tc.project.join(given), tc.run_dir.join(given)]
        .into_iter()
        .find(|p| p.is_file())
}

/// The refusal for a path that `harness/<H>/captures.json` lists as deleted (`pruned`): the tool
/// deleted the file to keep the large captures of the run under `DLA_CAPTURE_KEEP_MB`, after a
/// summary had read it. The fix is the capture again, with the options of the set.
fn pruned_refusal(tc: &ToolCtx, given: &Path, shown: &str) -> Option<Failure> {
    use dla_core::harness::{CaptureKind, HARNESS_DIR};
    let pointer = tc.pointer("");
    let mut rel = shown.to_owned();
    if let Ok(r) = given.strip_prefix(&tc.run_dir) {
        rel = to_slash(r);
    }
    let rel = rel
        .strip_prefix(pointer.as_str())
        .unwrap_or(&rel)
        .to_owned();
    for e in fs::read_dir(tc.run_dir.join(HARNESS_DIR)).ok()?.flatten() {
        let Ok(text) = fs::read_to_string(e.path().join("captures.json")) else {
            continue;
        };
        let Some(files) = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v.get("files").cloned())
            .and_then(|f| serde_json::from_value::<Vec<dla_core::harness::CaptureFile>>(f).ok())
        else {
            continue;
        };
        let Some(hit) = files.iter().find(|f| f.pruned && f.path == rel) else {
            continue;
        };
        let harness = e.file_name().to_string_lossy().into_owned();
        let kind = match hit.kind {
            CaptureKind::Jfr => "jfr",
            _ => "heap",
        };
        let mut fix = format!("dla harness run {harness}");
        if let Some(mode) = ["fresh", "repeated"]
            .iter()
            .find(|m| hit.label.ends_with(&format!("-{m}")))
        {
            fix.push_str(&format!(" --mode {mode}"));
        }
        fix.push_str(&format!(" --capture {kind}"));
        if !hit.label.ends_with("-native") {
            let dumps = files
                .iter()
                .filter(|f| f.label == hit.label && f.kind == hit.kind && f.pruned)
                .count();
            if dumps > 1 {
                fix.push_str(&format!(" --dumps {dumps}"));
            }
        }
        return Some(Failure::refuse(
            "artifact-not-found",
            format!(
                "{shown} was deleted: the run keeps its large captures under DLA_CAPTURE_KEEP_MB, and a summary had read this one (captures.json of {harness} lists it as pruned, with its digest)"
            ),
            "the capture file, which the run deleted",
            fix,
        ));
    }
    None
}

/// Resolves every input of `op`: each must be a file, inside the run's scope (files under
/// `.dla/` and outside the project are always readable), and given once. Nothing is
/// written.
pub fn resolve_all(tc: &ToolCtx, files: &[PathBuf], op: Op) -> Res<Vec<Input>> {
    let mut out: Vec<Input> = Vec::new();
    let mut canon: Vec<PathBuf> = Vec::new();
    for given in files {
        let shown = to_slash(given);
        let Some(abs) = locate(tc, given) else {
            if let Some(pruned) = pruned_refusal(tc, given, &shown) {
                return Err(pruned);
            }
            return Err(Failure::refuse(
                "artifact-not-found",
                format!("{shown} is not a file"),
                format!("an existing {} input file", op.name()),
                format!(
                    "dla summarize {} <project-relative or absolute path, or a path under .dla/<run>/>",
                    op.name()
                ),
            ));
        };
        let rel = match abs.strip_prefix(&tc.project) {
            Ok(r) => {
                let r = to_slash(r);
                // Files under the run directory are exempt from the scope; a path that
                // only starts with `.dla/` but steps out with `..` is not.
                let parts: Vec<&str> = r.split('/').collect();
                let under_dla = parts.first() == Some(&DLA_DIR) && !parts.contains(&"..");
                if !under_dla && !tc.in_scope(&r) {
                    return Err(Failure::refuse(
                        "out-of-scope",
                        format!("{r} is outside the run's scope"),
                        "an input inside the run's scope",
                        "dla run init   (with a scope that includes the input)",
                    ));
                }
                r
            }
            Err(_) => to_slash(&abs),
        };
        let c = fs::canonicalize(&abs).unwrap_or_else(|_| abs.clone());
        if canon.contains(&c) {
            return Err(Failure::refuse(
                "duplicate-artifact",
                format!("{rel} is given twice"),
                "each input once",
                format!("dla summarize {} <file1> <file2> <file3>", op.name()),
            ));
        }
        canon.push(c);
        let sha256 = sha256_of(&abs)?;
        let head = head_of(&abs)?;
        let size = fs::metadata(&abs)
            .map_err(|e| Failure::from(Error::io(&abs, e)))?
            .len();
        out.push(Input {
            rel,
            abs,
            sha256,
            size,
            head,
        });
    }
    Ok(out)
}
