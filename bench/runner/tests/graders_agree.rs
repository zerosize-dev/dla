//! The Rust and Python graders must write the same metrics for every sample
//! under testdata/, including the synthetic ledgers that cover the worker,
//! merge, owner and artifact rules of bench/GRADING.md.

use std::{fs, path::Path, process::Command};

use dla_bench::{compare, grade};

fn python() -> Option<&'static str> {
    ["python3", "python"].into_iter().find(|p| {
        Command::new(p)
            .args(["-c", "import tomllib"])
            .output()
            .is_ok_and(|o| o.status.success())
    })
}

#[test]
fn both_graders_write_identical_metrics_for_every_sample() {
    let Some(py) = python() else {
        eprintln!("skipped: no python3 with tomllib on PATH");
        return;
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = tempfile::tempdir().unwrap();
    let mut samples = 0;
    for e in fs::read_dir(root.join("testdata")).unwrap().flatten() {
        let s = e.path();
        if !s.join("results").is_dir() {
            continue;
        }
        samples += 1;
        let name = e.file_name().to_string_lossy().into_owned();
        let m = grade::grade_dir(&s.join("results"), &s.join("fixtures")).unwrap();
        let rust: serde_json::Value = serde_json::from_str(&grade::render(&m)).unwrap();
        let py_out = out.path().join(format!("{name}.json"));
        let status = Command::new(py)
            .arg("-I")
            .arg(root.join("grade.py"))
            .arg(s.join("results"))
            .arg("--fixtures")
            .arg(s.join("fixtures"))
            .arg("--out")
            .arg(&py_out)
            .status()
            .unwrap();
        assert!(status.success(), "grade.py failed on {name}");
        let python: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&py_out).unwrap()).unwrap();
        let diff = compare::diff(&rust, &python);
        assert!(diff.is_empty(), "{name}: graders differ: {diff:#?}");
        for golden in ["expected-metrics.json", "metrics.json"] {
            if let Ok(text) = fs::read_to_string(s.join(golden)) {
                let want: serde_json::Value = serde_json::from_str(&text).unwrap();
                let diff = compare::diff(&rust, &want);
                assert!(diff.is_empty(), "{name}/{golden}: {diff:#?}");
            }
        }
    }
    assert!(samples >= 3, "expected the three samples, saw {samples}");
}
