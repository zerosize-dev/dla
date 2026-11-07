//! Privilege and platform facts.

use std::{fs, path::Path, time::Duration};

use dla_core::probe::PrivilegeFacts;

use crate::exec;

fn uid() -> Option<u32> {
    if let Ok(s) = fs::read_to_string("/proc/self/status") {
        for l in s.lines() {
            if let Some(rest) = l.strip_prefix("Uid:") {
                return rest.split_whitespace().next()?.parse().ok();
            }
        }
    }
    if cfg!(unix) {
        let o = exec::run(Path::new("id"), &["-u"], None, &[], Duration::from_secs(3));
        if o.success {
            return o.text.trim().parse().ok();
        }
    }
    None
}

fn container_hints() -> Vec<String> {
    let mut v = Vec::new();
    if Path::new("/.dockerenv").exists() {
        v.push("/.dockerenv".to_owned());
    }
    if Path::new("/run/.containerenv").exists() {
        v.push("/run/.containerenv".to_owned());
    }
    if let Ok(c) = fs::read_to_string("/proc/1/cgroup") {
        for k in ["docker", "kubepods", "containerd", "lxc", "podman"] {
            if c.contains(k) {
                v.push(format!("cgroup:{k}"));
            }
        }
    }
    if std::env::var_os("KUBERNETES_SERVICE_HOST").is_some() {
        v.push("env:KUBERNETES_SERVICE_HOST".to_owned());
    }
    if std::env::var_os("container").is_some() {
        v.push("env:container".to_owned());
    }
    v
}

/// Collects the facts for this machine.
pub fn collect(timeout: Duration) -> PrivilegeFacts {
    let os = std::env::consts::OS.to_owned();
    let uid = uid();
    let ptrace_scope = fs::read_to_string("/proc/sys/kernel/yama/ptrace_scope")
        .ok()
        .and_then(|s| s.trim().parse().ok());
    let macos_sip = (os == "macos")
        .then(|| {
            let o = exec::run(Path::new("csrutil"), &["status"], None, &[], timeout);
            (o.success || !o.text.is_empty()).then(|| exec::shorten(o.text.trim(), 200))
        })
        .flatten();
    PrivilegeFacts {
        root: uid == Some(0),
        os,
        uid,
        ptrace_scope,
        container: container_hints(),
        macos_sip,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_the_os_and_a_consistent_root_flag() {
        let p = collect(Duration::from_secs(3));
        assert_eq!(p.os, std::env::consts::OS);
        assert_eq!(p.root, p.uid == Some(0));
        if cfg!(target_os = "linux") {
            assert!(p.uid.is_some());
        }
    }
}
