//! The `auto` budget, `dla run extend` and the `budget` object of `dla run status`,
//! through the binary.

use std::path::Path;

use assert_cmd::Command;
use chrono::{DateTime, TimeDelta, Utc};
use dla_core::{
    Authority, EvidenceKind, Ledger, NewEvidence, NewFinding, Origin, RunState, Source, Tool,
    evidence::EvidenceCoverage,
    state::{RunStatus, WorkerStatus, now_utc},
};
use serde_json::{Value, json};

struct Run {
    dir: tempfile::TempDir,
    run_id: String,
}

fn base(project: &Path) -> Command {
    let mut c = Command::cargo_bin("dla").unwrap();
    for var in [
        "DLA_RUN",
        "DLA_WORKER",
        "DLA_DEFAULT_BUDGET",
        "DLA_ARCHETYPES",
        "CLAUDE_SESSION_ID",
        "CLAUDE_CODE_SESSION_ID",
        "CLAUDE_PROJECT_DIR",
        "DLA_CONSENT_TIERS",
        "DLA_CONSENT_RAW_VALUES",
    ] {
        c.env_remove(var);
    }
    c.arg("--project").arg(project);
    c
}

fn mins(n: i64) -> TimeDelta {
    TimeDelta::try_minutes(n).unwrap()
}

impl Run {
    fn path(&self) -> &Path {
        self.dir.path()
    }
    /// What a tool leaves when it refuses a command for time: with it, `time_budget` is a
    /// reason the ledger accepts while the budget has room.
    fn note_time_refusal(&self) {
        use std::io::Write;
        let ts = (now_utc() + chrono::TimeDelta::seconds(5))
            .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.run_dir().join(dla_core::toolctx::TIME_REFUSALS_FILE))
            .unwrap();
        writeln!(f, "{{\"ts\":\"{ts}\",\"worker\":null}}").unwrap();
    }
    fn run_dir(&self) -> std::path::PathBuf {
        self.path().join(".dla").join(&self.run_id)
    }
    fn out(&self, args: &[&str]) -> (i32, Value, String) {
        let o = base(self.path()).args(args).output().unwrap();
        let stdout = String::from_utf8(o.stdout).unwrap();
        (
            o.status.code().unwrap(),
            serde_json::from_str(&stdout).unwrap_or(Value::Null),
            String::from_utf8(o.stderr).unwrap(),
        )
    }
    fn ok(&self, args: &[&str]) -> Value {
        let (code, v, err) = self.out(args);
        assert_eq!(code, 0, "{args:?}: {v} {err}");
        v
    }
    fn state(&self) -> RunState {
        RunState::load(&self.run_dir()).unwrap()
    }
    fn update(&self, f: impl FnOnce(&mut RunState)) {
        RunState::update(&self.run_dir(), |s| {
            f(s);
            Ok(())
        })
        .unwrap();
    }
    fn answer_intake(&self) {
        for item in [
            "intake.target",
            "intake.scope",
            "intake.symptom",
            "intake.runtimes",
            "intake.build",
            "intake.workload",
            "intake.consent",
            "intake.budget",
        ] {
            self.ok(&[
                "ledger",
                "resolve",
                item,
                "--status",
                "answered",
                "--evidence",
                "E1",
            ]);
        }
    }
    fn gate(&self, stage: &str) -> (i32, Value) {
        let (c, v, _) = if stage == "final" {
            self.out(&["gate", "check", "--final"])
        } else {
            self.out(&["gate", "check", "--stage", stage])
        };
        (c, v)
    }
    /// `python` and `go` candidates (one static hit each carries the language).
    fn findings(&self, py: usize, go: usize) {
        let mut l = Ledger::load(&self.run_dir()).unwrap();
        let mut hits = Vec::new();
        for lang in ["python", "go"] {
            l.add_evidence(NewEvidence {
                source: Source::Repo {
                    commit: "abc".into(),
                    path: None,
                },
                tool: Tool {
                    name: "t".into(),
                    version: "0".into(),
                },
                authority: Authority::Static,
                kind: EvidenceKind::StaticHit,
                result: json!({ "lang": lang }),
                raw: None,
                coverage: EvidenceCoverage::default(),
                limitations: vec![],
                unknowns: vec![],
            })
            .unwrap();
            hits.push(
                l.evidence()
                    .filter(|e| e.kind == EvidenceKind::StaticHit)
                    .last()
                    .unwrap()
                    .evidence_id
                    .clone(),
            );
        }
        let mut n = 0;
        for (count, hit, ext) in [(py, &hits[0], "py"), (go, &hits[1], "go")] {
            for _ in 0..count {
                n += 1;
                let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![hit.clone()]);
                nf.site = Some(format!("src/f{n}.{ext}:{n}"));
                l.add_finding(nf).unwrap();
            }
        }
    }
}

/// A symptom run (no probe or sweep rules) with its intake answered.
fn init(extra: &[&str], env: &[(&str, &str)]) -> Run {
    let dir = tempfile::tempdir().unwrap();
    let mut c = base(dir.path());
    c.env("DLA_CONSENT_TIERS", "1,2");
    for (k, v) in env {
        c.env(k, v);
    }
    let o = c
        .args(["run", "init", "--session-id", "t", "--entry", "symptom"])
        .args(extra)
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&o.stdout).unwrap_or(Value::Null);
    assert_eq!(v["ok"], true, "{v} {}", String::from_utf8_lossy(&o.stderr));
    let r = Run {
        run_id: v["run_id"].as_str().unwrap().to_owned(),
        dir,
    };
    r.answer_intake();
    r
}

fn secs(a: DateTime<Utc>, b: DateTime<Utc>) -> i64 {
    (a - b).num_seconds()
}

// ------------------------------------------------------------------ init

#[test]
fn init_defaults_to_an_auto_budget_and_a_fixed_value_stays_fixed() {
    let r = init(&[], &[]);
    let b = r.state().budget;
    assert!(b.auto && !b.settled && b.estimate.is_none());
    assert_eq!(b.seconds, 3600);
    assert_eq!(secs(b.deadline, b.started_at), 3600);

    // Any case, and the flag.
    let r = init(&["--budget", "AUTO"], &[]);
    assert!(r.state().budget.auto);
    // The environment default.
    let r = init(&[], &[("DLA_DEFAULT_BUDGET", "45m")]);
    let b = r.state().budget;
    assert!(!b.auto && b.seconds == 2700, "{b:?}");
    let r = init(&["--budget", "90s"], &[("DLA_DEFAULT_BUDGET", "auto")]);
    let b = r.state().budget;
    assert!(!b.auto && b.seconds == 90, "{b:?}");
    let r = init(&["--budget", "2h"], &[]);
    assert_eq!(r.state().budget.seconds, 7200);
    assert!(!r.state().budget.auto);
}

#[test]
fn init_refuses_a_bad_budget_and_names_auto() {
    let dir = tempfile::tempdir().unwrap();
    let o = base(dir.path())
        .args(["run", "init", "--budget", "soon"])
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(2));
    let v: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(
        v["error"]["missing"].as_str().unwrap().contains("auto"),
        "{v}"
    );
}

// ------------------------------------------------------------------ settle

#[test]
fn the_first_passing_gate_at_sweep_settles_an_auto_budget_once() {
    let r = init(&[], &[]);
    r.findings(3, 2);
    let start = r.state().budget.started_at;
    // intake and probe do not settle.
    let (c, v) = r.gate("intake");
    assert_eq!((c, v.get("budget")), (0, None), "{v}");
    let (c, v) = r.gate("probe");
    assert_eq!((c, v.get("budget")), (0, None), "{v}");
    assert!(!r.state().budget.settled);

    // sweep settles: 20 + 15 x 2 + 3 x 5 = 65 minutes.
    let (c, v) = r.gate("sweep");
    assert_eq!(c, 0, "{v}");
    assert_eq!(
        v["budget"],
        "budget (auto): 1h5m for 2 runtimes and 5 candidates (20 + 15x2 + 3x5 = 65 min)"
    );
    let st = r.state();
    assert!(st.budget.settled);
    let e = st.budget.estimate.clone().unwrap();
    assert_eq!(
        (e.runtimes, e.candidates, e.minutes, e.total_minutes),
        (2, 5, 65, 65)
    );
    assert_eq!(st.budget.deadline, start + mins(65));
    assert_eq!(st.budget.seconds, 65 * 60);

    // Settled once: more candidates and a later gate change nothing.
    r.findings(40, 0);
    let (c, v) = r.gate("sweep");
    assert_eq!(c, 0, "{v}");
    assert!(v.get("budget").is_none(), "{v}");
    assert_eq!(r.state().budget, st.budget);
}

#[test]
fn a_small_project_keeps_the_floor_and_a_large_one_hits_the_cap() {
    let r = init(&[], &[]);
    r.findings(1, 0);
    let start = r.state().budget.started_at;
    let (c, v) = r.gate("sweep");
    assert_eq!(c, 0, "{v}");
    assert!(
        v["budget"].as_str().unwrap().contains("raised to 1h"),
        "{v}"
    );
    assert_eq!(r.state().budget.deadline, start + mins(60));

    let r = init(&[], &[]);
    r.findings(362, 0);
    let start = r.state().budget.started_at;
    let (c, v) = r.gate("sweep");
    assert_eq!(c, 0, "{v}");
    assert_eq!(
        v["budget"],
        "budget (auto): 3h for 1 runtime and 362 candidates (20 + 15x1 + 3x362 = 1121 min, capped at 3h)"
    );
    assert_eq!(r.state().budget.deadline, start + mins(180));
}

#[test]
fn settling_never_moves_the_deadline_earlier() {
    let r = init(&[], &[]);
    r.findings(2, 0);
    // An extension came first: 60 + 4 hours.
    r.ok(&["run", "extend", "4h"]);
    let before = r.state().budget.deadline;
    let (c, v) = r.gate("sweep");
    assert_eq!(c, 0, "{v}");
    assert_eq!(r.state().budget.deadline, before);
    assert!(r.state().budget.settled);
}

#[test]
fn a_later_stage_settles_when_the_sweep_gate_did_not_run() {
    let r = init(&[], &[]);
    r.findings(3, 2);
    let (c, v) = r.gate("classify");
    assert_eq!(c, 0, "{v}");
    assert_eq!(
        v["budget"],
        "budget (auto): 1h5m for 2 runtimes and 5 candidates (20 + 15x2 + 3x5 = 65 min)"
    );
    assert!(r.state().budget.settled);
}

#[test]
fn the_final_gate_does_not_size_the_budget_from_nothing() {
    let r = init(&[], &[]);
    r.findings(2, 0);
    r.note_time_refusal();
    for f in ["F1", "F2"] {
        r.ok(&[
            "ledger",
            "resolve",
            f,
            "--as",
            "unresolved",
            "--reason",
            "time_budget",
            "--remedy",
            "more time",
        ]);
    }
    let (c, v) = r.gate("final");
    assert_eq!(c, 0, "{v}");
    assert!(v.get("budget").is_none(), "{v}");
    let b = r.state().budget;
    assert!(b.auto && !b.settled && b.estimate.is_none());
}

#[test]
fn an_extension_made_before_sizing_is_kept_and_the_gate_line_says_so() {
    let r = init(&[], &[]);
    r.findings(3, 2);
    r.ok(&["run", "extend", "30m"]);
    let start = r.state().budget.started_at;
    let (c, v) = r.gate("sweep");
    assert_eq!(c, 0, "{v}");
    assert_eq!(
        v["budget"],
        "budget (auto): 1h5m for 2 runtimes and 5 candidates (20 + 15x2 + 3x5 = 65 min); plus 30m already extended"
    );
    assert_eq!(r.state().budget.deadline, start + mins(95));
}

#[test]
fn a_worker_with_dispositioned_candidates_sizes_the_run_from_them() {
    use dla_core::{FindingState, UnresolvedReason};
    let r = init(&[], &[]);
    r.ok(&["run", "dispatch", "--workers", "python,go"]);
    let start = r.state().budget.started_at;
    // The worker's census and sweep add six candidates to its own slice (a dispatched
    // run's core writes none before the workers start).
    let ids = {
        let mut l = Ledger::load_for(&r.run_dir(), Some("python")).unwrap();
        l.add_evidence(NewEvidence {
            source: Source::Repo {
                commit: "abc".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "0".into(),
            },
            authority: Authority::Static,
            kind: EvidenceKind::StaticHit,
            result: json!({ "lang": "python" }),
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        })
        .unwrap();
        let hit = l.evidence().last().unwrap().evidence_id.clone();
        let mut ids = Vec::new();
        for n in 1..=6 {
            let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![hit.clone()]);
            nf.site = Some(format!("src/f{n}.py:{n}"));
            l.add_finding(nf).unwrap();
        }
        for f in l.findings() {
            ids.push(f.id.clone());
        }
        ids
    };
    assert_eq!(ids.len(), 6);
    // While they are open the worker's gate does not pass, so nothing is sized.
    let (c, v, err) = r.out(&["--worker", "python", "gate", "check", "--stage", "sweep"]);
    assert_eq!(c, 2, "{v} {err}");
    assert!(!r.state().budget.settled);
    {
        let mut l = Ledger::load_for(&r.run_dir(), Some("python")).unwrap();
        for id in &ids {
            l.transition(
                id,
                FindingState::Unresolved,
                Some(UnresolvedReason::NotExercised),
                Some("x".into()),
                vec![],
                "t",
            )
            .unwrap();
        }
    }
    let (c, v, err) = r.out(&["--worker", "python", "gate", "check", "--stage", "sweep"]);
    assert_eq!(c, 0, "{v} {err}");
    // 6 candidates and the run's 2 workers: 20 + 15x2 + 3x6 = 68 min, not the floor.
    assert_eq!(
        v["budget"],
        "budget (auto): 1h8m for 2 runtimes and 6 candidates (20 + 15x2 + 3x6 = 68 min)",
        "{v}"
    );
    assert_eq!(r.state().budget.deadline, start + mins(68));
}

/// Adds `n` census candidates (one site each) to `worker`'s slice, behind one static hit
/// of `lang`; returns their ids.
fn slice_candidates(r: &Run, worker: &str, lang: &str, n: usize) -> Vec<dla_core::FindingId> {
    let mut l = Ledger::load_for(&r.run_dir(), Some(worker)).unwrap();
    l.add_evidence(NewEvidence {
        source: Source::Repo {
            commit: "abc".into(),
            path: None,
        },
        tool: Tool {
            name: "t".into(),
            version: "0".into(),
        },
        authority: Authority::Static,
        kind: EvidenceKind::StaticHit,
        result: json!({ "lang": lang }),
        raw: None,
        coverage: EvidenceCoverage::default(),
        limitations: vec![],
        unknowns: vec![],
    })
    .unwrap();
    let hit = l.evidence().last().unwrap().evidence_id.clone();
    for i in 1..=n {
        let mut nf = NewFinding::new(Origin::parse("census").unwrap(), vec![hit.clone()]);
        nf.site = Some(format!("src/{worker}{i}.{lang}:{i}"));
        l.add_finding(nf).unwrap();
    }
    l.findings().map(|f| f.id.clone()).collect()
}

#[test]
fn the_worker_that_passes_first_sizes_the_run_from_every_workers_slice() {
    use dla_core::{FindingState, UnresolvedReason};
    let r = init(&[], &[]);
    r.ok(&["run", "dispatch", "--workers", "python,go"]);
    let start = r.state().budget.started_at;
    // Python holds 40 candidates, go none: go's gate passes first (nothing of its own is
    // open) and must not size the run from its empty slice.
    let ids = slice_candidates(&r, "python", "py", 40);
    assert_eq!(ids.len(), 40);
    let (c, v, err) = r.out(&["--worker", "go", "gate", "check", "--stage", "sweep"]);
    assert_eq!(c, 0, "{v} {err}");
    // 20 + 15x2 + 3x40 = 170 minutes.
    assert_eq!(
        v["budget"],
        "budget (auto): 2h50m for 2 runtimes and 40 candidates (20 + 15x2 + 3x40 = 170 min)",
        "{v}"
    );
    let st = r.state();
    assert_eq!(st.budget.deadline, start + mins(170));
    assert_eq!(st.budget.settled_workers, ["go"]);
    assert!(!st.budget.settled, "python has not sized it yet");
    // Python dispositions its candidates and passes: the budget is final and still counts
    // the 40.
    {
        let mut l = Ledger::load_for(&r.run_dir(), Some("python")).unwrap();
        for id in &ids {
            l.transition(
                id,
                FindingState::Unresolved,
                Some(UnresolvedReason::NotExercised),
                Some("x".into()),
                vec![],
                "t",
            )
            .unwrap();
        }
    }
    let (c, v, err) = r.out(&["--worker", "python", "gate", "check", "--stage", "sweep"]);
    assert_eq!(c, 0, "{v} {err}");
    let st = r.state();
    assert!(st.budget.settled);
    assert_eq!(st.budget.settled_workers, ["go", "python"]);
    assert_eq!(st.budget.estimate.as_ref().unwrap().candidates, 40);
    assert_eq!(st.budget.deadline, start + mins(170));
    // Final: more candidates later change nothing.
    slice_candidates(&r, "go", "go", 30);
    r.out(&["--worker", "go", "gate", "check", "--stage", "sweep"]);
    assert_eq!(r.state().budget.deadline, start + mins(170));
}

#[test]
fn a_fixed_budget_never_settles() {
    let r = init(&["--budget", "60m"], &[]);
    r.findings(100, 0);
    let before = r.state().budget;
    let (c, v) = r.gate("sweep");
    assert_eq!(c, 0, "{v}");
    assert!(v.get("budget").is_none(), "{v}");
    assert_eq!(r.state().budget, before);
    assert!(!r.state().budget.settled);
}

#[test]
fn settling_moves_the_deadline_of_every_unfinished_worker() {
    let r = init(&[], &[]);
    r.findings(10, 0);
    let at = now_utc() + mins(30);
    r.update(|s| {
        for (n, st) in [
            ("python", WorkerStatus::Running),
            ("go", WorkerStatus::Done),
        ] {
            s.workers.push(dla_core::state::WorkerEntry {
                name: n.into(),
                runtime: n.into(),
                share_secs: 1,
                deadline: at,
                started_at: None,
                status: st,
                slice: format!("ledger.{n}.jsonl"),
                stalled: false,
            });
        }
    });
    let (c, v) = r.gate("sweep");
    assert_eq!(c, 0, "{v}");
    // 20 + 15 + 30 = 65: five minutes more than the hour it started with.
    let st = r.state();
    let by = |n: &str| st.workers.iter().find(|w| w.name == n).unwrap().deadline;
    assert_eq!(by("python"), at + mins(5));
    assert_eq!(by("go"), at, "a finished worker keeps its deadline");
}

#[test]
fn a_workers_passing_gate_settles_too() {
    let r = init(&[], &[]);
    r.findings(30, 0);
    r.ok(&["run", "dispatch", "--workers", "python,go"]);
    let start = r.state().budget.started_at;
    let (c, v, err) = r.out(&["--worker", "python", "gate", "check", "--stage", "sweep"]);
    assert_eq!(c, 0, "{v} {err}");
    assert!(
        v["budget"].as_str().unwrap().starts_with("budget (auto): "),
        "{v}"
    );
    let st = r.state();
    assert_eq!(st.budget.settled_workers, ["python"]);
    assert!(!st.budget.settled, "go has not sized it yet");
    // 30 candidates of one runtime, but two workers in the run: 20 + 15x2 + 90 = 140
    // minutes, and the workers' deadlines moved with it.
    assert_eq!(st.budget.deadline, start + mins(140));
    assert!(
        st.workers.iter().all(|w| w.deadline > start + mins(60)),
        "{:?}",
        st.workers
    );
}

// ------------------------------------------------------------------ extend

#[test]
fn extend_adds_time_records_static_evidence_and_says_where_the_deadline_is() {
    let r = init(&["--budget", "40m"], &[]);
    let before = r.state().budget.deadline;
    let v = r.ok(&["run", "extend", "30m"]);
    let st = r.state();
    assert_eq!(st.budget.deadline, before + mins(30));
    assert_eq!(st.budget.seconds, 70 * 60);
    assert_eq!(st.budget.extensions.len(), 1);
    assert_eq!(v["extended_by_seconds"], 1800);
    assert_eq!(
        v["deadline"],
        json!(
            st.budget
                .deadline
                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        )
    );
    let left = v["minutes_left"].as_u64().unwrap();
    assert!((68..=70).contains(&left), "{v}");
    // The evidence is a static run-config record, so it is not a measurement.
    let l = Ledger::load(&r.run_dir()).unwrap();
    let e = l
        .evidence()
        .find(|e| e.result.get("extend_budget_seconds").is_some())
        .unwrap();
    assert_eq!(e.kind, EvidenceKind::RunConfig);
    assert_eq!(e.authority, Authority::Static);
    assert_eq!(e.evidence_id.to_string(), v["evidence"].as_str().unwrap());
    assert!(!dla_core::measured(&dla_core::RunView::load(&r.run_dir()).unwrap()).any());
}

#[test]
fn extend_from_a_spent_budget_counts_from_now_and_moves_workers() {
    let r = init(&["--budget", "40m"], &[]);
    let now = now_utc();
    r.update(|s| {
        s.budget.deadline = now - mins(10);
        s.workers.push(dla_core::state::WorkerEntry {
            name: "jvm".into(),
            runtime: "jvm".into(),
            share_secs: 1,
            deadline: now - mins(20),
            started_at: None,
            status: WorkerStatus::Running,
            slice: "ledger.jvm.jsonl".into(),
            stalled: false,
        });
    });
    r.ok(&["run", "extend", "1h"]);
    let st = r.state();
    let d = secs(st.budget.deadline, now);
    assert!((3590..=3610).contains(&d), "{d}");
    let w = secs(st.workers[0].deadline, now);
    assert!((3590..=3610).contains(&w), "{w}");
}

#[test]
fn extend_refuses_auto_a_bad_duration_a_worker_and_an_inactive_run() {
    let r = init(&[], &[]);
    for bad in ["auto", "soon", "0m", "1.5h"] {
        let (c, v, _) = r.out(&["run", "extend", bad]);
        assert_eq!(c, 2, "{bad}: {v}");
        assert!(
            v["error"]["fix"]
                .as_str()
                .unwrap()
                .contains("dla run extend"),
            "{v}"
        );
        // The refusal never offers `auto` in the same breath as rejecting it.
        let missing = v["error"]["missing"].as_str().unwrap();
        assert!(
            missing.contains("auto is not accepted") && !missing.starts_with("auto,"),
            "{v}"
        );
    }
    // A bad duration is refused in the words of an extension, not of `run init --budget`.
    for (bad, why) in [
        ("0s", "must be greater than zero"),
        ("5d", "the unit must be s, m or h"),
        ("soon", "the unit must be s, m or h"),
        ("m", "expected digits before the unit"),
    ] {
        let (c, v, _) = r.out(&["run", "extend", bad]);
        assert_eq!(c, 2, "{bad}: {v}");
        let msg = v["error"]["message"].as_str().unwrap();
        assert!(
            msg.starts_with("invalid extension") && msg.contains(why) && !msg.contains("budget"),
            "{bad}: {v}"
        );
    }
    let (c, v, _) = r.out(&["--worker", "jvm", "run", "extend", "auto"]);
    assert_eq!(c, 2, "{v}");
    let (c, v, _) = r.out(&["run", "extend", "AUTO"]);
    assert_eq!(c, 2, "{v}");
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("an extension is a fixed duration"),
        "{v}"
    );
    let (c, v, _) = r.out(&["--worker", "jvm", "run", "extend", "30m"]);
    assert_eq!(c, 2, "{v}");
    assert_eq!(r.state().budget.extensions.len(), 0);
    r.update(|s| s.status = RunStatus::Aborted);
    let (c, v, _) = r.out(&["run", "extend", "30m"]);
    assert_eq!(c, 2, "{v}");
    assert_eq!(v["error"]["code"], "run-not-active");
    assert_eq!(r.state().budget.extensions.len(), 0);
}

// ------------------------------------------------------------------ status

#[test]
fn status_has_a_budget_object() {
    let r = init(&[], &[]);
    r.findings(2, 1);
    let v = r.ok(&["run", "status"]);
    let b = &v["budget"];
    assert_eq!(b["auto"], true);
    assert_eq!(b["settled"], false);
    assert_eq!(b["estimate"], Value::Null);
    assert_eq!(b["seconds"], 3600);
    assert_eq!(b["extensions"], json!([]));
    let left = b["remaining_seconds"].as_u64().unwrap();
    assert!((3000..=3600).contains(&left), "{b}");
    // 3 open findings x 3 + 2 runtimes without a harness run x 15.
    assert_eq!(b["to_finish_minutes"], 39);

    r.gate("sweep");
    r.ok(&["run", "extend", "30m"]);
    let v = r.ok(&["run", "status"]);
    let b = &v["budget"];
    assert_eq!(b["settled"], true);
    assert_eq!(b["estimate"]["runtimes"], 2);
    assert_eq!(b["estimate"]["candidates"], 3);
    assert_eq!(b["estimate"]["total_minutes"], 60);
    assert_eq!(b["extensions"].as_array().unwrap().len(), 1);
    assert_eq!(b["extensions"][0]["seconds"], 1800);
    assert_eq!(b["seconds"], 90 * 60);
}

#[test]
fn status_of_a_fixed_budget_says_so() {
    let r = init(&["--budget", "45m"], &[]);
    let v = r.ok(&["run", "status"]);
    assert_eq!(v["budget"]["auto"], false);
    assert_eq!(v["budget"]["settled"], false);
    assert_eq!(v["budget"]["seconds"], 2700);
    assert_eq!(v["budget"]["to_finish_minutes"], 0);
}

// ------------------------------------------------------------------ hints

#[test]
fn a_spent_budget_points_at_extend_in_an_interactive_run() {
    let r = init(&["--budget", "40m"], &[]);
    // A run whose consent the user gave (`dla consent`, as at stage 1) is interactive.
    r.update(|s| {
        s.consent.as_mut().unwrap().given_by = dla_core::state::ConsentGiver::User;
        s.budget.deadline = now_utc() - mins(1);
    });
    let (c, v, _) = r.out(&["run", "dispatch", "--workers", "jvm"]);
    assert_eq!(c, 2, "{v}");
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(fix.contains("dla run extend"), "{v}");
    assert!(!fix.contains("fixed at init"), "{v}");
}

#[test]
fn an_unattended_run_is_never_told_to_extend() {
    // `init` here gives the run consent from the runner (headless): nobody can be asked.
    let r = init(&["--budget", "40m"], &[]);
    assert_eq!(
        r.state().consent.unwrap().given_by,
        dla_core::state::ConsentGiver::Runner
    );
    r.update(|s| s.budget.deadline = now_utc() - mins(1));
    let (c, v, _) = r.out(&["run", "dispatch", "--workers", "jvm"]);
    assert_eq!(c, 2, "{v}");
    let fix = v["error"]["fix"].as_str().unwrap();
    assert!(!fix.contains("dla run extend"), "{v}");
    assert!(fix.contains("--reason time_budget"), "{v}");
}
