//! Worker slices: `ledger.<worker>.jsonl` next to the main `ledger.jsonl`.
//!
//! Each worker appends only to its own slice. A worker that issues parallel `dla` calls
//! has several writers, so every slice write runs under an exclusive lock on
//! `ledger.<worker>.lock` ([`lock`]). The slice's first line is `slice_started`; a finished worker ends it with `slice_done`.
//! A process killed mid-write can leave a torn final line (no trailing newline, not
//! valid JSON); [`split`] finds it so replay can drop it and say so. The main ledger
//! never gets that treatment: it stays strict.
//!
//! The merge of slices into the main ledger lives in [`crate::merge`].

use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Weak},
};

use sha2::{Digest, Sha256};

use crate::{error::Error, ids::valid_worker_name, state::LEDGER_FILE};

/// Refusal codes of the M4 ledger rules.
pub mod codes {
    /// An artifact-run finding names a site, or a confirmation of one does.
    pub const SITE_FORBIDDEN: &str = "site-forbidden";
    /// The command or record is for the core only.
    pub const CORE_ONLY: &str = "core-only";
    /// The command or record is for a worker only.
    pub const WORKER_ONLY: &str = "worker-only";
    /// A worker slice touches an item that is not its own.
    pub const FOREIGN_ITEM: &str = "foreign-item";
    /// A worker slice allocated an ID without its own prefix.
    pub const BAD_PREFIX: &str = "bad-prefix";
    /// The slice is already closed by `slice_done`.
    pub const SLICE_CLOSED: &str = "slice-closed";
    /// A slice record is out of order, repeated or names another worker.
    pub const SLICE_STATE: &str = "slice-state";
    /// A merge record is not valid for the ledger it is appended to.
    pub const MERGE_STATE: &str = "merge-state";
    /// A worker name is not valid.
    pub const BAD_WORKER: &str = "bad-worker";
    /// A verification record is not valid.
    pub const BAD_VERIFICATION: &str = "bad-verification";
    /// A slice file cannot be replayed.
    pub const SLICE_INVALID: &str = "slice-invalid";
}

/// File name of a worker's slice, `ledger.<worker>.jsonl`.
pub fn slice_file_name(worker: &str) -> String {
    format!("ledger.{worker}.jsonl")
}

/// `<run_dir>/ledger.<worker>.lock`: the file the slice's writers lock.
pub fn lock_path(run_dir: &Path, worker: &str) -> PathBuf {
    run_dir.join(format!("ledger.{worker}.lock"))
}

/// Locks held by this process, so opening the same slice twice here does not wait for
/// itself.
static HELD: Mutex<BTreeMap<PathBuf, Weak<fs::File>>> = Mutex::new(BTreeMap::new());

/// Takes the exclusive lock of `worker`'s slice, waiting for other processes. The lock
/// is released when the last clone of the returned handle is dropped, and by the system
/// when the process dies, so a killed worker never stalls the others. A second request
/// for the same slice from the same process shares the handle instead of waiting.
pub fn lock(run_dir: &Path, worker: &str) -> Result<Arc<fs::File>, Error> {
    let path = lock_path(run_dir, worker);
    let mut held = HELD.lock().unwrap_or_else(|p| p.into_inner());
    if let Some(f) = held.get(&path).and_then(Weak::upgrade) {
        return Ok(f);
    }
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| Error::io(&path, e))?;
    file.lock().map_err(|e| Error::io(&path, e))?;
    let file = Arc::new(file);
    held.retain(|_, w| w.strong_count() > 0);
    held.insert(path, Arc::downgrade(&file));
    Ok(file)
}

/// Reads a slice file as text. Only the complete lines (up to the last newline) have to
/// be valid UTF-8, because that is all a process killed in the middle of a write leaves
/// behind: a final fragment that was cut inside a multi-byte character is decoded
/// lossily, so it is not valid JSON and [`split`] reports it as torn.
pub fn read_text(path: &Path) -> io::Result<String> {
    let bytes = fs::read(path)?;
    let cut = bytes.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
    let (done, tail) = bytes.split_at(cut);
    let mut text = std::str::from_utf8(done)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
        .to_owned();
    text.push_str(&String::from_utf8_lossy(tail));
    Ok(text)
}

/// `<run_dir>/ledger.<worker>.jsonl`.
pub fn slice_path(run_dir: &Path, worker: &str) -> PathBuf {
    run_dir.join(slice_file_name(worker))
}

/// The worker a slice file name belongs to: `ledger.jvm.jsonl` gives `jvm`.
pub fn worker_of_file(name: &str) -> Option<&str> {
    let w = name.strip_prefix("ledger.")?.strip_suffix(".jsonl")?;
    valid_worker_name(w).then_some(w)
}

/// Every worker that has a slice file in `run_dir`, sorted.
pub fn slice_workers(run_dir: &Path) -> Result<Vec<String>, Error> {
    let mut out = Vec::new();
    let rd = match fs::read_dir(run_dir) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
        Err(e) => return Err(Error::io(run_dir, e)),
    };
    for entry in rd {
        let entry = entry.map_err(|e| Error::io(run_dir, e))?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name == LEDGER_FILE || !entry.file_type().is_ok_and(|t| t.is_file()) {
            continue;
        }
        if let Some(w) = worker_of_file(name) {
            out.push(w.to_owned());
        }
    }
    out.sort();
    Ok(out)
}

/// The final line of a slice that a killed process left half written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TornTail {
    /// 1-based line number the fragment would have had.
    pub line: u64,
    /// Its length in bytes.
    pub bytes: usize,
}

/// A slice file cut into complete lines.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SliceText<'a> {
    /// The complete lines, without their newline. A final line without a newline that
    /// is valid JSON counts as complete.
    pub lines: Vec<&'a str>,
    /// The dropped torn final line, when there is one.
    pub torn: Option<TornTail>,
}

impl SliceText<'_> {
    /// Bytes the complete lines occupy in the file, each counted with its newline.
    pub fn valid_bytes(&self) -> usize {
        self.lines.iter().map(|l| l.len() + 1).sum()
    }

    /// SHA-256 (lowercase hex) of the first `n` complete lines, each followed by a
    /// newline: the digest `slice_merged` records.
    pub fn sha256_of_first(&self, n: usize) -> String {
        let mut h = Sha256::new();
        for l in self.lines.iter().take(n) {
            h.update(l.as_bytes());
            h.update(b"\n");
        }
        hex::encode(h.finalize())
    }
}

/// Cuts slice text into complete lines. A final fragment with no trailing newline that
/// does not parse as JSON is torn: it is reported and not returned as a line. (A
/// prefix of a JSON object is never a complete JSON value, so a fragment that parses
/// was fully written.)
pub fn split(text: &str) -> SliceText<'_> {
    let mut lines: Vec<&str> = text.split('\n').collect();
    // `split` yields a trailing empty piece when the text ends with a newline.
    let tail = lines.pop().unwrap_or("");
    let mut torn = None;
    if !tail.is_empty() {
        if serde_json::from_str::<serde_json::Value>(tail).is_ok() {
            lines.push(tail);
        } else {
            torn = Some(TornTail {
                line: lines.len() as u64 + 1,
                bytes: tail.len(),
            });
        }
    }
    SliceText { lines, torn }
}

/// What the report and the status need to know about a slice without replaying it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceSummary {
    /// The worker.
    pub worker: String,
    /// Complete lines in the slice file.
    pub lines: u64,
    /// True when one of them is `slice_done`.
    pub done: bool,
    /// True when the file ends in a torn fragment.
    pub torn_tail: bool,
}

/// Counts the lines of a slice and finds `slice_done`, without applying any rule.
pub fn summarize(worker: &str, text: &str) -> SliceSummary {
    let cut = split(text);
    let done = cut.lines.iter().any(|l| {
        serde_json::from_str::<serde_json::Value>(l)
            .ok()
            .and_then(|v| {
                v.get("type")
                    .and_then(|t| t.as_str())
                    .map(|t| t == "slice_done")
            })
            .unwrap_or(false)
    });
    SliceSummary {
        worker: worker.to_owned(),
        lines: cut.lines.len() as u64,
        done,
        torn_tail: cut.torn.is_some(),
    }
}

/// Summaries of every slice file in `run_dir`, sorted by worker. A file that cannot be
/// read is skipped.
pub fn summaries(run_dir: &Path) -> Result<Vec<SliceSummary>, Error> {
    let mut out = Vec::new();
    for w in slice_workers(run_dir)? {
        if let Ok(text) = read_text(&slice_path(run_dir, &w)) {
            out.push(summarize(&w, &text));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_round_trip() {
        assert_eq!(slice_file_name("jvm"), "ledger.jvm.jsonl");
        assert_eq!(worker_of_file("ledger.jvm.jsonl"), Some("jvm"));
        assert_eq!(worker_of_file("ledger.jsonl"), None);
        assert_eq!(worker_of_file("ledger.JVM.jsonl"), None);
        assert_eq!(worker_of_file("ledger..jsonl"), None);
        assert_eq!(worker_of_file("ledger.a.b.jsonl"), None);
    }

    #[test]
    fn split_finds_a_torn_tail_only_when_it_is_not_json() {
        let s = split("{\"a\":1}\n{\"b\":2}\n");
        assert_eq!(s.lines.len(), 2);
        assert!(s.torn.is_none());
        assert_eq!(s.valid_bytes(), 16);

        let s = split("{\"a\":1}\n{\"b\":");
        assert_eq!(s.lines, ["{\"a\":1}"]);
        assert_eq!(s.torn, Some(TornTail { line: 2, bytes: 5 }));

        let s = split("{\"a\":1}\n{\"b\":2}");
        assert_eq!(s.lines.len(), 2);
        assert!(s.torn.is_none());

        let s = split("");
        assert!(s.lines.is_empty() && s.torn.is_none());
        let s = split("{");
        assert!(s.lines.is_empty());
        assert_eq!(s.torn, Some(TornTail { line: 1, bytes: 1 }));
    }

    #[test]
    fn a_tear_inside_a_multibyte_character_is_a_torn_tail() {
        let dir = std::env::temp_dir().join(format!("dla-slice-read-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join("s.jsonl");
        let mut b = b"{\"a\":1}\n{\"t\":\"caf".to_vec();
        b.push(0xC3);
        fs::write(&p, &b).unwrap();
        let text = read_text(&p).unwrap();
        let s = split(&text);
        assert_eq!(s.lines, ["{\"a\":1}"]);
        assert!(s.torn.is_some());
        // Bad bytes in a complete line stay an error.
        fs::write(&p, b"{\"a\":\"\xC3\"}\n").unwrap();
        assert!(read_text(&p).is_err());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn digest_covers_the_first_lines_with_newlines() {
        let s = split("a\nb\nc\n");
        let want = hex::encode(Sha256::digest(b"a\nb\n"));
        assert_eq!(s.sha256_of_first(2), want);
    }

    #[test]
    fn summary_counts_and_detects_done() {
        let text = "{\"type\":\"slice_started\"}\n{\"type\":\"slice_done\"}\n{\"ty";
        let s = summarize("jvm", text);
        assert_eq!((s.lines, s.done, s.torn_tail), (2, true, true));
    }
}
