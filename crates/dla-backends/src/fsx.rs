//! File-system helpers: tool discovery, project walking, path normalisation.

use std::{
    collections::HashMap,
    fs,
    path::{Component, Path, PathBuf},
};

/// Directories no analysis descends into.
pub const SKIP_DIRS: [&str; 12] = [
    ".git",
    ".dla",
    "target",
    "build",
    "node_modules",
    "venv",
    ".venv",
    "out",
    "dist",
    "__pycache__",
    ".gradle",
    ".idea",
];

/// Where tools are looked for.
#[derive(Debug, Clone, Default)]
pub struct ToolEnv {
    /// `$DLA_DATA_DIR/tools`.
    pub tools_dir: Option<PathBuf>,
    /// Directories of `PATH`.
    pub path: Vec<PathBuf>,
    /// Extra variables that matter to discovery (`DLA_<TOOL>_HOME`, `JAVA_HOME`).
    pub vars: HashMap<String, String>,
}

/// Sub-directories of the tools directory that may hold executables.
const BIN_SUBDIRS: [&str; 7] = [
    "bin",
    "venv/bin",
    "venv/Scripts",
    "node/node_modules/.bin",
    "rapx/bin",
    "codeql/codeql",
    "codeql",
];

impl ToolEnv {
    /// Reads `PATH` and the discovery variables from the process, tools dir from `ctx`.
    pub fn from_process(tools_dir: Option<PathBuf>) -> Self {
        let path = std::env::var_os("PATH")
            .map(|p| std::env::split_paths(&p).collect())
            .unwrap_or_default();
        let mut vars = HashMap::new();
        for (k, v) in std::env::vars() {
            if k == "JAVA_HOME"
                || k == crate::sweep::ENV_TIMEOUT
                || (k.starts_with("DLA_") && k.ends_with("_HOME"))
            {
                vars.insert(k, v);
            }
        }
        Self {
            tools_dir,
            path,
            vars,
        }
    }

    /// The first executable named `name`: tools directory, then `PATH`.
    pub fn find_program(&self, name: &str) -> Option<PathBuf> {
        if let Some(t) = &self.tools_dir {
            for sub in BIN_SUBDIRS {
                if let Some(p) = exe_in(&t.join(sub), name) {
                    return Some(p);
                }
            }
        }
        self.path.iter().find_map(|d| exe_in(d, name))
    }

    /// The first of `names` that resolves.
    pub fn find_any(&self, names: &[&str]) -> Option<PathBuf> {
        names.iter().find_map(|n| self.find_program(n))
    }

    /// The directory holding a jar-based tool: `<tools>/jars/<tool>`, else the
    /// directory named by `DLA_<TOOL>_HOME`.
    pub fn jar_dir(&self, tool: &str) -> Option<PathBuf> {
        if let Some(t) = &self.tools_dir {
            let d = t.join("jars").join(tool);
            if has_jars(&d) {
                return Some(d);
            }
        }
        let key = format!("DLA_{}_HOME", tool.to_uppercase().replace('-', "_"));
        let d = PathBuf::from(self.vars.get(&key)?);
        has_jars(&d).then_some(d)
    }

    /// A JDK tool (`javac`, `java`): `JAVA_HOME/bin`, then `PATH`.
    pub fn find_jdk_tool(&self, name: &str) -> Option<PathBuf> {
        if let Some(h) = self.vars.get("JAVA_HOME")
            && let Some(p) = exe_in(&Path::new(h).join("bin"), name)
        {
            return Some(p);
        }
        self.find_program(name)
    }
}

fn has_jars(d: &Path) -> bool {
    fs::read_dir(d).is_ok_and(|rd| {
        rd.flatten()
            .any(|e| e.path().extension().is_some_and(|x| x == "jar"))
    })
}

fn exe_in(dir: &Path, name: &str) -> Option<PathBuf> {
    let candidates: &[&str] = if cfg!(windows) {
        &["", ".exe", ".cmd", ".bat"]
    } else {
        &[""]
    };
    for suffix in candidates {
        let p = dir.join(format!("{name}{suffix}"));
        if p.is_file() && is_executable(&p) {
            return Some(p);
        }
    }
    None
}

#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(p).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(_p: &Path) -> bool {
    true
}

/// The jars of `dir`, sorted by file name.
pub fn jars_in(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|x| x == "jar"))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// True when a directory name is never descended into.
pub fn skipped(name: &str) -> bool {
    SKIP_DIRS.contains(&name)
}

/// Every file under `root` (relative `/` paths, sorted) for which `keep(rel)` holds,
/// without descending into [`SKIP_DIRS`]. At most `limit` files.
pub fn walk(root: &Path, limit: usize, keep: &dyn Fn(&str) -> bool) -> Vec<String> {
    walk_with(root, limit, &skipped, keep)
}

/// Like [`walk`] with a custom directory filter: `skip(name)` true means do not
/// descend into a directory of that name.
pub fn walk_with(
    root: &Path,
    limit: usize,
    skip: &dyn Fn(&str) -> bool,
    keep: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![(root.to_path_buf(), String::new())];
    while let Some((dir, rel)) = stack.pop() {
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        let mut entries: Vec<_> = rd.flatten().collect();
        entries.sort_by_key(|e| e.file_name());
        for e in entries {
            let name = e.file_name().to_string_lossy().into_owned();
            let Ok(ft) = e.file_type() else { continue };
            let r = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            if ft.is_dir() {
                if !skip(&name) {
                    stack.push((e.path(), r));
                }
            } else if ft.is_file() && keep(&r) {
                out.push(r);
                if out.len() >= limit {
                    out.sort();
                    return out;
                }
            }
        }
    }
    out.sort();
    out
}

/// Files of `root` whose extension is one of `exts` (without the dot).
pub fn files_with_ext(root: &Path, exts: &[&str]) -> Vec<String> {
    walk(root, 200_000, &|r| {
        Path::new(r)
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| exts.contains(&e))
    })
}

/// Copies `src` to `dst`, leaving out [`SKIP_DIRS`] (a project copy for tools that
/// write next to their input).
pub fn copy_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        let name = e.file_name();
        let ft = e.file_type()?;
        if ft.is_dir() {
            if !skipped(&name.to_string_lossy()) {
                copy_tree(&e.path(), &dst.join(&name))?;
            }
        } else if ft.is_file() {
            fs::copy(e.path(), dst.join(&name))?;
        }
    }
    Ok(())
}

/// Collapses `.` and `..` lexically.
pub fn normalize(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                if !out.pop() {
                    out.push("..");
                }
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

/// Turns the paths tools print into project-relative `/` paths.
#[derive(Debug, Clone)]
pub struct PathMap {
    /// Roots whose absolute paths map to the project (the project, its canonical
    /// form, a copy of it).
    roots: Vec<PathBuf>,
    /// Directories relative paths are resolved against (the tool's working directory).
    bases: Vec<PathBuf>,
}

impl PathMap {
    /// Paths under `project` (resolved from `bases`, default the project itself).
    pub fn new(project: &Path) -> Self {
        let mut roots = vec![project.to_path_buf()];
        if let Ok(c) = project.canonicalize()
            && c != project
        {
            roots.push(c);
        }
        Self {
            roots,
            bases: vec![project.to_path_buf()],
        }
    }

    /// Also treats `copy` as the project: absolute paths under it map to the project.
    pub fn with_copy(mut self, copy: &Path) -> Self {
        self.roots.push(copy.to_path_buf());
        if let Ok(c) = copy.canonicalize() {
            self.roots.push(c);
        }
        self
    }

    /// Resolves relative paths against `base` instead of the project root. The
    /// base is a directory inside a root.
    pub fn with_base(mut self, base: &Path) -> Self {
        self.bases = vec![base.to_path_buf()];
        self
    }

    /// The project-relative path for what a tool printed, or `None` when it points
    /// outside the project or into `.dla`.
    pub fn rel(&self, raw: &str) -> Option<String> {
        let mut s = raw.trim().to_owned();
        if let Some(r) = s.strip_prefix("file://") {
            s = r.to_owned();
            if s.starts_with("//") {
                s = s[2..].to_owned();
            }
            // file:///C:/x -> C:/x
            if s.len() > 2 && s.starts_with('/') && s.as_bytes()[2] == b':' {
                s = s[1..].to_owned();
            }
        } else if let Some(r) = s.strip_prefix("file:") {
            s = r.to_owned();
        }
        s = percent_decode(&s).replace('\\', "/");
        if s.is_empty() {
            return None;
        }
        let p = PathBuf::from(&s);
        let abs = if p.is_absolute() || (s.len() > 2 && s.as_bytes()[1] == b':') {
            normalize(&p)
        } else {
            normalize(&self.bases[0].join(&p))
        };
        for root in &self.roots {
            if let Ok(r) = abs.strip_prefix(normalize(root)) {
                return clean_rel(r);
            }
        }
        None
    }
}

fn clean_rel(r: &Path) -> Option<String> {
    let s = r
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/");
    if s.is_empty() || s == ".dla" || s.starts_with(".dla/") || s.starts_with("../") {
        None
    } else {
        Some(s)
    }
}

fn percent_decode(s: &str) -> String {
    if !s.contains('%') {
        return s.to_owned();
    }
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%'
            && i + 2 < b.len()
            && let Some(v) = s
                .get(i + 1..i + 3)
                .and_then(|h| u8::from_str_radix(h, 16).ok())
        {
            out.push(v);
            i += 3;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Finds the project file a tool named by a partial path (`com/acme/R.java`, `R.java`):
/// the first `files` entry that equals it or ends with `/<partial>`.
pub fn resolve_suffix(files: &[String], partial: &str) -> Option<String> {
    let p = partial.trim_start_matches("./").replace('\\', "/");
    let tail = format!("/{p}");
    files
        .iter()
        .find(|f| **f == p || f.ends_with(&tail))
        .cloned()
}

/// Project-relative source roots implied by `package` lines of Java/Kotlin files.
pub fn source_roots(project: &Path, files: &[String]) -> Vec<String> {
    let mut roots: Vec<String> = Vec::new();
    for f in files {
        let Ok(text) = fs::read_to_string(project.join(f)) else {
            continue;
        };
        let pkg = text.lines().take(60).find_map(|l| {
            let t = l.trim();
            let rest = t.strip_prefix("package ")?;
            Some(rest.trim_end_matches(';').trim().replace('`', ""))
        });
        let dir = f.rsplit_once('/').map_or("", |(d, _)| d);
        let root = match pkg {
            Some(p) if !p.is_empty() => {
                let want = p.replace('.', "/");
                match dir.strip_suffix(&want) {
                    Some(r) => r.trim_end_matches('/').to_owned(),
                    None => dir.to_owned(),
                }
            }
            _ => dir.to_owned(),
        };
        if !roots.contains(&root) {
            roots.push(root);
        }
    }
    roots.sort();
    roots
}

/// Writes `text` to `path`, creating parents.
pub fn write(path: &Path, text: &str) -> std::io::Result<()> {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p)?;
    }
    fs::write(path, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rel_maps_absolute_relative_and_uris() {
        let d = tempfile::tempdir().unwrap();
        let pm = PathMap::new(d.path());
        let abs = d.path().join("src/a.go");
        assert_eq!(pm.rel(abs.to_str().unwrap()).as_deref(), Some("src/a.go"));
        assert_eq!(pm.rel("./src/a.go").as_deref(), Some("src/a.go"));
        assert_eq!(pm.rel("src/../src/a.go").as_deref(), Some("src/a.go"));
        let uri = format!("file://{}", abs.display());
        assert_eq!(pm.rel(&uri).as_deref(), Some("src/a.go"));
        assert_eq!(pm.rel("/etc/passwd"), None);
        assert_eq!(pm.rel("../elsewhere/a.go"), None);
        assert_eq!(pm.rel(".dla/r1/tmp/x.go"), None);
        assert_eq!(pm.rel(""), None);
    }

    #[test]
    fn rel_handles_percent_escapes_and_copies() {
        let d = tempfile::tempdir().unwrap();
        let c = tempfile::tempdir().unwrap();
        let pm = PathMap::new(d.path()).with_copy(c.path());
        let abs = c.path().join("my dir/a.rs");
        let uri = format!("file://{}", abs.to_str().unwrap().replace(' ', "%20"));
        assert_eq!(pm.rel(&uri).as_deref(), Some("my dir/a.rs"));
    }

    #[test]
    fn suffix_resolution_prefers_exact_or_trailing_match() {
        let files = vec![
            "src/main/java/com/acme/R.java".to_owned(),
            "src/test/java/com/acme/RTest.java".to_owned(),
        ];
        assert_eq!(
            resolve_suffix(&files, "com/acme/R.java").as_deref(),
            Some("src/main/java/com/acme/R.java")
        );
        assert_eq!(resolve_suffix(&files, "me/R.java"), None);
        assert_eq!(
            resolve_suffix(&files, "R.java").as_deref(),
            Some("src/main/java/com/acme/R.java")
        );
    }

    #[test]
    fn source_roots_follow_package_lines() {
        let d = tempfile::tempdir().unwrap();
        fs::create_dir_all(d.path().join("src/shop")).unwrap();
        fs::write(d.path().join("src/shop/A.java"), "package shop;\nclass A{}").unwrap();
        fs::write(d.path().join("B.java"), "class B{}").unwrap();
        let files = vec!["B.java".to_owned(), "src/shop/A.java".to_owned()];
        assert_eq!(source_roots(d.path(), &files), vec!["", "src"]);
    }

    #[test]
    fn walk_skips_generated_directories() {
        let d = tempfile::tempdir().unwrap();
        for p in ["a.py", "build/b.py", ".dla/x/c.py", "pkg/d.py", "venv/e.py"] {
            let f = d.path().join(p);
            fs::create_dir_all(f.parent().unwrap()).unwrap();
            fs::write(f, "").unwrap();
        }
        assert_eq!(files_with_ext(d.path(), &["py"]), vec!["a.py", "pkg/d.py"]);
    }

    #[test]
    fn find_program_checks_tools_dir_before_path() {
        let t = tempfile::tempdir().unwrap();
        let p = tempfile::tempdir().unwrap();
        for d in [t.path().join("venv/bin"), p.path().to_path_buf()] {
            fs::create_dir_all(&d).unwrap();
            let f = d.join("ruff");
            fs::write(&f, "#!/bin/sh\n").unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&f, fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        let env = ToolEnv {
            tools_dir: Some(t.path().to_path_buf()),
            path: vec![p.path().to_path_buf()],
            vars: HashMap::new(),
        };
        assert_eq!(
            env.find_program("ruff").unwrap(),
            t.path().join("venv/bin/ruff")
        );
        assert!(env.find_program("nope").is_none());
    }
}
