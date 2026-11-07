//! `dla-bench run` and the plugin variants, with a stand-in for the claude executable.
#![cfg(unix)]

use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

use dla_bench::run::{Ablation, VariantSpec, build_variant};

fn write(p: &Path, text: &str) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn script(p: &Path, body: &str) {
    write(p, &format!("#!/bin/sh\n{body}\n"));
    fs::set_permissions(p, fs::Permissions::from_mode(0o755)).unwrap();
}

fn fixture(root: &Path, id: &str) {
    let d = root.join(id);
    write(&d.join("project/src/main.py"), "print('hi')\n");
    write(&d.join("project/build/skip.txt"), "x\n");
    write(&d.join("project/.gitignore"), "build/\n");
    write(&d.join("README.md"), "readme\n");
    write(
        &d.join("expected.toml"),
        &format!(
            "schema_version = 1\nid = \"{id}\"\nruntime = \"python\"\ncategory = \"non-leak\"\ndynamic_confirmation = true\n[task]\nprompt = \"The service grows.\"\n[[non_leak]]\nid = \"N1\"\ncause = \"N-WARM\"\ndiscriminator = \"d\"\n"
        ),
    );
}

const FAKE_OK: &str = r#"
echo called >> "$FAKE_LOG"
cfg="$CLAUDE_CONFIG_DIR"
printf '%s\n' "$@" > "$cfg/args.txt"
env | sort > "$cfg/env.txt"
pwd > "$cfg/pwd.txt"
echo '{"type":"system","subtype":"init","model":"fake-model"}'
echo '{"findings":[{"site":"src/main.py:1","verdict":"not-a-leak","discriminator":"flat"}]}' > findings.json
mkdir -p .dla/R1
echo R1 > .dla/current
echo '{"type":"finding_added","finding":{"id":"F1","site":"src/main.py:1"}}' > .dla/R1/ledger.jsonl
echo '{}' > .dla/R1/state.json
echo '{"type":"result","total_cost_usd":0.5,"num_turns":3,"usage":{"input_tokens":11,"output_tokens":22,"cache_read_input_tokens":33,"cache_creation_input_tokens":44}}'
"#;

struct Env {
    _t: tempfile::TempDir,
    root: PathBuf,
}

impl Env {
    fn new(fake_body: &str) -> Env {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().to_path_buf();
        fixture(&root.join("fx"), "fx-a");
        fixture(&root.join("fx"), "fx-b");
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(root.join("fx"))
            .status()
            .unwrap();
        script(&root.join("claude"), fake_body);
        fs::create_dir_all(root.join("extra-secret")).unwrap();
        fs::create_dir_all(root.join("repo")).unwrap();
        Env { _t: t, root }
    }

    fn cmd(&self, extra: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_dla-bench"));
        c.arg("run")
            .args(["--fixtures-dir"])
            .arg(self.root.join("fx"))
            .arg("--out")
            .arg(self.root.join("out/res"))
            .args(if extra.contains(&"--work-root") {
                vec![]
            } else {
                vec![
                    "--work-root".to_owned(),
                    self.root.join("work").display().to_string(),
                ]
            })
            .arg("--claude-bin")
            .arg(self.root.join("claude"))
            .args(["--repo-root"])
            .arg(self.root.join("repo"))
            .args(extra)
            .env("FAKE_LOG", self.root.join("calls.log"))
            .env("CLAUDE_CODE_SESSION_ID", "parent-session")
            .env("CLAUDE_SESSION_ID", "parent-session");
        c
    }

    fn calls(&self) -> usize {
        fs::read_to_string(self.root.join("calls.log"))
            .map(|t| t.lines().count())
            .unwrap_or(0)
    }

    fn run_dir(&self, abl: &str, arch: &str, fx: &str, n: u32) -> PathBuf {
        self.root
            .join(format!("out/res/runs/{abl}/archetypes-{arch}/{fx}/{n}"))
    }

    fn work_dir(&self, abl: &str, arch: &str, fx: &str, n: u32) -> PathBuf {
        self.root
            .join(format!("work/res/{abl}/archetypes-{arch}/{fx}/{n}"))
    }
}

#[test]
fn no_skill_run_collects_everything_and_resumes() {
    let e = Env::new(FAKE_OK);
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--runs",
            "2",
            "--fixtures",
            "fx-a,fx-b",
            "--keep-work",
            "--max-turns",
            "9",
            "--deny-read",
            e.root.join("extra-secret").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(e.calls(), 4);
    assert_eq!(
        stdout.lines().filter(|l| l.contains("exit 0")).count(),
        4,
        "{stdout}"
    );
    assert!(stdout.contains("total $2.0000"), "{stdout}");

    let rd = e.run_dir("no-skill", "on", "fx-a", 1);
    let rj: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(rd.join("run.json")).unwrap()).unwrap();
    assert_eq!(rj["schema_version"], 1);
    assert_eq!(rj["fixture"], "fx-a");
    assert_eq!(rj["ablation"], "no-skill");
    assert_eq!(rj["archetypes"], "on");
    assert_eq!(rj["run"], 1);
    assert_eq!(rj["exit_code"], 0);
    assert_eq!(rj["timed_out"], false);
    assert_eq!(rj["cost_usd"], 0.5);
    assert_eq!(rj["num_turns"], 3);
    assert_eq!(rj["model"], "fake-model");
    assert_eq!(rj["usage"]["cache_creation_input_tokens"], 44);
    assert!(rj["error"].is_null());
    assert!(rj["started_at"].as_str().unwrap().ends_with('Z'));
    for f in [
        "transcript.jsonl",
        "stderr.log",
        "findings.json",
        "ledger.jsonl",
        "state.json",
    ] {
        assert!(rd.join(f).is_file(), "{f} collected");
    }
    assert!(!rd.join("report.md").exists());
    let transcript = fs::read_to_string(rd.join("transcript.jsonl")).unwrap();
    assert!(transcript.contains("fake-model"));

    // the working copy: tracked files only, committed, outside the repository
    let work = e.work_dir("no-skill", "on", "fx-a", 1);
    let project = work.join("project");
    assert!(project.join("src/main.py").is_file());
    assert!(
        !project.join("build/skip.txt").exists(),
        "ignored files are not copied"
    );
    assert!(!project.join("expected.toml").exists());
    assert!(!project.join("README.md").exists());
    let log = Command::new("git")
        .args(["log", "--format=%an <%ae>"])
        .current_dir(&project)
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&log.stdout).trim(),
        "bench <bench@example.invalid>"
    );

    // the child: fresh config dir, no parent session, bypass flags, prompt
    let cfg = work.join("config");
    let envtxt = fs::read_to_string(cfg.join("env.txt")).unwrap();
    assert!(!envtxt.contains("CLAUDE_CODE_SESSION_ID"), "{envtxt}");
    assert!(!envtxt.contains("CLAUDE_SESSION_ID"));
    assert!(envtxt.contains(&format!("CLAUDE_CONFIG_DIR={}", cfg.display())));
    assert!(!envtxt.contains("DLA_BIN"));
    let args = fs::read_to_string(cfg.join("args.txt")).unwrap();
    let args: Vec<&str> = args.lines().collect();
    assert_eq!(args[0], "-p");
    assert!(args.contains(&"stream-json") && args.contains(&"--verbose"));
    assert!(args.windows(2).any(|w| w == ["--max-turns", "9"]));
    assert!(!args.contains(&"--permission-mode") && !args.contains(&"bypassPermissions"));
    let at = args
        .iter()
        .position(|a| *a == "--settings")
        .expect("--settings");
    assert_eq!(args[at + 1], work.join("settings.json").to_str().unwrap());
    let settings: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(work.join("settings.json")).unwrap()).unwrap();
    assert_eq!(settings["sandbox"]["enabled"], true);
    assert_eq!(settings["sandbox"]["allowUnsandboxedCommands"], false);
    assert_eq!(settings["sandbox"]["failIfUnavailable"], true);
    assert_eq!(settings["sandbox"]["autoAllowBashIfSandboxed"], true);
    let deny_read: Vec<&str> = settings["sandbox"]["filesystem"]["denyRead"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let canon = |p: PathBuf| p.canonicalize().unwrap().display().to_string();
    for must in [
        canon(e.root.join("repo")),
        canon(e.root.join("fx")),
        canon(e.root.join("out/res")),
        canon(e.root.join("extra-secret")),
    ] {
        assert!(
            deny_read.contains(&must.as_str()),
            "{must} in {deny_read:?}"
        );
    }
    let rules = settings["permissions"]["deny"].to_string();
    assert!(
        rules.contains(&format!("Read(/{}/**)", canon(e.root.join("fx")))),
        "{rules}"
    );
    assert!(rules.contains("WebFetch") && rules.contains("WebSearch"));
    let w = |sub: &str| work.join(sub).display().to_string();
    assert_eq!(
        settings["sandbox"]["filesystem"]["allowWrite"],
        serde_json::json!([w("project"), w("cache"), w("tmp")])
    );
    let allow = settings["permissions"]["allow"].to_string();
    assert!(
        allow.contains(&format!("\"Write(/{}/**)\"", w("project"))),
        "{allow}"
    );
    assert!(
        !allow.contains("\"Write\"") && !allow.contains("\"Edit\""),
        "{allow}"
    );
    assert!(envtxt.contains(&format!("CLAUDE_CODE_TMPDIR={}", w("tmp"))));
    for var in [
        "GOCACHE",
        "GOMODCACHE",
        "GOTMPDIR",
        "npm_config_cache",
        "PIP_CACHE_DIR",
        "CARGO_HOME",
    ] {
        assert!(
            envtxt.lines().any(|l| l.starts_with(&format!("{var}="))
                && l.contains(&format!("{}/cache/", work.display()))),
            "{var} under the run cache: {envtxt}"
        );
    }
    assert!(envtxt.contains(&format!("-Dmaven.repo.local={}/cache/m2", work.display())));
    assert!(!envtxt.contains("IS_SANDBOX"));
    assert!(work.join("cache/go-build").is_dir());
    assert!(
        args.iter()
            .any(|l| l.contains("This machine has no network access."))
    );
    assert!(args.contains(&"--strict-mcp-config"));
    assert!(!args.contains(&"--plugin-dir"));
    assert!(args.iter().any(|l| l.contains("The service grows.")));
    assert!(args.iter().any(|l| l.contains("write ./findings.json")));
    let pwd = fs::read_to_string(cfg.join("pwd.txt")).unwrap();
    assert_eq!(
        pwd.trim(),
        project.canonicalize().unwrap().to_str().unwrap()
    );

    // resume: nothing is started again
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--runs",
            "2",
            "--fixtures",
            "fx-a,fx-b",
            "--keep-work",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(e.calls(), 4);
    assert_eq!(
        String::from_utf8_lossy(&out.stdout)
            .matches("already done")
            .count(),
        4
    );

    // a third run is added, the first two are kept
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--runs",
            "3",
            "--fixtures",
            "fx-a,fx-b",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(e.calls(), 6);
    // clean runs without --keep-work remove their working copy
    assert!(!e.work_dir("no-skill", "on", "fx-a", 3).exists());
}

#[test]
fn timeout_kills_the_whole_process_group() {
    let e = Env::new("sleep 60 &\necho $! > \"$FAKE_PID\"\nwait");
    let pidfile = e.root.join("child.pid");
    let started = Instant::now();
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--runs",
            "1",
            "--fixtures",
            "fx-a",
            "--timeout",
            "1",
        ])
        .env("FAKE_PID", &pidfile)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(started.elapsed().as_secs() < 30);
    let rd = e.run_dir("no-skill", "on", "fx-a", 1);
    let rj: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(rd.join("run.json")).unwrap()).unwrap();
    assert_eq!(rj["timed_out"], true);
    assert!(rj["error"].as_str().unwrap().contains("timed out"));
    assert!(rj["cost_usd"].is_null());
    let pid = fs::read_to_string(&pidfile).unwrap().trim().to_owned();
    std::thread::sleep(std::time::Duration::from_millis(300));
    assert!(
        !Path::new(&format!("/proc/{pid}")).exists() || {
            // a zombie awaiting its parent still has a /proc entry; its state says so
            fs::read_to_string(format!("/proc/{pid}/stat"))
                .map(|s| s.contains(") Z"))
                .unwrap_or(true)
        },
        "background child {pid} survived the timeout"
    );
}

#[test]
fn budget_stops_new_runs() {
    let e = Env::new(FAKE_OK);
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--runs",
            "3",
            "--fixtures",
            "fx-a,fx-b",
            "--parallel",
            "1",
            "--budget-usd",
            "0.7",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(e.calls(), 2, "0.5 then 1.0 reaches the 0.7 budget");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("4 not started"), "{stdout}");
}

#[test]
fn work_root_inside_the_repository_is_refused() {
    let e = Env::new(FAKE_OK);
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--runs",
            "1",
            "--fixtures",
            "fx-a",
            "--work-root",
        ])
        .arg(e.root.join("repo/work"))
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("outside"));
}

#[test]
fn missing_claude_is_recorded_not_fatal() {
    let e = Env::new(FAKE_OK);
    fs::remove_file(e.root.join("claude")).unwrap();
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--runs",
            "1",
            "--fixtures",
            "fx-a",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    let rd = e.run_dir("no-skill", "on", "fx-a", 1);
    let rj: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(rd.join("run.json")).unwrap()).unwrap();
    assert_eq!(rj["exit_code"], -1);
    assert!(
        rj["error"]
            .as_str()
            .unwrap()
            .contains("cannot start claude")
    );
}

// ---------------------------------------------------------------- variants

fn mini_repo(root: &Path) {
    write(
        &root.join(".claude-plugin/plugin.json"),
        "{\"name\":\"dla\"}\n",
    );
    write(&root.join(".claude-plugin/marketplace.json"), "{}\n");
    write(
        &root.join("skills/detect-leak-anywhere/SKILL.md"),
        "---\nname: detect-leak-anywhere\ndescription: d\nhooks:\n  PreToolUse:\n    - matcher: Bash\n---\nbody\n",
    );
    write(
        &root.join("skills/detect-leak-anywhere/references/jvm.md"),
        "jvm\n",
    );
    write(
        &root.join("skills/detect-leak-anywhere/references/jvm-archetypes.md"),
        "a\n",
    );
    write(
        &root.join("skills/detect-leak-anywhere/references/known-leaks.md"),
        "k\n",
    );
    write(&root.join("agents/verifier.md"), "v\n");
    write(&root.join("agents/worker.md"), "w\n");
    write(&root.join("hooks/hooks.json"), "{}\n");
    script(&root.join("bin/dla"), "exit 0");
    write(&root.join("data/archetypes.toml"), "a=1\n");
    write(&root.join("data/known-leaks.toml"), "a=1\n");
    write(&root.join("data/other.toml"), "keep=1\n");
    write(
        &root.join("data/detectors/x.toml"),
        "[[rule]]\nid = \"a\"\nsuggested_archetype = \"L-COLLECT\"\n[[rule]]\nid = \"b\"\n  suggested_archetype = \"L-WORKER\"\nother = \"z\"\n",
    );
    write(
        &root.join("bench/ablations/prose/SKILL.md"),
        "---\nname: detect-leak-anywhere\ndescription: p\n---\nprose\n",
    );
}

fn variant(repo: &Path, abl: Ablation, off: bool, dest: &Path) {
    build_variant(
        &VariantSpec {
            repo_root: repo.to_path_buf(),
            ablations_dir: repo.join("bench/ablations"),
            ablation: abl,
            archetypes_off: off,
        },
        dest,
    )
    .unwrap();
}

#[test]
fn prose_variant_has_only_skill_and_references() {
    let t = tempfile::tempdir().unwrap();
    mini_repo(t.path());
    let d = t.path().join("v");
    variant(t.path(), Ablation::Prose, false, &d);
    assert_eq!(
        fs::read_to_string(d.join("skills/detect-leak-anywhere/SKILL.md")).unwrap(),
        "---\nname: detect-leak-anywhere\ndescription: p\n---\nprose\n"
    );
    assert!(
        d.join("skills/detect-leak-anywhere/references/jvm.md")
            .is_file()
    );
    assert!(
        d.join("skills/detect-leak-anywhere/references/jvm-archetypes.md")
            .is_file()
    );
    assert!(d.join(".claude-plugin/plugin.json").is_file());
    assert!(!d.join(".claude-plugin/marketplace.json").exists());
    for gone in ["bin", "hooks", "agents"] {
        assert!(!d.join(gone).exists(), "{gone}");
    }
    let d2 = t.path().join("v2");
    variant(t.path(), Ablation::Prose, true, &d2);
    assert!(
        d2.join("skills/detect-leak-anywhere/references/jvm.md")
            .is_file()
    );
    assert!(
        !d2.join("skills/detect-leak-anywhere/references/jvm-archetypes.md")
            .exists()
    );
    assert!(
        !d2.join("skills/detect-leak-anywhere/references/known-leaks.md")
            .exists()
    );
}

#[test]
fn gates_variant_has_no_hooks_no_verifier_and_no_hooks_block() {
    let t = tempfile::tempdir().unwrap();
    mini_repo(t.path());
    let d = t.path().join("v");
    variant(t.path(), Ablation::Gates, false, &d);
    assert!(!d.join("hooks").exists());
    assert!(!d.join("agents/verifier.md").exists());
    assert!(d.join("agents/worker.md").is_file());
    assert!(d.join("bin/dla").is_file());
    assert!(d.join("data/archetypes.toml").is_file());
    let skill = fs::read_to_string(d.join("skills/detect-leak-anywhere/SKILL.md")).unwrap();
    assert_eq!(
        skill,
        "---\nname: detect-leak-anywhere\ndescription: d\n---\nbody\n"
    );
    let mode = fs::metadata(d.join("bin/dla"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o111, 0o111, "bin/dla stays executable");
}

#[test]
fn full_variant_is_the_plugin_as_is_and_archetypes_off_strips_material() {
    let t = tempfile::tempdir().unwrap();
    mini_repo(t.path());
    let d = t.path().join("v");
    variant(t.path(), Ablation::Full, false, &d);
    assert!(d.join("hooks/hooks.json").is_file());
    assert!(d.join("agents/verifier.md").is_file());
    assert!(
        fs::read_to_string(d.join("skills/detect-leak-anywhere/SKILL.md"))
            .unwrap()
            .contains("hooks:")
    );
    let off = t.path().join("off");
    variant(t.path(), Ablation::Full, true, &off);
    assert!(!off.join("data/archetypes.toml").exists());
    assert!(!off.join("data/known-leaks.toml").exists());
    assert!(off.join("data/other.toml").is_file());
    assert!(
        !off.join("skills/detect-leak-anywhere/references/jvm-archetypes.md")
            .exists()
    );
    assert!(
        !off.join("skills/detect-leak-anywhere/references/known-leaks.md")
            .exists()
    );
    assert!(
        off.join("skills/detect-leak-anywhere/references/jvm.md")
            .is_file()
    );
    let det = fs::read_to_string(off.join("data/detectors/x.toml")).unwrap();
    assert!(det.contains("suggested_archetype = \"\""), "{det}");
    assert!(
        !det.contains("L-COLLECT") && !det.contains("L-WORKER"),
        "{det}"
    );
    assert!(det.contains("other = \"z\""));
    assert!(
        build_variant(
            &VariantSpec {
                repo_root: t.path().to_path_buf(),
                ablations_dir: t.path().join("bench/ablations"),
                ablation: Ablation::NoSkill,
                archetypes_off: false
            },
            &t.path().join("x")
        )
        .is_err()
    );
}

#[test]
fn gates_run_passes_the_plugin_dir_binary_and_consent() {
    let e = Env::new(FAKE_OK);
    mini_repo(&e.root.join("repo"));
    script(&e.root.join("fake-dla"), "exit 0");
    let out = e
        .cmd(&[
            "--ablation",
            "gates",
            "--archetypes",
            "off",
            "--runs",
            "1",
            "--fixtures",
            "fx-a",
            "--keep-work",
            "--dla-bin",
        ])
        .arg(e.root.join("fake-dla"))
        .arg("--ablations-dir")
        .arg(e.root.join("repo/bench/ablations"))
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let work = e.work_dir("gates", "off", "fx-a", 1);
    let envtxt = fs::read_to_string(work.join("config/env.txt")).unwrap();
    // the binary is copied next to the working copies: the repository is unreadable to the child
    let copy = e.root.join("work/res/bin/dla");
    assert!(copy.is_file());
    assert!(envtxt.contains(&format!("DLA_BIN={}", copy.display())));
    assert!(envtxt.contains("DLA_CONSENT_TIERS=1-4"));
    assert!(envtxt.contains("DLA_CONSENT_RAW_VALUES=1"));
    assert!(envtxt.contains("DLA_ARCHETYPES=off"));
    let args = fs::read_to_string(work.join("config/args.txt")).unwrap();
    let args: Vec<&str> = args.lines().collect();
    assert!(
        args.windows(2)
            .any(|w| w[0] == "--plugin-dir" && w[1] == work.join("plugin").to_str().unwrap())
    );
    assert!(args[1].starts_with("/dla:detect-leak-anywhere The service grows."));
    assert!(work.join("plugin/.claude-plugin/plugin.json").is_file());
    assert!(!work.join("plugin/hooks").exists());
    let rd = e.run_dir("gates", "off", "fx-a", 1);
    assert!(rd.join("ledger.jsonl").is_file());
}
