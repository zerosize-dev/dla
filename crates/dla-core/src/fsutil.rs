//! Small filesystem helpers shared by dla commands.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use serde::de::DeserializeOwned;

use crate::{
    error::Error,
    version::{Artifact, UpgradeError, upgrade},
};

/// Writes `bytes` to `path` atomically: a temporary file in the same directory is
/// written and synced, then renamed over `path`.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let tmp: PathBuf = dir.join(format!(".{name}.tmp.{}", std::process::id()));
    let result = (|| {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&tmp, path)
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(&tmp);
        return Err(Error::io(path, e));
    }
    Ok(())
}

/// Reads a versioned JSON document, applies migrations and deserialises it.
pub(crate) fn load_versioned<T: DeserializeOwned>(
    path: &Path,
    artifact: Artifact,
) -> Result<T, Error> {
    let text = fs::read_to_string(path).map_err(|e| Error::io(path, e))?;
    let malformed = |message: String| Error::Malformed {
        path: path.to_path_buf(),
        message,
    };
    let doc: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| malformed(e.to_string()))?;
    let what = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let doc = upgrade(doc, artifact, &what).map_err(|e| match e {
        UpgradeError::Malformed(m) => malformed(m),
        UpgradeError::Schema(s) => Error::Schema(s),
    })?;
    serde_json::from_value(doc).map_err(|e| malformed(e.to_string()))
}

/// Serialises `value` as pretty JSON with a trailing newline.
pub fn to_pretty_json<T: serde::Serialize>(value: &T) -> String {
    let mut s = serde_json::to_string_pretty(value).unwrap_or_default();
    s.push('\n');
    s
}

/// Name of the lock file inside a run directory.
pub const RUN_LOCK_FILE: &str = "run.lock";

/// Runs `f` while holding the exclusive lock on `<run_dir>/run.lock` (created when
/// missing). The lock covers every read-modify-write of `state.json` and
/// `coverage.json`; it is released when `f` returns and when the process dies, so a
/// killed worker never stalls the others. The lock is not re-entrant: do not call
/// [`crate::RunState::update`] or [`crate::CoverageManifest::update`] from inside `f`.
pub fn with_run_lock<T>(run_dir: &Path, f: impl FnOnce() -> Result<T, Error>) -> Result<T, Error> {
    let path = run_dir.join(RUN_LOCK_FILE);
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(&path)
        .map_err(|e| Error::io(&path, e))?;
    file.lock().map_err(|e| Error::io(&path, e))?;
    let out = f();
    let _ = file.unlock();
    out
}

/// Reserves the next `<prefix><n><suffix>` name in `dir` by creating it (an empty file,
/// or a directory when `dirs`), starting at the highest existing number plus one and
/// retrying the next number when another writer took it. The `dir` is created when
/// missing. Returns the reserved number and its path.
pub fn reserve_numbered(
    dir: &Path,
    prefix: char,
    suffix: &str,
    dirs: bool,
) -> Result<(u32, PathBuf), Error> {
    fs::create_dir_all(dir).map_err(|e| Error::io(dir, e))?;
    let mut n = crate::series::next_number(dir, prefix, suffix, dirs);
    loop {
        let path = dir.join(format!("{prefix}{n}{suffix}"));
        let made = if dirs {
            fs::create_dir(&path)
        } else {
            fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map(|_| ())
        };
        match made {
            Ok(()) => return Ok((n, path)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                n = n.checked_add(1).ok_or_else(|| {
                    Error::io(&path, std::io::Error::other("id counter overflow"))
                })?;
            }
            Err(e) => return Err(Error::io(&path, e)),
        }
    }
}

/// Whether the file at `path` exists and has content. A reserved-but-unwritten id
/// placeholder (see [`reserve_numbered`]) is an empty file and counts as absent.
pub fn is_written(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|m| m.len() > 0)
}
