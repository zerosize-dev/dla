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

/// Build-output directory names: skipped only where they really are outputs.
const OUTPUT_DIRS: [&str; 4] = ["target", "build", "out", "dist"];

/// Copies `src` to `dst` (a project copy for tools that write next to their input).
/// VCS and run state (`.git`, `.dla`), dependency and environment directories are
/// never copied. Build-output directories (`target`, `build`, `out`, `dist`) are
/// left out only at the project root and next to a `Cargo.toml` (a crate root),
/// so a source module such as `src/build/` survives.
pub fn copy_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
    copy_dir(src, dst, true)
}

fn copy_dir(src: &Path, dst: &Path, top: bool) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    let crate_root = top || src.join("Cargo.toml").is_file();
    for e in fs::read_dir(src)? {
        let e = e?;
        let name = e.file_name();
        let n = name.to_string_lossy();
        let ft = e.file_type()?;
        if ft.is_dir() {
            let always = matches!(
                n.as_ref(),
                ".git"
                    | ".dla"
                    | "node_modules"
                    | "venv"
                    | ".venv"
                    | "__pycache__"
                    | ".idea"
                    | ".gradle"
            );
            if always || (crate_root && OUTPUT_DIRS.contains(&n.as_ref())) {
                continue;
            }
            copy_dir(&e.path(), &dst.join(&name), false)?;
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

/// Like [`resolve_suffix`], but when several files match prefers one under any of the
/// `prefixes` (module directories paired with the analysed class roots). `""` is
/// ignored: it would match everything.
pub fn resolve_suffix_in(files: &[String], partial: &str, prefixes: &[String]) -> Option<String> {
    let p = partial.trim_start_matches("./").replace('\\', "/");
    let tail = format!("/{p}");
    let mut first: Option<&String> = None;
    for f in files.iter().filter(|f| **f == p || f.ends_with(&tail)) {
        if prefixes
            .iter()
            .any(|x| !x.is_empty() && f.starts_with(&format!("{}/", x.trim_end_matches('/'))))
        {
            return Some(f.clone());
        }
        first.get_or_insert(f);
    }
    first.cloned()
}

/// The module directory a build-output path belongs to: everything before the first
/// `target`, `build`, `out` or `bin` component (`a/target/classes/x` -> `a`).
pub fn module_prefix(class_dir: &str) -> String {
    let comps: Vec<&str> = class_dir.split('/').collect();
    let i = comps
        .iter()
        .position(|c| matches!(*c, "target" | "build" | "out" | "bin"))
        .unwrap_or(comps.len());
    comps[..i].join("/")
}

/// The class-root-relative part of a class file path: what follows the `classes`
/// directory (and a Gradle `java|kotlin|scala/<set>` pair), or the whole path.
pub fn class_key(class_file: &str) -> &str {
    let comps: Vec<&str> = class_file.split('/').collect();
    let Some(i) = comps.iter().position(|c| *c == "classes") else {
        return class_file;
    };
    let mut skip = i + 1;
    if comps
        .get(skip)
        .is_some_and(|c| matches!(*c, "java" | "kotlin" | "scala"))
        && comps.get(skip + 1).is_some()
        && comps.len() > skip + 2
    {
        skip += 2;
    }
    let off: usize = comps[..skip].iter().map(|c| c.len() + 1).sum();
    class_file.get(off..).unwrap_or(class_file)
}

/// Splits `groups` (each a set of keys) into batches in which no key repeats, keeping
/// as many groups together as possible. Returns, per batch, the group indexes.
pub fn conflict_free_batches(groups: &[Vec<String>]) -> Vec<Vec<usize>> {
    let mut batches: Vec<(Vec<usize>, std::collections::HashSet<&str>)> = Vec::new();
    for (i, g) in groups.iter().enumerate() {
        let slot = batches
            .iter()
            .position(|(_, keys)| g.iter().all(|k| !keys.contains(k.as_str())));
        let idx = slot.unwrap_or_else(|| {
            batches.push((Vec::new(), std::collections::HashSet::new()));
            batches.len() - 1
        });
        batches[idx].0.push(i);
        batches[idx].1.extend(g.iter().map(String::as_str));
    }
    batches.into_iter().map(|b| b.0).collect()
}

/// The source root (project-relative) implied by the `package` line of a Java or
/// Kotlin file.
pub fn source_root_of(project: &Path, f: &str) -> Option<String> {
    let text = fs::read_to_string(project.join(f)).ok()?;
    let pkg = text.lines().take(60).find_map(|l| {
        let t = l.trim();
        let rest = t.strip_prefix("package ")?;
        Some(rest.trim_end_matches(';').trim().replace('`', ""))
    });
    let dir = f.rsplit_once('/').map_or("", |(d, _)| d);
    Some(match pkg {
        Some(p) if !p.is_empty() => {
            let want = p.replace('.', "/");
            match dir.strip_suffix(&want) {
                Some(r) => r.trim_end_matches('/').to_owned(),
                None => dir.to_owned(),
            }
        }
        _ => dir.to_owned(),
    })
}

/// Project-relative source roots implied by `package` lines of Java/Kotlin files.
pub fn source_roots(project: &Path, files: &[String]) -> Vec<String> {
    let mut roots: Vec<String> = Vec::new();
    for f in files {
        let Some(root) = source_root_of(project, f) else {
            continue;
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
    fn copy_tree_keeps_source_modules_named_like_outputs() {
        let d = tempfile::tempdir().unwrap();
        let src = d.path().join("p");
        for f in [
            "Cargo.toml",
            "src/main.rs",
            "src/build/mod.rs",
            "src/target/mod.rs",
            "src/out/x.rs",
            "src/dist/y.rs",
            "target/debug/junk",
            "build/junk",
            ".git/HEAD",
            "sub/Cargo.toml",
            "sub/target/junk",
            "sub/src/build/z.rs",
        ] {
            let p = src.join(f);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, "x").unwrap();
        }
        let dst = d.path().join("copy");
        copy_tree(&src, &dst).unwrap();
        for kept in [
            "src/build/mod.rs",
            "src/target/mod.rs",
            "src/out/x.rs",
            "src/dist/y.rs",
            "sub/src/build/z.rs",
        ] {
            assert!(dst.join(kept).is_file(), "{kept} must be copied");
        }
        for gone in ["target", "build", ".git", "sub/target"] {
            assert!(!dst.join(gone).exists(), "{gone} must be left out");
        }
    }

    #[test]
    fn class_keys_and_module_prefixes() {
        assert_eq!(class_key("a/target/classes/com/x/U.class"), "com/x/U.class");
        assert_eq!(
            class_key("a/build/classes/java/main/com/x/U.class"),
            "com/x/U.class"
        );
        assert_eq!(class_key("Loose.class"), "Loose.class");
        assert_eq!(module_prefix("a/b/target/classes/com"), "a/b");
        assert_eq!(module_prefix("target/classes"), "");
        let g = vec![
            vec!["x".to_owned(), "y".to_owned()],
            vec!["x".to_owned()],
            vec!["z".to_owned()],
        ];
        assert_eq!(conflict_free_batches(&g), vec![vec![0, 2], vec![1]]);
    }

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
