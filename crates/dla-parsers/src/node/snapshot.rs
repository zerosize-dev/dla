//! The V8 `.heapsnapshot` reader.
//!
//! The file is one JSON object: `snapshot` (with `meta`, the field names and enums of
//! nodes and edges), then the integer arrays `nodes` and `edges`, optional trace arrays,
//! `locations`, and `strings` last. Field counts and enums come from `meta`, never from
//! constants, because V8 has added fields over time. Names are `u32` indexes into
//! `strings`, which arrive last, so they are resolved on demand.

use std::{fs::File, io::Read, path::Path};

use serde_json::Value;

use super::{NodeError, json::Reader};
use crate::model::{Histogram, HistogramRow};

/// Longest string kept from the string table, bytes.
const STRING_LIMIT: usize = 1 << 20;
/// Longest `snapshot` header read, bytes.
const HEADER_LIMIT: usize = 4 << 20;
/// Memory a load needs, as a multiple of the file size.
const MEMORY_FACTOR: u64 = 4;

/// Limits of a load.
#[derive(Debug, Clone)]
pub struct Options {
    /// Largest file read, bytes.
    pub max_bytes: u64,
    /// Available memory in bytes, when the caller knows it; else it is read from the
    /// operating system where that is possible.
    pub available_memory: Option<u64>,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            max_bytes: 3 << 30,
            available_memory: None,
        }
    }
}

/// True when the first bytes of a file look like a V8 heap snapshot.
pub fn sniff(head: &[u8]) -> bool {
    let text = String::from_utf8_lossy(head);
    let t = text.trim_start();
    t.starts_with('{') && t.contains("\"snapshot\"") && t.contains("\"node_fields\"")
}

/// `MemAvailable` of the host in bytes, where the operating system says.
fn host_available_memory() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let text = std::fs::read_to_string("/proc/meminfo").ok()?;
        let line = text.lines().find(|l| l.starts_with("MemAvailable:"))?;
        let kb: u64 = line.split_whitespace().nth(1)?.parse().ok()?;
        kb.checked_mul(1024)
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Refuses a file that would not fit in memory: a load holds about 1.5 times the file,
/// and the analysis about as much again, so the file may be at most a quarter of the
/// available memory.
pub fn preflight(size: u64, opts: &Options) -> Result<(), NodeError> {
    if size > opts.max_bytes {
        return Err(NodeError::TooLarge(format!(
            "the snapshot is {} MB, over the {} MB limit",
            size >> 20,
            opts.max_bytes >> 20
        )));
    }
    if let Some(avail) = opts.available_memory.or_else(host_available_memory)
        && size.saturating_mul(MEMORY_FACTOR) > avail
    {
        return Err(NodeError::TooLarge(format!(
            "the snapshot is {} MB and reading it needs about {} times that in memory, but {} MB is available",
            size >> 20,
            MEMORY_FACTOR,
            avail >> 20
        )));
    }
    Ok(())
}

/// The ids of node and edge types the analysis needs, found by name in the enums.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Kinds {
    pub(crate) n_object: Option<u32>,
    pub(crate) n_closure: Option<u32>,
    pub(crate) n_code: Option<u32>,
    pub(crate) n_array: Option<u32>,
    pub(crate) n_string: Option<u32>,
    pub(crate) n_cons_string: Option<u32>,
    pub(crate) n_sliced_string: Option<u32>,
    pub(crate) n_native: Option<u32>,
    pub(crate) n_synthetic: Option<u32>,
    pub(crate) n_hidden: Option<u32>,
    pub(crate) e_context: Option<u32>,
    pub(crate) e_element: Option<u32>,
    pub(crate) e_property: Option<u32>,
    pub(crate) e_internal: Option<u32>,
    pub(crate) e_hidden: Option<u32>,
    pub(crate) e_shortcut: Option<u32>,
    pub(crate) e_weak: Option<u32>,
}

fn index_of(list: &[String], name: &str) -> Option<u32> {
    list.iter().position(|s| s == name).map(|i| i as u32)
}

/// A loaded snapshot.
#[derive(Debug)]
pub struct Snapshot {
    nf: usize,
    ef: usize,
    f_type: usize,
    f_name: usize,
    f_id: usize,
    f_self: usize,
    e_type: usize,
    e_name: usize,
    e_to: usize,
    node_types: Vec<String>,
    edge_types: Vec<String>,
    nodes: Vec<u32>,
    edges: Vec<u32>,
    strings: Vec<String>,
    /// First edge of each node, and the end of the last: `node_count + 1` entries.
    first_edge: Vec<u32>,
    pub(crate) kinds: Kinds,
    /// `object_index, script_id, line, column` rows as the file has them.
    locations: Vec<u32>,
    /// `location_fields.len()`, 0 when the file has no locations.
    lf: usize,
}

fn string_list(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|x| x.as_str().unwrap_or_default().to_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn field(fields: &[String], name: &str) -> Option<usize> {
    fields.iter().position(|f| f == name)
}

impl Snapshot {
    /// Reads a snapshot file, after the memory preflight.
    pub fn read_file(path: &Path, opts: &Options) -> Result<Self, NodeError> {
        let f = File::open(path)?;
        let size = f.metadata()?.len();
        preflight(size, opts)?;
        Self::read(f, size)
    }

    /// Reads a snapshot from a stream; `size_hint` bounds array lengths.
    pub fn read(src: impl Read, size_hint: u64) -> Result<Self, NodeError> {
        let mut r = Reader::new(src);
        // A number takes at least two bytes (digit and comma).
        let max_elems = usize::try_from(size_hint / 2 + 8).unwrap_or(usize::MAX);
        r.skip_ws()?;
        r.expect(b'{')?;
        let mut header: Option<Value> = None;
        let mut nodes: Vec<u32> = Vec::new();
        let mut edges: Vec<u32> = Vec::new();
        let mut locations: Vec<u32> = Vec::new();
        let mut strings: Vec<String> = Vec::new();
        let mut seen_strings = false;
        if !r.eat(b'}')? {
            loop {
                let key = r.string(256)?;
                r.expect(b':')?;
                match key.as_str() {
                    "snapshot" => {
                        let raw = r.raw_value(Some(HEADER_LIMIT))?;
                        header = Some(serde_json::from_slice(&raw).map_err(|e| {
                            NodeError::Malformed(format!("the snapshot header is not JSON: {e}"))
                        })?);
                    }
                    "nodes" => r.uint_array(&mut nodes, max_elems)?,
                    "edges" => r.uint_array(&mut edges, max_elems)?,
                    "locations" => r.uint_array(&mut locations, max_elems)?,
                    "strings" => {
                        r.string_array(&mut strings, max_elems, STRING_LIMIT)?;
                        seen_strings = true;
                    }
                    _ => r.skip_value()?,
                }
                if r.eat(b',')? {
                    continue;
                }
                r.expect(b'}')?;
                break;
            }
        }
        let Some(header) = header else {
            return Err(NodeError::NotThisFormat(
                "no `snapshot` header: not a V8 heap snapshot".to_owned(),
            ));
        };
        if !seen_strings || nodes.is_empty() {
            return Err(NodeError::Truncated(
                "the snapshot has no nodes or no string table (an incomplete write?)".to_owned(),
            ));
        }
        Self::assemble(&header, nodes, edges, strings, locations)
    }

    fn assemble(
        header: &Value,
        nodes: Vec<u32>,
        edges: Vec<u32>,
        strings: Vec<String>,
        locations: Vec<u32>,
    ) -> Result<Self, NodeError> {
        let meta = header
            .get("meta")
            .ok_or_else(|| NodeError::NotThisFormat("the header has no `meta`".to_owned()))?;
        let nfields = string_list(meta.get("node_fields"));
        let efields = string_list(meta.get("edge_fields"));
        let lfields = string_list(meta.get("location_fields"));
        let enum_of = |key: &str| -> Vec<String> {
            meta.get(key)
                .and_then(Value::as_array)
                .and_then(|a| a.first())
                .map(|e| string_list(Some(e)))
                .unwrap_or_default()
        };
        let node_types = enum_of("node_types");
        let edge_types = enum_of("edge_types");
        let need = |fields: &[String], name: &str, what: &str| {
            field(fields, name)
                .ok_or_else(|| NodeError::Malformed(format!("meta.{what} has no `{name}`")))
        };
        let (nf, ef) = (nfields.len(), efields.len());
        let f_type = need(&nfields, "type", "node_fields")?;
        let f_name = need(&nfields, "name", "node_fields")?;
        let f_id = need(&nfields, "id", "node_fields")?;
        let f_self = need(&nfields, "self_size", "node_fields")?;
        let f_edges = need(&nfields, "edge_count", "node_fields")?;
        let e_type = need(&efields, "type", "edge_fields")?;
        let e_name = need(&efields, "name_or_index", "edge_fields")?;
        let e_to = need(&efields, "to_node", "edge_fields")?;
        if !nodes.len().is_multiple_of(nf) {
            return Err(NodeError::Malformed(format!(
                "the node array holds {} numbers, not a multiple of {nf}",
                nodes.len()
            )));
        }
        if !edges.len().is_multiple_of(ef) {
            return Err(NodeError::Malformed(format!(
                "the edge array holds {} numbers, not a multiple of {ef}",
                edges.len()
            )));
        }
        let node_count = nodes.len() / nf;
        let edge_count = edges.len() / ef;
        if u32::try_from(edge_count).is_err() || u32::try_from(nodes.len()).is_err() {
            return Err(NodeError::TooLarge(
                "the snapshot has more than 2^32 nodes or edges".to_owned(),
            ));
        }
        // Edges of node i follow those of node i - 1; the counts must add up.
        let mut first_edge: Vec<u32> = Vec::with_capacity(node_count + 1);
        let mut at: u64 = 0;
        for i in 0..node_count {
            first_edge.push(at as u32);
            at += u64::from(nodes[i * nf + f_edges]);
            if at > edge_count as u64 {
                return Err(NodeError::Malformed(format!(
                    "node {i} claims edges past the end of the edge array"
                )));
            }
        }
        if at != edge_count as u64 {
            return Err(NodeError::Malformed(format!(
                "the nodes claim {at} edges and the file holds {edge_count}"
            )));
        }
        first_edge.push(edge_count as u32);
        for e in 0..edge_count {
            let to = edges[e * ef + e_to] as usize;
            if !to.is_multiple_of(nf) || to / nf >= node_count {
                return Err(NodeError::Malformed(format!(
                    "edge {e} points at offset {to}, which is not a node"
                )));
            }
        }
        let lf = lfields.len();
        let locations = if lf > 0 && locations.len().is_multiple_of(lf) {
            locations
        } else {
            Vec::new()
        };
        let kinds = Kinds {
            n_object: index_of(&node_types, "object"),
            n_closure: index_of(&node_types, "closure"),
            n_code: index_of(&node_types, "code"),
            n_array: index_of(&node_types, "array"),
            n_string: index_of(&node_types, "string"),
            n_cons_string: index_of(&node_types, "concatenated string"),
            n_sliced_string: index_of(&node_types, "sliced string"),
            n_native: index_of(&node_types, "native"),
            n_synthetic: index_of(&node_types, "synthetic"),
            n_hidden: index_of(&node_types, "hidden"),
            e_context: index_of(&edge_types, "context"),
            e_element: index_of(&edge_types, "element"),
            e_property: index_of(&edge_types, "property"),
            e_internal: index_of(&edge_types, "internal"),
            e_hidden: index_of(&edge_types, "hidden"),
            e_shortcut: index_of(&edge_types, "shortcut"),
            e_weak: index_of(&edge_types, "weak"),
        };
        Ok(Self {
            nf,
            ef,
            f_type,
            f_name,
            f_id,
            f_self,
            e_type,
            e_name,
            e_to,
            node_types,
            edge_types,
            nodes,
            edges,
            strings,
            first_edge,
            kinds,
            locations,
            lf,
        })
    }

    /// Number of nodes.
    pub fn node_count(&self) -> usize {
        self.first_edge.len() - 1
    }

    /// Number of edges.
    pub fn edge_count(&self) -> usize {
        self.edges.len() / self.ef
    }

    /// Number of strings in the table.
    pub fn string_count(&self) -> usize {
        self.strings.len()
    }

    pub(crate) fn node_type(&self, n: usize) -> u32 {
        self.nodes[n * self.nf + self.f_type]
    }

    /// The type name of a node (`object`, `closure`, ...).
    pub fn node_type_name(&self, n: usize) -> &str {
        self.node_types
            .get(self.node_type(n) as usize)
            .map_or("", String::as_str)
    }

    fn string(&self, idx: u32) -> &str {
        self.strings.get(idx as usize).map_or("", String::as_str)
    }

    /// The name of a node: a constructor, a function, a string's text, ...
    pub fn node_name(&self, n: usize) -> &str {
        self.string(self.nodes[n * self.nf + self.f_name])
    }

    /// The object id V8 gave the node; stable within one process.
    pub fn node_id(&self, n: usize) -> u32 {
        self.nodes[n * self.nf + self.f_id]
    }

    /// The node's own size in bytes.
    pub fn self_size(&self, n: usize) -> u64 {
        u64::from(self.nodes[n * self.nf + self.f_self])
    }

    /// Largest object id in the file.
    pub fn max_id(&self) -> u32 {
        (0..self.node_count())
            .map(|n| self.node_id(n))
            .max()
            .unwrap_or(0)
    }

    /// Every object id, sorted.
    pub fn sorted_ids(&self) -> Vec<u32> {
        let mut v: Vec<u32> = (0..self.node_count()).map(|n| self.node_id(n)).collect();
        v.sort_unstable();
        v
    }

    /// The edge indexes of node `n`.
    pub fn edges_of(&self, n: usize) -> std::ops::Range<usize> {
        self.first_edge[n] as usize..self.first_edge[n + 1] as usize
    }

    pub(crate) fn edge_type(&self, e: usize) -> u32 {
        self.edges[e * self.ef + self.e_type]
    }

    /// The type name of an edge (`context`, `property`, ...).
    pub fn edge_type_name(&self, e: usize) -> &str {
        self.edge_types
            .get(self.edge_type(e) as usize)
            .map_or("", String::as_str)
    }

    /// The node an edge points at.
    pub fn edge_target(&self, e: usize) -> usize {
        self.edges[e * self.ef + self.e_to] as usize / self.nf
    }

    /// The name of a named edge (context, property, internal, shortcut, weak); `None`
    /// for an element or hidden edge, whose name field is an index.
    pub fn edge_name(&self, e: usize) -> Option<&str> {
        let t = self.edge_type(e);
        let k = &self.kinds;
        if Some(t) == k.e_element || Some(t) == k.e_hidden {
            return None;
        }
        Some(self.string(self.edges[e * self.ef + self.e_name]))
    }

    /// The index of an element or hidden edge.
    pub fn edge_index(&self, e: usize) -> u32 {
        self.edges[e * self.ef + self.e_name]
    }

    /// True for an edge that keeps its target alive and counts for retention: not weak,
    /// and not a shortcut unless it starts at the root (the DevTools rule).
    pub(crate) fn essential(&self, from: usize, t: u32) -> bool {
        Some(t) != self.kinds.e_weak && (Some(t) != self.kinds.e_shortcut || from == 0)
    }

    /// `(script_id, line, column)` of an object that has a location, by node index.
    pub fn locations(&self) -> impl Iterator<Item = (usize, u32, u32, u32)> + '_ {
        let lf = self.lf;
        (0..self.locations.len().checked_div(lf).unwrap_or(0)).map(move |i| {
            let r = &self.locations[i * lf..(i + 1) * lf];
            // `object_index` is an offset into the node array.
            (r[0] as usize / self.nf, r[1], r[2], r[3])
        })
    }

    /// True for a string-like node, whose name is data.
    pub(crate) fn is_stringish(&self, t: u32) -> bool {
        let k = &self.kinds;
        Some(t) == k.n_string || Some(t) == k.n_cons_string || Some(t) == k.n_sliced_string
    }

    /// The class name used by the histogram for a node: a constructor name, `(closure)
    /// name`, `(string)`, `(array)` and so on. Strings are never named after their text.
    pub fn class_name(&self, n: usize) -> String {
        let t = self.node_type(n);
        let k = &self.kinds;
        let name = self.node_name(n);
        if self.is_stringish(t) {
            return "(string)".to_owned();
        }
        if Some(t) == k.n_closure {
            return format!("(closure) {name}");
        }
        if Some(t) == k.n_array {
            return "(array)".to_owned();
        }
        if Some(t) == k.n_code {
            return "(code)".to_owned();
        }
        if Some(t) == k.n_synthetic {
            return format!("(synthetic) {name}");
        }
        if Some(t) == k.n_native {
            return format!("(native) {name}");
        }
        if Some(t) == k.n_object || Some(t) == k.n_hidden {
            return name.to_owned();
        }
        // regexp, number, symbol, bigint, object shape: one row per kind.
        format!("({})", self.node_type_name(n))
    }

    /// The class histogram: a row per class name with the count and the shallow bytes.
    pub fn histogram(&self) -> Histogram {
        let mut by: std::collections::BTreeMap<String, (u64, u64)> = Default::default();
        for n in 0..self.node_count() {
            let e = by.entry(self.class_name(n)).or_default();
            e.0 += 1;
            e.1 = e.1.saturating_add(self.self_size(n));
        }
        let mut rows: Vec<HistogramRow> = by
            .into_iter()
            .map(|(name, (count, bytes))| HistogramRow { name, count, bytes })
            .collect();
        rows.sort_by(|a, b| b.bytes.cmp(&a.bytes).then_with(|| a.name.cmp(&b.name)));
        Histogram {
            source: "v8-heapsnapshot".to_owned(),
            timestamp_ms: None,
            rows,
        }
    }
}

#[cfg(test)]
pub(crate) mod testing {
    //! A builder of small snapshots for the unit tests.

    use serde_json::json;

    /// `(type, name, self_size, [(edge type, name or index, target)])` per node.
    pub(crate) type N<'a> = (&'a str, &'a str, u32, Vec<(&'a str, &'a str, usize)>);

    const NODE_TYPES: [&str; 15] = [
        "hidden",
        "array",
        "string",
        "object",
        "code",
        "closure",
        "regexp",
        "number",
        "native",
        "synthetic",
        "concatenated string",
        "sliced string",
        "symbol",
        "bigint",
        "object shape",
    ];
    const EDGE_TYPES: [&str; 7] = [
        "context", "element", "property", "internal", "hidden", "shortcut", "weak",
    ];

    /// Builds the JSON text of a snapshot with the Node 22 field layout.
    pub(crate) fn build(nodes: &[N<'_>]) -> String {
        let mut strings: Vec<String> = vec![String::new()];
        let mut intern = |s: &str| -> u32 {
            if let Some(i) = strings.iter().position(|x| x == s) {
                return i as u32;
            }
            strings.push(s.to_owned());
            (strings.len() - 1) as u32
        };
        let mut nv: Vec<u32> = Vec::new();
        let mut ev: Vec<u32> = Vec::new();
        for (i, (t, name, size, edges)) in nodes.iter().enumerate() {
            let ti = NODE_TYPES.iter().position(|x| x == t).unwrap() as u32;
            nv.extend([
                ti,
                intern(name),
                (i as u32) * 2 + 1,
                *size,
                edges.len() as u32,
                0,
                0,
            ]);
            for (et, en, to) in edges {
                let eti = EDGE_TYPES.iter().position(|x| x == et).unwrap() as u32;
                let nm = if *et == "element" || *et == "hidden" {
                    en.parse::<u32>().unwrap_or(0)
                } else {
                    intern(en)
                };
                ev.extend([eti, nm, (*to as u32) * 7]);
            }
        }
        json!({
            "snapshot": {
                "meta": {
                    "node_fields": ["type","name","id","self_size","edge_count","trace_node_id","detachedness"],
                    "node_types": [NODE_TYPES, "string","number","number","number","number","number"],
                    "edge_fields": ["type","name_or_index","to_node"],
                    "edge_types": [EDGE_TYPES, "string_or_number", "node"],
                    "location_fields": ["object_index","script_id","line","column"],
                },
                "node_count": nodes.len(),
                "edge_count": ev.len() / 3,
            },
            "nodes": nv,
            "edges": ev,
            "trace_function_infos": [],
            "trace_tree": [],
            "samples": [],
            "locations": [],
            "strings": strings,
        })
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::testing::*;
    use super::*;

    fn tiny() -> String {
        build(&[
            ("synthetic", "", 0, vec![("element", "1", 1)]),
            ("synthetic", "(GC roots)", 0, vec![("element", "1", 2)]),
            ("object", "Holder", 32, vec![("property", "items", 3)]),
            (
                "object",
                "Array",
                16,
                vec![("element", "0", 4), ("element", "1", 5)],
            ),
            ("object", "Item", 24, vec![]),
            ("string", "secret-value", 24, vec![]),
        ])
    }

    #[test]
    fn a_small_snapshot_loads_and_makes_a_histogram() {
        let text = tiny();
        let s = Snapshot::read(text.as_bytes(), text.len() as u64).unwrap();
        assert_eq!(s.node_count(), 6);
        assert_eq!(s.edge_count(), 5);
        assert_eq!(s.node_name(2), "Holder");
        assert_eq!(s.edge_name(s.edges_of(2).start), Some("items"));
        assert_eq!(s.edge_name(s.edges_of(1).start), None);
        let h = s.histogram();
        let row = |n: &str| {
            h.rows
                .iter()
                .find(|r| r.name == n)
                .map(|r| (r.count, r.bytes))
        };
        assert_eq!(row("Item"), Some((1, 24)));
        // String text is data: only the class `(string)` shows.
        assert_eq!(row("(string)"), Some((1, 24)));
        assert!(h.rows.iter().all(|r| !r.name.contains("secret")));
    }

    #[test]
    fn damaged_snapshots_are_refused_with_a_reason() {
        let text = tiny();
        let cut = &text[..text.len() / 2];
        assert!(Snapshot::read(cut.as_bytes(), 100).is_err());
        assert!(matches!(
            Snapshot::read(&b"{\"a\":1}"[..], 10),
            Err(NodeError::NotThisFormat(_))
        ));
        // An edge that points past the node array.
        let bad = text.replacen("\"edges\":[", "\"edges\":[0,0,7000,", 1);
        assert!(Snapshot::read(bad.as_bytes(), bad.len() as u64).is_err());
        // The edge counts of the nodes must add up.
        let off = text.replace("\"edge_count\":5", "\"edge_count\":4");
        assert!(Snapshot::read(off.as_bytes(), off.len() as u64).is_ok());
    }

    #[test]
    fn the_preflight_refuses_a_file_that_does_not_fit() {
        let o = Options {
            max_bytes: 1 << 30,
            available_memory: Some(100 << 20),
        };
        assert!(preflight(10 << 20, &o).is_ok());
        let e = preflight(40 << 20, &o).unwrap_err().to_string();
        assert!(e.contains("available"), "{e}");
        let big = Options {
            max_bytes: 1 << 20,
            available_memory: None,
        };
        assert!(preflight(2 << 20, &big).is_err());
    }

    #[test]
    fn sniffing_looks_at_the_header() {
        let text = tiny();
        assert!(sniff(text.as_bytes()));
        assert!(!sniff(b"{\"head\":{}}"));
    }
}
