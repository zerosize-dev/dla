//! `verified` flags of `data/detectors/*.toml` agree with `docs/VERIFY.md`.

use std::{fs, path::PathBuf};

/// Rule ids that `docs/VERIFY.md` lists as unverified, with the text VERIFY.md must
/// still contain for them (so a closed VERIFY row forces this list to be updated).
const UNVERIFIED: &[(&str, &str)] = &[
    ("codeql/java/input-resource-leak", "codeql"),
    ("codeql/java/output-resource-leak", "codeql"),
    ("codeql/java/database-resource-leak", "codeql"),
    ("codeql/py/file-not-closed", "codeql"),
    ("codeql/py/should-use-with", "codeql"),
    (
        "detekt/SuspendFunInFinallySection",
        "SuspendFunInFinallySection",
    ),
    ("golangci-lint/contextcheck", "contextcheck"),
    ("golangci-lint/sqlclosecheck", "sqlclosecheck"),
    ("golangci-lint/spancheck", "spancheck"),
    ("golangci-lint/gocritic/deferInLoop", "deferInLoop"),
    ("staticcheck/SA1015", "SA1015"),
    ("staticcheck/SA5005", "SA5005"),
    ("spotbugs/ODR_OPEN_DATABASE_RESOURCE", "ODR_"),
    ("spotbugs/ODR_OPEN_DATABASE_RESOURCE_EXCEPTION_PATH", "ODR_"),
    ("spotbugs/SIC_THREADLOCAL_DEADLY_EMBRACE", "SIC_"),
    ("spotbugs/SIC_INNER_SHOULD_BE_STATIC", "SIC_"),
    ("spotbugs/SIC_INNER_SHOULD_BE_STATIC_ANON", "SIC_"),
    ("spotbugs/SIC_INNER_SHOULD_BE_STATIC_NEEDS_THIS", "SIC_"),
    ("spotbugs/OS_OPEN_STREAM", "OS_OPEN_STREAM"),
    ("spotbugs/OS_OPEN_STREAM_EXCEPTION_PATH", "OS_OPEN_STREAM"),
    ("errorprone/ThreadLocalUsage", "ThreadLocalUsage"),
    ("errorprone/ClassCanBeStatic", "ClassCanBeStatic"),
    ("ruff/SIM115", "SIM115"),
];

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rows() -> Vec<(String, String, bool)> {
    let mut out = Vec::new();
    for e in fs::read_dir(root().join("data/detectors")).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_none_or(|x| x != "toml") {
            continue;
        }
        let text = fs::read_to_string(&p).unwrap();
        assert!(
            !text.contains("dla-fragment"),
            "{} mentions a fragment file",
            p.display()
        );
        let doc: toml::Table = text.parse().unwrap();
        for d in doc["detector"].as_array().unwrap() {
            out.push((
                d["id"].as_str().unwrap().to_owned(),
                d["backend"].as_str().unwrap().to_owned(),
                d["verified"].as_bool().unwrap(),
            ));
        }
    }
    out
}

#[test]
fn unverified_in_verify_md_means_verified_false() {
    let verify = fs::read_to_string(root().join("docs/VERIFY.md")).unwrap();
    let rows = rows();
    for (id, needle) in UNVERIFIED {
        assert!(
            verify.contains(needle),
            "VERIFY.md no longer mentions {needle}; update this list"
        );
        let r = rows
            .iter()
            .find(|r| r.0 == *id)
            .unwrap_or_else(|| panic!("no row {id}"));
        assert!(
            !r.2,
            "{id} is unverified in VERIFY.md but has verified = true"
        );
    }
}

#[test]
fn only_listed_and_sonar_rows_are_unverified() {
    for (id, backend, verified) in rows() {
        let listed = UNVERIFIED.iter().any(|(i, _)| *i == id);
        if listed || backend == "sonar" {
            assert!(!verified, "{id}");
        } else {
            assert!(
                verified,
                "{id} has verified = false but is not listed in VERIFY.md"
            );
        }
    }
}
