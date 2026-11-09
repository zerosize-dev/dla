//! The goroutine dump text: `pprof.Lookup("goroutine").WriteTo(w, 2)`, the same text a
//! crashing Go program prints, and the `goroutineleak` profile at `debug=2`.
//!
//! ```text
//! goroutine 6 [select (leaked)]:
//! example.com/n/notify.(*Subscription).run(0xc000010000)
//!         /w/internal/notify/subscription.go:40 +0xff
//! created by example.com/n/notify.newSubscription in goroutine 1
//!         /w/internal/notify/subscription.go:33 +0x1b6
//! ```
//!
//! Each block is one goroutine: its wait reason, the stack it is blocked in (innermost
//! first) and, for every goroutine the program started, the `created by` frame. The
//! creation site is where the goroutine was made, so it ties a resource to the code that
//! leaks it. Arguments of the frames, label values and addresses are dropped: they are
//! data, not structure.

use std::collections::BTreeMap;

use super::{ParseError, normalize_path};
use crate::model::Frame;

/// Goroutines read from one dump.
pub const MAX_GOROUTINES: usize = 2_000_000;
/// Stack frames kept per goroutine (the blocked frame is the first).
const MAX_FRAMES: usize = 8;

/// One goroutine of a dump.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Goroutine {
    /// The goroutine id.
    pub id: u64,
    /// The wait reason or state (`select`, `chan receive`, `running`), without the
    /// `(leaked)` mark.
    pub wait: String,
    /// True when the runtime marked the goroutine `(leaked)`.
    pub leaked: bool,
    /// Minutes it has been blocked, when the dump says.
    pub minutes: Option<u64>,
    /// True when the goroutine carries pprof labels (their values are not read).
    pub labelled: bool,
    /// The blocked stack, innermost first.
    pub frames: Vec<Frame>,
    /// The function that started it and the line of the `go` statement.
    pub created_by: Option<Frame>,
}

/// A parsed dump.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Dump {
    /// The goroutines, in file order.
    pub goroutines: Vec<Goroutine>,
    /// `total N` of a `goroutineleak profile: total N` header, when the text has one.
    pub header_total: Option<u64>,
}

/// One creation site with the goroutines made there.
#[derive(Debug, Clone, PartialEq)]
pub struct CreationGroup {
    /// The creating function, as printed.
    pub function: String,
    /// The `go` statement's file (normalized), when printed.
    pub file: Option<String>,
    /// The `go` statement's line.
    pub line: Option<u32>,
    /// Goroutines made there.
    pub count: u64,
    /// How many of them are marked `(leaked)`.
    pub leaked: u64,
    /// The commonest blocked frame (innermost first frame of the stack).
    pub blocked: Option<Frame>,
    /// The commonest wait reason.
    pub wait: String,
}

/// True when `line` starts a goroutine block: `goroutine <digits> ... [<state>]:`.
fn is_header(line: &str) -> bool {
    let Some(rest) = line.strip_prefix("goroutine ") else {
        return false;
    };
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    digits > 0 && rest.contains('[') && line.ends_with("]:")
}

/// True when `text` looks like a goroutine dump: it has a goroutine header line.
pub fn looks_like_dump(text: &str) -> bool {
    text.lines().take(400).any(is_header)
}

/// True when `text` carries the goroutineleak profile's own header.
pub fn has_leak_header(text: &str) -> bool {
    text.lines()
        .take(8)
        .any(|l| l.starts_with("goroutineleak profile: total"))
}

fn parse_header(line: &str) -> Goroutine {
    let rest = &line["goroutine ".len()..];
    let digits = rest.bytes().take_while(u8::is_ascii_digit).count();
    let id = rest[..digits].parse().unwrap_or(0);
    let status = rest
        .find('[')
        .map(|i| &rest[i + 1..rest.len() - 2])
        .unwrap_or_default();
    // Label values may hold commas and brackets: cut them off first.
    let (status, labelled) = match status.find(", labels:") {
        Some(i) => (&status[..i], true),
        None => (status, false),
    };
    let mut parts = status.split(", ");
    let mut wait = parts.next().unwrap_or_default().trim().to_owned();
    let leaked = wait.ends_with(" (leaked)");
    if leaked {
        wait.truncate(wait.len() - " (leaked)".len());
    }
    let minutes = parts.find_map(|p| p.strip_suffix(" minutes")?.trim().parse().ok());
    Goroutine {
        id,
        wait,
        leaked,
        minutes,
        labelled,
        ..Goroutine::default()
    }
}

/// The function of a stack line: the text before the final argument list.
fn function_of(line: &str) -> String {
    let line = line.trim_end();
    let line = line.strip_suffix("(inlined)").map_or(line, str::trim_end);
    if !line.ends_with(')') {
        return line.to_owned();
    }
    let bytes = line.as_bytes();
    let mut depth = 0usize;
    for i in (0..bytes.len()).rev() {
        match bytes[i] {
            b')' => depth += 1,
            b'(' => {
                depth -= 1;
                if depth == 0 {
                    return line[..i].to_owned();
                }
            }
            _ => {}
        }
    }
    line.to_owned()
}

/// `\t/path/file.go:33 +0x1b6` to `(file, line)`.
fn location_of(line: &str) -> (Option<String>, Option<u32>) {
    let t = line.trim();
    let t = match t.rfind(" +0x") {
        Some(i) => &t[..i],
        None => t.find(" fp=").map_or(t, |i| &t[..i]),
    };
    match t.rsplit_once(':') {
        Some((f, l)) if !f.is_empty() => (
            Some(normalize_path(f)),
            l.trim().parse::<u32>().ok().filter(|l| *l > 0),
        ),
        _ => (None, None),
    }
}

/// Parses a goroutine dump. Text with no goroutine header is refused.
pub fn parse(text: &str) -> Result<Dump, ParseError> {
    let mut dump = Dump::default();
    let mut cur: Option<Goroutine> = None;
    // The frame function waiting for its location line.
    let mut pending: Option<(String, bool)> = None;
    for raw in text.lines() {
        if dump.header_total.is_none()
            && let Some(n) = raw.strip_prefix("goroutineleak profile: total ")
        {
            dump.header_total = n.trim().parse().ok();
            continue;
        }
        if is_header(raw) {
            if let Some(g) = cur.take() {
                dump.goroutines.push(g);
            }
            if dump.goroutines.len() >= MAX_GOROUTINES {
                return Err(ParseError::new(format!(
                    "the dump has more than {MAX_GOROUTINES} goroutines; capture it again with fewer or group it with `go tool pprof`"
                )));
            }
            cur = Some(parse_header(raw));
            pending = None;
            continue;
        }
        let Some(g) = cur.as_mut() else { continue };
        if raw.trim().is_empty() {
            if let Some(g) = cur.take() {
                dump.goroutines.push(g);
            }
            pending = None;
            continue;
        }
        if raw.starts_with(['\t', ' ']) {
            // A location line belongs to the function line before it.
            if let Some((function, creator)) = pending.take() {
                let (file, line) = location_of(raw);
                let frame = Frame {
                    function,
                    file,
                    line,
                    module: None,
                };
                if creator {
                    g.created_by = Some(frame);
                } else if g.frames.len() < MAX_FRAMES {
                    g.frames.push(frame);
                }
            }
            continue;
        }
        if raw.starts_with("...") {
            continue;
        }
        if let Some(c) = raw.strip_prefix("created by ") {
            let function = c.split_once(" in goroutine ").map_or(c, |(f, _)| f);
            pending = Some((function.trim().to_owned(), true));
        } else {
            pending = Some((function_of(raw), false));
        }
    }
    if let Some(g) = cur.take() {
        dump.goroutines.push(g);
    }
    if dump.goroutines.is_empty() {
        return Err(ParseError::new(
            "no goroutine block in the text; expected `goroutine <id> [<state>]:` lines from pprof.Lookup(\"goroutine\").WriteTo(w, 2) or the goroutineleak profile at debug=2",
        ));
    }
    Ok(dump)
}

/// The key with the highest count; the smallest key on a tie.
fn top<K: Ord + Clone>(m: &BTreeMap<K, u64>) -> Option<K> {
    m.iter()
        .max_by(|a, b| a.1.cmp(b.1).then_with(|| b.0.cmp(a.0)))
        .map(|(k, _)| k.clone())
}

/// Groups the goroutines that have a creation frame by that frame. With `only_leaked`,
/// only goroutines marked `(leaked)` count. Biggest groups first.
pub fn group_by_creator(goroutines: &[Goroutine], only_leaked: bool) -> Vec<CreationGroup> {
    type Key = (String, Option<String>, Option<u32>);
    #[derive(Default)]
    struct Acc {
        count: u64,
        leaked: u64,
        blocked: BTreeMap<(String, Option<String>, Option<u32>), u64>,
        wait: BTreeMap<String, u64>,
    }
    let mut by: BTreeMap<Key, Acc> = BTreeMap::new();
    for g in goroutines {
        if only_leaked && !g.leaked {
            continue;
        }
        let Some(c) = &g.created_by else { continue };
        let a = by
            .entry((c.function.clone(), c.file.clone(), c.line))
            .or_default();
        a.count += 1;
        a.leaked += u64::from(g.leaked);
        if let Some(b) = g.frames.first() {
            *a.blocked
                .entry((b.function.clone(), b.file.clone(), b.line))
                .or_default() += 1;
        }
        *a.wait.entry(g.wait.clone()).or_default() += 1;
    }
    let mut out: Vec<CreationGroup> = by
        .into_iter()
        .map(|((function, file, line), a)| CreationGroup {
            function,
            file,
            line,
            count: a.count,
            leaked: a.leaked,
            blocked: top(&a.blocked).map(|(function, file, line)| Frame {
                function,
                file,
                line,
                module: None,
            }),
            wait: top(&a.wait).unwrap_or_default(),
        })
        .collect();
    out.sort_by(|a, b| {
        b.count
            .cmp(&a.count)
            .then_with(|| (&a.function, &a.file, a.line).cmp(&(&b.function, &b.file, b.line)))
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const DUMP: &str = "goroutine 1 [running]:
runtime/pprof.writeGoroutineStacks({0x5f5ae8, 0x3b77d0f2008})
\t/usr/local/go/src/runtime/pprof/pprof.go:816 +0x69
main.main()
\t/w/harness/H1/work/cmd/x/main.go:33 +0x1e5

goroutine 6 [select (leaked)]:
example.com/n/notify.(*Subscription).run(0x3b77d208000)
\t/w/harness/H1/work/internal/notify/subscription.go:40 +0xff
created by example.com/n/notify.newSubscription in goroutine 1
\t/w/harness/H1/work/internal/notify/subscription.go:33 +0x1b6

goroutine 7 [select (leaked), 4 minutes]:
example.com/n/notify.(*Subscription).run(0x3b77d208080)
\t/w/harness/H1/work/internal/notify/subscription.go:40 +0xff
created by example.com/n/notify.newSubscription in goroutine 1
\t/w/harness/H1/work/internal/notify/subscription.go:33 +0x1b6

goroutine 9 [chan receive, labels:{\"user\":\"AKIA1, [x]\"}]:
example.com/n/q.(*W).loop(...)
\t/w/harness/H1/work/q/w.go:12
created by example.com/n/q.Start
\t/w/harness/H1/work/q/w.go:5 +0x4a
";

    #[test]
    fn blocks_become_goroutines_with_a_blocked_frame_and_a_creator() {
        let d = parse(DUMP).unwrap();
        assert_eq!(d.goroutines.len(), 4);
        let g = &d.goroutines[1];
        assert_eq!((g.id, g.wait.as_str(), g.leaked), (6, "select", true));
        assert_eq!(
            g.frames[0].function,
            "example.com/n/notify.(*Subscription).run"
        );
        assert_eq!(
            g.frames[0].file.as_deref(),
            Some("internal/notify/subscription.go")
        );
        assert_eq!(g.frames[0].line, Some(40));
        let c = g.created_by.as_ref().unwrap();
        assert_eq!(c.function, "example.com/n/notify.newSubscription");
        assert_eq!(
            (c.file.as_deref(), c.line),
            (Some("internal/notify/subscription.go"), Some(33))
        );
        assert_eq!(d.goroutines[2].minutes, Some(4));
        assert!(d.goroutines[0].created_by.is_none());
    }

    #[test]
    fn label_values_are_never_read() {
        let d = parse(DUMP).unwrap();
        let g = &d.goroutines[3];
        assert!(g.labelled);
        assert_eq!(g.wait, "chan receive");
        assert!(!format!("{g:?}").contains("AKIA1"));
        // `(...)` arguments and a creator without "in goroutine".
        assert_eq!(g.frames[0].function, "example.com/n/q.(*W).loop");
        assert_eq!(
            g.created_by.as_ref().unwrap().function,
            "example.com/n/q.Start"
        );
    }

    #[test]
    fn groups_count_by_creation_site() {
        let d = parse(DUMP).unwrap();
        let all = group_by_creator(&d.goroutines, false);
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].count, 2);
        assert_eq!(all[0].leaked, 2);
        assert_eq!(all[0].wait, "select");
        assert_eq!(all[0].blocked.as_ref().unwrap().line, Some(40));
        let leaked = group_by_creator(&d.goroutines, true);
        assert_eq!(leaked.len(), 1);
    }

    #[test]
    fn text_that_is_no_dump_is_refused() {
        assert!(parse("hello\nworld\n").is_err());
        assert!(!looks_like_dump("hello"));
        assert!(looks_like_dump(DUMP));
        let d = parse("goroutineleak profile: total 3\n\ngoroutine 5 [chan send (leaked)]:\nmain.f()\n\t/a/b.go:3 +0x1\ncreated by main.g in goroutine 1\n\t/a/b.go:9 +0x2\n").unwrap();
        assert_eq!(d.header_total, Some(3));
        assert!(has_leak_header("goroutineleak profile: total 3\n"));
    }

    #[test]
    fn nested_parentheses_in_arguments_do_not_confuse_the_name() {
        assert_eq!(function_of("a/b.f(0x1, {0x2, 0x3})"), "a/b.f");
        assert_eq!(function_of("a/b.(*T).m.func1(...)"), "a/b.(*T).m.func1");
        assert_eq!(function_of("a/b.F[...](0x1)"), "a/b.F[...]");
        assert_eq!(function_of("a/b.f(...) (inlined)"), "a/b.f");
    }
}
