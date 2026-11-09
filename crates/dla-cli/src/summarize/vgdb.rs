//! `heap-graph` over Valgrind gdbserver answers: what keeps the biggest still-reachable
//! (else definitely-lost) loss record alive.
//!
//! The harness (`dla harness run <H> --tool valgrind`) asks the running program, through
//! `vgdb`, for `leak_check full reachable any`, `block_list <n>` for the top record, and a
//! walk of `who_points_at` answers up from one of its blocks, and leaves the answers next
//! to the XML (`out/<label>/vgdb-*.txt`). The files of one directory are one point. This
//! reader orders each point's answers into a chain, names the root (a data symbol, a thread
//! stack or a mapped segment) and, from three or more points, follows the record's bytes
//! under the root's key.
//!
//! * The holder is a data symbol (`static/global`): the key is `static:<owner>` with the
//!   owner the demangled symbol (`crate::module::NAME` becomes `module.NAME`, the crate
//!   dropped; a C symbol stays bare).
//! * `holder:<key>` columns tie by holder identity (rule 1 of the tie rule) to the census
//!   finding whose owner ends with the symbol's module path and name, when exactly one
//!   finding matches. A bare name (a C static, a static in `main.rs` or `lib.rs`) ties only
//!   when exactly one finding in the run ends with it. Never by a bare name that several
//!   findings share.
//! * A holder-tied record's first project frame becomes a writer of the finding.
//! * A thread stack or an unnamed segment as the root has no owner: it is reported, and no
//!   column is tied.

use std::{collections::BTreeMap, path::Path};

use dla_core::{
    FindingId, Ledger,
    attribution::canonical,
    evidence::{
        Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, Signal, Source, tool_class,
    },
    record::FindingAnnotation,
    series::{SeriesRegion, SeriesSourceKind, Unit},
    toolctx::EvidenceSpec,
};
use dla_parsers::{
    model::{AllocProfile, Frame},
    native::{
        demangle::demangle_or_raw,
        valgrind as xml,
        vgdb::{self, BlockList, Holder, LeakCheck, LossRecord, RetentionChain, WhoPointsAt},
    },
};
use serde_json::{Map, Value, json};

use super::{
    Adapter, AdapterOut, CandidateRow, Ctx, Op, Opts, Recorded, emit, growth,
    input::Input,
    rule,
    tie::{self, TieReason},
    valgrind::Rebase,
};
use crate::fail::{Failure, Res};

/// Largest answer file read.
const MAX_BYTES: u64 = 16 << 20;
/// Most points shown in the command's output.
const SHOWN_POINTS: usize = 3;

/// The vgdb answer reader.
pub struct ValgrindVgdb;

/// One command's answer inside a file.
enum Answer {
    Leak(LeakCheck),
    Blocks(BlockList),
    Who(WhoPointsAt),
}

/// Splits a file into the commands' answers. A file without `sending command` lines is one
/// answer whose command is guessed from its content.
fn answers_of(text: &str, notes: &mut Vec<String>, shown: &str) -> Vec<Answer> {
    let mut segments: Vec<(String, String)> = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("sending command ") {
            let cmd = rest.split_whitespace().next().unwrap_or("").to_owned();
            segments.push((cmd, String::new()));
        } else if let Some((_, body)) = segments.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if segments.is_empty() {
        let cmd = if text.contains("Searching for pointers") {
            "who_points_at"
        } else if text.contains("loss record") && text.contains("]\n") {
            "block_list"
        } else {
            "leak_check"
        };
        segments.push((cmd.to_owned(), text.to_owned()));
    }
    let mut out = Vec::new();
    for (cmd, body) in segments {
        let parsed = match cmd.as_str() {
            "leak_check" => vgdb::parse_leak_check(&body).map(Answer::Leak),
            "block_list" => vgdb::parse_block_list(&body).map(Answer::Blocks),
            "who_points_at" => vgdb::parse_who_points_at(&body).map(Answer::Who),
            other => {
                notes.push(format!("{shown}: command {other} is not read"));
                continue;
            }
        };
        match parsed {
            Ok(a) => out.push(a),
            Err(e) => notes.push(format!("{shown}: {cmd}: {}", e.message())),
        }
    }
    out
}

/// The loss records of a memcheck XML as a `leak_check` answer.
fn check_of_xml(profile: &AllocProfile) -> LeakCheck {
    let of = u32::try_from(profile.groups.len()).unwrap_or(u32::MAX);
    LeakCheck {
        records: profile
            .groups
            .iter()
            .enumerate()
            .map(|(i, g)| LossRecord {
                nr: u32::try_from(i + 1).unwrap_or(u32::MAX),
                of,
                kind: g.kind.clone().unwrap_or_default(),
                bytes: g.live_bytes,
                blocks: g.live_blocks,
                frames: g.frames.clone(),
                ..LossRecord::default()
            })
            .collect(),
    }
}

/// The loss record the harness follows: the biggest still-reachable one, else the biggest
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

/// The answers of a walk, in the order they were asked: from the one nobody else points
/// to, following the block each answer names next.
fn order_chain(answers: Vec<WhoPointsAt>, blocks: Option<&BlockList>) -> Vec<WhoPointsAt> {
    if answers.len() < 2 {
        return answers;
    }
    let nexts: Vec<Option<u64>> = answers.iter().map(vgdb::next_block_to_query).collect();
    let is_next = |t: u64| nexts.iter().flatten().any(|n| *n == t);
    let in_record = |t: u64| blocks.is_some_and(|b| b.blocks.iter().any(|x| x.address == t));
    let start = answers
        .iter()
        .position(|a| !is_next(a.target) && in_record(a.target))
        .or_else(|| answers.iter().position(|a| !is_next(a.target)))
        .unwrap_or(0);
    let mut order = vec![start];
    while let Some(n) = nexts[*order.last().unwrap_or(&start)] {
        match answers
            .iter()
            .position(|a| a.target == n)
            .filter(|i| !order.contains(i))
        {
            Some(i) => order.push(i),
            None => break,
        }
    }
    // Answers off the chain are kept after it, in file order.
    for i in 0..answers.len() {
        if !order.contains(&i) {
            order.push(i);
        }
    }
    let mut slots: Vec<Option<WhoPointsAt>> = answers.into_iter().map(Some).collect();
    order.into_iter().filter_map(|i| slots[i].take()).collect()
}

/// One point: the answers of one directory.
struct Point {
    label: String,
    first: Input,
    sha256: String,
    files: usize,
    record: Option<LossRecord>,
    chain: Option<RetentionChain>,
    walked: usize,
    notes: Vec<String>,
}

/// The demangled data symbol as an owner: `crate::module::NAME` is `crate.module.NAME`.
fn symbol_display(symbol: &str) -> String {
    demangle_or_raw(symbol).replace("::", ".")
}

/// The data symbol's key and owner words: `(display, segments after the crate)`. A Rust
/// symbol (`::`) starts with its crate, which census owners do not carry; a C symbol is bare.
fn symbol_parts(symbol: &str) -> Option<(String, Vec<String>, bool)> {
    let d = demangle_or_raw(symbol);
    if d.is_empty() || d.contains(['<', '>', '{', ' ', '(']) {
        return None;
    }
    let rust = d.contains("::");
    let segs: Vec<String> = d.split("::").map(str::to_owned).collect();
    let display = segs.join(".");
    let rest: Vec<String> = if rust {
        segs.into_iter().skip(1).collect()
    } else {
        segs
    };
    (!rest.is_empty()).then_some((display, rest, rust))
}

/// The finding of a data symbol: the one finding whose owner ends with the symbol's module
/// path and name, and whose declaration site is where that symbol is defined (see
/// [`defined_at`]). A bare name (a C static, or a Rust static in the root module) needs
/// exactly one finding that ends with it; for the root module the owner's module must be
/// `main` or `lib`. Several suffix matches give none, listed as ambiguous; one suffix match
/// that is not the symbol's definition gives none.
fn finding_for_symbol(
    ledger: &Ledger,
    who: &tie::Who,
    project: &Path,
    symbol: &str,
) -> (Option<(FindingId, TieReason)>, Vec<FindingId>) {
    let Some((display, rest, rust)) = symbol_parts(symbol) else {
        return (None, Vec::new());
    };
    let name = rest.last().cloned().unwrap_or_default();
    let tail = rest.join(".");
    let hits: Vec<_> = ledger
        .findings()
        .filter(|f| who.sees(f))
        .filter(|f| {
            let Some(o) = f.owner.as_deref().and_then(canonical) else {
                return false;
            };
            let ends = |t: &str| o == t || o.ends_with(&format!(".{t}"));
            if rest.len() >= 2 {
                ends(&tail)
            } else if rust {
                ends(&format!("main.{name}")) || ends(&format!("lib.{name}"))
            } else {
                ends(&name)
            }
        })
        .collect();
    match hits.as_slice() {
        [one] if defined_at(project, one.site.as_deref(), symbol, &rest, rust) => (
            Some((one.id.clone(), TieReason::holder(&display))),
            Vec::new(),
        ),
        [_] | [] => (None, Vec::new()),
        _ => (None, hits.iter().map(|f| f.id.clone()).collect()),
    }
}

/// True when the finding's declaration site is where the symbol is defined, so a suffix of
/// the owner is not taken for an identity.
///
/// * Rust (`krate::m1::m2::NAME`): the site file is `<dir>/src/m1/m2.rs` (or `m1/m2/mod.rs`;
///   `main.rs` or `lib.rs` for the root module), and the `Cargo.toml` of `<dir>` names the
///   crate `krate`. A static of another crate, or of another directory of the same name,
///   does not match.
/// * C (a bare name): the site file is the only C source of the project that defines the
///   name at file scope.
fn defined_at(
    project: &Path,
    site: Option<&str>,
    symbol: &str,
    rest: &[String],
    rust: bool,
) -> bool {
    let Some(file) = site
        .and_then(|s| s.rsplit_once(':').map(|(f, _)| f).or(Some(s)))
        .map(|f| f.replace('\\', "/"))
        .map(|f| f.trim_start_matches("./").to_owned())
        .filter(|f| !f.is_empty())
    else {
        return false;
    };
    if rust {
        let d = demangle_or_raw(symbol);
        let Some(krate) = d.split("::").next() else {
            return false;
        };
        rust_defined_at(project, &file, krate, &rest[..rest.len().saturating_sub(1)])
    } else {
        rest.last()
            .is_some_and(|name| c_defined_only_at(project, &file, name))
    }
}

fn rust_defined_at(project: &Path, file: &str, krate: &str, mods: &[String]) -> bool {
    let Some(stem) = file.strip_suffix(".rs") else {
        return false;
    };
    let mut comps: Vec<&str> = stem.split('/').filter(|c| !c.is_empty()).collect();
    if mods.is_empty() {
        if !matches!(comps.last(), Some(&"main" | &"lib")) {
            return false;
        }
        comps.pop();
    } else {
        if comps.last() == Some(&"mod") {
            comps.pop();
        }
        if comps.len() < mods.len()
            || !comps[comps.len() - mods.len()..]
                .iter()
                .zip(mods)
                .all(|(c, m)| c == m)
        {
            return false;
        }
        comps.truncate(comps.len() - mods.len());
    }
    if comps.last() != Some(&"src") {
        return false;
    }
    comps.pop();
    let manifest = project.join(comps.join("/")).join("Cargo.toml");
    let Ok(text) = std::fs::read_to_string(manifest) else {
        return false;
    };
    crate_names(&text)
        .iter()
        .any(|n| n.replace('-', "_") == krate)
}

/// The crate names a manifest gives: `[package] name` and `[lib] name`.
fn crate_names(manifest: &str) -> Vec<String> {
    let mut section = "";
    let mut names = Vec::new();
    for line in manifest.lines() {
        let t = line.trim();
        if t.starts_with('[') {
            section = t;
        } else if matches!(section, "[package]" | "[lib]")
            && let Some((k, v)) = t.split_once('=')
            && k.trim() == "name"
        {
            names.push(v.trim().trim_matches('"').to_owned());
        }
    }
    names
}

/// Most source files looked at, and largest one, when searching for a C definition.
const C_SCAN_FILES: usize = 20_000;
const C_SCAN_BYTES: u64 = 4 << 20;

/// True when `file` is the only C or C++ source under `project` that defines `name` at
/// file scope (a line that starts in column 0, holds a type before the name, and has `=`,
/// `;`, `[` or `,` after it).
fn c_defined_only_at(project: &Path, file: &str, name: &str) -> bool {
    let mut defining: Vec<String> = Vec::new();
    let mut seen = 0usize;
    let mut stack = vec![project.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in rd.flatten() {
            let path = e.path();
            let fname = e.file_name().to_string_lossy().into_owned();
            let Ok(ft) = e.file_type() else { continue };
            if ft.is_dir() {
                if !matches!(fname.as_str(), ".git" | ".dla" | "target" | "node_modules") {
                    stack.push(path);
                }
                continue;
            }
            let ext = path.extension().and_then(|x| x.to_str()).unwrap_or("");
            if !matches!(ext, "c" | "h" | "cc" | "cpp" | "cxx" | "hpp" | "hh") {
                continue;
            }
            seen += 1;
            if seen > C_SCAN_FILES {
                return false;
            }
            if e.metadata().map_or(true, |m| m.len() > C_SCAN_BYTES) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if text.lines().any(|l| declares(l, name))
                && let Ok(rel) = path.strip_prefix(project)
            {
                defining.push(rel.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    defining.len() == 1 && defining[0] == file
}

/// True when a source line declares the file-scope variable `name`.
fn declares(line: &str, name: &str) -> bool {
    if line.is_empty() || line.starts_with([' ', '\t', '#', '/', '*', '}', '{']) {
        return false;
    }
    let ident = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let mut from = 0;
    while let Some(at) = line[from..].find(name) {
        let (start, end) = (from + at, from + at + name.len());
        from = end;
        let before = &line[..start];
        let after = line[end..].trim_start();
        let boundary = !before.ends_with(ident) && !line[end..].starts_with(ident);
        if boundary
            && !before.trim().is_empty()
            && !before.contains(['(', '=', '.', '>'])
            && after.starts_with(['=', ';', '[', ','])
        {
            return true;
        }
    }
    false
}

/// Records `site` as a writer of `f` (once).
fn add_writer(ledger: &mut Ledger, f: &FindingId, site: &str) -> Res<()> {
    let known = ledger
        .annotation(f)
        .is_some_and(|a| a.writers.iter().any(|w| w == site))
        || ledger
            .finding(f)
            .is_some_and(|x| x.site.as_deref() == Some(site));
    if known {
        return Ok(());
    }
    let mut a = FindingAnnotation::new(f.clone());
    a.writers.push(site.to_owned());
    ledger
        .annotate_record(a)
        .map_err(|e| Failure::from(e.to_refusal()))?;
    Ok(())
}

fn short_kind(k: &str) -> &str {
    match k {
        "still-reachable" => "reachable",
        "definitely-lost" => "lost",
        "indirectly-lost" => "indirect",
        "possibly-lost" => "possible",
        other => other,
    }
}

/// The root of a chain as text and as the key of a holder column.
fn root_of(chain: &RetentionChain) -> (String, Option<String>) {
    match &chain.root {
        Some(Holder::DataSymbol { symbol, .. }) => (
            format!("static {}", demangle_or_raw(symbol)),
            Some(format!("static:{}", symbol_display(symbol))),
        ),
        Some(Holder::Stack { thread, .. }) => (format!("stack of thread {thread}"), None),
        Some(Holder::Segment { description }) => (format!("segment {description}"), None),
        Some(Holder::Other { description }) => (description.clone(), None),
        Some(Holder::Block { .. }) | None => ("no root found".to_owned(), None),
    }
}

impl Adapter for ValgrindVgdb {
    fn name(&self) -> &'static str {
        "valgrind-vgdb"
    }
    fn op(&self) -> Op {
        Op::HeapGraph
    }
    fn runtime(&self) -> &'static str {
        "native"
    }
    fn tool_class(&self) -> &'static str {
        tool_class::LEAK_CHECKER
    }
    fn formats(&self) -> &'static str {
        "vgdb answers (leak_check, block_list, who_points_at) in text files, one directory per point: out/<label>/vgdb-*.txt, with the point's memcheck XML as the loss records"
    }
    fn sniff(&self, input: &Input) -> bool {
        let h = input.head_text();
        h.starts_with("sending command ")
            || h.contains("Searching for pointers to")
            || h.contains("<valgrindoutput>")
            || (h.contains("in loss record") && h.contains("=="))
    }
    fn run(&self, ctx: &Ctx<'_>, inputs: &[Input], opts: &Opts) -> Res<AdapterOut> {
        let tc = ctx.tc;
        let red = &ctx.redactor;
        let rebase = Rebase::new(&tc.project);
        // ---- points: one per directory, in the order given
        let mut groups: Vec<(std::path::PathBuf, Vec<&Input>)> = Vec::new();
        for i in inputs {
            let dir = i
                .abs
                .parent()
                .map(std::path::Path::to_path_buf)
                .unwrap_or_default();
            match groups.iter_mut().find(|(d, _)| *d == dir) {
                Some((_, v)) => v.push(i),
                None => groups.push((dir, vec![i])),
            }
        }
        let mut points: Vec<Point> = Vec::new();
        for (dir, mut files) in groups {
            files.sort_by_key(|i| i.name_lower());
            let mut notes = Vec::new();
            let mut checks: Vec<LeakCheck> = Vec::new();
            let mut lists: Vec<BlockList> = Vec::new();
            let mut whos: Vec<WhoPointsAt> = Vec::new();
            for f in &files {
                let text = f.read_text(MAX_BYTES)?;
                if text.contains("<valgrindoutput>") {
                    // The memcheck XML of the point: the loss records, with no holder.
                    match xml::parse(&text) {
                        Ok(r) => checks.insert(0, check_of_xml(&r.profile)),
                        Err(e) => notes.push(format!("{}: {}", f.shown(), e.message())),
                    }
                    continue;
                }
                for a in answers_of(&text, &mut notes, f.shown()) {
                    match a {
                        Answer::Leak(c) => checks.push(c),
                        Answer::Blocks(b) => lists.push(b),
                        Answer::Who(w) => whos.push(w),
                    }
                }
            }
            if checks.is_empty() && lists.is_empty() && whos.is_empty() {
                return Err(Failure::refuse(
                    "malformed-artifact",
                    format!(
                        "{} holds no readable vgdb answer{}",
                        files[0].shown(),
                        notes.first().map_or(String::new(), |n| format!(" ({n})"))
                    ),
                    "vgdb output of leak_check, block_list or who_points_at",
                    "dla harness run <H> --tool valgrind   writes out/<label>/vgdb-*.txt",
                ));
            }
            let blocks = lists.pop();
            let check = checks.pop();
            let mut record = blocks
                .as_ref()
                .map(|b| b.record.clone())
                .or_else(|| check.as_ref().and_then(top_record).cloned());
            if let (Some(r), Some(c)) = (record.as_mut(), check.as_ref()) {
                // The same record in the full check, when the numbers agree, has the full stack.
                if r.frames.is_empty()
                    && let Some(full) = c.records.iter().find(|x| x.nr == r.nr)
                {
                    r.frames.clone_from(&full.frames);
                }
            }
            if let Some(r) = record.as_mut() {
                for f in &mut r.frames {
                    rebase.frame(f);
                }
            }
            let walked = whos.len();
            let chain =
                (walked > 0).then(|| vgdb::build_chain(&order_chain(whos, blocks.as_ref())));
            let sha256 =
                emit::combined_sha(&files.iter().map(|f| f.sha256.as_str()).collect::<Vec<_>>());
            points.push(Point {
                label: dir
                    .file_name()
                    .map_or_else(|| "point".to_owned(), |n| n.to_string_lossy().into_owned()),
                first: files[0].clone(),
                sha256,
                files: files.len(),
                record,
                chain,
                walked,
                notes,
            });
        }

        let mut ledger_ref = ctx.ledger()?;
        let ledger = &mut *ledger_ref;
        let authority = ctx.authority(Authority::Live);
        let mut out = AdapterOut::default();
        let mut recorded: Vec<Recorded> = Vec::new();
        let mut last_raw: Option<emit::RawRef> = None;
        let mut point_json: Vec<Value> = Vec::new();
        let mut keys_by_point: Vec<Option<String>> = Vec::new();
        let who = tie::Who::new(tc.worker.as_deref(), None);
        // ---- one retention-path evidence per point
        for p in &points {
            let (root_text, key) = p
                .chain
                .as_ref()
                .map_or(("no chain".to_owned(), None), root_of);
            keys_by_point.push(key.clone());
            let path: Vec<String> = p
                .chain
                .as_ref()
                .map(|c| c.root_path().into_iter().skip(1).collect())
                .unwrap_or_default();
            let top = p.record.as_ref().map(|r| {
                json!({
                    "nr": r.nr,
                    "kind": r.kind,
                    "bytes": r.bytes,
                    "blocks": r.blocks,
                    "frames": r.frames.iter().take(6).map(|f| json!({
                        "function": red.name(&f.function),
                        "file": f.file.as_deref().map(|x| red.name(x)),
                        "line": f.line,
                    })).collect::<Vec<_>>(),
                })
            });
            let raw_value = json!({
                "point": p.label,
                "files": p.files,
                "record": top,
                "root": red.name(&root_text),
                "key": key.as_deref().map(|k| red.name(k)),
                "complete": p.chain.as_ref().is_some_and(|c| c.complete),
                "path": path.iter().map(|s| red.name(s)).collect::<Vec<_>>(),
                "walked": p.walked,
                "notes": p.notes.iter().map(|n| red.name(n)).collect::<Vec<_>>(),
                "redaction": red.mode(),
            });
            let rel = format!(
                "summaries/vgdb-{}-{}.json",
                emit::clean_stem(&p.label),
                &p.sha256[..8]
            );
            let raw = emit::write_raw(tc, &rel, &raw_value)?;
            let mut result = json!({
                "point": clip(&red.name(&p.label), 40),
                "root": clip(&red.name(&root_text), 80),
                "complete": p.chain.as_ref().is_some_and(|c| c.complete),
                "walked": p.walked,
                "path": path.iter().take(3).map(|s| clip(&red.name(s), 70)).collect::<Vec<_>>(),
            });
            if let Some(r) = &p.record {
                result["record"] = json!({
                    "nr": r.nr, "kind": r.kind, "bytes": r.bytes, "blocks": r.blocks,
                });
                if let Some(f) = r.frames.first() {
                    result["alloc"] = json!(clip(
                        &red.name(&format!(
                            "{} ({}:{})",
                            f.function,
                            f.file.as_deref().unwrap_or("?"),
                            f.line.unwrap_or(0)
                        )),
                        90
                    ));
                }
            }
            let mut spec = EvidenceSpec::new(
                EvidenceKind::RetentionPath,
                "valgrind-vgdb",
                env!("CARGO_PKG_VERSION"),
                result,
            );
            spec.authority = authority;
            spec.source = Some(Source::Artifact {
                path: p.first.rel.clone(),
                sha256: p.first.sha256.clone(),
            });
            spec.coverage = EvidenceCoverage {
                status: if p.chain.as_ref().is_some_and(|c| c.complete) {
                    CoverageStatus::Complete
                } else {
                    CoverageStatus::Partial
                },
                examined: Some(p.walked as u64),
                total: None,
            };
            spec.limitations = vec![
                "only the top loss record is followed, from one of its blocks; other records and blocks are not walked".into(),
                "a walk stops after 12 queries or at the first answer that names no root".into(),
            ];
            if p.chain.is_none() {
                spec.limitations
                    .push("the point has no who_points_at answers, so no holder was found".into());
            }
            let signal = Signal::new("valgrind-vgdb", tool_class::LEAK_CHECKER, &p.sha256);
            recorded.push(ctx.record(ledger, spec, &signal, &raw)?);
            point_json.push(json!({
                "p": clip(&red.name(&p.label), 24),
                "rec": p.record.as_ref().map(|r| format!("{} {}B", short_kind(&r.kind), r.bytes)),
                "root": clip(&red.name(&root_text), 60),
            }));
            last_raw = Some(raw);
        }

        let n = points.len();
        let mut body = Map::new();
        body.insert("points".into(), json!(n));
        let shown: Vec<Value> = point_json.iter().take(SHOWN_POINTS).cloned().collect();
        body.insert("walks".into(), json!(shown));
        body.insert("redaction".into(), json!(red.mode()));
        out.enumerated = points.iter().map(|p| p.walked as u64).sum();
        let keys: Vec<String> = {
            let mut k: Vec<String> = keys_by_point.iter().flatten().cloned().collect();
            k.sort();
            k.dedup();
            k
        };
        if n < rule::MIN_DUMPS || keys.is_empty() {
            body.insert("series".into(), Value::Null);
            body.insert("gate".into(), json!("fail"));
            let why = if n < rule::MIN_DUMPS {
                format!("points < {}", rule::MIN_DUMPS)
            } else {
                "no point's chain ends in a data symbol".to_owned()
            };
            body.insert("reasons".into(), json!([why]));
            body.insert(
                "needed".into(),
                json!(if n < rule::MIN_DUMPS {
                    "3 or more points of the same program run with growing work (dla harness run <H> --tool valgrind); rerun with all vgdb files"
                } else {
                    if points.iter().all(|p| p.walked == 0) {
                        "who_points_at answers for the top record (dla harness run <H> --tool valgrind writes them as out/<label>/vgdb-*.txt); a definitely-lost record has no holder: dla summarize alloc-stack-diff on the XML gives its site"
                    } else {
                        "a chain that ends in a static or global (a data symbol); a thread stack or an unnamed segment has no owner to tie"
                    }
                }),
            );
            out.evidence = recorded;
            out.raw = last_raw;
            out.body = body;
            return Ok(out);
        }

        // ---- series: the record's bytes under the key of each point's root
        let mut tracks: Vec<growth::Track> = Vec::new();
        let mut holder_of: BTreeMap<String, (FindingId, usize)> = BTreeMap::new();
        let mut ambiguous: Vec<(String, Vec<FindingId>)> = Vec::new();
        let symbols: BTreeMap<String, String> = points
            .iter()
            .zip(&keys_by_point)
            .filter_map(|(p, k)| {
                let k = k.clone()?;
                match p.chain.as_ref()?.root.as_ref()? {
                    Holder::DataSymbol { symbol, .. } => Some((k, symbol.clone())),
                    _ => None,
                }
            })
            .collect();
        for key in &keys {
            let values: Vec<Option<u64>> = points
                .iter()
                .zip(&keys_by_point)
                .map(|(p, k)| {
                    (k.as_ref() == Some(key))
                        .then(|| p.record.as_ref().map(|r| r.bytes))
                        .flatten()
                })
                .collect();
            let counts: Vec<Option<u64>> = points
                .iter()
                .zip(&keys_by_point)
                .map(|(p, k)| {
                    (k.as_ref() == Some(key))
                        .then(|| p.record.as_ref().map(|r| r.blocks))
                        .flatten()
                })
                .collect();
            let (found, amb) = symbols
                .get(key)
                .map(|s| finding_for_symbol(ledger, &who, &tc.project, s))
                .unwrap_or_default();
            let matches: Vec<_> = found.into_iter().collect();
            let mut r_tie = tie::resolve(matches, opts.finding.as_ref());
            if !amb.is_empty() {
                r_tie.ambiguous = amb;
            }
            let mut t = growth::Track {
                name: format!("holder:{}", clip(&red.name(key), 110)),
                unit: Unit::Bytes,
                values,
                counts: Some(counts),
                site: None,
                finding: None,
                candidates: Vec::new(),
                region: SeriesRegion::OffHeap,
                tie: None,
                ambiguous: r_tie.ambiguous,
            };
            if let Some((f, why)) = r_tie.tie {
                let site = tie::site_of(ledger, &f);
                holder_of.insert(t.name.clone(), (f.clone(), tracks.len()));
                t.tie_to(f, why, site);
            }
            tracks.push(t);
        }
        let monotone_ambiguous: Vec<(String, Vec<FindingId>)> = tracks
            .iter()
            .filter(|t| {
                !t.ambiguous.is_empty()
                    && rule::gate(&rule::Observed {
                        bytes: t.values.clone(),
                        counts: t.counts.clone().unwrap_or_else(|| t.values.clone()),
                    })
                    .is_ok()
            })
            .map(|t| (t.name.clone(), t.ambiguous.clone()))
            .collect();
        ambiguous.extend(monotone_ambiguous);
        let all_sha: Vec<&str> = points.iter().map(|p| p.sha256.as_str()).collect();
        let signal = Signal::new(
            "valgrind-vgdb",
            tool_class::LEAK_CHECKER,
            &emit::combined_sha(&all_sha),
        );
        let g = growth::write(
            ctx,
            ledger,
            growth::Plan {
                t_ms: vec![0; n],
                tracks,
                signal,
                tool: "valgrind-vgdb",
                authority,
                command: "dla summarize heap-graph".into(),
                source_kind: SeriesSourceKind::Dump,
                runtime: None,
                limitations: vec![
                    "each point is a separate process at exit, run with more work; the figure is the bytes of the top loss record whose chain ends in the key's data symbol".into(),
                    "the loss record is not the symbol's whole retained set: other records may hang from the same symbol".into(),
                ],
                what: "point",
                focus: opts.finding.clone(),
            },
        )?;
        let tied_ids = growth::tied_ids(&g, opts.finding.as_ref());
        // A holder-tied record shows where the holder is written: its first project frame.
        for (name, f) in &g.tied {
            let Some((_, _)) = holder_of.get(name) else {
                continue;
            };
            let key = name.strip_prefix("holder:").unwrap_or(name);
            let site = points
                .iter()
                .zip(&keys_by_point)
                .rev()
                .filter(|(_, k)| k.as_deref().map(|k| red.name(k)).as_deref() == Some(key))
                .find_map(|(p, _)| {
                    p.record.as_ref()?.frames.iter().find_map(|fr: &Frame| {
                        if ctx.artifact_run || tie::is_allocator_frame(&fr.function) {
                            return None;
                        }
                        ctx.resolver()
                            .frame_site(&fr.function, fr.file.as_deref(), fr.line)
                    })
                });
            if let Some(site) = site {
                add_writer(ledger, f, &site)?;
            }
        }
        for f in &tied_ids {
            let fnd = ledger.finding(f);
            out.candidates.push(CandidateRow {
                finding: f.clone(),
                owner: fnd.and_then(|x| x.owner.clone()),
                holder: fnd.and_then(|x| x.holder),
                site: fnd.and_then(|x| x.site.clone()),
                region: fnd.and_then(|x| x.region),
                origin: "growth-series".into(),
                added: false,
                via: None,
            });
        }
        if let Value::Object(m) = g.json.clone() {
            body.extend(m);
        }
        body.insert("tied".into(), json!(tied_ids));
        if let Some(a) = growth::ambiguous_json(&ambiguous) {
            body.insert("ambiguous".into(), a);
        }
        let needed = if !g.pass {
            "the record's bytes under the holder do not grow at every point: run the points with more work and rerun with all vgdb files".to_owned()
        } else if let Some(f) = tied_ids.first() {
            format!(
                "a refuted non-leak sibling and a holder, then dla ledger confirm {f} --evidence {}",
                g.evidence.id
            )
        } else if let Some(f) = &opts.finding {
            format!(
                "no passing column ties to {f} by holder, site or type: the data symbol's module path and name must end the owner of {f} (dla census --lang rust|native)"
            )
        } else {
            "the growing holder ties to no finding: run dla census for the runtime so a holder with that module path and name exists; a bare name ties only when one finding ends with it".to_owned()
        };
        body.insert("needed".into(), json!(needed));
        out.evidence = recorded;
        out.evidence.push(g.evidence);
        out.raw = Some(emit::RawRef {
            rel: format!("series/{}.json", g.series),
            sha256: series_sha(tc, &g.series),
        });
        out.body = body;
        Ok(out)
    }
}

fn series_sha(tc: &dla_core::toolctx::ToolCtx, series: &str) -> String {
    std::fs::read(tc.run_dir.join(format!("series/{series}.json")))
        .map(|b| emit::sha_hex(&b))
        .unwrap_or_default()
}

fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_owned()
    } else {
        let mut t: String = s.chars().take(n).collect();
        t.push_str("...");
        t
    }
}
