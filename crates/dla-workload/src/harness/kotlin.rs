//! The Kotlin standard library for driver-mode JVM harnesses.
//!
//! A driver is plain Java compiled against the project's classes. When the project is
//! Kotlin, those classes need `kotlin-stdlib` on the class path of the driver's build and
//! run. The jar is found, in order:
//!
//! 1. `$KOTLIN_HOME/lib/kotlin-stdlib.jar`;
//! 2. `<dir of kotlinc>/../lib/kotlin-stdlib.jar`, for the `kotlinc` on `PATH` (symbolic
//!    links resolved), so the program runs with the version it was compiled with;
//! 3. the project's own build script: a `kotlin-stdlib/<version>/kotlin-stdlib-<version>.jar`
//!    path in `build.sh` or `build.gradle*` is looked up in the Maven repository (`$M2`, else
//!    `~/.m2/repository`), and when that exact version is absent the newest version there.

use std::{
    fs,
    path::{Path, PathBuf},
};

/// The places a stdlib jar may be found; [`Lookup::current`] reads the process environment.
#[derive(Debug, Clone, Default)]
pub struct Lookup {
    pub kotlin_home: Option<PathBuf>,
    pub path: Vec<PathBuf>,
    pub home: Option<PathBuf>,
    pub m2: Option<PathBuf>,
}

impl Lookup {
    pub fn current() -> Self {
        let var = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty());
        Self {
            kotlin_home: var("KOTLIN_HOME").map(PathBuf::from),
            path: var("PATH")
                .map(|p| std::env::split_paths(&p).collect())
                .unwrap_or_default(),
            home: var("HOME")
                .or_else(|| var("USERPROFILE"))
                .map(PathBuf::from),
            m2: var("M2").map(PathBuf::from),
        }
    }
}

fn jar_in(dir: &Path) -> Option<PathBuf> {
    let p = dir.join("kotlin-stdlib.jar");
    p.is_file().then_some(p)
}

fn exe_names() -> [&'static str; 2] {
    ["kotlinc", "kotlinc.bat"]
}

/// Whether the scratch copy holds Kotlin sources (looked for a few levels down, skipping
/// build output and dependency directories).
pub fn has_kotlin_sources(root: &Path) -> bool {
    fn walk(dir: &Path, depth: usize) -> bool {
        let Ok(rd) = fs::read_dir(dir) else {
            return false;
        };
        for e in rd.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            let path = e.path();
            if path.is_dir() {
                let skip = matches!(
                    name.as_str(),
                    "target" | "node_modules" | "out" | "build" | ".git" | ".gradle" | "buildSrc"
                );
                if !skip && depth < 6 && walk(&path, depth + 1) {
                    return true;
                }
            } else if name.ends_with(".kt")
                || (name.ends_with(".kts") && !dla_core::paths::is_build_logic(&name))
            {
                return true;
            }
        }
        false
    }
    walk(root, 0)
}

/// The `<version>` of the first `kotlin-stdlib/<version>/kotlin-stdlib-<version>.jar` a
/// build script names.
fn version_in_script(text: &str) -> Option<String> {
    let at = text.find("kotlin-stdlib/")?;
    let rest = &text[at + "kotlin-stdlib/".len()..];
    let ver: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
        .collect();
    (ver.starts_with(|c: char| c.is_ascii_digit())
        && rest[ver.len()..].starts_with("/kotlin-stdlib-"))
    .then_some(ver)
}

/// Sort key of a version: the numeric parts, then whether it is a release (a suffix such as
/// `-RC` or `-Beta1` ranks below the same numbers without one), then the suffix.
fn version_key(v: &str) -> (Vec<u64>, bool, String) {
    let (nums, suffix) = match v.split_once('-') {
        Some((n, s)) => (n, s),
        None => (v, ""),
    };
    (
        nums.split('.')
            .map(|p| p.parse::<u64>().unwrap_or(0))
            .collect(),
        suffix.is_empty(),
        suffix.to_owned(),
    )
}

fn from_repository(repo: &Path, wanted: Option<&str>) -> Option<PathBuf> {
    let base = repo.join("org/jetbrains/kotlin/kotlin-stdlib");
    let jar = |v: &str| {
        let p = base.join(v).join(format!("kotlin-stdlib-{v}.jar"));
        p.is_file().then_some(p)
    };
    if let Some(p) = wanted.and_then(jar) {
        return Some(p);
    }
    let mut versions: Vec<String> = fs::read_dir(&base)
        .ok()?
        .flatten()
        .filter(|e| e.path().is_dir())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    versions.sort_by_key(|v| version_key(v));
    versions.iter().rev().find_map(|v| jar(v))
}

/// Finds `kotlin-stdlib.jar` for the project in `work`; `Err` says where it looked.
pub fn find_stdlib(work: &Path, env: &Lookup) -> Result<PathBuf, String> {
    if let Some(home) = &env.kotlin_home
        && let Some(p) = jar_in(&home.join("lib"))
    {
        return Ok(p);
    }
    for dir in &env.path {
        for name in exe_names() {
            let exe = dir.join(name);
            if !exe.is_file() {
                continue;
            }
            let real = fs::canonicalize(&exe).unwrap_or(exe);
            if let Some(p) = real
                .parent()
                .and_then(Path::parent)
                .and_then(|root| jar_in(&root.join("lib")))
            {
                return Ok(p);
            }
        }
    }
    let scripts = ["build.sh", "build.gradle", "build.gradle.kts"];
    let wanted = scripts.iter().find_map(|s| {
        fs::read_to_string(work.join(s))
            .ok()
            .and_then(|t| version_in_script(&t))
    });
    let repo = env
        .m2
        .clone()
        .or_else(|| env.home.as_ref().map(|h| h.join(".m2/repository")));
    if let Some(p) = repo.and_then(|r| from_repository(&r, wanted.as_deref())) {
        return Ok(p);
    }
    Err(
        "no kotlin-stdlib jar: set KOTLIN_HOME to the Kotlin install (lib/kotlin-stdlib.jar), put kotlinc on PATH, or build the project once so the Maven repository has kotlin-stdlib"
            .to_owned(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(p: &Path) {
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, b"jar").unwrap();
    }

    #[test]
    fn kotlin_home_comes_first() {
        let d = tempfile::tempdir().unwrap();
        touch(&d.path().join("kh/lib/kotlin-stdlib.jar"));
        touch(&d.path().join("kc/bin/kotlinc"));
        touch(&d.path().join("kc/lib/kotlin-stdlib.jar"));
        let env = Lookup {
            kotlin_home: Some(d.path().join("kh")),
            path: vec![d.path().join("kc/bin")],
            ..Lookup::default()
        };
        assert_eq!(
            find_stdlib(d.path(), &env).unwrap(),
            d.path().join("kh/lib/kotlin-stdlib.jar")
        );
    }

    #[test]
    fn the_jar_next_to_kotlinc_on_path_is_used() {
        let d = tempfile::tempdir().unwrap();
        touch(&d.path().join("kc/bin/kotlinc"));
        touch(&d.path().join("kc/lib/kotlin-stdlib.jar"));
        let env = Lookup {
            path: vec![d.path().join("none"), d.path().join("kc/bin")],
            ..Lookup::default()
        };
        let got = find_stdlib(d.path(), &env).unwrap();
        assert!(got.ends_with("kc/lib/kotlin-stdlib.jar"), "{got:?}");
    }

    #[cfg(unix)]
    #[test]
    fn a_kotlinc_link_is_followed_to_its_install() {
        let d = tempfile::tempdir().unwrap();
        touch(&d.path().join("kc/bin/kotlinc"));
        touch(&d.path().join("kc/lib/kotlin-stdlib.jar"));
        fs::create_dir_all(d.path().join("usr/bin")).unwrap();
        std::os::unix::fs::symlink(
            d.path().join("kc/bin/kotlinc"),
            d.path().join("usr/bin/kotlinc"),
        )
        .unwrap();
        let env = Lookup {
            path: vec![d.path().join("usr/bin")],
            ..Lookup::default()
        };
        let got = find_stdlib(d.path(), &env).unwrap();
        assert!(got.ends_with("kc/lib/kotlin-stdlib.jar"), "{got:?}");
    }

    #[test]
    fn the_build_script_names_the_version_in_the_maven_repository() {
        let d = tempfile::tempdir().unwrap();
        let repo = d.path().join("m2");
        for v in ["1.9.0", "2.0.21", "2.1.0"] {
            touch(&repo.join(format!(
                "org/jetbrains/kotlin/kotlin-stdlib/{v}/kotlin-stdlib-{v}.jar"
            )));
        }
        let work = d.path().join("work");
        fs::create_dir_all(&work).unwrap();
        fs::write(
            work.join("build.sh"),
            "K=$M/org/jetbrains/kotlin\nSTD=$K/kotlin-stdlib/2.0.21/kotlin-stdlib-2.0.21.jar\n",
        )
        .unwrap();
        let env = Lookup {
            m2: Some(repo.clone()),
            ..Lookup::default()
        };
        assert!(
            find_stdlib(&work, &env)
                .unwrap()
                .ends_with("2.0.21/kotlin-stdlib-2.0.21.jar")
        );
        // Without a script, the newest version.
        let bare = d.path().join("bare");
        fs::create_dir_all(&bare).unwrap();
        assert!(
            find_stdlib(&bare, &env)
                .unwrap()
                .ends_with("2.1.0/kotlin-stdlib-2.1.0.jar")
        );
        // A version the script names but the repository lacks falls back to the newest.
        fs::write(
            bare.join("build.sh"),
            "STD=$K/kotlin-stdlib/2.0.99/kotlin-stdlib-2.0.99.jar",
        )
        .unwrap();
        assert!(
            find_stdlib(&bare, &env)
                .unwrap()
                .ends_with("2.1.0/kotlin-stdlib-2.1.0.jar")
        );
    }

    #[test]
    fn home_is_the_default_repository() {
        let d = tempfile::tempdir().unwrap();
        touch(&d.path().join(
            "h/.m2/repository/org/jetbrains/kotlin/kotlin-stdlib/2.0.21/kotlin-stdlib-2.0.21.jar",
        ));
        let env = Lookup {
            home: Some(d.path().join("h")),
            ..Lookup::default()
        };
        assert!(find_stdlib(d.path(), &env).is_ok());
    }

    #[test]
    fn nothing_found_says_what_to_set() {
        let d = tempfile::tempdir().unwrap();
        let e = find_stdlib(d.path(), &Lookup::default()).unwrap_err();
        assert!(e.contains("KOTLIN_HOME"), "{e}");
    }

    #[test]
    fn kotlin_sources_are_found_but_build_output_is_skipped() {
        let d = tempfile::tempdir().unwrap();
        touch(&d.path().join("out/gen.kt"));
        assert!(!has_kotlin_sources(d.path()));
        touch(&d.path().join("kotlin/shop/Native.kt"));
        assert!(has_kotlin_sources(d.path()));
    }

    #[test]
    fn gradle_build_scripts_are_not_kotlin_sources() {
        let d = tempfile::tempdir().unwrap();
        touch(&d.path().join("build.gradle.kts"));
        touch(&d.path().join("sub/settings.gradle.kts"));
        touch(&d.path().join("init.gradle.kts"));
        touch(&d.path().join("docker.gradle.kts"));
        touch(&d.path().join("buildSrc/src/main/kotlin/Conventions.kt"));
        assert!(!has_kotlin_sources(d.path()));
        touch(&d.path().join("scripts/tool.main.kts"));
        assert!(has_kotlin_sources(d.path()));
    }

    #[test]
    fn a_release_beats_its_pre_releases_in_the_repository() {
        let d = tempfile::tempdir().unwrap();
        let repo = d.path().join("m2");
        for v in [
            "2.0.21",
            "2.1.0",
            "2.1.0-RC",
            "2.1.0-Beta1",
            "2.1.0-dev-123",
        ] {
            touch(&repo.join(format!(
                "org/jetbrains/kotlin/kotlin-stdlib/{v}/kotlin-stdlib-{v}.jar"
            )));
        }
        let got = from_repository(&repo, None).unwrap();
        assert!(got.ends_with("2.1.0/kotlin-stdlib-2.1.0.jar"), "{got:?}");
        // With only pre-releases, the newer number still wins.
        assert!(version_key("2.2.0-RC") > version_key("2.1.0"));
        assert!(version_key("2.1.0") > version_key("2.1.0-RC"));
    }

    #[test]
    fn script_versions_are_read_from_the_path_shape_only() {
        assert_eq!(
            version_in_script("x/kotlin-stdlib/2.0.21/kotlin-stdlib-2.0.21.jar"),
            Some("2.0.21".into())
        );
        assert_eq!(version_in_script("kotlin-stdlib/latest/x"), None);
        assert_eq!(version_in_script("nothing"), None);
    }
}
