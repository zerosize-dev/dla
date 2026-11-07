//! The ledger: append-only JSONL, replayed and validated on load.
//!
//! [`Ledger`] is the only writer. Every mutating method validates against the
//! replayed state first, appends exactly one line in append mode, and returns the
//! new [`LedgerLine`]. The file is never rewritten or truncated.

use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    SCHEMA_VERSION,
    checklist::{AnswerStatus, Checklist, ChecklistAnswer, ChecklistEntry},
    error::{Error, Refusal, ReplayError, codes},
    evidence::{EvidenceRecord, NewEvidence},
    finding::{
        Explanation, Finding, FindingState, Holder, Locus, Origin, Region, RepairClass, Shape,
        UNCLASSIFIED,
    },
    ids::{EvidenceId, FindingId, OpenItemId},
    machine::{self, EvidenceLookup, M0Rules, TransitionContext},
    record::{LedgerLine, OpenItem, OpenItemResolved, Record, Resolution, Transition},
    state::{LEDGER_FILE, now_utc},
    version::{Artifact, UpgradeError, upgrade},
};

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

/// Everything derived from replaying the lines.
#[derive(Debug, Clone, Default)]
struct Index {
    evidence: BTreeMap<EvidenceId, EvidenceRecord>,
    findings: BTreeMap<FindingId, Finding>,
    open_items: BTreeMap<OpenItemId, OpenItemState>,
    answers: BTreeMap<String, ChecklistAnswer>,
}

impl EvidenceLookup for Index {
    fn evidence(&self, id: &EvidenceId) -> Option<&EvidenceRecord> {
        self.evidence.get(id)
    }
}

fn empty(s: &Option<String>) -> bool {
    s.as_deref().is_none_or(|t| t.trim().is_empty())
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
    fn check(&self, record: &Record) -> Result<(), Refusal> {
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
            Record::FindingAdded { finding } => self.check_finding(finding),
            Record::Transition(t) => self.check_transition(t),
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
            Record::OpenItemResolved(r) => self.check_open_item_resolved(r),
            Record::ChecklistAnswer(a) => self.check_answer(a),
        }
    }

    fn check_finding(&self, f: &Finding) -> Result<(), Refusal> {
        let id = &f.id;
        if self.findings.contains_key(id) {
            return Err(Refusal::new(
                codes::DUPLICATE_ID,
                format!("finding {id} already exists"),
                "a fresh finding ID",
                "let the ledger allocate the ID",
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

    fn check_transition(&self, t: &Transition) -> Result<(), Refusal> {
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
        };
        machine::check_transition(&M0Rules, &ctx)
    }

    fn check_open_item_resolved(&self, r: &OpenItemResolved) -> Result<(), Refusal> {
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
            Resolution::Unresolved { remedy, .. } => {
                if remedy.trim().is_empty() {
                    return Err(Refusal::new(
                        codes::REMEDY_REQUIRED,
                        format!("resolving {} as unresolved needs a remedy", r.id),
                        "--remedy",
                        format!(
                            "dla ledger resolve {} --as unresolved --reason time_budget --remedy \"<text>\"",
                            r.id
                        ),
                    ));
                }
                Ok(())
            }
            Resolution::Hypothesis(f) => {
                self.require_finding(f, "open item resolution").map(|_| ())
            }
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
                            "dla ledger resolve {} --status unresolved --reason time_budget --remedy \"<text>\"",
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
                            "dla ledger resolve {} --status unresolved --reason time_budget --remedy \"<text>\"",
                            a.item
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

    /// Applies a record that [`Index::check`] accepted. Cannot fail.
    fn commit(&mut self, record: &Record) {
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
        }
    }
}

/// A replayed ledger, optionally bound to a file for appending.
#[derive(Debug, Clone)]
pub struct Ledger {
    path: Option<PathBuf>,
    lines: Vec<LedgerLine>,
    index: Index,
    ends_with_newline: bool,
}

fn replay_err(path: Option<&Path>, line: u64, malformed: bool, refusal: Refusal) -> Error {
    Error::Replay(Box::new(ReplayError {
        path: path.map(Path::to_path_buf),
        line,
        malformed,
        refusal,
    }))
}

impl Ledger {
    fn empty(path: Option<PathBuf>) -> Self {
        Self {
            path,
            lines: Vec::new(),
            index: Index::default(),
            ends_with_newline: true,
        }
    }

    /// `<run_dir>/ledger.jsonl`.
    pub fn path_in(run_dir: &Path) -> PathBuf {
        run_dir.join(LEDGER_FILE)
    }

    /// Creates an empty ledger file in `run_dir` (which must exist). Fails if one exists.
    pub fn create(run_dir: &Path) -> Result<Self, Error> {
        let path = Self::path_in(run_dir);
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|e| Error::io(&path, e))?;
        Ok(Self::empty(Some(path)))
    }

    /// Replays `<run_dir>/ledger.jsonl`, validating every line.
    pub fn load(run_dir: &Path) -> Result<Self, Error> {
        let path = Self::path_in(run_dir);
        let text = fs::read_to_string(&path).map_err(|e| Error::io(&path, e))?;
        Self::replay(&text, Some(path))
    }

    /// Replays ledger text that is not bound to a file (for tests and renderers).
    /// Mutating methods on the result only change memory.
    pub fn from_text(text: &str) -> Result<Self, Error> {
        Self::replay(text, None)
    }

    fn replay(text: &str, path: Option<PathBuf>) -> Result<Self, Error> {
        let mut ledger = Self::empty(path.clone());
        ledger.ends_with_newline = text.is_empty() || text.ends_with('\n');
        let p = path.as_deref();
        for (i, raw) in text.lines().enumerate() {
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
                serde_json::from_str(raw).map_err(|e| malformed(format!("invalid JSON: {e}")))?;
            let doc = upgrade(doc, Artifact::LedgerLine, &format!("ledger line {n}")).map_err(
                |e| match e {
                    UpgradeError::Malformed(m) => malformed(m),
                    UpgradeError::Schema(s) => Error::Schema(s),
                },
            )?;
            let line: LedgerLine = serde_json::from_value(doc)
                .map_err(|e| malformed(format!("not a valid ledger record: {e}")))?;
            let expected = ledger.next_seq();
            if line.seq != expected {
                return Err(replay_err(
                    p,
                    n,
                    false,
                    Refusal::new(
                        codes::BAD_SEQ,
                        format!(
                            "seq is {} but {expected} was expected (gap or reordering)",
                            line.seq
                        ),
                        format!("seq {expected}"),
                        format!("renumber line {n} so seq values run 1, 2, 3 without gaps"),
                    ),
                ));
            }
            ledger
                .index
                .check(&line.record)
                .map_err(|r| replay_err(p, n, false, r))?;
            ledger.index.commit(&line.record);
            ledger.lines.push(line);
        }
        Ok(ledger)
    }

    fn next_seq(&self) -> u64 {
        self.lines.len() as u64 + 1
    }

    // ----- read access -----

    /// The ledger file, when bound to one.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }
    /// Every line in order.
    pub fn lines(&self) -> &[LedgerLine] {
        &self.lines
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
    /// All open items, ordered by ID.
    pub fn open_items(&self) -> impl Iterator<Item = &OpenItemState> {
        self.index.open_items.values()
    }
    /// One open item.
    pub fn open_item(&self, id: &OpenItemId) -> Option<&OpenItemState> {
        self.index.open_items.get(id)
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

    // ----- ID allocation: next n = max existing + 1 -----

    /// The ID the next evidence record will get.
    pub fn next_evidence_id(&self) -> EvidenceId {
        let max = self
            .index
            .evidence
            .keys()
            .filter(|e| e.prefix().is_none())
            .map(EvidenceId::number)
            .max()
            .unwrap_or(0);
        EvidenceId::new(max + 1)
    }
    /// The ID the next top-level finding will get.
    pub fn next_finding_id(&self) -> FindingId {
        let max = self
            .index
            .findings
            .keys()
            .filter(|f| f.prefix().is_none())
            .map(FindingId::number)
            .max()
            .unwrap_or(0);
        FindingId::new(max + 1)
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
            .filter(|o| o.prefix().is_none())
            .map(OpenItemId::number)
            .max()
            .unwrap_or(0);
        OpenItemId::new(max + 1)
    }

    // ----- writers -----

    fn append(&mut self, record: Record) -> Result<LedgerLine, Error> {
        self.index.check(&record)?;
        let line = LedgerLine {
            schema_version: SCHEMA_VERSION,
            seq: self.next_seq(),
            ts: now_utc(),
            record,
        };
        if let Some(path) = &self.path {
            let mut text = String::new();
            if !self.ends_with_newline {
                text.push('\n');
            }
            text.push_str(&serde_json::to_string(&line).map_err(|e| Error::Malformed {
                path: path.clone(),
                message: e.to_string(),
            })?);
            text.push('\n');
            let mut f = OpenOptions::new()
                .append(true)
                .open(path)
                .map_err(|e| Error::io(path, e))?;
            f.write_all(text.as_bytes())
                .and_then(|()| f.sync_data())
                .map_err(|e| Error::io(path, e))?;
            self.ends_with_newline = true;
        }
        self.index.commit(&line.record);
        self.lines.push(line.clone());
        Ok(line)
    }

    /// Appends an evidence record with a freshly allocated ID.
    pub fn add_evidence(&mut self, new: NewEvidence) -> Result<LedgerLine, Error> {
        let rec = new.into_record(self.next_evidence_id());
        self.append(Record::Evidence(rec))
    }

    /// Appends a finding in state `candidate`, allocating `F<n>` or `F<n>.n<k>`.
    pub fn add_finding(&mut self, new: NewFinding) -> Result<LedgerLine, Error> {
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

    /// Raises an open item, allocating `O<n>`.
    pub fn open_item_add(
        &mut self,
        text: &str,
        finding: Option<FindingId>,
        opened_by: &str,
    ) -> Result<LedgerLine, Error> {
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
        self.append(Record::OpenItemResolved(OpenItemResolved {
            id: id.clone(),
            resolution,
        }))
    }

    /// Records the answer to a checklist item (the latest answer wins).
    pub fn answer_checklist(&mut self, answer: ChecklistAnswer) -> Result<LedgerLine, Error> {
        self.append(Record::ChecklistAnswer(answer))
    }
}
