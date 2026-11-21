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
/// JVM JUnit repeat driver template: runs the project's own JUnit 5 tests in a loop, no hooks.
pub const JVM_JUNIT_DRIVER: &str = include_str!("../../../../harness/jvm/DlaJUnitHarness.java");
/// Gradle init script of the JUnit repeat build step: prints the test runtime class path.
pub const JVM_JUNIT_INIT: &str = include_str!("../../../../harness/jvm/dla-junit.init.gradle");
/// Build step script of the JUnit repeat harness (class path, driver compile, argument file).
pub const JVM_JUNIT_BUILD: &str = include_str!("../../../../harness/jvm/dla-junit-build.sh");
/// JVM agent jar manifest.
pub const JVM_MANIFEST: &str = include_str!("../../../../harness/jvm/MANIFEST.MF");
/// Python agent (`sitecustomize`).
pub const PY_AGENT: &str = include_str!("../../../../harness/python/sitecustomize.py");
/// Python memray report helper, run with the target's interpreter (`--capture alloc`).
pub const PY_MEMRAY: &str = include_str!("../../../../harness/python/dla_memray_leaks.py");
/// Python driver template.
pub const PY_DRIVER: &str = include_str!("../../../../harness/python/dla_harness.py");
/// pytest-memray variant of the Python driver.
pub const PY_PYTEST: &str = include_str!("../../../../harness/python/test_dla_harness.py");
/// Node preload agent.
pub const NODE_AGENT: &str = include_str!("../../../../harness/node/dla-preload.cjs");
/// Node driver template.
pub const NODE_DRIVER: &str = include_str!("../../../../harness/node/dla-harness.cjs");
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
            ("SELECT", "\"shop.CartTest\""),
        ];
        for t in [
            JVM_DRIVER,
            JVM_JUNIT_DRIVER,
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

    /// The text between the engine markers of a Go template.
    fn go_engine(t: &str) -> &str {
        let a = t.find("// DLA-ENGINE-BEGIN").unwrap();
        let b = t.find("// DLA-ENGINE-END").unwrap();
        &t[a..b]
    }

    #[test]
    fn the_go_templates_share_one_engine_block_and_only_command_mode_samples() {
        assert_eq!(go_engine(GO_WRAPPER), go_engine(GO_DRIVER));
        // The sampler and its hooks being set belong to the command-mode part alone.
        assert!(GO_WRAPPER.contains("dlaInRunHook, dlaInRunProfile = "));
        assert!(!GO_DRIVER.contains("func dlaStartSampler"));
        assert!(GO_WRAPPER.contains("func dlaStartSampler"));
        assert!(!GO_DRIVER.contains("dlaInRunHook ="));
        assert!(!GO_DRIVER.contains("dlaInRunHook, "));
    }

    #[test]
    fn drivers_carry_their_hooks() {
        assert_eq!(hooks(JVM_DRIVER), ["setup", "unit"]);
        assert!(hooks(JVM_JUNIT_DRIVER).is_empty());
        assert!(hooks(NATIVE_DRIVER).contains(&"state".to_owned()));
    }
    /// The text between `DLA-CHOICE-BEGIN` and `DLA-CHOICE-END` of a template.
    fn choice_block(template: &str) -> &str {
        let a = template.find("DLA-CHOICE-BEGIN").expect("begin marker");
        let a = a + template[a..].find('\n').expect("line end") + 1;
        let b = template.find("DLA-CHOICE-END").expect("end marker");
        let b = template[..b].rfind('\n').expect("line start");
        &template[a..b]
    }

    /// Runs `program` (a file `name` in a temp dir) through `cmd`; `None` when the tool is
    /// missing. The program prints the number of distinct (slot 0, slot 1) pairs over 8 keys
    /// (2 x 4 options) and whether a second pass gave the same answers.
    fn run_choice_program(name: &str, program: &str, steps: &[&[&str]]) -> Option<String> {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(name), program).unwrap();
        let mut last = String::new();
        for step in steps {
            let out = std::process::Command::new(step[0])
                .args(&step[1..])
                .current_dir(dir.path())
                .output()
                .ok()?;
            assert!(
                out.status.success(),
                "{step:?}: {}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            last = String::from_utf8_lossy(&out.stdout).trim().to_owned();
        }
        Some(last)
    }

    const WALKED: &str = "pairs=8 stable=true";

    #[test]
    fn the_python_choice_helper_walks_every_combination_of_two_slots_in_n1_times_n2_keys() {
        let prog = format!(
            "{}\nseen = set()\nfirst = []\nfor rep in range(2):\n    got = []\n    for key in range(8):\n        got.append((dla_choice(key, 0, 2), dla_choice(key, 1, 4)))\n    if rep == 0:\n        first = got\n        seen = set(got)\nprint(f'pairs={{len(seen)}} stable={{str(got == first).lower()}}')\n",
            choice_block(PY_DRIVER)
        );
        if let Some(out) = run_choice_program("c.py", &prog, &[&["python3", "c.py"]]) {
            assert_eq!(out, WALKED);
        }
    }

    #[test]
    fn the_node_choice_helper_walks_every_combination_of_two_slots_in_n1_times_n2_keys() {
        let prog = format!(
            "{}\nlet first = null; let seen = new Set(); let stable = true;\nfor (let rep = 0; rep < 2; rep += 1) {{ const got = []; for (let key = 0; key < 8; key += 1) got.push(dlaChoice(key, 0, 2) + ',' + dlaChoice(key, 1, 4)); if (rep === 0) {{ first = got.join(';'); seen = new Set(got); }} else stable = got.join(';') === first; }}\nconsole.log(`pairs=${{seen.size}} stable=${{stable}}`);\n",
            choice_block(NODE_DRIVER)
        );
        if let Some(out) = run_choice_program("c.cjs", &prog, &[&["node", "c.cjs"]]) {
            assert_eq!(out, WALKED);
        }
    }

    #[test]
    fn the_go_choice_helper_walks_every_combination_of_two_slots_in_n1_times_n2_keys() {
        let prog = format!(
            "package main\n\nimport (\n\t\"fmt\"\n\tdlastrconv \"strconv\"\n)\n\n{}\nfunc main() {{\n\tseen := map[[2]int]bool{{}}\n\tvar first [8][2]int\n\tstable := true\n\tfor rep := 0; rep < 2; rep++ {{\n\t\tfor key := 0; key < 8; key++ {{\n\t\t\tp := [2]int{{dlaChoice(key, 0, 2), dlaChoice(key, 1, 4)}}\n\t\t\tif rep == 0 {{\n\t\t\t\tfirst[key] = p\n\t\t\t\tseen[p] = true\n\t\t\t}} else if first[key] != p {{\n\t\t\t\tstable = false\n\t\t\t}}\n\t\t}}\n\t}}\n\t_ = dlaChoicesNote(1)\n\tfmt.Printf(\"pairs=%d stable=%v\\n\", len(seen), stable)\n}}\n",
            choice_block(GO_DRIVER)
        );
        if let Some(out) = run_choice_program("c.go", &prog, &[&["go", "run", "c.go"]]) {
            assert_eq!(out, WALKED);
        }
    }

    #[test]
    fn the_rust_choice_helper_walks_every_combination_of_two_slots_in_n1_times_n2_keys() {
        let prog = format!(
            "use std::sync::atomic::{{AtomicU64, Ordering::Relaxed}};\n{}\nfn main() {{\n    let mut first = Vec::new();\n    let mut stable = true;\n    let mut seen = std::collections::BTreeSet::new();\n    for rep in 0..2 {{\n        for key in 0..8u64 {{\n            let p = (choice(key, 0, 2), choice(key, 1, 4));\n            if rep == 0 {{ first.push(p); seen.insert(p); }} else if first[key as usize] != p {{ stable = false; }}\n        }}\n    }}\n    let _ = choices_note(1);\n    println!(\"pairs={{}} stable={{stable}}\", seen.len());\n}}\n",
            choice_block(RUST_DRIVER)
        );
        if let Some(out) = run_choice_program(
            "c.rs",
            &prog,
            &[&["rustc", "--edition", "2024", "-o", "c", "c.rs"], &["./c"]],
        ) {
            assert_eq!(out, WALKED);
        }
    }

    #[test]
    fn the_c_choice_helper_walks_every_combination_of_two_slots_in_n1_times_n2_keys() {
        let prog = format!(
            "#include <stdio.h>\n{}\nint main(void) {{\n    int seen[2][4] = {{{{0}}}};\n    unsigned long first[8][2];\n    int stable = 1, pairs = 0;\n    for (int rep = 0; rep < 2; rep++)\n        for (unsigned long key = 0; key < 8; key++) {{\n            unsigned long a = dla_choice(key, 0, 2), b = dla_choice(key, 1, 4);\n            if (rep == 0) {{ first[key][0] = a; first[key][1] = b; if (!seen[a][b]++) pairs++; }}\n            else if (first[key][0] != a || first[key][1] != b) stable = 0;\n        }}\n    choices_report(12);\n    printf(\"pairs=%d stable=%s\\n\", pairs, stable ? \"true\" : \"false\");\n    return 0;\n}}\n",
            choice_block(NATIVE_DRIVER)
        );
        if let Some(out) = run_choice_program("c.c", &prog, &[&["cc", "-o", "c", "c.c"], &["./c"]])
        {
            assert_eq!(out, WALKED);
        }
    }

    #[test]
    fn the_jvm_choice_helper_walks_every_combination_of_two_slots_in_n1_times_n2_keys() {
        let prog = format!(
            "import java.util.*;\npublic class C {{\n{}\n    public static void main(String[] a) {{\n        Set<String> seen = new HashSet<>();\n        String[] first = new String[8];\n        boolean stable = true;\n        for (int rep = 0; rep < 2; rep++)\n            for (int key = 0; key < 8; key++) {{\n                String p = choice(key, 0, 2) + \",\" + choice(key, 1, 4);\n                if (rep == 0) {{ first[key] = p; seen.add(p); }} else if (!first[key].equals(p)) stable = false;\n            }}\n        choicesNote(12);\n        System.out.println(\"pairs=\" + seen.size() + \" stable=\" + stable);\n    }}\n}}\n",
            choice_block(JVM_DRIVER)
        );
        if let Some(out) = run_choice_program("C.java", &prog, &[&["java", "C.java"]]) {
            assert_eq!(out, WALKED);
        }
    }

    #[test]
    fn the_rust_driver_uses_no_let_chain() {
        // The driver builds in the project's own edition: let chains need 2024 (a real run of
        // the choice helper failed to build on a 2021 crate).
        assert!(
            !RUST_DRIVER.contains("&& let "),
            "let chain in the Rust driver"
        );
    }

    #[test]
    fn a_choice_call_is_counted_outside_comments_and_definitions_only() {
        use dla_core::harness::choice_calls_in;
        let text = "def dla_choice(key, slot, n):\n    pass\n# dla_choice(key, 0, 2)\ndef unit(i, key):\n    a = dla_choice(key, 0, 3)\n    b = x.dla_choice(key, 1, 2) + my_dla_choice(key, 2, 2)\n    return dla_choice(key, 1, 2)\n";
        assert_eq!(choice_calls_in(text, "dla_choice"), 3);
        assert_eq!(
            choice_calls_in(
                "fn choice(k: u64) {}\n// choice(1)\nlet a = choice(k, 0, 2);",
                "choice"
            ),
            1
        );
    }
}
