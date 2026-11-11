//! The grader: `bench/GRADING.md` implemented literally.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use regex::Regex;
use serde_json::Value;

use crate::{
    Res,
    fixture::load_expected,
    metrics_ext::{Agreement, DecoyRow, LedgerExtra, RunOutcome, ledger_extra},
    model::{
        Category, Cost, Entry, Expected, FixtureStats, Group, Leak, LeakCounts, Metrics,
        MetricsFile, PerRun, Ratio, Share, Sources, SumMean, Unexplained, UsageStats,
        VerificationStats, WallSeconds,
    },
};

/// Rounds to six decimals, half to even, on the exact binary value.
pub fn round6(x: f64) -> f64 {
    format!("{x:.6}").parse().unwrap_or(x)
}

/// State of a normalised finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Confirmed.
    Confirmed,
    /// Strong.
    Strong,
    /// Refuted.
    Refuted,
    /// Suspected (static only).
    Suspected,
    /// Unresolved.
    Unresolved,
    /// Anything else.
    Open,
}

/// A finding reduced to what grading needs.
#[derive(Debug, Clone, PartialEq)]
pub struct Finding {
    /// State.
    pub state: State,
    /// `path:line`.
    pub site: Option<String>,
    /// Owning field or container, `package.Class.field`.
    pub owner: Option<String>,
    /// Worker that wrote the finding (text before the first `/` of its key).
    pub worker: Option<String>,
    /// Holder.
    pub holder: Option<String>,
    /// Archetype label.
    pub label: Option<String>,
    /// Opened as a hypothesis under another finding.
    pub hypothesis: bool,
    /// Carries a ruling-out discriminator.
    pub discriminator: bool,
}

impl Finding {
    /// A non-hypothesis finding in state confirmed or strong.
    pub fn is_claim(&self) -> bool {
        !self.hypothesis && matches!(self.state, State::Confirmed | State::Strong)
    }

    /// A refuted finding with a discriminator.
    pub fn is_rule_out(&self) -> bool {
        self.state == State::Refuted && self.discriminator
    }
}

/// Where a run's findings came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// `ledger.jsonl`.
    Ledger,
    /// `findings.json`.
    Findings,
    /// Neither.
    None,
}

fn str_of(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(Value::as_str).map(str::to_owned)
}

fn state_of_to(to: Option<&str>) -> State {
    match to {
        Some("confirmed") => State::Confirmed,
        Some("strong") => State::Strong,
        Some("refuted") => State::Refuted,
        Some("suspected_static_only") => State::Suspected,
        Some("unresolved") => State::Unresolved,
        _ => State::Open,
    }
}

/// The key of a ledger finding id on a line: the id when it already carries a
/// worker prefix or the line has no worker, else `<worker>/<id>`.
pub(crate) fn ledger_key(id: &str, rec: &Value) -> String {
    let worker = rec
        .get("worker")
        .and_then(Value::as_str)
        .filter(|w| !w.is_empty());
    match worker {
        Some(w) if !id.contains('/') => format!("{w}/{id}"),
        _ => id.to_owned(),
    }
}

/// The worker of a finding key: the non-empty text before its first `/`.
fn worker_of_key(key: &str) -> Option<String> {
    key.split_once('/')
        .map(|(w, _)| w)
        .filter(|w| !w.is_empty())
        .map(str::to_owned)
}

/// What an independent verifier concluded about one finding (a ledger `verification`
/// record, schema 3 and later).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verif {
    /// The verifier's site is the finding's.
    pub agrees_site: bool,
    /// The verifier's holder is the finding's.
    pub agrees_holder: bool,
    /// The verifier's label is the finding's; `None` when the record has no label verdict
    /// (archetypes off, or a line older than schema 6).
    pub agrees_label: Option<bool>,
}

/// A replayed ledger: the findings with their keys, and the verification of each finding
/// that has one.
#[derive(Debug, Clone, Default)]
pub struct LedgerRead {
    /// Findings in the order of their (latest) `finding_added` lines, with their keys.
    pub findings: Vec<(String, Finding)>,
    /// The latest verification record per finding key.
    pub verifications: BTreeMap<String, Verif>,
}

/// Replays a ledger leniently. `None` when it holds no `finding_added` record.
pub fn findings_from_ledger(text: &str) -> Option<Vec<Finding>> {
    findings_from_ledger_bytes(text.as_bytes())
}

/// Like [`findings_from_ledger`] on raw bytes: lines are split on the byte
/// `\n` and decoded one by one; a line that is not valid UTF-8 is skipped
/// like any other line that is not a JSON object.
pub fn findings_from_ledger_bytes(bytes: &[u8]) -> Option<Vec<Finding>> {
    read_ledger_bytes(bytes).map(|r| r.findings.into_iter().map(|(_, f)| f).collect())
}

/// The verification a `verification` record states: `agrees_site` and `agrees_holder` must
/// be booleans, `agrees_label` a boolean or null or absent. A record of any other shape
/// (the pre-schema-3 `result` form) is not a verification.
fn verif_of(rec: &Value) -> Option<Verif> {
    let site = rec.get("agrees_site")?.as_bool()?;
    let holder = rec.get("agrees_holder")?.as_bool()?;
    let label = match rec.get("agrees_label") {
        None | Some(Value::Null) => None,
        Some(Value::Bool(b)) => Some(*b),
        Some(_) => return None,
    };
    Some(Verif {
        agrees_site: site,
        agrees_holder: holder,
        agrees_label: label,
    })
}

/// Replays a ledger leniently, keeping the keys of the findings and the verifications.
/// `None` when it holds no `finding_added` record.
pub fn read_ledger_bytes(bytes: &[u8]) -> Option<LedgerRead> {
    // Findings in the order of their (latest) `finding_added` lines.
    let mut list: Vec<(String, Finding)> = Vec::new();
    let mut verifications: BTreeMap<String, Verif> = BTreeMap::new();
    let mut saw_added = false;
    for line in bytes.split(|b| *b == b'\n') {
        let Ok(line) = std::str::from_utf8(line) else {
            continue;
        };
        let Ok(rec) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        if !rec.is_object() {
            continue;
        }
        let Some(kind) = rec.get("type").and_then(Value::as_str) else {
            continue;
        };
        match kind {
            "finding_added" => {
                saw_added = true;
                let Some(f) = rec.get("finding").filter(|f| f.is_object()) else {
                    continue;
                };
                let Some(id) = f.get("id").and_then(Value::as_str) else {
                    continue;
                };
                let key = ledger_key(id, &rec);
                let finding = Finding {
                    state: State::Open,
                    site: str_of(f, "site"),
                    owner: str_of(f, "owner"),
                    worker: worker_of_key(&key),
                    holder: str_of(f, "holder"),
                    label: str_of(f, "archetype"),
                    hypothesis: f
                        .get("parent")
                        .and_then(Value::as_str)
                        .is_some_and(|p| !p.is_empty()),
                    discriminator: false,
                };
                list.retain(|(i, _)| *i != key);
                list.push((key, finding));
            }
            "transition" => {
                let Some(id) = rec.get("finding").and_then(Value::as_str) else {
                    continue;
                };
                let key = ledger_key(id, &rec);
                if let Some((_, f)) = list.iter_mut().find(|(i, _)| *i == key) {
                    f.state = state_of_to(rec.get("to").and_then(Value::as_str));
                }
            }
            // `dla ledger annotate --holder/--site` fills a holder or site the finding
            // was added without. A record with `replaced_holder` (ledger schema 5, citing
            // growth evidence that observed the holder) changes the holder that was set.
            "finding_annotated" => {
                let Some(id) = rec.get("finding").and_then(Value::as_str) else {
                    continue;
                };
                let key = ledger_key(id, &rec);
                if let Some((_, f)) = list.iter_mut().find(|(i, _)| *i == key) {
                    if f.holder.is_none() || rec.get("replaced_holder").is_some() {
                        f.holder = str_of(&rec, "holder").or(f.holder.take());
                    }
                    if f.site.is_none() {
                        f.site = str_of(&rec, "site");
                    }
                    if f.owner.is_none() {
                        f.owner = str_of(&rec, "owner");
                    }
                }
            }
            // `dla ledger label` (ledger schema 6) sets the archetype label; the last
            // record of a finding wins.
            "finding_labeled" => {
                let Some(id) = rec.get("finding").and_then(Value::as_str) else {
                    continue;
                };
                let key = ledger_key(id, &rec);
                if let (Some((_, f)), Some(label)) = (
                    list.iter_mut().find(|(i, _)| *i == key),
                    str_of(&rec, "label"),
                ) {
                    f.label = Some(label);
                }
            }
            // `dla ledger verify` (schema 3): the last record of a finding wins; a record
            // of another shape changes nothing.
            "verification" => {
                let Some(id) = rec.get("finding").and_then(Value::as_str) else {
                    continue;
                };
                if let Some(v) = verif_of(&rec) {
                    verifications.insert(ledger_key(id, &rec), v);
                }
            }
            // The duplicate collapses into the canonical finding.
            "finding_merged" => {
                let (Some(canonical), Some(duplicate)) = (
                    rec.get("canonical").and_then(Value::as_str),
                    rec.get("duplicate").and_then(Value::as_str),
                ) else {
                    continue;
                };
                let (canonical, duplicate) =
                    (ledger_key(canonical, &rec), ledger_key(duplicate, &rec));
                if canonical != duplicate
                    && list.iter().any(|(i, _)| *i == canonical)
                    && list.iter().any(|(i, _)| *i == duplicate)
                {
                    list.retain(|(i, _)| *i != duplicate);
                }
            }
            _ => {}
        }
    }
    if !saw_added {
        return None;
    }
    let findings = list
        .into_iter()
        .map(|(k, mut f)| {
            f.discriminator = f.state == State::Refuted;
            (k, f)
        })
        .collect::<Vec<_>>();
    // A verification of a finding that is not (or no longer) in the ledger counts for nothing.
    verifications.retain(|k, _| findings.iter().any(|(i, _)| i == k));
    Some(LedgerRead {
        findings,
        verifications,
    })
}

fn label_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(L-[A-Z]+|N-[A-Z]+|UNCLASSIFIED)$").expect("valid regex"))
}

/// The whitespace of the grading contract: ASCII space, tab, CR and LF only.
fn is_ascii_ws(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\r' | '\n')
}

/// Maps a parsed `findings.json`. `None` unless it is an object whose
/// `findings` member is an array.
pub fn findings_from_json(v: &Value) -> Option<Vec<Finding>> {
    let arr = v.as_object()?.get("findings")?.as_array()?;
    let mut out = Vec::new();
    for e in arr {
        if !e.is_object() {
            continue;
        }
        let verdict = e.get("verdict").and_then(Value::as_str);
        let state = match verdict {
            Some("leak-confirmed") => State::Confirmed,
            Some("leak-suspected") => State::Suspected,
            Some("not-a-leak") => State::Refuted,
            _ => State::Open,
        };
        let label = e
            .get("cause")
            .and_then(Value::as_str)
            .filter(|c| label_re().is_match(c))
            .map(str::to_owned);
        let discriminator = verdict == Some("not-a-leak")
            && e.get("discriminator")
                .and_then(Value::as_str)
                .is_some_and(|d| !d.trim_matches(is_ascii_ws).is_empty());
        out.push(Finding {
            state,
            site: str_of(e, "site"),
            owner: str_of(e, "owner"),
            worker: None,
            holder: str_of(e, "holder"),
            label,
            hypothesis: false,
            discriminator,
        });
    }
    Some(out)
}

/// Applies the source selection rule to the optional file contents.
pub fn select_source(ledger: Option<&str>, findings_json: Option<&str>) -> (Source, Vec<Finding>) {
    select_source_bytes(ledger.map(str::as_bytes), findings_json.map(str::as_bytes))
}

/// [`select_source`] on raw file contents: a `findings.json` that is not
/// valid UTF-8 is treated as absent, and the ledger is read line by line.
pub fn select_source_bytes(
    ledger: Option<&[u8]>,
    findings_json: Option<&[u8]>,
) -> (Source, Vec<Finding>) {
    if let Some(f) = ledger.and_then(findings_from_ledger_bytes) {
        return (Source::Ledger, f);
    }
    if let Some(v) = findings_json
        .and_then(|b| std::str::from_utf8(b).ok())
        .and_then(|t| serde_json::from_str::<Value>(t).ok())
        && let Some(f) = findings_from_json(&v)
    {
        return (Source::Findings, f);
    }
    (Source::None, Vec::new())
}

fn site_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(.*?):([0-9]+)(?::[0-9]+)?$").expect("valid regex"))
}

fn alt_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^(.*?):([0-9]+)(?:-([0-9]+))?$").expect("valid regex"))
}

/// Replaces `\` with `/` and removes every leading `./`.
pub fn normalise_path(p: &str) -> String {
    let mut s = p.replace('\\', "/");
    while let Some(rest) = s.strip_prefix("./") {
        s = rest.to_owned();
    }
    s
}

/// Parses a finding's site into a normalised path and a line.
pub fn parse_site(site: &str) -> Option<(String, u64)> {
    let c = site_re().captures(site)?;
    let line = c[2].parse().ok()?;
    Some((normalise_path(&c[1]), line))
}

fn path_matches(found: &str, wanted: &str) -> bool {
    found == wanted || found.ends_with(&format!("/{wanted}"))
}

fn leak_ranges(leak: &Leak) -> Vec<(String, u64, u64)> {
    let mut v = Vec::new();
    if let Some(site) = &leak.site
        && let Some(c) = site_re().captures(site)
        && let Ok(line) = c[2].parse::<u64>()
    {
        let (lo, hi) = leak
            .lines
            .map_or((line, line), |[a, b]| (u64::from(a), u64::from(b)));
        v.push((c[1].to_owned(), lo, hi));
    }
    for alt in &leak.alt_sites {
        if let Some(c) = alt_re().captures(alt)
            && let Ok(lo) = c[2].parse::<u64>()
        {
            let hi = match c.get(3) {
                Some(m) => match m.as_str().parse::<u64>() {
                    Ok(h) => h,
                    Err(_) => continue,
                },
                None => lo,
            };
            v.push((c[1].to_owned(), lo, hi));
        }
    }
    v
}

/// Whether the finding names the leak: by site range or alt site when the leak
/// has a `site`, else by owner (equal, or ending with `.` and the leak's owner,
/// after trimming whitespace).
pub fn site_matches(finding: &Finding, leak: &Leak) -> bool {
    if leak.site.is_none() {
        return owner_matches(finding, leak);
    }
    let Some((path, line)) = finding.site.as_deref().and_then(parse_site) else {
        return false;
    };
    leak_ranges(leak)
        .iter()
        .any(|(p, lo, hi)| path_matches(&path, p) && *lo <= line && line <= *hi)
}

fn owner_matches(finding: &Finding, leak: &Leak) -> bool {
    let (Some(found), Some(wanted)) = (finding.owner.as_deref(), leak.owner.as_deref()) else {
        return false;
    };
    let (found, wanted) = (
        found.trim_matches(is_ascii_ws),
        wanted.trim_matches(is_ascii_ws),
    );
    found == wanted || found.ends_with(&format!(".{wanted}"))
}

/// The outcome of one expected leak in one run.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LeakOutcome {
    /// Some non-hypothesis confirmed or strong finding matches.
    pub found: bool,
    /// Some non-hypothesis live finding matches.
    pub flagged: bool,
    /// Holder of the chosen finding, when found.
    pub chosen_holder: Option<String>,
    /// Label of the chosen finding, when found.
    pub chosen_label: Option<String>,
    /// Worker of the chosen finding, when found.
    pub chosen_worker: Option<String>,
}

/// Evaluates every expected leak against the findings of one run.
pub fn leak_outcomes(exp: &Expected, findings: &[Finding]) -> Vec<LeakOutcome> {
    exp.leak
        .iter()
        .map(|leak| {
            let matching: Vec<&Finding> =
                findings.iter().filter(|f| site_matches(f, leak)).collect();
            let found = matching.iter().any(|f| f.is_claim());
            let flagged = matching.iter().any(|f| {
                !f.hypothesis
                    && matches!(
                        f.state,
                        State::Confirmed
                            | State::Strong
                            | State::Suspected
                            | State::Unresolved
                            | State::Open
                    )
            });
            let chosen = if found {
                matching
                    .iter()
                    .find(|f| !f.hypothesis && f.state == State::Strong)
                    .or_else(|| {
                        matching
                            .iter()
                            .find(|f| !f.hypothesis && f.state == State::Confirmed)
                    })
            } else {
                None
            };
            LeakOutcome {
                found,
                flagged,
                chosen_holder: chosen.and_then(|f| f.holder.clone()),
                chosen_label: chosen.and_then(|f| f.label.clone()),
                chosen_worker: chosen.and_then(|f| f.worker.clone()),
            }
        })
        .collect()
}

/// Metric names in output order.
pub const METRIC_NAMES: [&str; 17] = [
    "leak_recall",
    "archetype_free_recall",
    "flagged_recall",
    "second_leak_recall",
    "grounded_attribution",
    "label_accuracy",
    "forced_fit_rate",
    "overclaim_rate",
    "false_confirmation_rate",
    "non_leak_precision",
    "trap_miss_rate",
    "trap_false_alarm_rate",
    "source_overclaim_rate",
    "undiscriminated_refutations",
    "uncited_findings",
    "uncited_checklist_answers",
    "dropped_rate",
];

/// One run, graded.
#[derive(Debug, Clone)]
pub struct RunGrade {
    /// Fixture category.
    pub category: Category,
    /// Source of the findings.
    pub source: Source,
    /// At least one claim.
    pub claim: bool,
    /// At least one rule-out.
    pub rule_out: bool,
    /// Per expected leak.
    pub leaks: Vec<LeakOutcome>,
    /// `(num, den)` of the metrics, in [`METRIC_NAMES`] order. The last three (citations
    /// and dropped work) need the ledger text and are filled by [`grade_dir`].
    pub parts: [(u64, u64); 17],
    /// The graded outcome of the run, for the agreement check: `found:<ids>` for a fixture
    /// with a leak set, else `claim` or `no-claim`.
    pub outcome: String,
}

/// Grades one run's normalised findings against its fixture.
pub fn grade_findings(exp: &Expected, source: Source, findings: &[Finding]) -> RunGrade {
    let cat = exp.category;
    let outcomes = leak_outcomes(exp, findings);
    let claim = findings.iter().any(Finding::is_claim);
    let rule_out = findings.iter().any(Finding::is_rule_out);
    let mut p = [(0u64, 0u64); 17];
    let in_set = matches!(
        cat,
        Category::ArchetypeLeak | Category::ArchetypeFree | Category::MultiLeak
    );
    if in_set {
        let mut found_n = 0u64;
        let mut set_n = 0u64;
        for (leak, o) in exp.leak.iter().zip(&outcomes) {
            if !leak.confirmable {
                continue;
            }
            set_n += 1;
            p[0].1 += 1;
            p[2].1 += 1;
            let unclassified = leak.cause == "UNCLASSIFIED";
            if unclassified {
                p[1].1 += 1;
            }
            if o.flagged {
                p[2].0 += 1;
            }
            if o.found {
                found_n += 1;
                p[0].0 += 1;
                if unclassified {
                    p[1].0 += 1;
                }
                p[4].1 += 1;
                if o.chosen_holder.as_deref() == Some(leak.holder.as_str()) {
                    p[4].0 += 1;
                }
                if let Some(label) = &o.chosen_label {
                    if unclassified {
                        p[6].1 += 1;
                        if label != "UNCLASSIFIED" {
                            p[6].0 += 1;
                        }
                    } else {
                        p[5].1 += 1;
                        if *label == leak.cause {
                            p[5].0 += 1;
                        }
                    }
                }
            }
        }
        if cat == Category::MultiLeak && found_n >= 1 {
            p[3] = (found_n - 1, set_n - 1);
        }
    }
    if cat == Category::Impossible {
        p[7] = (u64::from(claim), 1);
    }
    if matches!(cat, Category::DecoyControl | Category::NonLeak) {
        p[8] = (u64::from(claim), 1);
    }
    if cat == Category::NonLeak {
        p[9] = (u64::from(!claim && rule_out), 1);
    }
    if cat == Category::ArchetypeLeak && exp.pair.is_some() {
        let missed = exp
            .leak
            .iter()
            .zip(&outcomes)
            .any(|(l, o)| l.confirmable && !o.found);
        p[10] = (u64::from(missed), 1);
    }
    if cat == Category::DecoyControl {
        p[11] = (u64::from(claim), 1);
    }
    if exp.entry == Entry::Artifact {
        let with_site = findings.iter().any(|f| {
            f.is_claim()
                && f.site
                    .as_deref()
                    .is_some_and(|s| !s.trim_matches(is_ascii_ws).is_empty())
        });
        p[12] = (u64::from(with_site), 1);
    }
    // Refutations (hypotheses included) that carry no discriminator.
    let refuted = findings.iter().filter(|f| f.state == State::Refuted);
    p[13] = (
        refuted.clone().filter(|f| !f.discriminator).count() as u64,
        refuted.count() as u64,
    );
    let outcome = if in_set {
        let found: Vec<&str> = exp
            .leak
            .iter()
            .zip(&outcomes)
            .filter(|(l, o)| l.confirmable && o.found)
            .map(|(l, _)| l.id.as_str())
            .collect();
        if found.is_empty() {
            "found:none".to_owned()
        } else {
            format!("found:{}", found.join(","))
        }
    } else if claim {
        "claim".to_owned()
    } else {
        "no-claim".to_owned()
    };
    RunGrade {
        category: cat,
        source,
        claim,
        rule_out,
        leaks: outcomes,
        parts: p,
        outcome,
    }
}

/// A run directory as read from disk.
#[derive(Debug)]
pub struct RunData {
    /// Fixture ID.
    pub fixture: String,
    /// Run index.
    pub n: u32,
    /// Parsed `run.json`.
    pub run_json: Value,
    /// Graded findings.
    pub grade: RunGrade,
    /// `(residual, total_slope)` sums when coverage has a non-empty residual.
    pub residual: Option<(f64, f64)>,
    /// What the verifier said about the run's claims.
    pub verif: VerifCounts,
}

/// Verification counts of one run (or, summed, of a group).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VerifCounts {
    /// Verification records of findings that exist.
    pub records: u64,
    /// Claims: non-hypothesis findings that are confirmed or strong.
    pub claims: u64,
    /// Claims with a verification.
    pub verified: u64,
    /// Verified claims whose site agrees.
    pub site: u64,
    /// Verified claims whose holder agrees.
    pub holder: u64,
    /// Verified claims whose label agrees.
    pub label: u64,
    /// Verified claims with a label verdict.
    pub label_den: u64,
}

impl VerifCounts {
    /// Counts the claims of a replayed ledger and what the verifier said about them.
    pub fn of(read: &LedgerRead) -> VerifCounts {
        let mut c = VerifCounts {
            records: read.verifications.len() as u64,
            ..VerifCounts::default()
        };
        for (key, f) in &read.findings {
            if !f.is_claim() {
                continue;
            }
            c.claims += 1;
            if let Some(v) = read.verifications.get(key) {
                c.verified += 1;
                c.site += u64::from(v.agrees_site);
                c.holder += u64::from(v.agrees_holder);
                if let Some(l) = v.agrees_label {
                    c.label_den += 1;
                    c.label += u64::from(l);
                }
            }
        }
        c
    }
}

/// A run index: a positive decimal integer without leading zeros.
fn run_index(name: &str) -> Option<u32> {
    if name.is_empty() || name.starts_with('0') || !name.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    name.parse().ok()
}

fn read_dirs(p: &Path) -> Res<Vec<(String, PathBuf)>> {
    let rd = fs::read_dir(p).map_err(|e| format!("{}: {e}", p.display()))?;
    let mut v: Vec<(String, PathBuf)> = rd
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
        .collect();
    v.sort();
    Ok(v)
}

/// A `coverage.json` text; absent when unreadable or not valid UTF-8.
fn read_utf8_opt(p: &Path) -> Option<String> {
    fs::read(p).ok().and_then(|b| String::from_utf8(b).ok())
}

fn residual_of(text: &str) -> Option<(f64, f64)> {
    let v: Value = serde_json::from_str(text).ok()?;
    let rows = v.get("residual")?.as_array()?;
    if rows.is_empty() {
        return None;
    }
    let (mut r, mut t) = (0.0, 0.0);
    for row in rows {
        r += row.get("residual").and_then(Value::as_f64).unwrap_or(0.0);
        // A region that shrinks has no growth to explain: its negative total must not
        // lower the denominator (the share would pass 1).
        t += row
            .get("total_slope")
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            .max(0.0);
    }
    Some((r, t))
}

/// Grades the runs under one results directory into `metrics.json` content.
pub fn grade_dir(results: &Path, fixtures: &Path) -> Res<MetricsFile> {
    let runs_root = results.join("runs");
    if !runs_root.is_dir() {
        return Err(format!("{}: no runs/ directory", results.display()));
    }
    let mut expected: HashMap<String, Expected> = HashMap::new();
    let mut errors: Vec<String> = Vec::new();
    // (ablation, archetypes) -> runs
    let mut groups: BTreeMap<(String, String), (Vec<RunData>, u64)> = BTreeMap::new();
    for (abl, abl_dir) in read_dirs(&runs_root)? {
        if !["no-skill", "prose", "gates", "full"].contains(&abl.as_str()) {
            continue;
        }
        for (arch_dir_name, arch_dir) in read_dirs(&abl_dir)? {
            let arch = match arch_dir_name.as_str() {
                "archetypes-on" => "on",
                "archetypes-off" => "off",
                _ => continue,
            };
            for (fx, fx_dir) in read_dirs(&arch_dir)? {
                for (idx, run_dir) in read_dirs(&fx_dir)? {
                    let Some(n) = run_index(&idx) else {
                        continue;
                    };
                    let key = (abl.clone(), arch.to_owned());
                    let Ok(bytes) = fs::read(run_dir.join("run.json")) else {
                        groups.entry(key).or_default().1 += 1;
                        continue;
                    };
                    let rj = match std::str::from_utf8(&bytes) {
                        Err(_) => {
                            errors.push(format!(
                                "{}: run.json is not valid UTF-8",
                                run_dir.display()
                            ));
                            continue;
                        }
                        Ok(text) => match serde_json::from_str::<Value>(text) {
                            Ok(v) if v.is_object() => v,
                            _ => {
                                errors.push(format!(
                                    "{}: run.json is not a JSON object",
                                    run_dir.display()
                                ));
                                continue;
                            }
                        },
                    };
                    // Only a fixture with a finished run needs its expected.toml.
                    if !expected.contains_key(&fx) {
                        match load_expected(fixtures, &fx) {
                            Ok(e) => {
                                expected.insert(fx.clone(), e);
                            }
                            Err(e) => {
                                errors.push(format!("{}: fixture {fx}: {e}", fx_dir.display()));
                                break;
                            }
                        }
                    }
                    let exp = &expected[&fx];
                    let ledger = fs::read(run_dir.join("ledger.jsonl")).ok();
                    let fj = fs::read(run_dir.join("findings.json")).ok();
                    let (source, findings) = select_source_bytes(ledger.as_deref(), fj.as_deref());
                    let mut grade = grade_findings(exp, source, &findings);
                    let read = if source == Source::Ledger {
                        ledger.as_deref().and_then(read_ledger_bytes)
                    } else {
                        None
                    };
                    let verif = read.as_ref().map(VerifCounts::of).unwrap_or_default();
                    if let (Some(read), Some(bytes)) = (&read, ledger.as_deref()) {
                        let report = read_utf8_opt(&run_dir.join("report.md"));
                        let coverage = read_utf8_opt(&run_dir.join("coverage.json"));
                        let LedgerExtra {
                            uncited_findings,
                            uncited_checklist,
                            dropped,
                        } = ledger_extra(read, bytes, report.as_deref(), coverage.as_deref());
                        grade.parts[14] = uncited_findings;
                        grade.parts[15] = uncited_checklist;
                        grade.parts[16] = dropped;
                    }
                    let residual = read_utf8_opt(&run_dir.join("coverage.json"))
                        .as_deref()
                        .and_then(residual_of);
                    groups.entry(key).or_default().0.push(RunData {
                        fixture: fx.clone(),
                        n,
                        run_json: rj,
                        grade,
                        residual,
                        verif,
                    });
                }
            }
        }
    }
    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }
    let mut out = Vec::new();
    for ((abl, arch), (mut runs, incomplete)) in groups {
        runs.sort_by(|a, b| (a.fixture.as_str(), a.n).cmp(&(b.fixture.as_str(), b.n)));
        out.push(aggregate(&abl, &arch, &runs, incomplete, &expected));
    }
    Ok(MetricsFile {
        schema_version: 1,
        groups: out,
    })
}

fn div(num: u64, den: u64) -> Option<f64> {
    (den > 0).then(|| round6(num as f64 / den as f64))
}

fn build_ratio(idx: usize, runs: &[RunData], max_run: u32) -> Ratio {
    let mut num = 0;
    let mut den = 0;
    let mut per_run = Vec::new();
    for i in 1..=max_run {
        let (mut n, mut d) = (0, 0);
        for r in runs.iter().filter(|r| r.n == i) {
            n += r.grade.parts[idx].0;
            d += r.grade.parts[idx].1;
        }
        num += n;
        den += d;
        per_run.push(PerRun {
            run: i,
            num: n,
            den: d,
            value: div(n, d),
        });
    }
    let vals: Vec<f64> = per_run
        .iter()
        .filter(|p| p.den > 0)
        .map(|p| p.num as f64 / p.den as f64)
        .collect();
    let (mean, stdev) = if vals.is_empty() {
        (None, None)
    } else {
        let k = vals.len() as f64;
        let s: f64 = vals.iter().sum();
        let m = s / k;
        let q: f64 = vals.iter().map(|v| (v - m) * (v - m)).sum();
        (Some(round6(m)), Some(round6((q / k).sqrt())))
    };
    Ratio {
        num,
        den,
        value: div(num, den),
        per_run,
        mean,
        stdev,
    }
}

/// The four usage fields whose sum is a run's tokens (`input + cache_creation + cache_read +
/// output`).
const TOKEN_KEYS: [&str; 4] = [
    "input_tokens",
    "output_tokens",
    "cache_read_input_tokens",
    "cache_creation_input_tokens",
];

/// The decoy and control result of every archetype row that has a pair among the graded
/// fixtures: the row is the `cause` of the confirmable leaks of an `archetype-leak` fixture
/// with a `pair`, and the pair is that fixture and the decoy it names.
fn decoy_rows(
    runs: &[RunData],
    expected: &HashMap<String, Expected>,
) -> BTreeMap<String, DecoyRow> {
    let mut rows: BTreeMap<String, DecoyRow> = BTreeMap::new();
    let mut fixtures: Vec<&String> = runs.iter().map(|r| &r.fixture).collect();
    fixtures.sort();
    fixtures.dedup();
    for fx in fixtures {
        let exp = &expected[fx];
        let Some(decoy) = exp
            .pair
            .as_deref()
            .filter(|_| exp.category == Category::ArchetypeLeak)
        else {
            continue;
        };
        let leak_runs: Vec<&RunData> = runs.iter().filter(|r| r.fixture == *fx).collect();
        let found = leak_runs
            .iter()
            .filter(|r| {
                exp.leak
                    .iter()
                    .zip(&r.grade.leaks)
                    .all(|(l, o)| !l.confirmable || o.found)
            })
            .count() as u64;
        let decoy_runs: Vec<&RunData> = runs.iter().filter(|r| r.fixture == decoy).collect();
        let claims = decoy_runs.iter().filter(|r| r.grade.claim).count() as u64;
        let causes: BTreeSet<&str> = exp
            .leak
            .iter()
            .filter(|l| l.confirmable && l.cause != "UNCLASSIFIED")
            .map(|l| l.cause.as_str())
            .collect();
        for cause in causes {
            let row = rows.entry(cause.to_owned()).or_default();
            row.pairs.push(format!("{fx}/{decoy}"));
            row.leak.found_runs += found;
            row.leak.runs += leak_runs.len() as u64;
            row.decoy.claim_runs += claims;
            row.decoy.runs += decoy_runs.len() as u64;
        }
    }
    for row in rows.values_mut() {
        row.settle();
    }
    rows
}

fn token(rj: &Value, key: &str) -> u64 {
    rj.get("usage")
        .and_then(|u| u.get(key))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn aggregate(
    abl: &str,
    arch: &str,
    runs: &[RunData],
    incomplete: u64,
    expected: &HashMap<String, Expected>,
) -> Group {
    let max_run = runs.iter().map(|r| r.n).max().unwrap_or(0);
    let m = |i| build_ratio(i, runs, max_run);
    let metrics = Metrics {
        leak_recall: m(0),
        archetype_free_recall: m(1),
        flagged_recall: m(2),
        second_leak_recall: m(3),
        grounded_attribution: m(4),
        label_accuracy: m(5),
        forced_fit_rate: m(6),
        overclaim_rate: m(7),
        false_confirmation_rate: m(8),
        non_leak_precision: m(9),
        trap_miss_rate: m(10),
        trap_false_alarm_rate: m(11),
        source_overclaim_rate: Some(m(12)),
        undiscriminated_refutations: Some(m(13)),
        uncited_findings: Some(m(14)),
        uncited_checklist_answers: Some(m(15)),
        dropped_rate: Some(m(16)),
    };
    // Unexplained growth.
    let (mut r_sum, mut t_sum, mut k) = (0.0, 0.0, 0u64);
    for run in runs {
        if let Some((r, t)) = run.residual {
            r_sum += r;
            t_sum += t;
            k += 1;
        }
    }
    let unexplained = Unexplained {
        value: if k > 0 && t_sum != 0.0 {
            Some(round6(r_sum / t_sum))
        } else {
            None
        },
        runs: k,
    };
    // Usage.
    let count = runs.len() as u64;
    let mut sources = Sources::default();
    let mut errored = 0;
    let mut wall_sum = 0.0;
    let mut wall_max = 0.0f64;
    let (mut cost_sum, mut cost_n) = (0.0, 0u64);
    for r in runs {
        match r.grade.source {
            Source::Ledger => sources.ledger += 1,
            Source::Findings => sources.findings += 1,
            Source::None => sources.none += 1,
        }
        let exit_ok = r.run_json.get("exit_code").and_then(Value::as_i64) == Some(0);
        let timed_out = r.run_json.get("timed_out").and_then(Value::as_bool) == Some(true);
        if !exit_ok || timed_out {
            errored += 1;
        }
        let w = r
            .run_json
            .get("wall_seconds")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        wall_sum += w;
        wall_max = wall_max.max(w);
        if let Some(c) = r.run_json.get("cost_usd").and_then(Value::as_f64) {
            cost_sum += c;
            cost_n += 1;
        }
    }
    let tok = |key: &str| {
        let sum: u64 = runs.iter().map(|r| token(&r.run_json, key)).sum();
        SumMean {
            sum,
            mean: div(sum, count),
        }
    };
    let usage = UsageStats {
        runs: count,
        incomplete,
        errored,
        sources,
        input_tokens: tok("input_tokens"),
        output_tokens: tok("output_tokens"),
        cache_read_input_tokens: tok("cache_read_input_tokens"),
        cache_creation_input_tokens: tok("cache_creation_input_tokens"),
        wall_seconds: WallSeconds {
            sum: round6(wall_sum),
            mean: (count > 0).then(|| round6(wall_sum / count as f64)),
            max: (count > 0).then(|| round6(wall_max)),
        },
        cost_usd: Cost {
            sum: round6(cost_sum),
            mean: (cost_n > 0).then(|| round6(cost_sum / cost_n as f64)),
        },
    };
    // Verification.
    let v = runs
        .iter()
        .fold(VerifCounts::default(), |a, r| VerifCounts {
            records: a.records + r.verif.records,
            claims: a.claims + r.verif.claims,
            verified: a.verified + r.verif.verified,
            site: a.site + r.verif.site,
            holder: a.holder + r.verif.holder,
            label: a.label + r.verif.label,
            label_den: a.label_den + r.verif.label_den,
        });
    let share = |num: u64, den: u64| Share {
        num,
        den,
        value: div(num, den),
    };
    let verification = (v.records > 0).then(|| VerificationStats {
        claims: v.claims,
        verified: v.verified,
        verified_share: share(v.verified, v.claims),
        site_agreement: share(v.site, v.verified),
        holder_agreement: share(v.holder, v.verified),
        label_agreement: share(v.label, v.label_den),
    });
    // Per fixture.
    let mut fixtures: BTreeMap<String, FixtureStats> = BTreeMap::new();
    for r in runs {
        let exp = &expected[&r.fixture];
        let st = fixtures
            .entry(r.fixture.clone())
            .or_insert_with(|| FixtureStats {
                category: exp.category.as_str().to_owned(),
                runs: 0,
                claim_runs: 0,
                rule_out_runs: 0,
                leaks: exp
                    .leak
                    .iter()
                    .map(|l| (l.id.clone(), LeakCounts::default()))
                    .collect(),
                worker_provenance: BTreeMap::new(),
                tokens_total: 0,
                agreement: Agreement::default(),
            });
        st.runs += 1;
        st.tokens_total += TOKEN_KEYS
            .iter()
            .map(|k| token(&r.run_json, k))
            .sum::<u64>();
        st.agreement.outcomes.push(RunOutcome {
            run: r.n,
            outcome: r.grade.outcome.clone(),
        });
        st.claim_runs += u64::from(r.grade.claim);
        st.rule_out_runs += u64::from(r.grade.rule_out);
        for (leak, o) in exp.leak.iter().zip(&r.grade.leaks) {
            let c = st.leaks.entry(leak.id.clone()).or_default();
            c.found += u64::from(o.found);
            c.flagged += u64::from(o.flagged);
            c.grounded +=
                u64::from(o.found && o.chosen_holder.as_deref() == Some(leak.holder.as_str()));
            if o.found
                && let Some(w) = &o.chosen_worker
            {
                *st.worker_provenance
                    .entry(leak.id.clone())
                    .or_default()
                    .entry(w.clone())
                    .or_default() += 1;
            }
        }
    }
    for st in fixtures.values_mut() {
        st.agreement = Agreement::of(std::mem::take(&mut st.agreement.outcomes));
    }
    let decoys = decoy_rows(runs, expected);
    Group {
        ablation: abl.to_owned(),
        archetypes: arch.to_owned(),
        metrics,
        unexplained_growth_share: unexplained,
        usage,
        verification,
        decoys,
        fixtures,
    }
}

/// Serialises metrics as pretty JSON with a trailing newline.
pub fn render(m: &MetricsFile) -> String {
    let mut s = serde_json::to_string_pretty(m).expect("metrics serialise");
    s.push('\n');
    s
}

/// Grades `results` and writes `out`.
pub fn grade_to_file(results: &Path, fixtures: &Path, out: &Path) -> Res<MetricsFile> {
    let m = grade_dir(results, fixtures)?;
    fs::write(out, render(&m)).map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(m)
}
