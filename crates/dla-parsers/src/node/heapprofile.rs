//! V8 sampling heap profiles (`.heapprofile` from `--heap-prof`, or the profile the
//! inspector's `HeapProfiler.stopSampling` returns).
//!
//! The file is a tree of call frames with `selfSize` bytes of live sampled allocations
//! on each node. A sample attributed to a native builtin (`url` empty, `lineNumber` -1:
//! `Array.push`, `repeat`, a typed array constructor) is charged to the nearest JavaScript
//! ancestor, so the stack names the code that allocated.

use std::{collections::BTreeMap, io::Read};

use super::{NodeError, json::Reader, url_to_path};
use crate::model::{AllocGroup, AllocProfile, Frame};

/// Deepest call tree read.
const MAX_DEPTH: usize = 2000;
/// Most tree nodes read.
const MAX_NODES: usize = 2_000_000;
/// Largest sample list read.
const MAX_SAMPLES: usize = 8_000_000;

#[derive(Debug, Default, Clone)]
struct RawNode {
    parent: Option<u32>,
    function: String,
    url: String,
    line: i64,
    column: i64,
    self_size: u64,
    id: u64,
}

/// True when the first bytes look like a heap profile.
pub fn sniff(head: &[u8]) -> bool {
    let text = String::from_utf8_lossy(head);
    let t = text.trim_start();
    t.starts_with('{') && t.contains("\"head\"") && t.contains("\"callFrame\"")
        || t.starts_with('{') && t.contains("\"profile\"") && t.contains("\"callFrame\"")
}

struct Parse<R: Read> {
    r: Reader<R>,
    nodes: Vec<RawNode>,
    samples: BTreeMap<u64, u64>,
    sample_count: usize,
}

impl<R: Read> Parse<R> {
    fn call_frame(&mut self, at: usize) -> Result<(), NodeError> {
        self.r.expect(b'{')?;
        if self.r.eat(b'}')? {
            return Ok(());
        }
        loop {
            let key = self.r.string(64)?;
            self.r.expect(b':')?;
            match key.as_str() {
                "functionName" => self.nodes[at].function = self.r.string(4096)?,
                "url" => self.nodes[at].url = self.r.string(8192)?,
                "lineNumber" => self.nodes[at].line = self.r.int()?,
                "columnNumber" => self.nodes[at].column = self.r.int()?,
                _ => self.r.skip_value()?,
            }
            if self.r.eat(b',')? {
                continue;
            }
            return self.r.expect(b'}');
        }
    }

    /// Reads the node object at the reader and everything below it. Iterative, so the
    /// depth of a call stack costs memory in a `Vec`, not stack.
    fn node(&mut self) -> Result<(), NodeError> {
        enum St {
            Open,
            Key,
            Next,
            Child,
        }
        let mut stack: Vec<(usize, St)> = Vec::new();
        let mut parent: Option<u32> = None;
        let mut opening = true;
        loop {
            if opening {
                opening = false;
                if stack.len() >= MAX_DEPTH {
                    return Err(NodeError::TooLarge(format!(
                        "the call tree is deeper than {MAX_DEPTH} frames"
                    )));
                }
                if self.nodes.len() >= MAX_NODES {
                    return Err(NodeError::TooLarge(format!(
                        "the call tree has more than {MAX_NODES} nodes"
                    )));
                }
                self.nodes.push(RawNode {
                    parent,
                    line: -1,
                    column: -1,
                    ..RawNode::default()
                });
                self.r.expect(b'{')?;
                stack.push((self.nodes.len() - 1, St::Open));
            }
            let Some((at, st)) = stack.last_mut() else {
                return Ok(());
            };
            let at = *at;
            match st {
                St::Open => {
                    if self.r.eat(b'}')? {
                        stack.pop();
                    } else {
                        *st = St::Key;
                    }
                }
                St::Key => {
                    let key = self.r.string(64)?;
                    self.r.expect(b':')?;
                    *st = St::Next;
                    match key.as_str() {
                        "callFrame" => self.call_frame(at)?,
                        "selfSize" => self.nodes[at].self_size = self.r.uint()?,
                        "id" => self.nodes[at].id = self.r.uint()?,
                        "children" => {
                            self.r.expect(b'[')?;
                            if !self.r.eat(b']')? {
                                if let Some((_, s)) = stack.last_mut() {
                                    *s = St::Child;
                                }
                                parent = Some(at as u32);
                                opening = true;
                            }
                        }
                        _ => self.r.skip_value()?,
                    }
                }
                St::Next => {
                    if self.r.eat(b',')? {
                        *st = St::Key;
                    } else {
                        self.r.expect(b'}')?;
                        stack.pop();
                    }
                }
                St::Child => {
                    if self.r.eat(b',')? {
                        parent = Some(at as u32);
                        opening = true;
                    } else {
                        self.r.expect(b']')?;
                        *st = St::Next;
                    }
                }
            }
        }
    }

    fn sample_list(&mut self) -> Result<(), NodeError> {
        self.r.expect(b'[')?;
        if self.r.eat(b']')? {
            return Ok(());
        }
        loop {
            self.r.expect(b'{')?;
            let mut id = None;
            if !self.r.eat(b'}')? {
                loop {
                    let key = self.r.string(64)?;
                    self.r.expect(b':')?;
                    if key == "nodeId" {
                        id = Some(self.r.uint()?);
                    } else {
                        self.r.skip_value()?;
                    }
                    if self.r.eat(b',')? {
                        continue;
                    }
                    self.r.expect(b'}')?;
                    break;
                }
            }
            self.sample_count += 1;
            if self.sample_count > MAX_SAMPLES {
                return Err(NodeError::TooLarge(format!(
                    "the profile lists more than {MAX_SAMPLES} samples"
                )));
            }
            if let Some(id) = id {
                *self.samples.entry(id).or_default() += 1;
            }
            if self.r.eat(b',')? {
                continue;
            }
            return self.r.expect(b']');
        }
    }

    /// The profile object: `head` and `samples`, possibly under `profile`.
    fn profile(&mut self, top: bool) -> Result<bool, NodeError> {
        self.r.expect(b'{')?;
        let mut found = false;
        if self.r.eat(b'}')? {
            return Ok(false);
        }
        loop {
            let key = self.r.string(64)?;
            self.r.expect(b':')?;
            match key.as_str() {
                "head" => {
                    self.node()?;
                    found = true;
                }
                "samples" => self.sample_list()?,
                "profile" if top => found |= self.profile(false)?,
                _ => self.r.skip_value()?,
            }
            if self.r.eat(b',')? {
                continue;
            }
            self.r.expect(b'}')?;
            return Ok(found);
        }
    }
}

/// Function, file and line of each JavaScript frame: what makes two stacks the same.
type StackKey = Vec<(String, String, i64)>;

/// Innermost JavaScript frames kept per stack; outer frames are dropped before grouping.
const MAX_STACK_FRAMES: usize = 128;
/// Most frames stored across all distinct stacks.
const MAX_STORED_FRAMES: usize = 3_000_000;

fn is_js(n: &RawNode) -> bool {
    !(n.url.is_empty() && n.line < 0) && !n.function.starts_with('(')
}

/// Maps a generated position (script path, zero-based line and column) to an original
/// file and one-based line, when the script has a source map.
pub type PositionMap<'a> = dyn FnMut(&str, u32, u32) -> Option<(String, u32)> + 'a;

/// Parses a heap profile into allocation groups.
pub fn parse(src: impl Read) -> Result<AllocProfile, NodeError> {
    parse_with(src, &mut |_, _, _| None)
}

/// [`parse`], with a source map lookup applied to every JavaScript frame.
pub fn parse_with(src: impl Read, map: &mut PositionMap<'_>) -> Result<AllocProfile, NodeError> {
    let mut p = Parse {
        r: Reader::new(src),
        nodes: Vec::new(),
        samples: BTreeMap::new(),
        sample_count: 0,
    };
    if !p.profile(true)? || p.nodes.is_empty() {
        return Err(NodeError::NotThisFormat(
            "no `head` call tree: not a V8 heap profile".to_owned(),
        ));
    }
    let nodes = p.nodes;
    // Groups by the JavaScript frames of the stack, innermost first.
    let mut groups: BTreeMap<StackKey, (Vec<Frame>, u64, u64)> = BTreeMap::new();
    let mut stored = 0usize;
    for (i, n) in nodes.iter().enumerate() {
        if n.self_size == 0 {
            continue;
        }
        let blocks = p.samples.get(&n.id).copied().unwrap_or(1).max(1);
        let mut frames: Vec<Frame> = Vec::new();
        let mut key: StackKey = Vec::new();
        let mut at = Some(i as u32);
        let mut hops = 0;
        while let Some(a) = at {
            let x = &nodes[a as usize];
            if is_js(x) && frames.len() < MAX_STACK_FRAMES {
                let function = if x.function.is_empty() {
                    "(anonymous)".to_owned()
                } else {
                    x.function.clone()
                };
                let mut file = (!x.url.is_empty()).then(|| url_to_path(&x.url));
                let mut line = (x.line >= 0).then(|| u32::try_from(x.line + 1).unwrap_or(u32::MAX));
                if let (Some(f), true, true) = (&file, x.line >= 0, x.column >= 0)
                    && let Some((orig, l)) = map(
                        f,
                        u32::try_from(x.line).unwrap_or(u32::MAX),
                        u32::try_from(x.column).unwrap_or(u32::MAX),
                    )
                {
                    file = Some(orig);
                    line = Some(l);
                }
                key.push((
                    function.clone(),
                    file.clone().unwrap_or_default(),
                    line.map_or(-1, i64::from),
                ));
                frames.push(Frame {
                    function,
                    file,
                    line,
                    module: None,
                });
            }
            at = x.parent;
            hops += 1;
            if hops > MAX_DEPTH + 2 {
                break;
            }
        }
        if frames.is_empty() {
            frames.push(Frame {
                function: "(native)".to_owned(),
                ..Frame::default()
            });
            key.push(("(native)".to_owned(), String::new(), -1));
        }
        if !groups.contains_key(&key) {
            stored = stored.saturating_add(frames.len());
            if stored > MAX_STORED_FRAMES {
                return Err(NodeError::TooLarge(format!(
                    "the heap profile's distinct stacks hold more than {MAX_STORED_FRAMES} frames"
                )));
            }
        }
        let e = groups.entry(key).or_insert((frames, 0, 0));
        e.1 = e.1.saturating_add(n.self_size);
        e.2 = e.2.saturating_add(blocks);
    }
    let mut out: Vec<AllocGroup> = groups
        .into_values()
        .map(|(frames, bytes, blocks)| AllocGroup {
            frames,
            live_bytes: bytes,
            live_blocks: blocks,
            total_bytes: None,
            root_path: Vec::new(),
            kind: None,
        })
        .collect();
    out.sort_by_key(|g| std::cmp::Reverse(g.live_bytes));
    Ok(AllocProfile {
        source: "node-heapprofile".to_owned(),
        timestamp_ms: None,
        sampled: true,
        groups: out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE: &str = r#"{"head":{"callFrame":{"functionName":"(root)","scriptId":"0","url":"","lineNumber":-1,"columnNumber":-1},"selfSize":0,"id":1,"children":[
      {"callFrame":{"functionName":"leak","scriptId":"7","url":"file:///work/src/a.js","lineNumber":1,"columnNumber":4},"selfSize":0,"id":2,"children":[
        {"callFrame":{"functionName":"push","scriptId":"0","url":"","lineNumber":-1,"columnNumber":-1},"selfSize":600,"id":3,"children":[]},
        {"callFrame":{"functionName":"helper","scriptId":"7","url":"file:///work/src/a.js","lineNumber":9,"columnNumber":1},"selfSize":100,"id":4,"children":[]}]},
      {"callFrame":{"functionName":"(program)","scriptId":"0","url":"","lineNumber":-1,"columnNumber":-1},"selfSize":50,"id":5,"children":[]}]},
      "samples":[{"size":300,"nodeId":3,"ordinal":1},{"size":300,"nodeId":3,"ordinal":2},{"size":100,"nodeId":4,"ordinal":3}]}"#;

    #[test]
    fn builtins_are_charged_to_the_nearest_javascript_frame() {
        let p = parse(PROFILE.as_bytes()).unwrap();
        assert!(p.sampled);
        let g = &p.groups[0];
        assert_eq!(g.live_bytes, 600);
        assert_eq!(g.live_blocks, 2);
        assert_eq!(g.frames.len(), 1);
        assert_eq!(g.frames[0].function, "leak");
        assert_eq!(g.frames[0].file.as_deref(), Some("/work/src/a.js"));
        assert_eq!(g.frames[0].line, Some(2));
        // helper is called from leak: two JS frames, innermost first.
        let h = p
            .groups
            .iter()
            .find(|g| g.frames[0].function == "helper")
            .unwrap();
        assert_eq!(h.frames[1].function, "leak");
        // A sample in `(program)` has no JavaScript frame at all.
        assert!(
            p.groups
                .iter()
                .any(|g| g.frames[0].function == "(native)" && g.live_bytes == 50)
        );
    }

    #[test]
    fn the_inspector_wrapper_and_bad_input_are_handled() {
        let wrapped = format!("{{\"profile\":{PROFILE}}}");
        assert_eq!(parse(wrapped.as_bytes()).unwrap().groups.len(), 3);
        assert!(parse(&b"{\"a\":1}"[..]).is_err());
        assert!(parse(&b"{\"head\":{\"callFrame\":"[..]).is_err());
        let deep = format!(
            "{{\"head\":{}{}}}",
            "{\"children\":[".repeat(3000),
            "]}".repeat(3000)
        );
        assert!(parse(deep.as_bytes()).is_err());
        assert!(sniff(PROFILE.as_bytes()));
        assert!(!sniff(b"{\"snapshot\":{}}"));
    }

    #[test]
    fn deep_chains_keep_only_the_innermost_frames() {
        let n = 1500;
        let mut json = String::from(r#"{"head":"#);
        for i in 0..n {
            json.push_str(&format!(
                r#"{{"callFrame":{{"functionName":"f{i}","scriptId":"1","url":"file:///a.js","lineNumber":{i},"columnNumber":0}},"selfSize":10,"id":{},"children":["#,
                i + 1
            ));
        }
        json.push_str(&"]}".repeat(n));
        json.push('}');
        let p = parse(json.as_bytes()).unwrap();
        assert_eq!(p.groups.len(), n);
        assert!(p.groups.iter().all(|g| g.frames.len() <= MAX_STACK_FRAMES));
        assert_eq!(
            p.groups.iter().map(|g| g.live_bytes).sum::<u64>(),
            10 * n as u64
        );
    }
}
