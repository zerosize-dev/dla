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

/// The Python launcher: Windows installs `python`, not `python3`.
pub(crate) fn python_program() -> &'static str {
    if cfg!(windows) { "python" } else { "python3" }
}

/// The program to start for `name`. Windows only starts `.exe` files by bare name, so
/// `npm`, `mvn` and `gradle` (`.cmd` or `.bat` launchers) are looked up on `PATH` with
/// their extensions; elsewhere the name is left to the OS.
fn program_path(name: &str) -> PathBuf {
    if cfg!(windows)
        && let Some(f) = crate::registry::locate(name, None, std::env::var_os("PATH").as_deref())
    {
        return f.path;
    }
    PathBuf::from(name)
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
                python_program(),
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
    // A native build links objects and executables in the tree it runs in, and `make`
    // finds them up to date in every copy made afterwards (their debug paths would point
    // at this tree). Like Go, Rust and Python, it builds in a copy under scratch.
    let mut copy_made: Option<PathBuf> = None;
    let build_root: PathBuf = if runtime == Runtime::Native {
        match copy_for_build(&tree.root, scratch, &d.root) {
            Ok(c) => {
                copy_made = Some(c.clone());
                c
            }
            Err(e) => {
                return BuildProbe {
                    runtime,
                    command: None,
                    status: BuildStatus::Skipped,
                    exit_code: None,
                    duration_ms: 0,
                    log_tail: Some(format!(
                        "could not copy the project to a scratch directory to build it there: {e}"
                    )),
                };
            }
        }
    } else {
        tree.root.clone()
    };
    let cwd: PathBuf = if d.root.is_empty() {
        build_root
    } else {
        build_root.join(&d.root)
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
    let o = exec::run(
        &program_path(&plan.program),
        &args,
        Some(&cwd),
        &env,
        timeout,
    );
    let duration_ms = u64::try_from(t.elapsed().as_millis()).unwrap_or(u64::MAX);
    // Nothing later reads the copy (its output is not evidence): remove it, so a large
    // tree is not left behind under the run directory.
    if let Some(c) = &copy_made {
        let _ = std::fs::remove_dir_all(c);
    }
    // A native build wrote into its scratch copy, not into the project.
    let wrote = if runtime == Runtime::Native {
        String::new()
    } else {
        new_paths(&cwd, &before, d.probe.runtime)
    };
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

/// Copies the project at `root` into a new directory under `scratch` for a build there and
/// returns it. In a git work tree the copy takes the files the harness copy takes
/// (`git ls-files --cached --others --exclude-standard`, see
/// [`crate::project_copy::git_files`]) with no directory skips, a tracked `build/common.mk`
/// is copied and an ignored `native/build/` is not, and leaves out what that copy leaves out
/// (object files, untracked libraries, see [`crate::project_copy::is_dropped_build_output`];
/// links that dangle or leave the project). An empty listing counts as no git work tree
/// (an enclosing repository ignoring the project). Outside git it walks the tree and leaves
/// out `.git`, `.dla`, `node_modules`, `__pycache__` and, at the project root and at the
/// build root `build_root` (relative, empty for the root), `target`, `venv`, `.venv`,
/// `build`, `out` and `dist`. Modification times and permissions are kept, so `make`
/// decides what to rebuild as it would in the project; an absolute symlink into the
/// project is re-pointed into the copy.
fn copy_for_build(root: &Path, scratch: &Path, build_root: &str) -> std::io::Result<PathBuf> {
    static SEQ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let dest = scratch.join(format!("native-build-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dest)?;
    // An enclosing repository that ignores the project lists nothing: that is no listing.
    if let Some(files) = crate::project_copy::git_files(root) {
        let tracked = crate::project_copy::git_tracked(root);
        for rel in files {
            if rel
                .components()
                .next()
                .is_some_and(|c| c.as_os_str() == ".dla")
                || crate::project_copy::is_dropped_build_output(&rel, true, &tracked)
            {
                continue;
            }
            copy_entry(root, &dest, &rel)?;
        }
        return Ok(dest);
    }
    let no_tracked = std::collections::HashSet::new();
    let build_dir = (!build_root.is_empty()).then(|| root.join(build_root));
    let mut stack = vec![PathBuf::new()];
    while let Some(rel) = stack.pop() {
        let from = root.join(&rel);
        let at_a_root = from == root || build_dir.as_deref() == Some(from.as_path());
        for e in std::fs::read_dir(&from)?.flatten() {
            let name = e.file_name();
            let child = rel.join(&name);
            let ft = e.file_type()?;
            if ft.is_dir() {
                let skip = match name.to_string_lossy().as_ref() {
                    ".git" | ".dla" | "node_modules" | "__pycache__" => true,
                    // Another toolchain's output at the project root or the build root: a
                    // developer's earlier `build/CMakeCache.txt` names another directory
                    // and makes `cmake -B build` refuse in the copy.
                    "target" | "venv" | ".venv" | "build" | "out" | "dist" => at_a_root,
                    _ => false,
                };
                if !skip {
                    std::fs::create_dir_all(dest.join(&child))?;
                    stack.push(child);
                }
            } else if (ft.is_file() || ft.is_symlink())
                && !crate::project_copy::is_dropped_build_output(&child, false, &no_tracked)
            {
                copy_entry(root, &dest, &child)?;
            }
        }
    }
    Ok(dest)
}

/// Copies the file or symlink `rel` of the project `root` into `dest`. A path listed but
/// missing (tracked and deleted) is skipped.
fn copy_entry(root: &Path, dest: &Path, rel: &Path) -> std::io::Result<()> {
    let (src, dst) = (root.join(rel), dest.join(rel));
    let Ok(meta) = std::fs::symlink_metadata(&src) else {
        return Ok(());
    };
    if let Some(parent) = dst.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if meta.is_symlink() {
        #[cfg(unix)]
        {
            // A link that dangles or leaves the project is not copied, as in the harness.
            if let Ok(link) = crate::project_copy::link_for_copy(root, dest, &src) {
                let _ = std::os::unix::fs::symlink(link, &dst);
            }
        }
        Ok(())
    } else if meta.is_file() {
        copy_file_keeping_time(&src, &dst)
    } else {
        Ok(())
    }
}

/// Copies one file with its modification time and permissions. The content goes through a
/// handle opened for writing, the time is set on that handle, and the permissions are copied
/// last, so a read-only source keeps its time too.
fn copy_file_keeping_time(from: &Path, to: &Path) -> std::io::Result<()> {
    let meta = std::fs::metadata(from)?;
    let mut src = std::fs::File::open(from)?;
    let mut dst = std::fs::File::create(to)?;
    std::io::copy(&mut src, &mut dst)?;
    if let Ok(t) = meta.modified() {
        // Best effort: a copy with a fresh time is still a copy.
        let _ = dst.set_modified(t);
    }
    drop(dst);
    std::fs::set_permissions(to, meta.permissions())
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
        assert!(
            p.line()
                .starts_with(&format!("{} -m compileall", python_program()))
        );
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

    /// True when `make` and `cc` run here.
    #[cfg(unix)]
    fn have_make_and_cc() -> bool {
        ["make", "cc"].iter().all(|t| {
            std::process::Command::new(t)
                .arg("--version")
                .output()
                .is_ok_and(|o| o.status.success())
        })
    }

    #[cfg(unix)]
    #[test]
    fn a_native_build_runs_in_a_scratch_copy_and_leaves_the_project_untouched() {
        if !have_make_and_cc() {
            return;
        }
        // An executable linked straight from sources: left in the project, `make` would
        // find it up to date in every later copy, with debug paths into this tree.
        let (d, t, det) = proj(&[
            (
                "Makefile",
                "app: main.c lease.h\n\tcc -g -o app main.c\n\ttest -x app\n",
            ),
            (
                "main.c",
                "#include \"lease.h\"\nint main(void) { return LEASE; }\n",
            ),
            ("lease.h", "#define LEASE 0\n"),
        ]);
        let scratch = tempfile::tempdir().unwrap();
        let b = run_one(&t, &det[0], scratch.path(), Duration::from_secs(60));
        assert_eq!(b.status, BuildStatus::Ok, "{:?}", b.log_tail);
        assert!(
            !d.path().join("app").exists(),
            "the build linked its executable in the project"
        );
        let tail = b.log_tail.unwrap_or_default();
        assert!(!tail.contains("wrote into the project"), "{tail}");
        // The copy is removed once the build is over.
        let left = std::fs::read_dir(scratch.path()).unwrap().flatten().count();
        assert_eq!(left, 0, "the build copy was left under scratch");
    }

    #[test]
    fn outside_git_the_build_copy_leaves_out_other_output_and_earlier_build_directories() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        for f in [
            "Makefile",
            "src/main.c",
            // Kept below the root: only the root is another toolchain's output.
            "src/build/gen.c",
            "src/target/x.c",
            // Left out.
            "build/CMakeCache.txt",
            "out/a.o",
            "dist/a.tar",
            "target/debug/x",
            "venv/bin/python",
            ".venv/bin/python",
            "pkg/__pycache__/m.pyc",
            "node_modules/m/i.js",
            ".git/HEAD",
        ] {
            let p = src.path().join(f);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, "x\n").unwrap();
        }
        let copy = copy_for_build(src.path(), dst.path(), "").unwrap();
        let mut got: Vec<String> = Vec::new();
        let mut stack = vec![copy.clone()];
        while let Some(d) = stack.pop() {
            for e in std::fs::read_dir(d).unwrap().flatten() {
                if e.file_type().unwrap().is_dir() {
                    stack.push(e.path());
                } else {
                    let rel = e.path().strip_prefix(&copy).unwrap().to_path_buf();
                    got.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        got.sort();
        assert_eq!(
            got,
            [
                "Makefile",
                "src/build/gen.c",
                "src/main.c",
                "src/target/x.c"
            ]
        );
    }

    /// Every file under `dir`, relative and sorted.
    fn files_under(dir: &Path) -> Vec<String> {
        let mut got = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(d) = stack.pop() {
            for e in fs::read_dir(d).unwrap().flatten() {
                if e.file_type().unwrap().is_dir() {
                    stack.push(e.path());
                } else {
                    let rel = e.path().strip_prefix(dir).unwrap().to_path_buf();
                    got.push(rel.to_string_lossy().replace('\\', "/"));
                }
            }
        }
        got.sort();
        got
    }

    /// Runs `tool args` in `dir`; false when the tool is missing or fails.
    #[cfg(unix)]
    fn have_run(dir: &Path, tool: &str, args: &[&str]) -> bool {
        std::process::Command::new(tool)
            .current_dir(dir)
            .args(args)
            .output()
            .is_ok_and(|o| o.status.success())
    }

    fn write_files(root: &Path, files: &[(&str, &str)]) {
        for (p, c) in files {
            let path = root.join(p);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, c).unwrap();
        }
    }

    #[test]
    fn outside_git_the_build_root_skips_its_own_output_directories() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        write_files(
            src.path(),
            &[
                ("native/CMakeLists.txt", "x\n"),
                ("native/src/main.c", "x\n"),
                // A developer's earlier cmake directory under the build root.
                ("native/build/CMakeCache.txt", "x\n"),
                ("native/out/a.o", "x\n"),
                // Deeper than the build root: source.
                ("native/src/build/gen.c", "x\n"),
                ("other/build/keep.c", "x\n"),
            ],
        );
        let copy = copy_for_build(src.path(), dst.path(), "native").unwrap();
        assert_eq!(
            files_under(&copy),
            [
                "native/CMakeLists.txt",
                "native/src/build/gen.c",
                "native/src/main.c",
                "other/build/keep.c",
            ]
        );
    }

    #[cfg(unix)]
    #[test]
    fn the_build_copy_repoints_an_absolute_symlink_into_the_copy() {
        let dst = tempfile::tempdir().unwrap();
        for in_git in [false, true] {
            let src = tempfile::tempdir().unwrap();
            if in_git && !have_run(src.path(), "git", &["init", "-q"]) {
                continue;
            }
            write_files(src.path(), &[("real/a.h", "x\n")]);
            std::os::unix::fs::symlink(src.path().join("real"), src.path().join("inc")).unwrap();
            std::os::unix::fs::symlink("real/a.h", src.path().join("rel.h")).unwrap();
            let copy = copy_for_build(src.path(), dst.path(), "").unwrap();
            assert_eq!(
                fs::read_link(copy.join("inc")).unwrap(),
                copy.join("real"),
                "absolute link, in_git={in_git}"
            );
            assert_eq!(
                fs::read_link(copy.join("rel.h")).unwrap(),
                Path::new("real/a.h"),
                "relative link, in_git={in_git}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn in_git_the_build_copy_takes_the_harness_file_list_with_no_directory_skips() {
        let src = tempfile::tempdir().unwrap();
        if !have_run(src.path(), "git", &["init", "-q"]) {
            return;
        }
        let dst = tempfile::tempdir().unwrap();
        write_files(
            src.path(),
            &[
                (".gitignore", "native/build/\nignored.c\n"),
                ("Makefile", "x\n"),
                ("build/common.mk", "x\n"),
                ("out/gen.c", "x\n"),
                ("native/CMakeLists.txt", "x\n"),
                ("native/build/CMakeCache.txt", "x\n"),
                ("ignored.c", "x\n"),
                ("node_modules/m/i.js", "x\n"),
            ],
        );
        assert!(have_run(src.path(), "git", &["add", "build/common.mk"]));
        let want: Vec<String> = crate::project_copy::git_files(src.path())
            .unwrap()
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        let copy = copy_for_build(src.path(), dst.path(), "native").unwrap();
        let got = files_under(&copy);
        assert!(got.contains(&"build/common.mk".to_owned()), "{got:?}");
        assert!(got.contains(&"out/gen.c".to_owned()), "{got:?}");
        assert!(
            !got.iter().any(|f| f.starts_with("native/build/")),
            "{got:?}"
        );
        assert!(!got.contains(&"ignored.c".to_owned()), "{got:?}");
        // Not ignored, so git lists it and the copy takes it, as the harness copy does.
        assert!(got.contains(&"node_modules/m/i.js".to_owned()), "{got:?}");
        assert_eq!(got, want);
    }

    #[cfg(unix)]
    #[test]
    fn a_makefile_include_of_a_tracked_build_directory_file_builds_in_the_copy() {
        if !have_make_and_cc() {
            return;
        }
        let d = tempfile::tempdir().unwrap();
        if !have_run(d.path(), "git", &["init", "-q"]) {
            return;
        }
        write_files(
            d.path(),
            &[
                (
                    "Makefile",
                    "include build/common.mk\napp: main.c\n\t$(CC) $(CFLAGS) -o app main.c\n",
                ),
                ("build/common.mk", "CFLAGS = -g\n"),
                ("main.c", "int main(void) { return 0; }\n"),
            ],
        );
        assert!(have_run(d.path(), "git", &["add", "."]));
        let t = Tree::walk(d.path());
        let det = detect(&t);
        let scratch = tempfile::tempdir().unwrap();
        let b = run_one(&t, &det[0], scratch.path(), Duration::from_secs(60));
        assert_eq!(b.status, BuildStatus::Ok, "{:?}", b.log_tail);
    }

    #[cfg(unix)]
    #[test]
    fn a_cmake_project_with_an_ignored_developer_build_directory_builds_in_the_copy() {
        let have = |t: &str| {
            std::process::Command::new(t)
                .arg("--version")
                .output()
                .is_ok_and(|o| o.status.success())
        };
        if !have("cmake") || !have_make_and_cc() {
            return;
        }
        let d = tempfile::tempdir().unwrap();
        if !have_run(d.path(), "git", &["init", "-q"]) {
            return;
        }
        write_files(
            d.path(),
            &[
                (".gitignore", "build/\n"),
                (
                    "native/CMakeLists.txt",
                    "cmake_minimum_required(VERSION 3.10)\nproject(app C)\nadd_executable(app main.c)\n",
                ),
                ("native/main.c", "int main(void) { return 0; }\n"),
            ],
        );
        // The developer's own build: its cache names this directory.
        let native = d.path().join("native");
        if !have_run(&native, "cmake", &["-S", ".", "-B", "build"]) {
            return;
        }
        assert!(native.join("build/CMakeCache.txt").exists());
        let t = Tree::walk(d.path());
        let det = detect(&t);
        let d0 = det
            .iter()
            .find(|x| x.probe.runtime == Runtime::Native)
            .expect("native detected");
        assert_eq!(d0.root, "native");
        let scratch = tempfile::tempdir().unwrap();
        let b = run_one(&t, d0, scratch.path(), Duration::from_secs(120));
        assert_eq!(b.status, BuildStatus::Ok, "{:?}", b.log_tail);
    }

    #[cfg(unix)]
    #[test]
    fn in_git_the_build_copy_drops_what_the_harness_copy_drops() {
        use std::os::unix::fs::symlink;
        let src = tempfile::tempdir().unwrap();
        if !have_run(src.path(), "git", &["init", "-q"]) {
            return;
        }
        let dst = tempfile::tempdir().unwrap();
        write_files(
            src.path(),
            &[
                ("blob.o", "x\n"),
                ("loose.o", "x\n"),
                ("sub/old.lo", "x\n"),
                ("libtracked.a", "x\n"),
                ("libloose.a", "x\n"),
                ("keep.c", "x\n"),
                (".dla/state", "x\n"),
            ],
        );
        symlink("/nonexistent/away", src.path().join("dangling")).unwrap();
        symlink("keep.c", src.path().join("inside")).unwrap();
        assert!(have_run(
            src.path(),
            "git",
            &["add", "blob.o", "libtracked.a", "keep.c"]
        ));
        let copy = copy_for_build(src.path(), dst.path(), "").unwrap();
        let got = files_under(&copy);
        // `files_under` lists links as files only when they resolve; check by name.
        assert!(copy.join("keep.c").is_file() && copy.join("libtracked.a").is_file());
        // A tracked object is a shipped input (a prebuilt `blob.o`); an untracked one is
        // another tree's output.
        assert!(copy.join("blob.o").is_file(), "{got:?}");
        assert!(!copy.join("loose.o").exists(), "{got:?}");
        assert!(!copy.join("sub/old.lo").exists(), "{got:?}");
        assert!(!copy.join("libloose.a").exists(), "{got:?}");
        assert!(!copy.join(".dla").exists(), "{got:?}");
        assert!(!copy.join("dangling").exists() && copy.join("inside").is_symlink());
        assert!(std::fs::symlink_metadata(copy.join("dangling")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn a_project_ignored_by_an_enclosing_repository_is_copied_by_walking_it() {
        let outer = tempfile::tempdir().unwrap();
        if !have_run(outer.path(), "git", &["init", "-q"]) {
            return;
        }
        write_files(
            outer.path(),
            &[
                (".gitignore", "proj/\n"),
                ("proj/Makefile", "all:\n"),
                ("proj/main.c", "x\n"),
            ],
        );
        let dst = tempfile::tempdir().unwrap();
        let copy = copy_for_build(&outer.path().join("proj"), dst.path(), "").unwrap();
        assert_eq!(files_under(&copy), vec!["Makefile", "main.c"]);
    }

    #[cfg(unix)]
    #[test]
    fn the_build_copy_keeps_the_time_of_a_read_only_file() {
        use std::os::unix::fs::PermissionsExt;
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        let f = src.path().join("lease.h");
        std::fs::write(&f, "x\n").unwrap();
        let old = std::time::SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
        std::fs::File::options()
            .write(true)
            .open(&f)
            .unwrap()
            .set_modified(old)
            .unwrap();
        std::fs::set_permissions(&f, std::fs::Permissions::from_mode(0o444)).unwrap();
        let to = dst.path().join("lease.h");
        copy_file_keeping_time(&f, &to).unwrap();
        let meta = std::fs::metadata(&to).unwrap();
        assert_eq!(meta.modified().unwrap(), old);
        assert_eq!(meta.permissions().mode() & 0o777, 0o444);
    }

    #[test]
    fn no_build_command_is_skipped_with_the_reason() {
        let (_d, t, det) = proj(&[("A.java", "")]);
        let b = run_one(&t, &det[0], Path::new("/tmp"), Duration::from_secs(5));
        assert_eq!(b.status, BuildStatus::Skipped);
        assert!(b.command.is_none() && b.log_tail.is_some());
    }
}
