//! Tests that sample real processes.

use std::process::{Child, Command, Stdio};
use std::time::Duration;

use dla_sample::os::{self, SampleError, Sampler, SamplerConfig};

fn current() -> u32 {
    std::process::id()
}

#[test]
fn samples_the_current_process() {
    let s = os::sample(current()).expect("own process exists");
    assert!(s.values["os.rss"] > 0.0);
    assert!(s.values["os.threads"] >= 1.0);
    for (name, v) in &s.values {
        assert!(v.is_finite() && *v >= 0.0, "{name} = {v}");
    }
}

#[test]
fn missing_process_is_the_only_error() {
    // Pids above the kernel maximum cannot exist on any supported platform.
    let pid = u32::MAX - 1;
    assert_eq!(os::sample(pid), Err(SampleError::NoSuchProcess(pid)));
    assert!(!os::is_alive(pid));
    assert_eq!(os::process_start_time(pid), None);
    assert!(os::descendants(pid).is_empty());
}

#[test]
fn start_time_is_stable_and_well_formed() {
    let a = os::process_start_time(current()).expect("start time");
    let b = os::process_start_time(current()).expect("start time");
    assert_eq!(a, b);
    assert_eq!(a.len(), 20, "{a}");
    assert!(a.ends_with('Z') && a.contains('T'), "{a}");
    assert!(os::is_alive(current()));
}

/// A child that sleeps for a while, in its own process.
fn sleeper() -> Child {
    #[cfg(unix)]
    let mut cmd = {
        let mut c = Command::new("sleep");
        c.arg("30");
        c
    };
    #[cfg(windows)]
    let mut cmd = {
        let mut c = Command::new("ping");
        c.args(["-n", "30", "127.0.0.1"]);
        c
    };
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn sleeper")
}

#[test]
fn samples_a_child_and_finds_it_as_a_descendant() {
    let mut child = sleeper();
    let pid = child.id();
    assert!(os::is_alive(pid));
    assert!(os::descendants(current()).contains(&pid));
    let s = os::sample(pid).expect("child exists");
    assert!(s.values["os.rss"] > 0.0);
    assert!(s.values["os.threads"] >= 1.0);

    child.kill().expect("kill");
    child.wait().expect("reap");
    assert!(!os::is_alive(pid));
    assert_eq!(os::sample(pid), Err(SampleError::NoSuchProcess(pid)));
}

#[test]
fn exited_but_unreaped_child_counts_as_gone() {
    let mut child = sleeper();
    let pid = child.id();
    child.kill().expect("kill");
    // Wait for the exit without reaping so the process lingers as a zombie on unix.
    for _ in 0..500 {
        if !os::is_alive(pid) {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(!os::is_alive(pid));
    assert_eq!(os::sample(pid), Err(SampleError::NoSuchProcess(pid)));
    child.wait().expect("reap");
}

#[test]
fn sampler_collects_until_stopped() {
    let sampler = Sampler::start(
        current(),
        SamplerConfig {
            interval: Duration::from_millis(20),
            tree: false,
        },
    )
    .expect("start");
    // Generous: the sampler needs two ticks after its first sample, and a loaded CI runner
    // can stall a 20 ms thread for a long time.
    std::thread::sleep(Duration::from_millis(600));
    assert!(sampler.is_running());
    let out = sampler.stop();
    assert!(out.samples.len() >= 3, "{} samples", out.samples.len());
    assert!(!out.exited);
    assert!(out.error.is_none());
    assert!(out.samples.windows(2).all(|w| w[0].t_ms <= w[1].t_ms));
    let max_rss = out
        .samples
        .iter()
        .map(|s| s.values["os.rss"])
        .fold(0.0, f64::max);
    assert_eq!(out.peaks["os.rss"], max_rss);
}

#[test]
fn sampler_ends_when_the_process_exits() {
    let mut child = sleeper();
    let sampler = Sampler::start(
        child.id(),
        SamplerConfig {
            interval: Duration::from_millis(20),
            tree: false,
        },
    )
    .expect("start");
    child.kill().expect("kill");
    child.wait().expect("reap");
    for _ in 0..500 {
        if !sampler.is_running() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    assert!(!sampler.is_running());
    let out = sampler.stop();
    assert!(out.exited);
    assert!(!out.samples.is_empty());
}

#[test]
fn sampler_rejects_a_missing_process() {
    let pid = u32::MAX - 1;
    assert_eq!(
        Sampler::start(pid, SamplerConfig::default()).err(),
        Some(SampleError::NoSuchProcess(pid))
    );
}

#[test]
fn tree_sampling_sums_descendants_and_names_the_largest() {
    let mut child = sleeper();
    let sampler = Sampler::start(
        current(),
        SamplerConfig {
            interval: Duration::from_millis(20),
            tree: true,
        },
    )
    .expect("start");
    std::thread::sleep(Duration::from_millis(100));
    let out = sampler.stop();
    let own = os::sample(current()).expect("own").values["os.rss"];
    let largest = out.largest_descendant.expect("a descendant was seen");
    assert!(os::descendants(current()).contains(&largest.pid));
    assert!(out.samples[0].values["os.rss"] > own * 0.5);
    child.kill().expect("kill");
    child.wait().expect("reap");
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;

    #[test]
    fn reports_the_linux_columns() {
        let s = os::sample(current()).expect("own process");
        for column in [
            "os.rss",
            "os.rss_anon",
            "os.rss_file",
            "os.vm_data",
            "os.vm_hwm",
            "os.threads",
            "os.fds",
        ] {
            assert!(s.values.contains_key(column), "missing {column}: {s:?}");
        }
        assert!(s.values["os.vm_hwm"] >= s.values["os.rss"] * 0.5);
        // Either smaps_rollup is readable or the loss is explained.
        assert!(
            s.values.contains_key("os.pss")
                || s.limitations.iter().any(|l| l.contains("smaps_rollup")),
            "{s:?}"
        );
        // stdin, stdout and stderr at least (other tests open and close fds concurrently).
        assert!(s.values["os.fds"] >= 3.0, "{s:?}");
    }

    #[test]
    fn container_memory_is_present_or_explained() {
        let s = os::sample(current()).expect("own process");
        if let Some(usage) = s.values.get("cgroup.current") {
            assert!(*usage > 0.0);
        } else {
            assert!(
                s.limitations.iter().any(|l| l.starts_with("cgroup.")),
                "{s:?}"
            );
        }
    }

    #[test]
    fn allocated_memory_shows_up_in_rss() {
        let before = os::sample(current()).expect("own").values["os.rss_anon"];
        let mut block = vec![0u8; 64 << 20];
        for i in (0..block.len()).step_by(4096) {
            block[i] = 1;
        }
        let after = os::sample(current()).expect("own").values["os.rss_anon"];
        assert!(
            after - before > 32.0 * 1024.0 * 1024.0,
            "{before} -> {after}"
        );
        std::hint::black_box(&block);
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;

    #[test]
    fn reports_the_macos_columns() {
        let s = os::sample(current()).expect("own process");
        for column in ["os.rss", "os.phys_footprint", "os.threads", "os.fds"] {
            assert!(s.values[column] > 0.0, "{column}: {s:?}");
        }
    }
}

#[cfg(windows)]
mod windows {
    use super::*;

    #[test]
    fn reports_the_windows_columns() {
        let s = os::sample(current()).expect("own process");
        for column in [
            "os.rss",
            "os.private",
            "os.pagefile",
            "os.handles",
            "os.threads",
        ] {
            assert!(s.values[column] > 0.0, "{column}: {s:?}");
        }
    }
}
