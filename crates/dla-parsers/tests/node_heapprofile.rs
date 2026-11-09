//! A real `node --heap-prof` profile and a real esbuild source map.

use std::{fs, path::PathBuf};

use dla_parsers::node::{heapprofile, sourcemap::SourceMap};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/node")
        .join(name)
}

#[test]
fn a_real_profile_charges_builtins_to_the_javascript_that_called_them() {
    let f = fs::File::open(fixture("retry.heapprofile")).unwrap();
    let p = heapprofile::parse(f).unwrap();
    assert!(p.sampled);
    assert_eq!(p.source, "node-heapprofile");
    assert!(p.groups.len() > 5);
    assert!(
        p.groups
            .windows(2)
            .all(|w| w[0].live_bytes >= w[1].live_bytes)
    );
    // No frame of a stack is a native builtin, and file paths are plain paths.
    for g in &p.groups {
        assert!(!g.frames.is_empty());
        for fr in &g.frames {
            assert!(
                fr.function == "(anonymous)" || !fr.function.starts_with('('),
                "{fr:?}"
            );
            if let Some(file) = &fr.file {
                assert!(!file.starts_with("file://"), "{file}");
            }
        }
    }
    // The workload's own code is there, with a one-based line.
    let ours: Vec<_> = p
        .groups
        .iter()
        .filter_map(|g| {
            g.frames.iter().find(|f| {
                f.file
                    .as_deref()
                    .is_some_and(|x| x.starts_with("/srv/webhook/src/"))
            })
        })
        .collect();
    assert!(!ours.is_empty());
    assert!(ours.iter().all(|f| f.line.is_some_and(|l| l >= 1)));
    // Samples give block counts.
    assert!(p.groups.iter().any(|g| g.live_blocks > 1));
}

#[test]
fn a_real_esbuild_map_resolves_the_minified_function_to_its_source() {
    let json = fs::read_to_string(fixture("bundle.js.map")).unwrap();
    let m = SourceMap::parse(&json, 100_000).unwrap();
    // `function o(e){...}` of out/app.js: line 0, column 15 is `addSession` of a.ts, line 2.
    let at = m.lookup(0, 20).unwrap();
    assert_eq!(at.source, "../src/a.ts");
    assert_eq!(at.line, 1);
    // The loop of main.ts is on its second line.
    let main = m.lookup(0, 120).unwrap();
    assert_eq!(main.source, "../src/main.ts");
    assert!(m.segments() > 10);
}
