//! Valgrind gdbserver monitor commands, driven with `vgdb`: `leak_check`, `block_list` and
//! `who_points_at`. They name what keeps a leaked block alive: a data symbol (a static or
//! global), a thread stack, or another heap block, so a chain of blocks can be walked up to
//! its holder.
//!
//! # How to produce this output
//!
//! 1. Start the target under Valgrind with a private prefix (the default `/tmp/vgdb-pipe-*`
//!    may be read-only in a sandbox):
//!    `valgrind --vgdb=yes --vgdb-prefix=<dir>/vgdb --leak-check=full --show-reachable=yes ./prog`
//!    The process must be one DLA starts; Valgrind cannot attach to a running pid.
//! 2. When the program has reached the state to inspect (it runs the workload and then
//!    sleeps, or the harness paused it), run each command as one `vgdb` call, with the pid
//!    that Valgrind prints in its `==<pid>==` lines:
//!    `vgdb --pid=<pid> --vgdb-prefix=<dir>/vgdb leak_check full reachable any`
//!    (use `increased` instead of `any` to see only records that grew since the last check),
//!    then `vgdb ... block_list <loss_record_nr>`, then
//!    `vgdb ... who_points_at 0xADDR [len]` for one block address of that record.
//! 3. Each `vgdb` call takes about a second. Record numbers are those of the last
//!    `leak_check`. To walk up a chain, query `who_points_at` for the block's start address;
//!    when the answer is another heap block, query that block's start
//!    ([`next_block_to_query`]) until a data symbol, a stack or nothing answers.
//!
//! `vgdb` prints `sending command ... to pid N` first; every parser here skips that line and
//! the `==pid==` prefixes. Error answers (`invalid loss record nr`, `missing or malformed
//! address`) become [`ParseError`]s.

use serde::{Deserialize, Serialize};

use super::ParseError;
use super::demangle::demangle_or_raw;
use super::frames::{make_frame, split_location};
use crate::model::Frame;

/// One loss record header with its stack, as `leak_check` and `block_list` print it.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct LossRecord {
    /// Record number, from 1.
    pub nr: u32,
    /// Number of records in the check.
    pub of: u32,
    /// `definitely-lost`, `indirectly-lost`, `possibly-lost` or `still-reachable`.
    pub kind: String,
    pub bytes: u64,
    pub blocks: u64,
    /// For a definitely-lost record that also owns other blocks: `(D direct, I indirect)`.
    pub direct_bytes: Option<u64>,
    pub indirect_bytes: Option<u64>,
    /// Growth since the previous check (`leak_check ... increased` prints `(+N)`).
    pub bytes_delta: Option<i64>,
    pub blocks_delta: Option<i64>,
    /// Allocation stack, innermost first, without Valgrind's own malloc frame.
    pub frames: Vec<Frame>,
}

/// `leak_check` output.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct LeakCheck {
    pub records: Vec<LossRecord>,
}

/// A heap block named by `block_list`.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRef {
    pub address: u64,
    pub size: u64,
}

/// `block_list <loss_record_nr>` output.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct BlockList {
    pub record: LossRecord,
    pub blocks: Vec<BlockRef>,
}

/// What holds a pointer to the queried block.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Holder {
    /// A static or global variable; `offset` is where inside it the pointer sits.
    DataSymbol { symbol: String, offset: u64 },
    /// Another heap block, with the allocation stack Valgrind printed for it.
    Block {
        /// Where the pointer sits.
        address: u64,
        /// Bytes from the block start to that address.
        offset: u64,
        size: u64,
        frames: Vec<Frame>,
    },
    /// A thread's stack; `frame` is Valgrind's `in frame #N, created by ...` text.
    Stack { thread: u32, frame: Option<String> },
    /// A mapped segment without a symbol (`rw- mapped file X segment`, `BSS segment of X`,
    /// `rw- anonymous segment`): the text after `Address ... is `.
    Segment { description: String },
    /// Anything else Valgrind said about the address.
    Other { description: String },
}

/// One pointer found by `who_points_at`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PointerRef {
    /// Address of the pointer itself.
    pub from: u64,
    pub holder: Holder,
}

/// `who_points_at <addr> [len]` output.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WhoPointsAt {
    pub target: u64,
    pub len: Option<u64>,
    pub refs: Vec<PointerRef>,
}

// ---------------------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------------------

/// Strips `==1234== ` (the prefix of every Valgrind line); other lines are returned as is.
fn strip_prefix(line: &str) -> &str {
    let t = line.trim_end();
    if let Some(rest) = t.strip_prefix("==")
        && let Some((pid, tail)) = rest.split_once("==")
        && !pid.is_empty()
        && pid.bytes().all(|b| b.is_ascii_digit())
    {
        return tail.strip_prefix(' ').unwrap_or(tail);
    }
    t
}

fn check_error(text: &str, what: &str) -> Result<(), ParseError> {
    for line in text.lines() {
        let l = strip_prefix(line).trim();
        if l == "invalid loss record nr"
            || l == "missing or malformed address"
            || l.starts_with("unknown command")
            || l.starts_with("A leak check must be done first")
            || l.starts_with("no previous leak search")
        {
            return Err(ParseError::new(format!(
                "vgdb {what} failed: {l:?} (run `leak_check full reachable any` first and use a record number it printed)"
            )));
        }
    }
    Ok(())
}

fn number(text: &str) -> Option<u64> {
    let t: String = text.chars().filter(|c| *c != ',').collect();
    if t.is_empty() {
        return None;
    }
    t.parse().ok()
}

fn signed(text: &str) -> Option<i64> {
    let t = text.trim();
    let (neg, digits) = match t.strip_prefix('-') {
        Some(d) => (true, d),
        None => (false, t.strip_prefix('+').unwrap_or(t)),
    };
    let v = i64::try_from(number(digits)?).ok()?;
    Some(if neg { -v } else { v })
}

/// `1,088 (+704) bytes in 17 (+11) blocks are still reachable in loss record 1 of 1`.
fn parse_record_header(line: &str) -> Option<LossRecord> {
    let (left, rest) = line.split_once(" bytes in ")?;
    let (blocks_part, rest) = rest.split_once(" blocks are ")?;
    let (kind_text, rest) = rest.split_once(" in loss record ")?;
    let (nr, of) = rest.split_once(" of ")?;
    let kind = match kind_text.trim() {
        "definitely lost" => "definitely-lost",
        "indirectly lost" => "indirectly-lost",
        "possibly lost" => "possibly-lost",
        "still reachable" => "still-reachable",
        _ => return None,
    };
    let (bytes_text, bytes_paren) = split_paren(left);
    let (blocks_text, blocks_paren) = split_paren(blocks_part);
    let mut rec = LossRecord {
        nr: nr.trim().parse().ok()?,
        of: of.trim().parse().ok()?,
        kind: kind.to_owned(),
        bytes: number(bytes_text)?,
        blocks: number(blocks_text)?,
        ..LossRecord::default()
    };
    if let Some(p) = bytes_paren {
        if p.contains(" direct") {
            let (direct, indirect) = p.split_once(", ").unwrap_or((p, ""));
            let first = |t: &str| t.trim().split(' ').next().and_then(number);
            rec.direct_bytes = first(direct);
            rec.indirect_bytes = first(indirect);
        } else {
            rec.bytes_delta = signed(p);
        }
    }
    if let Some(p) = blocks_paren {
        rec.blocks_delta = signed(p);
    }
    Some(rec)
}

/// `"1,088 (+704)"` into `("1,088", Some("+704"))`.
fn split_paren(text: &str) -> (&str, Option<&str>) {
    let t = text.trim();
    match t.split_once(" (") {
        Some((head, tail)) => (head, tail.strip_suffix(')')),
        None => (t, None),
    }
}

/// `at 0x4846828: malloc (in /usr/libexec/valgrind/vgpreload_memcheck-amd64-linux.so)`,
/// `by 0x109261: add_session (leak.c:11)`.
fn parse_stack_line(line: &str) -> Option<Frame> {
    let t = line.trim();
    let body = t.strip_prefix("at ").or_else(|| t.strip_prefix("by "))?;
    let body = body.strip_prefix("0x")?;
    let (_, body) = body.split_once(": ")?;
    if let Some(open) = body.rfind(" (")
        && let Some(inner) = body[open + 2..].strip_suffix(')')
    {
        let function = body[..open].trim();
        if let Some(module) = inner.strip_prefix("in ") {
            return Some(make_frame(
                function,
                None,
                None,
                Some(module.trim().to_owned()),
            ));
        }
        let (file, line_no) = split_location(inner);
        return Some(make_frame(function, Some(file), line_no, None));
    }
    Some(make_frame(body.trim(), None, None, None))
}

fn drop_runtime_frames(frames: &mut Vec<Frame>) {
    let skip = frames
        .iter()
        .take_while(|f| {
            f.module
                .as_deref()
                .is_some_and(|m| m.contains("/vgpreload_"))
        })
        .count();
    if skip < frames.len() {
        frames.drain(..skip);
    }
}

/// Parses `leak_check` output.
pub fn parse_leak_check(text: &str) -> Result<LeakCheck, ParseError> {
    check_error(text, "leak_check")?;
    let mut records: Vec<LossRecord> = Vec::new();
    let mut saw_summary = false;
    for line in text.lines() {
        let l = strip_prefix(line);
        if let Some(rec) = parse_record_header(l.trim()) {
            records.push(rec);
        } else if let Some(frame) = parse_stack_line(l) {
            if let Some(rec) = records.last_mut() {
                rec.frames.push(frame);
            }
        } else if l.trim() == "LEAK SUMMARY:" {
            saw_summary = true;
        }
    }
    if records.is_empty() && !saw_summary {
        return Err(ParseError::new(
            "not leak_check output: no loss record and no LEAK SUMMARY (send `leak_check full reachable any` through vgdb)",
        ));
    }
    for r in &mut records {
        drop_runtime_frames(&mut r.frames);
    }
    Ok(LeakCheck { records })
}

/// Parses `block_list <loss_record_nr>` output.
pub fn parse_block_list(text: &str) -> Result<BlockList, ParseError> {
    check_error(text, "block_list")?;
    let mut record: Option<LossRecord> = None;
    let mut blocks = Vec::new();
    for line in text.lines() {
        let l = strip_prefix(line);
        let t = l.trim();
        if let Some(rec) = parse_record_header(t) {
            record = Some(rec);
        } else if let Some(frame) = parse_stack_line(t) {
            if let Some(rec) = record.as_mut() {
                rec.frames.push(frame);
            }
        } else if let Some(b) = parse_block_ref(t) {
            blocks.push(b);
        }
    }
    let Some(mut record) = record else {
        return Err(ParseError::new(
            "not block_list output: no loss record header (expected \"N bytes in M blocks are ... in loss record X of Y\")",
        ));
    };
    drop_runtime_frames(&mut record.frames);
    Ok(BlockList { record, blocks })
}

/// `0x4A7B040[1008]`.
fn parse_block_ref(t: &str) -> Option<BlockRef> {
    let rest = t.strip_prefix("0x")?;
    let (addr, size) = rest.split_once('[')?;
    let size = size.strip_suffix(']')?;
    Some(BlockRef {
        address: u64::from_str_radix(addr, 16).ok()?,
        size: size.parse().ok()?,
    })
}

fn parse_hex(text: &str) -> Option<u64> {
    u64::from_str_radix(text.strip_prefix("0x")?, 16).ok()
}

/// Parses `who_points_at <addr> [len]` output. A block nobody points at gives no refs.
pub fn parse_who_points_at(text: &str) -> Result<WhoPointsAt, ParseError> {
    check_error(text, "who_points_at")?;
    let mut out: Option<WhoPointsAt> = None;
    // The reference being described by following `Address ...` and stack lines.
    let mut open: Option<(u64, Option<Holder>)> = None;
    fn close(open: &mut Option<(u64, Option<Holder>)>, out: &mut Option<WhoPointsAt>) {
        if let (Some((from, holder)), Some(o)) = (open.take(), out.as_mut()) {
            let holder = holder.unwrap_or(Holder::Other {
                description: "no description".to_owned(),
            });
            o.refs.push(PointerRef { from, holder });
        }
    }
    for line in text.lines() {
        let l = strip_prefix(line);
        let t = l.trim();
        if let Some(rest) = t.strip_prefix("Searching for pointers pointing in ") {
            // `N bytes from 0xADDR`
            if let Some((len, addr)) = rest.split_once(" bytes from ") {
                out = Some(WhoPointsAt {
                    target: parse_hex(addr.trim()).ok_or_else(|| bad_target(t))?,
                    len: len.trim().parse().ok(),
                    refs: Vec::new(),
                });
            }
        } else if let Some(addr) = t.strip_prefix("Searching for pointers to ") {
            out = Some(WhoPointsAt {
                target: parse_hex(addr.trim()).ok_or_else(|| bad_target(t))?,
                len: None,
                refs: Vec::new(),
            });
        } else if let Some(rest) = t.strip_prefix('*') {
            close(&mut open, &mut out);
            if let Some((from, _)) = rest.split_once(" points at ") {
                open = Some((parse_hex(from).ok_or_else(|| bad_target(t))?, None));
            }
        } else if t.starts_with("Address 0x") {
            if let Some((_, holder)) = open.as_mut() {
                *holder = Some(parse_description(t));
            }
        } else if let Some(frame_text) = t.strip_prefix("in frame #") {
            if let Some((_, Some(Holder::Stack { frame, .. }))) = open.as_mut() {
                *frame = Some(format!("frame #{frame_text}"));
            }
        } else if let Some(frame) = parse_stack_line(t)
            && let Some((_, Some(Holder::Block { frames, .. }))) = open.as_mut()
        {
            frames.push(frame);
        }
    }
    close(&mut open, &mut out);
    let Some(mut out) = out else {
        return Err(ParseError::new(
            "not who_points_at output: no \"Searching for pointers to 0xADDR\" line (send `who_points_at 0xADDR` through vgdb)",
        ));
    };
    for r in &mut out.refs {
        if let Holder::Block { frames, .. } = &mut r.holder {
            drop_runtime_frames(frames);
        }
    }
    Ok(out)
}

fn bad_target(line: &str) -> ParseError {
    ParseError::new(format!(
        "unreadable address in vgdb output: {line:?} (expected 0x<hex>)"
    ))
}

/// `Address 0x10c030 is 0 bytes inside data symbol "registry"` and its siblings.
fn parse_description(t: &str) -> Holder {
    let after_addr = t.strip_prefix("Address ").unwrap_or(t);
    let (addr, desc) = after_addr.split_once(" is ").unwrap_or((after_addr, ""));
    let address = parse_hex(addr).unwrap_or(0);
    let other = || Holder::Other {
        description: desc.to_owned(),
    };
    if let Some((off, rest)) = desc.split_once(" bytes inside ") {
        let offset = number(off).unwrap_or(0);
        if let Some(sym) = rest
            .strip_prefix("data symbol \"")
            .and_then(|s| s.strip_suffix('"'))
        {
            return Holder::DataSymbol {
                symbol: demangle_or_raw(sym),
                offset,
            };
        }
        if let Some(size) = rest
            .strip_prefix("a block of size ")
            .and_then(|s| s.strip_suffix(" alloc'd"))
        {
            return Holder::Block {
                address,
                offset,
                size: number(size).unwrap_or(0),
                frames: Vec::new(),
            };
        }
        return other();
    }
    if let Some(rest) = desc.strip_prefix("on thread ") {
        let thread = rest
            .split(|c: char| !c.is_ascii_digit())
            .next()
            .and_then(|n| n.parse().ok())
            .unwrap_or(0);
        return Holder::Stack {
            thread,
            frame: None,
        };
    }
    if desc.starts_with("in ") && desc.contains("segment") {
        return Holder::Segment {
            description: desc.trim_start_matches("in ").to_owned(),
        };
    }
    other()
}

// ---------------------------------------------------------------------------------------
// Retention chain
// ---------------------------------------------------------------------------------------

/// Where a walk up the `who_points_at` answers stands after one answer.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Step {
    /// A data symbol, stack or segment ends the chain.
    Root(Holder),
    /// A heap block keeps the queried block alive; query `start` next.
    Block { start: u64, holder: Holder },
    /// Nothing points at the queried block (it is leaked outright).
    Unreferenced,
}

/// Picks the holder to follow from one answer. A data symbol wins, then a heap block (the walk
/// continues), then a stack or segment.
pub fn step(answer: &WhoPointsAt) -> Step {
    let first = |pred: fn(&Holder) -> bool| answer.refs.iter().find(|r| pred(&r.holder));
    if let Some(r) = first(|h| matches!(h, Holder::DataSymbol { .. })) {
        return Step::Root(r.holder.clone());
    }
    if let Some(r) = first(|h| matches!(h, Holder::Block { .. }))
        && let Holder::Block {
            address, offset, ..
        } = &r.holder
    {
        return Step::Block {
            start: address.saturating_sub(*offset),
            holder: r.holder.clone(),
        };
    }
    match first(|h| matches!(h, Holder::Stack { .. } | Holder::Segment { .. })) {
        Some(r) => Step::Root(r.holder.clone()),
        None => Step::Unreferenced,
    }
}

/// The block start to query next, or `None` when the walk is over.
pub fn next_block_to_query(answer: &WhoPointsAt) -> Option<u64> {
    match step(answer) {
        Step::Block { start, .. } => Some(start),
        _ => None,
    }
}

/// What keeps a block alive, from the walk of several `who_points_at` answers.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct RetentionChain {
    /// The data symbol, stack or segment the chain ends in; `None` if the walk did not reach one.
    pub root: Option<Holder>,
    /// Heap blocks between the root and the queried block, outermost first. Neighbours with
    /// the same size and allocation site are merged and counted.
    pub blocks: Vec<ChainBlock>,
    /// The root was reached, or nothing points at the end of the chain (a lost block).
    pub complete: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ChainBlock {
    pub size: u64,
    pub frames: Vec<Frame>,
    /// How many consecutive blocks look the same (a linked list).
    pub count: u64,
}

/// Builds the chain from the answers of a walk, in the order they were asked: the answer for
/// the queried block first, then for each block [`next_block_to_query`] named.
pub fn build_chain(answers: &[WhoPointsAt]) -> RetentionChain {
    let mut chain = RetentionChain::default();
    let mut inner_first: Vec<ChainBlock> = Vec::new();
    for answer in answers {
        match step(answer) {
            Step::Root(h) => {
                chain.root = Some(h);
                chain.complete = true;
                break;
            }
            Step::Unreferenced => {
                chain.complete = true;
                break;
            }
            Step::Block { holder, .. } => {
                if let Holder::Block { size, frames, .. } = holder {
                    match inner_first.last_mut() {
                        Some(last) if last.size == size && last.frames == frames => last.count += 1,
                        _ => inner_first.push(ChainBlock {
                            size,
                            frames,
                            count: 1,
                        }),
                    }
                }
            }
        }
    }
    inner_first.reverse();
    chain.blocks = inner_first;
    chain
}

impl RetentionChain {
    /// The chain as `root_path` entries of an [`crate::model::AllocGroup`], outermost first:
    /// `static registry` (a data symbol, with `+offset` inside it), `stack of thread 1`,
    /// then `block <function> (<file>:<line>)` per heap hop, with `x<count>` when merged.
    pub fn root_path(&self) -> Vec<String> {
        let mut path = Vec::new();
        match &self.root {
            Some(Holder::DataSymbol { symbol, offset }) if *offset > 0 => {
                path.push(format!("static {symbol}+{offset}"));
            }
            Some(Holder::DataSymbol { symbol, .. }) => path.push(format!("static {symbol}")),
            Some(Holder::Stack { thread, .. }) => path.push(format!("stack of thread {thread}")),
            Some(Holder::Segment { description }) => path.push(format!("segment {description}")),
            Some(Holder::Other { description }) => path.push(description.clone()),
            Some(Holder::Block { .. }) | None => {}
        }
        for b in &self.blocks {
            let site = match b.frames.first() {
                Some(f) => match (&f.file, f.line) {
                    (Some(file), Some(line)) => format!("{} ({file}:{line})", f.function),
                    _ => f.function.clone(),
                },
                None => "unknown site".to_owned(),
            };
            if b.count > 1 {
                path.push(format!("block {site} x{}", b.count));
            } else {
                path.push(format!("block {site}"));
            }
        }
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headers_in_all_shapes() {
        let r = parse_record_header(
            "1,088 (+704) bytes in 17 (+11) blocks are still reachable in loss record 1 of 1",
        )
        .unwrap();
        assert_eq!(
            (r.bytes, r.bytes_delta, r.blocks, r.blocks_delta),
            (1088, Some(704), 17, Some(11))
        );
        let r = parse_record_header("1,232 (16 direct, 1,216 indirect) bytes in 1 blocks are definitely lost in loss record 3 of 4").unwrap();
        assert_eq!(
            (r.bytes, r.direct_bytes, r.indirect_bytes, r.nr, r.of),
            (1232, Some(16), Some(1216), 3, 4)
        );
        assert_eq!(r.kind, "definitely-lost");
        assert!(parse_record_header("hello").is_none());
        assert!(
            parse_record_header("1 bytes in 1 blocks are mysterious in loss record 1 of 1")
                .is_none()
        );
    }

    #[test]
    fn prefixes() {
        assert_eq!(strip_prefix("==12== abc"), "abc");
        assert_eq!(strip_prefix(" Address 0x1 is"), " Address 0x1 is");
        assert_eq!(strip_prefix("==x== abc"), "==x== abc");
    }
}
