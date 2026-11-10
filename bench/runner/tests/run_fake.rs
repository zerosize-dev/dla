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
        // Not in the system temporary directory: `run` refuses a work root there.
        let t = tempfile::Builder::new()
            .prefix("run-fake-")
            .tempdir_in(env!("CARGO_TARGET_TMPDIR"))
            .unwrap();
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
        serde_json::json!([w("project"), w("cache"), w("tmp"), "/tmp"]),
        "Bash may write in /tmp (JVM attach), after the run's own directories"
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
    assert!(
        allow.contains("\"Agent\"") && allow.contains("\"Task\""),
        "subagents are allowed: {allow}"
    );
    assert!(
        !allow.contains("(//tmp/**)") && !allow.contains("(//tmp)"),
        "Edit and Write get no rule for /tmp: {allow}"
    );
    assert!(envtxt.contains("DLA_DEFAULT_BUDGET=20m"), "{envtxt}");
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
    assert!(gone(&pid), "background child {pid} survived the timeout");
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
fn an_overflowing_child_budget_is_refused_up_front() {
    let e = Env::new(FAKE_OK);
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--runs",
            "1",
            "--fixtures",
            "fx-a",
            "--child-budget",
            "18446744073709551615h",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not a duration"));
    assert_eq!(e.calls(), 0);
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
fn work_root_inside_the_system_temp_directory_is_refused() {
    let e = Env::new(FAKE_OK);
    for root in ["/tmp/dla-bench-test-w", "/private/tmp/dla-bench-test-w"] {
        let out = e
            .cmd(&[
                "--ablation",
                "no-skill",
                "--runs",
                "1",
                "--fixtures",
                "fx-a",
                "--work-root",
                root,
            ])
            .output()
            .unwrap();
        assert!(!out.status.success(), "{root}");
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("system temporary directory"), "{err}");
        assert_eq!(e.calls(), 0);
    }
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

/// The shape of the real `hooks/hooks.json`: the gates call the shim by its plugin path.
const HOOKS_JSON: &str = r#"{"hooks":{"SubagentStop":[{"matcher":"^dla:worker$","hooks":[{"type":"command","command":"\"${CLAUDE_PLUGIN_ROOT}/bin/dla\" gate stop-hook --subagent","timeout":30}]}]}}
"#;

fn mini_repo(root: &Path) {
    write(
        &root.join(".claude-plugin/plugin.json"),
        "{\"name\":\"dla\"}\n",
    );
    write(&root.join(".claude-plugin/marketplace.json"), "{}\n");
    write(
        &root.join("skills/detect-leak-anywhere/SKILL.md"),
        "---\nname: detect-leak-anywhere\ndescription: d\nhooks:\n  PreToolUse:\n    - matcher: Bash\n---\nbody\n<!-- archetypes:begin -->\nlabel it with `dla ledger label`\n<!-- archetypes:end -->\nend\n",
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
    write(
        &root.join("agents/verifier.md"),
        "v\n<!-- archetypes:begin -->\nL-COLLECT: a collection\nL-TLS: a thread local\n<!-- archetypes:end -->\nafter\n",
    );
    write(&root.join("agents/worker.md"), "w\n");
    write(&root.join("hooks/hooks.json"), HOOKS_JSON);
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
        "---\nname: detect-leak-anywhere\ndescription: d\n---\nbody\n<!-- archetypes:begin -->\nlabel it with `dla ledger label`\n<!-- archetypes:end -->\nend\n"
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

#[test]
fn full_variant_copies_the_hook_gates_and_the_verifier_and_gates_copies_neither() {
    let t = tempfile::tempdir().unwrap();
    mini_repo(t.path());
    let full = t.path().join("full");
    variant(t.path(), Ablation::Full, false, &full);
    // hooks.json (the gates), the skill frontmatter hooks and the verifier are all kept.
    assert_eq!(
        fs::read_to_string(full.join("hooks/hooks.json")).unwrap(),
        HOOKS_JSON
    );
    assert!(full.join("agents/verifier.md").is_file());
    assert!(
        fs::read_to_string(full.join("skills/detect-leak-anywhere/SKILL.md"))
            .unwrap()
            .contains("hooks:\n  PreToolUse:")
    );
    let gates = t.path().join("gates");
    variant(t.path(), Ablation::Gates, false, &gates);
    assert!(!gates.join("hooks").exists());
    assert!(!gates.join("agents/verifier.md").exists());
    assert!(
        !fs::read_to_string(gates.join("skills/detect-leak-anywhere/SKILL.md"))
            .unwrap()
            .contains("hooks:")
    );
    // The same holds with archetypes off.
    let full_off = t.path().join("full-off");
    variant(t.path(), Ablation::Full, true, &full_off);
    assert_eq!(
        fs::read_to_string(full_off.join("hooks/hooks.json")).unwrap(),
        HOOKS_JSON
    );
    assert!(full_off.join("agents/verifier.md").is_file());
}

#[test]
fn archetypes_off_strips_every_marked_block_of_every_markdown_file() {
    let t = tempfile::tempdir().unwrap();
    mini_repo(t.path());
    for abl in [Ablation::Prose, Ablation::Gates, Ablation::Full] {
        let on = t.path().join(format!("{abl:?}-on"));
        let off = t.path().join(format!("{abl:?}-off"));
        variant(t.path(), abl, false, &on);
        variant(t.path(), abl, true, &off);
        let skill_on = fs::read_to_string(on.join("skills/detect-leak-anywhere/SKILL.md")).unwrap();
        let skill_off =
            fs::read_to_string(off.join("skills/detect-leak-anywhere/SKILL.md")).unwrap();
        if abl != Ablation::Prose {
            // The prose variant brings its own SKILL.md, which has no block here.
            assert!(skill_on.contains("archetypes:begin"), "{abl:?}");
            assert!(!skill_off.contains("archetypes:"), "{abl:?}: {skill_off}");
            assert!(!skill_off.contains("dla ledger label"), "{abl:?}");
            assert!(skill_off.ends_with("body\nend\n"), "{abl:?}: {skill_off}");
        }
        if abl == Ablation::Full {
            let v_on = fs::read_to_string(on.join("agents/verifier.md")).unwrap();
            let v_off = fs::read_to_string(off.join("agents/verifier.md")).unwrap();
            assert!(v_on.contains("L-COLLECT"));
            assert_eq!(v_off, "v\nafter\n", "markers and their body are gone");
        } else {
            // Prose and gates have no verifier.
            assert!(!off.join("agents/verifier.md").exists());
        }
    }
}

#[test]
fn unbalanced_archetype_markers_are_an_error() {
    use dla_bench::run::strip_archetype_blocks;
    assert_eq!(
        strip_archetype_blocks("a\n<!-- archetypes:begin -->\nx\n<!-- archetypes:end -->\nb\n")
            .unwrap(),
        "a\nb\n"
    );
    // Two blocks, and text with no block at all.
    assert_eq!(
        strip_archetype_blocks(
            "1\n<!-- archetypes:begin -->\nx\n<!-- archetypes:end -->\n2\n<!-- archetypes:begin -->y<!-- archetypes:end -->3\n"
        )
        .unwrap(),
        "1\n2\n3\n"
    );
    assert_eq!(strip_archetype_blocks("plain\n").unwrap(), "plain\n");
    for bad in [
        "a\n<!-- archetypes:begin -->\nx\n",
        "a\n<!-- archetypes:end -->\n",
        "<!-- archetypes:end -->\n<!-- archetypes:begin -->\nx\n<!-- archetypes:end -->\n",
        "<!-- archetypes:begin -->\n<!-- archetypes:begin -->\n<!-- archetypes:end -->\n",
    ] {
        assert!(strip_archetype_blocks(bad).is_err(), "{bad:?}");
    }
    // A variant with an unbalanced marker is refused, not built.
    let t = tempfile::tempdir().unwrap();
    mini_repo(t.path());
    write(
        &t.path()
            .join("skills/detect-leak-anywhere/references/jvm.md"),
        "<!-- archetypes:begin -->\nnever closed\n",
    );
    let res = build_variant(
        &VariantSpec {
            repo_root: t.path().to_path_buf(),
            ablations_dir: t.path().join("bench/ablations"),
            ablation: Ablation::Full,
            archetypes_off: true,
        },
        &t.path().join("v"),
    );
    assert!(res.unwrap_err().contains("without an end marker"));
}

/// The fake claude of the hook test: it finds the plugin directory in its arguments, runs
/// the command of the first hook of `hooks/hooks.json` the way Claude Code does (a shell
/// with `CLAUDE_PLUGIN_ROOT` set, in the project), and leaves a state and a bundle behind.
const FAKE_HOOKS: &str = r#"
cfg="$CLAUDE_CONFIG_DIR"
env | sort > "$cfg/env.txt"
plugin=""
prev=""
for a in "$@"; do
  if [ "$prev" = "--plugin-dir" ]; then plugin="$a"; fi
  prev="$a"
done
echo '{"type":"system","subtype":"init","model":"fake-model"}'
mkdir -p .dla/R1/bundle/B1 .dla/R1/integrity
echo R1 > .dla/current
echo '{"type":"finding_added","finding":{"id":"F1","site":"src/main.py:1"}}' > .dla/R1/ledger.jsonl
echo '{"id":"B1"}' > .dla/R1/bundle/B1/index.json
echo '{"manifest":true}' > .dla/R1/bundle/B1/manifest.json
echo 'excerpt one' > .dla/R1/bundle/B1/excerpt.txt
echo 'excerpt two' > .dla/R1/bundle/B1/excerpt-2.txt
echo 'raw' > .dla/R1/bundle/B1/raw.bin
echo '{"len":3,"sha256":"x","written_at":"t"}' > .dla/R1/integrity/ledger.jsonl.seal
echo 'not a seal' > .dla/R1/integrity/notes.txt
echo '{"incomplete_reason":"the stop hook let the run stop after 3 blocks","stop_blocks":3}' > .dla/R1/state.json
cmd=$(sed -n 's/.*"command":"\(.*\)","timeout.*/\1/p' "$plugin/hooks/hooks.json" | head -n 1 | sed 's/\\"/"/g')
CLAUDE_PLUGIN_ROOT="$plugin" sh -c "$cmd" > "$cfg/hook-out.txt" 2> "$cfg/hook-err.txt"
echo "exit=$?" >> "$cfg/hook-out.txt"
echo '{"type":"result","total_cost_usd":0.1,"num_turns":1,"usage":{}}'
"#;

/// The hook command is `"${CLAUDE_PLUGIN_ROOT}/bin/dla" ...`: the shim has to find the
/// binary the runner copied through `DLA_BIN`, from the environment the runner gives the
/// child, with nothing else set.
#[test]
fn a_hook_process_finds_the_dla_binary_through_the_child_environment() {
    let e = Env::new(FAKE_HOOKS);
    mini_repo(&e.root.join("repo"));
    // The real shim, so the test covers how it resolves the binary.
    write(
        &e.root.join("repo/bin/dla"),
        include_str!("../../../bin/dla"),
    );
    fs::set_permissions(
        e.root.join("repo/bin/dla"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    script(
        &e.root.join("fake-dla"),
        "echo \"args=$*\"\necho \"plugin_root=$DLA_PLUGIN_ROOT\"\necho \"data_dir=$DLA_DATA_DIR\"\nexit 7",
    );
    for (abl, hooks) in [("full", true), ("gates", false)] {
        let out = e
            .cmd(&[
                "--ablation",
                abl,
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
        assert!(
            out.status.success(),
            "{abl}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let work = e.work_dir(abl, "on", "fx-a", 1);
        let cfg = work.join("config");
        let envtxt = fs::read_to_string(cfg.join("env.txt")).unwrap();
        let copy = e.root.join("work/res/bin/dla");
        assert!(
            envtxt.contains(&format!("DLA_BIN={}", copy.display())),
            "{abl}: {envtxt}"
        );
        // The hook only runs where the variant has a hooks.json; `gates` has none, so the
        // fake finds no command there.
        let hook_out = fs::read_to_string(cfg.join("hook-out.txt")).unwrap_or_default();
        if hooks {
            let plugin = work.join("plugin");
            assert!(
                hook_out.contains("args=gate stop-hook --subagent"),
                "{hook_out}"
            );
            assert!(
                hook_out.contains(&format!("plugin_root={}", plugin.display())),
                "{hook_out}"
            );
            assert!(hook_out.contains("exit=7"), "{hook_out}");
            // Nothing but the child environment was needed: the shim exports the rest.
            assert!(hook_out.contains("data_dir="), "{hook_out}");
        } else {
            assert!(!hook_out.contains("args="), "{abl}: {hook_out}");
        }
    }
}

#[test]
fn a_run_collects_the_bundle_manifest_excerpts_seals_and_the_state_facts() {
    let e = Env::new(FAKE_HOOKS);
    mini_repo(&e.root.join("repo"));
    script(&e.root.join("fake-dla"), "exit 0");
    for abl in ["full", "no-skill"] {
        let mut args = vec![
            "--ablation",
            abl,
            "--runs",
            "1",
            "--fixtures",
            "fx-a",
            "--ablations-dir",
        ];
        let ab = e.root.join("repo/bench/ablations");
        args.push(ab.to_str().unwrap());
        let fake = e.root.join("fake-dla");
        args.extend(["--dla-bin", fake.to_str().unwrap()]);
        let out = e.cmd(&args).output().unwrap();
        assert!(
            out.status.success(),
            "{abl}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let rd = e.run_dir(abl, "on", "fx-a", 1);
        // Bundle: the index, the manifest and the excerpts, nothing else.
        let b = rd.join("bundle/B1");
        assert!(b.join("index.json").is_file(), "{abl}");
        assert_eq!(
            fs::read_to_string(b.join("manifest.json")).unwrap().trim(),
            "{\"manifest\":true}"
        );
        assert_eq!(
            fs::read_to_string(b.join("excerpt.txt")).unwrap().trim(),
            "excerpt one"
        );
        assert!(b.join("excerpt-2.txt").is_file());
        assert!(!b.join("raw.bin").exists());
        // Integrity seals, and only seals.
        assert!(rd.join("integrity/ledger.jsonl.seal").is_file(), "{abl}");
        assert!(!rd.join("integrity/notes.txt").exists());
        let rj = read_json(&rd.join("run.json"));
        if abl == "full" {
            assert_eq!(
                rj["incomplete_reason"],
                "the stop hook let the run stop after 3 blocks"
            );
            assert_eq!(rj["stop_blocks"], 3);
        } else {
            // No binary, no state: the fields are absent, not zero.
            assert!(rj.get("incomplete_reason").is_none(), "{rj}");
            assert!(rj.get("stop_blocks").is_none(), "{rj}");
        }
    }
}

#[test]
fn a_state_without_a_stop_block_count_reads_as_zero_blocks() {
    use dla_bench::run::read_state_facts;
    let t = tempfile::tempdir().unwrap();
    let p = t.path().join("state.json");
    assert_eq!(read_state_facts(&p), (None, None), "no file");
    fs::write(&p, "not json").unwrap();
    assert_eq!(read_state_facts(&p), (None, None), "not JSON");
    fs::write(&p, "{}").unwrap();
    assert_eq!(read_state_facts(&p), (None, Some(0)));
    fs::write(&p, r#"{"incomplete_reason":"r","stop_blocks":2}"#).unwrap();
    assert_eq!(read_state_facts(&p), (Some("r".to_owned()), Some(2)));
}

// ---- interrupts, lock, retry, budget, systematic failure, artifacts, errors

/// True when no live process has this pid: it does not exist, or it is a zombie that
/// only awaits its parent. `ps` answers on Linux and macOS alike (macOS has no `/proc`);
/// `/proc` is the fallback where `ps` is not installed.
fn gone(pid: &str) -> bool {
    match Command::new("ps").args(["-o", "stat=", "-p", pid]).output() {
        Ok(o) => {
            let state = String::from_utf8_lossy(&o.stdout);
            let state = state.trim();
            state.is_empty() || state.starts_with('Z')
        }
        Err(_) => {
            !Path::new(&format!("/proc/{pid}")).exists()
                || fs::read_to_string(format!("/proc/{pid}/stat"))
                    .map(|s| s.contains(") Z"))
                    .unwrap_or(true)
        }
    }
}

fn wait_for(what: &str, mut ok: impl FnMut() -> bool) {
    let start = Instant::now();
    while !ok() {
        assert!(
            start.elapsed().as_secs() < 20,
            "timed out waiting for {what}"
        );
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}

fn signal(pid: u32, sig: &str) {
    let ok = Command::new("kill")
        .args([sig, &pid.to_string()])
        .status()
        .unwrap()
        .success();
    assert!(ok);
}

const NO_FIXTURE_ARGS: [&str; 6] = [
    "--ablation",
    "no-skill",
    "--runs",
    "1",
    "--fixtures",
    "fx-a",
];

fn interrupted(sig: &str, expected_code: i32, fake: &str) -> (Env, std::time::Duration) {
    let e = Env::new(fake);
    let pidfile = e.root.join("child.pid");
    let mut runner = e
        .cmd(&NO_FIXTURE_ARGS)
        .env("FAKE_PID", &pidfile)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    wait_for("the child to start", || {
        fs::read_to_string(&pidfile).is_ok_and(|t| !t.trim().is_empty())
    });
    let pid = fs::read_to_string(&pidfile).unwrap().trim().to_owned();
    assert!(e.root.join("out/res/runner.lock").is_file());
    let sent = Instant::now();
    signal(runner.id(), sig);
    let status = loop {
        if let Some(s) = runner.try_wait().unwrap() {
            break s;
        }
        assert!(sent.elapsed().as_secs() < 30, "runner did not exit");
        std::thread::sleep(std::time::Duration::from_millis(25));
    };
    let took = sent.elapsed();
    assert_eq!(status.code(), Some(expected_code));
    wait_for("the background child to die", || gone(&pid));
    assert!(
        !e.run_dir("no-skill", "on", "fx-a", 1)
            .join("run.json")
            .exists(),
        "an interrupted run has no run.json"
    );
    assert!(
        !e.root.join("out/res/runner.lock").exists(),
        "lock released"
    );
    (e, took)
}

#[test]
fn sigterm_stops_every_child_group_and_exits_nonzero() {
    let (_e, took) = interrupted("-TERM", 143, "sleep 60 &\necho $! > \"$FAKE_PID\"\nwait");
    assert!(took.as_secs() < guard_grace_plus(), "{took:?}");
}

fn guard_grace_plus() -> u64 {
    dla_bench::guard::GRACE.as_secs() + 3
}

#[test]
fn sigint_escalates_to_sigkill_for_children_that_ignore_sigterm() {
    let (_e, took) = interrupted(
        "-INT",
        130,
        "trap '' TERM\nsleep 60 &\necho $! > \"$FAKE_PID\"\nwait",
    );
    assert!(
        took >= dla_bench::guard::GRACE,
        "SIGKILL only after the grace period: {took:?}"
    );
}

#[test]
fn a_second_runner_on_the_same_results_directory_is_refused() {
    let e = Env::new("sleep 60 &\necho $! > \"$FAKE_PID\"\nwait");
    let pidfile = e.root.join("child.pid");
    let mut first = e
        .cmd(&NO_FIXTURE_ARGS)
        .env("FAKE_PID", &pidfile)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    wait_for("the first runner", || pidfile.is_file());
    let out = e.cmd(&NO_FIXTURE_ARGS).output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("another dla-bench run") && err.contains(&first.id().to_string()),
        "{err}"
    );
    assert_eq!(e.calls(), 0, "the second runner started nothing");
    // the refused runner must not have removed the first one's lock
    assert!(e.root.join("out/res/runner.lock").is_file());
    signal(first.id(), "-TERM");
    first.wait().unwrap();
}

#[test]
fn a_stale_lock_is_replaced_and_released_after_the_run() {
    let e = Env::new(FAKE_OK);
    let mut dead = Command::new("true").spawn().unwrap();
    let dead_pid = dead.id();
    dead.wait().unwrap();
    write(
        &e.root.join("out/res/runner.lock"),
        &format!("{dead_pid}\n"),
    );
    let out = e.cmd(&NO_FIXTURE_ARGS).output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(e.calls(), 1);
    assert!(!e.root.join("out/res/runner.lock").exists());
    // a lock without a readable pid is not guessed at
    write(&e.root.join("out/res/runner.lock"), "garbage\n");
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--runs",
            "2",
            "--fixtures",
            "fx-a",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("without a pid"));
}

const FAKE_MODE: &str = r#"
echo called >> "$FAKE_LOG"
echo '{"type":"system","subtype":"init","model":"fake-model"}'
case "$FAKE_MODE" in
  maxturns)
    echo '{"findings":[]}' > findings.json
    mkdir -p .dla/R1; echo R1 > .dla/current; echo stale > .dla/R1/report.md
    echo '{"type":"result","subtype":"error_max_turns","is_error":true,"total_cost_usd":0.5,"num_turns":3,"usage":{"input_tokens":1}}'
    exit 1 ;;
  empty)
    echo '{"type":"result","subtype":"success","is_error":true,"total_cost_usd":0,"num_turns":0,"usage":{}}'
    exit 1 ;;
  isError)
    echo '{"type":"result","subtype":"success","is_error":true,"total_cost_usd":0.1,"num_turns":1,"usage":{"input_tokens":1}}'
    exit 1 ;;
  *)
    echo '{"type":"result","subtype":"success","is_error":false,"total_cost_usd":0.5,"num_turns":3,"usage":{"input_tokens":11}}' ;;
esac
"#;

fn read_json(p: &Path) -> serde_json::Value {
    serde_json::from_str(&fs::read_to_string(p).unwrap()).unwrap()
}

#[test]
fn result_failures_are_recorded_and_retry_errored_runs_again_with_a_clean_slate() {
    let e = Env::new(FAKE_MODE);
    let args = [
        "--ablation",
        "no-skill",
        "--runs",
        "1",
        "--fixtures",
        "fx-a",
    ];
    let out = e.cmd(&args).env("FAKE_MODE", "maxturns").output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let rd = e.run_dir("no-skill", "on", "fx-a", 1);
    let rj = read_json(&rd.join("run.json"));
    assert_eq!(rj["exit_code"], 1);
    assert_eq!(rj["error"], "result: error_max_turns");
    assert!(rd.join("findings.json").is_file());
    assert!(rd.join("report.md").is_file());

    // without the flag the errored run is final
    let out = e.cmd(&args).output().unwrap();
    assert!(out.status.success());
    assert_eq!(e.calls(), 1);
    assert!(String::from_utf8_lossy(&out.stdout).contains("already done"));

    // with it, the run is redone and nothing of the first attempt survives
    let out = e.cmd(&args).arg("--retry-errored").output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(e.calls(), 2);
    let rj = read_json(&rd.join("run.json"));
    assert_eq!(rj["exit_code"], 0);
    assert!(rj["error"].is_null());
    for stale in [
        "findings.json",
        "report.md",
        "ledger.jsonl",
        "state.json",
        "coverage.json",
    ] {
        assert!(!rd.join(stale).exists(), "{stale} of the earlier attempt");
    }

    // a clean run is not redone
    let out = e.cmd(&args).arg("--retry-errored").output().unwrap();
    assert!(out.status.success());
    assert_eq!(e.calls(), 2);
}

#[test]
fn is_error_without_a_failure_subtype_is_recorded_too() {
    let e = Env::new(FAKE_MODE);
    let out = e
        .cmd(&NO_FIXTURE_ARGS)
        .env("FAKE_MODE", "isError")
        .output()
        .unwrap();
    assert!(out.status.success());
    let rj = read_json(&e.run_dir("no-skill", "on", "fx-a", 1).join("run.json"));
    assert_eq!(rj["error"], "result: is_error");
}

#[test]
fn retry_errored_treats_timeouts_nonzero_exit_and_errors_as_unfinished() {
    let e = Env::new(FAKE_MODE);
    let base = [
        "--ablation",
        "no-skill",
        "--runs",
        "1",
        "--fixtures",
        "fx-a",
    ];
    let rd = e.run_dir("no-skill", "on", "fx-a", 1);
    let cases = [
        r#"{"exit_code":1,"timed_out":false,"error":null}"#,
        r#"{"exit_code":0,"timed_out":true,"error":null}"#,
        r#"{"exit_code":0,"timed_out":false,"error":"timed out"}"#,
        r#"{"timed_out":false,"error":null}"#,
        "not json",
    ];
    for (i, c) in cases.iter().enumerate() {
        write(&rd.join("run.json"), c);
        let out = e.cmd(&base).arg("--retry-errored").output().unwrap();
        assert!(out.status.success());
        assert_eq!(e.calls(), i + 1, "case {c}");
    }
    // finished and clean stays
    write(
        &rd.join("run.json"),
        r#"{"exit_code":0,"timed_out":false,"error":null,"cost_usd":0.5}"#,
    );
    let out = e.cmd(&base).arg("--retry-errored").output().unwrap();
    assert!(out.status.success());
    assert_eq!(e.calls(), cases.len());
}

#[test]
fn budget_counts_runs_finished_by_earlier_invocations() {
    let e = Env::new(FAKE_MODE);
    let common = [
        "--ablation",
        "no-skill",
        "--fixtures",
        "fx-a,fx-b",
        "--parallel",
        "1",
    ];
    let out = e.cmd(&common).args(["--runs", "1"]).output().unwrap();
    assert!(out.status.success());
    assert_eq!(e.calls(), 2, "0.5 + 0.5 spent");
    // 1.0 is already spent: with a limit of 1.2 only one more run fits
    let out = e
        .cmd(&common)
        .args(["--runs", "3", "--budget-usd", "1.2"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(e.calls(), 3, "one run (0.5) takes the total over 1.2");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("total $1.5000"), "{stdout}");
    assert!(
        stdout.contains("cost $0.5000 ($1.5000 in the results directory)"),
        "{stdout}"
    );
    // a limit that is already reached starts nothing
    let out = e
        .cmd(&common)
        .args(["--runs", "3", "--budget-usd", "1.5"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(e.calls(), 3);
}

#[test]
fn three_consecutive_empty_runs_abort_the_queue_and_retry_continues() {
    let e = Env::new(FAKE_MODE);
    let common = [
        "--ablation",
        "no-skill",
        "--fixtures",
        "fx-a,fx-b",
        "--runs",
        "4",
        "--parallel",
        "1",
    ];
    let out = e.cmd(&common).env("FAKE_MODE", "empty").output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("3 consecutive runs")
            && err.contains("systematic failure")
            && err.contains("--retry-errored"),
        "{err}"
    );
    assert!(err.contains("5 of 8 run(s) were not started"), "{err}");
    assert_eq!(e.calls(), 3);
    assert!(!e.root.join("out/res/runner.lock").exists());
    // the three failures are on disk; once the cause is fixed, retry redoes them and the rest
    let out = e.cmd(&common).arg("--retry-errored").output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(e.calls(), 3 + 8);
}

#[test]
fn an_empty_run_between_productive_ones_does_not_abort() {
    // the streak counts consecutive runs: errors that did burn tokens reset it
    let e = Env::new(FAKE_MODE);
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--fixtures",
            "fx-a,fx-b",
            "--runs",
            "3",
            "--parallel",
            "1",
        ])
        .env("FAKE_MODE", "isError")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(e.calls(), 6);
}

const FAKE_WORKERS: &str = r#"
cfg="$CLAUDE_CONFIG_DIR"
env | sort > "$cfg/env.txt"
echo '{"type":"system","subtype":"init","model":"fake-model"}'
mkdir -p .dla/R1/bundle/B1 .dla/R1/bundle/B2/raw
echo R1 > .dla/current
echo '{"type":"finding_added","finding":{"id":"jvm/F1","site":"src/main.py:1"}}' > .dla/R1/ledger.jsonl
echo '{"type":"slice_started","worker":"jvm"}' > .dla/R1/ledger.jvm.jsonl
echo '{"type":"slice_started","worker":"rust"}' > .dla/R1/ledger.rust.jsonl
echo '{"type":"x"}' > .dla/R1/ledger.notes.txt
echo '{"id":"B1"}' > .dla/R1/bundle/B1/index.json
echo '{"id":"B2"}' > .dla/R1/bundle/B2/index.json
echo 'raw evidence' > .dla/R1/bundle/B2/raw/heap.txt
echo '{}' > .dla/R1/state.json
echo '{"type":"result","total_cost_usd":0.1,"num_turns":1,"usage":{}}'
"#;

#[test]
fn worker_ledgers_and_bundle_indexes_are_collected_and_the_budget_is_set() {
    let e = Env::new(FAKE_WORKERS);
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--fixtures",
            "fx-a",
            "--runs",
            "1",
            "--keep-work",
            "--child-budget",
            "7m",
        ])
        .env("DLA_DEFAULT_BUDGET", "9h")
        .env("DLA_WORKER", "parent-worker")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let rd = e.run_dir("no-skill", "on", "fx-a", 1);
    assert!(rd.join("ledger.jsonl").is_file());
    assert!(rd.join("ledger.jvm.jsonl").is_file());
    assert!(rd.join("ledger.rust.jsonl").is_file());
    assert!(!rd.join("ledger.notes.txt").exists());
    assert_eq!(
        fs::read_to_string(rd.join("bundle/B1/index.json"))
            .unwrap()
            .trim(),
        "{\"id\":\"B1\"}"
    );
    assert!(rd.join("bundle/B2/index.json").is_file());
    assert!(
        !rd.join("bundle/B2/raw").exists(),
        "only index files are collected"
    );
    let cfg = e.work_dir("no-skill", "on", "fx-a", 1).join("config");
    let envtxt = fs::read_to_string(cfg.join("env.txt")).unwrap();
    assert!(envtxt.contains("DLA_DEFAULT_BUDGET=7m"), "{envtxt}");
    assert!(
        !envtxt.contains("DLA_DEFAULT_BUDGET=9h") && !envtxt.contains("DLA_WORKER"),
        "{envtxt}"
    );

    // A second attempt does not keep files of the first one.
    fs::remove_file(rd.join("run.json")).unwrap();
    fs::write(rd.join("ledger.stale.jsonl"), "x\n").unwrap();
    fs::create_dir_all(rd.join("bundle/OLD")).unwrap();
    fs::write(rd.join("bundle/OLD/index.json"), "x").unwrap();
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--fixtures",
            "fx-a",
            "--runs",
            "1",
        ])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(!rd.join("ledger.stale.jsonl").exists());
    assert!(!rd.join("bundle/OLD").exists());
    assert!(rd.join("ledger.jvm.jsonl").is_file());
}

#[test]
fn an_invalid_child_budget_is_refused_before_anything_starts() {
    let e = Env::new(FAKE_OK);
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--fixtures",
            "fx-a",
            "--child-budget",
            "soon",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("--child-budget") && err.contains("20m"),
        "{err}"
    );
    assert_eq!(e.calls(), 0);
}

#[test]
fn the_default_work_root_is_in_the_cache_directory() {
    let e = Env::new(FAKE_OK);
    let xdg = e.root.join("xdg-cache");
    let mut c = Command::new(env!("CARGO_BIN_EXE_dla-bench"));
    c.arg("run")
        .arg("--fixtures-dir")
        .arg(e.root.join("fx"))
        .arg("--out")
        .arg(e.root.join("out/res"))
        .arg("--claude-bin")
        .arg(e.root.join("claude"))
        .arg("--repo-root")
        .arg(e.root.join("repo"))
        .args([
            "--ablation",
            "no-skill",
            "--fixtures",
            "fx-a",
            "--runs",
            "1",
            "--keep-work",
        ])
        .env("FAKE_LOG", e.root.join("calls.log"))
        .env("XDG_CACHE_HOME", &xdg);
    let out = c.output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        xdg.join("dla-bench/res/no-skill/archetypes-on/fx-a/1/project/src/main.py")
            .is_file()
    );
    // Without XDG_CACHE_HOME the directory comes from HOME.
    let home = e.root.join("home");
    fs::remove_dir_all(e.root.join("out")).unwrap();
    let mut c = Command::new(env!("CARGO_BIN_EXE_dla-bench"));
    c.arg("run")
        .arg("--fixtures-dir")
        .arg(e.root.join("fx"))
        .arg("--out")
        .arg(e.root.join("out/res"))
        .arg("--claude-bin")
        .arg(e.root.join("claude"))
        .arg("--repo-root")
        .arg(e.root.join("repo"))
        .args([
            "--ablation",
            "no-skill",
            "--fixtures",
            "fx-a",
            "--runs",
            "1",
            "--keep-work",
        ])
        .env("FAKE_LOG", e.root.join("calls.log"))
        .env_remove("XDG_CACHE_HOME")
        .env("HOME", &home);
    let out = c.output().unwrap();
    assert!(out.status.success());
    assert!(
        home.join(".cache/dla-bench/res/no-skill/archetypes-on/fx-a/1/project")
            .is_dir()
    );
    // A work root inside a denied path is still refused.
    let out = e
        .cmd(&[
            "--ablation",
            "no-skill",
            "--fixtures",
            "fx-a",
            "--work-root",
            e.root.join("out/res/inside").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("inside the denied path"));
}
