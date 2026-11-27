//! `dla ledger list` and `dla ledger close-rest`: the ranked candidates, and the close of
//! what the measurement queue does not hold back.

use clap::Args;
use dla_core::{
    FindingId, FindingState, RunView, UnresolvedReason,
    evidence::{
        Authority, CoverageStatus, EvidenceCoverage, EvidenceKind, NewEvidence, Source, Tool,
    },
    finding::Finding,
    queue::{self, Closed, Index, Standing, Why},
    rank,
    trend::Verdict,
};
use serde_json::{Value, json};

use super::{Loaded, ResolveArgs, ResolveAs, open, resolve_finding};
use crate::{ctx::Ctx, fail::Res, parse::parse_enum};

/// Findings `dla ledger list` prints by default.
const DEFAULT_LIMIT: usize = 40;

/// Flags of `dla ledger list`.
#[derive(Args, Debug)]
pub struct ListArgs {
    /// Only findings in this state (candidate, under_test, confirmed, strong, refuted,
    /// suspected_static_only, unresolved).
    #[arg(long, value_parser = parse_enum::<FindingState>)]
    state: Option<FindingState>,
    /// At most this many findings (default 40).
    #[arg(long, value_name = "N")]
    limit: Option<usize>,
    /// Every finding, no limit.
    #[arg(long, conflicts_with = "limit")]
    all: bool,
}

/// Flags of `dla ledger close-rest`.
#[derive(Args, Debug)]
pub struct CloseRestArgs {
    /// The remedy every closed finding gets (default: one built from the finding's history).
    #[arg(long)]
    remedy: Option<String>,
}

/// The queue status of a finding as `ledger list` prints it.
fn status_of(
    r: &queue::Ranked,
    ix: &Index,
    queues: &[queue::RuntimeQueue],
) -> (&'static str, Option<Standing>) {
    if !r.eligible() {
        return ("not-eligible", None);
    }
    if r.state != FindingState::Candidate {
        return ("closed", None);
    }
    let _ = queues;
    match ix.standing(&r.id) {
        Standing::Pending => ("pending", Some(Standing::Pending)),
        s => ("settled", Some(s)),
    }
}

/// `dla ledger list`: one line of JSON per finding in rank order, the ranked candidates
/// first, then the other findings in ledger order; a first line with the totals.
pub fn list(ctx: &Ctx, a: ListArgs) -> Res<String> {
    let view = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
    let ranked = queue::ranked(&view);
    let ix = Index::of(&view);
    let queues = queue::queues(&view, ctx.worker.as_deref());
    let triggers = dla_core::trigger::Triggers::of(&view);
    let mut order: Vec<&FindingId> = ranked.iter().map(|r| &r.id).collect();
    let in_rank: std::collections::BTreeSet<&FindingId> = order.iter().copied().collect();
    let others: Vec<FindingId> = view
        .ledger
        .findings()
        .filter(|f| !in_rank.contains(&f.id))
        .map(|f| f.id.clone())
        .collect();
    order.extend(others.iter());
    let by_id: std::collections::BTreeMap<&FindingId, &queue::Ranked> =
        ranked.iter().map(|r| (&r.id, r)).collect();

    let mut lines: Vec<(String, Value)> = Vec::new();
    for id in order {
        let Some(f) = view.ledger.finding(id) else {
            continue;
        };
        if a.state.is_some_and(|s| s != f.state) {
            continue;
        }
        let ann = view.ledger.annotation(id);
        let r = by_id.get(id);
        let (mut qs, mut standing) =
            r.map_or(("not-eligible", None), |r| status_of(r, &ix, &queues));
        // A candidate one flat run settled that waits for a different driver is pending.
        let second_q = queues.iter().find(|q| q.second.contains_key(id));
        if second_q.is_some() {
            (qs, standing) = ("pending", Some(Standing::Pending));
        }
        let tier = r.map_or_else(|| rank::tier(f, ann), |r| r.tier);
        let mut e = json!({
            "id": id.to_string(),
            "state": f.state_label(),
            "tier": tier,
            "rank": r.and_then(|r| r.rank),
            "score": rank::score(f, ann),
            "why": rank::why(f, ann, 3)
                .into_iter()
                .map(|(name, points)| format!("{name} {points:+}"))
                .collect::<Vec<_>>(),
            "origin": f.origin.as_str(),
            "site": f.site,
            "holder": f.holder.map(|h| h.to_string()),
            "owner": f.owner,
            "writers": ann.map_or(0, |a| a.writers.len()),
            "removers": ann.map_or(0, |a| a.removers.len()),
            "queue": qs,
        });
        if let Some(r) = super::ledger_release::list_facts(&view, id) {
            e["release"] = r;
        }
        if !f.also_origins.is_empty() {
            e["also_origins"] = json!(
                f.also_origins
                    .iter()
                    .map(|o| o.as_str())
                    .collect::<Vec<_>>()
            );
        }
        let targeted = ix.targeted_in(id);
        if !targeted.is_empty() {
            let mut m = serde_json::Map::new();
            for (h, v) in targeted {
                m.insert(h, json!(v.map(Verdict::as_str)));
            }
            e["harnesses"] = Value::Object(m);
        }
        match (second_q, ix.second_look(id)) {
            (Some(q), _) if q.second[id].detail == queue::THREAD_REASON => {
                e["second_look"] = json!(format!(
                    "pending: {} ({} read it flat); {} gives it a second look whose units come from other threads",
                    queue::THREAD_REASON,
                    q.second[id].harness,
                    q.batch_command
                ));
            }
            (Some(q), _) => {
                e["second_look"] = json!(format!(
                    "pending: one flat run of one driver ({}) settled it and budget is left; {} gives it a different driver",
                    q.second[id].harness, q.batch_command
                ));
            }
            (None, queue::SecondLook::Awaiting(h)) if r.is_some_and(|r| r.eligible()) => {
                e["second_look"] = json!(format!(
                    "after the first pass, if budget is left (one flat run of {h} settled it)"
                ));
            }
            (None, queue::SecondLook::Done(h)) => {
                e["second_look"] = json!(format!("done in {h}"));
            }
            _ => {}
        }
        if dla_core::trigger::is_subject(&view, f) {
            e["trigger"] = json!(triggers.trigger(id).name());
        }
        if ix.best_verdict(id) == Some(Verdict::Unknown) {
            e["verdict_note"] = json!(queue::NO_COVERAGE);
        }
        if let Some(Standing::Settled(why)) = standing {
            e["settled_by"] = json!(match why {
                Why::Moved => "growth",
                Why::Executed => "executed",
                Why::NotReached => "not_reached",
            });
            if let Some(h) = ix.not_reached_in(id) {
                e["not_reached_in"] = json!(format!(
                    "missed by two different drivers, the focused harness {h} among them"
                ));
            }
        } else if let Some(m) = ix.missed_in(id) {
            // Missed once: it waits for a focus harness.
            e["missed_in"] = json!(format!(
                "{} {}: {}",
                m.harness,
                m.verdict.as_str(),
                m.detail
            ));
            let command = queues.iter().find(|q| q.pending.contains(id)).map_or_else(
                || "dla harness gen --driver --next".to_owned(),
                |q| q.missed_command(id),
            );
            e["focus"] = match ix.setup_hold(id) {
                Some(h) => json!(format!(
                    "pending: {}; {command} gives it a harness of its own",
                    queue::setup_hold_sentence(id, &h, None)
                )),
                None => json!(format!("pending: {command} gives it a harness of its own")),
            };
        }
        lines.push((f.state.as_str().to_owned(), e));
    }
    let total = lines.len();
    let limit = if a.all {
        usize::MAX
    } else {
        a.limit.unwrap_or(DEFAULT_LIMIT)
    };
    let eligible = ranked.iter().filter(|r| r.eligible()).count();
    let mut totals = json!({
        "totals": {
            "findings": view.ledger.findings().count(),
            "listed": total.min(limit),
            "matching": total,
            "ranked": ranked.len(),
            "eligible": eligible,
            "queue": queues.iter().map(|q| {
                let mut e = json!({
                    "eligible": q.eligible, "settled": q.settled, "pending": q.pending.len(),
                    "open": q.open(), "closed": q.closed,
                });
                // The command `dla run status` prints for this queue, while it is open.
                if q.open() {
                    e["command"] = json!(q.command);
                }
                (q.runtime.clone(), e)
            }).collect::<serde_json::Map<_, _>>(),
        }
    });
    if total > limit {
        totals["totals"]["more"] = json!(format!("{} more: dla ledger list --all", total - limit));
    }
    let mut out = totals.to_string();
    for (_, e) in lines.into_iter().take(limit) {
        out.push('\n');
        out.push_str(&e.to_string());
    }
    Ok(out)
}

/// The remedy `close-rest` builds from a finding's history.
fn default_remedy(
    f: &Finding,
    r: Option<&queue::Ranked>,
    ix: &Index,
    runtime: Option<&str>,
    key_from_caller: bool,
) -> String {
    let place = f
        .owner
        .as_deref()
        .filter(|o| !o.is_empty())
        .or(f.site.as_deref())
        .unwrap_or("its holder");
    let targeted = ix.targeted_in(&f.id);
    if targeted.is_empty() {
        let lang = runtime.map_or_else(String::new, |rt| format!(" --lang {rt}"));
        return match r {
            Some(r) if r.eligible() => format!(
                "ranked {} (tier {}), not targeted: dla harness gen{lang} --driver --probe {}",
                r.rank.unwrap_or(0),
                r.tier,
                f.id
            ),
            Some(r) => format!(
                "tier {} (not ranked for measurement), not targeted: dla harness gen{lang} --driver --probe {} if it matters",
                r.tier, f.id
            ),
            None => format!(
                "not targeted: dla harness gen{lang} --driver --probe {}",
                f.id
            ),
        };
    }
    let names: Vec<String> = targeted.iter().map(|(h, _)| h.clone()).collect();
    let list = match names.as_slice() {
        [one] => one.clone(),
        [a, b] => format!("{a} and {b}"),
        more => format!(
            "{} and {}",
            more[..more.len() - 1].join(", "),
            more[more.len() - 1]
        ),
    };
    let best = ix.best_verdict(&f.id);
    let outcome = match best {
        Some(Verdict::Constant) => "its holder did not change".to_owned(),
        Some(Verdict::NotLoaded) => {
            "its probe never loaded: the workload did not import its module".to_owned()
        }
        Some(Verdict::NotExecuted) => "its code did not run".to_owned(),
        Some(Verdict::WriterNotExecuted) => {
            "its function ran but the lines that add to the holder never ran".to_owned()
        }
        Some(Verdict::Executed) => "its code ran and showed no growth".to_owned(),
        Some(Verdict::Unknown) => queue::NO_COVERAGE.to_owned(),
        Some(Verdict::Changed) => "its holder changed but did not pass the growth gate".to_owned(),
        Some(Verdict::Grew) => "its holder grew".to_owned(),
        None => "no run of it was read with dla trend".to_owned(),
    };
    // A holder whose key comes from the caller: the unit must pass a key it has not passed
    // before (a fresh object when the key is compared by identity).
    let key = if key_from_caller {
        " (the key comes from the caller: pass a key the holder has not seen; a key compared by identity needs a fresh object per unit)"
    } else {
        ""
    };
    // Rust: a feature with gated sites that no harness enabled comes before any other advice.
    if ix.missed_in(&f.id).is_some()
        && let Some((gaps, command)) = ix.untried_features(&f.id)
    {
        return format!(
            "targeted in {list}, {outcome}: {}. Then write a unit that adds a new key to {place}{key}",
            dla_core::features::lead(&gaps, &command)
        );
    }
    if best == Some(Verdict::WriterNotExecuted) {
        return format!(
            "targeted in {list}, {outcome}: write a unit that puts the object in the state in which those lines run (a state that normal set-up removes needs a unit or a set-up that does not remove it) and adds a new key to {place}{key}"
        );
    }
    format!("targeted in {list}, {outcome}: write a unit that adds a new key to {place}{key}")
}

/// What `close-rest` did with one finding.
struct Closed1 {
    id: String,
    to: &'static str,
    reason: UnresolvedReason,
}

/// `dla ledger close-rest`: dispositions every open candidate that is not pending in an open
/// queue, through the rules of `dla ledger resolve`.
pub fn close_rest(ctx: &Ctx, a: CloseRestArgs) -> Res<Value> {
    let mut ledger = open(ctx)?;
    let view = RunView::load_for(&ctx.run_dir, ctx.worker.as_deref())?;
    let ranked = queue::ranked(&view);
    let by_id: std::collections::BTreeMap<&FindingId, &queue::Ranked> =
        ranked.iter().map(|r| (&r.id, r)).collect();
    let ix = Index::of(&view);
    let queues = queue::queues(&view, ctx.worker.as_deref());
    // Loaded once: the refusals of `resolve` read these for every finding instead of loading
    // the run again.
    let grown = dla_core::grown::grown_unmeasured(&view);
    let loaded = Loaded {
        view: &view,
        ranked: &ranked,
        queues: &queues,
        grown: &grown,
    };
    let mut only: Option<Option<String>> = None;
    let mine = |f: &Finding| {
        ctx.worker
            .as_deref()
            .is_none_or(|w| f.id.prefix() == Some(w))
    };
    let candidates: Vec<Finding> = view
        .ledger
        .findings()
        .filter(|f| f.state == FindingState::Candidate)
        .filter(|f| view.ledger.merged_into(&f.id).is_none())
        .filter(|f| mine(f))
        .cloned()
        .collect();
    let mut closed: Vec<Closed1> = Vec::new();
    let mut left: Vec<(String, String)> = Vec::new();
    // Closed with a miss on record and no focus harness that settled it: "missed once".
    let mut missed_once = 0usize;
    // Candidates left for their second look, by the room each runtime's queue has for them.
    let mut second_left: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();
    for f in &candidates {
        let runtime = dla_core::gate::finding_runtime_with(&view, f, &mut only);
        let q = runtime
            .as_deref()
            .and_then(|rt| queues.iter().find(|q| q.runtime == rt));
        // A candidate whose column grew is a lead to decide, not one to close: a passed
        // column points at it and no alloc-test decided it. The commands are printed in
        // order and each one is accepted as printed.
        if let Some(g) = grown.iter().find(|g| g.finding == f.id) {
            let why = if g.dump.is_some() {
                format!(
                    "{} and nothing decided it: {}",
                    g.grew_text(),
                    g.commands().join("   then   ")
                )
            } else if let Some(l) = &g.longer {
                format!(
                    "its column grew, and {}: {}",
                    l.text(),
                    g.commands().join("   then   ")
                )
            } else if let Some(sh) = &g.shared {
                format!(
                    "its column grew, and the alloc-test {} on {} ({}, not a driver made for {}) favored {}: growth there may be kept by another holder: {}",
                    sh.evidence,
                    sh.harness,
                    sh.why,
                    f.id,
                    sh.favors,
                    g.commands().join("   then   ")
                )
            } else {
                format!("its column grew: {}", g.commands().join("   then   "))
            };
            left.push((f.id.to_string(), why));
            continue;
        }
        // An alloc-test that favors leak decided it: it is confirmed (or refuted by a newer
        // alloc-test), never closed. The same commands `ledger resolve` prints in its refusal.
        if let Some(route) = dla_core::grown::route_for(&view, &f.id) {
            left.push((
                f.id.to_string(),
                format!("{}. Fix: {}", route.refusal_message(&f.id), route.fix()),
            ));
            continue;
        }
        // Pending in an open queue: it waits for a measurement.
        if q.is_some_and(|q| q.open() && q.pending.contains(&f.id)) {
            let cmd = q.map(|q| q.command_for(&f.id)).unwrap_or_default();
            // Rust: a feature with gated sites that no harness enabled comes first.
            let gated = ix
                .missed_in(&f.id)
                .and_then(|_| ix.untried_features(&f.id))
                .map(|(gaps, command)| {
                    format!("{}. Then: ", dla_core::features::lead(&gaps, &command))
                })
                .unwrap_or_default();
            let held = ix
                .setup_hold(&f.id)
                .map(|h| format!("{}. Then: ", queue::setup_hold_sentence(&f.id, &h, None)))
                .unwrap_or_default();
            left.push((
                f.id.to_string(),
                format!(
                    "pending in the {} queue: {gated}{held}{cmd}",
                    runtime.unwrap_or_default()
                ),
            ));
            continue;
        }
        // One flat run of one driver settled it and a second look with a different driver is
        // still to come: while the first pass has candidates pending, the queue is open, and
        // room is left under `SECOND_LOOK_MAX`, it is not closed (the first pass would end,
        // find no candidate, and never offer the second look).
        if let (Some(q), Some(rt)) = (q, runtime.as_deref())
            && q.open()
            && !q.pending.is_empty()
            && matches!(ix.second_look(&f.id), queue::SecondLook::Awaiting(_))
            && second_left.get(rt).copied().unwrap_or(0)
                < queue::SECOND_LOOK_MAX.saturating_sub(ix.second_looks_used(rt)) * queue::FOCUS_MAX
        {
            *second_left.entry(rt.to_owned()).or_default() += 1;
            left.push((
                f.id.to_string(),
                format!(
                    "a flat run settled it and it waits for its second look with a different driver once the first pass is over ({} pending): {}",
                    q.pending.len(),
                    q.command
                ),
            ));
            continue;
        }
        if ix.best_verdict(&f.id) == Some(Verdict::Grew) {
            left.push((
                f.id.to_string(),
                format!(
                    "its column grew: dla ledger test {} --evidence <workload evidence of the harness>, then dla alloc-test <H> --finding {}",
                    f.id, f.id
                ),
            ));
            continue;
        }
        // A finding that was measured and left undecided is `inconclusive`, at the budget
        // floor too; one nothing reached is `not_exercised`, or `time_budget` at the floor.
        let measured = ledger.supported_reason(&f.id) == UnresolvedReason::Inconclusive;
        let static_evidence = f
            .supports
            .iter()
            .filter_map(|e| view.ledger.evidence_by_id(e))
            .any(|e| matches!(e.kind, EvidenceKind::StaticHit | EvidenceKind::CensusHolder));
        let (to, disposition) = if static_evidence && !measured {
            ("suspected_static_only", ResolveAs::SuspectedStaticOnly)
        } else {
            ("unresolved", ResolveAs::Unresolved)
        };
        let reason = if measured {
            UnresolvedReason::Inconclusive
        } else if q.is_some_and(|q| q.closed == Some(Closed::Floor)) {
            // The queue closed because the budget reached the floor: that is what time_budget says.
            UnresolvedReason::TimeBudget
        } else {
            UnresolvedReason::NotExercised
        };
        let remedy = a.remedy.clone().unwrap_or_else(|| {
            let key_from_caller = view
                .ledger
                .annotation(&f.id)
                .is_some_and(|a| a.facts.key_from_caller());
            default_remedy(
                f,
                by_id.get(&f.id).copied(),
                &ix,
                runtime.as_deref(),
                key_from_caller,
            )
        });
        let args = ResolveArgs {
            target: f.id.to_string(),
            disposition: Some(disposition),
            reason: Some(reason),
            remedy: Some(remedy),
            by: None,
            status: None,
            evidence: Vec::new(),
            note: None,
            runtime: None,
        };
        match resolve_finding(ctx, &mut ledger, f.id.clone(), args, Some(&loaded)) {
            Ok(_) => {
                if ix.missed_in(&f.id).is_some() && ix.standing(&f.id) == Standing::Pending {
                    missed_once += 1;
                }
                closed.push(Closed1 {
                    id: f.id.to_string(),
                    to,
                    reason,
                });
            }
            // A finding an alloc-test decided for leak is left with the commands that confirm it.
            Err(e) if e.refusal.code == dla_core::error::codes::LEAK_FAVORED => left.push((
                f.id.to_string(),
                format!("{} Fix: {}", e.refusal.message, e.refusal.fix),
            )),
            Err(e) => left.push((f.id.to_string(), e.refusal.message.clone())),
        }
    }
    let count = |to: &str| closed.iter().filter(|c| c.to == to).count();
    let by_reason = |r: UnresolvedReason| closed.iter().filter(|c| c.reason == r).count();
    // One run-config record lists what was closed; the full list is a file.
    let mut record = Value::Null;
    if !closed.is_empty() {
        let n = ledger.lines().len() + 1;
        let rel = format!("queue/close-rest-{n}.json");
        let path = ctx.run_dir.join(&rel);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| dla_core::Error::io(dir, e))?;
        }
        let full = json!({
            "closed": closed.iter().map(|c| json!({"id": c.id, "as": c.to, "reason": c.reason.as_str()})).collect::<Vec<_>>(),
            "left_open": left.iter().map(|(i, w)| json!({"id": i, "why": w})).collect::<Vec<_>>(),
        });
        std::fs::write(
            &path,
            serde_json::to_string_pretty(&full).unwrap_or_default() + "\n",
        )
        .map_err(|e| dla_core::Error::io(&path, e))?;
        let project = ctx
            .run_dir
            .parent()
            .and_then(std::path::Path::parent)
            .map(std::path::Path::to_path_buf);
        let commit = project
            .as_deref()
            .and_then(crate::git::head_commit)
            .unwrap_or_else(|| "unknown".to_owned());
        let ids: Vec<&str> = closed.iter().take(40).map(|c| c.id.as_str()).collect();
        let line = ledger.add_evidence(NewEvidence {
            source: Source::Repo { commit, path: None },
            tool: Tool {
                name: "dla".to_owned(),
                version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            authority: Authority::Static,
            kind: EvidenceKind::RunConfig,
            result: json!({
                "close_rest": {
                    "closed": closed.len(),
                    "suspected_static_only": count("suspected_static_only"),
                    "unresolved": count("unresolved"),
                    "not_exercised": by_reason(UnresolvedReason::NotExercised),
                    "inconclusive": by_reason(UnresolvedReason::Inconclusive),
                    "time_budget": by_reason(UnresolvedReason::TimeBudget),
                    "left_open": left.len(),
                    "missed_once_no_focus_harness": missed_once,
                    "ids": ids.join(","),
                },
            }),
            raw: Some(rel),
            coverage: EvidenceCoverage {
                status: CoverageStatus::Complete,
                examined: None,
                total: None,
            },
            limitations: Vec::new(),
            unknowns: Vec::new(),
        })?;
        record = json!(super::pointer(ctx, line.seq));
    }
    let mut out = json!({
        "ok": true,
        "closed": closed.len(),
        "as": {
            "suspected_static_only": count("suspected_static_only"),
            "unresolved": count("unresolved"),
        },
        "reason": {
            "not_exercised": by_reason(UnresolvedReason::NotExercised),
            "inconclusive": by_reason(UnresolvedReason::Inconclusive),
            "time_budget": by_reason(UnresolvedReason::TimeBudget),
        },
        "missed_once_no_focus_harness": missed_once,
        "left_open": left.len(),
        "left": left.iter().take(8).map(|(i, w)| json!({"id": i, "why": w})).collect::<Vec<_>>(),
        "record": record,
    });
    if let Some(q) = queues.iter().find(|q| q.open()) {
        out["batch_command"] = json!(q.command);
    }
    // The test loop of any open queue, not only of the first one.
    if let Some(line) = queues
        .iter()
        .filter(|q| q.open())
        .find_map(|q| q.test_loop_line())
    {
        out["next"] = json!(line);
    }
    if out.get("next").is_none()
        && let Some(q) = queues.iter().find(|q| q.open() && !q.pending.is_empty())
    {
        out["next"] = json!(format!(
            "{} candidate(s) stay pending in the open {} queue: {}",
            q.pending.len(),
            q.runtime,
            q.command
        ));
    }
    if left.len() > 8 {
        out["left_more"] = json!(left.len() - 8);
    }
    Ok(out)
}
