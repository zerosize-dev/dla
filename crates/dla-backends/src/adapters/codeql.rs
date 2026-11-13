//! CodeQL: build a database, analyse with the query files of the listed rules, read the
//! SARIF. Java without Kotlin is built without compiling (`--build-mode=none`); Kotlin is
//! extracted only by a traced build, so a project with Kotlin sources gets one in a copy
//! (tier 1) or is told plainly that its Kotlin was not read. Written against the CodeQL CLI
//! bundle; it has not been run in the development sandbox (the bundle could not be
//! unpacked there), so the command construction, the build plan and the SARIF reader are
//! covered by unit tests and a fake `codeql` only.

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use dla_core::{runtime::Runtime, static_hit::StaticHit};
use serde_json::{Value, json};

use super::{Job, Outcome, Ran, attach_file, dedupe, hit, isolate, run_record, version_of};
use crate::dla_rules::is_test_path;
use crate::{
    exec::{self, Cmd},
    fsx::{self, ToolEnv},
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

/// `codeql database create` arguments. `build` is the traced build command of a Java
/// database (`--build-mode=manual`) and the directory it runs in (`--working-dir`; CodeQL
/// runs the command in the source root when it is not given, whatever the process's own
/// working directory is); without it a Java database is built without compiling
/// (`--build-mode=none`, Java files only).
pub fn create_args(
    db: &Path,
    language: &str,
    source_root: &Path,
    caches: &Path,
    build: Option<(&str, &Path)>,
) -> Vec<OsString> {
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
        match build {
            Some((cmd, dir)) => {
                a.push("--build-mode=manual".into());
                a.push("--command".into());
                a.push(cmd.into());
                a.push("--working-dir".into());
                a.push(dir.into());
            }
            None => a.push("--build-mode=none".into()),
        }
    }
    a.push(format!("--common-caches={}", caches.display()).into());
    a
}

/// A traced build CodeQL can run in a copy of the project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracedBuild {
    /// Directory of the build root, relative to the project (`""` is the project root).
    pub dir: String,
    /// The command line CodeQL traces.
    pub command: String,
}

/// Every compile stays inside the traced process tree (no daemon, no cache restore, the
/// Kotlin compiler in process) and offline: a sweep has no online fallback.
const GRADLE_FLAGS: &str = "--no-daemon --offline -q --rerun-tasks --no-build-cache -Dorg.gradle.configuration-cache=false -Pkotlin.compiler.execution.strategy=in-process classes";
const MAVEN_FLAGS: &str = "-o -q -B -DskipTests compile";

/// The traced build of the project's Gradle or Maven build, looked for at `dirs` (project
/// relative, first match wins): a wrapper in the directory, else the tool on the search
/// path. `None` when no directory holds a Gradle or Maven build with a way to run it.
pub fn find_traced_build(project: &Path, dirs: &[&str], env: &ToolEnv) -> Option<TracedBuild> {
    let wrapper = |dir: &Path, unix: &str, win: &str| -> Option<String> {
        let name = if cfg!(windows) { win } else { unix };
        dir.join(name).is_file().then(|| {
            if cfg!(windows) {
                name.to_owned()
            } else {
                format!("./{name}")
            }
        })
    };
    // The project's own wrapper (module or root) pins the tool version: it wins over a
    // `gradle` or `mvn` on PATH, which is the second choice in every directory.
    // A root wrapper stands in for the module's own build only when the root build is the same
    // kind: a Gradle `classes` task never compiles the sources of a Maven module.
    let kind_in = |d: &str| -> Option<bool> {
        let dir = if d.is_empty() {
            project.to_path_buf()
        } else {
            project.join(d)
        };
        let has = |n: &str| dir.join(n).is_file();
        if has("build.gradle.kts")
            || has("build.gradle")
            || has("settings.gradle.kts")
            || has("settings.gradle")
        {
            Some(true)
        } else {
            has("pom.xml").then_some(false)
        }
    };
    let own_kind = dirs.first().and_then(|d| kind_in(d));
    for use_path in [false, true] {
        for (i, d) in dirs.iter().enumerate() {
            let dir = if d.is_empty() {
                project.to_path_buf()
            } else {
                project.join(d)
            };
            let has = |n: &str| dir.join(n).is_file();
            let same_kind = |gradle: bool| i == 0 || own_kind.is_none_or(|k| k == gradle);
            if same_kind(true)
                && (has("build.gradle.kts")
                    || has("build.gradle")
                    || has("settings.gradle.kts")
                    || has("settings.gradle"))
            {
                let tool = wrapper(&dir, "gradlew", "gradlew.bat").or_else(|| {
                    use_path
                        .then(|| env.find_program("gradle"))
                        .flatten()
                        .map(|p| p.to_string_lossy().into_owned())
                });
                if let Some(tool) = tool {
                    return Some(TracedBuild {
                        dir: (*d).to_owned(),
                        command: format!("{tool} {GRADLE_FLAGS}"),
                    });
                }
            }
            if same_kind(false) && has("pom.xml") {
                let tool = wrapper(&dir, "mvnw", "mvnw.cmd").or_else(|| {
                    use_path
                        .then(|| env.find_program("mvn"))
                        .flatten()
                        .map(|p| p.to_string_lossy().into_owned())
                });
                if let Some(tool) = tool {
                    return Some(TracedBuild {
                        dir: (*d).to_owned(),
                        command: format!("{tool} {MAVEN_FLAGS}"),
                    });
                }
            }
        }
    }
    None
}

/// How a Java database is built, decided before CodeQL runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JavaPlan {
    /// No Kotlin in scope: built without compiling.
    Buildless,
    /// Kotlin in scope and a traced build is possible.
    Traced(TracedBuild),
    /// Java and Kotlin, no traced build: built without compiling; the limitation names the
    /// Kotlin files that were not extracted.
    JavaOnly(String),
    /// Kotlin only, no traced build: CodeQL would extract nothing.
    Blocked(String),
}

/// `1 Kotlin file`, `3 Kotlin files`.
fn kotlin_files_text(n: usize) -> String {
    format!("{n} Kotlin file{}", if n == 1 { "" } else { "s" })
}

/// The plan for `kotlin` and `java` source files in scope. `traced` is the build when one
/// can run, else the reason it cannot (tier 1 consent missing, no Gradle or Maven build).
pub fn plan_java(kotlin: usize, java: usize, traced: Result<TracedBuild, String>) -> JavaPlan {
    if kotlin == 0 {
        return JavaPlan::Buildless;
    }
    let files = kotlin_files_text;
    match traced {
        Ok(b) => JavaPlan::Traced(b),
        Err(why) if java > 0 => JavaPlan::JavaOnly(format!(
            "{} not extracted: build-mode none reads Java only, and a traced build needs {why}",
            files(kotlin)
        )),
        Err(why) => JavaPlan::Blocked(format!(
            "CodeQL extracts Kotlin only from a traced build (build-mode none reads Java only); {}, no Java files; the traced build runs the project's Gradle or Maven build and needs {why}",
            files(kotlin)
        )),
    }
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

/// Longest reason line kept.
const REASON_CHARS: usize = 200;

/// Why a CodeQL step failed, or `None` when it succeeded. A timeout, a capture
/// overflow or a spawn failure is named as such; otherwise the exit code and the most
/// telling line of the output tail: the first starting with `A fatal error occurred` or
/// `Error:`, else the first of the last three. Every `scrub` path (the run directory) is
/// shown as `<run>`, and a line over 200 characters is cut at a word and ends in `...`.
pub fn failure_text(step: &str, out: &exec::Output, scrub: &[&Path]) -> Option<String> {
    if let Some(r) = out.failure_reason() {
        return Some(if r == "timed out" {
            format!("{step} timed out")
        } else {
            format!("{step} {r}")
        });
    }
    if out.exit == Some(0) {
        return None;
    }
    let tail = out.tail(20);
    let line = tail
        .lines()
        .map(str::trim)
        .find(|l| l.starts_with("A fatal error occurred") || l.starts_with("Error:"))
        .map(str::to_owned)
        .unwrap_or_else(|| out.tail(3).lines().next().unwrap_or("").trim().to_owned());
    let mut line = line;
    for p in scrub {
        let p = p.to_string_lossy();
        if !p.is_empty() {
            line = line.replace(p.as_ref(), "<run>");
        }
    }
    let line = shorten(&line, REASON_CHARS);
    let code = out
        .exit
        .map_or_else(|| "no exit code".to_owned(), |c| format!("exit {c}"));
    Some(format!("{step} failed ({code}): {line}"))
}

/// `line` cut to at most `max` characters at a word boundary, with `...` when cut.
fn shorten(line: &str, max: usize) -> String {
    if line.chars().count() <= max {
        return line.to_owned();
    }
    let cut: String = line.chars().take(max).collect();
    let cut = match cut.rfind(char::is_whitespace) {
        Some(i) if i > 0 => &cut[..i],
        _ => cut.as_str(),
    };
    format!("{} ...", cut.trim_end())
}

/// In-scope Java and Kotlin main sources (no build logic, no tests: the traced build runs
/// `classes`, which compiles neither): `(kotlin, java)`.
fn count_sources(job: &Job) -> (usize, usize) {
    let mut kotlin = 0;
    let mut java = 0;
    for f in fsx::files_with_ext(job.project(), &["java", "kt"]) {
        if !job.in_scope(&f) || dla_core::paths::is_build_logic(&f) || is_test_path(&f) {
            continue;
        }
        if f.ends_with(".kt") {
            kotlin += 1;
        } else {
            java += 1;
        }
    }
    (kotlin, java)
}

/// What to try when a traced build failed because the dependencies could not be fetched
/// offline; empty for every other failure (a compile error is not fixed by a cache).
fn cache_hint(out: &exec::Output) -> &'static str {
    let text = format!("{}\n{}", out.stderr, out.stdout).to_ascii_lowercase();
    let cold = [
        "could not resolve",
        "no cached version",
        "offline mode",
        "unknown host",
        "unknownhostexception",
        "could not transfer artifact",
        "could not get 'http",
        "could not head 'http",
        "unable to tunnel",
        "network is unreachable",
        "connection refused",
        "connect timed out",
    ];
    if cold.iter().any(|m| text.contains(m)) {
        "; the dependencies are not in the build tool's cache: run the project's own build online once, or a dla harness run of a JUnit or driver harness (its build step retries online), then sweep again"
    } else {
        ""
    }
}

/// The traced build available to `job`, or why there is none.
fn traced_build(job: &Job) -> Result<TracedBuild, String> {
    if job
        .ctx
        .require_tier(1, "codeql traced build (runs the project's build)")
        .is_err()
    {
        return Err("tier 1 consent (dla consent)".to_owned());
    }
    let target = job.target();
    let target = if target == "." { "" } else { target.as_str() };
    let mut dirs = vec![target];
    if !target.is_empty() {
        dirs.push("");
    }
    find_traced_build(job.project(), &dirs, job.env)
        .ok_or_else(|| "a Gradle or Maven build with gradlew, gradle, mvnw or mvn".to_owned())
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
    let mut limitations = Vec::new();
    // Kotlin is extracted only by a traced build: decide before running anything.
    let mut kotlin_files = 0;
    let plan = if language == "java" {
        let (kotlin, java) = count_sources(job);
        kotlin_files = kotlin;
        let traced = if kotlin == 0 {
            Err(String::new())
        } else {
            traced_build(job)
        };
        plan_java(kotlin, java, traced)
    } else {
        JavaPlan::Buildless
    };
    let java_files = count_sources(job).1;
    let (source_root, build_dir, mut build) = match plan {
        JavaPlan::Blocked(reason) => return Outcome::blocked(reason, vec![]),
        JavaPlan::Buildless => {
            limitations.push("database built without compiling (build-mode none)".to_owned());
            (job.project().to_path_buf(), None, None)
        }
        JavaPlan::JavaOnly(note) => {
            limitations.push("database built without compiling (build-mode none)".to_owned());
            limitations.push(note);
            (job.project().to_path_buf(), None, None)
        }
        JavaPlan::Traced(b) => {
            // The build writes into its tree: run it on a copy under the scratch directory.
            let copy = job.scratch.join("src");
            let _ = fs::remove_dir_all(&copy);
            if let Err(e) = fsx::copy_tree(job.project(), &copy) {
                return Outcome::blocked(format!("cannot copy the project: {e}"), vec![]);
            }
            let dir = if b.dir.is_empty() {
                copy.clone()
            } else {
                copy.join(&b.dir)
            };
            limitations.push(format!(
                "database built by a traced build in a copy of the project ({}); Kotlin is extracted only when the project's Kotlin version is one the CodeQL bundle supports",
                b.command
            ));
            (copy, Some(dir), Some(b.command))
        }
    };
    let create_cmd = |root: &Path, build: Option<(&str, &Path)>| {
        isolate(Cmd::new(&prog), job)
            .args(create_args(&db, language, root, &caches, build))
            .timeout(job.timeout)
    };
    let mut create = create_cmd(&source_root, build.as_deref().zip(build_dir.as_deref()));
    let mut c_out = exec::run(&create);
    let mut c_rec = run_record(&create, &c_out);
    let mut records = Vec::new();
    if build.is_some()
        && java_files > 0
        && failure_text("codeql database create", &c_out, &[]).is_some()
    {
        // A project with Java must not do worse for consenting to the build: say why the
        // traced build failed and analyse the Java without compiling, as without consent.
        let why = failure_text("the traced build", &c_out, &[job.ctx.run_dir.as_path()])
            .unwrap_or_default();
        limitations.retain(|l| !l.starts_with("database built by a traced build"));
        // The Kotlin note comes first: the coverage row's reason is cut to one short line.
        limitations.push(format!(
            "{} not extracted: {why}{}",
            kotlin_files_text(kotlin_files),
            cache_hint(&c_out)
        ));
        limitations.push("database built without compiling (build-mode none)".to_owned());
        records.push(c_rec);
        build = None;
        let _ = fs::remove_dir_all(job.scratch.join("src"));
        create = create_cmd(job.project(), None);
        c_out = exec::run(&create);
        c_rec = run_record(&create, &c_out);
    }
    if let Some(r) = failure_text(
        "codeql database create",
        &c_out,
        &[job.ctx.run_dir.as_path()],
    ) {
        let r = if build.is_some() {
            format!("{r} (traced offline build in a copy){}", cache_hint(&c_out))
        } else {
            r
        };
        records.push(c_rec);
        let _ = fs::remove_dir_all(job.scratch.join("src"));
        return Outcome::blocked(r, records);
    }
    records.push(c_rec);
    let analyze = isolate(Cmd::new(&prog), job)
        .args(analyze_args(&db, &queries, &sarif_path, &caches))
        .timeout(job.timeout);
    let a_out = exec::run(&analyze);
    // The database holds what was extracted; the build copy is not needed any more.
    let _ = fs::remove_dir_all(job.scratch.join("src"));
    let mut a_rec = run_record(&analyze, &a_out);
    if let Some(r) = failure_text(
        "codeql database analyze",
        &a_out,
        &[job.ctx.run_dir.as_path()],
    ) {
        records.push(a_rec);
        return Outcome::blocked(r, records);
    }
    let Ok(text) = fs::read_to_string(&sarif_path) else {
        records.push(a_rec);
        return Outcome::blocked("codeql wrote no SARIF", records);
    };
    attach_file(&mut a_rec, "output_file_json", &text);
    let Ok(log) = serde_json::from_str::<Value>(&text) else {
        records.push(a_rec);
        return Outcome::blocked("codeql wrote malformed SARIF", records);
    };
    let hits = dedupe(parse(&log, &ids, &job.paths()));
    limitations.extend(unknown.into_iter().map(|u| format!("no query for {u}")));
    Outcome::Ran(Box::new(Ran {
        version,
        command: analyze.display(),
        exit: a_out.exit,
        hits,
        load_errors: 0,
        limitations,
        note: None,
        raw: json!({ "runs": records.into_iter().chain([a_rec]).collect::<Vec<_>>() }),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsx::PathMap;

    #[test]
    fn a_timeout_is_named_and_long_tails_are_shortened() {
        let o = exec::Output {
            timed_out: true,
            stderr: "Initializing database\nmore".into(),
            ..exec::Output::default()
        };
        assert_eq!(
            failure_text("codeql database create", &o, &[]).as_deref(),
            Some("codeql database create timed out")
        );
        let o = exec::Output {
            exit: Some(2),
            stderr: format!("{}\nsecond", "x".repeat(500)),
            ..exec::Output::default()
        };
        let r = failure_text("codeql database analyze", &o, &[]).unwrap();
        assert!(
            r.starts_with("codeql database analyze failed (exit 2): xxx"),
            "{r}"
        );
        assert!(r.len() < 260 && !r.contains("second"), "{r}");
        let ok = exec::Output {
            exit: Some(0),
            ..exec::Output::default()
        };
        assert!(failure_text("s", &ok, &[]).is_none());
    }

    #[test]
    fn a_fatal_error_line_beats_the_third_from_last_and_the_run_dir_is_scrubbed() {
        let run = Path::new("/home/u/proj/.dla/20261112-abcd");
        let o = exec::Output {
            exit: Some(32),
            stderr: format!(
                "Initializing database at {}/tmp/jvm-codeql/db.\nA fatal error occurred: No source code was seen and extracted to {}/tmp/jvm-codeql/db.\nThis can occur when the build did not compile anything.\nSee the log at {}/tmp/log.\nExiting.",
                run.display(),
                run.display(),
                run.display()
            ),
            ..exec::Output::default()
        };
        let r = failure_text("codeql database create", &o, &[run]).unwrap();
        assert_eq!(
            r,
            "codeql database create failed (exit 32): A fatal error occurred: No source code was seen and extracted to <run>/tmp/jvm-codeql/db."
        );
        // No fatal line: the first of the last three, still scrubbed.
        let o = exec::Output {
            exit: None,
            stderr: "a\nb\nc /x/.dla/r\nd\n".into(),
            ..exec::Output::default()
        };
        let r = failure_text("s", &o, &[Path::new("/x/.dla/r")]).unwrap();
        assert_eq!(r, "s failed (no exit code): b");
    }

    #[test]
    fn a_long_line_is_cut_at_a_word_and_says_so() {
        let word = "extraction ";
        let o = exec::Output {
            exit: Some(1),
            stderr: format!("Error: {}", word.repeat(40)),
            ..exec::Output::default()
        };
        let r = failure_text("s", &o, &[]).unwrap();
        assert!(r.ends_with(" ..."), "{r}");
        assert!(!r.contains("extractio ..."), "cut mid-word: {r}");
        assert!(r.chars().count() < 230, "{r}");
    }

    fn traced() -> TracedBuild {
        TracedBuild {
            dir: String::new(),
            command: "./gradlew classes".into(),
        }
    }

    #[test]
    fn java_without_kotlin_is_built_without_compiling() {
        assert_eq!(plan_java(0, 12, Err(String::new())), JavaPlan::Buildless);
        assert_eq!(plan_java(0, 12, Ok(traced())), JavaPlan::Buildless);
    }

    #[test]
    fn kotlin_with_a_possible_traced_build_gets_it() {
        assert_eq!(plan_java(40, 0, Ok(traced())), JavaPlan::Traced(traced()));
        assert_eq!(plan_java(3, 9, Ok(traced())), JavaPlan::Traced(traced()));
    }

    #[test]
    fn mixed_sources_name_the_unextracted_kotlin_files() {
        let JavaPlan::JavaOnly(note) = plan_java(7, 2, Err("tier 1 consent (dla consent)".into()))
        else {
            panic!("a mixed project without a build still runs");
        };
        assert!(note.starts_with("7 Kotlin files not extracted"), "{note}");
        assert!(note.contains("build-mode none reads Java only"), "{note}");
        assert!(note.contains("tier 1"), "{note}");
    }

    #[test]
    fn kotlin_only_without_a_traced_build_is_blocked_with_the_reason() {
        let JavaPlan::Blocked(why) = plan_java(1, 0, Err("tier 1 consent (dla consent)".into()))
        else {
            panic!("nothing to extract");
        };
        assert!(why.contains("1 Kotlin file, no Java files"), "{why}");
        assert!(why.contains("traced build"), "{why}");
        assert!(why.contains("tier 1"), "{why}");
        assert!(!why.contains("Some("), "{why}");
    }

    #[test]
    fn the_traced_build_prefers_the_wrapper_and_falls_back_to_the_tool() {
        let d = tempfile::tempdir().unwrap();
        let env = ToolEnv {
            tools_dir: None,
            path: Vec::new(),
            vars: Default::default(),
        };
        // Nothing to build.
        assert_eq!(find_traced_build(d.path(), &[""], &env), None);
        fs::write(d.path().join("build.gradle.kts"), "").unwrap();
        // A Gradle build but no way to run it.
        assert_eq!(find_traced_build(d.path(), &[""], &env), None);
        fs::write(d.path().join("gradlew"), "").unwrap();
        fs::write(d.path().join("gradlew.bat"), "").unwrap();
        let b = find_traced_build(d.path(), &[""], &env).unwrap();
        assert!(b.dir.is_empty());
        // On Windows the wrapper is the .bat file.
        let gradlew = if cfg!(windows) {
            "gradlew.bat"
        } else {
            "gradlew"
        };
        let mvnw = if cfg!(windows) { "mvnw.cmd" } else { "mvnw" };
        assert!(
            b.command
                .contains(&format!("{gradlew} --no-daemon --offline -q --rerun-tasks"))
                && b.command.ends_with("classes"),
            "{}",
            b.command
        );
        assert!(
            b.command
                .contains("kotlin.compiler.execution.strategy=in-process")
        );
        // Maven in a module directory is found from the second candidate.
        let m = tempfile::tempdir().unwrap();
        fs::create_dir_all(m.path().join("svc")).unwrap();
        fs::write(m.path().join("svc/pom.xml"), "").unwrap();
        fs::write(m.path().join("svc/mvnw"), "").unwrap();
        fs::write(m.path().join("svc/mvnw.cmd"), "").unwrap();
        let b = find_traced_build(m.path(), &["svc", ""], &env).unwrap();
        assert_eq!(b.dir, "svc");
        assert!(
            b.command
                .contains(&format!("{mvnw} -o -q -B -DskipTests compile")),
            "{}",
            b.command
        );
    }

    #[cfg(unix)]
    #[test]
    fn the_projects_wrapper_wins_over_a_tool_on_path() {
        use std::os::unix::fs::PermissionsExt;
        let bin = tempfile::tempdir().unwrap();
        let g = bin.path().join("gradle");
        fs::write(&g, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&g, fs::Permissions::from_mode(0o755)).unwrap();
        let env = ToolEnv {
            tools_dir: None,
            path: vec![bin.path().to_path_buf()],
            vars: Default::default(),
        };
        let d = tempfile::tempdir().unwrap();
        fs::create_dir_all(d.path().join("svc")).unwrap();
        fs::write(d.path().join("svc/build.gradle.kts"), "").unwrap();
        fs::write(d.path().join("settings.gradle.kts"), "").unwrap();
        fs::write(d.path().join("gradlew"), "").unwrap();
        // A module scope: the root wrapper pins the version, not the `gradle` on PATH.
        let b = find_traced_build(d.path(), &["svc", ""], &env).unwrap();
        assert_eq!(b.dir, "");
        assert!(b.command.starts_with("./gradlew "), "{}", b.command);
        // Without any wrapper the tool on PATH is used, in the first directory.
        fs::remove_file(d.path().join("gradlew")).unwrap();
        let b = find_traced_build(d.path(), &["svc", ""], &env).unwrap();
        assert_eq!(b.dir, "svc");
        assert!(b.command.contains("/gradle "), "{}", b.command);
    }

    #[cfg(unix)]
    #[test]
    fn a_root_wrapper_of_another_build_kind_does_not_stand_in_for_the_modules_build() {
        use std::os::unix::fs::PermissionsExt;
        let bin = tempfile::tempdir().unwrap();
        let mvn = bin.path().join("mvn");
        fs::write(&mvn, "#!/bin/sh\n").unwrap();
        fs::set_permissions(&mvn, fs::Permissions::from_mode(0o755)).unwrap();
        let env = ToolEnv {
            tools_dir: None,
            path: vec![bin.path().to_path_buf()],
            vars: Default::default(),
        };
        let d = tempfile::tempdir().unwrap();
        fs::create_dir_all(d.path().join("svc")).unwrap();
        fs::write(d.path().join("svc/pom.xml"), "").unwrap();
        fs::write(d.path().join("settings.gradle.kts"), "").unwrap();
        fs::write(d.path().join("gradlew"), "").unwrap();
        // A Maven module under an unrelated Gradle root: `mvn` in the module, not `./gradlew`.
        let b = find_traced_build(d.path(), &["svc", ""], &env).unwrap();
        assert_eq!(b.dir, "svc");
        assert!(b.command.contains("/mvn "), "{}", b.command);
        // Without `mvn` the root Gradle build would compile nothing of the module: no build.
        fs::remove_file(&mvn).unwrap();
        assert_eq!(find_traced_build(d.path(), &["svc", ""], &env), None);
        // A Maven wrapper at a Maven root is the same kind and stands in.
        let m = tempfile::tempdir().unwrap();
        fs::create_dir_all(m.path().join("svc")).unwrap();
        fs::write(m.path().join("svc/pom.xml"), "").unwrap();
        fs::write(m.path().join("pom.xml"), "").unwrap();
        fs::write(m.path().join("mvnw"), "").unwrap();
        let b = find_traced_build(m.path(), &["svc", ""], &env).unwrap();
        assert_eq!(b.dir, "");
        assert!(b.command.starts_with("./mvnw "), "{}", b.command);
    }

    #[test]
    fn the_cache_hint_names_every_offline_marker_of_the_harness_build() {
        for text in [
            "java.io.IOException: Unable to tunnel through proxy. Proxy returns \"HTTP/1.1 403 Forbidden\"",
            "connect: Network is unreachable",
            "java.net.ConnectException: Connection refused",
            "Read timed out: connect timed out",
            "Could not GET 'https://repo.maven.apache.org/x'",
            "java.net.UnknownHostException: repo.maven.apache.org",
        ] {
            let out = exec::Output {
                exit: Some(2),
                stderr: text.into(),
                ..exec::Output::default()
            };
            assert!(
                cache_hint(&out).contains("run the project's own build online"),
                "{text}"
            );
        }
    }

    #[test]
    fn a_build_script_error_is_not_a_cold_cache() {
        let out = exec::Output {
            exit: Some(1),
            stderr: "Could not get unknown property 'x' for root project 'app'".into(),
            ..exec::Output::default()
        };
        assert_eq!(cache_hint(&out), "");
    }

    #[test]
    fn one_kotlin_file_reads_in_the_singular() {
        assert_eq!(kotlin_files_text(1), "1 Kotlin file");
        assert_eq!(kotlin_files_text(3), "3 Kotlin files");
    }

    #[test]
    fn the_cache_hint_is_given_only_for_a_cold_dependency_cache() {
        let cold = exec::Output {
            exit: Some(1),
            stderr: "Could not resolve org.jetbrains.kotlin:kotlin-stdlib:1.9.24.\n> No cached version available for offline mode".into(),
            ..exec::Output::default()
        };
        let h = cache_hint(&cold);
        assert!(h.contains("run the project's own build online"), "{h}");
        assert!(h.contains("harness run"), "{h}");
        // `dla probe --build` is offline too: it cannot fill a cold cache.
        assert!(!h.contains("probe --build"), "{h}");
        let compile = exec::Output {
            exit: Some(1),
            stderr: "e: A.kt:3:1 Unresolved reference: foo".into(),
            ..exec::Output::default()
        };
        assert_eq!(cache_hint(&compile), "");
        // A wrapper that cannot download its distribution offline is a cold cache.
        let wrapper = exec::Output {
            exit: Some(1),
            stderr: "Exception in thread main java.net.UnknownHostException: services.gradle.org"
                .into(),
            ..exec::Output::default()
        };
        assert!(cache_hint(&wrapper).contains("online"), "unknown host");
        // A compile error that says "cannot access" is not.
        let access = exec::Output {
            exit: Some(1),
            stderr: "error: cannot access Foo\n  class file for Foo not found".into(),
            ..exec::Output::default()
        };
        assert_eq!(cache_hint(&access), "");
    }

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
            None,
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
            None,
        );
        assert!(!py.iter().any(|x| x == "--build-mode=none"));
        let k = create_args(
            Path::new("/t/db"),
            "java",
            Path::new("/t/copy"),
            Path::new("/t/cc"),
            Some(("./gradlew classes", Path::new("/t/copy/svc"))),
        );
        let k: Vec<String> = k.iter().map(|x| x.to_string_lossy().into_owned()).collect();
        assert!(k.contains(&"--build-mode=manual".to_owned()), "{k:?}");
        assert!(!k.contains(&"--build-mode=none".to_owned()), "{k:?}");
        let at = k.iter().position(|x| x == "--command").unwrap();
        assert_eq!(k[at + 1], "./gradlew classes");
        // CodeQL runs the command in the source root unless told otherwise: a module's
        // build directory must be passed, not left to the process's working directory.
        let wd = k.iter().position(|x| x == "--working-dir").unwrap();
        assert_eq!(k[wd + 1], "/t/copy/svc");
        assert!(!s.iter().any(|x| x == "--working-dir"), "{s:?}");
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
