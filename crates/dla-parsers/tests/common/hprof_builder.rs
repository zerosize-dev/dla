//! A tiny HPROF writer for tests: enough of the format to build small heaps with known
//! shapes (classes, instances, arrays, roots, threads, references) deterministically.

#![allow(dead_code, missing_docs)]

use std::collections::HashMap;

pub const T_OBJ: u8 = 2;
pub const T_BOOL: u8 = 4;
pub const T_BYTE: u8 = 8;
pub const T_INT: u8 = 10;
pub const T_LONG: u8 = 11;

/// A field value of an instance's data.
pub enum F {
    Id(u64),
    I4(i32),
    I8(i64),
    B(u8),
}

pub struct B {
    pub id_size: usize,
    ts: u64,
    next: u64,
    serial: u32,
    strings: HashMap<String, u64>,
    pre: Vec<u8>,
    heap: Vec<Vec<u8>>,
    post: Vec<u8>,
}

fn put_u4(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_be_bytes());
}

impl B {
    pub fn new(id_size: usize, ts_ms: u64) -> Self {
        Self {
            id_size,
            ts: ts_ms,
            next: 0x1_0000,
            serial: 0,
            strings: HashMap::new(),
            pre: Vec::new(),
            heap: Vec::new(),
            post: Vec::new(),
        }
    }

    fn put_id(&self, v: &mut Vec<u8>, id: u64) {
        if self.id_size == 8 {
            v.extend_from_slice(&id.to_be_bytes());
        } else {
            put_u4(v, id as u32);
        }
    }

    pub fn new_id(&mut self) -> u64 {
        self.next += 8;
        self.next
    }

    fn record(out: &mut Vec<u8>, tag: u8, body: &[u8]) {
        out.push(tag);
        put_u4(out, 0);
        put_u4(out, body.len() as u32);
        out.extend_from_slice(body);
    }

    pub fn string(&mut self, s: &str) -> u64 {
        if let Some(&id) = self.strings.get(s) {
            return id;
        }
        let id = self.new_id();
        let mut body = Vec::new();
        self.put_id(&mut body, id);
        body.extend_from_slice(s.as_bytes());
        Self::record(&mut self.pre, 0x01, &body);
        self.strings.insert(s.to_owned(), id);
        id
    }

    /// Allocates a class id and writes its LOAD_CLASS record (`name` uses dots).
    pub fn declare(&mut self, name: &str) -> u64 {
        let id = self.new_id();
        let name_id = self.string(&name.replace('.', "/"));
        self.serial += 1;
        let mut body = Vec::new();
        put_u4(&mut body, self.serial);
        self.put_id(&mut body, id);
        put_u4(&mut body, 0);
        self.put_id(&mut body, name_id);
        Self::record(&mut self.pre, 0x02, &body);
        id
    }

    pub fn serial_of_last_class(&self) -> u32 {
        self.serial
    }

    /// Writes a CLASS_DUMP: object statics `statics`, plus one int constant, one long
    /// static and the instance `fields` (name, type).
    pub fn class_dump(
        &mut self,
        id: u64,
        super_id: u64,
        loader: u64,
        inst_size: u32,
        fields: &[(&str, u8)],
        statics: &[(&str, u64)],
    ) {
        let mut b = vec![0x20u8];
        self.put_id(&mut b, id);
        put_u4(&mut b, 0);
        self.put_id(&mut b, super_id);
        self.put_id(&mut b, loader);
        for _ in 0..4 {
            self.put_id(&mut b, 0);
        }
        put_u4(&mut b, inst_size);
        b.extend_from_slice(&1u16.to_be_bytes()); // one constant pool entry
        b.extend_from_slice(&3u16.to_be_bytes());
        b.push(T_INT);
        put_u4(&mut b, 7);
        let n = statics.len() as u16 + 1;
        b.extend_from_slice(&n.to_be_bytes());
        let vname = self.string("VERSION");
        self.put_id(&mut b, vname);
        b.push(T_LONG);
        b.extend_from_slice(&42u64.to_be_bytes());
        for (name, value) in statics {
            let nid = self.string(name);
            self.put_id(&mut b, nid);
            b.push(T_OBJ);
            self.put_id(&mut b, *value);
        }
        b.extend_from_slice(&(fields.len() as u16).to_be_bytes());
        for (name, t) in fields {
            let nid = self.string(name);
            self.put_id(&mut b, nid);
            b.push(*t);
        }
        self.heap.push(b);
    }

    pub fn data(&self, parts: &[F]) -> Vec<u8> {
        let mut d = Vec::new();
        for p in parts {
            match p {
                F::Id(v) => self.put_id(&mut d, *v),
                F::I4(v) => d.extend_from_slice(&v.to_be_bytes()),
                F::I8(v) => d.extend_from_slice(&v.to_be_bytes()),
                F::B(v) => d.push(*v),
            }
        }
        d
    }

    pub fn instance(&mut self, class_id: u64, data: &[F]) -> u64 {
        let id = self.new_id();
        let d = self.data(data);
        let mut b = vec![0x21u8];
        self.put_id(&mut b, id);
        put_u4(&mut b, 0);
        self.put_id(&mut b, class_id);
        put_u4(&mut b, d.len() as u32);
        b.extend_from_slice(&d);
        self.heap.push(b);
        id
    }

    pub fn obj_array(&mut self, array_class: u64, elems: &[u64]) -> u64 {
        let id = self.new_id();
        let mut b = vec![0x22u8];
        self.put_id(&mut b, id);
        put_u4(&mut b, 0);
        put_u4(&mut b, elems.len() as u32);
        self.put_id(&mut b, array_class);
        for e in elems {
            self.put_id(&mut b, *e);
        }
        self.heap.push(b);
        id
    }

    pub fn byte_array(&mut self, bytes: &[u8]) -> u64 {
        let id = self.new_id();
        let mut b = vec![0x23u8];
        self.put_id(&mut b, id);
        put_u4(&mut b, 0);
        put_u4(&mut b, bytes.len() as u32);
        b.push(T_BYTE);
        b.extend_from_slice(bytes);
        self.heap.push(b);
        id
    }

    /// A `java.lang.String` instance holding `text` as a Latin-1 byte array.
    pub fn java_string(&mut self, string_class: u64, text: &str) -> u64 {
        let arr = self.byte_array(text.as_bytes());
        self.instance(string_class, &[F::Id(arr), F::B(0), F::I4(0)])
    }

    /// GC root records; `a` and `b` are the thread serial and frame or trace number.
    pub fn root(&mut self, kind: u8, id: u64, a: u32, b2: u32) {
        let mut b = vec![kind];
        self.put_id(&mut b, id);
        match kind {
            0x01 => self.put_id(&mut b, 0xdead),
            0x02 | 0x03 | 0x08 | 0x8e => {
                put_u4(&mut b, a);
                put_u4(&mut b, b2);
            }
            0x04 | 0x06 => put_u4(&mut b, a),
            _ => {}
        }
        self.heap.push(b);
    }

    /// A HEAP_DUMP_INFO sub-record and an Android-style primitive array without data.
    pub fn exotic_subrecords(&mut self) {
        let mut b = vec![0xfeu8];
        put_u4(&mut b, 1);
        let n = self.string("app");
        self.put_id(&mut b, n);
        self.heap.push(b);
        let id = self.new_id();
        let mut c = vec![0xc3u8];
        self.put_id(&mut c, id);
        put_u4(&mut c, 0);
        put_u4(&mut c, 5);
        c.push(T_INT);
        self.heap.push(c);
    }

    pub fn frame(&mut self, id: u64, method: &str, file: &str, class_serial: u32, line: i32) {
        let m = self.string(method);
        let s = self.string("()V");
        let f = self.string(file);
        let mut b = Vec::new();
        self.put_id(&mut b, id);
        self.put_id(&mut b, m);
        self.put_id(&mut b, s);
        self.put_id(&mut b, f);
        put_u4(&mut b, class_serial);
        put_u4(&mut b, line as u32);
        Self::record(&mut self.pre, 0x04, &b);
    }

    pub fn trace(&mut self, serial: u32, thread: u32, frames: &[u64]) {
        let mut b = Vec::new();
        put_u4(&mut b, serial);
        put_u4(&mut b, thread);
        put_u4(&mut b, frames.len() as u32);
        for f in frames {
            self.put_id(&mut b, *f);
        }
        Self::record(&mut self.pre, 0x05, &b);
    }

    /// A top-level record of any tag, to check that unknown ones are skipped.
    pub fn raw_record(&mut self, tag: u8, body: &[u8]) {
        Self::record(&mut self.post, tag, body);
    }

    /// The file: header, string and class records, the heap split into `segments`
    /// HEAP_DUMP_SEGMENT records, an unknown record, and HEAP_DUMP_END.
    pub fn finish(self, segments: usize) -> Vec<u8> {
        let mut out = b"JAVA PROFILE 1.0.2\0".to_vec();
        put_u4(&mut out, self.id_size as u32);
        out.extend_from_slice(&self.ts.to_be_bytes());
        out.extend_from_slice(&self.pre);
        let per = self.heap.len().div_ceil(segments.max(1)).max(1);
        for chunk in self.heap.chunks(per) {
            let body: Vec<u8> = chunk.iter().flatten().copied().collect();
            Self::record(&mut out, 0x1c, &body);
        }
        out.extend_from_slice(&self.post);
        Self::record(&mut out, 0x2c, &[]);
        out
    }
}

/// Sizes the model heap assumes (compressed references, 8-byte alignment).
pub fn align8(x: u64) -> u64 {
    (x + 7) & !7
}

/// What [`model`] builds, with the numbers a correct reader must find.
pub struct Model {
    pub dump: Vec<u8>,
    /// Retained bytes of `app.Holder.ITEMS`'s list.
    pub items_retained: u64,
    /// Objects dominated by it.
    pub items_count: u64,
    /// Retained bytes of the thread `worker-1` (its object, name and local list).
    pub thread_retained: u64,
    /// Retained bytes of the JNI global `app.Blob` group.
    pub jni_retained: u64,
    /// Retained bytes of the VM-internal `app.Hub` root.
    pub hub_retained: u64,
    /// Retained bytes of the static `ClassLoader.scl` loader.
    pub loader_retained: u64,
    /// Shallow bytes of the weak reference object.
    pub weak_shallow: u64,
}

/// A small application heap: `app.Holder.ITEMS` holds `n_items` items, a weak reference
/// guards a large array, a class loader reaches `app.Holder`, thread `worker-1` keeps a
/// local list of `n_local` items, a JNI global and a VM-internal root hold arrays.
/// Sizes follow the reader's assumptions: compressed references; object header 12 (8 for
/// 4-byte identifiers); array header 16 (12).
pub fn model(id_size: usize, ts_ms: u64, n_items: usize, n_local: usize) -> Model {
    let mut b = B::new(id_size, ts_ms);
    let (hdr, arr_hdr, refsz) = if id_size == 8 {
        (12u64, 16u64, 4u64)
    } else {
        (8, 12, 4)
    };
    let sh = |fields: u64| align8(hdr + fields);
    let obj = b.declare("java.lang.Object");
    let class_cls = b.declare("java.lang.Class");
    let string = b.declare("java.lang.String");
    let thread = b.declare("java.lang.Thread");
    let reference = b.declare("java.lang.ref.Reference");
    let weak = b.declare("java.lang.ref.WeakReference");
    let loader = b.declare("java.lang.ClassLoader");
    let app_loader = b.declare("jdk.internal.loader.ClassLoaders$AppClassLoader");
    let list = b.declare("java.util.ArrayList");
    let obj_arr = b.declare("[Ljava/lang/Object;");
    let byte_arr = b.declare("[B");
    let item = b.declare("app.Item");
    let holder = b.declare("app.Holder");
    let blob = b.declare("app.Blob");
    let hub = b.declare("app.Hub");

    let item_size = sh(refsz + 4);
    let list_size = sh(refsz + 4);
    // Items and the list.
    let make_list = |b: &mut B, n: usize, tag: u8| -> (u64, u64, u64) {
        let mut ids = Vec::new();
        let mut bytes = 0u64;
        for i in 0..n {
            let payload = b.byte_array(&vec![tag; 40 + (i % 3)]);
            bytes += align8(arr_hdr + 40 + (i % 3) as u64);
            let it = b.instance(item, &[F::Id(payload), F::I4(i as i32)]);
            bytes += item_size;
            ids.push(it);
        }
        let arr = b.obj_array(obj_arr, &ids);
        bytes += align8(arr_hdr + refsz * n as u64);
        let l = b.instance(list, &[F::Id(arr), F::I4(n as i32)]);
        bytes += list_size;
        (l, bytes, 2 + 2 * n as u64)
    };
    let (items, items_retained, items_count) = make_list(&mut b, n_items, 1);
    let (local, local_retained, _) = make_list(&mut b, n_local, 2);

    // A weakly held large array.
    let big = b.byte_array(&vec![9u8; 100_000]);
    let wref = b.instance(weak, &[F::Id(big), F::Id(0), F::Id(0)]);
    let weak_shallow = sh(3 * refsz);

    // The loader reaches the class object of app.Holder through its class list.
    let classes_arr = b.obj_array(obj_arr, &[holder]);
    let classes_list = b.instance(list, &[F::Id(classes_arr), F::I4(1)]);
    let loader_obj = b.instance(app_loader, &[F::Id(classes_list)]);
    let loader_retained = sh(refsz) + list_size + align8(arr_hdr + refsz);

    // Thread worker-1.
    let tname = b.java_string(string, "worker-1");
    let tobj = b.instance(thread, &[F::Id(tname), F::I8(11)]);
    let thread_retained = local_retained + sh(refsz + 8) + sh(refsz + 1 + 4) + align8(arr_hdr + 8);
    b.root(0x08, tobj, 1, 5);
    b.root(0x03, local, 1, 0);
    b.frame(0x50, "run", "Worker.java", b.serial_of_last_class(), 42);
    b.frame(0x51, "loop", "Worker.java", b.serial_of_last_class(), 17);
    b.trace(5, 1, &[0x50, 0x51]);

    // JNI global and VM-internal roots.
    let jarr = b.byte_array(&[3u8; 800]);
    let jblob = b.instance(blob, &[F::Id(jarr)]);
    b.root(0x01, jblob, 0, 0);
    let jni_retained = sh(refsz) + align8(arr_hdr + 800);
    let harr = b.byte_array(&[4u8; 300]);
    let hubobj = b.instance(hub, &[F::Id(harr)]);
    b.root(0x8d, hubobj, 0, 0);
    let hub_retained = sh(refsz) + align8(arr_hdr + 300);
    // Other root kinds that carry only an object.
    let istr = b.java_string(string, "interned");
    b.root(0x89, istr, 0, 0);
    let lonely = b.instance(blob, &[F::Id(0)]);
    b.root(0x90, lonely, 0, 0); // unreachable: not a root
    let mon = b.instance(blob, &[F::Id(0)]);
    b.root(0x8e, mon, 1, 0);
    b.exotic_subrecords();

    // Class dumps (after the instances: the reader must not depend on order). The
    // instance sizes are nonsense on purpose: the reader must not use them.
    b.class_dump(obj, 0, 0, 999, &[], &[]);
    b.class_dump(class_cls, obj, 0, 999, &[], &[]);
    b.class_dump(
        string,
        obj,
        0,
        999,
        &[("value", T_OBJ), ("coder", T_BYTE), ("hash", T_INT)],
        &[],
    );
    b.class_dump(
        thread,
        obj,
        0,
        999,
        &[("name", T_OBJ), ("tid", T_LONG)],
        &[],
    );
    b.class_dump(
        reference,
        obj,
        0,
        999,
        &[("referent", T_OBJ), ("queue", T_OBJ), ("next", T_OBJ)],
        &[],
    );
    b.class_dump(weak, reference, 0, 999, &[], &[]);
    b.class_dump(loader, obj, 0, 999, &[], &[("scl", loader_obj)]);
    b.class_dump(app_loader, loader, 0, 999, &[("classes", T_OBJ)], &[]);
    b.class_dump(
        list,
        obj,
        0,
        999,
        &[("elementData", T_OBJ), ("size", T_INT)],
        &[],
    );
    b.class_dump(obj_arr, obj, 0, 999, &[], &[]);
    b.class_dump(byte_arr, obj, 0, 999, &[], &[]);
    b.class_dump(item, obj, 0, 999, &[("payload", T_OBJ), ("n", T_INT)], &[]);
    b.class_dump(
        holder,
        obj,
        loader_obj,
        999,
        &[],
        &[("ITEMS", items), ("WATCH", wref)],
    );
    b.class_dump(blob, obj, 0, 999, &[("data", T_OBJ)], &[]);
    b.class_dump(hub, obj, 0, 999, &[("data", T_OBJ)], &[]);
    b.raw_record(0x99, &[1, 2, 3, 4, 5]);
    Model {
        dump: b.finish(3),
        items_retained,
        items_count,
        thread_retained,
        jni_retained,
        hub_retained,
        loader_retained,
        weak_shallow,
    }
}
