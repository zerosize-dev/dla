//! File descriptor kinds for the `fd.<kind>` columns (Linux).
//!
//! Each `/proc/<pid>/fd/N` link target is classified:
//!
//! | Target | Column |
//! |---|---|
//! | `socket:[123]` | `fd.socket` |
//! | `pipe:[123]` | `fd.pipe` |
//! | `anon_inode:[eventfd]`, `[eventpoll]`, `[timerfd]` | `fd.eventfd`, `fd.eventpoll`, `fd.timerfd` (the inner name; `anon_inode:inotify` gives `fd.inotify`) |
//! | `/memfd:name (deleted)` | `fd.memfd` |
//! | `/tmp/x (deleted)` | `fd.deleted` |
//! | `/dev/...` | `fd.dev` |
//! | any other absolute path | `fd.file` |
//! | anything else | `fd.other` |
//!
//! The fixed kinds in [`BASE_KINDS`] are always reported (as 0 when none is open) so a series
//! has a value at every sample; other kinds appear when a descriptor of that kind exists. The
//! columns add up to `os.fds`, less descriptors closed or unreadable while the directory was
//! read. The classification itself is `dla_parsers::native::fd::classify`.

use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;

use super::OsSample;

/// Kinds reported at every sample.
pub(super) const BASE_KINDS: [&str; 8] = [
    "socket",
    "pipe",
    "file",
    "deleted",
    "dev",
    "eventfd",
    "eventpoll",
    "timerfd",
];

/// The kind of one link target (the classification lives in `dla_parsers::native::fd`).
pub(super) fn classify(target: &str) -> String {
    dla_parsers::native::fd::classify(target)
}

/// Records the `fd.<kind>` columns from the results of reading each descriptor link of `pid`.
/// A link that vanished (the descriptor was closed meanwhile) is skipped; when links exist but
/// none can be read the columns are left out and the loss is explained.
pub(super) fn record(
    out: &mut OsSample,
    pid: u32,
    links: impl IntoIterator<Item = io::Result<PathBuf>>,
) {
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let (mut read, mut denied) = (0u64, 0u64);
    let mut first_error = None;
    for link in links {
        match link {
            Ok(target) => {
                read += 1;
                *counts
                    .entry(classify(&target.to_string_lossy()))
                    .or_insert(0) += 1;
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => {
                denied += 1;
                first_error.get_or_insert(e);
            }
        }
    }
    if read == 0 && denied > 0 {
        let why = first_error.map(|e| e.to_string()).unwrap_or_default();
        out.limit(format!(
            "fd.<kind>: /proc/{pid}/fd links unreadable ({why})"
        ));
        return;
    }
    if denied > 0 {
        out.limit(format!(
            "fd.<kind>: {denied} of {} links in /proc/{pid}/fd unreadable",
            read + denied
        ));
    }
    for kind in BASE_KINDS {
        counts.entry(kind.to_owned()).or_insert(0);
    }
    for (kind, n) in counts {
        out.put(&format!("fd.{kind}"), n);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The same table as dla-parsers/tests/native_fd.rs and native::fd.
    #[test]
    fn classification_table() {
        for (target, kind) in [
            ("socket:[48112]", "socket"),
            ("pipe:[9023]", "pipe"),
            ("anon_inode:[eventfd]", "eventfd"),
            ("anon_inode:[eventpoll]", "eventpoll"),
            ("anon_inode:[timerfd]", "timerfd"),
            ("anon_inode:[signalfd]", "signalfd"),
            ("anon_inode:inotify", "inotify"),
            ("anon_inode:bpf-map", "bpf_map"),
            ("anon_inode:", "anon_inode"),
            ("/memfd:jit (deleted)", "memfd"),
            ("/tmp/#682325 (deleted)", "deleted"),
            ("/dev/null", "dev"),
            ("/dev/pts/3", "dev"),
            ("/etc/hostname", "file"),
            ("/home/user/with space.txt", "file"),
            ("net:[4026531840]", "other"),
            ("", "other"),
        ] {
            assert_eq!(classify(target), kind, "{target}");
        }
    }

    #[test]
    fn record_counts_and_explains_losses() {
        let ok = |s: &str| Ok(PathBuf::from(s));
        let mut out = OsSample::default();
        record(
            &mut out,
            1,
            [
                ok("socket:[1]"),
                ok("socket:[2]"),
                ok("/etc/hostname"),
                Err(io::ErrorKind::NotFound.into()),
            ],
        );
        assert_eq!(out.values["fd.socket"], 2.0);
        assert_eq!(out.values["fd.file"], 1.0);
        assert_eq!(out.values["fd.pipe"], 0.0);
        assert!(out.limitations.is_empty());

        let mut out = OsSample::default();
        record(
            &mut out,
            7,
            [Err(io::ErrorKind::PermissionDenied.into()), ok("pipe:[3]")],
        );
        assert_eq!(out.values["fd.pipe"], 1.0);
        assert!(
            out.limitations[0].contains("1 of 2"),
            "{:?}",
            out.limitations
        );

        let mut out = OsSample::default();
        record(&mut out, 7, [Err(io::ErrorKind::PermissionDenied.into())]);
        assert!(out.values.is_empty());
        assert!(out.limitations[0].contains("unreadable"));

        // A process with no descriptors still reports zeros.
        let mut out = OsSample::default();
        record(&mut out, 7, []);
        assert_eq!(out.values["fd.socket"], 0.0);
    }
}
