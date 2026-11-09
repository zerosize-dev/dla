//! Foreign-function boundaries between runtimes (JNI in M5 cut 1).
//!
//! A JNI boundary is a JVM source that calls `System.loadLibrary` together with native code
//! that exports `Java_` functions: a Rust crate with `crate-type = ["cdylib"]` or a C/C++
//! file with `JNIEXPORT`. The harness uses the boundary to preload the allocator shim into
//! the JVM, and the exported names to map native stacks to the managed method.

use std::collections::BTreeSet;

use dla_core::{
    probe::{BoundaryMethod, FfiBoundary},
    runtime::Runtime,
};
use regex::Regex;

use crate::scan::Tree;

/// What one JVM source says about native code.
#[derive(Debug, Default)]
struct JvmSide {
    /// `(file, library)` of every `System.loadLibrary("x")`.
    loads: Vec<(String, String)>,
    /// `(fully qualified method, file:line)` of every `native` or `external` method.
    declared: Vec<(String, String)>,
}

/// One export found in native sources.
#[derive(Debug, Clone)]
struct Export {
    name: String,
    at: String,
    dir: String,
    rust: bool,
}

fn ext_of(rel: &str) -> &str {
    rel.rsplit_once('.').map_or("", |(_, e)| e)
}

fn dir_of(rel: &str) -> &str {
    rel.rfind('/').map_or("", |i| &rel[..i])
}

fn line_of(text: &str, byte: usize) -> usize {
    text[..byte.min(text.len())].matches('\n').count() + 1
}

/// Decodes a JNI-mangled name (`shop_codec_Native_open`, with `_1` for `_`) into a dotted
/// path. Returns `None` for a name that has no class and method part.
pub fn unmangle(export: &str) -> Option<String> {
    let rest = export.strip_prefix("Java_")?;
    // An overloaded function carries `__<signature>` after the method.
    let rest = rest.split_once("__").map_or(rest, |(a, _)| a);
    let mut out = String::new();
    let b: Vec<char> = rest.chars().collect();
    let mut i = 0;
    while i < b.len() {
        if b[i] == '_' {
            match b.get(i + 1) {
                Some('1') => {
                    out.push('_');
                    i += 2;
                }
                Some('2') => {
                    out.push(';');
                    i += 2;
                }
                Some('3') => {
                    out.push('[');
                    i += 2;
                }
                Some('0') if i + 5 < b.len() + 1 && b.len() >= i + 6 => {
                    let hex: String = b[i + 2..i + 6].iter().collect();
                    match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                        Some(c) => out.push(c),
                        None => out.push('?'),
                    }
                    i += 6;
                }
                _ => {
                    out.push('.');
                    i += 1;
                }
            }
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    let ok = out.contains('.') && !out.ends_with('.') && !out.starts_with('.');
    ok.then_some(out)
}

fn jvm_side(tree: &Tree) -> JvmSide {
    let load = Regex::new(r#"System\s*\.\s*loadLibrary\s*\(\s*"([^"]+)"\s*\)"#).expect("static");
    let package = Regex::new(r"(?m)^\s*package\s+([\w.]+)").expect("static");
    let class =
        Regex::new(r"(?m)^\s*(?:[\w@()]+\s+)*(?:class|object|interface)\s+(\w+)").expect("static");
    let java_native = Regex::new(r"(?m)\bnative\b[^;{=()]*?\b(\w+)\s*\(").expect("static");
    let kt_external = Regex::new(r"(?m)\bexternal\s+fun\s+(?:<[^>]*>\s*)?(?:[\w.]+\.)?(\w+)\s*\(")
        .expect("static");
    let mut side = JvmSide::default();
    for rel in tree
        .files
        .iter()
        .filter(|f| matches!(ext_of(f), "java" | "kt"))
    {
        let Some(text) = tree.read(rel) else { continue };
        if !text.contains("loadLibrary") && !text.contains("native ") && !text.contains("external ")
        {
            continue;
        }
        for c in load.captures_iter(&text) {
            side.loads.push((rel.clone(), c[1].to_owned()));
        }
        let pkg = package.captures(&text).map(|c| c[1].to_owned());
        let classes: Vec<(usize, String)> = class
            .captures_iter(&text)
            .filter_map(|c| c.get(1).map(|m| (m.start(), m.as_str().to_owned())))
            .collect();
        let re = if ext_of(rel) == "kt" {
            &kt_external
        } else {
            &java_native
        };
        for c in re.captures_iter(&text) {
            let Some(m) = c.get(1) else { continue };
            // Skip text inside a line comment.
            let line_start = text[..m.start()].rfind('\n').map_or(0, |i| i + 1);
            if text[line_start..m.start()].trim_start().starts_with("//") {
                continue;
            }
            let Some((_, cls)) = classes.iter().rev().find(|(at, _)| *at < m.start()) else {
                continue;
            };
            let fq = match &pkg {
                Some(p) => format!("{p}.{cls}.{}", m.as_str()),
                None => format!("{cls}.{}", m.as_str()),
            };
            side.declared
                .push((fq, format!("{rel}:{}", line_of(&text, m.start()))));
        }
    }
    side
}

/// Crate directories whose `Cargo.toml` declares a cdylib.
fn cdylib_dirs(tree: &Tree) -> Vec<String> {
    tree.files
        .iter()
        .filter(|f| f.rsplit('/').next() == Some("Cargo.toml"))
        .filter_map(|rel| {
            let text = tree.read(rel)?;
            let doc: toml::Value = toml::from_str(&text).ok()?;
            let kinds = doc.get("lib")?.get("crate-type")?.as_array()?;
            kinds
                .iter()
                .any(|k| k.as_str() == Some("cdylib"))
                .then(|| dir_of(rel).to_owned())
        })
        .collect()
}

fn exports(tree: &Tree, cdylibs: &[String]) -> Vec<Export> {
    let rust = Regex::new(r"\bfn\s+(Java_\w+)\s*\(").expect("static");
    let c = Regex::new(r"\b(Java_\w+)\s*\(").expect("static");
    let mut out = Vec::new();
    for rel in &tree.files {
        let ext = ext_of(rel);
        let is_rust = ext == "rs";
        if !(is_rust || matches!(ext, "c" | "cc" | "cpp" | "cxx")) {
            continue;
        }
        let Some(text) = tree.read(rel) else { continue };
        if !text.contains("Java_") {
            continue;
        }
        let dir = if is_rust {
            // The cdylib crate this file belongs to, else its own directory.
            cdylibs
                .iter()
                .filter(|d| d.is_empty() || rel.starts_with(&format!("{d}/")))
                .max_by_key(|d| d.len())
                .cloned()
                .unwrap_or_else(|| dir_of(rel).to_owned())
        } else {
            dir_of(rel).to_owned()
        };
        let re = if is_rust { &rust } else { &c };
        let mut seen = BTreeSet::new();
        for m in re.captures_iter(&text) {
            let Some(g) = m.get(1) else { continue };
            if !seen.insert(g.as_str().to_owned()) {
                continue;
            }
            // A C call site is not a definition: require JNIEXPORT before it on the line
            // above or the same line.
            if !is_rust {
                let from = text[..g.start()].len().saturating_sub(160);
                let from = (from..=g.start())
                    .find(|i| text.is_char_boundary(*i))
                    .unwrap_or(0);
                if !text[from..g.start()].contains("JNIEXPORT") {
                    continue;
                }
            }
            out.push(Export {
                name: g.as_str().to_owned(),
                at: format!("{rel}:{}", line_of(&text, g.start())),
                dir: dir.clone(),
                rust: is_rust,
            });
        }
    }
    out
}

/// Finds the JNI boundaries of the project: JVM sources that load a library plus native
/// code that exports `Java_` functions. One boundary per native directory.
pub fn detect(tree: &Tree) -> Vec<FfiBoundary> {
    let jvm = jvm_side(tree);
    if jvm.loads.is_empty() {
        return Vec::new();
    }
    let cdylibs = cdylib_dirs(tree);
    let all = exports(tree, &cdylibs);
    let mut dirs: Vec<(String, bool)> = Vec::new();
    for e in &all {
        if !dirs.iter().any(|(d, _)| *d == e.dir) {
            dirs.push((e.dir.clone(), e.rust));
        }
    }
    // A cdylib with no `Java_` export is still the native side of `System.loadLibrary`.
    if dirs.is_empty() {
        dirs.extend(cdylibs.iter().map(|d| (d.clone(), true)));
    }
    let mut libs: Vec<String> = jvm.loads.iter().map(|(_, l)| l.clone()).collect();
    libs.dedup();
    let mut loaded_by: Vec<String> = jvm.loads.iter().map(|(f, _)| f.clone()).collect();
    loaded_by.sort();
    loaded_by.dedup();
    dirs.into_iter()
        .map(|(dir, rust)| {
            let methods: Vec<BoundaryMethod> = all
                .iter()
                .filter(|e| e.dir == dir)
                .filter_map(|e| {
                    let managed = unmangle(&e.name)?;
                    let declared_at = jvm
                        .declared
                        .iter()
                        .find(|(fq, _)| *fq == managed)
                        .map(|(_, at)| at.clone());
                    Some(BoundaryMethod {
                        export: e.name.clone(),
                        managed,
                        declared_at,
                        defined_at: Some(e.at.clone()),
                    })
                })
                .collect();
            FfiBoundary {
                kind: "jni".to_owned(),
                managed: Runtime::Jvm,
                native: if rust { Runtime::Rust } else { Runtime::Native },
                library: libs.first().cloned(),
                loaded_by: loaded_by.clone(),
                native_dir: Some(if dir.is_empty() { ".".to_owned() } else { dir }),
                cdylib: rust && !cdylibs.is_empty(),
                methods,
            }
        })
        .collect()
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

    #[test]
    fn jni_names_unmangle() {
        assert_eq!(
            unmangle("Java_shop_codec_Native_open").as_deref(),
            Some("shop.codec.Native.open")
        );
        assert_eq!(
            unmangle("Java_a_B_snake_1case").as_deref(),
            Some("a.B.snake_case")
        );
        assert_eq!(
            unmangle("Java_a_B_f__I").as_deref(),
            Some("a.B.f"),
            "overload signature is dropped"
        );
        assert_eq!(unmangle("Java_x"), None);
        assert_eq!(unmangle("not_java"), None);
    }

    #[test]
    fn rust_cdylib_and_kotlin_external_form_a_boundary() {
        let (_d, t) = tree(&[
            (
                "kotlin/shop/Native.kt",
                "package shop.codec\n\nobject Native {\n    init { System.loadLibrary(\"codec\") }\n    @JvmStatic external fun open(data: ByteArray): Long\n    @JvmStatic external fun close(handle: Long)\n}\n",
            ),
            (
                "native/Cargo.toml",
                "[package]\nname = \"codec\"\n[lib]\ncrate-type = [\"cdylib\"]\n",
            ),
            (
                "native/src/lib.rs",
                "#[no_mangle]\npub unsafe extern \"system\" fn Java_shop_codec_Native_open(env: *mut u8) -> i64 { 0 }\n#[no_mangle]\npub unsafe extern \"system\" fn Java_shop_codec_Native_close(env: *mut u8) {}\n",
            ),
        ]);
        let b = detect(&t);
        assert_eq!(b.len(), 1);
        let b = &b[0];
        assert_eq!(b.kind, "jni");
        assert_eq!(b.library.as_deref(), Some("codec"));
        assert_eq!(b.native_dir.as_deref(), Some("native"));
        assert!(b.cdylib);
        assert_eq!(b.native, Runtime::Rust);
        assert_eq!(b.methods.len(), 2);
        let open = b
            .methods
            .iter()
            .find(|m| m.export.ends_with("open"))
            .unwrap();
        assert_eq!(open.managed, "shop.codec.Native.open");
        assert_eq!(open.declared_at.as_deref(), Some("kotlin/shop/Native.kt:5"));
        assert_eq!(open.defined_at.as_deref(), Some("native/src/lib.rs:2"));
    }

    #[test]
    fn c_jniexport_and_java_native_form_a_boundary() {
        let (_d, t) = tree(&[
            (
                "src/Lib.java",
                "package app;\npublic class Lib {\n  static { System.loadLibrary(\"app\"); }\n  public static native int add(int a, int b);\n}\n",
            ),
            (
                "jni/app.c",
                "#include <jni.h>\nJNIEXPORT jint JNICALL Java_app_Lib_add(JNIEnv *e, jclass c, jint a, jint b) { return a + b; }\n",
            ),
        ]);
        let b = detect(&t);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].native, Runtime::Native);
        assert!(!b[0].cdylib);
        assert_eq!(b[0].native_dir.as_deref(), Some("jni"));
        assert_eq!(b[0].methods[0].managed, "app.Lib.add");
        assert_eq!(
            b[0].methods[0].declared_at.as_deref(),
            Some("src/Lib.java:4")
        );
    }

    #[test]
    fn no_boundary_without_load_library_or_without_native_code() {
        let (_d, t) = tree(&[
            ("src/A.java", "class A { static native int f(); }\n"),
            (
                "n/lib.rs",
                "pub extern \"C\" fn Java_a_A_f() -> i32 { 0 }\n",
            ),
        ]);
        assert!(detect(&t).is_empty(), "no loadLibrary");
        let (_d, t) = tree(&[(
            "src/A.java",
            "class A { static { System.loadLibrary(\"x\"); } }\n",
        )]);
        assert!(detect(&t).is_empty(), "no native code at all");
    }

    #[test]
    fn a_malformed_toml_and_binary_junk_do_not_panic() {
        let (_d, t) = tree(&[
            ("Cargo.toml", "[lib\ncrate-type = "),
            ("a.java", "System.loadLibrary(\"x\""),
            ("b.rs", "fn Java_("),
        ]);
        assert!(detect(&t).is_empty());
    }
}
