//! `heaptrack_print` text on real output (heaptrack 1.5.0 on a C program and a Rust program)
//! and on malformed input.

use std::{fs, path::PathBuf};

use dla_parsers::native::heaptrack::{self, FoldedCost};

fn fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/native")
        .join(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

#[test]
fn c_leaks_peak_and_totals() {
    let r = heaptrack::parse(&fixture("heaptrack_c.txt")).unwrap();
    assert!(!r.is_diff);
    assert_eq!(r.commands, ["./leak 3000"]);
    assert_eq!(r.totals.allocation_calls, Some(7001));
    assert_eq!(r.totals.temporary_allocations, Some(1000));
    assert_eq!(r.totals.runtime_s, Some(0.11));
    assert_eq!(r.totals.leaked_bytes, Some(4_781_507)); // 4.56M, rounded by the tool
    assert!(r.calls.len() >= 2);
    assert_eq!(r.calls[0].count, 3000);
    assert_eq!(r.calls[0].frames[0].function, "add_session");

    // Leaks: add_session leaked 3000 blocks, called from main (merged backtrace).
    assert_eq!(r.leaks.len(), 2);
    let a = &r.leaks[0];
    assert_eq!((a.count, a.bytes), (3000, 3_166_700));
    let names: Vec<_> = a.frames.iter().map(|f| f.function.as_str()).collect();
    assert_eq!(names, ["add_session", "main"]);
    assert_eq!(a.frames[0].line, Some(11));
    assert!(
        a.frames[0]
            .file
            .as_deref()
            .unwrap()
            .ends_with("/work/leak.c")
    );
    assert!(
        a.frames[0]
            .module
            .as_deref()
            .unwrap()
            .ends_with("/work/leak")
    );
    assert_eq!(a.frames[1].line, Some(23));
    assert_eq!(r.leaks[1].frames[0].function, "drop_ptr");

    // Peak: the same two stacks plus the dynamic loader's own block.
    assert_eq!(r.peak.len(), 3);
    assert_eq!(r.peak[2].frames[0].function, "0x7fc4a90b738e");
    assert_eq!(
        r.peak[2].frames[0].module.as_deref(),
        Some("/lib/x86_64-linux-gnu/libstdc++.so.6")
    );
    let last: Vec<_> = r.peak[2]
        .frames
        .iter()
        .map(|f| f.function.as_str())
        .collect();
    assert_eq!(
        last,
        [
            "0x7fc4a90b738e",
            "call_init",
            "_dl_init::call_init",
            "_dl_init",
            "_dl_start_user"
        ]
    );

    let p = r.leak_profile();
    assert_eq!(p.source, "heaptrack");
    assert_eq!(p.groups.len(), 2);
    assert_eq!(p.groups[0].live_blocks, 3000);
    assert_eq!(p.groups[0].kind.as_deref(), Some("leaked"));
    assert_eq!(r.peak_profile().groups.len(), 3);
}

#[test]
fn diff_between_two_runs_shows_the_growth() {
    let r = heaptrack::parse(&fixture("heaptrack_c_diff.txt")).unwrap();
    assert!(r.is_diff);
    assert_eq!(r.commands.len(), 2);
    // 3000 iterations minus 1000: 2000 more blocks leaked per site.
    assert_eq!(r.leaks.len(), 2);
    assert_eq!(r.leaks[0].count, 2000);
    assert_eq!(r.leaks[0].frames[0].function, "add_session");
    assert_eq!(r.leaks[1].count, 2000);
    // Negative figures keep their sign and the calls section is present in a diff.
    assert!(
        r.calls.iter().any(|s| s.bytes < 0 && s.count == 666),
        "{:?}",
        r.calls
    );
    assert!(
        r.temporary
            .iter()
            .any(|s| s.count == -1 && s.total == Some(0))
    );
    // But the profile has only what grew.
    assert!(r.peak_profile().groups.iter().all(|g| g.live_bytes > 0));
    assert_eq!(r.peak_profile().groups.len(), 2);
}

#[test]
fn unmerged_backtraces() {
    let r = heaptrack::parse(&fixture("heaptrack_c_m0.txt")).unwrap();
    assert_eq!(r.leaks.len(), 2);
    let names: Vec<_> = r.leaks[1]
        .frames
        .iter()
        .map(|f| f.function.as_str())
        .collect();
    assert_eq!(names, ["drop_ptr", "main"]);
    assert_eq!(r.leaks[1].frames[0].line, Some(16));
}

#[test]
fn rust_stacks_are_demangled_and_trimmed() {
    let r = heaptrack::parse(&fixture("heaptrack_rust.txt")).unwrap();
    assert!(!r.leaks.is_empty());
    for s in r.leaks.iter().chain(&r.peak).chain(&r.calls) {
        for f in &s.frames {
            assert!(
                !f.function.starts_with("_R"),
                "not demangled: {}",
                f.function
            );
        }
        let top = &s.frames[0].function;
        assert!(
            !top.starts_with("alloc::alloc")
                && !top.starts_with("<alloc::")
                && !top.starts_with("alloc::raw_vec"),
            "allocator frame left on top: {top}"
        );
    }
    // The map's table and the payload vectors both come through the project's code.
    let big = r
        .leaks
        .iter()
        .find(|s| s.bytes > 1_000_000 && s.frames.iter().any(|f| f.function == "rleak::main"))
        .expect("a large leak stack through rleak::main");
    let names: Vec<_> = big.frames.iter().map(|f| f.function.as_str()).collect();
    assert!(
        names[0].starts_with("hashbrown::") || names[0].starts_with("<hashbrown::"),
        "{names:?}"
    );
    let open = r
        .leaks
        .iter()
        .flat_map(|s| &s.frames)
        .find(|f| f.function == "rleak::open_session")
        .expect("open_session in a leak stack");
    assert_eq!(open.line, Some(12));
    assert!(open.file.as_deref().unwrap().ends_with("src/main.rs"));
    // The std runtime frames of main are kept as printed.
    assert!(big.frames.iter().any(|f| f.function == "main"));
}

#[test]
fn folded_stacks_have_exact_numbers() {
    let p =
        heaptrack::parse_folded(&fixture("heaptrack_c_leaked.folded"), FoldedCost::Bytes).unwrap();
    // The two zero-cost stacks are skipped.
    assert_eq!(p.groups.len(), 2);
    assert_eq!(p.groups[0].live_bytes, 3_024_000);
    let names: Vec<_> = p.groups[0]
        .frames
        .iter()
        .map(|f| f.function.as_str())
        .collect();
    assert_eq!(names, ["add_session", "main"]);
    assert_eq!(p.groups[0].frames[0].file.as_deref(), Some("leak.c"));
    assert_eq!(p.groups[1].live_bytes, 1_536_000);
    let c = heaptrack::parse_folded("main (a.c);f (a.c); 7\n", FoldedCost::Calls).unwrap();
    assert_eq!((c.groups[0].live_bytes, c.groups[0].live_blocks), (0, 7));
}

#[test]
fn malformed_input() {
    for text in ["", "hello world\n", "<xml/>"] {
        let e = heaptrack::parse(text).unwrap_err();
        assert!(e.message().contains("heaptrack_print"), "{e}");
    }
    // A stray line inside a section.
    let e = heaptrack::parse("MEMORY LEAKS\nthis is not a header\n").unwrap_err();
    assert!(e.message().contains("unexpected line"), "{e}");
    // A section with no records, and a cut-off group, are fine.
    let r = heaptrack::parse("MEMORY LEAKS\n\ntotal runtime: 1s.\n").unwrap();
    assert!(r.leaks.is_empty());
    let r =
        heaptrack::parse("MEMORY LEAKS\n1.00K leaked over 2 calls from\nf\n  at a.c:1\n").unwrap();
    assert_eq!(r.leaks[0].bytes, 1024);
    assert_eq!(r.leaks[0].frames[0].line, Some(1));
    // Bad folded input.
    assert!(heaptrack::parse_folded("main (a.c) x\n", FoldedCost::Bytes).is_err());
    assert!(heaptrack::parse_folded("justonetoken\n", FoldedCost::Bytes).is_err());
    assert!(
        heaptrack::parse_folded("", FoldedCost::Bytes)
            .unwrap()
            .groups
            .is_empty()
    );
}
