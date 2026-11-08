//! The `node-inspect` backend: Runtime.evaluate over the Node.js inspector websocket.
//!
//! Columns: `node.heap_used`, `node.heap_total`, `node.external`, `node.array_buffers` from
//! `process.memoryUsage()` and `node.active.<Type>` (a count per type) from
//! `process.getActiveResourcesInfo()`. The evaluation runs a read-only expression in the
//! target; it creates no objects that outlive the call.
//!
//! The inspector port comes from `--counters node-inspect=<port>`, from `--inspect[=host:port]`
//! in the command line, or from the TCP ports the process listens on (so `--inspect=0`
//! works on Linux). When there is none and the process is a Node.js binary, SIGUSR1 starts the
//! inspector on port 9229 (Unix only).

use std::{
    io::{self, Read, Write},
    net::TcpStream,
    thread,
    time::{Duration, Instant},
};

use serde_json::{Value, json};
use tungstenite::{Message, WebSocket};

use crate::{
    backend::{Backend, BackendError, OpenCtx, Reading},
    http, proc,
};

const IO_TIMEOUT: Duration = Duration::from_secs(5);
const PROBE_TIMEOUT: Duration = Duration::from_millis(800);
/// How long to wait for the inspector after SIGUSR1.
const SIGNAL_WAIT: Duration = Duration::from_secs(5);
/// The inspector port SIGUSR1 opens by default.
const DEFAULT_PORT: u16 = 9229;
/// What the inspector evaluates in the target on each poll.
pub const EXPRESSION: &str = "JSON.stringify({m: process.memoryUsage(), r: typeof process.getActiveResourcesInfo === 'function' ? process.getActiveResourcesInfo() : [], p: process.pid})";

/// The inspector endpoint named by `--inspect`, `--inspect=PORT`, `--inspect=HOST:PORT`
/// (also `--inspect-brk`, `--inspect-wait`, `--inspect-port`). Port 0 means a random port
/// and yields `None` for the port.
pub fn inspect_flag(args: &[String]) -> Option<(String, Option<u16>)> {
    for a in args {
        let Some(rest) = [
            "--inspect-brk",
            "--inspect-wait",
            "--inspect-port",
            "--inspect",
        ]
        .iter()
        .find_map(|p| a.strip_prefix(p)) else {
            continue;
        };
        let value = rest.strip_prefix('=');
        let Some(v) = value.filter(|v| !v.is_empty()) else {
            return Some(("127.0.0.1".to_owned(), Some(DEFAULT_PORT)));
        };
        let (host, port) = match v.rsplit_once(':') {
            Some((h, p)) => (h.trim_matches(['[', ']']), p),
            None if v.bytes().all(|b| b.is_ascii_digit()) => ("127.0.0.1", v),
            None => (v, "9229"),
        };
        let host = if host.is_empty() || host == "0.0.0.0" || host == "::" {
            "127.0.0.1"
        } else {
            host
        };
        let port = port.parse::<u16>().ok().filter(|p| *p != 0);
        return Some((host.to_owned(), port));
    }
    None
}

/// Memory and resource figures from the evaluated JSON string.
pub fn parse_evaluation(json_text: &str) -> Result<(Reading, Option<u32>), String> {
    let v: Value =
        serde_json::from_str(json_text).map_err(|e| format!("inspector value is not JSON: {e}"))?;
    let mut reading = Reading::default();
    for (key, column) in [
        ("heapUsed", "node.heap_used"),
        ("heapTotal", "node.heap_total"),
        ("external", "node.external"),
        ("arrayBuffers", "node.array_buffers"),
    ] {
        if let Some(n) = v["m"][key].as_f64() {
            reading.values.insert(column.to_owned(), n);
        }
    }
    if reading.values.is_empty() {
        return Err("inspector value has no process.memoryUsage() figures".to_owned());
    }
    if let Some(types) = v["r"].as_array() {
        for t in types.iter().filter_map(Value::as_str) {
            *reading
                .values
                .entry(format!("node.active.{t}"))
                .or_insert(0.0) += 1.0;
        }
    }
    let pid = v["p"].as_u64().and_then(|p| u32::try_from(p).ok());
    Ok((reading, pid))
}

/// The `result.result.value` string of a `Runtime.evaluate` response, or the error.
pub fn evaluation_value(response: &str) -> Result<String, String> {
    let v: Value = serde_json::from_str(response).map_err(|e| e.to_string())?;
    if let Some(msg) = v["error"]["message"].as_str() {
        return Err(format!("inspector error: {msg}"));
    }
    if let Some(ex) = v["result"]["exceptionDetails"]["text"].as_str() {
        return Err(format!("evaluation threw: {ex}"));
    }
    v["result"]["result"]["value"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "evaluation returned no string value".to_owned())
}

/// `ws://` URL for the first target in a `/json/list` body, rebuilt on `host:port` (the
/// inspector echoes the `Host` header, which may be wrong from our side).
pub fn debugger_path(list_body: &str) -> Option<String> {
    let v: Value = serde_json::from_str(list_body).ok()?;
    let url = v
        .as_array()?
        .iter()
        .find_map(|t| t["webSocketDebuggerUrl"].as_str())?;
    let rest = url.strip_prefix("ws://")?;
    let path = rest.find('/').map_or("/", |i| &rest[i..]);
    Some(path.to_owned())
}

/// A socket whose every read and write is bounded by one absolute deadline, so a peer that
/// drips bytes cannot stretch the websocket handshake or a reply past it.
struct DeadlineStream {
    inner: TcpStream,
    deadline: Instant,
}

impl DeadlineStream {
    fn arm(&self) -> io::Result<()> {
        if http::arm(&self.inner, IO_TIMEOUT, self.deadline) {
            Ok(())
        } else {
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "the inspector did not answer in time",
            ))
        }
    }
}

impl DeadlineStream {
    /// A socket timeout is reported as `TimedOut` on every platform (macOS gives `WouldBlock`).
    fn timed(r: io::Result<usize>) -> io::Result<usize> {
        r.map_err(|e| {
            if http::is_timeout(&e) {
                io::Error::new(io::ErrorKind::TimedOut, e)
            } else {
                e
            }
        })
    }
}

impl Read for DeadlineStream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.arm()?;
        Self::timed(self.inner.read(buf))
    }
}

impl Write for DeadlineStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.arm()?;
        Self::timed(self.inner.write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

struct NodeInspect {
    ws: WebSocket<DeadlineStream>,
    pid: u32,
    next_id: u64,
}

impl std::fmt::Debug for NodeInspect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NodeInspect")
            .field("pid", &self.pid)
            .finish()
    }
}

/// The earlier of `limit` and [`IO_TIMEOUT`] from now.
fn io_deadline(limit: Instant) -> Instant {
    limit.min(Instant::now() + IO_TIMEOUT)
}

fn connect(
    host: &str,
    port: u16,
    timeout: Duration,
    limit: Instant,
) -> Result<WebSocket<DeadlineStream>, String> {
    let list = http::get(host, port, "/json/list", timeout, io_deadline(limit))?;
    if list.status != 200 {
        return Err(format!("{host}:{port} /json/list answered {}", list.status));
    }
    let path = debugger_path(&list.text())
        .ok_or_else(|| format!("{host}:{port} lists no inspectable target"))?;
    let deadline = io_deadline(limit);
    let stream = http::connect(
        host,
        port,
        IO_TIMEOUT.min(deadline.saturating_duration_since(Instant::now())),
    )?;
    let stream = DeadlineStream {
        inner: stream,
        deadline,
    };
    let url = format!("ws://{host}:{port}{path}");
    let (ws, _) = tungstenite::client(url, stream)
        .map_err(|e| format!("websocket handshake with {host}:{port} failed: {e}"))?;
    Ok(ws)
}

impl NodeInspect {
    /// Evaluates [`EXPRESSION`]; the whole exchange must end by `deadline`.
    fn evaluate(&mut self, deadline: Instant) -> Result<(Reading, Option<u32>), String> {
        self.next_id += 1;
        let id = self.next_id;
        self.ws.get_mut().deadline = deadline;
        let request = json!({
            "id": id,
            "method": "Runtime.evaluate",
            "params": {"expression": EXPRESSION, "returnByValue": true, "silent": true},
        });
        self.ws
            .send(Message::text(request.to_string()))
            .map_err(|e| format!("sending to the inspector failed: {e}"))?;
        loop {
            if Instant::now() > deadline {
                return Err("the inspector did not answer in time (is the process paused?)".into());
            }
            let msg = self
                .ws
                .read()
                .map_err(|e| format!("reading the inspector failed: {e}"))?;
            let Message::Text(text) = msg else { continue };
            let Ok(v) = serde_json::from_str::<Value>(text.as_str()) else {
                continue;
            };
            if v["id"].as_u64() != Some(id) {
                continue;
            }
            return parse_evaluation(&evaluation_value(text.as_str())?);
        }
    }

    /// Connects to `host:port` and checks it is the inspector of `pid`.
    fn attach(host: &str, port: u16, pid: u32, limit: Instant) -> Result<Self, String> {
        let ws = connect(host, port, PROBE_TIMEOUT, limit)?;
        let mut me = Self {
            ws,
            pid,
            next_id: 0,
        };
        let (_, seen) = me.evaluate(io_deadline(limit))?;
        match seen {
            Some(p) if p != pid => Err(format!(
                "the inspector on {host}:{port} belongs to process {p}, not {pid}"
            )),
            _ => Ok(me),
        }
    }
}

fn looks_like_node(args: &[String]) -> bool {
    args.first()
        .is_some_and(|a| matches!(proc::exe_name(a), "node" | "nodejs"))
}

/// Finds the inspector of the target and returns the backend.
pub fn open(port: Option<u16>, ctx: &OpenCtx) -> Result<Box<dyn Backend>, BackendError> {
    let pid = ctx.pid;
    let (host, flag_port) = inspect_flag(&ctx.args).unwrap_or(("127.0.0.1".to_owned(), None));
    let mut errors: Vec<String> = Vec::new();
    let try_ports = |ports: &[u16], errors: &mut Vec<String>| -> Option<NodeInspect> {
        for &p in ports {
            match NodeInspect::attach(&host, p, pid, ctx.deadline) {
                Ok(n) => return Some(n),
                Err(e) => errors.push(e),
            }
        }
        None
    };
    let mut ports: Vec<u16> = port.into_iter().chain(flag_port).collect();
    if port.is_none() {
        ports.extend(proc::listening_ports(pid));
    }
    ports.dedup();
    if let Some(n) = try_ports(&ports, &mut errors) {
        return Ok(Box::new(n));
    }
    if port.is_some() {
        return Err(BackendError::blocked(format!(
            "node-inspect: {}",
            errors.last().cloned().unwrap_or_default()
        )));
    }
    // No inspector yet: ask a Node.js process to start one.
    if flag_port.is_none() && inspect_flag(&ctx.args).is_none() && looks_like_node(&ctx.args) {
        proc::signal_usr1(pid).map_err(|e| {
            BackendError::blocked(format!("node-inspect: SIGUSR1 to {pid} failed: {e}"))
        })?;
        let deadline = (Instant::now() + SIGNAL_WAIT).min(ctx.deadline);
        while Instant::now() < deadline {
            let mut candidates = vec![DEFAULT_PORT];
            candidates.extend(proc::listening_ports(pid));
            candidates.dedup();
            if let Some(n) = try_ports(&candidates, &mut errors) {
                return Ok(Box::new(n));
            }
            thread::sleep(Duration::from_millis(200));
        }
        return Err(BackendError::blocked(format!(
            "node-inspect: sent SIGUSR1 to {pid} but no inspector answered on port {DEFAULT_PORT} within {}s (the port may be taken; start node with --inspect=<port>)",
            SIGNAL_WAIT.as_secs()
        )));
    }
    let why = errors
        .last()
        .map_or_else(|| "no inspector port found".to_owned(), Clone::clone);
    Err(BackendError::blocked(format!(
        "node-inspect: {why}; start node with --inspect=<port> or pass --counters node-inspect=<port>"
    )))
}

impl Backend for NodeInspect {
    fn name(&self) -> &'static str {
        "node-inspect"
    }

    fn runtime(&self) -> &'static str {
        "node"
    }

    fn min_interval(&self) -> Duration {
        Duration::from_millis(100)
    }

    fn poll(&mut self) -> Result<Reading, BackendError> {
        self.evaluate(Instant::now() + IO_TIMEOUT)
            .map(|(r, _)| r)
            .map_err(|e| BackendError::blocked(format!("node-inspect: {e}")))
    }

    fn zero_fill_prefixes(&self) -> &'static [&'static str] {
        &["node.active."]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn inspect_flags() {
        assert_eq!(inspect_flag(&args(&["node", "a.js"])), None);
        assert_eq!(
            inspect_flag(&args(&["node", "--inspect", "a.js"])),
            Some(("127.0.0.1".into(), Some(9229)))
        );
        assert_eq!(
            inspect_flag(&args(&["node", "--inspect=9230", "a.js"])),
            Some(("127.0.0.1".into(), Some(9230)))
        );
        assert_eq!(
            inspect_flag(&args(&["node", "--inspect-brk=0.0.0.0:9300"])),
            Some(("127.0.0.1".into(), Some(9300)))
        );
        assert_eq!(
            inspect_flag(&args(&["node", "--inspect=0"])),
            Some(("127.0.0.1".into(), None))
        );
        assert_eq!(
            inspect_flag(&args(&["node", "--inspect=localhost:0"])),
            Some(("localhost".into(), None))
        );
    }

    #[test]
    fn evaluation_parsing() {
        let value = r#"{"m":{"rss":50000000,"heapTotal":6000000,"heapUsed":4500000,"external":1500000,"arrayBuffers":10500},"r":["Timeout","Timeout","TTYWrap","Immediate"],"p":4242}"#;
        let (r, pid) = parse_evaluation(value).unwrap();
        assert_eq!(pid, Some(4242));
        assert_eq!(r.values["node.heap_used"], 4_500_000.0);
        assert_eq!(r.values["node.heap_total"], 6_000_000.0);
        assert_eq!(r.values["node.external"], 1_500_000.0);
        assert_eq!(r.values["node.array_buffers"], 10_500.0);
        assert_eq!(r.values["node.active.Timeout"], 2.0);
        assert_eq!(r.values["node.active.TTYWrap"], 1.0);
        assert!(parse_evaluation("{}").is_err());
        assert!(parse_evaluation("nope").is_err());
    }

    #[test]
    fn evaluate_responses() {
        let ok = json!({"id": 1, "result": {"result": {"type": "string", "value": "{\"a\":1}"}}});
        assert_eq!(evaluation_value(&ok.to_string()).unwrap(), "{\"a\":1}");
        let threw = json!({"id": 1, "result": {"result": {"type": "object"}, "exceptionDetails": {"text": "Uncaught"}}});
        assert!(
            evaluation_value(&threw.to_string())
                .unwrap_err()
                .contains("Uncaught")
        );
        let err = json!({"id": 1, "error": {"code": -32601, "message": "not found"}});
        assert!(
            evaluation_value(&err.to_string())
                .unwrap_err()
                .contains("not found")
        );
    }

    #[test]
    fn a_dripping_inspector_hits_the_deadline() {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let h = thread::spawn(move || {
            let (mut s, _) = l.accept().unwrap();
            for _ in 0..100 {
                if s.write_all(b"x").is_err() {
                    return;
                }
                thread::sleep(Duration::from_millis(100));
            }
        });
        let mut s = DeadlineStream {
            inner: TcpStream::connect(("127.0.0.1", port)).unwrap(),
            deadline: Instant::now() + Duration::from_millis(500),
        };
        let started = Instant::now();
        let mut b = [0u8; 1];
        let err = loop {
            if let Err(e) = s.read(&mut b) {
                break e;
            }
        };
        assert_eq!(err.kind(), io::ErrorKind::TimedOut);
        assert!(started.elapsed() < Duration::from_secs(3));
        drop(s);
        h.join().unwrap();
    }

    #[test]
    fn debugger_url_from_the_target_list() {
        let list = r#"[ {"description": "node.js instance", "id": "bd4e", "title": "n.js", "type": "node",
            "webSocketDebuggerUrl": "ws://127.0.0.1:42693/bd4e597f-bf0c-42d4-89c7-ec477d633928"} ]"#;
        assert_eq!(
            debugger_path(list).as_deref(),
            Some("/bd4e597f-bf0c-42d4-89c7-ec477d633928")
        );
        assert_eq!(debugger_path("[]"), None);
        assert_eq!(debugger_path("not json"), None);
    }
}
