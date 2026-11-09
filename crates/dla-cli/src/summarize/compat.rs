//! Whether two heap dumps can be successive dumps of one process, as far as the files show.

use std::path::{Path, PathBuf};

use dla_parsers::hprof::HeapSummary;

use super::input::Input;

/// Share of the earlier dump's classes the later dump must still have loaded.
const CLASS_OVERLAP: f64 = 0.9;

fn shared(a: &[u64], b: &[u64]) -> usize {
    let (mut i, mut j, mut n) = (0, 0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                n += 1;
                i += 1;
                j += 1;
            }
        }
    }
    n
}

/// Checks that `later` can follow `earlier`: same HPROF version and identifier size, a
/// later header timestamp, and a class set that still contains at least 90% of the
/// earlier dump's classes and of its application (non-boot loader) classes. Two runs of the same program pass these checks too: a heap dump
/// carries no process id, so "same JVM" can be refuted but not proven.
pub fn check(earlier: &HeapSummary, later: &HeapSummary) -> Result<(), String> {
    let (a, b) = (&earlier.header, &later.header);
    if a.version != b.version {
        return Err(format!(
            "HPROF versions differ ({} and {})",
            a.version, b.version
        ));
    }
    if a.id_size != b.id_size {
        return Err(format!(
            "identifier sizes differ ({} and {} bytes): different JVM builds or settings",
            a.id_size, b.id_size
        ));
    }
    if a.timestamp_ms >= b.timestamp_ms {
        return Err(format!(
            "both headers carry the timestamp {}",
            a.timestamp_ms
        ));
    }
    let app_total = earlier.app_class_hashes.len();
    let app_common = shared(&earlier.app_class_hashes, &later.app_class_hashes);
    if app_total > 0 && (app_common as f64) < CLASS_OVERLAP * app_total as f64 {
        return Err(format!(
            "the later dump has only {app_common} of the earlier dump's {app_total} application classes: a different program"
        ));
    }
    if earlier.class_hashes.is_empty() {
        return Err(
            "the earlier dump lists no loaded classes, so it cannot be compared with the later one"
                .to_owned(),
        );
    }
    let common = shared(&earlier.class_hashes, &later.class_hashes);
    let total = earlier.class_hashes.len();
    let share = common as f64 / total as f64;
    if share < CLASS_OVERLAP {
        return Err(format!(
            "the later dump has only {common} of the earlier dump's {total} loaded classes ({:.0}%, at least {:.0}% expected from one process)",
            share * 100.0,
            CLASS_OVERLAP * 100.0
        ));
    }
    Ok(())
}

/// Whether `inputs` are the heap files of one command-mode harness run, one process per
/// point, and at which `x` (the iteration count `n`) each was taken.
///
/// `Ok(None)` means the inputs are not such a set and the one-process rules apply
/// unchanged: fewer than three inputs, an input that `captures.json` of no harness lists
/// as a heap capture (an artifact run, an attach dump), or driver-mode captures (several
/// dumps of one process). `Ok(Some(xs))` gives one `x` per input, in input order, for a
/// set that is all command-mode captures of one harness with distinct points and
/// distinct `x`. `Err` is a set of harness captures that cannot be that series: other
/// harnesses, command mode mixed with driver mode, a point listed twice, a missing or
/// repeated `x`.
pub fn per_point(run_dir: &Path, inputs: &[Input]) -> Result<Option<Vec<f64>>, String> {
    if inputs.len() < 3 {
        return Ok(None);
    }
    struct Cap {
        harness: String,
        label: String,
        x: Option<f64>,
    }
    let canon = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    let mut known: Vec<(PathBuf, Cap)> = Vec::new();
    let Ok(rd) = std::fs::read_dir(run_dir.join("harness")) else {
        return Ok(None);
    };
    for e in rd.flatten() {
        let Ok(text) = std::fs::read_to_string(e.path().join("captures.json")) else {
            continue;
        };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        let harness = e.file_name().to_string_lossy().into_owned();
        for f in v
            .get("files")
            .and_then(|f| f.as_array())
            .into_iter()
            .flatten()
        {
            let (Some(kind), Some(path), Some(label)) = (
                f.get("kind").and_then(|k| k.as_str()),
                f.get("path").and_then(|p| p.as_str()),
                f.get("label").and_then(|l| l.as_str()),
            ) else {
                continue;
            };
            if kind != "heap" {
                continue;
            }
            known.push((
                canon(&run_dir.join(path)),
                Cap {
                    harness: harness.clone(),
                    label: label.to_owned(),
                    x: f.get("x").and_then(|x| x.as_f64()),
                },
            ));
        }
    }
    let mut caps: Vec<&Cap> = Vec::new();
    for i in inputs {
        let abs = canon(&i.abs);
        match known.iter().find(|(p, _)| *p == abs) {
            Some((_, c)) => caps.push(c),
            None => return Ok(None),
        }
    }
    let name = |i: usize| inputs[i].rel.rsplit('/').next().unwrap_or(&inputs[i].rel);
    let command = |c: &Cap| c.label.ends_with("-native");
    if !caps.iter().any(|c| command(c)) {
        // Driver mode: successive dumps of one process, today's rules.
        return Ok(None);
    }
    if let Some(c) = caps.iter().find(|c| c.harness != caps[0].harness) {
        return Err(format!(
            "the dumps come from different harnesses ({} and {})",
            caps[0].harness, c.harness
        ));
    }
    if let Some(i) = caps.iter().position(|c| !command(c)) {
        return Err(format!(
            "{} was taken in driver mode and the others in command mode (one process per point): the two modes do not form one series",
            name(i)
        ));
    }
    for (i, c) in caps.iter().enumerate() {
        if let Some(j) = (0..i).find(|&j| caps[j].label == c.label) {
            return Err(format!(
                "{} and {} are two dumps of the same point {}: a per-point series has one dump per point",
                name(j),
                name(i),
                c.label
            ));
        }
    }
    let mut xs = Vec::with_capacity(caps.len());
    for (i, c) in caps.iter().enumerate() {
        let Some(x) = c.x.filter(|x| x.is_finite()) else {
            return Err(format!(
                "{} has no recorded n in captures.json (point {})",
                name(i),
                c.label
            ));
        };
        if let Some(j) = xs.iter().position(|&y| y == x) {
            return Err(format!(
                "{} and {} were taken at the same n ({x}): a series needs distinct n",
                name(j),
                name(i)
            ));
        }
        xs.push(x);
    }
    Ok(Some(xs))
}

/// The format checks that hold for dumps of separate processes: the same HPROF version
/// and identifier size.
pub fn check_format(a: &HeapSummary, b: &HeapSummary) -> Result<(), String> {
    let (a, b) = (&a.header, &b.header);
    if a.version != b.version {
        return Err(format!(
            "HPROF versions differ ({} and {})",
            a.version, b.version
        ));
    }
    if a.id_size != b.id_size {
        return Err(format!(
            "identifier sizes differ ({} and {} bytes): different JVM builds or settings",
            a.id_size, b.id_size
        ));
    }
    Ok(())
}

/// Renders the `n` values of a per-point series.
pub fn n_list(xs: &[f64]) -> String {
    xs.iter()
        .map(|x| format!("{x}"))
        .collect::<Vec<_>>()
        .join(", ")
}
