//! Interrupt handling, live process groups and the results-directory lock.
//!
//! Every child session runs in its own process group, so a Ctrl-C in the
//! terminal never reaches it. The watcher started by [`install`] turns
//! SIGINT and SIGTERM into an orderly stop: SIGTERM to every live group, a
//! short grace period, SIGKILL, then a non-zero exit.

use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

#[cfg(unix)]
use std::{
    process::{Command, Stdio},
    time::Instant,
};

use crate::Res;

/// Seconds between SIGTERM and SIGKILL when the runner is interrupted.
pub const GRACE: Duration = Duration::from_secs(2);

const SIGINT: i32 = 2;
const SIGTERM: i32 = 15;

struct Flags {
    int: Arc<AtomicBool>,
    term: Arc<AtomicBool>,
}

static FLAGS: OnceLock<Flags> = OnceLock::new();
static GROUPS: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static LOCK_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

/// The signal that asked the runner to stop, if any.
pub fn stop_signal() -> Option<i32> {
    let f = FLAGS.get()?;
    if f.int.load(Ordering::SeqCst) {
        Some(SIGINT)
    } else if f.term.load(Ordering::SeqCst) {
        Some(SIGTERM)
    } else {
        None
    }
}

#[cfg(unix)]
fn groups() -> Vec<u32> {
    GROUPS.lock().expect("lock").clone()
}

#[cfg(unix)]
fn signal_group(pgid: u32, sig: &str) {
    let _ = Command::new("kill")
        .args([sig, "--", &format!("-{pgid}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(not(unix))]
fn signal_group(_pgid: u32, _sig: &str) {}

/// Remembers a live child process group (the child's pid) until the guard drops.
pub struct GroupGuard(u32);

/// Registers a child process group. When a stop was already requested, the
/// group is killed at once, so a child spawned during the stop cannot survive.
pub fn register_group(pgid: u32) -> GroupGuard {
    GROUPS.lock().expect("lock").push(pgid);
    if stop_signal().is_some() {
        signal_group(pgid, "-KILL");
    }
    GroupGuard(pgid)
}

impl Drop for GroupGuard {
    fn drop(&mut self) {
        GROUPS.lock().expect("lock").retain(|g| *g != self.0);
    }
}

/// Stops every live group: SIGTERM, a grace period, SIGKILL.
#[cfg(unix)]
fn terminate_groups() {
    let snapshot = groups();
    for g in &snapshot {
        signal_group(*g, "-TERM");
    }
    let start = Instant::now();
    while start.elapsed() < GRACE && !groups().is_empty() {
        thread::sleep(Duration::from_millis(25));
    }
    // Also reaches stragglers of a group whose leader already exited.
    for g in snapshot.into_iter().chain(groups()) {
        signal_group(g, "-KILL");
    }
}

/// Installs the SIGINT and SIGTERM handlers and the watcher thread that stops
/// the children and exits with 128 plus the signal number.
pub fn install() -> Res<()> {
    #[cfg(unix)]
    {
        let flags = Flags {
            int: Arc::new(AtomicBool::new(false)),
            term: Arc::new(AtomicBool::new(false)),
        };
        signal_hook::flag::register(SIGINT, Arc::clone(&flags.int))
            .map_err(|e| format!("cannot install the SIGINT handler: {e}"))?;
        signal_hook::flag::register(SIGTERM, Arc::clone(&flags.term))
            .map_err(|e| format!("cannot install the SIGTERM handler: {e}"))?;
        if FLAGS.set(flags).is_err() {
            return Ok(());
        }
        thread::spawn(|| {
            loop {
                if let Some(sig) = stop_signal() {
                    let name = if sig == SIGINT { "SIGINT" } else { "SIGTERM" };
                    eprintln!(
                        "interrupted ({name}): stopping {} child process group(s)",
                        groups().len()
                    );
                    terminate_groups();
                    if let Some(p) = LOCK_PATH.lock().expect("lock").take() {
                        let _ = fs::remove_file(p);
                    }
                    eprintln!(
                        "interrupted runs have no run.json; start the same command again to continue"
                    );
                    std::process::exit(128 + sig);
                }
                thread::sleep(Duration::from_millis(25));
            }
        });
    }
    Ok(())
}

/// Called by the thread that finished its work while a stop was requested:
/// the watcher exits the process once the children are gone, so this waits
/// for it instead of letting the process end with a success code.
pub fn wait_for_exit() {
    if stop_signal().is_some() {
        thread::sleep(GRACE * 3 + Duration::from_secs(5));
    }
}

/// Whether a process with this pid exists.
#[cfg(unix)]
fn pid_alive(pid: u32) -> bool {
    if Path::new(&format!("/proc/{pid}")).exists() {
        return true;
    }
    Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Without a process-liveness check every lock counts as held, so a stale
/// lock file must be removed by hand. `dla-bench run` is unix-only anyway.
#[cfg(not(unix))]
fn pid_alive(_pid: u32) -> bool {
    true
}

/// The exclusive lock on a results directory; released on drop.
#[derive(Debug)]
pub struct Lock {
    path: PathBuf,
}

/// Name of the lock file inside the results directory.
pub const LOCK_FILE: &str = "runner.lock";

impl Lock {
    /// Takes the lock in `dir`. A lock file whose pid is not alive is stale
    /// and replaced; one whose pid is alive refuses the start.
    pub fn acquire(dir: &Path) -> Res<Lock> {
        let path = dir.join(LOCK_FILE);
        for _ in 0..2 {
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut f) => {
                    writeln!(f, "{}", std::process::id())
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                    *LOCK_PATH.lock().expect("lock") = Some(path.clone());
                    return Ok(Lock { path });
                }
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                    let text = fs::read_to_string(&path).unwrap_or_default();
                    match text.trim().parse::<u32>() {
                        Ok(pid) if pid_alive(pid) => {
                            return Err(format!(
                                "{}: another dla-bench run (pid {pid}) is using this results directory; wait for it or stop it first",
                                path.display()
                            ));
                        }
                        Ok(_) => {
                            let _ = fs::remove_file(&path);
                        }
                        Err(_) => {
                            return Err(format!(
                                "{}: lock file without a pid; remove it if no dla-bench run is active",
                                path.display()
                            ));
                        }
                    }
                }
                Err(e) => return Err(format!("{}: {e}", path.display())),
            }
        }
        Err(format!("{}: cannot take the lock", path.display()))
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        *LOCK_PATH.lock().expect("lock") = None;
        let _ = fs::remove_file(&self.path);
    }
}
