//! A minimal blocking HTTP/1.1 GET over `std::net`, enough for local pprof and inspector
//! endpoints. No TLS, no redirects, no keep-alive.

use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::{Duration, Instant},
};

/// Largest response body accepted.
const MAX_BODY: usize = 32 * 1024 * 1024;

/// A parsed response.
#[derive(Debug, PartialEq, Eq)]
pub struct Response {
    /// Status code.
    pub status: u16,
    /// Decoded body (chunked encoding removed).
    pub body: Vec<u8>,
}

impl Response {
    /// The body as lossy UTF-8.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

/// Connects to `host:port` within `timeout`.
pub fn connect(host: &str, port: u16, timeout: Duration) -> Result<TcpStream, String> {
    let addrs = (host, port)
        .to_socket_addrs()
        .map_err(|e| format!("cannot resolve {host}: {e}"))?;
    let mut last = format!("no address for {host}");
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(s) => {
                let _ = s.set_read_timeout(Some(timeout));
                let _ = s.set_write_timeout(Some(timeout));
                let _ = s.set_nodelay(true);
                return Ok(s);
            }
            Err(e) => last = format!("cannot connect to {host}:{port}: {e}"),
        }
    }
    Err(last)
}

/// How close to the deadline a read timeout counts as the deadline's.
const SLACK: Duration = Duration::from_millis(20);

/// True for the errors a socket read timeout produces: `TimedOut` on Linux and Windows,
/// `WouldBlock` (EAGAIN) on macOS and the BSDs.
pub fn is_timeout(e: &std::io::Error) -> bool {
    matches!(
        e.kind(),
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
    )
}

/// Sets both socket timeouts to the smaller of `timeout` and the time left until `deadline`.
/// Returns false when the deadline has passed.
pub fn arm(stream: &TcpStream, timeout: Duration, deadline: Instant) -> bool {
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        return false;
    }
    let t = left.min(timeout);
    let _ = stream.set_read_timeout(Some(t));
    let _ = stream.set_write_timeout(Some(t));
    true
}

/// `GET path` from `host:port`. `timeout` bounds each socket operation; `deadline` bounds the
/// whole exchange, so a peer that sends one byte at a time cannot hold the caller.
pub fn get(
    host: &str,
    port: u16,
    path: &str,
    timeout: Duration,
    deadline: Instant,
) -> Result<Response, String> {
    let left = deadline.saturating_duration_since(Instant::now());
    if left.is_zero() {
        return Err(format!("no time left to reach {host}:{port}"));
    }
    let mut stream = connect(host, port, timeout.min(left))?;
    let expired = || format!("reading {host}:{port}{path} took longer than the allowed time");
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {host}:{port}\r\nConnection: close\r\nAccept: */*\r\nUser-Agent: dla-sample\r\n\r\n"
    );
    if !arm(&stream, timeout, deadline) {
        return Err(expired());
    }
    stream
        .write_all(request.as_bytes())
        .map_err(|e| format!("request to {host}:{port} failed: {e}"))?;
    let mut raw = Vec::new();
    let mut chunk = [0u8; 16 * 1024];
    loop {
        if !arm(&stream, timeout, deadline) {
            return Err(expired());
        }
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                raw.extend_from_slice(&chunk[..n]);
                if raw.len() > MAX_BODY + 64 * 1024 {
                    return Err(format!("response from {host}:{port} is over 32 MiB"));
                }
            }
            // A read timeout that the overall deadline caused is an error, whatever was
            // read so far: the body is incomplete.
            Err(e) if is_timeout(&e) && Instant::now() + SLACK >= deadline => {
                return Err(expired());
            }
            // A per-read timeout after the headers arrived still leaves a usable partial body
            // for servers that do not close; treat it as the end only when something was read.
            Err(e) if raw.is_empty() => {
                return Err(format!("reading {host}:{port}{path} failed: {e}"));
            }
            Err(_) => break,
        }
    }
    parse_response(&raw)
}

/// Splits a raw HTTP/1.1 response into status and decoded body.
pub fn parse_response(raw: &[u8]) -> Result<Response, String> {
    let split = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or("response has no header terminator")?;
    let head = String::from_utf8_lossy(&raw[..split]).into_owned();
    let mut lines = head.lines();
    let status_line = lines.next().unwrap_or_default();
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| format!("bad status line {status_line:?}"))?;
    let chunked = lines.any(|l| {
        let l = l.to_ascii_lowercase();
        l.starts_with("transfer-encoding:") && l.contains("chunked")
    });
    let body = &raw[split + 4..];
    let body = if chunked {
        decode_chunked(body)?
    } else {
        body.to_vec()
    };
    Ok(Response { status, body })
}

fn decode_chunked(mut data: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    loop {
        let eol = data
            .windows(2)
            .position(|w| w == b"\r\n")
            .ok_or("truncated chunked body")?;
        let size_text = String::from_utf8_lossy(&data[..eol]).into_owned();
        let size_text = size_text.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_text, 16)
            .map_err(|_| format!("bad chunk size {size_text:?}"))?;
        data = &data[eol + 2..];
        if size == 0 {
            return Ok(out);
        }
        if data.len() < size {
            // A cut-off final chunk: keep what arrived.
            out.extend_from_slice(data);
            return Ok(out);
        }
        out.extend_from_slice(&data[..size]);
        data = &data[size..];
        data = data.strip_prefix(b"\r\n").unwrap_or(data);
    }
}

/// A parsed `http://host[:port][/prefix]` base URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseUrl {
    /// Host name or address.
    pub host: String,
    /// Port (80 when absent).
    pub port: u16,
    /// Path prefix without a trailing slash (may be empty).
    pub prefix: String,
}

/// Parses an `http://` base URL. `https://` is refused: there is no TLS client here.
pub fn parse_base_url(url: &str) -> Result<BaseUrl, String> {
    let rest = if let Some(r) = url.strip_prefix("http://") {
        r
    } else if url.starts_with("https://") {
        return Err("https is not supported (no TLS client); use an http:// address".to_owned());
    } else {
        return Err(format!("{url:?} is not an http:// address"));
    };
    let (authority, path) = rest.split_once('/').map_or((rest, ""), |(a, p)| (a, p));
    let bad_port = |p: &str| format!("bad port {p:?} in {url:?}");
    let (host, port) = if let Some(inner) = authority.strip_prefix('[') {
        let (h, rest) = inner
            .split_once(']')
            .ok_or_else(|| format!("unclosed [ in {url:?}"))?;
        match rest.strip_prefix(':') {
            Some(p) => (h, p.parse().map_err(|_| bad_port(p))?),
            None => (h, 80),
        }
    } else {
        match authority.rsplit_once(':') {
            Some((h, p)) => (h, p.parse().map_err(|_| bad_port(p))?),
            None => (authority, 80),
        }
    };
    if host.is_empty() {
        return Err(format!("no host in {url:?}"));
    }
    Ok(BaseUrl {
        host: host.to_owned(),
        port,
        prefix: format!("/{path}").trim_end_matches('/').to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_and_chunked_responses() {
        let r = parse_response(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\n\r\nhello").unwrap();
        assert_eq!((r.status, r.text().as_str()), (200, "hello"));
        let r = parse_response(
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n6;x=1\r\n world\r\n0\r\n\r\n",
        )
        .unwrap();
        assert_eq!(r.text(), "hello world");
        let r = parse_response(b"HTTP/1.1 404 Not Found\r\n\r\nnope").unwrap();
        assert_eq!(r.status, 404);
        assert!(parse_response(b"garbage").is_err());
    }

    #[test]
    fn parses_base_urls() {
        let u = parse_base_url("http://127.0.0.1:6060").unwrap();
        assert_eq!(
            (u.host.as_str(), u.port, u.prefix.as_str()),
            ("127.0.0.1", 6060, "")
        );
        let u = parse_base_url("http://localhost/app/").unwrap();
        assert_eq!((u.port, u.prefix.as_str()), (80, "/app"));
        let u = parse_base_url("http://[::1]:9000").unwrap();
        assert_eq!((u.host.as_str(), u.port), ("::1", 9000));
        assert!(parse_base_url("https://x").unwrap_err().contains("https"));
        assert!(parse_base_url("ftp://x").is_err());
        assert!(parse_base_url("http://h:notaport").is_err());
    }

    #[test]
    fn gets_from_a_local_server() {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let h = std::thread::spawn(move || {
            let (mut s, _) = l.accept().unwrap();
            let mut buf = [0u8; 1024];
            let n = s.read(&mut buf).unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).into_owned();
            assert!(req.starts_with("GET /x?y=1 HTTP/1.1\r\n"));
            s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
                .unwrap();
        });
        let t = Duration::from_secs(5);
        let r = get("127.0.0.1", port, "/x?y=1", t, Instant::now() + t).unwrap();
        assert_eq!(r.text(), "ok");
        h.join().unwrap();
    }

    #[test]
    fn a_dripping_peer_hits_the_overall_deadline() {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let h = std::thread::spawn(move || {
            let (mut s, _) = l.accept().unwrap();
            let mut buf = [0u8; 1024];
            let _ = s.read(&mut buf);
            let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\n\r\n");
            // One byte every 100 ms: every read is well inside the per-read timeout.
            for _ in 0..100 {
                if s.write_all(b"x").is_err() {
                    return;
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        let started = Instant::now();
        let e = get(
            "127.0.0.1",
            port,
            "/",
            Duration::from_secs(5),
            Instant::now() + Duration::from_millis(600),
        )
        .unwrap_err();
        assert!(e.contains("longer than the allowed"), "{e}");
        assert!(started.elapsed() < Duration::from_secs(3));
        h.join().unwrap();
    }
}
