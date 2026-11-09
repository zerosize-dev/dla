//! The vgdb chain of `dla harness run --tool valgrind`: what keeps the biggest leaked
//! block alive.
//!
//! Valgrind runs with `--vgdb=yes --vgdb-stop-at=exit --vgdb-prefix=<run tmp>/vgdb/vg`, so
//! it stops at exit and waits for a gdb client (the prefix is under the run directory
//! because the sandbox's `/tmp` may be read-only). A standalone `vgdb <command>` call lets
//! the process go on as soon as it returns, so one stop allows one call only. This module
//! therefore keeps one `vgdb --pid=<pid>` relay open and speaks the small part of the gdb
//! remote protocol that monitor commands need (`qRcmd`), which is what `vgdb <command>`
//! sends itself.
//!
//! Commands, in order, each answered with the text Valgrind prints:
//!
//! 1. `leak_check full reachable any`;
//! 2. `block_list <n>` for the top record: the biggest still-reachable one, or the biggest
//!    definitely-lost one when nothing is reachable;
//! 3. `who_points_at <start> <size>` for a block of that record, then for the block each
//!    answer names (`next_block_to_query`), up to a data symbol, a stack or nothing. At most
//!    [`MAX_CALLS`] queries; the walk starts at the last listed block (the newest
//!    allocation, the head of a list that grows at the front) and, if that gives no root,
//!    the first one.
//!
//! The answers go next to the XML as `vgdb-1-leak.txt`, `vgdb-2-blocks.txt` and
//! `vgdb-3-who.txt`, each answer behind the `sending command ... to pid N` line `vgdb`
//! itself prints. The whole chain gives up after [`BUDGET`] and says so in the notes.
//! Whatever happens, the relay is detached at the end so the process exits normally and
//! Valgrind writes its XML.

use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError},
    },
    thread,
    time::{Duration, Instant},
};

use dla_parsers::native::vgdb::{self, BlockRef, Holder, LeakCheck, LossRecord, Step};

/// Most `who_points_at` queries of the chain.
pub(super) const MAX_CALLS: usize = 12;
/// Longest the chain keeps going once Valgrind has stopped.
pub(super) const BUDGET: Duration = Duration::from_secs(30);
/// Longest one command may take.
const COMMAND_LIMIT: Duration = Duration::from_secs(15);

/// What the driver needs to find and ask the process.
#[derive(Clone, Debug)]
pub(super) struct VgdbPlan {
    /// The `vgdb` executable.
    pub vgdb: PathBuf,
    /// The `--vgdb-prefix` given to Valgrind.
    pub prefix: PathBuf,
    /// The run's combined log, where Valgrind says it waits for `vgdb`.
    pub log: PathBuf,
    /// Where the answer files go (`out/<label>/`).
    pub dir: PathBuf,
}

/// The files and notes the chain produced.
#[derive(Default, Debug)]
pub(super) struct VgdbOutcome {
    /// Answer files written, in command order.
    pub files: Vec<PathBuf>,
    /// Limitations, plain text.
    pub notes: Vec<String>,
    /// `who_points_at` queries asked.
    pub calls: usize,
    /// The walk reached a data symbol, a stack or a segment.
    pub rooted: bool,
}

/// The Valgrind flags that make a process wait at exit for `vgdb`.
pub(super) fn valgrind_flags(prefix: &Path) -> Vec<String> {
    vec![
        "--vgdb=yes".to_owned(),
        "--vgdb-stop-at=exit".to_owned(),
        format!("--vgdb-prefix={}", prefix.display()),
    ]
}

// ---------------------------------------------------------------------------------------
// Remote protocol

/// `$<payload>#<checksum>`.
fn packet(payload: &str) -> Vec<u8> {
    let sum = payload.bytes().fold(0u8, |a, b| a.wrapping_add(b));
    format!("${payload}#{sum:02x}").into_bytes()
}

/// Undoes the protocol's run-length coding: `X*N` is `X` repeated `N - 29` more times.
fn unrle(payload: &str) -> String {
    let mut out = String::with_capacity(payload.len());
    let mut chars = payload.chars();
    while let Some(c) = chars.next() {
        if c == '*'
            && let Some(n) = chars.next()
            && let Some(last) = out.chars().last()
        {
            let extra = (n as u32).saturating_sub(29);
            for _ in 0..extra {
                out.push(last);
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn hex_encode(text: &str) -> String {
    text.bytes().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(hex.get(i..i + 2)?, 16).ok())
        .collect()
}

/// One decoded reply packet.
#[derive(Debug, PartialEq)]
enum Reply {
    /// Console output (`O<hex>`).
    Output(Vec<u8>),
    /// The command is finished.
    Ok,
    /// `E<nn>`: the command failed.
    Error(String),
    /// Anything else (stop replies).
    Other,
}

/// Takes the complete packets off the front of `buf`.
fn take_packets(buf: &mut Vec<u8>) -> Vec<Reply> {
    let mut out = Vec::new();
    loop {
        let Some(start) = buf.iter().position(|b| *b == b'$') else {
            buf.clear();
            return out;
        };
        let Some(hash) = buf[start..].iter().position(|b| *b == b'#') else {
            buf.drain(..start);
            return out;
        };
        let end = start + hash;
        if buf.len() < end + 3 {
            buf.drain(..start);
            return out;
        }
        let payload = unrle(&String::from_utf8_lossy(&buf[start + 1..end]));
        buf.drain(..end + 3);
        out.push(if payload == "OK" {
            Reply::Ok
        } else if let Some(hex) = payload.strip_prefix('O') {
            match hex_decode(hex) {
                Some(bytes) => Reply::Output(bytes),
                None => Reply::Other,
            }
        } else if payload.starts_with('E') && payload.len() == 3 {
            Reply::Error(payload)
        } else {
            Reply::Other
        });
    }
}

/// An open `vgdb --pid` relay.
struct Session {
    child: Child,
    rx: Receiver<Vec<u8>>,
    buf: Vec<u8>,
    deadline: Instant,
}

impl Session {
    fn open(plan: &VgdbPlan, pid: u32, deadline: Instant) -> Result<Self, String> {
        let mut child = Command::new(&plan.vgdb)
            .arg(format!("--pid={pid}"))
            .arg(format!("--vgdb-prefix={}", plan.prefix.display()))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("cannot start {}: {e}", plan.vgdb.display()))?;
        let mut stdout = child.stdout.take().ok_or("vgdb has no output pipe")?;
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut chunk = [0u8; 8192];
            while let Ok(n) = stdout.read(&mut chunk) {
                if n == 0 || tx.send(chunk[..n].to_vec()).is_err() {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            rx,
            buf: Vec::new(),
            deadline,
        })
    }

    fn send(&mut self, payload: &str) -> Result<(), String> {
        let stdin = self.child.stdin.as_mut().ok_or("vgdb input is closed")?;
        stdin
            .write_all(&packet(payload))
            .and_then(|()| stdin.flush())
            .map_err(|e| format!("cannot write to vgdb: {e}"))
    }

    fn ack(&mut self) {
        if let Some(stdin) = self.child.stdin.as_mut() {
            let _ = stdin.write_all(b"+").and_then(|()| stdin.flush());
        }
    }

    /// Runs one monitor command and returns its text.
    fn command(&mut self, text: &str) -> Result<String, String> {
        self.send(&format!("qRcmd,{}", hex_encode(text)))?;
        let limit = Instant::now() + COMMAND_LIMIT;
        let mut out: Vec<u8> = Vec::new();
        loop {
            for reply in take_packets(&mut self.buf) {
                self.ack();
                match reply {
                    Reply::Output(b) => out.extend(b),
                    Reply::Ok => return Ok(String::from_utf8_lossy(&out).into_owned()),
                    Reply::Error(e) => return Err(format!("{text}: vgdb answered {e}")),
                    Reply::Other => {}
                }
            }
            let now = Instant::now();
            let until = limit.min(self.deadline);
            if now >= until {
                return Err(format!(
                    "{text}: no answer within {} s",
                    COMMAND_LIMIT.as_secs()
                ));
            }
            match self.rx.recv_timeout(until - now) {
                Ok(chunk) => self.buf.extend(chunk),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(format!("{text}: vgdb closed the connection"));
                }
            }
        }
    }

    /// Lets the process go on: detach, then close the relay.
    fn detach(mut self) {
        let _ = self.send("D");
        let until = Instant::now() + Duration::from_secs(3);
        'wait: while Instant::now() < until {
            for reply in take_packets(&mut self.buf) {
                self.ack();
                if reply == Reply::Ok {
                    break 'wait;
                }
            }
            match self.rx.recv_timeout(Duration::from_millis(100)) {
                Ok(chunk) => self.buf.extend(chunk),
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
        drop(self.child.stdin.take());
        let until = Instant::now() + Duration::from_secs(3);
        while Instant::now() < until {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Resumes a stopped process with a standalone `vgdb` call (which detaches when it is done).
fn release(plan: &VgdbPlan, pid: u32) {
    let child = Command::new(&plan.vgdb)
        .arg(format!("--pid={pid}"))
        .arg(format!("--vgdb-prefix={}", plan.prefix.display()))
        .arg("v.info")
        .arg("n_errs_found")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    if let Ok(mut c) = child {
        let until = Instant::now() + Duration::from_secs(5);
        while Instant::now() < until {
            if matches!(c.try_wait(), Ok(Some(_))) {
                return;
            }
            thread::sleep(Duration::from_millis(20));
        }
        let _ = c.kill();
        let _ = c.wait();
    }
}

// ---------------------------------------------------------------------------------------
// The chain

/// Waits until the log says Valgrind stopped at exit. `None` when `stop` is set first.
/// Reads only what was appended since the last poll.
fn wait_for_stop(log: &Path, stop: &AtomicBool) -> Option<()> {
    let mut file: Option<fs::File> = None;
    let mut seen = 0u64;
    let mut tail = String::new();
    let mut chunk = vec![0u8; 8192];
    loop {
        if file.is_none() {
            file = fs::File::open(log).ok();
        }
        if let Some(f) = file.as_mut()
            && f.seek(SeekFrom::Start(seen)).is_ok()
        {
            loop {
                let n = match f.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => n,
                };
                seen += n as u64;
                tail.push_str(&String::from_utf8_lossy(&chunk[..n]));
                if tail.contains("vgdb me ...") {
                    return Some(());
                }
                // Keep a short tail: the marker may straddle two reads.
                if tail.len() > 256 {
                    let cut = tail.len() - 128;
                    let at = (cut..tail.len())
                        .find(|i| tail.is_char_boundary(*i))
                        .unwrap_or(tail.len());
                    tail.drain(..at);
                }
            }
        }
        if stop.load(Ordering::Relaxed) {
            return None;
        }
        thread::sleep(Duration::from_millis(40));
    }
}

/// The record the chain follows: the biggest still-reachable one, else the biggest
/// definitely-lost one.
fn top_record(check: &LeakCheck) -> Option<&LossRecord> {
    let biggest = |kind: &str| {
        check
            .records
            .iter()
            .filter(|r| r.kind == kind)
            .max_by_key(|r| r.bytes)
    };
    biggest("still-reachable").or_else(|| biggest("definitely-lost"))
}

fn answer_file(pid: u32, command: &str, text: &str) -> String {
    let mut s = format!("sending command {command} to pid {pid}\n");
    s.push_str(text);
    if !text.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// One walk of `who_points_at` from `(start, size)`. Returns the answers' text (with their
/// command lines) and whether it ended in a root or in an unreferenced block (a cycle, a
/// failed call or an exhausted budget are not).
fn walk(
    ask: &mut dyn FnMut(&str) -> Result<String, String>,
    pid: u32,
    first: (u64, u64),
    calls: &mut usize,
    cap: usize,
    notes: &mut Vec<String>,
) -> (String, bool) {
    let mut text = String::new();
    let mut target = first;
    let mut seen: Vec<u64> = vec![first.0];
    while *calls < cap {
        let cmd = format!("who_points_at 0x{:x} {}", target.0, target.1);
        *calls += 1;
        let answer = match ask(&cmd) {
            Ok(a) => a,
            Err(e) => {
                notes.push(e);
                return (text, false);
            }
        };
        text.push_str(&answer_file(pid, &cmd, &answer));
        let parsed = match vgdb::parse_who_points_at(&answer) {
            Ok(p) => p,
            Err(e) => {
                notes.push(format!("{cmd}: {}", e.message()));
                return (text, false);
            }
        };
        match vgdb::step(&parsed) {
            Step::Root(_) | Step::Unreferenced => return (text, true),
            Step::Block { start, holder } => {
                let size = match holder {
                    Holder::Block { size, .. } => size,
                    _ => target.1,
                };
                if seen.contains(&start) || start == target.0 {
                    // Not a root: the caller still tries the other start.
                    notes.push("the blocks point at each other in a cycle".to_owned());
                    return (text, false);
                }
                seen.push(start);
                target = (start, size);
            }
        }
    }
    (text, false)
}

/// Margin left to the point's timeout after the chain (detach, release, the exit itself).
const TIMEOUT_MARGIN: Duration = Duration::from_secs(5);

/// The chain's budget: [`BUDGET`], or less when the point's timeout comes sooner.
fn chain_budget(remaining: Duration) -> Duration {
    BUDGET.min(remaining.saturating_sub(TIMEOUT_MARGIN))
}

/// Runs the chain against the stopped process `pid`; writes the answer files to `plan.dir`.
///
/// `remaining` is what the point's timeout still allows once the process is running; the
/// chain budget is capped by it (less a margin) so the kill at the timeout does not land in
/// the middle of the chain.
pub(super) fn drive(
    plan: &VgdbPlan,
    pid: u32,
    stop: &AtomicBool,
    remaining: impl FnOnce() -> Duration,
) -> VgdbOutcome {
    let mut out = VgdbOutcome::default();
    if wait_for_stop(&plan.log, stop).is_none() {
        out.notes.push(
            "the program ended before Valgrind stopped at exit, so no vgdb answer was taken"
                .to_owned(),
        );
        return out;
    }
    let started = Instant::now();
    let budget = chain_budget(remaining());
    let deadline = started + budget;
    let mut session = match Session::open(plan, pid, deadline) {
        Ok(s) => s,
        Err(e) => {
            out.notes.push(format!("vgdb chain: {e}"));
            release(plan, pid);
            return out;
        }
    };
    chain(&mut session, plan, pid, &mut out);
    if Instant::now() >= deadline {
        out.notes.push(format!(
            "the vgdb chain stopped after {} s; the answers so far are kept",
            budget.as_secs()
        ));
    }
    session.detach();
    // The relay is gone; if the process is still stopped, let it go.
    thread::sleep(Duration::from_millis(100));
    if !stop.load(Ordering::Relaxed) && pid_waits(&plan.log) {
        release(plan, pid);
    }
    out
}

/// Whether the log still ends in the stop message with nothing after it (the process did not
/// go on after the relay detached).
fn pid_waits(log: &Path) -> bool {
    let Ok(mut f) = fs::File::open(log) else {
        return false;
    };
    let len = f.metadata().map_or(0, |m| m.len());
    if f.seek(SeekFrom::Start(len.saturating_sub(512))).is_err() {
        return false;
    }
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).is_ok()
        && String::from_utf8_lossy(&buf)
            .trim_end()
            .ends_with("vgdb me ...")
}

fn write_file(plan: &VgdbPlan, name: &str, text: &str, out: &mut VgdbOutcome) {
    let path = plan.dir.join(name);
    match fs::write(&path, text) {
        Ok(()) => out.files.push(path),
        Err(e) => out
            .notes
            .push(format!("cannot write {}: {e}", path.display())),
    }
}

fn chain(session: &mut Session, plan: &VgdbPlan, pid: u32, out: &mut VgdbOutcome) {
    let cmd = "leak_check full reachable any";
    let text = match session.command(cmd) {
        Ok(t) => t,
        Err(e) => {
            out.notes.push(format!("vgdb chain: {e}"));
            return;
        }
    };
    write_file(plan, "vgdb-1-leak.txt", &answer_file(pid, cmd, &text), out);
    let check = match vgdb::parse_leak_check(&text) {
        Ok(c) => c,
        Err(e) => {
            out.notes
                .push(format!("leak_check answer not read: {}", e.message()));
            return;
        }
    };
    let Some(record) = top_record(&check) else {
        out.notes
            .push("no still-reachable or definitely-lost record, so no block was followed".into());
        return;
    };
    let cmd = format!("block_list {}", record.nr);
    let text = match session.command(&cmd) {
        Ok(t) => t,
        Err(e) => {
            out.notes.push(format!("vgdb chain: {e}"));
            return;
        }
    };
    write_file(
        plan,
        "vgdb-2-blocks.txt",
        &answer_file(pid, &cmd, &text),
        out,
    );
    let blocks = match vgdb::parse_block_list(&text) {
        Ok(b) => b.blocks,
        Err(e) => {
            out.notes
                .push(format!("block_list answer not read: {}", e.message()));
            return;
        }
    };
    let (Some(first), Some(last)) = (blocks.first(), blocks.last()) else {
        out.notes.push("block_list named no block".to_owned());
        return;
    };
    follow(&mut |c| session.command(c), plan, pid, (*first, *last), out);
}

/// The `who_points_at` walks over the listed blocks (`first` is the oldest, `last` the newest).
fn follow(
    ask: &mut dyn FnMut(&str) -> Result<String, String>,
    plan: &VgdbPlan,
    pid: u32,
    (first, last): (BlockRef, BlockRef),
    out: &mut VgdbOutcome,
) {
    // The newest allocation first: the head of a list that grows at the front. If that walk
    // does not end in a root, the oldest block, with the calls that are left. Only the walk
    // that ends is kept, so the files hold one chain.
    let mut calls = 0usize;
    let mut best = String::new();
    let mut rooted = false;
    let starts = if first.address != last.address {
        vec![last, first]
    } else {
        vec![last]
    };
    for (i, b) in starts.iter().enumerate() {
        let cap = if i == 0 && starts.len() > 1 {
            MAX_CALLS - MAX_CALLS / 3
        } else {
            MAX_CALLS
        };
        let (text, done) = walk(
            ask,
            pid,
            (b.address, b.size),
            &mut calls,
            cap,
            &mut out.notes,
        );
        if done || best.is_empty() {
            best = text;
            rooted = done;
        }
        if done {
            break;
        }
    }
    out.calls = calls;
    out.rooted = rooted;
    if !best.is_empty() {
        write_file(plan, "vgdb-3-who.txt", &best, out);
    }
    if !rooted {
        out.notes.push(format!(
            "the who_points_at walk used its {MAX_CALLS} calls without reaching a data symbol or a stack; the chain is partial"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packets_carry_the_checksum_of_the_payload() {
        assert_eq!(packet("D"), b"$D#44");
        assert_eq!(packet("OK"), b"$OK#9a");
    }

    #[test]
    fn replies_split_into_output_and_ok() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"+$O");
        buf.extend_from_slice(hex_encode("==1== hi\n").as_bytes());
        buf.extend_from_slice(b"#00$OK#9a$O3");
        let got = take_packets(&mut buf);
        assert_eq!(got, vec![Reply::Output(b"==1== hi\n".to_vec()), Reply::Ok]);
        // The cut packet stays for the next read.
        assert_eq!(buf, b"$O3");
        buf.extend_from_slice(b"0#00");
        assert_eq!(take_packets(&mut buf), vec![Reply::Output(b"0".to_vec())]);
        assert!(buf.is_empty());
    }

    #[test]
    fn run_length_coding_is_undone() {
        assert_eq!(unrle("2c3*!92c"), "2c3333392c");
        let mut buf = b"$O2c3*!92c#00".to_vec();
        assert_eq!(
            take_packets(&mut buf),
            vec![Reply::Output(vec![0x2c, 0x33, 0x33, 0x39, 0x2c])]
        );
    }

    #[test]
    fn an_error_packet_is_an_error() {
        let mut buf = b"$E01#a6".to_vec();
        assert_eq!(take_packets(&mut buf), vec![Reply::Error("E01".into())]);
    }

    #[test]
    fn hex_round_trips() {
        let s = "leak_check full reachable any";
        assert_eq!(hex_decode(&hex_encode(s)).unwrap(), s.as_bytes());
        assert!(hex_decode("abc").is_none());
    }

    #[test]
    fn the_top_record_is_the_biggest_reachable_else_lost() {
        let rec = |nr: u32, kind: &str, bytes: u64| LossRecord {
            nr,
            of: 3,
            kind: kind.to_owned(),
            bytes,
            ..LossRecord::default()
        };
        let check = LeakCheck {
            records: vec![
                rec(1, "definitely-lost", 900),
                rec(2, "still-reachable", 10),
                rec(3, "still-reachable", 70),
            ],
        };
        assert_eq!(top_record(&check).unwrap().nr, 3);
        let check = LeakCheck {
            records: vec![rec(1, "definitely-lost", 5), rec(2, "definitely-lost", 9)],
        };
        assert_eq!(top_record(&check).unwrap().nr, 2);
        assert!(top_record(&LeakCheck::default()).is_none());
    }

    #[test]
    fn the_flags_name_the_prefix() {
        let f = valgrind_flags(Path::new("/r/tmp/vgdb/vg"));
        assert!(f.contains(&"--vgdb-stop-at=exit".to_owned()));
        assert!(f.contains(&"--vgdb-prefix=/r/tmp/vgdb/vg".to_owned()));
    }

    fn plan_in(dir: &Path) -> VgdbPlan {
        VgdbPlan {
            vgdb: PathBuf::from("vgdb"),
            prefix: dir.join("vg"),
            log: dir.join("run.log"),
            dir: dir.to_path_buf(),
        }
    }

    fn who(target: u64, holder: &str) -> String {
        format!(
            "Searching for pointers to 0x{target:x}\n*0x9000 points at 0x{target:x}\n   Address 0x9000 is {holder}\n"
        )
    }

    fn block(address: u64) -> BlockRef {
        BlockRef { address, size: 16 }
    }

    #[test]
    fn a_cycle_is_not_a_root_and_the_oldest_block_is_tried() {
        let d = tempfile::tempdir().unwrap();
        let plan = plan_in(d.path());
        // 0xa0 (newest) and 0xb0 point at each other; 0xc0 (oldest) is held by a data symbol.
        let mut asked: Vec<String> = Vec::new();
        let mut ask = |c: &str| -> Result<String, String> {
            asked.push(c.to_owned());
            let inside = |_a: u64| "0 bytes inside a block of size 16 alloc'd".to_string();
            Ok(if c.contains("0xa0 ") {
                who(0xa0, &inside(0xb0)).replace("0x9000 is", "0xb0 is")
            } else if c.contains("0xb0 ") {
                who(0xb0, &inside(0xa0)).replace("0x9000 is", "0xa0 is")
            } else {
                who(0xc0, "0 bytes inside data symbol \"registry\"")
            })
        };
        let mut out = VgdbOutcome::default();
        follow(&mut ask, &plan, 1, (block(0xc0), block(0xa0)), &mut out);
        assert!(
            asked.iter().any(|c| c.contains("0xc0 ")),
            "the fallback start was not tried: {asked:?}"
        );
        assert!(out.rooted, "{:?}", out.notes);
        let kept = fs::read_to_string(d.path().join("vgdb-3-who.txt")).unwrap();
        assert!(
            kept.contains("registry") && !kept.contains("0xb0 16"),
            "{kept}"
        );
    }

    #[test]
    fn a_cycle_with_no_root_anywhere_is_partial() {
        let d = tempfile::tempdir().unwrap();
        let plan = plan_in(d.path());
        let mut ask = |c: &str| -> Result<String, String> {
            let (t, o) = if c.contains("0xa0 ") {
                (0xa0, 0xb0)
            } else {
                (0xb0, 0xa0)
            };
            Ok(who(t, "0 bytes inside a block of size 16 alloc'd")
                .replace("0x9000 is", &format!("0x{o:x} is")))
        };
        let mut out = VgdbOutcome::default();
        follow(&mut ask, &plan, 1, (block(0xb0), block(0xa0)), &mut out);
        assert!(!out.rooted);
        assert!(
            out.notes.iter().any(|n| n.contains("cycle")),
            "{:?}",
            out.notes
        );
        assert!(
            out.notes.iter().any(|n| n.contains("partial")),
            "{:?}",
            out.notes
        );
    }

    #[test]
    fn the_chain_budget_is_capped_by_the_remaining_timeout() {
        assert_eq!(chain_budget(Duration::from_secs(600)), BUDGET);
        assert_eq!(
            chain_budget(Duration::from_secs(20)),
            Duration::from_secs(15)
        );
        assert_eq!(chain_budget(Duration::from_secs(3)), Duration::ZERO);
    }

    #[test]
    fn the_stop_is_found_by_reading_only_new_bytes_and_survives_non_utf8() {
        let d = tempfile::tempdir().unwrap();
        let log = d.path().join("run.log");
        let mut f = fs::File::create(&log).unwrap();
        f.write_all(b"\xff\xfe noise vgdb m").unwrap();
        f.flush().unwrap();
        let stop = std::sync::Arc::new(AtomicBool::new(false));
        let (l2, s2) = (log.clone(), stop.clone());
        let t = thread::spawn(move || wait_for_stop(&l2, &s2));
        thread::sleep(Duration::from_millis(120));
        f.write_all(b"e ...\n").unwrap();
        f.flush().unwrap();
        assert_eq!(t.join().unwrap(), Some(()));
        assert!(pid_waits(&log));
        f.write_all(b"\xff more output\n").unwrap();
        assert!(!pid_waits(&log));
    }
}
