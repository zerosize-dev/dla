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
    // A plain query misses interior pointers, so it does not prove a lost block.
    assert!(!chain.complete);
    assert!(chain.root.is_none());
    assert!(chain.root_path().is_empty());
    // With a length, nothing found does.
    let mut with_len = parse_who_points_at(&fixture("vgdb_who_lost.txt")).unwrap();
    with_len.len = Some(64);
    assert!(build_chain(&[with_len]).complete);
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
==5== *0x48590d0 interior points at 4 bytes inside 0x4a7b040\n Address 0x48590d0 is in the BSS segment of /usr/libexec/valgrind/vgpreload_memcheck-amd64-linux.so\n\
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

#[test]
fn thousands_separators_in_record_numbers() {
    // More than 999 records: Valgrind prints `1,050 of 1,105`.
    let b = parse_block_list(&fixture("vgdb_block_list_1050.txt")).unwrap();
    assert_eq!(
        (b.record.nr, b.record.of, b.record.bytes),
        (1050, 1105, 1050)
    );
    assert_eq!(b.blocks.len(), 1);
    assert_eq!(b.record.frames[0].function, "main");
    let b = parse_block_list(&fixture("vgdb_block_list_5_of_1105.txt")).unwrap();
    assert_eq!((b.record.nr, b.record.of), (5, 1105));

    let mut text = String::from("sending command leak_check full reachable any to pid 9\n");
    for i in 1..=1105u32 {
        let nr = if i >= 1000 {
            format!("{},{:03}", i / 1000, i % 1000)
        } else {
            i.to_string()
        };
        text.push_str(&format!(
            "==9== {i} bytes in 1 blocks are still reachable in loss record {nr} of 1,105\n==9==    at 0x4846828: malloc (in /usr/libexec/valgrind/vgpreload_memcheck-amd64-linux.so)\n==9==    by 0x1091BE: main (many.c:{i})\n==9== \n"
        ));
    }
    text.push_str("==9== LEAK SUMMARY:\n==9==    still reachable: 611,065 bytes in 1,105 blocks\n");
    let c = parse_leak_check(&text).unwrap();
    assert_eq!(c.records.len(), 1105);
    assert_eq!((c.records[1049].nr, c.records[1049].of), (1050, 1105));
    assert_eq!(c.records[1049].frames[0].line, Some(1050));
}

#[test]
fn a_summary_with_lost_blocks_but_no_readable_record_is_an_error() {
    let text = "==1== not a header at all\n==1== LEAK SUMMARY:\n==1==    definitely lost: 480 bytes in 15 blocks\n";
    let e = parse_leak_check(text).unwrap_err();
    assert!(e.message().contains("no loss record header"), "{e}");
    // A clean run and an `increased` answer with nothing grown are fine.
    let clean = "==1== LEAK SUMMARY:\n==1==    definitely lost: 0 bytes in 0 blocks\n";
    assert!(parse_leak_check(clean).unwrap().records.is_empty());
    let none_grown =
        "==1== LEAK SUMMARY:\n==1==    definitely lost: 480 (+0) bytes in 15 (+0) blocks\n";
    assert!(parse_leak_check(none_grown).unwrap().records.is_empty());
}

#[test]
fn new_loss_records_keep_their_own_stacks() {
    let c = parse_leak_check(&fixture("vgdb_leak_check_increased_new.txt")).unwrap();
    assert_eq!(c.records.len(), 2);
    let (old, new) = (&c.records[0], &c.records[1]);
    assert!(!old.is_new && new.is_new);
    assert_eq!((old.bytes_delta, new.bytes_delta), (Some(192), Some(4000)));
    let names = |r: &dla_parsers::native::vgdb::LossRecord| {
        r.frames
            .iter()
            .map(|f| f.function.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(names(old), ["step", "main"]);
    assert_eq!(names(new), ["newsite", "step", "main"]);

    // Every record new: none are lost.
    let c = parse_leak_check(&fixture("vgdb_leak_check_all_new.txt")).unwrap();
    assert_eq!(c.records.len(), 2);
    assert!(c.records.iter().all(|r| r.is_new));
    assert_eq!(c.records[1].frames[0].function, "newsite");

    let b = parse_block_list(&fixture("vgdb_block_list_new.txt")).unwrap();
    assert!(b.record.is_new);
    assert_eq!((b.record.nr, b.record.of), (2, 2));
    assert_eq!(b.blocks.len(), 4);
    assert_eq!(b.record.frames[0].function, "newsite");
}

#[test]
fn deltas_with_indirect_blocks() {
    let c = parse_leak_check(&fixture("vgdb_leak_check_indirect_delta.txt")).unwrap();
    let r = &c.records[0];
    assert_eq!(
        (
            r.bytes,
            r.bytes_delta,
            r.direct_bytes,
            r.indirect_bytes,
            r.blocks,
            r.blocks_delta
        ),
        (3380, Some(696), Some(480), Some(2900), 15, Some(3))
    );
    assert!(!r.is_new);
    assert_eq!(r.frames[0].function, "leak_tree");
}

#[test]
fn new_header_with_indirect_blocks_is_kept() {
    let line = "2,684 (+2,684) (384 (+384) direct, 2,300 (+2,300) indirect) bytes in 12 (+12) blocks are definitely lost in new loss record 4 of 4";
    let text = format!("==1== {line}\n==1==    at 0x1: f (a.c:1)\n==1== \n");
    let c = parse_leak_check(&text).unwrap();
    let r = &c.records[0];
    assert!(r.is_new);
    assert_eq!(
        (
            r.bytes,
            r.bytes_delta,
            r.direct_bytes,
            r.indirect_bytes,
            r.blocks_delta
        ),
        (2684, Some(2684), Some(384), Some(2300), Some(12))
    );
}

#[test]
fn a_process_with_no_heap_blocks_is_an_empty_check() {
    let c = parse_leak_check(&fixture("vgdb_leak_check_no_heap.txt")).unwrap();
    assert!(c.records.is_empty());
}

#[test]
fn interior_pointers_are_found_with_a_length() {
    // An intrusive link held by a global: the pointer is 48 bytes into the block.
    let w = parse_who_points_at(&fixture("vgdb_who_interior.txt")).unwrap();
    assert_eq!((w.target, w.len), (0x4a7b040, Some(56)));
    assert_eq!(w.refs.len(), 1);
    assert_eq!(w.refs[0].interior, Some(48));
    assert_eq!(
        w.refs[0].holder,
        Holder::DataSymbol {
            symbol: "registry".into(),
            offset: 0
        }
    );
    assert_eq!(
        step(&w),
        Step::Root(Holder::DataSymbol {
            symbol: "registry".into(),
            offset: 0
        })
    );
    assert!(build_chain(&[w]).complete);

    // C++ new[]: the global points 8 bytes in; a stale stack slot points at the start. The
    // data symbol wins.
    let w = parse_who_points_at(&fixture("vgdb_who_interior_array.txt")).unwrap();
    assert_eq!(w.refs.len(), 2);
    assert_eq!((w.refs[0].interior, w.refs[1].interior), (Some(8), None));
    assert!(
        matches!(&step(&w), Step::Root(Holder::DataSymbol { symbol, .. }) if symbol == "cache")
    );

    // A line that is not understood is skipped, not an error.
    let w = parse_who_points_at("Searching for pointers to 0x10\n*zz points at 0x10\n*0x20 points at 0x10\n Address 0x20 is on thread 1's stack\n").unwrap();
    assert_eq!(w.refs.len(), 1);
}
