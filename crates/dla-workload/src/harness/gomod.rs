//! The Go modules of a tree: which `go.mod` holds a package, which modules of the repository
//! a module needs, and the `go.work` a work copy with several modules gets.
//!
//! A repository of nested modules (`core/go.mod`, `plugins/x/go.mod`) has no one module a
//! `go test` run could start in: the driver of a package is built and run from the module that
//! holds the package, and the `go.work` at the root of the copy makes every module of the
//! copy resolve from the copy, not from a released version.

use std::{
    fs,
    path::{Path, PathBuf},
};

/// Most directories looked at when a tree is searched for `go.mod` files.
const MAX_DIRS: usize = 50_000;

/// What the harness reads of a `go.mod`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GoMod {
    /// The module path.
    pub module: String,
    /// The `go` directive (`1.21`, `1.22.3`).
    pub go: Option<String>,
    /// The module paths of the `require` lines.
    pub requires: Vec<String>,
    /// The directories (as written, relative to the module) of the `replace` lines that
    /// point at a local path.
    pub local_replaces: Vec<String>,
}

/// The words of a `go.mod` line without its comment.
fn words(line: &str) -> Vec<&str> {
    line.split("//")
        .next()
        .unwrap_or("")
        .split_whitespace()
        .collect()
}

/// Reads the parts of a `go.mod` the harness needs.
pub fn parse(text: &str) -> GoMod {
    let mut out = GoMod::default();
    // The directive of the block being read (`require (`), if any.
    let mut block: Option<String> = None;
    for line in text.lines() {
        let w = words(line);
        if w.is_empty() {
            continue;
        }
        let (directive, args): (String, &[&str]) = match &block {
            Some(_) if w[0] == ")" => {
                block = None;
                continue;
            }
            Some(b) => (b.clone(), &w[..]),
            None if w.len() >= 2 && w[1] == "(" => {
                block = Some(w[0].to_owned());
                continue;
            }
            None => (w[0].to_owned(), &w[1..]),
        };
        match directive.as_str() {
            "module" => {
                if let Some(m) = args.first() {
                    out.module = m.trim_matches('"').to_owned();
                }
            }
            "go" => out.go = args.first().map(|v| (*v).to_owned()),
            "require" => {
                if let Some(m) = args.first() {
                    out.requires.push(m.trim_matches('"').to_owned());
                }
            }
            "replace" => {
                if let Some(at) = args.iter().position(|a| *a == "=>")
                    && let Some(to) = args.get(at + 1)
                {
                    let to = to.trim_matches('"');
                    if to.starts_with("./") || to.starts_with("../") || to == "." || to == ".." {
                        out.local_replaces.push(to.to_owned());
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Every module under `root`: its directory relative to `root` (`""` for the root itself,
/// `/` separators) and its `go.mod`, sorted by directory. The directories Go itself ignores
/// (`testdata`, names that start with `.` or `_`), `vendor`, `node_modules` and the harness's
/// own `.dla` are not entered.
pub fn modules_in(root: &Path) -> Vec<(String, GoMod)> {
    let mut found = Vec::new();
    let mut stack: Vec<(PathBuf, String)> = vec![(root.to_path_buf(), String::new())];
    let mut seen = 0usize;
    while let Some((dir, rel)) = stack.pop() {
        seen += 1;
        if seen > MAX_DIRS {
            break;
        }
        let Ok(rd) = fs::read_dir(&dir) else { continue };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                if name.starts_with(['.', '_'])
                    || matches!(name.as_str(), "testdata" | "vendor" | "node_modules")
                {
                    continue;
                }
                let child = if rel.is_empty() {
                    name
                } else {
                    format!("{rel}/{name}")
                };
                stack.push((e.path(), child));
            } else if name == "go.mod"
                && let Ok(text) = fs::read_to_string(e.path())
            {
                found.push((rel.clone(), parse(&text)));
            }
        }
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

/// The directory (relative to `root`, `""` for the root) of the nearest `go.mod` at or above
/// `pkg_dir` (relative to `root`), and not above `root`.
pub fn module_dir_of(root: &Path, pkg_dir: &str) -> Option<String> {
    let mut dir: Vec<&str> = pkg_dir
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    loop {
        let rel = dir.join("/");
        if root.join(&rel).join("go.mod").is_file() {
            return Some(rel);
        }
        dir.pop()?;
    }
}

/// `a/b/../c` as `a/c`; `None` when the path climbs above the root.
fn normalize(path: &str) -> Option<String> {
    let mut out: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                out.pop()?;
            }
            p => out.push(p),
        }
    }
    Some(out.join("/"))
}

/// The module directories the module at `module_dir` needs from the same tree: itself, the
/// modules of the tree it requires by module path, and the modules a local `replace` points
/// at, and what those need in turn. Sorted.
pub fn needed_modules(root: &Path, module_dir: &str) -> Vec<String> {
    let all = modules_in(root);
    let by_path = |path: &str| {
        all.iter()
            .find(|(_, m)| m.module == path)
            .map(|(d, _)| d.clone())
    };
    let mut out: Vec<String> = Vec::new();
    let mut todo = vec![module_dir.to_owned()];
    while let Some(dir) = todo.pop() {
        if out.contains(&dir) {
            continue;
        }
        let Some((_, m)) = all.iter().find(|(d, _)| *d == dir) else {
            continue;
        };
        out.push(dir.clone());
        todo.extend(m.requires.iter().filter_map(|r| by_path(r)));
        for local in &m.local_replaces {
            let joined = if dir.is_empty() {
                local.clone()
            } else {
                format!("{dir}/{local}")
            };
            if let Some(d) = normalize(&joined)
                && all.iter().any(|(x, _)| *x == d)
            {
                todo.push(d);
            }
        }
    }
    out.sort();
    out
}

/// A `go` directive as numbers, to compare versions (`1.21` < `1.21.3` < `1.22`); a suffix
/// such as `rc1` is ignored.
fn version_key(v: &str) -> Vec<u64> {
    let mut key: Vec<u64> = v
        .split('.')
        .map(|p| {
            p.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse()
                .unwrap_or(0)
        })
        .collect();
    while key.last() == Some(&0) {
        key.pop();
    }
    key
}

/// What [`write_go_work`] did.
#[derive(Debug, PartialEq, Eq)]
pub enum GoWork {
    /// One module or none: no workspace is needed.
    NotNeeded,
    /// The tree has its own `go.work`.
    Own,
    /// A `go.work` was written: the module directories it lists, and the `go` line.
    Written {
        /// Module directories (`""` is the root), sorted.
        dirs: Vec<String>,
        /// The highest `go` directive of the modules.
        go: Option<String>,
        /// The distinct `go` directives of the modules when they differ.
        versions: Vec<String>,
    },
}

/// Writes `go.work` at the root of `work` when it holds more than one module and has no
/// `go.work` of its own. The `go` line is the highest `go` directive of the modules: a
/// workspace that names a lower version than one of its modules is refused by the go
/// command.
pub fn write_go_work(work: &Path) -> std::io::Result<GoWork> {
    if work.join("go.work").is_file() {
        return Ok(GoWork::Own);
    }
    let mods = modules_in(work);
    if mods.len() < 2 {
        return Ok(GoWork::NotNeeded);
    }
    let mut versions: Vec<String> = mods.iter().filter_map(|(_, m)| m.go.clone()).collect();
    versions.sort_by_key(|v| version_key(v));
    // `1.21` and `1.21.0` name the same release.
    versions.dedup_by(|a, b| version_key(a) == version_key(b));
    let go = versions.last().cloned();
    let mut text = String::new();
    if let Some(go) = &go {
        text.push_str(&format!("go {go}\n\n"));
    }
    text.push_str("use (\n");
    for (dir, _) in &mods {
        if dir.is_empty() {
            text.push_str("\t.\n");
        } else {
            text.push_str(&format!("\t./{dir}\n"));
        }
    }
    text.push_str(")\n");
    fs::write(work.join("go.work"), text)?;
    Ok(GoWork::Written {
        dirs: mods.into_iter().map(|(d, _)| d).collect(),
        go,
        versions: if versions.len() > 1 {
            versions
        } else {
            Vec::new()
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_mod_directives_are_read_with_blocks_and_comments() {
        let m = parse(
            "module example.test/svc // the service\n\ngo 1.21\n\nrequire example.test/root v0.0.0\n\nrequire (\n\texample.test/a v1.0.0 // indirect\n\texample.test/b v1.2.0\n)\n\nreplace example.test/root => ../\n\nreplace (\n\texample.test/a => ./vendor-a\n\texample.test/b => example.org/b v1.3.0\n)\n",
        );
        assert_eq!(m.module, "example.test/svc");
        assert_eq!(m.go.as_deref(), Some("1.21"));
        assert_eq!(
            m.requires,
            ["example.test/root", "example.test/a", "example.test/b"]
        );
        assert_eq!(m.local_replaces, ["../", "./vendor-a"]);
    }

    fn tree(files: &[(&str, &str)]) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        for (p, t) in files {
            let f = d.path().join(p);
            fs::create_dir_all(f.parent().unwrap()).unwrap();
            fs::write(f, t).unwrap();
        }
        d
    }

    #[test]
    fn the_module_of_a_package_is_the_nearest_go_mod_at_or_above_it() {
        let d = tree(&[
            ("go.mod", "module example.test/root\n"),
            ("svc/go.mod", "module example.test/svc\n"),
            ("svc/store/s.go", "package store\n"),
            ("other/o.go", "package other\n"),
        ]);
        assert_eq!(module_dir_of(d.path(), "svc/store").as_deref(), Some("svc"));
        assert_eq!(module_dir_of(d.path(), "svc").as_deref(), Some("svc"));
        assert_eq!(module_dir_of(d.path(), "other").as_deref(), Some(""));
        assert_eq!(module_dir_of(d.path(), "").as_deref(), Some(""));
        let none = tree(&[("a/a.go", "package a\n")]);
        assert_eq!(module_dir_of(none.path(), "a"), None);
    }

    #[test]
    fn a_module_needs_the_modules_it_requires_and_replaces_locally() {
        let d = tree(&[
            ("go.mod", "module example.test/root\n\ngo 1.21\n"),
            (
                "svc/go.mod",
                "module example.test/svc\n\nrequire example.test/root v0.0.0\n\nrequire example.test/lib v0.0.0\n\nreplace example.test/lib => ../lib\n",
            ),
            ("lib/go.mod", "module example.test/lib\n"),
            ("unrelated/go.mod", "module example.test/unrelated\n"),
            ("testdata/go.mod", "module example.test/fixture\n"),
        ]);
        assert_eq!(needed_modules(d.path(), "svc"), ["", "lib", "svc"]);
        assert_eq!(needed_modules(d.path(), "lib"), ["lib"]);
        let all: Vec<String> = modules_in(d.path()).into_iter().map(|(d, _)| d).collect();
        assert_eq!(all, ["", "lib", "svc", "unrelated"]);
    }

    #[test]
    fn the_go_work_lists_every_module_and_names_the_highest_go_version() {
        let d = tree(&[
            ("go.mod", "module example.test/root\n\ngo 1.21\n"),
            ("svc/go.mod", "module example.test/svc\n\ngo 1.23.1\n"),
            ("lib/go.mod", "module example.test/lib\n\ngo 1.22\n"),
            ("lib2/go.mod", "module example.test/lib2\n\ngo 1.21.0\n"),
        ]);
        let got = write_go_work(d.path()).unwrap();
        assert_eq!(
            got,
            GoWork::Written {
                dirs: vec![String::new(), "lib".into(), "lib2".into(), "svc".into()],
                go: Some("1.23.1".into()),
                versions: vec!["1.21".into(), "1.22".into(), "1.23.1".into()],
            }
        );
        assert_eq!(
            fs::read_to_string(d.path().join("go.work")).unwrap(),
            "go 1.23.1\n\nuse (\n\t.\n\t./lib\n\t./lib2\n\t./svc\n)\n"
        );
        // A second call finds the file and leaves it.
        assert_eq!(write_go_work(d.path()).unwrap(), GoWork::Own);
        let one = tree(&[("go.mod", "module example.test/root\n")]);
        assert_eq!(write_go_work(one.path()).unwrap(), GoWork::NotNeeded);
        assert!(!one.path().join("go.work").exists());
    }
}
