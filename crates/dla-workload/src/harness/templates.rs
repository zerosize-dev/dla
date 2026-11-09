//! The agent and driver sources embedded from the repository's `harness/` directory
//! (spec 4.4), and the placeholder substitution applied when they are written.
//!
//! A placeholder is `@@DLA_NAME@@`. [`fill`] replaces the given ones and fails on any
//! `@@DLA_` left over: an unreplaced placeholder is a generation bug.

/// JVM premain agent.
pub const JVM_AGENT: &str = include_str!("../../../../harness/jvm/DlaAgent.java");
/// JVM dump and probe code shared by the agent and the driver.
pub const JVM_DUMP: &str = include_str!("../../../../harness/jvm/DlaDump.java");
/// JVM raw captures (`--capture`): heap dumps, histograms, flight recording, non-heap reports.
pub const JVM_CAPTURE: &str = include_str!("../../../../harness/jvm/DlaCapture.java");
/// JVM driver template.
pub const JVM_DRIVER: &str = include_str!("../../../../harness/jvm/DlaHarness.java");
/// JVM agent jar manifest.
pub const JVM_MANIFEST: &str = include_str!("../../../../harness/jvm/MANIFEST.MF");
/// Python agent (`sitecustomize`).
pub const PY_AGENT: &str = include_str!("../../../../harness/python/sitecustomize.py");
/// Python driver template.
pub const PY_DRIVER: &str = include_str!("../../../../harness/python/dla_harness.py");
/// pytest-memray variant of the Python driver.
pub const PY_PYTEST: &str = include_str!("../../../../harness/python/test_dla_harness.py");
/// Node preload agent.
pub const NODE_AGENT: &str = include_str!("../../../../harness/node/dla-preload.js");
/// Node driver template.
pub const NODE_DRIVER: &str = include_str!("../../../../harness/node/dla-harness.js");
/// Go command-mode wrapper test.
pub const GO_WRAPPER: &str = include_str!("../../../../harness/go/zz_dla_harness_test.go.tmpl");
/// Go driver test.
pub const GO_DRIVER: &str = include_str!("../../../../harness/go/zz_dla_driver_test.go.tmpl");
/// Go goleak check (build tag `dlagoleak`).
pub const GO_GOLEAK: &str = include_str!("../../../../harness/go/zz_dla_goleak_test.go.tmpl");
/// Allocator shim.
pub const NATIVE_SHIM: &str = include_str!("../../../../harness/native/dla_alloc_shim.c");
/// C driver template.
pub const NATIVE_DRIVER: &str = include_str!("../../../../harness/native/dla_harness.c");
/// Rust driver template.
pub const RUST_DRIVER: &str = include_str!("../../../../harness/rust/dla_harness.rs");

/// File name of the Go wrapper test written next to a `package main`.
pub const GO_WRAPPER_FILE: &str = "zz_dla_harness_test.go";
/// File name of the Go driver test written into the package under test.
pub const GO_DRIVER_FILE: &str = "zz_dla_driver_test.go";
/// File name of the Go goleak check.
pub const GO_GOLEAK_FILE: &str = "zz_dla_goleak_test.go";

/// Replaces every `@@DLA_<NAME>@@` of `vars` (given as `("NAME", value)`) in `template`.
/// An empty `PACKAGE` removes the whole `package ...;` line (default Java package).
/// Errors with the first placeholder left over.
pub fn fill(template: &str, vars: &[(&str, &str)]) -> Result<String, String> {
    let mut out = template.to_owned();
    for (name, value) in vars {
        let token = format!("@@DLA_{name}@@");
        if *name == "PACKAGE" && value.is_empty() {
            out = out
                .lines()
                .filter(|l| !(l.contains(&token) && l.trim_start().starts_with("package ")))
                .collect::<Vec<_>>()
                .join("\n");
            out.push('\n');
        }
        out = out.replace(&token, value);
    }
    if let Some(i) = out.find("@@DLA_") {
        let rest = &out[i..];
        let end = rest[2..].find("@@").map_or(rest.len(), |e| e + 4);
        return Err(format!(
            "unreplaced placeholder {}",
            &rest[..end.min(rest.len())]
        ));
    }
    Ok(out)
}

/// The hook names (`DLA-HOOK <name>`) a filled-in driver asks the model to replace.
pub fn hooks(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        if let Some(i) = line.find("DLA-HOOK ") {
            let rest = &line[i + "DLA-HOOK ".len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            let is_hook = rest[name.len()..].starts_with(':');
            if is_hook && !name.is_empty() && !out.contains(&name) {
                out.push(name);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_replaces_and_flags_leftovers() {
        assert_eq!(fill("a @@DLA_X@@ b", &[("X", "1")]).unwrap(), "a 1 b");
        let err = fill("a @@DLA_Y@@", &[("X", "1")]).unwrap_err();
        assert!(err.contains("@@DLA_Y@@"), "{err}");
    }

    #[test]
    fn empty_package_drops_the_line() {
        let t = "package @@DLA_PACKAGE@@;\n\nclass A {}\n";
        let out = fill(t, &[("PACKAGE", "")]).unwrap();
        assert!(!out.contains("package"), "{out}");
        assert!(out.contains("class A"));
        assert!(
            fill(t, &[("PACKAGE", "shop")])
                .unwrap()
                .contains("package shop;")
        );
    }

    #[test]
    fn every_template_fills_cleanly() {
        let all = [
            ("PACKAGE", "shop"),
            ("CLASS", "DlaLoop"),
            ("STEP", "1"),
            ("WARMUP", "1"),
            ("ROUNDS", "8"),
            ("REPEAT", "64"),
            ("CRATE", "demo"),
        ];
        for t in [
            JVM_DRIVER,
            PY_DRIVER,
            PY_PYTEST,
            NODE_DRIVER,
            GO_WRAPPER,
            GO_DRIVER,
            NATIVE_DRIVER,
            RUST_DRIVER,
            JVM_AGENT,
            JVM_DUMP,
            PY_AGENT,
            NODE_AGENT,
            NATIVE_SHIM,
            GO_GOLEAK,
        ] {
            fill(t, &all).unwrap();
        }
    }

    #[test]
    fn drivers_carry_their_hooks() {
        assert_eq!(hooks(JVM_DRIVER), ["setup", "unit"]);
        assert!(hooks(NATIVE_DRIVER).contains(&"state".to_owned()));
    }
}
