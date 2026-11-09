//! Go formats: pprof profiles (the gzipped protobuf `runtime/pprof` writes), heap profiles
//! as allocation stacks and function histograms, goroutine profiles, and the `debug=2`
//! goroutine dump text (also the text of the `goroutineleak` profile).
//!
//! Everything is read with std and serde only. Lengths and counts read from files are
//! checked, and the decompressed size of a profile is capped.

use std::fmt;

pub mod dump;
pub mod goroutine;
pub mod heap;
pub mod pprof;

/// Why a Go tool output could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(String);

impl ParseError {
    pub(crate) fn new(msg: impl Into<String>) -> Self {
        Self(msg.into())
    }

    /// The message: what was found and what was expected.
    pub fn message(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

/// Marker of a build done in a scratch copy of the project: `/harness/H<n>/work/`.
const SCRATCH_MARK: &str = "/harness/H";

/// Makes a source path from a Go profile comparable with the project's files.
///
/// * A file in the harness scratch copy (`.../harness/H1/work/internal/x.go`) becomes the
///   project-relative `internal/x.go`.
/// * The standard library (`<GOROOT>/src/fmt/print.go`, or the same under a downloaded
///   toolchain) becomes `$GOROOT/src/fmt/print.go`; the module cache becomes
///   `$GOMODCACHE/<module>@<version>/...`.
/// * Anything else is returned as printed.
pub fn normalize_path(path: &str) -> String {
    let p = path.replace('\\', "/");
    let mut from = 0;
    while let Some(at) = p[from..].find(SCRATCH_MARK) {
        let start = from + at + SCRATCH_MARK.len();
        from = start;
        let digits = p[start..].bytes().take_while(u8::is_ascii_digit).count();
        if digits > 0 && p[start + digits..].starts_with("/work/") {
            return p[start + digits + "/work/".len()..].to_owned();
        }
    }
    if let Some(at) = p.find("/pkg/mod/") {
        let rest = &p[at + "/pkg/mod/".len()..];
        // A toolchain downloaded by GOTOOLCHAIN lives in the module cache too.
        if let Some(src) = rest.find("/src/")
            && rest.starts_with("golang.org/toolchain@")
        {
            return format!("$GOROOT{}", &rest[src..]);
        }
        return format!("$GOMODCACHE/{rest}");
    }
    for marker in [
        "/go/src/",
        "/golang/src/",
        "/usr/local/go/src/",
        "/goroot/src/",
    ] {
        if let Some(at) = p.find(marker) {
            let rest = &p[at + marker.len()..];
            // A GOPATH checkout (`.../go/src/github.com/acme/app`) is not the standard
            // library: its first element has a dot.
            if rest.split('/').next().is_some_and(|e| e.contains('.')) {
                continue;
            }
            return format!("$GOROOT/src/{rest}");
        }
    }
    p
}

/// True when `function` is a standard-library function (`strings.(*Builder).grow`,
/// `net/http.(*Server).Serve`, `runtime.gopark`): the first element of its import path
/// has no dot and is not `main`. A module path whose first element has no dot is read as
/// standard library; the answer only picks which frame labels a histogram row.
pub fn is_std_function(function: &str) -> bool {
    let first = match function.split_once('/') {
        Some((head, _)) => head,
        None => function.split_once('.').map_or(function, |(p, _)| p),
    };
    !first.contains('.') && first != "main"
}

/// The package part of a Go function name (`example.com/a/b.(*T).m` is `example.com/a/b`).
pub fn package_of(function: &str) -> &str {
    let slash = function.rfind('/').map_or(0, |i| i + 1);
    match function[slash..].find('.') {
        Some(dot) => &function[..slash + dot],
        None => function,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scratch_paths_become_project_relative() {
        assert_eq!(
            normalize_path("/p/.dla/20260101-aa/harness/H1/work/internal/notify/subscription.go"),
            "internal/notify/subscription.go"
        );
        assert_eq!(
            normalize_path("/r/harness/Hx/work/a.go"),
            "/r/harness/Hx/work/a.go"
        );
    }

    #[test]
    fn a_gopath_project_file_is_not_the_standard_library() {
        assert_eq!(
            normalize_path("/home/u/go/src/github.com/x/y/z.go"),
            "/home/u/go/src/github.com/x/y/z.go"
        );
        assert_eq!(
            normalize_path("/usr/local/go/src/fmt/print.go"),
            "$GOROOT/src/fmt/print.go"
        );
    }

    #[test]
    fn toolchain_and_module_cache_paths_are_shortened() {
        assert_eq!(
            normalize_path(
                "/root/go/pkg/mod/golang.org/toolchain@v0.0.1-go1.27.0.linux-amd64/src/runtime/proc.go"
            ),
            "$GOROOT/src/runtime/proc.go"
        );
        assert_eq!(
            normalize_path("/root/go/pkg/mod/github.com/x/y@v1.2.3/z.go"),
            "$GOMODCACHE/github.com/x/y@v1.2.3/z.go"
        );
        assert_eq!(
            normalize_path("/usr/local/go/src/fmt/print.go"),
            "$GOROOT/src/fmt/print.go"
        );
        assert_eq!(normalize_path("/home/u/app/main.go"), "/home/u/app/main.go");
    }

    #[test]
    fn std_and_project_functions_are_told_apart() {
        for f in [
            "runtime.gopark",
            "strings.(*Builder).grow",
            "net/http.(*Server).Serve",
            "internal/bytealg.MakeNoZero",
        ] {
            assert!(is_std_function(f), "{f}");
        }
        for f in [
            "main.main",
            "example.com/notifyd/internal/notify.newSubscription",
            "github.com/a/b.F",
        ] {
            assert!(!is_std_function(f), "{f}");
        }
        assert_eq!(
            package_of("example.com/notifyd/internal/notify.(*Hub).Open"),
            "example.com/notifyd/internal/notify"
        );
        assert_eq!(package_of("main.main"), "main");
    }
}
