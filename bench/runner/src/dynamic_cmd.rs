//! `dla-bench dynamic`: the M3 acceptance driver.
//!
//! Like `static` it runs outside the sandbox and without a model. Every
//! fixture is copied to a scratch directory, a run is created with the
//! runner's consent for tiers 1-4, the static stages run (`probe --build`,
//! `sweep` and `census` per detected runtime, `workload discover`) and then a
//! fixed policy drives the dynamic commands. The policy is identical for every
//! fixture and reads only the outputs of `dla`; `expected.toml` is read after
//! the run to grade the ledger with the grader of the benchmark
//! ([`crate::grade`]).
//!
//! # The policy
//!
//! 1. For every detected runtime `dla harness gen --lang L` (default
//!    workload; when gen refuses with `workload-required` or
//!    `iteration-arg-unknown`, or the harness does not run, the next main or
//!    script workload of the discovery that has an iteration argument, at most
//!    two workloads) and `dla harness run H`. Without any harness that ran,
//!    every leak-side candidate is resolved `suspected_static_only`
//!    (`build_fails` when the probe's build of the runtime failed, else
//!    `no_harness`) and the policy goes to the grading.
//! 2. `dla trend S` per series. For every passed column with candidates, the
//!    first candidate (the trend lists the column's own finding first; N-*
//!    siblings are never listed) is chosen once per run. For each chosen
//!    leak-side finding F: add an N-WARM sibling, `ledger test F`,
//!    `dla alloc-test H --finding F`. A verdict that favors leak refutes the
//!    sibling, annotates the holder of F when it has none (the holder of the
//!    census entry at the same site, else `blocked-stack` for a goroutine or
//!    thread column, `registration-table` for a timer, listener or handle
//!    column, else `static/global`) and confirms F once step 5 ran (the
//!    confirmation cites the summarize evidence that ties a passing column
//!    to F, so the leak is dump backed); a verdict that favors non-leak
//!    refutes F; an inconclusive one resolves F `unresolved`.
//! 3. The first candidate of every non-passing, non-container column that no
//!    passed column tied to a chosen finding: N-WARM sibling, test,
//!    alloc-test; refuted when the verdict favors non-leak, else unresolved.
//!
//!    Then, when no passed column had candidates: one unbound
//!    `dla alloc-test H`; when it favors non-leak, every leak-side candidate
//!    with a site that is still open is refuted with it (nothing the process
//!    measures grows).
//! 4. When no passed column had candidates but a non-container column
//!    passed: a finding with origin `growth-series` (no site), N-HEADROOM and
//!    N-ALLOC siblings, test, an unbound alloc-test; refuted when the verdict
//!    favors non-leak, else unresolved.
//! 5. The per-operation steps (module `ops`), for every harness that grew:
//!    `dla summarize --list`; a JVM harness runs again with `--capture all`, a
//!    rust or native harness with `--tool valgrind` and `--tool heaptrack`
//!    (those with listed adapters; a missing tool refuses and leaves a coverage
//!    row); every `dla summarize` command the harness printed runs once. Each
//!    call is checked (stdout within 2048 bytes, `raw` with a matching sha256,
//!    evidence ids in the ledger, evidence results within 2048 bytes, a `tie`
//!    reason on every tied column); a violation fails the fixture. The policy
//!    never names a backend: it reads `--list` and the printed commands.
//!    Driver mode needs hooks filled in by judgement, so the policy stays in
//!    command mode.
//! 6. `dla gap S --residual` (refusals ignored); every leak-side candidate or
//!    finding under test left over is resolved `suspected_static_only`
//!    (`not_exercised`) when it has static evidence, else `unresolved`
//!    (`time_budget`). Finally `dla report`.
//!
//! At most [`MAX_ALLOC_TESTS`] alloc-tests run per fixture; the findings that
//! do not get one are resolved `unresolved` (`time_budget`).
//!
//! # Checks per fixture
//!
//! Leak fixtures with `dynamic_confirmation = true`: every confirmable leak
//! is found (the grader's `leak_outcomes`); decoy-control and non-leak
//! fixtures: no claim; non-leak: at least one rule-out; impossible: no claim;
//! every confirmed finding satisfies the M3 rules (re-read from the ledger: a
//! test preceded the confirmation, a refuted N-* sibling exists, the cited
//! growth evidence has gate pass, site and holder are set); the unexplained
//! growth share is computable when a residual row exists.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    env, fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, mpsc},
    time::{Duration, Instant},
};

use clap::Args;
use serde::Serialize;
use serde_json::Value;

use crate::{
    Res,
    fixture::{fixture_dirs, load_expected},
    grade::{Finding, Source, State, findings_from_ledger, grade_findings, site_matches},
    model::{Category, Entry, Expected},
    run::{Archetypes, apply_archetypes_off, collect_run_files, copy_tree},
};

#[path = "dynamic/artifact.rs"]
mod artifact;
#[path = "dynamic/ops.rs"]
mod ops;
#[path = "dynamic/without.rs"]
pub mod without;
#[path = "dynamic/workers.rs"]
mod workers;

pub use workers::{MergeRow, WorkerRow};

/// Alloc-tests run per fixture at most (each re-runs the harness).
pub const MAX_ALLOC_TESTS: usize = 8;

/// Workloads tried per runtime at most.
const MAX_WORKLOADS: usize = 2;

/// Arguments of `dla-bench dynamic`.
#[derive(Args, Debug, Clone)]
pub struct DynamicArgs {
    /// Directory with the fixtures.
    #[arg(long, default_value = "bench/fixtures")]
    pub fixtures: PathBuf,
    /// Whether archetype data (archetype tables, known leaks, suggested archetypes) is present.
    #[arg(long, value_enum, default_value = "on")]
    pub archetypes: Archetypes,
    /// The dla binary (default: next to dla-bench, else target/debug/dla, else dla on PATH).
    #[arg(long)]
    pub dla_bin: Option<PathBuf>,
    /// Write the full result as JSON to this file.
    #[arg(long)]
    pub json: Option<PathBuf>,
    /// Repository root holding data/ (the plugin data files).
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Only fixtures whose id contains this text.
    #[arg(long)]
    pub only: Option<String>,
    /// Fixtures run at the same time (harness runs take 10-90 s each).
    #[arg(long, default_value_t = 2)]
    pub parallel: usize,
    /// Directory for the per-fixture logs (default: next to --json, else the temp dir).
    #[arg(long)]
    pub log_dir: Option<PathBuf>,
    /// Keep the scratch copies of the fixtures.
    #[arg(long)]
    pub keep_work: bool,
    /// Longest time one dla command may run, in seconds; its process group is killed after
    /// that and the fixture fails.
    #[arg(long, default_value_t = DEFAULT_COMMAND_TIMEOUT_S)]
    pub command_timeout: u64,
    /// Hide these tools (comma separated) from every dla call: PATH becomes a directory of
    /// links to every other executable, and the tool-home variables (JAVA_HOME and the
    /// like) are unset. Each run must still complete, and the coverage row of a hidden
    /// backend must read `missing`.
    #[arg(long, value_delimiter = ',')]
    pub without: Vec<String>,
    /// Check `dump_backed` leaks: the chosen confirmed finding must cite retention or
    /// allocation evidence with dump or live authority (the summarize steps of the policy
    /// produce it). On by default; `--check-dump-backed=false` turns it off.
    #[arg(long, default_value_t = CHECK_DUMP_BACKED)]
    pub check_dump_backed: bool,
}

/// Default of `--check-dump-backed`: on, because the policy runs the summarize steps
/// (heap-graph, histogram, alloc-stack) before it confirms.
pub const CHECK_DUMP_BACKED: bool = true;

/// Default for `--command-timeout`.
pub const DEFAULT_COMMAND_TIMEOUT_S: u64 = 900;

/// The run budget the driver gives each fixture's run (a run is cut off at its budget, so
/// this stays well above the per-command limits).
const RUN_BUDGET: &str = "2h";

/// A fixture the acceptance is known to fail, and why. The policy of this driver is
/// mechanical (it cannot read the code), so a leak whose evidence is only reachable by
/// judgement is listed here instead of being weakened away. A listed fixture is reported as
/// `xfail` and does not fail the run. The run fails when a listed fixture passes (the entry is
/// stale) or fails in any other way (a regression hiding behind the entry).
pub struct ExpectedFailure {
    /// Fixture id.
    pub fixture: &'static str,
    /// The exact failure text the fixture produces.
    pub failure: &'static str,
    /// Why this failure is accepted.
    pub reason: &'static str,
}

/// Fixtures expected to fail; see [`ExpectedFailure`] and bench/README.md.
pub const EXPECTED_FAILURES: &[ExpectedFailure] = &[
    ExpectedFailure {
        fixture: "rust-tree-parent-links",
        failure: "leak L1 (src/section.rs:27) is not confirmed",
        reason: "the fixed policy confirms the `children` edge of the Rc cycle (section.rs:11) while the fixture expects the `parent` back-link (section.rs:27); choosing the edge to weaken needs reading the code, which the mechanical policy cannot do",
    },
    ExpectedFailure {
        fixture: "node-retry-scheduler",
        failure: "leak L1 (src/scheduler.js:31) is confirmed with holder container-field, expected static/global",
        reason: "the leak is confirmed through the probe column of `RetryScheduler.latest` (an alternative site, scheduler.js:15), and the census classifies an instance field as `container-field`; the fixture expects the module-level `shared` that roots it (`static/global`). The census does not follow an instance back to its static root, the heap-graph holder needs a series of snapshots from one process (driver mode, hand-filled hooks) which the command-mode capture of the fixed policy does not give, and the policy cannot read the code to choose the root",
    },
    ExpectedFailure {
        fixture: "node-socket-heartbeat",
        failure: "leak L1 (src/heartbeat.js:15) is confirmed with holder blocked-stack, expected registration-table",
        reason: "the census classifies the `setInterval` timer at heartbeat.js:15 as `blocked-stack` (non-heap) and the policy takes that holder from the census entry at the site; the fixture expects the table the sessions register in (`registration-table`, heap). The timer census does not know which table keeps the timer reachable; node-resources ties the timer group to the finding by site, but only the heap-graph holder (driver mode) names the table, and the policy does not annotate a holder it did not see",
    },
];

fn is_zero(n: &usize) -> bool {
    *n == 0
}

/// The result for one fixture.
#[derive(Serialize, Debug, Clone, Default)]
pub struct FixtureRow {
    /// Fixture id.
    pub id: String,
    /// Fixture category.
    pub category: String,
    /// Runtimes the probe detected.
    pub runtimes: Vec<String>,
    /// Harness ids that ran.
    pub harnesses: Vec<String>,
    /// Series written by the harness runs.
    pub series: Vec<String>,
    /// Passed columns over all trends.
    pub passed_columns: usize,
    /// Passed columns with candidates.
    pub passed_with_candidates: usize,
    /// Confirmed findings (without hypotheses).
    pub confirmed: usize,
    /// Refuted findings (without hypotheses).
    pub refuted: usize,
    /// The grader sees a claim.
    pub claims: bool,
    /// The grader sees a rule-out.
    pub rule_outs: bool,
    /// Expected confirmable leaks the ledger confirms.
    pub leaks_found: usize,
    /// Expected confirmable leaks.
    pub leaks_expected: usize,
    /// Residual over total of the residual rows, when present.
    pub unexplained_growth_share: Option<f64>,
    /// The entry path of the fixture (`repository`, `artifact` or `symptom`).
    pub entry: String,
    /// Coverage rows of hidden tools seen in a `--without` run.
    #[serde(skip_serializing_if = "is_zero")]
    pub hidden_backend_rows: usize,
    /// The same count per hidden tool (not serialized).
    #[serde(skip)]
    pub hidden_rows_by_tool: BTreeMap<String, usize>,
    /// Workers of a multi-runtime fixture and what the merge took from each.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub workers: Vec<WorkerRow>,
    /// The merge of a multi-runtime fixture.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge: Option<MergeRow>,
    /// Capture runs of the summarize steps (`H1:capture-all`, `H1:tool-valgrind`).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub captures: Vec<String>,
    /// Summarize calls that succeeded, as `<op>:<backend>`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ops: Vec<String>,
    /// Evidence ids the summarize calls wrote.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub summarize_evidence: Vec<String>,
    /// Largest stdout of a summarize call, in bytes.
    #[serde(skip_serializing_if = "is_zero")]
    pub max_stdout: usize,
    /// What the policy did (one line per decision), for the log and the JSON.
    pub steps: Vec<String>,
    /// The measurements behind the decisions: passing trend columns and alloc-test column
    /// verdicts, printed with a failing fixture's decisions (CI logs carry no run dirs).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<String>,
    /// Commands that failed or were refused.
    pub errors: Vec<String>,
    /// Reasons this fixture fails the acceptance.
    pub failures: Vec<String>,
    /// Why the fixture was not run (its entry path has no source tree yet).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<String>,
    /// The reason this fixture is on the expected-failure list, when it failed as listed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xfail: Option<String>,
    /// The failures that the expected-failure list accepted.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub expected_failures: Vec<String>,
}

/// The whole result.
#[derive(Serialize, Debug, Clone)]
pub struct DynamicResult {
    /// `on` or `off`.
    pub archetypes: String,
    /// Fixtures run at the same time.
    pub parallel: usize,
    /// Directory with the logs.
    pub log_dir: String,
    /// Per fixture.
    pub fixtures: Vec<FixtureRow>,
    /// Failures over all fixtures.
    pub failures: Vec<String>,
}

// ---------------------------------------------------------------------------
// Running dla
// ---------------------------------------------------------------------------

/// The result of one dla command.
struct Out {
    /// Exit 0 (and `ok: true` for the commands that print JSON).
    ok: bool,
    /// Parsed stdout (null when it was not JSON).
    json: Value,
    /// Bytes of stdout.
    bytes: usize,
    /// Exit code (-1 when killed or not started).
    exit: i32,
}

impl Out {
    /// The refusal code, when the command refused.
    fn code(&self) -> &str {
        self.json["error"]["code"].as_str().unwrap_or("")
    }

    fn evidence(&self) -> Vec<String> {
        strings(&self.json["evidence"])
    }

    fn message(&self) -> String {
        let e = &self.json["error"];
        match (e["code"].as_str(), e["message"].as_str()) {
            (Some(c), Some(m)) => format!("{c}: {m}"),
            _ => self
                .json
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("no JSON output")
                .to_owned(),
        }
    }
}

/// A JSON array of strings (a lone string counts as one element).
fn strings(v: &Value) -> Vec<String> {
    if let Some(s) = v.as_str() {
        return vec![s.to_owned()];
    }
    v.as_array()
        .map(|a| {
            a.iter()
                .filter_map(|s| s.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

/// What the task states about the fixture before the run: the entry path, the number of
/// runtimes it spans and the dump files it hands over. None of it reveals a leak.
struct Plan {
    entry: Entry,
    /// Runtimes the fixture declares (1 when it declares none).
    runtimes: usize,
    /// Heap dumps in the project (artifact entry), relative to the project root.
    artifacts: Vec<String>,
}

/// One fixture's session: the scratch project, the logs and the notes.
struct Fx<'a> {
    plan: Plan,
    bin: &'a Path,
    project: PathBuf,
    plugin_root: &'a Path,
    data_dir: PathBuf,
    log_dir: PathBuf,
    seq: usize,
    row: FixtureRow,
    /// Longest time one command may run.
    timeout: Duration,
    /// A command timed out: the rest of the fixture is skipped.
    timed_out: bool,
    /// The worker the commands run as (`--worker <name>`), in worker mode.
    worker: Option<String>,
    /// The `PATH` directory of `--without` runs; the tool-home variables are unset with it.
    path_dir: Option<PathBuf>,
    /// The tool-home variables unset for `--without` calls.
    unset_vars: Vec<&'static str>,
    /// Leak confirmations held back until the summarize steps ran ([`ops::run`]).
    pending: Vec<ops::Pending>,
}

/// How a command ended.
enum Ended {
    /// Exit code (-1 when killed by a signal), stdout, stderr.
    Done(i32, String, String),
    /// Killed after the time limit; whatever output had arrived.
    TimedOut(String, String),
    /// Could not be started.
    Spawn(String),
}

/// Reads a pipe to its end on a thread and sends the text.
fn drain<R: Read + Send + 'static>(mut pipe: R) -> mpsc::Receiver<String> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let _ = pipe.read_to_end(&mut buf);
        let _ = tx.send(String::from_utf8_lossy(&buf).into_owned());
    });
    rx
}

/// Kills the process group of `child` (the child itself where there are no groups).
fn kill_tree(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let _ = Command::new("kill")
            .args(["-KILL", "--", &format!("-{}", child.id())])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
}

/// Runs `cmd` with piped output read on threads, in its own process group, and kills the
/// group after `limit`. A leftover grandchild that keeps a pipe open cannot hold the caller:
/// the readers are waited for only briefly once the command has ended.
fn run_limited(cmd: &mut Command, limit: Duration) -> Ended {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => return Ended::Spawn(e.to_string()),
    };
    let out = child.stdout.take().map(drain);
    let err = child.stderr.take().map(drain);
    let deadline = Instant::now() + limit;
    let mut code = None;
    while code.is_none() {
        match child.try_wait() {
            Ok(Some(status)) => code = Some(status.code().unwrap_or(-1)),
            Ok(None) if Instant::now() >= deadline => break,
            Ok(None) => std::thread::sleep(Duration::from_millis(25)),
            Err(e) => return Ended::Spawn(e.to_string()),
        }
    }
    let timed_out = code.is_none();
    if timed_out {
        kill_tree(&mut child);
        let _ = child.wait();
    }
    let grace = Duration::from_secs(if timed_out { 2 } else { 5 });
    let collect = |rx: Option<mpsc::Receiver<String>>| {
        rx.and_then(|r| r.recv_timeout(grace).ok())
            .unwrap_or_default()
    };
    let (stdout, stderr) = (collect(out), collect(err));
    match code {
        Some(c) => Ended::Done(c, stdout, stderr),
        None => Ended::TimedOut(stdout, stderr),
    }
}

impl Fx<'_> {
    /// Runs one dla command, logs it, and records a failure in `errors`.
    fn call(&mut self, args: &[&str]) -> Out {
        let out = self.call_quiet(args);
        if !out.ok {
            self.row
                .errors
                .push(format!("dla {}: {}", args.join(" "), out.message()));
        }
        out
    }

    /// Like [`Fx::call`] without recording a failure (the caller decides).
    fn call_quiet(&mut self, args: &[&str]) -> Out {
        self.seq += 1;
        if self.timed_out {
            return Out {
                ok: false,
                json: Value::Null,
                bytes: 0,
                exit: -1,
            };
        }
        let mut cmd = Command::new(self.bin);
        if let Some(dir) = &self.path_dir {
            cmd.env("PATH", dir);
            for var in &self.unset_vars {
                cmd.env_remove(var);
            }
        }
        cmd.arg("--project").arg(&self.project);
        if let Some(w) = &self.worker {
            cmd.args(["--worker", w]);
        }
        let result = run_limited(
            cmd.args(args)
                .env("DLA_PLUGIN_ROOT", self.plugin_root)
                .env("DLA_DATA_DIR", &self.data_dir)
                .env("DLA_CONSENT_TIERS", "1-4")
                .env_remove("DLA_RUN")
                .env_remove("DLA_WORKER")
                .current_dir(&self.project),
            self.timeout,
        );
        let (code, stdout, stderr) = match result {
            Ended::Done(c, o, e) => (c, o, e),
            Ended::TimedOut(o, e) => {
                self.timed_out = true;
                self.row.failures.push(format!(
                    "dla {} timed out after {}s and was killed; the rest of the fixture was skipped",
                    args.join(" "),
                    self.timeout.as_secs()
                ));
                (-1, o, e)
            }
            Ended::Spawn(e) => (-1, String::new(), format!("spawn failed: {e}")),
        };
        let slug: String = args
            .iter()
            .take(2)
            .map(|a| {
                a.chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("-");
        let log = format!(
            "$ dla {}\nexit {code}\n{}\n{}",
            args.join(" "),
            stdout.trim_end(),
            if stderr.trim().is_empty() {
                String::new()
            } else {
                format!("--- stderr\n{}\n", stderr.trim_end())
            }
        );
        let _ = fs::create_dir_all(&self.log_dir);
        let _ = fs::write(
            self.log_dir.join(format!("{:03}-{slug}.log", self.seq)),
            log,
        );
        let json: Value = serde_json::from_str(stdout.trim()).unwrap_or(Value::Null);
        // A crashed dla command fails the fixture in every run. With a hidden tool it must
        // degrade to a refusal (JSON with ok:false), never a crash.
        if !self.timed_out
            && code != 0
            && (code == 101 || (json.is_null() && args.first() != Some(&"report")))
        {
            self.row.failures.push(format!(
                "dla {} crashed{} (exit {code}, {})",
                args.join(" "),
                if self.path_dir.is_some() {
                    " with a hidden tool"
                } else {
                    ""
                },
                if json.is_null() { "no JSON" } else { "panic" }
            ));
        }
        // `dla report` prints text; every other command prints one JSON object.
        let ok = if args.first() == Some(&"report") {
            code == 0
        } else {
            code == 0 && json.get("ok") == Some(&Value::Bool(true))
        };
        Out {
            ok,
            json,
            bytes: stdout.trim().len(),
            exit: code,
        }
    }

    fn step(&mut self, line: String) {
        self.row.steps.push(line);
    }

    fn run_dir(&self) -> Result<PathBuf, String> {
        let id = fs::read_to_string(self.project.join(".dla/current"))
            .map_err(|e| format!(".dla/current: {e}"))?;
        Ok(self.project.join(".dla").join(id.trim()))
    }

    /// The ledger as the current actor sees it: the main ledger, plus the worker's slice in
    /// worker mode (a worker replays both).
    fn view(&self) -> View {
        match self.run_dir() {
            Ok(d) => match &self.worker {
                Some(w) => {
                    View::read_files(&[d.join("ledger.jsonl"), d.join(format!("ledger.{w}.jsonl"))])
                }
                None => View::read(&d.join("ledger.jsonl")),
            },
            Err(_) => View::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// Ledger view (re-read from the file; nothing is taken on trust)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
struct Fnd {
    id: String,
    state: String,
    origin: String,
    site: Option<String>,
    owner: Option<String>,
    holder: Option<String>,
    parent: Option<String>,
    hypothesis: Option<String>,
    supports: Vec<String>,
    /// `from` and cited evidence of the transition to `confirmed`.
    confirmed: Option<(String, Vec<String>)>,
}

impl Fnd {
    fn is_nonleak(&self) -> bool {
        self.hypothesis
            .as_deref()
            .is_some_and(|h| h.starts_with("N-"))
    }
}

#[derive(Debug, Clone)]
struct Ev {
    kind: String,
    authority: String,
    result: Value,
}

#[derive(Debug, Default)]
struct View {
    findings: Vec<Fnd>,
    evidence: BTreeMap<String, Ev>,
    /// Evidence ids in ledger order.
    order: Vec<String>,
}

impl View {
    fn read(path: &Path) -> View {
        View::read_files(&[path.to_path_buf()])
    }

    /// Replays several ledger files in order (the main ledger, then a worker's slice).
    fn read_files(paths: &[PathBuf]) -> View {
        let mut v = View::default();
        let text: String = paths
            .iter()
            .filter_map(|p| fs::read_to_string(p).ok())
            .collect::<Vec<_>>()
            .join("\n");
        for line in text.lines() {
            let Ok(rec) = serde_json::from_str::<Value>(line) else {
                continue;
            };
            let s = |x: &Value, k: &str| x.get(k).and_then(Value::as_str).map(str::to_owned);
            match rec["type"].as_str() {
                Some("evidence") => {
                    if let Some(id) = s(&rec, "evidence_id") {
                        v.order.push(id.clone());
                        v.evidence.insert(
                            id,
                            Ev {
                                kind: s(&rec, "kind").unwrap_or_default(),
                                authority: s(&rec, "authority").unwrap_or_default(),
                                result: rec["result"].clone(),
                            },
                        );
                    }
                }
                Some("finding_added") => {
                    let f = &rec["finding"];
                    let Some(id) = s(f, "id") else { continue };
                    v.findings.retain(|x| x.id != id);
                    v.findings.push(Fnd {
                        id,
                        state: "candidate".to_owned(),
                        origin: s(f, "origin").unwrap_or_default(),
                        site: s(f, "site"),
                        owner: s(f, "owner"),
                        holder: s(f, "holder"),
                        parent: s(f, "parent"),
                        hypothesis: s(f, "hypothesis"),
                        supports: strings(&f["supports"]),
                        confirmed: None,
                    });
                }
                Some("finding_annotated") => {
                    let Some(id) = s(&rec, "finding") else {
                        continue;
                    };
                    if let Some(f) = v.findings.iter_mut().find(|f| f.id == id) {
                        if f.holder.is_none() {
                            f.holder = s(&rec, "holder");
                        }
                        if f.site.is_none() {
                            f.site = s(&rec, "site");
                        }
                        if f.owner.is_none() {
                            f.owner = s(&rec, "owner");
                        }
                    }
                }
                Some("transition") => {
                    let Some(id) = s(&rec, "finding") else {
                        continue;
                    };
                    let Some(to) = s(&rec, "to") else { continue };
                    if let Some(f) = v.findings.iter_mut().find(|f| f.id == id) {
                        if to == "confirmed" {
                            f.confirmed = Some((
                                s(&rec, "from").unwrap_or_default(),
                                strings(&rec["evidence"]),
                            ));
                        }
                        f.state = to;
                    }
                }
                _ => {}
            }
        }
        v
    }

    fn finding(&self, id: &str) -> Option<&Fnd> {
        self.findings.iter().find(|f| f.id == id)
    }

    /// Latest evidence of a kind (optionally with a `harness` field) in ledger order.
    fn latest(&self, kind: &str, harness: Option<&str>) -> Option<String> {
        self.order
            .iter()
            .rev()
            .find(|id| {
                let e = &self.evidence[*id];
                e.kind == kind && harness.is_none_or(|h| e.result["harness"].as_str() == Some(h))
            })
            .cloned()
    }

    /// The first evidence among `ids` of a kind.
    fn of_kind(&self, ids: &[String], kind: &str) -> Option<String> {
        ids.iter()
            .find(|id| self.evidence.get(*id).is_some_and(|e| e.kind == kind))
            .cloned()
    }

    fn discriminators_after(&self, before: &BTreeSet<String>) -> Vec<String> {
        self.order
            .iter()
            .filter(|id| !before.contains(*id) && self.evidence[*id].kind == "discriminator")
            .cloned()
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Trend columns
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Col {
    name: String,
    kind: String,
    region: String,
    shape: String,
    pass: bool,
    candidates: Vec<String>,
}

impl Col {
    fn container(&self) -> bool {
        self.name.starts_with("cgroup.") || self.region == "container"
    }
}

fn read_columns(path: &Path) -> Vec<Col> {
    let Ok(text) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    v["columns"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|c| Col {
                    name: c["name"].as_str().unwrap_or("").to_owned(),
                    kind: c["kind"].as_str().unwrap_or("gauge").to_owned(),
                    region: c["region"].as_str().unwrap_or("").to_owned(),
                    shape: c["shape"].as_str().unwrap_or("").to_owned(),
                    pass: c["gate"].as_str() == Some("pass"),
                    candidates: strings(&c["candidates"]),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The growth evidence of a trend that covers a column kind.
fn trend_evidence(view: &View, ids: &[String], col: &Col) -> Option<String> {
    let want = if col.kind == "counter" {
        "unit-count-series"
    } else {
        "growth-series"
    };
    view.of_kind(ids, want)
        .or_else(|| view.of_kind(ids, "growth-series"))
        .or_else(|| view.of_kind(ids, "unit-count-series"))
}

// ---------------------------------------------------------------------------
// The policy
// ---------------------------------------------------------------------------

/// A harness that ran.
struct Ran {
    harness: String,
    runtime: String,
    series: Vec<String>,
}

/// Main and script workloads with an iteration argument, best rank first.
fn read_workloads(run_dir: &Path, worker: Option<&str>) -> Vec<(String, String)> {
    // The core writes `workload/workloads.json`, a worker `workload/<worker>.json`.
    let file = match worker {
        Some(w) => format!("workload/{w}.json"),
        None => "workload/workloads.json".to_owned(),
    };
    let Ok(text) = fs::read_to_string(run_dir.join(file)) else {
        return Vec::new();
    };
    let Ok(v) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    let mut list: Vec<(i64, String, String)> = v["workloads"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter(|w| matches!(w["kind"].as_str(), Some("main" | "script")))
                .filter(|w| w["iteration_arg"].as_str().is_some_and(|s| !s.is_empty()))
                .map(|w| {
                    (
                        w["rank"].as_i64().unwrap_or(0),
                        w["id"].as_str().unwrap_or("").to_owned(),
                        w["runtime"].as_str().unwrap_or("").to_owned(),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    list.sort_by_key(|(r, _, _)| std::cmp::Reverse(*r));
    list.into_iter().map(|(_, id, rt)| (id, rt)).collect()
}

/// Step 1: harness gen and run per runtime. Returns the harnesses that ran and
/// the reason (`build_fails` or `no_harness`) for the first runtime without one.
fn make_harnesses(
    fx: &mut Fx,
    runtimes: &[String],
    builds: &Value,
) -> (Vec<Ran>, Option<&'static str>) {
    let mut ran = Vec::new();
    let mut missing = None;
    let run_dir = fx.run_dir().unwrap_or_default();
    for lang in runtimes {
        let others: Vec<String> = read_workloads(&run_dir, fx.worker.as_deref())
            .into_iter()
            .filter(|(_, rt)| rt == lang)
            .map(|(id, _)| id)
            .collect();
        let mut tried: Vec<String> = Vec::new();
        let mut next_other = 0usize;
        for attempt in 0..MAX_WORKLOADS {
            let workload = if attempt == 0 {
                None
            } else {
                let mut pick = None;
                while next_other < others.len() {
                    let id = &others[next_other];
                    next_other += 1;
                    if !tried.contains(id) {
                        pick = Some(id.clone());
                        break;
                    }
                }
                if pick.is_none() {
                    break;
                }
                pick
            };
            let mut args = vec!["harness", "gen", "--lang", lang.as_str()];
            if let Some(w) = &workload {
                args.extend(["--workload", w.as_str()]);
            }
            let gen_out = fx.call_quiet(&args);
            if let Some(w) = gen_out.json["summary"]["workload"].as_str() {
                tried.push(w.to_owned());
            } else if let Some(w) = &workload {
                tried.push(w.clone());
            }
            if !gen_out.ok {
                let msg = format!("harness gen {lang}: {}", gen_out.message());
                fx.step(msg.clone());
                if matches!(
                    gen_out.code(),
                    "workload-required" | "iteration-arg-unknown"
                ) {
                    continue;
                }
                fx.row.errors.push(msg);
                break;
            }
            let Some(h) = gen_out.json["summary"]["harness"]
                .as_str()
                .map(str::to_owned)
            else {
                fx.row
                    .errors
                    .push("harness gen printed no harness id".to_owned());
                break;
            };
            let run_out = fx.call_quiet(&["harness", "run", &h]);
            if !run_out.ok {
                let msg = format!("harness run {h}: {}", run_out.message());
                fx.step(msg.clone());
                fx.row.errors.push(msg);
                continue;
            }
            let series = strings(&run_out.json["summary"]["series"]);
            fx.step(format!(
                "{h} ({lang}, {}) ran: series {}",
                gen_out.json["summary"]["workload"].as_str().unwrap_or("?"),
                series.join(",")
            ));
            ran.push(Ran {
                harness: h,
                runtime: lang.clone(),
                series,
            });
            break;
        }
        if !ran.iter().any(|r| &r.runtime == lang) {
            let reason = if builds[lang.as_str()].as_str().is_some_and(|s| s != "ok") {
                "build_fails"
            } else {
                "no_harness"
            };
            fx.step(format!("no harness ran for {lang}: {reason}"));
            missing.get_or_insert(reason);
        }
    }
    (ran, missing)
}

/// What an alloc-test said.
#[derive(Debug, Clone, PartialEq)]
enum Verdict {
    Leak,
    NonLeak,
    Inconclusive,
    /// The command refused or failed.
    Failed(String),
}

struct Alloc {
    verdict: Verdict,
    evidence: Option<String>,
}

/// Runs `dla alloc-test` and reads the verdict from the discriminator it wrote.
fn alloc_test(fx: &mut Fx, harness: &str, finding: Option<&str>) -> Alloc {
    let before: BTreeSet<String> = fx.view().order.into_iter().collect();
    let mut args = vec!["alloc-test", harness];
    if let Some(f) = finding {
        args.extend(["--finding", f]);
    }
    let out = fx.call_quiet(&args);
    if !out.ok {
        let msg = out.message();
        fx.row.errors.push(format!("dla {}: {msg}", args.join(" ")));
        return Alloc {
            verdict: Verdict::Failed(msg),
            evidence: None,
        };
    }
    let view = fx.view();
    let mut cands = out.evidence();
    cands.retain(|id| {
        view.evidence
            .get(id)
            .is_some_and(|e| e.kind == "discriminator")
    });
    if cands.is_empty() {
        cands = view.discriminators_after(&before);
    }
    let Some(id) = cands.last().cloned() else {
        return Alloc {
            verdict: Verdict::Failed("alloc-test wrote no discriminator".to_owned()),
            evidence: None,
        };
    };
    let verdict = match view.evidence[&id].result["favors"].as_str().unwrap_or("") {
        "leak" => Verdict::Leak,
        "non-leak" => Verdict::NonLeak,
        _ => Verdict::Inconclusive,
    };
    Alloc {
        verdict,
        evidence: Some(id),
    }
}

/// Adds a sibling hypothesis; returns its id.
fn add_sibling(fx: &mut Fx, root: &str, hyp: &str, evidence: &str) -> Option<String> {
    let refute_if = match hyp {
        "N-WARM" => "growth continues over the extra cycles",
        "N-HEADROOM" => "the growth does not drop after a garbage collection or a trim",
        _ => "the growth does not drop after the allocator returns free memory",
    };
    let out = fx.call(&[
        "ledger",
        "add",
        "--sibling-of",
        root,
        "--origin",
        "hypothesis",
        "--hypothesis",
        hyp,
        "--refute-if",
        refute_if,
        "--evidence",
        evidence,
    ]);
    if out.ok {
        out.json["id"].as_str().map(str::to_owned)
    } else {
        None
    }
}

fn resolve(fx: &mut Fx, id: &str, disposition: &str, reason: &str, remedy: &str) -> bool {
    fx.call(&[
        "ledger",
        "resolve",
        id,
        "--as",
        disposition,
        "--reason",
        reason,
        "--remedy",
        remedy,
    ])
    .ok
}

/// The holder of a finding without one, with the rule that picked it.
fn pick_holder(run_dir: &Path, f: &Fnd, col: &Col) -> (String, &'static str) {
    if let Some(site) = &f.site
        && let Ok(rd) = fs::read_dir(run_dir.join("census"))
    {
        for e in rd.flatten() {
            let Ok(text) = fs::read_to_string(e.path()) else {
                continue;
            };
            let Ok(v) = serde_json::from_str::<Value>(&text) else {
                continue;
            };
            for entry in v["entries"].as_array().into_iter().flatten() {
                if entry["site"].as_str() == Some(site.as_str())
                    && let Some(h) = entry["holder"].as_str()
                {
                    return (h.to_owned(), "census entry at the site");
                }
            }
        }
    }
    let n = col.name.to_ascii_lowercase();
    if n.contains("goroutine") || n.contains("thread") {
        ("blocked-stack".to_owned(), "goroutine or thread column")
    } else if n.contains("timer")
        || n.contains("listener")
        || n.contains("handle")
        || n.contains("interval")
    {
        ("registration-table".to_owned(), "timer or listener column")
    } else {
        ("static/global".to_owned(), "default")
    }
}

/// The trend of one series.
struct TrendInfo {
    harness: String,
    cols: Vec<Col>,
    evidence: Vec<String>,
    series: String,
}

/// Steps 2 to 4.
fn analyse(fx: &mut Fx, ran: &[Ran]) {
    // Native growth behind a managed method is correlated before the trends, so the
    // candidate it adds is tied by the native column.
    for r in ran
        .iter()
        .filter(|r| r.runtime == "jvm" && !r.series.is_empty())
    {
        ops::correlate(fx, &r.harness);
    }
    let mut trends: Vec<TrendInfo> = Vec::new();
    for r in ran {
        for s in &r.series {
            let out = fx.call(&["trend", s]);
            if !out.ok {
                continue;
            }
            let trend_id = out.json["trend"].as_str().unwrap_or("");
            let cols = fx
                .run_dir()
                .map(|d| read_columns(&d.join("trend").join(format!("{trend_id}.json"))))
                .unwrap_or_default();
            fx.row.passed_columns += cols.iter().filter(|c| c.pass && !c.container()).count();
            trends.push(TrendInfo {
                harness: r.harness.clone(),
                cols,
                evidence: out.evidence(),
                series: s.clone(),
            });
        }
    }
    let mut chosen: BTreeSet<String> = BTreeSet::new();
    let mut chosen_sites: BTreeSet<String> = BTreeSet::new();
    let mut alloc_tests = 0usize;
    let mut any_with_candidates = false;

    // Step 2: passed columns with candidates.
    for t in &trends {
        for col in t.cols.iter().filter(|c| c.pass && !c.container()) {
            if col.candidates.is_empty() {
                continue;
            }
            any_with_candidates = true;
            fx.row.passed_with_candidates += 1;
            let view = fx.view();
            let Some(f) = first_candidate(&view, col) else {
                continue;
            };
            if !chosen.insert(f.id.clone()) {
                continue;
            }
            if let Some(site) = &f.site {
                chosen_sites.insert(site.clone());
            }
            let Some(ev) = trend_evidence(&view, &t.evidence, col) else {
                fx.row
                    .errors
                    .push(format!("trend {} wrote no growth evidence", t.series));
                continue;
            };
            let outcome = test_candidate(fx, t, &f, col, &ev, true, &mut alloc_tests);
            fx.step(format!("{} via {}: {outcome}", f.id, col.name));
        }
    }

    // Step 3: candidates tied to non-passing columns.
    for t in &trends {
        for col in t.cols.iter().filter(|c| !c.pass && !c.container()) {
            let view = fx.view();
            let Some(f) = first_candidate(&view, col) else {
                continue;
            };
            if chosen.contains(&f.id) || f.site.as_ref().is_some_and(|s| chosen_sites.contains(s)) {
                continue;
            }
            chosen.insert(f.id.clone());
            if let Some(site) = &f.site {
                chosen_sites.insert(site.clone());
            }
            let Some(ev) = trend_evidence(&view, &t.evidence, col) else {
                continue;
            };
            let outcome = test_candidate(fx, t, &f, col, &ev, false, &mut alloc_tests);
            fx.step(format!("{} via {} (not passed): {outcome}", f.id, col.name));
        }
    }

    // Step 3b: nothing tied to a candidate grew. One unbound alloc-test judges the
    // whole workload; when it favors non-leak it rules out every leak-side
    // candidate still open, because no measured column of the process grows.
    if !any_with_candidates && let Some(t) = trends.first() {
        let view = fx.view();
        let open: Vec<String> = view
            .findings
            .iter()
            .filter(|f| f.state == "candidate" && !f.is_nonleak() && f.site.is_some())
            .map(|f| f.id.clone())
            .collect();
        if !open.is_empty() && alloc_tests < MAX_ALLOC_TESTS {
            alloc_tests += 1;
            let a = alloc_test(fx, &t.harness, None);
            if let (Verdict::NonLeak, Some(e)) = (&a.verdict, &a.evidence) {
                let mut refuted = 0usize;
                for f in &open {
                    if fx.call(&["ledger", "refute", f, "--evidence", e]).ok {
                        refuted += 1;
                    }
                }
                fx.step(format!(
                    "unbound alloc-test {e} favors non-leak: refuted {refuted} of {} open candidates",
                    open.len()
                ));
            } else {
                fx.step(format!(
                    "unbound alloc-test: {:?}; open candidates left for step 5",
                    a.verdict
                ));
            }
        }
    }

    // Step 4: a passing region column without candidates.
    if !any_with_candidates
        && let Some((t, col)) = trends.iter().find_map(|t| {
            t.cols
                .iter()
                .find(|c| c.pass && !c.container())
                .map(|c| (t, c))
        })
    {
        let view = fx.view();
        if let Some(ev) = trend_evidence(&view, &t.evidence, col) {
            let shape = col.shape.clone();
            let mut args = vec![
                "ledger",
                "add",
                "--origin",
                "growth-series",
                "--evidence",
                &ev,
            ];
            if !shape.is_empty() {
                args.extend(["--shape", &shape]);
            }
            let out = fx.call(&args);
            let id = if out.ok {
                out.json["id"].as_str().map(str::to_owned)
            } else {
                None
            };
            if let Some(g) = id {
                let outcome = test_growth(fx, t, &g, &ev, &mut alloc_tests);
                fx.step(format!("{g} (growth in {}): {outcome}", col.name));
            }
        }
    }

    // Step 5: capture, summarize and check for the harnesses that grew, then the
    // confirmations held back above.
    let grew: Vec<(String, String)> = ran
        .iter()
        .filter(|r| {
            trends
                .iter()
                .any(|t| t.harness == r.harness && t.cols.iter().any(|c| c.pass && !c.container()))
        })
        .map(|r| (r.harness.clone(), r.runtime.clone()))
        .collect();
    ops::run(fx, &grew);
}

/// The column's own candidate: the first leak-side finding it lists, which
/// must still be a candidate (a finding another column already took is not
/// replaced by the next one).
fn first_candidate(view: &View, col: &Col) -> Option<Fnd> {
    col.candidates
        .iter()
        .filter_map(|id| view.finding(id))
        .find(|f| !f.is_nonleak())
        .filter(|f| f.state == "candidate")
        .cloned()
}

/// Tests a finding with the workload evidence of its harness.
fn test_finding(fx: &mut Fx, harness: &str, f: &str) -> bool {
    let view = fx.view();
    let Some(w) = view.latest("workload", Some(harness)) else {
        fx.row
            .errors
            .push(format!("no workload evidence for {harness}"));
        return false;
    };
    fx.call(&["ledger", "test", f, "--evidence", &w]).ok
}

/// Steps 2 and 3 for one candidate; returns what happened, for the log.
fn test_candidate(
    fx: &mut Fx,
    t: &TrendInfo,
    f: &Fnd,
    col: &Col,
    ev: &str,
    passed: bool,
    alloc_tests: &mut usize,
) -> String {
    let Some(sib) = add_sibling(fx, &f.id, "N-WARM", ev) else {
        return "could not add the N-WARM sibling".to_owned();
    };
    if !test_finding(fx, &t.harness, &f.id) {
        return "ledger test refused".to_owned();
    }
    if *alloc_tests >= MAX_ALLOC_TESTS {
        resolve(
            fx,
            &f.id,
            "unresolved",
            "time_budget",
            "the alloc-test budget of this run is used up; run dla alloc-test for this finding",
        );
        return "alloc-test budget used up".to_owned();
    }
    *alloc_tests += 1;
    let a = alloc_test(fx, &t.harness, Some(&f.id));
    match (&a.verdict, &a.evidence) {
        (Verdict::Leak, Some(e)) if passed => {
            fx.call(&["ledger", "refute", &sib, "--evidence", e]);
            let view = fx.view();
            let mut note = String::from("favors leak");
            if view.finding(&f.id).is_some_and(|x| x.holder.is_none())
                && let Ok(run_dir) = fx.run_dir()
            {
                let (holder, rule) = pick_holder(&run_dir, f, col);
                fx.call(&["ledger", "annotate", &f.id, "--holder", &holder]);
                note.push_str(&format!(", holder {holder} ({rule})"));
            }
            // The confirmation waits for the summarize steps, which add their evidence to it.
            fx.pending.push(ops::Pending {
                finding: f.id.clone(),
                evidence: ev.to_owned(),
            });
            note.push_str(", confirmation after the summarize steps");
            note
        }
        (Verdict::NonLeak, Some(e)) => {
            let r = fx.call(&["ledger", "refute", &f.id, "--evidence", e]);
            if r.ok {
                "favors non-leak, refuted".to_owned()
            } else {
                format!("favors non-leak, refute refused ({})", r.code())
            }
        }
        (v, _) => {
            let why = match v {
                Verdict::Failed(m) => format!("alloc-test failed: {m}"),
                Verdict::Leak => "growth continues in a column that did not pass".to_owned(),
                _ => "alloc-test is inconclusive".to_owned(),
            };
            let reason = if matches!(v, Verdict::Failed(_)) {
                "env_unavailable"
            } else {
                "time_budget"
            };
            resolve(
                fx,
                &f.id,
                "unresolved",
                reason,
                "re-run dla alloc-test with more extra cycles (--extra) or a longer workload",
            );
            why
        }
    }
}

/// Step 4 for a growth finding without a site.
fn test_growth(fx: &mut Fx, t: &TrendInfo, g: &str, ev: &str, alloc_tests: &mut usize) -> String {
    for hyp in ["N-HEADROOM", "N-ALLOC"] {
        if add_sibling(fx, g, hyp, ev).is_none() {
            return format!("could not add {hyp}");
        }
    }
    if !test_finding(fx, &t.harness, g) {
        return "ledger test refused".to_owned();
    }
    if *alloc_tests >= MAX_ALLOC_TESTS {
        resolve(
            fx,
            g,
            "unresolved",
            "time_budget",
            "run dla alloc-test for this growth",
        );
        return "alloc-test budget used up".to_owned();
    }
    *alloc_tests += 1;
    let a = alloc_test(fx, &t.harness, None);
    match (&a.verdict, &a.evidence) {
        (Verdict::NonLeak, Some(e)) => {
            let r = fx.call(&["ledger", "refute", g, "--evidence", e]);
            if r.ok {
                "favors non-leak, refuted".to_owned()
            } else {
                format!("favors non-leak, refute refused ({})", r.code())
            }
        }
        (v, _) => {
            let reason = if matches!(v, Verdict::Failed(_)) {
                "env_unavailable"
            } else {
                "time_budget"
            };
            resolve(
                fx,
                g,
                "unresolved",
                reason,
                "attribute the growth to a site: dla harness gen --probe <finding> on the suspected holder, then dla alloc-test --finding",
            );
            format!("{v:?}: left unresolved")
        }
    }
}

/// Step 5: gap, then resolve what is left, then the report.
fn finish(fx: &mut Fx, ran: &[Ran], no_harness: Option<&str>, report: bool) {
    for r in ran {
        for s in &r.series {
            // Refusals are ignored: the gap needs series the runtime may not write.
            fx.call_quiet(&["gap", s, "--residual"]);
        }
    }
    let view = fx.view();
    let todo: Vec<(String, bool)> = view
        .findings
        .iter()
        .filter(|f| matches!(f.state.as_str(), "candidate" | "under_test") && !f.is_nonleak())
        .map(|f| {
            let has_static = f.origin.starts_with("detector:")
                || f.origin == "census"
                || f.supports.iter().any(|e| {
                    view.evidence
                        .get(e)
                        .is_some_and(|e| e.authority == "static")
                });
            (f.id.clone(), has_static)
        })
        .collect();
    for (id, has_static) in todo {
        if !has_static {
            resolve(
                fx,
                &id,
                "unresolved",
                "time_budget",
                "no dynamic evidence reached this finding; re-run dla alloc-test with a probe",
            );
            continue;
        }
        let (reason, remedy) = match no_harness {
            Some("build_fails") => (
                "build_fails",
                "fix the build (see probe/probe.json), then dla harness gen --lang <l>",
            ),
            Some(_) => (
                "no_harness",
                "write a loop harness: dla harness gen --lang <l> --driver",
            ),
            None => (
                "not_exercised",
                "no loop test reached this site; extend the harness (dla harness gen --probe <finding>), then dla ledger test",
            ),
        };
        resolve(fx, &id, "suspected_static_only", reason, remedy);
    }
    if report {
        fx.call(&["report"]);
    }
}

// ---------------------------------------------------------------------------
// Checks
// ---------------------------------------------------------------------------

/// The M3 rules, re-checked from the ledger for every confirmed leak-side finding.
fn check_m3_rules(view: &View, entry: Entry) -> Vec<String> {
    let mut bad = Vec::new();
    for f in view
        .findings
        .iter()
        .filter(|f| f.state == "confirmed" && !f.is_nonleak())
    {
        let Some((from, cited)) = &f.confirmed else {
            bad.push(format!("{} is confirmed without a transition record", f.id));
            continue;
        };
        if from != "under_test" {
            bad.push(format!(
                "{} was confirmed from {from}, not under_test",
                f.id
            ));
        }
        let root = f.parent.as_deref().unwrap_or(&f.id);
        let refuted_sibling = view
            .findings
            .iter()
            .any(|g| g.is_nonleak() && g.state == "refuted" && g.parent.as_deref() == Some(root));
        if !refuted_sibling {
            bad.push(format!("{} has no refuted N-* sibling", f.id));
        }
        let growth = f.supports.iter().chain(cited).any(|id| {
            view.evidence.get(id).is_some_and(|e| {
                matches!(e.kind.as_str(), "growth-series" | "unit-count-series")
                    && if entry == Entry::Artifact {
                        e.authority == "dump"
                    } else {
                        matches!(e.authority.as_str(), "test" | "live" | "dump")
                    }
                    && e.result["gate"].as_str() == Some("pass")
            })
        });
        if !growth {
            bad.push(format!("{} cites no growth evidence with gate pass", f.id));
        }
        if entry == Entry::Artifact {
            // The dumps' frames are thread context only: a confirmed artifact finding names
            // its owner and never claims a source site.
            if f.owner.is_none() {
                bad.push(format!("{} has no owner", f.id));
            }
            if f.site.is_some() {
                bad.push(format!("{} claims a source site on an artifact run", f.id));
            }
        } else if f.site.is_none() {
            bad.push(format!("{} has no site", f.id));
        }
        if f.holder.is_none() {
            bad.push(format!("{} has no holder", f.id));
        }
    }
    bad
}

/// Evidence kinds that show what retains or allocates the memory.
/// `non-heap` (a goroutine or thread dump grouped by creation site) and `should-be-dead` (the
/// goroutine leak profile) count with `dump` or `live` authority, for the marked leaks of
/// runtimes without a heap dump.
const DUMP_KINDS: [&str; 5] = [
    "retention-path",
    "histogram-diff",
    "alloc-stack",
    "non-heap",
    "should-be-dead",
];

/// The `dump_backed` rule: for every confirmable leak that the expected file marks, the chosen
/// confirmed finding (the first confirmed leak-side finding that names the leak) cites evidence
/// of a retention or allocation kind with `dump` or `live` authority, in its supports or in
/// the evidence of its confirming transition. A leak that no finding names is not reported
/// here (the grading failure covers it).
fn check_dump_backed(view: &View, expected: &Expected) -> Vec<String> {
    let mut bad = Vec::new();
    for leak in expected
        .leak
        .iter()
        .filter(|l| l.dump_backed && l.confirmable)
    {
        let chosen = view.findings.iter().find(|f| {
            f.state == "confirmed" && !f.is_nonleak() && f.parent.is_none() && {
                let probe = Finding {
                    state: State::Confirmed,
                    site: f.site.clone(),
                    owner: f.owner.clone(),
                    worker: None,
                    holder: f.holder.clone(),
                    label: None,
                    hypothesis: false,
                    discriminator: false,
                };
                site_matches(&probe, leak)
            }
        });
        let Some(f) = chosen else { continue };
        let mut cited = f
            .supports
            .iter()
            .chain(f.confirmed.iter().flat_map(|(_, c)| c));
        let backed = cited.any(|id| {
            view.evidence.get(id).is_some_and(|e| {
                DUMP_KINDS.contains(&e.kind.as_str())
                    && matches!(e.authority.as_str(), "dump" | "live")
            })
        });
        if !backed {
            bad.push(format!(
                "leak {} is marked dump_backed but {} cites no retention-path, histogram-diff or alloc-stack evidence with dump or live authority",
                leak.id, f.id
            ));
        }
    }
    bad
}

/// One line per trend report (its passing columns with shape, Sen slope and noise) and per
/// alloc-test discriminator (each column's verdicts), at most 12 lines.
fn diagnostics(run_dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut trends: Vec<_> = fs::read_dir(run_dir.join("trend"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    trends.sort();
    for t in trends {
        let Some(v) = fs::read_to_string(&t)
            .ok()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        else {
            continue;
        };
        let cols: Vec<String> = v["columns"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| c["gate"] == "pass")
            .map(|c| {
                format!(
                    "{} {} slope {:.3} noise {:.0}",
                    c["name"].as_str().unwrap_or("?"),
                    c["shape"].as_str().unwrap_or("?"),
                    c["sen"]["slope"].as_f64().unwrap_or(f64::NAN),
                    c["noise"].as_f64().unwrap_or(f64::NAN)
                )
            })
            .collect();
        out.push(format!(
            "{} of {} passed: {}",
            v["id"].as_str().unwrap_or("trend"),
            v["series"].as_str().unwrap_or("?"),
            if cols.is_empty() {
                "none".to_owned()
            } else {
                cols.join("; ")
            }
        ));
    }
    let ledger = fs::read_to_string(run_dir.join("ledger.jsonl")).unwrap_or_default();
    for line in ledger.lines() {
        let Ok(v) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if v["type"] != "evidence" || v["kind"] != "discriminator" {
            continue;
        }
        let Some(cols) = v["result"]["columns"].as_array() else {
            continue;
        };
        let cols: Vec<String> = cols
            .iter()
            .map(|c| {
                let p = &c["plateau"];
                let num = |k: &str| p[k].as_f64().map_or("-".to_owned(), |v| format!("{v:.3}"));
                format!(
                    "{} {} ({} main {} extra {} rise {} noise {}, purge {})",
                    c["name"].as_str().unwrap_or("?"),
                    c["favors"].as_str().unwrap_or("?"),
                    p["verdict"].as_str().unwrap_or("-"),
                    num("main_slope"),
                    num("extra_slope"),
                    num("extra_rise"),
                    num("noise"),
                    c["purge"]["verdict"].as_str().unwrap_or("-")
                )
            })
            .collect();
        out.push(format!(
            "alloc-test {}: {}",
            v["evidence_id"].as_str().unwrap_or("?"),
            cols.join("; ")
        ));
    }
    out.truncate(12);
    out
}

/// Backends the dump-backed evidence needs: tools that dump or allocation adapters and the
/// harness run.
const DUMP_BACKENDS: &[&str] = &[
    "valgrind",
    "heaptrack",
    "heaptrack_print",
    "jfr",
    "jfr-old-object-sample",
    "addr2line",
];

/// The [`DUMP_BACKENDS`] whose coverage row reads `missing` in the run.
fn missing_dump_backends(run_dir: &Path) -> Vec<String> {
    let Ok(text) = fs::read_to_string(run_dir.join("coverage.json")) else {
        return Vec::new();
    };
    let Ok(cov) = serde_json::from_str::<Value>(&text) else {
        return Vec::new();
    };
    let mut out: Vec<String> = cov["workers"]
        .as_object()
        .into_iter()
        .flat_map(|w| w.values())
        .filter_map(|w| w["backends"].as_object())
        .flat_map(|rows| rows.iter())
        .filter(|(name, row)| {
            DUMP_BACKENDS.contains(&name.as_str()) && row["status"].as_str() == Some("missing")
        })
        .map(|(name, _)| name.clone())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Residual over total of the residual evidence; `Err` when a row cannot be computed.
fn residual_share(view: &View) -> Result<Option<f64>, String> {
    let mut residual = 0.0;
    let mut total = 0.0;
    let mut rows = 0;
    for e in view.evidence.values().filter(|e| e.kind == "residual") {
        for r in e.result["rows"].as_array().into_iter().flatten() {
            rows += 1;
            let (Some(a), Some(b)) = (r["residual"].as_f64(), r["total"].as_f64()) else {
                return Err("a residual row has no numeric residual and total".to_owned());
            };
            residual += a;
            total += b;
        }
    }
    if rows == 0 {
        return Ok(None);
    }
    Ok(Some(if total > 0.0 { residual / total } else { 0.0 }))
}

/// Grades the ledger against `expected.toml` (read only now) and fills the row.
fn grade_row(row: &mut FixtureRow, fixtures: &Path, ledger: &Path) {
    let expected = match load_expected(fixtures, &row.id) {
        Ok(e) => e,
        Err(e) => {
            row.failures.push(e);
            return;
        }
    };
    row.category = expected.category.as_str().to_owned();
    let text = fs::read_to_string(ledger).unwrap_or_default();
    let findings = match findings_from_ledger(&text) {
        Some(f) => f,
        // An impossible fixture (nothing to diagnose) ends correctly with no finding at all.
        None if expected.category == Category::Impossible => Vec::new(),
        None => {
            row.failures.push("the ledger has no finding".to_owned());
            return;
        }
    };
    let grade = grade_findings(&expected, Source::Ledger, &findings);
    row.claims = grade.claim;
    row.rule_outs = grade.rule_out;
    row.confirmed = findings
        .iter()
        .filter(|f| !f.hypothesis && f.state == State::Confirmed)
        .count();
    row.refuted = findings
        .iter()
        .filter(|f| !f.hypothesis && f.state == State::Refuted)
        .count();
    let confirmable: Vec<_> = expected
        .leak
        .iter()
        .zip(&grade.leaks)
        .filter(|(l, _)| l.confirmable)
        .collect();
    row.leaks_expected = confirmable.len();
    row.leaks_found = confirmable.iter().filter(|(_, o)| o.found).count();
    match expected.category {
        Category::ArchetypeLeak | Category::ArchetypeFree | Category::MultiLeak => {
            if expected.dynamic_confirmation {
                for (l, o) in &confirmable {
                    let at = l.site.as_deref().or(l.owner.as_deref()).unwrap_or("");
                    if !o.found {
                        row.failures
                            .push(format!("leak {} ({at}) is not confirmed", l.id));
                    } else if o.chosen_holder.as_deref() != Some(l.holder.as_str()) {
                        row.failures.push(format!(
                            "leak {} ({at}) is confirmed with holder {}, expected {}",
                            l.id,
                            o.chosen_holder.as_deref().unwrap_or("none"),
                            l.holder.as_str()
                        ));
                    }
                }
            }
        }
        Category::DecoyControl => {
            if grade.claim {
                row.failures.push("a claim on a decoy control".to_owned());
            }
        }
        Category::NonLeak => {
            if grade.claim {
                row.failures
                    .push("a claim on a non-leak fixture".to_owned());
            }
            if !grade.rule_out {
                row.failures
                    .push("no rule-out on a non-leak fixture".to_owned());
            }
        }
        Category::Impossible => {
            if grade.claim {
                row.failures
                    .push("a claim on an impossible fixture".to_owned());
            }
        }
    }
}

// ---------------------------------------------------------------------------
// One fixture
// ---------------------------------------------------------------------------

fn skip_dir(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".dla"
            | "target"
            | "build"
            | "node_modules"
            | "venv"
            | ".venv"
            | "out"
            | "dist"
            | "__pycache__"
    )
}

fn walk_files(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Res<()> {
    let rd = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            if !skip_dir(&e.file_name().to_string_lossy()) {
                walk_files(root, &p, out)?;
            }
        } else if let Ok(rel) = p.strip_prefix(root) {
            out.push(rel.to_path_buf());
        }
    }
    Ok(())
}

/// The tracked files of a fixture project (git index in a checkout, else a walk).
fn project_files(project: &Path) -> Res<Vec<PathBuf>> {
    let tracked = Command::new("git")
        .arg("-C")
        .arg(project)
        .args(["ls-files", "-z", "--", "."])
        .output();
    if let Ok(o) = tracked
        && o.status.success()
        && !o.stdout.is_empty()
    {
        return Ok(o
            .stdout
            .split(|b| *b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| PathBuf::from(String::from_utf8_lossy(s).into_owned()))
            .collect());
    }
    let mut v = Vec::new();
    walk_files(project, project, &mut v)?;
    v.sort();
    Ok(v)
}

fn copy_project(project: &Path, dest: &Path) -> Res<()> {
    fs::create_dir_all(dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    for rel in project_files(project)? {
        let to = dest.join(&rel);
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        fs::copy(project.join(&rel), &to)
            .map_err(|e| format!("{}: {e}", project.join(&rel).display()))?;
    }
    Ok(())
}

fn git(dir: &Path, args: &[&str]) -> Res<()> {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=bench@example.invalid",
            "-c",
            "user.name=bench",
        ])
        .args(["-c", "commit.gpgsign=false"])
        .args(args)
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git {}: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// The dla binary as an absolute path: it is run from inside each fixture copy.
fn find_dla(arg: Option<&Path>) -> PathBuf {
    let found = if let Some(p) = arg {
        p.to_path_buf()
    } else {
        let exe = if cfg!(windows) { "dla.exe" } else { "dla" };
        let beside = env::current_exe()
            .ok()
            .and_then(|me| me.parent().map(|d| d.join(exe)))
            .filter(|p| p.is_file());
        beside.unwrap_or_else(|| {
            let dev = Path::new("target/debug").join(exe);
            if dev.is_file() {
                dev
            } else {
                PathBuf::from("dla")
            }
        })
    };
    if found.components().count() > 1 {
        std::path::absolute(&found).unwrap_or(found)
    } else {
        found
    }
}

struct Shared<'a> {
    args: &'a DynamicArgs,
    bin: &'a Path,
    plugin_root: &'a Path,
    work: &'a Path,
    log_root: &'a Path,
    /// The `PATH` directory of a `--without` run.
    path_dir: Option<&'a Path>,
}

fn run_fixture(sh: &Shared, fixture_dir: &Path) -> FixtureRow {
    let id = fixture_dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut row = FixtureRow {
        id: id.clone(),
        ..FixtureRow::default()
    };
    let expected = match load_expected(&sh.args.fixtures, &id) {
        Ok(e) => e,
        Err(e) => {
            row.failures.push(e);
            return row;
        }
    };
    row.category = expected.category.as_str().to_owned();
    row.entry = expected.entry.as_str().to_owned();
    let mut plan = Plan {
        entry: expected.entry,
        runtimes: expected.runtimes.as_ref().map_or(1, Vec::len).max(1),
        artifacts: Vec::new(),
    };
    let project = sh.work.join(&id);
    let setup = copy_project(&fixture_dir.join("project"), &project)
        .and_then(|()| git(&project, &["init", "-q"]))
        .and_then(|()| git(&project, &["add", "-A"]))
        .and_then(|()| git(&project, &["commit", "-q", "-m", "fixture"]));
    if let Err(e) = setup {
        row.failures.push(format!("scratch copy: {e}"));
        return row;
    }
    let data_dir = sh.work.join("data").join(&id);
    let _ = fs::create_dir_all(&data_dir);
    if plan.entry == Entry::Artifact {
        plan.artifacts = project_files(&project)
            .unwrap_or_default()
            .into_iter()
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .filter(|p| p.ends_with(".hprof") || p.ends_with(".hprof.gz"))
            .collect();
        plan.artifacts.sort();
    }
    let mut fx = Fx {
        plan,
        bin: sh.bin,
        project: project.clone(),
        plugin_root: sh.plugin_root,
        data_dir,
        log_dir: sh.log_root.join(&id),
        seq: 0,
        row,
        timeout: Duration::from_secs(sh.args.command_timeout.max(1)),
        timed_out: false,
        worker: None,
        path_dir: sh.path_dir.map(Path::to_path_buf),
        unset_vars: without::home_vars_for(&sh.args.without),
        pending: Vec::new(),
    };
    let _ = fs::remove_dir_all(&fx.log_dir);
    drive(&mut fx);
    let mut row = std::mem::take(&mut fx.row);
    if let Ok(run_dir) = fx.run_dir() {
        let view = View::read(&run_dir.join("ledger.jsonl"));
        grade_row(&mut row, &sh.args.fixtures, &run_dir.join("ledger.jsonl"));
        for m in check_m3_rules(&view, expected.entry) {
            row.failures.push(format!("M3 rule: {m}"));
        }
        if sh.args.check_dump_backed {
            let bad = check_dump_backed(&view, &expected);
            // Dump-backed evidence comes from the dump and allocation backends: when the
            // machine lacks one, the check cannot hold, and saying so beats a failure.
            let missing = missing_dump_backends(&run_dir);
            for m in bad {
                if missing.is_empty() {
                    row.failures.push(format!("dump check: {m}"));
                } else {
                    row.steps.push(format!(
                        "dump check not applied ({} missing here): {m}",
                        missing.join(", ")
                    ));
                }
            }
        }
        row.diagnostics = diagnostics(&run_dir);
        match residual_share(&view) {
            Ok(s) => row.unexplained_growth_share = s,
            Err(e) => row.failures.push(e),
        }
        if !sh.args.without.is_empty() {
            let (bad, seen) = without::check_coverage(&run_dir, &sh.args.without);
            row.hidden_backend_rows = seen.values().sum();
            row.failures.extend(without::check_attempted(
                &row.captures,
                &seen,
                &sh.args.without,
            ));
            row.hidden_rows_by_tool = seen;
            row.failures.extend(bad);
        }
        collect_run_files(&run_dir, &fx.log_dir);
    } else {
        row.failures.push("dla run init failed".to_owned());
    }
    if sh.args.without.is_empty() {
        apply_expected_failures(&mut row, EXPECTED_FAILURES);
    } else {
        settle_without(&mut row);
    }
    if let Ok(text) = serde_json::to_string_pretty(&row) {
        let _ = fs::write(fx.log_dir.join("row.json"), text + "\n");
    }
    if !sh.args.keep_work {
        let _ = fs::remove_dir_all(&project);
        let _ = fs::remove_dir_all(sh.work.join("data").join(&id));
    }
    row
}

/// Settles the failures of a `--without` row: leak gaps are accepted only when the run saw a
/// coverage row of a hidden tool; otherwise the row is judged as in a normal run.
fn settle_without(row: &mut FixtureRow) {
    if row.hidden_backend_rows > 0 {
        accept_hidden_tool_gaps(row);
    } else {
        apply_expected_failures(row, EXPECTED_FAILURES);
    }
}

/// In a `--without` run a leak may stay unconfirmed because its backend is hidden. Such a
/// failure is reported as an expected one, but only for a fixture whose run wrote a coverage
/// row of a hidden tool (the caller checks); every other failure (a crash of dla, a timeout, a
/// coverage row that does not read `missing`) stays a failure.
fn accept_hidden_tool_gaps(row: &mut FixtureRow) {
    let gap = |f: &String| {
        (f.starts_with("leak ")
            && (f.ends_with("is not confirmed") || f.contains("is confirmed with holder")))
            // The evidence of a hidden backend cannot be cited.
            || f.starts_with("dump check: ")
    };
    let (gaps, rest): (Vec<String>, Vec<String>) = std::mem::take(&mut row.failures)
        .into_iter()
        .partition(|f| gap(f));
    row.failures = rest;
    if !gaps.is_empty() && row.failures.is_empty() {
        row.xfail = Some("a tool is hidden by --without".to_owned());
        row.expected_failures = gaps;
    } else {
        row.failures.extend(gaps);
    }
}

/// Moves the failures that `list` accepts for this fixture out of `failures`. A fixture on
/// the list that passes, or that fails with anything but the listed text, keeps (or gets) a
/// failure saying so, so the list can neither go stale nor hide a regression.
pub fn apply_expected_failures(row: &mut FixtureRow, list: &[ExpectedFailure]) {
    let mine: Vec<&ExpectedFailure> = list.iter().filter(|e| e.fixture == row.id).collect();
    if mine.is_empty() {
        return;
    }
    if row.failures.is_empty() {
        row.failures.push(
            "listed as an expected failure but it passes: remove it from EXPECTED_FAILURES"
                .to_owned(),
        );
        return;
    }
    let accepted = |f: &String| mine.iter().any(|e| e.failure == f);
    let unexpected: Vec<String> = row
        .failures
        .iter()
        .filter(|f| !accepted(f))
        .map(|f| format!("{f} (not the expected failure)"))
        .collect();
    let missing: Vec<&str> = mine
        .iter()
        .map(|e| e.failure)
        .filter(|f| !row.failures.iter().any(|x| x == f))
        .collect();
    if !unexpected.is_empty() || !missing.is_empty() {
        for m in missing {
            row.failures
                .push(format!("the expected failure {m:?} did not occur"));
        }
        row.failures.extend(unexpected);
        // Keep each message once, in order.
        let mut seen = BTreeSet::new();
        row.failures.retain(|f| seen.insert(f.clone()));
        return;
    }
    row.xfail = Some(mine.iter().map(|e| e.reason).collect::<Vec<_>>().join("; "));
    row.expected_failures = std::mem::take(&mut row.failures);
}

fn drive(fx: &mut Fx) {
    match fx.plan.entry {
        Entry::Artifact => return artifact::drive(fx),
        Entry::Symptom => return artifact::drive_symptom(fx),
        Entry::Repository => {}
    }
    if !fx.call(&["run", "init", "--budget", RUN_BUDGET]).ok {
        return;
    }
    let mut runtimes: Vec<String> = Vec::new();
    let mut builds = Value::Null;
    let p = fx.call(&["probe", "--build"]);
    if p.ok {
        runtimes = strings(&p.json["summary"]["runtimes"]);
        builds = p.json["summary"]["builds"].clone();
    }
    fx.row.runtimes = runtimes.clone();
    if fx.plan.runtimes >= 2 {
        // Two or more runtimes: one worker per runtime, then the merge.
        return workers::drive(fx, &runtimes, &builds);
    }
    for lang in &runtimes {
        fx.call(&["sweep", "--lang", lang]);
        fx.call(&["census", "--lang", lang]);
    }
    fx.call(&["workload", "discover"]);
    let (ran, missing) = make_harnesses(fx, &runtimes, &builds);
    fx.row.harnesses = ran.iter().map(|r| r.harness.clone()).collect();
    fx.row.series = ran.iter().flat_map(|r| r.series.clone()).collect();
    if !ran.is_empty() {
        analyse(fx, &ran);
    }
    // Only when nothing ran at all is the missing harness the reason for the rest.
    finish(fx, &ran, if ran.is_empty() { missing } else { None }, true);
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// The text table.
pub fn render_table(res: &DynamicResult) -> String {
    let mut s = format!("archetypes: {}\n\n", res.archetypes);
    let w = res
        .fixtures
        .iter()
        .map(|f| f.id.len())
        .max()
        .unwrap_or(7)
        .max(7);
    s.push_str(&format!(
        "{:<w$}  {:<14}  {:<8}  {:<4}  {:>6}  {:>6}  {:>4}  {:>4}  {:>5}  {:>8}  {:>5}  {:>10}  result\n",
        "fixture",
        "category",
        "runtime",
        "ran",
        "series",
        "passed",
        "conf",
        "refu",
        "claim",
        "rule-out",
        "leaks",
        "ops/ev/B",
    ));
    for f in &res.fixtures {
        s.push_str(&format!(
            "{:<w$}  {:<14}  {:<8}  {:<4}  {:>6}  {:>6}  {:>4}  {:>4}  {:>5}  {:>8}  {:>5}  {:>10}  {}\n",
            f.id,
            f.category,
            if f.runtimes.is_empty() && f.entry != "repository" {
                f.entry.clone()
            } else {
                f.runtimes.join(",")
            },
            if f.harnesses.is_empty() { "no" } else { "yes" },
            f.series.len(),
            f.passed_columns,
            f.confirmed,
            f.refuted,
            if f.claims { "yes" } else { "no" },
            if f.rule_outs { "yes" } else { "no" },
            format!("{}/{}", f.leaks_found, f.leaks_expected),
            ops_cell(f),
            if let Some(why) = &f.skipped {
                format!("skipped: {why}")
            } else if let Some(why) = &f.xfail {
                format!("xfail: {why}")
            } else if f.failures.is_empty() {
                "pass".to_owned()
            } else {
                format!("FAIL: {}", f.failures.join("; "))
            },
        ));
    }
    let xfail = res.fixtures.iter().filter(|f| f.xfail.is_some()).count();
    let pass = res
        .fixtures
        .iter()
        .filter(|f| f.failures.is_empty() && f.xfail.is_none())
        .count();
    s.push_str(&format!(
        "\ntotals: {pass}/{} pass, {xfail} xfail (expected failures, not counted as failures), {} confirmed, {} refuted, {} with a claim, {} with a rule-out\n",
        res.fixtures.len(),
        res.fixtures.iter().map(|f| f.confirmed).sum::<usize>(),
        res.fixtures.iter().map(|f| f.refuted).sum::<usize>(),
        res.fixtures.iter().filter(|f| f.claims).count(),
        res.fixtures.iter().filter(|f| f.rule_outs).count(),
    ));
    s.push_str(&format!("logs: {}\n", res.log_dir));
    s
}

/// The compact summarize cell of the table: ops run, evidence ids, largest stdout in bytes.
fn ops_cell(f: &FixtureRow) -> String {
    if f.ops.is_empty() && f.max_stdout == 0 {
        return "-".to_owned();
    }
    format!(
        "{}/{}/{}",
        f.ops.len(),
        f.summarize_evidence.len(),
        f.max_stdout
    )
}

/// Runs the driver.
pub fn run(args: &DynamicArgs) -> Res<String> {
    let bin = find_dla(args.dla_bin.as_deref());
    let data_src = args.repo_root.join("data");
    if !data_src.is_dir() {
        return Err(format!("{}: no data directory", data_src.display()));
    }
    let work = env::temp_dir().join(format!("dla-bench-dynamic-{}", std::process::id()));
    let _ = fs::remove_dir_all(&work);
    fs::create_dir_all(&work).map_err(|e| format!("{}: {e}", work.display()))?;
    let log_root = match (&args.log_dir, &args.json) {
        (Some(d), _) => d.clone(),
        (None, Some(j)) => j.with_extension("logs"),
        (None, None) => {
            env::temp_dir().join(format!("dla-bench-dynamic-logs-{}", std::process::id()))
        }
    };
    let log_root = std::path::absolute(&log_root).unwrap_or(log_root);
    let result = run_in(args, &bin, &data_src, &work, &log_root);
    if args.keep_work {
        eprintln!("scratch kept: {}", work.display());
    } else {
        let _ = fs::remove_dir_all(&work);
    }
    let res = result?;
    print!("{}", render_table(&res));
    if let Some(out) = &args.json {
        let text = serde_json::to_string_pretty(&res).map_err(|e| e.to_string())?;
        fs::write(out, text + "\n").map_err(|e| format!("{}: {e}", out.display()))?;
    }
    if res.failures.is_empty() {
        let xfail = res.fixtures.iter().filter(|f| f.xfail.is_some()).count();
        Ok(format!(
            "ok: {} fixture(s) pass, {xfail} expected failure(s)",
            res.fixtures.len() - xfail
        ))
    } else {
        // What the policy decided for each failing fixture, so a CI log shows why without
        // the uploaded logs.
        let mut trail = String::new();
        for f in res.fixtures.iter().filter(|f| !f.failures.is_empty()) {
            trail.push_str(&format!("\n{} decisions:", f.id));
            for line in f.steps.iter().chain(f.errors.iter()) {
                trail.push_str("\n  ");
                trail.push_str(line);
            }
            for line in &f.diagnostics {
                trail.push_str("\n  | ");
                trail.push_str(line);
            }
        }
        Err(format!(
            "{} failure(s):\n{}{trail}",
            res.failures.len(),
            res.failures.join("\n")
        ))
    }
}

fn run_in(
    args: &DynamicArgs,
    bin: &Path,
    data_src: &Path,
    work: &Path,
    log_root: &Path,
) -> Res<DynamicResult> {
    let plugin_root = work.join("plugin");
    copy_tree(data_src, &plugin_root.join("data"))?;
    let off = args.archetypes == Archetypes::Off;
    if off {
        apply_archetypes_off(&plugin_root)?;
    }
    fs::create_dir_all(log_root).map_err(|e| format!("{}: {e}", log_root.display()))?;
    let path_dir = if args.without.is_empty() {
        None
    } else {
        if let Some(bad) = args
            .without
            .iter()
            .find(|t| t.is_empty() || t.contains(['/', '\\']))
        {
            return Err(format!("--without: {bad:?} is not a tool name"));
        }
        let dir = without::path_dir_of(work);
        let path = env::var_os("PATH").unwrap_or_default();
        let hidden = without::build_path_dir(&args.without, &path, &dir)?;
        eprintln!(
            "--without {}: {} link(s) in the changed PATH directories",
            args.without.join(","),
            hidden.links
        );
        Some(PathBuf::from(hidden.path))
    };
    let dirs: Vec<PathBuf> = fixture_dirs(&args.fixtures)?
        .into_iter()
        .filter(|d| {
            let name = d.file_name().map(|n| n.to_string_lossy().into_owned());
            name.as_ref()
                .is_some_and(|n| args.only.as_ref().is_none_or(|o| n.contains(o.as_str())))
                && d.join("expected.toml").is_file()
        })
        .collect();
    if dirs.is_empty() {
        return Err(format!("no fixtures under {}", args.fixtures.display()));
    }
    let sh = Shared {
        args,
        bin,
        plugin_root: &plugin_root,
        work,
        log_root,
        path_dir: path_dir.as_deref(),
    };
    let queue: Mutex<VecDeque<PathBuf>> = Mutex::new(dirs.into());
    let done: Mutex<Vec<FixtureRow>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        for _ in 0..args.parallel.max(1) {
            scope.spawn(|| {
                loop {
                    let next = queue.lock().ok().and_then(|mut q| q.pop_front());
                    let Some(dir) = next else { return };
                    let row = run_fixture(&sh, &dir);
                    eprintln!(
                        "{}: {}",
                        row.id,
                        if row.xfail.is_some() {
                            "xfail"
                        } else if row.failures.is_empty() {
                            "pass"
                        } else {
                            "FAIL"
                        }
                    );
                    if let Ok(mut d) = done.lock() {
                        d.push(row);
                    }
                }
            });
        }
    });
    let mut fixtures = done.into_inner().map_err(|e| e.to_string())?;
    fixtures.sort_by(|a, b| a.id.cmp(&b.id));
    let mut failures: Vec<String> = fixtures
        .iter()
        .flat_map(|f| f.failures.iter().map(|m| format!("{}: {m}", f.id)))
        .collect();
    for tool in &args.without {
        let rows: usize = fixtures
            .iter()
            .map(|f| f.hidden_rows_by_tool.get(tool).copied().unwrap_or(0))
            .sum();
        if rows == 0 {
            failures.push(format!(
                "hidden tool {tool}: no coverage row depends on it in any fixture, so hiding it was not exercised"
            ));
        }
    }
    Ok(DynamicResult {
        archetypes: if off { "off" } else { "on" }.to_owned(),
        parallel: args.parallel.max(1),
        log_dir: log_root.display().to_string(),
        fixtures,
        failures,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listed() -> [ExpectedFailure; 1] {
        [ExpectedFailure {
            fixture: "fx",
            failure: "leak L1 is not confirmed",
            reason: "because",
        }]
    }

    #[test]
    fn leak_gaps_need_a_hidden_backend_row() {
        let mut r = row(&["leak L1 (a:1) is not confirmed"]);
        r.id = "unlisted".into();
        settle_without(&mut r);
        assert_eq!(r.failures.len(), 1, "no hidden row: the failure stays");
        assert!(r.xfail.is_none());
        let mut r = row(&["leak L1 (a:1) is not confirmed"]);
        r.hidden_backend_rows = 1;
        settle_without(&mut r);
        assert!(r.failures.is_empty());
        assert!(r.xfail.is_some());
    }

    fn row(failures: &[&str]) -> FixtureRow {
        FixtureRow {
            id: "fx".into(),
            failures: failures.iter().map(|f| (*f).to_owned()).collect(),
            ..FixtureRow::default()
        }
    }

    #[test]
    fn a_listed_failure_becomes_an_xfail() {
        let mut r = row(&["leak L1 is not confirmed"]);
        apply_expected_failures(&mut r, &listed());
        assert!(r.failures.is_empty());
        assert_eq!(r.xfail.as_deref(), Some("because"));
        assert_eq!(r.expected_failures, ["leak L1 is not confirmed"]);
    }

    #[test]
    fn a_listed_fixture_that_passes_or_fails_differently_fails_the_run() {
        let mut r = row(&[]);
        apply_expected_failures(&mut r, &listed());
        assert!(r.xfail.is_none());
        assert!(r.failures[0].contains("passes"), "{:?}", r.failures);

        let mut r = row(&["leak L1 is not confirmed", "a claim on a decoy control"]);
        apply_expected_failures(&mut r, &listed());
        assert!(r.xfail.is_none());
        assert!(r.failures.iter().any(|f| f.contains("decoy control")));

        let mut r = row(&["something else"]);
        apply_expected_failures(&mut r, &listed());
        assert!(r.xfail.is_none());
        assert!(r.failures.iter().any(|f| f.contains("did not occur")));

        // Other fixtures are untouched.
        let mut r = row(&["boom"]);
        r.id = "other".into();
        apply_expected_failures(&mut r, &listed());
        assert_eq!(r.failures, ["boom"]);
    }

    #[test]
    fn the_shipped_list_names_the_rc_cycle_fixture() {
        assert!(
            EXPECTED_FAILURES
                .iter()
                .any(|e| e.fixture == "rust-tree-parent-links" && e.reason.contains("parent"))
        );
    }

    #[test]
    fn the_jni_fixture_has_no_expected_failure_left() {
        assert!(
            EXPECTED_FAILURES
                .iter()
                .all(|e| e.fixture != "jni-order-codec")
        );
    }

    #[test]
    fn the_readme_table_matches_the_shipped_list() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../README.md");
        let readme = fs::read_to_string(path).unwrap();
        let rows: Vec<&str> = readme
            .lines()
            .filter(|l| l.starts_with("| `") && l.contains("` | `leak "))
            .collect();
        assert_eq!(rows.len(), EXPECTED_FAILURES.len(), "{rows:?}");
        for e in EXPECTED_FAILURES {
            let want = format!("| `{}` | `{}` |", e.fixture, e.failure);
            assert!(rows.iter().any(|r| r.starts_with(&want)), "{want}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_stuck_command_and_its_grandchild_are_killed_at_the_limit() {
        let started = Instant::now();
        // The background sleep inherits the pipes and would hold them for a minute.
        let mut c = Command::new("sh");
        c.args(["-c", "echo partial; sleep 60 & sleep 60"]);
        let r = run_limited(&mut c, Duration::from_millis(400));
        assert!(started.elapsed() < Duration::from_secs(10));
        match r {
            Ended::TimedOut(out, _) => assert!(out.contains("partial"), "{out}"),
            _ => panic!("expected a timeout"),
        }
        let mut c = Command::new("sh");
        c.args(["-c", "echo hi; exit 3"]);
        match run_limited(&mut c, Duration::from_secs(10)) {
            Ended::Done(3, out, _) => assert_eq!(out.trim(), "hi"),
            _ => panic!("expected exit 3"),
        }
    }

    fn view(tag: &str, lines: &[&str]) -> View {
        let dir = std::env::temp_dir().join(format!("dla-dyn-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let p = dir.join(format!("{tag}.jsonl"));
        fs::write(&p, lines.join("\n")).unwrap();
        View::read(&p)
    }

    const EV_PASS: &str = r#"{"type":"evidence","evidence_id":"E1","authority":"test","kind":"growth-series","result":{"gate":"pass"}}"#;
    const ADD: &str = r#"{"type":"finding_added","finding":{"id":"F1","origin":"census","site":"a.py:1","holder":"static/global"}}"#;
    const SIB: &str =
        r#"{"type":"finding_added","finding":{"id":"F1.n1","parent":"F1","hypothesis":"N-WARM"}}"#;

    #[test]
    fn a_complete_confirmation_passes_the_m3_rules() {
        let v = view(
            "ok",
            &[
                EV_PASS,
                ADD,
                SIB,
                r#"{"type":"transition","finding":"F1","from":"candidate","to":"under_test","evidence":[]}"#,
                r#"{"type":"transition","finding":"F1.n1","from":"candidate","to":"refuted","evidence":[]}"#,
                r#"{"type":"transition","finding":"F1","from":"under_test","to":"confirmed","evidence":["E1"]}"#,
            ],
        );
        assert!(check_m3_rules(&v, Entry::Repository).is_empty());
    }

    #[test]
    fn an_artifact_confirmation_names_an_owner_and_never_a_site() {
        let dump_growth = r#"{"type":"evidence","evidence_id":"E1","authority":"dump","kind":"growth-series","result":{"gate":"pass"}}"#;
        let owner_add = r#"{"type":"finding_added","finding":{"id":"F1","origin":"growth-series","owner":"a.B.c","holder":"static/global"}}"#;
        let steps = [
            r#"{"type":"transition","finding":"F1","from":"candidate","to":"under_test","evidence":[]}"#,
            r#"{"type":"transition","finding":"F1.n1","from":"candidate","to":"refuted","evidence":[]}"#,
            r#"{"type":"transition","finding":"F1","from":"under_test","to":"confirmed","evidence":["E1"]}"#,
        ];
        let mut ok = vec![dump_growth, owner_add, SIB];
        ok.extend(steps);
        assert!(check_m3_rules(&view("art-ok", &ok), Entry::Artifact).is_empty());
        // A site on an artifact run is a source claim the dumps cannot support.
        let mut sited = vec![dump_growth, ADD, SIB];
        sited.extend(steps);
        let bad = check_m3_rules(&view("art-site", &sited), Entry::Artifact).join("|");
        assert!(
            bad.contains("no owner") && bad.contains("source site"),
            "{bad}"
        );
        // Growth from a harness (authority test) does not confirm an artifact finding.
        let mut tested = vec![EV_PASS, owner_add, SIB];
        tested.extend(steps);
        let bad = check_m3_rules(&view("art-test", &tested), Entry::Artifact).join("|");
        assert!(bad.contains("no growth evidence"), "{bad}");
    }

    #[test]
    fn a_confirmation_without_sibling_refutation_is_reported() {
        let v = view(
            "nosib",
            &[
                EV_PASS,
                ADD,
                SIB,
                r#"{"type":"transition","finding":"F1","from":"under_test","to":"confirmed","evidence":["E1"]}"#,
            ],
        );
        let bad = check_m3_rules(&v, Entry::Repository);
        assert!(
            bad.iter().any(|b| b.contains("refuted N-* sibling")),
            "{bad:?}"
        );
    }

    #[test]
    fn missing_holder_and_failed_gate_are_reported() {
        let v = view(
            "gate",
            &[
                r#"{"type":"evidence","evidence_id":"E1","authority":"test","kind":"growth-series","result":{"gate":"fail"}}"#,
                r#"{"type":"finding_added","finding":{"id":"F1","site":"a.py:1"}}"#,
                r#"{"type":"transition","finding":"F1","from":"under_test","to":"confirmed","evidence":["E1"]}"#,
            ],
        );
        let bad = check_m3_rules(&v, Entry::Repository).join("|");
        assert!(
            bad.contains("gate pass") && bad.contains("no holder"),
            "{bad}"
        );
    }

    #[test]
    fn annotations_fill_the_holder() {
        let v = view(
            "annot",
            &[
                r#"{"type":"finding_added","finding":{"id":"F1","site":"a.py:1"}}"#,
                r#"{"type":"finding_annotated","finding":"F1","holder":"blocked-stack"}"#,
            ],
        );
        assert_eq!(
            v.finding("F1").unwrap().holder.as_deref(),
            Some("blocked-stack")
        );
    }

    #[test]
    fn residual_share_is_computed_from_rows() {
        let v = view(
            "resid",
            &[
                r#"{"type":"evidence","evidence_id":"E1","authority":"test","kind":"residual","result":{"rows":[{"residual":25.0,"total":100.0},{"residual":0.0,"total":100.0}]}}"#,
            ],
        );
        assert_eq!(residual_share(&v), Ok(Some(0.125)));
        assert_eq!(residual_share(&View::default()), Ok(None));
    }

    fn col(name: &str, kind: &str) -> Col {
        Col {
            name: name.into(),
            kind: kind.into(),
            region: "heap".into(),
            shape: String::new(),
            pass: true,
            candidates: vec![],
        }
    }

    #[test]
    fn holders_follow_the_column_without_a_census_entry() {
        let f = Fnd {
            id: "F1".into(),
            site: Some("x.go:3".into()),
            ..Fnd::default()
        };
        let none = Path::new("/nonexistent");
        assert_eq!(
            pick_holder(none, &f, &col("go.goroutines", "gauge")).0,
            "blocked-stack"
        );
        assert_eq!(
            pick_holder(none, &f, &col("node.timers", "gauge")).0,
            "registration-table"
        );
        assert_eq!(
            pick_holder(none, &f, &col("probe:a.b", "gauge")).0,
            "static/global"
        );
    }

    #[test]
    fn trend_evidence_follows_the_column_kind() {
        let v = view(
            "kind",
            &[
                r#"{"type":"evidence","evidence_id":"E1","authority":"test","kind":"growth-series","result":{}}"#,
                r#"{"type":"evidence","evidence_id":"E2","authority":"test","kind":"unit-count-series","result":{}}"#,
            ],
        );
        let ids = vec!["E1".to_owned(), "E2".to_owned()];
        assert_eq!(
            trend_evidence(&v, &ids, &col("n", "counter")).as_deref(),
            Some("E2")
        );
        assert_eq!(
            trend_evidence(&v, &ids, &col("n", "gauge")).as_deref(),
            Some("E1")
        );
    }

    const EXPECTED: &str = r#"schema_version = 1
id = "fx"
runtime = "python"
category = "archetype-leak"
dynamic_confirmation = true

[task]
prompt = "p"

[[leak]]
id = "L1"
site = "a.py:1"
holder = "static/global"
region = "heap"
cause = "L-COLLECT"
"#;

    fn expected(extra: &str) -> Expected {
        toml::from_str(&format!("{EXPECTED}{extra}")).unwrap()
    }

    fn confirmed_view(tag: &str, ev: &str, supports_ev: bool) -> View {
        let add = if supports_ev {
            r#"{"type":"finding_added","finding":{"id":"F1","origin":"census","site":"a.py:1","holder":"static/global","supports":["E2"]}}"#
        } else {
            ADD
        };
        view(
            tag,
            &[
                EV_PASS,
                ev,
                add,
                r#"{"type":"transition","finding":"F1","from":"under_test","to":"confirmed","evidence":["E1"]}"#,
            ],
        )
    }

    #[test]
    fn missing_backends_and_diagnostics_are_read_from_the_run_dir() {
        let d = tempfile::tempdir().unwrap();
        let run = d.path();
        fs::write(
            run.join("coverage.json"),
            r#"{"workers":{"native":{"backends":{"valgrind":{"status":"missing"},"heaptrack":{"status":"ran"},"sweep":{"status":"missing"}}}}}"#,
        )
        .unwrap();
        assert_eq!(missing_dump_backends(run), ["valgrind"]);
        fs::create_dir_all(run.join("trend")).unwrap();
        fs::write(
            run.join("trend/T1.json"),
            r#"{"id":"T1","series":"S1","columns":[{"name":"os.rss","gate":"pass","shape":"stair","sen":{"slope":63.1},"noise":63763.7},{"name":"py.traced","gate":"fail"}]}"#,
        )
        .unwrap();
        fs::write(
            run.join("ledger.jsonl"),
            r#"{"type":"evidence","evidence_id":"E13","kind":"discriminator","result":{"columns":[{"name":"os.vm_data","favors":"inconclusive","plateau":{"verdict":"plateau"},"purge":{"verdict":"retained"}}]}}"#,
        )
        .unwrap();
        let lines = diagnostics(run);
        assert_eq!(lines.len(), 2, "{lines:?}");
        assert!(
            lines[0].starts_with("T1 of S1 passed: os.rss stair slope 63.100"),
            "{lines:?}"
        );
        assert!(!lines[0].contains("py.traced"), "{lines:?}");
        assert_eq!(
            lines[1],
            "alloc-test E13: os.vm_data inconclusive (plateau main - extra - rise - noise -, purge retained)"
        );
        // Nothing in an empty run dir.
        let e = tempfile::tempdir().unwrap();
        assert!(diagnostics(e.path()).is_empty());
        assert!(missing_dump_backends(e.path()).is_empty());
    }

    #[test]
    fn dump_backed_leaks_need_retention_or_allocation_evidence() {
        let exp = expected("dump_backed = true\n");
        let none = confirmed_view("dump-none", EV_PASS, false);
        let bad = check_dump_backed(&none, &exp);
        assert_eq!(bad.len(), 1, "{bad:?}");
        assert!(bad[0].contains("L1") && bad[0].contains("F1"), "{bad:?}");
        for (kind, auth, ok) in [
            ("retention-path", "dump", true),
            ("alloc-stack", "live", true),
            ("histogram-diff", "dump", true),
            ("alloc-stack", "test", false),
            ("growth-series", "dump", false),
        ] {
            let ev = format!(
                r#"{{"type":"evidence","evidence_id":"E2","authority":"{auth}","kind":"{kind}","result":{{}}}}"#
            );
            let v = confirmed_view(&format!("dump-{kind}-{auth}"), &ev, true);
            assert_eq!(check_dump_backed(&v, &exp).is_empty(), ok, "{kind} {auth}");
        }
    }

    #[test]
    fn the_dump_check_ignores_unmarked_leaks_and_unconfirmed_ones() {
        let v = confirmed_view("dump-unmarked", EV_PASS, false);
        assert!(check_dump_backed(&v, &expected("")).is_empty());
        let open = view("dump-open", &[EV_PASS, ADD]);
        assert!(check_dump_backed(&open, &expected("dump_backed = true\n")).is_empty());
    }

    #[test]
    fn the_holder_of_a_confirmed_leak_is_checked() {
        let t = std::env::temp_dir().join(format!("dla-dyn-holder-{}", std::process::id()));
        let fx = t.join("fx");
        fs::create_dir_all(&fx).unwrap();
        fs::write(fx.join("expected.toml"), EXPECTED).unwrap();
        let ledger = |holder: &str| {
            let text = [
                format!(
                    r#"{{"type":"finding_added","finding":{{"id":"F1","origin":"census","site":"a.py:1","holder":"{holder}"}}}}"#
                ),
                r#"{"type":"transition","finding":"F1","from":"under_test","to":"confirmed","evidence":[]}"#.to_owned(),
            ]
            .join("\n");
            let p = t.join(format!("{}.jsonl", holder.replace('/', "-")));
            fs::write(&p, text).unwrap();
            p
        };
        let mut good = FixtureRow {
            id: "fx".into(),
            ..FixtureRow::default()
        };
        grade_row(&mut good, &t, &ledger("static/global"));
        assert!(good.failures.is_empty(), "{:?}", good.failures);
        let mut bad = FixtureRow {
            id: "fx".into(),
            ..FixtureRow::default()
        };
        grade_row(&mut bad, &t, &ledger("container-field"));
        assert_eq!(
            bad.failures,
            ["leak L1 (a.py:1) is confirmed with holder container-field, expected static/global"]
        );
        let _ = fs::remove_dir_all(&t);
    }

    #[test]
    fn a_hidden_tool_gap_is_accepted_only_when_nothing_else_failed() {
        let mut r = row(&["leak L1 (a.py:1) is not confirmed"]);
        accept_hidden_tool_gaps(&mut r);
        assert!(r.failures.is_empty() && r.xfail.is_some());
        let mut r = row(&[
            "leak L1 (a.py:1) is not confirmed",
            "hidden tool jcmd: boom",
        ]);
        accept_hidden_tool_gaps(&mut r);
        assert_eq!(r.failures.len(), 2);
        assert!(r.xfail.is_none());
    }
}
