//! JVM: Error Prone (javac plugin, text), SpotBugs (SARIF, needs class files) and
//! detekt 2.x (SARIF, full analysis).

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use dla_core::static_hit::StaticHit;
use regex::Regex;
use serde_json::{Value, json};

use super::{Job, Outcome, Ran, attach_file, dedupe, hit, isolate, run_record, version_of};
use crate::{
    exec::{self, Cmd},
    fsx::{self, PathMap, ToolEnv},
    sarif,
};

fn not_applicable(version: String, why: &str) -> Outcome {
    Outcome::Ran(Box::new(Ran {
        version,
        note: Some(format!("not applicable: {why}")),
        raw: json!({ "runs": [] }),
        ..Ran::default()
    }))
}

/// Jars found in the project (libraries a build copied next to the sources).
fn project_jars(project: &Path, limit: usize) -> Vec<PathBuf> {
    fsx::walk_with(
        project,
        limit,
        &|n| matches!(n, ".git" | ".dla" | "node_modules"),
        &|r| r.ends_with(".jar"),
    )
    .into_iter()
    .map(|r| project.join(r))
    .collect()
}

fn join_paths(items: &[PathBuf]) -> OsString {
    std::env::join_paths(items).unwrap_or_default()
}

/// Major version in `javac 21.0.12` / `openjdk version "17.0.2"` / `1.8.0_292`.
pub fn java_major(version: &str) -> Option<u32> {
    let mut it = version.split(['.', '_', '-']);
    let first: u32 = it.next()?.parse().ok()?;
    if first == 1 {
        it.next()?.parse().ok()
    } else {
        Some(first)
    }
}

// ---------------------------------------------------------------- Error Prone

/// The javac invocation of Error Prone enabling exactly `checks` as warnings.
#[allow(clippy::too_many_arguments)]
pub fn errorprone_args(
    classes: &Path,
    processor_path: &[PathBuf],
    checks: &[&str],
    classpath: &[PathBuf],
    argfile: &Path,
) -> Vec<OsString> {
    let mut a: Vec<OsString> = vec!["-d".into(), classes.into(), "-proc:none".into()];
    a.extend(
        [
            "-Xmaxwarns",
            "1000000",
            "-Xmaxerrs",
            "1000000",
            "-encoding",
            "UTF-8",
        ]
        .map(OsString::from),
    );
    for pkg in [
        "api",
        "file",
        "main",
        "model",
        "parser",
        "processing",
        "tree",
        "util",
        "code",
        "comp",
    ] {
        a.push(
            format!("-J--add-exports=jdk.compiler/com.sun.tools.javac.{pkg}=ALL-UNNAMED").into(),
        );
    }
    for pkg in ["code", "comp"] {
        a.push(format!("-J--add-opens=jdk.compiler/com.sun.tools.javac.{pkg}=ALL-UNNAMED").into());
    }
    a.extend(
        [
            "-XDcompilePolicy=simple",
            "-XDaddTypeAnnotationsToSymbol=true",
            "--should-stop=ifError=FLOW",
        ]
        .map(OsString::from),
    );
    a.push("-processorpath".into());
    a.push(join_paths(processor_path));
    let mut plugin = String::from("-Xplugin:ErrorProne -XepDisableAllChecks");
    for c in checks {
        plugin.push_str(&format!(" -Xep:{c}:WARN"));
    }
    a.push(plugin.into());
    if !classpath.is_empty() {
        a.push("-cp".into());
        a.push(join_paths(classpath));
    }
    let mut at = OsString::from("@");
    at.push(argfile);
    a.push(at);
    a
}

/// Hits and compile errors from javac's default diagnostics.
#[derive(Debug, Default)]
pub struct EpParsed {
    /// Hits for wanted checks.
    pub hits: Vec<StaticHit>,
    /// Compile errors that are not Error Prone checks.
    pub errors: u64,
    /// The first such error.
    pub first_error: Option<String>,
}

/// Parses `file:line: warning: [Check] message` diagnostics.
pub fn parse_errorprone(text: &str, wanted: &[&str], paths: &PathMap) -> EpParsed {
    let re = Regex::new(
        r"^(?P<file>.+\.java):(?P<line>\d+): (?P<sev>warning|error): (?:\[(?P<rule>[A-Za-z0-9_]+)\] )?(?P<msg>.*)$",
    )
    .expect("static regex");
    let mut p = EpParsed::default();
    for line in text.lines() {
        let Some(c) = re.captures(line) else { continue };
        let rule = c.name("rule").map(|m| m.as_str());
        match rule {
            Some(r) if wanted.contains(&r) => {
                let line_no: u32 = c["line"].parse().unwrap_or(1);
                if let Some(f) = paths.rel(&c["file"]) {
                    p.hits.push(hit("errorprone", r, &f, line_no, &c["msg"]));
                }
            }
            Some(_) => {}
            None => {
                if &c["sev"] == "error" {
                    p.errors += 1;
                    p.first_error.get_or_insert_with(|| line.to_owned());
                }
            }
        }
    }
    p
}

/// Compiles the project's Java sources with the listed Error Prone checks enabled.
pub fn errorprone(job: &Job) -> Outcome {
    let Some(javac) = job.env.find_jdk_tool("javac") else {
        return Outcome::missing("javac not found: a JDK 17 or newer is required");
    };
    let Some(dir) = job.env.jar_dir("errorprone") else {
        return Outcome::missing(
            "Error Prone jars not found in the tools directory (or DLA_ERRORPRONE_HOME)",
        );
    };
    let jars = fsx::jars_in(&dir);
    let version = jars
        .iter()
        .filter_map(|j| j.file_name()?.to_str().map(str::to_owned))
        .find_map(|n| {
            n.strip_prefix("error_prone_core-")
                .and_then(|r| r.strip_suffix(".jar"))
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "unknown".to_owned());
    let javac_version = version_of(
        Cmd::new(&javac)
            .arg("-version")
            .env_remove("JAVA_TOOL_OPTIONS"),
    );
    if java_major(&javac_version).is_some_and(|m| m < 17) {
        return Outcome::blocked(
            format!("Error Prone needs JDK 17 or newer; javac is {javac_version}"),
            vec![],
        );
    }
    let sources = fsx::files_with_ext(job.project(), &["java"]);
    if sources.is_empty() {
        return not_applicable(version, "no Java sources");
    }
    let classes = job.scratch.join("classes");
    let argfile = job.scratch.join("sources.args");
    let mut af = String::new();
    for s in &sources {
        let abs = job.project().join(s);
        af.push('"');
        af.push_str(
            &abs.to_string_lossy()
                .replace('\\', "\\\\")
                .replace('"', "\\\""),
        );
        af.push_str("\"\n");
    }
    if let Err(e) = fs::create_dir_all(&classes).and_then(|()| fs::write(&argfile, af)) {
        return Outcome::blocked(format!("cannot prepare the scratch directory: {e}"), vec![]);
    }
    let ids = job.rule_ids();
    let cp = project_jars(job.project(), 500);
    let cmd = isolate(Cmd::new(&javac), job)
        .cwd(job.project())
        .env_remove("JAVA_TOOL_OPTIONS")
        .args(errorprone_args(&classes, &jars, &ids, &cp, &argfile))
        .timeout(job.timeout);
    let out = exec::run(&cmd);
    let rec = run_record(&cmd, &out);
    if let Some(r) = out.failure_reason() {
        return Outcome::blocked(format!("javac {r}"), vec![rec]);
    }
    let text = format!("{}\n{}", out.stdout, out.stderr);
    let parsed = parse_errorprone(&text, &ids, &job.paths());
    if parsed.hits.is_empty() && parsed.errors > 0 {
        return Outcome::blocked(
            format!(
                "javac failed with {} error(s), Error Prone did not run to completion; first: {}",
                parsed.errors,
                parsed.first_error.as_deref().unwrap_or("")
            ),
            vec![rec],
        );
    }
    let mut limitations = Vec::new();
    if parsed.errors > 0 {
        limitations.push(format!(
            "{} compile error(s) (classpath incomplete?); first: {}",
            parsed.errors,
            parsed.first_error.as_deref().unwrap_or("")
        ));
    }
    if cp.is_empty() {
        limitations.push(
            "compiled without dependency jars; checks needing library types may not fire"
                .to_owned(),
        );
    }
    Outcome::Ran(Box::new(Ran {
        version,
        command: cmd.display(),
        exit: out.exit,
        hits: dedupe(parsed.hits),
        load_errors: parsed.errors,
        limitations,
        note: None,
        raw: json!({ "runs": [rec], "sources": sources.len() }),
    }))
}

// ------------------------------------------------------------------- SpotBugs

/// The bug-pattern filter file for `patterns`.
pub fn spotbugs_filter(patterns: &[&str]) -> String {
    format!(
        "<FindBugsFilter><Match><Bug pattern=\"{}\"/></Match></FindBugsFilter>\n",
        patterns.join(",")
    )
}

/// Topmost directories that directly contain class files (project-relative).
pub fn class_roots(class_files: &[String]) -> Vec<String> {
    let mut dirs: Vec<String> = class_files
        .iter()
        .map(|f| f.rsplit_once('/').map_or("", |(d, _)| d).to_owned())
        .collect();
    dirs.sort();
    dirs.dedup();
    let mut roots: Vec<String> = Vec::new();
    for d in dirs {
        let covered = roots
            .iter()
            .any(|r| r.is_empty() || d == *r || d.starts_with(&format!("{r}/")));
        if !covered {
            roots.push(d);
        }
    }
    roots
}

/// Parses SpotBugs SARIF; `uri` is relative to a source root, so it is matched
/// against the project's source files by suffix. Returns hits and the number of hits
/// whose file could not be found.
pub fn parse_spotbugs(log: &Value, wanted: &[&str], sources: &[String]) -> (Vec<StaticHit>, u64) {
    let mut hits = Vec::new();
    let mut unresolved = 0;
    for r in sarif::results(log) {
        if !wanted.contains(&r.rule.as_str()) {
            continue;
        }
        match fsx::resolve_suffix(sources, &r.uri) {
            Some(f) => hits.push(hit("spotbugs", &r.rule, &f, r.line, &r.message)),
            None => unresolved += 1,
        }
    }
    (hits, unresolved)
}

/// Runs SpotBugs over the compiled classes found in the project.
pub fn spotbugs(job: &Job) -> Outcome {
    let java = job.env.find_jdk_tool("java");
    let (base, version_cmd): (Cmd, Cmd) = if let Some(dir) = job.env.jar_dir("spotbugs") {
        let Some(java) = java else {
            return Outcome::missing("java not found: SpotBugs needs a JDK or JRE");
        };
        let cp = dir.join("*");
        let mk = || {
            Cmd::new(&java)
                .env_remove("JAVA_TOOL_OPTIONS")
                .args(["-Xmx1g", "-cp"])
                .arg(&cp)
                .arg("edu.umd.cs.findbugs.FindBugs2")
        };
        (mk(), mk().arg("-version"))
    } else if let Some(p) = job.env.find_program("spotbugs") {
        (
            Cmd::new(&p).env_remove("JAVA_TOOL_OPTIONS"),
            Cmd::new(&p).env_remove("JAVA_TOOL_OPTIONS").arg("-version"),
        )
    } else {
        return Outcome::missing("SpotBugs not found in the tools directory or on PATH");
    };
    let version = version_of(version_cmd);
    let class_files = fsx::walk_with(
        job.project(),
        200_000,
        &|n| matches!(n, ".git" | ".dla" | "node_modules"),
        &|r| r.ends_with(".class"),
    );
    if class_files.is_empty() {
        return Outcome::blocked("no class files: run dla probe --build", vec![]);
    }
    let roots = class_roots(&class_files);
    let ids = job.rule_ids();
    let filter = job.scratch.join("include.xml");
    let out_path = job.scratch.join("spotbugs.sarif");
    if let Err(e) = fsx::write(&filter, &spotbugs_filter(&ids)) {
        return Outcome::blocked(format!("cannot write the filter: {e}"), vec![]);
    }
    let _ = fs::remove_file(&out_path);
    let mut cmd =
        isolate(base, job)
            .cwd(job.project())
            .args(["-experimental", "-effort:max", "-low"]);
    cmd = cmd
        .arg("-include")
        .arg(&filter)
        .arg("-sarif")
        .arg("-output")
        .arg(&out_path);
    let jars = project_jars(job.project(), 500);
    if !jars.is_empty() {
        cmd = cmd.arg("-auxclasspath").arg(join_paths(&jars));
    }
    for r in &roots {
        cmd = cmd.arg(if r.is_empty() {
            ".".to_owned()
        } else {
            r.clone()
        });
    }
    let cmd = cmd.timeout(job.timeout);
    let out = exec::run(&cmd);
    let mut rec = run_record(&cmd, &out);
    if let Some(r) = out.failure_reason() {
        return Outcome::blocked(format!("spotbugs {r}"), vec![rec]);
    }
    let Ok(text) = fs::read_to_string(&out_path) else {
        return Outcome::blocked(
            format!(
                "spotbugs wrote no SARIF (exit {:?}): {}",
                out.exit,
                out.tail(3)
            ),
            vec![rec],
        );
    };
    attach_file(&mut rec, "output_file_json", &text);
    let Ok(log) = serde_json::from_str::<Value>(&text) else {
        return Outcome::blocked("spotbugs wrote malformed SARIF", vec![rec]);
    };
    let sources = fsx::files_with_ext(job.project(), &["java", "kt", "scala", "groovy"]);
    let (hits, unresolved) = parse_spotbugs(&log, &ids, &sources);
    let mut limitations = Vec::new();
    if unresolved > 0 {
        limitations.push(format!(
            "{unresolved} hit(s) in files not found among the project sources"
        ));
    }
    if jars.is_empty() {
        limitations.push("no auxiliary classpath: library calls are not modelled".to_owned());
    }
    Outcome::Ran(Box::new(Ran {
        version,
        command: cmd.display(),
        exit: out.exit,
        hits: dedupe(hits),
        load_errors: 0,
        limitations,
        note: None,
        raw: json!({ "runs": [rec], "class_roots": roots }),
    }))
}

// --------------------------------------------------------------------- detekt

/// The numeric ordering key of a version string.
fn version_key(v: &str) -> Vec<u64> {
    v.split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().unwrap_or(0))
        .collect()
}

/// The jar with the highest version among `candidates` named `<prefix>-<version>.jar`.
pub fn best_jar(candidates: Vec<PathBuf>, prefix: &str) -> Option<PathBuf> {
    candidates
        .into_iter()
        .filter_map(|p| {
            let n = p.file_name()?.to_str()?.to_owned();
            let v = n
                .strip_prefix(prefix)?
                .strip_prefix('-')?
                .strip_suffix(".jar")?;
            if v.contains("sources")
                || v.contains("javadoc")
                || !v.starts_with(|c: char| c.is_ascii_digit())
            {
                return None;
            }
            Some((version_key(v), p))
        })
        .max_by(|a, b| a.0.cmp(&b.0))
        .map(|(_, p)| p)
}

fn jars_under(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if depth > 0 {
                jars_under(&p, depth - 1, out);
            }
        } else if p.extension().is_some_and(|x| x == "jar") {
            out.push(p);
        }
    }
}

/// The Kotlin standard library and coroutines jars detekt's full analysis needs,
/// from the tools directory, the project, `~/.m2` and the Gradle cache.
pub fn kotlin_classpath(
    env: &ToolEnv,
    project_jars: &[PathBuf],
    home: Option<&Path>,
) -> (Option<PathBuf>, Option<PathBuf>) {
    let mut all: Vec<PathBuf> = project_jars.to_vec();
    if let Some(t) = &env.tools_dir {
        jars_under(&t.join("jars/kotlin-cp"), 0, &mut all);
    }
    if let Some(h) = home {
        jars_under(
            &h.join(".m2/repository/org/jetbrains/kotlin/kotlin-stdlib"),
            2,
            &mut all,
        );
        jars_under(
            &h.join(".m2/repository/org/jetbrains/kotlinx/kotlinx-coroutines-core-jvm"),
            2,
            &mut all,
        );
        jars_under(
            &h.join(".gradle/caches/modules-2/files-2.1/org.jetbrains.kotlin/kotlin-stdlib"),
            3,
            &mut all,
        );
        jars_under(
            &h.join(".gradle/caches/modules-2/files-2.1/org.jetbrains.kotlinx/kotlinx-coroutines-core-jvm"),
            3,
            &mut all,
        );
    }
    (
        best_jar(all.clone(), "kotlin-stdlib"),
        best_jar(all, "kotlinx-coroutines-core-jvm"),
    )
}

/// The detekt configuration enabling exactly `rules` of the coroutines rule set.
/// `v2` is false for detekt 1.x, which lacks `SuspendFunInFinallySection`; the second
/// value lists the rules left out.
pub fn detekt_config(rules: &[&str], v2: bool) -> (String, Vec<String>) {
    let mut y = String::from("coroutines:\n  active: true\n");
    let mut dropped = Vec::new();
    for r in rules {
        if !v2 && *r == "SuspendFunInFinallySection" {
            dropped.push((*r).to_owned());
            continue;
        }
        y.push_str(&format!("  {r}:\n    active: true\n"));
    }
    (y, dropped)
}

/// Parses detekt SARIF: `detekt.<ruleset>.<Rule>` -> `<Rule>`; the file is relative to
/// the analysed input.
pub fn parse_detekt(log: &Value, wanted: &[&str], paths: &PathMap) -> Vec<StaticHit> {
    sarif::results(log)
        .into_iter()
        .filter_map(|r| {
            let rule = r.rule.rsplit('.').next()?.to_owned();
            if !wanted.contains(&rule.as_str()) {
                return None;
            }
            let f = paths.rel(&r.uri)?;
            Some(hit("detekt", &rule, &f, r.line, &r.message))
        })
        .collect()
}

/// Runs detekt (2.x preferred) in full analysis mode when the Kotlin classpath is
/// available, light mode otherwise.
pub fn detekt(job: &Job) -> Outcome {
    let java = job.env.find_jdk_tool("java");
    let base: Cmd = if let Some(p) = job.env.find_program("detekt2") {
        Cmd::new(p)
    } else if let Some(dir) = job.env.jar_dir("detekt")
        && let Some(jar) = fsx::jars_in(&dir).into_iter().find(|j| {
            j.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("detekt-cli-") && n.ends_with("-all.jar"))
        })
        && let Some(java) = java.clone()
    {
        Cmd::new(java).arg("-jar").arg(jar)
    } else if let Some(p) = job.env.find_program("detekt") {
        Cmd::new(p)
    } else {
        return Outcome::missing("detekt not found on PATH or in the tools directory");
    };
    let base = base.env_remove("JAVA_TOOL_OPTIONS");
    let mut vcmd = base.clone();
    vcmd = vcmd.arg("--version");
    let version = version_of(vcmd);
    let v2 = java_major(&version).is_some_and(|m| m >= 2);
    if fsx::files_with_ext(job.project(), &["kt"]).is_empty() {
        return not_applicable(version, "no Kotlin sources");
    }
    let ids = job.rule_ids();
    let (cfg_text, dropped) = detekt_config(&ids, v2);
    let cfg = job.scratch.join("detekt.yml");
    let report = job.scratch.join("detekt.sarif");
    if let Err(e) = fsx::write(&cfg, &cfg_text) {
        return Outcome::blocked(format!("cannot write the config: {e}"), vec![]);
    }
    let _ = fs::remove_file(&report);
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from);
    let pjars = project_jars(job.project(), 500);
    let (stdlib, coroutines) = kotlin_classpath(job.env, &pjars, home.as_deref());
    let mut cmd = isolate(base, job)
        .cwd(job.project())
        .arg("--input")
        .arg(job.project())
        .arg("--config")
        .arg(&cfg)
        .arg("--excludes")
        .arg("**/.dla/**,**/build/**,**/target/**,**/node_modules/**,**/.git/**,**/out/**")
        .arg("--report")
        .arg(format!("sarif:{}", report.display()));
    let mut limitations = Vec::new();
    if let (Some(s), Some(c)) = (&stdlib, &coroutines) {
        let mut cp = vec![s.clone(), c.clone()];
        cp.extend(pjars.iter().cloned());
        let jvm = java
            .as_ref()
            .map(|j| version_of(Cmd::new(j).arg("-version").env_remove("JAVA_TOOL_OPTIONS")))
            .and_then(|v| java_major(&v))
            .filter(|m| (9..=24).contains(m))
            .map_or_else(|| "21".to_owned(), |m| m.to_string());
        cmd = cmd
            .args(["--analysis-mode", "full", "--jvm-target"])
            .arg(jvm)
            .arg("--classpath")
            .arg(join_paths(&cp));
    } else {
        limitations.push(
            "kotlin-stdlib and kotlinx-coroutines jars not found: light analysis mode, only GlobalCoroutineUsage can fire"
                .to_owned(),
        );
    }
    if !dropped.is_empty() {
        limitations.push(format!(
            "detekt {version} lacks {} (detekt 2.x only)",
            dropped.join(", ")
        ));
    }
    let cmd = cmd.timeout(job.timeout);
    let out = exec::run(&cmd);
    let mut rec = run_record(&cmd, &out);
    if let Some(r) = out.failure_reason() {
        return Outcome::blocked(format!("detekt {r}"), vec![rec]);
    }
    let Ok(text) = fs::read_to_string(&report) else {
        return Outcome::blocked(
            format!(
                "detekt wrote no SARIF (exit {:?}): {}",
                out.exit,
                out.tail(3)
            ),
            vec![rec],
        );
    };
    attach_file(&mut rec, "output_file_json", &text);
    let Ok(log) = serde_json::from_str::<Value>(&text) else {
        return Outcome::blocked("detekt wrote malformed SARIF", vec![rec]);
    };
    let hits = parse_detekt(&log, &ids, &job.paths());
    Outcome::Ran(Box::new(Ran {
        version,
        command: cmd.display(),
        exit: out.exit,
        hits: dedupe(hits),
        load_errors: 0,
        limitations,
        note: None,
        raw: json!({ "runs": [rec] }),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(name: &str) -> String {
        fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/data")
                .join(name),
        )
        .unwrap_or_else(|e| panic!("{name}: {e}"))
    }

    #[test]
    fn java_major_versions() {
        assert_eq!(java_major("21.0.12"), Some(21));
        assert_eq!(java_major("1.8.0_292"), Some(8));
        assert_eq!(java_major("2.0.0-alpha.6"), Some(2));
        assert_eq!(java_major("unknown"), None);
    }

    #[test]
    fn errorprone_command_enables_only_the_listed_checks() {
        let a = errorprone_args(
            Path::new("/t/classes"),
            &[PathBuf::from("/ep/a.jar"), PathBuf::from("/ep/b.jar")],
            &["StreamResourceLeak", "MustBeClosedChecker"],
            &[],
            Path::new("/t/src.args"),
        );
        let s: Vec<String> = a.iter().map(|x| x.to_string_lossy().into_owned()).collect();
        assert!(s.contains(&"-XDaddTypeAnnotationsToSymbol=true".to_owned()));
        assert!(s.contains(&"-proc:none".to_owned()));
        assert!(s.iter().any(|x| x == "-Xplugin:ErrorProne -XepDisableAllChecks -Xep:StreamResourceLeak:WARN -Xep:MustBeClosedChecker:WARN"));
        assert!(s.contains(
            &"-J--add-opens=jdk.compiler/com.sun.tools.javac.comp=ALL-UNNAMED".to_owned()
        ));
        assert_eq!(s.last().unwrap(), "@/t/src.args");
        assert!(!s.contains(&"-cp".to_owned()));
        let pp = s.iter().position(|x| x == "-processorpath").unwrap();
        assert!(s[pp + 1].contains("a.jar"));
    }

    #[test]
    fn errorprone_golden() {
        let text = data("errorprone.txt");
        let root = text
            .lines()
            .next()
            .unwrap()
            .rsplit_once('/')
            .unwrap()
            .0
            .to_owned();
        let p = parse_errorprone(
            &text,
            &["StreamResourceLeak", "ThreadLocalUsage", "ClassCanBeStatic"],
            &PathMap::new(Path::new(&root)),
        );
        let got: Vec<_> = p
            .hits
            .iter()
            .map(|h| format!("{}@{}", h.rule, h.site()))
            .collect();
        assert_eq!(
            got,
            [
                "ThreadLocalUsage@P.java:3",
                "ClassCanBeStatic@P.java:4",
                "StreamResourceLeak@P.java:5",
                "ClassCanBeStatic@Q.java:6"
            ]
        );
        assert_eq!(p.errors, 0);
        let only = parse_errorprone(
            &text,
            &["StreamResourceLeak"],
            &PathMap::new(Path::new(&root)),
        );
        assert_eq!(only.hits.len(), 1);
    }

    #[test]
    fn errorprone_compile_errors_are_counted() {
        let t = "/p/A.java:3: error: cannot find symbol\n  Foo f;\n  ^\n/p/A.java:9: warning: [StreamResourceLeak] msg\n";
        let p = parse_errorprone(t, &["StreamResourceLeak"], &PathMap::new(Path::new("/p")));
        assert_eq!(p.errors, 1);
        assert_eq!(p.hits.len(), 1);
        assert!(p.first_error.unwrap().contains("cannot find symbol"));
    }

    #[test]
    fn class_roots_are_the_topmost_class_directories() {
        let files: Vec<String> = [
            "target/classes/shop/A.class",
            "target/classes/shop/util/B.class",
            "target/classes/other/C.class",
            "Loose.class",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        assert_eq!(class_roots(&files), vec![""]);
        let files: Vec<String> = [
            "target/classes/shop/A.class",
            "target/classes/shop/util/B.class",
            "build/classes/x/C.class",
        ]
        .iter()
        .map(ToString::to_string)
        .collect();
        assert_eq!(
            class_roots(&files),
            vec!["build/classes/x", "target/classes/shop"]
        );
    }

    #[test]
    fn spotbugs_filter_lists_patterns() {
        let f = spotbugs_filter(&["OS_OPEN_STREAM", "OBL_UNSATISFIED_OBLIGATION"]);
        assert!(f.contains("pattern=\"OS_OPEN_STREAM,OBL_UNSATISFIED_OBLIGATION\""));
    }

    #[test]
    fn spotbugs_golden_resolves_sources_by_suffix() {
        let log: Value = serde_json::from_str(&data("spotbugs.sarif")).unwrap();
        let sources = vec!["P.java".to_owned(), "Q.java".to_owned()];
        let wanted = [
            "OBL_UNSATISFIED_OBLIGATION",
            "OBL_UNSATISFIED_OBLIGATION_EXCEPTION_EDGE",
            "OS_OPEN_STREAM",
            "OS_OPEN_STREAM_EXCEPTION_PATH",
        ];
        let (hits, unresolved) = parse_spotbugs(&log, &wanted, &sources);
        assert_eq!(unresolved, 0);
        let got: Vec<_> = dedupe(hits)
            .iter()
            .map(|h| format!("{}@{}", h.rule, h.site()))
            .collect();
        assert!(
            got.contains(&"OS_OPEN_STREAM@P.java:6".to_owned()),
            "{got:?}"
        );
        assert!(got.contains(&"OBL_UNSATISFIED_OBLIGATION_EXCEPTION_EDGE@Q.java:5".to_owned()));
        let (hits, unresolved) = parse_spotbugs(&log, &wanted, &[]);
        assert!(hits.is_empty());
        assert!(unresolved > 0);
    }

    #[test]
    fn detekt_config_respects_the_version() {
        let (y, dropped) = detekt_config(
            &["GlobalCoroutineUsage", "SuspendFunInFinallySection"],
            true,
        );
        assert!(y.contains("SuspendFunInFinallySection:\n    active: true"));
        assert!(dropped.is_empty());
        let (y, dropped) = detekt_config(
            &["GlobalCoroutineUsage", "SuspendFunInFinallySection"],
            false,
        );
        assert!(!y.contains("SuspendFunInFinallySection"));
        assert_eq!(dropped, ["SuspendFunInFinallySection"]);
    }

    #[test]
    fn detekt_golden() {
        let log: Value = serde_json::from_str(&data("detekt.sarif")).unwrap();
        let hits = parse_detekt(
            &log,
            &[
                "GlobalCoroutineUsage",
                "SuspendFunSwallowedCancellation",
                "SuspendFunInFinallySection",
            ],
            &PathMap::new(Path::new("/proj")),
        );
        let got: Vec<_> = hits
            .iter()
            .map(|h| format!("{}@{}", h.rule, h.site()))
            .collect();
        assert_eq!(
            got,
            [
                "GlobalCoroutineUsage@A.kt:2",
                "SuspendFunSwallowedCancellation@A.kt:3",
                "SuspendFunInFinallySection@A.kt:3"
            ]
        );
    }

    #[test]
    fn kotlin_classpath_picks_the_newest_jars() {
        let h = tempfile::tempdir().unwrap();
        for p in [
            ".m2/repository/org/jetbrains/kotlin/kotlin-stdlib/1.9.23/kotlin-stdlib-1.9.23.jar",
            ".m2/repository/org/jetbrains/kotlin/kotlin-stdlib/2.0.21/kotlin-stdlib-2.0.21.jar",
            ".m2/repository/org/jetbrains/kotlin/kotlin-stdlib/2.0.21/kotlin-stdlib-2.0.21-sources.jar",
            ".m2/repository/org/jetbrains/kotlinx/kotlinx-coroutines-core-jvm/1.6.4/kotlinx-coroutines-core-jvm-1.6.4.jar",
        ] {
            let f = h.path().join(p);
            fs::create_dir_all(f.parent().unwrap()).unwrap();
            fs::write(f, "").unwrap();
        }
        let env = ToolEnv::default();
        let (s, c) = kotlin_classpath(&env, &[], Some(h.path()));
        assert!(s.unwrap().ends_with("kotlin-stdlib-2.0.21.jar"));
        assert!(
            c.unwrap()
                .ends_with("kotlinx-coroutines-core-jvm-1.6.4.jar")
        );
        let (s, c) = kotlin_classpath(&env, &[], None);
        assert!(s.is_none() && c.is_none());
    }
}
