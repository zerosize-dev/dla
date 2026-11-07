//! Minimal, read-only git lookups (no git binary, no network).

use std::{
    fs,
    path::{Path, PathBuf},
};

/// Walks up from `start` looking for a `.git` entry.
pub fn find_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|d| d.join(".git").exists())
        .map(Path::to_path_buf)
}

fn is_hash(s: &str) -> bool {
    matches!(s.len(), 40 | 64) && s.bytes().all(|b| b.is_ascii_hexdigit())
}

fn resolve_gitdir(project: &Path) -> Option<PathBuf> {
    let dotgit = project.join(".git");
    if dotgit.is_dir() {
        return Some(dotgit);
    }
    let text = fs::read_to_string(&dotgit).ok()?;
    let rel = text.trim().strip_prefix("gitdir:")?.trim();
    Some(project.join(rel))
}

/// The commit `HEAD` points at, if it can be read from the `.git` directory.
pub fn head_commit(project: &Path) -> Option<String> {
    let gitdir = resolve_gitdir(project)?;
    let head = fs::read_to_string(gitdir.join("HEAD")).ok()?;
    let head = head.trim();
    let Some(refname) = head.strip_prefix("ref:").map(str::trim) else {
        return is_hash(head).then(|| head.to_owned());
    };
    let mut dirs = vec![gitdir.clone()];
    if let Ok(c) = fs::read_to_string(gitdir.join("commondir")) {
        dirs.push(gitdir.join(c.trim()));
    }
    for d in &dirs {
        if let Ok(h) = fs::read_to_string(d.join(refname)) {
            let h = h.trim();
            if is_hash(h) {
                return Some(h.to_owned());
            }
        }
    }
    for d in &dirs {
        if let Ok(packed) = fs::read_to_string(d.join("packed-refs")) {
            for line in packed.lines() {
                if let Some((h, r)) = line.split_once(' ')
                    && r.trim() == refname
                    && is_hash(h)
                {
                    return Some(h.to_owned());
                }
            }
        }
    }
    None
}
