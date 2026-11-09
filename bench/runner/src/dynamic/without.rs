//! `dla-bench dynamic --without <tool>[,<tool>...]`: runs the policy with tools hidden.
//!
//! The hidden tools are removed from the `PATH` of every `dla` call by building a directory
//! of links to every executable of the current `PATH` except the named ones (matched on the
//! file name). Variables that point at a tool's home directory would bring a hidden tool
//! back, so they are unset for the same calls. A run with hidden tools must still complete,
//! and the coverage row of each hidden backend must read `missing`.

use std::{
    collections::BTreeSet,
    env,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::Res;

/// Variables that name the home of a tool and so bring a hidden tool back.
pub const TOOL_HOME_VARS: &[&str] = &[
    "JAVA_HOME",
    "JDK_HOME",
    "JRE_HOME",
    "GRAALVM_HOME",
    "GOROOT",
    "ASYNC_PROFILER_HOME",
    "ASPROF_HOME",
    "MAT_HOME",
    "NODE_HOME",
];

/// Whether `path` names a file a shell could run.
fn is_executable(path: &Path) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    if !meta.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

/// The file name without a Windows executable suffix.
fn tool_name(file: &str) -> &str {
    let lower = file.to_ascii_lowercase();
    for ext in [".exe", ".cmd", ".bat"] {
        if lower.ends_with(ext) {
            return &file[..file.len() - ext.len()];
        }
    }
    file
}

/// Builds `dest` with a link to every executable on `path` whose name is not in `without`.
/// The first entry of `path` wins for a name, as in a shell lookup. Returns the number of
/// links made.
pub fn build_path_dir(without: &[String], path: &OsStr, dest: &Path) -> Res<usize> {
    fs::create_dir_all(dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    let hidden: BTreeSet<&str> = without.iter().map(String::as_str).collect();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut made = 0;
    for dir in env::split_paths(path) {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        let mut names: Vec<_> = entries.filter_map(Result::ok).collect();
        names.sort_by_key(|e| e.file_name());
        for e in names {
            let file = e.file_name().to_string_lossy().into_owned();
            if hidden.contains(file.as_str()) || hidden.contains(tool_name(&file)) {
                continue;
            }
            let src = e.path();
            if !is_executable(&src) || !seen.insert(file.clone()) {
                continue;
            }
            link(&src, &dest.join(&file))?;
            made += 1;
        }
    }
    Ok(made)
}

#[cfg(unix)]
fn link(src: &Path, dst: &Path) -> Res<()> {
    std::os::unix::fs::symlink(src, dst).map_err(|e| format!("{}: {e}", dst.display()))
}

#[cfg(not(unix))]
fn link(_src: &Path, _dst: &Path) -> Res<()> {
    Err("--without needs symbolic links, which this platform build does not use".to_owned())
}

/// Checks the coverage manifest of a finished run: every row of a hidden backend must read
/// `missing`. Returns the failures and the number of hidden-backend rows seen.
pub fn check_coverage(run_dir: &Path, without: &[String]) -> (Vec<String>, usize) {
    let mut bad = Vec::new();
    let mut seen = 0;
    let Ok(text) = fs::read_to_string(run_dir.join("coverage.json")) else {
        return (bad, 0);
    };
    let Ok(cov) = serde_json::from_str::<Value>(&text) else {
        return (vec!["coverage.json is not JSON".to_owned()], 0);
    };
    let Some(workers) = cov["workers"].as_object() else {
        return (bad, 0);
    };
    for (worker, w) in workers {
        for tool in without {
            let row = &w["backends"][tool.as_str()];
            if row.is_null() {
                continue;
            }
            seen += 1;
            let status = row["status"].as_str().unwrap_or("");
            if status != "missing" {
                bad.push(format!(
                    "hidden tool {tool}: the {worker} coverage row reads {status:?}, expected \"missing\""
                ));
            }
        }
    }
    (bad, seen)
}

/// The directory `dla` calls get as `PATH`, under the work directory.
pub fn path_dir_of(work: &Path) -> PathBuf {
    work.join("bin-without")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn exe(dir: &Path, name: &str) {
        use std::os::unix::fs::PermissionsExt;
        fs::create_dir_all(dir).unwrap();
        let p = dir.join(name);
        fs::write(&p, "#!/bin/sh\nexit 0\n").unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn the_farm_holds_every_executable_but_the_hidden_ones() {
        let t = tempfile::tempdir().unwrap();
        let (a, b) = (t.path().join("a"), t.path().join("b"));
        exe(&a, "jcmd");
        exe(&a, "git");
        exe(&b, "git");
        exe(&b, "valgrind");
        fs::write(b.join("notes.txt"), "not executable").unwrap();
        let path = env::join_paths([&a, &b]).unwrap();
        let dest = t.path().join("farm");
        let n = build_path_dir(&["jcmd".into(), "valgrind".into()], &path, &dest).unwrap();
        assert_eq!(n, 1);
        let names: Vec<_> = fs::read_dir(&dest)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        assert_eq!(names, ["git"]);
        assert_eq!(fs::read_link(dest.join("git")).unwrap(), a.join("git"));
    }

    #[test]
    fn windows_suffixes_do_not_hide_the_name() {
        assert_eq!(tool_name("jcmd.EXE"), "jcmd");
        assert_eq!(tool_name("jcmd"), "jcmd");
    }

    #[test]
    fn coverage_rows_of_hidden_tools_must_read_missing() {
        let t = tempfile::tempdir().unwrap();
        fs::write(
            t.path().join("coverage.json"),
            r#"{"workers":{"jvm":{"backends":{"jcmd":{"status":"ran"},"jfr":{"status":"missing"},"git":{"status":"ran"}}}}}"#,
        )
        .unwrap();
        let (bad, seen) = check_coverage(t.path(), &["jcmd".into(), "jfr".into(), "memray".into()]);
        assert_eq!(seen, 2);
        assert_eq!(bad.len(), 1, "{bad:?}");
        assert!(bad[0].contains("jcmd") && bad[0].contains("\"ran\""));
    }

    #[test]
    fn a_missing_or_broken_manifest_adds_no_failure_but_garbage_is_named() {
        let t = tempfile::tempdir().unwrap();
        assert_eq!(check_coverage(t.path(), &["jcmd".into()]), (vec![], 0));
        fs::write(t.path().join("coverage.json"), "{").unwrap();
        assert_eq!(check_coverage(t.path(), &["jcmd".into()]).0.len(), 1);
    }
}
