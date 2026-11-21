//! The measurement queue: which ranked candidates still need a workload that reaches them,
//! which are settled by what a harness showed, and when the queue is open.
//!
//! The ranking is [`crate::rank`]. A candidate is eligible for the queue when it is a
//! leak-side finding in state `candidate` (not a sibling, not a duplicate) of tier 1 to 4.
//! It is settled by what the stored trend reports say about it (see [`Standing`]) and
//! pending otherwise. Everything here is a pure function of a [`RunView`] loaded from disk;
//! a view without a run directory has no queue.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
    finding::{Finding, FindingState, Polarity},
    gate,
    harness::{HarnessManifest, MAX_TARGETS},
    ids::FindingId,
    rank,
    trend::Verdict,
    view::RunView,
};

/// The most candidates one batch (`dla harness gen --next`) declares.
pub const MAX_BATCH: usize = MAX_TARGETS;

/// States a ranked finding can be in: the queue's universe. A closed finding keeps its
/// rank so that the report can say how it was ranked.
fn in_universe(f: &Finding) -> bool {
    matches!(
        f.state,
        FindingState::Candidate | FindingState::Unresolved | FindingState::SuspectedStaticOnly
    )
}

/// One finding of the universe with its tier and rank.
#[derive(Debug, Clone)]
pub struct Ranked {
    /// The finding.
    pub id: FindingId,
    /// Its state.
    pub state: FindingState,
    /// Tier 1 to 6.
    pub tier: u8,
    /// 1-based position among the eligible (tier 1 to 4) findings of the universe, in
    /// rank order; `None` for tier 5 and 6.
    pub rank: Option<usize>,
    /// The runtime the finding belongs to, when known.
    pub runtime: Option<String>,
}

impl Ranked {
    /// True when the tier is one the queue takes.
    pub fn eligible(&self) -> bool {
        self.rank.is_some()
    }
}

/// Every leak-side, non-sibling, non-duplicate finding in state `candidate`, `unresolved`
/// or `suspected_static_only`, in rank order: tier, then findings with more than one origin
/// first, then ledger order. The eligible ones carry their rank.
pub fn ranked(view: &RunView) -> Vec<Ranked> {
    let mut only: Option<Option<String>> = None;
    let mut list: Vec<(u8, u8, &Finding)> = view
        .ledger
        .findings()
        .filter(|f| {
            in_universe(f)
                && !f.id.is_sibling()
                && f.polarity() == Polarity::Leak
                && view.ledger.merged_into(&f.id).is_none()
        })
        .map(|f| {
            (
                rank::tier(f, view.ledger.annotation(&f.id)),
                rank::multi_origin_key(f),
                f,
            )
        })
        .collect();
    // Stable: ledger order is the tiebreak.
    list.sort_by_key(|(t, m, _)| (*t, *m));
    let mut n = 0;
    list.into_iter()
        .map(|(tier, _, f)| {
            let rank = rank::eligible_tier(tier).then(|| {
                n += 1;
                n
            });
            Ranked {
                id: f.id.clone(),
                state: f.state,
                tier,
                rank,
                runtime: gate::finding_runtime_with(view, f, &mut only),
            }
        })
        .collect()
}

/// One declared run of a finding: a series of a harness that declared it a target.
#[derive(Debug, Clone)]
struct DeclaredRun {
    harness: String,
    hook_driver: bool,
    /// The driver digest of the run, or the harness id when none was recorded.
    driver_key: String,
    verdict: Option<Verdict>,
}

/// What the stored trend reports and harness records say, indexed by finding.
#[derive(Debug, Default)]
pub struct Index {
    /// Every verdict stored for the finding, with the harness of its series.
    verdicts: BTreeMap<FindingId, Vec<(String, Verdict)>>,
    /// The series of harnesses that declared the finding.
    declared: BTreeMap<FindingId, Vec<DeclaredRun>>,
    /// Harnesses that declared the finding, series or not, oldest first.
    declared_by: BTreeMap<FindingId, Vec<String>>,
    /// Harnesses that have at least one series.
    run: std::collections::BTreeSet<String>,
    /// The runtime of each harness.
    runtime: BTreeMap<String, String>,
}

impl Index {
    /// Reads the harness manifests and the trend reports of `view`.
    pub fn of(view: &RunView) -> Self {
        let mut ix = Self::default();
        let Some(run_dir) = view.run_dir.as_deref() else {
            return ix;
        };
        let mut manifests: BTreeMap<String, HarnessManifest> = BTreeMap::new();
        for (_, h) in gate::harness_ids(run_dir) {
            let Ok(m) = HarnessManifest::load(run_dir, &h) else {
                continue;
            };
            if HarnessManifest::dir_in(run_dir, &h)
                .join(crate::harness::CLONE_MARKER)
                .is_file()
            {
                continue;
            }
            ix.runtime.insert(h.clone(), m.runtime.as_str().to_owned());
            for t in &m.targets {
                ix.declared_by.entry(t.clone()).or_default().push(h.clone());
            }
            manifests.insert(h, m);
        }
        // The latest trend report of each series.
        let mut latest: BTreeMap<&str, &crate::trend::TrendReport> = BTreeMap::new();
        for t in &view.trends {
            latest.insert(t.series.as_str(), t);
        }
        for (series, h) in &view.series_harness {
            let Some(m) = manifests.get(h) else { continue };
            ix.run.insert(h.clone());
            let report = latest.get(series.as_str());
            let key = crate::harness::run_record_of(run_dir, h, series)
                .and_then(|r| r.driver_digest)
                .unwrap_or_else(|| h.clone());
            for t in report.map(|r| r.targets.as_slice()).unwrap_or_default() {
                ix.verdicts
                    .entry(t.finding.clone())
                    .or_default()
                    .push((h.clone(), t.verdict));
            }
            for f in &m.targets {
                let verdict = report
                    .and_then(|r| r.targets.iter().find(|t| &t.finding == f))
                    .map(|t| t.verdict);
                ix.declared.entry(f.clone()).or_default().push(DeclaredRun {
                    harness: h.clone(),
                    hook_driver: m.is_hook_driver(),
                    driver_key: key.clone(),
                    verdict,
                });
            }
        }
        ix
    }

    /// True when the harness has a series.
    pub fn harness_ran(&self, h: &str) -> bool {
        self.run.contains(h)
    }

    /// What the run's stored reports make of `id`.
    pub fn standing(&self, id: &FindingId) -> Standing {
        let verdicts = self.verdicts.get(id).map_or(&[][..], Vec::as_slice);
        if verdicts
            .iter()
            .any(|(_, v)| matches!(v, Verdict::Grew | Verdict::Changed))
        {
            return Standing::Settled(Why::Moved);
        }
        let runs = self.declared.get(id).map_or(&[][..], Vec::as_slice);
        if runs.iter().any(|r| {
            r.hook_driver && matches!(r.verdict, Some(Verdict::Executed | Verdict::Unknown))
        }) {
            return Standing::Settled(Why::Executed);
        }
        let keys: std::collections::BTreeSet<&str> = runs
            .iter()
            .filter(|r| r.hook_driver)
            .map(|r| r.driver_key.as_str())
            .collect();
        if keys.len() >= 2 {
            return Standing::Settled(Why::TwoDrivers);
        }
        Standing::Pending
    }

    /// The harnesses that declared `id` as a target, with the best verdict each showed
    /// (`None`: the harness has not run, or its trend was not read).
    pub fn targeted_in(&self, id: &FindingId) -> Vec<(String, Option<Verdict>)> {
        let mut out: Vec<(String, Option<Verdict>)> = Vec::new();
        for h in self.declared_by.get(id).map_or(&[][..], Vec::as_slice) {
            let best = self
                .declared
                .get(id)
                .into_iter()
                .flatten()
                .filter(|r| &r.harness == h)
                .filter_map(|r| r.verdict)
                .min_by_key(|v| priority(*v));
            if !out.iter().any(|(x, _)| x == h) {
                out.push((h.clone(), best));
            }
        }
        out
    }

    /// The best verdict of `id` over every harness that declared it.
    pub fn best_verdict(&self, id: &FindingId) -> Option<Verdict> {
        self.targeted_in(id)
            .into_iter()
            .filter_map(|(_, v)| v)
            .chain(
                self.verdicts
                    .get(id)
                    .into_iter()
                    .flatten()
                    .map(|(_, v)| *v),
            )
            .min_by_key(|v| priority(*v))
    }

    /// The first harness of `runtime` with pending targets that has not run, if any.
    fn in_flight(&self, runtime: &str, pending: &[FindingId]) -> Option<String> {
        self.runtime
            .iter()
            .filter(|(h, rt)| rt.as_str() == runtime && !self.run.contains(*h))
            .map(|(h, _)| h)
            .find(|h| {
                pending
                    .iter()
                    .any(|p| self.declared_by.get(p).is_some_and(|v| v.contains(h)))
            })
            .cloned()
    }
}

/// Why a candidate is settled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    /// A verdict is `grew` or `changed`.
    Moved,
    /// A driver run targeted it and saw its code run (or could not tell).
    Executed,
    /// Two driver runs with different drivers targeted it.
    TwoDrivers,
}

/// Settled or pending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// Measured enough.
    Settled(Why),
    /// Needs a workload that reaches it.
    Pending,
}

/// How good a verdict is for the reader (lower is better).
pub fn priority(v: Verdict) -> u8 {
    match v {
        Verdict::Grew => 0,
        Verdict::Changed => 1,
        Verdict::Executed => 2,
        Verdict::Unknown => 3,
        Verdict::Constant => 4,
        Verdict::NotExecuted => 5,
        Verdict::NotLoaded => 6,
    }
}

/// Why a runtime's queue is not open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Closed {
    /// The run was started with `--queue off`.
    Off,
    /// The budget is at or under the floor.
    Floor,
    /// The workload is recorded as blocked.
    Blocked,
    /// No pending candidate is left.
    Empty,
}

impl Closed {
    /// A short explanation.
    pub fn text(self) -> &'static str {
        match self {
            Self::Off => "the run was started with --queue off",
            Self::Floor => "the budget is at the floor",
            Self::Blocked => "intake.workload records a blocker",
            Self::Empty => "no candidate is waiting for a measurement",
        }
    }
}

/// The queue of one runtime.
#[derive(Debug, Clone)]
pub struct RuntimeQueue {
    /// The runtime (`python`).
    pub runtime: String,
    /// Eligible candidates in state `candidate`.
    pub eligible: usize,
    /// Of those, settled.
    pub settled: usize,
    /// Pending candidates in rank order.
    pub pending: Vec<FindingId>,
    /// `(id, rank, tier)` of the pending candidates, parallel to `pending`.
    pub pending_rank: Vec<(usize, u8)>,
    /// The next batch: the first pending candidates that no unrun harness declares.
    pub batch: Vec<FindingId>,
    /// A generated harness that declares pending candidates and has not run.
    pub in_flight: Option<String>,
    /// The command that makes the next batch.
    pub batch_command: String,
    /// The command to run next: the batch command, or the run of the harness in flight when
    /// every pending candidate is already declared by it.
    pub command: String,
    /// Why the queue is closed; `None` while it is open.
    pub closed: Option<Closed>,
    /// Whole minutes of budget left and in total, when known.
    pub minutes: Option<(u64, u64)>,
}

impl RuntimeQueue {
    /// True while the queue refuses to close pending candidates unmeasured.
    pub fn open(&self) -> bool {
        self.closed.is_none()
    }

    /// The rank and tier of a pending candidate.
    pub fn rank_of(&self, id: &FindingId) -> Option<(usize, u8)> {
        self.pending
            .iter()
            .position(|p| p == id)
            .and_then(|i| self.pending_rank.get(i).copied())
    }
}

/// The command that makes the next batch of `runtime`. A JVM project that has a JUnit repeat
/// harness and no driver with hooks keeps generating JUnit repeats.
pub fn batch_command(view: &RunView, runtime: &str) -> String {
    let junit_only = runtime == "jvm"
        && view.run_dir.as_deref().is_some_and(|d| {
            let all: Vec<_> = gate::harness_ids(d)
                .into_iter()
                .filter_map(|(_, h)| HarnessManifest::load(d, &h).ok())
                .filter(|m| m.runtime.as_str() == runtime)
                .collect();
            !all.is_empty() && all.iter().all(|m| !m.is_hook_driver())
                && all
                    .iter()
                    .any(|m| m.tools.iter().any(|t| t == crate::harness::JUNIT_TOOL))
        });
    if junit_only {
        format!("dla harness gen --lang {runtime} --junit --next")
    } else {
        format!("dla harness gen --lang {runtime} --driver --next")
    }
}

/// The queue of every runtime that has an eligible candidate, in runtime order. `worker`
/// is the worker whose slice `view` is (worker mode): only its runtime is returned and its
/// deadline counts. Empty for a view without a run directory.
pub fn queues(view: &RunView, worker: Option<&str>) -> Vec<RuntimeQueue> {
    if view.run_dir.is_none() {
        return Vec::new();
    }
    let worker_rt = worker
        .and_then(|w| view.state.worker(w))
        .map(|w| w.runtime.clone());
    let all = ranked(view);
    let ix = Index::of(view);
    let mut by_rt: BTreeMap<String, Vec<&Ranked>> = BTreeMap::new();
    for r in &all {
        if !r.eligible() {
            continue;
        }
        let Some(rt) = &r.runtime else { continue };
        if worker_rt.as_ref().is_some_and(|w| w != rt) {
            continue;
        }
        by_rt.entry(rt.clone()).or_default().push(r);
    }
    let budget = gate::budget_over_floor(&view.state, worker);
    let blocked = gate::workload_unresolved(view, true);
    by_rt
        .into_iter()
        .map(|(runtime, list)| {
            // Only a finding still in `candidate` is waiting for a disposition.
            let open_ones: Vec<&&Ranked> = list
                .iter()
                .filter(|r| r.state == FindingState::Candidate)
                .collect();
            let eligible = open_ones.len();
            let mut pending = Vec::new();
            let mut pending_rank = Vec::new();
            for r in &open_ones {
                if ix.standing(&r.id) == Standing::Pending {
                    pending.push(r.id.clone());
                    pending_rank.push((r.rank.unwrap_or(0), r.tier));
                }
            }
            let in_flight = ix.in_flight(&runtime, &pending);
            let batch: Vec<FindingId> = pending
                .iter()
                .filter(|p| {
                    in_flight
                        .as_ref()
                        .is_none_or(|h| !ix.declared_by.get(*p).is_some_and(|v| v.contains(h)))
                })
                .take(MAX_BATCH)
                .cloned()
                .collect();
            let batch_command = batch_command(view, &runtime);
            let command = match (&in_flight, batch.is_empty()) {
                (Some(h), true) => format!(
                    "fill the DLA-HOOK bodies of {h} so each unit reaches the targets listed in its driver with a new key, dla harness run {h} --mode both, dla trend S<n>"
                ),
                _ => batch_command.clone(),
            };
            let closed = if !view.state.queue {
                Some(Closed::Off)
            } else if budget.is_none() {
                Some(Closed::Floor)
            } else if blocked {
                Some(Closed::Blocked)
            } else if pending.is_empty() {
                Some(Closed::Empty)
            } else {
                None
            };
            RuntimeQueue {
                runtime,
                eligible,
                settled: eligible - pending.len(),
                pending,
                pending_rank,
                batch,
                in_flight,
                batch_command,
                command,
                closed,
                minutes: budget,
            }
        })
        .collect()
}

/// The open queue of `runtime`, if there is one.
pub fn open_queue(view: &RunView, runtime: &str, worker: Option<&str>) -> Option<RuntimeQueue> {
    queues(view, worker)
        .into_iter()
        .find(|q| q.runtime == runtime && q.open())
}

/// The full text of the guidance a refusal or an item gives for an open queue.
pub fn steps(q: &RuntimeQueue, run_id: &str) -> String {
    let _ = run_id;
    format!(
        "{}. Fill the hooks so each unit reaches the listed targets with a new key, dla harness run <H> --mode both, then dla trend S<n>. If no driver can be built or run for {}, record why: dla ledger resolve intake.workload --status unresolved --reason <env_unavailable|tool_missing|build_fails|privilege_declined> --remedy \"<text>\" (that closes the queue of the runtime)",
        q.command, q.runtime
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Ledger, NewEvidence, NewFinding, Origin,
        evidence::{Authority, EvidenceCoverage, EvidenceKind, Source, Tool},
        state::{RunState, now_utc},
        trend::{TargetVerdict, TrendReport},
    };
    use serde_json::json;

    /// A run directory with a ledger of findings `(origin, site, lang)`, harnesses and
    /// trend reports written by the caller.
    struct Run {
        dir: tempfile::TempDir,
    }

    fn evidence(lang: &str) -> NewEvidence {
        NewEvidence {
            source: Source::Repo {
                commit: "c".into(),
                path: None,
            },
            tool: Tool {
                name: "t".into(),
                version: "1".into(),
            },
            authority: Authority::Static,
            kind: EvidenceKind::CensusHolder,
            result: json!({"lang": lang}),
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: vec![],
            unknowns: vec![],
        }
    }

    fn manifest(id: &str, targets: &[u32], driver: bool) -> HarnessManifest {
        let text = json!({
            "id": id, "runtime": "python", "mode": if driver {"driver"} else {"command"},
            "scratch": "x", "run": {"argv": ["x"], "cwd": "."},
            "warmup": 1, "iterations": 8, "step": 1, "key_modes": ["fresh"],
            "targets": targets.iter().map(|n| format!("F{n}")).collect::<Vec<_>>(),
            "agent": "driver-python", "timeout_secs": 60,
        });
        serde_json::from_value(text).unwrap()
    }

    impl Run {
        /// Findings 1..=n are census candidates with writers (tier 4) at `pkg/m<i>.py`.
        fn new(n: u32) -> (Self, RunView) {
            let dir = tempfile::tempdir().unwrap();
            let st = RunState::init("/p", None, 3600, None, now_utc()).unwrap();
            st.save(dir.path()).unwrap();
            let mut l = Ledger::create(dir.path()).unwrap();
            l.add_evidence(evidence("python")).unwrap();
            for i in 1..=n {
                let mut nf = NewFinding::new(
                    Origin::parse("census").unwrap(),
                    vec![crate::EvidenceId::new(1)],
                );
                nf.site = Some(format!("pkg/m{i}.py:3"));
                l.add_finding(nf).unwrap();
                l.annotate(
                    &FindingId::new(i),
                    vec![format!("pkg/m{i}.py:9")],
                    vec![],
                    Some("n".into()),
                )
                .unwrap();
            }
            let view = RunView::load(dir.path()).unwrap();
            (Self { dir }, view)
        }

        fn harness(&self, m: &HarnessManifest) {
            m.save(self.dir.path()).unwrap();
        }

        fn series(&self, s: &str, h: &str) {
            let text = json!({
                "id": s, "source": {"kind": "harness", "harness": h},
                "axis": "n", "columns": [], "points": []
            });
            let p = self.dir.path().join("series");
            std::fs::create_dir_all(&p).unwrap();
            std::fs::write(p.join(format!("{s}.json")), text.to_string()).unwrap();
        }

        fn trend(&self, t: &str, series: &str, verdicts: &[(u32, Verdict)]) {
            let r = TrendReport {
                schema_version: 1,
                id: t.into(),
                series: series.into(),
                thresholds: dla_stats::GateThresholds::default(),
                columns: vec![],
                targets: verdicts
                    .iter()
                    .map(|(n, v)| TargetVerdict {
                        finding: FindingId::new(*n),
                        verdict: *v,
                        column: None,
                        detail: String::new(),
                        declared: true,
                    })
                    .collect(),
            };
            r.save(self.dir.path()).unwrap();
        }

        fn reload(&self) -> RunView {
            RunView::load(self.dir.path()).unwrap()
        }
    }

    fn pending_of(view: &RunView) -> Vec<String> {
        queues(view, None)
            .into_iter()
            .flat_map(|q| q.pending)
            .map(|f| f.to_string())
            .collect()
    }

    #[test]
    fn candidates_are_pending_until_a_verdict_settles_them() {
        let (run, view) = Run::new(3);
        assert_eq!(pending_of(&view), ["F1", "F2", "F3"]);
        let q = &queues(&view, None)[0];
        assert!(q.open());
        assert_eq!(q.batch_command, "dla harness gen --lang python --driver --next");
        // F1 grew in a trend of an automatic probe: settled by growth alone.
        run.harness(&manifest("H1", &[2], true));
        run.series("S1", "H1");
        run.trend("T1", "S1", &[(1, Verdict::Grew), (2, Verdict::Constant)]);
        assert_eq!(pending_of(&run.reload()), ["F2", "F3"]);
    }

    #[test]
    fn a_changed_verdict_settles() {
        let (run, _) = Run::new(2);
        run.harness(&manifest("H1", &[1], true));
        run.series("S1", "H1");
        run.trend("T1", "S1", &[(1, Verdict::Changed)]);
        assert_eq!(pending_of(&run.reload()), ["F2"]);
    }

    #[test]
    fn executed_and_unknown_settle_only_a_declared_driver_target() {
        let (run, _) = Run::new(3);
        run.harness(&manifest("H1", &[1, 2], true));
        run.series("S1", "H1");
        run.trend(
            "T1",
            "S1",
            &[(1, Verdict::Executed), (2, Verdict::Unknown), (3, Verdict::Executed)],
        );
        // F3 was not declared by the harness: its verdict does not settle it.
        assert_eq!(pending_of(&run.reload()), ["F3"]);
    }

    #[test]
    fn a_fixed_command_settles_only_by_growth() {
        let (run, _) = Run::new(2);
        run.harness(&manifest("H1", &[1, 2], false));
        run.series("S1", "H1");
        run.trend("T1", "S1", &[(1, Verdict::Executed), (2, Verdict::Unknown)]);
        assert_eq!(pending_of(&run.reload()), ["F1", "F2"]);
        run.trend("T2", "S1", &[(1, Verdict::Grew), (2, Verdict::Executed)]);
        assert_eq!(pending_of(&run.reload()), ["F2"]);
    }

    #[test]
    fn a_junit_repeat_is_a_fixed_command() {
        let (run, _) = Run::new(1);
        let mut m = manifest("H1", &[1], true);
        m.tools.push(crate::harness::JUNIT_TOOL.to_owned());
        run.harness(&m);
        run.series("S1", "H1");
        run.trend("T1", "S1", &[(1, Verdict::Executed)]);
        assert_eq!(pending_of(&run.reload()), ["F1"]);
    }

    #[test]
    fn two_runs_of_one_driver_do_not_settle_two_different_drivers_do() {
        let (run, _) = Run::new(1);
        run.harness(&manifest("H1", &[1], true));
        let driver = |h: &str, text: &str| {
            let d = run.dir.path().join("harness").join(h).join("driver");
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("dla_harness.py"), text).unwrap();
        };
        let record = |h: &str, first: &str, series: &[&str]| {
            let digest = crate::harness::driver_digest(run.dir.path(), h).unwrap();
            let text = json!({"series": series, "driver_digest": digest});
            std::fs::write(
                run.dir.path().join("harness").join(h).join(format!("run-{first}.json")),
                text.to_string(),
            )
            .unwrap();
        };
        driver("H1", "a");
        run.series("S1", "H1");
        record("H1", "S1", &["S1"]);
        run.trend("T1", "S1", &[(1, Verdict::Constant)]);
        // The same driver run again.
        run.series("S2", "H1");
        record("H1", "S2", &["S2"]);
        run.trend("T2", "S2", &[(1, Verdict::NotExecuted)]);
        assert_eq!(pending_of(&run.reload()), ["F1"]);
        // The driver changed, the harness ran again: two different drivers, whatever the verdicts.
        driver("H1", "b");
        run.series("S3", "H1");
        record("H1", "S3", &["S3"]);
        run.trend("T3", "S3", &[(1, Verdict::Constant)]);
        assert!(pending_of(&run.reload()).is_empty());
    }

    #[test]
    fn the_queue_is_closed_by_off_a_blocker_and_when_empty() {
        let (run, view) = Run::new(1);
        assert!(queues(&view, None)[0].open());
        let mut off = view.clone();
        off.state.queue = false;
        assert_eq!(queues(&off, None)[0].closed, Some(Closed::Off));
        run.harness(&manifest("H1", &[1], true));
        run.series("S1", "H1");
        run.trend("T1", "S1", &[(1, Verdict::Grew)]);
        assert_eq!(queues(&run.reload(), None)[0].closed, Some(Closed::Empty));
    }

    #[test]
    fn a_batch_skips_candidates_a_harness_that_has_not_run_already_declares() {
        let (run, _) = Run::new(3);
        run.harness(&manifest("H1", &[1, 2], true));
        let q = queues(&run.reload(), None).remove(0);
        assert_eq!(q.in_flight.as_deref(), Some("H1"));
        assert_eq!(q.batch, [FindingId::new(3)]);
        assert_eq!(q.pending.len(), 3);
    }

    #[test]
    fn ranking_orders_by_tier_then_multiple_origins_then_ledger_order() {
        let dir = tempfile::tempdir().unwrap();
        let st = RunState::init("/p", None, 3600, None, now_utc()).unwrap();
        st.save(dir.path()).unwrap();
        let mut l = Ledger::create(dir.path()).unwrap();
        l.add_evidence(evidence("python")).unwrap();
        for (origin, site) in [
            ("growth-series", "pkg/a.py:1"),
            ("detector:semgrep/x", "pkg/b.py:1"),
            ("detector:dla/python-unbounded-cache", "pkg/c.py:1"),
            ("detector:semgrep/y", "tests/d.py:1"),
            ("detector:semgrep/z", "pkg/e.py:1"),
        ] {
            let mut nf = NewFinding::new(Origin::parse(origin).unwrap(), vec![crate::EvidenceId::new(1)]);
            nf.site = Some(site.into());
            l.add_finding(nf).unwrap();
        }
        let view = RunView::load(dir.path()).unwrap();
        let r = ranked(&view);
        let order: Vec<(String, u8, Option<usize>)> =
            r.iter().map(|x| (x.id.to_string(), x.tier, x.rank)).collect();
        assert_eq!(
            order,
            [
                ("F3".into(), 1, Some(1)),
                ("F2".into(), 3, Some(2)),
                ("F5".into(), 3, Some(3)),
                ("F1".into(), 5, None),
                ("F4".into(), 6, None),
            ]
        );
    }
}
