//! Linux backend: `/proc/<pid>/{status,smaps_rollup,stat,fd,cgroup}` and the cgroup filesystem.

use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::time::rfc3339_utc;
use super::{OsSample, SampleError};

const CGROUP_ROOT: &str = "/sys/fs/cgroup";

fn proc_path(pid: u32, file: &str) -> PathBuf {
    PathBuf::from(format!("/proc/{pid}/{file}"))
}

/// Whether a read failure means the process is gone.
fn gone(e: &std::io::Error) -> bool {
    e.kind() == ErrorKind::NotFound || e.raw_os_error() == Some(libc::ESRCH)
}

/// Value of the first token after `key:` on a `/proc` style `Key:   123 kB` line.
fn field(text: &str, key: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let rest = line.strip_prefix(key)?.strip_prefix(':')?;
        rest.split_whitespace().next()?.parse().ok()
    })
}

/// Same as [`field`] but for `kB` values, converted to bytes.
fn field_bytes(text: &str, key: &str) -> Option<u64> {
    field(text, key).map(|kb| kb * 1024)
}

/// Value of a `key value` line (cgroup `memory.stat`).
fn stat_value(text: &str, key: &str) -> Option<u64> {
    text.lines().find_map(|line| {
        let mut it = line.split_whitespace();
        (it.next()? == key).then(|| it.next()?.parse().ok())?
    })
}

pub(super) fn sample(pid: u32) -> Result<OsSample, SampleError> {
    let status = match fs::read_to_string(proc_path(pid, "status")) {
        Ok(s) => s,
        Err(e) if gone(&e) => return Err(SampleError::NoSuchProcess(pid)),
        Err(e) => {
            return Err(SampleError::Os {
                pid,
                message: format!("/proc/{pid}/status: {e}"),
            });
        }
    };
    if status
        .lines()
        .find_map(|l| l.strip_prefix("State:"))
        .is_some_and(|s| s.trim_start().starts_with('Z'))
    {
        return Err(SampleError::NoSuchProcess(pid));
    }

    let mut out = OsSample::default();
    let table = [
        ("VmRSS", "os.rss"),
        ("RssAnon", "os.rss_anon"),
        ("RssFile", "os.rss_file"),
        ("RssShmem", "os.rss_shmem"),
        ("VmSwap", "os.swap"),
        ("VmData", "os.vm_data"),
        ("VmHWM", "os.vm_hwm"),
    ];
    let mut missing = Vec::new();
    for (key, column) in table {
        match field_bytes(&status, key) {
            Some(v) => out.put(column, v),
            None => missing.push(column),
        }
    }
    if !missing.is_empty() {
        out.limit(format!(
            "/proc/{pid}/status lacks {} (kernel thread or old kernel)",
            missing.join(", ")
        ));
    }
    match field(&status, "Threads") {
        Some(v) => out.put("os.threads", v),
        None => out.limit("os.threads: no Threads line in /proc status"),
    }

    smaps_rollup(pid, &mut out);
    fds(pid, &mut out);
    cgroup(pid, &mut out);
    Ok(out)
}

fn smaps_rollup(pid: u32, out: &mut OsSample) {
    let text = match fs::read_to_string(proc_path(pid, "smaps_rollup")) {
        Ok(t) => t,
        Err(e) => {
            out.limit(format!(
                "os.pss, os.pss_anon, os.private_dirty: /proc/{pid}/smaps_rollup unreadable ({e})"
            ));
            return;
        }
    };
    let mut missing = Vec::new();
    for (key, column) in [
        ("Pss", "os.pss"),
        ("Pss_Anon", "os.pss_anon"),
        ("Private_Dirty", "os.private_dirty"),
    ] {
        match field_bytes(&text, key) {
            Some(v) => out.put(column, v),
            None => missing.push(column),
        }
    }
    if !missing.is_empty() {
        out.limit(format!("smaps_rollup lacks {}", missing.join(", ")));
    }
}

fn fds(pid: u32, out: &mut OsSample) {
    match fs::read_dir(proc_path(pid, "fd")) {
        Ok(dir) => {
            let entries: Vec<_> = dir.filter_map(Result::ok).collect();
            out.put("os.fds", entries.len() as u64);
            super::fd::record(out, pid, entries.iter().map(|e| fs::read_link(e.path())));
        }
        Err(e) => out.limit(format!("os.fds: /proc/{pid}/fd unreadable ({e})")),
    }
}

/// Which cgroup hierarchy holds the memory controller of a process.
struct Memcg {
    /// `true` for cgroup v2 (`memory.current`), `false` for v1 (`memory.usage_in_bytes`).
    v2: bool,
    /// The cgroup path from `/proc/<pid>/cgroup`.
    path: String,
}

fn memory_cgroup(text: &str) -> Option<Memcg> {
    let mut v2 = None;
    for line in text.lines() {
        let mut parts = line.splitn(3, ':');
        let (Some(id), Some(controllers), Some(path)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if controllers.split(',').any(|c| c == "memory") {
            return Some(Memcg {
                v2: false,
                path: path.to_owned(),
            });
        }
        if id == "0" && controllers.is_empty() {
            v2 = Some(Memcg {
                v2: true,
                path: path.to_owned(),
            });
        }
    }
    v2
}

fn cgroup(pid: u32, out: &mut OsSample) {
    let text = match fs::read_to_string(proc_path(pid, "cgroup")) {
        Ok(t) => t,
        Err(e) => {
            out.limit(format!("cgroup.*: /proc/{pid}/cgroup unreadable ({e})"));
            return;
        }
    };
    let Some(memcg) = memory_cgroup(&text) else {
        out.limit("cgroup.*: no memory cgroup for the process");
        return;
    };
    let mount = if memcg.v2 {
        PathBuf::from(CGROUP_ROOT)
    } else {
        Path::new(CGROUP_ROOT).join("memory")
    };
    // Inside a cgroup namespace the reported path may not exist under the mount; the mount
    // root is then the container's own cgroup.
    let relative = memcg.path.trim_start_matches('/');
    let mut candidates = Vec::new();
    if !relative.is_empty() {
        candidates.push(mount.join(relative));
    }
    candidates.push(mount);
    let (usage_file, stat_keys): (&str, [(&str, &str); 2]) = if memcg.v2 {
        (
            "memory.current",
            [("anon", "cgroup.anon"), ("file", "cgroup.file")],
        )
    } else {
        (
            "memory.usage_in_bytes",
            [("rss", "cgroup.anon"), ("cache", "cgroup.file")],
        )
    };
    for dir in &candidates {
        let Some(usage) = fs::read_to_string(dir.join(usage_file))
            .ok()
            .and_then(|t| t.trim().parse::<u64>().ok())
        else {
            continue;
        };
        out.put("cgroup.current", usage);
        match fs::read_to_string(dir.join("memory.stat")) {
            Ok(stat) => {
                for (key, column) in stat_keys {
                    match stat_value(&stat, key) {
                        Some(v) => out.put(column, v),
                        None => out.limit(format!("{column}: no '{key}' in memory.stat")),
                    }
                }
            }
            Err(e) => out.limit(format!(
                "cgroup.anon, cgroup.file: memory.stat unreadable ({e})"
            )),
        }
        return;
    }
    out.limit(format!(
        "cgroup.*: {usage_file} unreadable for cgroup '{}'",
        memcg.path
    ));
}

/// Fields of `/proc/<pid>/stat` after the parenthesised command name, starting at `state`
/// (field 3). The command may contain spaces and parentheses, so split at the last `)`.
fn stat_fields(text: &str) -> Option<Vec<&str>> {
    let (_, rest) = text.rsplit_once(')')?;
    Some(rest.split_whitespace().collect())
}

fn read_stat(pid: u32) -> Option<String> {
    fs::read_to_string(proc_path(pid, "stat")).ok()
}

#[allow(unsafe_code)]
fn clock_ticks_per_second() -> u64 {
    // SAFETY: sysconf has no preconditions; it returns -1 for an unsupported name.
    let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    u64::try_from(hz).ok().filter(|&h| h > 0).unwrap_or(100)
}

fn boot_time_secs() -> Option<u64> {
    fs::read_to_string("/proc/stat")
        .ok()?
        .lines()
        .find_map(|l| l.strip_prefix("btime ")?.trim().parse().ok())
}

pub(super) fn process_start_time(pid: u32) -> Option<String> {
    let text = read_stat(pid)?;
    let fields = stat_fields(&text)?;
    // Field 22 (starttime, in clock ticks since boot) is index 19 after `state` (field 3).
    let ticks: u64 = fields.get(19)?.parse().ok()?;
    let secs = boot_time_secs()? + ticks / clock_ticks_per_second();
    Some(rfc3339_utc(secs))
}

pub(super) fn is_alive(pid: u32) -> bool {
    read_stat(pid)
        .as_deref()
        .and_then(stat_fields)
        .is_some_and(|f| f.first().is_some_and(|state| *state != "Z"))
}

pub(super) fn descendants(pid: u32) -> Vec<u32> {
    let mut children: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    let Ok(dir) = fs::read_dir("/proc") else {
        return Vec::new();
    };
    for entry in dir.flatten() {
        let Some(child) = entry
            .file_name()
            .to_str()
            .and_then(|n| n.parse::<u32>().ok())
        else {
            continue;
        };
        let Some(ppid) = read_stat(child)
            .as_deref()
            .and_then(stat_fields)
            .and_then(|f| f.get(1)?.parse::<u32>().ok())
        else {
            continue;
        };
        children.entry(ppid).or_default().push(child);
    }
    let mut found = Vec::new();
    let mut queue = VecDeque::from([pid]);
    while let Some(parent) = queue.pop_front() {
        for &child in children.get(&parent).into_iter().flatten() {
            if child != pid && !found.contains(&child) {
                found.push(child);
                queue.push_back(child);
            }
        }
    }
    found.sort_unstable();
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status_style_fields() {
        let text = "Name:\tx\nVmRSS:\t   2048 kB\nThreads:\t7\n";
        assert_eq!(field_bytes(text, "VmRSS"), Some(2048 * 1024));
        assert_eq!(field(text, "Threads"), Some(7));
        assert_eq!(field(text, "VmSwap"), None);
    }

    #[test]
    fn stat_fields_survive_odd_command_names() {
        let text = "12 (a) b (c)) S 1 2 3";
        assert_eq!(stat_fields(text).unwrap(), ["S", "1", "2", "3"]);
    }

    #[test]
    fn picks_memory_cgroup_for_v1_and_v2() {
        let v1 = "4:memory:/a/b\n1:cpu,cpuacct:/\n0::/\n";
        let m = memory_cgroup(v1).unwrap();
        assert!(!m.v2);
        assert_eq!(m.path, "/a/b");
        let v2 = "0::/user.slice/x\n";
        let m = memory_cgroup(v2).unwrap();
        assert!(m.v2);
        assert_eq!(m.path, "/user.slice/x");
        assert!(memory_cgroup("1:cpu:/\n").is_none());
    }

    #[test]
    fn reads_cgroup_stat_keys() {
        let stat = "cache 10\nrss 20\ntotal_rss 30\n";
        assert_eq!(stat_value(stat, "rss"), Some(20));
        assert_eq!(stat_value(stat, "cache"), Some(10));
        assert_eq!(stat_value(stat, "anon"), None);
    }
}
