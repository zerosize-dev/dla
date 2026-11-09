//! Kotlin `DebugMetadata` and Spring proxy names in the class-file census.
//!
//! `LeakyKt$handle$1.class` is the output of the Kotlin 2.0 compiler for a suspend
//! function that spills two parameters (`session`, `tag`) across one suspension point.
//! `Gateway$$SpringCGLIB$$0.class` is built by javac; it stands in for a Spring proxy
//! class that sits on disk.

use std::{fs, path::PathBuf};

use dla_census::{
    classfile::{self, ElementValue},
    jvm_class::{self, DEBUG_METADATA},
    model::{Scope, SourceFile},
    parse::Lang,
};

fn fixture(name: &str) -> Vec<u8> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/kotlin_metadata")
        .join(name);
    fs::read(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

#[test]
fn debug_metadata_is_read_from_a_real_continuation_class() {
    let c = classfile::parse(&fixture("LeakyKt$handle$1.class")).unwrap();
    assert_eq!(c.name, "shop/LeakyKt$handle$1");
    assert_eq!(c.source_file.as_deref(), Some("Leaky.kt"));
    let a = c.annotation(DEBUG_METADATA).expect("DebugMetadata");
    assert_eq!(a.str("f"), Some("Leaky.kt"));
    assert_eq!(a.str("m"), Some("handle"));
    assert_eq!(a.str("c"), Some("shop.LeakyKt"));
    assert_eq!(a.ints("l"), [16]);
    assert_eq!(a.ints("i"), [0, 0]);
    assert_eq!(a.strs("s"), ["L$0", "L$1"]);
    assert_eq!(a.strs("n"), ["session", "tag"]);
    assert!(matches!(a.get("l"), Some(ElementValue::Array(_))));
    // The other annotation (kotlin.Metadata) is read too and does not disturb the rest.
    assert!(c.annotation("kotlin/Metadata").is_some());
}

#[test]
fn captures_map_spilled_fields_to_variables_and_lines() {
    let c = classfile::parse(&fixture("LeakyKt$handle$1.class")).unwrap();
    let caps = jvm_class::coroutine_captures(&c);
    assert_eq!(caps.len(), 2);
    assert_eq!(caps[0].class, "shop.LeakyKt$handle$1");
    assert_eq!(caps[0].field, "L$0");
    assert_eq!(caps[0].variable, "session");
    assert_eq!(caps[0].line, Some(16));
    assert_eq!(caps[0].file, "Leaky.kt");
    assert_eq!(caps[0].method, "handle");
    assert_eq!(caps[0].declaring_class, "shop.LeakyKt");
    assert_eq!(caps[1].field, "L$1");
    assert_eq!(caps[1].variable, "tag");
}

fn project(files: &[(&str, Vec<u8>)]) -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    for (rel, bytes) in files {
        let p = t.path().join("target/classes").join(rel);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, bytes).unwrap();
    }
    t
}

#[test]
fn census_lists_the_captures_as_continuation_holders() {
    let t = project(&[(
        "shop/LeakyKt$handle$1.class",
        fixture("LeakyKt$handle$1.class"),
    )]);
    let src = SourceFile {
        rel: "src/shop/Leaky.kt".into(),
        text: "package shop\n".into(),
        lang: Lang::Kotlin,
        is_test: false,
    };
    let r = jvm_class::holders(t.path(), std::slice::from_ref(&src));
    let caps: Vec<_> = r
        .raws
        .iter()
        .filter(|h| h.kind == "coroutine-capture")
        .collect();
    assert_eq!(caps.len(), 2, "{:?}", r.raws);
    assert_eq!(caps[0].owner, "shop.LeakyKt$handle$1.L$0");
    assert_eq!(caps[0].file, "src/shop/Leaky.kt");
    assert_eq!(caps[0].line, 16);
    assert_eq!(caps[0].holder.as_str(), "closure/continuation");
    assert_eq!(caps[0].name.as_deref(), Some("session"));
    assert_eq!(caps[0].scope, Scope::Lines(16, 16));
    assert_eq!(caps[1].owner, "shop.LeakyKt$handle$1.L$1");
    assert_eq!(caps[1].name.as_deref(), Some("tag"));
}

#[test]
fn spring_cglib_owners_name_the_user_class() {
    let t = project(&[(
        "shop/Gateway$$SpringCGLIB$$0.class",
        fixture("Gateway$$SpringCGLIB$$0.class"),
    )]);
    let r = jvm_class::holders(t.path(), &[]);
    let cache = r
        .raws
        .iter()
        .find(|h| h.owner.ends_with(".CACHE"))
        .unwrap_or_else(|| panic!("{:?}", r.raws));
    assert_eq!(cache.owner, "shop.Gateway.CACHE");
    assert_eq!(
        jvm_class::deproxy_name("a.B$$EnhancerBySpringCGLIB$$abcd1234"),
        "a.B"
    );
    assert_eq!(jvm_class::deproxy_name("a.B$Inner"), "a.B$Inner");
    assert_eq!(jvm_class::deproxy_name("a.B$$Lambda/0x01"), "a.B");
}

#[test]
fn malformed_annotations_do_not_lose_the_class() {
    let mut b = fixture("LeakyKt$handle$1.class");
    // Find the RuntimeVisibleAnnotations attribute: pool index 67, a length, then two
    // annotations; break the tag of the first element value.
    let at = (0..b.len() - 8)
        .find(|&i| b[i] == 0 && b[i + 1] == 67 && b[i + 6] == 0 && b[i + 7] == 2)
        .expect("attribute");
    b[at + 6 + 2 + 2 + 2 + 2] = b'X';
    let c = classfile::parse(&b).unwrap();
    assert_eq!(c.source_file.as_deref(), Some("Leaky.kt"));
    assert!(c.annotations.is_empty());
    assert!(jvm_class::coroutine_captures(&c).is_empty());
    // A truncated file is still an error.
    assert!(classfile::parse(&fixture("LeakyKt$handle$1.class")[..600]).is_err());
}

#[test]
fn classes_without_annotations_have_no_captures() {
    let c = classfile::parse(&fixture("Gateway$$SpringCGLIB$$0.class")).unwrap();
    assert!(c.annotations.is_empty());
    assert!(jvm_class::coroutine_captures(&c).is_empty());
}

/// Primitive spills (`I$n`, `J$n`, ...) cannot retain heap and are not census entries.
#[test]
fn primitive_spills_are_skipped() {
    let mut b = fixture("LeakyKt$handle$1.class");
    let at = b
        .windows(3)
        .position(|w| w == b"L$1")
        .expect("L$1 in the constant pool");
    b[at] = b'I';
    let c = classfile::parse(&b).unwrap();
    assert_eq!(
        c.annotation(DEBUG_METADATA).unwrap().strs("s"),
        ["L$0", "I$1"]
    );
    let caps = jvm_class::coroutine_captures(&c);
    assert_eq!(caps.len(), 1);
    assert_eq!(caps[0].field, "L$0");
}
