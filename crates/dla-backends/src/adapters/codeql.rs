//! CodeQL: build a database without compiling, analyse with the query files of the
//! listed rules, read the SARIF. Written against the CodeQL CLI bundle; it has not
//! been run in the development sandbox (the bundle could not be unpacked there), so
//! the command construction and the SARIF reader are covered by unit tests only.

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use dla_core::{runtime::Runtime, static_hit::StaticHit};
use serde_json::{Value, json};

use super::{Job, Outcome, Ran, attach_file, dedupe, hit, isolate, run_record, version_of};
use crate::{
    exec::{self, Cmd},
    sarif,
};

/// Query id -> (query pack, path of the `.ql` inside the pack).
pub const QUERIES: [(&str, &str, &str); 5] = [
    (
        "java/input-resource-leak",
        "java-queries",
        "Likely Bugs/Resource Leaks/CloseReader.ql",
    ),
    (
        "java/output-resource-leak",
        "java-queries",
        "Likely Bugs/Resource Leaks/CloseWriter.ql",
    ),
    (
        "java/database-resource-leak",
        "java-queries",
        "Likely Bugs/Resource Leaks/CloseSql.ql",
    ),
    (
        "py/file-not-closed",
        "python-queries",
        "Resources/FileNotAlwaysClosed.ql",
    ),
    (
        "py/should-use-with",
        "python-queries",
        "Statements/ShouldUseWithStatement.ql",
    ),
];

/// The CodeQL language of a runtime.
pub fn language(lang: Runtime) -> Option<&'static str> {
    match lang {
        Runtime::Jvm => Some("java"),
        Runtime::Python => Some("python"),
        _ => None,
    }
}

/// Newest version directory of `<home>/qlpacks/codeql/<pack>`.
fn pack_dir(home: &Path, pack: &str) -> Option<PathBuf> {
    let base = home.join("qlpacks/codeql").join(pack);
    let mut vers: Vec<PathBuf> = fs::read_dir(base)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    vers.sort();
    vers.pop()
}

/// Query files for the wanted rule ids and the ids that have no known query.
pub fn query_files(home: &Path, lang_prefix: &str, rules: &[&str]) -> (Vec<PathBuf>, Vec<String>) {
    let mut files = Vec::new();
    let mut unknown = Vec::new();
    for r in rules {
        if !r.starts_with(lang_prefix) {
            continue;
        }
        match QUERIES.iter().find(|(id, _, _)| id == r) {
            Some((_, pack, rel)) => match pack_dir(home, pack) {
                Some(d) if d.join(rel).is_file() => files.push(d.join(rel)),
                _ => unknown.push(format!("{r} (query pack {pack} not found in the bundle)")),
            },
            None => unknown.push((*r).to_owned()),
        }
    }
    (files, unknown)
}

/// `codeql database create` arguments.
pub fn create_args(db: &Path, language: &str, source_root: &Path, caches: &Path) -> Vec<OsString> {
    let mut a: Vec<OsString> = vec![
        "database".into(),
        "create".into(),
        db.into(),
        format!("--language={language}").into(),
        "--source-root".into(),
        source_root.into(),
        "--overwrite".into(),
        "--threads=0".into(),
    ];
    if language == "java" {
        a.push("--build-mode=none".into());
    }
    a.push(format!("--common-caches={}", caches.display()).into());
    a
}

/// `codeql database analyze` arguments.
pub fn analyze_args(db: &Path, queries: &[PathBuf], out: &Path, caches: &Path) -> Vec<OsString> {
    let mut a: Vec<OsString> = vec!["database".into(), "analyze".into(), db.into()];
    a.extend(queries.iter().map(OsString::from));
    a.extend([
        "--format=sarif-latest".into(),
        format!("--output={}", out.display()).into(),
        "--threads=0".into(),
        "--rerun".into(),
        format!("--common-caches={}", caches.display()).into(),
    ]);
    a
}

/// Reads CodeQL SARIF: the rule id is the query id, locations are project-relative.
pub fn parse(log: &Value, wanted: &[&str], paths: &crate::fsx::PathMap) -> Vec<StaticHit> {
    sarif::results(log)
        .into_iter()
        .filter(|r| wanted.contains(&r.rule.as_str()))
        .filter_map(|r| {
            let f = paths.rel(&r.uri)?;
            Some(hit("codeql", &r.rule, &f, r.line, &r.message))
        })
        .collect()
}

/// Runs CodeQL for the runtime's language.
pub fn run(job: &Job) -> Outcome {
    let Some(prog) = job.env.find_program("codeql") else {
        return Outcome::missing("codeql is not on PATH or in the tools directory");
    };
    let version = version_of(Cmd::new(&prog).args(["version", "--format=terse"]));
    let Some(language) = language(job.lang) else {
        return Outcome::blocked("CodeQL has no adapter for this runtime", vec![]);
    };
    let prefix = if language == "java" { "java/" } else { "py/" };
    let home = prog
        .canonicalize()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .unwrap_or_default();
    let ids = job.rule_ids();
    let (queries, unknown) = query_files(&home, prefix, &ids);
    if queries.is_empty() {
        return Outcome::blocked(
            format!(
                "no query files found for the listed rules: {}",
                unknown.join(", ")
            ),
            vec![],
        );
    }
    let db = job.scratch.join("db");
    let caches = job.scratch.join("cache/codeql");
    let sarif_path = job.scratch.join("codeql.sarif");
    let _ = fs::remove_file(&sarif_path);
    let create = isolate(Cmd::new(&prog), job)
        .args(create_args(&db, language, job.project(), &caches))
        .timeout(job.timeout);
    let c_out = exec::run(&create);
    let c_rec = run_record(&create, &c_out);
    if c_out.exit != Some(0) || c_out.failure_reason().is_some() {
        return Outcome::blocked(
            format!(
                "codeql database create failed (exit {:?}): {}",
                c_out.exit,
                c_out.tail(3)
            ),
            vec![c_rec],
        );
    }
    let analyze = isolate(Cmd::new(&prog), job)
        .args(analyze_args(&db, &queries, &sarif_path, &caches))
        .timeout(job.timeout);
    let a_out = exec::run(&analyze);
    let mut a_rec = run_record(&analyze, &a_out);
    if a_out.exit != Some(0) || a_out.failure_reason().is_some() {
        return Outcome::blocked(
            format!(
                "codeql database analyze failed (exit {:?}): {}",
                a_out.exit,
                a_out.tail(3)
            ),
            vec![c_rec, a_rec],
        );
    }
    let Ok(text) = fs::read_to_string(&sarif_path) else {
        return Outcome::blocked("codeql wrote no SARIF", vec![c_rec, a_rec]);
    };
    attach_file(&mut a_rec, "output_file_json", &text);
    let Ok(log) = serde_json::from_str::<Value>(&text) else {
        return Outcome::blocked("codeql wrote malformed SARIF", vec![c_rec, a_rec]);
    };
    let hits = dedupe(parse(&log, &ids, &job.paths()));
    let mut limitations = vec!["database built without compiling (build-mode none)".to_owned()];
    limitations.extend(unknown.into_iter().map(|u| format!("no query for {u}")));
    Outcome::Ran(Box::new(Ran {
        version,
        command: analyze.display(),
        exit: a_out.exit,
        hits,
        load_errors: 0,
        limitations,
        note: None,
        raw: json!({ "runs": [c_rec, a_rec] }),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsx::PathMap;

    #[test]
    fn queries_resolve_inside_the_bundle() {
        let home = tempfile::tempdir().unwrap();
        let ql = home
            .path()
            .join("qlpacks/codeql/java-queries/1.12.0/Likely Bugs/Resource Leaks/CloseReader.ql");
        fs::create_dir_all(ql.parent().unwrap()).unwrap();
        fs::write(&ql, "").unwrap();
        let (files, unknown) = query_files(
            home.path(),
            "java/",
            &[
                "java/input-resource-leak",
                "java/output-resource-leak",
                "java/nonexistent",
                "py/file-not-closed",
            ],
        );
        assert_eq!(files, vec![ql]);
        assert_eq!(unknown.len(), 2);
        assert!(unknown[0].contains("java/output-resource-leak"));
        assert_eq!(unknown[1], "java/nonexistent");
    }

    #[test]
    fn commands_are_built_for_java_without_a_build() {
        let c = create_args(
            Path::new("/t/db"),
            "java",
            Path::new("/p"),
            Path::new("/t/cc"),
        );
        let s: Vec<String> = c.iter().map(|x| x.to_string_lossy().into_owned()).collect();
        assert_eq!(&s[..3], ["database", "create", "/t/db"]);
        assert!(s.contains(&"--language=java".to_owned()));
        assert!(s.contains(&"--build-mode=none".to_owned()));
        assert!(s.contains(&"--common-caches=/t/cc".to_owned()));
        let py = create_args(
            Path::new("/t/db"),
            "python",
            Path::new("/p"),
            Path::new("/t/cc"),
        );
        assert!(!py.iter().any(|x| x == "--build-mode=none"));
        let a = analyze_args(
            Path::new("/t/db"),
            &[PathBuf::from("/q/A.ql")],
            Path::new("/t/o.sarif"),
            Path::new("/t/cc"),
        );
        let s: Vec<String> = a.iter().map(|x| x.to_string_lossy().into_owned()).collect();
        assert_eq!(&s[..4], ["database", "analyze", "/t/db", "/q/A.ql"]);
        assert!(s.contains(&"--format=sarif-latest".to_owned()));
        assert!(s.contains(&"--output=/t/o.sarif".to_owned()));
    }

    #[test]
    fn sarif_results_become_hits() {
        let log = json!({"runs":[{"results":[
            {"ruleId":"java/input-resource-leak","message":{"text":"This FileInputStream is not always closed."},
             "locations":[{"physicalLocation":{"artifactLocation":{"uri":"src/A.java","uriBaseId":"%SRCROOT%"},"region":{"startLine":12}}}]},
            {"ruleId":"java/other","message":{"text":"x"},
             "locations":[{"physicalLocation":{"artifactLocation":{"uri":"src/A.java"},"region":{"startLine":1}}}]}]}]});
        let h = parse(
            &log,
            &["java/input-resource-leak"],
            &PathMap::new(Path::new("/p")),
        );
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].site(), "src/A.java:12");
        assert_eq!(h[0].origin(), "detector:codeql/java/input-resource-leak");
    }

    #[test]
    fn languages() {
        assert_eq!(language(Runtime::Jvm), Some("java"));
        assert_eq!(language(Runtime::Python), Some("python"));
        assert_eq!(language(Runtime::Go), None);
    }
}
