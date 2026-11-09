//! LeakSanitizer text on real reports (gcc 13 `-fsanitize=leak` and `-fsanitize=address`, and
//! `liblsan.so` preloaded into a Rust binary, all with `LSAN_OPTIONS=use_globals=0`).

use std::{fs, path::PathBuf};

use dla_parsers::native::lsan;

fn fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/native")
        .join(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

#[test]
fn c_program_direct_and_indirect_leaks() {
    let r = lsan::parse(&fixture("lsan_c.txt")).unwrap();
    assert_eq!(r.pid, Some(1726));
    assert_eq!(r.summary_bytes, Some(45_600));
    assert_eq!(r.summary_allocations, Some(60));
    assert_eq!(r.profile.source, "lsan");
    let g = &r.profile.groups;
    assert_eq!(g.len(), 3);
    assert_eq!(g[0].kind.as_deref(), Some("direct-leak"));
    assert_eq!((g[0].live_bytes, g[0].live_blocks), (15_360, 30));
    // The interceptor frame (malloc in lsan_interceptors.cpp) is gone.
    assert_eq!(g[0].frames[0].function, "drop_ptr");
    assert_eq!(g[0].frames[0].file.as_deref(), Some("/work/leak.c"));
    assert_eq!(g[0].frames[0].line, Some(16));
    // The reachable list: the head is direct, the rest indirect (use_globals=0).
    assert_eq!(
        (g[1].kind.as_deref(), g[1].live_blocks),
        (Some("direct-leak"), 1)
    );
    assert_eq!(g[1].frames[0].function, "add_session");
    assert_eq!(
        (g[2].kind.as_deref(), g[2].live_blocks, g[2].live_bytes),
        (Some("indirect-leak"), 29, 29_232)
    );
    // Frames past main keep the module of an unsymbolized binary frame.
    let start = g[2].frames.last().unwrap();
    assert_eq!(start.function, "_start");
    assert_eq!(start.module.as_deref(), Some("/work/leak_lsan"));
    let total: u64 = g.iter().map(|x| x.live_bytes).sum();
    assert_eq!(Some(total), r.summary_bytes);
}

#[test]
fn asan_leak_report_uses_the_same_records() {
    let r = lsan::parse(&fixture("asan_c.txt")).unwrap();
    assert_eq!(r.summary_bytes, Some(45_600));
    assert_eq!(r.profile.groups.len(), 3);
    assert_eq!(r.profile.groups[0].frames[0].function, "drop_ptr");
}

#[test]
fn rust_symbols_are_demangled_and_allocator_glue_trimmed() {
    let r = lsan::parse(&fixture("lsan_r.txt")).unwrap();
    assert_eq!(r.profile.groups.len(), 3);
    for g in &r.profile.groups {
        assert!(!g.frames[0].function.starts_with("_R"), "{:?}", g.frames[0]);
        assert!(
            !g.frames[0].function.starts_with("alloc::alloc"),
            "{:?}",
            g.frames[0]
        );
    }
    // The payload group: its top frame is hashbrown or the project, never the allocator.
    let names: Vec<_> = r
        .profile
        .groups
        .iter()
        .flat_map(|g| g.frames.iter().map(|f| f.function.as_str()))
        .collect();
    assert!(names.contains(&"rleak::open_session"), "{names:?}");
    assert!(
        names
            .iter()
            .any(|n| n.starts_with("<hashbrown::raw::RawTable<(u64, alloc::vec::Vec<u8>)>>")),
        "{names:?}"
    );
}

#[test]
fn clean_run_and_fatal_error() {
    let r = lsan::parse("").unwrap();
    assert!(r.profile.groups.is_empty());
    let r = lsan::parse("done 30\n").unwrap();
    assert!(r.profile.groups.is_empty());
    let e = lsan::parse(&fixture("lsan_fatal.txt")).unwrap_err();
    assert!(e.message().contains("could not run"), "{e}");
    assert!(e.message().contains("ptrace"), "{e}");
}

#[test]
fn memory_error_before_the_leak_check_is_named() {
    let text = "==9==ERROR: AddressSanitizer: heap-buffer-overflow on address 0x1 at pc 0x2\nWRITE of size 1\n";
    let r = lsan::parse(text).unwrap();
    assert_eq!(
        r.memory_error
            .as_deref()
            .map(|e| e.split(' ').next().unwrap()),
        Some("heap-buffer-overflow")
    );
    assert!(r.profile.groups.is_empty());
}

#[test]
fn malformed_records_are_errors_not_panics() {
    for (text, needle) in [
        ("Direct leak of lots of bytes\n", "counts"),
        (
            "Direct leak of 8 byte(s) in 1 object(s) allocated from:\n\nSUMMARY: x\n",
            "no stack",
        ),
        (
            "Indirect leak of 8 byte(s) in 1 object(s) allocated from:\n    #0 garbage\n",
            "unreadable",
        ),
    ] {
        let e = lsan::parse(text).unwrap_err();
        assert!(e.message().contains(needle), "{text:?}: {e}");
    }
    // A cut-off last frame still parses (the function name is all there is).
    let r = lsan::parse("Direct leak of 8 byte(s) in 1 object(s) allocated from:\n    #0 0x1 in f")
        .unwrap();
    assert_eq!(r.profile.groups[0].frames[0].function, "f");
}

#[test]
fn a_hostile_v0_symbol_in_a_frame_does_not_hang() {
    // Frame #1 is a 60-level nested v0 symbol (instantiating-crate path of nested backrefs).
    fn b62(n: usize) -> String {
        const D: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let mut n = n - 1;
        let mut d = Vec::new();
        while n > 0 {
            d.push(D[n % 62]);
            n /= 62;
        }
        if d.is_empty() {
            d.push(b'0');
        }
        d.reverse();
        format!("{}_", String::from_utf8(d).unwrap())
    }
    fn nest(l: usize, start: usize) -> String {
        if l == 0 {
            return "C1b".into();
        }
        format!("I{}B{}E", nest(l - 1, start + 1), b62(start + 1))
    }
    let sym = format!("_RC1a{}", nest(60, 3));
    let text = format!(
        "Direct leak of 8 byte(s) in 1 object(s) allocated from:\n    #0 0x1 in malloc (/x/liblsan.so+0x1)\n    #1 0x2 in {sym} /w/a.rs:3\n    #2 0x3 in main /w/a.rs:9\n"
    );
    let t = std::time::Instant::now();
    let r = lsan::parse(&text).unwrap();
    assert!(t.elapsed() < std::time::Duration::from_secs(10));
    assert_eq!(r.profile.groups.len(), 1);
}
