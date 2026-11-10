//! Cloning a harness into the next harness ID.
//!
//! The clone has the manifest (new ID, new scratch path, every path of the old harness
//! directory re-pointed), the probes, `agent/` and `driver/` as they are, and `work/`
//! without build outputs. It has no build marker, so the first run builds it. The project and
//! the old harness are only read.

use std::{
    fs,
    path::{Path, PathBuf},
};

use dla_core::{
    Error, Refusal,
    harness::{HARNESS_DIR, HarnessManifest},
    toolctx::ToolCtx,
};
use dla_workload::harness::{AGENT, DRIVER, PROBES, WORK, harness_dir, scratch::copy_tree};
use serde_json::Value;

/// The clone.
#[derive(Debug, Clone)]
pub struct Cloned {
    /// The new manifest.
    pub manifest: HarnessManifest,
    /// The clone's `work/` directory.
    pub work: PathBuf,
    /// Files copied into `work/`.
    pub files: usize,
}

/// True when `c` can continue a harness ID (`H1` must not match inside `H12`).
fn id_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

/// Replaces `harness/<old>` and `harness\<old>` followed by a non-ID character with the
/// same path for `new`.
pub fn repoint(text: &str, old: &str, new: &str) -> String {
    let mut out = text.to_owned();
    for sep in ['/', '\\'] {
        let needle = format!("{HARNESS_DIR}{sep}{old}");
        let with = format!("{HARNESS_DIR}{sep}{new}");
        let mut res = String::with_capacity(out.len());
        let mut rest = out.as_str();
        while let Some(i) = rest.find(&needle) {
            let after = &rest[i + needle.len()..];
            res.push_str(&rest[..i]);
            if after.chars().next().is_some_and(id_char) {
                res.push_str(&needle);
            } else {
                res.push_str(&with);
            }
            rest = after;
        }
        res.push_str(rest);
        out = res;
    }
    out
}

fn repoint_value(v: &mut Value, old: &str, new: &str) {
    match v {
        Value::String(s) => {
            if s.contains(HARNESS_DIR) {
                *s = repoint(s, old, new);
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|x| repoint_value(x, old, new)),
        Value::Object(m) => m.values_mut().for_each(|x| repoint_value(x, old, new)),
        _ => {}
    }
}

fn copy_verbatim(src: &Path, dest: &Path) -> std::io::Result<()> {
    if !src.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(dest)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        let (from, to) = (e.path(), dest.join(e.file_name()));
        let ft = e.file_type()?;
        if ft.is_dir() {
            copy_verbatim(&from, &to)?;
        } else if ft.is_symlink() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(fs::read_link(&from)?, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// Clones harness `src` into a new harness ID of the run.
pub fn clone_harness(tc: &ToolCtx, src: &HarnessManifest) -> Result<Cloned, Refusal> {
    let new_id = tc.next_harness_id();
    let from = harness_dir(&tc.run_dir, &src.id);
    let to = harness_dir(&tc.run_dir, &new_id);
    let io = |p: &Path, e: std::io::Error| Error::io(p, e).to_refusal();

    let mut value = serde_json::to_value(src).map_err(|e| {
        Error::Malformed {
            path: from.clone(),
            message: e.to_string(),
        }
        .to_refusal()
    })?;
    repoint_value(&mut value, &src.id, &new_id);
    let mut manifest: HarnessManifest = serde_json::from_value(value).map_err(|e| {
        Error::Malformed {
            path: to.clone(),
            message: e.to_string(),
        }
        .to_refusal()
    })?;
    manifest.id.clone_from(&new_id);
    manifest.scratch = format!("{HARNESS_DIR}/{new_id}/{WORK}");
    // Files a run wrote into the old harness are not part of the clone.
    manifest.captures.clear();

    let probes = from.join(PROBES);
    if probes.is_file() {
        let text = fs::read_to_string(&probes).map_err(|e| io(&probes, e))?;
        let dest = to.join(PROBES);
        fs::write(&dest, repoint(&text, &src.id, &new_id)).map_err(|e| io(&dest, e))?;
    }
    for sub in [AGENT, DRIVER] {
        copy_verbatim(&from.join(sub), &to.join(sub)).map_err(|e| io(&to.join(sub), e))?;
    }
    let work = to.join(WORK);
    let files = copy_tree(&from.join(WORK), &work).map_err(|e| io(&work, e))?;
    manifest.save(&tc.run_dir).map_err(|e| e.to_refusal())?;
    Ok(Cloned {
        manifest,
        work,
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repoint_changes_the_harness_path_and_not_a_longer_id() {
        let t = "/r/harness/H1/work /r/harness/H12/work harness/H1 harness\\H1\\agent";
        assert_eq!(
            repoint(t, "H1", "H4"),
            "/r/harness/H4/work /r/harness/H12/work harness/H4 harness\\H4\\agent"
        );
    }

    #[test]
    fn repoint_leaves_other_text_alone() {
        assert_eq!(repoint("no path here", "H1", "H2"), "no path here");
        assert_eq!(
            repoint("/r/harness/H2/out", "H1", "H3"),
            "/r/harness/H2/out"
        );
    }

    #[test]
    fn values_are_repointed_in_place() {
        let mut v = serde_json::json!({"a": ["/x/harness/H1/work"], "n": 3, "b": {"c": "keep"}});
        repoint_value(&mut v, "H1", "H2");
        assert_eq!(v["a"][0], "/x/harness/H2/work");
        assert_eq!(v["b"]["c"], "keep");
    }
}
