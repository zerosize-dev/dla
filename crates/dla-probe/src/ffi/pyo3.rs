//! PyO3 boundaries: a Rust `cdylib` with a `#[pymodule]` that Python source imports.
//!
//! The module name is the `#[pymodule]` function (or `#[pyo3(name = "x")]`), the name the
//! interpreter looks up as `PyInit_<name>`. A boundary needs both sides: the Rust module and a
//! Python file that imports it by that name (`import name`, `from name import f`,
//! `from pkg.name import f`). Each `#[pyfunction]` of the crate is an exported method.

use dla_core::{
    probe::{BoundaryMethod, FfiBoundary},
    runtime::Runtime,
};
use std::collections::BTreeSet;

use regex::Regex;

use super::{dir_of, ext_of, line_of};
use crate::scan::Tree;

struct Module {
    name: String,
    dir: String,
    cdylib: bool,
}

fn crate_dir_of(rel: &str, crates: &[(String, bool)]) -> (String, bool) {
    crates
        .iter()
        .filter(|(d, _)| d.is_empty() || rel.starts_with(&format!("{d}/")))
        .max_by_key(|(d, _)| d.len())
        .cloned()
        .unwrap_or_else(|| (dir_of(rel).to_owned(), false))
}

/// `(crate directory, is a cdylib)` of every `Cargo.toml` that names `pyo3`.
fn pyo3_crates(tree: &Tree) -> Vec<(String, bool)> {
    tree.files
        .iter()
        .filter(|f| f.rsplit('/').next() == Some("Cargo.toml"))
        .filter_map(|rel| {
            let text = tree.read(rel)?;
            let doc: toml::Value = toml::from_str(&text).ok()?;
            let has = |t: &str| doc.get(t).and_then(|d| d.get("pyo3")).is_some();
            if !(has("dependencies") || has("dev-dependencies")) {
                return None;
            }
            let cdylib = doc
                .get("lib")
                .and_then(|l| l.get("crate-type"))
                .and_then(|k| k.as_array())
                .is_some_and(|k| k.iter().any(|x| x.as_str() == Some("cdylib")));
            Some((dir_of(rel).to_owned(), cdylib))
        })
        .collect()
}

fn modules(tree: &Tree, crates: &[(String, bool)]) -> Vec<Module> {
    // `#[pymodule] fn name`, `#[pymodule] mod name`, with optional attributes between.
    let re = Regex::new(
        r"#\[pymodule(?:\([^)]*\))?\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:fn|mod)\s+(\w+)",
    )
    .expect("static");
    let rename = Regex::new(r#"#\[pyo3\(\s*name\s*=\s*"(\w+)""#).expect("static");
    let mut out: Vec<Module> = Vec::new();
    for rel in tree.files.iter().filter(|f| ext_of(f) == "rs") {
        let Some(text) = tree.read(rel) else { continue };
        if !text.contains("pymodule") {
            continue;
        }
        for c in re.captures_iter(&text) {
            let Some(m) = c.get(0) else { continue };
            // An explicit `#[pyo3(name = "x")]` on the item wins.
            let head = &text[m.start()..m.end()];
            let name = rename
                .captures(head)
                .map_or_else(|| c[1].to_owned(), |r| r[1].to_owned());
            let (dir, cdylib) = crate_dir_of(rel, crates);
            if !out.iter().any(|o| o.name == name && o.dir == dir) {
                out.push(Module { name, dir, cdylib });
            }
        }
    }
    out
}

/// Where Python code can import a project package from: the directory names that hold
/// Python files, the first segment of a maturin `module-name`, and the crate directories.
fn local_packages(tree: &Tree, mods: &[Module]) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for f in tree
        .files
        .iter()
        .filter(|f| matches!(ext_of(f), "py" | "pyi"))
    {
        let d = dir_of(f);
        if let Some(last) = d.rsplit('/').next().filter(|x| !x.is_empty()) {
            out.insert(last.to_owned());
        }
    }
    for rel in tree
        .files
        .iter()
        .filter(|f| f.rsplit('/').next() == Some("pyproject.toml"))
    {
        let Some(text) = tree.read(rel) else { continue };
        let Ok(doc) = toml::from_str::<toml::Value>(&text) else {
            continue;
        };
        if let Some(n) = doc
            .get("tool")
            .and_then(|t| t.get("maturin"))
            .and_then(|m| m.get("module-name"))
            .and_then(|n| n.as_str())
            && let Some((first, _)) = n.split_once('.')
        {
            out.insert(first.to_owned());
        }
    }
    for m in mods {
        if let Some(last) = m.dir.rsplit('/').next().filter(|x| !x.is_empty()) {
            out.insert(last.to_owned());
        }
    }
    out
}

/// True when `text` imports the Rust module `name`: bare (`import name`), relative
/// (`from . import name`) or through a package of this project (`from pkg import name`).
/// A qualifier from outside the project (`import numpy.core`, `from os import path`) names
/// a different module, and so does a bare import when a Python module of the same name
/// lives in the project.
fn imports(text: &str, name: &str, local: &BTreeSet<String>, shadowed: bool) -> bool {
    let n = regex::escape(name);
    let ok_qual = |dots: &str, qual: &str| {
        if !dots.is_empty() {
            return true;
        }
        let q = qual.trim_end_matches('.');
        if q.is_empty() {
            return !shadowed;
        }
        local.contains(q.split('.').next().unwrap_or(q))
    };
    let from_mod = Regex::new(&format!(
        r"(?m)^\s*from\s+(\.*)((?:[\w.]+\.)?){n}(?:\.\w+)*\s+import\b"
    ))
    .expect("escaped name");
    let from_pkg = Regex::new(&format!(
        r"(?m)^\s*from\s+(\.*)([\w.]*)\s+import\s+\(?\s*(?:\w+\s*,\s*)*{n}\b"
    ))
    .expect("escaped name");
    let plain =
        Regex::new(&format!(r"(?m)^\s*import\s+((?:[\w.]+\.)?){n}\b")).expect("escaped name");
    from_mod.captures_iter(text).any(|c| ok_qual(&c[1], &c[2]))
        || from_pkg.captures_iter(text).any(|c| {
            // `from . import name`: dots only; `from pkg import name`: a project package.
            if !c[1].is_empty() && c[2].is_empty() {
                return true;
            }
            ok_qual(&c[1], &format!("{}.", &c[2])) && !c[2].is_empty()
        })
        || plain.captures_iter(text).any(|c| ok_qual("", &c[1]))
}

/// True when a Python module or package called `name` exists outside the crate.
fn shadowed_by_python(tree: &Tree, name: &str, crate_dir: &str) -> bool {
    tree.files.iter().any(|f| {
        let outside = crate_dir.is_empty() || !f.starts_with(&format!("{crate_dir}/"));
        outside
            && (f.rsplit('/').next() == Some(&format!("{name}.py"))
                || f.ends_with(&format!("/{name}/__init__.py"))
                || f == &format!("{name}/__init__.py"))
    })
}

fn functions(tree: &Tree, dir: &str) -> Vec<BoundaryMethod> {
    let re = Regex::new(
        r"#\[pyfunction(?:\([^)]*\))?\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?fn\s+(\w+)",
    )
    .expect("static");
    let mut out = Vec::new();
    for rel in tree.files.iter().filter(|f| ext_of(f) == "rs") {
        let in_crate = dir.is_empty() || rel.starts_with(&format!("{dir}/"));
        if !in_crate {
            continue;
        }
        let Some(text) = tree.read(rel) else { continue };
        if !text.contains("pyfunction") {
            continue;
        }
        for c in re.captures_iter(&text) {
            let Some(g) = c.get(1) else { continue };
            out.push((
                g.as_str().to_owned(),
                format!("{rel}:{}", line_of(&text, g.start())),
            ));
        }
    }
    out.into_iter()
        .map(|(name, at)| BoundaryMethod {
            export: name.clone(),
            managed: name,
            declared_at: None,
            defined_at: Some(at),
        })
        .collect()
}

/// The PyO3 boundaries of the project, one per `#[pymodule]` that Python imports.
pub fn detect(tree: &Tree) -> Vec<FfiBoundary> {
    let crates = pyo3_crates(tree);
    let mods = modules(tree, &crates);
    if mods.is_empty() {
        return Vec::new();
    }
    let py: Vec<(&String, String)> = tree
        .files
        .iter()
        .filter(|f| ext_of(f) == "py" || ext_of(f) == "pyi")
        .filter_map(|f| Some((f, tree.read(f)?)))
        .collect();
    let local = local_packages(tree, &mods);
    let mut out = Vec::new();
    for m in mods {
        let shadowed = shadowed_by_python(tree, &m.name, &m.dir);
        let mut loaded_by: Vec<String> = py
            .iter()
            .filter(|(_, text)| imports(text, &m.name, &local, shadowed))
            .map(|(f, _)| (*f).clone())
            .collect();
        loaded_by.sort();
        loaded_by.dedup();
        if loaded_by.is_empty() {
            continue;
        }
        let methods = functions(tree, &m.dir)
            .into_iter()
            .map(|mut b| {
                b.managed = format!("{}.{}", m.name, b.export);
                b
            })
            .collect();
        out.push(FfiBoundary {
            kind: "pyo3".to_owned(),
            managed: Runtime::Python,
            native: Runtime::Rust,
            library: Some(m.name),
            loaded_by,
            native_dir: Some(if m.dir.is_empty() {
                ".".to_owned()
            } else {
                m.dir
            }),
            cdylib: m.cdylib,
            methods,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn tree(files: &[(&str, &str)]) -> (tempfile::TempDir, Tree) {
        let dir = tempfile::tempdir().unwrap();
        for (p, c) in files {
            let path = dir.path().join(p);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, c).unwrap();
        }
        let t = Tree::walk(dir.path());
        (dir, t)
    }

    const CARGO: &str = "[package]\nname = \"fastcodec\"\n[lib]\nname = \"fastcodec\"\ncrate-type = [\"cdylib\"]\n[dependencies]\npyo3 = \"0.22\"\n";
    const LIB: &str = "use pyo3::prelude::*;\n\n#[pyfunction]\nfn open(data: Vec<u8>) -> usize { data.len() }\n\n#[pyfunction]\npub fn close(h: usize) {}\n\n#[pymodule]\nfn fastcodec(m: &Bound<'_, PyModule>) -> PyResult<()> {\n    m.add_function(wrap_pyfunction!(open, m)?)?;\n    Ok(())\n}\n";

    #[test]
    fn a_pymodule_that_python_imports_is_a_boundary() {
        let (_d, t) = tree(&[
            ("native/Cargo.toml", CARGO),
            ("native/src/lib.rs", LIB),
            (
                "app/main.py",
                "import os\nimport fastcodec\n\nfastcodec.open(b'x')\n",
            ),
        ]);
        let b = detect(&t);
        assert_eq!(b.len(), 1);
        let b = &b[0];
        assert_eq!(b.kind, "pyo3");
        assert_eq!(b.managed, Runtime::Python);
        assert_eq!(b.native, Runtime::Rust);
        assert_eq!(b.library.as_deref(), Some("fastcodec"));
        assert_eq!(b.native_dir.as_deref(), Some("native"));
        assert!(b.cdylib);
        assert_eq!(b.loaded_by, ["app/main.py"]);
        assert_eq!(b.methods.len(), 2);
        assert_eq!(b.methods[0].managed, "fastcodec.open");
        assert_eq!(
            b.methods[0].defined_at.as_deref(),
            Some("native/src/lib.rs:4")
        );
    }

    #[test]
    fn a_from_import_and_a_package_path_count() {
        for src in [
            "from fastcodec import open\n",
            "from pkg.fastcodec import open\n",
            "import pkg.fastcodec as fc\n",
        ] {
            let (_d, t) = tree(&[
                ("native/Cargo.toml", CARGO),
                ("native/src/lib.rs", LIB),
                ("pkg/__init__.py", ""),
                ("app.py", src),
            ]);
            assert_eq!(detect(&t).len(), 1, "{src}");
        }
    }

    #[test]
    fn unrelated_imports_of_a_common_name_are_not_a_boundary() {
        let lib = |n: &str| format!("#[pymodule]\nfn {n}(m: &PyModule) {{}}\n");
        let cargo = "[package]\nname = \"x\"\n[lib]\ncrate-type = [\"cdylib\"]\n[dependencies]\npyo3 = \"0.22\"\n";
        for (n, src) in [
            ("core", "import numpy.core\n"),
            ("path", "from os import path\n"),
            ("utils", "from helpers import utils\n"),
        ] {
            let (_d, t) = tree(&[
                ("native/Cargo.toml", cargo),
                ("native/src/lib.rs", &lib(n)),
                ("app.py", src),
            ]);
            assert!(detect(&t).is_empty(), "{src}");
        }
        // A bare import that a Python module of the project answers is not the Rust module.
        let (_d, t) = tree(&[
            ("native/Cargo.toml", cargo),
            ("native/src/lib.rs", &lib("utils")),
            ("app.py", "import utils\n"),
            ("utils.py", "x = 1\n"),
        ]);
        assert!(detect(&t).is_empty());
        // The same imports through a project package still count.
        let (_d, t) = tree(&[
            ("native/Cargo.toml", cargo),
            ("native/src/lib.rs", &lib("core")),
            ("mypkg/__init__.py", "from . import core\n"),
            ("app.py", "from mypkg import core\n"),
        ]);
        assert_eq!(detect(&t).len(), 1);
    }

    #[test]
    fn a_pymodule_nobody_imports_is_not_a_boundary() {
        let (_d, t) = tree(&[
            ("native/Cargo.toml", CARGO),
            ("native/src/lib.rs", LIB),
            ("app.py", "import fastcodecs\nprint('fastcodec')\n"),
        ]);
        assert!(detect(&t).is_empty());
    }

    #[test]
    fn a_pymodule_attribute_rename_is_honoured() {
        let lib = "#[pymodule]\n#[pyo3(name = \"_fast\")]\nfn init(m: &PyModule) {}\n";
        let (_d, t) = tree(&[
            ("Cargo.toml", CARGO),
            ("src/lib.rs", lib),
            (
                "pkg/__init__.py",
                "from pkg import _fast\nfrom ._fast import open\n",
            ),
        ]);
        let b = detect(&t);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].library.as_deref(), Some("_fast"));
        assert_eq!(b[0].native_dir.as_deref(), Some("."));
    }

    #[test]
    fn plain_python_and_rust_without_pyo3_give_nothing() {
        let (_d, t) = tree(&[
            ("a.py", "import fastcodec\n"),
            ("src/lib.rs", "pub fn fastcodec() {}\n"),
        ]);
        assert!(detect(&t).is_empty());
    }
}
