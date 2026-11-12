//! `dla-bench dynamic --without <tool>[,<tool>...]`: runs the policy with tools hidden.
//!
//! The hidden tools are removed from the `PATH` of every `dla` call: a `PATH` directory that
//! holds a hidden tool is replaced by a directory of links to its other executables (matched
//! on the file name), the other directories are kept. Variables that point at the home of a
//! hidden tool would bring it back, so they are unset for the same calls. A run with hidden tools must still complete,
//! and the coverage row of each hidden backend must read `missing`.

use std::{
    collections::BTreeMap,
    env,
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::Res;

/// Variables that name the home of a tool, and the tools they bring back.
const TOOL_HOME_VARS: &[(&str, &[&str])] = &[
    ("JAVA_HOME", JDK_TOOLS),
    ("JDK_HOME", JDK_TOOLS),
    ("JRE_HOME", JDK_TOOLS),
    ("GRAALVM_HOME", JDK_TOOLS),
    ("GOROOT", &["go", "gofmt"]),
    ("ASYNC_PROFILER_HOME", &["asprof", "async-profiler"]),
    ("ASPROF_HOME", &["asprof", "async-profiler"]),
    ("MAT_HOME", &["mat", "ParseHeapDump"]),
    ("NODE_HOME", &["node", "npm", "npx"]),
];

const JDK_TOOLS: &[&str] = &[
    "java",
    "javac",
    "jcmd",
    "jfr",
    "jmap",
    "jstack",
    "jps",
    "jinfo",
    "jstat",
    "jdb",
    "jhsdb",
    "jlink",
    "jar",
    "native-image",
];

/// The tool-home variables to unset when `without` is hidden: only those that belong to a
/// hidden tool, so hiding `valgrind` leaves the JVM and Go runs alone.
pub fn home_vars_for(without: &[String]) -> Vec<&'static str> {
    TOOL_HOME_VARS
        .iter()
        .filter(|(_, tools)| {
            tools
                .iter()
                .any(|t| without.iter().any(|w| names_equal(w, t)))
        })
        .map(|(var, _)| *var)
        .collect()
}

/// Whether the coverage row `row` depends on the PATH tool `tool`: it is named after the tool
/// (`jcmd`) or starts with it and a dash (`jcmd-class-histogram`). The `harness-jfr` row does
/// not: the JVM writes the recording itself and only reading it needs the `jfr` tool.
pub fn row_depends_on(row: &str, tool: &str) -> bool {
    row == tool
        || row
            .strip_prefix(tool)
            .is_some_and(|rest| rest.starts_with('-'))
}

#[cfg(any(windows, target_os = "macos"))]
fn names_equal(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

#[cfg(not(any(windows, target_os = "macos")))]
fn names_equal(a: &str, b: &str) -> bool {
    a == b
}

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
        has_windows_exec_ext(&path.to_string_lossy())
    }
}

/// Windows executable suffixes (a fixed PATHEXT), matched without case.
const WINDOWS_EXTS: &[&str] = &[".exe", ".cmd", ".bat", ".com"];

/// Whether `file` ends in a Windows executable suffix. Kept free of cfg so unix tests cover it.
#[cfg_attr(unix, allow(dead_code))]
fn has_windows_exec_ext(file: &str) -> bool {
    let lower = file.to_ascii_lowercase();
    WINDOWS_EXTS.iter().any(|ext| lower.ends_with(ext))
}

/// The file name without a Windows executable suffix.
fn tool_name(file: &str) -> &str {
    let lower = file.to_ascii_lowercase();
    for ext in WINDOWS_EXTS {
        if lower.ends_with(ext) {
            return &file[..file.len() - ext.len()];
        }
    }
    file
}

fn is_hidden(without: &[String], file: &str) -> bool {
    without
        .iter()
        .any(|w| names_equal(w, file) || names_equal(w, tool_name(file)))
}

/// The `PATH` a `--without` run uses, and how many links the farms hold.
pub struct HiddenPath {
    pub path: OsString,
    pub links: usize,
}

/// Builds the `PATH` for a run with `without` hidden. A `PATH` directory that holds no hidden
/// tool is kept as it is, in its place, so interpreters in a virtualenv or any other directory
/// behave as usual. A directory that holds a hidden tool is replaced by a directory under
/// `dest` with absolute links to its other executables (an interpreter of a virtualenv gets a
/// small wrapper instead, so it still finds its `pyvenv.cfg`).
pub fn build_path_dir(without: &[String], path: &OsStr, dest: &Path) -> Res<HiddenPath> {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut links = 0;
    for (i, dir) in env::split_paths(path).enumerate() {
        let Ok(entries) = fs::read_dir(&dir) else {
            dirs.push(dir);
            continue;
        };
        let mut names: Vec<_> = entries.filter_map(Result::ok).collect();
        names.sort_by_key(|e| e.file_name());
        let holds_hidden = names.iter().any(|e| {
            is_hidden(without, &e.file_name().to_string_lossy()) && is_executable(&e.path())
        });
        if !holds_hidden {
            dirs.push(dir);
            continue;
        }
        let abs = std::path::absolute(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let farm = dest.join(i.to_string());
        fs::create_dir_all(&farm).map_err(|e| format!("{}: {e}", farm.display()))?;
        let venv = abs.parent().is_some_and(|p| p.join("pyvenv.cfg").is_file());
        for e in names {
            let file = e.file_name().to_string_lossy().into_owned();
            if is_hidden(without, &file) {
                continue;
            }
            let src = abs.join(&file);
            if !is_executable(&src) {
                continue;
            }
            if venv && file.starts_with("python") {
                wrap(&src, &farm.join(&file))?;
            } else {
                link(&src, &farm.join(&file))?;
            }
            links += 1;
        }
        dirs.push(farm);
    }
    let path = env::join_paths(dirs).map_err(|e| format!("PATH: {e}"))?;
    Ok(HiddenPath { path, links })
}

#[cfg(unix)]
fn link(src: &Path, dst: &Path) -> Res<()> {
    std::os::unix::fs::symlink(src, dst).map_err(|e| format!("{}: {e}", dst.display()))
}

#[cfg(not(unix))]
fn link(src: &Path, dst: &Path) -> Res<()> {
    link_or_copy(src, dst)
}

/// A hard link, or a copy when the link is refused (another volume, no permission).
#[cfg_attr(unix, allow(dead_code))]
fn link_or_copy(src: &Path, dst: &Path) -> Res<()> {
    fs::hard_link(src, dst)
        .or_else(|_| fs::copy(src, dst).map(|_| ()))
        .map_err(|e| format!("{}: {e}", dst.display()))
}

/// A script that runs `src` under the name it is called by in the farm. Python looks for
/// `pyvenv.cfg` next to the path it was started with, so a link would lose the virtualenv.
#[cfg(unix)]
fn wrap(src: &Path, dst: &Path) -> Res<()> {
    use std::os::unix::fs::PermissionsExt;
    let quoted = src.to_string_lossy().replace('\'', "'\\''");
    fs::write(dst, format!("#!/bin/sh\nexec '{quoted}' \"$@\"\n"))
        .and_then(|()| fs::set_permissions(dst, fs::Permissions::from_mode(0o755)))
        .map_err(|e| format!("{}: {e}", dst.display()))
}

#[cfg(not(unix))]
fn wrap(src: &Path, dst: &Path) -> Res<()> {
    link(src, dst)
}

/// Checks the coverage manifest of a finished run: every row that depends on a hidden tool
/// must read `missing`. Returns the failures and, per hidden tool, the number of rows seen.
pub fn check_coverage(
    run_dir: &Path,
    without: &[String],
) -> (Vec<String>, BTreeMap<String, usize>) {
    let mut bad = Vec::new();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    let Ok(text) = fs::read_to_string(run_dir.join("coverage.json")) else {
        return (bad, seen);
    };
    let Ok(cov) = serde_json::from_str::<Value>(&text) else {
        return (vec!["coverage.json is not JSON".to_owned()], seen);
    };
    let Some(workers) = cov["workers"].as_object() else {
        return (bad, seen);
    };
    for (worker, w) in workers {
        let Some(rows) = w["backends"].as_object() else {
            continue;
        };
        for tool in without {
            for (name, row) in rows {
                if row.is_null() || !row_depends_on(name, tool) {
                    continue;
                }
                *seen.entry(tool.clone()).or_default() += 1;
                let status = row["status"].as_str().unwrap_or("");
                if status != "missing" {
                    bad.push(format!(
                        "hidden tool {tool}: the {worker} coverage row {name} reads {status:?}, expected \"missing\""
                    ));
                }
            }
        }
    }
    (bad, seen)
}

/// The capture run of the per-operation policy that needs `tool`, as the suffix of the
/// `FixtureRow::captures` entries (`H1:capture-all`, `H1:tool-valgrind`). `jfr` records a
/// flight recording and reads it; `heaptrack_print` reads the heaptrack output.
fn capture_needing(tool: &str) -> Option<&'static str> {
    match tool {
        "jfr" => Some(":capture-all"),
        "valgrind" => Some(":tool-valgrind"),
        "heaptrack" | "heaptrack_print" => Some(":tool-heaptrack"),
        _ => None,
    }
}

/// Per hidden tool: when the policy ran the capture that needs it, the run must have written
/// a coverage row for the tool (`seen` counts the rows of [`check_coverage`]); the row itself
/// is checked to read `missing` there. `heaptrack_print` is looked up after `heaptrack`, so it
/// is only expected when `heaptrack` is not hidden too.
pub fn check_attempted(
    captures: &[String],
    seen: &BTreeMap<String, usize>,
    without: &[String],
) -> Vec<String> {
    without
        .iter()
        .filter_map(|tool| {
            if tool == "heaptrack_print" && without.iter().any(|w| w == "heaptrack") {
                return None;
            }
            let suffix = capture_needing(tool)?;
            let ran = captures.iter().any(|c| c.ends_with(suffix));
            (ran && seen.get(tool).copied().unwrap_or(0) == 0).then(|| {
                format!(
                    "hidden tool {tool}: the run tried the capture that needs it but wrote no coverage row for it"
                )
            })
        })
        .collect()
}

/// The first hidden tool that a fixture's `[toolchain]` would bring back: an executable entry
/// of a hidden name in one of its `PATH` directories (on Windows also `<name>.exe`, and the
/// other suffixes of a fixed `PATHEXT`; a file that is not executable does not count), or a tool-home variable (`JAVA_HOME`, `GOROOT`)
/// of a hidden tool. Returns the tool and where it comes from.
pub fn provided_conflict(
    without: &[String],
    path_first: &[PathBuf],
    env: &[(String, String)],
) -> Option<(String, String)> {
    for dir in path_first {
        let Ok(entries) = fs::read_dir(dir) else {
            continue;
        };
        let mut names: Vec<String> = entries
            .flatten()
            .filter(|e| is_executable(&e.path()))
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        for name in names {
            if is_hidden(without, &name) {
                return Some((
                    tool_name(&name).to_owned(),
                    format!("the bin directory {}", dir.display()),
                ));
            }
        }
    }
    for (var, tools) in TOOL_HOME_VARS {
        if !env.iter().any(|(k, _)| k == var) {
            continue;
        }
        if let Some(w) = without
            .iter()
            .find(|w| tools.iter().any(|t| names_equal(w, t)))
        {
            return Some((w.clone(), (*var).to_owned()));
        }
    }
    None
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
    fn a_toolchain_that_provides_a_hidden_tool_is_a_conflict() {
        let t = tempfile::tempdir().unwrap();
        let bin = t.path().join("jdk/bin");
        exe(&bin, "java");
        exe(&bin, "jcmd");
        let env = vec![("JAVA_HOME".to_owned(), "/x/jdk".to_owned())];
        let w = |v: &[&str]| v.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let c = provided_conflict(&w(&["jcmd"]), std::slice::from_ref(&bin), &[]).unwrap();
        assert_eq!(c.0, "jcmd");
        assert!(c.1.contains("jdk/bin"), "{c:?}");
        // The home variable alone brings the tool back too.
        let c = provided_conflict(&w(&["jmap"]), &[], &env).unwrap();
        assert_eq!((c.0.as_str(), c.1.as_str()), ("jmap", "JAVA_HOME"));
        // A tool the toolchain does not provide stays hideable.
        assert!(provided_conflict(&w(&["valgrind"]), std::slice::from_ref(&bin), &env).is_none());
        // A file of a hidden name that is not executable (a doc, a data file) is no tool.
        fs::write(bin.join("valgrind"), "notes").unwrap();
        assert!(provided_conflict(&w(&["valgrind"]), &[bin], &env).is_none());
    }

    #[cfg(windows)]
    #[test]
    fn a_toolchain_exe_of_a_hidden_name_is_a_conflict_on_windows() {
        let t = tempfile::tempdir().unwrap();
        let bin = t.path().join("jdk/bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join("jcmd.exe"), "").unwrap();
        fs::write(bin.join("README.txt"), "").unwrap();
        let c = provided_conflict(&["jcmd".to_owned()], std::slice::from_ref(&bin), &[]).unwrap();
        assert_eq!(c.0, "jcmd");
        assert!(provided_conflict(&["README".to_owned()], &[bin], &[]).is_none());
    }

    #[cfg(unix)]
    #[test]
    fn only_directories_with_a_hidden_tool_are_farmed() {
        let t = tempfile::tempdir().unwrap();
        let (a, b, c) = (t.path().join("a"), t.path().join("b"), t.path().join("c"));
        exe(&a, "jcmd");
        exe(&a, "git");
        exe(&b, "git");
        exe(&b, "valgrind");
        fs::write(b.join("notes.txt"), "not executable").unwrap();
        exe(&c, "ls");
        let path = env::join_paths([&a, &b, &c]).unwrap();
        let dest = t.path().join("farm");
        let h = build_path_dir(&["jcmd".into(), "valgrind".into()], &path, &dest).unwrap();
        let dirs: Vec<PathBuf> = env::split_paths(&h.path).collect();
        assert_eq!(dirs, [dest.join("0"), dest.join("1"), c.clone()]);
        assert_eq!(h.links, 2);
        let names = |d: &Path| -> Vec<String> {
            fs::read_dir(d)
                .unwrap()
                .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
                .collect()
        };
        assert_eq!(names(&dirs[0]), ["git"]);
        assert_eq!(names(&dirs[1]), ["git"]);
        assert_eq!(fs::read_link(dest.join("0/git")).unwrap(), a.join("git"));
    }

    #[cfg(unix)]
    #[test]
    fn a_relative_path_entry_does_not_hide_a_later_tool() {
        let t = tempfile::tempdir().unwrap();
        let cwd = env::current_dir().unwrap();
        let rel = cwd.join("target/without-rel-test");
        let _ = fs::remove_dir_all(&rel);
        exe(&rel, "mytool");
        exe(&rel, "jcmd");
        let abs = t.path().join("abs");
        exe(&abs, "mytool");
        let rel_entry = rel.strip_prefix(&cwd).unwrap().to_path_buf();
        let path = env::join_paths([rel_entry, abs.clone()]).unwrap();
        let dest = t.path().join("farm");
        let h = build_path_dir(&["jcmd".into()], &path, &dest).unwrap();
        let link = dest.join("0/mytool");
        let target = fs::read_link(&link).unwrap();
        assert!(target.is_absolute(), "{target:?}");
        assert!(link.exists(), "the link dangles");
        assert_eq!(env::split_paths(&h.path).nth(1).unwrap(), abs);
        let _ = fs::remove_dir_all(&rel);
    }

    #[cfg(unix)]
    #[test]
    fn a_venv_interpreter_keeps_its_virtualenv() {
        let t = tempfile::tempdir().unwrap();
        let bin = t.path().join("venv/bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(t.path().join("venv/pyvenv.cfg"), "home = /usr/bin\n").unwrap();
        exe(&bin, "jcmd");
        exe(&bin, "python3");
        let path = env::join_paths([&bin]).unwrap();
        let dest = t.path().join("farm");
        build_path_dir(&["jcmd".into()], &path, &dest).unwrap();
        let wrapper = fs::read_to_string(dest.join("0/python3")).unwrap();
        assert!(wrapper.contains(&format!("exec '{}'", bin.join("python3").display())));
        assert!(fs::read_link(dest.join("0/python3")).is_err(), "not a link");
    }

    #[test]
    fn home_variables_follow_the_hidden_tools() {
        assert!(home_vars_for(&["valgrind".into()]).is_empty());
        assert!(home_vars_for(&["jcmd".into()]).contains(&"JAVA_HOME"));
        assert_eq!(home_vars_for(&["go".into()]), ["GOROOT"]);
    }

    #[test]
    fn rows_are_matched_by_tool_prefix() {
        assert!(row_depends_on("jcmd", "jcmd"));
        assert!(row_depends_on("jcmd-class-histogram", "jcmd"));
        assert!(row_depends_on("jfr-old-object-sample", "jfr"));
        assert!(!row_depends_on("harness-jfr", "jfr"));
        assert!(!row_depends_on("jcmdx", "jcmd"));
        assert!(!row_depends_on("dla-hprof", "jcmd"));
    }

    #[test]
    fn an_attempted_capture_needs_a_row_for_its_hidden_tool() {
        let caps = vec!["H1:capture-all".to_owned(), "H2:tool-heaptrack".to_owned()];
        let none = BTreeMap::new();
        let w = |t: &[&str]| t.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        // jfr and heaptrack were attempted and left no row; valgrind was never attempted.
        let bad = check_attempted(&caps, &none, &w(&["jfr", "heaptrack", "valgrind", "jcmd"]));
        assert_eq!(bad.len(), 2, "{bad:?}");
        assert!(bad[0].contains("jfr") && bad[1].contains("heaptrack"));
        // heaptrack_print rides on the heaptrack run.
        assert_eq!(
            check_attempted(&caps, &none, &w(&["heaptrack_print"])).len(),
            1
        );
        // ... unless heaptrack itself is hidden: the run stops before it needs the printer.
        let both = w(&["heaptrack", "heaptrack_print"]);
        assert_eq!(check_attempted(&caps, &none, &both).len(), 1);
        // A row (any status; check_coverage judges it) satisfies the expectation.
        let seen = BTreeMap::from([("jfr".to_owned(), 1)]);
        assert!(check_attempted(&caps, &seen, &w(&["jfr"])).is_empty());
        assert!(check_attempted(&[], &none, &w(&["jfr", "valgrind"])).is_empty());
    }

    #[test]
    fn windows_suffixes_do_not_hide_the_name() {
        assert_eq!(tool_name("jcmd.EXE"), "jcmd");
        assert_eq!(tool_name("jcmd.com"), "jcmd");
        assert_eq!(tool_name("jcmd"), "jcmd");
    }

    #[test]
    fn windows_executables_are_found_by_suffix_without_case() {
        for f in ["JCMD.EXE", "a.cmd", "b.Bat", "c.COM"] {
            assert!(has_windows_exec_ext(f), "{f}");
        }
        for f in ["notes.txt", "jcmd", "x.dll", "exe"] {
            assert!(!has_windows_exec_ext(f), "{f}");
        }
        // The hidden-name match on a Windows file name, case-insensitive on the suffix.
        assert!(tool_name("JCMD.EXE").eq_ignore_ascii_case("jcmd"));
    }

    #[test]
    fn the_windows_farm_falls_back_to_a_hard_link_or_copy() {
        let t = tempfile::tempdir().unwrap();
        let src = t.path().join("tool.exe");
        fs::write(&src, "bin").unwrap();
        let dst = t.path().join("farm-tool.exe");
        link_or_copy(&src, &dst).unwrap();
        assert_eq!(fs::read_to_string(&dst).unwrap(), "bin");
        // A missing source fails with the destination named.
        let err = link_or_copy(&t.path().join("none.exe"), &t.path().join("x.exe")).unwrap_err();
        assert!(err.contains("x.exe"), "{err}");
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
        assert_eq!(seen.values().sum::<usize>(), 2);
        assert_eq!(bad.len(), 1, "{bad:?}");
        assert!(bad[0].contains("jcmd") && bad[0].contains("\"ran\""));
    }

    #[test]
    fn a_missing_or_broken_manifest_adds_no_failure_but_garbage_is_named() {
        let t = tempfile::tempdir().unwrap();
        assert_eq!(
            check_coverage(t.path(), &["jcmd".into()]),
            (vec![], BTreeMap::new())
        );
        fs::write(t.path().join("coverage.json"), "{").unwrap();
        assert_eq!(check_coverage(t.path(), &["jcmd".into()]).0.len(), 1);
    }
}
