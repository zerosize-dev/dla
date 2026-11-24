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

/// The runtime name of Go queues, whose batches are one package each.
const GO: &str = "go";

/// The most candidates one batch (`dla harness gen --next`) takes by rank.
pub const MAX_BATCH: usize = MAX_TARGETS;
/// The most candidates one batch declares once the siblings of its candidates are added.
pub const MAX_BATCH_WITH_SIBLINGS: usize = crate::harness::MAX_TARGETS_WITH_SIBLINGS;

/// The next batch from the pending candidates in rank order: the first [`MAX_BATCH`], then,
/// in rank order, every other one that is state of the same owner as a candidate already in
/// it (fields of one struct or class; module-level holders that share a writer function), up
/// to [`MAX_BATCH_WITH_SIBLINGS`] in all. A driver that exercises an owner reaches all of its
/// state, so the holders of one owner are measured together. The map says which candidate
/// each added one came with.
pub fn with_siblings(
    view: &RunView,
    pending: &[&FindingId],
    head: usize,
) -> (Vec<FindingId>, BTreeMap<FindingId, FindingId>) {
    let head = head.min(MAX_BATCH);
    let mut batch: Vec<FindingId> = pending.iter().take(head).map(|f| (*f).clone()).collect();
    let mut siblings = BTreeMap::new();
    let of = |id: &FindingId| {
        view.ledger
            .finding(id)
            .map(|f| (f, view.ledger.annotation(id)))
    };
    for cand in pending.iter().skip(head) {
        if batch.len() >= MAX_BATCH_WITH_SIBLINGS {
            break;
        }
        let Some(c) = of(cand) else { continue };
        let with = batch
            .iter()
            .find(|b| of(b).is_some_and(|b| rank::same_owner(b, c)));
        if let Some(with) = with {
            siblings.insert((*cand).clone(), with.clone());
            batch.push((*cand).clone());
        }
    }
    (batch, siblings)
}

/// The package directory of a Go site (`health/server.go:49` -> `health`; a file at the
/// project root -> the empty string).
pub fn go_package_dir(site: &str) -> String {
    let file = match site.rsplit_once(':') {
        Some((f, l)) if !l.is_empty() && l.bytes().all(|b| b.is_ascii_digit()) => f,
        _ => site,
    };
    let file = file.replace('\\', "/");
    let file = file.strip_prefix("./").unwrap_or(&file);
    file.rsplit_once('/').map_or("", |(d, _)| d).to_owned()
}

/// How a package directory is named in output (`.` for the project root).
pub fn package_label(dir: &str) -> &str {
    if dir.is_empty() { "." } else { dir }
}

/// The package directory of a finding of a Go queue, from its site.
fn package_of(view: &RunView, id: &FindingId) -> Option<String> {
    view.ledger
        .finding(id)
        .and_then(|f| f.site.as_deref())
        .map(go_package_dir)
}

/// The package directories of `pending` in rank order, each once, by its best-ranked member.
pub fn go_packages(view: &RunView, pending: &[&FindingId]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for f in pending {
        if let Some(p) = package_of(view, f)
            && !out.contains(&p)
        {
            out.push(p);
        }
    }
    out
}

/// The batch of a Go queue: the pending candidates whose site is in the package directory
/// `package`, in rank order, the first `head` (at most [`MAX_BATCH`]) and then the
/// same-owner siblings up to [`MAX_BATCH_WITH_SIBLINGS`] (see [`with_siblings`]). A Go test
/// file reads the unexported names of its own package only, so the candidates of one batch
/// are the ones one driver can reach and probe.
pub fn package_batch(
    view: &RunView,
    pending: &[&FindingId],
    head: usize,
    package: &str,
) -> (Vec<FindingId>, BTreeMap<FindingId, FindingId>) {
    let inside: Vec<&FindingId> = pending
        .iter()
        .filter(|f| package_of(view, f).is_some_and(|p| p == package))
        .copied()
        .collect();
    with_siblings(view, &inside, head)
}

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
    /// The score the order inside the eligible set follows ([`rank::score`]).
    pub score: i32,
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
/// or `suspected_static_only`, in rank order: descending [`rank::score`], then tier, then
/// findings with more than one origin first, then ledger order. The eligible ones (tier 1 to
/// 4) come first and carry their rank; the others follow by tier.
pub fn ranked(view: &RunView) -> Vec<Ranked> {
    let mut only: Option<Option<String>> = None;
    let mut list: Vec<(u8, i32, &Finding)> = view
        .ledger
        .findings()
        .filter(|f| {
            in_universe(f)
                && !f.id.is_sibling()
                && f.polarity() == Polarity::Leak
                && view.ledger.merged_into(&f.id).is_none()
        })
        .map(|f| {
            let a = view.ledger.annotation(&f.id);
            (rank::tier(f, a), rank::score(f, a), f)
        })
        .collect();
    // The eligible ones in score order (ties: tier, then more than one origin first), the
    // rest after them by tier. Stable: ledger order is the last tiebreak.
    list.sort_by_key(|(t, s, f)| {
        (
            !rank::eligible_tier(*t),
            if rank::eligible_tier(*t) { -*s } else { 0 },
            *t,
            rank::multi_origin_key(f),
        )
    });
    let mut n = 0;
    list.into_iter()
        .map(|(tier, score, f)| {
            let rank = rank::eligible_tier(tier).then(|| {
                n += 1;
                n
            });
            Ranked {
                id: f.id.clone(),
                state: f.state,
                tier,
                score,
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
    /// A test-loop harness: the project's own JUnit 5 tests in a loop, no hooks.
    test_loop: bool,
    /// The driver digest of the run, or the harness id when none was recorded.
    driver_key: String,
    verdict: Option<Verdict>,
    /// The reason text the trend gave for the verdict.
    detail: Option<String>,
    /// The writer lines and their counts the trend gave for a verdict `writer_not_executed`.
    writers: Vec<crate::trend::WriterCount>,
    /// The harness declares at most [`FOCUS_MAX`] targets.
    focused: bool,
    /// An entry-path harness: it has its own rules, so it is no run of the queue's rule.
    entry: bool,
    /// A second-look harness.
    second_look: bool,
    /// Rust: the features of the target's crate that have gated sites and that this harness
    /// is built without, and the command that generates it again with them on.
    feature_gaps: Vec<crate::features::FeatureGap>,
    feature_command: String,
}

impl DeclaredRun {
    /// One flat reading of a first pass: a hook driver read `executed`; the test loop read
    /// `executed` or `constant`. A fixed command's flat reading says nothing about other call
    /// orders, so it is one reading, and the second look is what follows it.
    fn flat_reading(&self) -> bool {
        (self.hook_driver && self.verdict == Some(Verdict::Executed))
            || (self.test_loop
                && matches!(self.verdict, Some(Verdict::Executed | Verdict::Constant)))
    }
}

/// Where a candidate stands with respect to its second look.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecondLook {
    /// Not a candidate for one: it did not read `executed` in a hook driver, grew, was missed,
    /// or an entry-path harness is all that ran.
    No,
    /// One driver read it `executed` (a flat reading of one arbitrary unit): it waits for a
    /// different driver. Holds the harness of that first reading.
    Awaiting(String),
    /// A focused harness with another driver has run for it (the harness is named).
    Done(String),
}

/// A run that missed a target: where, and what the trend said.
#[derive(Debug, Clone)]
pub struct Miss {
    /// The harness.
    pub harness: String,
    /// The verdict (one of [`is_miss`]).
    pub verdict: Verdict,
    /// The reason text the trend gave.
    pub detail: String,
    /// The writer lines and how often each ran, when the verdict is `writer_not_executed`.
    pub writers: Vec<crate::trend::WriterCount>,
}

/// The most targets of a focus harness (and so of a FOCUSED harness).
pub const FOCUS_MAX: usize = 3;

/// The most second-look harnesses per runtime: one flat run of one driver does not end a run
/// that has budget left, but the second look has to end (3 harnesses of [`FOCUS_MAX`]
/// targets: 9 candidates, one look each).
pub const SECOND_LOOK_MAX: usize = 3;

/// A verdict that says a harness run did not reach the target: it ran, its trend was read and
/// nothing shows the target's code run (or its holder move).
pub fn is_miss(v: Verdict) -> bool {
    matches!(
        v,
        Verdict::Constant
            | Verdict::NotLoaded
            | Verdict::NotExecuted
            | Verdict::WriterNotExecuted
            | Verdict::Unknown
    )
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
    /// The second-look harnesses (any state) of each runtime.
    second_looks: BTreeMap<String, Vec<String>>,
    /// The test-loop harnesses (any state).
    test_loops: std::collections::BTreeSet<String>,
    /// The findings a test-loop harness declared, run or not.
    test_loop_declared: std::collections::BTreeSet<FindingId>,
    /// The test-loop harnesses whose attempt failed (they did not build, or every unit
    /// failed): they are not run again, and what they declared waits for a driver.
    test_loop_failed: std::collections::BTreeSet<String>,
    /// The full pytest-loop harnesses (any state, generated without `--tests`), oldest first. A
    /// narrowed loop is not listed: it does not stand for the first step of the queue.
    pytest_loops: Vec<String>,
    /// True when `dla harness gen --lang python --pytest` was refused in this run (pytest does
    /// not import in the harness interpreter): a failed attempt of the loop, with no harness.
    pytest_refused: bool,
}

impl Index {
    /// Reads the harness manifests and the trend reports of `view`.
    pub fn of(view: &RunView) -> Self {
        let mut ix = Self::default();
        let Some(run_dir) = view.run_dir.as_deref() else {
            return ix;
        };
        ix.pytest_refused = crate::harness::HarnessAttempt::pytest_loop_refused(run_dir);
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
            if m.is_second_look() {
                ix.second_looks
                    .entry(m.runtime.as_str().to_owned())
                    .or_default()
                    .push(h.clone());
            }
            for t in &m.targets {
                ix.declared_by.entry(t.clone()).or_default().push(h.clone());
            }
            if m.is_pytest_loop()
                && !m.is_pytest_narrowed()
                && m.runtime == crate::runtime::Runtime::Python
            {
                ix.pytest_loops.push(h.clone());
            }
            if m.is_test_loop() {
                ix.test_loops.insert(h.clone());
                ix.test_loop_declared.extend(m.targets.iter().cloned());
                if gate::attempt_of(run_dir, &h).is_some_and(|a| {
                    a.outcome == crate::harness::AttemptOutcome::Failed
                        && !gate::attempt_is_no_try(a.reason.as_deref())
                }) {
                    ix.test_loop_failed.insert(h.clone());
                }
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
            let features = crate::features::HarnessFeatures::load(run_dir, h);
            for f in &m.targets {
                let verdict = report
                    .and_then(|r| r.targets.iter().find(|t| &t.finding == f))
                    .map(|t| t.verdict);
                let detail = report
                    .and_then(|r| r.targets.iter().find(|t| &t.finding == f))
                    .map(|t| t.detail.clone());
                let writers = report
                    .and_then(|r| r.targets.iter().find(|t| &t.finding == f))
                    .map(|t| t.writers.clone())
                    .unwrap_or_default();
                ix.declared.entry(f.clone()).or_default().push(DeclaredRun {
                    harness: h.clone(),
                    hook_driver: m.is_hook_driver(),
                    test_loop: m.is_test_loop(),
                    driver_key: key.clone(),
                    verdict,
                    detail,
                    writers,
                    focused: m.targets.len() <= FOCUS_MAX,
                    entry: m.is_entry_path(),
                    second_look: m.is_second_look(),
                    feature_gaps: features
                        .as_ref()
                        .map(|x| x.gaps_of(f).to_vec())
                        .unwrap_or_default(),
                    feature_command: features
                        .as_ref()
                        .map(|x| x.command.clone())
                        .unwrap_or_default(),
                });
            }
        }
        ix
    }

    /// Python: the first command of the queue while the project's own tests have not run in a
    /// loop: `dla harness gen --lang python --pytest` until a pytest-loop harness exists, then
    /// `dla harness run <H>` while one is generated and not run. `None` once a full pytest-loop
    /// harness (generated without `--tests`) wrote a series, or an attempt of the full loop
    /// failed (the refusal of the generation, a failed run, a run with no usable point). A
    /// narrowed loop changes nothing: it is the agent's own fallback, not the first step.
    pub fn pytest_loop_step(&self) -> Option<String> {
        if self.pytest_loops.iter().any(|h| self.run.contains(h)) {
            return None;
        }
        if let Some(h) = self
            .pytest_loops
            .iter()
            .find(|h| !self.test_loop_failed.contains(*h))
        {
            return Some(format!("dla harness run {h}"));
        }
        if self.pytest_refused || !self.pytest_loops.is_empty() {
            return None;
        }
        Some(PYTEST_LOOP_COMMAND.to_owned())
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
        // `unknown` (the run had no coverage data) says nothing about the target's code: it
        // counts as a driver run for the two-drivers rule below, not as a settled target.
        if runs.iter().any(|r| r.flat_reading()) {
            return Standing::Settled(Why::Executed);
        }
        // Not reached: missed by two different drivers, at least one of the runs a focused
        // harness (at most FOCUS_MAX targets). Two big batches naming a target settle nothing.
        let missed = self.misses(id);
        let keys: std::collections::BTreeSet<&str> =
            missed.iter().map(|r| r.driver_key.as_str()).collect();
        // Rust: a feature of the crate with gated sites that no harness enabled keeps the
        // target open, however many drivers missed it: the code behind the feature cannot run
        // in a build without it, so the misses say nothing about the target.
        if keys.len() >= 2
            && missed.iter().any(|r| r.focused)
            && self.untried_features(id).is_none()
        {
            return Standing::Settled(Why::NotReached);
        }
        Standing::Pending
    }

    /// Rust: the features of the crate of `id` with gated sites that none of the harnesses that
    /// ran for it enabled, and the command that generates a harness with them on (the latest
    /// harness's, so it carries the features that harness already had). `None` when every
    /// feature with gated sites was on in some harness, or no harness recorded any.
    pub fn untried_features(
        &self,
        id: &FindingId,
    ) -> Option<(Vec<crate::features::FeatureGap>, String)> {
        let runs = self.declared.get(id)?;
        let last = runs.last()?;
        let gaps: Vec<crate::features::FeatureGap> = last
            .feature_gaps
            .iter()
            .filter(|g| {
                runs.iter()
                    .all(|r| r.feature_gaps.iter().any(|x| x.feature == g.feature))
            })
            .cloned()
            .collect();
        (!gaps.is_empty()).then(|| (gaps, last.feature_command.clone()))
    }

    /// The second look of `id`: one read `executed` in a hook driver (not entry-path), nothing
    /// grew or missed, and no focused harness with another driver has run for it yet
    /// ([`SecondLook::Awaiting`]); or one has ([`SecondLook::Done`]).
    pub fn second_look(&self, id: &FindingId) -> SecondLook {
        let runs = self.declared.get(id).map_or(&[][..], Vec::as_slice);
        let moved = self.verdicts.get(id).is_some_and(|v| {
            v.iter()
                .any(|(_, x)| matches!(x, Verdict::Grew | Verdict::Changed))
        });
        let live = |r: &&DeclaredRun| r.hook_driver && !r.entry;
        // One flat reading: a hook driver that read `executed`, or the test loop (the project's
        // own tests) that read `executed` or `constant`. Only a hand-written focus harness ends
        // the wait (`done` below).
        let executed: Vec<&DeclaredRun> = runs.iter().filter(|r| r.flat_reading()).collect();
        if moved || executed.is_empty() || self.missed_in(id).is_some() {
            return SecondLook::No;
        }
        let done = runs.iter().filter(live).find(|r| {
            r.verdict.is_some()
                && (r.second_look
                    || (r.focused && executed.iter().any(|e| e.driver_key != r.driver_key)))
        });
        match done {
            Some(r) => SecondLook::Done(r.harness.clone()),
            None => SecondLook::Awaiting(executed[0].harness.clone()),
        }
    }

    /// The verdict a second-look harness read for `id`, when one has run and its trend was read.
    pub fn second_look_verdict(&self, id: &FindingId) -> Option<Verdict> {
        self.declared
            .get(id)?
            .iter()
            .filter(|r| r.second_look)
            .find_map(|r| r.verdict)
    }

    /// The runs of hook-driver harnesses (not entry-path) that missed `id`.
    fn misses(&self, id: &FindingId) -> Vec<&DeclaredRun> {
        self.declared
            .get(id)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .filter(|r| {
                !r.entry
                    && ((r.hook_driver && r.verdict.is_some_and(is_miss))
                        // The test loop's flat reading is `constant` or `executed`; the rest is
                        // a miss, as for a driver.
                        || (r.test_loop
                            && r.verdict.is_some_and(|v| {
                                is_miss(v) && v != Verdict::Constant
                            })))
            })
            .collect()
    }

    /// The first run that missed `id`; `None` when no run missed it.
    pub fn missed_in(&self, id: &FindingId) -> Option<Miss> {
        self.misses(id).first().and_then(|r| {
            Some(Miss {
                harness: r.harness.clone(),
                verdict: r.verdict?,
                detail: r.detail.clone().unwrap_or_default(),
                writers: r.writers.clone(),
            })
        })
    }

    /// The latest run of a hook driver that read `id` as `writer_not_executed`: its function ran
    /// and the lines that add to its holder did not. `None` when no run did.
    pub fn writer_miss(&self, id: &FindingId) -> Option<Miss> {
        let r = self
            .declared
            .get(id)?
            .iter()
            .rev()
            .find(|r| !r.entry && r.verdict == Some(Verdict::WriterNotExecuted))?;
        Some(Miss {
            harness: r.harness.clone(),
            verdict: Verdict::WriterNotExecuted,
            detail: r.detail.clone().unwrap_or_default(),
            writers: r.writers.clone(),
        })
    }

    /// The focused harness whose miss settled `id` as not reached.
    pub fn not_reached_in(&self, id: &FindingId) -> Option<String> {
        self.misses(id)
            .iter()
            .find(|r| r.focused)
            .map(|r| r.harness.clone())
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
            .chain(self.verdicts.get(id).into_iter().flatten().map(|(_, v)| *v))
            .min_by_key(|v| priority(*v))
    }

    /// The first harness of `runtime` with pending targets that has not run, if any.
    fn in_flight(&self, runtime: &str, pending: &[FindingId]) -> Option<String> {
        self.runtime
            .iter()
            .filter(|(h, rt)| {
                rt.as_str() == runtime
                    && !self.run.contains(*h)
                    && !self.test_loop_failed.contains(*h)
            })
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
    /// A driver run targeted it and its code ran (a verdict `executed`).
    Executed,
    /// Two driver runs with different drivers missed it, one of them a focused harness
    /// (at most three targets).
    NotReached,
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
        Verdict::WriterNotExecuted => 5,
        Verdict::NotLoaded => 6,
    }
}

/// Why a runtime's queue is not open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Closed {
    /// The run was started with `--queue off`.
    Off,
    /// The budget is at or under the floor (and only then).
    Floor,
    /// The run is over: aborted, or made incomplete by a merge. A stop-hook stamp is not
    /// over: the queue is as for a going run. Time may be left.
    Over,
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
            Self::Over => "the run is stamped incomplete or aborted",
            Self::Blocked => "intake.workload records a blocker",
            Self::Empty => "no candidate is waiting for a measurement",
        }
    }
}

/// A batch `--next` declares.
#[derive(Debug, Clone, Default)]
pub struct Batch {
    /// The candidates, in rank order (a focus batch: best-ranked first, then the others in
    /// the order of their choice).
    pub ids: Vec<FindingId>,
    /// The candidates held because of their owner, each with the one it came with (a broad
    /// batch only).
    pub siblings: BTreeMap<FindingId, FindingId>,
    /// True for a focus batch: the best-ranked pending candidate was one that a harness
    /// missed, so the batch is that candidate and at most [`FOCUS_MAX`] - 1 more missed ones.
    pub focus: bool,
    /// Go: the package directory of the batch.
    pub package: Option<String>,
    /// True for a focus batch of candidates that wait for a second look.
    pub second: bool,
}

/// Chooses the batch from `free` (the declarable pending candidates in rank order).
/// `missed` holds the candidates that a harness run missed. With `package` (Go) only the
/// candidates of that directory are taken from.
///
/// When the best-ranked candidate is one that was missed, the batch is a FOCUS batch: it,
/// then up to two more missed ones (same owner first, then same file, then next by rank),
/// no siblings and no fresh candidates. Otherwise it is the broad batch of fresh candidates
/// only: the first `head`, then their same-owner siblings. A missed candidate is never put
/// into a broad batch again. A Go batch is one package.
pub fn choose_batch<V>(
    view: &RunView,
    free: &[&FindingId],
    missed: &BTreeMap<FindingId, V>,
    head: usize,
    package: Option<&str>,
    go: bool,
) -> Batch {
    let in_pkg: Vec<&FindingId> = free
        .iter()
        .filter(|f| package.is_none_or(|p| package_of(view, f).is_some_and(|q| q == p)))
        .copied()
        .collect();
    let Some(first) = in_pkg.first() else {
        return Batch::default();
    };
    if missed.contains_key(*first) {
        let pkg = go.then(|| package_of(view, first)).flatten();
        let pool: Vec<&FindingId> = in_pkg
            .iter()
            .skip(1)
            .filter(|f| missed.contains_key(**f))
            .filter(|f| !go || package_of(view, f) == pkg)
            .copied()
            .collect();
        let of = |id: &FindingId| {
            view.ledger
                .finding(id)
                .map(|f| (f, view.ledger.annotation(id)))
        };
        let file = |id: &FindingId| {
            view.ledger
                .finding(id)
                .and_then(|f| f.site.as_deref())
                .and_then(|s| s.rsplit_once(':').map(|(f, _)| f.to_owned()))
        };
        let first_file = file(first);
        let mut ranked_pool: Vec<(u8, &FindingId)> = pool
            .iter()
            .map(|c| {
                let same_owner =
                    matches!((of(first), of(c)), (Some(a), Some(b)) if rank::same_owner(a, b));
                let class = if same_owner {
                    0
                } else if first_file.is_some() && file(c) == first_file {
                    1
                } else {
                    2
                };
                (class, *c)
            })
            .collect();
        // Stable: inside a class the rank order stays.
        ranked_pool.sort_by_key(|(class, _)| *class);
        let take = head.clamp(1, FOCUS_MAX);
        let mut ids: Vec<FindingId> = vec![(*first).clone()];
        ids.extend(
            ranked_pool
                .into_iter()
                .take(take - 1)
                .map(|(_, c)| c.clone()),
        );
        return Batch {
            ids,
            siblings: BTreeMap::new(),
            focus: true,
            package: pkg,
            second: false,
        };
    }
    let fresh: Vec<&FindingId> = in_pkg
        .iter()
        .filter(|f| !missed.contains_key(**f))
        .copied()
        .collect();
    if go {
        let p = package
            .map(str::to_owned)
            .or_else(|| go_packages(view, &fresh).into_iter().next());
        return match p {
            Some(p) => {
                let (ids, siblings) = package_batch(view, &fresh, head, &p);
                Batch {
                    ids,
                    siblings,
                    focus: false,
                    package: Some(p),
                    second: false,
                }
            }
            None => Batch::default(),
        };
    }
    let (ids, siblings) = with_siblings(view, &fresh, head);
    Batch {
        ids,
        siblings,
        focus: false,
        package: None,
        second: false,
    }
}

/// The queue of one runtime.
#[derive(Debug, Clone, Default)]
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
    /// The next batch: the first pending candidates that no unrun harness declares, then
    /// every other pending candidate that is state of the same owner as one of them (see
    /// [`MAX_BATCH_WITH_SIBLINGS`]).
    pub batch: Vec<FindingId>,
    /// The candidates `batch` holds because of their owner, each with the earlier candidate
    /// it came with.
    pub siblings: BTreeMap<FindingId, FindingId>,
    /// Go: the package directory every candidate of `batch` is in (`""` is the project
    /// root); `None` for other runtimes and for an empty batch.
    pub package: Option<String>,
    /// True when `batch` is a focus batch (see [`choose_batch`]).
    pub focus: bool,
    /// The pending candidates that wait for a second look (see [`SECOND_LOOK_MAX`]), each with
    /// the first harness that read it flat. Non-empty only while no first-pass candidate is
    /// pending; `batch` is then a second-look focus batch.
    pub second: BTreeMap<FindingId, Miss>,
    /// The pending candidates that a hook-driver harness run missed, each with the first
    /// harness that missed it and its verdict.
    pub missed: BTreeMap<FindingId, Miss>,
    /// The pending candidates whose best verdict is `unknown`: a driver ran, there was no
    /// coverage data, and nothing says their code ran.
    pub no_coverage: Vec<FindingId>,
    /// The pending candidates no unrun harness declares, in rank order: what a batch is taken
    /// from.
    pub declarable: Vec<FindingId>,
    /// A generated harness that declares pending candidates and has not run.
    pub in_flight: Option<String>,
    /// True when `in_flight` is a test-loop harness: nothing to fill, only to run.
    pub in_flight_test_loop: bool,
    /// The pending candidates the harness in flight declares.
    pub in_flight_ids: Vec<FindingId>,
    /// The command that makes the next batch: the test loop while a test-loop batch is left,
    /// else the driver.
    pub batch_command: String,
    /// The test loop (JVM): the pending candidates that have a test class, cut into the
    /// batches `dla harness gen --lang jvm --junit --next` makes, and the candidates that need
    /// a driver. Empty when the project has no tests the loop can run, and once every batch is
    /// made. While it holds a batch, `batch` is the first one and `batch_command` makes it.
    pub test_loop: crate::testloop::Plan,
    /// The command to run next: the batch command, or the run of the harness in flight when
    /// every pending candidate is already declared by it; for a Python queue that has test
    /// files, first the project's own tests in a loop (`py_loop`).
    pub command: String,
    /// Python: the first step, the project's own tests in a loop (`dla harness gen --lang
    /// python --pytest`, or the run of the loop's harness once generated), nothing to write.
    /// `None` when the project has no Python test file, once a pytest loop wrote a series or
    /// failed, and while the queue is not open. It is an order, not a gate rule: `command`
    /// is this step while it is `Some`, and `after_loop` is the command the queue goes on with.
    pub py_loop: Option<String>,
    /// What `command` would be without the Python test loop.
    pub after_loop: String,
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

    /// The command that gives the pending candidate `id` its measurement: the test loop's when
    /// a test-loop batch holds it, the driver's when a test-loop batch is left that does not
    /// (the candidate has no test class the loop can run), else the queue's next command.
    pub fn command_for(&self, id: &FindingId) -> String {
        if self.py_loop.is_some() {
            // The loop declares no target: the candidate's own command is the driver's.
            return self.after_loop.clone();
        }
        if self.test_loop.batches.is_empty()
            || self.test_loop.batches.iter().any(|b| b.ids.contains(id))
            // Declared by the test-loop harness that is generated and not run yet: run it.
            || (self.in_flight_test_loop && self.in_flight_ids.contains(id))
        {
            self.command.clone()
        } else {
            batch_command(&self.runtime, false)
        }
    }

    /// The command that gives the missed candidate `id` a harness of its own: the test loop's
    /// while a test-loop batch holds it (a hook driver missed it and its test class has not
    /// run yet), else a driver's.
    pub fn missed_command(&self, id: &FindingId) -> String {
        batch_command(
            &self.runtime,
            self.test_loop.batches.iter().any(|b| b.ids.contains(id)),
        )
    }

    /// The sentence that names the next step with the test loop first while a test-loop batch
    /// is left: "N candidates in M test-loop batches, nothing to write; then K candidates need
    /// a driver: <command>". `None` without a test-loop batch.
    pub fn test_loop_line(&self) -> Option<String> {
        (!self.test_loop.batches.is_empty())
            .then(|| format!("{}: {}", self.test_loop.sentence(), self.command))
    }

    /// The batch of `--next n`: the first `n` (at most [`MAX_BATCH`]) declarable candidates
    /// and the siblings of the owners they hold.
    pub fn batch_for(&self, view: &RunView, n: usize) -> Batch {
        let free: Vec<&FindingId> = self.declarable.iter().collect();
        let go = self.runtime == GO;
        let pool = self.pool();
        // A Go batch is one package: the one of the best-ranked candidate when that one was
        // missed or waits for a second look (a focus batch), else of the best-ranked fresh one.
        let package = go
            .then(|| {
                let first = free.first()?;
                if pool.contains_key(*first) {
                    package_of(view, first)
                } else {
                    let fresh: Vec<&FindingId> = free
                        .iter()
                        .filter(|f| !pool.contains_key(**f))
                        .copied()
                        .collect();
                    go_packages(view, &fresh).into_iter().next()
                }
            })
            .flatten();
        if go && package.is_none() {
            return Batch::default();
        }
        let mut b = choose_batch(view, &free, &pool, n, package.as_deref(), go);
        b.second = b.focus && self.missed.is_empty() && !self.second.is_empty();
        b
    }

    /// The candidates a focus batch is chosen from: the missed ones, or (when the first pass
    /// is over) the ones that wait for a second look.
    fn pool(&self) -> BTreeMap<FindingId, ()> {
        self.missed
            .keys()
            .chain(self.second.keys())
            .map(|f| (f.clone(), ()))
            .collect()
    }

    /// Go: the package directories of the declarable candidates, in the rank order of their
    /// best candidate. Empty for other runtimes.
    pub fn packages(&self, view: &RunView) -> Vec<String> {
        if self.runtime != GO {
            return Vec::new();
        }
        let free: Vec<&FindingId> = self.declarable.iter().collect();
        go_packages(view, &free)
    }

    /// Go: the batch of the declarable candidates in the package directory `package`.
    pub fn batch_in_package(&self, view: &RunView, n: usize, package: &str) -> Batch {
        let free: Vec<&FindingId> = self.declarable.iter().collect();
        let mut b = choose_batch(view, &free, &self.pool(), n, Some(package), true);
        b.second = b.focus && self.missed.is_empty() && !self.second.is_empty();
        b
    }

    /// The rank and tier of a pending candidate.
    pub fn rank_of(&self, id: &FindingId) -> Option<(usize, u8)> {
        self.pending
            .iter()
            .position(|p| p == id)
            .and_then(|i| self.pending_rank.get(i).copied())
    }
}

/// The Python test loop: the project's own tests in a loop in one process, nothing to write.
pub const PYTEST_LOOP_COMMAND: &str = "dla harness gen --lang python --pytest";

/// The command that makes the next batch of `runtime`: the project's own tests in a loop
/// (`test_loop`, JVM) while a test-loop batch is left, else a driver with hooks.
pub fn batch_command(runtime: &str, test_loop: bool) -> String {
    if test_loop {
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
    let py_tests = by_rt
        .contains_key("python")
        .then(|| crate::testloop::python_tests_of(view, worker))
        .flatten();
    let tindex = by_rt
        .contains_key("jvm")
        .then(|| crate::testloop::TestIndex::of(view, worker))
        .flatten();
    by_rt
        .into_iter()
        .map(|(runtime, list)| {
            // A blocker recorded on `intake.workload` closes the queue of the runtime it names
            // (every queue when it names none).
            let blocked = gate::workload_unresolved(view, true, &runtime);
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
            // The second look: the first pass first; then, while the run is going and budget is
            // left, the candidates one flat run of one driver settled wait for a different
            // driver (see `SECOND_LOOK_MAX`).
            let mut second: BTreeMap<FindingId, Miss> = BTreeMap::new();
            if pending.is_empty() && view.state.queue && budget.is_some() && !blocked {
                let used = ix.second_looks.get(&runtime).map_or(0, Vec::len);
                let in_flight_second = |id: &FindingId| {
                    ix.declared_by.get(id).is_some_and(|hs| {
                        hs.iter().any(|h| {
                            ix.second_looks.get(&runtime).is_some_and(|l| l.contains(h))
                                && !ix.run.contains(h)
                        })
                    })
                };
                let mut room = SECOND_LOOK_MAX.saturating_sub(used) * FOCUS_MAX;
                for r in &open_ones {
                    let SecondLook::Awaiting(first) = ix.second_look(&r.id) else {
                        continue;
                    };
                    if in_flight_second(&r.id) || room > 0 {
                        if !in_flight_second(&r.id) {
                            room -= 1;
                        }
                        pending.push(r.id.clone());
                        pending_rank.push((r.rank.unwrap_or(0), r.tier));
                        second.insert(
                            r.id.clone(),
                            Miss {
                                harness: first,
                                verdict: Verdict::Executed,
                                detail: "its code ran and its holder stayed flat under one driver"
                                    .into(),
                                writers: Vec::new(),
                            },
                        );
                    }
                }
            }
            let no_coverage: Vec<FindingId> = pending
                .iter()
                .filter(|p| ix.best_verdict(p) == Some(Verdict::Unknown))
                .cloned()
                .collect();
            let in_flight = ix.in_flight(&runtime, &pending);
            let in_flight_of = |p: &FindingId| {
                in_flight
                    .as_ref()
                    .is_some_and(|h| ix.declared_by.get(p).is_some_and(|v| v.contains(h)))
            };
            let in_flight_ids: Vec<FindingId> = pending
                .iter()
                .filter(|p| in_flight_of(p))
                .cloned()
                .collect();
            // The test loop first: the pending candidates that have a test class and no
            // test-loop harness yet, in batches (see `testloop`).
            let mut test_loop = crate::testloop::Plan::default();
            if runtime == "jvm"
                && view.state.queue
                && budget.is_some()
                && !blocked
                && let Some(tix) = &tindex
            {
                let order: Vec<(FindingId, Option<crate::testloop::Pick>)> = pending
                    .iter()
                    .filter(|p| !ix.test_loop_declared.contains(*p) && !second.contains_key(*p))
                    .filter(|p| !in_flight_of(p))
                    .map(|p| {
                        let file = view
                            .ledger
                            .finding(p)
                            .and_then(|f| f.site.as_deref())
                            .and_then(|s| s.rsplit_once(':'))
                            .map(|(f, _)| f.replace('\\', "/"));
                        (p.clone(), file.and_then(|f| tix.pick(&f)))
                    })
                    .collect();
                test_loop = crate::testloop::plan(tix, &order);
            }
            let planned: std::collections::BTreeSet<&FindingId> =
                test_loop.batches.iter().flat_map(|b| &b.ids).collect();
            let free: Vec<&FindingId> = pending
                .iter()
                .filter(|p| !in_flight_of(p) && !planned.contains(p))
                .collect();
            let missed: BTreeMap<FindingId, Miss> = pending
                .iter()
                .filter_map(|p| ix.missed_in(p).map(|m| (p.clone(), m)))
                .collect();
            let declarable: Vec<FindingId> = free.iter().map(|f| (*f).clone()).collect();
            let chooser = RuntimeQueue {
                runtime: runtime.clone(),
                declarable: declarable.clone(),
                missed: missed.clone(),
                second: second.clone(),
                ..RuntimeQueue::default()
            };
            let Batch {
                ids: batch,
                siblings,
                focus,
                package,
                ..
            } = chooser.batch_for(view, MAX_BATCH);
            test_loop.driver = declarable.clone();
            let (batch, siblings, focus) = match test_loop.batches.first() {
                Some(b) => (b.ids.clone(), BTreeMap::new(), false),
                None => (batch, siblings, focus),
            };
            let batch_command = batch_command(&runtime, !test_loop.batches.is_empty());
            let after_loop = match (&in_flight, batch.is_empty()) {
                (Some(h), _) if ix.test_loops.contains(h) => format!("dla harness run {h}"),
                (Some(h), true) => format!("dla harness run {h} --mode both"),
                _ => batch_command.clone(),
            };
            let closed = if !view.state.queue {
                Some(Closed::Off)
            } else if budget.is_none() {
                // `budget_over_floor` is also `None` for a run that is over, whatever time is
                // left: the queue then says so and not that the floor was reached.
                if matches!(
                    view.state.status,
                    crate::state::RunStatus::Aborted | crate::state::RunStatus::Incomplete
                ) && !view.state.stop_stamped()
                {
                    Some(Closed::Over)
                } else {
                    Some(Closed::Floor)
                }
            } else if blocked {
                Some(Closed::Blocked)
            } else if pending.is_empty() {
                Some(Closed::Empty)
            } else {
                None
            };
            // Python: the project's own tests in a loop come first, for an open queue of a run
            // that has Python test files (see `Index::pytest_loop_step`).
            let py_loop = if runtime == "python" && closed.is_none() && py_tests.is_some() {
                ix.pytest_loop_step()
            } else {
                None
            };
            let command = py_loop.clone().unwrap_or_else(|| after_loop.clone());
            RuntimeQueue {
                runtime,
                eligible,
                settled: eligible - pending.len(),
                pending,
                pending_rank,
                batch,
                siblings,
                package,
                focus,
                missed,
                second,
                no_coverage,
                declarable,
                in_flight_test_loop: in_flight
                    .as_ref()
                    .is_some_and(|h| ix.test_loops.contains(h)),
                in_flight_ids,
                in_flight,
                batch_command,
                test_loop: if closed.is_some() {
                    crate::testloop::Plan::default()
                } else {
                    test_loop
                },
                command,
                py_loop,
                after_loop,
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

/// What a verdict `unknown` means for a target, in every text that explains its standing.
pub const NO_COVERAGE: &str = "the run has no coverage data, so whether its code ran is not known: run a second, different driver, or one with a size reading";

/// The full text of the guidance a refusal or an item gives for an open queue.
pub fn steps(q: &RuntimeQueue, run_id: &str) -> String {
    let _ = run_id;
    let focus = if q.focus && q.missed.is_empty() && !q.second.is_empty() {
        " The next batch is a second look: one flat run of one driver settled these candidates, and budget is left, so they get a harness of their own (3 targets at most) with a different driver (other call orders, objects derived from derived objects, failing and cancelling paths, overlapping lifetimes) and a VARIES.txt line per target; `dla harness run` refuses hooks equal to an earlier harness's."
    } else if q.focus {
        " The next batch is a focus batch: the best-ranked pending candidate was missed by an earlier harness, so it gets a harness of its own (at most 3 targets, no siblings); a target no driver can reach is given up only with a reason: dla ledger resolve <F> --as unresolved --reason env_unavailable --remedy \"<what it needs>\"."
    } else {
        ""
    };
    let unknown = if q.no_coverage.is_empty() {
        String::new()
    } else {
        let ids: Vec<String> = q.no_coverage.iter().map(ToString::to_string).collect();
        format!(" {} read `unknown`: {NO_COVERAGE}.", ids.join(", "))
    };
    let resolve = format!(
        "If no driver can be built or run for {rt}, record why: dla ledger resolve intake.workload --status unresolved --reason <env_unavailable|tool_missing|build_fails|privilege_declined> --remedy \"<text>\" --runtime {rt} (that closes the queue of {rt} only; the queues of other runtimes stay open).{unknown}{focus}",
        rt = q.runtime
    );
    if q.in_flight_test_loop {
        return format!(
            "{}: the project's own tests in a loop, nothing to fill; then dla trend S<n>. {}{resolve}",
            q.command,
            test_loop_rules()
        );
    }
    let driver = batch_command(&q.runtime, false);
    if let Some(first) = &q.py_loop {
        let how = if first.starts_with("dla harness run") {
            format!("{first}, dla trend S<n>")
        } else {
            format!("{first}, dla harness run <H>, dla trend S<n>")
        };
        return format!(
            "First the project's own tests in a loop, nothing to write: {how}; a line of the project that grows there gets a finding (the trend prints the command). The loop declares no target, so the candidates still need their drivers: {driver}, fill the hooks so each unit reaches the listed targets with a new key, dla harness run <H> --mode both, then dla trend S<n>. {resolve}"
        );
    }
    if !q.test_loop.batches.is_empty() {
        return format!(
            "{}. Next: {} (it chooses the test classes from the owner classes of the candidates and prints which and why; nothing to fill), dla harness run <H>, then dla trend S<n>. {} After the test loop, for the candidates that need a driver: {driver}, fill the hooks so each unit reaches the listed targets with a new key, dla harness run <H> --mode both, then dla trend S<n>. {resolve}",
            capitalized(&q.test_loop.sentence()),
            q.command,
            test_loop_rules()
        );
    }
    format!(
        "{}. Fill the hooks so each unit reaches the listed targets with a new key, dla harness run <H> --mode both, then dla trend S<n>. {resolve}",
        q.command
    )
}

/// What a reading of the test loop means for the queue.
pub fn test_loop_rules() -> &'static str {
    "A target that grew goes on as usual (sibling, refute, test, confirm); a flat reading (`constant`, `executed`) is one reading, and a hand-written harness with a different driver looks at it again while budget is left; a target no test reached (`not_loaded`, `not_executed`) is a miss and gets a focus harness; if the harness cannot be built or every test fails, the run prints the driver command for the same targets and they are not counted as measured."
}

fn capitalized(s: &str) -> String {
    let mut c = s.chars();
    c.next()
        .map(|f| f.to_uppercase().collect::<String>() + c.as_str())
        .unwrap_or_default()
}

/// What the report says about the queue: its header line and, per finding, how it was
/// targeted.
///
/// The header line counts every finding the queue took when it was built, whatever its state
/// now: a leak-side, non-sibling, non-duplicate finding of an eligible tier (1 to 4) in any
/// state. A finding that grew and was then taken under test, confirmed or refuted still
/// counts as targeted and, by its best verdict, as grew; the line is a record of the
/// measurement the queue did, not of the candidates still open.
#[derive(Debug)]
pub struct ReportFacts {
    ranks: BTreeMap<FindingId, (Option<usize>, u8)>,
    /// The findings the header line counts.
    counted: Vec<FindingId>,
    index: Index,
    queue_on: bool,
    /// Of the candidates no harness declared: how many have a test class the test loop runs,
    /// in how many batches (JVM).
    test_loop: Option<(usize, usize)>,
}

impl ReportFacts {
    /// The facts of a view loaded from disk; `None` for a view without a run directory.
    pub fn of(view: &RunView) -> Option<Self> {
        view.run_dir.as_ref()?;
        let all = ranked(view);
        let index = Index::of(view);
        let test_loop = crate::testloop::TestIndex::of(view, None).and_then(|tix| {
            let order: Vec<(FindingId, Option<crate::testloop::Pick>)> = all
                .iter()
                .filter(|r| {
                    r.eligible()
                        && r.runtime.as_deref() == Some("jvm")
                        && index.targeted_in(&r.id).is_empty()
                        // A growth a harness's trend tied to it settled it: no test loop needed.
                        && index.standing(&r.id) == Standing::Pending
                })
                .map(|r| {
                    let file = view
                        .ledger
                        .finding(&r.id)
                        .and_then(|f| f.site.as_deref())
                        .and_then(|s| s.rsplit_once(':'))
                        .map(|(f, _)| f.replace('\\', "/"));
                    (r.id.clone(), file.and_then(|f| tix.pick(&f)))
                })
                .collect();
            let plan = crate::testloop::plan(&tix, &order);
            (plan.candidates() > 0).then(|| (plan.candidates(), plan.batches.len()))
        });
        let ranks = all.into_iter().map(|r| (r.id, (r.rank, r.tier))).collect();
        let counted = view
            .ledger
            .findings()
            .filter(|f| {
                !f.id.is_sibling()
                    && f.polarity() == Polarity::Leak
                    && view.ledger.merged_into(&f.id).is_none()
                    && rank::eligible_tier(rank::tier(f, view.ledger.annotation(&f.id)))
            })
            .map(|f| f.id.clone())
            .collect();
        Some(Self {
            ranks,
            counted,
            index,
            queue_on: view.state.queue,
            test_loop,
        })
    }

    /// The `Queue:` header line, or `None` when nothing was ranked for measurement.
    pub fn line(&self) -> Option<String> {
        if !self.queue_on {
            return Some(
                "off: candidates the workloads did not reach were closed without a targeted workload"
                    .to_owned(),
            );
        }
        let ranked: Vec<&FindingId> = self.counted.iter().collect();
        if ranked.is_empty() {
            return None;
        }
        let mut harnesses: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        let (mut grew, mut changed, mut executed, mut no_coverage, mut missed, mut unread) =
            (0, 0, 0, 0, 0, 0);
        let mut missed_once = 0;
        let mut missed_gated = 0;
        let mut targeted = 0;
        for id in &ranked {
            let t = self.index.targeted_in(id);
            if t.is_empty() {
                continue;
            }
            targeted += 1;
            harnesses.extend(t.iter().map(|(h, _)| h.clone()));
            match self.index.best_verdict(id) {
                Some(Verdict::Grew) => grew += 1,
                Some(Verdict::Changed) => changed += 1,
                Some(Verdict::Executed) => executed += 1,
                Some(Verdict::Unknown) => no_coverage += 1,
                Some(
                    Verdict::Constant
                    | Verdict::NotExecuted
                    | Verdict::WriterNotExecuted
                    | Verdict::NotLoaded,
                ) => {
                    // "Not reached" is only what the rule settled: missed by two drivers, one
                    // of them a focused harness. A miss with no focus harness is "missed once".
                    if self.index.standing(id) == Standing::Settled(Why::NotReached) {
                        missed += 1;
                    } else if self.index.untried_features(id).is_some() {
                        missed_gated += 1;
                    } else {
                        missed_once += 1;
                    }
                }
                None => unread += 1,
            }
        }
        let mut parts: Vec<String> = Vec::new();
        for (n, what) in [
            (grew, "grew"),
            (changed, "changed"),
            (executed, "executed"),
            (no_coverage, "no coverage data"),
            (missed, "not reached"),
            (missed_once, "missed once, no focus harness"),
            (
                missed_gated,
                "missed, a feature with gated sites not enabled",
            ),
            (unread, "not read yet"),
        ] {
            if n > 0 {
                parts.push(format!("{what} {n}"));
            }
        }
        let in_h = if harnesses.is_empty() {
            String::new()
        } else {
            format!(
                " in {} harness{}",
                harnesses.len(),
                if harnesses.len() == 1 { "" } else { "es" }
            )
        };
        let verdicts = if parts.is_empty() {
            String::new()
        } else {
            format!(": {}", parts.join(", "))
        };
        // Candidates one flat run settled that got a different driver, and how many of those grew.
        let looked: Vec<Verdict> = ranked
            .iter()
            .filter_map(|id| self.index.second_look_verdict(id))
            .collect();
        let grew_on_look = looked.iter().filter(|v| **v == Verdict::Grew).count();
        let second = if looked.is_empty() {
            String::new()
        } else {
            format!("; second look {} (grew {grew_on_look})", looked.len())
        };
        let not_targeted = ranked.len() - targeted;
        // Where the rest goes: the test loop first (nothing to write), then a driver.
        let split = match self.test_loop {
            Some((n, m)) if n <= not_targeted => format!(
                " ({n} in {m} test-loop batch{}, nothing to write; {} need a driver)",
                if m == 1 { "" } else { "es" },
                not_targeted - n
            ),
            _ => String::new(),
        };
        Some(format!(
            "{targeted} of {} ranked candidates targeted{in_h}{verdicts}; {not_targeted} not targeted{split}{second}",
            ranked.len()
        ))
    }

    /// How a closed finding was targeted: `Targeted: H2 constant, H3 not executed` or
    /// `Not targeted (rank 143)`. `None` when the queue was off.
    pub fn targeted(&self, id: &FindingId) -> Option<String> {
        if !self.queue_on {
            return None;
        }
        let t = self.index.targeted_in(id);
        if t.is_empty() {
            return Some(match self.ranks.get(id) {
                Some((Some(rank), _)) => format!("Not targeted (rank {rank})"),
                Some((None, tier)) => {
                    format!("Not targeted (tier {tier}, not ranked for measurement)")
                }
                None => "Not targeted".to_owned(),
            });
        }
        let list: Vec<String> = t
            .iter()
            .map(|(h, v)| {
                let what = v.map_or("not run".to_owned(), |v| v.as_str().replace('_', " "));
                if *v == Some(Verdict::Unknown) {
                    format!("{h} {what} ({NO_COVERAGE})")
                } else {
                    format!("{h} {what}")
                }
            })
            .collect();
        let untried = self.index.untried_features(id);
        let tail = match (self.index.not_reached_in(id), self.index.missed_in(id)) {
            (Some(h), _) if self.index.standing(id) == Standing::Settled(Why::NotReached) => {
                format!("; not reached (missed by two drivers, the focused harness {h} among them)")
            }
            (_, Some(_)) if untried.is_some() => {
                let (gaps, command) = untried.unwrap_or_default();
                format!(
                    "; missed, and not settled: {}",
                    crate::features::lead(&gaps, &command)
                )
            }
            (_, Some(_)) => "; missed once, no focus harness".to_owned(),
            _ => String::new(),
        };
        Some(format!("Targeted: {}{tail}", list.join(", ")))
    }
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
            Self::with_budget(n, 3600)
        }

        /// As [`Run::new`] with a budget of `secs` seconds.
        fn with_budget(n: u32, secs: u64) -> (Self, RunView) {
            let dir = tempfile::tempdir().unwrap();
            let st = RunState::init("/p", None, secs, None, now_utc()).unwrap();
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
                        writers: Vec::new(),
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
        assert_eq!(
            q.batch_command,
            "dla harness gen --lang python --driver --next"
        );
        // F1 grew in a trend of an automatic probe: settled by growth alone.
        run.harness(&manifest("H1", &[2], true));
        run.series("S1", "H1");
        run.trend("T1", "S1", &[(1, Verdict::Grew), (2, Verdict::Constant)]);
        assert_eq!(pending_of(&run.reload()), ["F2", "F3"]);
    }

    #[test]
    fn the_queue_line_counts_a_finding_that_left_the_candidate_state() {
        let (run, _) = Run::new(3);
        run.harness(&manifest("H1", &[1, 2, 3], true));
        run.series("S1", "H1");
        run.trend(
            "T1",
            "S1",
            &[
                (1, Verdict::Grew),
                (2, Verdict::Executed),
                (3, Verdict::Constant),
            ],
        );
        let line = |run: &Run| ReportFacts::of(&run.reload()).unwrap().line().unwrap();
        let before = line(&run);
        assert_eq!(
            before,
            "3 of 3 ranked candidates targeted in 1 harness: grew 1, executed 1, missed once, no focus harness 1; 0 not targeted"
        );
        // F1 grew and was taken under test (later: confirmed); it still counts, as grew.
        let mut l = Ledger::load(run.dir.path()).unwrap();
        let mut ev = evidence("python");
        ev.kind = EvidenceKind::Workload;
        ev.authority = Authority::Test;
        ev.result = json!({"harness": "H1", "mode": "driver", "runtime": "python"});
        l.add_evidence(ev).unwrap();
        let mut sib = NewFinding::new(
            Origin::parse("hypothesis").unwrap(),
            vec![crate::EvidenceId::new(1)],
        );
        sib.sibling_of = Some(FindingId::new(1));
        sib.hypothesis = Some("N-WARM".into());
        sib.refute_if = Some("keeps growing".into());
        l.add_finding(sib).unwrap();
        l.transition(
            &FindingId::new(1),
            FindingState::UnderTest,
            None,
            None,
            vec![crate::EvidenceId::new(2)],
            "t",
        )
        .unwrap();
        assert!(!in_universe(
            RunView::load(run.dir.path())
                .unwrap()
                .ledger
                .finding(&FindingId::new(1))
                .unwrap()
        ));
        assert_eq!(line(&run), before);
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
    fn executed_settles_only_a_declared_driver_target_and_unknown_does_not() {
        let (run, _) = Run::new(3);
        run.harness(&manifest("H1", &[1, 2], true));
        run.series("S1", "H1");
        run.trend(
            "T1",
            "S1",
            &[
                (1, Verdict::Executed),
                (2, Verdict::Unknown),
                (3, Verdict::Executed),
            ],
        );
        // F3 was not declared by the harness: its verdict does not settle it. F2 read
        // `unknown` (no coverage data): nothing says its code ran.
        assert_eq!(pending_of(&run.reload()), ["F2", "F3"]);
        let q = queues(&run.reload(), None).remove(0);
        assert_eq!(q.no_coverage, [FindingId::new(2)]);
        assert!(
            steps(&q, "R").contains("F2 read `unknown`"),
            "{}",
            steps(&q, "R")
        );
        assert!(
            steps(&q, "R").contains("run a second, different driver, or one with a size reading")
        );
    }

    #[test]
    fn an_unknown_run_counts_as_one_of_two_drivers() {
        let (run, _) = Run::new(1);
        run.harness(&manifest("H1", &[1], true));
        let driver = |text: &str| {
            let d = run.dir.path().join("harness").join("H1").join("driver");
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("dla_harness.py"), text).unwrap();
        };
        let record = |series: &str| {
            let digest = crate::harness::driver_digest(run.dir.path(), "H1").unwrap();
            let text = json!({"series": [series], "driver_digest": digest});
            std::fs::write(
                run.dir
                    .path()
                    .join("harness")
                    .join("H1")
                    .join(format!("run-{series}.json")),
                text.to_string(),
            )
            .unwrap();
        };
        driver("a");
        run.series("S1", "H1");
        record("S1");
        run.trend("T1", "S1", &[(1, Verdict::Unknown)]);
        assert_eq!(pending_of(&run.reload()), ["F1"]);
        driver("b");
        run.series("S2", "H1");
        record("S2");
        run.trend("T2", "S2", &[(1, Verdict::Unknown)]);
        assert!(pending_of(&run.reload()).is_empty());
    }

    #[test]
    fn the_targeted_line_says_what_unknown_means() {
        let (run, _) = Run::new(1);
        run.harness(&manifest("H1", &[1], true));
        run.series("S1", "H1");
        run.trend("T1", "S1", &[(1, Verdict::Unknown)]);
        let facts = ReportFacts::of(&run.reload()).unwrap();
        let t = facts.targeted(&FindingId::new(1)).unwrap();
        assert!(
            t.contains("H1 unknown (the run has no coverage data"),
            "{t}"
        );
        let line = facts.line().unwrap();
        assert!(
            line.contains("no coverage data 1") && !line.contains("executed"),
            "{line}"
        );
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
                run.dir
                    .path()
                    .join("harness")
                    .join(h)
                    .join(format!("run-{first}.json")),
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
    fn floor_is_the_reason_only_when_the_time_left_is_at_the_floor() {
        use crate::state::RunStatus;
        let (_run, view) = Run::new(1);
        // Time left and the run going: open.
        assert!(queues(&view, None)[0].open());
        // An aborted run, or one a merge made incomplete, with most of its time left is over,
        // not at the floor.
        let mut aborted = view.clone();
        aborted.state.status = RunStatus::Aborted;
        assert_eq!(queues(&aborted, None)[0].closed, Some(Closed::Over));
        let mut merged = view.clone();
        merged.state.status = RunStatus::Incomplete;
        merged.state.incomplete_reason = Some("workers did not finish".into());
        assert_eq!(queues(&merged, None)[0].closed, Some(Closed::Over));
        // A stop-hook stamp does not close the queue: the time left decides.
        let mut stamped = view.clone();
        stamped.state.stamp_stop(format!(
            "{} 3 stop attempts with no ledger change: F1",
            crate::gate::STOP_BLOCKED_REASON
        ));
        assert!(queues(&stamped, None)[0].open());
        stamped.state.budget.deadline = crate::state::now_utc() - chrono::Duration::minutes(1);
        assert_eq!(queues(&stamped, None)[0].closed, Some(Closed::Floor));
        // No time left on a run that is going: the floor.
        let mut late = view.clone();
        late.state.budget.deadline = crate::state::now_utc() - chrono::Duration::minutes(1);
        assert_eq!(queues(&late, None)[0].closed, Some(Closed::Floor));
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
            let mut nf = NewFinding::new(
                Origin::parse(origin).unwrap(),
                vec![crate::EvidenceId::new(1)],
            );
            nf.site = Some(site.into());
            l.add_finding(nf).unwrap();
        }
        let view = RunView::load(dir.path()).unwrap();
        let r = ranked(&view);
        let order: Vec<(String, u8, Option<usize>)> = r
            .iter()
            .map(|x| (x.id.to_string(), x.tier, x.rank))
            .collect();
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

    /// A run of census candidates `(owner, site, lifetime, writer function)`, each with one
    /// insert site in the named function, in ledger order.
    fn owned_run(items: &[(&str, &str, crate::census::Lifetime, &str)]) -> (Run, RunView) {
        use crate::census::{ContainerClass, HolderFacts, SiteFact, SiteKind};
        let dir = tempfile::tempdir().unwrap();
        let st = RunState::init("/p", None, 3600, None, now_utc()).unwrap();
        st.save(dir.path()).unwrap();
        let mut l = Ledger::create(dir.path()).unwrap();
        l.add_evidence(evidence("python")).unwrap();
        for (i, (owner, site, life, function)) in items.iter().enumerate() {
            let mut nf = NewFinding::new(
                Origin::parse("census").unwrap(),
                vec![crate::EvidenceId::new(1)],
            );
            nf.site = Some((*site).to_owned());
            nf.owner = Some((*owner).to_owned());
            l.add_finding(nf).unwrap();
            let file = site.rsplit_once(':').unwrap().0;
            let w = format!("{file}:{}", 50 + i);
            l.annotate_record(crate::FindingAnnotation {
                writers: vec![w.clone()],
                facts: HolderFacts {
                    container: Some(ContainerClass::Container),
                    lifetime: Some(*life),
                    writer_sites: vec![SiteFact {
                        site: w,
                        kind: SiteKind::Insert,
                        depth: 0,
                        function: Some((*function).to_owned()),
                        init: false,
                        startup: false,
                        startup_via: Vec::new(),
                        param: true,
                        constant: false,
                        exported: true,
                    }],
                    ..HolderFacts::default()
                },
                ..crate::FindingAnnotation::new(FindingId::new(u32::try_from(i + 1).unwrap()))
            })
            .unwrap();
        }
        let view = RunView::load(dir.path()).unwrap();
        (Run { dir }, view)
    }

    #[test]
    fn the_order_inside_the_eligible_set_is_the_score_not_the_ledger() {
        use crate::census::Lifetime::Object;
        // F1 and F2 are the same tier in ledger order; F1's holder is weak, F2's is plain.
        let (run, _) = owned_run(&[
            ("pkg.m.A.weak", "pkg/m.py:3", Object, "put"),
            ("pkg.n.B.plain", "pkg/n.py:3", Object, "put"),
        ]);
        let mut l = Ledger::load(run.dir.path()).unwrap();
        l.annotate_record(crate::FindingAnnotation {
            facts: crate::census::HolderFacts {
                bound: Some(crate::census::Bound::WeakKeys),
                ..Default::default()
            },
            ..crate::FindingAnnotation::new(FindingId::new(1))
        })
        .unwrap();
        let r = ranked(&run.reload());
        let order: Vec<String> = r.iter().map(|x| x.id.to_string()).collect();
        assert_eq!(order, ["F2", "F1"]);
        assert!(r[0].score > r[1].score);
        assert_eq!(r[0].rank, Some(1));
    }

    #[test]
    fn a_batch_brings_the_holders_of_one_owner_with_it() {
        use crate::census::Lifetime::Object;
        let mut items = Vec::new();
        let sites: Vec<String> = (0..9).map(|i| format!("pkg/m{i}.py:3")).collect();
        let owners: Vec<String> = (0..9).map(|i| format!("pkg.m{i}.C{i}.x")).collect();
        for i in 0..9 {
            items.push((owners[i].as_str(), sites[i].as_str(), Object, "put"));
        }
        // F10: another field of the class of F1 (same file, same owner path).
        items.push(("pkg.m0.C0.y", "pkg/m0.py:4", Object, "put"));
        // F11: a field of a class of the same name in another file: not a sibling.
        items.push(("pkg.z.C0.y", "pkg/z.py:4", Object, "put"));
        let (_run, view) = owned_run(&items);
        let q = &queues(&view, None)[0];
        let ids: Vec<String> = q.batch.iter().map(ToString::to_string).collect();
        assert_eq!(
            ids,
            ["F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F10"],
            "{ids:?}"
        );
        assert_eq!(q.siblings.len(), 1);
        assert_eq!(q.siblings[&FindingId::new(10)], FindingId::new(1));
        // `--next 3` takes the first three by rank and the sibling of F1; F9 is not related.
        let b3 = q.batch_for(&view, 3);
        let ids: Vec<String> = b3.ids.iter().map(ToString::to_string).collect();
        assert_eq!(ids, ["F1", "F2", "F3", "F10"]);
        assert_eq!(b3.siblings.len(), 1);
        assert!(!b3.focus);
    }

    #[test]
    fn module_level_holders_are_siblings_only_through_a_shared_writer_function() {
        use crate::census::Lifetime::Process;
        let mut items = vec![("pkg.m.FIRST", "pkg/m.py:1", Process, "register")];
        // Seven unrelated candidates fill the head of the batch...
        let sites: Vec<String> = (0..7).map(|i| format!("pkg/o{i}.py:1")).collect();
        let owners: Vec<String> = (0..7).map(|i| format!("pkg.o{i}.V{i}")).collect();
        for i in 0..7 {
            items.push((owners[i].as_str(), sites[i].as_str(), Process, "x"));
        }
        // ...and then two module-level holders of F1's module, one sharing its writer function.
        items.push(("pkg.m.SECOND", "pkg/m.py:2", Process, "register"));
        items.push(("pkg.m.THIRD", "pkg/m.py:3", Process, "elsewhere"));
        let (_run, view) = owned_run(&items);
        let q = &queues(&view, None)[0];
        assert!(q.batch.contains(&FindingId::new(9)), "{:?}", q.batch);
        assert!(!q.batch.contains(&FindingId::new(10)), "{:?}", q.batch);
        assert_eq!(q.siblings[&FindingId::new(9)], FindingId::new(1));
    }

    /// Writes harness `h` declaring `targets` as a hook driver with the driver text `driver`
    /// (so two harnesses with different text are two different drivers), one series `s` and
    /// a trend `t` with `verdicts`.
    fn ran(
        run: &Run,
        h: &str,
        targets: &[u32],
        driver: &str,
        s: &str,
        verdicts: &[(u32, Verdict)],
    ) {
        run.harness(&manifest(h, targets, true));
        let d = run.dir.path().join("harness").join(h).join("driver");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("dla_harness.py"), driver).unwrap();
        run.series(s, h);
        let digest = crate::harness::driver_digest(run.dir.path(), h).unwrap();
        std::fs::write(
            run.dir
                .path()
                .join("harness")
                .join(h)
                .join(format!("run-{s}.json")),
            json!({"series": [s], "driver_digest": digest}).to_string(),
        )
        .unwrap();
        run.trend(&format!("T{}", &s[1..]), s, verdicts);
    }

    #[test]
    fn two_big_batches_that_missed_a_target_leave_it_pending_and_missed() {
        let (run, _) = Run::new(5);
        ran(
            &run,
            "H1",
            &[1, 2, 3, 4],
            "a",
            "S1",
            &[
                (1, Verdict::NotLoaded),
                (2, Verdict::Grew),
                (3, Verdict::Grew),
                (4, Verdict::Grew),
            ],
        );
        ran(
            &run,
            "H2",
            &[1, 2, 3, 4],
            "b",
            "S2",
            &[(1, Verdict::NotLoaded)],
        );
        let view = run.reload();
        // Two different drivers missed F1, but neither harness is focused (4 targets): pending.
        assert_eq!(pending_of(&view), ["F1", "F5"]);
        let q = queues(&view, None).remove(0);
        assert_eq!(
            q.missed.keys().map(ToString::to_string).collect::<Vec<_>>(),
            ["F1"]
        );
        assert_eq!(q.missed[&FindingId::new(1)].harness, "H1");
        assert!(q.focus);
    }

    #[test]
    fn a_focus_batch_is_chosen_when_the_best_ranked_pending_candidate_was_missed() {
        let (run, _) = Run::new(7);
        // F1, F3, F4 and F5 are missed (a constant, a not_loaded, an unknown and a
        // not_executed run); F2 grew; F6 and F7 are fresh.
        ran(
            &run,
            "H1",
            &[1, 2, 3, 4, 5],
            "a",
            "S1",
            &[
                (1, Verdict::Constant),
                (2, Verdict::Grew),
                (3, Verdict::NotLoaded),
                (4, Verdict::Unknown),
                (5, Verdict::NotExecuted),
            ],
        );
        let q = queues(&run.reload(), None).remove(0);
        assert_eq!(
            pending_of(&run.reload()),
            ["F1", "F3", "F4", "F5", "F6", "F7"]
        );
        assert!(q.focus);
        // The best-ranked missed one and the next two missed ones by rank: no fresh F6, no
        // reached F2, and no sibling expansion; F5 is over the cap of three.
        assert_eq!(
            q.batch,
            [FindingId::new(1), FindingId::new(3), FindingId::new(4)]
        );
        assert!(q.siblings.is_empty());
        let item = crate::view::queue_item(&q);
        assert!(
            item.description.contains("Next batch (focus): F1, F3, F4.")
                && item.description.contains("Missed once: F1, F3, F4, F5."),
            "{}",
            item.description
        );
        // `--next 8` is capped at three for a focus batch.
        let b = q.batch_for(&run.reload(), 8);
        assert!(b.focus && b.ids.len() == 3, "{b:?}");
    }

    #[test]
    fn a_broad_batch_holds_fresh_candidates_only() {
        let (run, _) = Run::new(4);
        // F2 is missed but F1, the best-ranked pending, is fresh: a broad batch of the fresh
        // candidates; the missed one waits for its turn at the head.
        ran(
            &run,
            "H1",
            &[2, 3, 4, 1],
            "a",
            "S1",
            &[
                (2, Verdict::Constant),
                (3, Verdict::Grew),
                (4, Verdict::Grew),
            ],
        );
        // F1 is declared by H1 but its verdict was not read: fresh.
        let q = queues(&run.reload(), None).remove(0);
        assert_eq!(pending_of(&run.reload()), ["F1", "F2"]);
        assert!(!q.focus);
        assert_eq!(q.batch, [FindingId::new(1)]);
        assert!(q.missed.contains_key(&FindingId::new(2)));
    }

    #[test]
    fn the_focus_batch_prefers_the_same_owner_then_the_same_file_then_the_rank() {
        use crate::census::Lifetime::Object;
        let items = [
            ("pkg.m0.C0.x", "pkg/m0.py:3", Object, "put"),
            ("pkg.m1.C1.x", "pkg/m1.py:3", Object, "put"),
            ("pkg.m0.D0.z", "pkg/m0.py:5", Object, "put"),
            ("pkg.m0.C0.y", "pkg/m0.py:4", Object, "put"),
        ];
        let (run, _) = owned_run(&items);
        ran(
            &run,
            "H1",
            &[1, 2, 3, 4],
            "a",
            "S1",
            &[
                (1, Verdict::Constant),
                (2, Verdict::Constant),
                (3, Verdict::Constant),
                (4, Verdict::Constant),
            ],
        );
        let q = queues(&run.reload(), None).remove(0);
        // F4 shares F1's owner, F3 its file, F2 only the rank: F2 is left out.
        assert_eq!(
            q.batch,
            [FindingId::new(1), FindingId::new(4), FindingId::new(3)]
        );
        assert!(q.focus && q.siblings.is_empty());
    }

    #[test]
    fn one_focused_miss_after_a_broad_miss_settles_and_the_same_driver_twice_does_not() {
        let (run, _) = Run::new(2);
        ran(
            &run,
            "H1",
            &[1, 2, 3, 4],
            "a",
            "S1",
            &[(1, Verdict::Constant), (2, Verdict::Constant)],
        );
        assert_eq!(pending_of(&run.reload()), ["F1", "F2"]);
        // A focused harness with the same driver text again: the same driver key, no settle.
        ran(&run, "H2", &[1], "a", "S2", &[(1, Verdict::NotLoaded)]);
        // (H2 and H1 differ in digest only when their driver files differ: the key is a digest
        // of the driver directory of each harness, so the same text is the same key.)
        assert_eq!(pending_of(&run.reload()), ["F1", "F2"]);
        // A different driver in a focused harness settles F1 as not reached; F2 stays.
        ran(&run, "H3", &[1], "c", "S3", &[(1, Verdict::Constant)]);
        let view = run.reload();
        assert_eq!(pending_of(&view), ["F2"]);
        let ix = Index::of(&view);
        assert_eq!(
            ix.standing(&FindingId::new(1)),
            Standing::Settled(Why::NotReached)
        );
        assert_eq!(ix.not_reached_in(&FindingId::new(1)).as_deref(), Some("H2"));
    }

    #[test]
    fn a_function_that_ran_without_its_writer_lines_is_a_miss_that_a_focus_harness_settles() {
        assert!(is_miss(Verdict::WriterNotExecuted));
        let (run, _) = Run::new(2);
        ran(
            &run,
            "H1",
            &[1, 2, 3],
            "a",
            "S1",
            &[(1, Verdict::WriterNotExecuted), (2, Verdict::Executed)],
        );
        let view = run.reload();
        // F1 stays pending, missed once, and the focus batch is for it.
        assert_eq!(pending_of(&view), ["F1"]);
        let ix = Index::of(&view);
        let m = ix.missed_in(&FindingId::new(1)).unwrap();
        assert_eq!(m.verdict, Verdict::WriterNotExecuted);
        assert_eq!(m.harness, "H1");
        assert_eq!(ix.writer_miss(&FindingId::new(2)).map(|m| m.harness), None);
        let facts = ReportFacts::of(&view).unwrap();
        assert!(
            facts
                .line()
                .unwrap()
                .contains("missed once, no focus harness 1"),
            "{}",
            facts.line().unwrap()
        );
        // A focused harness with another driver that also did not run the lines settles it.
        ran(
            &run,
            "H2",
            &[1],
            "c",
            "S2",
            &[(1, Verdict::WriterNotExecuted)],
        );
        let view = run.reload();
        let ix = Index::of(&view);
        assert_eq!(
            ix.standing(&FindingId::new(1)),
            Standing::Settled(Why::NotReached)
        );
        assert_eq!(
            ix.writer_miss(&FindingId::new(1)).map(|m| m.harness),
            Some("H2".to_owned())
        );
        // It never counts as a measurement of the holder.
        assert!(
            !crate::trend::GrowthTarget {
                finding: FindingId::new(1),
                verdict: Verdict::WriterNotExecuted,
                declared: true,
            }
            .is_measurement()
        );
    }

    /// Writes the `features.json` of harness `h`: target F1 has a gap for each of `features`.
    fn gaps(run: &Run, h: &str, features: &[&str]) {
        let mut x = crate::features::HarnessFeatures {
            command: format!(
                "dla harness gen --lang rust --driver --features {} --probe F1",
                features.join(",")
            ),
            ..Default::default()
        };
        x.gaps.insert(
            "F1".into(),
            features
                .iter()
                .map(|f| crate::features::FeatureGap {
                    feature: (*f).to_owned(),
                    file: "src/span.rs".into(),
                    in_file: 25,
                    in_crate: 43,
                    in_entry: 0,
                    by_module: false,
                    default_on: false,
                })
                .collect(),
        );
        x.save(&run.dir.path().join("harness").join(h)).unwrap();
    }

    #[test]
    fn a_target_stays_open_while_a_feature_with_gated_sites_was_on_in_no_harness() {
        let (run, _) = Run::new(2);
        ran(
            &run,
            "H1",
            &[1, 2, 3, 4],
            "a",
            "S1",
            &[(1, Verdict::NotExecuted), (2, Verdict::NotExecuted)],
        );
        gaps(&run, "H1", &["enable"]);
        // A focused harness with another driver also reads not executed, still without the
        // feature: the rule that settles it as not reached does not apply yet.
        ran(&run, "H2", &[1], "c", "S2", &[(1, Verdict::NotExecuted)]);
        gaps(&run, "H2", &["enable"]);
        let view = run.reload();
        assert_eq!(pending_of(&view), ["F1", "F2"]);
        let ix = Index::of(&view);
        assert_eq!(ix.standing(&FindingId::new(1)), Standing::Pending);
        let (g, command) = ix.untried_features(&FindingId::new(1)).unwrap();
        assert_eq!(crate::features::names(&g), ["enable"]);
        assert!(
            command.contains("--features enable --probe F1"),
            "{command}"
        );
        // The queue line and the target's history say so, and lead with the feature.
        let facts = ReportFacts::of(&view).unwrap();
        assert!(
            facts
                .line()
                .unwrap()
                .contains("missed, a feature with gated sites not enabled 1"),
            "{}",
            facts.line().unwrap()
        );
        let t = facts.targeted(&FindingId::new(1)).unwrap();
        assert!(t.contains("; missed, and not settled: feature enable (25 gated sites in src/span.rs, 43 in the crate)"), "{t}");
        // A third harness that enabled the feature (no gap) and missed too: settled.
        ran(&run, "H3", &[1], "d", "S3", &[(1, Verdict::NotExecuted)]);
        let view = run.reload();
        let ix = Index::of(&view);
        assert!(ix.untried_features(&FindingId::new(1)).is_none());
        assert_eq!(
            ix.standing(&FindingId::new(1)),
            Standing::Settled(Why::NotReached)
        );
        assert_eq!(pending_of(&view), ["F2"]);
    }

    #[test]
    fn a_feature_that_an_earlier_harness_had_on_does_not_keep_the_target_open() {
        let (run, _) = Run::new(1);
        // H1 was built with the feature (no gap recorded) and missed; H2, a focused harness
        // with another driver, was built without it and missed too: the feature was tried.
        ran(&run, "H1", &[1], "a", "S1", &[(1, Verdict::NotExecuted)]);
        ran(&run, "H2", &[1], "c", "S2", &[(1, Verdict::NotExecuted)]);
        gaps(&run, "H2", &["enable"]);
        let ix = Index::of(&run.reload());
        assert!(ix.untried_features(&FindingId::new(1)).is_none());
        assert_eq!(
            ix.standing(&FindingId::new(1)),
            Standing::Settled(Why::NotReached)
        );
    }

    #[test]
    fn an_entry_path_miss_is_not_a_miss_of_the_queue_and_a_reached_target_is_never_missed() {
        let (run, _) = Run::new(2);
        let mut m = manifest("H1", &[1], true);
        m.purpose = Some(crate::harness::HarnessPurpose::EntryPath);
        run.harness(&m);
        run.series("S1", "H1");
        run.trend("T1", "S1", &[(1, Verdict::Constant)]);
        let ix = Index::of(&run.reload());
        assert!(ix.missed_in(&FindingId::new(1)).is_none());
        ran(&run, "H2", &[2], "a", "S2", &[(2, Verdict::Executed)]);
        let ix = Index::of(&run.reload());
        assert_eq!(
            ix.standing(&FindingId::new(2)),
            Standing::Settled(Why::Executed)
        );
        assert!(ix.missed_in(&FindingId::new(2)).is_none());
    }

    /// Like `ran` for a second-look harness.
    fn looked(run: &Run, h: &str, targets: &[u32], driver: &str, s: &str, v: &[(u32, Verdict)]) {
        let mut m = manifest(h, targets, true);
        m.purpose = Some(crate::harness::HarnessPurpose::SecondLook);
        run.harness(&m);
        let d = run.dir.path().join("harness").join(h).join("driver");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(d.join("dla_harness.py"), driver).unwrap();
        run.series(s, h);
        let digest = crate::harness::driver_digest(run.dir.path(), h).unwrap();
        std::fs::write(
            run.dir
                .path()
                .join("harness")
                .join(h)
                .join(format!("run-{s}.json")),
            json!({"series": [s], "driver_digest": digest}).to_string(),
        )
        .unwrap();
        run.trend(&format!("T{}", &s[1..]), s, v);
    }

    #[test]
    fn one_flat_run_leaves_the_candidates_for_a_second_look_when_the_first_pass_is_over() {
        let (run, _) = Run::new(5);
        // H1 (5 targets) read F1..F4 `executed`; F5 was never declared: the first pass is
        // not over, so there is no second look yet.
        ran(
            &run,
            "H1",
            &[1, 2, 3, 4],
            "a",
            "S1",
            &[
                (1, Verdict::Executed),
                (2, Verdict::Executed),
                (3, Verdict::Executed),
                (4, Verdict::Executed),
            ],
        );
        let q = queues(&run.reload(), None).remove(0);
        assert_eq!(pending_of(&run.reload()), ["F5"]);
        assert!(q.second.is_empty() && !q.focus);
        // F5 read flat too: the first pass is over, the second look starts, best-ranked first,
        // three to a harness.
        ran(&run, "H2", &[5], "b", "S2", &[(5, Verdict::Executed)]);
        let view = run.reload();
        let q = queues(&view, None).remove(0);
        assert!(q.open());
        assert_eq!(pending_of(&view), ["F1", "F2", "F3", "F4", "F5"]);
        assert_eq!(
            q.batch,
            [FindingId::new(1), FindingId::new(2), FindingId::new(3)]
        );
        assert!(q.focus && q.second.len() == 5 && q.missed.is_empty());
        assert_eq!(q.second[&FindingId::new(1)].harness, "H1");
        assert!(q.batch_for(&view, 8).second);
        assert!(steps(&q, "R").contains("The next batch is a second look"));
        // F5's single run was a focused harness, but its own driver is the only one.
        let item = crate::view::queue_item(&q);
        assert!(
            item.description.contains("Next batch (second look"),
            "{}",
            item.description
        );
    }

    #[test]
    fn a_second_look_ends_and_settles_after_a_different_driver() {
        let (run, _) = Run::new(2);
        ran(
            &run,
            "H1",
            &[1, 2, 3, 4],
            "a",
            "S1",
            &[(1, Verdict::Executed), (2, Verdict::Executed)],
        );
        assert_eq!(pending_of(&run.reload()), ["F1", "F2"]);
        // The second look runs: a focused harness with another driver, flat again.
        looked(
            &run,
            "H2",
            &[1, 2],
            "b",
            "S2",
            &[(1, Verdict::Executed), (2, Verdict::Executed)],
        );
        let view = run.reload();
        assert!(pending_of(&view).is_empty());
        assert_eq!(queues(&view, None)[0].closed, Some(Closed::Empty));
        let ix = Index::of(&view);
        assert_eq!(
            ix.second_look(&FindingId::new(1)),
            SecondLook::Done("H2".into())
        );
        assert_eq!(
            ReportFacts::of(&view).unwrap().line().unwrap(),
            "2 of 2 ranked candidates targeted in 2 harnesses: executed 2; 0 not targeted; second look 2 (grew 0)"
        );
    }

    #[test]
    fn the_second_look_is_bounded_and_skips_grew_missed_and_floor() {
        // Ten candidates, all flat under H1: nine get a look (three harnesses of three), the
        // tenth is settled.
        let (run, _) = Run::new(10);
        let all: Vec<u32> = (1..=10).collect();
        let flat: Vec<(u32, Verdict)> = all.iter().map(|n| (*n, Verdict::Executed)).collect();
        ran(&run, "H1", &all[..8], "a", "S1", &flat[..8]);
        ran(&run, "H2", &all[8..], "b", "S2", &flat[8..]);
        let view = run.reload();
        assert_eq!(pending_of(&view).len(), 9);
        assert!(!pending_of(&view).contains(&"F10".to_owned()));
        // Two second-look harnesses have been generated (any state): room for one more look.
        let mut m = manifest("H3", &[1, 2, 3], true);
        m.purpose = Some(crate::harness::HarnessPurpose::SecondLook);
        run.harness(&m);
        let mut m = manifest("H4", &[4, 5, 6], true);
        m.purpose = Some(crate::harness::HarnessPurpose::SecondLook);
        run.harness(&m);
        // H3 and H4 have not run: their candidates stay pending (in flight), and F7..F9 are
        // the last three that may get a look.
        assert_eq!(pending_of(&run.reload()).len(), 9);
        let mut m = manifest("H5", &[7, 8, 9], true);
        m.purpose = Some(crate::harness::HarnessPurpose::SecondLook);
        run.harness(&m);
        // Three second-look harnesses exist: no candidate beyond the ones they declare.
        assert_eq!(pending_of(&run.reload()).len(), 9);
        // Grew and missed ones are no candidates for a look.
        let (run, _) = Run::new(3);
        ran(
            &run,
            "H1",
            &[1, 2, 3, 4],
            "a",
            "S1",
            &[
                (1, Verdict::Executed),
                (2, Verdict::Grew),
                (3, Verdict::Constant),
            ],
        );
        // F3 was missed: it is first-pass work, so the second look waits (rule 4); F2 grew and
        // is never part of it.
        let q = queues(&run.reload(), None).remove(0);
        assert_eq!(pending_of(&run.reload()), ["F3"]);
        assert!(q.second.is_empty() && q.focus);
        let (run, _) = Run::new(2);
        ran(
            &run,
            "H1",
            &[1, 2, 3, 4],
            "a",
            "S1",
            &[(1, Verdict::Executed), (2, Verdict::Grew)],
        );
        let q = queues(&run.reload(), None).remove(0);
        assert_eq!(pending_of(&run.reload()), ["F1"]);
        assert!(q.second.contains_key(&FindingId::new(1)));
        // At the floor nothing is asked: the queue is closed and has no second look.
        let (run, _) = Run::with_budget(2, 60);
        ran(
            &run,
            "H1",
            &[1, 2, 3, 4],
            "a",
            "S1",
            &[(1, Verdict::Executed), (2, Verdict::Executed)],
        );
        let q = queues(&run.reload(), None).remove(0);
        assert!(
            q.second.is_empty() && q.pending.is_empty(),
            "{:?}",
            q.closed
        );
    }

    #[test]
    fn a_go_site_names_its_package_directory() {
        assert_eq!(go_package_dir("health/server.go:49"), "health");
        assert_eq!(go_package_dir("./a/b/c.go:7"), "a/b");
        assert_eq!(go_package_dir("main.go:3"), "");
        assert_eq!(go_package_dir("a\\b.go:3"), "a");
        assert_eq!(package_label(""), ".");
        assert_eq!(package_label("health"), "health");
    }

    #[test]
    fn a_batch_with_siblings_never_passes_sixteen() {
        use crate::census::Lifetime::Object;
        let mut items = vec![("pkg.m.C.f0", "pkg/m.py:1", Object, "put")];
        let owners: Vec<String> = (1..30).map(|i| format!("pkg.m.C.f{i}")).collect();
        let sites: Vec<String> = (1..30).map(|i| format!("pkg/m.py:{}", i + 1)).collect();
        for i in 0..29 {
            items.push((owners[i].as_str(), sites[i].as_str(), Object, "put"));
        }
        let (_run, view) = owned_run(&items);
        let q = &queues(&view, None)[0];
        assert_eq!(q.batch.len(), MAX_BATCH_WITH_SIBLINGS);
        assert_eq!(q.siblings.len(), MAX_BATCH_WITH_SIBLINGS - MAX_BATCH);
    }
    /// A JVM run: a project directory with a `mvnw`, the given test files (project-relative
    /// path, source), and census candidates whose sites are `src/main/java/a/<Owner>.java:3`
    /// (F1 for the first owner, and so on). The workload discovery result lists every test
    /// file as a Maven test workload.
    fn jvm_run(tests: &[(&str, &str)], owners: &[&str]) -> (Run, tempfile::TempDir) {
        let project = tempfile::tempdir().unwrap();
        std::fs::write(project.path().join("mvnw"), "#!/bin/sh\n").unwrap();
        let mut workloads = Vec::new();
        for (file, text) in tests {
            let path = project.path().join(file);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, text).unwrap();
            workloads.push(json!({
                "id": format!("jvm:test:{file}"), "kind": "test", "runtime": "jvm",
                "command": "mvn -q -o test", "cwd": ".", "file": file, "rank": 1,
            }));
        }
        let dir = tempfile::tempdir().unwrap();
        let st = RunState::init(
            project.path().to_str().unwrap(),
            None,
            3600,
            None,
            now_utc(),
        )
        .unwrap();
        st.save(dir.path()).unwrap();
        std::fs::create_dir_all(dir.path().join("workload")).unwrap();
        std::fs::write(
            dir.path().join("workload").join("workloads.json"),
            json!({"schema_version": 1, "workloads": workloads, "entry_points": []}).to_string(),
        )
        .unwrap();
        let mut l = Ledger::create(dir.path()).unwrap();
        l.add_evidence(evidence("jvm")).unwrap();
        for (i, owner) in owners.iter().enumerate() {
            let n = u32::try_from(i + 1).unwrap();
            let mut nf = NewFinding::new(
                Origin::parse("census").unwrap(),
                vec![crate::EvidenceId::new(1)],
            );
            let file = format!("src/main/java/a/{owner}.java");
            nf.site = Some(format!("{file}:3"));
            l.add_finding(nf).unwrap();
            l.annotate(
                &FindingId::new(n),
                vec![format!("{file}:9")],
                vec![],
                Some("n".into()),
            )
            .unwrap();
        }
        (Run { dir }, project)
    }

    const JUNIT5: &str =
        "import org.junit.jupiter.api.Test;\nclass X {\n@Test void a() {}\n@Test void b() {}\n}\n";

    fn jvm_queue(run: &Run) -> RuntimeQueue {
        queues(&run.reload(), None).remove(0)
    }

    /// A test-loop harness H<n> (driver mode, the launcher in its tools) declaring `targets`.
    fn test_loop_harness(run: &Run, h: &str, targets: &[u32]) {
        let mut m = manifest(h, targets, true);
        m.runtime = crate::runtime::Runtime::Jvm;
        m.tools.push(crate::harness::JUNIT_TOOL.to_owned());
        run.harness(&m);
    }

    fn read(run: &Run, h: &str, s: &str, v: &[(u32, Verdict)]) {
        run.series(s, h);
        run.trend(&format!("T{}", &s[1..]), s, v);
    }

    #[test]
    fn the_test_loop_is_the_first_pass_of_a_jvm_queue_and_the_driver_follows() {
        let (run, _p) = jvm_run(
            &[
                ("src/test/java/a/AaaTests.java", JUNIT5),
                (
                    "src/test/java/a/UsesCccTests.java",
                    &format!("{JUNIT5}// Ccc\n"),
                ),
            ],
            &["Aaa", "Bbb", "Ccc"],
        );
        let q = jvm_queue(&run);
        assert_eq!(q.batch_command, "dla harness gen --lang jvm --junit --next");
        assert_eq!(q.command, q.batch_command);
        assert_eq!(
            q.batch.iter().map(ToString::to_string).collect::<Vec<_>>(),
            ["F1", "F3"]
        );
        assert_eq!(q.test_loop.driver, [FindingId::new(2)]);
        assert_eq!(
            q.test_loop.sentence(),
            "2 candidates in 1 test-loop batch, nothing to write; then 1 candidate needs a driver"
        );
        // Everything that names the next step says so.
        let item = crate::view::queue_item(&q);
        assert!(
            item.description.contains(&q.test_loop.sentence()),
            "{}",
            item.description
        );
        assert!(item.fix.starts_with(&q.batch_command), "{}", item.fix);
        assert!(item.fix.contains("--driver --next"), "{}", item.fix);
        let steps = steps(&q, "r");
        assert!(
            steps.starts_with("2 candidates in 1 test-loop batch"),
            "{steps}"
        );
        assert!(
            steps.contains("--junit --next") && steps.contains("--driver --next"),
            "{steps}"
        );
        // The candidate with no test is told the driver command in its own finding item.
        let st = crate::view::status(&run.reload());
        let fix_of = |f: &str| {
            st.blocking
                .iter()
                .find(|b| b.id == f)
                .map(|b| b.fix.clone())
                .unwrap_or_default()
        };
        assert_eq!(fix_of("F2"), "dla harness gen --lang jvm --driver --next");
        assert_eq!(fix_of("F1"), "dla harness gen --lang jvm --junit --next");
    }

    #[test]
    fn a_project_without_a_test_the_loop_can_run_keeps_the_driver() {
        // No test files at all.
        let (run, _p) = jvm_run(&[], &["Aaa"]);
        let q = jvm_queue(&run);
        assert_eq!(
            q.batch_command,
            "dla harness gen --lang jvm --driver --next"
        );
        assert!(q.test_loop.batches.is_empty() && q.test_loop.sentence().is_empty());
        // A JUnit 4 test is not one the launcher runs.
        let (run, _p) = jvm_run(
            &[(
                "src/test/java/a/AaaTests.java",
                "import org.junit.Test;\n@Test void a() {}\n",
            )],
            &["Aaa"],
        );
        assert_eq!(
            jvm_queue(&run).batch_command,
            "dla harness gen --lang jvm --driver --next"
        );
    }

    #[test]
    fn without_a_build_tool_the_loop_is_not_offered() {
        let (run, p) = jvm_run(&[("src/test/java/a/AaaTests.java", JUNIT5)], &["Aaa"]);
        assert_eq!(jvm_queue(&run).test_loop.batches.len(), 1);
        std::fs::remove_file(p.path().join("mvnw")).unwrap();
        // `mvn` may be on the PATH of the machine that runs this test: then it is offered.
        if !crate::testloop::on_path("mvn") {
            assert!(jvm_queue(&run).test_loop.batches.is_empty());
        }
    }

    #[test]
    fn a_flat_test_loop_reading_is_one_reading_and_a_miss_is_a_miss() {
        for flat in [Verdict::Executed, Verdict::Constant] {
            let (run, _p) = jvm_run(&[("src/test/java/a/AaaTests.java", JUNIT5)], &["Aaa"]);
            test_loop_harness(&run, "H1", &[1]);
            read(&run, "H1", "S1", &[(1, flat)]);
            let q = jvm_queue(&run);
            // Settled by the first pass, then the second look by a hand-written harness.
            assert_eq!(q.settled, 0, "{flat:?}: the second look counts it pending");
            assert!(q.second.contains_key(&FindingId::new(1)), "{flat:?}");
            assert!(q.test_loop.batches.is_empty());
            assert_eq!(
                q.batch_command,
                "dla harness gen --lang jvm --driver --next"
            );
            assert!(q.focus && q.missed.is_empty());
        }
        for miss in [Verdict::NotLoaded, Verdict::NotExecuted, Verdict::Unknown] {
            let (run, _p) = jvm_run(&[("src/test/java/a/AaaTests.java", JUNIT5)], &["Aaa"]);
            test_loop_harness(&run, "H1", &[1]);
            read(&run, "H1", "S1", &[(1, miss)]);
            let q = jvm_queue(&run);
            assert!(q.missed.contains_key(&FindingId::new(1)), "{miss:?}");
            assert!(q.second.is_empty());
            // A focus batch by a hand-written harness, never the test loop again.
            assert!(q.focus && q.test_loop.batches.is_empty(), "{miss:?}");
            assert_eq!(
                q.batch_command,
                "dla harness gen --lang jvm --driver --next"
            );
        }
    }

    #[test]
    fn a_missed_candidate_is_told_the_command_that_gives_it_a_harness() {
        // Missed by a hook driver; its test class has not run: the test loop is its harness.
        let (run, _p) = jvm_run(&[("src/test/java/a/AaaTests.java", JUNIT5)], &["Aaa"]);
        ran(
            &run,
            "H1",
            &[1],
            "a driver",
            "S1",
            &[(1, Verdict::NotExecuted)],
        );
        let q = jvm_queue(&run);
        assert!(q.missed.contains_key(&FindingId::new(1)));
        assert_eq!(
            q.missed_command(&FindingId::new(1)),
            "dla harness gen --lang jvm --junit --next"
        );
        // Missed by the test loop: a driver.
        let (run, _p) = jvm_run(&[("src/test/java/a/AaaTests.java", JUNIT5)], &["Aaa"]);
        test_loop_harness(&run, "H1", &[1]);
        read(&run, "H1", "S1", &[(1, Verdict::NotLoaded)]);
        assert_eq!(
            jvm_queue(&run).missed_command(&FindingId::new(1)),
            "dla harness gen --lang jvm --driver --next"
        );
    }

    #[test]
    fn a_test_loop_miss_and_a_focus_miss_are_two_drivers_and_settle_the_candidate() {
        let (run, _p) = jvm_run(&[("src/test/java/a/AaaTests.java", JUNIT5)], &["Aaa"]);
        test_loop_harness(&run, "H1", &[1]);
        read(&run, "H1", "S1", &[(1, Verdict::NotLoaded)]);
        ran(
            &run,
            "H2",
            &[1],
            "a driver written for F1",
            "S2",
            &[(1, Verdict::NotExecuted)],
        );
        let view = run.reload();
        assert_eq!(
            Index::of(&view).standing(&FindingId::new(1)),
            Standing::Settled(Why::NotReached)
        );
    }

    #[test]
    fn a_grown_test_loop_target_settles_by_growth_as_before() {
        let (run, _p) = jvm_run(&[("src/test/java/a/AaaTests.java", JUNIT5)], &["Aaa"]);
        test_loop_harness(&run, "H1", &[1]);
        read(&run, "H1", "S1", &[(1, Verdict::Grew)]);
        let view = run.reload();
        assert_eq!(
            Index::of(&view).standing(&FindingId::new(1)),
            Standing::Settled(Why::Moved)
        );
    }

    #[test]
    fn a_generated_test_loop_harness_is_run_before_more_are_made() {
        let (run, _p) = jvm_run(
            &[
                ("src/test/java/a/AaaTests.java", JUNIT5),
                ("src/test/java/a/BbbTests.java", JUNIT5),
            ],
            &["Aaa", "Bbb"],
        );
        test_loop_harness(&run, "H1", &[1]);
        let q = jvm_queue(&run);
        assert!(q.in_flight_test_loop);
        assert_eq!(q.command, "dla harness run H1");
        // Its own candidate is told to run it, not to write a driver, while another test-loop
        // batch is still planned; the planned one is told the queue's command too.
        assert_eq!(q.command_for(&FindingId::new(1)), "dla harness run H1");
        assert_eq!(q.command_for(&FindingId::new(2)), "dla harness run H1");
        // The other candidate is still planned, so the sentence counts it.
        assert_eq!(q.test_loop.candidates(), 1);
        let item = crate::view::queue_item(&q);
        assert!(item.fix.starts_with("dla harness run H1"), "{}", item.fix);
        assert!(
            steps(&q, "r").starts_with("dla harness run H1"),
            "{}",
            steps(&q, "r")
        );
    }

    #[test]
    fn a_test_loop_harness_that_failed_is_not_run_again_and_its_targets_need_a_driver() {
        use crate::harness::{AttemptOutcome, HarnessAttempt};
        let (run, _p) = jvm_run(&[("src/test/java/a/AaaTests.java", JUNIT5)], &["Aaa"]);
        test_loop_harness(&run, "H1", &[1]);
        let attempt = |reason: &str| HarnessAttempt {
            harness: "H1".into(),
            started_at: HarnessAttempt::now(),
            finished_at: HarnessAttempt::now(),
            outcome: AttemptOutcome::Failed,
            reason: Some(reason.into()),
            message: None,
        };
        // A run refused for time says nothing about the tests: the harness is still to run.
        attempt("time_budget").save(run.dir.path()).unwrap();
        assert_eq!(jvm_queue(&run).command, "dla harness run H1");
        for reason in ["build_fails", "no_harness"] {
            attempt(reason).save(run.dir.path()).unwrap();
            let q = jvm_queue(&run);
            assert!(q.in_flight.is_none(), "{reason}");
            assert_eq!(q.pending, [FindingId::new(1)], "{reason}: not measured");
            assert!(q.test_loop.batches.is_empty(), "{reason}");
            assert_eq!(q.declarable, [FindingId::new(1)], "{reason}");
            assert_eq!(
                q.command, "dla harness gen --lang jvm --driver --next",
                "{reason}"
            );
        }
    }

    #[test]
    fn a_candidate_settled_by_a_tied_growth_is_not_counted_for_the_test_loop_in_the_report() {
        let (run, _p) = jvm_run(
            &[
                ("src/test/java/a/AaaTests.java", JUNIT5),
                ("src/test/java/a/BbbTests.java", JUNIT5),
                ("src/test/java/a/CccTests.java", JUNIT5),
            ],
            &["Aaa", "Bbb", "Ccc"],
        );
        // H1 declares F2 only; its trend ties a growth to F1, which no harness declared.
        test_loop_harness(&run, "H1", &[2]);
        read(
            &run,
            "H1",
            "S1",
            &[(1, Verdict::Grew), (2, Verdict::Executed)],
        );
        let line = ReportFacts::of(&run.reload()).unwrap().line().unwrap();
        assert!(
            line.contains("(1 in 1 test-loop batch, nothing to write;"),
            "{line}"
        );
    }

    #[test]
    fn the_report_line_splits_the_candidates_nobody_targeted() {
        let (run, _p) = jvm_run(
            &[("src/test/java/a/AaaTests.java", JUNIT5)],
            &["Aaa", "Bbb", "Ccc"],
        );
        let line = ReportFacts::of(&run.reload()).unwrap().line().unwrap();
        assert!(
            line.contains(
                "3 not targeted (1 in 1 test-loop batch, nothing to write; 2 need a driver)"
            ),
            "{line}"
        );
    }

    // -- The pytest loop: the first step of a Python queue -----------------------------------

    /// Lists `files` as Python test workloads (working directory `.`) of the run.
    fn py_tests(run: &Run, files: &[&str]) {
        let workloads: Vec<_> = files
            .iter()
            .map(|f| {
                json!({
                    "id": format!("python:test:{f}"), "kind": "test", "runtime": "python",
                    "command": format!("python3 -m pytest -q {f}"), "cwd": ".", "file": f,
                    "rank": 1,
                })
            })
            .collect();
        std::fs::create_dir_all(run.dir.path().join("workload")).unwrap();
        std::fs::write(
            run.dir.path().join("workload").join("workloads.json"),
            json!({"schema_version": 1, "workloads": workloads, "entry_points": []}).to_string(),
        )
        .unwrap();
    }

    /// A pytest-loop harness: a driver that lists the loop's tool and declares no target.
    fn pytest_harness(run: &Run, h: &str) {
        let mut m = manifest(h, &[], true);
        m.tools.push(crate::harness::PYTEST_TOOL.to_owned());
        run.harness(&m);
    }

    /// A pytest loop generated with `--tests`: the loop's tool and the narrowed marker.
    fn pytest_narrowed_harness(run: &Run, h: &str) {
        let mut m = manifest(h, &[], true);
        m.tools.push(crate::harness::PYTEST_TOOL.to_owned());
        m.tools
            .push(crate::harness::PYTEST_NARROWED_TOOL.to_owned());
        run.harness(&m);
    }

    fn failed_attempt(run: &Run, h: &str, reason: &str) {
        use crate::harness::{AttemptOutcome, HarnessAttempt};
        HarnessAttempt {
            harness: h.into(),
            started_at: HarnessAttempt::now(),
            finished_at: HarnessAttempt::now(),
            outcome: AttemptOutcome::Failed,
            reason: Some(reason.into()),
            message: None,
        }
        .save(run.dir.path())
        .unwrap();
    }

    fn py_queue(run: &Run) -> RuntimeQueue {
        queues(&run.reload(), None).remove(0)
    }

    #[test]
    fn an_open_python_queue_with_test_files_starts_with_the_pytest_loop() {
        let (run, _) = Run::new(2);
        py_tests(&run, &["tests/test_a.py", "tests/test_b.py"]);
        let q = py_queue(&run);
        assert!(q.open());
        assert_eq!(q.command, "dla harness gen --lang python --pytest");
        assert_eq!(q.py_loop.as_deref(), Some(q.command.as_str()));
        // The loop declares no candidate: the batch and the candidates' own command stay a driver.
        assert_eq!(
            q.batch_command,
            "dla harness gen --lang python --driver --next"
        );
        assert_eq!(q.command_for(&FindingId::new(1)), q.batch_command);
        assert_eq!(q.batch.len(), 2);
        let text = steps(&q, "r");
        assert!(
            text.starts_with("First the project's own tests in a loop, nothing to write: dla harness gen --lang python --pytest, dla harness run <H>, dla trend S<n>; a line of the project that grows there gets a finding (the trend prints the command)"),
            "{text}"
        );
        assert!(
            text.contains("dla harness gen --lang python --driver --next"),
            "{text}"
        );
        let item = crate::view::queue_item(&q);
        assert!(
            item.fix
                .starts_with("dla harness gen --lang python --pytest, dla harness run <H>"),
            "{}",
            item.fix
        );
        // It is an order, not a gate item of its own: one queue item, the fix is its text.
        assert_eq!(item.kind, crate::view::BlockingKind::Queue);
    }

    #[test]
    fn a_python_queue_without_test_files_never_shows_the_loop() {
        let (run, view) = Run::new(2);
        let q = &queues(&view, None)[0];
        assert!(q.py_loop.is_none());
        assert_eq!(q.command, q.batch_command);
        // Discovery lists a main and no test file.
        std::fs::create_dir_all(run.dir.path().join("workload")).unwrap();
        std::fs::write(
            run.dir.path().join("workload").join("workloads.json"),
            json!({"schema_version": 1, "workloads": [{
                "id": "python:main:run.py", "kind": "main", "runtime": "python",
                "command": "python3 run.py", "cwd": ".", "file": "run.py", "rank": 1
            }], "entry_points": []})
            .to_string(),
        )
        .unwrap();
        let q = py_queue(&run);
        assert!(q.py_loop.is_none() && q.command == q.batch_command);
        assert!(!steps(&q, "r").contains("First the project's own tests"));
    }

    #[test]
    fn a_closed_or_blocked_python_queue_never_shows_the_loop() {
        let (run, view) = Run::new(2);
        py_tests(&run, &["tests/test_a.py"]);
        let mut off = run.reload();
        off.state.queue = false;
        let q = &queues(&off, None)[0];
        assert_eq!(q.closed, Some(Closed::Off));
        assert!(q.py_loop.is_none());
        let mut late = run.reload();
        late.state.budget.deadline = crate::state::now_utc() - chrono::Duration::minutes(1);
        let q = &queues(&late, None)[0];
        assert_eq!(q.closed, Some(Closed::Floor));
        assert!(q.py_loop.is_none());
        // Every candidate settled: the queue is empty.
        run.harness(&manifest("H1", &[1, 2], true));
        run.series("S1", "H1");
        run.trend("T1", "S1", &[(1, Verdict::Grew), (2, Verdict::Grew)]);
        let q = py_queue(&run);
        assert_eq!(q.closed, Some(Closed::Empty));
        assert!(q.py_loop.is_none());
        // A blocker recorded for the runtime closes it.
        let (run, view2) = Run::new(2);
        py_tests(&run, &["tests/test_a.py"]);
        let mut l = Ledger::load(run.dir.path()).unwrap();
        l.answer_checklist(crate::checklist::ChecklistAnswer {
            item: "intake.workload".into(),
            status: crate::checklist::AnswerStatus::Unresolved,
            evidence: Vec::new(),
            note: None,
            reason: Some(crate::finding::UnresolvedReason::EnvUnavailable),
            remedy: Some("a service".into()),
            runtime: Some("python".into()),
        })
        .unwrap();
        let q = py_queue(&run);
        assert_eq!(q.closed, Some(Closed::Blocked));
        assert!(q.py_loop.is_none());
        let _ = (view, view2);
    }

    #[test]
    fn the_loop_is_first_until_a_pytest_loop_wrote_a_series_and_then_the_driver_batch_follows() {
        let (run, _) = Run::new(2);
        py_tests(&run, &["tests/test_a.py"]);
        // Generated, not run: run it.
        pytest_harness(&run, "H1");
        let q = py_queue(&run);
        assert_eq!(q.command, "dla harness run H1");
        assert_eq!(q.py_loop.as_deref(), Some("dla harness run H1"));
        assert!(!q.in_flight_test_loop, "it declares no target");
        assert!(steps(&q, "r").starts_with("First the project's own tests in a loop, nothing to write: dla harness run H1, dla trend S<n>;"));
        assert!(
            crate::view::queue_item(&q)
                .fix
                .starts_with("dla harness run H1, dla trend S<n>"),
            "{}",
            crate::view::queue_item(&q).fix
        );
        // A run refused for time says nothing about the tests: still to run.
        use crate::harness::{AttemptOutcome, HarnessAttempt};
        let attempt = |reason: &str| HarnessAttempt {
            harness: "H1".into(),
            started_at: HarnessAttempt::now(),
            finished_at: HarnessAttempt::now(),
            outcome: AttemptOutcome::Failed,
            reason: Some(reason.into()),
            message: None,
        };
        attempt("time_budget").save(run.dir.path()).unwrap();
        assert_eq!(py_queue(&run).command, "dla harness run H1");
        // It wrote a series: the driver batch, exactly as without the loop.
        run.series("S1", "H1");
        let q = py_queue(&run);
        assert!(q.py_loop.is_none());
        assert_eq!(q.command, "dla harness gen --lang python --driver --next");
        assert_eq!(q.command, q.batch_command);
        assert!(!steps(&q, "r").contains("First the project's own tests"));
        // The loop's series settles no candidate: both are still pending.
        assert_eq!(q.pending.len(), 2);
    }

    #[test]
    fn a_failed_attempt_of_the_loop_lets_the_queue_go_on() {
        use crate::harness::{AttemptOutcome, HarnessAttempt};
        let (run, _) = Run::new(2);
        py_tests(&run, &["tests/test_a.py"]);
        pytest_harness(&run, "H1");
        for reason in ["harness-failed", "no_harness", "build_fails"] {
            HarnessAttempt {
                harness: "H1".into(),
                started_at: HarnessAttempt::now(),
                finished_at: HarnessAttempt::now(),
                outcome: AttemptOutcome::Failed,
                reason: Some(reason.into()),
                message: None,
            }
            .save(run.dir.path())
            .unwrap();
            let q = py_queue(&run);
            assert!(q.py_loop.is_none(), "{reason}");
            assert_eq!(
                q.command, "dla harness gen --lang python --driver --next",
                "{reason}"
            );
        }
    }

    #[test]
    fn a_refused_generation_is_a_failed_attempt_and_a_later_generation_still_counts() {
        let (run, _) = Run::new(2);
        py_tests(&run, &["tests/test_a.py"]);
        assert_eq!(
            py_queue(&run).command,
            "dla harness gen --lang python --pytest"
        );
        crate::harness::HarnessAttempt::save_pytest_loop_refusal(
            run.dir.path(),
            "tool_missing",
            "pytest does not import",
        )
        .unwrap();
        let q = py_queue(&run);
        assert!(q.py_loop.is_none());
        assert_eq!(q.command, "dla harness gen --lang python --driver --next");
        // Pytest was installed since and the loop generated: it is to run, and then it counts.
        pytest_harness(&run, "H1");
        assert_eq!(py_queue(&run).command, "dla harness run H1");
        run.series("S1", "H1");
        let q = py_queue(&run);
        assert!(q.py_loop.is_none());
        assert_eq!(q.command, q.batch_command);
    }

    #[test]
    fn a_narrowed_loop_that_wrote_a_series_leaves_the_step_at_the_full_loop() {
        let (run, _) = Run::new(2);
        py_tests(&run, &["tests/test_a.py", "tests/test_b.py"]);
        // No full loop yet: the step is to generate it.
        pytest_narrowed_harness(&run, "H1");
        run.series("S1", "H1");
        let q = py_queue(&run);
        assert_eq!(
            q.py_loop.as_deref(),
            Some("dla harness gen --lang python --pytest")
        );
        assert_eq!(q.command, "dla harness gen --lang python --pytest");
        // The full loop is generated and not run: the step is to run it.
        pytest_harness(&run, "H2");
        let q = py_queue(&run);
        assert_eq!(q.py_loop.as_deref(), Some("dla harness run H2"));
        assert_eq!(q.command, "dla harness run H2");
        assert!(
            crate::view::queue_item(&q)
                .fix
                .starts_with("dla harness run H2")
        );
        // The full loop ran: the queue goes on with the driver batches.
        run.series("S2", "H2");
        let q = py_queue(&run);
        assert!(q.py_loop.is_none());
        assert_eq!(q.command, q.batch_command);
    }

    #[test]
    fn a_full_loop_that_wrote_a_series_ends_the_step_whatever_a_narrowed_loop_did() {
        let (run, _) = Run::new(2);
        py_tests(&run, &["tests/test_a.py"]);
        pytest_harness(&run, "H1");
        run.series("S1", "H1");
        assert!(py_queue(&run).py_loop.is_none());
        pytest_narrowed_harness(&run, "H2");
        assert!(py_queue(&run).py_loop.is_none());
    }

    #[test]
    fn a_failed_full_loop_ends_the_step_and_a_narrowed_loop_after_it_changes_nothing() {
        let (run, _) = Run::new(2);
        py_tests(&run, &["tests/test_a.py"]);
        pytest_harness(&run, "H1");
        failed_attempt(&run, "H1", "harness-failed");
        assert!(py_queue(&run).py_loop.is_none());
        // The agent's own fallback: a narrowed loop, generated, run, failed.
        pytest_narrowed_harness(&run, "H2");
        let q = py_queue(&run);
        assert!(q.py_loop.is_none());
        assert_eq!(q.command, "dla harness gen --lang python --driver --next");
        run.series("S1", "H2");
        assert!(py_queue(&run).py_loop.is_none());
        failed_attempt(&run, "H2", "no_harness");
        assert!(py_queue(&run).py_loop.is_none());
        // A refused generation is the full loop's failed attempt too.
        let (run, _) = Run::new(2);
        py_tests(&run, &["tests/test_a.py"]);
        crate::harness::HarnessAttempt::save_pytest_loop_refusal(
            run.dir.path(),
            "tool_missing",
            "pytest does not import",
        )
        .unwrap();
        pytest_narrowed_harness(&run, "H1");
        assert!(py_queue(&run).py_loop.is_none());
    }

    #[test]
    fn a_narrowed_loop_whose_run_failed_leaves_the_step_at_the_full_loop() {
        let (run, _) = Run::new(2);
        py_tests(&run, &["tests/test_a.py", "tests/test_b.py"]);
        pytest_narrowed_harness(&run, "H1");
        failed_attempt(&run, "H1", "harness-failed");
        let q = py_queue(&run);
        assert_eq!(
            q.py_loop.as_deref(),
            Some("dla harness gen --lang python --pytest")
        );
        pytest_harness(&run, "H2");
        let q = py_queue(&run);
        assert_eq!(q.py_loop.as_deref(), Some("dla harness run H2"));
    }

    #[test]
    fn the_pytest_loop_is_a_test_loop_and_not_a_hook_driver() {
        let mut m = manifest("H1", &[], true);
        assert!(m.is_hook_driver() && !m.is_test_loop() && !m.is_pytest_loop());
        m.tools.push(crate::harness::PYTEST_TOOL.to_owned());
        assert!(m.is_test_loop() && m.is_pytest_loop() && !m.is_hook_driver());
        let mut j = manifest("H2", &[], true);
        j.tools.push(crate::harness::JUNIT_TOOL.to_owned());
        assert!(j.is_test_loop() && !j.is_pytest_loop() && !j.is_hook_driver());
        // A command-mode harness is neither.
        let mut c = manifest("H3", &[], false);
        c.tools.push(crate::harness::PYTEST_TOOL.to_owned());
        assert!(!c.is_test_loop() && !c.is_hook_driver());
        // `--tests` marks the loop as narrowed; it stays a pytest loop.
        assert!(!m.is_pytest_narrowed());
        m.tools
            .push(crate::harness::PYTEST_NARROWED_TOOL.to_owned());
        assert!(m.is_pytest_narrowed() && m.is_pytest_loop());
        assert!(!j.is_pytest_narrowed());
    }

    #[test]
    fn the_python_test_files_are_those_of_the_working_directory_with_the_most() {
        let report: crate::workload::WorkloadReport = serde_json::from_value(json!({
            "schema_version": 1,
            "workloads": [
                {"id": "a", "kind": "test", "runtime": "python", "command": "x", "cwd": ".", "file": "tests/test_a.py", "rank": 1},
                {"id": "b", "kind": "test", "runtime": "python", "command": "x", "cwd": "svc", "file": "svc/tests/test_b.py", "rank": 1},
                {"id": "c", "kind": "test", "runtime": "python", "command": "x", "cwd": "svc", "file": "svc/tests/test_c.py", "rank": 1},
                {"id": "d", "kind": "main", "runtime": "python", "command": "x", "cwd": ".", "file": "run.py", "rank": 1},
                {"id": "e", "kind": "test", "runtime": "jvm", "command": "x", "cwd": ".", "file": "T.java", "rank": 1},
            ],
            "entry_points": []
        }))
        .unwrap();
        let t = crate::testloop::python_tests(&report).unwrap();
        assert_eq!(t.cwd, "svc");
        assert_eq!(t.files, ["svc/tests/test_b.py", "svc/tests/test_c.py"]);
        assert_eq!(t.elsewhere, [(".".to_owned(), 1)]);
        assert_eq!(t.local("svc/tests/test_b.py"), "tests/test_b.py");
    }
}
