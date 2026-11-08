//! Free disk, available memory and the arithmetic of the attach preflight.
//!
//! Everything that parses text is a pure function, so the numbers can be tested without a
//! container. The readers at the bottom feed them from `/proc`, `/sys/fs/cgroup` and
//! `statvfs`.

use std::path::Path;

use serde::Serialize;

/// Dump size and peak extra memory an attach is expected to need.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Estimate {
    /// Bytes the artifact files are expected to take.
    pub dump_bytes: u64,
    /// Extra memory the target (or the tool) may need at the peak, in bytes.
    pub peak_bytes: u64,
    /// Where the two numbers come from.
    pub basis: String,
}

/// Memory figures, in bytes. `None` is "could not be read" or "no limit".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct MemoryAvailable {
    /// `MemAvailable` of the host.
    pub host: Option<u64>,
    /// Headroom under the cgroup limit (limit - usage + inactive file cache).
    pub cgroup: Option<u64>,
}

impl MemoryAvailable {
    /// The smaller of the figures that are known.
    pub fn effective(&self) -> Option<u64> {
        match (self.host, self.cgroup) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }
}

/// Why the preflight refuses, one entry per failed rule.
pub fn gate(est: &Estimate, free_disk: Option<u64>, memory: Option<u64>) -> Vec<String> {
    let mut reasons = Vec::new();
    if let Some(disk) = free_disk
        && dump_exceeds_disk(est.dump_bytes, disk)
    {
        reasons.push(format!(
            "the expected dump of {} (x1.2 = {}) does not fit the {} of free disk",
            human(est.dump_bytes),
            human(est.dump_bytes.saturating_mul(12) / 10),
            human(disk)
        ));
    }
    if let Some(mem) = memory
        && peak_exceeds_memory(est.peak_bytes, mem)
    {
        reasons.push(format!(
            "the expected peak extra memory of {} is over 80% of the {} available",
            human(est.peak_bytes),
            human(mem)
        ));
    }
    reasons
}

/// `dump * 1.2 > free`, in integers.
pub fn dump_exceeds_disk(dump: u64, free: u64) -> bool {
    u128::from(dump) * 12 > u128::from(free) * 10
}

/// `peak > 0.8 * available`, in integers.
pub fn peak_exceeds_memory(peak: u64, available: u64) -> bool {
    u128::from(peak) * 5 > u128::from(available) * 4
}

/// Bytes as a short human figure (`1.5 GiB`).
pub fn human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut v = bytes as f64;
    let mut i = 0;
    while v >= 1024.0 && i + 1 < UNITS.len() {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{bytes} B")
    } else {
        format!("{v:.1} {}", UNITS[i])
    }
}

/// Limits at or above this are "no limit" (cgroup v1 reports `9223372036854771712`).
const UNLIMITED: u64 = 1 << 60;

/// `MemAvailable` of `/proc/meminfo`, in bytes.
pub fn parse_meminfo_available(text: &str) -> Option<u64> {
    let line = text.lines().find(|l| l.starts_with("MemAvailable:"))?;
    let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
    kb.checked_mul(1024)
}

/// The value of `key` in a `memory.stat` file.
pub fn stat_value(stat: &str, key: &str) -> Option<u64> {
    stat.lines().find_map(|l| {
        let (k, v) = l.split_once(' ')?;
        (k == key).then(|| v.trim().parse().ok())?
    })
}

fn parse_limit(text: &str) -> Option<u64> {
    let t = text.trim();
    if t == "max" {
        return None;
    }
    t.parse().ok().filter(|v| *v < UNLIMITED)
}

/// cgroup v2 headroom: `memory.max - memory.current + inactive_file`. `None` when the group
/// has no limit.
pub fn cgroup_v2_available(max: &str, current: &str, stat: &str) -> Option<u64> {
    let limit = parse_limit(max)?;
    let used: u64 = current.trim().parse().ok()?;
    let cache = stat_value(stat, "inactive_file").unwrap_or(0);
    Some(limit.saturating_sub(used).saturating_add(cache))
}

/// cgroup v1 headroom: `limit_in_bytes - usage_in_bytes + total_inactive_file`.
pub fn cgroup_v1_available(limit: &str, usage: &str, stat: &str) -> Option<u64> {
    let limit = parse_limit(limit)?;
    let used: u64 = usage.trim().parse().ok()?;
    let cache = stat_value(stat, "total_inactive_file")
        .or_else(|| stat_value(stat, "inactive_file"))
        .unwrap_or(0);
    Some(limit.saturating_sub(used).saturating_add(cache))
}

/// The cgroup path of a controller in `/proc/self/cgroup` (`""` for v2 when `controller` is
/// empty).
pub fn cgroup_path(proc_cgroup: &str, controller: &str) -> Option<String> {
    proc_cgroup.lines().find_map(|l| {
        let mut p = l.splitn(3, ':');
        let _id = p.next()?;
        let ctrls = p.next()?;
        let path = p.next()?;
        let hit = if controller.is_empty() {
            ctrls.is_empty()
        } else {
            ctrls.split(',').any(|c| c == controller)
        };
        hit.then(|| path.to_owned())
    })
}

/// Reads the host and cgroup memory figures of this machine. The cgroup is the one of process
/// `pid` (the attach target, whose memory the extra use lands in), or DLA's own when `pid` is
/// `None` or unreadable.
pub fn read_memory_available(pid: Option<u32>) -> MemoryAvailable {
    let host = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|t| parse_meminfo_available(&t));
    let cgroup_text = pid
        .and_then(|p| std::fs::read_to_string(format!("/proc/{p}/cgroup")).ok())
        .or_else(|| std::fs::read_to_string("/proc/self/cgroup").ok())
        .unwrap_or_default();
    MemoryAvailable {
        host,
        cgroup: cgroup_available_at(Path::new("/sys/fs/cgroup"), &cgroup_text),
    }
}

/// The smallest headroom over the cgroup of `proc_cgroup` (the text of a `/proc/<pid>/cgroup`)
/// and all its ancestors below the mount `root`: a limit on a parent slice binds the child.
/// `None` when no level has a limit.
pub fn cgroup_available_at(root: &Path, proc_cgroup: &str) -> Option<u64> {
    let read = |p: &Path| std::fs::read_to_string(p).ok();
    // v2: the unified hierarchy.
    if root.join("cgroup.controllers").is_file() {
        let sub = cgroup_path(proc_cgroup, "").unwrap_or_default();
        return min_headroom(root, sub.trim_matches('/'), |d| {
            let stat = read(&d.join("memory.stat")).unwrap_or_default();
            cgroup_v2_available(
                &read(&d.join("memory.max"))?,
                &read(&d.join("memory.current"))?,
                &stat,
            )
        });
    }
    // v1: the memory controller. The nested path of a container is often not mounted, so
    // the mount root is the fallback.
    let v1 = root.join("memory");
    let sub = cgroup_path(proc_cgroup, "memory").unwrap_or_default();
    min_headroom(&v1, sub.trim_matches('/'), |d| {
        let stat = read(&d.join("memory.stat")).unwrap_or_default();
        cgroup_v1_available(
            &read(&d.join("memory.limit_in_bytes"))?,
            &read(&d.join("memory.usage_in_bytes"))?,
            &stat,
        )
    })
}

/// Applies `figure` to the directory `root/sub` and each parent up to `root`; the minimum of
/// the figures that exist.
fn min_headroom(root: &Path, sub: &str, figure: impl Fn(&Path) -> Option<u64>) -> Option<u64> {
    let mut dir = root.join(sub);
    // A nested path that is not mounted here: start from the mount root.
    if !sub.is_empty() && !dir.is_dir() {
        dir = root.to_path_buf();
    }
    let mut best: Option<u64> = None;
    loop {
        if let Some(v) = figure(&dir) {
            best = Some(best.map_or(v, |b| b.min(v)));
        }
        if dir == root || !dir.pop() || !dir.starts_with(root) {
            return best;
        }
    }
}

/// Free bytes for unprivileged writers (`f_bavail * f_frsize`) of the filesystem holding
/// `path`; the nearest existing ancestor is asked when `path` does not exist yet. `None` when
/// it cannot be read.
pub fn free_disk(path: &Path) -> Option<u64> {
    let mut p = path;
    while !p.exists() {
        p = p.parent()?;
    }
    disk::free_bytes(p)
}

#[cfg(unix)]
mod disk {
    #![allow(unsafe_code)]

    use std::{ffi::CString, os::unix::ffi::OsStrExt, path::Path};

    pub fn free_bytes(path: &Path) -> Option<u64> {
        let c = CString::new(path.as_os_str().as_bytes()).ok()?;
        let mut st = std::mem::MaybeUninit::<libc::statvfs>::zeroed();
        // SAFETY: `c` is a valid NUL-terminated path and `st` points to writable storage of
        // the right type; statvfs(3) only writes into it.
        let rc = unsafe { libc::statvfs(c.as_ptr(), st.as_mut_ptr()) };
        if rc != 0 {
            return None;
        }
        // SAFETY: statvfs returned 0, so the structure is initialised.
        let st = unsafe { st.assume_init() };
        #[allow(clippy::unnecessary_cast)]
        Some((st.f_bavail as u64).saturating_mul(st.f_frsize as u64))
    }
}

#[cfg(not(unix))]
mod disk {
    use std::path::Path;

    pub fn free_bytes(_path: &Path) -> Option<u64> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn est(dump: u64, peak: u64) -> Estimate {
        Estimate {
            dump_bytes: dump,
            peak_bytes: peak,
            basis: "test".into(),
        }
    }

    #[test]
    fn dump_rule_is_one_point_two_times_free_disk() {
        // 100 * 1.2 = 120: fits in exactly 120, not in 119.
        assert!(!dump_exceeds_disk(100, 120));
        assert!(dump_exceeds_disk(100, 119));
        assert!(!dump_exceeds_disk(0, 0));
    }

    #[test]
    fn peak_rule_is_eighty_percent_of_available_memory() {
        assert!(!peak_exceeds_memory(80, 100));
        assert!(peak_exceeds_memory(81, 100));
        assert!(
            peak_exceeds_memory(u64::MAX, u64::MAX),
            "peak 100% of available is over 80%"
        );
    }

    #[test]
    fn gate_names_each_failed_rule_and_tolerates_unknown_figures() {
        assert!(gate(&est(100, 80), Some(120), Some(100)).is_empty());
        let r = gate(&est(100, 81), Some(119), Some(100));
        assert_eq!(r.len(), 2);
        assert!(r[0].contains("free disk"));
        assert!(r[1].contains("80%"));
        assert!(gate(&est(u64::MAX, u64::MAX), None, None).is_empty());
    }

    #[test]
    fn meminfo_is_read_in_bytes() {
        let t = "MemTotal: 16000000 kB\nMemFree: 1 kB\nMemAvailable:   8192 kB\n";
        assert_eq!(parse_meminfo_available(t), Some(8192 * 1024));
        assert_eq!(parse_meminfo_available("MemTotal: 1 kB\n"), None);
    }

    #[test]
    fn cgroup_v1_headroom_adds_the_inactive_cache() {
        let stat = "cache 5\ntotal_inactive_file 300\n";
        assert_eq!(cgroup_v1_available("1000\n", "900\n", stat), Some(400));
        // Unlimited container: no figure.
        assert_eq!(
            cgroup_v1_available("9223372036854771712\n", "900\n", stat),
            None
        );
        // Usage above the limit never underflows.
        assert_eq!(cgroup_v1_available("100", "500", ""), Some(0));
    }

    #[test]
    fn cgroup_v2_headroom_reads_max_and_current() {
        let stat = "anon 1\ninactive_file 50\n";
        assert_eq!(cgroup_v2_available("1000\n", "400\n", stat), Some(650));
        assert_eq!(cgroup_v2_available("max\n", "400\n", stat), None);
    }

    fn write(root: &Path, rel: &str, text: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    #[test]
    fn a_limit_on_an_ancestor_cgroup_binds_the_target() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        write(r, "cgroup.controllers", "memory\n");
        write(r, "work.slice/memory.max", "1000\n");
        write(r, "work.slice/memory.current", "100\n");
        write(r, "work.slice/app/memory.max", "max\n");
        write(r, "work.slice/app/memory.current", "50\n");
        write(r, "other.slice/memory.max", "5000\n");
        write(r, "other.slice/memory.current", "0\n");
        // The target sits under work.slice/app (no limit of its own): the slice limit applies.
        assert_eq!(cgroup_available_at(r, "0::/work.slice/app\n"), Some(900));
        // Another process in another slice reads that slice's figure.
        assert_eq!(cgroup_available_at(r, "0::/other.slice\n"), Some(5000));
        // A nearer, smaller limit wins.
        write(r, "work.slice/app/memory.max", "300\n");
        assert_eq!(cgroup_available_at(r, "0::/work.slice/app\n"), Some(250));
        // Nothing limited anywhere.
        assert_eq!(cgroup_available_at(r, "0::/missing\n"), None);
    }

    #[test]
    fn cgroup_v1_walks_up_from_the_target_group() {
        let d = tempfile::tempdir().unwrap();
        let r = d.path();
        write(r, "memory/svc/memory.limit_in_bytes", "2000\n");
        write(r, "memory/svc/memory.usage_in_bytes", "500\n");
        write(
            r,
            "memory/svc/app/memory.limit_in_bytes",
            "9223372036854771712\n",
        );
        write(r, "memory/svc/app/memory.usage_in_bytes", "10\n");
        assert_eq!(cgroup_available_at(r, "4:memory:/svc/app\n"), Some(1500));
    }

    #[test]
    fn effective_memory_is_the_smaller_known_figure() {
        let m = |h, c| MemoryAvailable { host: h, cgroup: c }.effective();
        assert_eq!(m(Some(5), Some(3)), Some(3));
        assert_eq!(m(Some(5), None), Some(5));
        assert_eq!(m(None, Some(3)), Some(3));
        assert_eq!(m(None, None), None);
    }

    #[test]
    fn cgroup_paths_come_from_the_matching_line() {
        let t = "4:memory:/a/b\n3:cpu,cpuacct:/c\n0::/uni\n";
        assert_eq!(cgroup_path(t, "memory").as_deref(), Some("/a/b"));
        assert_eq!(cgroup_path(t, "cpu").as_deref(), Some("/c"));
        assert_eq!(cgroup_path(t, "").as_deref(), Some("/uni"));
        assert_eq!(cgroup_path(t, "pids"), None);
    }

    #[test]
    fn human_sizes() {
        assert_eq!(human(10), "10 B");
        assert_eq!(human(1536), "1.5 KiB");
        assert_eq!(human(3 << 30), "3.0 GiB");
    }

    #[cfg(unix)]
    #[test]
    fn free_disk_of_a_missing_child_asks_the_parent() {
        let d = std::env::temp_dir().join("dla-no-such-dir-xyz/child");
        assert!(free_disk(&d).is_some_and(|b| b > 0));
    }
}
