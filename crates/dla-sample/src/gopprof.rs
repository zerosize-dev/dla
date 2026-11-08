//! The `go-pprof=<base url>` backend: reads `/debug/pprof/heap?debug=1` for the
//! `runtime.MemStats` figures and `/debug/pprof/goroutine?debug=2` for goroutines grouped by
//! the site that started them.
//!
//! Columns: `go.heap_live` (HeapAlloc), `go.heap_inuse`, `go.heap_objects`, `go.total` (Sys),
//! `go.released` (HeapReleased), `go.stacks` (stack in use), `go.goroutines` and
//! `go.goroutines@<file:line>` per creation site. HeapAlloc includes garbage that has not
//! been collected yet, so none of these are after-GC figures.

use std::{collections::BTreeMap, time::Duration};

use crate::{
    backend::{Backend, BackendError, OpenCtx, Reading},
    http::{self, BaseUrl},
};

const TIMEOUT: Duration = Duration::from_secs(10);
/// Goroutine dumps stop the world; do not take them more often than this.
const MIN_INTERVAL: Duration = Duration::from_millis(500);

/// Parses the `# runtime.MemStats` block of `heap?debug=1` into series columns.
pub fn parse_memstats(text: &str) -> BTreeMap<String, f64> {
    let mut out = BTreeMap::new();
    let mut in_block = false;
    for line in text.lines() {
        if line.starts_with("# runtime.MemStats") {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        let Some(rest) = line.strip_prefix("# ") else {
            continue;
        };
        let Some((key, value)) = rest.split_once('=') else {
            continue;
        };
        // `Stack = inuse / sys` has two numbers; the first is the one in use.
        let first = value.split('/').next().unwrap_or("").trim();
        let Ok(v) = first.parse::<f64>() else {
            continue;
        };
        let column = match key.trim() {
            "HeapAlloc" => "go.heap_live",
            "HeapInuse" => "go.heap_inuse",
            "HeapObjects" => "go.heap_objects",
            "Sys" => "go.total",
            "HeapReleased" => "go.released",
            "Stack" | "StackInuse" => "go.stacks",
            _ => continue,
        };
        out.insert(column.to_owned(), v);
    }
    out
}

/// Goroutines counted from a `goroutine?debug=2` dump.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Goroutines {
    /// All goroutines in the dump.
    pub total: u64,
    /// Goroutines per creating site (`file:line` of the `go` statement), absolute paths.
    pub by_site: BTreeMap<String, u64>,
}

/// Groups a `goroutine?debug=2` dump by the `created by` site. Goroutines with no creator
/// (main, runtime workers) count in `total` only.
pub fn parse_goroutines(text: &str) -> Goroutines {
    let mut g = Goroutines::default();
    let mut after_created_by = false;
    for line in text.lines() {
        if line.starts_with("goroutine ") && line.trim_end().ends_with(':') {
            g.total += 1;
            after_created_by = false;
        } else if line.starts_with("created by ") {
            after_created_by = true;
        } else if after_created_by && line.starts_with('\t') {
            after_created_by = false;
            if let Some(site) = line.split_whitespace().next() {
                *g.by_site.entry(site.to_owned()).or_insert(0) += 1;
            }
        } else {
            after_created_by = false;
        }
    }
    g
}

/// Makes a site path project-relative: strips `roots`, the Go source tree
/// (`.../go1.24/src/`) and the module cache (`.../pkg/mod/`).
pub fn relative_site(site: &str, roots: &[String]) -> String {
    let (path, line) = site.rsplit_once(':').unwrap_or((site, ""));
    let mut rel = None;
    for root in roots {
        let root = root.trim_end_matches('/');
        if let Some(r) = path.strip_prefix(root).and_then(|r| r.strip_prefix('/')) {
            rel = Some(r.to_owned());
            break;
        }
    }
    let rel = rel.unwrap_or_else(|| {
        if let Some(i) = path.find("/pkg/mod/") {
            return path[i + "/pkg/mod/".len()..].to_owned();
        }
        let mut from = 0;
        while let Some(i) = path[from..].find("/src/") {
            let at = from + i;
            let component = path[..at].rsplit('/').next().unwrap_or("");
            if component.starts_with("go") {
                return path[at + "/src/".len()..].to_owned();
            }
            from = at + 1;
        }
        path.to_owned()
    });
    if line.is_empty() {
        rel
    } else {
        format!("{rel}:{line}")
    }
}

#[derive(Debug)]
struct GoPprof {
    base: BaseUrl,
    roots: Vec<String>,
    interval: Duration,
}

/// Checks the endpoint answers and returns the backend.
pub fn open(url: &str, ctx: &OpenCtx) -> Result<Box<dyn Backend>, BackendError> {
    let base =
        http::parse_base_url(url).map_err(|e| BackendError::blocked(format!("go-pprof: {e}")))?;
    let b = GoPprof {
        base,
        roots: ctx.roots.clone(),
        interval: ctx.interval,
    };
    b.get_text("/debug/pprof/heap?debug=1").map_err(|e| {
        BackendError::blocked(format!(
            "go-pprof: {e}; import net/http/pprof in the target and serve it on that address"
        ))
    })?;
    Ok(Box::new(b))
}

impl GoPprof {
    fn get_text(&self, path: &str) -> Result<String, String> {
        let full = format!("{}{path}", self.base.prefix);
        let r = http::get(&self.base.host, self.base.port, &full, TIMEOUT)?;
        if r.status != 200 {
            return Err(format!(
                "GET {full} on {}:{} answered {}",
                self.base.host, self.base.port, r.status
            ));
        }
        Ok(r.text())
    }
}

impl Backend for GoPprof {
    fn name(&self) -> &'static str {
        "go-pprof"
    }

    fn runtime(&self) -> &'static str {
        "go"
    }

    fn min_interval(&self) -> Duration {
        self.interval.max(MIN_INTERVAL)
    }

    fn poll(&mut self) -> Result<Reading, BackendError> {
        let heap = self
            .get_text("/debug/pprof/heap?debug=1")
            .map_err(|e| BackendError::blocked(format!("go-pprof: {e}")))?;
        let mut reading = Reading {
            values: parse_memstats(&heap),
            notes: Vec::new(),
        };
        if reading.values.is_empty() {
            return Err(BackendError::blocked(
                "go-pprof: heap?debug=1 has no runtime.MemStats block",
            ));
        }
        match self.get_text("/debug/pprof/goroutine?debug=2") {
            Ok(text) => {
                let g = parse_goroutines(&text);
                reading
                    .values
                    .insert("go.goroutines".into(), g.total as f64);
                for (site, n) in g.by_site {
                    let name = format!("go.goroutines@{}", relative_site(&site, &self.roots));
                    *reading.values.entry(name).or_insert(0.0) += n as f64;
                }
            }
            Err(e) => reading.notes.push(format!("go-pprof goroutines: {e}")),
        }
        Ok(reading)
    }

    fn zero_fill_prefixes(&self) -> &'static [&'static str] {
        &["go.goroutines@"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAP: &str = "heap profile: 1: 2 [3: 4] @ heapprofile\n# runtime.MemStats\n# Alloc = 781096\n# TotalAlloc = 781096\n# Sys = 8344840\n# HeapAlloc = 781096\n# HeapSys = 3866624\n# HeapIdle = 2433024\n# HeapInuse = 1433600\n# HeapReleased = 2400256\n# HeapObjects = 1293\n# Stack = 327680 / 327680\n# NextGC = 4194304\n# PauseNs = [0 0 0]\n# DebugGC = false\n";

    #[test]
    fn memstats_lines() {
        let m = parse_memstats(HEAP);
        assert_eq!(m["go.heap_live"], 781_096.0);
        assert_eq!(m["go.heap_inuse"], 1_433_600.0);
        assert_eq!(m["go.total"], 8_344_840.0);
        assert_eq!(m["go.released"], 2_400_256.0);
        assert_eq!(m["go.stacks"], 327_680.0);
        assert_eq!(m["go.heap_objects"], 1293.0);
        assert!(parse_memstats("no block here").is_empty());
        let newer = "# runtime.MemStats\n# StackInuse = 99\n";
        assert_eq!(parse_memstats(newer)["go.stacks"], 99.0);
    }

    const DUMP: &str = "goroutine 18 [running]:\nruntime/pprof.writeGoroutineStacks({0x7c6dc0, 0xc0000001c0})\n\t/usr/local/go1.24.7/src/runtime/pprof/pprof.go:764 +0x6a\ncreated by net/http.(*Server).Serve in goroutine 1\n\t/usr/local/go1.24.7/src/net/http/server.go:3454 +0x485\n\ngoroutine 1 [IO wait]:\nmain.main()\n\t/work/proj/main.go:25 +0xc5\n\ngoroutine 7 [sleep]:\ntime.Sleep(0x989680)\n\t/usr/local/go1.24.7/src/runtime/time.go:338 +0x165\nmain.leak(...)\n\t/work/proj/main.go:12\ncreated by main.main in goroutine 1\n\t/work/proj/main.go:22 +0x3f\n\ngoroutine 8 [sleep]:\nmain.leak(...)\n\t/work/proj/main.go:12\ncreated by main.main in goroutine 1\n\t/work/proj/main.go:22 +0x3f\n\ngoroutine 9 [select]:\nmain.main.func1()\n\t/work/proj/main.go:23 +0x1\ncreated by main.main\n\t/work/proj/main.go:23 +0x2\n";

    #[test]
    fn goroutines_group_by_creator() {
        let g = parse_goroutines(DUMP);
        assert_eq!(g.total, 5);
        assert_eq!(g.by_site["/work/proj/main.go:22"], 2);
        assert_eq!(g.by_site["/work/proj/main.go:23"], 1);
        assert_eq!(
            g.by_site["/usr/local/go1.24.7/src/net/http/server.go:3454"],
            1
        );
        assert_eq!(g.by_site.len(), 3);
    }

    #[test]
    fn sites_become_relative() {
        let roots = vec!["/work/proj".to_owned()];
        assert_eq!(
            relative_site("/work/proj/internal/n/s.go:33", &roots),
            "internal/n/s.go:33"
        );
        assert_eq!(
            relative_site("/usr/local/go1.24.7/src/net/http/server.go:3454", &roots),
            "net/http/server.go:3454"
        );
        assert_eq!(
            relative_site("/home/u/go/pkg/mod/github.com/x/y@v1.0.0/a.go:5", &[]),
            "github.com/x/y@v1.0.0/a.go:5"
        );
        assert_eq!(
            relative_site("/opt/other/a.go:9", &roots),
            "/opt/other/a.go:9"
        );
    }
}
