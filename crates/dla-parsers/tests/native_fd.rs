//! `/proc/<pid>/fd` classification on a real listing (a Python process holding sockets, pipes,
//! an eventfd, epoll, timerfd, inotify, a memfd, an unlinked file and ordinary files).

use std::{collections::BTreeMap, fs, path::PathBuf};

use dla_parsers::native::fd;

fn fixture(name: &str) -> String {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/native")
        .join(name);
    fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

#[test]
fn counts_per_kind_on_a_real_listing() {
    let text = fixture("fd_listing.txt");
    let targets = fd::targets_from_listing(&text);
    assert_eq!(targets.len(), 18);
    let c = fd::counts(targets.iter().copied());
    let expected: BTreeMap<String, u64> = [
        ("dev", 2),
        ("deleted", 1),
        ("eventfd", 1),
        ("eventpoll", 1),
        ("file", 2),
        ("inotify", 1),
        ("memfd", 1),
        ("pipe", 5),
        ("socket", 3),
        ("timerfd", 1),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v))
    .collect();
    assert_eq!(c, expected);
    assert_eq!(c.values().sum::<u64>(), 18);
}

#[test]
fn groups_and_snapshot() {
    let text = fixture("fd_listing.txt");
    let s = fd::snapshot(fd::targets_from_listing(&text));
    assert_eq!(s.source, "proc-fd");
    assert_eq!(s.metrics[0].value, 18);
    assert!(s.groups.iter().all(|g| g.kind == "fd" && g.site.is_none()));
    // Most numerous first, ties by name.
    assert_eq!((s.groups[0].key.as_str(), s.groups[0].count), ("pipe", 5));
    assert_eq!((s.groups[1].key.as_str(), s.groups[1].count), ("socket", 3));
    assert_eq!(s.groups[2].key, "dev");
    assert_eq!(s.groups.iter().map(|g| g.count).sum::<u64>(), 18);
}

#[test]
fn empty_and_odd_input() {
    assert!(fd::counts(std::iter::empty()).is_empty());
    assert!(fd::groups(std::iter::empty()).is_empty());
    assert_eq!(fd::snapshot(std::iter::empty()).metrics[0].value, 0);
    assert!(fd::targets_from_listing("").is_empty());
    assert!(fd::targets_from_listing("\n  \n").is_empty());
    // Garbage is a kind of its own, not an error or a panic.
    let c = fd::counts(["???", "\u{0}", "anon_inode:[Weird Name!]"]);
    assert_eq!(c["other"], 2);
    assert_eq!(c["weird_name"], 1);
}

#[test]
fn an_arrow_inside_the_target_is_kept() {
    // Split at the first arrow: the fd side never contains one.
    assert_eq!(
        fd::targets_from_listing("4 -> /tmp/a -> b"),
        ["/tmp/a -> b"]
    );
    assert_eq!(fd::classify("/tmp/a -> b"), "file");
    assert_eq!(
        fd::targets_from_listing("lrwx------ 1 u g 64 Nov 9 07:00 5 -> /tmp/x -> y (deleted)"),
        ["/tmp/x -> y (deleted)"]
    );
    // A plain target line is not cut.
    assert_eq!(fd::targets_from_listing("/tmp/a -> b"), ["/tmp/a -> b"]);
}
