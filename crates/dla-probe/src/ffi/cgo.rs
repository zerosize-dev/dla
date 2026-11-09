//! cgo boundaries: Go packages that `import "C"` and call C functions the project defines.
//!
//! A package with `import "C"` is a boundary. Its methods are the C functions that the Go files
//! call as `C.name(...)` and that the project defines, in the cgo preamble (the comment above
//! the import) or in a `.c`, `.cc`, `.cpp`, `.h` file of the same directory. A function the
//! project does not define (`C.puts`, `C.free`) and a conversion (`C.int(x)`) is not a method.
//! The managed side of a method is the Go function that makes the call.

use std::collections::BTreeMap;

use dla_core::{
    probe::{BoundaryMethod, FfiBoundary},
    runtime::Runtime,
};
use regex::Regex;

use super::{dir_of, ext_of, line_of};
use crate::scan::Tree;

/// cgo's built-in helpers and C type names that look like calls.
const NOT_FUNCTIONS: &[&str] = &[
    "CString",
    "GoString",
    "GoStringN",
    "GoBytes",
    "CBytes",
    "malloc",
    "calloc",
    "realloc",
    "free",
    "int",
    "uint",
    "long",
    "ulong",
    "short",
    "ushort",
    "char",
    "schar",
    "uchar",
    "float",
    "double",
    "longlong",
    "ulonglong",
    "size_t",
    "ssize_t",
    "uintptr_t",
    "intptr_t",
    "int8_t",
    "int16_t",
    "int32_t",
    "int64_t",
    "uint8_t",
    "uint16_t",
    "uint32_t",
    "uint64_t",
    "bool",
    "void",
    "sizeof",
    "struct",
    "enum",
    "union",
];

fn imports_c(text: &str) -> bool {
    Regex::new(r#"(?m)^\s*import\s+"C"|^\s*"C"\s*$"#)
        .expect("static")
        .is_match(text)
}

/// The Go function (name) around byte `at`, from the nearest `func` before it.
fn enclosing_func(text: &str, at: usize, funcs: &Regex) -> Option<(String, usize)> {
    funcs
        .captures_iter(&text[..at.min(text.len())])
        .last()
        .and_then(|c| Some((c.get(1)?.as_str().to_owned(), c.get(0)?.start())))
}

/// Where the C function `name` is defined: the file and line.
fn definition(tree: &Tree, dir: &str, go_files: &[(String, String)], name: &str) -> Option<String> {
    let def = Regex::new(&format!(
        r"(?m)^[ \t]*(?:static\s+|inline\s+|extern\s+)*[A-Za-z_][\w \t\*]*?[ \t\*]{}\s*\([^;{{]*\)\s*\{{",
        regex::escape(name)
    ))
    .expect("escaped name");
    // The preamble of a Go file, then C and header files of the directory.
    for (rel, text) in go_files {
        if let Some(m) = def.find(text) {
            return Some(format!("{rel}:{}", line_of(text, m.start())));
        }
    }
    for rel in tree.files.iter().filter(|f| {
        matches!(ext_of(f), "c" | "cc" | "cpp" | "cxx" | "h" | "hpp") && dir_of(f) == dir
    }) {
        let text = tree.read(rel)?;
        if let Some(m) = def.find(&text) {
            return Some(format!("{rel}:{}", line_of(&text, m.start())));
        }
    }
    None
}

/// The cgo boundaries of the project, one per Go package directory with `import "C"`.
pub fn detect(tree: &Tree) -> Vec<FfiBoundary> {
    let pkg = Regex::new(r"(?m)^\s*package\s+(\w+)").expect("static");
    let funcs = Regex::new(r"(?m)^func\s+(?:\([^)]*\)\s*)?(\w+)").expect("static");
    let call = Regex::new(r"\bC\.(\w+)\s*\(").expect("static");
    let mut by_dir: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for rel in tree.files.iter().filter(|f| ext_of(f) == "go") {
        let Some(text) = tree.read(rel) else { continue };
        if text.contains("\"C\"") && imports_c(&text) {
            by_dir
                .entry(dir_of(rel).to_owned())
                .or_default()
                .push((rel.clone(), text));
        }
    }
    let mut out = Vec::new();
    for (dir, files) in by_dir {
        let package = files
            .iter()
            .find_map(|(_, t)| pkg.captures(t).map(|c| c[1].to_owned()))
            .unwrap_or_else(|| "main".to_owned());
        let mut methods: Vec<BoundaryMethod> = Vec::new();
        for (rel, text) in &files {
            for c in call.captures_iter(text) {
                let (Some(g), Some(whole)) = (c.get(1), c.get(0)) else {
                    continue;
                };
                let name = g.as_str();
                if NOT_FUNCTIONS.contains(&name) {
                    continue;
                }
                let Some(defined_at) = definition(tree, &dir, &files, name) else {
                    continue;
                };
                let (caller, declared_at) = match enclosing_func(text, whole.start(), &funcs) {
                    Some((f, start)) => (f, Some(format!("{rel}:{}", line_of(text, start)))),
                    None => (String::from("init"), None),
                };
                let managed = format!("{package}.{caller}");
                if methods
                    .iter()
                    .any(|m| m.export == name && m.managed == managed)
                {
                    continue;
                }
                methods.push(BoundaryMethod {
                    export: name.to_owned(),
                    managed,
                    declared_at,
                    defined_at: Some(defined_at),
                });
            }
        }
        out.push(FfiBoundary {
            kind: "cgo".to_owned(),
            managed: Runtime::Go,
            native: Runtime::Native,
            library: None,
            loaded_by: files.iter().map(|(f, _)| f.clone()).collect(),
            native_dir: Some(if dir.is_empty() { ".".to_owned() } else { dir }),
            cdylib: false,
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

    const PREAMBLE: &str = "package glyph\n\n/*\n#include <stdlib.h>\nstatic int cache_put(int k) {\n    return k;\n}\n*/\nimport \"C\"\n\nfunc Put(k int) int {\n\tp := C.CString(\"x\")\n\tdefer C.free(unsafe.Pointer(p))\n\treturn int(C.cache_put(C.int(k)))\n}\n";

    #[test]
    fn a_preamble_function_called_from_go_is_a_method() {
        let (_d, t) = tree(&[("glyph/glyph.go", PREAMBLE)]);
        let b = detect(&t);
        assert_eq!(b.len(), 1);
        let b = &b[0];
        assert_eq!(b.kind, "cgo");
        assert_eq!(b.managed, Runtime::Go);
        assert_eq!(b.native, Runtime::Native);
        assert_eq!(b.native_dir.as_deref(), Some("glyph"));
        assert_eq!(b.loaded_by, ["glyph/glyph.go"]);
        assert_eq!(b.methods.len(), 1, "{:?}", b.methods);
        let m = &b.methods[0];
        assert_eq!(m.export, "cache_put");
        assert_eq!(m.managed, "glyph.Put");
        assert_eq!(m.declared_at.as_deref(), Some("glyph/glyph.go:11"));
        assert_eq!(m.defined_at.as_deref(), Some("glyph/glyph.go:5"));
    }

    #[test]
    fn a_function_defined_in_a_c_file_of_the_package_is_a_method() {
        let (_d, t) = tree(&[
            (
                "pool/pool.go",
                "package pool\n\n// #include \"pool.h\"\nimport \"C\"\n\nfunc (p *Pool) Grow(n int) {\n\tC.pool_grow(C.size_t(n))\n}\n",
            ),
            ("pool/pool.h", "void pool_grow(size_t n);\n"),
            (
                "pool/pool.c",
                "#include \"pool.h\"\nvoid pool_grow(size_t n) {\n  (void)n;\n}\n",
            ),
        ]);
        let b = detect(&t);
        assert_eq!(b.len(), 1);
        let m = &b[0].methods[0];
        assert_eq!(m.export, "pool_grow");
        assert_eq!(m.managed, "pool.Grow");
        assert_eq!(m.defined_at.as_deref(), Some("pool/pool.c:2"));
    }

    #[test]
    fn a_package_with_import_c_but_no_project_function_is_still_a_boundary() {
        let (_d, t) = tree(&[(
            "m/m.go",
            "package m\n\n// #include <stdio.h>\nimport \"C\"\n\nfunc Say() {\n\tC.puts(C.CString(\"x\"))\n}\n",
        )]);
        let b = detect(&t);
        assert_eq!(b.len(), 1);
        assert!(b[0].methods.is_empty());
    }

    #[test]
    fn go_without_import_c_gives_nothing() {
        let (_d, t) = tree(&[
            (
                "a.go",
                "package a\n\n// C is just a comment about \"C\" here\nfunc F() {}\n",
            ),
            ("b.c", "int f() { return 0; }\n"),
        ]);
        assert!(detect(&t).is_empty());
    }

    #[test]
    fn grouped_imports_with_c_count() {
        let (_d, t) = tree(&[(
            "g.go",
            "package g\n\nimport (\n\t\"fmt\"\n\t\"C\"\n)\n\nfunc F() { fmt.Println() }\n",
        )]);
        assert_eq!(detect(&t).len(), 1);
    }
}
