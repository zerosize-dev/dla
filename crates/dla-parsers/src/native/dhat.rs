//! DHAT JSON (`dhatFileVersion` 1 and 2), written by `valgrind --tool=dhat
//! --dhat-out-file=<file>` and by the `dhat` crate (`dhat::Profiler`, "dhat-heap.json").
//!
//! `pps` holds one entry per allocation stack: `tb`/`tbk` total bytes and blocks, `mb`/`mbk`
//! the largest size, `gb`/`gbk` bytes and blocks at the global peak, `eb`/`ebk` bytes and
//! blocks still alive when the program ended, and `fs`, indexes into `ftbl` (innermost frame
//! first; entry 0 is the `[root]` marker). A group's live figures are `eb`/`ebk`.
//!
//! Frames come as the tool printed them: `0xADDR: function (file:line:col)` (dhat crate) or
//! `0xADDR: malloc (in /path/vgpreload_dhat-amd64-linux.so)` (Valgrind). Leading allocator
//! frames are trimmed, Rust symbols demangled.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::ParseError;
use super::frames::{make_frame, split_location, trim_allocator_frames};
use crate::model::{AllocGroup, AllocProfile, Frame};

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct DhatReport {
    /// One group per stack, largest live bytes first; `total_bytes` is `tb`.
    pub profile: AllocProfile,
    /// `heap`, `rust-heap`, `copy` or `ad-hoc`.
    pub mode: String,
    pub cmd: Option<String>,
    pub pid: Option<u64>,
    /// Bytes and blocks alive at the end of the run (the sum of `eb`, `ebk`).
    pub exit_bytes: u64,
    pub exit_blocks: u64,
    /// Bytes alive at the global peak (the sum of `gb`).
    pub peak_bytes: u64,
    /// Bytes and blocks allocated over the whole run (the sum of `tb`, `tbk`).
    pub total_bytes: u64,
    pub total_blocks: u64,
}

/// Parses a DHAT file.
pub fn parse(input: &str) -> Result<DhatReport, ParseError> {
    let v: Value = serde_json::from_str(input).map_err(|e| {
        ParseError::new(format!(
            "not DHAT JSON: {e} (expected the file written by --dhat-out-file or dhat::Profiler)"
        ))
    })?;
    let obj = v
        .as_object()
        .ok_or_else(|| ParseError::new("not a DHAT file: the JSON top level is not an object"))?;
    let version = obj
        .get("dhatFileVersion")
        .and_then(Value::as_u64)
        .ok_or_else(|| ParseError::new("not a DHAT file: no dhatFileVersion field"))?;
    if !(1..=2).contains(&version) {
        return Err(ParseError::new(format!(
            "unsupported dhatFileVersion {version} (this reader knows versions 1 and 2)"
        )));
    }
    let ftbl: Vec<&str> = obj
        .get("ftbl")
        .and_then(Value::as_array)
        .ok_or_else(|| ParseError::new("DHAT file has no ftbl array"))?
        .iter()
        .map(|f| f.as_str().unwrap_or("???"))
        .collect();
    let pps = obj
        .get("pps")
        .and_then(Value::as_array)
        .ok_or_else(|| ParseError::new("DHAT file has no pps array"))?;

    let mut report = DhatReport {
        mode: obj
            .get("mode")
            .and_then(Value::as_str)
            .unwrap_or("heap")
            .to_owned(),
        cmd: obj.get("cmd").and_then(Value::as_str).map(str::to_owned),
        pid: obj.get("pid").and_then(Value::as_u64),
        profile: AllocProfile {
            source: "dhat".to_owned(),
            ..AllocProfile::default()
        },
        ..DhatReport::default()
    };
    for (i, pp) in pps.iter().enumerate() {
        let pp = pp
            .as_object()
            .ok_or_else(|| ParseError::new(format!("pps[{i}] is not an object")))?;
        let num = |key: &str| -> Result<u64, ParseError> {
            match pp.get(key) {
                None => Ok(0),
                Some(v) => v.as_u64().ok_or_else(|| {
                    ParseError::new(format!("pps[{i}].{key} is not a non-negative integer: {v}"))
                }),
            }
        };
        let (tb, tbk, eb, ebk, gb) = (num("tb")?, num("tbk")?, num("eb")?, num("ebk")?, num("gb")?);
        let mut frames = Vec::new();
        for idx in pp
            .get("fs")
            .and_then(Value::as_array)
            .ok_or_else(|| ParseError::new(format!("pps[{i}] has no fs array")))?
        {
            let idx = idx
                .as_u64()
                .ok_or_else(|| ParseError::new(format!("pps[{i}].fs holds a non-integer: {idx}")))?
                as usize;
            let text = ftbl.get(idx).ok_or_else(|| {
                ParseError::new(format!(
                    "pps[{i}].fs refers to ftbl[{idx}] but ftbl has {} entries",
                    ftbl.len()
                ))
            })?;
            if *text != "[root]" {
                frames.push(parse_frame(text));
            }
        }
        report.exit_bytes = report.exit_bytes.saturating_add(eb);
        report.exit_blocks = report.exit_blocks.saturating_add(ebk);
        report.peak_bytes = report.peak_bytes.saturating_add(gb);
        report.total_bytes = report.total_bytes.saturating_add(tb);
        report.total_blocks = report.total_blocks.saturating_add(tbk);
        report.profile.groups.push(AllocGroup {
            frames: trim_allocator_frames(frames),
            live_bytes: eb,
            live_blocks: ebk,
            total_bytes: Some(tb),
            root_path: Vec::new(),
            kind: None,
        });
    }
    report.profile.groups.sort_by(|a, b| {
        b.live_bytes
            .cmp(&a.live_bytes)
            .then(b.total_bytes.cmp(&a.total_bytes))
    });
    Ok(report)
}

/// `0x5622ac56974f: rleak::open_session (rleak/src/main.rs:12:40)`,
/// `0x4846828: malloc (in /usr/libexec/valgrind/vgpreload_dhat-amd64-linux.so)`,
/// `0x1092AD: drop_ptr (leak.c:16)`.
fn parse_frame(text: &str) -> Frame {
    let body = match text.split_once(": ") {
        Some((addr, rest)) if addr.starts_with("0x") => rest,
        _ => text,
    };
    if let Some(open) = body.rfind(" (")
        && let Some(inner) = body[open + 2..].strip_suffix(')')
    {
        let function = body[..open].trim();
        if let Some(module) = inner.strip_prefix("in ") {
            return make_frame(function, None, None, Some(module.trim().to_owned()));
        }
        let (file, line) = split_location(inner);
        return make_frame(function, Some(file), line, None);
    }
    make_frame(body.trim(), None, None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_shapes() {
        let f = parse_frame("0x5622ac56974f: rleak::open_session (rleak/src/main.rs:12:40)");
        assert_eq!(
            (f.function.as_str(), f.file.as_deref(), f.line),
            ("rleak::open_session", Some("rleak/src/main.rs"), Some(12))
        );
        let f = parse_frame(
            "0x4846828: malloc (in /usr/libexec/valgrind/vgpreload_dhat-amd64-linux.so)",
        );
        assert_eq!(
            f.module.as_deref(),
            Some("/usr/libexec/valgrind/vgpreload_dhat-amd64-linux.so")
        );
        let f = parse_frame(
            "0x1: <hashbrown::map::HashMap<u64, alloc::vec::Vec<u8>>>::insert (hashbrown-0.17.1/src/map.rs:1812:14)",
        );
        assert_eq!(
            f.function,
            "<hashbrown::map::HashMap<u64, alloc::vec::Vec<u8>>>::insert"
        );
        assert_eq!(f.line, Some(1812));
        let f = parse_frame("0xDEAD: ???");
        assert_eq!(f.function, "???");
    }
}
