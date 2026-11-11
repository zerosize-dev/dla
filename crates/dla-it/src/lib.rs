//! Helpers for the dla integration tests that run inside microVMs.
//!
//! The parsing and quoting code here has no VM dependency and is unit tested
//! everywhere. The VM code lives in [`vm`] and needs the `sandbox-it` feature.
//!
//! Environment variables the suite reads:
//!
//! - `DLA_IT_SKIP_VM=1`: every VM test prints a notice and passes without booting.
//! - `DLA_IT_REQUIRE_VM=1`: a host that cannot boot a microVM fails the test
//!   instead of skipping it (CI sets this).
//! - `DLA_IT_BIN_DIR`: directory that holds the static `dla` binary built for
//!   `x86_64-unknown-linux-musl` (default: the musl release directory of the
//!   workspace `target/`).
//! - `DLA_IT_OUT`: directory where tests write the facts they record
//!   (`kernel-facts.json`). Unset: nothing is written.

use serde_json::{Map, Value, json};

#[cfg(feature = "sandbox-it")]
pub mod vm;

/// Quotes `s` for a POSIX shell: one single-quoted word.
pub fn shell_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if c == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(c);
        }
    }
    out.push('\'');
    out
}

/// Parses `key=value` lines (the output of the kernel probe script) into a map.
/// Lines without `=` are ignored. The first `=` splits, so values may contain `=`.
/// A repeated key keeps its last value.
pub fn parse_kv(text: &str) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let k = k.trim();
        if k.is_empty() || k.contains(char::is_whitespace) {
            continue;
        }
        let v = v.trim().to_string();
        if let Some(slot) = out.iter_mut().find(|(key, _)| key == k) {
            slot.1 = v;
        } else {
            out.push((k.to_string(), v));
        }
    }
    out
}

/// The shell script that gathers the kernel facts. It prints `key=value` lines and
/// never fails: a fact that cannot be read prints as `no` or stays empty.
/// `yes` and `no` are used for booleans.
pub const KERNEL_PROBE_SCRIPT: &str = r#"
yn() { if "$@" >/dev/null 2>&1; then echo yes; else echo no; fi; }
echo "uname_s=$(uname -s)"
echo "uname_r=$(uname -r)"
echo "uname_m=$(uname -m)"
echo "uid=$(id -u)"
echo "proc_mounted=$(yn test -r /proc/self/status)"
echo "cap_eff=$(sed -n 's/^CapEff:[[:space:]]*//p' /proc/self/status 2>/dev/null)"
echo "btf=$(yn test -r /sys/kernel/btf/vmlinux)"
echo "kallsyms=$(yn test -r /proc/kallsyms)"
echo "config_gz=$(yn test -r /proc/config.gz)"
mkdir -p /sys/kernel/tracing 2>/dev/null
mount -t tracefs tracefs /sys/kernel/tracing >/dev/null 2>&1
echo "tracefs=$(yn test -d /sys/kernel/tracing/events)"
echo "tracefs_syscalls=$(yn test -d /sys/kernel/tracing/events/syscalls)"
echo "kprobe_events=$(yn test -w /sys/kernel/tracing/kprobe_events)"
echo "uprobe_events=$(yn test -w /sys/kernel/tracing/uprobe_events)"
echo "cgroup2=$(yn grep -q cgroup2 /proc/mounts)"
echo "cgroup_controllers=$(cat /sys/fs/cgroup/cgroup.controllers 2>/dev/null)"
echo "cgroup_memory_max=$(yn test -e /sys/fs/cgroup/memory.max)"
echo "ptrace_scope=$(cat /proc/sys/kernel/yama/ptrace_scope 2>/dev/null)"
echo "perf_event_paranoid=$(cat /proc/sys/kernel/perf_event_paranoid 2>/dev/null)"
echo "unprivileged_bpf_disabled=$(cat /proc/sys/kernel/unprivileged_bpf_disabled 2>/dev/null)"
echo "mem_total_kb=$(sed -n 's/^MemTotal:[[:space:]]*\([0-9]*\).*/\1/p' /proc/meminfo 2>/dev/null)"
echo "container_env=$(printenv container 2>/dev/null)"
echo "dockerenv=$(yn test -e /.dockerenv)"
"#;

/// The kernel facts of a guest, as read by [`KERNEL_PROBE_SCRIPT`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelFacts {
    pub raw: Vec<(String, String)>,
}

impl KernelFacts {
    pub fn parse(text: &str) -> Self {
        Self {
            raw: parse_kv(text),
        }
    }

    /// The value of a fact, if the probe printed it.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.raw
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// A `yes`/`no` fact. `None` when it is missing or not a boolean.
    pub fn flag(&self, key: &str) -> Option<bool> {
        match self.get(key)? {
            "yes" => Some(true),
            "no" => Some(false),
            _ => None,
        }
    }

    pub fn is_root(&self) -> bool {
        self.get("uid") == Some("0")
    }

    /// Whether `dla attach bpftrace` can have what it needs from the kernel:
    /// BTF, the syscall tracepoints and root. A fact, not a requirement: the
    /// tests that need bpftrace read this instead of assuming it.
    pub fn bpftrace_ok(&self) -> bool {
        self.flag("btf") == Some(true)
            && self.flag("tracefs_syscalls") == Some(true)
            && self.is_root()
    }

    /// The facts as JSON: every raw value plus the derived `bpftrace_ok`.
    pub fn to_json(&self) -> Value {
        let mut m = Map::new();
        for (k, v) in &self.raw {
            m.insert(k.clone(), json!(v));
        }
        m.insert("bpftrace_ok".into(), json!(self.bpftrace_ok()));
        Value::Object(m)
    }
}

/// The parts of `dla run init` / `dla probe` output the tests read.
pub fn json_line(stdout: &str) -> Result<Value, String> {
    let line = stdout
        .lines()
        .rev()
        .find(|l| l.trim_start().starts_with('{'))
        .ok_or_else(|| format!("no JSON line in output: {stdout:?}"))?;
    serde_json::from_str(line).map_err(|e| format!("bad JSON ({e}): {line}"))
}

/// Status of one backend in a `coverage.json` document, for a runtime worker.
pub fn coverage_status(coverage: &Value, worker: &str, backend: &str) -> Option<String> {
    coverage
        .get("workers")?
        .get(worker)?
        .get("backends")?
        .get(backend)?
        .get("status")?
        .as_str()
        .map(str::to_string)
}

/// The install hint of one backend in a `coverage.json` document.
pub fn coverage_hint(coverage: &Value, worker: &str, backend: &str) -> Option<String> {
    coverage
        .get("workers")?
        .get(worker)?
        .get("backends")?
        .get(backend)?
        .get("install_hint")?
        .as_str()
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_handles_single_quotes_and_spaces() {
        assert_eq!(shell_quote("a b"), "'a b'");
        assert_eq!(shell_quote("it's"), "'it'\\''s'");
        assert_eq!(shell_quote(""), "''");
    }

    #[test]
    fn parse_kv_splits_on_first_equals_and_keeps_last() {
        let kv = parse_kv("a=1\nnoise\nb=x=y\na=2\n =3\nc d=4\ne=\n");
        assert_eq!(
            kv,
            vec![
                ("a".to_string(), "2".to_string()),
                ("b".to_string(), "x=y".to_string()),
                ("e".to_string(), String::new()),
            ]
        );
    }

    #[test]
    fn bpftrace_needs_btf_syscall_tracepoints_and_root() {
        let ok = KernelFacts::parse("uid=0\nbtf=yes\ntracefs_syscalls=yes\n");
        assert!(ok.bpftrace_ok());
        for text in [
            "uid=0\nbtf=no\ntracefs_syscalls=yes\n",
            "uid=0\nbtf=yes\ntracefs_syscalls=no\n",
            "uid=1000\nbtf=yes\ntracefs_syscalls=yes\n",
            "uid=0\nbtf=yes\n",
        ] {
            assert!(!KernelFacts::parse(text).bpftrace_ok(), "{text}");
        }
    }

    #[test]
    fn facts_json_carries_raw_values_and_the_derived_flag() {
        let f = KernelFacts::parse("uname_s=Linux\nuid=0\nbtf=no\n");
        let j = f.to_json();
        assert_eq!(j["uname_s"], "Linux");
        assert_eq!(j["bpftrace_ok"], false);
        assert_eq!(f.flag("btf"), Some(false));
        assert_eq!(f.flag("uname_s"), None);
    }

    #[test]
    fn probe_script_prints_the_keys_the_tests_read() {
        for key in [
            "uname_s",
            "uid",
            "proc_mounted",
            "btf",
            "tracefs_syscalls",
            "kprobe_events",
            "uprobe_events",
            "cgroup2",
            "ptrace_scope",
        ] {
            assert!(
                KERNEL_PROBE_SCRIPT.contains(&format!("echo \"{key}=")),
                "{key}"
            );
        }
    }

    #[test]
    fn json_line_takes_the_last_json_object() {
        let v = json_line("warning\n{\"a\":1}\n{\"ok\":true}\n").unwrap();
        assert_eq!(v["ok"], true);
        assert!(json_line("nothing").is_err());
    }

    #[test]
    fn coverage_lookup_reads_status_and_hint() {
        let c: Value = serde_json::from_str(
            r#"{"workers":{"go":{"backends":{"golangci-lint":
               {"status":"missing","install_hint":"dla install golangci-lint"}}}}}"#,
        )
        .unwrap();
        assert_eq!(
            coverage_status(&c, "go", "golangci-lint").as_deref(),
            Some("missing")
        );
        assert_eq!(
            coverage_hint(&c, "go", "golangci-lint").as_deref(),
            Some("dla install golangci-lint")
        );
        assert_eq!(coverage_status(&c, "go", "nope"), None);
    }
}
