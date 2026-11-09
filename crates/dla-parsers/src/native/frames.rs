//! Helpers shared by the native parsers: frame construction, source locations and the
//! trimming of allocator-internal frames.

use super::demangle::demangle_or_raw;
use crate::model::Frame;

/// Splits `path/file.rs:12:51` or `file.c:23` into the file and the line. A missing or zero
/// line gives `None`. Text without a numeric suffix is returned whole as the file.
pub fn split_location(text: &str) -> (String, Option<u32>) {
    let mut rest = text;
    let mut numbers: Vec<u32> = Vec::new();
    // At most line and column.
    for _ in 0..2 {
        match rest.rsplit_once(':') {
            Some((head, tail))
                if !head.is_empty()
                    && !tail.is_empty()
                    && tail.bytes().all(|b| b.is_ascii_digit()) =>
            {
                match tail.parse() {
                    Ok(n) => numbers.push(n),
                    Err(_) => break,
                }
                rest = head;
            }
            _ => break,
        }
    }
    // `numbers` holds the column first when two were found.
    let line = match numbers.as_slice() {
        [] => None,
        [line] => Some(*line),
        [_, line] => Some(*line),
        _ => None,
    };
    (rest.to_owned(), line.filter(|&l| l > 0))
}

/// A frame whose function name is demangled when it is a Rust symbol.
pub fn make_frame(
    function: &str,
    file: Option<String>,
    line: Option<u32>,
    module: Option<String>,
) -> Frame {
    Frame {
        function: demangle_or_raw(function),
        file: file.filter(|f| !f.is_empty()),
        line,
        module: module.filter(|m| !m.is_empty()),
    }
}

/// Whether a frame is allocator plumbing (the C allocator entry points, the Rust global
/// allocator glue, `alloc::` internals and dhat's wrapper) and not code that asked for memory.
pub fn is_allocator_frame(frame: &Frame) -> bool {
    let f = frame.function.as_str();
    const C_ENTRY: [&str; 12] = [
        "malloc",
        "calloc",
        "realloc",
        "reallocarray",
        "memalign",
        "aligned_alloc",
        "posix_memalign",
        "valloc",
        "pvalloc",
        "operator new",
        "operator new[]",
        "__libc_malloc",
    ];
    if C_ENTRY.contains(&f) {
        return true;
    }
    if f.starts_with("__rust_")
        || f.starts_with("__rdl_")
        || f.starts_with("__rg_")
        || f.starts_with("RawVec")
    {
        return true;
    }
    // `<T as alloc::x::Y>::f` counts through either side of the `as`.
    let inner = f.trim_start_matches('<');
    let owned = |p: &str| {
        let p = p.trim_start_matches('<');
        p.starts_with("alloc::") || p.starts_with("core::alloc::") || p.starts_with("dhat::")
    };
    match inner.split_once(" as ") {
        Some((ty, tr)) => owned(ty) || owned(tr),
        None => owned(inner),
    }
}

/// Drops the leading (innermost) allocator-internal frames so the first frame is code that
/// asked for the memory. A stack made only of such frames is returned unchanged.
pub fn trim_allocator_frames(frames: Vec<Frame>) -> Vec<Frame> {
    let skip = frames.iter().take_while(|f| is_allocator_frame(f)).count();
    if skip == 0 || skip == frames.len() {
        return frames;
    }
    frames.into_iter().skip(skip).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(name: &str) -> Frame {
        Frame {
            function: name.to_owned(),
            ..Frame::default()
        }
    }

    #[test]
    fn locations() {
        assert_eq!(split_location("leak.c:11"), ("leak.c".into(), Some(11)));
        assert_eq!(
            split_location("src/main.rs:12:51"),
            ("src/main.rs".into(), Some(12))
        );
        assert_eq!(
            split_location("src/lib.rs:0:0"),
            ("src/lib.rs".into(), None)
        );
        assert_eq!(split_location("plain"), ("plain".into(), None));
        assert_eq!(split_location("a:b"), ("a:b".into(), None));
        assert_eq!(split_location(""), (String::new(), None));
    }

    #[test]
    fn trims_only_leading_allocator_frames() {
        let stack = vec![
            f("malloc"),
            f("alloc::alloc::alloc"),
            f("<alloc::alloc::Global as core::alloc::Allocator>::allocate"),
            f("<alloc::raw_vec::RawVecInner>::try_allocate_in"),
            f("<u8 as alloc::vec::spec_from_elem::SpecFromElem>::from_elem"),
            f("rleak::open_session"),
            f("alloc::vec::from_elem"),
            f("rleak::main"),
        ];
        let out = trim_allocator_frames(stack);
        let names: Vec<_> = out.iter().map(|x| x.function.as_str()).collect();
        assert_eq!(
            names,
            [
                "rleak::open_session",
                "alloc::vec::from_elem",
                "rleak::main"
            ]
        );
    }

    #[test]
    fn keeps_a_stack_of_only_allocator_frames() {
        let stack = vec![f("malloc"), f("alloc::alloc::alloc")];
        assert_eq!(trim_allocator_frames(stack.clone()), stack);
        assert!(trim_allocator_frames(Vec::new()).is_empty());
    }

    #[test]
    fn project_frames_are_not_allocator_frames() {
        assert!(!is_allocator_frame(&f("add_session")));
        assert!(!is_allocator_frame(&f(
            "hashbrown::raw::RawTable<T>::reserve"
        )));
        assert!(!is_allocator_frame(&f(
            "std::collections::hash::map::HashMap<K,V>::insert"
        )));
    }
}
