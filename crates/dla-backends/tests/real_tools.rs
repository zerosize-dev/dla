//! The adapters against the real analyzers, on the small probe projects in
//! `tests/data/probes`. Each test skips (and says so) when its tool is not installed,
//! so the suite passes on machines without analyzers.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use dla_backends::{fsx::ToolEnv, sweep::sweep_with};
use dla_core::{
    Ledger, RunState,
    coverage::{BackendStatus, CoverageManifest},
    runtime::Runtime,
    state::{Consent, ConsentGiver, now_utc},
    toolctx::ToolCtx,
};

fn copy(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for e in fs::read_dir(src).unwrap().flatten() {
        let to = dst.join(e.file_name());
        if e.path().is_dir() {
            copy(&e.path(), &to);
        } else {
            fs::copy(e.path(), to).unwrap();
        }
    }
}

struct Probe {
    _dir: tempfile::TempDir,
    ctx: ToolCtx,
}

fn probe(name: &str) -> Probe {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("proj");
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    copy(&manifest.join("tests/data/probes").join(name), &project);
    let run_dir = project.join(".dla/r1");
    fs::create_dir_all(&run_dir).unwrap();
    let mut st =
        RunState::init(project.to_str().unwrap(), Some("s"), 3600, None, now_utc()).unwrap();
    st.run_id = "r1".into();
    st.consent = Some(Consent {
        tiers: vec![1],
        raw_values: false,
        given_by: ConsentGiver::Runner,
        recorded_at: now_utc(),
    });
    st.save(&run_dir).unwrap();
    Ledger::create(&run_dir).unwrap();
    let ctx = ToolCtx {
        project,
        run_dir,
        run_id: "r1".into(),
        source_commit: "abc".into(),
        data_flag: Some(manifest.join("../../data")),
    };
    Probe { _dir: dir, ctx }
}

fn env() -> ToolEnv {
    ToolEnv::from_process(None)
}

fn sites(p: &Probe, backend: &str) -> BTreeSet<String> {
    let prefix = format!("detector:{backend}/");
    p.ctx
        .ledger()
        .unwrap()
        .findings()
        .filter(|f| f.origin.to_string().starts_with(&prefix))
        .map(|f| {
            format!(
                "{}@{}",
                &f.origin.to_string()[prefix.len()..],
                f.site.clone().unwrap()
            )
        })
        .collect()
}

fn status(p: &Probe, lang: &str, backend: &str) -> BackendStatus {
    CoverageManifest::load(&p.ctx.run_dir)
        .unwrap()
        .unwrap()
        .workers[lang]
        .backends[backend]
        .status
}

fn have(names: &[&str]) -> bool {
    let e = env();
    let ok = names.iter().all(|n| e.find_program(n).is_some());
    if !ok {
        eprintln!("skipped: needs {names:?} on PATH");
    }
    ok
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(ToString::to_string).collect()
}

#[test]
fn go_probe_with_the_go_analyzers() {
    if !have(&["go"]) {
        return;
    }
    let p = probe("go");
    sweep_with(&p.ctx, Runtime::Go, &env()).unwrap();
    assert_eq!(status(&p, "go", "govet"), BackendStatus::Ran);
    assert_eq!(sites(&p, "govet"), set(&["lostcancel@p.go:41"]));
    if have(&["staticcheck"]) {
        assert_eq!(
            sites(&p, "staticcheck"),
            set(&[
                "SA2001@p.go:62",
                "SA2003@p.go:64",
                "SA5003@p.go:51",
                "SA6002@q.go:14",
                "SA9001@q.go:25"
            ])
        );
    }
    if have(&["golangci-lint"]) {
        let got = sites(&p, "golangci-lint");
        for want in [
            "bodyclose@p.go:18",
            "fatcontext@p.go:41",
            "gocritic/deferInLoop@p.go:48",
            "noctx@p.go:23",
        ] {
            assert!(got.contains(want), "{want} in {got:?}");
        }
    }
}

#[test]
fn python_probe_with_ruff_and_pylint() {
    if !have(&["ruff"]) {
        return;
    }
    let p = probe("python");
    sweep_with(&p.ctx, Runtime::Python, &env()).unwrap();
    assert_eq!(status(&p, "python", "ruff"), BackendStatus::Ran);
    assert_eq!(
        sites(&p, "ruff"),
        set(&[
            "B019@leaks.py:3",
            "B019@leaks.py:5",
            "RUF006@leaks.py:8",
            "SIM115@leaks.py:10"
        ])
    );
    if have(&["pylint"]) {
        assert_eq!(
            sites(&p, "pylint"),
            set(&["W1518@leaks.py:3", "W1518@leaks.py:5"])
        );
    }
}

#[test]
fn node_probe_with_eslint_when_the_plugins_are_installed() {
    let Some(eslint) = env().find_program("eslint") else {
        eprintln!("skipped: needs eslint");
        return;
    };
    let installed = eslint.canonicalize().ok().is_some_and(|r| {
        r.ancestors().any(|a| {
            a.join("typescript-eslint").is_dir() && a.join("eslint-plugin-rxjs-x").is_dir()
        })
    });
    if !installed {
        eprintln!("skipped: typescript-eslint and eslint-plugin-rxjs-x are not next to eslint");
        return;
    }
    let p = probe("node");
    sweep_with(&p.ctx, Runtime::Node, &env()).unwrap();
    assert_eq!(status(&p, "node", "eslint"), BackendStatus::Ran);
    let got = sites(&p, "eslint");
    assert!(got.contains("rxjs-x/no-nested-subscribe@a.ts:6"), "{got:?}");
    assert!(
        got.contains("@typescript-eslint/no-floating-promises@a.ts:8"),
        "{got:?}"
    );
}

#[test]
fn java_probe_blocks_spotbugs_without_classes_and_reads_it_with_them() {
    if !have(&["javac", "java"]) {
        return;
    }
    let p = probe("java");
    let e = env();
    sweep_with(&p.ctx, Runtime::Jvm, &e).unwrap();
    let cov = CoverageManifest::load(&p.ctx.run_dir).unwrap().unwrap();
    let sb = &cov.workers["jvm"].backends["spotbugs"];
    if sb.status != BackendStatus::Missing {
        assert_eq!(sb.status, BackendStatus::Blocked);
        assert_eq!(
            sb.reason.as_deref(),
            Some("no class files: run dla probe --build")
        );
    }
    // Compile next to the sources (a build step), then sweep again.
    let out: PathBuf = p.ctx.project.join("out");
    fs::create_dir_all(&out).unwrap();
    let st = std::process::Command::new("javac")
        .env_remove("JAVA_TOOL_OPTIONS")
        .args(["-g", "-d"])
        .arg(&out)
        .arg(p.ctx.project.join("P.java"))
        .arg(p.ctx.project.join("Q.java"))
        .status()
        .unwrap();
    assert!(st.success());
    sweep_with(&p.ctx, Runtime::Jvm, &e).unwrap();
    if status(&p, "jvm", "spotbugs") == BackendStatus::Ran {
        let got = sites(&p, "spotbugs");
        assert!(got.contains("OS_OPEN_STREAM@P.java:6"), "{got:?}");
        assert!(
            got.contains("OBL_UNSATISFIED_OBLIGATION_EXCEPTION_EDGE@Q.java:5"),
            "{got:?}"
        );
    }
    if status(&p, "jvm", "errorprone") == BackendStatus::Ran {
        let got = sites(&p, "errorprone");
        assert!(got.contains("StreamResourceLeak@P.java:5"), "{got:?}");
        assert!(got.contains("ThreadLocalUsage@P.java:3"), "{got:?}");
    }
}

#[test]
fn kotlin_probe_with_detekt() {
    if !have(&["detekt2"]) {
        return;
    }
    let p = probe("kotlin");
    sweep_with(&p.ctx, Runtime::Jvm, &env()).unwrap();
    assert_eq!(status(&p, "jvm", "detekt"), BackendStatus::Ran);
    let got = sites(&p, "detekt");
    assert!(got.contains("GlobalCoroutineUsage@A.kt:2"), "{got:?}");
}
