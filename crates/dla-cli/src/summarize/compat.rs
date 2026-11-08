//! Whether two heap dumps can be successive dumps of one process, as far as the files show.

use dla_parsers::hprof::HeapSummary;

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
    let common = shared(&earlier.class_hashes, &later.class_hashes);
    let total = earlier.class_hashes.len().max(1);
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
