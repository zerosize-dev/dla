//! The size limit of the large captures of a run, and the free-disk check before a capture.
//!
//! One Node harness taking nine V8 snapshots of about 110 MB keeps 1 GB, and a long run makes
//! tens of harnesses. A full disk breaks every later write of the run (ledger, series,
//! captures), and the errors read like findings about the project. So:
//!
//! 1. After a capturing run writes its `captures.json`, [`enforce`] adds up the bytes of the
//!    large files (heap dumps and flight recordings, see [`dla_core::capture_keep::is_large`])
//!    that the `captures.json` of every harness of the run directory lists. Over
//!    `DLA_CAPTURE_KEEP_MB` (default 2048) it deletes whole capture sets of other harnesses,
//!    oldest first. A set is the large files of one harness and one label. The set the current
//!    run wrote, and a set that no summary read, are never deleted. The deleted files stay
//!    listed with `pruned: true` and their digest.
//! 2. Before a capturing run starts any process, [`preflight`] estimates what it will write
//!    (dumps per point and key mode times the largest dump seen before), deletes what rule 1 may
//!    delete if the disk would be left under `DLA_DISK_FLOOR_MB` (default 1024), and refuses when
//!    that does not suffice.
//!
//! Workers share the run directory and the numbering of harnesses (`harness/H<n>`), and they
//! share the disk, so the sets of every harness of the run directory are counted, whichever
//! worker made them. The summaries that read a set are those the run's ledger holds (the core's
//! and the calling worker's slice) and the raw and summary files in the run directory, which
//! also show a summary of another worker whose slice is not merged yet.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use dla_core::{
    capture_keep::{self, Reads, SetInfo},
    error::Refusal,
    fsutil::write_atomic,
    harness::{CaptureFile, CaptureKind, HARNESS_DIR, HarnessManifest, HarnessMode},
    toolctx::ToolCtx,
};
use serde_json::{Value, json};

use super::{capture, run::RunOptions};

/// The refusal code of a capture that would leave too little free disk (the code `dla attach`
/// uses for the same condition).
pub const DISK_LOW: &str = "insufficient-resources";
/// Fewest heap dumps a series of dumps has (the gate needs three snapshots).
pub const MIN_DUMPS: u64 = 3;

/// One deleted capture set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pruned {
    /// The harness the set belongs to.
    pub harness: String,
    /// The label of the point.
    pub label: String,
    /// Files deleted.
    pub files: usize,
    /// Bytes freed.
    pub bytes: u64,
    /// The evidence records of the summaries that read it (may be empty when only a summary file
    /// names it).
    pub read_by: Vec<String>,
}

impl Pruned {
    /// `{harness, label, files, bytes}`.
    pub fn to_json(&self) -> Value {
        json!({
            "harness": self.harness,
            "label": self.label,
            "files": self.files,
            "bytes": self.bytes,
        })
    }
}

/// A capture set that no summary read, so that it may not be deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unread {
    /// The harness the set belongs to.
    pub harness: String,
    /// The label of the point.
    pub label: String,
    /// Bytes of its large files.
    pub bytes: u64,
    /// When its newest large file was written.
    pub captured_at: String,
    /// Run-relative paths of its large files.
    pub paths: Vec<String>,
}

/// What [`enforce`] did.
#[derive(Debug, Default, Clone)]
pub struct Outcome {
    /// Sets deleted, in the order of deletion.
    pub pruned: Vec<Pruned>,
    /// Sets of other harnesses that nothing read, oldest first. Filled only when the total was
    /// over the limit or room was asked for.
    pub unread: Vec<Unread>,
    /// Bytes of large files on disk before.
    pub total_before: u64,
}

impl Outcome {
    /// Bytes freed.
    pub fn freed(&self) -> u64 {
        self.pruned.iter().map(|p| p.bytes).sum()
    }
}

/// The `captures.json` of one harness, as read.
struct Listed {
    harness: String,
    path: PathBuf,
    doc: Value,
    files: Vec<CaptureFile>,
    dirty: bool,
}

fn load_lists(run_dir: &Path) -> Vec<Listed> {
    let Ok(rd) = fs::read_dir(run_dir.join(HARNESS_DIR)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for e in rd.flatten() {
        let path = e.path().join("captures.json");
        let Some(doc) = fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
        else {
            continue;
        };
        let Some(files) = doc
            .get("files")
            .and_then(|f| serde_json::from_value::<Vec<CaptureFile>>(f.clone()).ok())
        else {
            continue;
        };
        out.push(Listed {
            harness: e.file_name().to_string_lossy().into_owned(),
            path,
            doc,
            files,
            dirty: false,
        });
    }
    out.sort_by(|a, b| a.harness.cmp(&b.harness));
    out
}

/// The large files that are on disk, grouped by harness and label: `(list, label) -> file indexes`.
fn groups(run_dir: &Path, lists: &[Listed]) -> BTreeMap<(usize, String), Vec<usize>> {
    let mut g: BTreeMap<(usize, String), Vec<usize>> = BTreeMap::new();
    for (li, l) in lists.iter().enumerate() {
        for (fi, f) in l.files.iter().enumerate() {
            if capture_keep::is_large(f) && !f.pruned && run_dir.join(&f.path).is_file() {
                g.entry((li, f.label.clone())).or_default().push(fi);
            }
        }
    }
    g
}

fn size_on_disk(run_dir: &Path, f: &CaptureFile) -> u64 {
    fs::metadata(run_dir.join(&f.path)).map_or(0, |m| m.len())
}

/// Deletes capture sets so that the large files of the run total at most `limit` bytes and at
/// least `room` bytes are freed besides (when sets that may go exist): the rules are in the
/// module text. Nothing is read or hashed when the total is under the limit and no room is
/// asked for. `current` is the harness being run: none of its sets is deleted.
pub fn enforce(ctx: &ToolCtx, current: &str, limit: u64, room: u64) -> Outcome {
    let run_dir = ctx.run_dir.as_path();
    let mut lists = load_lists(run_dir);
    let g = groups(run_dir, &lists);
    let bytes_of = |idx: &[usize], li: usize, lists: &[Listed]| -> u64 {
        idx.iter()
            .map(|&fi| size_on_disk(run_dir, &lists[li].files[fi]))
            .sum()
    };
    let total: u64 = g
        .iter()
        .map(|((li, _), idx)| bytes_of(idx, *li, &lists))
        .sum();
    let need = total.saturating_sub(limit).max(room);
    let mut outcome = Outcome {
        total_before: total,
        ..Outcome::default()
    };
    if need == 0 {
        return outcome;
    }
    let ledger = ctx.ledger().ok();
    let reads: Reads = capture_keep::read_digests(run_dir, ledger.as_ref());
    drop(ledger);
    let keys: Vec<&(usize, String)> = g.keys().collect();
    let mut infos = Vec::new();
    let mut read_by: Vec<Vec<String>> = Vec::new();
    for key in &keys {
        let (li, label) = (key.0, &key.1);
        let idx = &g[*key];
        let current_set = lists[li].harness == current;
        // A set is read when a summary read one of its files: the model picks the dumps it
        // summarizes (four of nine, say), and the others are no work it still owes.
        let mut any_read = false;
        let mut ids: Vec<String> = Vec::new();
        if !current_set {
            for &fi in idx {
                if lists[li].files[fi].sha256.is_none() {
                    let path = run_dir.join(&lists[li].files[fi].path);
                    lists[li].files[fi].sha256 = capture_keep::sha256_file(&path);
                    lists[li].dirty = true;
                }
                let digest = lists[li].files[fi].sha256.clone();
                if let Some(by) = digest.as_deref().and_then(|d| reads.read_by(d)) {
                    any_read = true;
                    ids.extend(by);
                }
            }
        }
        ids.sort();
        ids.dedup();
        let at = idx
            .iter()
            .map(|&fi| lists[li].files[fi].captured_at.clone())
            .max()
            .unwrap_or_default();
        infos.push(SetInfo {
            harness: lists[li].harness.clone(),
            label: label.clone(),
            bytes: bytes_of(idx, li, &lists),
            captured_at: at,
            read: any_read && !current_set,
            current: current_set,
        });
        read_by.push(ids);
    }
    let chosen = capture_keep::choose_to_free(&infos, need);
    for &c in &chosen {
        let (li, label) = keys[c];
        let (li, label) = (*li, label.clone());
        let mut deleted = 0usize;
        for &fi in &g[&(li, label.clone())] {
            let path = run_dir.join(&lists[li].files[fi].path);
            if fs::remove_file(&path).is_ok() {
                deleted += 1;
                let f = &mut lists[li].files[fi];
                f.pruned = true;
                lists[li].dirty = true;
                if let Some(dir) = path.parent() {
                    let _ = fs::remove_dir(dir);
                }
            }
        }
        outcome.pruned.push(Pruned {
            harness: lists[li].harness.clone(),
            label,
            files: deleted,
            bytes: infos[c].bytes,
            read_by: read_by[c].clone(),
        });
    }
    outcome.unread = infos
        .iter()
        .enumerate()
        .filter(|(i, s)| !s.read && !s.current && s.bytes > 0 && !chosen.contains(i))
        .map(|(i, s)| {
            let (li, label) = keys[i];
            Unread {
                harness: s.harness.clone(),
                label: s.label.clone(),
                bytes: s.bytes,
                captured_at: s.captured_at.clone(),
                paths: g[&(*li, label.clone())]
                    .iter()
                    .map(|&fi| lists[*li].files[fi].path.clone())
                    .collect(),
            }
        })
        .collect();
    outcome
        .unread
        .sort_by(|a, b| (&a.captured_at, &a.harness).cmp(&(&b.captured_at, &b.harness)));
    for l in lists.iter_mut().filter(|l| l.dirty) {
        save(run_dir, l);
    }
    outcome
}

/// Writes the list back (digests filled in, files marked deleted) and does the same in the
/// manifest of the harness, which lists the same files.
fn save(run_dir: &Path, l: &mut Listed) {
    if let Ok(files) = serde_json::to_value(&l.files) {
        l.doc["files"] = files;
        let text = serde_json::to_string_pretty(&l.doc).unwrap_or_default() + "\n";
        let _ = write_atomic(&l.path, text.as_bytes());
    }
    if let Ok(mut m) = HarnessManifest::load(run_dir, &l.harness) {
        let mut changed = false;
        for f in &mut m.captures {
            if let Some(n) = l.files.iter().find(|n| n.path == f.path)
                && (f.pruned != n.pruned || f.sha256 != n.sha256)
            {
                f.pruned = n.pruned;
                f.sha256.clone_from(&n.sha256);
                changed = true;
            }
        }
        if changed {
            let _ = m.save(run_dir);
        }
    }
}

/// Free bytes of the volume of `path`: `DLA_FREE_DISK_BYTES` when set (tests, and a box whose
/// figure is wrong), else the measured figure; `None` when it cannot be read.
pub fn free_disk(path: &Path) -> Option<u64> {
    std::env::var(capture_keep::FREE_DISK_ENV)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .or_else(|| dla_sample::attach::free_disk(path))
}

/// What a capturing run is about to write.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Estimate {
    /// Heap dumps one process takes (driver) or one key mode takes in all (command mode).
    pub dumps: u64,
    /// Key modes run.
    pub modes: u64,
    /// The largest heap dump of the harness's earlier captures, else of any harness.
    pub largest: u64,
}

impl Estimate {
    /// Bytes the run writes.
    pub fn bytes(&self) -> u64 {
        self.dumps
            .saturating_mul(self.modes)
            .saturating_mul(self.largest)
    }
}

/// Bytes of the largest heap dump the lists name, among those of `harness` first.
fn largest_dump(lists: &[Listed], harness: &str) -> Option<u64> {
    let max = |only: Option<&str>| {
        lists
            .iter()
            .filter(|l| only.is_none_or(|h| l.harness == h))
            .flat_map(|l| &l.files)
            .filter(|f| f.kind == CaptureKind::Heap && capture_keep::is_large(f))
            .map(|f| f.bytes)
            .max()
    };
    max(Some(harness)).or_else(|| max(None))
}

/// Heap dumps and key modes of a run of `manifest` with `opts`: `None` when the run takes no
/// heap dump.
fn dumps_and_modes(manifest: &HarnessManifest, opts: &RunOptions) -> Option<(u64, u64)> {
    if !opts.capture.heap {
        return None;
    }
    let want = opts.capture_dumps.unwrap_or(capture::DEFAULT_DUMPS);
    if manifest.mode == HarnessMode::Command {
        let total = manifest.warmup + opts.iterations.unwrap_or(manifest.iterations) + opts.extra;
        let n = capture::select_points(total, manifest.warmup, want).len() as u64;
        return Some((n, 1));
    }
    let modes = if opts.modes.is_empty() {
        manifest.key_modes.len()
    } else {
        opts.modes.len()
    };
    Some((u64::from(want), if modes == 0 { 2 } else { modes as u64 }))
}

/// The estimate of what the run writes; `None` when it takes no heap dump or no earlier dump
/// of any harness tells its size.
pub fn estimate(ctx: &ToolCtx, manifest: &HarnessManifest, opts: &RunOptions) -> Option<Estimate> {
    let (dumps, modes) = dumps_and_modes(manifest, opts)?;
    let largest = largest_dump(&load_lists(&ctx.run_dir), &manifest.id)?;
    Some(Estimate {
        dumps,
        modes,
        largest,
    })
}

/// `MB` text of a byte count (`980 MB`, `1.5 GB`).
pub fn human(bytes: u64) -> String {
    let mb = bytes as f64 / (1024.0 * 1024.0);
    if mb >= 1024.0 {
        format!("{:.1} GB", mb / 1024.0)
    } else if mb >= 10.0 {
        format!("{mb:.0} MB")
    } else {
        format!("{mb:.1} MB")
    }
}

/// The figure of a megabyte limit as the variable spells it (`2048`, `0.5`).
pub fn mb_text(bytes: u64) -> String {
    let mb = bytes as f64 / (1024.0 * 1024.0);
    if (mb - mb.round()).abs() < 1e-9 {
        format!("{}", mb.round() as u64)
    } else {
        format!("{mb:.2}")
    }
}

/// The summarize command of an unread set: the printed command of its harness's
/// `captures.json` that names one of its files, else the command built from the files.
fn summarize_command(ctx: &ToolCtx, set: &Unread) -> Option<String> {
    let lists = load_lists(&ctx.run_dir);
    let l = lists.iter().find(|l| l.harness == set.harness)?;
    let named = l
        .doc
        .get("summarize")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .find(|c| set.paths.iter().any(|p| c.contains(p.as_str())));
    if let Some(c) = named {
        return Some(c.to_owned());
    }
    let files: Vec<CaptureFile> = l
        .files
        .iter()
        .filter(|f| set.paths.contains(&f.path))
        .cloned()
        .collect();
    capture::summarize_commands(&files, &|p| ctx.pointer(p), false)
        .into_iter()
        .next()
}

/// The check before a capturing run starts any process. Deletes what the size limit allows when
/// the disk would be left under the floor, then refuses when it still would be. The sets deleted
/// are returned for the run's output. Does nothing when the run takes no heap dump or the free
/// disk cannot be read.
pub fn preflight(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    opts: &RunOptions,
) -> Result<Vec<Pruned>, Refusal> {
    if opts.tool.is_some() || dumps_and_modes(manifest, opts).is_none() {
        return Ok(Vec::new());
    }
    let Some(free) = free_disk(&ctx.run_dir) else {
        return Ok(Vec::new());
    };
    let floor =
        capture_keep::megabytes_from_env(capture_keep::FLOOR_ENV, capture_keep::DEFAULT_FLOOR_MB);
    let limit =
        capture_keep::megabytes_from_env(capture_keep::KEEP_ENV, capture_keep::DEFAULT_KEEP_MB);
    let est = estimate(ctx, manifest, opts);
    let wanted = est.map_or(0, |e| e.bytes()).saturating_add(floor);
    let outcome = enforce(ctx, &manifest.id, limit, wanted.saturating_sub(free));
    let available = free.saturating_add(outcome.freed());
    if available >= wanted {
        return Ok(outcome.pruned);
    }
    Err(refusal(
        ctx, manifest, opts, est, available, floor, &outcome,
    ))
}

fn refusal(
    ctx: &ToolCtx,
    manifest: &HarnessManifest,
    opts: &RunOptions,
    est: Option<Estimate>,
    free: u64,
    floor: u64,
    outcome: &Outcome,
) -> Refusal {
    let writes = est.map_or(0, |e| e.bytes());
    let mut message = match est {
        Some(e) => format!(
            "free disk is {} and this capture would write about {} ({} heap dump(s) x {} key mode(s) x {}, the largest dump kept before), which leaves {} under the {} floor (DLA_DISK_FLOOR_MB={})",
            human(free),
            human(writes),
            e.dumps,
            e.modes,
            human(e.largest),
            human(free.saturating_sub(writes)),
            human(floor),
            mb_text(floor),
        ),
        None => format!(
            "free disk is {}, under the {} floor (DLA_DISK_FLOOR_MB={}), and no earlier heap dump of this run tells how much this capture writes",
            human(free),
            human(floor),
            mb_text(floor),
        ),
    };
    if !outcome.unread.is_empty() {
        let shown: Vec<String> = outcome
            .unread
            .iter()
            .take(3)
            .map(|u| format!("{} {} ({})", u.harness, u.label, human(u.bytes)))
            .collect();
        let more = outcome.unread.len().saturating_sub(3);
        message.push_str(&format!(
            "; the heap dumps of {}{} could not be deleted because no summary read them",
            shown.join(", "),
            if more > 0 {
                format!(" and {more} more")
            } else {
                String::new()
            }
        ));
    }
    let mut fix = String::new();
    if let Some(oldest) = outcome.unread.first() {
        match summarize_command(ctx, oldest) {
            Some(c) => fix.push_str(&format!(
                "read the oldest unread set so that it may be deleted: {c}; "
            )),
            None => fix.push_str(&format!(
                "summarize the heap dumps of {} {} (dla summarize heap-graph <its files in harness/{}/captures.json>) so that they may be deleted; ",
                oldest.harness, oldest.label, oldest.harness
            )),
        }
    }
    let fits = est.and_then(|e| {
        let per_dump = e.modes.saturating_mul(e.largest);
        (per_dump > 0).then(|| free.saturating_sub(floor) / per_dump)
    });
    let kinds = opts.capture.names().join(",");
    let mode = if opts.modes.is_empty() {
        String::new()
    } else {
        format!(
            " --mode {}",
            opts.modes
                .iter()
                .map(|m| m.as_str())
                .collect::<Vec<_>>()
                .join(",")
        )
    };
    match fits {
        Some(n) if n >= MIN_DUMPS => fix.push_str(&format!(
            "then dla harness run {}{mode} --capture {kinds} --dumps {n}",
            manifest.id
        )),
        _ => fix.push_str(&format!(
            "even {MIN_DUMPS} dumps do not fit: free disk (delete files outside the run, or give the run a larger volume), then dla harness run {}{mode} --capture {kinds} --dumps {MIN_DUMPS}",
            manifest.id
        )),
    }
    Refusal::new(
        DISK_LOW,
        message,
        format!("{} of free disk after the capture", human(floor)),
        fix,
    )
}

/// The `_attention` sentence for sets deleted in this run; empty when none was.
pub fn attention(pruned: &[Pruned], limit: u64) -> Option<String> {
    if pruned.is_empty() {
        return None;
    }
    let one = |p: &Pruned| {
        let by = if p.read_by.is_empty() {
            String::new()
        } else {
            format!(", read by {}", p.read_by.join(", "))
        };
        format!("{} {} ({}){by}", p.harness, p.label, human(p.bytes))
    };
    let shown: Vec<String> = pruned.iter().take(3).map(one).collect();
    let more = pruned.len().saturating_sub(3);
    Some(format!(
        "deleted the heap dumps of {}{}, to stay under DLA_CAPTURE_KEEP_MB={}",
        shown.join("; "),
        if more > 0 {
            format!("; and {more} more set(s)")
        } else {
            String::new()
        },
        mb_text(limit)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_and_limits_are_spelled_short() {
        assert_eq!(human(980 * 1024 * 1024), "980 MB");
        assert_eq!(human(3 * 1024 * 1024 / 2), "1.5 MB");
        assert_eq!(human(3 * 1024 * 1024 * 1024 / 2), "1.5 GB");
        assert_eq!(mb_text(2048 * 1024 * 1024), "2048");
        assert_eq!(mb_text(1024 * 1024 / 2), "0.50");
    }

    #[test]
    fn the_attention_line_names_the_sets_their_readers_and_the_limit() {
        let p = |h: &str, by: &[&str]| Pruned {
            harness: h.to_owned(),
            label: "p1-fresh".to_owned(),
            files: 9,
            bytes: 980 * 1024 * 1024,
            read_by: by.iter().map(|s| (*s).to_owned()).collect(),
        };
        assert_eq!(attention(&[], 2048 * 1024 * 1024), None);
        assert_eq!(
            attention(&[p("H2", &["E41"])], 2048 * 1024 * 1024).unwrap(),
            "deleted the heap dumps of H2 p1-fresh (980 MB), read by E41, to stay under DLA_CAPTURE_KEEP_MB=2048"
        );
        let many = attention(
            &[p("H1", &[]), p("H2", &[]), p("H3", &[]), p("H4", &[])],
            2048 * 1024 * 1024,
        )
        .unwrap();
        assert!(many.contains("and 1 more set(s)"), "{many}");
    }
}
