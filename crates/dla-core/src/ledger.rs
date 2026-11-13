//! The ledger: append-only JSONL, replayed and validated on load.
//!
//! [`Ledger`] is the only writer. Every mutating method validates against the
//! replayed state first, appends exactly one line in append mode, and returns the
//! new [`LedgerLine`]. The main ledger is never rewritten or truncated.
//!
//! In worker mode ([`Ledger::load_for`]) the same type holds the main ledger (read only)
//! plus one worker's slice `ledger.<worker>.jsonl`; writes go to the slice, IDs carry the
//! `<worker>/` prefix, and the only repair ever made to a file is cutting a half-written
//! final slice line left by a killed process. `dla ledger merge` (see [`crate::merge`])
//! copies slices into the main ledger.

use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use chrono::{DateTime, Utc};

use crate::{
    LEDGER_SCHEMA_VERSION,
    checklist::{AnswerStatus, Checklist, ChecklistAnswer, ChecklistEntry},
    detectors::ARCHETYPE_IDS,
    error::{Error, Refusal, ReplayError, codes},
    evidence::{Authority, EvidenceKind, EvidenceRecord, NewEvidence},
    explain::{self, ExplainInput},
    finding::{
        Explanation, Finding, FindingState, Holder, Locus, NON_LEAK_IDS, Origin, Region,
        RepairClass, Shape, UNCLASSIFIED, UnresolvedReason, check_site, claims_nonleak, nonleak_id,
    },
    ids::{EvidenceId, FindingId, OpenItemId},
    labels::{self, Archetypes, LabelInput},
    m6::codes as m6_codes,
    machine::{
        self, EvidenceLookup, FindingLookup, M0Rules, M3Rules, M4Rules, M5Rules, M6Rules, M7Rules,
        TransitionContext,
    },
    record::{
        FindingAnnotation, FindingExplained, FindingLabeled, FindingMerged, IntegrityResealed,
        LedgerLine, OpenItem, OpenItemResolved, Record, RemovalClaim, Resolution, SliceDone,
        SliceMerged, SliceStarted, Transition, Verification,
    },
    slice::{self, TornTail, codes as m4},
    state::{EntryPath, LEDGER_FILE, RunState, WorkerStatus, now_utc},
    strict, verify,
    version::{Artifact, UpgradeError, upgrade},
};

/// Tools whose growth evidence may back a holder change: the HPROF reader and the V8
/// heap snapshot reader.
const HOLDER_EVIDENCE_TOOLS: [&str; 2] = ["dla-hprof", "dla-v8-snapshot"];

/// The Node resources reader. Its growth evidence observes the timer table, so it may back a
/// change to `registration-table` (and to no other holder).
pub const TIMER_EVIDENCE_TOOL: &str = "node-resources";

/// True for the name of a live timer or listener group of the Node resources reader
/// (`group:handle/<type>@<site>`): the groups of a registration table.
pub fn is_timer_table_group(column: &str) -> bool {
    let Some(kind) = column
        .strip_prefix("group:handle/")
        .and_then(|r| r.split('@').next())
    else {
        return false;
    };
    let lower = kind.to_ascii_lowercase();
    lower == "timeout" || lower == "timerwrap" || lower.contains("listener")
}

/// True for a `file:line` site in a JavaScript or TypeScript file: the only findings the Node
/// resources reader can observe.
fn is_node_site(site: &str) -> bool {
    let file = site.rsplit_once(':').map_or(site, |(f, _)| f);
    ["js", "mjs", "cjs", "jsx", "ts", "mts", "cts", "tsx"]
        .iter()
        .any(|e| file.rsplit_once('.').is_some_and(|(_, x)| x == *e))
}

/// The holder a passing column of the Node resources evidence observes: a live timer or
/// listener group tied to the finding by the finding's own site (the tie key is that site,
/// not a writer's) is a registration table. A column tied any other way, or of another
/// type, observes none.
fn resources_holder_kind(
    column: &serde_json::Value,
    own_site: Option<&str>,
) -> Option<&'static str> {
    if column.pointer("/tie/rule").and_then(|v| v.as_str()) != Some("site") {
        return None;
    }
    let key = column.pointer("/tie/key").and_then(|v| v.as_str())?;
    if own_site != Some(key) {
        return None;
    }
    is_timer_table_group(column.get("column")?.as_str()?).then_some("registration-table")
}

/// An open item with its resolution, if any.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenItemState {
    /// The item as raised.
    pub item: OpenItem,
    /// How it was closed; `None` while open.
    pub resolution: Option<Resolution>,
}

/// Input for [`Ledger::add_finding`].
#[derive(Debug, Clone, PartialEq)]
pub struct NewFinding {
    /// Validated origin.
    pub origin: Origin,
    /// Supporting evidence; at least one.
    pub supports: Vec<EvidenceId>,
    /// Makes this a sibling hypothesis of the given top-level finding.
    pub sibling_of: Option<FindingId>,
    /// Must be `None` (or `UNCLASSIFIED`); any other value is refused.
    pub archetype: Option<String>,
    /// Archetype tag (never filters anything).
    pub suggested_archetype: Option<String>,
    /// Required for a sibling.
    pub hypothesis: Option<String>,
    /// Required for a sibling.
    pub refute_if: Option<String>,
    /// Holder.
    pub holder: Option<Holder>,
    /// Owner.
    pub owner: Option<String>,
    /// Region.
    pub region: Option<Region>,
    /// Shape.
    pub shape: Option<Shape>,
    /// Locus.
    pub locus: Option<Locus>,
    /// `file:line`.
    pub site: Option<String>,
    /// Amplifiers.
    pub amplifiers: Vec<String>,
    /// Repair class.
    pub repair_class: Option<RepairClass>,
    /// Explanation.
    pub explanation: Option<Explanation>,
}

impl NewFinding {
    /// The mandatory fields; everything else unset.
    pub fn new(origin: Origin, supports: Vec<EvidenceId>) -> Self {
        Self {
            origin,
            supports,
            sibling_of: None,
            archetype: None,
            suggested_archetype: None,
            hypothesis: None,
            refute_if: None,
            holder: None,
            owner: None,
            region: None,
            shape: None,
            locus: None,
            site: None,
            amplifiers: Vec::new(),
            repair_class: None,
            explanation: None,
        }
    }
}

/// What the ledger knows about one worker's slice: its first and last records and the
/// latest merge of it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SliceRecord {
    /// The `slice_started` record.
    pub started: Option<SliceStarted>,
    /// The `slice_done` record, once the worker finished.
    pub done: Option<SliceDone>,
    /// The latest `slice_merged` record.
    pub merged: Option<SliceMerged>,
}

/// Everything derived from replaying the lines.
#[derive(Debug, Clone)]
struct Index {
    evidence: BTreeMap<EvidenceId, EvidenceRecord>,
    findings: BTreeMap<FindingId, Finding>,
    open_items: BTreeMap<OpenItemId, OpenItemState>,
    answers: BTreeMap<String, ChecklistAnswer>,
    annotations: BTreeMap<FindingId, Annotation>,
    /// The run's entry path (the artifact rules apply to `artifact`).
    entry: EntryPath,
    /// Set while a worker's slice is replayed or appended: the worker's own scope.
    slice_worker: Option<String>,
    slices: BTreeMap<String, SliceRecord>,
    /// Duplicate finding -> canonical finding (`finding_merged`).
    merged_into: BTreeMap<FindingId, FindingId>,
    verifications: BTreeMap<FindingId, Vec<Verification>>,
    /// Every `finding_labeled` record per finding, oldest first (schema 6).
    labels: BTreeMap<FindingId, Vec<FindingLabeled>>,
    /// The latest `finding_explained` record per finding (schema 6).
    explanations: BTreeMap<FindingId, FindingExplained>,
    /// Every `integrity_resealed` record, oldest first (schema 6).
    reseals: Vec<IntegrityResealed>,
}

impl Index {
    fn new(entry: EntryPath) -> Self {
        Self {
            evidence: BTreeMap::new(),
            findings: BTreeMap::new(),
            open_items: BTreeMap::new(),
            answers: BTreeMap::new(),
            annotations: BTreeMap::new(),
            entry,
            slice_worker: None,
            slices: BTreeMap::new(),
            merged_into: BTreeMap::new(),
            verifications: BTreeMap::new(),
            labels: BTreeMap::new(),
            explanations: BTreeMap::new(),
            reseals: Vec::new(),
        }
    }
}

/// Everything recorded about a finding's writers and removers, merged over all
/// `finding_annotated` lines in ledger order without duplicates.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Annotation {
    /// Sites that add to the holder.
    pub writers: Vec<String>,
    /// Sites that remove from the holder.
    pub removers: Vec<String>,
    /// Notes, oldest first.
    pub notes: Vec<String>,
    /// Holder corrections backed by evidence, oldest first.
    pub holder_changes: Vec<HolderChange>,
}

/// A holder replaced by an evidence-backed annotation (schema 5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderChange {
    /// The holder before.
    pub from: Holder,
    /// The holder after.
    pub to: Holder,
    /// The growth evidence that backs the change.
    pub evidence: Vec<EvidenceId>,
}

impl EvidenceLookup for Index {
    fn evidence(&self, id: &EvidenceId) -> Option<&EvidenceRecord> {
        self.evidence.get(id)
    }
    fn evidence_ids(&self) -> Vec<EvidenceId> {
        self.evidence.keys().cloned().collect()
    }
}

impl FindingLookup for Index {
    fn finding(&self, id: &FindingId) -> Option<&Finding> {
        self.findings.get(id)
    }
    fn family(&self, id: &FindingId) -> Vec<&Finding> {
        let Some(f) = self.findings.get(id) else {
            return Vec::new();
        };
        let root = f.root_id();
        let Some(root) = self.findings.get(&root) else {
            return vec![f];
        };
        std::iter::once(root)
            .chain(root.siblings.iter().filter_map(|s| self.findings.get(s)))
            .collect()
    }
}

fn empty(s: &Option<String>) -> bool {
    s.as_deref().is_none_or(|t| t.trim().is_empty())
}

fn core_only(what: &str) -> Refusal {
    Refusal::new(
        m4::CORE_ONLY,
        format!("{what} is written by the core, not by a worker"),
        "the core (no --worker)",
        "run the same command without --worker (and without DLA_WORKER)",
    )
}

fn bad_worker(name: &str) -> Refusal {
    Refusal::new(
        m4::BAD_WORKER,
        format!("{name:?} is not a worker name"),
        "a worker name: lowercase letters, digits and -, starting with a letter (a runtime name such as jvm)",
        "dla run dispatch --workers jvm,rust",
    )
}

/// The `type` tag of a record, for messages.
fn record_name(r: &Record) -> &'static str {
    match r {
        Record::Evidence(_) => "evidence",
        Record::FindingAdded { .. } => "finding_added",
        Record::Transition(_) => "transition",
        Record::OpenItem(_) => "open_item",
        Record::OpenItemResolved(_) => "open_item_resolved",
        Record::ChecklistAnswer(_) => "checklist_answer",
        Record::FindingAnnotated(_) => "finding_annotated",
        Record::SliceStarted(_) => "slice_started",
        Record::SliceDone(_) => "slice_done",
        Record::SliceMerged(_) => "slice_merged",
        Record::FindingMerged(_) => "finding_merged",
        Record::Verification(_) => "verification",
        Record::FindingLabeled(_) => "finding_labeled",
        Record::FindingExplained(_) => "finding_explained",
        Record::IntegrityResealed(_) => "integrity_resealed",
    }
}

impl Index {
    fn evidence_hint(&self) -> String {
        self.evidence
            .keys()
            .next_back()
            .map_or_else(|| "E1".to_owned(), |e| e.to_string())
    }

    fn require_evidence(&self, ids: &[EvidenceId], what: &str, fix: &str) -> Result<(), Refusal> {
        for id in ids {
            if !self.evidence.contains_key(id) {
                let known: Vec<String> = self.evidence.keys().map(|e| e.to_string()).collect();
                return Err(Refusal::new(
                    codes::UNKNOWN_EVIDENCE,
                    format!("{what} cites {id}, which does not exist earlier in the ledger"),
                    format!("evidence {id}"),
                    format!(
                        "{fix} (existing evidence: {})",
                        if known.is_empty() {
                            "none".to_owned()
                        } else {
                            known.join(", ")
                        }
                    ),
                ));
            }
        }
        Ok(())
    }

    fn require_finding(&self, id: &FindingId, what: &str) -> Result<&Finding, Refusal> {
        self.findings.get(id).ok_or_else(|| {
            Refusal::new(
                codes::UNKNOWN_FINDING,
                format!("{what} refers to {id}, which does not exist earlier in the ledger"),
                format!("finding {id}"),
                "dla run status",
            )
        })
    }

    /// Validates a record against the current state without changing anything.
    /// `version` is the schema version of the line that carries it: version 1 lines
    /// replay under the M0 rules, later ones under the M3 rules.
    fn check(&self, record: &Record, version: u32) -> Result<(), Refusal> {
        if let Some(w) = &self.slice_worker {
            self.check_slice_scope(record, w)?;
        }
        match record {
            Record::Evidence(e) => {
                if self.evidence.contains_key(&e.evidence_id) {
                    return Err(Refusal::new(
                        codes::DUPLICATE_ID,
                        format!("evidence {} already exists", e.evidence_id),
                        "a fresh evidence ID",
                        "let the ledger allocate the ID",
                    ));
                }
                e.check_result_size()
            }
            Record::FindingAdded { finding } => self.check_finding(finding, version),
            Record::Transition(t) => self.check_transition(t, version),
            Record::OpenItem(o) => {
                if self.open_items.contains_key(&o.id) {
                    return Err(Refusal::new(
                        codes::DUPLICATE_ID,
                        format!("open item {} already exists", o.id),
                        "a fresh open item ID",
                        "let the ledger allocate the ID",
                    ));
                }
                if o.text.trim().is_empty() {
                    return Err(Refusal::new(
                        codes::EMPTY_FIELD,
                        "an open item needs text",
                        "text",
                        "describe what is open",
                    ));
                }
                if let Some(f) = &o.finding {
                    self.require_finding(f, "open item")?;
                }
                Ok(())
            }
            Record::OpenItemResolved(r) => self.check_open_item_resolved(r, version),
            Record::ChecklistAnswer(a) => self.check_answer(a),
            Record::FindingAnnotated(a) => self.check_annotation(a, version),
            Record::SliceStarted(s) => self.check_slice_started(s),
            Record::SliceDone(d) => self.check_slice_done(d),
            Record::SliceMerged(m) => self.check_slice_merged(m),
            Record::FindingMerged(m) => self.check_finding_merged(m),
            Record::Verification(v) => self.check_verification(v, version),
            Record::FindingLabeled(l) => {
                self.require_schema6(record, version)?;
                self.check_worker_evidence(&l.satisfied.evidence)?;
                labels::check(self, self, l)
            }
            Record::FindingExplained(x) => {
                self.require_schema6(record, version)?;
                self.check_worker_evidence(x.evidence.as_slice())?;
                explain::check(self, self, x, self.entry == EntryPath::Artifact)
            }
            Record::IntegrityResealed(r) => {
                self.require_schema6(record, version)?;
                if r.reason.trim().is_empty() || r.files.is_empty() {
                    return Err(Refusal::new(
                        codes::EMPTY_FIELD,
                        "integrity_resealed needs the files sealed again and a reason",
                        "files and reason",
                        "dla run reseal --reason \"<why the files changed outside dla>\"",
                    ));
                }
                Ok(())
            }
        }
    }

    /// The reason `inconclusive` exists from schema 7 on: an older line that carries it was
    /// not written by a build that checks it.
    fn require_schema7_reason(
        &self,
        reason: Option<UnresolvedReason>,
        version: u32,
        what: &str,
    ) -> Result<(), Refusal> {
        if reason != Some(UnresolvedReason::Inconclusive) || version >= 7 {
            return Ok(());
        }
        Err(Refusal::new(
            codes::INVALID_VALUE,
            format!(
                "{what} carries the reason inconclusive, which is a schema 7 reason, but this line says schema version {version}"
            ),
            "schema_version 7 or later",
            "write the line with a build that writes schema 7",
        ))
    }

    /// A record type that exists from schema 6 on.
    fn require_schema6(&self, record: &Record, version: u32) -> Result<(), Refusal> {
        if version >= 6 {
            return Ok(());
        }
        Err(Refusal::new(
            codes::INVALID_VALUE,
            format!(
                "{} is a schema 6 record, but this line says schema version {version}",
                record_name(record)
            ),
            "schema_version 6 or later",
            "write the line with a build that writes schema 6",
        ))
    }

    /// A worker may cite its own evidence or the main ledger's, never another worker's.
    fn check_worker_evidence(&self, evidence: &[EvidenceId]) -> Result<(), Refusal> {
        let Some(w) = &self.slice_worker else {
            return Ok(());
        };
        if let Some(e) = evidence.iter().find(|e| e.prefix().is_some_and(|p| p != w)) {
            return Err(Refusal::new(
                m4::FOREIGN_ITEM,
                format!(
                    "worker {w} cites {e}, which belongs to another worker's slice; a worker may cite its own evidence or the main ledger's"
                ),
                format!("evidence of {w} or of the main ledger"),
                format!(
                    "dla --worker {w} ledger ... --evidence <an id of {w} or of the main ledger>"
                ),
            ));
        }
        Ok(())
    }

    /// Rules for a record appended to (or replayed from) worker `w`'s slice: it begins
    /// with `slice_started`, is closed by `slice_done`, creates only IDs with the
    /// worker's prefix, changes only the worker's own findings and open items, and
    /// leaves merge records and verification to the core.
    fn check_slice_scope(&self, record: &Record, w: &str) -> Result<(), Refusal> {
        let own = |prefix: Option<&str>| prefix == Some(w);
        let bad_prefix = |what: &str, id: &dyn std::fmt::Display| {
            Refusal::new(
                m4::BAD_PREFIX,
                format!("worker {w} allocated {what} {id} without its own prefix {w}/"),
                format!("an ID prefixed {w}/"),
                format!("let the ledger allocate the ID (dla --worker {w} ledger add ...)"),
            )
        };
        let foreign = |what: &str, id: &dyn std::fmt::Display| {
            Refusal::new(
                m4::FOREIGN_ITEM,
                format!(
                    "worker {w} may only change its own {what}s ({w}/...), not {id}: that one belongs to the main ledger or another worker"
                ),
                format!("a {what} of {w}"),
                format!(
                    "record your own hypothesis instead: dla --worker {w} ledger add --origin hypothesis --evidence <id> ..."
                ),
            )
        };
        let state = self.slices.get(w);
        if state.is_some_and(|s| s.done.is_some()) {
            return Err(Refusal::new(
                m4::SLICE_CLOSED,
                format!("slice of {w} is closed by slice_done; it takes no more records"),
                "an open slice",
                "the core runs `dla ledger merge`, then records the item in the main ledger (`dla ledger add ...`); a closed slice cannot be reopened",
            ));
        }
        if !matches!(record, Record::SliceStarted(_)) && state.is_none_or(|s| s.started.is_none()) {
            return Err(Refusal::new(
                m4::SLICE_STATE,
                format!("slice of {w} does not begin with slice_started"),
                "a slice_started record as the first line",
                format!("dla --worker {w} ledger add ... (the first write starts the slice)"),
            ));
        }
        match record {
            Record::Evidence(e) if !own(e.evidence_id.prefix()) => {
                Err(bad_prefix("evidence", &e.evidence_id))
            }
            Record::FindingAdded { finding } if !own(finding.id.prefix()) => {
                Err(bad_prefix("finding", &finding.id))
            }
            Record::OpenItem(o) if !own(o.id.prefix()) => Err(bad_prefix("open item", &o.id)),
            Record::Transition(t) if !own(t.finding.prefix()) => {
                Err(foreign("finding", &t.finding))
            }
            Record::FindingAnnotated(a) if !own(a.finding.prefix()) => {
                Err(foreign("finding", &a.finding))
            }
            Record::OpenItemResolved(r) if !own(r.id.prefix()) => Err(foreign("open item", &r.id)),
            Record::SliceStarted(s) if s.worker != w => Err(Refusal::new(
                m4::SLICE_STATE,
                format!("slice of {w} starts with a record for worker {}", s.worker),
                format!("worker {w}"),
                format!("dla --worker {w} ledger add ..."),
            )),
            Record::SliceDone(d) if d.worker != w => Err(Refusal::new(
                m4::SLICE_STATE,
                format!("slice of {w} is closed by a record for worker {}", d.worker),
                format!("worker {w}"),
                format!("dla --worker {w} ledger done"),
            )),
            Record::FindingLabeled(l) if !own(l.finding.prefix()) => {
                Err(foreign("finding", &l.finding))
            }
            Record::FindingExplained(x) if !own(x.finding.prefix()) => {
                Err(foreign("finding", &x.finding))
            }
            // A worker records duplicates among its own findings (`dla ledger duplicate`).
            Record::FindingMerged(m) if own(m.canonical.prefix()) && own(m.duplicate.prefix()) => {
                Ok(())
            }
            Record::SliceMerged(_)
            | Record::FindingMerged(_)
            | Record::Verification(_)
            | Record::IntegrityResealed(_) => Err(Refusal::new(
                m4::CORE_ONLY,
                format!(
                    "a worker slice cannot hold {}: the core writes it",
                    record_name(record)
                ),
                "the core (no --worker)",
                "dla ledger merge   or   dla ledger verify <F> ...   (without --worker)",
            )),
            _ => Ok(()),
        }
    }

    fn check_slice_started(&self, s: &SliceStarted) -> Result<(), Refusal> {
        if !crate::ids::valid_worker_name(&s.worker) {
            return Err(bad_worker(&s.worker));
        }
        if s.runtime.trim().is_empty() {
            return Err(Refusal::new(
                codes::EMPTY_FIELD,
                format!("slice_started of {} has no runtime", s.worker),
                "runtime",
                "dla run dispatch --workers <runtimes>",
            ));
        }
        if self
            .slices
            .get(&s.worker)
            .is_some_and(|r| r.started.is_some())
        {
            return Err(Refusal::new(
                m4::SLICE_STATE,
                format!("slice of {} already started", s.worker),
                "a single slice_started per worker",
                "dla ledger merge   (a worker has one slice)",
            ));
        }
        Ok(())
    }

    fn check_slice_done(&self, d: &SliceDone) -> Result<(), Refusal> {
        let Some(r) = self.slices.get(&d.worker).filter(|r| r.started.is_some()) else {
            return Err(Refusal::new(
                m4::SLICE_STATE,
                format!("slice_done for {} without a slice_started", d.worker),
                "a started slice",
                format!("dla --worker {} ledger add ...", d.worker),
            ));
        };
        if r.done.is_some() {
            return Err(Refusal::new(
                m4::SLICE_STATE,
                format!("slice of {} is already done", d.worker),
                "a slice that is not done",
                "dla ledger merge",
            ));
        }
        Ok(())
    }

    fn check_slice_merged(&self, m: &SliceMerged) -> Result<(), Refusal> {
        let fix = "dla ledger merge";
        let Some(r) = self.slices.get(&m.worker).filter(|r| r.started.is_some()) else {
            return Err(Refusal::new(
                m4::MERGE_STATE,
                format!("slice_merged for {} without its slice_started", m.worker),
                "the slice's lines merged first",
                fix,
            ));
        };
        let before = r.merged.as_ref().map_or(0, |m| m.lines);
        if m.lines <= before {
            return Err(Refusal::new(
                m4::MERGE_STATE,
                format!(
                    "slice_merged for {} says {} lines but {before} are merged already",
                    m.worker, m.lines
                ),
                "a larger line count than the last merge",
                fix,
            ));
        }
        if !verify::is_sha256(&m.sha256) {
            return Err(Refusal::new(
                m4::MERGE_STATE,
                format!("slice_merged for {} has no valid sha256", m.worker),
                "a lowercase hex SHA-256",
                fix,
            ));
        }
        Ok(())
    }

    fn check_finding_merged(&self, m: &FindingMerged) -> Result<(), Refusal> {
        let fix = "dla ledger merge";
        let bad = |why: String| {
            Refusal::new(
                m4::MERGE_STATE,
                why,
                "two distinct top-level findings of different workers, or duplicates of one scope",
                fix,
            )
        };
        self.require_finding(&m.canonical, "finding_merged")?;
        self.require_finding(&m.duplicate, "finding_merged")?;
        if m.canonical == m.duplicate {
            return Err(bad(format!("{} cannot be merged into itself", m.canonical)));
        }
        if m.canonical.is_sibling() || m.duplicate.is_sibling() {
            return Err(bad(format!(
                "{} and {} must be top-level findings, not sibling hypotheses",
                m.canonical, m.duplicate
            )));
        }
        // Two cases write this record: the merge of findings of different workers, and
        // `dla ledger duplicate` (one scope, exact identity). Anything else is refused.
        let cross_worker = matches!(
            (m.canonical.prefix(), m.duplicate.prefix()),
            (Some(a), Some(b)) if a != b
        );
        if !cross_worker {
            let identical = match (self.finding(&m.canonical), self.finding(&m.duplicate)) {
                (Some(c), Some(d)) => crate::duplicate::duplicate_by(c, d).is_some(),
                _ => false,
            };
            if !identical {
                return Err(bad(format!(
                    "{} and {} are neither findings of different workers nor duplicates in one worker and runtime with the same owner or site",
                    m.canonical, m.duplicate
                )));
            }
        }
        // Repeating a merge re-unions the supports; the reverse of an earlier merge
        // (the duplicate has since become the more advanced finding) swaps the roles.
        let repeat = self.merged_into.get(&m.duplicate) == Some(&m.canonical);
        let swap = self.merged_into.get(&m.canonical) == Some(&m.duplicate);
        if !repeat && self.merged_into.contains_key(&m.duplicate) {
            return Err(bad(format!(
                "{} is already merged into another finding",
                m.duplicate
            )));
        }
        if !swap && self.merged_into.contains_key(&m.canonical) {
            return Err(bad(format!(
                "{} is itself a duplicate; merge into its canonical finding",
                m.canonical
            )));
        }
        Ok(())
    }

    fn check_verification(&self, v: &Verification, version: u32) -> Result<(), Refusal> {
        let fix = format!(
            "dla ledger verify {} --site <file:line|none> --holder <holder> --mechanism \"<text>\" --bundle <sha256 printed by dla bundle>",
            v.finding
        );
        let bad = |missing: &str, why: String| {
            Refusal::new(m4::BAD_VERIFICATION, why, missing.to_owned(), fix.clone())
        };
        self.require_finding(&v.finding, "verification")?;
        if v.mechanism.trim().is_empty() {
            return Err(bad(
                "--mechanism",
                format!("verification of {} has no mechanism", v.finding),
            ));
        }
        if v.label.trim().is_empty() || v.label.chars().any(char::is_whitespace) {
            return Err(bad(
                "--label an archetype ID or UNCLASSIFIED",
                format!("verification of {} has label {:?}", v.finding, v.label),
            ));
        }
        if !verify::is_sha256(&v.bundle) {
            return Err(bad(
                "--bundle, the sha256 `dla bundle` printed",
                format!("verification of {} has no valid bundle sha256", v.finding),
            ));
        }
        if let Some(site) = &v.site
            && let Err(m) = check_site(site)
        {
            return Err(bad("--site file:line or none", m));
        }
        if version >= 6 {
            self.check_verification_label(v)?;
        }
        Ok(())
    }

    /// Schema 6: the verifier's label is an archetype ID or `UNCLASSIFIED`, and
    /// `agrees_label` is what comparing it with the finding's label gives (absent when
    /// archetypes were off, which allows `UNCLASSIFIED` only).
    fn check_verification_label(&self, v: &Verification) -> Result<(), Refusal> {
        let f = self.require_finding(&v.finding, "verification")?;
        let known = v.label == UNCLASSIFIED || ARCHETYPE_IDS.contains(&v.label.as_str());
        if !known {
            return Err(Refusal::new(
                m6_codes::UNKNOWN_ARCHETYPE,
                format!(
                    "verification of {} has label {:?}, which is not an archetype ID",
                    v.finding, v.label
                ),
                "an archetype ID or UNCLASSIFIED",
                format!("use one of: {}, UNCLASSIFIED", ARCHETYPE_IDS.join(", ")),
            ));
        }
        match v.agrees_label {
            None if v.label != UNCLASSIFIED => Err(Refusal::new(
                m6_codes::ARCHETYPES_OFF,
                format!(
                    "verification of {} has label {} but compares no label: with archetypes off only UNCLASSIFIED is accepted",
                    v.finding, v.label
                ),
                "--label UNCLASSIFIED",
                format!("dla ledger verify {} ... --label UNCLASSIFIED", v.finding),
            )),
            Some(b) if b != (f.archetype == v.label) => Err(Refusal::new(
                codes::INVALID_VALUE,
                format!(
                    "verification of {} says agrees_label {b}, but its label {} against the finding's {} says otherwise",
                    v.finding, v.label, f.archetype
                ),
                "agrees_label computed from the labels",
                format!("dla ledger verify {} ...", v.finding),
            )),
            _ => Ok(()),
        }
    }

    fn check_annotation(&self, a: &FindingAnnotation, version: u32) -> Result<(), Refusal> {
        self.require_finding(&a.finding, "annotation")?;
        if version >= 3 && self.entry == EntryPath::Artifact && a.site.is_some() {
            return Err(Refusal::new(
                m4::SITE_FORBIDDEN,
                format!(
                    "{} cannot get a site on an artifact run: a dump holds no source lines, and a confirmed artifact finding must have no site",
                    a.finding
                ),
                "an annotation without --site",
                format!("dla ledger annotate {} --owner <class.field>", a.finding),
            ));
        }
        for site in a.writers.iter().chain(&a.removers).chain(&a.site) {
            check_site(site).map_err(|m| {
                Refusal::new(
                    codes::INVALID_SITE,
                    m,
                    "a site written file:line",
                    format!(
                        "dla ledger annotate {} --writer src/Main.java:42",
                        a.finding
                    ),
                )
            })?;
        }
        let note_blank = a.note.as_deref().is_some_and(|n| n.trim().is_empty());
        if note_blank {
            return Err(Refusal::new(
                codes::EMPTY_FIELD,
                "the annotation note is blank",
                "note text, or omit --note",
                format!("dla ledger annotate {} --note \"<text>\"", a.finding),
            ));
        }
        if a.owner.as_deref().is_some_and(|o| o.trim().is_empty()) {
            return Err(Refusal::new(
                codes::EMPTY_FIELD,
                "the annotation owner is blank",
                "owner text, or omit --owner",
                format!("dla ledger annotate {} --owner \"<field>\"", a.finding),
            ));
        }
        if a.writers.is_empty()
            && a.removers.is_empty()
            && a.note.is_none()
            && a.holder.is_none()
            && a.owner.is_none()
            && a.region.is_none()
            && a.site.is_none()
        {
            return Err(Refusal::new(
                codes::EMPTY_ANNOTATION,
                format!(
                    "annotation of {} has no writer, remover, note, holder, owner, region or site",
                    a.finding
                ),
                "at least one --writer, --remover, --note, --holder, --owner, --region or --site",
                format!(
                    "dla ledger annotate {} --writer src/Main.java:42",
                    a.finding
                ),
            ));
        }
        let f = self.require_finding(&a.finding, "annotation")?;
        if version >= 5 && !a.evidence.is_empty() {
            self.check_annotation_evidence(a)?;
        }
        let conflicts = [
            (
                "holder",
                f.holder.map(|h| h.to_string()),
                a.holder.map(|h| h.to_string()),
            ),
            ("owner", f.owner.clone(), a.owner.clone()),
            ("site", f.site.clone(), a.site.clone()),
            (
                "region",
                f.region.map(|r| r.to_string()),
                a.region.map(|r| r.to_string()),
            ),
        ];
        for (field, have, want) in conflicts {
            if let (Some(have), Some(want)) = (have, want)
                && have != want
            {
                if field == "holder" && version >= 5 && !a.evidence.is_empty() {
                    self.check_holder_change(a, f)?;
                    continue;
                }
                if field == "owner" {
                    return Err(Refusal::new(
                        codes::ANNOTATION_CONFLICT,
                        format!(
                            "{} already has owner {have:?}; an owner is set once and cannot be changed to {want:?}",
                            a.finding
                        ),
                        format!("owner {have:?}, or a new finding with the right owner"),
                        format!(
                            "when the owner is wrong, add a finding with the owner in the form the evidence names at the same site (dla ledger add --origin growth-series --evidence <E> --site {} --owner {want}) and merge the old one into it (dla ledger duplicate {} --of <new>), or resolve the old one",
                            f.site.as_deref().unwrap_or("<the same file:line>"),
                            a.finding
                        ),
                    ));
                }
                return Err(Refusal::new(
                    codes::ANNOTATION_CONFLICT,
                    format!(
                        "{} already has {field} {have:?}; an annotation only fills a {field} that is not set yet, and it cannot change it to {want:?}",
                        a.finding
                    ),
                    format!("{field} {have:?} or a new finding"),
                    format!(
                        "keep {have:?}, or record the corrected {field} on a new hypothesis: dla ledger add --origin hypothesis --sibling-of {} --hypothesis \"<cause>\" --refute-if \"<result>\" --evidence <id>{}",
                        f.root_id(),
                        if field == "holder" {
                            format!(
                                "; or, when heap-graph growth evidence observed the holder: dla ledger annotate {} --holder {want} --evidence <growth evidence id>",
                                a.finding
                            )
                        } else {
                            String::new()
                        }
                    ),
                ));
            }
        }
        Ok(())
    }

    /// Evidence cited by an annotation must exist, and a worker may cite only its own
    /// evidence or the main ledger's.
    fn check_annotation_evidence(&self, a: &FindingAnnotation) -> Result<(), Refusal> {
        if let Some(w) = &self.slice_worker
            && let Some(e) = a
                .evidence
                .iter()
                .find(|e| e.prefix().is_some_and(|p| p != w))
        {
            return Err(Refusal::new(
                m4::FOREIGN_ITEM,
                format!(
                    "worker {w} cites {e}, which belongs to another worker's slice; a worker may cite its own evidence or the main ledger's"
                ),
                format!("evidence of {w} or of the main ledger"),
                format!("dla --worker {w} summarize heap-graph ... and cite that evidence"),
            ));
        }
        self.require_evidence(
            &a.evidence,
            &format!("annotation of {}", a.finding),
            "cite growth evidence from `dla summarize heap-graph` (dla run status lists the IDs)",
        )?;
        Ok(())
    }

    /// The rule for changing a holder already set (schema 5): a cited growth-series
    /// evidence of authority dump or live, written by the HPROF or V8 snapshot reader,
    /// has a passing column tied to the finding whose `holder_kind` is the new holder.
    fn check_holder_change(&self, a: &FindingAnnotation, f: &Finding) -> Result<(), Refusal> {
        let (Some(want), Some(have)) = (a.holder, f.holder) else {
            return Ok(());
        };
        if let Some(old) = a.replaced_holder
            && old != have
        {
            return Err(Refusal::new(
                codes::ANNOTATION_CONFLICT,
                format!(
                    "annotation of {} says it replaces holder {old}, but the finding has {have}",
                    a.finding
                ),
                format!("replaced_holder {have}"),
                format!(
                    "dla ledger annotate {} --holder {want} --evidence <growth evidence id>",
                    a.finding
                ),
            ));
        }
        let fix = if want == Holder::RegistrationTable {
            let heap = format!(
                "dla summarize heap-graph <dump...> --finding {} (then) dla ledger annotate {} --holder {want} --evidence <growth evidence id>",
                a.finding, a.finding
            );
            if self
                .findings
                .get(&a.finding)
                .and_then(|f| f.site.as_deref())
                .is_some_and(is_node_site)
            {
                format!(
                    "{heap}; for a timer or listener: dla summarize non-heap --backend node-resources <resources.json x3 or more> --finding {} (then) the same annotate with its growth evidence",
                    a.finding
                )
            } else {
                heap
            }
        } else {
            format!(
                "dla summarize heap-graph <dump...> --finding {} (then) dla ledger annotate {} --holder {want} --evidence <growth evidence id>",
                a.finding, a.finding
            )
        };
        let refuse = |why: String, expected: String| {
            Refusal::new(
                codes::ANNOTATION_CONFLICT,
                format!(
                    "{} already has holder {have:?}; changing it to {want:?} needs growth evidence that observed {want}: {why}",
                    a.finding
                ),
                expected,
                fix.clone(),
            )
        };
        let id = a.finding.to_string();
        let own_site = self
            .findings
            .get(&a.finding)
            .and_then(|f| f.site.as_deref());
        let mut reasons = Vec::new();
        for eid in &a.evidence {
            let Some(e) = self.evidence.get(eid) else {
                continue;
            };
            if e.kind != EvidenceKind::GrowthSeries {
                reasons.push(format!("{eid} is {}, not growth-series", e.kind));
                continue;
            }
            if !matches!(e.authority, Authority::Dump | Authority::Live) {
                reasons.push(format!(
                    "{eid} has authority {}, not dump or live",
                    e.authority
                ));
                continue;
            }
            let timer_table = e.tool.name == TIMER_EVIDENCE_TOOL;
            if timer_table && want != Holder::RegistrationTable {
                reasons.push(format!(
                    "{eid} was written by {TIMER_EVIDENCE_TOOL}, which observes timer and listener tables only: it backs {}, not {want}",
                    Holder::RegistrationTable
                ));
                continue;
            }
            if !timer_table && !HOLDER_EVIDENCE_TOOLS.contains(&e.tool.name.as_str()) {
                reasons.push(format!(
                    "{eid} was written by {}, not a heap-graph reader ({}) or the timer table reader ({TIMER_EVIDENCE_TOOL})",
                    e.tool.name,
                    HOLDER_EVIDENCE_TOOLS.join(" or ")
                ));
                continue;
            }
            let columns = e
                .result
                .get("passed")
                .and_then(|p| p.as_array())
                .map(Vec::as_slice)
                .unwrap_or_default();
            let mut tied = false;
            let mut kinds = Vec::new();
            for c in columns {
                let ties = c.get("finding").and_then(|v| v.as_str()) == Some(id.as_str())
                    || c.get("candidates")
                        .and_then(|v| v.as_array())
                        .is_some_and(|l| l.iter().any(|v| v.as_str() == Some(id.as_str())));
                if !ties {
                    continue;
                }
                tied = true;
                let kind = if timer_table {
                    resources_holder_kind(c, own_site)
                } else {
                    c.get("holder_kind").and_then(|v| v.as_str())
                };
                match kind {
                    Some(k) if k == want.as_str() => return Ok(()),
                    Some(k) => kinds.push(k.to_owned()),
                    None => {}
                }
            }
            if !tied {
                reasons.push(format!(
                    "{eid} has no passing column tied to {id} (by finding or candidates)"
                ));
            } else if kinds.is_empty() && timer_table {
                reasons.push(format!(
                    "the passing columns of {eid} tied to {id} are not timer or listener groups tied by the finding's own creation site (a writer's site does not count)"
                ));
            } else if kinds.is_empty() {
                reasons.push(format!(
                    "the passing columns of {eid} tied to {id} carry no holder_kind"
                ));
            } else {
                reasons.push(format!(
                    "the passing columns of {eid} tied to {id} observed {}, not {want}",
                    kinds.join(", ")
                ));
            }
        }
        Err(refuse(
            reasons.join("; "),
            format!(
                "growth-series evidence (dump or live) with a passing column tied to {id} and holder_kind {want}"
            ),
        ))
    }

    fn check_finding(&self, f: &Finding, version: u32) -> Result<(), Refusal> {
        let id = &f.id;
        if self.findings.contains_key(id) {
            return Err(Refusal::new(
                codes::DUPLICATE_ID,
                format!("finding {id} already exists"),
                "a fresh finding ID",
                "let the ledger allocate the ID",
            ));
        }
        if version >= 3 && self.entry == EntryPath::Artifact && f.site.is_some() {
            return Err(Refusal::new(
                m4::SITE_FORBIDDEN,
                format!(
                    "{id} names a site on an artifact run: a dump holds no source lines, and a confirmed artifact finding must have no site"
                ),
                "a finding without --site, identified by its owner",
                "dla ledger add --origin growth-series --evidence <id> --owner <class.field> --holder <holder>",
            ));
        }
        if f.state != FindingState::Candidate {
            return Err(Refusal::new(
                codes::NOT_CANDIDATE,
                format!("a finding enters the ledger as candidate, not {}", f.state),
                "state candidate",
                "add it as a candidate, then move it with `dla ledger resolve`",
            ));
        }
        if f.reason.is_some() || f.remedy.is_some() {
            return Err(Refusal::new(
                codes::UNEXPECTED_REASON,
                format!("candidate {id} carries a reason or remedy"),
                "no reason and no remedy",
                "add the finding first, then use `dla ledger resolve`",
            ));
        }
        if version >= 6 && f.explanation.is_some() {
            return Err(Refusal::new(
                m6_codes::EXPLANATION_LOCKED,
                format!(
                    "{id} sets an explanation; from schema 6 an explanation is recorded by `dla ledger explain`, which checks it"
                ),
                "no explanation on `ledger add`",
                "dla ledger explain <finding> --tag inferred --adds <file:line> --removal never --text \"<text>\"",
            ));
        }
        if f.archetype != UNCLASSIFIED {
            return Err(Refusal::new(
                codes::ARCHETYPE_LOCKED,
                format!(
                    "{id} sets archetype {:?}; findings enter as UNCLASSIFIED",
                    f.archetype
                ),
                "archetype UNCLASSIFIED",
                "omit the archetype; only `dla ledger label` assigns one",
            ));
        }
        if version >= 2
            && let Some(h) = f.hypothesis.as_deref()
            && claims_nonleak(h)
            && nonleak_id(h).is_none()
        {
            return Err(Refusal::new(
                codes::INVALID_VALUE,
                format!(
                    "hypothesis {h:?} of {id} begins with N- but does not name a non-leak explanation: use one of {}, followed by nothing, a space or a colon",
                    NON_LEAK_IDS.join(", ")
                ),
                format!(
                    "a hypothesis starting with one of {}",
                    NON_LEAK_IDS.join("|")
                ),
                format!(
                    "dla ledger add --origin hypothesis --sibling-of {} --hypothesis \"N-WARM: <detail>\" --refute-if \"<result>\" --evidence <id>",
                    f.root_id()
                ),
            ));
        }
        if !f.siblings.is_empty() {
            return Err(Refusal::new(
                codes::SIBLINGS_MANAGED,
                format!("{id} lists siblings; the ledger maintains that list"),
                "an empty siblings list",
                "add sibling hypotheses with `dla ledger add --sibling-of <finding>`",
            ));
        }
        match id.parent() {
            Some(parent) => {
                if f.parent.as_ref() != Some(&parent) {
                    return Err(Refusal::new(
                        codes::PARENT_MISMATCH,
                        format!(
                            "{id} is a sibling of {parent} but its parent field says otherwise"
                        ),
                        format!("parent {parent}"),
                        format!(
                            "dla ledger add --origin hypothesis --evidence {} --sibling-of {parent} --hypothesis <cause> --refute-if <result>",
                            self.evidence_hint()
                        ),
                    ));
                }
                self.require_finding(&parent, "sibling").map_err(|r| {
                    Refusal::new(codes::UNKNOWN_PARENT, r.message, r.missing, r.fix)
                })?;
                if empty(&f.hypothesis) || empty(&f.refute_if) {
                    return Err(Refusal::new(
                        codes::SIBLING_FIELDS,
                        format!("sibling {id} needs both a hypothesis and a refute_if"),
                        "hypothesis and refute_if",
                        format!(
                            "dla ledger add --origin {} --evidence {} --sibling-of {parent} --hypothesis <cause> --refute-if <result>",
                            f.origin,
                            self.evidence_hint()
                        ),
                    ));
                }
            }
            None => {
                if f.parent.is_some() {
                    return Err(Refusal::new(
                        codes::PARENT_MISMATCH,
                        format!("{id} is top-level but names a parent"),
                        "no parent",
                        "use `--sibling-of` so the ledger allocates a sibling ID",
                    ));
                }
            }
        }
        if f.supports.is_empty() {
            return Err(Refusal::new(
                codes::MISSING_EVIDENCE,
                format!("{id} cites no supporting evidence"),
                "at least one evidence ID in supports",
                format!(
                    "dla ledger add --origin {} --evidence {}",
                    f.origin,
                    self.evidence_hint()
                ),
            ));
        }
        let fix = format!(
            "dla ledger add --origin {} --evidence <existing evidence id>",
            f.origin
        );
        self.require_evidence(&f.supports, &format!("finding {id}"), &fix)?;
        self.require_evidence(&f.contradicts, &format!("finding {id}"), &fix)
    }

    fn check_transition(&self, t: &Transition, version: u32) -> Result<(), Refusal> {
        let finding = self.require_finding(&t.finding, "transition")?;
        if finding.state != t.from {
            return Err(Refusal::new(
                codes::FROM_MISMATCH,
                format!(
                    "transition of {} says it leaves {}, but it is {}",
                    t.finding, t.from, finding.state
                ),
                format!("from {}", finding.state),
                "dla run status",
            ));
        }
        let ctx = TransitionContext {
            finding,
            to: t.to,
            reason: t.reason,
            remedy: t.remedy.as_deref(),
            evidence: &t.evidence,
            lookup: self,
            findings: self,
            entry: self.entry,
        };
        self.require_schema7_reason(t.reason, version, &format!("the move of {}", t.finding))?;
        match version {
            0..=1 => machine::check_transition(&M0Rules, &ctx),
            2 => machine::check_transition(&M3Rules, &ctx),
            3 => machine::check_transition(&M4Rules, &ctx),
            4..=5 => machine::check_transition(&M5Rules, &ctx),
            6 => machine::check_transition(&M6Rules, &ctx),
            _ => machine::check_transition(&M7Rules, &ctx),
        }
    }

    fn check_open_item_resolved(&self, r: &OpenItemResolved, version: u32) -> Result<(), Refusal> {
        let Some(state) = self.open_items.get(&r.id) else {
            return Err(Refusal::new(
                codes::UNKNOWN_OPEN_ITEM,
                format!("open item {} does not exist earlier in the ledger", r.id),
                format!("open item {}", r.id),
                "dla run status",
            ));
        };
        if state.resolution.is_some() {
            return Err(Refusal::new(
                codes::ALREADY_RESOLVED,
                format!("open item {} is already resolved", r.id),
                "an unresolved open item",
                "dla run status",
            ));
        }
        match &r.resolution {
            Resolution::Unresolved { remedy, reason } => {
                if remedy.trim().is_empty() {
                    return Err(Refusal::new(
                        codes::REMEDY_REQUIRED,
                        format!("resolving {} as unresolved needs a remedy", r.id),
                        "--remedy",
                        format!(
                            "dla ledger resolve {} --as unresolved --reason {} --remedy \"<text>\"",
                            r.id,
                            self.open_item_reason(&state.item)
                        ),
                    ));
                }
                if *reason == UnresolvedReason::Inconclusive {
                    self.require_schema7_reason(
                        Some(*reason),
                        version,
                        &format!("the resolution of {}", r.id),
                    )?;
                    if !self.open_item_measured(&state.item) {
                        return Err(machine::inconclusive_refusal(&r.id.to_string()));
                    }
                }
                Ok(())
            }
            Resolution::Hypothesis(f) => {
                self.require_finding(f, "open item resolution").map(|_| ())
            }
        }
    }

    /// An open item was raised by a measurement when `dla gap --residual` opened it, or
    /// when its finding has a measurement tied to the finding's family.
    fn open_item_measured(&self, item: &OpenItem) -> bool {
        item.opened_by == crate::record::RESIDUAL_OPENED_BY
            || item
                .finding
                .as_ref()
                .and_then(|f| self.findings.get(f))
                .is_some_and(|f| machine::tied_measurement(self, self, f))
    }

    /// The reason the evidence supports for closing an open item unresolved.
    fn open_item_reason(&self, item: &OpenItem) -> UnresolvedReason {
        if self.open_item_measured(item) {
            UnresolvedReason::Inconclusive
        } else {
            UnresolvedReason::NotExercised
        }
    }

    fn check_answer(&self, a: &ChecklistAnswer) -> Result<(), Refusal> {
        if !Checklist::standard().contains(&a.item) {
            return Err(Refusal::new(
                codes::UNKNOWN_CHECKLIST_ITEM,
                format!("{} is not a checklist item", a.item),
                "a checklist item ID",
                "use one of the intake.* items shown by `dla run status`",
            ));
        }
        match a.status {
            AnswerStatus::Answered | AnswerStatus::Partial => {
                if a.evidence.is_empty() {
                    return Err(Refusal::new(
                        codes::MISSING_EVIDENCE,
                        format!("{} cannot be {} without evidence", a.item, a.status),
                        "at least one evidence ID",
                        format!(
                            "dla ledger resolve {} --status {} --evidence {}",
                            a.item,
                            a.status,
                            self.evidence_hint()
                        ),
                    ));
                }
                self.require_evidence(
                    &a.evidence,
                    &format!("answer to {}", a.item),
                    &format!(
                        "dla ledger resolve {} --status {} --evidence <existing id>",
                        a.item, a.status
                    ),
                )
            }
            AnswerStatus::Unresolved => {
                if a.reason.is_none() {
                    return Err(Refusal::new(
                        codes::REASON_REQUIRED,
                        format!("{} cannot be unresolved without a reason", a.item),
                        "--reason",
                        format!(
                            "dla ledger resolve {} --status unresolved --reason <env_unavailable, tool_missing, no_harness, not_exercised, build_fails or privilege_declined> --remedy \"<text>\"",
                            a.item
                        ),
                    ));
                }
                if a.reason == Some(UnresolvedReason::Inconclusive) {
                    // An intake question is not a measurement.
                    return Err(Refusal::new(
                        codes::INVALID_VALUE,
                        format!(
                            "{} cannot be unresolved with reason inconclusive: an intake question is answered or left open for a reason, it is not measured",
                            a.item
                        ),
                        "a reason that says why the question is open",
                        format!(
                            "dla ledger resolve {} --status unresolved --reason env_unavailable --remedy \"<text>\"   (or tool_missing, no_harness, not_exercised, build_fails, privilege_declined)",
                            a.item
                        ),
                    ));
                }
                if empty(&a.remedy) {
                    return Err(Refusal::new(
                        codes::REMEDY_REQUIRED,
                        format!("{} cannot be unresolved without a remedy", a.item),
                        "--remedy",
                        format!(
                            "dla ledger resolve {} --status unresolved --reason {} --remedy \"<text>\"",
                            a.item,
                            a.reason.unwrap_or(UnresolvedReason::NotExercised)
                        ),
                    ));
                }
                self.require_evidence(
                    &a.evidence,
                    &format!("answer to {}", a.item),
                    &format!(
                        "dla ledger resolve {} --status unresolved --evidence <existing id>",
                        a.item
                    ),
                )
            }
        }
    }

    /// Fills `shape` and `region` of a just-confirmed leak-side finding from its
    /// growth evidence. Done during replay, so no record field carries it.
    fn derive_on_confirm(&mut self, id: &FindingId) {
        let Some(f) = self.findings.get(id) else {
            return;
        };
        let derived = machine::derive_attributes(f, self);
        if let Some(f) = self.findings.get_mut(id) {
            f.shape = f.shape.or(derived.shape);
            f.region = f.region.or(derived.region);
        }
    }

    /// Applies a record that [`Index::check`] accepted. Cannot fail.
    fn commit(&mut self, record: &Record, version: u32) {
        match record {
            Record::Evidence(e) => {
                self.evidence.insert(e.evidence_id.clone(), e.clone());
            }
            Record::FindingAdded { finding } => {
                if let Some(parent) = finding.id.parent()
                    && let Some(p) = self.findings.get_mut(&parent)
                {
                    p.siblings.push(finding.id.clone());
                }
                self.findings.insert(finding.id.clone(), finding.clone());
            }
            Record::Transition(t) => {
                if let Some(f) = self.findings.get_mut(&t.finding) {
                    f.state = t.to;
                    f.reason = None;
                    f.remedy = None;
                    if t.to.needs_reason() {
                        f.reason = t.reason;
                        f.remedy = t.remedy.clone();
                    }
                    let list = if t.to == FindingState::Refuted {
                        &mut f.contradicts
                    } else {
                        &mut f.supports
                    };
                    for e in &t.evidence {
                        if !list.contains(e) {
                            list.push(e.clone());
                        }
                    }
                }
                if version >= 2 && t.to == FindingState::Confirmed {
                    self.derive_on_confirm(&t.finding);
                }
            }
            Record::OpenItem(o) => {
                self.open_items.insert(
                    o.id.clone(),
                    OpenItemState {
                        item: o.clone(),
                        resolution: None,
                    },
                );
            }
            Record::OpenItemResolved(r) => {
                if let Some(s) = self.open_items.get_mut(&r.id) {
                    s.resolution = Some(r.resolution.clone());
                }
            }
            Record::ChecklistAnswer(a) => {
                self.answers.insert(a.item.clone(), a.clone());
            }
            Record::FindingAnnotated(a) => {
                if let Some(f) = self.findings.get_mut(&a.finding) {
                    match (f.holder, a.holder) {
                        (None, h) => f.holder = h,
                        // The check let this pass: the change is backed by evidence.
                        (Some(old), Some(new)) if version >= 5 && old != new => {
                            f.holder = Some(new);
                            self.annotations
                                .entry(a.finding.clone())
                                .or_default()
                                .holder_changes
                                .push(HolderChange {
                                    from: old,
                                    to: new,
                                    evidence: a.evidence.clone(),
                                });
                        }
                        _ => {}
                    }
                    f.region = f.region.or(a.region);
                    if f.site.is_none() {
                        f.site.clone_from(&a.site);
                    }
                    if f.owner.is_none() {
                        f.owner.clone_from(&a.owner);
                    }
                }
                let slot = self.annotations.entry(a.finding.clone()).or_default();
                for (list, new) in [
                    (&mut slot.writers, &a.writers),
                    (&mut slot.removers, &a.removers),
                ] {
                    for s in new {
                        if !list.contains(s) {
                            list.push(s.clone());
                        }
                    }
                }
                if let Some(n) = &a.note
                    && !slot.notes.contains(n)
                {
                    slot.notes.push(n.clone());
                }
            }
            Record::SliceStarted(s) => {
                self.slices.entry(s.worker.clone()).or_default().started = Some(s.clone());
            }
            Record::SliceDone(d) => {
                self.slices.entry(d.worker.clone()).or_default().done = Some(d.clone());
            }
            Record::SliceMerged(m) => {
                self.slices.entry(m.worker.clone()).or_default().merged = Some(m.clone());
            }
            Record::FindingMerged(m) => {
                let extra: Vec<EvidenceId> = self
                    .findings
                    .get(&m.duplicate)
                    .map(|d| d.supports.clone())
                    .unwrap_or_default();
                if let Some(c) = self.findings.get_mut(&m.canonical) {
                    for e in extra {
                        if !c.supports.contains(&e) {
                            c.supports.push(e);
                        }
                    }
                }
                // The canonical may have been a duplicate of the new duplicate (a swap),
                // and the new duplicate may have duplicates of its own: point them all
                // at the canonical so the chain stays one level deep.
                self.merged_into.remove(&m.canonical);
                for target in self.merged_into.values_mut() {
                    if *target == m.duplicate {
                        target.clone_from(&m.canonical);
                    }
                }
                self.merged_into
                    .insert(m.duplicate.clone(), m.canonical.clone());
            }
            Record::Verification(v) => {
                self.verifications
                    .entry(v.finding.clone())
                    .or_default()
                    .push(v.clone());
            }
            Record::FindingLabeled(l) => {
                if let Some(f) = self.findings.get_mut(&l.finding) {
                    f.archetype.clone_from(&l.label);
                }
                self.labels
                    .entry(l.finding.clone())
                    .or_default()
                    .push(l.clone());
            }
            Record::FindingExplained(x) => {
                if let Some(f) = self.findings.get_mut(&x.finding) {
                    f.explanation = Some(Explanation {
                        text: x.text.clone(),
                        tag: x.tag,
                    });
                }
                self.explanations.insert(x.finding.clone(), x.clone());
            }
            Record::IntegrityResealed(r) => self.reseals.push(r.clone()),
        }
    }
}

/// A replayed ledger, optionally bound to a file for appending.
///
/// In worker mode (see [`Ledger::load_for`]) it holds the main ledger (read only) plus
/// the worker's slice; [`Ledger::lines`] then returns the slice's own lines, which
/// have their own `seq`, and every write goes to the slice.
#[derive(Debug, Clone)]
pub struct Ledger {
    path: Option<PathBuf>,
    lines: Vec<LedgerLine>,
    index: Index,
    ends_with_newline: bool,
    slice: Option<SliceMode>,
    torn_tail: Option<TornTail>,
    /// Bytes of the main ledger file this replay covers; a write finding a different
    /// size under the lock replays the file again first (another process appended).
    disk_len: Option<u64>,
    /// The main ledger's write lock, held for the length of one write operation.
    main_lock: Option<std::sync::Arc<fs::File>>,
}

/// What a ledger opened for a worker adds to a plain one.
#[derive(Debug, Clone)]
struct SliceMode {
    worker: String,
    run_dir: Option<PathBuf>,
    /// Lines of the main ledger replayed under the slice.
    base_len: u64,
    /// Bytes of the slice file that hold complete lines.
    valid_bytes: u64,
    /// A torn fragment follows them and the next write cuts it off.
    torn_pending: bool,
    /// The slice's write lock, held while this ledger lives (see [`slice::lock`]).
    lock: Option<std::sync::Arc<fs::File>>,
}

/// A record on its way into the file.
struct Pending {
    record: Record,
    ts: Option<DateTime<Utc>>,
    worker: Option<String>,
    version: u32,
}

impl Pending {
    fn new(record: Record) -> Self {
        Self {
            record,
            ts: None,
            worker: None,
            version: LEDGER_SCHEMA_VERSION,
        }
    }
}

fn replay_err(path: Option<&Path>, line: u64, malformed: bool, refusal: Refusal) -> Error {
    Error::Replay(Box::new(ReplayError {
        path: path.map(Path::to_path_buf),
        line,
        malformed,
        refusal,
    }))
}

/// The entry path recorded in `state.json`; `repository` when there is none.
fn entry_of(run_dir: &Path) -> EntryPath {
    RunState::load(run_dir).map_or(EntryPath::Repository, |s| s.entry)
}

impl Ledger {
    fn empty(path: Option<PathBuf>, entry: EntryPath) -> Self {
        Self {
            path,
            lines: Vec::new(),
            index: Index::new(entry),
            ends_with_newline: true,
            slice: None,
            torn_tail: None,
            disk_len: None,
            main_lock: None,
        }
    }

    /// `<run_dir>/ledger.jsonl`.
    pub fn path_in(run_dir: &Path) -> PathBuf {
        run_dir.join(LEDGER_FILE)
    }

    /// Creates an empty ledger file in `run_dir` (which must exist). Fails if one exists.
    /// The entry path is `repository` until [`Ledger::set_entry`] says otherwise.
    pub fn create(run_dir: &Path) -> Result<Self, Error> {
        let path = Self::path_in(run_dir);
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| Error::io(&path, e))?;
        crate::integrity::seal_file(run_dir, LEDGER_FILE)?;
        let mut ledger = Self::empty(Some(path), entry_of(run_dir));
        ledger.disk_len = Some(0);
        Ok(ledger)
    }

    /// Replays `<run_dir>/ledger.jsonl`, validating every line. The entry path comes
    /// from `state.json` (`repository` when there is none).
    pub fn load(run_dir: &Path) -> Result<Self, Error> {
        let path = Self::path_in(run_dir);
        let text = fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
        Self::replay(&text, Some(path), entry_of(run_dir))
    }

    /// Opens the main ledger (`worker` is `None`) or, in worker mode, the worker's
    /// slice `ledger.<worker>.jsonl` replayed on top of the main ledger; appends then go
    /// to the slice and new IDs carry the `<worker>/` prefix.
    ///
    /// The main ledger is replayed read only and strictly, up to the `base_seq` the
    /// slice recorded when it started, so a slice line cannot cite a main ID created
    /// later. A slice whose final line was torn by a kill loses that line (see
    /// [`Ledger::torn_tail`]); every other defect in a slice is an error. The first
    /// write starts the slice with a `slice_started` record and marks the worker's
    /// entry in `state.json` `running`.
    pub fn load_for(run_dir: &Path, worker: Option<&str>) -> Result<Self, Error> {
        let Some(w) = worker else {
            return Self::load(run_dir);
        };
        if !crate::ids::valid_worker_name(w) {
            return Err(bad_worker(w).into());
        }
        // The lock covers the replay and every write made through the result, so
        // parallel calls of one worker never allocate the same seq or ID.
        let lock = slice::lock(run_dir, w)?;
        let sp = slice::slice_path(run_dir, w);
        let text = match slice::read_text(&sp) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(Error::io(&sp, e)),
        };
        let mut ledger = Self::load_slice_from(run_dir, w, &text)?;
        if let Some(sm) = &mut ledger.slice {
            sm.lock = Some(lock);
        }
        Ok(ledger)
    }

    /// Like [`Ledger::load_for`] for slice text already in memory (the merge reads the
    /// file once and replays that snapshot).
    pub fn load_slice_from(run_dir: &Path, worker: &str, slice_text: &str) -> Result<Self, Error> {
        if !crate::ids::valid_worker_name(worker) {
            return Err(bad_worker(worker).into());
        }
        let main_path = Self::path_in(run_dir);
        let main_text = fs::read_to_string(&main_path).map_err(|e| Error::io(&main_path, e))?;
        let sp = slice::slice_path(run_dir, worker);
        let cut = slice::split(slice_text);
        let base_seq = cut
            .lines
            .first()
            .and_then(|l| serde_json::from_str::<serde_json::Value>(l).ok())
            .filter(|v| v.get("type").and_then(|t| t.as_str()) == Some("slice_started"))
            .and_then(|v| v.get("base_seq").and_then(|b| b.as_u64()));
        let main_lines: Vec<&str> = main_text.lines().collect();
        let limit = base_seq.map(|b| usize::try_from(b).unwrap_or(usize::MAX));
        let mut main = Self::empty(Some(main_path.clone()), entry_of(run_dir));
        main.replay_lines(&main_lines, Some(&main_path), limit)?;
        let base_len = main.lines.len() as u64;
        if let Some(b) = base_seq
            && b > base_len
        {
            return Err(replay_err(
                Some(&sp),
                1,
                false,
                Refusal::new(
                    m4::SLICE_STATE,
                    format!(
                        "slice of {worker} says the main ledger had {b} lines when it started, but it has {base_len}"
                    ),
                    "the main ledger the slice was started against",
                    "restore ledger.jsonl, or start a new run with `dla run init`",
                ),
            ));
        }
        let mut ledger = Self {
            path: Some(sp.clone()),
            lines: Vec::new(),
            index: main.index,
            ends_with_newline: slice_text.is_empty()
                || slice_text.ends_with('\n')
                || cut.torn.is_some(),
            slice: Some(SliceMode {
                worker: worker.to_owned(),
                run_dir: Some(run_dir.to_path_buf()),
                base_len,
                valid_bytes: cut.valid_bytes() as u64,
                torn_pending: cut.torn.is_some(),
                lock: None,
            }),
            torn_tail: cut.torn.clone(),
            disk_len: None,
            main_lock: None,
        };
        ledger.index.slice_worker = Some(worker.to_owned());
        ledger.replay_lines(&cut.lines, Some(&sp), None)?;
        Ok(ledger)
    }

    /// Replays ledger text that is not bound to a file (for tests and renderers).
    /// Mutating methods on the result only change memory. The entry path is
    /// `repository`; see [`Ledger::from_text_for`].
    pub fn from_text(text: &str) -> Result<Self, Error> {
        Self::replay(text, None, EntryPath::Repository)
    }

    /// Like [`Ledger::from_text`] for a run with the given entry path.
    pub fn from_text_for(text: &str, entry: EntryPath) -> Result<Self, Error> {
        Self::replay(text, None, entry)
    }

    /// Sets the entry path the artifact rules look at (see [`Ledger::create`]).
    pub fn set_entry(&mut self, entry: EntryPath) {
        self.index.entry = entry;
    }

    fn replay(text: &str, path: Option<PathBuf>, entry: EntryPath) -> Result<Self, Error> {
        let mut ledger = Self::empty(path.clone(), entry);
        ledger.disk_len = path.as_ref().map(|_| text.len() as u64);
        ledger.ends_with_newline = text.is_empty() || text.ends_with('\n');
        let lines: Vec<&str> = text.lines().collect();
        ledger.replay_lines(&lines, path.as_deref(), None)?;
        Ok(ledger)
    }

    /// Replays `lines` (at most `limit` of them) on top of what is already replayed.
    fn replay_lines(
        &mut self,
        lines: &[&str],
        p: Option<&Path>,
        limit: Option<usize>,
    ) -> Result<(), Error> {
        let take = limit.map_or(lines.len(), |l| l.min(lines.len()));
        for (i, raw) in lines[..take].iter().enumerate() {
            let n = (i + 1) as u64;
            if raw.trim().is_empty() {
                return Err(replay_err(
                    p,
                    n,
                    false,
                    Refusal::new(
                        codes::BLANK_LINE,
                        "blank line in ledger",
                        "one JSON record per line",
                        format!("delete blank line {n}"),
                    ),
                ));
            }
            let malformed = |msg: String| {
                replay_err(
                    p,
                    n,
                    true,
                    Refusal::new(
                        codes::MALFORMED,
                        msg,
                        "a valid ledger record",
                        format!(
                            "correct or delete line {n}; ledger records are written by `dla ledger add` and `dla ledger resolve`"
                        ),
                    ),
                )
            };
            let doc: serde_json::Value =
                strict::parse(raw).map_err(|e| malformed(format!("invalid JSON: {e}")))?;
            let doc = upgrade(doc, Artifact::LedgerLine, &format!("ledger line {n}")).map_err(
                |e| match e {
                    UpgradeError::Malformed(m) => malformed(m),
                    UpgradeError::Schema(s) => Error::Schema(s),
                },
            )?;
            let line: LedgerLine = serde_json::from_value(doc.clone())
                .map_err(|e| malformed(format!("not a valid ledger record: {e}")))?;
            if let Ok(back) = serde_json::to_value(&line) {
                let unknown = strict::unknown_fields(&doc, &back);
                if !unknown.is_empty() {
                    return Err(malformed(format!(
                        "unknown field(s) {}: not part of this record type (misspelled?)",
                        unknown
                            .iter()
                            .map(|u| format!("`{u}`"))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )));
                }
            }
            let expected = self.next_seq();
            if line.seq != expected {
                let repair = "dla has no command that rewrites ledger history. Keep a copy of ledger.jsonl, edit it by hand so seq values run 1, 2, 3 without gaps and no record repeats, then run `dla run reseal --reason <what you changed>` (the audit line records it); or start a new run with `dla run init`".to_owned();
                let (message, fix) = if line.seq < expected && line.seq >= 1 {
                    (
                        format!(
                            "seq {} appears on line {n} and on line {} (two writers appended at the same time, which a dla without the ledger lock could do)",
                            line.seq, line.seq
                        ),
                        format!(
                            "lines 1 to {} are intact; line {n} and the lines after it repeat or follow a duplicate. {repair}",
                            n - 1
                        ),
                    )
                } else {
                    (
                        format!(
                            "seq is {} but {expected} was expected (gap or reordering)",
                            line.seq
                        ),
                        format!(
                            "renumber line {n} so seq values run 1, 2, 3 without gaps. {repair}"
                        ),
                    )
                };
                return Err(replay_err(
                    p,
                    n,
                    false,
                    Refusal::new(codes::BAD_SEQ, message, format!("seq {expected}"), fix),
                ));
            }
            self.index
                .check(&line.record, line.schema_version)
                .map_err(|mut r| {
                    if let Some(sm) = &self.slice
                        && matches!(
                            r.code.as_str(),
                            codes::UNKNOWN_EVIDENCE
                                | codes::UNKNOWN_FINDING
                                | codes::UNKNOWN_PARENT
                                | codes::UNKNOWN_OPEN_ITEM
                        )
                    {
                        r.message = format!(
                            "{} (slice of {} may cite main-ledger items only up to line {})",
                            r.message, sm.worker, sm.base_len
                        );
                    }
                    replay_err(p, n, false, r)
                })?;
            self.index.commit(&line.record, line.schema_version);
            self.lines.push(line);
        }
        Ok(())
    }

    fn next_seq(&self) -> u64 {
        self.lines.len() as u64 + 1
    }

    // ----- read access -----

    /// The file appends go to (the slice in worker mode), when bound to one.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    /// Every line of the file in order (the slice's own lines in worker mode).
    pub fn lines(&self) -> &[LedgerLine] {
        &self.lines
    }
    /// The worker whose slice this ledger appends to; `None` for the main ledger.
    pub fn worker(&self) -> Option<&str> {
        self.slice.as_ref().map(|s| s.worker.as_str())
    }
    /// Lines of the main ledger replayed under a slice (its `base_seq`); 0 in core mode.
    pub fn base_len(&self) -> u64 {
        self.slice.as_ref().map_or(0, |s| s.base_len)
    }
    /// The torn final line dropped when the slice was replayed, if any.
    pub fn torn_tail(&self) -> Option<&TornTail> {
        self.torn_tail.as_ref()
    }
    /// The run's entry path as the rules see it.
    pub fn entry(&self) -> EntryPath {
        self.index.entry
    }
    /// What the ledger records about `worker`'s slice, when it has any record of it.
    pub fn slice_info(&self, worker: &str) -> Option<&SliceRecord> {
        self.index.slices.get(worker)
    }
    /// Every slice the ledger records, by worker.
    pub fn slices(&self) -> impl Iterator<Item = (&str, &SliceRecord)> {
        self.index.slices.iter().map(|(w, r)| (w.as_str(), r))
    }
    /// Slice lines of `worker` already merged into this ledger.
    pub fn merged_lines(&self, worker: &str) -> u64 {
        self.index
            .slices
            .get(worker)
            .and_then(|r| r.merged.as_ref())
            .map_or(0, |m| m.lines)
    }
    /// True when the last merge of `worker`'s slice saw its torn final line and dropped it.
    pub fn merged_torn_tail(&self, worker: &str) -> bool {
        self.index
            .slices
            .get(worker)
            .and_then(|r| r.merged.as_ref())
            .is_some_and(|m| m.torn_tail)
    }
    /// The finding `id` was merged into as a duplicate, if it is one.
    pub fn merged_into(&self, id: &FindingId) -> Option<&FindingId> {
        self.index.merged_into.get(id)
    }
    /// The duplicates merged into `canonical`, in ID order.
    pub fn duplicates_of(&self, canonical: &FindingId) -> Vec<&FindingId> {
        self.index
            .merged_into
            .iter()
            .filter(|(_, c)| *c == canonical)
            .map(|(d, _)| d)
            .collect()
    }
    /// The verifications recorded for `id`, oldest first.
    pub fn verifications(&self, id: &FindingId) -> &[Verification] {
        self.index.verifications.get(id).map_or(&[], Vec::as_slice)
    }
    /// The `finding_labeled` records of `id`, oldest first (schema 6).
    pub fn labels(&self, id: &FindingId) -> &[FindingLabeled] {
        self.index.labels.get(id).map_or(&[], Vec::as_slice)
    }
    /// The latest label record of `id`, with the proof it was accepted on.
    pub fn label_record(&self, id: &FindingId) -> Option<&FindingLabeled> {
        self.index.labels.get(id).and_then(|l| l.last())
    }
    /// The latest explanation record of `id` (schema 6); earlier ones stay in the file.
    pub fn explanation(&self, id: &FindingId) -> Option<&FindingExplained> {
        self.index.explanations.get(id)
    }
    /// Every `integrity_resealed` record, oldest first (schema 6).
    pub fn reseals(&self) -> &[IntegrityResealed] {
        &self.index.reseals
    }
    /// All evidence, ordered by ID.
    pub fn evidence(&self) -> impl Iterator<Item = &EvidenceRecord> {
        self.index.evidence.values()
    }
    /// One evidence record.
    pub fn evidence_by_id(&self, id: &EvidenceId) -> Option<&EvidenceRecord> {
        self.index.evidence.get(id)
    }
    /// All findings (replayed state), ordered by ID.
    pub fn findings(&self) -> impl Iterator<Item = &Finding> {
        self.index.findings.values()
    }
    /// One finding.
    pub fn finding(&self, id: &FindingId) -> Option<&Finding> {
        self.index.findings.get(id)
    }
    /// The reason the evidence supports for closing a finding unresolved: `inconclusive`
    /// when a measurement is tied to its family, else `not_exercised`.
    pub fn supported_reason(&self, id: &FindingId) -> UnresolvedReason {
        match self.index.findings.get(id) {
            Some(f) => machine::supported_reason(&self.index, &self.index, f),
            None => UnresolvedReason::NotExercised,
        }
    }
    /// The reason the evidence supports for closing an open item unresolved.
    pub fn open_item_supported_reason(&self, id: &OpenItemId) -> UnresolvedReason {
        match self.index.open_items.get(id) {
            Some(o) => self.index.open_item_reason(&o.item),
            None => UnresolvedReason::NotExercised,
        }
    }
    /// All open items, ordered by ID.
    pub fn open_items(&self) -> impl Iterator<Item = &OpenItemState> {
        self.index.open_items.values()
    }
    /// One open item.
    pub fn open_item(&self, id: &OpenItemId) -> Option<&OpenItemState> {
        self.index.open_items.get(id)
    }
    /// The merged writers, removers and notes recorded for a finding, if any.
    pub fn annotation(&self, id: &FindingId) -> Option<&Annotation> {
        self.index.annotations.get(id)
    }
    /// The latest answer per checklist item ID.
    pub fn answers(&self) -> &BTreeMap<String, ChecklistAnswer> {
        &self.index.answers
    }
    /// The standard checklist with the latest answers.
    pub fn checklist(&self) -> Vec<ChecklistEntry> {
        Checklist::standard()
            .items
            .into_iter()
            .map(|item| ChecklistEntry {
                answer: self.index.answers.get(&item.id).cloned(),
                item,
            })
            .collect()
    }

    // ----- IDs -----

    /// The prefix new IDs carry: the worker's name in worker mode.
    pub(crate) fn prefix(&self) -> Option<&str> {
        self.slice.as_ref().map(|s| s.worker.as_str())
    }

    /// In worker mode, maps an ID typed without a prefix to the worker's own (`F3` to
    /// `jvm/F3`) when that exists, or when no main-ledger `F3` exists to mean instead.
    /// Prefixed IDs, and every ID in core mode, are returned unchanged.
    pub fn own_finding_id(&self, id: &FindingId) -> FindingId {
        match self.prefix() {
            Some(w) if id.prefix().is_none() => {
                let own = id.reprefixed(Some(w));
                if self.index.findings.contains_key(&own) || !self.index.findings.contains_key(id) {
                    own
                } else {
                    id.clone()
                }
            }
            _ => id.clone(),
        }
    }
    /// Like [`Ledger::own_finding_id`] for evidence.
    pub fn own_evidence_id(&self, id: &EvidenceId) -> EvidenceId {
        match self.prefix() {
            Some(w) if id.prefix().is_none() => {
                let own = id.reprefixed(Some(w));
                if self.index.evidence.contains_key(&own) || !self.index.evidence.contains_key(id) {
                    own
                } else {
                    id.clone()
                }
            }
            _ => id.clone(),
        }
    }
    /// Like [`Ledger::own_finding_id`] for open items.
    pub fn own_open_item_id(&self, id: &OpenItemId) -> OpenItemId {
        match self.prefix() {
            Some(w) if id.prefix().is_none() => {
                let own = id.reprefixed(Some(w));
                if self.index.open_items.contains_key(&own)
                    || !self.index.open_items.contains_key(id)
                {
                    own
                } else {
                    id.clone()
                }
            }
            _ => id.clone(),
        }
    }

    // ----- ID allocation: next n = max existing + 1, among IDs with this ledger's prefix -----

    /// The ID the next evidence record will get.
    pub fn next_evidence_id(&self) -> EvidenceId {
        let max = self
            .index
            .evidence
            .keys()
            .filter(|e| e.prefix() == self.prefix())
            .map(EvidenceId::number)
            .max()
            .unwrap_or(0);
        EvidenceId::with_prefix(self.prefix(), max + 1)
    }
    /// The ID the next top-level finding will get.
    pub fn next_finding_id(&self) -> FindingId {
        let max = self
            .index
            .findings
            .keys()
            .filter(|f| f.prefix() == self.prefix())
            .map(FindingId::number)
            .max()
            .unwrap_or(0);
        FindingId::with_prefix(self.prefix(), max + 1)
    }
    /// The ID the next sibling of `parent` will get (`F<n>.n<k>`).
    pub fn next_sibling_id(&self, parent: &FindingId) -> FindingId {
        let max = self
            .index
            .findings
            .keys()
            .filter(|f| f.parent().as_ref() == Some(parent))
            .filter_map(FindingId::sibling_number)
            .max()
            .unwrap_or(0);
        parent.sibling(max + 1)
    }
    /// The ID the next open item will get.
    pub fn next_open_item_id(&self) -> OpenItemId {
        let max = self
            .index
            .open_items
            .keys()
            .filter(|o| o.prefix() == self.prefix())
            .map(OpenItemId::number)
            .max()
            .unwrap_or(0);
        OpenItemId::with_prefix(self.prefix(), max + 1)
    }

    // ----- writers -----

    /// Marks the worker's entry in `state.json` `running` (when it is `registered`).
    fn mark_running(&self) -> Result<(), Error> {
        let Some(sm) = &self.slice else {
            return Ok(());
        };
        let Some(dir) = &sm.run_dir else {
            return Ok(());
        };
        if !dir.join(crate::state::STATE_FILE).is_file() {
            return Ok(());
        }
        RunState::update(dir, |s| {
            if let Some(w) = s.workers.iter_mut().find(|w| w.name == sm.worker)
                && w.status == WorkerStatus::Registered
            {
                w.status = WorkerStatus::Running;
                w.started_at = Some(now_utc());
            }
            Ok(())
        })
    }

    /// The `slice_started` record for this worker, from its registry entry when it has
    /// one (else the runtime is the worker name and the deadline the run's).
    fn slice_started(&self) -> Record {
        let (worker, base_seq, run_dir) = self
            .slice
            .as_ref()
            .map(|s| (s.worker.clone(), s.base_len, s.run_dir.clone()))
            .unwrap_or_default();
        let state = run_dir.as_deref().and_then(|d| RunState::load(d).ok());
        let entry = state.as_ref().and_then(|s| s.worker(&worker));
        Record::SliceStarted(SliceStarted {
            runtime: entry.map_or_else(|| worker.clone(), |w| w.runtime.clone()),
            deadline: entry
                .map(|w| w.deadline)
                .or_else(|| state.as_ref().map(|s| s.budget.deadline))
                .unwrap_or_else(now_utc),
            worker,
            base_seq,
        })
    }

    /// Validates `pending` against the replayed state, appends it to the file in one
    /// write and applies it. Nothing is written or applied when any record is refused.
    fn append_all(&mut self, pending: Vec<Pending>) -> Result<Vec<LedgerLine>, Error> {
        // Only reached outside `locked` by a caller that already built its records from
        // a view it did not refresh; every public writer goes through `locked`.
        self.locked(|l| l.append_all_held(pending))
    }

    /// Runs one write operation on the main ledger under its exclusive lock. The file
    /// is replayed again when another process appended since this handle was loaded,
    /// so the `seq`, the IDs and the checks `f` makes all come from the file's current
    /// tail, and the line and its seal are written before the lock is released. A
    /// worker slice holds its own lock for the life of the handle; an unbound ledger
    /// has no file.
    fn locked<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T, Error>) -> Result<T, Error> {
        if self.slice.is_some() || self.main_lock.is_some() {
            return f(self);
        }
        let Some(path) = self.path.clone() else {
            return f(self);
        };
        let dir = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let lock = crate::integrity::lock_ledger(&dir)?;
        let size = fs::metadata(&path).map_err(|e| Error::io(&path, e))?.len();
        if self.disk_len != Some(size) {
            let text = fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
            let fresh = Self::replay(&text, Some(path.clone()), self.index.entry)?;
            self.lines = fresh.lines;
            self.index = fresh.index;
            self.ends_with_newline = fresh.ends_with_newline;
            self.disk_len = fresh.disk_len;
        }
        self.main_lock = Some(std::sync::Arc::new(lock));
        let out = f(self);
        self.main_lock = None;
        out
    }

    fn append_all_held(&mut self, pending: Vec<Pending>) -> Result<Vec<LedgerLine>, Error> {
        let mut trial = (pending.len() > 1).then(|| self.index.clone());
        let first_seq = self.next_seq();
        let mut new_lines = Vec::with_capacity(pending.len());
        for (seq, p) in (first_seq..).zip(pending) {
            let idx = trial.as_ref().unwrap_or(&self.index);
            idx.check(&p.record, p.version)?;
            let line = LedgerLine {
                schema_version: p.version,
                seq,
                ts: p.ts.unwrap_or_else(now_utc),
                worker: p.worker,
                record: p.record,
            };
            if let Some(t) = trial.as_mut() {
                t.commit(&line.record, line.schema_version);
            }
            new_lines.push(line);
        }
        if let Some(path) = &self.path {
            // The write and its seal go together under one lock: the main ledger's own
            // lock, or the slice lock the handle already holds.
            let run_dir = path.parent().map(Path::to_path_buf);
            debug_assert!(
                self.slice.is_some() || self.main_lock.is_some(),
                "a main-ledger write runs inside `locked`"
            );
            let guard = match (&run_dir, path.file_name().and_then(|n| n.to_str())) {
                (Some(d), Some(name)) => Some(crate::integrity::Guard::new(d, name)),
                _ => None,
            };
            let cut_torn = self
                .slice
                .as_ref()
                .filter(|s| s.torn_pending)
                .map(|s| s.valid_bytes);
            let mut text = String::new();
            if cut_torn.is_none() && !self.ends_with_newline {
                text.push('\n');
            }
            let mut added = 0u64;
            for line in &new_lines {
                let json = serde_json::to_string(line).map_err(|e| Error::Malformed {
                    path: path.clone(),
                    message: e.to_string(),
                })?;
                added += json.len() as u64 + 1;
                text.push_str(&json);
                text.push('\n');
            }
            if let Some(keep) = cut_torn {
                // Drop the half-written fragment a killed process left; it was never a line.
                let f = OpenOptions::new()
                    .write(true)
                    .open(path)
                    .map_err(|e| Error::io(path, e))?;
                f.set_len(keep).map_err(|e| Error::io(path, e))?;
            }
            let mut f = OpenOptions::new()
                .append(true)
                .create(self.slice.is_some())
                .open(path)
                .map_err(|e| Error::io(path, e))?;
            f.write_all(text.as_bytes())
                .and_then(|()| f.sync_data())
                .map_err(|e| Error::io(path, e))?;
            if let Some(g) = guard {
                g.seal()?;
            }
            if self.slice.is_none() {
                self.disk_len = fs::metadata(path).ok().map(|m| m.len());
            }
            self.ends_with_newline = true;
            if let Some(sm) = &mut self.slice {
                sm.torn_pending = false;
                sm.valid_bytes += added;
            }
        }
        match trial {
            Some(t) => self.index = t,
            None => {
                for l in &new_lines {
                    self.index.commit(&l.record, l.schema_version);
                }
            }
        }
        self.lines.extend(new_lines.iter().cloned());
        Ok(new_lines)
    }

    fn append(&mut self, record: Record) -> Result<LedgerLine, Error> {
        let first_of_slice = self.slice.is_some() && self.lines.is_empty();
        let mut batch = Vec::new();
        if first_of_slice {
            batch.push(Pending::new(self.slice_started()));
        }
        batch.push(Pending::new(record));
        if first_of_slice {
            // Refuse before touching the registry or the file.
            let mut trial = self.index.clone();
            for p in &batch {
                trial.check(&p.record, p.version)?;
                trial.commit(&p.record, p.version);
            }
            self.mark_running()?;
        }
        let mut written = self.append_all(batch)?;
        Ok(written.pop().expect("a batch of at least one record"))
    }

    /// Appends an evidence record with a freshly allocated ID.
    pub fn add_evidence(&mut self, new: NewEvidence) -> Result<LedgerLine, Error> {
        self.locked(|this| this.add_evidence_held(new))
    }

    fn add_evidence_held(&mut self, new: NewEvidence) -> Result<LedgerLine, Error> {
        let rec = new.into_record(self.next_evidence_id());
        self.append(Record::Evidence(rec))
    }

    /// Appends a finding in state `candidate`, allocating `F<n>` or `F<n>.n<k>`.
    pub fn add_finding(&mut self, new: NewFinding) -> Result<LedgerLine, Error> {
        self.locked(|this| this.add_finding_held(new))
    }

    fn add_finding_held(&mut self, mut new: NewFinding) -> Result<LedgerLine, Error> {
        new.supports = new
            .supports
            .iter()
            .map(|e| self.own_evidence_id(e))
            .collect();
        new.sibling_of = new.sibling_of.as_ref().map(|p| self.own_finding_id(p));
        if let (Some(w), Some(parent)) = (self.worker(), &new.sibling_of)
            && parent.prefix() != Some(w)
        {
            return Err(Refusal::new(
                m4::FOREIGN_ITEM,
                format!(
                    "worker {w} cannot add a sibling to {parent}: that finding belongs to the main ledger or another worker"
                ),
                format!("a finding of {w} as --sibling-of"),
                format!(
                    "dla --worker {w} ledger add --origin hypothesis --evidence <id> --hypothesis <cause> --refute-if <result>   (a finding of your own)"
                ),
            )
            .into());
        }
        if let Some(a) = &new.archetype
            && a != UNCLASSIFIED
        {
            return Err(Refusal::new(
                codes::ARCHETYPE_LOCKED,
                format!("`ledger add` cannot set archetype {a:?}"),
                "archetype UNCLASSIFIED",
                "omit the archetype; only `dla ledger label` assigns one",
            )
            .into());
        }
        let id = match &new.sibling_of {
            Some(parent) => {
                self.index
                    .require_finding(parent, "--sibling-of")
                    .map_err(|r| {
                        Refusal::new(codes::UNKNOWN_PARENT, r.message, r.missing, r.fix)
                    })?;
                if parent.is_sibling() {
                    return Err(Refusal::new(
                        codes::SIBLING_FIELDS,
                        format!("{parent} is itself a sibling; siblings attach to a top-level finding"),
                        "a top-level parent finding",
                        format!(
                            "dla ledger add --origin {} --evidence {} --sibling-of {} --hypothesis <cause> --refute-if <result>",
                            new.origin,
                            self.index.evidence_hint(),
                            parent.parent().map(|p| p.to_string()).unwrap_or_default()
                        ),
                    )
                    .into());
                }
                self.next_sibling_id(parent)
            }
            None => self.next_finding_id(),
        };
        let mut f = Finding::candidate(id, new.origin, new.supports);
        f.parent = new.sibling_of;
        f.suggested_archetype = new.suggested_archetype;
        f.hypothesis = new.hypothesis;
        f.refute_if = new.refute_if;
        f.holder = new.holder;
        f.owner = new.owner;
        f.region = new.region;
        f.shape = new.shape;
        f.locus = new.locus;
        f.site = new.site;
        f.amplifiers = new.amplifiers;
        f.repair_class = new.repair_class;
        f.explanation = new.explanation;
        self.append(Record::FindingAdded { finding: f })
    }

    /// Moves a finding to `to`. `from` is taken from the replayed state.
    pub fn transition(
        &mut self,
        finding: &FindingId,
        to: FindingState,
        reason: Option<crate::finding::UnresolvedReason>,
        remedy: Option<String>,
        evidence: Vec<EvidenceId>,
        by: &str,
    ) -> Result<LedgerLine, Error> {
        self.locked(|this| this.transition_held(finding, to, reason, remedy, evidence, by))
    }

    fn transition_held(
        &mut self,
        finding: &FindingId,
        to: FindingState,
        reason: Option<crate::finding::UnresolvedReason>,
        remedy: Option<String>,
        evidence: Vec<EvidenceId>,
        by: &str,
    ) -> Result<LedgerLine, Error> {
        let finding = &self.own_finding_id(finding);
        let evidence: Vec<EvidenceId> = evidence.iter().map(|e| self.own_evidence_id(e)).collect();
        let from = self
            .index
            .require_finding(finding, "transition")
            .map_err(Error::Refused)?
            .state;
        self.append(Record::Transition(Transition {
            finding: finding.clone(),
            from,
            to,
            reason,
            remedy,
            evidence,
            by: by.to_owned(),
        }))
    }

    /// Runs the checks [`Self::transition`] would run, writing nothing. A caller that adds
    /// a rule of its own checks after the ledger's, so the ledger's refusal comes first.
    pub fn check_transition_only(
        &self,
        finding: &FindingId,
        to: FindingState,
        reason: Option<crate::finding::UnresolvedReason>,
        remedy: Option<String>,
        evidence: Vec<EvidenceId>,
    ) -> Result<(), Error> {
        let finding = self.own_finding_id(finding);
        let evidence: Vec<EvidenceId> = evidence.iter().map(|e| self.own_evidence_id(e)).collect();
        let from = self
            .index
            .require_finding(&finding, "transition")
            .map_err(Error::Refused)?
            .state;
        let record = Record::Transition(Transition {
            finding,
            from,
            to,
            reason,
            remedy,
            evidence,
            by: String::new(),
        });
        self.index
            .check(&record, LEDGER_SCHEMA_VERSION)
            .map_err(Error::Refused)
    }

    /// Runs the checks [`Self::resolve_open_item`] would run, writing nothing.
    pub fn check_open_item_resolution(
        &self,
        id: &OpenItemId,
        resolution: Resolution,
    ) -> Result<(), Error> {
        let resolution = match resolution {
            Resolution::Hypothesis(f) => Resolution::Hypothesis(self.own_finding_id(&f)),
            other => other,
        };
        let record = Record::OpenItemResolved(OpenItemResolved {
            id: self.own_open_item_id(id),
            resolution,
        });
        self.index
            .check(&record, LEDGER_SCHEMA_VERSION)
            .map_err(Error::Refused)
    }

    /// When the line that raised finding `id` was written; `None` when it is not in this
    /// ledger's lines.
    pub fn finding_added_at(&self, id: &FindingId) -> Option<DateTime<Utc>> {
        let id = self.own_finding_id(id);
        self.lines.iter().find_map(|l| match &l.record {
            Record::FindingAdded { finding } if finding.id == id => Some(l.ts),
            _ => None,
        })
    }

    /// When the line that raised open item `id` was written.
    pub fn open_item_added_at(&self, id: &OpenItemId) -> Option<DateTime<Utc>> {
        let id = self.own_open_item_id(id);
        self.lines.iter().find_map(|l| match &l.record {
            Record::OpenItem(o) if o.id == id => Some(l.ts),
            _ => None,
        })
    }

    /// Raises an open item, allocating `O<n>`.
    pub fn open_item_add(
        &mut self,
        text: &str,
        finding: Option<FindingId>,
        opened_by: &str,
    ) -> Result<LedgerLine, Error> {
        self.locked(|this| this.open_item_add_held(text, finding, opened_by))
    }

    fn open_item_add_held(
        &mut self,
        text: &str,
        finding: Option<FindingId>,
        opened_by: &str,
    ) -> Result<LedgerLine, Error> {
        let finding = finding.map(|f| self.own_finding_id(&f));
        self.append(Record::OpenItem(OpenItem {
            id: self.next_open_item_id(),
            text: text.to_owned(),
            finding,
            opened_by: opened_by.to_owned(),
        }))
    }

    /// Closes an open item.
    pub fn resolve_open_item(
        &mut self,
        id: &OpenItemId,
        resolution: Resolution,
    ) -> Result<LedgerLine, Error> {
        self.locked(|this| this.resolve_open_item_held(id, resolution))
    }

    fn resolve_open_item_held(
        &mut self,
        id: &OpenItemId,
        resolution: Resolution,
    ) -> Result<LedgerLine, Error> {
        let resolution = match resolution {
            Resolution::Hypothesis(f) => Resolution::Hypothesis(self.own_finding_id(&f)),
            other => other,
        };
        self.append(Record::OpenItemResolved(OpenItemResolved {
            id: self.own_open_item_id(id),
            resolution,
        }))
    }

    /// Records the answer to a checklist item (the latest answer wins).
    pub fn answer_checklist(&mut self, answer: ChecklistAnswer) -> Result<LedgerLine, Error> {
        self.locked(|this| this.answer_checklist_held(answer))
    }

    fn answer_checklist_held(&mut self, mut answer: ChecklistAnswer) -> Result<LedgerLine, Error> {
        answer.evidence = answer
            .evidence
            .iter()
            .map(|e| self.own_evidence_id(e))
            .collect();
        self.append(Record::ChecklistAnswer(answer))
    }

    /// Records writers and removers for a finding. Refuses an unknown finding, a
    /// malformed `file:line` site and an annotation with nothing in it. The
    /// finding's state does not change.
    pub fn annotate(
        &mut self,
        finding: &FindingId,
        writers: Vec<String>,
        removers: Vec<String>,
        note: Option<String>,
    ) -> Result<LedgerLine, Error> {
        self.annotate_record(FindingAnnotation {
            writers,
            removers,
            note,
            ..FindingAnnotation::new(finding.clone())
        })
    }

    /// Appends a full annotation: writers, removers, a note, and the holder, owner,
    /// region and site of a finding that has none yet. A value that differs from one the finding
    /// already has is refused with `annotation-conflict`; the same value is accepted.
    pub fn annotate_record(&mut self, annotation: FindingAnnotation) -> Result<LedgerLine, Error> {
        self.locked(|this| this.annotate_record_held(annotation))
    }

    fn annotate_record_held(
        &mut self,
        mut annotation: FindingAnnotation,
    ) -> Result<LedgerLine, Error> {
        annotation.finding = self.own_finding_id(&annotation.finding);
        annotation.evidence = annotation
            .evidence
            .iter()
            .map(|e| self.own_evidence_id(e))
            .collect();
        annotation.replaced_holder = None;
        if let (Some(new), Some(f)) = (
            annotation.holder,
            self.index.findings.get(&annotation.finding),
        ) && f.holder.is_some_and(|old| old != new)
            && !annotation.evidence.is_empty()
        {
            annotation.replaced_holder = f.holder;
        }
        self.append(Record::FindingAnnotated(annotation))
    }

    /// Labels a confirmed or strong finding (`dla ledger label`). `archetypes` is the
    /// table the label is checked against, or off. Builds the proof, appends one
    /// `finding_labeled` line and returns it.
    pub fn label(
        &mut self,
        finding: &FindingId,
        label: &str,
        evidence: &[EvidenceId],
        note: Option<&str>,
        archetypes: Archetypes<'_>,
    ) -> Result<LedgerLine, Error> {
        self.locked(|this| this.label_held(finding, label, evidence, note, archetypes))
    }

    fn label_held(
        &mut self,
        finding: &FindingId,
        label: &str,
        evidence: &[EvidenceId],
        note: Option<&str>,
        archetypes: Archetypes<'_>,
    ) -> Result<LedgerLine, Error> {
        let finding = self.own_finding_id(finding);
        let evidence: Vec<EvidenceId> = evidence.iter().map(|e| self.own_evidence_id(e)).collect();
        let rec = labels::plan(
            &self.index,
            &self.index,
            &LabelInput {
                finding: &finding,
                label,
                evidence: &evidence,
                note,
                archetypes,
            },
        )?;
        self.append(Record::FindingLabeled(rec))
    }

    /// Read only: the refusal `label` would give before it looks at any evidence.
    pub fn label_precheck(
        &self,
        finding: &FindingId,
        label: &str,
        archetypes: Archetypes<'_>,
    ) -> Result<(), Error> {
        let finding = self.own_finding_id(finding);
        Ok(labels::precheck(&self.index, &finding, label, archetypes)?)
    }

    /// Read only: per archetype that fits `finding`, whether it can be labelled now.
    pub fn suggest_labels(
        &self,
        finding: &FindingId,
        table: &crate::archetypes::Table,
    ) -> Result<Vec<labels::SuggestRow>, Error> {
        let finding = self.own_finding_id(finding);
        Ok(labels::suggest(&self.index, &self.index, &finding, table)?)
    }

    /// The writers and removers annotations recorded for `finding` (`file:line`).
    pub fn recorded_sites(&self, finding: &FindingId) -> (Vec<String>, Vec<String>) {
        let finding = self.own_finding_id(finding);
        self.index
            .annotations
            .get(&finding)
            .map_or((Vec::new(), Vec::new()), |a| {
                (a.writers.clone(), a.removers.clone())
            })
    }

    /// Records a checked explanation (`dla ledger explain`). `census_writers` and
    /// `census_removers` are the heuristic sites the census found for the finding's
    /// owner; they join the ones annotations recorded. Appends one `finding_explained`
    /// line; the latest line for a finding is the explanation.
    #[allow(clippy::too_many_arguments)]
    pub fn explain(
        &mut self,
        finding: &FindingId,
        tag: crate::finding::ExplanationTag,
        adds: &[String],
        removal: RemovalClaim,
        remover: Option<&str>,
        evidence: Option<&EvidenceId>,
        text: &str,
        census_writers: &[String],
        census_removers: &[String],
    ) -> Result<LedgerLine, Error> {
        self.locked(|this| {
            this.explain_held(
                finding,
                tag,
                adds,
                removal,
                remover,
                evidence,
                text,
                census_writers,
                census_removers,
            )
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn explain_held(
        &mut self,
        finding: &FindingId,
        tag: crate::finding::ExplanationTag,
        adds: &[String],
        removal: RemovalClaim,
        remover: Option<&str>,
        evidence: Option<&EvidenceId>,
        text: &str,
        census_writers: &[String],
        census_removers: &[String],
    ) -> Result<LedgerLine, Error> {
        let finding = self.own_finding_id(finding);
        let evidence = evidence.map(|e| self.own_evidence_id(e));
        let (mut writers, mut removers) = self.recorded_sites(&finding);
        for w in census_writers {
            if !writers.contains(w) {
                writers.push(w.clone());
            }
        }
        for r in census_removers {
            if !removers.contains(r) {
                removers.push(r.clone());
            }
        }
        let rec = explain::plan(
            &self.index,
            &self.index,
            &ExplainInput {
                finding: &finding,
                tag,
                adds,
                removal,
                remover,
                evidence: evidence.as_ref(),
                text,
                writers: &writers,
                removers: &removers,
                artifact: self.index.entry == EntryPath::Artifact,
            },
        )?;
        self.append(Record::FindingExplained(rec))
    }

    /// Core mode: records that protected files were sealed again by `dla run reseal`.
    pub fn integrity_resealed(
        &mut self,
        files: Vec<String>,
        reason: &str,
    ) -> Result<LedgerLine, Error> {
        self.locked(|this| this.integrity_resealed_held(files, reason))
    }

    fn integrity_resealed_held(
        &mut self,
        files: Vec<String>,
        reason: &str,
    ) -> Result<LedgerLine, Error> {
        if self.slice.is_some() {
            return Err(core_only("integrity_resealed").into());
        }
        self.append(Record::IntegrityResealed(IntegrityResealed {
            files,
            reason: reason.to_owned(),
        }))
    }

    // ----- worker slices, merge and verification -----

    /// Ends the worker's slice with `slice_done`, listing the slice's own findings that
    /// are not dispositioned and its open items that are not resolved, and marks the
    /// worker `done` in `state.json`. Refused in core mode.
    pub fn finish_slice(&mut self) -> Result<LedgerLine, Error> {
        let Some(w) = self.worker().map(str::to_owned) else {
            return Err(Refusal::new(
                m4::WORKER_ONLY,
                "`ledger done` ends a worker's slice; this is the core ledger",
                "worker mode (--worker <name>)",
                "dla --worker <name> ledger done",
            )
            .into());
        };
        let mut open: Vec<String> = self
            .index
            .findings
            .values()
            .filter(|f| f.id.prefix() == Some(w.as_str()) && !f.state.is_dispositioned())
            .map(|f| f.id.to_string())
            .collect();
        open.extend(
            self.index
                .open_items
                .values()
                .filter(|o| o.item.id.prefix() == Some(w.as_str()) && o.resolution.is_none())
                .map(|o| o.item.id.to_string()),
        );
        let line = self.append(Record::SliceDone(SliceDone {
            worker: w.clone(),
            open,
        }))?;
        if let Some(dir) = self.slice.as_ref().and_then(|s| s.run_dir.clone())
            && dir.join(crate::state::STATE_FILE).is_file()
        {
            RunState::update(&dir, |s| {
                if let Some(e) = s.workers.iter_mut().find(|e| e.name == w) {
                    e.status = WorkerStatus::Done;
                }
                Ok(())
            })?;
        }
        Ok(line)
    }

    /// Core mode: appends `lines` of `worker`'s slice verbatim (the original timestamp
    /// and schema version, a new `seq`, the `worker` field set), then the
    /// `slice_merged` record. All or nothing: a line the merged ledger would refuse
    /// leaves the file untouched.
    pub fn append_merged(
        &mut self,
        worker: &str,
        lines: &[LedgerLine],
        merged: SliceMerged,
    ) -> Result<Vec<LedgerLine>, Error> {
        self.locked(|this| this.append_merged_held(worker, lines, merged))
    }

    fn append_merged_held(
        &mut self,
        worker: &str,
        lines: &[LedgerLine],
        merged: SliceMerged,
    ) -> Result<Vec<LedgerLine>, Error> {
        if self.slice.is_some() {
            return Err(core_only("merge").into());
        }
        let mut batch: Vec<Pending> = lines
            .iter()
            .map(|l| Pending {
                record: l.record.clone(),
                ts: Some(l.ts),
                worker: Some(worker.to_owned()),
                version: l.schema_version,
            })
            .collect();
        batch.push(Pending::new(Record::SliceMerged(merged)));
        self.append_all(batch)
    }

    /// Core mode: records that `duplicate` is the same finding as `canonical`.
    pub fn merge_findings(
        &mut self,
        canonical: &FindingId,
        duplicate: &FindingId,
    ) -> Result<LedgerLine, Error> {
        self.locked(|this| this.merge_findings_held(canonical, duplicate))
    }

    fn merge_findings_held(
        &mut self,
        canonical: &FindingId,
        duplicate: &FindingId,
    ) -> Result<LedgerLine, Error> {
        if self.slice.is_some() {
            return Err(core_only("finding_merged").into());
        }
        self.append(Record::FindingMerged(FindingMerged {
            canonical: canonical.clone(),
            duplicate: duplicate.clone(),
        }))
    }

    /// Records that `duplicate` is the same leak as `canonical` (`dla ledger duplicate`),
    /// when [`crate::duplicate::check`] accepts the pair. Works in core mode and in a
    /// worker's slice (both findings must be the worker's own there). The duplicate's
    /// supports join the canonical finding at replay, as in a cross-worker merge.
    pub fn duplicate_finding(
        &mut self,
        duplicate: &FindingId,
        canonical: &FindingId,
    ) -> Result<(LedgerLine, crate::duplicate::DuplicateBy), Error> {
        self.locked(|this| {
            let by = crate::duplicate::check(this, duplicate, canonical)?;
            let line = this.append(Record::FindingMerged(FindingMerged {
                canonical: canonical.clone(),
                duplicate: duplicate.clone(),
            }))?;
            Ok((line, by))
        })
    }

    /// Core mode: records what a verifier concluded about `finding` from its bundle.
    /// The agreement with the finding's site (same file, within 3 lines, or both none)
    /// and holder is computed here; a disagreement also opens an open item naming both
    /// values. Returns the verification line and the open item line, if any.
    #[allow(clippy::too_many_arguments)]
    pub fn verify(
        &mut self,
        finding: &FindingId,
        site: Option<String>,
        holder: crate::finding::Holder,
        mechanism: &str,
        label: &str,
        bundle: &str,
        archetypes: Archetypes<'_>,
    ) -> Result<(LedgerLine, Option<LedgerLine>), Error> {
        self.locked(|this| {
            this.verify_held(finding, site, holder, mechanism, label, bundle, archetypes)
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn verify_held(
        &mut self,
        finding: &FindingId,
        site: Option<String>,
        holder: crate::finding::Holder,
        mechanism: &str,
        label: &str,
        bundle: &str,
        archetypes: Archetypes<'_>,
    ) -> Result<(LedgerLine, Option<LedgerLine>), Error> {
        if self.slice.is_some() {
            return Err(core_only("verification").into());
        }
        let f = self
            .index
            .require_finding(finding, "verification")
            .map_err(Error::Refused)?;
        let label = label.trim();
        // An empty or spaced label is refused by the record check, with the verify fix.
        let plain = !label.is_empty() && !label.chars().any(char::is_whitespace);
        match archetypes {
            Archetypes::On(t) if plain && label != UNCLASSIFIED && t.get(label).is_none() => {
                return Err(Refusal::new(
                    m6_codes::UNKNOWN_ARCHETYPE,
                    format!("--label {label:?} is not an archetype ID"),
                    "an archetype ID or UNCLASSIFIED",
                    format!(
                        "use one of: {}, UNCLASSIFIED",
                        t.ids().collect::<Vec<_>>().join(", ")
                    ),
                )
                .into());
            }
            Archetypes::Off if plain && label != UNCLASSIFIED => {
                return Err(Refusal::new(
                    m6_codes::ARCHETYPES_OFF,
                    format!(
                        "archetypes are off: --label {label:?} cannot be checked, only UNCLASSIFIED is accepted"
                    ),
                    "--label UNCLASSIFIED",
                    format!("dla ledger verify {finding} ... --label UNCLASSIFIED"),
                )
                .into());
            }
            _ => {}
        }
        let v = verify::record(
            f,
            site,
            holder,
            mechanism.to_owned(),
            label.to_owned(),
            bundle.to_owned(),
            matches!(archetypes, Archetypes::On(_)),
        );
        let mut batch = vec![Pending::new(Record::Verification(v.clone()))];
        if !verify::agrees(&v) {
            batch.push(Pending::new(Record::OpenItem(OpenItem {
                id: self.next_open_item_id(),
                text: verify::disagreement_text(f, &v),
                finding: Some(finding.clone()),
                opened_by: "dla ledger verify".to_owned(),
            })));
        }
        let mut written = self.append_all(batch)?;
        let item = (written.len() > 1).then(|| written.remove(1));
        Ok((written.remove(0), item))
    }

    /// The warning to print after confirming `id`: set when no passing growth column the
    /// finding cites is tied to it. `None` for other findings and states.
    pub fn confirm_warning(&self, id: &FindingId) -> Option<String> {
        let f = self.index.findings.get(id)?;
        machine::confirm_warning(f, &self.index)
    }
}
