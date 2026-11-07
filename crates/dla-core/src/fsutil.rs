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
