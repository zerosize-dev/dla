//! Node.js inspector operations: `process.memoryUsage()` and a streamed heap snapshot.
//!
//! The websocket client here is separate from the sampler's: a snapshot arrives as about a
//! thousand `HeapProfiler.addHeapSnapshotChunk` events and the main thread of the target is
//! blocked for seconds before the first one, so reads wait on an idle timeout and an overall
//! deadline instead of the sampler's few-second bound, and every chunk goes straight to the
//! file.

use std::{
    cell::RefCell,
    fs::File,
    io::{self, BufWriter, Read, Write},
    net::TcpStream,
    path::Path,
    thread,
    time::{Duration, Instant},
};

use serde::Deserialize;
use serde_json::{Map, Value, json};
use tungstenite::{Message, WebSocket, protocol::WebSocketConfig};

use super::{
    Backend, Detail, Done, Op, Plan, Request, Tracker,
    resources::Estimate,
    util::{AttachError, Cleanup, output_name},
};
use crate::{http, proc};

/// The inspector port `SIGUSR1` opens by default.
const DEFAULT_PORT: u16 = 9229;
/// How long a read may block with nothing arriving while the target works.
const IDLE: Duration = Duration::from_secs(5);
/// How long to wait for the inspector after `SIGUSR1`.
const SIGNAL_WAIT: Duration = Duration::from_secs(5);
const EXPRESSION: &str =
    "JSON.stringify({pid: process.pid, m: process.memoryUsage(), up: process.uptime()})";
const MIB: u64 = 1 << 20;

pub(crate) struct Prep {
    inspector: RefCell<Option<Inspector>>,
    port: u16,
    heap_used: u64,
    memory: Value,
}

struct Stream {
    inner: TcpStream,
    deadline: Instant,
}

impl Stream {
    fn arm(&self) -> io::Result<()> {
        if http::arm(&self.inner, IDLE, self.deadline) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "the inspector did not answer in time",
            ))
        }
    }
}

fn timed(r: io::Result<usize>) -> io::Result<usize> {
    r.map_err(|e| {
        if http::is_timeout(&e) {
            io::Error::new(io::ErrorKind::TimedOut, e)
        } else {
            e
        }
    })
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.arm()?;
        timed(self.inner.read(buf))
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.arm()?;
        timed(self.inner.write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

struct Inspector {
    ws: WebSocket<Stream>,
    next_id: u64,
}

#[derive(Deserialize)]
struct Incoming {
    id: Option<u64>,
    method: Option<String>,
    params: Option<ChunkParams>,
    result: Option<Value>,
    error: Option<Value>,
}

#[derive(Deserialize)]
struct ChunkParams {
    chunk: Option<String>,
}

fn ws_config() -> WebSocketConfig {
    let mut c = WebSocketConfig::default();
    // A chunk is normally about 100 KB; allow far more so one large string table is not a
    // protocol error.
    c.max_message_size = Some(512 << 20);
    c.max_frame_size = Some(256 << 20);
    c
}

impl Inspector {
    fn connect(host: &str, port: u16, deadline: Instant) -> Result<Self, String> {
        let list = http::get(
            host,
            port,
            "/json/list",
            Duration::from_millis(800),
            deadline,
        )?;
        if list.status != 200 {
            return Err(format!("{host}:{port} /json/list answered {}", list.status));
        }
        let path = crate::node::debugger_path(&list.text())
            .ok_or_else(|| format!("{host}:{port} lists no inspectable target"))?;
        let inner = http::connect(host, port, Duration::from_secs(3))?;
        let stream = Stream { inner, deadline };
        let (ws, _) = tungstenite::client::client_with_config(
            format!("ws://{host}:{port}{path}"),
            stream,
            Some(ws_config()),
        )
        .map_err(|e| format!("websocket handshake with {host}:{port} failed: {e}"))?;
        Ok(Self { ws, next_id: 0 })
    }

    /// Sends one command and waits for its answer; chunk events go to `on_chunk`.
    fn call(
        &mut self,
        method: &str,
        params: Value,
        deadline: Instant,
        on_chunk: &mut dyn FnMut(&str) -> io::Result<()>,
    ) -> Result<Value, String> {
        self.next_id += 1;
        let id = self.next_id;
        self.ws.get_mut().deadline = deadline;
        let request = json!({"id": id, "method": method, "params": params});
        self.ws
            .send(Message::text(request.to_string()))
            .map_err(|e| format!("sending {method} failed: {e}"))?;
        loop {
            let msg = match self.ws.read() {
                Ok(m) => m,
                Err(tungstenite::Error::Io(e))
                    if http::is_timeout(&e) || e.kind() == io::ErrorKind::TimedOut =>
                {
                    if Instant::now() >= deadline {
                        return Err(format!(
                            "the inspector did not finish {method} in time (is the process paused?)"
                        ));
                    }
                    continue;
                }
                Err(e) => return Err(format!("reading the inspector failed during {method}: {e}")),
            };
            let Message::Text(text) = msg else { continue };
            let Ok(m) = serde_json::from_str::<Incoming>(text.as_str()) else {
                continue;
            };
            if m.method.as_deref() == Some("HeapProfiler.addHeapSnapshotChunk") {
                if let Some(chunk) = m.params.and_then(|p| p.chunk) {
                    on_chunk(&chunk).map_err(|e| format!("writing the snapshot failed: {e}"))?;
                }
                continue;
            }
            if m.id != Some(id) {
                continue;
            }
            if let Some(err) = m.error {
                return Err(format!(
                    "{method} failed: {}",
                    err["message"].as_str().unwrap_or("inspector error")
                ));
            }
            return Ok(m.result.unwrap_or(Value::Null));
        }
    }

    /// The value of `expression` as a JSON string, evaluated in the target.
    fn evaluate(&mut self, expression: &str, deadline: Instant) -> Result<String, String> {
        let r = self.call(
            "Runtime.evaluate",
            json!({"expression": expression, "returnByValue": true, "silent": true}),
            deadline,
            &mut |_| Ok(()),
        )?;
        if let Some(ex) = r["exceptionDetails"]["text"].as_str() {
            return Err(format!("the evaluation threw: {ex}"));
        }
        r["result"]["value"]
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| "the evaluation returned no string".to_owned())
    }
}

fn is_node(req: &Request) -> bool {
    matches!(req.target.exe_name().as_str(), "node" | "nodejs")
}

/// Finds an inspector that belongs to the target. Returns it with the port and whether
/// `SIGUSR1` had to be sent.
fn open(req: &Request, deadline: Instant) -> Result<(Inspector, u16, bool, Value), AttachError> {
    let pid = req.target.pid;
    let flag = crate::node::inspect_flag(&req.target.args);
    let host = flag
        .as_ref()
        .map_or("127.0.0.1".to_owned(), |f| f.0.clone());
    let mut errors: Vec<String> = Vec::new();
    let mut try_ports = |ports: &[u16]| -> Option<(Inspector, u16, Value)> {
        for &p in ports {
            let attempt = Inspector::connect(&host, p, deadline).and_then(|mut i| {
                let text = i.evaluate(EXPRESSION, deadline)?;
                let v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                match v["pid"].as_u64() {
                    Some(seen) if seen != u64::from(pid) => Err(format!(
                        "the inspector on {host}:{p} belongs to process {seen}, not {pid}"
                    )),
                    _ => Ok((i, v["m"].clone())),
                }
            });
            match attempt {
                Ok((i, m)) => return Some((i, p, m)),
                Err(e) => errors.push(e),
            }
        }
        None
    };
    let mut ports: Vec<u16> = flag.and_then(|f| f.1).into_iter().collect();
    ports.extend(proc::listening_ports(pid));
    ports.dedup();
    if let Some((i, p, m)) = try_ports(&ports) {
        return Ok((i, p, false, m));
    }
    if !is_node(req) {
        let why = errors
            .last()
            .cloned()
            .unwrap_or_else(|| "no inspector port found".into());
        return Err(AttachError::not_ready(
            format!(
                "process {pid} has no reachable inspector ({why}) and is not a node or nodejs executable, so SIGUSR1 is not sent"
            ),
            "a Node.js process with an open inspector",
            "start the target with node --inspect=<port>, then dla attach node-inspector --pid <pid> --op memory",
        ));
    }
    proc::signal_usr1(pid).map_err(|e| {
        AttachError::not_ready(
            format!("SIGUSR1 to process {pid} failed: {e}"),
            "permission to signal the Node.js process",
            "run dla as the same user as the process",
        )
    })?;
    let until = (Instant::now() + SIGNAL_WAIT).min(deadline);
    while Instant::now() < until {
        let mut candidates = vec![DEFAULT_PORT];
        candidates.extend(proc::listening_ports(pid));
        candidates.dedup();
        if let Some((i, p, m)) = try_ports(&candidates) {
            return Ok((i, p, true, m));
        }
        thread::sleep(Duration::from_millis(200));
    }
    Err(AttachError::not_ready(
        format!(
            "sent SIGUSR1 to {pid} but no inspector answered on port {DEFAULT_PORT} within {}s (the port may be taken)",
            SIGNAL_WAIT.as_secs()
        ),
        "an inspector port the target listens on",
        "start the target with node --inspect=<port>, then repeat the command",
    ))
}

pub(crate) fn plan(req: &Request) -> Result<Plan, AttachError> {
    let ext = "heapsnapshot";
    let (stem, ext) = match req.op {
        Op::Memory => ("memory", "json"),
        _ => ("heap", ext),
    };
    let file = match &req.out_name {
        Some(n) => output_name(n, ext).map_err(|m| {
            AttachError::not_ready(
                m,
                "--out as a plain file name",
                "dla attach ... --out heap.heapsnapshot",
            )
        })?,
        None => format!("{stem}.{ext}"),
    };
    let (inspector, port, signalled, memory) = open(req, Instant::now() + Duration::from_secs(30))?;
    let heap_used = memory["heapUsed"].as_u64().unwrap_or(0);
    let pid = req.target.pid;
    let mut notes = Vec::new();
    let mut effects = Vec::new();
    if signalled {
        effects.push(format!(
            "SIGUSR1 was sent to {pid}: the inspector now listens on 127.0.0.1:{port}"
        ));
        notes.push(
            "the inspector had to be opened with SIGUSR1 to read heapUsed for the estimate"
                .to_owned(),
        );
    } else {
        effects.push(format!(
            "the inspector of {pid} on port {port} is used; no signal is sent"
        ));
    }
    let (estimate, signal, stopping) = if req.op == Op::Memory {
        effects.push("the target evaluates process.memoryUsage() (about 2 ms)".to_owned());
        (
            Estimate {
                dump_bytes: 4096,
                peak_bytes: MIB,
                basis: "one small JSON object".into(),
            },
            "rss, heapTotal, heapUsed, external and arrayBuffers of the process".to_owned(),
            "the evaluation returns".to_owned(),
        )
    } else {
        effects.push("the target builds a heap snapshot on its main thread, blocking it for seconds (5 s for a 66 MB heap) while no other inspector call is answered".to_owned());
        effects.push("the snapshot is streamed to a file by DLA; it holds object contents and may contain secrets".to_owned());
        (
            Estimate {
                dump_bytes: heap_used.saturating_mul(7) / 4,
                peak_bytes: heap_used.saturating_mul(12),
                basis: format!(
                    "heapUsed {heap_used} bytes from process.memoryUsage(): file up to 1.75x, peak extra memory about 12x for object-dense heaps (measured 828 MB over 66 MB; the earlier estimate of 2x was wrong)"
                ),
            },
            "a .heapsnapshot JSON file streamed in chunks".to_owned(),
            "the final HeapProfiler.takeHeapSnapshot reply arrives after the last chunk".to_owned(),
        )
    };
    Ok(Plan {
        backend: Backend::NodeInspector,
        op: req.op,
        tool: "Node.js inspector (websocket, Chrome DevTools protocol)".to_owned(),
        expected_signal: signal,
        stopping,
        effects,
        cleanup: vec![format!(
            "note the inspector of process {pid} stays open on port {port} until the process exits"
        )],
        estimate,
        notes,
        unverified: false,
        file,
        detail: Detail::Node(Box::new(Prep {
            inspector: RefCell::new(Some(inspector)),
            port,
            heap_used,
            memory,
        })),
    })
}

pub(crate) fn run(
    p: &Prep,
    plan: &Plan,
    req: &Request,
    out_dir: &Path,
    t: &mut Tracker<'_>,
) -> Result<Done, AttachError> {
    let pid = req.target.pid;
    t.note(format!(
        "the inspector of process {pid} stays open on port {} until the process exits",
        p.port
    ));
    let mut inspector = p.inspector.borrow_mut().take().ok_or_else(|| {
        AttachError::failed(
            "the inspector connection was already used",
            "repeat the command",
        )
    })?;
    let path = out_dir.join(&plan.file);
    let mut done = Done::default();
    let mut dur = Map::new();
    match req.op {
        Op::Memory => {
            let text = inspector
                .evaluate(EXPRESSION, Instant::now() + Duration::from_secs(20))
                .map_err(|e| AttachError::failed(e, "repeat the command"))?;
            let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
            let out = serde_json::to_string_pretty(&v).unwrap_or(text);
            std::fs::write(&path, out).map_err(|e| {
                AttachError::failed(
                    format!("cannot write {}: {e}", path.display()),
                    "check free disk",
                )
            })?;
            done.summary = json!({"memory": v["m"], "uptime_s": v["up"]});
        }
        _ => {
            t.need(Cleanup::Rm(path.clone()));
            let file = File::create(&path).map_err(|e| {
                AttachError::failed(
                    format!("cannot create {}: {e}", path.display()),
                    "check free disk",
                )
            })?;
            let mut w = BufWriter::with_capacity(1 << 20, file);
            let mut chunks = 0u64;
            let mut bytes = 0u64;
            let before = crate::os::sample(pid).ok();
            let t0 = Instant::now();
            // Generous: the main thread is blocked, and a big heap takes minutes.
            let deadline = Instant::now() + Duration::from_secs(900);
            let res = inspector.call(
                "HeapProfiler.takeHeapSnapshot",
                json!({"reportProgress": false, "captureNumericValue": false}),
                deadline,
                &mut |c| {
                    chunks += 1;
                    bytes += c.len() as u64;
                    w.write_all(c.as_bytes())
                },
            );
            let flushed = w.flush();
            drop(w);
            dur.insert("snapshot_ms".into(), json!(t0.elapsed().as_millis() as u64));
            if let Err(e) = res {
                let _ = std::fs::remove_file(&path);
                return Err(AttachError::failed(
                    format!("{e}; the partial snapshot was deleted"),
                    "repeat the command; a very large heap may need more time or memory",
                ));
            }
            flushed.map_err(|e| {
                AttachError::failed(
                    format!("cannot write {}: {e}", path.display()),
                    "check free disk",
                )
            })?;
            if !snapshot_looks_complete(&path) {
                return Err(AttachError::failed(
                    "the snapshot file is not a complete V8 heap snapshot",
                    "repeat the command",
                ));
            }
            t.keep(&Cleanup::Rm(path.clone()));
            let after = crate::os::sample(pid).ok();
            let col = |s: &Option<crate::os::OsSample>, c: &str| {
                s.as_ref().and_then(|s| s.values.get(c)).map(|v| *v as u64)
            };
            done.summary = json!({
                "bytes": bytes,
                "chunks": chunks,
                "heap_used_before": p.heap_used,
                "rss_before": col(&before, "os.rss"),
                "rss_after": col(&after, "os.rss"),
                "vm_hwm_after": col(&after, "os.vm_hwm"),
            });
            let _ = &p.memory;
        }
    }
    done.files.push(plan.file.clone());
    done.durations = dur;
    Ok(done)
}

/// A V8 snapshot is one JSON object that starts with `{"snapshot":` and ends with `}`.
pub(crate) fn snapshot_looks_complete(path: &Path) -> bool {
    use std::io::{Seek, SeekFrom};
    let Ok(mut f) = File::open(path) else {
        return false;
    };
    let mut head = [0u8; 12];
    if f.read_exact(&mut head).is_err() || &head != b"{\"snapshot\":" {
        return false;
    }
    let Ok(len) = f.seek(SeekFrom::End(0)) else {
        return false;
    };
    let from = len.saturating_sub(16);
    let mut tail = Vec::new();
    if f.seek(SeekFrom::Start(from)).is_err() || f.read_to_end(&mut tail).is_err() {
        return false;
    }
    tail.trim_ascii_end().last() == Some(&b'}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_snapshot_check() {
        let d = std::env::temp_dir().join(format!("dla-snap-{}", std::process::id()));
        std::fs::create_dir_all(&d).unwrap();
        let ok = d.join("ok");
        std::fs::write(&ok, "{\"snapshot\":{\"meta\":{}},\"nodes\":[]}\n").unwrap();
        assert!(snapshot_looks_complete(&ok));
        let cut = d.join("cut");
        std::fs::write(&cut, "{\"snapshot\":{\"meta\":{}},\"nodes\":[1,2").unwrap();
        assert!(!snapshot_looks_complete(&cut));
        let other = d.join("other");
        std::fs::write(&other, "hello world, not a snapshot").unwrap();
        assert!(!snapshot_looks_complete(&other));
        std::fs::remove_dir_all(&d).unwrap();
    }

    #[test]
    fn incoming_messages_decode_chunks_and_replies() {
        let c: Incoming = serde_json::from_str(
            r#"{"method":"HeapProfiler.addHeapSnapshotChunk","params":{"chunk":"abc"}}"#,
        )
        .unwrap();
        assert_eq!(c.params.unwrap().chunk.as_deref(), Some("abc"));
        let r: Incoming = serde_json::from_str(r#"{"id":4,"result":{}}"#).unwrap();
        assert_eq!(r.id, Some(4));
        assert!(r.error.is_none());
    }
}
