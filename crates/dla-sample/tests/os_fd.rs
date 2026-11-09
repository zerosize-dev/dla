//! `fd.<kind>` columns from `/proc/<pid>/fd` (Linux only).
#![cfg(target_os = "linux")]
// The test opens eventfd, epoll and timerfd descriptors with libc.
#![allow(unsafe_code)]

use std::io::Write;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::process::{Child, Command, Stdio};

use dla_sample::os;

/// The tests open and close descriptors in this process, so they run one at a time.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn serial() -> std::sync::MutexGuard<'static, ()> {
    SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn sleeper(stdin: Stdio, stdout: Stdio, stderr: Stdio) -> Child {
    Command::new("sleep")
        .arg("60")
        .stdin(stdin)
        .stdout(stdout)
        .stderr(stderr)
        .spawn()
        .expect("sleep")
}

fn column(s: &os::OsSample, name: &str) -> f64 {
    *s.values
        .get(name)
        .unwrap_or_else(|| panic!("missing {name}: {s:?}"))
}

#[test]
fn a_child_with_a_socket_a_pipe_and_dev_null() {
    let _guard = serial();
    let (ours, theirs) = UnixStream::pair().expect("socketpair");
    let mut child = sleeper(
        Stdio::from(OwnedFd::from(theirs)),
        Stdio::piped(),
        Stdio::null(),
    );
    drop(ours);
    // `sleep` opens nothing else, but it inherits any descriptor the test runner left open
    // without close-on-exec (CI runners pass extra pipes), so the figures are lower bounds.
    let mut s = os::sample(child.id()).expect("sample child");
    for _ in 0..50 {
        if s.values.get("fd.socket").is_some_and(|v| *v >= 1.0) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
        s = os::sample(child.id()).expect("sample child");
    }
    assert!(column(&s, "fd.socket") >= 1.0, "{s:?}");
    assert!(column(&s, "fd.pipe") >= 1.0, "{s:?}");
    assert!(column(&s, "fd.dev") >= 1.0, "{s:?}");
    // The columns add up to os.fds.
    let sum: f64 = s
        .values
        .iter()
        .filter(|(k, _)| k.starts_with("fd."))
        .map(|(_, v)| v)
        .sum();
    assert_eq!(sum, column(&s, "os.fds"), "{s:?}");
    child.kill().expect("kill");
    child.wait().expect("reap");
}

#[test]
fn own_descriptors_by_kind() {
    let _guard = serial();
    let before = os::sample(std::process::id()).expect("own");
    // Hold one of each: a deleted file, a regular file, a pipe, eventfd, epoll and timerfd.
    let dir = tempfile::tempdir().expect("tmp dir");
    let path = dir.path().join("gone");
    std::fs::File::create(&path)
        .and_then(|mut f| writeln!(f, "x"))
        .expect("write");
    let deleted = std::fs::File::open(&path).expect("open");
    std::fs::remove_file(&path).expect("unlink");
    let regular = std::fs::File::open("/etc/hostname").expect("open");
    let (reader, writer) = std::io::pipe().expect("pipe");
    let event = unsafe { libc::eventfd(0, libc::EFD_CLOEXEC) };
    let epoll = unsafe { libc::epoll_create1(libc::EPOLL_CLOEXEC) };
    let timer = unsafe { libc::timerfd_create(libc::CLOCK_MONOTONIC, libc::TFD_CLOEXEC) };
    assert!(event >= 0 && epoll >= 0 && timer >= 0);

    let after = os::sample(std::process::id()).expect("own");
    let grew = |name: &str| column(&after, name) - column(&before, name);
    assert!(grew("fd.eventfd") >= 1.0, "{after:?}");
    assert!(grew("fd.eventpoll") >= 1.0);
    assert!(grew("fd.timerfd") >= 1.0);
    assert!(grew("fd.pipe") >= 2.0);
    assert!(grew("fd.file") >= 1.0);
    assert!(grew("fd.deleted") >= 1.0);
    for name in [
        "fd.socket",
        "fd.pipe",
        "fd.file",
        "fd.deleted",
        "fd.dev",
        "fd.eventfd",
        "fd.eventpoll",
        "fd.timerfd",
    ] {
        assert!(
            after.values.contains_key(name),
            "base column {name} always present"
        );
    }
    for fd in [event, epoll, timer] {
        unsafe { libc::close(fd) };
    }
    drop((deleted, regular, reader, writer));
}

#[test]
fn a_missing_process_still_fails_cleanly() {
    assert!(os::sample(u32::MAX - 1).is_err());
}
