//! `dla probe --build`: the per-runtime build commands.

use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use dla_core::{
    probe::{BuildProbe, BuildStatus},
    runtime::Runtime,
};

use crate::{exec, scan::Detected, scan::Tree};

/// A build command: program, arguments, extra environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    /// Program (looked up on PATH unless it contains a slash).
    pub program: String,
    /// Arguments.
    pub args: Vec<String>,
    /// Extra environment.
    pub env: Vec<(String, String)>,
}

impl Plan {
    fn new(program: &str, args: &[&str]) -> Self {
        Self {
            program: program.to_owned(),
            args: args.iter().map(|s| (*s).to_owned()).collect(),
            env: Vec::new(),
        }
    }

    /// The command line as one string.
    pub fn line(&self) -> String {
        std::iter::once(self.program.as_str())
            .chain(self.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

fn sub(scratch: &Path, name: &str) -> String {
    scratch.join(name).to_string_lossy().into_owned()
}

fn exists(tree: &Tree, root: &str, name: &str) -> bool {
    let rel = if root.is_empty() {
        name.to_owned()
    } else {
        format!("{root}/{name}")
    };
    tree.files.contains(&rel)
}

/// Picks the build command for a detected runtime, or the reason there is none.
pub fn plan(tree: &Tree, d: &Detected, scratch: &Path) -> Result<Plan, String> {
    let root = d.root.as_str();
    let has = |n: &str| exists(tree, root, n);
    match d.probe.runtime {
        Runtime::Jvm => {
            if has("build.sh") {
                Ok(Plan::new("sh", &["./build.sh"]))
            } else if has("pom.xml") {
                Ok(Plan::new("mvn", &["-q", "-o", "package", "-DskipTests"]))
            } else if has("gradlew") {
                Ok(Plan::new(
                    "sh",
                    &["./gradlew", "--offline", "-q", "assemble"],
                ))
            } else if has("build.gradle") || has("build.gradle.kts") {
                Ok(Plan::new("gradle", &["--offline", "-q", "assemble"]))
            } else {
                Err("no build.sh, pom.xml or Gradle build file".into())
            }
        }
        Runtime::Go => {
            let mut p = Plan::new("go", &["build", "./..."]);
            // The build cache goes to scratch; `-modcacherw` keeps any module cache
            // the build fills removable. The module cache itself stays where
            // offline builds find their dependencies.
            p.env.push(("GOCACHE".into(), sub(scratch, "gocache")));
            p.env
                .push(("GOFLAGS".into(), "-buildvcs=false -modcacherw".into()));
            Ok(p)
        }
        Runtime::Rust => {
            let mut p = Plan::new("cargo", &["build"]);
            // Build output goes to scratch, not into the project's `target/`.
            p.env
                .push(("CARGO_TARGET_DIR".into(), sub(scratch, "cargo-target")));
            Ok(p)
        }
        Runtime::Node => {
            let pkg = if root.is_empty() {
                "package.json".to_owned()
            } else {
                format!("{root}/package.json")
            };
            let defined = tree
                .read(&pkg)
                .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
                .is_some_and(|v| v.pointer("/scripts/build").is_some());
            if defined {
                Ok(Plan::new("npm", &["run", "build"]))
            } else {
                Err("package.json defines no build script".into())
            }
        }
        Runtime::Python => {
            let mut p = Plan::new(
                "python3",
                &[
                    "-m",
                    "compileall",
                    "-q",
                    "-x",
                    r"(^|/)(\.git|\.dla|venv|\.venv|node_modules|build|dist|target|out)(/|$)",
                    ".",
                ],
            );
            // Byte-code goes to scratch, not into the project tree.
            p.env.push((
                "PYTHONPYCACHEPREFIX".into(),
                scratch.join("pycache").to_string_lossy().into_owned(),
            ));
            Ok(p)
        }
        Runtime::Native => {
            if has("Makefile") || has("makefile") || has("GNUmakefile") {
                Ok(Plan::new("make", &[]))
            } else if has("CMakeLists.txt") {
                Ok(Plan::new(
                    "sh",
                    &["-c", "cmake -S . -B build && cmake --build build"],
                ))
            } else {
                Err("no Makefile or CMakeLists.txt".into())
            }
        }
    }
}

/// Runs the build of one runtime.
pub fn run_one(tree: &Tree, d: &Detected, scratch: &Path, timeout: Duration) -> BuildProbe {
    let runtime = d.probe.runtime;
    let plan = match plan(tree, d, scratch) {
        Ok(p) => p,
        Err(why) => {
            return BuildProbe {
                runtime,
                command: None,
                status: BuildStatus::Skipped,
                exit_code: None,
                duration_ms: 0,
                log_tail: Some(why),
            };
        }
    };
    let cwd: PathBuf = if d.root.is_empty() {
        tree.root.clone()
    } else {
        tree.root.join(&d.root)
    };
    let mut command = plan.line();
    if !d.root.is_empty() {
        command.push_str(&format!(" (in {}/)", d.root));
    }
    let args: Vec<&str> = plan.args.iter().map(String::as_str).collect();
    let env: Vec<(&str, &str)> = plan
        .env
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();
    let before = snapshot(&cwd);
    let t = Instant::now();
    let o = exec::run(Path::new(&plan.program), &args, Some(&cwd), &env, timeout);
    let duration_ms = u64::try_from(t.elapsed().as_millis()).unwrap_or(u64::MAX);
    let wrote = new_paths(&cwd, &before, d.probe.runtime);
    let (status, tail) = if let Some(e) = &o.spawn_error {
        (
            BuildStatus::Failed,
            format!("could not start `{}`: {e}", plan.program),
        )
    } else if o.timed_out {
        (
            BuildStatus::Timeout,
            format!(
                "killed after {}s\n{}",
                timeout.as_secs(),
                exec::tail_lines(&o.text, 15)
            ),
        )
    } else if o.success {
        (BuildStatus::Ok, exec::tail_lines(&o.text, 5))
    } else {
        (BuildStatus::Failed, exec::tail_lines(&o.text, 20))
    };
    let mut tail = tail;
    if !wrote.is_empty() {
        if !tail.is_empty() {
            tail.push('\n');
        }
        tail.push_str(&wrote);
    }
    BuildProbe {
        runtime,
        command: Some(command),
        status,
        exit_code: o.code,
        duration_ms,
        log_tail: (!tail.is_empty()).then_some(tail),
    }
}

/// Every path under `root` (relative, `/`-separated; directories included), without
/// descending into `.git`, `.dla` or `node_modules`. Capped.
fn snapshot(root: &Path) -> HashSet<String> {
    let mut out = HashSet::new();
    let mut stack = vec![(root.to_path_buf(), String::new())];
    while let Some((dir, rel)) = stack.pop() {
        let Ok(rd) = std::fs::read_dir(&dir) else {
            continue;
        };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let r = if rel.is_empty() {
                name.clone()
            } else {
                format!("{rel}/{name}")
            };
            let is_dir = e.file_type().is_ok_and(|t| t.is_dir());
            if is_dir && !matches!(name.as_str(), ".git" | ".dla" | "node_modules") {
                stack.push((e.path(), r.clone()));
            }
            out.insert(r);
            if out.len() >= 200_000 {
                return out;
            }
        }
    }
    out
}

/// What a build created in the project: the topmost new paths. A `Cargo.lock` that
/// the Rust build generated is removed again (it is not part of the project); the
/// rest is only reported.
fn new_paths(root: &Path, before: &HashSet<String>, runtime: Runtime) -> String {
    let after = snapshot(root);
    let mut fresh: Vec<&String> = after
        .iter()
        .filter(|p| !before.contains(*p))
        .filter(|p| {
            p.rsplit_once('/')
                .is_none_or(|(parent, _)| before.contains(parent))
        })
        .collect();
    fresh.sort();
    let mut notes = Vec::new();
    let mut kept = Vec::new();
    for p in fresh {
        if runtime == Runtime::Rust
            && p.ends_with("Cargo.lock")
            && std::fs::remove_file(root.join(p)).is_ok()
        {
            notes.push(format!("removed the generated {p}"));
        } else {
            kept.push(p.as_str());
        }
    }
    if !kept.is_empty() {
        let shown: Vec<&str> = kept.iter().copied().take(10).collect();
        let more = kept.len().saturating_sub(shown.len());
        let mut s = format!("the build wrote into the project: {}", shown.join(", "));
        if more > 0 {
            s.push_str(&format!(" (+{more} more)"));
        }
        notes.push(s);
    }
    notes.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::detect;
    use std::fs;

    fn proj(files: &[(&str, &str)]) -> (tempfile::TempDir, Tree, Vec<Detected>) {
        let d = tempfile::tempdir().unwrap();
        for (p, c) in files {
            let path = d.path().join(p);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, c).unwrap();
        }
        let t = Tree::walk(d.path());
        let det = detect(&t);
        (d, t, det)
    }

    #[test]
    fn plans_follow_the_build_files() {
        let scratch = Path::new("/tmp/s");
        // Environment values are native paths handed to the build tool, so the
        // expected values use the platform separator.
        let under = |sub: &str| scratch.join(sub).display().to_string();
        let (_d, t, det) = proj(&[("pom.xml", ""), ("A.java", "")]);
        assert_eq!(
            plan(&t, &det[0], scratch).unwrap().line(),
            "mvn -q -o package -DskipTests"
        );
        let (_d, t, det) = proj(&[("build.sh", "#!/bin/sh\n"), ("A.java", ""), ("pom.xml", "")]);
        assert_eq!(plan(&t, &det[0], scratch).unwrap().line(), "sh ./build.sh");
        let (_d, t, det) = proj(&[("go.mod", "go 1.22\n")]);
        let p = plan(&t, &det[0], scratch).unwrap();
        assert_eq!(p.line(), "go build ./...");
        assert!(p.env.contains(&("GOCACHE".to_owned(), under("gocache"))));
        assert!(
            p.env
                .iter()
                .any(|(k, v)| k == "GOFLAGS" && v.contains("-modcacherw"))
        );
        let (_d, t, det) = proj(&[("Cargo.toml", "")]);
        let p = plan(&t, &det[0], scratch).unwrap();
        assert_eq!(p.line(), "cargo build");
        assert!(
            p.env
                .contains(&("CARGO_TARGET_DIR".to_owned(), under("cargo-target")))
        );
        let (_d, t, det) = proj(&[("Makefile", "all:\n"), ("a.c", "")]);
        assert_eq!(plan(&t, &det[0], scratch).unwrap().line(), "make");
        let (_d, t, det) = proj(&[("package.json", r#"{"scripts":{"build":"tsc"}}"#)]);
        assert_eq!(plan(&t, &det[0], scratch).unwrap().line(), "npm run build");
        let (_d, t, det) = proj(&[("package.json", r#"{"scripts":{"test":"x"}}"#)]);
        assert!(plan(&t, &det[0], scratch).is_err());
        let (_d, t, det) = proj(&[("a.py", "")]);
        let p = plan(&t, &det[0], scratch).unwrap();
        assert!(p.line().starts_with("python3 -m compileall"));
        assert!(p.env[0].1.starts_with("/tmp/s"));
    }

    #[test]
    fn a_nested_build_file_sets_the_working_directory() {
        let (_d, t, det) = proj(&[("svc/go.mod", "go 1.22\n"), ("svc/m.go", "package main\n")]);
        assert_eq!(det[0].root, "svc");
        let b = run_one(&t, &det[0], Path::new("/tmp"), Duration::from_secs(60));
        assert!(b.command.unwrap().ends_with("(in svc/)"));
    }

    #[cfg(unix)]
    #[test]
    fn files_a_build_writes_into_the_project_are_recorded() {
        let (d, t, det) = proj(&[
            (
                "build.sh",
                "mkdir -p target/classes && echo x > target/classes/A.class\necho y > gen.txt\n",
            ),
            ("A.java", ""),
        ]);
        let b = run_one(&t, &det[0], Path::new("/tmp"), Duration::from_secs(30));
        let tail = b.log_tail.unwrap();
        assert!(
            tail.contains("the build wrote into the project: gen.txt, target"),
            "{tail}"
        );
        assert!(
            d.path().join("target/classes/A.class").is_file(),
            "JVM outputs stay"
        );
    }

    #[test]
    fn a_generated_cargo_lock_is_removed_and_recorded() {
        let d = tempfile::tempdir().unwrap();
        let before = snapshot(d.path());
        std::fs::write(d.path().join("Cargo.lock"), "x").unwrap();
        let note = new_paths(d.path(), &before, Runtime::Rust);
        assert_eq!(note, "removed the generated Cargo.lock");
        assert!(!d.path().join("Cargo.lock").exists());
        // A lock file that existed before is untouched.
        std::fs::write(d.path().join("Cargo.lock"), "x").unwrap();
        let before = snapshot(d.path());
        assert_eq!(new_paths(d.path(), &before, Runtime::Rust), "");
        assert!(d.path().join("Cargo.lock").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn a_failing_build_is_recorded_with_its_log_tail() {
        let (_d, t, det) = proj(&[
            (
                "build.sh",
                "echo compiling\necho 'boom: cannot resolve dependency' 1>&2\nexit 7\n",
            ),
            ("A.java", ""),
        ]);
        let b = run_one(&t, &det[0], Path::new("/tmp"), Duration::from_secs(30));
        assert_eq!(b.status, BuildStatus::Failed);
        assert_eq!(b.exit_code, Some(7));
        assert!(b.log_tail.unwrap().contains("boom"));
    }

    #[cfg(unix)]
    #[test]
    fn a_passing_build_and_a_timeout() {
        let (_d, t, det) = proj(&[("build.sh", "echo fine\n"), ("A.java", "")]);
        let b = run_one(&t, &det[0], Path::new("/tmp"), Duration::from_secs(30));
        assert_eq!((b.status, b.exit_code), (BuildStatus::Ok, Some(0)));
        let (_d, t, det) = proj(&[("build.sh", "sleep 30\n"), ("A.java", "")]);
        let b = run_one(&t, &det[0], Path::new("/tmp"), Duration::from_millis(300));
        assert_eq!(b.status, BuildStatus::Timeout);
    }

    #[test]
    fn no_build_command_is_skipped_with_the_reason() {
        let (_d, t, det) = proj(&[("A.java", "")]);
        let b = run_one(&t, &det[0], Path::new("/tmp"), Duration::from_secs(5));
        assert_eq!(b.status, BuildStatus::Skipped);
        assert!(b.command.is_none() && b.log_tail.is_some());
    }
}
