//! DHAT JSON on real files: `valgrind --tool=dhat` on a C program and the `dhat` crate in a
//! Rust program; plus malformed input.

use std::{fs, path::PathBuf};

use dla_parsers::native::dhat;

fn fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/native")
        .join(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

#[test]
fn valgrind_dhat_on_a_c_program() {
    let r = dhat::parse(&fixture("dhat_valgrind_c.json")).unwrap();
    assert_eq!(r.mode, "heap");
    assert_eq!(r.cmd.as_deref(), Some("./leak 40"));
    assert_eq!(r.pid, Some(17738));
    assert_eq!(r.profile.source, "dhat");
    // Live at exit: 40 sessions of 1008 B and 40 lost blocks of 512 B.
    assert_eq!((r.exit_bytes, r.exit_blocks), (40_320 + 20_480, 80));
    assert_eq!(r.total_bytes, 40_320 + 20_480 + 896);
    let g = &r.profile.groups;
    assert_eq!(g.len(), 3);
    assert_eq!(
        (g[0].live_bytes, g[0].live_blocks, g[0].total_bytes),
        (40_320, 40, Some(40_320))
    );
    // The malloc replacement frame is gone; the allocation site is first.
    assert_eq!(g[0].frames[0].function, "add_session");
    assert_eq!(g[0].frames[0].file.as_deref(), Some("leak.c"));
    assert_eq!(g[0].frames[0].line, Some(11));
    assert_eq!(g[1].frames[0].function, "drop_ptr");
    // A stack that was freed again has no live bytes but keeps its total.
    assert_eq!((g[2].live_bytes, g[2].total_bytes), (0, Some(896)));
}

#[test]
fn dhat_crate_on_a_rust_program() {
    let r = dhat::parse(&fixture("dhat_rust.json")).unwrap();
    assert_eq!(r.mode, "rust-heap");
    // The survey run: 3000 sessions of 2048 bytes held by the static map.
    let top = &r.profile.groups[0];
    assert_eq!(top.live_bytes, 6_144_000);
    assert_eq!(top.live_blocks, 3000);
    // Allocator frames are trimmed: the first frame is the project function.
    assert_eq!(top.frames[0].function, "rleak::open_session");
    assert_eq!(top.frames[0].line, Some(12));
    assert!(
        top.frames
            .iter()
            .any(|f| f.function == "rleak::open_session" && f.line == Some(12))
    );
    assert_eq!(
        r.exit_bytes,
        r.profile.groups.iter().map(|g| g.live_bytes).sum::<u64>()
    );
    // Groups are ordered by live bytes.
    assert!(
        r.profile
            .groups
            .windows(2)
            .all(|w| w[0].live_bytes >= w[1].live_bytes)
    );
    for g in &r.profile.groups {
        assert!(!g.frames.is_empty());
        let f = &g.frames[0].function;
        assert!(
            !f.starts_with("<alloc::alloc::Global") && !f.starts_with("<dhat::"),
            "{f}"
        );
    }
}

#[test]
fn malformed_files() {
    for (text, needle) in [
        ("", "not DHAT JSON"),
        ("hello", "not DHAT JSON"),
        ("[1,2]", "not an object"),
        ("{}", "dhatFileVersion"),
        (r#"{"dhatFileVersion":9,"pps":[],"ftbl":[]}"#, "unsupported"),
        (r#"{"dhatFileVersion":2,"ftbl":[]}"#, "pps"),
        (r#"{"dhatFileVersion":2,"pps":[]}"#, "ftbl"),
        (
            r#"{"dhatFileVersion":2,"pps":[{"eb":1}],"ftbl":["[root]"]}"#,
            "no fs",
        ),
        (
            r#"{"dhatFileVersion":2,"pps":[{"eb":1,"fs":[7]}],"ftbl":["[root]"]}"#,
            "ftbl[7]",
        ),
        (
            r#"{"dhatFileVersion":2,"pps":[{"eb":-1,"fs":[]}],"ftbl":["[root]"]}"#,
            "pps[0].eb",
        ),
        (
            r#"{"dhatFileVersion":2,"pps":[{"fs":["x"]}],"ftbl":["[root]"]}"#,
            "non-integer",
        ),
        (
            r#"{"dhatFileVersion":2,"pps":[1],"ftbl":[]}"#,
            "not an object",
        ),
    ] {
        let e = dhat::parse(text).unwrap_err();
        assert!(e.message().contains(needle), "{text}: {e}");
    }
    let empty = dhat::parse(r#"{"dhatFileVersion":2,"pps":[],"ftbl":["[root]"]}"#).unwrap();
    assert!(empty.profile.groups.is_empty());
}

#[test]
fn valgrind_preload_and_operator_new_frames_are_dropped() {
    let text = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/native/dhat_valgrind_cpp.json"),
    )
    .unwrap();
    let r = dhat::parse(&text).unwrap();
    for g in &r.profile.groups {
        let f = &g.frames[0];
        assert!(!f.function.starts_with("operator new"), "{f:?}");
        assert!(
            !f.module.as_deref().unwrap_or("").contains("/vgpreload_"),
            "{f:?}"
        );
    }
    // The `new Session()` site loses `operator new` and starts at the project function.
    assert!(
        r.profile
            .groups
            .iter()
            .any(|g| g.frames[0].function == "add_session()")
    );
}

#[test]
fn rust_release_stacks_start_at_the_project_function() {
    let text = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/native/dhat_valgrind_rust.json"),
    )
    .unwrap();
    let r = dhat::parse(&text).unwrap();
    let mut keys = Vec::new();
    for fname in ["rleak::open_session", "rleak::fill_cache"] {
        let g = r
            .profile
            .groups
            .iter()
            .find(|g| g.frames.iter().any(|f| f.function == fname))
            .unwrap_or_else(|| panic!("{fname} missing"));
        assert_eq!(g.frames[0].function, fname);
        keys.push(
            g.frames
                .iter()
                .take(8)
                .map(|f| f.function.clone())
                .collect::<Vec<_>>(),
        );
    }
    assert_ne!(keys[0], keys[1]);
}
