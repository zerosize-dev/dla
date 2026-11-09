//! LeakSanitizer text reports.
//!
//! LeakSanitizer comes with `gcc -fsanitize=leak` or `-fsanitize=address`, or can be
//! `LD_PRELOAD`ed (`liblsan.so`) into an unmodified binary. Useful settings:
//!
//! - `LSAN_OPTIONS=use_globals=0` makes globals non-roots, so a leak that stays reachable from
//!   a static is reported (as a direct leak for the head and indirect leaks behind it). The
//!   default hides it. The report does not name the global that held the memory.
//! - `LSAN_OPTIONS=log_path=<file>` writes the report to `<file>.<pid>`; the exit code is 23
//!   when something leaked.
//!
//! Records become [`AllocGroup`]s with kind `direct-leak` or `indirect-leak`. The sanitizer's
//! own interceptor frames and the Rust allocator glue at the top of a stack are dropped.

use serde::{Deserialize, Serialize};

use super::ParseError;
use super::frames::{make_frame, split_location, trim_allocator_frames};
use crate::model::{AllocGroup, AllocProfile, Frame};

/// A parsed LeakSanitizer report.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct LsanReport {
    pub profile: AllocProfile,
    pub pid: Option<u32>,
    /// Bytes from the `SUMMARY:` line, when present.
    pub summary_bytes: Option<u64>,
    /// Allocations from the `SUMMARY:` line, when present.
    pub summary_allocations: Option<u64>,
    /// The first `ERROR: AddressSanitizer: <kind>` line, when the run died of a memory error
    /// before the leak check.
    pub memory_error: Option<String>,
}

/// Parses a report. Text without any leak record gives an empty profile (a clean run prints
/// nothing); a report that says LeakSanitizer could not run is an error.
pub fn parse(input: &str) -> Result<LsanReport, ParseError> {
    let mut report = LsanReport {
        profile: AllocProfile {
            source: "lsan".to_owned(),
            ..AllocProfile::default()
        },
        ..LsanReport::default()
    };
    let mut lines = input.lines().peekable();
    while let Some(raw) = lines.next() {
        let line = raw.trim();
        if line.contains("LeakSanitizer has encountered a fatal error")
            || line.contains("LeakSanitizer does not work under ptrace")
        {
            return Err(ParseError::new(format!(
                "LeakSanitizer could not run: {line} (run the program without strace or gdb, or set LSAN_OPTIONS=detect_leaks=1 only where ptrace is allowed)"
            )));
        }
        if let Some(rest) = line.strip_prefix("==") {
            if let Some((pid, tail)) = rest.split_once("==") {
                if report.pid.is_none() {
                    report.pid = pid.parse().ok();
                }
                if report.memory_error.is_none()
                    && let Some(kind) = tail.trim().strip_prefix("ERROR: AddressSanitizer:")
                {
                    report.memory_error = Some(kind.trim().to_owned());
                }
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("SUMMARY:") {
            let rest = rest.trim();
            if rest.starts_with("LeakSanitizer:") || rest.starts_with("AddressSanitizer:") {
                let nums = numbers(rest);
                if rest.contains("leaked in") {
                    report.summary_bytes = nums.first().copied();
                    report.summary_allocations = nums.get(1).copied();
                }
            }
            continue;
        }
        let (kind, rest) = if let Some(r) = line.strip_prefix("Direct leak of ") {
            ("direct-leak", r)
        } else if let Some(r) = line.strip_prefix("Indirect leak of ") {
            ("indirect-leak", r)
        } else {
            continue;
        };
        let nums = numbers(rest);
        let (Some(&bytes), Some(&objects)) = (nums.first(), nums.get(1)) else {
            return Err(ParseError::new(format!(
                "LeakSanitizer record header has no byte and object counts: {line:?} (expected \"Direct leak of N byte(s) in M object(s) allocated from:\")"
            )));
        };
        let mut frames = Vec::new();
        while let Some(next) = lines.peek() {
            let t = next.trim_start();
            if !t.starts_with('#') {
                break;
            }
            let t = t.to_owned();
            lines.next();
            frames.push(parse_frame(&t)?);
        }
        if frames.is_empty() {
            return Err(ParseError::new(format!(
                "LeakSanitizer record {line:?} has no stack (symbolization may be off; set LSAN_OPTIONS=fast_unwind_on_malloc=0 or check that the program has debug info)"
            )));
        }
        let skip = frames.iter().take_while(|f| is_sanitizer_frame(f)).count();
        if skip < frames.len() {
            frames.drain(..skip);
        }
        report.profile.groups.push(AllocGroup {
            frames: trim_allocator_frames(frames),
            live_bytes: bytes,
            live_blocks: objects,
            total_bytes: None,
            root_path: Vec::new(),
            kind: Some(kind.to_owned()),
        });
    }
    Ok(report)
}

fn numbers(text: &str) -> Vec<u64> {
    text.split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect()
}

fn is_sanitizer_frame(frame: &Frame) -> bool {
    frame.file.as_deref().is_some_and(|f| {
        f.contains("/libsanitizer/")
            || f.contains("lsan_interceptors")
            || f.contains("asan_malloc")
            || f.contains("asan_new_delete")
            || f.contains("sanitizer_common")
    })
}

/// `#1 0x55dab06332ad in add_session /path/leak.c:11`,
/// `#5 0x... in _start (/path/prog+0x1184) (BuildId: ...)`, `#2 0x... (/lib/libc.so.6+0x1)`.
fn parse_frame(line: &str) -> Result<Frame, ParseError> {
    let bad = || {
        ParseError::new(format!(
            "unreadable LeakSanitizer stack frame: {line:?} (expected \"#N 0xADDR in function file:line\")"
        ))
    };
    let rest = line.strip_prefix('#').ok_or_else(bad)?;
    let (index, rest) = rest.split_once(char::is_whitespace).ok_or_else(bad)?;
    if index.is_empty() || !index.bytes().all(|b| b.is_ascii_digit()) {
        return Err(bad());
    }
    let rest = rest.trim_start();
    let (addr, rest) = rest.split_once(char::is_whitespace).unwrap_or((rest, ""));
    if !addr.starts_with("0x") {
        return Err(bad());
    }
    let mut rest = rest.trim();
    if let Some(p) = rest.rfind(" (BuildId:") {
        rest = rest[..p].trim_end();
    }
    let Some(body) = rest.strip_prefix("in ") else {
        // No symbol: `(module+0xoff)` or nothing at all.
        let module = rest
            .strip_prefix('(')
            .and_then(|m| m.strip_suffix(')'))
            .map(module_of);
        return Ok(make_frame(addr, None, None, module));
    };
    // `fn (module+0xoff)`: a trailing parenthesised module, with an offset.
    if let Some(open) = body.rfind(" (")
        && let Some(inner) = body[open + 2..].strip_suffix(')')
        && inner.contains("+0x")
    {
        return Ok(make_frame(
            body[..open].trim(),
            None,
            None,
            Some(module_of(inner)),
        ));
    }
    // `fn file:line`: the location is the last space-free token.
    if let Some((function, loc)) = body.rsplit_once(' ') {
        let (file, line_no) = split_location(loc);
        if line_no.is_some() || loc.contains('/') {
            return Ok(make_frame(function.trim(), Some(file), line_no, None));
        }
    }
    Ok(make_frame(body, None, None, None))
}

fn module_of(inner: &str) -> String {
    inner
        .rsplit_once("+0x")
        .map_or(inner, |(m, _)| m)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_in_the_three_shapes() {
        let f = parse_frame("#1 0x55dab0633261 in add_session /w/leak.c:11").unwrap();
        assert_eq!(
            (f.function.as_str(), f.file.as_deref(), f.line),
            ("add_session", Some("/w/leak.c"), Some(11))
        );
        let f = parse_frame("#5 0x55 in _start (/w/leak_lsan+0x1184) (BuildId: 02790e)").unwrap();
        assert_eq!(
            (f.function.as_str(), f.module.as_deref()),
            ("_start", Some("/w/leak_lsan"))
        );
        let f = parse_frame("#2 0x7f12 (/lib/x86_64-linux-gnu/libc.so.6+0x2a1c9)").unwrap();
        assert_eq!(
            (f.function.as_str(), f.module.as_deref()),
            ("0x7f12", Some("/lib/x86_64-linux-gnu/libc.so.6"))
        );
        let f = parse_frame("#3 0x1 in operator new(unsigned long) ../../../../src/libsanitizer/asan/asan_new_delete.cpp:95").unwrap();
        assert_eq!(f.function, "operator new(unsigned long)");
        assert_eq!(f.line, Some(95));
    }

    #[test]
    fn bad_frames_are_errors() {
        for bad in ["#x 0x1 in f a.c:1", "#1 nothex in f", "#1", "1 0x1 in f"] {
            assert!(parse_frame(bad).is_err(), "{bad}");
        }
    }
}
