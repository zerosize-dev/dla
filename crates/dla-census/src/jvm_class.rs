//! JVM census from class files: static fields, singletons, instance collection
//! fields, `<clinit>` registrations and call sites, mapped back to source lines with
//! the `LineNumberTable` and `SourceFile` attribute. Kotlin coroutine continuation
//! classes are read through their `DebugMetadata` annotation: each spilled `L$n` field
//! becomes a `coroutine-capture` holder named after the source variable, at the line of
//! its suspension point.

use std::{borrow::Cow, collections::HashSet, fs, path::Path};

use dla_core::finding::{Holder, Region};
use regex::Regex;

use crate::{
    classfile::{
        ACC_ENUM, ACC_NATIVE, ACC_STATIC, ACC_SYNTHETIC, ClassFile, Method, Op, descriptor_type,
        parse, simple_name,
    },
    lang::{
        java::{FieldSpec, call_table, classify_field},
        package_of,
    },
    model::{ProjectInfo, Raw, Scope, SourceFile},
    vocab,
    walk::{is_test_path, list_files},
};

/// Class files found under the project, parsed. Unreadable files are counted.
pub struct Loaded {
    /// `(project-relative class path, parsed class)`.
    pub classes: Vec<(String, ClassFile)>,
    /// Class files that could not be parsed.
    pub unreadable: usize,
}

/// Reads every `*.class` under `root` (build output directories included), except
/// test class directories.
pub fn load(root: &Path) -> Loaded {
    let rels = list_files(root, true, &|r| {
        r.ends_with(".class")
            && !r.contains("test-classes")
            && !r.contains("/test/")
            && !r.starts_with("test/")
    });
    let mut classes = Vec::new();
    let mut unreadable = 0;
    for rel in rels {
        match fs::read(root.join(&rel)).map(|b| parse(&b)) {
            Ok(Ok(c)) => classes.push((rel, c)),
            _ => unreadable += 1,
        }
    }
    Loaded {
        classes,
        unreadable,
    }
}

/// Finds the source file of a class among the project's sources.
fn source_of<'a>(c: &ClassFile, sources: &'a [SourceFile]) -> Option<&'a SourceFile> {
    let sf = c.source_file.as_deref()?;
    let pkg_path = c.name.rsplit_once('/').map_or("", |(p, _)| p);
    let want = if pkg_path.is_empty() {
        sf.to_owned()
    } else {
        format!("{pkg_path}/{sf}")
    };
    let named: Vec<&SourceFile> = sources
        .iter()
        .filter(|s| s.rel == sf || s.rel.ends_with(&format!("/{sf}")))
        .collect();
    named
        .iter()
        .find(|s| s.rel == want || s.rel.ends_with(&format!("/{want}")))
        .or_else(|| {
            named
                .iter()
                .find(|s| package_of(&s.text).is_some_and(|p| p == pkg_path.replace('/', ".")))
        })
        .or_else(|| (named.len() == 1).then(|| &named[0]))
        .copied()
}

fn decl_line(text: &str, name: &str) -> Option<u32> {
    let re = Regex::new(&format!(r"\b{}\b\s*(?:=|;|,|:|\()", regex::escape(name))).ok()?;
    text.lines()
        .position(|l| re.is_match(l))
        .and_then(|i| u32::try_from(i + 1).ok())
}

/// The line declaring field `name` (a type, then the name, then `=`, `;` or `,`).
fn field_line(text: &str, name: &str) -> Option<u32> {
    let re = Regex::new(&format!(
        r"^\s*(?:(?:private|public|protected|static|final|volatile|transient)\s+)*[\w<>\[\],.? ]+\s+{}\s*[=;,)]",
        regex::escape(name)
    ))
    .ok()?;
    text.lines()
        .position(|l| re.is_match(l) && !l.trim_start().starts_with("return"))
        .and_then(|i| u32::try_from(i + 1).ok())
}

fn assigned_on_line(text: &str, line: u32) -> Option<String> {
    let l = text
        .lines()
        .nth(usize::try_from(line).ok()?.checked_sub(1)?)?;
    let re = Regex::new(r"([A-Za-z_]\w*)\s*=[^=]").ok()?;
    re.captures(l).map(|c| c[1].to_owned())
}

fn dotted(internal: &str) -> String {
    internal.replace('/', ".")
}

/// Marks that start the generated part of a proxy class name. This list repeats the one
/// in `dla_parsers::jvm::deproxy` (this crate does not depend on dla-parsers); keep the
/// two in step.
const PROXY_MARKERS: [&str; 12] = [
    "$$SpringCGLIB$$",
    "$$EnhancerBySpringCGLIB$$",
    "$$FastClassBySpringCGLIB$$",
    "$$EnhancerByCGLIB$$",
    "$$FastClassByCGLIB$$",
    "$$EnhancerByMockitoWithCGLIB$$",
    "$HibernateProxy$",
    "$ByteBuddy$",
    "$MockitoMock$",
    "_$$_jvst",
    "_$$_javassist",
    "$$Lambda",
];

/// A class name (dotted or internal form) with the generated proxy or lambda suffix
/// removed: `a.B$$SpringCGLIB$$0` gives `a.B`. Other names are returned as they are.
pub fn deproxy_name(name: &str) -> Cow<'_, str> {
    let cut = PROXY_MARKERS
        .iter()
        .filter_map(|m| name.find(m))
        .filter(|&p| p > 0)
        .min();
    match cut {
        Some(p) => Cow::Owned(name[..p].to_owned()),
        None => Cow::Borrowed(name),
    }
}

/// Internal name of the annotation the Kotlin compiler puts on coroutine continuations.
pub const DEBUG_METADATA: &str = "kotlin/coroutines/jvm/internal/DebugMetadata";

/// One variable a coroutine continuation keeps in a spilled field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    /// The continuation class, dotted (`shop.LeakyKt$handle$1`).
    pub class: String,
    /// The spilled field (`L$0`).
    pub field: String,
    /// The source variable (`session`).
    pub variable: String,
    /// The suspend function (`handle`) and the class that declares it (`shop.LeakyKt`).
    pub method: String,
    /// Dotted class of the suspend function.
    pub declaring_class: String,
    /// Source file name as the compiler recorded it (`Leaky.kt`).
    pub file: String,
    /// Source line of the suspension point the variable is live at, when known.
    pub line: Option<u32>,
}

/// The captures listed by a continuation class's `DebugMetadata`: `s` names the spilled
/// fields, `n` the variables, `i` maps each to an index into `l`, the suspension point
/// lines. Empty for any other class.
pub fn coroutine_captures(c: &ClassFile) -> Vec<Capture> {
    let Some(a) = c.annotation(DEBUG_METADATA) else {
        return Vec::new();
    };
    let spilled = a.strs("s");
    let names = a.strs("n");
    let points = a.ints("i");
    let lines = a.ints("l");
    let file = a
        .str("f")
        .filter(|f| !f.is_empty())
        .map(str::to_owned)
        .or_else(|| c.source_file.clone())
        .unwrap_or_default();
    let class = dotted(&c.name);
    let mut out = Vec::new();
    for (k, field) in spilled.iter().enumerate() {
        let Some(variable) = names.get(k).filter(|n| !n.is_empty()) else {
            continue;
        };
        if field.is_empty() {
            continue;
        }
        let line = points
            .get(k)
            .and_then(|&ix| usize::try_from(ix).ok())
            .and_then(|ix| lines.get(ix))
            .and_then(|&l| u32::try_from(l).ok())
            .filter(|&l| l > 0);
        out.push(Capture {
            class: class.clone(),
            field: field.clone(),
            variable: variable.clone(),
            method: a.str("m").unwrap_or_default().to_owned(),
            declaring_class: a.str("c").unwrap_or_default().to_owned(),
            file: file.clone(),
            line,
        });
    }
    out
}

/// The result of reading class files: holders and the source files they cover.
pub struct FromClasses {
    /// Holders found.
    pub raws: Vec<Raw>,
    /// Source files (project-relative) a class file covers; tree-sitter skips them.
    pub covered: HashSet<String>,
    /// Number of classes read.
    pub classes: usize,
    /// Class files that could not be parsed.
    pub unreadable: usize,
}

/// Turns the project's class files into holders.
pub fn holders(root: &Path, sources: &[SourceFile]) -> FromClasses {
    let loaded = load(root);
    let mut raws = Vec::new();
    let mut covered = HashSet::new();
    let mut info = ProjectInfo::default();
    for (_, c) in &loaded.classes {
        let own = format!("L{};", c.name);
        if c.fields.iter().any(|f| f.descriptor == own) {
            info.selfref.insert(simple_name(&c.name).to_owned());
        }
    }
    let mut count = 0;
    for (path, c) in &loaded.classes {
        let src = source_of(c, sources);
        if src.is_some_and(|s| s.is_test) {
            continue;
        }
        if c.name.ends_with("module-info") || c.name.ends_with("package-info") {
            continue;
        }
        count += 1;
        let (file, text) = match src {
            Some(s) => {
                covered.insert(s.rel.clone());
                (s.rel.clone(), s.text.as_str())
            }
            None => {
                if is_test_path(path) {
                    continue;
                }
                (path.clone(), "")
            }
        };
        class_holders(c, &file, text, &info, &mut raws);
    }
    FromClasses {
        raws,
        covered,
        classes: count,
        unreadable: loaded.unreadable,
    }
}

fn class_holders(c: &ClassFile, file: &str, text: &str, info: &ProjectInfo, out: &mut Vec<Raw>) {
    // Owners name the user class, not a generated proxy of it.
    let class_dotted = deproxy_name(&dotted(&c.name)).into_owned();
    for cap in coroutine_captures(c) {
        let line = cap.line.unwrap_or(1);
        out.push(
            Raw::new(
                "coroutine-capture",
                Holder::ClosureContinuation,
                Region::Heap,
                file,
                line,
                format!("{}.{}", cap.class, cap.field),
            )
            .tracking(&cap.variable, Scope::Lines(line, line), false),
        );
    }
    let own_desc = format!("L{};", c.name);
    let singleton = c.access & ACC_ENUM != 0
        || c.fields
            .iter()
            .any(|f| f.access & ACC_STATIC != 0 && f.descriptor == own_desc);
    let clinit = c.methods.iter().find(|m| m.name == "<clinit>");
    let init = c.methods.iter().find(|m| m.name == "<init>");
    let first_line = c
        .methods
        .iter()
        .flat_map(|m| m.lines.iter().map(|(_, l)| u32::from(*l)))
        .min()
        .unwrap_or(1);
    // A Kotlin `object` keeps its properties as static fields of the class; `INSTANCE`
    // only points at the object, so it is a holder only when no property is.
    let kotlin_object = c.source_file.as_deref().is_some_and(|f| f.ends_with(".kt"));
    let mut instance: Option<Raw> = None;
    let mut properties = 0_usize;
    for f in &c.fields {
        if f.access & ACC_SYNTHETIC != 0 || f.access & ACC_ENUM != 0 {
            continue;
        }
        if matches!(
            f.name.as_str(),
            "$VALUES" | "$assertionsDisabled" | "serialVersionUID"
        ) || f.name.starts_with("$jacoco")
        {
            continue;
        }
        let is_static = f.access & ACC_STATIC != 0;
        let full = descriptor_type(&f.descriptor);
        let ty = full.rsplit('.').next().unwrap_or(&full).replace('$', ".");
        if is_static && f.descriptor == own_desc {
            let line = put_line(clinit, &c.name, &f.name)
                .or_else(|| decl_line(text, &f.name))
                .unwrap_or(first_line);
            let raw = Raw::new(
                "singleton-instance",
                Holder::StaticGlobal,
                Region::Heap,
                file,
                line,
                format!("{class_dotted}.{}", f.name),
            )
            .tracking(&f.name, Scope::Project, false);
            if kotlin_object && f.name == "INSTANCE" {
                instance = Some(raw);
            } else {
                out.push(raw);
            }
            continue;
        }
        if is_static && f.name == "Companion" {
            let line = put_line(clinit, &c.name, &f.name)
                .or_else(|| decl_line(text, "companion"))
                .unwrap_or(first_line);
            out.push(Raw::new(
                "companion",
                Holder::StaticGlobal,
                Region::Heap,
                file,
                line,
                format!("{class_dotted}.Companion"),
            ));
            continue;
        }
        let spec = FieldSpec {
            ty: &ty,
            name: &f.name,
            is_static,
            class: simple_name(&c.name),
            init: "",
        };
        let Some((kind, holder, region)) = classify_field(&spec, info) else {
            continue;
        };
        let kind = if singleton && !is_static && holder == Holder::ContainerField {
            "singleton-field"
        } else {
            kind
        };
        let line = if is_static {
            put_line(clinit, &c.name, &f.name).or_else(|| field_line(text, &f.name))
        } else {
            field_line(text, &f.name).or_else(|| put_line(init, &c.name, &f.name))
        }
        .or_else(|| decl_line(text, &f.name))
        .unwrap_or(first_line);
        if is_static {
            properties += 1;
        }
        let private = f.access & crate::classfile::ACC_PRIVATE != 0;
        let scope = if private && !is_static {
            Scope::File
        } else {
            Scope::Project
        };
        out.push(
            Raw::new(
                kind,
                holder,
                region,
                file,
                line,
                format!("{class_dotted}.{}", f.name),
            )
            .tracking(&f.name, scope, !is_static && !private),
        );
    }
    if let Some(raw) = instance
        && properties == 0
    {
        out.push(raw);
    }
    for m in &c.methods {
        if m.access & ACC_NATIVE != 0 {
            let line = decl_line(text, &m.name).unwrap_or(first_line);
            out.push(Raw::new(
                "native-method",
                Holder::NativeHandleGlobalRef,
                Region::OffHeap,
                file,
                line,
                format!("{class_dotted}.{}", m.name),
            ));
        }
        method_calls(m, file, text, &class_dotted, out);
    }
}

fn put_line(m: Option<&Method>, class: &str, field: &str) -> Option<u32> {
    let m = m?;
    m.insns
        .iter()
        .find(|i| matches!(&i.op, Op::PutStatic(o, n) | Op::PutField(o, n) if o == class && n == field))
        .and_then(|i| m.line_at(i.pc))
}

fn method_calls(m: &Method, file: &str, text: &str, class_dotted: &str, out: &mut Vec<Raw>) {
    let table = call_table();
    let in_clinit = m.name == "<clinit>";
    let owner = format!("{class_dotted}.{}", m.name);
    for i in &m.insns {
        let callee = match &i.op {
            Op::New(cl) => format!("new {}", dotted(cl)),
            Op::Invoke(o, n, _, _) => format!("{}.{n}", dotted(o)),
            _ => continue,
        };
        let line = m.line_at(i.pc).unwrap_or(1);
        if let Some(rule) = table.iter().find(|r| r.re.is_match(&callee)) {
            let mut raw = Raw::new(
                rule.kind,
                rule.holder,
                rule.region,
                file,
                line,
                owner.clone(),
            );
            if let Some(n) = assigned_on_line(text, line).filter(|_| rule.assigned) {
                raw = raw.tracking(&n, Scope::File, false);
            }
            out.push(raw);
            continue;
        }
        if in_clinit && let Op::Invoke(o, n, _, true) = &i.op {
            let verb = n.starts_with("add")
                || n.starts_with("put")
                || n.starts_with("register")
                || n.starts_with("offer");
            let simple = simple_name(o);
            if verb
                && (vocab::jvm_container().is_match(simple)
                    || vocab::registration_name().is_match(simple)
                    || vocab::registration_name().is_match(n))
            {
                out.push(Raw::new(
                    "static-init-registration",
                    Holder::RegistrationTable,
                    Region::Heap,
                    file,
                    line,
                    owner.clone(),
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classfile::testutil::sample;

    #[test]
    fn static_field_gets_the_clinit_line_and_the_source_file() {
        let t = tempfile::tempdir().unwrap();
        let dir = t.path().join("target/classes/a/b");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("Foo.class"),
            sample("a/b/Foo", "CACHE", "Ljava/util/Map;"),
        )
        .unwrap();
        let src = SourceFile {
            rel: "src/a/b/Foo.java".into(),
            text: "package a.b;\nclass Foo {\n}\n".into(),
            lang: crate::parse::Lang::Java,
            is_test: false,
        };
        let r = holders(t.path(), std::slice::from_ref(&src));
        assert_eq!(r.classes, 1);
        assert!(r.covered.contains("src/a/b/Foo.java"));
        let got: Vec<(String, u32, String, String)> = r
            .raws
            .iter()
            .map(|h| {
                (
                    h.file.clone(),
                    h.line,
                    h.holder.as_str().to_owned(),
                    h.kind.clone(),
                )
            })
            .collect();
        assert_eq!(
            got[0],
            (
                "src/a/b/Foo.java".into(),
                7,
                "static/global".into(),
                "static-container".into()
            )
        );
        // The class also does `new Thread` in its static initializer.
        assert_eq!(got[1].3, "thread");
    }

    fn kotlin_object(props: &[(&str, &str)]) -> Vec<Raw> {
        use crate::classfile::Field;
        let st = ACC_STATIC;
        let mut fields = vec![Field {
            access: st,
            name: "INSTANCE".into(),
            descriptor: "Lshop/Audit;".into(),
        }];
        fields.extend(props.iter().map(|(n, d)| Field {
            access: st,
            name: (*n).into(),
            descriptor: (*d).into(),
        }));
        let c = ClassFile {
            name: "shop/Audit".into(),
            source_file: Some("Audit.kt".into()),
            fields,
            ..ClassFile::default()
        };
        let text = "package shop\nobject Audit {\n  val trail = ArrayList<Entry>()\n}\n";
        let mut out = Vec::new();
        class_holders(&c, "Audit.kt", text, &ProjectInfo::default(), &mut out);
        out
    }

    #[test]
    fn a_kotlin_object_reports_its_growing_property_not_instance() {
        let out = kotlin_object(&[("trail", "Ljava/util/ArrayList;")]);
        let owners: Vec<&str> = out.iter().map(|r| r.owner.as_str()).collect();
        assert_eq!(owners, ["shop.Audit.trail"], "{owners:?}");
        assert_eq!(out[0].line, 3);
    }

    #[test]
    fn a_kotlin_object_without_properties_keeps_instance() {
        let out = kotlin_object(&[]);
        let owners: Vec<&str> = out.iter().map(|r| r.owner.as_str()).collect();
        assert_eq!(owners, ["shop.Audit.INSTANCE"]);
    }
}
