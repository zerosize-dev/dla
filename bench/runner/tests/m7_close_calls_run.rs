//! The command `dla-bench close-calls` prints adds runs 4 and 5 and touches no other run:
//! `dla-bench run --runs 5` on a results directory that holds runs 1 to 3 starts two runs.
#![cfg(unix)]

use std::{fs, os::unix::fs::PermissionsExt, path::Path, process::Command};

fn write(p: &Path, text: &str) {
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

const FAKE: &str = r#"#!/bin/sh
echo called >> "$FAKE_LOG"
echo '{"type":"system","subtype":"init","model":"fake-model"}'
echo '{"findings":[{"site":"src/main.py:1","verdict":"not-a-leak","discriminator":"flat"}]}' > findings.json
echo '{"type":"result","total_cost_usd":0.5,"num_turns":3,"usage":{"input_tokens":1,"output_tokens":2,"cache_read_input_tokens":3,"cache_creation_input_tokens":4}}'
"#;

#[test]
fn runs_five_adds_runs_four_and_five_and_leaves_the_first_three_alone() {
    let t = tempfile::Builder::new()
        .prefix("close-calls-")
        .tempdir_in(env!("CARGO_TARGET_TMPDIR"))
        .unwrap();
    let root = t.path();
    let fx = root.join("fx/fx-a");
    write(&fx.join("project/src/main.py"), "print('hi')\n");
    write(
        &fx.join("expected.toml"),
        "schema_version = 1\nid = \"fx-a\"\nruntime = \"python\"\ncategory = \"non-leak\"\ndynamic_confirmation = true\n[task]\nprompt = \"p\"\n[[non_leak]]\nid = \"N1\"\ncause = \"N-WARM\"\ndiscriminator = \"d\"\n",
    );
    Command::new("git")
        .args(["init", "-q"])
        .current_dir(root.join("fx"))
        .status()
        .unwrap();
    write(&root.join("claude"), FAKE);
    fs::set_permissions(root.join("claude"), fs::Permissions::from_mode(0o755)).unwrap();
    fs::create_dir_all(root.join("repo")).unwrap();
    let run = |runs: &str| {
        Command::new(env!("CARGO_BIN_EXE_dla-bench"))
            .args([
                "run",
                "--ablation",
                "no-skill",
                "--runs",
                runs,
                "--fixtures",
                "fx-a",
            ])
            .arg("--fixtures-dir")
            .arg(root.join("fx"))
            .arg("--out")
            .arg(root.join("out"))
            .arg("--work-root")
            .arg(root.join("work"))
            .arg("--claude-bin")
            .arg(root.join("claude"))
            .arg("--repo-root")
            .arg(root.join("repo"))
            .env("FAKE_LOG", root.join("calls.log"))
            .env("CLAUDE_CODE_SESSION_ID", "parent-session")
            .env("CLAUDE_SESSION_ID", "parent-session")
            .output()
            .unwrap()
    };
    let calls = || fs::read_to_string(root.join("calls.log")).map_or(0, |t| t.lines().count());
    let out = run("3");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(calls(), 3);
    let dir = |n: u32| root.join(format!("out/runs/no-skill/archetypes-on/fx-a/{n}"));
    let before: Vec<String> = (1..=3)
        .map(|n| fs::read_to_string(dir(n).join("run.json")).unwrap())
        .collect();
    let out = run("5");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(calls(), 5, "only runs 4 and 5 start");
    assert!(dir(4).join("run.json").is_file() && dir(5).join("run.json").is_file());
    let after: Vec<String> = (1..=3)
        .map(|n| fs::read_to_string(dir(n).join("run.json")).unwrap())
        .collect();
    assert_eq!(before, after);
}
