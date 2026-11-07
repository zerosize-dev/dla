//! Where the plugin's data files and the downloaded analyzers live.
//!
//! Two directories matter:
//!
//! * the **data files** directory, `data/` of the plugin (detector tables, archetype
//!   tables), found by [`data_files_dir`];
//! * the **plugin data directory** (`$DLA_DATA_DIR`, the per-install writable directory
//!   `bin/dla` exports), whose `tools/` subdirectory holds analyzers installed by
//!   `dla install` ([`tools_dir`]).

use std::path::{Path, PathBuf};

use crate::error::{Refusal, codes};

/// Environment variable holding the plugin root (exported by `bin/dla`).
pub const ENV_PLUGIN_ROOT: &str = "DLA_PLUGIN_ROOT";
/// Environment variable holding the plugin data directory (exported by `bin/dla`).
pub const ENV_DATA_DIR: &str = "DLA_DATA_DIR";
/// Name of the subdirectory of the plugin data directory that holds installed analyzers.
pub const TOOLS_SUBDIR: &str = "tools";

fn env_nonempty(env: &dyn Fn(&str) -> Option<String>, name: &str) -> Option<PathBuf> {
    env(name)
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from)
}

fn real_env(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// The `data/` directory of the git checkout that contains `exe` (development
/// builds run from `<checkout>/target/<profile>/dla`).
fn checkout_data_dir(exe: &Path) -> Option<PathBuf> {
    for dir in exe.ancestors().skip(1) {
        if dir.join(".git").exists() {
            let data = dir.join("data");
            return data.is_dir().then_some(data);
        }
    }
    None
}

fn no_data_dir(tried: &str) -> Refusal {
    Refusal::new(
        codes::NO_DATA_DIR,
        format!("cannot find the plugin data files ({tried})"),
        "the plugin data directory (detector tables)",
        "dla --data <dir> ... where <dir> is the plugin's data/ directory, or run through bin/dla so DLA_PLUGIN_ROOT is set",
    )
}

/// Resolves the plugin data files directory with explicit inputs (testable form of
/// [`data_files_dir`]): `flag` (`--data`), else `$DLA_PLUGIN_ROOT/data`, else the
/// `data/` directory of the git checkout enclosing `exe`, else a refusal that names
/// the `--data` flag. A directory given by `flag` or the environment must exist.
pub fn resolve_data_files_dir(
    flag: Option<&Path>,
    env: &dyn Fn(&str) -> Option<String>,
    exe: Option<&Path>,
) -> Result<PathBuf, Refusal> {
    if let Some(f) = flag {
        return if f.is_dir() {
            Ok(f.to_path_buf())
        } else {
            Err(Refusal::new(
                codes::NO_DATA_DIR,
                format!("--data {} is not a directory", crate::paths::to_slash(f)),
                "an existing plugin data directory",
                "dla --data <dir> ...",
            ))
        };
    }
    if let Some(root) = env_nonempty(env, ENV_PLUGIN_ROOT) {
        let d = root.join("data");
        if d.is_dir() {
            return Ok(d);
        }
    }
    if let Some(d) = exe.and_then(checkout_data_dir) {
        return Ok(d);
    }
    Err(no_data_dir(
        "no --data flag, no $DLA_PLUGIN_ROOT/data, and the executable is not inside a checkout with a data/ directory",
    ))
}

/// The plugin data files directory for this process (see [`resolve_data_files_dir`]).
pub fn data_files_dir(flag: Option<&Path>) -> Result<PathBuf, Refusal> {
    let exe = std::env::current_exe().ok();
    resolve_data_files_dir(flag, &real_env, exe.as_deref())
}

/// `$DLA_DATA_DIR/tools`; `None` when `$DLA_DATA_DIR` is unset or empty.
pub fn resolve_tools_dir(env: &dyn Fn(&str) -> Option<String>) -> Option<PathBuf> {
    env_nonempty(env, ENV_DATA_DIR).map(|d| d.join(TOOLS_SUBDIR))
}

/// The analyzer tools directory for this process: `$DLA_DATA_DIR/tools`.
pub fn tools_dir() -> Option<PathBuf> {
    resolve_tools_dir(&real_env)
}

/// Renders `s` with `/` separators (the form every recorded path takes, whatever the OS).
pub fn slash(s: &str) -> String {
    s.replace('\\', "/")
}

/// Renders `path` as a `/`-separated string. Use for every path written to the ledger,
/// evidence, coverage, census, workload or probe output; keep [`Path`] for arguments
/// passed to child processes.
pub fn to_slash(path: &Path) -> String {
    slash(&path.to_string_lossy())
}

/// `path` relative to `root`, `/`-separated, or `None` when `path` is not under `root`.
pub fn rel_slash(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root).ok().map(to_slash)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
        let v: Vec<(String, String)> = pairs
            .iter()
            .map(|(a, b)| ((*a).to_owned(), (*b).to_owned()))
            .collect();
        move |k| v.iter().find(|(a, _)| a == k).map(|(_, b)| b.clone())
    }

    #[test]
    fn flag_wins_over_plugin_root_and_checkout() {
        let t = tempfile::tempdir().unwrap();
        let flag = t.path().join("flagdata");
        let root = t.path().join("root");
        std::fs::create_dir_all(&flag).unwrap();
        std::fs::create_dir_all(root.join("data")).unwrap();
        let env = env_of(&[(ENV_PLUGIN_ROOT, root.to_str().unwrap())]);
        let got = resolve_data_files_dir(Some(&flag), &env, None).unwrap();
        assert_eq!(got, flag);
    }

    #[test]
    fn missing_flag_directory_is_refused() {
        let t = tempfile::tempdir().unwrap();
        let env = env_of(&[]);
        let err = resolve_data_files_dir(Some(&t.path().join("nope")), &env, None).unwrap_err();
        assert_eq!(err.code, codes::NO_DATA_DIR);
        assert!(err.message.contains("--data"));
    }

    #[test]
    fn plugin_root_data_is_used_without_a_flag() {
        let t = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(t.path().join("data")).unwrap();
        let env = env_of(&[(ENV_PLUGIN_ROOT, t.path().to_str().unwrap())]);
        let got = resolve_data_files_dir(None, &env, None).unwrap();
        assert_eq!(got, t.path().join("data"));
    }

    #[test]
    fn dev_fallback_finds_the_enclosing_checkout() {
        let t = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(t.path().join(".git")).unwrap();
        std::fs::create_dir_all(t.path().join("data")).unwrap();
        std::fs::create_dir_all(t.path().join("target/debug")).unwrap();
        let exe = t.path().join("target/debug/dla");
        let env = env_of(&[(ENV_PLUGIN_ROOT, "")]);
        let got = resolve_data_files_dir(None, &env, Some(&exe)).unwrap();
        assert_eq!(got, t.path().join("data"));
    }

    #[test]
    fn nothing_found_names_the_flag() {
        let t = tempfile::tempdir().unwrap();
        let exe = t.path().join("bin/dla");
        let env = env_of(&[]);
        let err = resolve_data_files_dir(None, &env, Some(&exe)).unwrap_err();
        assert_eq!(err.code, codes::NO_DATA_DIR);
        assert!(err.fix.contains("--data"));
    }

    #[test]
    fn tools_dir_is_under_the_plugin_data_dir() {
        let env = env_of(&[(ENV_DATA_DIR, "/x/data")]);
        assert_eq!(
            resolve_tools_dir(&env),
            Some(Path::new("/x/data").join("tools"))
        );
        assert_eq!(resolve_tools_dir(&env_of(&[])), None);
        assert_eq!(resolve_tools_dir(&env_of(&[(ENV_DATA_DIR, " ")])), None);
    }

    #[test]
    fn slash_rewrites_backslashes() {
        assert_eq!(slash(r"src\main\A.java"), "src/main/A.java");
        assert_eq!(slash("already/fine"), "already/fine");
        assert_eq!(to_slash(Path::new(r"a\b/c")), "a/b/c");
    }

    #[test]
    fn rel_slash_is_relative_and_uses_forward_slashes() {
        let root = Path::new("proj");
        let file = root.join("src").join("lib").join("a.rs");
        assert_eq!(rel_slash(root, &file).as_deref(), Some("src/lib/a.rs"));
        assert_eq!(rel_slash(root, Path::new("other/a.rs")), None);
    }
}
