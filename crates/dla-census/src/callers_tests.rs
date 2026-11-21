use super::*;

fn file(rel: &str, text: &str) -> SourceFile {
    SourceFile {
        rel: rel.to_owned(),
        text: text.to_owned(),
        lang: Lang::from_path(std::path::Path::new(rel)).unwrap(),
        is_test: false,
    }
}

fn names(h: &Hop) -> Vec<String> {
    h.callers.iter().map(|c| c.func.name.clone()).collect()
}

/// The three-hop chain every language test builds: `store` is called by `handle`, which is
/// called by `route`, which is called by `serve`.
fn chain_of(files: &[SourceFile], start: &str, at: &str, line: usize) -> Vec<Hop> {
    let ix = Index::build(files);
    let f = ix.function_at(at, line).unwrap();
    assert_eq!(f.name, start);
    ix.chain(files, &f)
}

#[test]
fn python_chain_has_three_hops_public_first() {
    let src = "\
_cache = {}


def store(key, value):
    _cache[key] = value


def _helper(k):
    store(k, 1)


class Server:
    def _inner(self, name):
        return store(name, 2)

    def handle(self, path):
        return self._inner(path)


def route(request):
    return Server().handle(request.path)


def serve(req):
    return route(req)
";
    let hops = chain_of(&[file("app/web.py", src)], "store", "app/web.py", 5);
    assert_eq!(hops.len(), 3, "{hops:?}");
    // The two callers of store: a private function and a private method; the order is by
    // file and line when neither is public.
    assert_eq!(names(&hops[0]), ["_helper", "Server._inner"]);
    assert!(hops[0].callers.iter().all(|c| !c.func.public));
    assert_eq!(names(&hops[1]), ["Server.handle"]);
    assert!(hops[1].callers[0].func.public);
    assert_eq!(names(&hops[2]), ["route"]);
}

#[test]
fn public_callers_come_first_and_a_hop_holds_five() {
    let mut src = String::from("def sink(x):\n    pass\n\n\n");
    // Seven callers: the underscore ones are defined first, the public ones last.
    for i in 0..4 {
        src.push_str(&format!("def _p{i}():\n    sink({i})\n\n\n"));
    }
    for i in 0..3 {
        src.push_str(&format!("def api{i}():\n    sink({i})\n\n\n"));
    }
    let hops = chain_of(&[file("m.py", &src)], "sink", "m.py", 2);
    let h = &hops[0];
    assert_eq!(h.callers.len(), MAX_PER_HOP);
    assert_eq!(h.more, 2);
    assert_eq!(names(h)[..3], ["api0", "api1", "api2"], "{:?}", names(h));
    assert!(h.callers[..3].iter().all(|c| c.func.public));
    assert!(h.callers[3..].iter().all(|c| !c.func.public));
}

#[test]
fn a_function_is_listed_once_and_recursion_ends_the_chain() {
    let src = "\
def a():
    b()


def b():
    a()
    b()
";
    let hops = chain_of(&[file("m.py", src)], "a", "m.py", 2);
    assert_eq!(names(&hops[0]), ["b"]);
    assert_eq!(hops.len(), 1, "{hops:?}");
}

#[test]
fn calls_match_by_name_alone() {
    // `other.put` is a different object, and it is still matched.
    let src = "\
class Store:
    def put(self, k):
        pass


def one(s):
    s.put(1)


def two(other):
    other.put(2)
";
    let files = [file("m.py", src)];
    let ix = Index::build(&files);
    let put = ix.function_at("m.py", 3).unwrap();
    assert_eq!(put.name, "Store.put");
    let hop = &ix.chain(&files, &put)[0];
    assert_eq!(names(hop), ["one", "two"]);
}

#[test]
fn a_python_constructor_is_called_by_the_class_name() {
    let src = "\
class Reg:
    def __init__(self, name):
        self.name = name


def make(n):
    return Reg(n)
";
    let files = [file("m.py", src)];
    let ix = Index::build(&files);
    let init = ix.function_at("m.py", 3).unwrap();
    assert_eq!(init.name, "Reg.__init__");
    assert_eq!(init.call_name(), "Reg");
    assert_eq!(init.params, ["name"]);
    assert_eq!(names(&ix.chain(&files, &init)[0]), ["make"]);
}

#[test]
fn a_decorator_line_belongs_to_the_decorated_function() {
    let src = "\
import functools


@functools.lru_cache(None)
def build(handler, apps):
    return handler
";
    let ix = Index::build(&[file("m.py", src)]);
    let f = ix.function_at("m.py", 4).unwrap();
    assert_eq!(f.name, "build");
    assert_eq!(f.params, ["handler", "apps"]);
}

#[test]
fn test_files_are_not_callers() {
    let mut t = file("tests/test_m.py", "def test_it():\n    sink(1)\n");
    t.is_test = true;
    let m = file("m.py", "def sink(x):\n    pass\n");
    let files = [m, t];
    let ix = Index::build(&files);
    let f = ix.function_at("m.py", 2).unwrap();
    assert!(ix.chain(&files, &f).is_empty());
}

#[test]
fn go_chain_has_three_hops() {
    let src = "\
package web

var cache = map[string]int{}

func store(key string, v int) {
	cache[key] = v
}

func (s *Server) handle(path string) {
	store(path, 1)
}

func (s *Server) Route(p string) {
	s.handle(p)
}

func Serve(p string) {
	var s Server
	s.Route(p)
}
";
    let hops = chain_of(&[file("web/web.go", src)], "store", "web/web.go", 6);
    assert_eq!(hops.len(), 3, "{hops:?}");
    assert_eq!(names(&hops[0]), ["Server.handle"]);
    assert!(!hops[0].callers[0].func.public);
    assert_eq!(names(&hops[1]), ["Server.Route"]);
    assert!(hops[1].callers[0].func.public);
    assert_eq!(names(&hops[2]), ["Serve"]);
}

#[test]
fn java_chain_has_three_hops() {
    let src = "\
package shop;

public class Registry {
    private static final java.util.Map<String, Object> CACHE = new java.util.HashMap<>();

    static void store(String key, Object v) {
        CACHE.put(key, v);
    }

    void handle(String path) {
        store(path, null);
    }

    public void route(String p) {
        handle(p);
    }
}

class Front {
    public static void serve(String p) {
        new Registry().route(p);
    }
}
";
    let hops = chain_of(
        &[file("src/Registry.java", src)],
        "Registry.store",
        "src/Registry.java",
        7,
    );
    assert_eq!(hops.len(), 3, "{hops:?}");
    assert_eq!(names(&hops[0]), ["Registry.handle"]);
    assert!(!hops[0].callers[0].func.public);
    assert_eq!(names(&hops[1]), ["Registry.route"]);
    assert!(hops[1].callers[0].func.public);
    assert_eq!(names(&hops[2]), ["Front.serve"]);
}

#[test]
fn javascript_chain_has_three_hops() {
    let src = "\
const cache = new Map();

function store(key, v) {
  cache.set(key, v);
}

export class Server {
  handle(path) {
    store(path, 1);
  }
  _route(p) {
    this.handle(p);
  }
}

export function serve(p) {
  new Server()._route(p);
}
";
    let hops = chain_of(&[file("src/web.js", src)], "store", "src/web.js", 4);
    assert_eq!(hops.len(), 3, "{hops:?}");
    assert_eq!(names(&hops[0]), ["Server.handle"]);
    assert!(hops[0].callers[0].func.public);
    assert_eq!(names(&hops[1]), ["Server._route"]);
    assert!(!hops[1].callers[0].func.public);
    assert_eq!(names(&hops[2]), ["serve"]);
    assert!(hops[2].callers[0].func.public);
}

#[test]
fn typescript_arrow_functions_take_the_name_they_are_assigned_to() {
    let src = "\
export const store = (key: string, v: number) => {
  cache.set(key, v);
};

export const handle = (path: string) => store(path, 1);
";
    let files = [file("src/web.ts", src)];
    let ix = Index::build(&files);
    let store = ix.function_at("src/web.ts", 2).unwrap();
    assert_eq!(store.name, "store");
    assert_eq!(store.params, ["key", "v"]);
    let hop = &ix.chain(&files, &store)[0];
    assert_eq!(names(hop), ["handle"]);
    assert!(hop.callers[0].func.public);
}

#[test]
fn the_key_comes_from_the_parameters_the_writing_statement_uses() {
    let src = "\
_cache = {}


def store(request, key, note, path=None):
    _cache[key] = request.path
    return note


def handle(req):
    return store(req, req.path + '/x', note='n')


def other(r):
    return store(r, 'fixed', 'z')
";
    let files = [file("m.py", src)];
    let ix = Index::build(&files);
    let store = ix.function_at("m.py", 5).unwrap();
    let p = ix.provenance(&files, &store, Some(5), false);
    // `path` is an attribute name, not the parameter of another name; `request` is used.
    assert_eq!(p.params, ["request", "key"]);
    assert_eq!(p.sites.len(), 2);
    assert_eq!(p.sites[0].caller, "handle");
    assert_eq!(p.sites[0].at, "m.py:10");
    assert_eq!(
        p.sites[0].passed,
        [
            ("request".to_owned(), "req".to_owned()),
            ("key".to_owned(), "req.path + '/x'".to_owned())
        ]
    );
    assert_eq!(p.sites[1].passed[1].1, "'fixed'");
}

#[test]
fn a_keyword_argument_is_passed_for_its_own_parameter() {
    let src = "\
def put(a, b):
    return {a: b}


def f(x):
    return put(b=x, a=1)
";
    let files = [file("m.py", src)];
    let ix = Index::build(&files);
    let put = ix.function_at("m.py", 2).unwrap();
    let p = ix.provenance(&files, &put, None, true);
    assert_eq!(
        p.sites[0].passed,
        [
            ("a".to_owned(), "1".to_owned()),
            ("b".to_owned(), "x".to_owned())
        ]
    );
}

#[test]
fn a_key_that_is_no_parameter_has_none() {
    let src = "\
import itertools

_n = itertools.count()
_cache = {}


def make():
    _cache[next(_n)] = 1


def go():
    make()
";
    let files = [file("m.py", src)];
    let ix = Index::build(&files);
    let make = ix.function_at("m.py", 8).unwrap();
    let p = ix.provenance(&files, &make, Some(8), false);
    assert!(p.params.is_empty());
    assert_eq!(p.sites.len(), 1);
}

#[test]
fn other_languages_have_no_chain() {
    assert!(!supported(Lang::Rust));
    assert!(!supported(Lang::Kotlin));
    assert!(!supported(Lang::C));
    assert!(supported(Lang::Python) && supported(Lang::Go) && supported(Lang::Java));
    assert!(supported(Lang::JavaScript) && supported(Lang::TypeScript));
}

#[test]
fn a_passed_local_name_shows_the_assignments_that_set_it() {
    let src = "\
import functools


@functools.lru_cache(None)
def wrap(handler):
    return handler


class Router:
    def dispatch(self, path):
        h = self.routes.get(path)
        if h is None:
            h = Handler(404)
        later = 1
        wrap(h)
        h = Other()
        return h == 2
";
    let files = [file("app/r.py", src)];
    let ix = Index::build(&files);
    let wrap = ix.function_at("app/r.py", 4).unwrap();
    let p = ix.provenance(&files, &wrap, None, true);
    assert_eq!(p.sites.len(), 1);
    assert_eq!(p.sites[0].passed, [("handler".to_owned(), "h".to_owned())]);
    // The assignment after the call and the comparison are not origins.
    assert_eq!(
        p.sites[0].origins,
        [(
            "h".to_owned(),
            vec![
                "self.routes.get(path)".to_owned(),
                "Handler(404)".to_owned()
            ]
        )]
    );
}

#[test]
fn a_function_nothing_calls_is_followed_by_where_it_is_passed() {
    let src = "\
class App:
    async def _handle(self, request):
        return request

    def make_handler(self, **kw):
        return Server(self._handle, **kw)

    def _private_user(self):
        # self._handle is only mentioned in this comment
        return self._handle


class Logger:
    def __init__(self, log, fmt):
        pass


def run_app(logger_class=Logger):
    return logger_class(1, 2)
";
    let files = [file("app/a.py", src)];
    let ix = Index::build(&files);
    let handle = ix.function_at("app/a.py", 3).unwrap();
    assert_eq!(handle.name, "App._handle");
    let hops = ix.chain(&files, &handle);
    // Nothing calls it by name: it is mentioned in two methods, the public one first; the
    // definition and the comment are not mentions.
    assert_eq!(names(&hops[0]), ["App.make_handler", "App._private_user"]);
    assert!(hops[0].callers.iter().all(|c| c.by_reference));
    assert!(hops[0].callers[0].func.public);
    // A class stored as a default parameter is mentioned there (and where it is built by a
    // variable of another name, nothing says so).
    let init = ix.function_at("app/a.py", 14).unwrap();
    let hops = ix.chain(&files, &init);
    assert_eq!(names(&hops[0]), ["run_app"]);
    assert!(hops[0].callers[0].by_reference);
    // A function that is called is not followed by its mentions.
    let src = "def fetch():\n    pass\n\n\ndef g():\n    fetch()\n\n\nTABLE = [fetch]\n";
    let files = [file("m.py", src)];
    let ix = Index::build(&files);
    let f = ix.function_at("m.py", 2).unwrap();
    let hop = &ix.chain(&files, &f)[0];
    assert_eq!(names(hop), ["g"]);
    assert!(!hop.callers[0].by_reference);
}

#[test]
fn a_call_under_an_if_says_what_must_hold_for_it_to_happen() {
    let src = "\
def sink(h):
    return h


class App:
    def handle(self, request):
        h = request.handler
        if self._run_middlewares:
            if not self._legacy:
                h = sink(h)
            else:
                h = sink(h)
        return sink(request)
";
    let files = [file("a.py", src)];
    let ix = Index::build(&files);
    let sink = ix.function_at("a.py", 2).unwrap();
    let p = ix.provenance(&files, &sink, None, true);
    assert_eq!(p.sites.len(), 3);
    // Innermost condition first; the else branch is not under the condition of its `if`; a call
    // outside every `if` has none.
    assert_eq!(
        p.sites[0].guards,
        ["not self._legacy", "self._run_middlewares"]
    );
    assert_eq!(p.sites[1].guards, ["self._run_middlewares"]);
    assert!(p.sites[2].guards.is_empty(), "{:?}", p.sites[2].guards);
    // Go and JavaScript read their `if` too.
    let go = [file(
        "w.go",
        "package w\n\nfunc sink(h int) {}\n\nfunc handle(h int) {\n\tif enabled(h) {\n\t\tsink(h)\n\t}\n}\n",
    )];
    let ix = Index::build(&go);
    let sink = ix.function_at("w.go", 3).unwrap();
    assert_eq!(
        ix.provenance(&go, &sink, None, true).sites[0].guards,
        ["enabled(h)"]
    );
    let js = [file(
        "w.js",
        "function sink(h) {}\nfunction handle(h) {\n  if (h.ok) {\n    sink(h);\n  }\n}\n",
    )];
    let ix = Index::build(&js);
    let sink = ix.function_at("w.js", 1).unwrap();
    assert_eq!(
        ix.provenance(&js, &sink, None, true).sites[0].guards,
        ["h.ok"]
    );
}
