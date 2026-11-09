//! N-API boundaries: a native Node addon that JavaScript loads.
//!
//! The native side is any of: a `binding.gyp` (node-gyp, usually with `node-addon-api`), a Rust
//! crate that depends on `napi` (napi-rs, a cdylib), or a prebuilt `.node` file in the tree. A
//! boundary needs the JavaScript side too: a source file that loads the addon by a `.node`
//! path, by `bindings('name')`, by `node-gyp-build`, or by requiring the package name. The
//! exported methods are the `#[napi]` functions (named in camelCase, as napi-rs does) or the
//! names C++ code sets on `exports`.

use dla_core::{
    probe::{BoundaryMethod, FfiBoundary},
    runtime::Runtime,
};
use regex::Regex;

use super::{dir_of, ext_of, line_of};
use crate::scan::Tree;

struct Addon {
    name: String,
    dir: String,
    rust: bool,
    cdylib: bool,
}

fn camel(snake: &str) -> String {
    let mut out = String::new();
    let mut up = false;
    for c in snake.chars() {
        if c == '_' {
            up = !out.is_empty();
        } else if up {
            out.extend(c.to_uppercase());
            up = false;
        } else {
            out.push(c);
        }
    }
    out
}

fn napi_crates(tree: &Tree, out: &mut Vec<Addon>) {
    for rel in tree
        .files
        .iter()
        .filter(|f| f.rsplit('/').next() == Some("Cargo.toml"))
    {
        let Some(text) = tree.read(rel) else { continue };
        let Ok(doc) = toml::from_str::<toml::Value>(&text) else {
            continue;
        };
        let has = |k: &str| doc.get(k).and_then(|d| d.get("napi")).is_some();
        if !(has("dependencies") || has("build-dependencies")) {
            continue;
        }
        let kinds = doc
            .get("lib")
            .and_then(|l| l.get("crate-type"))
            .and_then(|k| k.as_array());
        let cdylib = kinds.is_some_and(|k| k.iter().any(|x| x.as_str() == Some("cdylib")));
        let name = doc
            .get("lib")
            .and_then(|l| l.get("name"))
            .and_then(toml::Value::as_str)
            .or_else(|| {
                doc.get("package")
                    .and_then(|p| p.get("name"))
                    .and_then(toml::Value::as_str)
            });
        if let Some(name) = name {
            out.push(Addon {
                name: name.to_owned(),
                dir: dir_of(rel).to_owned(),
                rust: true,
                cdylib,
            });
        }
    }
}

fn gyp_targets(tree: &Tree, out: &mut Vec<Addon>) {
    let target = Regex::new(r#""target_name"\s*:\s*"([\w.-]+)""#).expect("static");
    for rel in tree
        .files
        .iter()
        .filter(|f| f.rsplit('/').next() == Some("binding.gyp"))
    {
        let Some(text) = tree.read(rel) else { continue };
        for c in target.captures_iter(&text) {
            out.push(Addon {
                name: c[1].to_owned(),
                dir: dir_of(rel).to_owned(),
                rust: false,
                cdylib: false,
            });
        }
    }
}

fn prebuilt(tree: &Tree, out: &mut Vec<Addon>) {
    for rel in tree.files.iter().filter(|f| ext_of(f) == "node") {
        let file = rel.rsplit('/').next().unwrap_or(rel);
        let stem = file.split('.').next().unwrap_or(file);
        if stem.is_empty() || out.iter().any(|a| a.name == stem) {
            continue;
        }
        out.push(Addon {
            name: stem.to_owned(),
            dir: dir_of(rel).to_owned(),
            rust: false,
            cdylib: false,
        });
    }
}

fn loader(name: &str) -> Regex {
    let n = regex::escape(name);
    Regex::new(&format!(
        r#"(?:['"][^'"\n]*{n}[^'"\n]*\.node['"]|bindings?\s*\(\s*['"]{n}(?:\.node)?['"]|['"]bindings['"]\s*\)\s*\(\s*['"]{n}['"]|require\s*\(\s*['"]{n}['"]\s*\)|from\s+['"]{n}['"])"#
    ))
    .expect("escaped name")
}

fn methods(tree: &Tree, a: &Addon) -> Vec<BoundaryMethod> {
    let in_dir = |rel: &str| a.dir.is_empty() || rel.starts_with(&format!("{}/", a.dir));
    let mut out: Vec<BoundaryMethod> = Vec::new();
    if a.rust {
        let re = Regex::new(
            r"#\[napi(?:\(([^)]*)\))?\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)",
        )
        .expect("static");
        let js_name = Regex::new(r#"js_name\s*=\s*"(\w+)""#).expect("static");
        for rel in tree.files.iter().filter(|f| ext_of(f) == "rs" && in_dir(f)) {
            let Some(text) = tree.read(rel) else { continue };
            if !text.contains("napi") {
                continue;
            }
            for c in re.captures_iter(&text) {
                let Some(g) = c.get(2) else { continue };
                let js = c
                    .get(1)
                    .and_then(|attr| js_name.captures(attr.as_str()))
                    .map_or_else(|| camel(g.as_str()), |j| j[1].to_owned());
                out.push(BoundaryMethod {
                    export: g.as_str().to_owned(),
                    managed: format!("{}.{js}", a.name),
                    declared_at: None,
                    defined_at: Some(format!("{rel}:{}", line_of(&text, g.start()))),
                });
            }
        }
        return out;
    }
    let set = Regex::new(
        r#"(?:exports\s*\.\s*Set\s*\(\s*(?:Napi::String::New\s*\(\s*env\s*,\s*)?"(\w+)"|exports\s*\[\s*"(\w+)"\s*\]\s*=|Napi::Function::New\s*\([^,]+,\s*\w+\s*,\s*"(\w+)"|napi_create_function\s*\(\s*\w+\s*,\s*"(\w+)")"#,
    )
    .expect("static");
    for rel in tree
        .files
        .iter()
        .filter(|f| matches!(ext_of(f), "cc" | "cpp" | "cxx" | "c") && in_dir(f))
    {
        let Some(text) = tree.read(rel) else { continue };
        for c in set.captures_iter(&text) {
            let Some(g) = (1..=4).find_map(|i| c.get(i)) else {
                continue;
            };
            if out.iter().any(|m| m.export == g.as_str()) {
                continue;
            }
            out.push(BoundaryMethod {
                export: g.as_str().to_owned(),
                managed: format!("{}.{}", a.name, g.as_str()),
                declared_at: None,
                defined_at: Some(format!("{rel}:{}", line_of(&text, g.start()))),
            });
        }
    }
    out
}

/// The N-API boundaries of the project: addons that JavaScript loads.
pub fn detect(tree: &Tree) -> Vec<FfiBoundary> {
    let mut addons: Vec<Addon> = Vec::new();
    napi_crates(tree, &mut addons);
    gyp_targets(tree, &mut addons);
    prebuilt(tree, &mut addons);
    if addons.is_empty() {
        return Vec::new();
    }
    let js: Vec<(&String, String)> = tree
        .files
        .iter()
        .filter(|f| matches!(ext_of(f), "js" | "cjs" | "mjs" | "ts" | "tsx" | "jsx"))
        .filter_map(|f| Some((f, tree.read(f)?)))
        .collect();
    let mut out = Vec::new();
    for a in addons {
        let re = loader(&a.name);
        let mut loaded_by: Vec<String> = js
            .iter()
            .filter(|(_, text)| re.is_match(text))
            .map(|(f, _)| (*f).clone())
            .collect();
        loaded_by.sort();
        loaded_by.dedup();
        if loaded_by.is_empty() {
            continue;
        }
        let methods = methods(tree, &a);
        out.push(FfiBoundary {
            kind: "napi".to_owned(),
            managed: Runtime::Node,
            native: if a.rust {
                Runtime::Rust
            } else {
                Runtime::Native
            },
            library: Some(a.name),
            loaded_by,
            native_dir: Some(if a.dir.is_empty() {
                ".".to_owned()
            } else {
                a.dir
            }),
            cdylib: a.cdylib,
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

    #[test]
    fn camel_case_follows_napi_rs() {
        assert_eq!(camel("open_frame"), "openFrame");
        assert_eq!(camel("open"), "open");
        assert_eq!(camel("_x_y"), "xY");
    }

    #[test]
    fn a_node_gyp_addon_loaded_with_bindings_is_a_boundary() {
        let (_d, t) = tree(&[
            (
                "binding.gyp",
                "{ \"targets\": [ { \"target_name\": \"codec\", \"sources\": [ \"src/codec.cc\" ], \"dependencies\": [\"<!(node -p \\\"require('node-addon-api').gyp\\\")\"] } ] }\n",
            ),
            (
                "src/codec.cc",
                "#include <napi.h>\nNapi::Object Init(Napi::Env env, Napi::Object exports) {\n  exports.Set(Napi::String::New(env, \"open\"), Napi::Function::New(env, Open));\n  exports.Set(\"close\", Napi::Function::New(env, Close));\n  return exports;\n}\nNODE_API_MODULE(codec, Init)\n",
            ),
            (
                "lib/index.js",
                "const codec = require('bindings')('codec');\nmodule.exports = codec;\n",
            ),
        ]);
        let b = detect(&t);
        assert_eq!(b.len(), 1);
        let b = &b[0];
        assert_eq!(b.kind, "napi");
        assert_eq!(b.managed, Runtime::Node);
        assert_eq!(b.native, Runtime::Native);
        assert_eq!(b.library.as_deref(), Some("codec"));
        assert_eq!(b.native_dir.as_deref(), Some("."));
        assert_eq!(b.loaded_by, ["lib/index.js"]);
        let names: Vec<&str> = b.methods.iter().map(|m| m.managed.as_str()).collect();
        assert_eq!(names, ["codec.open", "codec.close"]);
        assert_eq!(b.methods[0].defined_at.as_deref(), Some("src/codec.cc:3"));
    }

    #[test]
    fn a_napi_rs_crate_with_a_node_loader_is_a_boundary() {
        let (_d, t) = tree(&[
            (
                "addon/Cargo.toml",
                "[package]\nname = \"fastcodec\"\n[lib]\ncrate-type = [\"cdylib\"]\n[dependencies]\nnapi = \"2\"\nnapi-derive = \"2\"\n",
            ),
            (
                "addon/src/lib.rs",
                "use napi_derive::napi;\n\n#[napi]\npub fn open_frame(n: u32) -> u32 { n }\n\n#[napi(js_name = \"shut\")]\nfn close_it() {}\n",
            ),
            (
                "addon/index.js",
                "module.exports = require('./fastcodec.linux-x64-gnu.node');\n",
            ),
        ]);
        let b = detect(&t);
        assert_eq!(b.len(), 1);
        let b = &b[0];
        assert_eq!(b.native, Runtime::Rust);
        assert!(b.cdylib);
        assert_eq!(b.native_dir.as_deref(), Some("addon"));
        let names: Vec<&str> = b.methods.iter().map(|m| m.managed.as_str()).collect();
        assert_eq!(names, ["fastcodec.openFrame", "fastcodec.shut"]);
        assert_eq!(b.methods[0].export, "open_frame");
    }

    #[test]
    fn a_prebuilt_node_file_that_is_required_is_a_boundary() {
        let (_d, t) = tree(&[
            ("vendor/glyphs.node", "\u{7f}ELF"),
            (
                "src/app.js",
                "const g = require('../vendor/glyphs.node');\n",
            ),
        ]);
        let b = detect(&t);
        assert_eq!(b.len(), 1);
        assert_eq!(b[0].library.as_deref(), Some("glyphs"));
        assert!(b[0].methods.is_empty());
    }

    #[test]
    fn an_addon_nobody_loads_is_not_a_boundary() {
        let (_d, t) = tree(&[
            ("binding.gyp", "{\"targets\":[{\"target_name\":\"codec\"}]}"),
            ("src/app.js", "console.log('no addon');\n"),
        ]);
        assert!(detect(&t).is_empty());
    }

    #[test]
    fn a_crate_without_napi_is_not_an_addon() {
        let (_d, t) = tree(&[
            (
                "Cargo.toml",
                "[package]\nname = \"x\"\n[lib]\ncrate-type = [\"cdylib\"]\n",
            ),
            ("index.js", "require('./x.node')\n"),
        ]);
        assert!(detect(&t).is_empty());
    }
}
