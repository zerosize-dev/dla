//! The two passes over a dump and the analysis built on them: classes and roots first,
//! then edges, dominators, retained sizes and holders.

use std::{
    collections::BTreeMap,
    fs::File,
    io::{self, Read, Seek},
    path::Path,
    time::Instant,
};

use super::{
    HprofError,
    graph::{Graph, dominators, retained},
    ids::{IdMap, IdTable, NONE},
    reader::{ClassDump, RawHeader, Rd, Visitor, read_header, scan, type_size},
    summary::{
        Accumulation, ClassStat, Counts, HeaderInfo, HeapSummary, Holder, HolderKind, SIZE_NOTES,
        Stats, name_hash,
    },
};

/// Limits of one analysis.
pub(crate) struct Params {
    pub max_objects: usize,
    pub max_passes: u32,
    pub compressed_oops: Option<bool>,
}

const K_CLASS: u8 = 0;
const K_INST: u8 = 1;
const K_OBJARR: u8 = 2;
const K_PRIMARR: u8 = 3;

const TOP_CLASSES: usize = 25;
const TOP_HOLDERS: usize = 20;
const TRACKED: usize = 200;
const DETAILED: usize = 60;
const MAX_PATH: usize = 12;
const FAN_OUT: u32 = 16;
const MAX_INSTANCE_BYTES: u32 = 64 << 20;

const PRIM_NAMES: [&str; 8] = [
    "boolean", "char", "float", "double", "byte", "short", "int", "long",
];

fn root_kind_name(kind: u8) -> &'static str {
    match kind {
        0xff => "unknown",
        0x01 => "jni-global",
        0x02 => "jni-local",
        0x03 => "java-frame",
        0x04 => "native-stack",
        0x05 => "sticky-class",
        0x06 => "thread-block",
        0x07 => "monitor-used",
        0x08 => "thread-obj",
        0x89 => "interned-string",
        0x8a => "finalizing",
        0x8b => "debugger",
        0x8c => "reference-cleanup",
        0x8d => "vm-internal",
        0x8e => "jni-monitor",
        0x8f => "root-8f",
        0x90 => "unreachable",
        _ => "other",
    }
}

/// `java/util/Map$Entry` to `java.util.Map$Entry`; `[Ljava/lang/Object;` to
/// `java.lang.Object[]`; `[B` to `byte[]`.
pub fn pretty_class(raw: &str) -> String {
    let dims = raw.bytes().take_while(|&b| b == b'[').count();
    if dims == 0 {
        return raw.replace('/', ".");
    }
    let rest = &raw[dims..];
    let base = match rest.as_bytes().first() {
        Some(b'L') => rest[1..].trim_end_matches(';').replace('/', "."),
        Some(b'B') => "byte".into(),
        Some(b'C') => "char".into(),
        Some(b'D') => "double".into(),
        Some(b'F') => "float".into(),
        Some(b'I') => "int".into(),
        Some(b'J') => "long".into(),
        Some(b'S') => "short".into(),
        Some(b'Z') => "boolean".into(),
        _ => rest.replace('/', "."),
    };
    format!("{base}{}", "[]".repeat(dims))
}

// ---------------------------------------------------------------------------------------
// Pass 1
// ---------------------------------------------------------------------------------------

#[derive(Default)]
struct Frame {
    method: u64,
    file: u64,
    class_serial: u32,
    line: i32,
}

struct Pass1 {
    strings: IdMap<u64, (u64, u32)>,
    load: IdMap<u64, u64>,
    serial_class: IdMap<u32, u64>,
    frames: IdMap<u64, Frame>,
    traces: IdMap<u32, Vec<u64>>,
    classes: Vec<ClassDump>,
    ids: Vec<u64>,
    offs: Vec<u64>,
    kinds: Vec<u8>,
    roots: Vec<(u8, u64, u32, u32)>,
    root_counts: [u64; 256],
    n_inst: u64,
    n_objarr: u64,
    n_primarr: u64,
    n_frames: u64,
    max: usize,
    overflow: bool,
}

impl Pass1 {
    fn add(&mut self, id: u64, off: u64, kind: u8) {
        if self.ids.len() >= self.max {
            self.overflow = true;
            return;
        }
        self.ids.push(id);
        self.offs.push(off);
        self.kinds.push(kind);
    }
}

impl Visitor for Pass1 {
    fn utf8(&mut self, id: u64, off: u64, len: u32) {
        self.strings.insert(id, (off, len));
    }
    fn load_class(&mut self, serial: u32, class_id: u64, name_id: u64) {
        self.load.insert(class_id, name_id);
        self.serial_class.insert(serial, class_id);
    }
    fn frame(&mut self, id: u64, method: u64, file: u64, class_serial: u32, line: i32) {
        self.n_frames += 1;
        self.frames.insert(
            id,
            Frame {
                method,
                file,
                class_serial,
                line,
            },
        );
    }
    fn trace(&mut self, serial: u32, _thread: u32, frames: &[u64]) {
        self.traces
            .insert(serial, frames.iter().copied().take(8).collect());
    }
    fn root(&mut self, kind: u8, id: u64, a: u32, b: u32) {
        self.root_counts[kind as usize] += 1;
        self.roots.push((kind, id, a, b));
    }
    fn class_dump(&mut self, off: u64, c: ClassDump) {
        let id = c.id;
        self.classes.push(c);
        self.add(id, off, K_CLASS);
    }
    fn instance(
        &mut self,
        off: u64,
        id: u64,
        _class_id: u64,
        _len: u32,
        _rd: &mut Rd<impl Read + Seek>,
    ) -> io::Result<()> {
        self.n_inst += 1;
        self.add(id, off, K_INST);
        Ok(())
    }
    fn obj_array(
        &mut self,
        off: u64,
        id: u64,
        _class_id: u64,
        _n: u32,
        _rd: &mut Rd<impl Read + Seek>,
    ) -> io::Result<()> {
        self.n_objarr += 1;
        self.add(id, off, K_OBJARR);
        Ok(())
    }
    fn prim_array(&mut self, off: u64, id: u64, _elem: u8, _n: u32) {
        self.n_primarr += 1;
        self.add(id, off, K_PRIMARR);
    }
    fn stopped(&self) -> bool {
        self.overflow
    }
}

// ---------------------------------------------------------------------------------------
// Strings
// ---------------------------------------------------------------------------------------

struct Strings {
    f: File,
    map: IdMap<u64, (u64, u32)>,
    cache: IdMap<u64, String>,
}

impl Strings {
    fn get(&mut self, id: u64) -> String {
        if let Some(s) = self.cache.get(&id) {
            return s.clone();
        }
        let s = match self.map.get(&id) {
            Some(&(off, len)) => {
                let n = (len as usize).min(1 << 16);
                let mut buf = vec![0u8; n];
                let ok = self.f.seek(io::SeekFrom::Start(off)).is_ok()
                    && self.f.read_exact(&mut buf).is_ok();
                if ok {
                    String::from_utf8_lossy(&buf).into_owned()
                } else {
                    format!("<string {id:#x}>")
                }
            }
            None => format!("<string {id:#x}>"),
        };
        self.cache.insert(id, s.clone());
        s
    }
}

// ---------------------------------------------------------------------------------------
// Layouts
// ---------------------------------------------------------------------------------------

struct Layout {
    /// Offsets of strong object-typed fields in the instance data.
    ref_offs: Vec<u32>,
    /// Assumed size of an instance in bytes: header plus fields, rounded up to 8.
    shallow: u32,
}

#[derive(Clone, Copy)]
struct Slot {
    decl: u32,
    name: u64,
    ty: u8,
    off: u32,
}

struct Shape<'a> {
    classes: &'a [ClassDump],
    class_idx: &'a IdMap<u64, u32>,
    id_size: usize,
}

impl Shape<'_> {
    /// The instance fields of `cidx` and its superclasses, own class first.
    fn slots(&self, cidx: u32) -> Vec<Slot> {
        let mut out = Vec::new();
        let mut off = 0u32;
        let mut cur = cidx;
        for _ in 0..4096 {
            let c = &self.classes[cur as usize];
            for &(name, ty) in &c.fields {
                out.push(Slot {
                    decl: cur,
                    name,
                    ty,
                    off,
                });
                off += type_size(ty, self.id_size).unwrap_or(0) as u32;
            }
            match self.class_idx.get(&c.super_id) {
                Some(&s) if c.super_id != 0 => cur = s,
                _ => break,
            }
        }
        out
    }
}

// ---------------------------------------------------------------------------------------
// Pass 2
// ---------------------------------------------------------------------------------------

struct Build<'a> {
    ids: &'a [u64],
    table: &'a IdTable,
    class_idx: &'a IdMap<u64, u32>,
    layouts: &'a [Layout],
    kinds: &'a [u8],
    prim_base: u32,
    unknown_ty: u32,
    ref_size: u64,
    arr_hdr: u64,
    k: usize,
    eoff: Vec<u32>,
    edges: Vec<u32>,
    shallow: Vec<u32>,
    ty: Vec<u32>,
    hist: Vec<(u64, u64)>,
    statics_out: Vec<(u32, u64, u32)>,
    scratch: Vec<u8>,
    oversized_instances: u64,
    mismatch: Option<String>,
    edge_overflow: bool,
}

fn align8(x: u64) -> u64 {
    (x + 7) & !7
}

impl Build<'_> {
    fn begin(&mut self, kind: u8) -> bool {
        if self.kinds.get(self.k) != Some(&kind) {
            if self.mismatch.is_none() {
                self.mismatch = Some(format!(
                    "the two passes disagree at object {} (the file changed between passes?)",
                    self.k
                ));
            }
            return false;
        }
        self.eoff.push(self.edges.len() as u32);
        true
    }

    fn edge(&mut self, target_id: u64) {
        let t = self.table.find(self.ids, target_id);
        if t == NONE {
            return;
        }
        if self.edges.len() >= u32::MAX as usize - 8 {
            self.edge_overflow = true;
            return;
        }
        if self.edges.last() == Some(&t)
            && self.edges.len() as u32 > *self.eoff.last().unwrap_or(&0)
        {
            return;
        }
        self.edges.push(t);
    }

    fn account(&mut self, ty: u32, shallow: u64) {
        self.ty.push(ty);
        self.shallow.push(shallow.min(u64::from(u32::MAX)) as u32);
        let h = &mut self.hist[ty as usize];
        h.0 += 1;
        h.1 += shallow;
    }
}

impl Visitor for Build<'_> {
    fn class_dump(&mut self, _off: u64, c: ClassDump) {
        if !self.begin(K_CLASS) {
            self.k += 1;
            return;
        }
        let cidx = self
            .class_idx
            .get(&c.id)
            .copied()
            .unwrap_or(self.unknown_ty);
        for &(name, value) in &c.statics {
            let before = self.edges.len();
            self.edge(value);
            if self.edges.len() > before {
                let t = self.edges[self.edges.len() - 1];
                self.statics_out.push((cidx, name, t));
            }
        }
        self.edge(c.super_id);
        self.edge(c.loader_id);
        self.ty.push(cidx);
        self.shallow.push(0);
        self.k += 1;
    }

    fn instance(
        &mut self,
        _off: u64,
        _id: u64,
        class_id: u64,
        len: u32,
        rd: &mut Rd<impl Read + Seek>,
    ) -> io::Result<()> {
        if !self.begin(K_INST) {
            self.k += 1;
            return Ok(());
        }
        let cidx = self.class_idx.get(&class_id).copied();
        let ty = cidx.unwrap_or(self.unknown_ty);
        let mut sh = 16;
        if let Some(c) = cidx {
            sh = u64::from(self.layouts[c as usize].shallow);
            if len > MAX_INSTANCE_BYTES {
                self.oversized_instances += 1;
            } else {
                rd.read_vec(len as usize, &mut self.scratch)?;
                let ids = rd.id_size;
                let data = std::mem::take(&mut self.scratch);
                let layouts = self.layouts;
                for &o in &layouts[c as usize].ref_offs {
                    let o = o as usize;
                    if o + ids <= data.len() {
                        let id = if ids == 8 {
                            u64::from_be_bytes(data[o..o + 8].try_into().unwrap_or([0; 8]))
                        } else {
                            u64::from(u32::from_be_bytes(
                                data[o..o + 4].try_into().unwrap_or([0; 4]),
                            ))
                        };
                        self.edge(id);
                    }
                }
                self.scratch = data;
            }
        }
        self.account(ty, sh.max(8));
        self.k += 1;
        Ok(())
    }

    fn obj_array(
        &mut self,
        _off: u64,
        _id: u64,
        class_id: u64,
        n: u32,
        rd: &mut Rd<impl Read + Seek>,
    ) -> io::Result<()> {
        if !self.begin(K_OBJARR) {
            self.k += 1;
            return Ok(());
        }
        for _ in 0..n {
            let id = rd.id()?;
            if id != 0 {
                self.edge(id);
            }
        }
        let ty = self
            .class_idx
            .get(&class_id)
            .copied()
            .unwrap_or(self.unknown_ty);
        let sh = align8(self.arr_hdr + u64::from(n) * self.ref_size);
        self.account(ty, sh);
        self.k += 1;
        Ok(())
    }

    fn prim_array(&mut self, _off: u64, _id: u64, elem: u8, n: u32) {
        if !self.begin(K_PRIMARR) {
            self.k += 1;
            return;
        }
        let (ty, sz) = if (4..=11).contains(&elem) {
            (
                self.prim_base + u32::from(elem - 4),
                type_size(elem, 8).unwrap_or(1) as u64,
            )
        } else {
            (self.unknown_ty, 1)
        };
        let sh = align8(self.arr_hdr + u64::from(n) * sz);
        self.account(ty, sh);
        self.k += 1;
    }
}

// ---------------------------------------------------------------------------------------
// Orchestration
// ---------------------------------------------------------------------------------------

struct Syn {
    key: String,
    kind: HolderKind,
    owner_class: String,
    field: Option<String>,
    /// The thread object, for labeling the first hop.
    thread_obj: Option<u32>,
    context: Vec<String>,
}

enum Src {
    Static { class: u32, name: u64 },
    Syn(usize),
    Other(u8),
}

struct Cand {
    retained: u64,
    node: u32,
    src: Src,
}

struct Analysis<'a> {
    path: &'a Path,
    header: &'a RawHeader,
    ids: Vec<u64>,
    offs: Vec<u64>,
    kinds: Vec<u8>,
    ty: Vec<u32>,
    class_names: Vec<String>,
    classes: Vec<ClassDump>,
    class_idx: IdMap<u64, u32>,
    strs: Strings,
    g: Graph,
    syns: Vec<Syn>,
    statics_out: Vec<(u32, u64, u32)>,
    ret: Vec<u64>,
    cnt: Vec<u32>,
    kidc: Vec<u32>,
    heavy: Vec<u32>,
    rd: Rd<File>,
    prim_base: u32,
    table: Option<IdTable>,
}

impl Analysis<'_> {
    fn find(&self, id: u64) -> Option<u32> {
        let t = self.table.as_ref()?.find(&self.ids, id);
        (t != NONE).then_some(t)
    }

    fn shape(&self) -> Shape<'_> {
        Shape {
            classes: &self.classes,
            class_idx: &self.class_idx,
            id_size: self.header.id_size as usize,
        }
    }

    fn ty_name(&self, ty: u32) -> String {
        let t = ty as usize;
        if t < self.class_names.len() {
            self.class_names[t].clone()
        } else if ty >= self.prim_base && ty < self.prim_base + 8 {
            format!("{}[]", PRIM_NAMES[(ty - self.prim_base) as usize])
        } else {
            "<unknown class>".into()
        }
    }

    fn node_class(&self, v: u32) -> String {
        let i = v as usize;
        if i < self.g.n {
            if self.kinds[i] == K_CLASS {
                format!("class {}", self.ty_name(self.ty[i]))
            } else {
                self.ty_name(self.ty[i])
            }
        } else if i < self.g.n + self.syns.len() {
            self.syns[i - self.g.n].key.clone()
        } else {
            "<root>".into()
        }
    }

    /// Reads the instance record of object node `v`: its class index and data.
    fn read_instance(&mut self, v: u32) -> Option<(u32, Vec<u8>)> {
        if self.kinds[v as usize] != K_INST {
            return None;
        }
        self.rd.seek_to(self.offs[v as usize]).ok()?;
        self.rd.u1().ok()?;
        self.rd.id().ok()?;
        self.rd.u4().ok()?;
        let class_id = self.rd.id().ok()?;
        let n = self.rd.u4().ok()?;
        if n > MAX_INSTANCE_BYTES {
            return None;
        }
        let mut data = Vec::new();
        self.rd.read_vec(n as usize, &mut data).ok()?;
        Some((*self.class_idx.get(&class_id)?, data))
    }

    fn id_at(&self, data: &[u8], off: u32) -> u64 {
        let o = off as usize;
        if self.header.id_size == 8 {
            data.get(o..o + 8)
                .map_or(0, |b| u64::from_be_bytes(b.try_into().unwrap_or([0; 8])))
        } else {
            data.get(o..o + 4).map_or(0, |b| {
                u64::from(u32::from_be_bytes(b.try_into().unwrap_or([0; 4])))
            })
        }
    }

    fn slot_named(&mut self, cidx: u32, field: &str, decl_class: Option<&str>) -> Option<Slot> {
        let slots = self.shape().slots(cidx);
        let mut fallback = None;
        for s in slots {
            if self.strs.get(s.name) == field {
                if let Some(d) = decl_class
                    && self.class_names[s.decl as usize] == d
                {
                    return Some(s);
                }
                fallback.get_or_insert(s);
            }
        }
        fallback
    }

    /// The text of a `java.lang.String` object (first 200 characters).
    fn java_string(&mut self, v: u32) -> Option<String> {
        let (cidx, data) = self.read_instance(v)?;
        let value = self.slot_named(cidx, "value", Some("java.lang.String"))?;
        let coder = self
            .slot_named(cidx, "coder", Some("java.lang.String"))
            .filter(|s| s.ty == 8);
        let arr = self.id_at(&data, value.off);
        let arr_node = self.find(arr)?;
        if self.kinds[arr_node as usize] != K_PRIMARR {
            return None;
        }
        self.rd.seek_to(self.offs[arr_node as usize]).ok()?;
        self.rd.u1().ok()?;
        self.rd.id().ok()?;
        self.rd.u4().ok()?;
        let n = self.rd.u4().ok()? as usize;
        let t = self.rd.u1().ok()?;
        let take = n.min(400);
        let mut bytes = Vec::new();
        match t {
            8 => {
                self.rd.read_vec(take, &mut bytes).ok()?;
                let utf16 = coder.is_some_and(|c| data.get(c.off as usize) == Some(&1));
                Some(if utf16 {
                    let u: Vec<u16> = bytes
                        .chunks_exact(2)
                        .map(|c| u16::from_le_bytes([c[0], c[1]]))
                        .collect();
                    String::from_utf16_lossy(&u)
                } else {
                    bytes.iter().map(|&b| b as char).collect()
                })
            }
            5 => {
                self.rd.read_vec((take * 2).min(n * 2), &mut bytes).ok()?;
                let u: Vec<u16> = bytes
                    .chunks_exact(2)
                    .map(|c| u16::from_be_bytes([c[0], c[1]]))
                    .collect();
                Some(String::from_utf16_lossy(&u))
            }
            _ => None,
        }
    }

    /// Label of the reference `p -> c` for a path entry.
    fn edge_label(&mut self, p: u32, c: u32) -> String {
        let n = self.g.n;
        let pi = p as usize;
        if pi >= n {
            if pi < n + self.syns.len() {
                let s = &self.syns[pi - n];
                return match s.kind {
                    HolderKind::Thread if s.thread_obj == Some(c) => "thread object".into(),
                    HolderKind::Thread => "stack local".into(),
                    _ => "jni global".into(),
                };
            }
            return "root".into();
        }
        if !self.g.succ(p).contains(&c) {
            return "~".into();
        }
        match self.kinds[pi] {
            K_CLASS => {
                let cidx = self.ty[pi];
                let hit = self
                    .statics_out
                    .iter()
                    .find(|&&(ci, _, t)| ci == cidx && t == c)
                    .map(|&(_, name, _)| name);
                match hit {
                    Some(name) => format!("static {}", self.strs.get(name)),
                    None if self.ty[c as usize] == self.ty[pi] => "super".into(),
                    None => "loader".into(),
                }
            }
            K_OBJARR => "[]".into(),
            K_INST => {
                let child_id = self.ids[c as usize];
                let Some((cidx, data)) = self.read_instance(p) else {
                    return "field".into();
                };
                let slots = self.shape().slots(cidx);
                for s in slots {
                    if s.ty == 2 && self.id_at(&data, s.off) == child_id {
                        let decl = self.class_names[s.decl as usize].clone();
                        let short = decl.rsplit('.').next().unwrap_or(&decl).to_owned();
                        return format!("{short}.{}", self.strs.get(s.name));
                    }
                }
                "field".into()
            }
            _ => "~".into(),
        }
    }

    fn thread_name(&mut self, obj: u32) -> Option<String> {
        let (cidx, data) = self.read_instance(obj)?;
        let slot = self.slot_named(cidx, "name", Some("java.lang.Thread"))?;
        if slot.ty != 2 {
            return None;
        }
        let id = self.id_at(&data, slot.off);
        let node = self.find(id)?;
        self.java_string(node).filter(|s| !s.is_empty())
    }
}

fn peak_rss_kib() -> Option<u64> {
    let s = std::fs::read_to_string("/proc/self/status").ok()?;
    s.lines()
        .find(|l| l.starts_with("VmHWM:"))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

fn ms(t: Instant) -> u64 {
    t.elapsed().as_millis() as u64
}

/// Analyzes the plain (not gzip) HPROF file at `path`.
pub(crate) fn analyze(path: &Path, p: &Params) -> Result<HeapSummary, HprofError> {
    let header = read_header(&mut io::BufReader::new(File::open(path)?))?;
    let id_size = header.id_size as usize;
    let mut rd = Rd::new(File::open(path)?, id_size)?;
    let file_bytes = rd.file_len();
    let mut warnings: Vec<String> = Vec::new();

    // ---- pass 1
    let t1 = Instant::now();
    let mut p1 = Pass1 {
        strings: IdMap::default(),
        load: IdMap::default(),
        serial_class: IdMap::default(),
        frames: IdMap::default(),
        traces: IdMap::default(),
        classes: Vec::new(),
        ids: Vec::new(),
        offs: Vec::new(),
        kinds: Vec::new(),
        roots: Vec::new(),
        root_counts: [0; 256],
        n_inst: 0,
        n_objarr: 0,
        n_primarr: 0,
        n_frames: 0,
        max: p.max_objects.min(u32::MAX as usize - 1024),
        overflow: false,
    };
    scan(&mut rd, &header, &mut p1, &mut warnings)?;
    let pass1_ms = ms(t1);

    let mut strs = Strings {
        f: File::open(path)?,
        map: std::mem::take(&mut p1.strings),
        cache: IdMap::default(),
    };
    let classes = std::mem::take(&mut p1.classes);
    let mut class_idx: IdMap<u64, u32> = IdMap::default();
    for (i, c) in classes.iter().enumerate() {
        class_idx.insert(c.id, i as u32);
    }
    let class_names: Vec<String> = classes
        .iter()
        .map(|c| match p1.load.get(&c.id) {
            Some(&n) => pretty_class(&strs.get(n)),
            None => format!("<class {:#x}>", c.id),
        })
        .collect();
    let mut class_hashes: Vec<u64> = p1.load.values().map(|&n| name_hash(&strs.get(n))).collect();
    class_hashes.sort_unstable();
    class_hashes.dedup();
    let mut app_class_hashes: Vec<u64> = classes
        .iter()
        .filter(|c| c.loader_id != 0)
        .filter_map(|c| p1.load.get(&c.id).copied())
        .map(|n| name_hash(&strs.get(n)))
        .collect();
    app_class_hashes.sort_unstable();
    app_class_hashes.dedup();

    let mut gc_roots_by_kind = BTreeMap::new();
    let mut gc_roots = 0u64;
    for (k, &c) in p1.root_counts.iter().enumerate() {
        if c > 0 {
            gc_roots += c;
            gc_roots_by_kind.insert(root_kind_name(k as u8).to_owned(), c);
        }
    }
    let mut counts = Counts {
        objects: p1.ids.len() as u64,
        classes: classes.len() as u64,
        instances: p1.n_inst,
        object_arrays: p1.n_objarr,
        primitive_arrays: p1.n_primarr,
        strings: strs.map.len() as u64,
        frames: p1.n_frames,
        gc_roots,
        ..Counts::default()
    };
    let head = HeaderInfo {
        version: header.version.clone(),
        id_size: header.id_size,
        timestamp_ms: header.timestamp_ms,
    };
    let mut stats = Stats {
        file_bytes,
        pass1_ms,
        ..Stats::default()
    };
    let mut notes: Vec<String> = SIZE_NOTES.iter().map(|s| (*s).to_owned()).collect();

    if p1.overflow {
        warnings.push(format!(
            "object cap of {} reached: the scan stopped, counts are partial and no holders were computed (raise it with --max-objects if memory allows)",
            p.max_objects
        ));
        stats.peak_rss_kib = peak_rss_kib();
        return Ok(HeapSummary {
            header: head,
            counts,
            gc_roots_by_kind,
            top_classes: Vec::new(),
            holders: Vec::new(),
            notes,
            truncated: true,
            warnings,
            stats,
            tracked: Vec::new(),
            class_hashes,
            app_class_hashes,
        });
    }

    // ---- layouts
    let shape = Shape {
        classes: &classes,
        class_idx: &class_idx,
        id_size,
    };
    let ref_cls = class_names
        .iter()
        .position(|n| n == "java.lang.ref.Reference")
        .map(|i| i as u32);
    let mut referent_names: Vec<u64> = Vec::new();
    if let Some(rc) = ref_cls {
        for &(name, _) in &classes[rc as usize].fields {
            if strs.get(name) == "referent" {
                referent_names.push(name);
            }
        }
    }
    // HPROF carries no object sizes (a HotSpot class dump's instance size is the sum of
    // its fields with identifier-sized references and no header), so sizes are assumed:
    // 64-bit dumps of heaps under 24 GiB are taken to use compressed oops.
    let compressed = id_size == 8 && p.compressed_oops.unwrap_or(file_bytes < (24 << 30));
    let (obj_hdr, ref_size, arr_hdr): (u64, u64, u64) = if id_size == 4 {
        (8, 4, 12)
    } else if compressed {
        (12, 4, 16)
    } else {
        (16, 8, 24)
    };
    stats.compressed_oops = compressed;
    let mut layouts: Vec<Layout> = Vec::with_capacity(classes.len());
    for i in 0..classes.len() {
        let slots = shape.slots(i as u32);
        let mut ref_offs = Vec::new();
        let mut bytes = 0u64;
        for s in &slots {
            if s.ty == 2 {
                if !(Some(s.decl) == ref_cls && referent_names.contains(&s.name)) {
                    ref_offs.push(s.off);
                }
                bytes += ref_size;
            } else {
                bytes += type_size(s.ty, id_size).unwrap_or(0) as u64;
            }
        }
        layouts.push(Layout {
            ref_offs,
            shallow: align8(obj_hdr + bytes).min(u64::from(u32::MAX)) as u32,
        });
    }
    notes.push(format!(
        "assumed sizes: object header {obj_hdr} bytes, references {ref_size} bytes, array header {arr_hdr} bytes{}",
        if compressed { " (compressed oops assumed for this 64-bit dump)" } else { "" }
    ));

    // ---- pass 2
    let t2 = Instant::now();
    let table = IdTable::build(&p1.ids);
    let prim_base = classes.len() as u32;
    let mut build = Build {
        ids: &p1.ids,
        table: &table,
        class_idx: &class_idx,
        layouts: &layouts,
        kinds: &p1.kinds,
        prim_base,
        unknown_ty: prim_base + 8,
        ref_size,
        arr_hdr,
        k: 0,
        eoff: Vec::with_capacity(p1.ids.len() + 1),
        edges: Vec::with_capacity(p1.ids.len() + p1.ids.len() / 2),
        shallow: Vec::with_capacity(p1.ids.len()),
        ty: Vec::with_capacity(p1.ids.len()),
        hist: vec![(0, 0); classes.len() + 9],
        statics_out: Vec::new(),
        scratch: Vec::new(),
        oversized_instances: 0,
        mismatch: None,
        edge_overflow: false,
    };
    scan(&mut rd, &header, &mut build, &mut warnings)?;
    build.eoff.push(build.edges.len() as u32);
    if let Some(m) = build.mismatch.take() {
        return Err(HprofError::Malformed(m));
    }
    if build.k != p1.ids.len() {
        return Err(HprofError::Malformed(format!(
            "pass 2 saw {} objects, pass 1 saw {}",
            build.k,
            p1.ids.len()
        )));
    }
    let mut truncated = false;
    if build.edge_overflow {
        truncated = true;
        warnings.push("more than 4 billion reference edges: edges beyond that were dropped".into());
    }
    if build.oversized_instances > 0 {
        warnings.push(format!(
            "{} instances larger than {} MiB were not scanned for references",
            build.oversized_instances,
            MAX_INSTANCE_BYTES >> 20
        ));
    }
    let Build {
        eoff,
        edges,
        shallow,
        ty,
        hist,
        statics_out,
        ..
    } = build;
    counts.edges = edges.len() as u64;
    stats.pass2_ms = ms(t2);

    // ---- roots, threads, synthetic nodes
    let n = p1.ids.len();
    let mut an = Analysis {
        path,
        header: &header,
        ids: std::mem::take(&mut p1.ids),
        offs: std::mem::take(&mut p1.offs),
        kinds: std::mem::take(&mut p1.kinds),
        ty,
        class_names,
        classes,
        class_idx,
        strs,
        g: Graph {
            n,
            eoff,
            edges,
            syn: Vec::new(),
            root_succ: Vec::new(),
        },
        syns: Vec::new(),
        statics_out,
        ret: Vec::new(),
        cnt: Vec::new(),
        kidc: Vec::new(),
        heavy: Vec::new(),
        rd: Rd::new(File::open(path)?, id_size)?,
        prim_base,
        table: Some(table),
    };
    let _ = an.path;

    struct ThreadInfo {
        obj: Option<u32>,
        locals: Vec<u32>,
        trace: u32,
    }
    let mut threads: BTreeMap<u32, ThreadInfo> = BTreeMap::new();
    let mut jni: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    let mut other: Vec<(u8, u32)> = Vec::new();
    for &(kind, id, a, b) in &p1.roots {
        let Some(node) = an.find(id) else { continue };
        match kind {
            0x08 => {
                let t = threads.entry(a).or_insert(ThreadInfo {
                    obj: None,
                    locals: Vec::new(),
                    trace: 0,
                });
                t.obj = Some(node);
                t.trace = b;
            }
            0x02..=0x04 | 0x06 => {
                threads
                    .entry(a)
                    .or_insert(ThreadInfo {
                        obj: None,
                        locals: Vec::new(),
                        trace: 0,
                    })
                    .locals
                    .push(node);
            }
            0x01 => {
                if an.kinds[node as usize] != K_CLASS {
                    let name = an.ty_name(an.ty[node as usize]);
                    jni.entry(name).or_default().push(node);
                }
            }
            0x90 => {}
            _ => {
                if an.kinds[node as usize] != K_CLASS {
                    other.push((kind, node));
                }
            }
        }
    }
    other.sort_unstable();
    other.dedup();
    let mut syn_succ: Vec<Vec<u32>> = Vec::new();
    let mut syns: Vec<Syn> = Vec::new();
    for (serial, t) in &threads {
        let mut succ: Vec<u32> = t.obj.into_iter().chain(t.locals.iter().copied()).collect();
        succ.sort_unstable();
        succ.dedup();
        if succ.is_empty() {
            continue;
        }
        let name = t
            .obj
            .and_then(|o| an.thread_name(o))
            .unwrap_or_else(|| format!("thread-{serial}"));
        let mut context = Vec::new();
        if let Some(frames) = p1.traces.get(&t.trace) {
            for f in frames.iter().take(3) {
                if let Some(fr) = p1.frames.get(f) {
                    let class = p1
                        .serial_class
                        .get(&fr.class_serial)
                        .and_then(|cid| p1.load.get(cid))
                        .map(|&nid| pretty_class(&an.strs.get(nid)))
                        .unwrap_or_default();
                    let method = an.strs.get(fr.method);
                    let file = an.strs.get(fr.file);
                    context.push(format!("{class}.{method}({file}:{})", fr.line));
                }
            }
        }
        syns.push(Syn {
            key: format!("thread:{name}"),
            kind: HolderKind::Thread,
            owner_class: name,
            field: None,
            thread_obj: t.obj,
            context,
        });
        syn_succ.push(succ);
    }
    for (class, nodes) in &mut jni {
        nodes.sort_unstable();
        nodes.dedup();
        syns.push(Syn {
            key: format!("jni-global:{class}"),
            kind: HolderKind::JniGlobal,
            owner_class: class.clone(),
            field: None,
            thread_obj: None,
            context: Vec::new(),
        });
        syn_succ.push(nodes.clone());
    }
    let mut root_succ: Vec<u32> = (0..n as u32)
        .filter(|&i| an.kinds[i as usize] == K_CLASS)
        .collect();
    root_succ.extend((0..syns.len()).map(|i| (n + i) as u32));
    root_succ.extend(other.iter().map(|&(_, v)| v));
    an.g.syn = syn_succ;
    an.g.root_succ = root_succ;
    an.syns = syns;
    an.table = None;
    drop(p1);

    // ---- dominators
    let t3 = Instant::now();
    let dom = dominators(&an.g, p.max_passes);
    if !dom.converged {
        truncated = true;
        warnings.push(format!(
            "dominator iteration stopped at the cap of {} passes: retained sizes are approximate",
            p.max_passes
        ));
    }
    let (ret, cnt) = retained(&an.g, &dom, &shallow);
    let nn = an.g.nodes();
    let mut kidc = vec![0u32; nn];
    let mut heavy = vec![NONE; nn];
    let root = an.g.root();
    let mut reachable = 0u64;
    for &v in &dom.post {
        if (v as usize) < n {
            reachable += 1;
        }
        if v == root {
            continue;
        }
        let par = dom.idom[v as usize];
        if par == NONE {
            continue;
        }
        kidc[par as usize] += 1;
        let h = heavy[par as usize];
        if h == NONE || ret[v as usize] > ret[h as usize] {
            heavy[par as usize] = v;
        }
    }
    counts.reachable = reachable;
    counts.unreachable = n as u64 - reachable;
    stats.dominator_passes = dom.passes;
    stats.dominators_ms = ms(t3);
    drop(dom);
    an.ret = ret;
    an.cnt = cnt;
    an.kidc = kidc;
    an.heavy = heavy;
    drop(shallow);

    // ---- histogram
    let mut top: Vec<ClassStat> = hist
        .iter()
        .enumerate()
        .filter(|(_, h)| h.0 > 0)
        .map(|(i, h)| ClassStat {
            class: an.ty_name(i as u32),
            count: h.0,
            shallow: h.1,
        })
        .collect();
    top.sort_by(|a, b| {
        b.shallow
            .cmp(&a.shallow)
            .then_with(|| a.class.cmp(&b.class))
    });
    top.truncate(TOP_CLASSES);

    // ---- holders
    let mut cands: Vec<Cand> = Vec::new();
    for &(class, name, target) in &an.statics_out {
        if an.kinds[target as usize] == K_CLASS {
            continue;
        }
        cands.push(Cand {
            retained: an.ret[target as usize],
            node: target,
            src: Src::Static { class, name },
        });
    }
    for i in 0..an.syns.len() {
        let node = (n + i) as u32;
        cands.push(Cand {
            retained: an.ret[node as usize],
            node,
            src: Src::Syn(i),
        });
    }
    for &(kind, node) in &other {
        cands.push(Cand {
            retained: an.ret[node as usize],
            node,
            src: Src::Other(kind),
        });
    }
    cands.sort_by(|a, b| b.retained.cmp(&a.retained).then(a.node.cmp(&b.node)));
    cands.truncate(TRACKED + 100);
    let mut merged: Vec<(Holder, u32)> = Vec::new();
    let mut by_key: IdMap<u64, usize> = IdMap::default();
    for c in &cands {
        let h = an.make_holder(c);
        let hk = name_hash(&h.key);
        match by_key.get(&hk) {
            Some(&i) if merged[i].0.key == h.key => {
                merged[i].0.retained += h.retained;
                merged[i].0.dominated_count += h.dominated_count;
            }
            _ => {
                by_key.insert(hk, merged.len());
                merged.push((h, c.node));
            }
        }
    }
    merged.sort_by(|a, b| {
        b.0.retained
            .cmp(&a.0.retained)
            .then_with(|| a.0.key.cmp(&b.0.key))
    });
    merged.truncate(TRACKED);
    for (h, node) in merged.iter_mut().take(DETAILED) {
        let (path, acc) = an.walk(h, *node);
        h.path = path;
        h.accumulation = acc;
    }
    let merged: Vec<Holder> = merged.into_iter().map(|(h, _)| h).collect();
    let holders: Vec<Holder> = merged.iter().take(TOP_HOLDERS).cloned().collect();

    if !truncated {
        notes.push("the dominator iteration converged".into());
    }
    stats.peak_rss_kib = peak_rss_kib();
    Ok(HeapSummary {
        header: head,
        counts,
        gc_roots_by_kind,
        top_classes: top,
        holders,
        notes,
        truncated,
        warnings,
        stats,
        tracked: merged,
        class_hashes,
        app_class_hashes,
    })
}

impl Analysis<'_> {
    fn holder_key(&mut self, c: &Cand) -> String {
        match c.src {
            Src::Static { class, name } => {
                format!(
                    "static:{}.{}",
                    self.class_names[class as usize],
                    self.strs.get(name)
                )
            }
            Src::Syn(i) => self.syns[i].key.clone(),
            Src::Other(kind) => format!(
                "instance:{}>{}",
                root_kind_name(kind),
                self.ty_name(self.ty[c.node as usize])
            ),
        }
    }

    fn make_holder(&mut self, c: &Cand) -> Holder {
        let key = self.holder_key(c);
        let (kind, owner_class, field, context) = match c.src {
            Src::Static { class, name } => (
                HolderKind::Static,
                self.class_names[class as usize].clone(),
                Some(self.strs.get(name)),
                Vec::new(),
            ),
            Src::Syn(i) => {
                let s = &self.syns[i];
                (
                    s.kind,
                    s.owner_class.clone(),
                    s.field.clone(),
                    s.context.clone(),
                )
            }
            Src::Other(_) => (
                HolderKind::Instance,
                self.ty_name(self.ty[c.node as usize]),
                None,
                Vec::new(),
            ),
        };
        Holder {
            key,
            kind,
            owner_class,
            field,
            retained: c.retained,
            dominated_count: u64::from(self.cnt[c.node as usize]),
            path: Vec::new(),
            accumulation: None,
            context,
        }
    }

    /// The root entry, then the heavy path of the dominator tree below `node`, and the
    /// point where it fans out.
    fn walk(&mut self, h: &Holder, node: u32) -> (Vec<String>, Option<Accumulation>) {
        let mut path = Vec::new();
        let is_syn = (node as usize) >= self.g.n;
        match h.kind {
            HolderKind::Static => {
                path.push(format!("root class {}", h.owner_class));
                path.push(format!(
                    "static {}.{} -> {}",
                    h.owner_class,
                    h.field.as_deref().unwrap_or("?"),
                    self.node_class(node)
                ));
            }
            HolderKind::Thread => path.push(format!("root thread {}", h.owner_class)),
            HolderKind::JniGlobal => path.push(format!("root jni-global {}", h.owner_class)),
            HolderKind::Instance => {
                path.push(format!("root {}", h.key));
                path.push(self.node_class(node));
            }
        }
        let mut cur = node;
        loop {
            let kids = self.kidc[cur as usize];
            if kids == 0 || kids >= FAN_OUT || path.len() >= MAX_PATH {
                break;
            }
            let hv = self.heavy[cur as usize];
            if hv == NONE || self.ret[hv as usize] * 2 < self.ret[cur as usize] {
                break;
            }
            let label = self.edge_label(cur, hv);
            path.push(format!("{label} -> {}", self.node_class(hv)));
            cur = hv;
        }
        let acc = (!(is_syn && cur == node)).then(|| Accumulation {
            class: self.node_class(cur),
            children: u64::from(self.kidc[cur as usize]),
        });
        (path, acc)
    }
}
