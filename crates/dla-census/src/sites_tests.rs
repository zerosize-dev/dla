//! What the syntax-tree pass records for each holder and each writer and remover site, over
//! small synthetic projects.

use dla_core::{
    census::{Bound, CensusEntry, ContainerClass, Lifetime, OwnerCreation, Receiver, SiteKind},
    runtime::Runtime,
};

use super::{census, tests::project};

/// The entries the census of `rt` wrote for the project `files`.
fn entries(files: &[(&str, &str)], rt: Runtime) -> Vec<CensusEntry> {
    let (_d, ctx) = project(files);
    census(&ctx, rt).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(
        &std::fs::read(ctx.run_dir.join(format!("census/{}.json", rt.as_str()))).unwrap(),
    )
    .unwrap();
    serde_json::from_value(raw["entries"].clone()).unwrap()
}

fn entry<'e>(es: &'e [CensusEntry], owner_end: &str) -> &'e CensusEntry {
    es.iter()
        .find(|e| e.owner.ends_with(owner_end))
        .unwrap_or_else(|| panic!("no entry {owner_end}: {:?}", es.iter().map(|e| &e.owner).collect::<Vec<_>>()))
}

const GO_SERVER: &str = "package health\n\ntype Server struct {\n\tstatusMap map[string]int\n\tupdates   map[string]map[int]chan int\n}\n\nfunc NewServer() *Server {\n\treturn &Server{statusMap: map[string]int{}}\n}\n\nfunc (s *Server) Set(service string, st int) {\n\ts.statusMap[service] = st\n\ts.updates[service][st] = nil\n}\n\nfunc (s *Server) Watch(service string, c chan int) {\n\ts.updates[service] = make(map[int]chan int)\n\ts.updates[service][1] = c\n}\n\nfunc (s *Server) Done(service string, st int) {\n\tdelete(s.updates[service], st)\n}\n";

#[test]
fn go_element_writes_and_deletes_carry_their_depth() {
    let es = entries(&[("health/s.go", GO_SERVER)], Runtime::Go);
    let u = entry(&es, "Server.updates");
    let by = |line: &str| {
        u.facts
            .writer_sites
            .iter()
            .chain(&u.facts.remover_sites)
            .find(|s| s.site.ends_with(line))
            .unwrap_or_else(|| panic!("{line}: {u:?}"))
    };
    // `s.updates[service][st] = nil` writes an element of an element: depth 1.
    assert_eq!((by(":14").kind, by(":14").depth), (SiteKind::Insert, 1));
    // `s.updates[service] = make(..)` stores a value under a key of the holder: an insert at
    // depth 0, and not a reset of the holder.
    assert_eq!((by(":18").kind, by(":18").depth), (SiteKind::Insert, 0));
    assert!(!u.removers.iter().any(|r| r.ends_with(":18")), "{u:?}");
    // `delete(s.updates[service], st)` deletes from an element: depth 1.
    assert_eq!((by(":23").kind, by(":23").depth), (SiteKind::Delete, 1));
    assert_eq!(u.removers, ["health/s.go:23"]);
    assert_eq!(u.facts.container, Some(ContainerClass::Container));
    assert_eq!(u.facts.lifetime, Some(Lifetime::Object));
}

#[test]
fn go_a_site_knows_its_function_its_initialiser_and_its_parameters() {
    let src = "package reg\n\nvar registry = map[string]int{}\nvar byName = map[string]int{}\n\nfunc init() {\n\tregistry[\"a\"] = 1\n}\n\nfunc Register(name string, v int) {\n\tbyName[name] = v\n}\n\nfunc helper() {\n\tregistry[\"b\"] = 2\n}\n\nfunc start() {\n\thelper()\n}\n\nfunc init2() {\n\tstart()\n}\n\nfunc init() {\n\tinit2()\n}\n";
    let es = entries(&[("reg/r.go", src)], Runtime::Go);
    let r = entry(&es, "registry");
    let w = &r.facts.writer_sites;
    let a = w.iter().find(|s| s.site.ends_with(":7")).unwrap();
    assert!(a.init && a.startup && a.constant && !a.param, "{a:?}");
    assert_eq!(a.function.as_deref(), Some("init"));
    // `helper` is called only from `start`, which only `init2` calls: start-up only, three
    // calls up, though none of them is an initialiser itself.
    let b = w.iter().find(|s| s.site.ends_with(":15")).unwrap();
    assert!(!b.init && b.startup, "{b:?}");
    let n = entry(&es, "byName");
    let s = &n.facts.writer_sites[0];
    assert_eq!(s.function.as_deref(), Some("Register"));
    assert!(s.param && s.exported && !s.init && !s.startup && !s.constant, "{s:?}");
    assert_eq!(n.facts.lifetime, Some(Lifetime::Process));
}

#[test]
fn go_an_unexported_holder_is_not_written_from_another_package_by_name() {
    let es = entries(
        &[
            ("a/a.go", "package a\n\nvar items = map[string]int{}\n\nfunc Add(k string) {\n\titems[k] = 1\n}\n"),
            ("b/b.go", "package b\n\nvar items = map[string]int{}\n\nfunc Add(k string) {\n\titems[k] = 2\n}\n"),
        ],
        Runtime::Go,
    );
    let a = entry(&es, "a.a.items");
    assert_eq!(a.writers, ["a/a.go:6"]);
    assert_eq!(a.facts.private, Some(true));
    let b = entry(&es, "b.b.items");
    assert_eq!(b.writers, ["b/b.go:6"]);
}

#[test]
fn go_another_struct_s_field_of_the_same_name_is_not_a_writer() {
    let src = "package p\n\ntype Hub struct {\n\tsubs []int\n}\n\ntype Other struct {\n\tsubs []int\n}\n\nfunc (h *Hub) Add(x int) {\n\th.subs = append(h.subs, x)\n}\n\nfunc (o *Other) Add(x int) {\n\to.subs = append(o.subs, x)\n}\n";
    let es = entries(&[("p/p.go", src)], Runtime::Go);
    let hub = entry(&es, "Hub.subs");
    assert_eq!(hub.writers, ["p/p.go:12"], "{hub:?}");
}

/// The sites the census of `rt` left out as holders that are not containers.
fn not_listed(files: &[(&str, &str)], rt: Runtime) -> Vec<String> {
    let (_d, ctx) = project(files);
    census(&ctx, rt).unwrap();
    let raw: serde_json::Value = serde_json::from_slice(
        &std::fs::read(ctx.run_dir.join(format!("census/{}.json", rt.as_str()))).unwrap(),
    )
    .unwrap();
    raw["not_listed"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["site"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn go_a_pointer_that_is_only_assigned_as_a_whole_is_not_a_container_and_not_listed() {
    let src = "package p\n\ntype Config struct{ n int }\n\nvar current *Config\nvar mu sync.Mutex\nvar items = map[string]int{}\n\nfunc Set(c *Config) {\n\tcurrent = c\n}\n\nfunc Add(k string) {\n\titems[k] = 1\n}\n";
    let files = [("p/p.go", src)];
    let es = entries(&files, Runtime::Go);
    // The pointer is only reassigned and the mutex is no container: neither is a holder.
    assert!(es.iter().all(|e| !e.owner.ends_with("p.current") && !e.owner.ends_with("p.mu")), "{es:?}");
    assert_eq!(not_listed(&files, Runtime::Go), ["p/p.go:5", "p/p.go:6"]);
    // A real container stays.
    assert!(entry(&es, "p.items").facts.container == Some(ContainerClass::Container));
    // A field wrapped around its own value is a chain that grows: it stays, with its writer.
    let chain = "package p\n\ntype Poller struct {\n\tctx context.Context\n}\n\nfunc (p *Poller) Lookup(s string) {\n\tp.ctx = trace.Start(p.ctx, s)\n}\n";
    let es = entries(&[("p/c.go", chain)], Runtime::Go);
    let e = entry(&es, "Poller.ctx");
    assert_eq!(e.writers, ["p/c.go:8"], "{e:?}");
    assert_eq!(e.facts.writer_sites[0].kind, SiteKind::Insert);
    assert_eq!(e.facts.container, Some(ContainerClass::NotContainer));
}

#[test]
fn go_bounds_pools_literal_tables_and_owner_creation() {
    let src = "package p\n\nvar bufs = sync.Pool{}\nvar table = map[string]int{\"a\": 1, \"b\": 2}\nvar ring = make(chan int, 16)\nvar free = make(chan int)\n\ntype Hub struct {\n\tsubs []int\n}\n\nfunc New() *Hub { return &Hub{} }\n\nfunc main() {\n\th := New()\n\t_ = h\n}\n";
    let es = entries(&[("p/p.go", src)], Runtime::Go);
    assert!(entry(&es, "p.bufs").facts.pool);
    assert!(entry(&es, "p.table").facts.literal);
    assert_eq!(entry(&es, "p.ring").facts.bound, Some(Bound::Bounded));
    assert_eq!(entry(&es, "p.free").facts.bound, Some(Bound::None));
    let per_call = "package p\n\ntype Hub struct {\n\tsubs []int\n}\n\nfunc Handle() {\n\th := &Hub{}\n\t_ = h\n}\n";
    let es2 = entries(&[("p/q.go", per_call)], Runtime::Go);
    let h = es2.iter().find(|e| e.owner.ends_with("Hub.subs"));
    // `Hub` is not long-lived here, so it is not listed at all.
    assert!(h.is_none() || h.unwrap().facts.owner_creation == Some(OwnerCreation::PerCall));
}

const JAVA_A: &str = "package a;\n\nimport java.util.*;\n\npublic class A {\n  private final Map<String, String> resolved = new HashMap<>();\n  private static final Map<String, String> CACHE = new WeakHashMap<>();\n  private final List<String> items;\n\n  public A() {\n    this.items = new ArrayList<>();\n    items.add(\"first\");\n  }\n\n  public void put(String k, String v) {\n    resolved.put(k, v);\n    CACHE.put(k, v);\n    items.add(k);\n  }\n\n  void evict(String k) {\n    resolved.remove(k);\n    items.clear();\n  }\n}\n";
const JAVA_B: &str = "package a;\n\nimport java.util.*;\n\nclass B {\n  private final Map<String, String> resolved = new HashMap<>();\n\n  void put(String k, String v) {\n    resolved.put(k, v);\n  }\n}\n";

#[test]
fn java_private_holders_ignore_a_same_named_variable_in_another_file() {
    let es = entries(&[("src/a/A.java", JAVA_A), ("src/a/B.java", JAVA_B)], Runtime::Jvm);
    let a = entry(&es, "A.resolved");
    assert_eq!(a.writers, ["src/a/A.java:16"], "{a:?}");
    assert_eq!(a.facts.private, Some(true));
    assert_eq!(a.removers, ["src/a/A.java:22"]);
    let b = entry(&es, "B.resolved");
    assert_eq!(b.writers, ["src/a/B.java:9"]);
}

#[test]
fn java_a_constructor_assignment_is_initialisation_not_removal() {
    let es = entries(&[("src/a/A.java", JAVA_A)], Runtime::Jvm);
    let items = entry(&es, "A.items");
    // `this.items = new ArrayList<>()` in the constructor is not a remover; `items.clear()`
    // in `evict` is the only one.
    assert_eq!(items.removers, ["src/a/A.java:23"], "{items:?}");
    let ctor = items
        .facts
        .writer_sites
        .iter()
        .find(|s| s.site.ends_with(":12"))
        .unwrap();
    assert!(ctor.init && ctor.startup, "{ctor:?}");
    let put = items
        .facts
        .writer_sites
        .iter()
        .find(|s| s.site.ends_with(":18"))
        .unwrap();
    assert!(!put.init && put.param && put.exported, "{put:?}");
    assert_eq!(put.function.as_deref(), Some("put"));
}

#[test]
fn java_weak_and_bounded_types_and_owner_creation() {
    let es = entries(&[("src/a/A.java", JAVA_A)], Runtime::Jvm);
    assert_eq!(entry(&es, "A.CACHE").facts.bound, Some(Bound::WeakKeys));
    assert_eq!(entry(&es, "A.CACHE").facts.lifetime, Some(Lifetime::Process));
    assert_eq!(entry(&es, "A.resolved").facts.bound, Some(Bound::None));
    let lru = "package a;\n\nimport java.util.*;\n\nclass L {\n  static final Map<String, String> M = new LinkedHashMap<>(16, 0.75f, true) {\n    protected boolean removeEldestEntry(Map.Entry e) { return size() > 10; }\n  };\n}\n";
    let es = entries(&[("src/a/L.java", lru)], Runtime::Jvm);
    assert_eq!(entry(&es, "L.M").facts.bound, Some(Bound::Bounded));
    let owner = "package a;\n\nimport java.util.*;\n\nclass Reg {\n  private final List<String> subs = new ArrayList<>();\n  void add(String s) { subs.add(s); }\n}\n\nclass Use {\n  static void each(String s) {\n    new Reg().add(s);\n  }\n}\n";
    let es = entries(&[("src/a/Reg.java", owner)], Runtime::Jvm);
    let r = es.iter().find(|e| e.owner.ends_with("Reg.subs"));
    if let Some(r) = r {
        assert_eq!(r.facts.owner_creation, Some(OwnerCreation::PerCall), "{r:?}");
    }
}

const PY: &str = "import functools\nfrom weakref import WeakValueDictionary\n\n_CACHE = {}\n_LOG = None\ntable = {\"a\": 1, \"b\": 2}\nREFS = WeakValueDictionary()\n\n\ndef remember(key, value):\n    _CACHE[key] = value\n\n\ndef forget(key):\n    del _CACHE[key]\n\n\n@functools.lru_cache(maxsize=None)\ndef parse(x):\n    return x\n\n\n@functools.lru_cache(maxsize=64)\ndef small(x):\n    return x\n\n\n@functools.cache\ndef big(x):\n    return x\n\n\nclass Hub:\n    def __init__(self):\n        self.subs = []\n        self.subs.append(1)\n\n    def add(self, fn):\n        self.subs.append(fn)\n";

#[test]
fn python_module_container_sites_and_bounds() {
    let es = entries(&[("app/m.py", PY)], Runtime::Python);
    let c = entry(&es, "m._CACHE");
    assert_eq!(c.facts.container, Some(ContainerClass::Container));
    assert_eq!(c.facts.lifetime, Some(Lifetime::Process));
    assert_eq!(c.facts.private, Some(true));
    let w = &c.facts.writer_sites[0];
    assert_eq!((w.kind, w.depth, w.function.as_deref()), (SiteKind::Insert, 0, Some("remember")));
    assert!(w.param && w.exported && !w.init, "{w:?}");
    // `del _CACHE[key]` deletes a key of the holder itself.
    let d = &c.facts.remover_sites[0];
    assert_eq!((d.kind, d.depth), (SiteKind::Delete, 0), "{d:?}");
    assert_eq!(entry(&es, "m.REFS").facts.bound, Some(Bound::WeakValues));
    assert!(entry(&es, "m.table").facts.literal);
    assert_eq!(entry(&es, "m.parse").facts.bound, Some(Bound::UnboundedCache));
    assert_eq!(entry(&es, "m.small").facts.bound, Some(Bound::Bounded));
    assert_eq!(entry(&es, "m.big").facts.bound, Some(Bound::UnboundedCache));
    // A cache decorator is a container whose key is the call's arguments.
    assert_eq!(entry(&es, "m.parse").facts.container, Some(ContainerClass::Container));
}

#[test]
fn python_init_writers_and_a_private_name_in_another_module() {
    let es = entries(
        &[
            ("app/m.py", PY),
            ("app/n.py", "_CACHE = {}\n\n\ndef other(k):\n    _CACHE[k] = 1\n"),
        ],
        Runtime::Python,
    );
    let subs = entry(&es, "Hub.subs");
    let init = subs.facts.writer_sites.iter().find(|s| s.site.ends_with(":36")).unwrap();
    assert!(init.init && init.startup, "{init:?}");
    let add = subs.facts.writer_sites.iter().find(|s| s.site.ends_with(":39")).unwrap();
    assert!(!add.init && add.param, "{add:?}");
    assert_eq!(subs.facts.lifetime, Some(Lifetime::Object));
    let m = entry(&es, "m._CACHE");
    assert!(m.writers.iter().all(|w| w.starts_with("app/m.py")), "{m:?}");
    let n = entry(&es, "n._CACHE");
    assert_eq!(n.writers, ["app/n.py:5"], "{n:?}");
}

#[test]
fn python_a_name_in_a_string_or_a_comment_is_not_a_site() {
    let src = "_L = []\n\n\ndef f(x):\n    # _L.append(x)\n    return \"_L.append(x)\"\n";
    let es = entries(&[("a.py", src)], Runtime::Python);
    let e = entry(&es, "a._L");
    assert!(e.writers.is_empty() && e.removers.is_empty(), "{e:?}");
}

const JS: &str = "'use strict';\nconst cache = new Map();\nconst _hidden = new Map();\nexport const open = new Set();\n\nfunction put(key, value) {\n  cache.set(key, value);\n  _hidden.set(key, 1);\n  open.add('x');\n}\n\nclass Hub {\n  constructor() {\n    this.subs = [];\n    this.subs.push(1);\n  }\n  add(fn) {\n    this.subs.push(fn);\n  }\n}\n\ncache.set('boot', 1);\nmodule.exports = { put };\n";

#[test]
fn javascript_sites_params_constants_and_module_level() {
    let es = entries(&[("src/a.js", JS), ("src/b.js", "const cache = new Map();\ncache.set('x', 2);\n")], Runtime::Node);
    let c = entries_by(&es, "a.cache");
    let put = c.facts.writer_sites.iter().find(|s| s.site.ends_with(":7")).unwrap();
    assert!(put.param && !put.init && put.exported, "{put:?}");
    let top = c.facts.writer_sites.iter().find(|s| s.site.ends_with(":22")).unwrap();
    assert!(top.init && top.startup && top.constant, "{top:?}");
    assert_eq!(c.facts.private, Some(true));
    assert!(c.writers.iter().all(|w| w.starts_with("src/a.js")), "{c:?}");
    assert_eq!(entries_by(&es, "a.open").facts.private, Some(false));
    let subs = entries_by(&es, "Hub.subs");
    let add = subs.facts.writer_sites.iter().find(|s| s.site.ends_with(":18")).unwrap();
    assert!(add.param && !add.init, "{add:?}");
    let ctor = subs.facts.writer_sites.iter().find(|s| s.site.ends_with(":15")).unwrap();
    assert!(ctor.init, "{ctor:?}");
}

fn entries_by<'e>(es: &'e [CensusEntry], owner_end: &str) -> &'e CensusEntry {
    entry(es, owner_end)
}

#[test]
fn registrations_say_where_their_receiver_comes_from() {
    let src = "import signal\n\n\ndef install(bus):\n    bus.subscribe(handler)\n\n\ndef scoped():\n    local = Bus()\n    local.subscribe(handler)\n";
    let es = entries(&[("a/r.py", src)], Runtime::Python);
    let by = |line: &str| {
        es.iter()
            .find(|e| e.site.ends_with(line) && e.kind == "registration")
            .unwrap_or_else(|| panic!("{line}: {es:?}"))
    };
    assert_eq!(by(":5").facts.receiver, Some(Receiver::Outer));
    assert_eq!(by(":10").facts.receiver, Some(Receiver::Local));
}
