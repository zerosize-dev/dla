//! The trigger of a leak: can a caller of the public API make new keys reach the holder?
//!
//! A holder GROWS when new keys reach it (what the drivers of the measurement queue show);
//! a caller can MAKE new keys reach it (the trigger). A hook driver feeds keys straight into
//! the holder's writer, so growth under it says the holder is unbounded and nothing about who
//! feeds the keys. The trigger of a finding is a pure function of the run directory:
//!
//! * [`Trigger::Reproduced`]: the finding has the verdict `grew` in the trend of an
//!   entry-path harness (`dla harness gen --driver --entry`: units call public entry points
//!   only), or in a harness that runs the project's own workload (a fixed command of the
//!   project or a JUnit repeat: no hand-written hooks).
//! * [`Trigger::NotFound`]: at least two entry-path harnesses with different driver digests
//!   targeted it, ran and were read with `dla trend`, and it grew in none.
//! * [`Trigger::Unchecked`]: anything else (it grew only in drivers that are not entry-path).
//!
//! The leak-side findings in state `confirmed` or `strong` whose trigger is unchecked give an
//! item of kind `trigger` in the gate while the run was started with the queue on and more
//! than the floor of the budget is left ([`items`]).

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    finding::{Finding, FindingState, Polarity},
    gate,
    harness::{HarnessManifest, MAX_TARGETS},
    ids::FindingId,
    trend::Verdict,
    view::{BlockingItem, BlockingKind, RunView},
};

/// The trigger of one finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "trigger", rename_all = "snake_case")]
pub enum Trigger {
    /// A caller of the public API makes the holder grow.
    Reproduced {
        /// The harness that showed it.
        harness: String,
        /// What a caller varies per unit: the `VARIES.txt` line of an entry-path harness, or
        /// `the project's workload <id>`.
        varies: String,
    },
    /// Two different entry-path drivers did not make it grow.
    NotFound {
        /// The first two entry-path harnesses that ran without growth, with their `varies`.
        runs: Vec<(String, String)>,
    },
    /// The holder grew only when the harness fed it keys.
    Unchecked,
}

impl Trigger {
    /// `reproduced`, `not_found` or `unchecked`.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Reproduced { .. } => "reproduced",
            Self::NotFound { .. } => "not_found",
            Self::Unchecked => "unchecked",
        }
    }
}

/// One read series of an entry-path harness that targeted a finding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryRun {
    /// The harness.
    pub harness: String,
    /// The series.
    pub series: String,
    /// The driver digest of the run.
    pub digest: String,
    /// What a caller varies per unit, from the run record.
    pub varies: Option<String>,
    /// The finding's verdict in the trend of the series.
    pub verdict: Option<Verdict>,
}

/// One series of a harness, as far as a trigger reads it.
#[derive(Debug, Clone)]
struct Run {
    harness: String,
    series: String,
    entry: bool,
    /// A fixed command of the project or a JUnit repeat.
    own_workload: bool,
    workload: Option<String>,
    digest: String,
    varies: Option<String>,
    /// The trend of the series was read.
    read: bool,
    /// The series reuses no keys (a `repeated` series says nothing about new keys).
    fresh: bool,
    verdicts: BTreeMap<FindingId, Verdict>,
    declared: BTreeSet<FindingId>,
}

/// What the stored trend reports and harness records say about triggers.
#[derive(Debug, Default)]
pub struct Triggers {
    runs: Vec<Run>,
    /// Entry-path harnesses that declared the finding, series or not, oldest first.
    entry_declared: BTreeMap<FindingId, Vec<String>>,
    /// Harnesses that have at least one series.
    ran: BTreeSet<String>,
    /// The first writer function of each finding's entry-path targets.
    writer: BTreeMap<FindingId, String>,
    /// The harnesses that never ran, by finding, for the fix text.
    runtime: BTreeMap<String, String>,
}

impl Triggers {
    /// Reads the harness manifests and the trend reports of `view`. A view without a run
    /// directory has no triggers.
    pub fn of(view: &RunView) -> Self {
        let mut t = Self::default();
        let Some(run_dir) = view.run_dir.as_deref() else {
            return t;
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
            t.runtime.insert(h.clone(), m.runtime.as_str().to_owned());
            if m.is_entry_path() {
                for f in &m.targets {
                    t.entry_declared
                        .entry(f.clone())
                        .or_default()
                        .push(h.clone());
                }
                t.read_writers(run_dir, &h, &m);
            }
            manifests.insert(h, m);
        }
        let mut latest: BTreeMap<&str, &crate::trend::TrendReport> = BTreeMap::new();
        for r in &view.trends {
            latest.insert(r.series.as_str(), r);
        }
        for (series, h) in &view.series_harness {
            let Some(m) = manifests.get(h) else { continue };
            t.ran.insert(h.clone());
            let rec = crate::harness::run_record_of(run_dir, h, series);
            let report = latest.get(series.as_str());
            t.runs.push(Run {
                harness: h.clone(),
                series: series.clone(),
                entry: m.is_entry_path(),
                own_workload: !m.is_hook_driver() && !m.is_entry_path(),
                workload: m.workload.clone(),
                digest: rec
                    .as_ref()
                    .and_then(|r| r.driver_digest.clone())
                    .unwrap_or_else(|| h.clone()),
                varies: rec.and_then(|r| r.varies),
                read: report.is_some(),
                fresh: crate::series::SeriesFile::load(run_dir, series)
                    .ok()
                    .and_then(|f| f.source.mode)
                    != Some(crate::series::KeyMode::Repeated),
                verdicts: report
                    .map(|r| r.targets.iter().map(|v| (v.finding.clone(), v.verdict)).collect())
                    .unwrap_or_default(),
                declared: m.targets.iter().cloned().collect(),
            });
        }
        t
    }

    fn read_writers(&mut self, run_dir: &std::path::Path, h: &str, m: &HarnessManifest) {
        let path = HarnessManifest::dir_in(run_dir, h).join("targets.json");
        let Ok(text) = std::fs::read_to_string(path) else {
            return;
        };
        let Ok(doc) = serde_json::from_str::<serde_json::Value>(&text) else {
            return;
        };
        for t in doc
            .get("targets")
            .and_then(|t| t.as_array())
            .into_iter()
            .flatten()
        {
            let (Some(f), Some(w)) = (
                t.get("finding").and_then(|f| f.as_str()),
                t.get("writer_functions")
                    .and_then(|w| w.as_array())
                    .and_then(|w| w.first())
                    .and_then(|w| w.as_str()),
            ) else {
                continue;
            };
            if let Some(id) = m.targets.iter().find(|id| id.to_string() == f) {
                self.writer.entry(id.clone()).or_insert_with(|| w.to_owned());
            }
        }
    }

    /// The entry-path runs that targeted `id` and were read, the one that made it grow first,
    /// then the others in series order.
    pub fn entry_runs(&self, id: &FindingId) -> Vec<EntryRun> {
        let mut v: Vec<EntryRun> = self
            .runs
            .iter()
            .filter(|r| r.entry && r.read && r.declared.contains(id))
            .map(|r| EntryRun {
                harness: r.harness.clone(),
                series: r.series.clone(),
                digest: r.digest.clone(),
                varies: r.varies.clone(),
                verdict: r.verdicts.get(id).copied(),
            })
            .collect();
        v.sort_by_key(|r| r.verdict != Some(Verdict::Grew));
        v
    }

    /// The first writer function an entry-path harness named for `id`.
    pub fn writer_function(&self, id: &FindingId) -> Option<&str> {
        self.writer.get(id).map(String::as_str)
    }

    /// The runtime of a harness.
    pub fn runtime_of(&self, harness: &str) -> Option<&str> {
        self.runtime.get(harness).map(String::as_str)
    }

    /// True when `id` has the verdict `grew` in the trend of some harness.
    pub fn grew_in_a_harness(&self, id: &FindingId) -> bool {
        self.runs
            .iter()
            .any(|r| r.verdicts.get(id) == Some(&Verdict::Grew))
    }

    /// The trigger of `id`.
    pub fn trigger(&self, id: &FindingId) -> Trigger {
        let grew = |r: &&Run| r.verdicts.get(id) == Some(&Verdict::Grew);
        if let Some(r) = self.runs.iter().filter(|r| r.entry).find(grew) {
            return Trigger::Reproduced {
                harness: r.harness.clone(),
                varies: r.varies.clone().unwrap_or_else(|| "not recorded".to_owned()),
            };
        }
        if let Some(r) = self.runs.iter().filter(|r| r.own_workload).find(grew) {
            return Trigger::Reproduced {
                harness: r.harness.clone(),
                varies: format!(
                    "the project's workload {}",
                    r.workload.as_deref().unwrap_or(&r.harness)
                ),
            };
        }
        // The first run of each of two harnesses whose drivers differ.
        let mut runs: Vec<(String, String)> = Vec::new();
        let mut digests: BTreeSet<&str> = BTreeSet::new();
        let none = self.without_growth(id);
        for r in &none {
            if runs.len() == 2 {
                break;
            }
            if !runs.iter().any(|(h, _)| *h == r.harness) && digests.insert(r.digest.as_str()) {
                runs.push((
                    r.harness.clone(),
                    r.varies.clone().unwrap_or_else(|| "not recorded".to_owned()),
                ));
            }
        }
        if runs.len() == 2 {
            return Trigger::NotFound { runs };
        }
        Trigger::Unchecked
    }

    /// The runs of entry-path harnesses that targeted `id`, ran, were read, and did not make
    /// it grow. A `repeated` series is left out: it reuses its keys, so a flat holder in it
    /// says nothing about new keys.
    fn without_growth(&self, id: &FindingId) -> Vec<&Run> {
        self.runs
            .iter()
            .filter(|r| r.entry && r.read && r.fresh && r.declared.contains(id))
            .filter(|r| r.verdicts.get(id) != Some(&Verdict::Grew))
            .collect()
    }

    /// The entry-path harnesses that ran and were read without making `id` grow: one per
    /// distinct driver digest, in harness order.
    pub fn entry_runs_without_growth(&self, id: &FindingId) -> Vec<(String, String)> {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        self.without_growth(id)
            .into_iter()
            .filter(|r| seen.insert(r.digest.as_str()))
            .map(|r| {
                (
                    r.harness.clone(),
                    r.varies.clone().unwrap_or_else(|| "not recorded".to_owned()),
                )
            })
            .collect()
    }

    /// An entry-path harness that declared `id` and has no series yet: generated, not run.
    pub fn entry_in_flight(&self, id: &FindingId) -> Option<&str> {
        self.entry_declared
            .get(id)?
            .iter()
            .find(|h| !self.ran.contains(*h))
            .map(String::as_str)
    }
}

/// A leak-side finding whose trigger the run states: state `confirmed` or `strong`, not a
/// duplicate merged into another worker's finding.
pub fn is_subject(view: &RunView, f: &Finding) -> bool {
    matches!(f.state, FindingState::Confirmed | FindingState::Strong)
        && f.polarity() == Polarity::Leak
        && view.ledger.merged_into(&f.id).is_none()
}

/// Why the trigger of a finding that is not shown could not be checked, when it cannot be:
/// the queue is off, the time is at the floor, or the workload is recorded as blocked.
pub fn unchecked_because(view: &RunView) -> Option<&'static str> {
    if !view.state.queue {
        return Some("the run was started with --queue off");
    }
    if gate::budget_over_floor(&view.state, None).is_none() {
        return Some("not enough time was left: the budget was at the floor");
    }
    if gate::workload_unresolved(view, true) {
        return Some("intake.workload records a blocker");
    }
    None
}

/// The items of kind `trigger`: one per confirmed or strong leak-side finding whose trigger
/// is unchecked although a harness fed it keys and it grew, while the run was started with the
/// queue on and more than the floor of the budget is left. The fix is the `--entry` command
/// with every such finding of the runtime; after one entry-path harness without growth it
/// asks for a second, different one.
pub fn items(view: &RunView) -> Vec<BlockingItem> {
    if view.run_dir.is_none() || unchecked_because(view).is_some() {
        return Vec::new();
    }
    let t = Triggers::of(view);
    let mut only: Option<Option<String>> = None;
    // (finding, runtime) of every finding that needs an entry-path run.
    let mut open: Vec<(FindingId, String)> = Vec::new();
    for f in view.ledger.findings() {
        if !is_subject(view, f) || t.trigger(&f.id) != Trigger::Unchecked || !t.grew_in_a_harness(&f.id)
        {
            continue;
        }
        let Some(rt) = gate::finding_runtime_with(view, f, &mut only) else {
            continue;
        };
        open.push((f.id.clone(), rt));
    }
    let mut out = Vec::new();
    for (id, rt) in &open {
        let ids: Vec<String> = open
            .iter()
            .filter(|(_, r)| r == rt)
            .take(MAX_TARGETS)
            .map(|(f, _)| f.to_string())
            .collect();
        let command = format!(
            "dla harness gen --lang {rt} --driver --entry {}",
            ids.join(",")
        );
        let none = t.entry_runs_without_growth(id);
        let (description, fix) = if let Some(h) = t.entry_in_flight(id) {
            (
                format!(
                    "{id} grew when the harness fed it new keys; whether a caller of the public API can do that is not shown, and the entry-path harness {h} has not been run"
                ),
                format!(
                    "fill the hooks and VARIES.txt of {h} so each unit calls public entry points only, dla harness run {h} --mode both, dla trend S<n>"
                ),
            )
        } else if let Some((h, varies)) = none.first() {
            (
                format!(
                    "{id} did not grow in the entry-path harness {h} (varies: {varies}); one more entry-path harness with a different driver is needed before no trigger is found"
                ),
                format!(
                    "write a second, different entry-path harness: {command}, then fill it with other entry points, other inputs and the rejected-input paths (unknown paths, unsupported methods, malformed input), write VARIES.txt, dla harness run <H> --mode both, dla trend S<n>"
                ),
            )
        } else {
            (
                format!(
                    "{id} grew when the harness fed it new keys; whether a caller of the public API can do that is not shown"
                ),
                format!(
                    "{command}, fill the hooks so each unit calls public entry points only with data a caller controls, write VARIES.txt, dla harness run <H> --mode both, dla trend S<n>"
                ),
            )
        };
        out.push(BlockingItem {
            kind: BlockingKind::Trigger,
            id: id.to_string(),
            description,
            fix,
        });
    }
    out
}

/// The report line of a finding's trigger, without the `Trigger: ` prefix.
pub fn line(view: &RunView, t: &Triggers, f: &Finding) -> String {
    match t.trigger(&f.id) {
        Trigger::Reproduced { harness, varies } => format!(
            "reproduced through the public API in {harness} (varies: {})",
            one_line(&varies)
        ),
        Trigger::NotFound { runs } => {
            let writer = t
                .writer_function(&f.id)
                .map(str::to_owned)
                .or_else(|| f.owner.clone())
                .or_else(|| f.site.clone())
                .unwrap_or_else(|| "its holder".to_owned());
            let list: Vec<String> = runs
                .iter()
                .map(|(h, v)| format!("{h}: {}", one_line(v)))
                .collect();
            format!(
                "not found: it grows when the harness passes new keys to {writer}; two workloads through the public API did not grow it ({})",
                list.join("; ")
            )
        }
        Trigger::Unchecked => {
            let why = unchecked_because(view).map_or_else(
                || {
                    let none = t.entry_runs_without_growth(&f.id);
                    if let Some((h, varies)) = none.first() {
                        format!(
                            "the entry-path harness {h} did not grow it (varies: {}) and a second, different one was not run",
                            one_line(varies)
                        )
                    } else if t.grew_in_a_harness(&f.id) {
                        "no entry-path harness was run for it".to_owned()
                    } else {
                        "no harness series that grew is tied to it".to_owned()
                    }
                },
                str::to_owned,
            );
            format!("not checked ({why})")
        }
    }
}

fn one_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Counts of the confirmed and strong leak-side findings by trigger: `(reproduced, not found,
/// not checked)`.
pub fn counts(view: &RunView, t: &Triggers) -> (usize, usize, usize) {
    let mut c = (0, 0, 0);
    for f in view.ledger.findings().filter(|f| is_subject(view, f)) {
        match t.trigger(&f.id) {
            Trigger::Reproduced { .. } => c.0 += 1,
            Trigger::NotFound { .. } => c.1 += 1,
            Trigger::Unchecked => c.2 += 1,
        }
    }
    c
}

/// What the report says about triggers: a line per confirmed or strong leak-side finding and the
/// counts of the `Findings:` line.
#[derive(Debug)]
pub struct ReportFacts {
    lines: BTreeMap<FindingId, String>,
    reproduced: BTreeSet<FindingId>,
    /// Reproduced, not found, not checked.
    pub counts: (usize, usize, usize),
}

impl ReportFacts {
    /// The facts of a view loaded from disk; `None` for a view without a run directory.
    pub fn of(view: &RunView) -> Option<Self> {
        view.run_dir.as_ref()?;
        let t = Triggers::of(view);
        let mut lines = BTreeMap::new();
        let mut reproduced = BTreeSet::new();
        for f in view.ledger.findings().filter(|f| is_subject(view, f)) {
            if matches!(t.trigger(&f.id), Trigger::Reproduced { .. }) {
                reproduced.insert(f.id.clone());
            }
            lines.insert(f.id.clone(), line(view, &t, f));
        }
        Some(Self {
            lines,
            reproduced,
            counts: counts(view, &t),
        })
    }

    /// The `Trigger:` line of a finding, without its prefix.
    pub fn line(&self, id: &FindingId) -> Option<&str> {
        self.lines.get(id).map(String::as_str)
    }

    /// True when a caller of the public API was shown to make the finding's holder grow.
    pub fn reproduced(&self, id: &FindingId) -> bool {
        self.reproduced.contains(id)
    }

    /// The clause of the `Findings:` line, or `None` when no finding is confirmed or strong.
    pub fn clause(&self) -> Option<String> {
        let (r, n, u) = self.counts;
        (r + n + u > 0).then(|| {
            format!(
                "confirmed and strong by trigger: reproduced through the public API {r}, no trigger found {n}, trigger not checked {u}"
            )
        })
    }
}

#[cfg(test)]
#[path = "trigger_tests.rs"]
mod tests;
