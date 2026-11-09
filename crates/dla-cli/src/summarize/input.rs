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

/// Resolves every input of `op`: each must be a file, inside the run's scope (files under
/// `.dla/` and outside the project are always readable), and given once. Nothing is
/// written.
pub fn resolve_all(tc: &ToolCtx, files: &[PathBuf], op: Op) -> Res<Vec<Input>> {
    let mut out: Vec<Input> = Vec::new();
    let mut canon: Vec<PathBuf> = Vec::new();
    for given in files {
        let shown = to_slash(given);
        let Some(abs) = locate(tc, given) else {
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
                if !r.starts_with(&format!("{DLA_DIR}/")) && !tc.in_scope(&r) {
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
