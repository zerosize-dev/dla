//! Valgrind memcheck XML on real output (valgrind 3.22, small C programs) and on bad input.

use std::{fs, path::PathBuf};

use dla_parsers::native::valgrind;

fn fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/native")
        .join(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

#[test]
fn definite_and_reachable_records_with_stacks() {
    let r = valgrind::parse(&fixture("valgrind_leak.xml")).unwrap();
    assert!(r.complete);
    assert_eq!(r.pid, Some(3881));
    assert_eq!(r.command.as_deref(), Some("./leak_plain 500"));
    assert_eq!(r.profile.source, "valgrind");
    assert!(!r.profile.sampled);
    assert_eq!(r.profile.groups.len(), 2);

    let lost = &r.profile.groups[0];
    assert_eq!(lost.kind.as_deref(), Some("definitely-lost"));
    assert_eq!((lost.live_bytes, lost.live_blocks), (256_000, 500));
    // The malloc replacement frame is dropped; the first frame is the project function.
    assert_eq!(lost.frames[0].function, "drop_ptr");
    assert_eq!(lost.frames[0].line, Some(16));
    assert_eq!(lost.frames[0].file.as_deref(), Some("/work/leak.c"));
    assert_eq!(lost.frames[1].function, "main");

    let reachable = &r.profile.groups[1];
    assert_eq!(reachable.kind.as_deref(), Some("still-reachable"));
    assert_eq!(
        (reachable.live_bytes, reachable.live_blocks),
        (504_000, 500)
    );
    assert_eq!(reachable.frames[0].function, "add_session");
    assert_eq!(reachable.frames[0].line, Some(11));
}

#[test]
fn all_four_leak_kinds() {
    let r = valgrind::parse(&fixture("valgrind_kinds.xml")).unwrap();
    let kinds: Vec<_> = r
        .profile
        .groups
        .iter()
        .map(|g| (g.kind.clone().unwrap(), g.live_bytes, g.live_blocks))
        .collect();
    assert_eq!(
        kinds,
        [
            ("still-reachable".to_owned(), 100, 1),
            ("possibly-lost".to_owned(), 200, 1),
            ("indirectly-lost".to_owned(), 1200, 4),
            ("definitely-lost".to_owned(), 32, 4),
        ]
    );
    assert!(r.other_errors.is_empty());
    let possibly = &r.profile.groups[1];
    assert_eq!(possibly.frames[0].function, "mk_possible");
    assert_eq!(possibly.frames[0].module.as_deref(), Some("/work/kinds"));
}

#[test]
fn memory_errors_are_listed_apart_from_leaks() {
    let r = valgrind::parse(&fixture("valgrind_invalid_write.xml")).unwrap();
    assert_eq!(r.other_errors, ["InvalidWrite"]);
    assert_eq!(r.profile.groups.len(), 2);
    assert!(
        r.profile
            .groups
            .iter()
            .all(|g| g.kind.as_deref() == Some("definitely-lost"))
    );
}

#[test]
fn a_killed_run_is_parsed_as_far_as_it_goes() {
    let full = fixture("valgrind_leak.xml");
    // Cut inside the second record: the first one is whole.
    let cut = &full[..full.find("loss record 2").unwrap()];
    let r = valgrind::parse(cut).unwrap();
    assert!(!r.complete);
    assert_eq!(r.profile.groups.len(), 1);
}

#[test]
fn cpp_names_with_entities_are_decoded() {
    let xml = r#"<valgrindoutput><tool>memcheck</tool><status><state>FINISHED</state></status>
<error><kind>Leak_DefinitelyLost</kind>
<xwhat><text>t</text><leakedbytes>48</leakedbytes><leakedblocks>2</leakedblocks></xwhat>
<stack><frame><ip>0x1</ip><obj>/usr/libexec/valgrind/vgpreload_memcheck-amd64-linux.so</obj><fn>operator new(unsigned long)</fn></frame>
<frame><ip>0x2</ip><obj>/a/app</obj><fn>std::vector&lt;int, std::allocator&lt;int&gt; &gt;::push_back(int const&amp;)</fn><dir>/src</dir><file>v.cc</file><line>7</line></frame></stack></error>
</valgrindoutput>"#;
    let r = valgrind::parse(xml).unwrap();
    assert!(r.complete);
    let g = &r.profile.groups[0];
    assert_eq!(g.frames.len(), 1);
    assert_eq!(
        g.frames[0].function,
        "std::vector<int, std::allocator<int> >::push_back(int const&)"
    );
    assert_eq!(g.frames[0].file.as_deref(), Some("/src/v.cc"));
}

#[test]
fn rust_symbols_in_frames_are_demangled() {
    let xml = r#"<valgrindoutput><tool>memcheck</tool>
<error><kind>Leak_StillReachable</kind>
<xwhat><leakedbytes>9</leakedbytes><leakedblocks>1</leakedblocks></xwhat>
<stack><frame><ip>0x2</ip><fn>_RNvCsinJrbhJGnG5_5rleak12open_session</fn></frame></stack></error>
</valgrindoutput>"#;
    let r = valgrind::parse(xml).unwrap();
    assert_eq!(
        r.profile.groups[0].frames[0].function,
        "rleak::open_session"
    );
    assert!(!r.complete);
}

#[test]
fn bad_input_gives_clear_errors() {
    for (text, needle) in [
        ("", "valgrindoutput"),
        (
            "==123== Memcheck, a memory error detector",
            "valgrindoutput",
        ),
        ("<html><body/></html>", "valgrindoutput"),
        (
            "<valgrindoutput><tool>helgrind</tool></valgrindoutput>",
            "helgrind",
        ),
        (&"<a>".repeat(500), "nested"),
    ] {
        let e = valgrind::parse(text).unwrap_err();
        assert!(e.message().contains(needle), "{text:.40}: {e}");
    }
    // Missing numbers do not panic; the group is empty of figures.
    let r = valgrind::parse(
        "<valgrindoutput><error><kind>Leak_PossiblyLost</kind></error></valgrindoutput>",
    )
    .unwrap();
    assert_eq!(r.profile.groups[0].live_bytes, 0);
}

#[test]
fn owned_blocks_are_not_counted_twice() {
    // lp.c: a 16-byte block that holds a 100-byte block, both dropped. memcheck writes
    // leakedbytes = 116 for the head, and the 100 again as its own indirectly-lost record.
    let r = valgrind::parse(&fixture("valgrind_owned.xml")).unwrap();
    let by_kind = |k: &str| {
        r.profile
            .groups
            .iter()
            .find(|g| g.kind.as_deref() == Some(k))
            .map(|g| (g.live_bytes, g.live_blocks))
    };
    assert_eq!(by_kind("definitely-lost"), Some((16, 1)));
    assert_eq!(by_kind("indirectly-lost"), Some((100, 1)));
    let total: u64 = r.profile.groups.iter().map(|g| g.live_bytes).sum();
    assert_eq!(total, 116);

    // ll.c: a 10-node list pushed from one stack.
    let r = valgrind::parse(&fixture("valgrind_list_owned.xml")).unwrap();
    let total: u64 = r.profile.groups.iter().map(|g| g.live_bytes).sum();
    assert_eq!(total, 640);
}

#[test]
fn rust_library_frames_are_trimmed_and_two_sites_stay_apart() {
    // Release Rust binary under memcheck: Valgrind names inlined std frames without paths.
    let r = valgrind::parse(&fixture("valgrind_rust_two_sites.xml")).unwrap();
    let key = |g: &dla_parsers::model::AllocGroup| {
        g.frames
            .iter()
            .take(8)
            .map(|f| format!("{}@{:?}", f.function, f.line))
            .collect::<Vec<_>>()
    };
    let sites: Vec<_> = r
        .profile
        .groups
        .iter()
        .filter(|g| g.frames.iter().any(|f| f.function.starts_with("rleak::")))
        .collect();
    assert!(sites.len() >= 2);
    for g in &sites {
        assert!(
            g.frames[0].function.starts_with("rleak::"),
            "{:?}",
            g.frames.iter().map(|f| &f.function).collect::<Vec<_>>()
        );
    }
    let mut keys: Vec<_> = sites.iter().map(|g| key(g)).collect();
    keys.sort();
    keys.dedup();
    assert_eq!(keys.len(), sites.len());
}
