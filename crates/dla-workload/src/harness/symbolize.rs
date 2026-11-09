//! Symbolizing allocator-shim return addresses with addr2line.
//!
//! The shim (`harness/native/dla_alloc_shim.c`) cannot pick a project frame itself: it
//! records, per distinct call stack, the first return addresses outside the shim together
//! with the module they fall in and the offset to hand to `addr2line -e <module>`. This
//! module resolves those addresses in one `addr2line` call per module, takes for each
//! stack the first frame whose source file lies inside the scratch project, and sums live
//! bytes and blocks per `file:line`.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use serde_json::Value;

/// Live bytes and blocks of one source site.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SiteTotals {
    /// Live bytes.
    pub bytes: f64,
    /// Live blocks.
    pub blocks: f64,
}

/// The shim's allocation sites of one dump, mapped to project source lines.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Sites {
    /// `file:line` (project-relative, `/` separators) to its totals.
    pub by_site: BTreeMap<String, SiteTotals>,
    /// The function the project frame of each site is in (`Java_shop_codec_Native_open`),
    /// from the largest stack of the site.
    pub functions: BTreeMap<String, String>,
    /// Live bytes in stacks with no project frame (library-internal or unresolved).
    pub unattributed_bytes: f64,
    /// Why some stacks could not be attributed.
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone)]
struct Frame {
    module: usize,
    offset: u64,
}

#[derive(Debug, Clone)]
struct Stack {
    bytes: f64,
    blocks: f64,
    frames: Vec<Frame>,
}

/// Parses `0x1f` or a plain number out of a JSON value.
fn as_u64(v: &Value) -> Option<u64> {
    match v {
        Value::Number(n) => n.as_u64(),
        Value::String(s) => {
            let t = s.trim();
            match t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
                Some(h) => u64::from_str_radix(h, 16).ok(),
                None => t.parse().ok(),
            }
        }
        _ => None,
    }
}

fn as_f64(v: Option<&Value>) -> f64 {
    v.and_then(Value::as_f64).unwrap_or(0.0)
}

fn parse_stacks(raw_sites: &Value) -> Vec<Stack> {
    let Some(map) = raw_sites.as_object() else {
        return Vec::new();
    };
    map.values()
        .filter_map(|s| {
            let o = s.as_object()?;
            let frames = o
                .get("frames")?
                .as_array()?
                .iter()
                .filter_map(|f| {
                    Some(Frame {
                        module: usize::try_from(as_u64(f.get("m")?)?).ok()?,
                        offset: as_u64(f.get("o")?)?,
                    })
                })
                .collect();
            Some(Stack {
                bytes: as_f64(o.get("live_bytes")),
                blocks: as_f64(o.get("live_blocks")),
                frames,
            })
        })
        .collect()
}

/// One resolved frame: the innermost-first inline frames of an address, as
/// `(file, line, function)`.
type Resolved = Vec<(String, u32, String)>;

/// Parses `addr2line -a -f -C -i` output into one list of `(file, line)` per requested
/// address, in request order.
fn parse_addr2line(out: &str, count: usize) -> Vec<Resolved> {
    let mut groups: Vec<Resolved> = Vec::new();
    let mut pending_fn = false;
    let mut func = String::new();
    for line in out.lines() {
        let t = line.trim_end();
        if t.starts_with("0x") && t[2..].bytes().all(|b| b.is_ascii_hexdigit()) && !pending_fn {
            groups.push(Vec::new());
            continue;
        }
        let Some(group) = groups.last_mut() else {
            continue;
        };
        if !pending_fn {
            pending_fn = true; // function name line
            func = t.to_owned();
            continue;
        }
        pending_fn = false;
        // `file:line`, optionally followed by ` (discriminator N)`.
        let loc = t.split(" (").next().unwrap_or(t);
        if let Some((file, ln)) = loc.rsplit_once(':')
            && let Ok(ln) = ln.parse::<u32>()
            && file != "??"
            && ln > 0
        {
            group.push((file.to_owned(), ln, func.clone()));
        }
    }
    groups.resize(count, Vec::new());
    groups
}

/// Runs addr2line for `offsets` in `module`. `None` when the tool is missing or fails.
fn run_addr2line(module: &Path, offsets: &[u64]) -> Option<Vec<Resolved>> {
    let mut cmd = Command::new("addr2line");
    cmd.args(["-a", "-f", "-C", "-i", "-e"]).arg(module);
    for o in offsets {
        cmd.arg(format!("{o:#x}"));
    }
    let out = cmd
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(parse_addr2line(
        &String::from_utf8_lossy(&out.stdout),
        offsets.len(),
    ))
}

/// A file inside the project, as a project-relative `/` path. Build output (`target/`,
/// `node_modules/`) and anything outside `work` is not a project frame.
fn project_relative(work: &Path, file: &str) -> Option<String> {
    let p = Path::new(file);
    let rel: PathBuf = if p.is_absolute() {
        let canon = p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        canon
            .strip_prefix(work)
            .or_else(|_| p.strip_prefix(work))
            .ok()?
            .to_path_buf()
    } else {
        let trimmed = p.strip_prefix(".").unwrap_or(p);
        if !work.join(trimmed).exists() {
            return None;
        }
        trimmed.to_path_buf()
    };
    let s = rel.to_string_lossy().replace('\\', "/");
    let outside = s.starts_with("../")
        || s.starts_with("target/")
        || s.starts_with("node_modules/")
        || s.starts_with(".cargo/");
    (!outside && !s.is_empty()).then_some(s)
}

/// Resolves the shim records of one dump (`raw_sites`, `modules`) to project sites.
///
/// `exe` is the executable path from the dump, used when the main module has no path.
/// `work` is the scratch project root. Both the shim's stack-object form and nothing
/// else are understood; any other shape yields an empty result with a limitation.
pub fn symbolize(
    raw_sites: &Value,
    modules: Option<&Value>,
    exe: Option<&str>,
    work: &Path,
) -> Sites {
    let mut sites = Sites::default();
    let stacks = parse_stacks(raw_sites);
    if stacks.is_empty() {
        if raw_sites.as_object().is_some_and(|m| !m.is_empty()) {
            sites
                .limitations
                .push("raw_sites has no stack frames to symbolize (unrecognised shape)".to_owned());
        }
        return sites;
    }
    let work = work.canonicalize().unwrap_or_else(|_| work.to_path_buf());
    let paths: Vec<Option<PathBuf>> = modules
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .enumerate()
                .map(|(i, m)| {
                    let p = m.get("path").and_then(Value::as_str).unwrap_or("");
                    let p = if p.is_empty() && i == 0 {
                        exe.unwrap_or("")
                    } else {
                        p
                    };
                    let path = PathBuf::from(p);
                    (!p.is_empty() && path.is_file()).then_some(path)
                })
                .collect()
        })
        .unwrap_or_default();
    // Batch the offsets per module.
    let mut wanted: BTreeMap<usize, BTreeSet<u64>> = BTreeMap::new();
    for s in &stacks {
        for f in &s.frames {
            if paths.get(f.module).is_some_and(Option::is_some) {
                wanted.entry(f.module).or_default().insert(f.offset);
            }
        }
    }
    let mut resolved: BTreeMap<(usize, u64), Resolved> = BTreeMap::new();
    let mut missing_tool = false;
    for (module, offsets) in &wanted {
        let Some(path) = paths[*module].as_deref() else {
            continue;
        };
        let offsets: Vec<u64> = offsets.iter().copied().collect();
        // Keep the command line bounded.
        for chunk in offsets.chunks(512) {
            match run_addr2line(path, chunk) {
                Some(groups) => {
                    for (o, g) in chunk.iter().zip(groups) {
                        resolved.insert((*module, *o), g);
                    }
                }
                None => missing_tool = true,
            }
        }
    }
    if missing_tool {
        sites.limitations.push(
            "addr2line is missing or failed: allocator sites could not be mapped to source lines"
                .to_owned(),
        );
    }
    let mut cache: BTreeMap<String, Option<String>> = BTreeMap::new();
    let mut biggest: BTreeMap<String, f64> = BTreeMap::new();
    for s in &stacks {
        let site = s.frames.iter().find_map(|f| {
            resolved
                .get(&(f.module, f.offset))?
                .iter()
                .find_map(|(file, line, func)| {
                    let rel = cache
                        .entry(file.clone())
                        .or_insert_with(|| project_relative(&work, file))
                        .clone()?;
                    Some((format!("{rel}:{line}"), func.clone()))
                })
        });
        match site {
            Some((site, func)) => {
                if !func.is_empty()
                    && func != "??"
                    && biggest.get(&site).is_none_or(|b| s.bytes > *b)
                {
                    biggest.insert(site.clone(), s.bytes);
                    sites.functions.insert(site.clone(), func);
                }
                let t = sites.by_site.entry(site).or_default();
                t.bytes += s.bytes;
                t.blocks += s.blocks;
            }
            None => sites.unattributed_bytes += s.bytes,
        }
    }
    if sites.unattributed_bytes > 0.0 {
        sites.limitations.push(format!(
            "{} live bytes sit in allocations with no frame in the project (library-internal or without debug info)",
            sites.unattributed_bytes
        ));
    }
    sites
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn addr2line_output_groups_inline_frames_per_address() {
        let out = "0x0000000000001000\ninner\n/w/src/a.c:5\nouter\n/w/src/a.c:9 (discriminator 3)\n0x0000000000001010\n_start\n??:?\n0x0000000000001020\nf\n??:0\n";
        let g = parse_addr2line(out, 3);
        assert_eq!(
            g[0],
            vec![
                ("/w/src/a.c".to_owned(), 5, "inner".to_owned()),
                ("/w/src/a.c".to_owned(), 9, "outer".to_owned())
            ]
        );
        assert!(g[1].is_empty() && g[2].is_empty());
    }

    #[test]
    fn hex_strings_and_numbers_both_parse() {
        assert_eq!(as_u64(&json!("0x17b6")), Some(0x17b6));
        assert_eq!(as_u64(&json!(42)), Some(42));
        assert_eq!(as_u64(&json!("12")), Some(12));
        assert_eq!(as_u64(&json!(null)), None);
    }

    #[test]
    fn project_files_are_relative_and_build_output_is_not() {
        let dir = tempfile::tempdir().unwrap();
        let work = dir.path().canonicalize().unwrap();
        std::fs::create_dir_all(work.join("src")).unwrap();
        std::fs::write(work.join("src/a.c"), "x").unwrap();
        let abs = work.join("src/a.c");
        assert_eq!(
            project_relative(&work, abs.to_str().unwrap()).as_deref(),
            Some("src/a.c")
        );
        assert_eq!(
            project_relative(&work, "src/a.c").as_deref(),
            Some("src/a.c")
        );
        assert_eq!(project_relative(&work, "/usr/include/x.h"), None);
        assert_eq!(
            project_relative(&work, work.join("target/debug/b.rs").to_str().unwrap()),
            None
        );
    }

    #[test]
    fn an_unknown_shape_is_a_limitation_not_a_panic() {
        let s = symbolize(
            &json!({"native.live_bytes": {"0x55": 4096}}),
            None,
            None,
            Path::new("/nonexistent"),
        );
        assert!(s.by_site.is_empty());
        assert_eq!(s.limitations.len(), 1);
    }
}
