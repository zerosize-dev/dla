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

fn imports(name: &str) -> Regex {
    let n = regex::escape(name);
    Regex::new(&format!(
        r"(?m)^\s*(?:from\s+\.*(?:[\w.]+\.)?{n}(?:\.\w+)*\s+import\b|from\s+[\w.]+\s+import\s+\(?\s*(?:\w+\s*,\s*)*{n}\b|import\s+(?:[\w.]+\.)?{n}\b)"
    ))
    .expect("escaped name")
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
    let mut out = Vec::new();
    for m in mods {
        let re = imports(&m.name);
        let mut loaded_by: Vec<String> = py
            .iter()
            .filter(|(_, text)| re.is_match(text))
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
                ("app.py", src),
            ]);
            assert_eq!(detect(&t).len(), 1, "{src}");
        }
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
