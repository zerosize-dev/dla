//! `dla probe --build`: the per-runtime build commands.

use std::{
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
        Runtime::Go => Ok(Plan::new("go", &["build", "./..."])),
        Runtime::Rust => Ok(Plan::new("cargo", &["build"])),
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
    let t = Instant::now();
    let o = exec::run(Path::new(&plan.program), &args, Some(&cwd), &env, timeout);
    let duration_ms = u64::try_from(t.elapsed().as_millis()).unwrap_or(u64::MAX);
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
    BuildProbe {
        runtime,
        command: Some(command),
        status,
        exit_code: o.code,
        duration_ms,
        log_tail: (!tail.is_empty()).then_some(tail),
    }
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
        let (_d, t, det) = proj(&[("pom.xml", ""), ("A.java", "")]);
        assert_eq!(
            plan(&t, &det[0], scratch).unwrap().line(),
            "mvn -q -o package -DskipTests"
        );
        let (_d, t, det) = proj(&[("build.sh", "#!/bin/sh\n"), ("A.java", ""), ("pom.xml", "")]);
        assert_eq!(plan(&t, &det[0], scratch).unwrap().line(), "sh ./build.sh");
        let (_d, t, det) = proj(&[("go.mod", "go 1.22\n")]);
        assert_eq!(plan(&t, &det[0], scratch).unwrap().line(), "go build ./...");
        let (_d, t, det) = proj(&[("Cargo.toml", "")]);
        assert_eq!(plan(&t, &det[0], scratch).unwrap().line(), "cargo build");
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
