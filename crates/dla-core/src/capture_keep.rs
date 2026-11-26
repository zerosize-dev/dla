//! Which large capture files of a run may be deleted to keep the run under a size limit.
//!
//! A capturing `dla harness run` of a JVM or Node harness writes heap dumps of 100 MB and more
//! (HPROF, V8 snapshots) and flight recordings, and a run makes tens of harnesses, so the
//! captures of a long run can fill the disk. The tool keeps the large captures of the run under
//! `DLA_CAPTURE_KEEP_MB` by deleting whole capture sets (the large files of one harness and one
//! label), oldest first, and only sets that a summary already read and that the current run did
//! not just write. This module holds the rules that need no disk access (which files are large,
//! which sets go) and the reading of which digests a summary read; the deleting is in
//! `dla-workload`.
//!
//! Which kinds count: a file is large when it is a heap dump (`heap-<n>.hprof`, `heap-<n>.heapsnapshot`)
//! or a flight recording (`recording.jfr`). Class histograms, thread dumps, native memory
//! summaries, sampling heap profiles, holder chains (`py-<n>.retain.json`) and the text reports
//! of valgrind and heaptrack are text or small profiles (kilobytes to a few megabytes) and are
//! neither counted nor deleted.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::Path,
};

use sha2::{Digest, Sha256};

use crate::{
    evidence::Source,
    harness::{CaptureFile, CaptureKind},
    ledger::Ledger,
};

/// The limit, in megabytes, of the large captures a run keeps.
pub const KEEP_ENV: &str = "DLA_CAPTURE_KEEP_MB";
/// The default of [`KEEP_ENV`].
pub const DEFAULT_KEEP_MB: f64 = 2048.0;
/// Free disk, in megabytes, a capturing run leaves after the dumps it is about to take.
pub const FLOOR_ENV: &str = "DLA_DISK_FLOOR_MB";
/// The default of [`FLOOR_ENV`].
pub const DEFAULT_FLOOR_MB: f64 = 1024.0;
/// Free disk, in bytes, instead of the measured figure (tests, and a box whose figure is wrong).
pub const FREE_DISK_ENV: &str = "DLA_FREE_DISK_BYTES";

/// Bytes of the megabyte figure in the environment variable `name`, `default_mb` when it is
/// unset or not a non-negative number. Decimals are accepted (`0.5`).
pub fn megabytes_from_env(name: &str, default_mb: f64) -> u64 {
    let mb = std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0)
        .unwrap_or(default_mb);
    (mb * 1024.0 * 1024.0) as u64
}

/// SHA-256 of a file's bytes (lowercase hex), streamed; `None` when it cannot be read.
pub fn sha256_file(path: &Path) -> Option<String> {
    let mut f = fs::File::open(path).ok()?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = f.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Some(hex::encode(h.finalize()))
}

/// True when `f` is a file the size limit counts and may delete: a heap dump or a flight
/// recording (see the module text for what is left out).
pub fn is_large(f: &CaptureFile) -> bool {
    matches!(f.kind, CaptureKind::Heap | CaptureKind::Jfr) && !f.path.ends_with(".json")
}

/// One capture set: the large files of one harness and one label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetInfo {
    /// The harness (`H3`).
    pub harness: String,
    /// The label of the point (`p1-fresh`).
    pub label: String,
    /// Bytes of the large files still on disk.
    pub bytes: u64,
    /// When the newest large file of the set was written (RFC 3339, UTC: sorts as text).
    pub captured_at: String,
    /// A summary read at least one large file of the set (the model chose the dumps it read).
    pub read: bool,
    /// The set belongs to the run being made now (never deleted).
    pub current: bool,
}

/// The sets to delete, as indexes into `sets`, so that at least `need` bytes are freed: oldest
/// `captured_at` first, only sets that were read and are not current, and no more than it takes.
/// Fewer bytes are freed when too few sets may go; an empty answer when `need` is 0.
pub fn choose_to_free(sets: &[SetInfo], need: u64) -> Vec<usize> {
    if need == 0 {
        return Vec::new();
    }
    let mut order: Vec<usize> = (0..sets.len())
        .filter(|&i| sets[i].read && !sets[i].current && sets[i].bytes > 0)
        .collect();
    order.sort_by(|&a, &b| {
        (&sets[a].captured_at, &sets[a].harness, &sets[a].label).cmp(&(
            &sets[b].captured_at,
            &sets[b].harness,
            &sets[b].label,
        ))
    });
    let mut freed = 0u64;
    let mut out = Vec::new();
    for i in order {
        if freed >= need {
            break;
        }
        freed = freed.saturating_add(sets[i].bytes);
        out.push(i);
    }
    out
}

/// The sets to delete to bring the total of `sets` under `limit` bytes: nothing when it is
/// under already, else [`choose_to_free`] for the excess.
pub fn choose(sets: &[SetInfo], limit: u64) -> Vec<usize> {
    let total: u64 = sets.iter().map(|s| s.bytes).sum();
    choose_to_free(sets, total.saturating_sub(limit))
}

/// The directories of a run whose JSON files list, in a `captures` array, the SHA-256 of each
/// input a summary read.
const SUMMARY_DIRS: [&str; 4] = ["allocs", "retain", "histograms", "nonheap"];

/// The digests of the input files that summaries read, with the evidence that read each.
#[derive(Debug, Default, Clone)]
pub struct Reads {
    /// Full digest to the evidence ids that name it (empty when only a file names it).
    full: BTreeMap<String, BTreeSet<String>>,
    /// The first 8 hex digits of a digest named by a per-snapshot summary file
    /// (`summaries/<stem>-<digest8>.json`), which a worker's summary leaves before its evidence
    /// is merged into the ledger.
    short: BTreeSet<String>,
}

impl Reads {
    /// The summaries that read the file with `digest`: `None` when none did, `Some` (the ids
    /// of the evidence records, possibly none when only a summary file names it) when one did.
    pub fn read_by(&self, digest: &str) -> Option<Vec<String>> {
        if let Some(ids) = self.full.get(digest) {
            return Some(ids.iter().filter(|i| !i.is_empty()).cloned().collect());
        }
        (digest.len() >= 8 && self.short.contains(&digest[..8])).then(Vec::new)
    }

    /// Notes that evidence `id` (empty when unknown) read the file with `digest`.
    pub fn add(&mut self, digest: &str, id: &str) {
        self.full
            .entry(digest.to_owned())
            .or_default()
            .insert(id.to_owned());
    }
}

/// Which input digests the summaries of `run_dir` read: the sources and the `captures` lists of
/// the evidence records of `ledger` (when given), the `captures` lists of the raw files in
/// `allocs/`, `retain/`, `histograms/` and `nonheap/`, and the digest in the name of each
/// per-snapshot summary in `summaries/`.
pub fn read_digests(run_dir: &Path, ledger: Option<&Ledger>) -> Reads {
    let mut reads = Reads::default();
    let captures_of = |doc: &serde_json::Value| -> Vec<String> {
        doc.get("captures")
            .and_then(|c| c.as_array())
            .into_iter()
            .flatten()
            .filter_map(|c| c.get("sha256").and_then(|d| d.as_str()).map(str::to_owned))
            .collect()
    };
    if let Some(ledger) = ledger {
        for e in ledger.evidence() {
            let id = e.evidence_id.to_string();
            if let Source::Artifact { sha256, .. } = &e.source {
                reads.add(sha256, &id);
            }
            let Some(raw) = e.raw.as_deref().filter(|r| {
                !Path::new(r)
                    .components()
                    .any(|c| !matches!(c, std::path::Component::Normal(_)))
                    && SUMMARY_DIRS.iter().any(|d| r.starts_with(&format!("{d}/")))
            }) else {
                continue;
            };
            if let Some(doc) = fs::read(run_dir.join(raw))
                .ok()
                .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
            {
                for d in captures_of(&doc) {
                    reads.add(&d, &id);
                }
            }
        }
    }
    for dir in SUMMARY_DIRS {
        let Ok(rd) = fs::read_dir(run_dir.join(dir)) else {
            continue;
        };
        for e in rd.flatten() {
            if let Some(doc) = fs::read(e.path())
                .ok()
                .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok())
            {
                for d in captures_of(&doc) {
                    reads.full.entry(d).or_default();
                }
            }
        }
    }
    if let Ok(rd) = fs::read_dir(run_dir.join("summaries")) {
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if let Some(stem) = name.strip_suffix(".json")
                && let Some((_, tail)) = stem.rsplit_once('-')
                && tail.len() == 8
                && tail.bytes().all(|b| b.is_ascii_hexdigit())
            {
                reads.short.insert(tail.to_ascii_lowercase());
            }
        }
    }
    reads
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(h: &str, label: &str, mb: u64, at: &str, read: bool, current: bool) -> SetInfo {
        SetInfo {
            harness: h.to_owned(),
            label: label.to_owned(),
            bytes: mb * 1024 * 1024,
            captured_at: at.to_owned(),
            read,
            current,
        }
    }

    const MB: u64 = 1024 * 1024;

    #[test]
    fn nothing_goes_while_the_total_is_under_the_limit() {
        let sets = [
            set("H1", "p1-fresh", 900, "2026-01-01T00:00:01Z", true, false),
            set("H2", "p1-fresh", 900, "2026-01-01T00:00:02Z", true, false),
        ];
        assert!(choose(&sets, 1800 * MB).is_empty());
        assert!(choose(&sets, 5000 * MB).is_empty());
    }

    #[test]
    fn the_oldest_read_sets_go_first_and_it_stops_as_soon_as_the_total_fits() {
        let sets = [
            set("H3", "p1-fresh", 900, "2026-01-01T00:00:03Z", true, false),
            set("H1", "p1-fresh", 900, "2026-01-01T00:00:01Z", true, false),
            set("H2", "p1-fresh", 900, "2026-01-01T00:00:02Z", true, false),
            set("H4", "p1-fresh", 900, "2026-01-01T00:00:04Z", true, true),
        ];
        // 3600 MB, limit 2048: two sets must go (one leaves 2700), H1 then H2.
        assert_eq!(choose(&sets, 2048 * MB), vec![1, 2]);
        // Limit 2700: only the oldest.
        assert_eq!(choose(&sets, 2700 * MB), vec![1]);
    }

    #[test]
    fn a_set_nothing_read_is_never_chosen_even_when_the_total_stays_over() {
        let sets = [
            set("H1", "p1-fresh", 900, "2026-01-01T00:00:01Z", false, false),
            set("H2", "p1-fresh", 900, "2026-01-01T00:00:02Z", true, false),
            set("H3", "p1-fresh", 900, "2026-01-01T00:00:03Z", false, false),
        ];
        assert_eq!(choose(&sets, 100 * MB), vec![1]);
        let none_read = [
            set("H1", "p1-fresh", 900, "2026-01-01T00:00:01Z", false, false),
            set("H2", "p1-fresh", 900, "2026-01-01T00:00:02Z", false, false),
        ];
        assert!(choose(&none_read, 1).is_empty());
    }

    #[test]
    fn the_current_set_is_never_chosen_even_when_it_is_the_oldest_and_read() {
        let sets = [
            set("H1", "p1-fresh", 900, "2026-01-01T00:00:01Z", true, true),
            set("H2", "p1-fresh", 900, "2026-01-01T00:00:02Z", true, false),
        ];
        assert_eq!(choose(&sets, 1000 * MB), vec![1]);
        assert_eq!(choose(&sets, 1), vec![1]);
    }

    #[test]
    fn ties_in_time_go_by_harness_then_label_and_freeing_room_takes_no_more_than_needed() {
        let sets = [
            set("H2", "p1-fresh", 100, "2026-01-01T00:00:01Z", true, false),
            set("H1", "p2-fresh", 100, "2026-01-01T00:00:01Z", true, false),
            set("H1", "p1-fresh", 100, "2026-01-01T00:00:01Z", true, false),
        ];
        assert_eq!(choose_to_free(&sets, 150 * MB), vec![2, 1]);
        assert_eq!(choose_to_free(&sets, 100 * MB), vec![2]);
        assert!(choose_to_free(&sets, 0).is_empty());
        // More is needed than all deletable sets hold: all of them, no more.
        assert_eq!(choose_to_free(&sets, 5000 * MB).len(), 3);
    }

    #[test]
    fn only_dumps_and_recordings_are_large() {
        let f = |kind, path: &str| CaptureFile {
            kind,
            path: path.to_owned(),
            label: "p1-fresh".to_owned(),
            seq: 1,
            x: None,
            role: None,
            bytes: 1,
            ms: None,
            captured_at: String::new(),
            sha256: None,
            pruned: false,
        };
        assert!(is_large(&f(CaptureKind::Heap, "a/heap-1.hprof")));
        assert!(is_large(&f(CaptureKind::Heap, "a/heap-1.heapsnapshot")));
        assert!(is_large(&f(CaptureKind::Jfr, "a/recording.jfr")));
        assert!(!is_large(&f(CaptureKind::Heap, "a/py-1.retain.json")));
        assert!(!is_large(&f(CaptureKind::Histogram, "a/histogram-1.txt")));
        assert!(!is_large(&f(CaptureKind::Alloc, "a/alloc-1.heapprofile")));
        assert!(!is_large(&f(CaptureKind::NonHeap, "a/threads.json")));
    }

    #[test]
    fn megabytes_come_from_the_environment_or_the_default() {
        assert_eq!(
            megabytes_from_env("DLA_TEST_UNSET_MB_VARIABLE", 2.0),
            2 * MB
        );
    }

    #[test]
    fn digests_named_by_summary_files_and_raw_lists_are_read() {
        let dir = tempfile::tempdir().unwrap();
        let d = "ab".repeat(32);
        let e = "cd".repeat(32);
        fs::create_dir_all(dir.path().join("retain")).unwrap();
        fs::write(
            dir.path().join("retain/R1.json"),
            format!(r#"{{"captures":[{{"path":"x","sha256":"{d}"}}]}}"#),
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("summaries")).unwrap();
        fs::write(
            dir.path()
                .join(format!("summaries/heap-1-{}.json", &e[..8])),
            "{}",
        )
        .unwrap();
        let reads = read_digests(dir.path(), None);
        assert!(reads.read_by(&d).is_some());
        assert!(reads.read_by(&e).is_some());
        assert!(reads.read_by(&"ef".repeat(32)).is_none());
    }
}
