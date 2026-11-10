//! The M6 part of the fixed policy: label, explain, intervene, strong and the final gate.
//!
//! It runs once per fixture after the investigation steps ([`super::drive`]) and reads only
//! the JSON that `dla` prints and the ledger file. The investigation itself is unchanged.
//!
//! 1. Explain. Every confirmed leak-side finding is explained, with flags and text built
//!    from ledger facts: the site, the holder, the owner, and the recorded writers and
//!    removers. The mechanical policy cannot read the code, so when no writer is recorded
//!    (the census found none and nobody annotated one) it records the finding's site as the
//!    adding site: the site is the line the leak was located at. A refused explain is a
//!    failure. A finding with no site (an artifact run has no source) cannot get a writer;
//!    it is reported as a gap of the gate, not as a failure of the policy.
//! 2. Intervene. On a repository fixture, `dla intervene --stub-writer` for each confirmed
//!    leak with recorded writers, on the harness the finding was confirmed on. The effect is
//!    reported; it never fails the fixture. It runs before the label because some rows accept
//!    an intervention as their second record.
//! 3. Label. Every confirmed leak-side finding that matches an expected leak whose cause is
//!    a shipped archetype row gets `ledger label F --as <cause>`; a leak whose cause is
//!    `UNCLASSIFIED` gets `--as UNCLASSIFIED --note`. When the ledger refuses for missing
//!    evidence the policy follows the fix it prints (`--evidence E..`, or the summarize
//!    commands that already ran, again with `--finding F`). A refused label of a shipped row
//!    is a failure (the report says whether the label is reachable). With archetypes off the
//!    labels of shipped rows must be refused with `archetypes-off`; `UNCLASSIFIED` stays
//!    allowed; `--suggest` must say that archetypes are off.
//! 4. Strong. `ledger strong` with each eligible record (an intervention that removed the
//!    growth, or a retention record) when one exists. Reported, never gating.
//! 5. Close-out. `dla gate check --final` must pass. What it names is done with the command
//!    it names (answer the intake checklist, analyse a series, tie a trend column to a
//!    finding and disposition that finding, resolve the residual open item); a column that
//!    only a re-run of `dla trend` is offered for cannot be reached and is reported as a gap.
//!    Anything else still missing fails the fixture and is named in the failure.

use std::{collections::BTreeSet, fs, path::Path};

use serde::Serialize;
use serde_json::Value;

use super::{Fnd, Fx, Out, View, ops, strings};
use crate::{
    grade::{self, State},
    model::{Entry, Expected, Leak},
};

/// The archetype rows with a prose row in a shipped `references/<lang>-archetypes.md`
/// (spec D1). `tests/archetype_lists.rs` keeps this equal to the rows of those files.
pub const SHIPPED_ROWS: [&str; 9] = [
    "L-CAPTURE",
    "L-COLLECT",
    "L-CYCLE",
    "L-HANDLE",
    "L-OWNERSHIP",
    "L-REGISTER",
    "L-TLS",
    "L-UNLOAD",
    "L-WORKER",
];

/// Rounds of the close-out loop at most.
const MAX_ROUNDS: usize = 6;

/// Writers one intervention stubs at most.
const MAX_STUBS: usize = 3;

/// Remedy text of the dispositions the close-out records.
const REMEDY: &str = "the fixed bench policy does not investigate this further; re-run dla alloc-test or dla trend with a narrower probe";

/// One step of the policy for one finding.
#[derive(Serialize, Debug, Clone, Default, PartialEq)]
pub struct Note {
    /// The finding.
    pub finding: String,
    /// What was asked (the expected leak and cause, the stubbed writers, the evidence).
    pub detail: String,
    /// What `dla` answered: `ok`, `refused <code>`, or why the step did not run.
    pub result: String,
}

/// The final gate of a fixture.
#[derive(Serialize, Debug, Clone, Default, PartialEq)]
pub struct GateNote {
    /// `dla gate check --final` exits 0 (possibly after the close-out steps).
    pub passed: bool,
    /// Nothing is missing except known gaps of the gate itself (`passed`, or the rest is
    /// listed in `known_gaps`).
    pub ok: bool,
    /// Rounds of the close-out loop that ran.
    pub rounds: usize,
    /// Items the gate named at the start.
    pub first_missing: usize,
    /// Items still missing at the end, as `kind id: description`, outside the known gaps.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
    /// Items left that the policy cannot reach, with the reason.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub known_gaps: Vec<String>,
}

/// What the policy did for one fixture.
#[derive(Serialize, Debug, Clone, Default, PartialEq)]
pub struct PolicyRow {
    /// Labels.
    pub labels: Vec<Note>,
    /// Explanations.
    pub explains: Vec<Note>,
    /// Interventions.
    pub interventions: Vec<Note>,
    /// `ledger strong` attempts.
    pub strong: Vec<Note>,
    /// The final gate.
    pub gate: Option<GateNote>,
}

impl PolicyRow {
    /// Nothing was recorded.
    pub fn is_empty(&self) -> bool {
        *self == PolicyRow::default()
    }
}

/// A confirmed or strong leak-side finding and the expected leak it matches.
struct Target {
    f: Fnd,
    leak: Option<Leak>,
}

fn as_graded(f: &Fnd) -> grade::Finding {
    grade::Finding {
        state: State::Confirmed,
        site: f.site.clone(),
        owner: f.owner.clone(),
        worker: None,
        holder: f.holder.clone(),
        label: f.label.clone(),
        hypothesis: false,
        discriminator: false,
    }
}

fn targets(view: &View, expected: &Expected) -> Vec<Target> {
    view.findings
        .iter()
        .filter(|f| matches!(f.state.as_str(), "confirmed" | "strong") && !f.is_nonleak())
        .map(|f| {
            let g = as_graded(f);
            Target {
                leak: expected
                    .leak
                    .iter()
                    .find(|l| grade::site_matches(&g, l))
                    .cloned(),
                f: f.clone(),
            }
        })
        .collect()
}

/// The writers and removers the census recorded for the finding's owner.
fn census_sites(run_dir: &Path, f: &Fnd) -> (Vec<String>, Vec<String>) {
    let mut writers = Vec::new();
    let mut removers = Vec::new();
    let Ok(rd) = fs::read_dir(run_dir.join("census")) else {
        return (writers, removers);
    };
    let mut files: Vec<_> = rd
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    for p in files {
        let Some(v) = fs::read_to_string(&p)
            .ok()
            .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        else {
            continue;
        };
        for e in v["entries"].as_array().into_iter().flatten() {
            let same_owner = f
                .owner
                .as_deref()
                .is_some_and(|o| e["owner"].as_str() == Some(o));
            // `ledger explain` resolves the census by the finding's owner only, so the
            // facts used here have to come the same way.
            if same_owner {
                for w in strings(&e["writers"]) {
                    if !writers.contains(&w) {
                        writers.push(w);
                    }
                }
                for r in strings(&e["removers"]) {
                    if !removers.contains(&r) {
                        removers.push(r);
                    }
                }
            }
        }
    }
    (writers, removers)
}

/// The recorded writers and removers of a finding: annotations, then the census.
fn recorded_sites(run_dir: &Path, f: &Fnd) -> (Vec<String>, Vec<String>) {
    let (cw, cr) = census_sites(run_dir, f);
    let mut writers = f.writers.clone();
    let mut removers = f.removers.clone();
    for w in cw {
        if !writers.contains(&w) {
            writers.push(w);
        }
    }
    for r in cr {
        if !removers.contains(&r) {
            removers.push(r);
        }
    }
    (writers, removers)
}

fn short(msg: &str) -> String {
    msg.chars().take(160).collect()
}

/// `refused <code>` or `no JSON`.
fn refusal(out: &Out) -> String {
    if out.code().is_empty() {
        format!("failed: {}", short(&out.message()))
    } else {
        format!("refused {}", out.code())
    }
}

// ---------------------------------------------------------------------------
// 1. Label
// ---------------------------------------------------------------------------

fn label_all(fx: &mut Fx, ts: &[Target], off: bool, row: &mut PolicyRow) {
    for t in ts {
        let id = t.f.id.clone();
        // `--suggest` is read only and works either way; with archetypes off it says so.
        let s = fx.call_quiet(&["ledger", "label", &id, "--suggest"]);
        if !s.ok {
            fx.row.failures.push(format!(
                "label {id} --suggest failed: {}",
                short(&s.message())
            ));
        } else if off && s.json["archetypes"].as_str() != Some("off") {
            fx.row.failures.push(format!(
                "label {id} --suggest does not say archetypes are off"
            ));
        } else if !off && s.json["archetypes"].as_str() != Some("on") {
            fx.row.failures.push(format!(
                "label {id} --suggest does not say archetypes are on"
            ));
        }
        let Some(leak) = &t.leak else {
            row.labels.push(Note {
                finding: id,
                detail: "no expected leak".to_owned(),
                result: "not labelled".to_owned(),
            });
            continue;
        };
        let detail = format!("{} {}", leak.id, leak.cause);
        if t.f.holder.as_deref() != Some(leak.holder.as_str()) {
            // A wrong holder is a failure of the fixture already; the label rule on the
            // holder would only repeat it.
            row.labels.push(Note {
                finding: id,
                detail,
                result: format!(
                    "not tried: holder {} differs from the expected {}",
                    t.f.holder.as_deref().unwrap_or("none"),
                    leak.holder.as_str()
                ),
            });
            continue;
        }
        let cause = leak.cause.as_str();
        if cause == "UNCLASSIFIED" {
            let o = fx.call_quiet(&[
                "ledger",
                "label",
                &id,
                "--as",
                "UNCLASSIFIED",
                "--note",
                "the holder and the mechanism are stated; no shipped row describes this growth",
            ]);
            let result = if o.ok { "ok".to_owned() } else { refusal(&o) };
            if !o.ok {
                fx.row.failures.push(format!(
                    "leak {} (UNCLASSIFIED) label {}",
                    leak.id,
                    refusal(&o)
                ));
                fx.row
                    .errors
                    .push(format!("dla ledger label {id}: {}", o.message()));
            }
            row.labels.push(Note {
                finding: id,
                detail,
                result,
            });
        } else if SHIPPED_ROWS.contains(&cause) {
            let (o, followed) = label_with_the_printed_fixes(fx, &id, cause, off);
            let result = match (off, o.ok, o.code()) {
                (false, true, _) => "ok".to_owned(),
                (true, false, "archetypes-off") => "refused archetypes-off (expected)".to_owned(),
                _ => refusal_or_accept(&o),
            };
            let result = if followed.is_empty() {
                result
            } else {
                format!("{result} (after {})", followed.join(", "))
            };
            match (off, o.ok, o.code()) {
                (false, true, _) | (true, false, "archetypes-off") => {}
                (false, false, _) => {
                    fx.row.failures.push(format!(
                        "leak {} ({cause}) label {}",
                        leak.id,
                        refusal(&o)
                    ));
                    fx.row.errors.push(format!(
                        "dla ledger label {id} --as {cause}: {}",
                        o.message()
                    ));
                }
                (true, true, _) => fx.row.failures.push(format!(
                    "leak {} ({cause}) label was accepted with archetypes off",
                    leak.id
                )),
                (true, false, _) => fx.row.failures.push(format!(
                    "leak {} ({cause}) label {}, expected archetypes-off",
                    leak.id,
                    refusal(&o)
                )),
            }
            row.labels.push(Note {
                finding: id,
                detail,
                result,
            });
        } else {
            row.labels.push(Note {
                finding: id,
                detail,
                result: format!("not tried: {cause} has no shipped row"),
            });
        }
    }
}

/// `ledger label F --as C`, then what a refusal for missing evidence prints as its fix:
/// `--evidence E..` when the ledger has the records but the finding does not cite them, and
/// `dla summarize <op> <inputs> --finding F` when the records have to be tied to the finding
/// first (the summarize commands that already ran are run again narrowed to the finding).
/// Returns the last answer and what was followed.
fn label_with_the_printed_fixes(
    fx: &mut Fx,
    id: &str,
    cause: &str,
    off: bool,
) -> (Out, Vec<String>) {
    let mut followed: Vec<String> = Vec::new();
    let mut o = fx.call_quiet(&["ledger", "label", id, "--as", cause]);
    let mut narrowed: BTreeSet<String> = BTreeSet::new();
    for _ in 0..4 {
        if off || o.ok || o.code() != "label-evidence-missing" {
            break;
        }
        let fix = o.json["error"]["fix"].as_str().unwrap_or("").to_owned();
        if let Some(ev) = evidence_in_label_fix(&fix) {
            followed.push(format!("--evidence {ev}"));
            o = fx.call_quiet(&["ledger", "label", id, "--as", cause, "--evidence", &ev]);
            continue;
        }
        let mut ran = 0;
        for op in summarize_ops_in(&fix) {
            if !narrowed.insert(op.clone()) {
                continue;
            }
            let cmds: Vec<Vec<String>> = fx
                .row
                .summarize_cmds
                .iter()
                .filter(|c| c.get(1).map(String::as_str) == Some(op.as_str()))
                .take(MAX_NARROWED)
                .cloned()
                .collect();
            for mut c in cmds {
                c.extend(["--finding".to_owned(), id.to_owned()]);
                let args: Vec<&str> = c.iter().map(String::as_str).collect();
                fx.call_quiet(&args);
                ran += 1;
            }
            if ran > 0 {
                followed.push(format!("summarize {op} --finding"));
            }
        }
        if ran == 0 {
            break;
        }
        o = fx.call_quiet(&["ledger", "label", id, "--as", cause]);
    }
    (o, followed)
}

/// Summarize commands run again per operation at most.
const MAX_NARROWED: usize = 3;

/// The `--evidence` list of a printed `dla ledger label ... --evidence E1,E2` fix.
fn evidence_in_label_fix(fix: &str) -> Option<String> {
    if !fix.starts_with("dla ledger label ") {
        return None;
    }
    let words: Vec<&str> = fix.split_whitespace().collect();
    let i = words.iter().position(|w| *w == "--evidence")?;
    let ev = words.get(i + 1)?;
    (!ev.starts_with('<') && !ev.is_empty()).then(|| (*ev).to_owned())
}

/// The operations of the `dla summarize <op> ...` commands a fix text names, in order.
fn summarize_ops_in(fix: &str) -> Vec<String> {
    let mut ops: Vec<String> = Vec::new();
    for part in fix.split("dla summarize ").skip(1) {
        let op: String = part
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
            .collect();
        if !op.is_empty() && !ops.contains(&op) {
            ops.push(op);
        }
    }
    ops
}

fn refusal_or_accept(o: &Out) -> String {
    if o.ok {
        "accepted".to_owned()
    } else {
        refusal(o)
    }
}

// ---------------------------------------------------------------------------
// 2. Explain
// ---------------------------------------------------------------------------

/// The text of an explanation, built from the ledger facts only.
fn explanation_text(
    site: &str,
    holder: &str,
    owner: Option<&str>,
    add: &str,
    remover: Option<&str>,
) -> String {
    let held = match owner {
        Some(o) => format!("{holder} holder {o}"),
        None => format!("{holder} holder"),
    };
    match remover {
        None => format!(
            "{site} is where the {held} gains entries, added at {add}; nothing removes entries from it (never removed), so every cycle leaves its entries behind."
        ),
        Some(r) => format!(
            "{site} is where the {held} gains entries, added at {add}; the remover at {r} runs only when its condition holds, so entries it does not match stay and the holder grows."
        ),
    }
}

/// The explanation of an artifact finding: a dump holds no source, so only the owner, the
/// holder and the absence of a remover can be stated.
fn artifact_explain(fx: &mut Fx, id: &str, holder: &str, owner: &str) -> Out {
    let text = format!(
        "{owner} is the {holder} holder that keeps the entries of {id} alive; the dump shows no code, and no remover found for it."
    );
    fx.call_quiet(&[
        "ledger",
        "explain",
        id,
        "--tag",
        "inferred",
        "--removal",
        "not-found",
        "--text",
        &text,
    ])
}

fn explain_all(fx: &mut Fx, ts: &[Target], row: &mut PolicyRow) {
    for t in ts {
        let id = t.f.id.clone();
        if t.f.site.is_none()
            && let (Some(owner), Some(holder)) = (t.f.owner.clone(), t.f.holder.clone())
        {
            let o = artifact_explain(fx, &id, &holder, &owner);
            let result = if o.ok { "ok".to_owned() } else { refusal(&o) };
            if !o.ok {
                fx.row
                    .failures
                    .push(format!("explain of {id} {}", refusal(&o)));
            }
            row.explains.push(Note {
                finding: id,
                detail: "artifact: owner and holder, no remover found".to_owned(),
                result,
            });
            continue;
        }
        let (Some(site), Some(holder)) = (t.f.site.clone(), t.f.holder.clone()) else {
            row.explains.push(Note {
                finding: id,
                detail: String::new(),
                result: "not tried: no site, so no writer can be recorded".to_owned(),
            });
            continue;
        };
        let Ok(run_dir) = fx.run_dir() else { return };
        let view = fx.view();
        let cur = view.finding(&id).cloned().unwrap_or_else(|| t.f.clone());
        let (mut writers, removers) = recorded_sites(&run_dir, &cur);
        if writers.is_empty() {
            let a = fx.call_quiet(&["ledger", "annotate", &id, "--writer", &site]);
            if a.ok {
                fx.step(format!(
                    "{id}: no writer recorded, the site {site} stands as the adding site"
                ));
                writers.push(site.clone());
            } else {
                fx.row.failures.push(format!(
                    "ledger annotate {id} --writer {site} {}",
                    refusal(&a)
                ));
                row.explains.push(Note {
                    finding: id,
                    detail: format!("writer {site}"),
                    result: refusal(&a),
                });
                continue;
            }
        }
        let adds: Vec<String> = writers.iter().take(MAX_STUBS).cloned().collect();
        let remover = removers.first().cloned();
        let text = explanation_text(
            &site,
            &holder,
            cur.owner.as_deref(),
            &adds[0],
            remover.as_deref(),
        );
        let adds_arg = adds.join(",");
        let explain = |fx: &mut Fx, remover: Option<&str>, text: &str| {
            let mut args = vec![
                "ledger",
                "explain",
                id.as_str(),
                "--tag",
                "inferred",
                "--adds",
                adds_arg.as_str(),
            ];
            match remover {
                Some(r) => args.extend(["--removal", "conditional", "--remover", r]),
                None => args.extend(["--removal", "never"]),
            }
            args.extend(["--text", text]);
            fx.call_quiet(&args)
        };
        let mut o = explain(fx, remover.as_deref(), &text);
        let mut remover = remover;
        if !o.ok && remover.is_some() && o.code() == "explain-remover" {
            // The ledger does not know the remover this policy read from the census (the
            // census is resolved by owner there): say that nothing removes.
            fx.step(format!(
                "{id}: the ledger does not record the remover, explained as never removed"
            ));
            remover = None;
            let text = explanation_text(&site, &holder, cur.owner.as_deref(), &adds[0], None);
            o = explain(fx, None, &text);
        }
        let result = if o.ok { "ok".to_owned() } else { refusal(&o) };
        if !o.ok {
            fx.row
                .failures
                .push(format!("explain of {id} {}", refusal(&o)));
            fx.row
                .errors
                .push(format!("dla ledger explain {id}: {}", o.message()));
        }
        row.explains.push(Note {
            finding: id,
            detail: format!(
                "adds {adds_arg}; {}",
                remover
                    .as_deref()
                    .map_or("never removed".to_owned(), |r| format!("remover {r}"))
            ),
            result,
        });
    }
}

// ---------------------------------------------------------------------------
// 3. Intervene, 4. Strong
// ---------------------------------------------------------------------------

/// The harness a finding was confirmed on: the `harness` of the evidence it cites, or of the
/// workload record that lists the series of that evidence.
fn harness_of(view: &View, f: &Fnd, fallback: Option<&str>) -> Option<String> {
    let cited = f
        .confirmed
        .as_ref()
        .map(|c| c.1.clone())
        .unwrap_or_default();
    for id in cited.iter().chain(f.supports.iter()) {
        let Some(e) = view.evidence.get(id) else {
            continue;
        };
        if let Some(h) = e.result["harness"].as_str() {
            return Some(h.to_owned());
        }
        if let Some(series) = e.result["series"].as_str() {
            let hit = view.order.iter().rev().find_map(|w| {
                let w = &view.evidence[w];
                (w.kind == "workload" && strings(&w.result["series"]).iter().any(|s| s == series))
                    .then(|| w.result["harness"].as_str().map(str::to_owned))
                    .flatten()
            });
            if hit.is_some() {
                return hit;
            }
        }
    }
    fallback.map(str::to_owned)
}

/// The effect an `intervene` answer names.
fn effect_of(o: &Out, view: &View) -> Option<String> {
    for p in ["/effect", "/intervention/effect", "/result/effect"] {
        if let Some(e) = o.json.pointer(p).and_then(Value::as_str) {
            return Some(e.to_owned());
        }
    }
    o.evidence().iter().find_map(|id| {
        view.evidence
            .get(id)
            .and_then(|e| e.result["effect"].as_str().map(str::to_owned))
    })
}

fn intervene_all(
    fx: &mut Fx,
    ts: &[Target],
    row: &mut PolicyRow,
    grew: &mut Vec<(String, Vec<String>)>,
) {
    if fx.plan.entry != Entry::Repository || fx.worker.is_some() {
        return;
    }
    if fx.plan.runtimes >= 2 {
        // The harnesses belong to the workers' slices, which the merge has closed.
        for t in ts {
            row.interventions.push(Note {
                finding: t.f.id.clone(),
                detail: String::new(),
                result: "not tried: the harness belongs to a worker".to_owned(),
            });
        }
        return;
    }
    for t in ts {
        let id = t.f.id.clone();
        let Ok(run_dir) = fx.run_dir() else { return };
        let view = fx.view();
        let cur = view.finding(&id).cloned().unwrap_or_else(|| t.f.clone());
        let (writers, _) = recorded_sites(&run_dir, &cur);
        if writers.is_empty() {
            row.interventions.push(Note {
                finding: id,
                detail: String::new(),
                result: "not tried: no recorded writer".to_owned(),
            });
            continue;
        }
        let fallback = fx.row.harnesses.last().cloned();
        let Some(harness) = harness_of(&view, &cur, fallback.as_deref()) else {
            row.interventions.push(Note {
                finding: id,
                detail: String::new(),
                result: "not tried: no harness".to_owned(),
            });
            continue;
        };
        let stubs: Vec<&String> = writers.iter().take(MAX_STUBS).collect();
        let mut args = vec!["intervene", harness.as_str(), "--finding", id.as_str()];
        for w in &stubs {
            args.extend(["--stub-writer", w.as_str()]);
        }
        let o = fx.call_quiet(&args);
        let after = fx.view();
        let result = if o.ok {
            let effect = effect_of(&o, &after).unwrap_or_else(|| "no effect named".to_owned());
            if effect == "growth-removed" {
                grew.push((id.clone(), o.evidence()));
            }
            effect
        } else {
            refusal(&o)
        };
        row.interventions.push(Note {
            finding: id,
            detail: format!(
                "{harness}: stub {}",
                stubs
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            result,
        });
    }
}

fn strong_all(fx: &mut Fx, ts: &[Target], removed: &[(String, Vec<String>)], row: &mut PolicyRow) {
    for t in ts {
        let id = t.f.id.clone();
        let view = fx.view();
        if view.finding(&id).is_some_and(|f| f.state == "strong") {
            continue;
        }
        let mut cands: Vec<String> = removed
            .iter()
            .filter(|(f, _)| *f == id)
            .flat_map(|(_, ev)| ev.iter().cloned())
            .filter(|e| {
                view.evidence
                    .get(e)
                    .is_some_and(|x| x.kind == "intervention")
            })
            .collect();
        cands.extend(
            view.order
                .iter()
                .rev()
                .filter(|e| view.evidence[*e].kind == "retention-path")
                .take(3)
                .cloned(),
        );
        if cands.is_empty() {
            row.strong.push(Note {
                finding: id,
                detail: String::new(),
                result: "no eligible record".to_owned(),
            });
            continue;
        }
        for e in cands {
            let o = fx.call_quiet(&["ledger", "strong", &id, "--evidence", &e]);
            let result = if o.ok { "ok".to_owned() } else { refusal(&o) };
            let done = o.ok;
            row.strong.push(Note {
                finding: id.clone(),
                detail: e,
                result,
            });
            if done {
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 5. Close-out
// ---------------------------------------------------------------------------

/// One item of the final gate.
#[derive(Debug, Clone, PartialEq)]
struct Item {
    kind: String,
    id: String,
    description: String,
    fix: String,
}

fn items_of(fx: &Fx, out: &Out) -> Vec<Item> {
    let from_file = out.json["pointer"]
        .as_str()
        .and_then(|p| fs::read_to_string(fx.project.join(p)).ok())
        .and_then(|t| serde_json::from_str::<Value>(&t).ok());
    let list = from_file
        .as_ref()
        .map(|v| v["missing"].clone())
        .unwrap_or_else(|| out.json["missing"].clone());
    list.as_array()
        .into_iter()
        .flatten()
        .map(|m| Item {
            kind: m["kind"].as_str().unwrap_or("").to_owned(),
            id: m["id"].as_str().unwrap_or("").to_owned(),
            description: m["description"].as_str().unwrap_or("").to_owned(),
            fix: m["fix"].as_str().unwrap_or("").to_owned(),
        })
        .collect()
}

/// The gate items the policy cannot reach, with the reason. None: every item the gate prints
/// is followed; an item left over is a failure of the fixture. The list is kept (and shown in
/// the JSON as `known_gaps`) so that a gap can be named again if DLA cannot satisfy an item.
fn known_gap(_item: &Item, _view: &View, _run_dir: &Path) -> Option<&'static str> {
    None
}

/// A candidate that a detector or the census found, or that cites static evidence: the only
/// evidence it has is static.
fn has_static_evidence(view: &View, id: &str) -> bool {
    view.finding(id).is_some_and(|f| {
        f.state == "candidate"
            && (f.origin.starts_with("detector:")
                || f.origin == "census"
                || f.supports.iter().any(|e| {
                    view.evidence
                        .get(e)
                        .is_some_and(|e| e.authority == "static")
                }))
    })
}

/// Splits the fix text at the comment the gate appends (`   # column os.rss`).
fn fix_command(fix: &str) -> &str {
    // The gate adds a comment after a `#` or, for the residual command, a parenthesis
    // set off by spaces.
    let cut = fix.split(" #").next().unwrap_or(fix);
    cut.split("   (").next().unwrap_or(cut).trim()
}

fn run_dla_line(fx: &mut Fx, line: &str) -> Out {
    let words = ops::split_command(line);
    let args: Vec<&str> = words.iter().skip(1).map(String::as_str).collect();
    fx.call_quiet(&args)
}

fn resolve_unresolved(fx: &mut Fx, target: &str) -> Out {
    fx.call_quiet(&[
        "ledger",
        "resolve",
        target,
        "--as",
        "unresolved",
        "--reason",
        "not_exercised",
        "--remedy",
        REMEDY,
    ])
}

/// One item, with the command the gate names for it. Returns what was done, if anything.
fn fix_item(fx: &mut Fx, item: &Item, tried: &mut BTreeSet<String>) -> Option<String> {
    let cmd = fix_command(&item.fix).to_owned();
    match item.kind.as_str() {
        "checklist_item" | "series" | "slice" | "coverage" => {
            if cmd.contains('<') || !tried.insert(cmd.clone()) {
                return None;
            }
            let o = run_dla_line(fx, &cmd);
            Some(format!(
                "{cmd}: {}",
                if o.ok { "ok".to_owned() } else { refusal(&o) }
            ))
        }
        "open_item" | "finding" => {
            // The fix text carries a placeholder for the remedy; the disposition is the
            // same as the one the investigation steps give: static evidence only, else
            // unresolved.
            if !tried.insert(format!("resolve {}", item.id)) {
                return None;
            }
            let statics = item.kind == "finding" && has_static_evidence(&fx.view(), &item.id);
            let o = if statics {
                fx.call_quiet(&[
                    "ledger",
                    "resolve",
                    &item.id,
                    "--as",
                    "suspected_static_only",
                    "--reason",
                    "not_exercised",
                    "--remedy",
                    REMEDY,
                ])
            } else {
                resolve_unresolved(fx, &item.id)
            };
            Some(format!(
                "resolve {} {}: {}",
                item.id,
                if statics {
                    "suspected_static_only"
                } else {
                    "unresolved"
                },
                if o.ok { "ok".to_owned() } else { refusal(&o) }
            ))
        }
        "explanation" => {
            let f = fx.view().finding(&item.id).cloned()?;
            let (None, Some(owner), Some(holder)) = (f.site, f.owner, f.holder) else {
                return None;
            };
            if !tried.insert(format!("explain {}", item.id)) {
                return None;
            }
            let o = artifact_explain(fx, &item.id, &holder, &owner);
            Some(format!(
                "explain {} (artifact): {}",
                item.id,
                if o.ok { "ok".to_owned() } else { refusal(&o) }
            ))
        }
        "trend_column" if cmd.starts_with("dla trend ") || cmd.starts_with("dla gap ") => {
            // The gate asks for the trend of the series, or for the residual rows of a
            // whole-process column; the next round prints the follow-up.
            if !tried.insert(cmd.clone()) {
                return None;
            }
            let o = run_dla_line(fx, &cmd);
            Some(format!(
                "{cmd}: {}",
                if o.ok { "ok".to_owned() } else { refusal(&o) }
            ))
        }
        "trend_column" => {
            if !cmd.starts_with("dla ledger add --origin growth-series")
                || !tried.insert(cmd.clone())
            {
                return None;
            }
            let o = run_dla_line(fx, &cmd);
            let mut said = format!(
                "{cmd}: {}",
                if o.ok { "ok".to_owned() } else { refusal(&o) }
            );
            if o.ok
                && let Some(new) = o.json["id"].as_str()
            {
                let r = resolve_unresolved(fx, new);
                said.push_str(&format!(
                    "; resolve {new} unresolved: {}",
                    if r.ok { "ok".to_owned() } else { refusal(&r) }
                ));
            }
            Some(said)
        }
        _ => None,
    }
}

fn close_out(fx: &mut Fx, row: &mut PolicyRow) {
    let mut tried = BTreeSet::new();
    let mut note = GateNote::default();
    let mut last: Vec<Item> = Vec::new();
    for round in 0..=MAX_ROUNDS {
        let g = fx.call_quiet(&["gate", "check", "--final"]);
        if g.json.is_null() {
            fx.row.failures.push(format!(
                "gate check --final failed: {}",
                short(&g.message())
            ));
            row.gate = Some(note);
            return;
        }
        if g.ok {
            note.passed = true;
            note.ok = true;
            note.rounds = round;
            last.clear();
            break;
        }
        last = items_of(fx, &g);
        if round == 0 {
            note.first_missing = g.json["missing_total"]
                .as_u64()
                .map_or(last.len(), |n| n as usize);
        }
        note.rounds = round;
        if round == MAX_ROUNDS {
            break;
        }
        let mut did = 0;
        for item in &last {
            if let Some(said) = fix_item(fx, item, &mut tried) {
                fx.step(format!("final gate: {said}"));
                did += 1;
            }
        }
        if did == 0 {
            break;
        }
    }
    if !note.ok {
        let view = fx.view();
        let run_dir = fx.run_dir().unwrap_or_default();
        for item in &last {
            match known_gap(item, &view, &run_dir) {
                Some(why) => note
                    .known_gaps
                    .push(format!("{} {}: {why}", item.kind, item.id)),
                None => note.missing.push(format!(
                    "{} {}: {}",
                    item.kind,
                    item.id,
                    short(&item.description)
                )),
            }
        }
        // The gap list repeats per column; keep each reason once with a count.
        note.known_gaps = collapse(&note.known_gaps);
        if note.missing.is_empty() {
            note.ok = true;
        } else {
            let first: Vec<String> = note.missing.iter().take(4).cloned().collect();
            fx.row.failures.push(format!(
                "gate check --final: {} item(s) missing: {}",
                note.missing.len(),
                first.join(" | ")
            ));
        }
    }
    row.gate = Some(note);
}

/// `kind id: reason` lines with the same reason become one line `kind id, id2 (n): reason`.
fn collapse(lines: &[String]) -> Vec<String> {
    let mut out: Vec<(String, Vec<String>)> = Vec::new();
    for l in lines {
        let (head, why) = l.split_once(": ").unwrap_or((l.as_str(), ""));
        match out.iter_mut().find(|(w, _)| w == why) {
            Some((_, heads)) => heads.push(head.to_owned()),
            None => out.push((why.to_owned(), vec![head.to_owned()])),
        }
    }
    out.into_iter()
        .map(|(why, heads)| {
            let n = heads.len();
            let shown: Vec<&str> = heads.iter().take(3).map(String::as_str).collect();
            format!(
                "{}{}: {why}",
                shown.join(", "),
                if n > 3 {
                    format!(" and {} more", n - 3)
                } else {
                    String::new()
                }
            )
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

/// Runs the M6 steps for one fixture.
pub(super) fn run(fx: &mut Fx, expected: &Expected, off: bool) {
    if fx.timed_out || fx.run_dir().is_err() {
        return;
    }
    let mut row = PolicyRow::default();
    let view = fx.view();
    let ts = targets(&view, expected);
    // The explanation records the writers; the intervention stubs them; the label can then
    // cite the intervention (some rows accept it as their second record).
    explain_all(fx, &ts, &mut row);
    let mut removed = Vec::new();
    intervene_all(fx, &ts, &mut row, &mut removed);
    label_all(fx, &ts, off, &mut row);
    strong_all(fx, &ts, &removed, &mut row);
    close_out(fx, &mut row);
    // The report shows the labels, explanations and strong states just recorded.
    fx.call_quiet(&["report"]);
    fx.row.policy = row;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- against a stand-in dla

    #[cfg(unix)]
    const FAKE: &str = r#"
. ./fake.env
shift 2
echo "$*" >> "$FAKE_LOG"
case "$1 $2" in
"ledger label")
  case "$*" in
  *--suggest*) echo "{\"ok\":true,\"archetypes\":\"$FAKE_ARCH\"}";;
  *) if [ -n "$FAKE_LABEL_CODE" ]; then
       echo "{\"ok\":false,\"error\":{\"code\":\"$FAKE_LABEL_CODE\",\"message\":\"m\"}}"; exit 2
     fi
     case "$*" in *--evidence*) echo '{"ok":true}'; exit 0;; esac
     if [ -n "$FAKE_LABEL_FIX" ] && [ ! -f narrowed ]; then
       echo "{\"ok\":false,\"error\":{\"code\":\"label-evidence-missing\",\"message\":\"m\",\"fix\":\"$FAKE_LABEL_FIX\"}}"; exit 2
     fi
     echo '{"ok":true}';;
  esac;;
"ledger annotate")
  printf '{"type":"finding_annotated","finding":"%s","writers":["%s"],"removers":[]}\n' "$3" "$5" >> .dla/R1/ledger.jsonl
  echo '{"ok":true}';;
"ledger explain")
  if [ -n "$FAKE_EXPLAIN_CODE" ]; then
    echo "{\"ok\":false,\"error\":{\"code\":\"$FAKE_EXPLAIN_CODE\",\"message\":\"m\"}}"; exit 2
  fi
  echo '{"ok":true}';;
"summarize heap-graph") touch narrowed; echo '{"ok":true}';;
"ledger add") echo '{"ok":true,"id":"F12"}';;
"ledger resolve") echo '{"ok":true}';;
"trend S2") echo '{"ok":true}';;
"gate check")
  n=$(cat gate.count 2>/dev/null || echo 0)
  echo $((n + 1)) > gate.count
  if [ "$n" = 0 ]; then
    echo '{"ok":false,"missing_total":6,"pointer":"gate1.json","missing":[]}'; exit 1
  fi
  if [ -n "$FAKE_GAPS" ]; then
    echo '{"ok":false,"missing_total":2,"pointer":"gate2.json","missing":[]}'; exit 1
  fi
  echo '{"ok":true}';;
"intervene H1")
  echo '{"ok":true,"effect":"growth-removed","evidence":["E9"]}';;
"ledger strong")
  if [ -n "$FAKE_STRONG_CODE" ]; then
    echo "{\"ok\":false,\"error\":{\"code\":\"$FAKE_STRONG_CODE\",\"message\":\"m\"}}"; exit 2
  fi
  echo '{"ok":true}';;
*) echo '{"ok":true}';;
esac
"#;

    #[cfg(unix)]
    const GATE1: &str = r#"{"missing":[
      {"kind":"checklist_item","id":"intake.target","description":"d","fix":"dla ledger resolve intake.target --status answered --evidence E1"},
      {"kind":"series","id":"S2","description":"d","fix":"dla trend S2"},
      {"kind":"open_item","id":"O1","description":"d","fix":"dla ledger resolve O1 --as unresolved --reason time_budget --remedy \"<text>\""},
      {"kind":"trend_column","id":"T1:os.rss","description":"d","fix":"dla ledger add --origin growth-series --evidence E11   # column os.rss"},
      {"kind":"trend_column","id":"T2:os.rss","description":"d","fix":"dla trend S3"},
      {"kind":"explanation","id":"F9","description":"d","fix":"dla ledger explain F9 ..."}]}"#;

    #[cfg(unix)]
    const GATE2: &str = r#"{"missing":[
      {"kind":"trend_column","id":"T2:os.rss","description":"d","fix":"dla trend S3"},
      {"kind":"explanation","id":"F9","description":"d","fix":"dla ledger explain F9 ..."}]}"#;

    #[cfg(unix)]
    const LEDGER: &str = concat!(
        r#"{"type":"evidence","evidence_id":"E5","kind":"workload","authority":"test","result":{"harness":"H1","series":["S1"]}}"#,
        "\n",
        r#"{"type":"evidence","evidence_id":"E6","kind":"growth-series","authority":"test","result":{"series":"S1","trend":"T1"}}"#,
        "\n",
        r#"{"type":"finding_added","finding":{"id":"F1","origin":"census","site":"src/Cache.java:20","owner":"app.Cache.map","holder":"container-field","supports":["E6"]}}"#,
        "\n",
        r#"{"type":"transition","finding":"F1","from":"under_test","to":"confirmed","evidence":["E6"]}"#,
        "\n",
        r#"{"type":"finding_added","finding":{"id":"F9","origin":"census","owner":"app.Other.table","holder":"static/global","supports":["E6"]}}"#,
        "\n",
        r#"{"type":"transition","finding":"F9","from":"under_test","to":"confirmed","evidence":["E6"]}"#,
        "\n",
    );

    #[cfg(unix)]
    fn expected(cause: &str, holder: &str) -> Expected {
        toml::from_str(&format!(
            r#"
schema_version = 1
id = "t"
runtime = "jvm"
category = "archetype-leak"
pair = "t-ctl"
dynamic_confirmation = true
[task]
prompt = "p"
[[leak]]
id = "L1"
site = "src/Cache.java:20"
holder = "{holder}"
region = "heap"
cause = "{cause}"
"#
        ))
        .expect("expected.toml parses")
    }

    /// Runs `run` against the stand-in; the row and the calls it made.
    #[cfg(unix)]
    fn drive_fake(
        expected: &Expected,
        off: bool,
        env: &[(&str, &str)],
    ) -> (super::super::FixtureRow, String) {
        drive_fake_with(expected, off, env, GATE2)
    }

    /// Like [`drive_fake`], with the items the gate names on its second answer.
    #[cfg(unix)]
    fn drive_fake_with(
        expected: &Expected,
        off: bool,
        env: &[(&str, &str)],
        gate2: &str,
    ) -> (super::super::FixtureRow, String) {
        use std::os::unix::fs::PermissionsExt;
        let t = tempfile::tempdir().unwrap();
        let bin = t.path().join("dla");
        fs::write(&bin, format!("#!/bin/sh\n{FAKE}")).unwrap();
        fs::set_permissions(&bin, fs::Permissions::from_mode(0o755)).unwrap();
        let project = t.path().join("project");
        fs::create_dir_all(project.join(".dla/R1")).unwrap();
        fs::write(project.join(".dla/current"), "R1\n").unwrap();
        fs::write(project.join(".dla/R1/ledger.jsonl"), LEDGER).unwrap();
        fs::write(project.join("gate1.json"), GATE1).unwrap();
        fs::write(project.join("gate2.json"), gate2).unwrap();
        let log = t.path().join("calls.log");
        let mut settings = format!(
            "FAKE_LOG='{}'\nFAKE_ARCH='{}'\n",
            log.display(),
            if off { "off" } else { "on" }
        );
        for (k, v) in env {
            settings.push_str(&format!("{k}='{v}'\n"));
        }
        fs::write(project.join("fake.env"), settings).unwrap();
        let mut fx = Fx {
            plan: super::super::Plan {
                entry: Entry::Repository,
                runtimes: 1,
                artifacts: Vec::new(),
            },
            bin: &bin,
            project,
            plugin_root: t.path(),
            data_dir: t.path().join("data"),
            log_dir: t.path().join("logs"),
            seq: 0,
            row: super::super::FixtureRow::default(),
            timeout: std::time::Duration::from_secs(30),
            timed_out: false,
            worker: None,
            path_dir: None,
            unset_vars: Vec::new(),
            pending: Vec::new(),
        };
        fx.row.harnesses = vec!["H1".to_owned()];
        fx.row.summarize_cmds = vec![
            vec!["summarize".into(), "heap-graph".into(), "a.hprof".into()],
            vec![
                "summarize".into(),
                "type-histogram-diff".into(),
                "h.txt".into(),
            ],
        ];
        run(&mut fx, expected, off);
        let calls = fs::read_to_string(&log).unwrap_or_default();
        (std::mem::take(&mut fx.row), calls)
    }

    #[cfg(unix)]
    #[test]
    fn a_shipped_row_is_labelled_with_its_cause() {
        let (row, calls) = drive_fake(&expected("L-COLLECT", "container-field"), false, &[]);
        assert!(
            calls.contains("ledger label F1 --as L-COLLECT\n"),
            "{calls}"
        );
        assert!(calls.contains("ledger label F1 --suggest\n"), "{calls}");
        let l = &row.policy.labels[0];
        assert_eq!((l.finding.as_str(), l.result.as_str()), ("F1", "ok"));
        assert_eq!(l.detail, "L1 L-COLLECT");
        // F9 matches no expected leak: it is not labelled.
        assert_eq!(row.policy.labels[1].result, "not labelled");
        assert!(!calls.contains("label F9 --as"), "{calls}");
        assert!(
            row.failures.iter().all(|f| !f.starts_with("label ")),
            "{:?}",
            row.failures
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_refused_label_of_a_shipped_row_is_a_failure() {
        let (row, _) = drive_fake(
            &expected("L-COLLECT", "container-field"),
            false,
            &[("FAKE_LABEL_CODE", "label-evidence-missing")],
        );
        assert!(
            row.failures
                .iter()
                .any(|f| f == "leak L1 (L-COLLECT) label refused label-evidence-missing"),
            "{:?}",
            row.failures
        );
        assert_eq!(
            row.policy.labels[0].result,
            "refused label-evidence-missing"
        );
    }

    #[cfg(unix)]
    #[test]
    fn with_archetypes_off_the_label_must_be_refused_with_archetypes_off() {
        let e = expected("L-COLLECT", "container-field");
        let (row, _) = drive_fake(&e, true, &[("FAKE_LABEL_CODE", "archetypes-off")]);
        assert!(
            row.failures.iter().all(|f| !f.starts_with("label ")),
            "{:?}",
            row.failures
        );
        assert_eq!(
            row.policy.labels[0].result,
            "refused archetypes-off (expected)"
        );
        // Accepted with archetypes off: a failure.
        let (row, _) = drive_fake(&e, true, &[]);
        assert!(
            row.failures
                .iter()
                .any(|f| f == "leak L1 (L-COLLECT) label was accepted with archetypes off"),
            "{:?}",
            row.failures
        );
        // Another refusal is a failure too.
        let (row, _) = drive_fake(&e, true, &[("FAKE_LABEL_CODE", "label-state")]);
        assert!(
            row.failures
                .iter()
                .any(|f| f.contains("expected archetypes-off")),
            "{:?}",
            row.failures
        );
        // `--suggest` has to say that archetypes are off.
        let (row, _) = drive_fake(&e, false, &[]);
        assert!(row.failures.is_empty() || row.failures.iter().all(|f| !f.contains("suggest")));
    }

    #[cfg(unix)]
    #[test]
    fn a_refusal_for_missing_evidence_is_followed_through_its_printed_fix() {
        let e = expected("L-COLLECT", "container-field");
        // `--evidence E..`: the ledger has the records, the finding does not cite them.
        let (row, calls) = drive_fake(
            &e,
            false,
            &[(
                "FAKE_LABEL_FIX",
                "dla ledger label F1 --as L-COLLECT --evidence E5,E6",
            )],
        );
        assert!(
            calls.contains("ledger label F1 --as L-COLLECT --evidence E5,E6\n"),
            "{calls}"
        );
        assert_eq!(row.policy.labels[0].result, "ok (after --evidence E5,E6)");
        assert!(row.failures.iter().all(|f| !f.starts_with("label ")));
        // `dla summarize heap-graph ... --finding F1`: the command that ran is run again
        // narrowed to the finding (the other operations are left alone), then the label.
        let (row, calls) = drive_fake(
            &e,
            false,
            &[(
                "FAKE_LABEL_FIX",
                "dla harness run <H> --capture heap, dla summarize heap-graph <dumps> --finding F1   (native: the vgdb chain)",
            )],
        );
        let lines: Vec<&str> = calls.lines().collect();
        let narrowed = lines
            .iter()
            .position(|l| *l == "summarize heap-graph a.hprof --finding F1")
            .expect("narrowed summarize");
        assert!(lines[narrowed + 1..].contains(&"ledger label F1 --as L-COLLECT"));
        assert!(!calls.contains("type-histogram-diff"), "{calls}");
        assert_eq!(
            row.policy.labels[0].result,
            "ok (after summarize heap-graph --finding)"
        );
        // A fix that names nothing the policy can do stays a refusal.
        let (row, _) = drive_fake(
            &e,
            false,
            &[("FAKE_LABEL_FIX", "dla summarize non-heap <captures>")],
        );
        assert!(
            row.failures
                .iter()
                .any(|f| f == "leak L1 (L-COLLECT) label refused label-evidence-missing")
        );
        // With archetypes off nothing is followed.
        let (_, calls) = drive_fake(
            &e,
            true,
            &[
                ("FAKE_LABEL_CODE", "archetypes-off"),
                (
                    "FAKE_LABEL_FIX",
                    "dla ledger label F1 --as L-COLLECT --evidence E5",
                ),
            ],
        );
        assert!(!calls.contains("--as L-COLLECT --evidence"), "{calls}");
    }

    #[test]
    fn the_printed_fix_is_read_for_evidence_and_operations() {
        assert_eq!(
            evidence_in_label_fix("dla ledger label F1 --as L-HANDLE --evidence E10,E14"),
            Some("E10,E14".to_owned())
        );
        assert_eq!(
            evidence_in_label_fix("dla ledger label F1 --as X --evidence <E>"),
            None
        );
        assert_eq!(
            evidence_in_label_fix("dla summarize heap-graph x --evidence E1"),
            None
        );
        assert_eq!(
            summarize_ops_in(
                "dla harness run <H> --capture heap, dla summarize heap-graph <dumps> --finding F6   or   dla summarize type-histogram-diff <captures> --finding F6"
            ),
            ["heap-graph", "type-histogram-diff"]
        );
        assert!(summarize_ops_in("dla gap   or   dla intervene <H> --finding F1").is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn unclassified_needs_a_note_and_is_allowed_with_archetypes_off() {
        for off in [false, true] {
            let (row, calls) = drive_fake(&expected("UNCLASSIFIED", "container-field"), off, &[]);
            assert!(
                calls.contains("ledger label F1 --as UNCLASSIFIED --note "),
                "{calls}"
            );
            assert!(
                row.failures.iter().all(|f| !f.starts_with("label ")),
                "{:?}",
                row.failures
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_row_without_a_shipped_prose_row_and_a_wrong_holder_are_not_tried() {
        let (row, calls) = drive_fake(&expected("L-STALE", "container-field"), false, &[]);
        assert!(!calls.contains("--as L-STALE"), "{calls}");
        assert!(row.policy.labels[0].result.contains("no shipped row"));
        let (row, calls) = drive_fake(&expected("L-COLLECT", "static/global"), false, &[]);
        assert!(!calls.contains("--as L-COLLECT"), "{calls}");
        assert!(row.policy.labels[0].result.starts_with("not tried: holder"));
        assert!(row.failures.iter().all(|f| !f.starts_with("label ")));
    }

    #[cfg(unix)]
    #[test]
    fn the_label_comes_after_the_explanation_and_the_intervention() {
        let (_, calls) = drive_fake(&expected("L-COLLECT", "container-field"), false, &[]);
        let pos = |s: &str| calls.lines().position(|l| l.starts_with(s)).expect(s);
        assert!(pos("ledger explain F1") < pos("intervene H1"));
        assert!(pos("intervene H1") < pos("ledger label F1 --as L-COLLECT"));
        assert!(pos("ledger label F1 --as L-COLLECT") < pos("gate check --final"));
    }

    #[cfg(unix)]
    #[test]
    fn a_finding_without_a_writer_gets_its_site_as_the_writer_then_an_explanation() {
        let (row, calls) = drive_fake(&expected("L-COLLECT", "container-field"), false, &[]);
        let lines: Vec<&str> = calls.lines().collect();
        let annotate = lines
            .iter()
            .position(|l| *l == "ledger annotate F1 --writer src/Cache.java:20")
            .expect("annotate");
        let explain = lines
            .iter()
            .position(|l| l.starts_with("ledger explain F1 --tag inferred --adds src/Cache.java:20 --removal never --text "))
            .expect("explain");
        assert!(annotate < explain);
        assert_eq!(row.policy.explains[0].result, "ok");
        // F9 has an owner and no site (an artifact finding): the owner and holder are named
        // and no remover was found.
        assert_eq!(row.policy.explains[1].result, "ok");
        assert!(
            calls.lines().any(|l| l.starts_with(
                "ledger explain F9 --tag inferred --removal not-found --text app.Other.table is the static/global holder"
            )),
            "{calls}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_refused_explanation_is_a_failure() {
        let (row, _) = drive_fake(
            &expected("L-COLLECT", "container-field"),
            false,
            &[("FAKE_EXPLAIN_CODE", "explain-adds")],
        );
        assert!(
            row.failures
                .iter()
                .any(|f| f == "explain of F1 refused explain-adds"),
            "{:?}",
            row.failures
        );
    }

    #[cfg(unix)]
    #[test]
    fn intervene_stubs_the_writers_on_the_confirming_harness_and_strong_follows() {
        let (row, calls) = drive_fake(&expected("L-COLLECT", "container-field"), false, &[]);
        assert!(
            calls.contains("intervene H1 --finding F1 --stub-writer src/Cache.java:20\n"),
            "{calls}"
        );
        let i = &row.policy.interventions[0];
        assert_eq!(i.result, "growth-removed");
        // The evidence the intervention wrote is not in the stand-in's ledger, so strong has
        // no eligible record and is not asked.
        assert_eq!(row.policy.strong[0].result, "no eligible record");
    }

    #[cfg(unix)]
    #[test]
    fn the_close_out_follows_the_commands_the_gate_names_and_reports_the_rest() {
        let (row, calls) = drive_fake(
            &expected("L-COLLECT", "container-field"),
            false,
            &[("FAKE_GAPS", "1")],
        );
        let c = |s: &str| calls.lines().any(|l| l.starts_with(s));
        assert!(
            c("ledger resolve intake.target --status answered --evidence E1"),
            "{calls}"
        );
        assert!(c("trend S2"));
        assert!(c(
            "ledger resolve O1 --as unresolved --reason not_exercised --remedy "
        ));
        assert!(c("ledger add --origin growth-series --evidence E11"));
        assert!(!calls.contains("# column"), "the gate's comment is cut");
        assert!(c(
            "ledger resolve F12 --as unresolved --reason not_exercised"
        ));
        // The gate's own fixes are followed, the artifact explanation included. The stand-in
        // gate never passes, so what it still names is missing, not a known gap.
        assert!(c("trend S3"));
        assert!(c(
            "ledger explain F9 --tag inferred --removal not-found --text "
        ));
        let g = row.policy.gate.as_ref().unwrap();
        assert!(!g.passed && !g.ok, "{g:?}");
        assert_eq!(g.first_missing, 6);
        assert!(g.known_gaps.is_empty(), "{:?}", g.known_gaps);
        assert_eq!(g.missing.len(), 2);
    }

    #[cfg(unix)]
    #[test]
    fn a_gate_that_passes_after_the_close_out_is_not_a_gap() {
        let (row, _) = drive_fake(&expected("L-COLLECT", "container-field"), false, &[]);
        let g = row.policy.gate.as_ref().unwrap();
        assert!(g.passed && g.ok && g.known_gaps.is_empty(), "{g:?}");
        assert_eq!(g.rounds, 1);
    }

    #[cfg(unix)]
    #[test]
    fn an_item_the_policy_cannot_fix_fails_the_fixture_and_is_named() {
        let gate2 = r#"{"missing":[
          {"kind":"accounting","id":"jvm/census","description":"backend census: 2 of 3 enumerated item(s) are dispositioned","fix":"dla ledger resolve <F> --as unresolved --reason time_budget --remedy \"<text>\""},
          {"kind":"explanation","id":"F9","description":"d","fix":"dla ledger explain F9 ..."}]}"#;
        let (row, _) = drive_fake_with(
            &expected("L-COLLECT", "container-field"),
            false,
            &[("FAKE_GAPS", "1")],
            gate2,
        );
        let g = row.policy.gate.as_ref().unwrap();
        assert!(!g.passed && !g.ok, "{g:?}");
        assert!(g.missing[0].starts_with("accounting jvm/census: backend census"));
        assert!(g.known_gaps.is_empty());
        assert!(
            row.failures
                .iter()
                .any(|f| f
                    .starts_with("gate check --final: 2 item(s) missing: accounting jvm/census")),
            "{:?}",
            row.failures
        );
    }

    #[test]
    fn the_text_names_everything_the_explain_rules_look_for() {
        let t = explanation_text(
            "src/a/B.java:41",
            "static/global",
            Some("a.B.cache"),
            "src/a/B.java:41",
            None,
        );
        assert!((40..=600).contains(&t.chars().count()), "{t}");
        assert!(t.contains("src/a/B.java:41") && t.contains("static/global"));
        assert!(t.contains("a.B.cache") && t.contains("never"));
        let c = explanation_text("x.py:3", "container-field", None, "x.py:9", Some("x.py:20"));
        assert!(c.contains("x.py:20") && c.contains("only when"));
        assert!(!c.chars().any(char::is_control));
    }

    #[test]
    fn the_fix_comment_is_cut() {
        assert_eq!(
            fix_command("dla ledger add --origin growth-series --evidence E11   # column os.rss"),
            "dla ledger add --origin growth-series --evidence E11"
        );
        assert_eq!(fix_command("dla trend S2"), "dla trend S2");
        assert_eq!(
            fix_command(
                "dla gap S3 --residual   (rows under the threshold, or over it with a resolved open item, account for the column)"
            ),
            "dla gap S3 --residual"
        );
    }

    #[test]
    fn gaps_are_collapsed_by_reason() {
        let lines = vec![
            "trend_column T2:a: why".to_owned(),
            "trend_column T2:b: why".to_owned(),
            "explanation F4: other".to_owned(),
        ];
        let c = collapse(&lines);
        assert_eq!(c.len(), 2);
        assert!(c[0].starts_with("trend_column T2:a, trend_column T2:b: why"));
    }

    #[test]
    fn merged_lines_of_two_workers_do_not_collide_in_the_view() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("ledger.jsonl");
        let lines = [
            r#"{"type":"evidence","worker":"jvm","evidence_id":"E1","kind":"growth-series","authority":"test","result":{}}"#,
            r#"{"type":"finding_added","worker":"jvm","finding":{"id":"F1","origin":"census","site":"a.kt:1","supports":["E1"]}}"#,
            r#"{"type":"evidence","worker":"rust","evidence_id":"E1","kind":"growth-series","authority":"test","result":{}}"#,
            r#"{"type":"finding_added","worker":"rust","finding":{"id":"F1","origin":"census","site":"b.rs:2","supports":["E1"]}}"#,
            r#"{"type":"transition","worker":"rust","finding":"F1","from":"under_test","to":"confirmed","evidence":["E1"]}"#,
            r#"{"type":"finding_annotated","worker":"jvm","finding":"F1","writers":["a.kt:9"],"removers":[]}"#,
            r#"{"type":"finding_labeled","finding":"rust/F1","label":"L-OWNERSHIP"}"#,
        ];
        fs::write(&p, lines.join("\n")).unwrap();
        let v = View::read(&p);
        assert_eq!(v.findings.len(), 2);
        assert_eq!(v.evidence.len(), 2);
        let jvm = v.finding("jvm/F1").unwrap();
        let rust = v.finding("rust/F1").unwrap();
        assert_eq!(jvm.supports, ["jvm/E1"]);
        assert_eq!(jvm.writers, ["a.kt:9"]);
        assert_eq!(jvm.state, "candidate");
        assert_eq!(rust.state, "confirmed");
        assert_eq!(rust.confirmed.as_ref().unwrap().1, ["rust/E1"]);
        assert_eq!(rust.label.as_deref(), Some("L-OWNERSHIP"));
    }

    #[test]
    fn only_the_shipped_rows_are_labelled() {
        assert!(SHIPPED_ROWS.contains(&"L-COLLECT"));
        assert!(!SHIPPED_ROWS.contains(&"L-STALE"));
        let mut sorted = SHIPPED_ROWS;
        sorted.sort_unstable();
        assert_eq!(sorted, SHIPPED_ROWS, "keep the list sorted");
    }
}
