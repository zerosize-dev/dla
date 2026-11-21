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
    /// The driver digest of the run, or the harness id when none was recorded.
    driver_key: String,
    verdict: Option<Verdict>,
    /// The reason text the trend gave for the verdict.
    detail: Option<String>,
    /// The harness declares at most [`FOCUS_MAX`] targets.
    focused: bool,
    /// An entry-path harness: it has its own rules, so it is no run of the queue's rule.
    entry: bool,
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
}

/// The most targets of a focus harness (and so of a FOCUSED harness).
pub const FOCUS_MAX: usize = 3;

/// A verdict that says a harness run did not reach the target: it ran, its trend was read and
/// nothing shows the target's code run (or its holder move).
pub fn is_miss(v: Verdict) -> bool {
    matches!(
        v,
        Verdict::Constant | Verdict::NotLoaded | Verdict::NotExecuted | Verdict::Unknown
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
                let detail = report
                    .and_then(|r| r.targets.iter().find(|t| &t.finding == f))
                    .map(|t| t.detail.clone());
                ix.declared.entry(f.clone()).or_default().push(DeclaredRun {
                    harness: h.clone(),
                    hook_driver: m.is_hook_driver(),
                    driver_key: key.clone(),
                    verdict,
                    detail,
                    focused: m.targets.len() <= FOCUS_MAX,
                    entry: m.is_entry_path(),
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
        // `unknown` (the run had no coverage data) says nothing about the target's code: it
        // counts as a driver run for the two-drivers rule below, not as a settled target.
        if runs
            .iter()
            .any(|r| r.hook_driver && r.verdict == Some(Verdict::Executed))
        {
            return Standing::Settled(Why::Executed);
        }
        // Not reached: missed by two different drivers, at least one of the runs a focused
        // harness (at most FOCUS_MAX targets). Two big batches naming a target settle nothing.
        let missed = self.misses(id);
        let keys: std::collections::BTreeSet<&str> =
            missed.iter().map(|r| r.driver_key.as_str()).collect();
        if keys.len() >= 2 && missed.iter().any(|r| r.focused) {
            return Standing::Settled(Why::NotReached);
        }
        Standing::Pending
    }

    /// The runs of hook-driver harnesses (not entry-path) that missed `id`.
    fn misses(&self, id: &FindingId) -> Vec<&DeclaredRun> {
        self.declared
            .get(id)
            .map_or(&[][..], Vec::as_slice)
            .iter()
            .filter(|r| r.hook_driver && !r.entry && r.verdict.is_some_and(is_miss))
            .collect()
    }

    /// The first run that missed `id`; `None` when no run missed it.
    pub fn missed_in(&self, id: &FindingId) -> Option<Miss> {
        self.misses(id).first().and_then(|r| {
            Some(Miss {
                harness: r.harness.clone(),
                verdict: r.verdict?,
                detail: r.detail.clone().unwrap_or_default(),
            })
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
    /// The run is over: aborted, or stamped incomplete and not going on. Time may be left.
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

    /// The batch of `--next n`: the first `n` (at most [`MAX_BATCH`]) declarable candidates
    /// and the siblings of the owners they hold.
    pub fn batch_for(&self, view: &RunView, n: usize) -> Batch {
        let free: Vec<&FindingId> = self.declarable.iter().collect();
        let go = self.runtime == GO;
        // A Go batch is one package: the one of the best-ranked candidate when that one was
        // missed (a focus batch), else of the best-ranked fresh one.
        let package = go
            .then(|| {
                let first = free.first()?;
                if self.missed.contains_key(*first) {
                    package_of(view, first)
                } else {
                    let fresh: Vec<&FindingId> = free
                        .iter()
                        .filter(|f| !self.missed.contains_key(**f))
                        .copied()
                        .collect();
                    go_packages(view, &fresh).into_iter().next()
                }
            })
            .flatten();
        if go && package.is_none() {
            return Batch::default();
        }
        choose_batch(view, &free, &self.missed, n, package.as_deref(), go)
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
        choose_batch(view, &free, &self.missed, n, Some(package), true)
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
            !all.is_empty()
                && all.iter().all(|m| !m.is_hook_driver())
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
            let no_coverage: Vec<FindingId> = pending
                .iter()
                .filter(|p| ix.best_verdict(p) == Some(Verdict::Unknown))
                .cloned()
                .collect();
            let in_flight = ix.in_flight(&runtime, &pending);
            let free: Vec<&FindingId> = pending
                .iter()
                .filter(|p| {
                    in_flight
                        .as_ref()
                        .is_none_or(|h| !ix.declared_by.get(*p).is_some_and(|v| v.contains(h)))
                })
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
                ..RuntimeQueue::default()
            };
            let Batch {
                ids: batch,
                siblings,
                focus,
                package,
            } = chooser.batch_for(view, MAX_BATCH);
            let batch_command = batch_command(view, &runtime);
            let command = match (&in_flight, batch.is_empty()) {
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
                ) {
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
                no_coverage,
                declarable,
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

/// What a verdict `unknown` means for a target, in every text that explains its standing.
pub const NO_COVERAGE: &str = "the run has no coverage data, so whether its code ran is not known: run a second, different driver, or one with a size reading";

/// The full text of the guidance a refusal or an item gives for an open queue.
pub fn steps(q: &RuntimeQueue, run_id: &str) -> String {
    let _ = run_id;
    let focus = if q.focus {
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
    format!(
        "{}. Fill the hooks so each unit reaches the listed targets with a new key, dla harness run <H> --mode both, then dla trend S<n>. If no driver can be built or run for {}, record why: dla ledger resolve intake.workload --status unresolved --reason <env_unavailable|tool_missing|build_fails|privilege_declined> --remedy \"<text>\" (that closes the queue of the runtime).{unknown}{focus}",
        q.command, q.runtime
    )
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
}

impl ReportFacts {
    /// The facts of a view loaded from disk; `None` for a view without a run directory.
    pub fn of(view: &RunView) -> Option<Self> {
        view.run_dir.as_ref()?;
        let ranks = ranked(view)
            .into_iter()
            .map(|r| (r.id, (r.rank, r.tier)))
            .collect();
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
            index: Index::of(view),
            queue_on: view.state.queue,
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
                Some(Verdict::Constant | Verdict::NotExecuted | Verdict::NotLoaded) => {
                    // "Not reached" is only what the rule settled: missed by two drivers, one
                    // of them a focused harness. A miss with no focus harness is "missed once".
                    if self.index.standing(id) == Standing::Settled(Why::NotReached) {
                        missed += 1;
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
        Some(format!(
            "{targeted} of {} ranked candidates targeted{in_h}{verdicts}; {} not targeted",
            ranked.len(),
            ranked.len() - targeted
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
        let tail = match (self.index.not_reached_in(id), self.index.missed_in(id)) {
            (Some(h), _) if self.index.standing(id) == Standing::Settled(Why::NotReached) => {
                format!("; not reached (missed by two drivers, the focused harness {h} among them)")
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
        // A run stamped incomplete or aborted with most of its time left is over, not at the
        // floor.
        for status in [RunStatus::Incomplete, RunStatus::Aborted] {
            let mut over = view.clone();
            over.state.status = status;
            assert_eq!(queues(&over, None)[0].closed, Some(Closed::Over));
        }
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
}
