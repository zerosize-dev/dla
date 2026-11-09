//! vgdb monitor output on real answers from a C program run under `valgrind --vgdb=yes`
//! (valgrind 3.22): `leak_check`, `block_list` and `who_points_at`.

use std::{fs, path::PathBuf};

use dla_parsers::native::vgdb::{
    Holder, Step, build_chain, next_block_to_query, parse_block_list, parse_leak_check,
    parse_who_points_at, step,
};

fn fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/native")
        .join(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

#[test]
fn leak_check_records() {
    let c = parse_leak_check(&fixture("vgdb_leak_check.txt")).unwrap();
    assert_eq!(c.records.len(), 4);
    let r = &c.records[1];
    assert_eq!(
        (r.nr, r.of, r.kind.as_str(), r.bytes, r.blocks),
        (2, 4, "definitely-lost", 128, 1)
    );
    // Valgrind's calloc frame is dropped.
    assert_eq!(r.frames[0].function, "mk");
    assert_eq!(r.frames[0].line, Some(9));
    assert_eq!(r.frames[1].function, "main");
    assert_eq!(c.records[2].blocks, 3);
}

#[test]
fn leak_check_increased_has_deltas() {
    let c = parse_leak_check(&fixture("vgdb_leak_check_increased.txt")).unwrap();
    assert_eq!(c.records.len(), 1);
    let r = &c.records[0];
    assert_eq!(
        (r.bytes, r.bytes_delta, r.blocks, r.blocks_delta),
        (1088, Some(704), 17, Some(11))
    );
    assert_eq!(r.kind, "still-reachable");
}

#[test]
fn block_list_names_the_blocks_of_a_record() {
    let b = parse_block_list(&fixture("vgdb_block_list.txt")).unwrap();
    assert_eq!(b.record.nr, 3);
    assert_eq!(b.record.blocks, 3);
    let addrs: Vec<_> = b.blocks.iter().map(|x| (x.address, x.size)).collect();
    assert_eq!(addrs, [(0x4A7B230, 64), (0x4A7B2B0, 64), (0x4A7B330, 64)]);
    assert_eq!(b.record.frames[0].function, "mk");

    let l = parse_block_list(&fixture("vgdb_block_list_leak.txt")).unwrap();
    assert_eq!(l.blocks.len(), 20);
    assert_eq!(l.blocks[0].size, 1008);
    assert_eq!(l.record.frames[0].function, "add_session");
    assert_eq!(l.record.frames[0].file.as_deref(), Some("leak.c"));
}

#[test]
fn a_data_symbol_holds_the_head_of_a_list() {
    let w = parse_who_points_at(&fixture("vgdb_who_leak_head.txt")).unwrap();
    assert_eq!(w.target, 0x4A82E10);
    assert_eq!(w.refs.len(), 1);
    assert_eq!(w.refs[0].from, 0x10c030);
    assert_eq!(
        w.refs[0].holder,
        Holder::DataSymbol {
            symbol: "registry".into(),
            offset: 0
        }
    );
    assert_eq!(step(&w), Step::Root(w.refs[0].holder.clone()));
    assert_eq!(next_block_to_query(&w), None);
}

#[test]
fn an_array_symbol_has_an_offset() {
    let w = parse_who_points_at(&fixture("vgdb_who_array.txt")).unwrap();
    assert_eq!(
        w.refs[0].holder,
        Holder::DataSymbol {
            symbol: "cache".into(),
            offset: 16
        }
    );
}

#[test]
fn a_stack_holder_names_its_frame() {
    let w = parse_who_points_at(&fixture("vgdb_who_stack.txt")).unwrap();
    match &w.refs[0].holder {
        Holder::Stack { thread, frame } => {
            assert_eq!(*thread, 1);
            assert!(
                frame
                    .as_deref()
                    .unwrap()
                    .contains("created by main (hold.c:10)"),
                "{frame:?}"
            );
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn a_lost_block_has_no_holder() {
    let w = parse_who_points_at(&fixture("vgdb_who_lost.txt")).unwrap();
    assert!(w.refs.is_empty());
    assert_eq!(step(&w), Step::Unreferenced);
    let chain = build_chain(&[w]);
    assert!(chain.complete);
    assert!(chain.root.is_none());
    assert!(chain.root_path().is_empty());
}

#[test]
fn a_block_holder_carries_its_allocation_stack() {
    let w = parse_who_points_at(&fixture("vgdb_who_block.txt")).unwrap();
    match &w.refs[0].holder {
        Holder::Block {
            address,
            offset,
            size,
            frames,
        } => {
            assert_eq!((*address, *offset, *size), (0x4a7b2b0, 0, 64));
            assert_eq!(frames[0].function, "mk");
            assert_eq!(frames[1].line, Some(13));
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(next_block_to_query(&w), Some(0x4a7b2b0));
}

#[test]
fn walking_up_a_list_reaches_the_global() {
    // Three blocks of a list: the tail block, its holder block, and the head, held by `registry`
    // (and also by a local on the stack; the data symbol wins).
    let answers: Vec<_> = [
        "vgdb_who_block.txt",
        "vgdb_who_block2.txt",
        "vgdb_who_head.txt",
    ]
    .iter()
    .map(|f| parse_who_points_at(&fixture(f)).unwrap())
    .collect();
    assert_eq!(answers[2].refs.len(), 2);
    assert_eq!(next_block_to_query(&answers[0]), Some(0x4a7b2b0));
    assert_eq!(next_block_to_query(&answers[1]), Some(0x4a7b330));
    let chain = build_chain(&answers);
    assert!(chain.complete);
    assert_eq!(
        chain.root,
        Some(Holder::DataSymbol {
            symbol: "registry".into(),
            offset: 0
        })
    );
    // Two hops through the same allocation site merge into one entry.
    assert_eq!(chain.blocks.len(), 1);
    assert_eq!(chain.blocks[0].count, 2);
    assert_eq!(
        chain.root_path(),
        ["static registry", "block mk (hold.c:9) x2"]
    );

    // A walk cut short has no root and is not complete.
    let cut = build_chain(&answers[..2]);
    assert!(!cut.complete);
    assert!(cut.root.is_none());
    assert_eq!(cut.root_path(), ["block mk (hold.c:9) x2"]);
}

#[test]
fn many_holders_of_a_small_value_are_all_typed() {
    let w = parse_who_points_at(&fixture("vgdb_who_many.txt")).unwrap();
    assert_eq!(w.target, 1);
    assert!(w.refs.len() >= 10);
    assert!(
        w.refs
            .iter()
            .all(|r| matches!(r.holder, Holder::DataSymbol { .. }))
    );
    assert!(w.refs.iter().any(|r| matches!(&r.holder, Holder::DataSymbol { symbol, offset } if symbol == "_rtld_local" && *offset == 56)));
}

#[test]
fn segments_stacks_and_len_argument() {
    let text = "==5== Searching for pointers pointing in 8 bytes from 0x4a7b040\n\
==5== *0x483a4d0 points at 0x4a7b040\n Address 0x483a4d0 is in a rw- anonymous segment\n\
==5== *0x48590d0 points at 0x4a7b044\n Address 0x48590d0 is in the BSS segment of /usr/libexec/valgrind/vgpreload_memcheck-amd64-linux.so\n\
==5== *0x4a7b100 points at 0x4a7b040\n Address 0x4a7b100 is 0 bytes inside a block of size 16 free'd\n";
    let w = parse_who_points_at(text).unwrap();
    assert_eq!(w.len, Some(8));
    assert_eq!(w.refs.len(), 3);
    assert!(
        matches!(&w.refs[0].holder, Holder::Segment { description } if description == "a rw- anonymous segment")
    );
    assert!(
        matches!(&w.refs[1].holder, Holder::Segment { description } if description.starts_with("the BSS segment of"))
    );
    assert!(matches!(&w.refs[2].holder, Holder::Other { .. }));
    // A freed block is not a holder to follow.
    assert_eq!(step(&w), Step::Root(w.refs[0].holder.clone()));
}

#[test]
fn error_answers_and_junk_are_errors() {
    let e = parse_block_list(&fixture("vgdb_block_list_bad.txt")).unwrap_err();
    assert!(e.message().contains("invalid loss record nr"), "{e}");
    assert!(e.message().contains("leak_check"), "{e}");
    let e = parse_who_points_at(&fixture("vgdb_who_bad.txt")).unwrap_err();
    assert!(e.message().contains("malformed address"), "{e}");
    for text in ["", "random words", "==1== *0x1 points at 0x2\n"] {
        assert!(parse_who_points_at(text).is_err(), "{text:?}");
        assert!(parse_block_list(text).is_err(), "{text:?}");
        assert!(parse_leak_check(text).is_err(), "{text:?}");
    }
    assert!(parse_who_points_at("Searching for pointers to nothex\n").is_err());
}
