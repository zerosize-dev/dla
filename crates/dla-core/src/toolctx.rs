//! The context object every stage crate receives, and the helpers that write through
//! the ledger and the coverage manifest.
//!
//! # Design: the crate does the writes
//!
//! `dla probe`, `dla sweep`, `dla census`, `dla workload discover` and `dla install`
//! each call ONE public function in their crate. That function receives a
//! [`ToolCtx`] and performs its own ledger and coverage writes with the helpers here
//! ([`ToolCtx::add_evidence`], [`CandidateWriter`], [`CoverageManifest::update`]). It
//! returns a [`StageOutcome`], a small bounded summary the CLI prints as the
//! command's one JSON object, or a [`Refusal`] the CLI prints with exit code 2.
//!
//! Rules every stage follows:
//!
//! * one evidence record per tool run, with a bounded `result` and a `raw` pointer to
//!   the full output under `.dla/<run>/<stage>/` ([`ToolCtx::stage_dir`]);
//! * every candidate cites that evidence in `supports` ([`Candidate::evidence`]);
//! * caches and scratch files go under [`ToolCtx::tmp_dir`], never `HOME` and never
//!   the project source;
//! * the same `(origin, site)` is added at most once per run ([`CandidateWriter`]).

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{
    Ledger, NewFinding, Origin, Refusal, RunState,
    consent::format_tiers,
    error::{Error, codes},
    evidence::{Authority, EvidenceCoverage, EvidenceKind, NewEvidence, Source, Tool, Unknown},
    finding::{Holder, Region, check_site},
    harness::{HARNESS_DIR, HarnessManifest},
    ids::{EvidenceId, FindingId},
    paths,
    series::{SERIES_DIR, next_number},
    state::{DLA_DIR, ProcessRecord, now_utc},
    trend::TREND_DIR,
};

/// Where a stage runs and what it may do.
#[derive(Debug, Clone)]
pub struct ToolCtx {
    /// Absolute project root (the directory under analysis).
    pub project: PathBuf,
    /// `<project>/.dla/<run-id>`.
    pub run_dir: PathBuf,
    /// Run ID.
    pub run_id: String,
    /// Commit hash of the project's HEAD, or the project path when none is readable;
    /// recorded as the `repo` source of every evidence record.
    pub source_commit: String,
    /// Value of the global `--data` flag, if given.
    pub data_flag: Option<PathBuf>,
    /// Worker name in worker mode (global `--worker` / `DLA_WORKER`): ledger writes go
    /// to `ledger.<worker>.jsonl` and IDs carry the `<worker>/` prefix.
    pub worker: Option<String>,
}

impl ToolCtx {
    /// Plugin data files directory (`data/`): `--data`, else `$DLA_PLUGIN_ROOT/data`,
    /// else the enclosing checkout's `data/`; a refusal naming `--data` otherwise.
    pub fn data_files_dir(&self) -> Result<PathBuf, Refusal> {
        paths::data_files_dir(self.data_flag.as_deref())
    }

    /// `$DLA_DATA_DIR/tools`, where `dla install` puts analyzers; `None` when
    /// `$DLA_DATA_DIR` is unset.
    pub fn tools_dir(&self) -> Option<PathBuf> {
        paths::tools_dir()
    }

    /// `.dla/<run>/<rel>`, project-relative with `/` separators (what `raw` and
    /// artifact pointers use is relative to the run dir; this one is for CLI output).
    pub fn pointer(&self, rel: &str) -> String {
        format!("{DLA_DIR}/{}/{rel}", self.run_id)
    }

    /// Creates and returns `<run>/<stage>` (for example `sweep`, `probe`).
    pub fn stage_dir(&self, stage: &str) -> Result<PathBuf, Refusal> {
        make_dir(&self.run_dir.join(stage))
    }

    /// Creates and returns `<run>/tmp`: the home of tool caches and scratch output.
    pub fn tmp_dir(&self) -> Result<PathBuf, Refusal> {
        make_dir(&self.run_dir.join("tmp"))
    }

    /// The consent tiers recorded in `state.json` (empty when no consent was given).
    pub fn consent_tiers(&self) -> Result<Vec<u8>, Error> {
        Ok(RunState::load(&self.run_dir)?
            .consent
            .map(|c| c.tiers)
            .unwrap_or_default())
    }

    /// Refuses unless consent includes `tier`. The refusal names the tier needed. When
    /// no consent was given yet, the fix is the run's one consent question,
    /// `dla consent --tiers 1-4`, which covers the tier. When consent was already given
    /// (there is one consent per run), the fix is a new run, whose consent covers the
    /// tiers given so far plus the one needed.
    pub fn require_tier(&self, tier: u8, what: &str) -> Result<(), Refusal> {
        let state = RunState::load(&self.run_dir).map_err(|e| e.to_refusal())?;
        let Some(consent) = state.consent else {
            return Err(Refusal::new(
                codes::CONSENT_REQUIRED,
                format!("{what} needs consent tier {tier}, and this run has no consent yet"),
                format!("consent tier {tier} in state.json"),
                "dla consent --tiers 1-4",
            ));
        };
        if consent.tiers.contains(&tier) {
            return Ok(());
        }
        let mut wanted = consent.tiers.clone();
        wanted.push(tier);
        let wanted = format_tiers(&wanted);
        Err(Refusal::new(
            codes::CONSENT_REQUIRED,
            format!(
                "{what} needs consent tier {tier}, but this run's consent covers only tier(s) {}; a run has one consent, so a new run is needed",
                format_tiers(&consent.tiers)
            ),
            format!("consent tier {tier} in state.json"),
            format!("dla run init   then   dla consent --tiers {wanted}"),
        ))
    }

    /// The ID the next series gets: `S<n>`, the highest `series/S<n>.json` plus one.
    pub fn next_series_id(&self) -> String {
        format!(
            "S{}",
            next_number(&self.run_dir.join(SERIES_DIR), 'S', ".json", false)
        )
    }

    /// The ID the next trend report gets: `T<n>`, the highest `trend/T<n>.json` plus one.
    pub fn next_trend_id(&self) -> String {
        format!(
            "T{}",
            next_number(&self.run_dir.join(TREND_DIR), 'T', ".json", false)
        )
    }

    /// The ID the next harness gets: `H<n>`, the highest `harness/H<n>` directory plus one.
    pub fn next_harness_id(&self) -> String {
        format!(
            "H{}",
            next_number(&self.run_dir.join(HARNESS_DIR), 'H', "", true)
        )
    }

    /// The manifest of harness `id`.
    pub fn harness(&self, id: &str) -> Result<HarnessManifest, Refusal> {
        HarnessManifest::load(&self.run_dir, id).map_err(|e| e.to_refusal())
    }

    /// `Source::Artifact` for the file at `rel`, a path relative to the run directory:
    /// the path (with `/` separators) and the SHA-256 of the file's bytes.
    pub fn artifact_source(&self, rel: &str) -> Result<Source, Refusal> {
        let path = self.run_dir.join(rel);
        let bytes = fs::read(&path).map_err(|e| Error::io(&path, e).to_refusal())?;
        Ok(Source::Artifact {
            path: rel.replace('\\', "/"),
            sha256: hex::encode(Sha256::digest(&bytes)),
        })
    }

    /// `Source::Process` for a process observed live.
    pub fn process_source(pid: u32, start_time: &str) -> Source {
        Source::Process {
            pid,
            start_time: start_time.to_owned(),
        }
    }

    /// Records a process DLA started in `state.json`'s `processes`. The state is read
    /// again right before the atomic write, so a concurrent change to another field
    /// is not lost.
    pub fn register_process(&self, record: ProcessRecord) -> Result<(), Refusal> {
        let mut state = RunState::load(&self.run_dir).map_err(|e| e.to_refusal())?;
        state.processes.push(record);
        state.save(&self.run_dir).map_err(|e| e.to_refusal())
    }

    /// Marks the registered process `pid` (with `start_time`, when both sides have one)
    /// as stopped now. Returns whether a running record matched. Re-reads the state
    /// right before the atomic write.
    pub fn mark_process_stopped(
        &self,
        pid: u32,
        start_time: Option<&str>,
    ) -> Result<bool, Refusal> {
        let mut state = RunState::load(&self.run_dir).map_err(|e| e.to_refusal())?;
        let hit = state.processes.iter_mut().rev().find(|p| {
            p.pid == pid
                && p.stopped_at.is_none()
                && match (start_time, p.start_time.as_deref()) {
                    (Some(a), Some(b)) => a == b,
                    _ => true,
                }
        });
        let Some(p) = hit else {
            return Ok(false);
        };
        p.stopped_at = Some(now_utc());
        state.save(&self.run_dir).map_err(|e| e.to_refusal())?;
        Ok(true)
    }

    /// Replays the run's ledger.
    pub fn ledger(&self) -> Result<Ledger, Refusal> {
        Ledger::load_for(&self.run_dir, self.worker.as_deref()).map_err(|e| e.to_refusal())
    }

    /// Whether a project-relative path (`/` separators) is inside the run's scope.
    // M4 skeleton: implemented by the run-state work (reads `state.json` scope).
    pub fn in_scope(&self, _rel: &str) -> bool {
        true
    }

    /// Refuses with code `time-budget` when the worker's deadline (worker mode) or the
    /// run's deadline has passed. `what` names the step for the message.
    // M4 skeleton: implemented by the run-state work.
    pub fn require_time(&self, _what: &str) -> Result<(), Refusal> {
        Ok(())
    }

    /// Appends one evidence record and returns its ID. `raw` is relative to the run
    /// directory (for example `sweep/staticcheck.json`).
    pub fn add_evidence(
        &self,
        ledger: &mut Ledger,
        spec: EvidenceSpec,
    ) -> Result<EvidenceId, Refusal> {
        let line = ledger
            .add_evidence(NewEvidence {
                source: spec.source.unwrap_or_else(|| Source::Repo {
                    commit: self.source_commit.clone(),
                    path: None,
                }),
                tool: Tool {
                    name: spec.tool,
                    version: spec.tool_version,
                },
                authority: spec.authority,
                kind: spec.kind,
                result: spec.result,
                raw: spec.raw,
                coverage: spec.coverage,
                limitations: spec.limitations,
                unknowns: spec.unknowns,
            })
            .map_err(|e| e.to_refusal())?;
        match line.record {
            crate::Record::Evidence(e) => Ok(e.evidence_id),
            _ => Err(Refusal::new(
                codes::MALFORMED,
                "ledger returned a non-evidence line",
                "an evidence record",
                "report this as a dla bug",
            )),
        }
    }
}

fn make_dir(p: &Path) -> Result<PathBuf, Refusal> {
    fs::create_dir_all(p).map_err(|e| Error::io(p, e).to_refusal())?;
    Ok(p.to_path_buf())
}

/// What a stage records as evidence.
#[derive(Debug, Clone)]
pub struct EvidenceSpec {
    /// Evidence kind, for example `probe`, `static-hit`, `workload` or `growth-series`.
    pub kind: EvidenceKind,
    /// Tool name (the backend, or `dla`).
    pub tool: String,
    /// Tool version (`unknown` when it cannot be read).
    pub tool_version: String,
    /// How the evidence was obtained (`static` by default; `test`, `live` or `dump` for
    /// dynamic evidence).
    pub authority: Authority,
    /// Bounded summary (serialized size under [`crate::MAX_RESULT_BYTES`]).
    pub result: Value,
    /// Full output, relative to the run directory.
    pub raw: Option<String>,
    /// How much of the subject it covers.
    pub coverage: EvidenceCoverage,
    /// Known limits.
    pub limitations: Vec<String>,
    /// Where the evidence came from, when not the project's commit: an artifact file
    /// ([`ToolCtx::artifact_source`]) or a live process ([`ToolCtx::process_source`]).
    pub source: Option<Source>,
    /// Open questions the evidence leaves.
    pub unknowns: Vec<Unknown>,
}

impl EvidenceSpec {
    /// Static evidence of `kind` from `tool`, complete coverage, no limitations.
    pub fn new(kind: EvidenceKind, tool: &str, tool_version: &str, result: Value) -> Self {
        Self {
            kind,
            tool: tool.to_owned(),
            tool_version: tool_version.to_owned(),
            authority: Authority::Static,
            result,
            raw: None,
            coverage: EvidenceCoverage::default(),
            limitations: Vec::new(),
            source: None,
            unknowns: Vec::new(),
        }
    }
}

/// A candidate finding as a stage describes it.
#[derive(Debug, Clone, Default)]
pub struct Candidate {
    /// `detector:<backend>/<rule>` or `census`.
    pub origin: String,
    /// The evidence record of the run that produced it (becomes `supports`).
    pub evidence: Vec<EvidenceId>,
    /// `file:line`, project-relative.
    pub site: String,
    /// Holder class (census).
    pub holder: Option<Holder>,
    /// Owner (census).
    pub owner: Option<String>,
    /// Region (census).
    pub region: Option<Region>,
    /// Archetype tag; the empty string and `None` both mean no suggestion.
    pub suggested_archetype: Option<String>,
}

/// Adds candidates to a ledger, skipping an `(origin, site)` pair already present.
#[derive(Debug, Default)]
pub struct CandidateWriter {
    seen: HashSet<(String, String)>,
    added: u64,
    skipped: u64,
}

impl CandidateWriter {
    /// Starts from the `(origin, site)` pairs already in `ledger`.
    pub fn new(ledger: &Ledger) -> Self {
        let seen = ledger
            .findings()
            .filter_map(|f| f.site.clone().map(|s| (f.origin.to_string(), s)))
            .collect();
        Self {
            seen,
            added: 0,
            skipped: 0,
        }
    }

    /// Adds `c` unless `(origin, site)` exists. Returns the new ID, or `None` for a
    /// duplicate.
    pub fn add(&mut self, ledger: &mut Ledger, c: Candidate) -> Result<Option<FindingId>, Refusal> {
        check_site(&c.site).map_err(|m| {
            Refusal::new(
                codes::INVALID_SITE,
                m,
                "a site written file:line",
                "report this as a dla bug",
            )
        })?;
        let key = (c.origin.clone(), c.site.clone());
        if self.seen.contains(&key) {
            self.skipped += 1;
            return Ok(None);
        }
        let origin = Origin::parse(&c.origin).map_err(|e| {
            Refusal::new(
                codes::INVALID_ORIGIN,
                e.to_string(),
                "a valid origin",
                "report this as a dla bug",
            )
        })?;
        let mut nf = NewFinding::new(origin, c.evidence);
        nf.site = Some(c.site);
        nf.holder = c.holder;
        nf.owner = c.owner;
        nf.region = c.region;
        nf.suggested_archetype = c.suggested_archetype.filter(|a| !a.is_empty());
        let line = ledger.add_finding(nf).map_err(|e| e.to_refusal())?;
        self.seen.insert(key);
        self.added += 1;
        match line.record {
            crate::Record::FindingAdded { finding } => Ok(Some(finding.id)),
            _ => Ok(None),
        }
    }

    /// Candidates added so far.
    pub fn added(&self) -> u64 {
        self.added
    }

    /// Duplicates skipped so far.
    pub fn skipped(&self) -> u64 {
        self.skipped
    }
}

/// What a stage reports; the CLI prints it as the command's JSON object (bounded).
#[derive(Debug, Clone, Default, Serialize)]
pub struct StageOutcome {
    /// Evidence IDs written (one per tool run).
    pub evidence: Vec<String>,
    /// Candidates added to the ledger.
    pub candidates: u64,
    /// Candidates skipped because `(origin, site)` was already present.
    pub skipped_duplicates: u64,
    /// Run-relative-to-project pointers (`.dla/<run>/...`) to files written.
    pub artifacts: Vec<String>,
    /// Small stage-specific summary (keep it well under 1 KB).
    pub summary: Value,
}

/// The refusal a stage returns until its implementation exists.
pub fn not_implemented(what: &str) -> Refusal {
    Refusal::new(
        codes::NOT_IMPLEMENTED,
        format!("{what} is not implemented in this build"),
        what,
        "use a dla build that includes it",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        EvidenceKind,
        evidence::{Authority, Unknown},
        state::{Consent, ConsentGiver, now_utc},
    };
    use serde_json::json;

    fn run(tiers: Option<Vec<u8>>) -> (tempfile::TempDir, ToolCtx) {
        let dir = tempfile::tempdir().unwrap();
        let run_dir = dir.path().join(".dla/r1");
        fs::create_dir_all(&run_dir).unwrap();
        let mut st = RunState::init(
            dir.path().to_str().unwrap(),
            Some("s"),
            3600,
            None,
            now_utc(),
        )
        .unwrap();
        st.run_id = "r1".into();
        st.consent = tiers.map(|tiers| Consent {
            tiers,
            raw_values: false,
            given_by: ConsentGiver::Runner,
            recorded_at: now_utc(),
        });
        st.save(&run_dir).unwrap();
        Ledger::create(&run_dir).unwrap();
        let ctx = ToolCtx {
            project: dir.path().to_path_buf(),
            run_dir,
            run_id: "r1".into(),
            source_commit: "abc".into(),
            data_flag: None,
            worker: None,
        };
        (dir, ctx)
    }

    #[test]
    fn require_tier_refuses_without_consent_and_names_the_fix() {
        let (_d, ctx) = run(None);
        let e = ctx.require_tier(1, "building").unwrap_err();
        assert_eq!(e.code, codes::CONSENT_REQUIRED);
        assert_eq!(e.fix, "dla consent --tiers 1-4");
        assert!(e.message.contains("tier 1"), "{}", e.message);
        let e = ctx.require_tier(3, "attaching").unwrap_err();
        assert_eq!(e.fix, "dla consent --tiers 1-4");
        assert!(e.message.contains("tier 3"), "{}", e.message);
        let (_d, ctx) = run(Some(vec![1, 2]));
        ctx.require_tier(1, "building").unwrap();
    }

    #[test]
    fn require_tier_after_consent_says_a_new_run_is_needed() {
        let (_d, ctx) = run(Some(vec![2]));
        let e = ctx.require_tier(1, "building").unwrap_err();
        assert_eq!(e.code, codes::CONSENT_REQUIRED);
        assert_eq!(e.fix, "dla run init   then   dla consent --tiers 1,2");
        assert!(
            e.message.contains("tier 1") && e.message.contains("new run"),
            "{}",
            e.message
        );
        let (_d, ctx) = run(Some(vec![1, 2, 3]));
        let e = ctx.require_tier(4, "heap dump").unwrap_err();
        assert_eq!(e.fix, "dla run init   then   dla consent --tiers 1-4");
    }

    #[test]
    fn ids_are_allocated_from_the_run_directory() {
        let (_d, ctx) = run(None);
        assert_eq!(
            (
                ctx.next_series_id(),
                ctx.next_trend_id(),
                ctx.next_harness_id()
            ),
            ("S1".to_owned(), "T1".to_owned(), "H1".to_owned())
        );
        for (dir, name) in [
            ("series", "S3.json"),
            ("series", "S1.json"),
            ("trend", "T7.json"),
        ] {
            let d = ctx.run_dir.join(dir);
            fs::create_dir_all(&d).unwrap();
            fs::write(d.join(name), "{}").unwrap();
        }
        fs::create_dir_all(ctx.run_dir.join("harness/H2/work")).unwrap();
        fs::create_dir_all(ctx.run_dir.join("harness/H10")).unwrap();
        fs::write(ctx.run_dir.join("harness/H11"), "a file, not a harness").unwrap();
        assert_eq!(ctx.next_series_id(), "S4");
        assert_eq!(ctx.next_trend_id(), "T8");
        assert_eq!(ctx.next_harness_id(), "H11");
    }

    #[test]
    fn processes_are_registered_and_marked_stopped() {
        let (_d, ctx) = run(Some(vec![1]));
        let rec = |pid, start: &str| ProcessRecord {
            pid,
            start_time: Some(start.to_owned()),
            kind: crate::state::ProcessKind::Harness,
            command: "x".into(),
            started_at: now_utc(),
            stopped_at: None,
            role: crate::state::ProcessRole::Started,
            worker: None,
            cleanup: Vec::new(),
        };
        ctx.register_process(rec(10, "a")).unwrap();
        ctx.register_process(rec(10, "b")).unwrap();
        assert!(!ctx.mark_process_stopped(10, Some("zzz")).unwrap());
        assert!(ctx.mark_process_stopped(10, Some("a")).unwrap());
        assert!(!ctx.mark_process_stopped(10, Some("a")).unwrap());
        assert!(ctx.mark_process_stopped(10, None).unwrap());
        assert!(!ctx.mark_process_stopped(11, None).unwrap());
        let st = RunState::load(&ctx.run_dir).unwrap();
        assert_eq!(st.processes.len(), 2);
        assert!(st.processes.iter().all(|p| p.stopped_at.is_some()));
        assert_eq!(st.consent.unwrap().tiers, vec![1], "other fields survive");
    }

    #[test]
    fn evidence_can_override_the_source_and_carry_unknowns() {
        let (_d, ctx) = run(None);
        fs::create_dir_all(ctx.run_dir.join("series")).unwrap();
        fs::write(ctx.run_dir.join("series/S1.json"), "abc").unwrap();
        let mut ledger = ctx.ledger().unwrap();
        let mut spec = EvidenceSpec::new(
            EvidenceKind::GrowthSeries,
            "dla",
            "0",
            json!({"gate": "pass"}),
        );
        spec.authority = Authority::Test;
        spec.source = Some(ctx.artifact_source("series/S1.json").unwrap());
        spec.unknowns = vec![Unknown {
            question: "q".into(),
            resolve_by: "r".into(),
        }];
        let id = ctx.add_evidence(&mut ledger, spec).unwrap();
        let reloaded = ctx.ledger().unwrap();
        let e = reloaded.evidence_by_id(&id).unwrap();
        assert_eq!(
            e.source,
            Source::Artifact {
                path: "series/S1.json".into(),
                sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into()
            }
        );
        assert_eq!(e.unknowns.len(), 1);
        assert!(ctx.artifact_source("series/missing.json").is_err());
        assert_eq!(
            ToolCtx::process_source(5, "t"),
            Source::Process {
                pid: 5,
                start_time: "t".into()
            }
        );
    }

    #[test]
    fn stage_dirs_live_in_the_run_directory() {
        let (_d, ctx) = run(None);
        let s = ctx.stage_dir("sweep").unwrap();
        assert_eq!(s, ctx.run_dir.join("sweep"));
        assert!(ctx.tmp_dir().unwrap().is_dir());
        assert_eq!(ctx.pointer("sweep/x.json"), ".dla/r1/sweep/x.json");
    }

    #[test]
    fn candidates_cite_evidence_and_duplicates_are_skipped() {
        let (_d, ctx) = run(None);
        let mut ledger = ctx.ledger().unwrap();
        let ev = ctx
            .add_evidence(
                &mut ledger,
                EvidenceSpec::new(EvidenceKind::StaticHit, "ruff", "0.1", json!({"hits": 2})),
            )
            .unwrap();
        let cand = |site: &str, arch: &str| Candidate {
            origin: "detector:ruff/B019".into(),
            evidence: vec![ev.clone()],
            site: site.into(),
            suggested_archetype: Some(arch.into()),
            ..Candidate::default()
        };
        let mut w = CandidateWriter::new(&ledger);
        let first = w
            .add(&mut ledger, cand("a.py:3", "L-COLLECT"))
            .unwrap()
            .unwrap();
        assert!(
            w.add(&mut ledger, cand("a.py:3", "L-COLLECT"))
                .unwrap()
                .is_none()
        );
        assert!(w.add(&mut ledger, cand("a.py:4", "")).unwrap().is_some());
        assert_eq!((w.added(), w.skipped()), (2, 1));
        let f = ledger.finding(&first).unwrap();
        assert_eq!(f.supports, std::slice::from_ref(&ev));
        assert_eq!(f.suggested_archetype.as_deref(), Some("L-COLLECT"));
        assert_eq!(f.site.as_deref(), Some("a.py:3"));
        // An empty archetype means none.
        let second = ledger
            .findings()
            .find(|f| f.site.as_deref() == Some("a.py:4"))
            .unwrap();
        assert_eq!(second.suggested_archetype, None);
        // A new writer over the reloaded ledger still skips the same pair.
        let mut reloaded = ctx.ledger().unwrap();
        let mut w2 = CandidateWriter::new(&reloaded);
        assert!(w2.add(&mut reloaded, cand("a.py:3", "")).unwrap().is_none());
    }

    #[test]
    fn a_malformed_site_is_refused() {
        let (_d, ctx) = run(None);
        let mut ledger = ctx.ledger().unwrap();
        let mut w = CandidateWriter::new(&ledger);
        let e = w
            .add(
                &mut ledger,
                Candidate {
                    origin: "census".into(),
                    site: "nolines".into(),
                    ..Candidate::default()
                },
            )
            .unwrap_err();
        assert_eq!(e.code, codes::INVALID_SITE);
    }
}
