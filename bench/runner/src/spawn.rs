//! Starting a child process whose executable may have just been written.
//!
//! On Unix, exec fails with ETXTBSY ("text file busy") while any process holds a write handle
//! of the file. A thread that writes a script and starts it at once can meet that even after
//! its own handle is closed: a fork in another thread inherits the handle and holds it until
//! that child execs. The tests of this crate write fake tools (a fake `dla`, `go`, `uv`) and
//! run them while other tests fork, so every start that can run such a file goes through here
//! and retries briefly.

use std::io;
use std::process::{Child, Command, Output};
use std::time::Duration;

/// How many times a start that met ETXTBSY is tried again.
const TRIES: u32 = 50;
/// The pause between two tries (one second in all).
const PAUSE: Duration = Duration::from_millis(20);

/// True for the error exec gives while a write handle of the executable is open.
fn busy(e: &io::Error) -> bool {
    cfg!(unix) && e.raw_os_error() == Some(26)
}

fn retry<T>(mut start: impl FnMut() -> io::Result<T>) -> io::Result<T> {
    let mut tries = 0;
    loop {
        match start() {
            Err(e) if busy(&e) && tries < TRIES => {
                tries += 1;
                std::thread::sleep(PAUSE);
            }
            r => return r,
        }
    }
}

/// [`Command::spawn`], tried again while the executable is busy.
pub fn spawn(cmd: &mut Command) -> io::Result<Child> {
    retry(|| cmd.spawn())
}

/// [`Command::output`], tried again while the executable is busy.
pub fn output(cmd: &mut Command) -> io::Result<Output> {
    retry(|| cmd.output())
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    /// A script whose write handle is open, closed by another thread after `ms`.
    fn busy_script(
        ms: u64,
    ) -> (
        tempfile::TempDir,
        std::path::PathBuf,
        std::thread::JoinHandle<()>,
    ) {
        let t = tempfile::tempdir().unwrap();
        let bin = t.path().join("fake");
        let mut file = fs::File::create(&bin).unwrap();
        file.write_all(b"#!/bin/sh\necho ran\n").unwrap();
        fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
        // While the handle is open, a plain start fails with ETXTBSY.
        assert_eq!(
            Command::new(&bin).output().unwrap_err().raw_os_error(),
            Some(26)
        );
        let closer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(ms));
            drop(file);
        });
        (t, bin, closer)
    }

    #[test]
    fn a_script_still_open_for_writing_starts_once_its_handle_is_closed() {
        let (_t, bin, closer) = busy_script(150);
        let out = output(&mut Command::new(&bin)).unwrap();
        closer.join().unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "ran");

        let (_t, bin, closer) = busy_script(150);
        let child = spawn(Command::new(&bin).stdout(std::process::Stdio::piped())).unwrap();
        closer.join().unwrap();
        let out = child.wait_with_output().unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "ran");
    }

    #[test]
    fn a_handle_that_stays_open_gives_the_error_after_a_second() {
        let (_t, bin, closer) = busy_script(3000);
        let started = std::time::Instant::now();
        let e = output(&mut Command::new(&bin)).unwrap_err();
        assert_eq!(e.raw_os_error(), Some(26));
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "{:?}",
            started.elapsed()
        );
        closer.join().unwrap();
    }
}
