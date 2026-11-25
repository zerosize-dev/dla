//! A holder that is kept per thread grows only with the threads the units come from. Two
//! classes keep a list that a thread-local initialiser adds to: one clears the thread-local
//! after every use (the initialiser runs again for the same live thread), the other never does
//! (one entry per thread). A first pass whose units run on one thread reads both flat, which
//! settles neither: both wait for a second look whose unit takes its threads from the key. The
//! driver block names the thread forms, a second-look unit that leaves the `threads` slot out
//! is refused, and with the units on a fixed pool of four threads the first class grows and the
//! second stays at four entries.
//!
//! Skips, printing the reason, when `java`, `javac` or `sh` is missing.

#![cfg(unix)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command as Std, Stdio},
};

use assert_cmd::Command;
use serde_json::Value;

fn has(tool: &str) -> bool {
    Std::new(tool)
        .arg("-version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn dla(project: &Path, args: &[&str]) -> (i32, Value, String) {
    let mut c = Command::cargo_bin("dla").unwrap();
    c.arg("--project").arg(project);
    for var in [
        "DLA_RUN",
        "DLA_DEFAULT_BUDGET",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "DLA_DATA_DIR",
    ] {
        c.env_remove(var);
    }
    let o = c.args(args).output().unwrap();
    let text = String::from_utf8(o.stdout).unwrap();
    (
        o.status.code().unwrap(),
        serde_json::from_str(&text).unwrap_or(Value::Null),
        text,
    )
}

fn ok(project: &Path, args: &[&str]) -> Value {
    let (code, json, text) = dla(project, args);
    assert_eq!(code, 0, "{args:?}: {text}");
    assert_eq!(json["ok"], true, "{text}");
    json
}

fn write(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

const BUILD: &str = "#!/bin/sh\nset -e\ncd \"$(dirname \"$0\")\"\nrm -rf out\nmkdir -p out\njavac -d out src/threads/*.java\nif [ \"$1\" = \"run\" ]; then\n    java -cp out threads.Main \"${2:-100}\"\nfi\n";

/// One of the two classes: `clears` makes `use()` remove the thread-local after reading it.
fn class(name: &str, clears: bool) -> String {
    let remove = if clears {
        "\n        own.remove();"
    } else {
        ""
    };
    format!(
        r#"package threads;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

public class {name} {{
    final List<Object> live = Collections.synchronizedList(new ArrayList<>());
    private final ThreadLocal<Object> own = ThreadLocal.withInitial(this::init);

    private Object init() {{
        Object entry = new byte[256];
        live.add(entry);
        return entry;
    }}

    public void use() {{
        own.get();{remove}
    }}

    public int size() {{
        return live.size();
    }}
}}
"#
    )
}

const MAIN: &str = "package threads;\n\npublic final class Main {\n    public static void main(String[] args) {\n        Churn c = new Churn();\n        Steady s = new Steady();\n        for (int i = 0; i < 100; i++) {\n            c.use();\n            s.use();\n        }\n        System.out.println(c.size() + \" \" + s.size());\n    }\n}\n";

struct Scene {
    _dir: tempfile::TempDir,
    project: PathBuf,
    run_dir: PathBuf,
}

fn scene() -> Scene {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("project");
    write(&project, "build.sh", BUILD);
    write(&project, "src/threads/Churn.java", &class("Churn", true));
    write(&project, "src/threads/Steady.java", &class("Steady", false));
    write(&project, "src/threads/Main.java", MAIN);
    let init = ok(&project, &["run", "init", "--session-id", "t"]);
    let run_dir = project.join(".dla").join(init["run_id"].as_str().unwrap());
    ok(&project, &["consent", "--tiers", "1-2"]);
    ok(&project, &["census", "--lang", "jvm"]);
    ok(&project, &["workload", "discover", "--lang", "jvm"]);
    Scene {
        _dir: dir,
        project,
        run_dir,
    }
}

impl Scene {
    fn driver(&self, h: &str) -> PathBuf {
        self.run_dir
            .join("harness")
            .join(h)
            .join("driver/DlaLoop.java")
    }

    /// Fills the hooks of harness `h`: `setup` and `unit` replace the not-filled-in lines.
    fn fill(&self, h: &str, setup: &str, unit: &str) {
        let p = self.driver(h);
        let text = fs::read_to_string(&p).unwrap();
        let text = text
            .replace(
                "        throw new UnsupportedOperationException(\"DLA-HOOK setup has not been filled in\");",
                setup,
            )
            .replace(
                "        throw new UnsupportedOperationException(\"DLA-HOOK unit has not been filled in\");",
                unit,
            )
            .replace(
                "    private DlaLoop() {}",
                "    private DlaLoop() {}\n\n    static Churn churn;\n    static Steady steady;\n    static java.util.concurrent.ExecutorService pool;",
            );
        assert!(!text.contains("has not been filled in"));
        fs::write(p, text).unwrap();
    }

    fn rows(&self) -> Vec<Value> {
        let (code, _, text) = dla(&self.project, &["ledger", "list", "--all"]);
        assert_eq!(code, 0, "{text}");
        text.lines()
            .skip(1)
            .map(|l| serde_json::from_str(l).unwrap())
            .collect()
    }

    fn row(&self, owner: &str) -> Value {
        self.rows()
            .into_iter()
            .find(|r| r["owner"] == owner)
            .unwrap_or_else(|| panic!("no finding of {owner}"))
    }
}

const ONE_THREAD_SETUP: &str = "        churn = new Churn();\n        steady = new Steady();";
const ONE_THREAD_UNIT: &str = "        churn.use();\n        steady.use();";

#[test]
fn a_list_kept_per_thread_waits_for_a_second_look_that_varies_the_threads() {
    if !(has("java") && has("javac")) {
        eprintln!("skipping: java or javac is not available");
        return;
    }
    let s = scene();
    // The census marks both lists: the class owns a thread-local, and the writer is the
    // initialiser that the thread-local runs.
    let census: Value =
        serde_json::from_str(&fs::read_to_string(s.run_dir.join("census/jvm.json")).unwrap())
            .unwrap();
    for e in census["entries"].as_array().unwrap() {
        if e["holder"] == "container-field" {
            assert_eq!(e["thread_class"], true, "{e}");
            assert_eq!(e["writer_sites"][0]["thread_init"], true, "{e}");
        }
    }

    // First pass: every unit on the caller's thread. The block already names the thread forms.
    ok(
        &s.project,
        &[
            "harness",
            "gen",
            "--lang",
            "jvm",
            "--driver",
            "--package",
            "threads",
            "--next",
            "4",
        ],
    );
    let block = fs::read_to_string(s.driver("H1")).unwrap();
    for needle in [
        "// Threads: a target marked `threads:` below is kept per thread",
        "not only from the caller's thread",
        "common fork-join pool",
        "executor service with a few threads that live across units",
        "new thread per unit that ends",
        "//   threads: this holder is kept per thread (see Threads above)",
    ] {
        assert!(block.contains(needle), "{needle}\n{block}");
    }
    s.fill("H1", ONE_THREAD_SETUP, ONE_THREAD_UNIT);
    let run = ok(&s.project, &["harness", "run", "H1", "--mode", "fresh"]);
    let series = run["summary"]["series"][0].as_str().unwrap().to_owned();
    let trend = ok(&s.project, &["trend", &series]);
    for t in trend["targets"].as_array().unwrap() {
        assert_eq!(t["verdict"], "executed", "{trend}");
    }

    // Neither list is settled: each waits for its second look, with the reason, while the
    // queue is open and `close-rest` leaves them.
    for owner in ["threads.Churn.live", "threads.Steady.live"] {
        let row = s.row(owner);
        assert_eq!(row["queue"], "pending", "{row}");
        let why = row["second_look"].as_str().unwrap();
        assert!(
            why.contains("kept per thread; the units came from one thread"),
            "{row}"
        );
    }
    let st = ok(&s.project, &["run", "status"]);
    assert_eq!(st["queue"]["jvm"]["pending"], 2, "{st}");
    assert_eq!(st["queue"]["jvm"]["second_look"], true, "{st}");
    let rest = ok(&s.project, &["ledger", "close-rest"]);
    let left: Vec<&str> = rest["left"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["id"].as_str().unwrap())
        .collect();
    assert_eq!(left, ["F1", "F3"], "{rest}");
    assert_eq!(s.row("threads.Churn.live")["state"], "candidate");
    assert_eq!(s.row("threads.Steady.live")["state"], "candidate");

    // The second look: the block names the sixth slot, the skeleton has it.
    let gen2 = ok(
        &s.project,
        &[
            "harness",
            "gen",
            "--lang",
            "jvm",
            "--driver",
            "--package",
            "threads",
            "--next",
            "3",
        ],
    );
    assert!(
        gen2["summary"]["focus"]
            .as_str()
            .unwrap()
            .starts_with("second look:"),
        "{gen2}"
    );
    let block = fs::read_to_string(s.driver("H2")).unwrap();
    for needle in [
        "common fork-join pool",
        "starts with six slot variables",
        "depth, ending, when, order, outcome, threads",
        "the threads slot cannot be given up",
        "(kept per thread; the units came from one thread): read the unit of that harness",
        "int threads = choice(key, 5, 4); // slot threads:",
    ] {
        assert!(block.contains(needle), "{needle}\n{block}");
    }
    fs::write(
        s.driver("H2").with_file_name("VARIES.txt"),
        "F1: runs the unit on pooled threads\nF3: runs the unit on pooled threads\n",
    )
    .unwrap();

    // A unit that uses the five slots of every second look and leaves the threads out is
    // refused, naming the slot.
    let sizes = s.project.join("sizes.txt");
    let setup = format!(
        "        churn = new Churn();\n        steady = new Steady();\n        pool = java.util.concurrent.Executors.newFixedThreadPool(4);\n        Runtime.getRuntime().addShutdownHook(new Thread(() -> {{\n            try {{ java.nio.file.Files.writeString(java.nio.file.Path.of(\"{}\"), churn.size() + \" \" + steady.size()); }} catch (Exception e) {{ }}\n        }}));",
        sizes.display()
    );
    let unit = |mix: &str| {
        format!(
            "        int mix = {mix};\n        pool.submit(() -> {{ churn.use(); steady.use(); }}).get();\n        if (mix < 0) throw new IllegalStateException();"
        )
    };
    s.fill(
        "H2",
        &setup,
        &unit("depth + ending + when + order + outcome"),
    );
    let (code, json, text) = dla(&s.project, &["harness", "run", "H2", "--mode", "fresh"]);
    assert_ne!(code, 0, "{text}");
    assert_eq!(json["error"]["code"], "invalid-value", "{text}");
    let msg = json["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("does not use slot variable(s): threads") && msg.contains("kept per thread"),
        "{text}"
    );

    // With the threads used and the units on a fixed pool of four threads, the list that the
    // thread-local clears grows with the units and the other stays at four entries.
    let p = s.driver("H2");
    let text = fs::read_to_string(&p).unwrap().replace(
        "int mix = depth + ending + when + order + outcome;",
        "int mix = depth + ending + when + order + outcome + threads;",
    );
    fs::write(&p, text).unwrap();
    let run = ok(&s.project, &["harness", "run", "H2", "--mode", "fresh"]);
    let series = run["summary"]["series"][0].as_str().unwrap().to_owned();
    ok(&s.project, &["trend", &series]);
    let sizes = fs::read_to_string(&sizes).unwrap();
    let (churn, steady) = sizes.split_once(' ').unwrap();
    assert!(churn.parse::<u32>().unwrap() >= 100, "{sizes}");
    assert_eq!(steady, "4", "{sizes}");
    // The second look has run: both are settled for good.
    for owner in ["threads.Churn.live", "threads.Steady.live"] {
        let row = s.row(owner);
        assert_eq!(row["queue"], "settled", "{row}");
        assert_eq!(row["second_look"], "done in H2", "{row}");
    }
}
