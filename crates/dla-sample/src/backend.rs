//! Runtime counter backends: what `--counters` can name, how `auto` chooses, and the
//! [`Backend`] trait the sampling [`session`](crate::session) drives.

use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use crate::{gopprof, jcmd, node, proc, pyremote};

/// One poll of a backend: column name to value, plus notes that belong in the limitations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Reading {
    /// Figures read now. Counts and bytes are plain numbers.
    pub values: BTreeMap<String, f64>,
    /// Facts about this reading that limit it (recorded once per distinct text).
    pub notes: Vec<String>,
}

/// Why a backend could not be opened or polled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendError {
    /// True when the tool the backend needs is not installed (coverage `missing`); false when
    /// it exists but could not run (coverage `blocked`).
    pub missing: bool,
    /// What went wrong, as one sentence fit for a limitation.
    pub message: String,
    /// How to install or enable what is missing.
    pub hint: Option<String>,
}

impl BackendError {
    /// A backend that exists but could not run.
    pub fn blocked(message: impl Into<String>) -> Self {
        Self {
            missing: false,
            message: message.into(),
            hint: None,
        }
    }

    /// A backend whose tool is not installed.
    pub fn missing(message: impl Into<String>, hint: impl Into<String>) -> Self {
        Self {
            missing: true,
            message: message.into(),
            hint: Some(hint.into()),
        }
    }
}

impl std::fmt::Display for BackendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// A source of runtime counters for one process.
pub trait Backend: Send {
    /// The name used in `--counters` and in coverage (`jcmd`, `go-pprof`, ...).
    fn name(&self) -> &'static str;
    /// The runtime worker the backend belongs to in coverage (`jvm`, `go`, `node`, ...).
    fn runtime(&self) -> &'static str;
    /// The shortest time between two polls.
    fn min_interval(&self) -> Duration;
    /// Reads the counters now.
    fn poll(&mut self) -> Result<Reading, BackendError>;
    /// Column prefixes of counts that are zero when the backend answered without them
    /// (a goroutine site that has no goroutines left).
    fn zero_fill_prefixes(&self) -> &'static [&'static str] {
        &[]
    }
}

/// A backend named on the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendSpec {
    /// `jcmd` against a JVM.
    Jcmd,
    /// `go-pprof=<base url>`.
    GoPprof(String),
    /// `node-inspect[=<port>]`.
    NodeInspect(Option<u16>),
    /// `python-remote`.
    PythonRemote,
    /// `tokio`: only readable in-process.
    Tokio,
}

/// Which family of backend a spec is (for status maps and coverage keys).
pub type BackendKind = &'static str;

impl BackendSpec {
    /// The backend name.
    pub fn name(&self) -> BackendKind {
        match self {
            Self::Jcmd => "jcmd",
            Self::GoPprof(_) => "go-pprof",
            Self::NodeInspect(_) => "node-inspect",
            Self::PythonRemote => "python-remote",
            Self::Tokio => "tokio",
        }
    }

    /// The runtime worker key for coverage.
    pub fn runtime(&self) -> &'static str {
        match self {
            Self::Jcmd => "jvm",
            Self::GoPprof(_) => "go",
            Self::NodeInspect(_) => "node",
            Self::PythonRemote => "python",
            Self::Tokio => "rust",
        }
    }
}

/// The parsed `--counters` value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CountersSpec {
    /// Choose by the process.
    Auto,
    /// OS columns only.
    None,
    /// Exactly these backends.
    List(Vec<BackendSpec>),
}

/// Parses `auto`, `none` or a comma-separated list of `jcmd`, `go-pprof=<url>`,
/// `node-inspect[=<port>]`, `python-remote` and `tokio`.
pub fn parse_counters(text: &str) -> Result<CountersSpec, String> {
    match text.trim() {
        "auto" => return Ok(CountersSpec::Auto),
        "none" => return Ok(CountersSpec::None),
        _ => {}
    }
    let mut list = Vec::new();
    for item in text.split(',').map(str::trim).filter(|i| !i.is_empty()) {
        let (name, arg) = match item.split_once('=') {
            Some((n, a)) => (n, Some(a)),
            None => (item, None),
        };
        let spec = match (name, arg) {
            ("jcmd", None) => BackendSpec::Jcmd,
            ("python-remote", None) => BackendSpec::PythonRemote,
            ("tokio", None) => BackendSpec::Tokio,
            ("node-inspect", None) => BackendSpec::NodeInspect(None),
            ("node-inspect", Some(p)) => BackendSpec::NodeInspect(Some(
                p.parse()
                    .map_err(|_| format!("node-inspect port {p:?} is not a port number"))?,
            )),
            ("go-pprof", Some(url)) if !url.is_empty() => BackendSpec::GoPprof(url.to_owned()),
            ("go-pprof", _) => {
                return Err("go-pprof needs a base URL: go-pprof=http://127.0.0.1:6060".to_owned());
            }
            _ => {
                return Err(format!(
                    "unknown counters backend {item:?}; use auto, none, or a list of jcmd, go-pprof=<url>, node-inspect[=<port>], python-remote, tokio"
                ));
            }
        };
        if !list.contains(&spec) {
            list.push(spec);
        }
    }
    if list.is_empty() {
        return Err("--counters is empty; use auto, none or a list of backends".to_owned());
    }
    Ok(CountersSpec::List(list))
}

/// The backends `--counters auto` picks for a process with command line `args`: `jcmd` for
/// `java`/`javaw`, `node-inspect` for `node` started with `--inspect`, none otherwise. It
/// never picks a backend that would signal or inject code into the process.
pub fn auto_specs(args: &[String]) -> Vec<BackendSpec> {
    let Some(first) = args.first() else {
        return Vec::new();
    };
    match proc::exe_name(first) {
        "java" | "javaw" => vec![BackendSpec::Jcmd],
        "node" | "nodejs" if args.iter().any(|a| a.starts_with("--inspect")) => {
            vec![BackendSpec::NodeInspect(None)]
        }
        _ => Vec::new(),
    }
}

/// What opening a backend may need to know about the target.
#[derive(Debug, Clone)]
pub struct OpenCtx {
    /// The sampled process.
    pub pid: u32,
    /// The sampling interval (backends poll no faster).
    pub interval: Duration,
    /// The command line of the process when readable.
    pub args: Vec<String>,
    /// Directory prefixes to strip from source paths so sites are project-relative.
    pub roots: Vec<String>,
    /// A writable scratch directory (the Python backend keeps its script and stats here).
    pub scratch: PathBuf,
}

/// Opens the backend `spec` names against the process in `ctx`.
pub fn open_backend(spec: &BackendSpec, ctx: &OpenCtx) -> Result<Box<dyn Backend>, BackendError> {
    match spec {
        BackendSpec::Jcmd => jcmd::open(ctx),
        BackendSpec::GoPprof(url) => gopprof::open(url, ctx),
        BackendSpec::NodeInspect(port) => node::open(*port, ctx),
        BackendSpec::PythonRemote => pyremote::open(ctx),
        BackendSpec::Tokio => Err(BackendError::blocked(
            "tokio metrics are read in-process by the Rust driver harness",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn parses_counters() {
        assert_eq!(parse_counters("auto").unwrap(), CountersSpec::Auto);
        assert_eq!(parse_counters("none").unwrap(), CountersSpec::None);
        assert_eq!(
            parse_counters("jcmd, go-pprof=http://127.0.0.1:6060,node-inspect=9229,tokio,jcmd")
                .unwrap(),
            CountersSpec::List(vec![
                BackendSpec::Jcmd,
                BackendSpec::GoPprof("http://127.0.0.1:6060".into()),
                BackendSpec::NodeInspect(Some(9229)),
                BackendSpec::Tokio,
            ])
        );
        assert!(parse_counters("go-pprof").is_err());
        assert!(parse_counters("node-inspect=x").is_err());
        assert!(parse_counters("perf").unwrap_err().contains("unknown"));
        assert!(parse_counters(" , ").is_err());
    }

    #[test]
    fn auto_picks_by_process_and_never_signals() {
        assert_eq!(
            auto_specs(&args(&["/usr/lib/jvm/bin/java", "-jar", "x.jar"])),
            [BackendSpec::Jcmd]
        );
        assert_eq!(
            auto_specs(&args(&["node", "--inspect=0", "app.js"])),
            [BackendSpec::NodeInspect(None)]
        );
        assert!(auto_specs(&args(&["node", "app.js"])).is_empty());
        assert!(auto_specs(&args(&["python3", "x.py"])).is_empty());
        assert!(auto_specs(&[]).is_empty());
    }

    #[test]
    fn tokio_is_a_recorded_limitation() {
        let ctx = OpenCtx {
            pid: 1,
            interval: Duration::from_millis(200),
            args: vec![],
            roots: vec![],
            scratch: std::env::temp_dir(),
        };
        let e = open_backend(&BackendSpec::Tokio, &ctx).err().unwrap();
        assert_eq!(
            e.message,
            "tokio metrics are read in-process by the Rust driver harness"
        );
        assert!(!e.missing);
    }
}
