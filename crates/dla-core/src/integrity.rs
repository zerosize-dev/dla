//! Integrity seals for the files `dla` protects.
//!
//! `<run>/integrity/<file>.seal` holds the length and SHA-256 of a protected file as `dla`
//! last wrote it. [`verify`] reports every file that no longer matches. Hooks are a guard
//! rail; the seals are what makes an out-of-band edit visible.
//!
//! A write and its seal are two steps, so a process killed between them leaves a file that
//! does not match its seal. Every guarded write therefore leaves a `<file>.pending` marker in
//! `integrity/` until its seal is written. When a marker is found, the file is accepted (and
//! sealed again by the next `dla` write, or by `dla ledger merge`) only if the difference is
//! provably `dla`'s own interrupted write: the sealed content is still the file's prefix
//! (appended files), or the file is exactly the content the writer announced (rewritten
//! files). Any other difference stays a mismatch.
//!
//! The protected files are `ledger.jsonl`, `ledger.<worker>.jsonl`, `state.json`,
//! `coverage.json` and `report.md`. Every writer seals right after its write, under the
//! lock that guarded the write. A run without an `integrity/` directory (written before
//! schema 6) is an unsealed run: nothing is sealed or verified, and that is not tampering.
//! The threat model is a model editing a file by accident or to please a gate, not an
//! attacker who can run the same algorithm.

use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    error::Error,
    fsutil::{to_pretty_json, write_atomic},
    slice,
    state::{COVERAGE_FILE, LEDGER_FILE, REPORT_FILE, STATE_FILE},
};

/// Directory inside the run directory that holds the seals.
pub const INTEGRITY_DIR: &str = "integrity";
/// Suffix of a seal file.
pub const SEAL_SUFFIX: &str = ".seal";

/// Suffix of the marker a guarded write leaves until its seal is written.
pub const PENDING_SUFFIX: &str = ".pending";

/// The seal of one protected file.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct Seal {
    /// Length in bytes.
    pub len: u64,
    /// SHA-256, lowercase hex.
    pub sha256: String,
    /// When the seal was written.
    pub written_at: DateTime<Utc>,
}

/// Why a protected file failed verification.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TamperKind {
    /// The file's length or hash differs from its seal.
    Modified,
    /// The file exists and has no seal in a run that has an `integrity/` directory.
    Unsealed,
    /// The seal exists and the file does not.
    Missing,
}

/// A protected file that does not match its seal.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct Tamper {
    /// File name relative to the run directory.
    pub file: String,
    /// What is wrong.
    pub kind: TamperKind,
    /// What the seal says (`len:sha256`), when there is one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sealed: Option<String>,
    /// What the file is now (`len:sha256`), when it exists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub found: Option<String>,
}

impl Tamper {
    /// One line for the report and the gate: what is wrong with which file.
    pub fn describe(&self) -> String {
        match self.kind {
            TamperKind::Modified => format!(
                "{} does not match the seal dla wrote (sealed {}, found {})",
                self.file,
                self.sealed.as_deref().unwrap_or("?"),
                self.found.as_deref().unwrap_or("?")
            ),
            TamperKind::Unsealed => format!("{} exists and was never sealed by dla", self.file),
            TamperKind::Missing => format!("{} is sealed and no longer exists", self.file),
        }
    }
}

/// Whether `name` (a file name inside a run directory) is a protected file.
pub fn is_protected(name: &str) -> bool {
    name == LEDGER_FILE
        || name == STATE_FILE
        || name == COVERAGE_FILE
        || name == REPORT_FILE
        || slice::worker_of_file(name).is_some()
}

/// `<run_dir>/integrity`.
pub fn dir(run_dir: &Path) -> PathBuf {
    run_dir.join(INTEGRITY_DIR)
}

/// True when the run has an `integrity/` directory: its protected files are sealed.
pub fn is_sealed_run(run_dir: &Path) -> bool {
    dir(run_dir).is_dir()
}

/// Creates the `integrity/` directory: from now on the run's protected files are sealed.
/// `dla run init` calls it before the first write.
pub fn init_dir(run_dir: &Path) -> Result<(), Error> {
    let d = dir(run_dir);
    fs::create_dir_all(&d).map_err(|e| Error::io(&d, e))
}

fn seal_path(run_dir: &Path, file: &str) -> PathBuf {
    dir(run_dir).join(format!("{file}{SEAL_SUFFIX}"))
}

fn pending_path(run_dir: &Path, file: &str) -> PathBuf {
    dir(run_dir).join(format!("{file}{PENDING_SUFFIX}"))
}

/// Length and SHA-256 of the first `n` bytes of a file, or `None` when it is shorter.
fn measure_prefix(path: &Path, n: u64) -> Option<String> {
    let mut f = fs::File::open(path).ok()?.take(n);
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut len = 0u64;
    loop {
        let k = f.read(&mut buf).ok()?;
        if k == 0 {
            break;
        }
        hasher.update(&buf[..k]);
        len += k as u64;
    }
    (len == n).then(|| hex::encode(hasher.finalize()))
}

/// True when `file` differs from its seal only because a `dla` write was interrupted: a
/// `.pending` marker exists and the file is the sealed content plus more, or exactly the
/// content the interrupted writer announced.
fn is_interrupted_write(run_dir: &Path, file: &str) -> bool {
    let Ok(marker) = fs::read_to_string(pending_path(run_dir, file)) else {
        return false;
    };
    let path = run_dir.join(file);
    let Ok((len, sha)) = measure(&path) else {
        return false;
    };
    if marker
        .trim()
        .strip_prefix("expect ")
        .is_some_and(|want| want == sha)
    {
        return true;
    }
    // A file never sealed counts as sealed empty: its creation was the interrupted write.
    let sp = seal_path(run_dir, file);
    let seal = if sp.exists() {
        match read_seal(&sp) {
            Some(s) => (s.len, s.sha256),
            None => return false,
        }
    } else {
        (0, hex::encode(Sha256::digest(b"")))
    };
    len >= seal.0 && measure_prefix(&path, seal.0).is_some_and(|p| p == seal.1)
}

/// True when `file` does not match its seal because a `dla` write was killed before it
/// was sealed. `dla ledger merge` and the next `dla` write of the file settle it.
pub fn interrupted(run_dir: &Path, file: &str) -> bool {
    is_sealed_run(run_dir)
        && is_interrupted_write(run_dir, file)
        && match (
            read_seal(&seal_path(run_dir, file)),
            measure(&run_dir.join(file)),
        ) {
            (Some(s), Ok((len, sha))) => s.len != len || s.sha256 != sha,
            (None, Ok(_)) => true,
            _ => false,
        }
}

/// Seals every protected file whose only difference from its seal is an interrupted `dla`
/// write, and clears the markers of the others that match. Returns the files sealed.
pub fn settle(run_dir: &Path) -> Result<Vec<String>, Error> {
    let mut sealed = Vec::new();
    if !is_sealed_run(run_dir) {
        return Ok(sealed);
    }
    for f in existing_protected(run_dir) {
        if interrupted(run_dir, &f) {
            seal_file(run_dir, &f)?;
            sealed.push(f.clone());
        }
        let _ = fs::remove_file(pending_path(run_dir, &f));
    }
    Ok(sealed)
}

/// Length and SHA-256 of a file, read in one pass.
fn measure(path: &Path) -> std::io::Result<(u64, String)> {
    let mut f = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut len = 0u64;
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        len += n as u64;
    }
    Ok((len, hex::encode(hasher.finalize())))
}

fn label(len: u64, sha: &str) -> String {
    format!("{len}:{sha}")
}

/// Seals `file` (a name relative to `run_dir`) as it is now and writes the seal file.
/// Callers hold the lock that guarded the write of `file`. Returns `None` and writes
/// nothing when the run is not sealed (no `integrity/` directory).
pub fn seal_file(run_dir: &Path, file: &str) -> Result<Option<Seal>, Error> {
    if !is_sealed_run(run_dir) {
        return Ok(None);
    }
    let path = run_dir.join(file);
    let (len, sha256) = measure(&path).map_err(|e| Error::io(&path, e))?;
    let seal = Seal {
        len,
        sha256,
        written_at: crate::state::now_utc(),
    };
    write_atomic(&seal_path(run_dir, file), to_pretty_json(&seal).as_bytes())?;
    Ok(Some(seal))
}

/// Taken just before a writer changes a protected file: remembers whether the file still
/// matched its seal. [`Guard::seal`] then seals the new content only if it did, so an
/// out-of-band edit stays visible after a later `dla` write instead of being sealed away.
/// `dla run reseal` is the one way to accept a changed file.
#[derive(Debug)]
pub struct Guard {
    run_dir: PathBuf,
    file: String,
    intact: bool,
}

impl Guard {
    /// Checks `file` against its seal. A file that does not exist and has no seal is
    /// intact (its first write); in an unsealed run everything is. A file left behind by
    /// an interrupted `dla` write (see the module docs) is intact too. Leaves the
    /// `.pending` marker that [`Guard::seal`] removes.
    pub fn new(run_dir: &Path, file: &str) -> Self {
        let intact = if !is_sealed_run(run_dir) {
            true
        } else {
            let path = run_dir.join(file);
            let matches = match (read_seal(&seal_path(run_dir, file)), measure(&path)) {
                (Some(s), Ok((len, sha))) => s.len == len && s.sha256 == sha,
                (None, Err(_)) => !seal_path(run_dir, file).exists(),
                _ => false,
            };
            let intact = matches || is_interrupted_write(run_dir, file);
            // Best effort: without the marker a killed write is a plain mismatch.
            let _ = fs::write(pending_path(run_dir, file), "");
            intact
        };
        Self {
            run_dir: run_dir.to_path_buf(),
            file: file.to_owned(),
            intact,
        }
    }

    /// For a writer that replaces the whole file: records the hash of the content about to
    /// be written, so that a kill between the write and the seal can be told from an edit.
    pub fn expect(&self, content: &[u8]) {
        if self.intact && is_sealed_run(&self.run_dir) {
            let _ = fs::write(
                pending_path(&self.run_dir, &self.file),
                format!("expect {}", hex::encode(Sha256::digest(content))),
            );
        }
    }

    /// Seals the file as it is now when it was intact before the write.
    pub fn seal(self) -> Result<Option<Seal>, Error> {
        let sealed = if self.intact {
            seal_file(&self.run_dir, &self.file)?
        } else {
            None
        };
        let _ = fs::remove_file(pending_path(&self.run_dir, &self.file));
        Ok(sealed)
    }
}

/// The protected files that exist in `run_dir`, sorted: the ledger, the slices, the
/// state, the coverage manifest and the report.
pub fn existing_protected(run_dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    if let Ok(rd) = fs::read_dir(run_dir) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if is_protected(&name) && e.file_type().is_ok_and(|t| t.is_file()) {
                out.push(name);
            }
        }
    }
    out.sort();
    out
}

/// Seals every protected file that exists (used by `dla run reseal`). Returns the names
/// sealed. Fails on an unsealed run: create the directory first with [`init_dir`].
pub fn seal_all(run_dir: &Path) -> Result<Vec<String>, Error> {
    let files = existing_protected(run_dir);
    for f in &files {
        seal_file(run_dir, f)?;
    }
    Ok(files)
}

/// Takes the exclusive lock that serialises writes of the main `ledger.jsonl` and its
/// seal; released when the returned handle is dropped (or the process dies). `None` for
/// an unsealed run, which has nothing to keep in step.
pub fn lock_ledger(run_dir: &Path) -> Result<Option<fs::File>, Error> {
    if !is_sealed_run(run_dir) {
        return Ok(None);
    }
    let path = run_dir.join("ledger.lock");
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| Error::io(&path, e))?;
    file.lock().map_err(|e| Error::io(&path, e))?;
    Ok(Some(file))
}

fn read_seal(path: &Path) -> Option<Seal> {
    serde_json::from_str(&fs::read_to_string(path).ok()?).ok()
}

/// Checks every protected file of the run against its seal. A run without an `integrity/`
/// directory (written before schema 6) reports nothing here. A seal that cannot be read
/// counts as a mismatch.
pub fn verify(run_dir: &Path) -> Vec<Tamper> {
    if !is_sealed_run(run_dir) {
        return Vec::new();
    }
    let mut out = Vec::new();
    let files = existing_protected(run_dir);
    for f in &files {
        let path = run_dir.join(f);
        let found = measure(&path).ok();
        let sp = seal_path(run_dir, f);
        let seal_text_exists = sp.exists();
        match (read_seal(&sp), found) {
            (Some(s), Some((len, sha))) => {
                if s.len != len || s.sha256 != sha {
                    out.push(Tamper {
                        file: f.clone(),
                        kind: TamperKind::Modified,
                        sealed: Some(label(s.len, &s.sha256)),
                        found: Some(label(len, &sha)),
                    });
                }
            }
            (Some(s), None) => out.push(Tamper {
                file: f.clone(),
                kind: TamperKind::Modified,
                sealed: Some(label(s.len, &s.sha256)),
                found: None,
            }),
            (None, found) if seal_text_exists => out.push(Tamper {
                file: f.clone(),
                kind: TamperKind::Modified,
                sealed: Some("unreadable seal".to_owned()),
                found: found.map(|(l, s)| label(l, &s)),
            }),
            (None, found) => out.push(Tamper {
                file: f.clone(),
                kind: TamperKind::Unsealed,
                sealed: None,
                found: found.map(|(l, s)| label(l, &s)),
            }),
        }
    }
    // Seals whose file is gone.
    if let Ok(rd) = fs::read_dir(dir(run_dir)) {
        let mut gone: Vec<String> = rd
            .flatten()
            .filter_map(|e| {
                let n = e.file_name().to_string_lossy().into_owned();
                let file = n.strip_suffix(SEAL_SUFFIX)?.to_owned();
                (is_protected(&file) && !files.contains(&file)).then_some(file)
            })
            .collect();
        gone.sort();
        for file in gone {
            let sealed = read_seal(&seal_path(run_dir, &file)).map(|s| label(s.len, &s.sha256));
            out.push(Tamper {
                file,
                kind: TamperKind::Missing,
                sealed,
                found: None,
            });
        }
    }
    out.sort_by(|a, b| a.file.cmp(&b.file));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[test]
    fn unsealed_run_seals_nothing_and_verifies_clean() {
        let d = run();
        fs::write(d.path().join(LEDGER_FILE), "x\n").unwrap();
        assert_eq!(seal_file(d.path(), LEDGER_FILE).unwrap(), None);
        assert!(!dir(d.path()).exists());
        assert!(verify(d.path()).is_empty());
    }

    #[test]
    fn seal_then_modify_is_a_mismatch() {
        let d = run();
        init_dir(d.path()).unwrap();
        fs::write(d.path().join(LEDGER_FILE), "one\n").unwrap();
        let s = seal_file(d.path(), LEDGER_FILE).unwrap().unwrap();
        assert_eq!(s.len, 4);
        assert!(verify(d.path()).is_empty());
        fs::write(d.path().join(LEDGER_FILE), "one\ntwo\n").unwrap();
        let t = verify(d.path());
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].kind, TamperKind::Modified);
        assert!(t[0].describe().contains("ledger.jsonl"));
    }

    #[test]
    fn file_without_seal_and_seal_without_file() {
        let d = run();
        init_dir(d.path()).unwrap();
        fs::write(d.path().join(STATE_FILE), "{}").unwrap();
        fs::write(d.path().join("ledger.jvm.jsonl"), "x\n").unwrap();
        seal_file(d.path(), "ledger.jvm.jsonl").unwrap();
        let t = verify(d.path());
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].kind, TamperKind::Unsealed);
        fs::remove_file(d.path().join("ledger.jvm.jsonl")).unwrap();
        let t = verify(d.path());
        assert!(t.iter().any(|x| x.kind == TamperKind::Missing));
    }

    #[test]
    fn other_files_are_not_protected() {
        assert!(is_protected("report.md"));
        assert!(is_protected("ledger.jvm.jsonl"));
        assert!(!is_protected("run.lock"));
        assert!(!is_protected("notes.md"));
        assert!(!is_protected("ledger.jvm.lock"));
    }
}
