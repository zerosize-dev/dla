//! C and C++ rules: a `malloc`-family or `fopen`-family result that is neither released,
//! returned nor stored on some return path of the function that acquired it.

use std::sync::LazyLock;

use dla_census::parse::tree_sitter::Node;
use dla_core::static_hit::StaticHit;
use regex::Regex;

use super::ast::{
    CallSite, ancestor, calls, find_all, last_token, line, named_children, tokens, txt,
};
use super::{RuleInfo, SrcFile, hit};

const MALLOC: &str = "native-malloc-leak-path";
const FOPEN: &str = "native-fopen-no-fclose";

/// Rule table (mirrors the `backend = "dla"` rows of `data/detectors/native.toml`).
pub const RULES: &[RuleInfo] = &[
    RuleInfo {
        rule: MALLOC,
        archetype: "L-OWNERSHIP",
        prior: 0.5,
    },
    RuleInfo {
        rule: FOPEN,
        archetype: "L-HANDLE",
        prior: 0.55,
    },
];

static OWNING_CALL: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)free|release|destroy|close|dispose|delete|clean|add|push|insert|append|store|set_|_set|attach|register|enqueue|queue|take|own|put|assign|adopt|save|list|hash|cache|link|wrap|handoff|give|transfer|send|submit|emplace|stash|keep|retain|reset|fini|shutdown|finish|done").expect("regex")
});

struct Spec {
    rule: &'static str,
    what: &'static str,
    acquire: &'static [&'static str],
    release: &'static [&'static str],
    non_owning: &'static [&'static str],
}

const SPECS: [Spec; 2] = [
    Spec {
        rule: MALLOC,
        what: "allocation",
        acquire: &[
            "malloc",
            "calloc",
            "strdup",
            "strndup",
            "aligned_alloc",
            "reallocarray",
        ],
        release: &["free"],
        non_owning: &[
            "memcpy",
            "memmove",
            "memset",
            "memcmp",
            "memchr",
            "strcpy",
            "strncpy",
            "strcat",
            "strncat",
            "strlen",
            "strcmp",
            "strncmp",
            "strchr",
            "strrchr",
            "strstr",
            "sprintf",
            "snprintf",
            "printf",
            "fprintf",
            "puts",
            "fputs",
            "fwrite",
            "fread",
            "read",
            "write",
            "send",
            "recv",
            "fgets",
            "sscanf",
            "strtol",
            "strtoul",
            "strtod",
            "atoi",
            "atol",
            "qsort",
            "bsearch",
            "strerror",
            "perror",
            "assert",
            "sizeof",
            "snprintf",
            "vsnprintf",
            "isalpha",
            "isdigit",
            "toupper",
            "tolower",
        ],
    },
    Spec {
        rule: FOPEN,
        what: "stream",
        acquire: &["fopen", "fdopen", "popen", "tmpfile", "freopen"],
        release: &["fclose", "pclose"],
        non_owning: &[
            "fprintf", "fputs", "fputc", "putc", "fgetc", "getc", "fgets", "fread", "fwrite",
            "fscanf", "fseek", "ftell", "rewind", "fflush", "feof", "ferror", "fileno", "setvbuf",
            "fgetpos", "fsetpos", "clearerr", "ungetc", "vfprintf", "fseeko", "ftello", "assert",
        ],
    },
];

/// Strips casts and parentheses.
fn unwrap_value(mut n: Node<'_>) -> Node<'_> {
    while matches!(n.kind(), "cast_expression" | "parenthesized_expression") {
        let kids = named_children(n);
        match kids.last() {
            Some(k) => n = *k,
            None => break,
        }
    }
    n
}

fn callee_name(call: Node<'_>, src: &str) -> String {
    call.child_by_field_name("function")
        .map(|f| last_token(txt(f, src)))
        .unwrap_or_default()
}

/// The variable name of a declarator or assignment target when it is a plain local.
fn target_name(n: Node<'_>, src: &str) -> Option<String> {
    let mut cur = n;
    loop {
        match cur.kind() {
            "identifier" => return Some(txt(cur, src).to_owned()),
            "pointer_declarator" | "reference_declarator" | "parenthesized_declarator" => {
                cur = named_children(cur).into_iter().last()?;
            }
            _ => return None,
        }
    }
}

/// Names declared locally in a function: parameters and every declaration in the body.
fn declared_names(func: Node<'_>, body: Node<'_>, src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut add = |d: Node<'_>| {
        let d = if d.kind() == "init_declarator" {
            d.child_by_field_name("declarator").unwrap_or(d)
        } else {
            d
        };
        if let Some(n) = target_name(d, src) {
            out.push(n);
        }
    };
    for decl in find_all(body, &["declaration", "for_range_loop"]) {
        let mut c = decl.walk();
        for d in decl.children_by_field_name("declarator", &mut c) {
            add(d);
        }
    }
    for p in find_all(func, &["parameter_declaration"]) {
        if let Some(d) = p.child_by_field_name("declarator") {
            add(d);
        }
    }
    out
}

/// True when `func` is a C++ member function (defined in a class body or out of line as `T::f`).
fn is_member_function(func: Node<'_>) -> bool {
    ancestor(func, &["class_specifier", "struct_specifier"]).is_some()
        || func
            .child_by_field_name("declarator")
            .is_some_and(|d| !find_all(d, &["qualified_identifier"]).is_empty())
}

struct Acq<'t> {
    var: String,
    call: Node<'t>,
    stmt_end: usize,
}

fn acquisitions<'t>(func: Node<'t>, body: Node<'t>, src: &str, spec: &Spec) -> Vec<Acq<'t>> {
    let mut out = Vec::new();
    let member_fn = is_member_function(func);
    let locals = declared_names(func, body, src);
    for n in find_all(body, &["init_declarator", "assignment_expression"]) {
        let (target, value) = if n.kind() == "init_declarator" {
            (
                n.child_by_field_name("declarator"),
                n.child_by_field_name("value"),
            )
        } else {
            (
                n.child_by_field_name("left"),
                n.child_by_field_name("right"),
            )
        };
        let (Some(t), Some(v)) = (target, value) else {
            continue;
        };
        let Some(var) = target_name(t, src) else {
            continue;
        };
        let v = unwrap_value(v);
        if v.kind() != "call_expression" || !spec.acquire.contains(&callee_name(v, src).as_str()) {
            continue;
        }
        // Assigning to a name that is not declared in the function stores into a member
        // (or a global); in a member function the owning object is responsible for it.
        if n.kind() == "assignment_expression" && member_fn && !locals.contains(&var) {
            continue;
        }
        out.push(Acq {
            var,
            call: v,
            stmt_end: n.end_byte(),
        });
    }
    out
}

/// Smallest region whose end the node is certain to reach once it runs.
fn region(e: Node<'_>) -> Node<'_> {
    let mut cur = e;
    while let Some(p) = cur.parent() {
        match p.kind() {
            "compound_statement"
            | "case_statement"
            | "translation_unit"
            | "function_definition"
            | "namespace_definition" => return p,
            "if_statement" | "else_clause" | "while_statement" | "for_statement"
            | "do_statement" | "for_range_loop" | "switch_statement" => return cur,
            _ => cur = p,
        }
    }
    cur
}

fn covers(e: Node<'_>, r: Node<'_>) -> bool {
    e.start_byte() < r.start_byte() && {
        let reg = region(e);
        reg.start_byte() <= r.start_byte() && r.end_byte() <= reg.end_byte()
    }
}

fn in_lambda(n: Node<'_>) -> bool {
    ancestor(n, &["lambda_expression"]).is_some()
}

/// True when the condition text (whitespace removed) says `var` is null.
fn says_null(cond: &str, var: &str) -> bool {
    if cond.contains("!=") {
        return false;
    }
    let at_boundary = |pat: &str| {
        cond.match_indices(pat).any(|(i, _)| {
            cond[i + pat.len()..]
                .chars()
                .next()
                .is_none_or(|c| !(c.is_alphanumeric() || c == '_'))
        })
    };
    let assigned = cond.contains(&format!("({var}="));
    at_boundary(&format!("!{var}"))
        || cond.contains(&format!("!({var}="))
        || ["NULL", "0", "nullptr"].iter().any(|z| {
            at_boundary(&format!("{var}=={z}"))
                || at_boundary(&format!("{z}=={var}"))
                || (assigned && cond.ends_with(&format!("=={z})")))
                || (assigned && cond.contains(&format!(")=={z}")))
        })
}

/// True when `r` runs only when `var` is null (an allocation-failure branch).
fn null_branch(r: Node<'_>, src: &str, var: &str) -> bool {
    let mut cur = r;
    while let Some(p) = cur.parent() {
        if p.kind() == "if_statement" {
            let cons = p.child_by_field_name("consequence");
            if cons
                .is_some_and(|c| c.start_byte() <= r.start_byte() && r.end_byte() <= c.end_byte())
            {
                if let Some(cond) = p.child_by_field_name("condition") {
                    let c: String = txt(cond, src)
                        .chars()
                        .filter(|c| !c.is_whitespace())
                        .collect();
                    if says_null(&c, var) {
                        return true;
                    }
                }
            }
        }
        cur = p;
    }
    false
}

fn check_function(f: &SrcFile, func: Node<'_>, spec: &Spec, out: &mut Vec<StaticHit>) {
    let src = f.text.as_str();
    let Some(body) = func.child_by_field_name("body") else {
        return;
    };
    let acqs = acquisitions(func, body, src, spec);
    if acqs.is_empty() {
        return;
    }
    let returns: Vec<Node<'_>> = find_all(body, &["return_statement"])
        .into_iter()
        .filter(|r| !in_lambda(*r))
        .collect();
    let cs: Vec<CallSite<'_>> = calls(f, body);
    let ends_with_return = named_children(body).last().is_some_and(|l| {
        matches!(
            l.kind(),
            "return_statement" | "goto_statement" | "labeled_statement"
        )
    });
    for a in acqs {
        let var = a.var.as_str();
        let mut protectors: Vec<Node<'_>> = Vec::new();
        for c in &cs {
            if c.node.start_byte() < a.stmt_end {
                continue;
            }
            let uses = c.args.iter().any(|arg| {
                let u = unwrap_value(*arg);
                let t = txt(u, src).trim_start_matches('&');
                t == var
            });
            if !uses {
                continue;
            }
            if spec.release.contains(&c.name.as_str())
                || (!spec.non_owning.contains(&c.name.as_str()) && OWNING_CALL.is_match(&c.name))
            {
                protectors.push(c.node);
            }
        }
        for n in find_all(body, &["assignment_expression", "init_declarator"]) {
            if n.start_byte() < a.stmt_end {
                continue;
            }
            let (left, right) = if n.kind() == "init_declarator" {
                (
                    n.child_by_field_name("declarator"),
                    n.child_by_field_name("value"),
                )
            } else {
                (
                    n.child_by_field_name("left"),
                    n.child_by_field_name("right"),
                )
            };
            let (Some(l), Some(r)) = (left, right) else {
                continue;
            };
            let rt = txt(unwrap_value(r), src);
            if rt == var && target_name(l, src).as_deref() != Some(var) {
                protectors.push(n);
            }
        }
        // Struct/array initialisers and other stores that mention the variable by itself.
        for n in find_all(body, &["initializer_list", "compound_literal_expression"]) {
            if n.start_byte() >= a.stmt_end && tokens(txt(n, src)).contains(&var) {
                protectors.push(n);
            }
        }
        let mut leak_line = None;
        for r in &returns {
            if r.start_byte() < a.stmt_end {
                continue;
            }
            let returned = tokens(txt(*r, src)).contains(&var);
            if returned || null_branch(*r, src, var) || protectors.iter().any(|p| covers(*p, *r)) {
                continue;
            }
            leak_line = Some(line(*r));
            break;
        }
        if leak_line.is_none() && !ends_with_return {
            let end_probe = body
                .child(u32::try_from(body.child_count().saturating_sub(1)).unwrap_or(0))
                .unwrap_or(body);
            let protected = protectors
                .iter()
                .any(|p| region(*p).id() == body.id() && p.start_byte() < end_probe.start_byte());
            if !protected {
                leak_line = Some(u32::try_from(body.end_position().row + 1).unwrap_or(u32::MAX));
            }
        }
        if let Some(l) = leak_line {
            out.push(hit(
                spec.rule,
                f,
                line(a.call),
                format!(
                    "{} `{var}` from {}() is not released, returned or stored on the path that leaves the function at line {l}",
                    spec.what,
                    callee_name(a.call, src)
                ),
            ));
        }
    }
}

/// Runs the native rules over the C and C++ files.
pub fn check(files: &[SrcFile]) -> Vec<StaticHit> {
    let mut hits = Vec::new();
    for f in files {
        for func in find_all(f.tree.root_node(), &["function_definition"]) {
            for spec in &SPECS {
                check_function(f, func, spec, &mut hits);
            }
        }
    }
    hits
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dla_rules::{parse_sources, sites};

    fn run(src: &str) -> Vec<StaticHit> {
        check(&parse_sources(&[("src/a.c", src)]))
    }

    #[test]
    fn early_return_after_malloc_leaks() {
        let src = "int f(int n) {\n    char *p = malloc(n);\n    if (!p) return -1;\n    if (fill(p) < 0) return -2;\n    use(p);\n    free(p);\n    return 0;\n}\n";
        let hits = run(src);
        assert_eq!(sites(&hits, MALLOC), ["src/a.c:2"]);
        assert!(hits[0].message.contains("line 4"), "{}", hits[0].message);
    }

    #[test]
    fn freed_on_every_path_is_quiet() {
        let src = "int f(int n) {\n    char *p = malloc(n);\n    if (!p) return -1;\n    if (fill(p) < 0) {\n        free(p);\n        return -2;\n    }\n    free(p);\n    return 0;\n}\n";
        assert!(run(src).is_empty());
        let goto_cleanup = "int f(int n) {\n    int rc = 0;\n    char *p = malloc(n);\n    if (!p) return -1;\n    if (fill(p) < 0) {\n        rc = -2;\n        goto out;\n    }\n    use(p);\nout:\n    free(p);\n    return rc;\n}\n";
        assert!(run(goto_cleanup).is_empty());
    }

    #[test]
    fn returned_stored_or_handed_off_is_quiet() {
        let returned = "char *f(int n) {\n    char *p = malloc(n);\n    if (!p) return NULL;\n    if (fill(p) < 0) return NULL;\n    return p;\n}\n";
        // The failing fill() path leaks.
        assert_eq!(sites(&run(returned), MALLOC), ["src/a.c:2"]);
        let stored = "int f(struct S *s, int n) {\n    char *p = malloc(n);\n    if (!p) return -1;\n    s->buf = p;\n    if (n > 3) return 1;\n    return 0;\n}\n";
        assert!(run(stored).is_empty());
        let handed = "int f(struct S *s, int n) {\n    char *p = malloc(n);\n    if (!p) return -1;\n    list_push(s, p);\n    return 0;\n}\n";
        assert!(run(handed).is_empty());
        let cast = "int f(int n) {\n    char *p;\n    if ((p = (char *)malloc(n)) == NULL) return -1;\n    free(p);\n    return 0;\n}\n";
        assert!(run(cast).is_empty());
    }

    #[test]
    fn conditional_free_does_not_cover_later_returns() {
        let src = "int f(int n, int k) {\n    char *p = malloc(n);\n    if (!p) return -1;\n    if (k) free(p);\n    return 0;\n}\n";
        assert_eq!(sites(&run(src), MALLOC), ["src/a.c:2"]);
    }

    #[test]
    fn void_function_falling_off_the_end_leaks() {
        let src = "void f(int n) {\n    char *p = malloc(n);\n    use(p);\n}\n";
        assert_eq!(sites(&run(src), MALLOC), ["src/a.c:2"]);
        let ok = "void f(int n) {\n    char *p = malloc(n);\n    use(p);\n    free(p);\n}\n";
        assert!(run(ok).is_empty());
    }

    #[test]
    fn fopen_without_fclose() {
        let leak = "int f(const char *path) {\n    FILE *fp = fopen(path, \"r\");\n    if (!fp) return -1;\n    if (parse(fp) < 0) return -2;\n    fclose(fp);\n    return 0;\n}\n";
        assert_eq!(sites(&run(leak), FOPEN), ["src/a.c:2"]);
        let ok = "int f(const char *path) {\n    FILE *fp = fopen(path, \"r\");\n    if (!fp) return -1;\n    char buf[8];\n    fgets(buf, 8, fp);\n    fclose(fp);\n    return 0;\n}\n";
        assert!(run(ok).is_empty());
        let never = "void f(void) {\n    FILE *fp;\n    fp = fopen(\"x\", \"w\");\n    fprintf(fp, \"hi\");\n}\n";
        assert_eq!(sites(&run(never), FOPEN), ["src/a.c:3"]);
    }

    #[test]
    fn member_stores_in_member_functions_are_not_leaks() {
        let run_cpp = |src: &str| check(&parse_sources(&[("src/a.cpp", src)]));
        let inline = "struct W {\n    FILE *f;\n    W(const char *n) { f = fopen(n, \"r\"); }\n    ~W() { fclose(f); }\n};\n";
        assert!(run_cpp(inline).is_empty());
        let outline = "W::W(const char *n) {\n    f = fopen(n, \"r\");\n}\n";
        assert!(run_cpp(outline).is_empty());
        let local = "struct W {\n    void g(const char *n) {\n        FILE *f;\n        f = fopen(n, \"r\");\n        use(f);\n    }\n};\n";
        assert_eq!(sites(&run_cpp(local), FOPEN), ["src/a.cpp:4"]);
        let global = "FILE *g;\nvoid f(const char *n) {\n    g = fopen(n, \"r\");\n}\n";
        assert_eq!(sites(&run_cpp(global), FOPEN), ["src/a.cpp:3"]);
    }

    #[test]
    fn cpp_files_are_scanned_too() {
        let src = "int f(int n) {\n    char *p = (char *)malloc(n);\n    if (n > 2) return 1;\n    free(p);\n    return 0;\n}\n";
        let hits = check(&parse_sources(&[("src/a.cpp", src)]));
        assert_eq!(sites(&hits, MALLOC), ["src/a.cpp:2"]);
    }
}
