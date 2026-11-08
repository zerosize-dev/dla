//! Facts about a process other than its memory: its command line, the TCP ports it
//! listens on, and the one signal DLA ever sends it.

use std::collections::HashSet;

/// The argument vector of `pid`, or `None` when it cannot be read.
///
/// Linux reads `/proc/<pid>/cmdline` (exact arguments). Other Unix systems ask `ps`, which
/// loses the quoting of arguments that contain spaces. Windows is not supported.
pub fn command_line(pid: u32) -> Option<Vec<String>> {
    #[cfg(target_os = "linux")]
    {
        let raw = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
        let args: Vec<String> = raw
            .split(|&b| b == 0)
            .filter(|a| !a.is_empty())
            .map(|a| String::from_utf8_lossy(a).into_owned())
            .collect();
        (!args.is_empty()).then_some(args)
    }
    #[cfg(all(unix, not(target_os = "linux")))]
    {
        let out = std::process::Command::new("ps")
            .args(["-o", "args=", "-p", &pid.to_string()])
            .output()
            .ok()?;
        let text = String::from_utf8_lossy(&out.stdout);
        let args: Vec<String> = text.split_whitespace().map(str::to_owned).collect();
        (out.status.success() && !args.is_empty()).then_some(args)
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        None
    }
}

/// The path of the executable image `pid` is running, from the kernel (not from `argv[0]`).
/// Only macOS can answer here (`proc_pidpath`); Linux callers read `/proc/<pid>/exe`.
#[cfg(not(target_os = "linux"))]
pub fn exe_path(pid: u32) -> Option<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    {
        imp::exe_path(pid)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = pid;
        None
    }
}

/// The base name of an executable path without a Windows `.exe` suffix.
pub fn exe_name(arg0: &str) -> &str {
    let base = arg0.rsplit(['/', '\\']).next().unwrap_or(arg0);
    base.strip_suffix(".exe").unwrap_or(base)
}

/// Ports `pid` is listening on (TCP over IPv4 and IPv6), sorted. Empty when unknown; only
/// Linux can answer.
pub fn listening_ports(pid: u32) -> Vec<u16> {
    #[cfg(target_os = "linux")]
    {
        let inodes = socket_inodes(pid);
        if inodes.is_empty() {
            return Vec::new();
        }
        let mut ports = Vec::new();
        for table in ["tcp", "tcp6"] {
            if let Ok(text) = std::fs::read_to_string(format!("/proc/{pid}/net/{table}")) {
                ports.extend(parse_listening(&text, &inodes));
            }
        }
        ports.sort_unstable();
        ports.dedup();
        ports
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        Vec::new()
    }
}

#[cfg(target_os = "linux")]
fn socket_inodes(pid: u32) -> HashSet<u64> {
    let Ok(dir) = std::fs::read_dir(format!("/proc/{pid}/fd")) else {
        return HashSet::new();
    };
    dir.filter_map(Result::ok)
        .filter_map(|e| std::fs::read_link(e.path()).ok())
        .filter_map(|t| {
            let t = t.to_string_lossy().into_owned();
            t.strip_prefix("socket:[")?.strip_suffix(']')?.parse().ok()
        })
        .collect()
}

/// Ports of the LISTEN rows (state `0A`) of a `/proc/net/tcp` table whose socket inode is in
/// `inodes`.
pub fn parse_listening(table: &str, inodes: &HashSet<u64>) -> Vec<u16> {
    table
        .lines()
        .skip(1)
        .filter_map(|line| {
            let f: Vec<&str> = line.split_whitespace().collect();
            if f.get(3) != Some(&"0A") {
                return None;
            }
            let inode: u64 = f.get(9)?.parse().ok()?;
            if !inodes.contains(&inode) {
                return None;
            }
            let port = f.get(1)?.rsplit(':').next()?;
            u16::from_str_radix(port, 16).ok()
        })
        .collect()
}

/// Sends `SIGUSR1` to `pid` (Unix). Node.js starts its inspector on receiving it. Fails on
/// other systems and when the process cannot be signalled.
pub fn signal_usr1(pid: u32) -> Result<(), String> {
    imp::signal_usr1(pid)
}

#[cfg(unix)]
mod imp {
    #![allow(unsafe_code)]

    #[cfg(target_os = "macos")]
    pub fn exe_path(pid: u32) -> Option<std::path::PathBuf> {
        use std::os::unix::ffi::OsStringExt;
        let raw = libc::c_int::try_from(pid).ok()?;
        // PROC_PIDPATHINFO_MAXSIZE is 4 * MAXPATHLEN.
        let mut buf = vec![0u8; 4096];
        // SAFETY: `buf` is writable for the `buf.len()` bytes passed.
        let n = unsafe { libc::proc_pidpath(raw, buf.as_mut_ptr().cast(), buf.len() as u32) };
        let n = usize::try_from(n).ok().filter(|&n| n > 0)?;
        buf.truncate(n);
        Some(std::path::PathBuf::from(std::ffi::OsString::from_vec(buf)))
    }

    pub fn signal_usr1(pid: u32) -> Result<(), String> {
        let pid = libc::pid_t::try_from(pid).map_err(|_| format!("pid {pid} is out of range"))?;
        // SAFETY: kill(2) takes plain integers and has no memory-safety preconditions.
        let rc = unsafe { libc::kill(pid, libc::SIGUSR1) };
        if rc == 0 {
            Ok(())
        } else {
            Err(std::io::Error::last_os_error().to_string())
        }
    }
}

#[cfg(not(unix))]
mod imp {
    pub fn signal_usr1(_pid: u32) -> Result<(), String> {
        Err("SIGUSR1 does not exist on this operating system".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exe_names() {
        assert_eq!(exe_name("/usr/bin/java"), "java");
        assert_eq!(exe_name("C:\\jdk\\bin\\javaw.exe"), "javaw");
        assert_eq!(exe_name("node"), "node");
    }

    #[test]
    fn listening_rows_are_matched_by_inode() {
        let table = "  sl  local_address rem_address   st tx_queue rx_queue tr tm->when retrnsmt   uid  timeout inode\n\
   0: 0100007F:2406 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 5551 1 0000 100 0 0 10 0\n\
   1: 0100007F:1F90 00000000:0000 0A 00000000:00000000 00:00000000 00000000  1000        0 7777 1 0000 100 0 0 10 0\n\
   2: 0100007F:C350 0100007F:2406 01 00000000:00000000 00:00000000 00000000  1000        0 5551 1 0000 100 0 0 10 0\n";
        let mine: HashSet<u64> = [5551].into_iter().collect();
        assert_eq!(parse_listening(table, &mine), [0x2406]);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn own_command_line_is_readable_and_a_listener_is_found() {
        let args = command_line(std::process::id()).unwrap();
        assert!(!args[0].is_empty());
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        assert!(listening_ports(std::process::id()).contains(&port));
    }
}
