//! The artifact and symptom entry paths of the driver.
//!
//! Artifact fixtures hold heap dumps and no source. The policy runs `dla summarize heap-graph`
//! over all dumps and reads only its output:
//!
//! 1. Gate pass: for every candidate holder, an N-WARM sibling is added, the candidate goes
//!    under test with the growth evidence, the sibling is refuted with the dump-plateau
//!    discriminator when it favors leak, and the candidate is confirmed citing the growth
//!    evidence. The finding carries its owner and never a site. A discriminator that favors
//!    non-leak refutes the candidate instead; an inconclusive one leaves it `unresolved`.
//! 2. Gate fail with a discriminator that favors non-leak (a fill that stopped): no claim. The
//!    ruled-out holder is recorded as a finding (owner from the holder key) and refuted with
//!    the discriminator.
//! 3. Fewer than three dumps: no claim. The holder that retains most in the newest dump is
//!    recorded as a finding and resolved `suspected_static_only` (`not_exercised`) with a
//!    remedy that asks for more dumps.
//!
//! Symptom fixtures name no target: `intake.target` is resolved `unresolved` with a remedy and
//! the run ends with no stage and no finding.

use serde_json::Value;

use super::{Fx, Out, resolve, strings};

/// Common remedy for runs that end without a series of three dumps.
const MORE_DUMPS: &str = "take at least 3 heap dumps of the same process, minutes apart under load, and run dla summarize heap-graph with all of them";

/// The owner (`Class.field`) a holder key names; `None` when the key is not `kind:name`.
fn owner_of(key: &str) -> Option<String> {
    let (_, name) = key.split_once(':')?;
    (!name.is_empty()).then(|| name.to_owned())
}

/// Adds a leak-side finding for an owner with the given evidence; returns its id.
fn add_owner_finding(fx: &mut Fx, owner: &str, evidence: &str) -> Option<String> {
    let out = fx.call(&[
        "ledger",
        "add",
        "--origin",
        "hypothesis",
        "--owner",
        owner,
        "--holder",
        "static/global",
        "--region",
        "heap",
        "--evidence",
        evidence,
    ]);
    out.ok.then(|| out.json["id"].as_str().map(str::to_owned))?
}

/// Confirms one candidate of a passing series; returns what happened, for the log.
fn confirm_candidate(fx: &mut Fx, f: &str, growth: &str, discriminators: &[Value]) -> String {
    let favors = |want: &str| {
        discriminators
            .iter()
            .filter(|d| d["finding"].as_str() == Some(f) && d["favors"].as_str() == Some(want))
            .find_map(|d| d["evidence"].as_str().map(str::to_owned))
    };
    let sibling = fx.call(&[
        "ledger",
        "add",
        "--sibling-of",
        f,
        "--origin",
        "hypothesis",
        "--hypothesis",
        "N-WARM: the growth is a warm-up fill that stops",
        "--refute-if",
        "the holder keeps growing across every dump interval",
        "--evidence",
        growth,
    ]);
    let Some(sib) = sibling
        .ok
        .then(|| sibling.json["id"].as_str().map(str::to_owned))
        .flatten()
    else {
        return "could not add the N-WARM sibling".to_owned();
    };
    if !fx.call(&["ledger", "test", f, "--evidence", growth]).ok {
        return "ledger test refused".to_owned();
    }
    if let Some(d) = favors("leak") {
        fx.call(&["ledger", "refute", &sib, "--evidence", &d]);
        let c = fx.call(&["ledger", "confirm", f, "--evidence", growth]);
        return if c.ok {
            format!("N-WARM refuted by {d}, confirmed from the dump series")
        } else {
            format!("confirm refused ({})", c.code())
        };
    }
    if let Some(d) = favors("non-leak") {
        let r = fx.call(&["ledger", "refute", f, "--evidence", &d]);
        return if r.ok {
            format!("{d} favors non-leak, refuted")
        } else {
            format!("refute refused ({})", r.code())
        };
    }
    resolve(
        fx,
        f,
        "unresolved",
        "time_budget",
        "take another dump after more traffic and rerun dla summarize heap-graph with all dumps",
    );
    "the dump-plateau discriminator is inconclusive: left unresolved".to_owned()
}

/// What the summary said about the dumps.
fn summarize(fx: &mut Fx, dumps: &[String]) -> Option<Out> {
    let mut args: Vec<&str> = vec!["summarize", "heap-graph"];
    args.extend(dumps.iter().map(String::as_str));
    let out = fx.call(&args);
    out.ok.then_some(out)
}

/// The artifact path.
pub(super) fn drive(fx: &mut Fx) {
    let dumps = fx.plan.artifacts.clone();
    if dumps.is_empty() {
        fx.row
            .failures
            .push("the project holds no .hprof or .hprof.gz file".to_owned());
        return;
    }
    let mut init: Vec<&str> = vec![
        "run",
        "init",
        "--budget",
        super::RUN_BUDGET,
        "--entry",
        "artifact",
    ];
    for d in &dumps {
        init.extend(["--artifact", d.as_str()]);
    }
    if !fx.call(&init).ok {
        return;
    }
    let Some(sum) = summarize(fx, &dumps) else {
        return;
    };
    let gate = sum.json["gate"].as_str().unwrap_or("").to_owned();
    let growth = sum.json["growth_evidence"].as_str().map(str::to_owned);
    let discriminators: Vec<Value> = sum.json["discriminators"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let reasons = strings(&sum.json["reasons"]).join("; ");
    fx.step(format!(
        "summarize heap-graph over {} dump(s): gate {gate}{}",
        dumps.len(),
        if reasons.is_empty() {
            String::new()
        } else {
            format!(" ({reasons})")
        }
    ));
    if gate == "pass" {
        let Some(growth) = growth else {
            fx.row
                .errors
                .push("the gate passed without growth evidence".to_owned());
            return finish(fx);
        };
        let candidates: Vec<String> = sum.json["candidates"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|c| c["finding"].as_str().map(str::to_owned))
            .collect();
        fx.row.passed_columns = sum.json["passed"].as_array().map_or(0, Vec::len);
        fx.row.passed_with_candidates = candidates.len();
        for f in candidates {
            let what = confirm_candidate(fx, &f, &growth, &discriminators);
            fx.step(format!("{f}: {what}"));
        }
        return finish(fx);
    }
    // Gate fail. Fewer than three dumps: suspected only. Otherwise the series itself speaks:
    // a holder whose growth stopped is ruled out.
    let few = dumps.len() < 3 || reasons.contains("dumps < 3");
    if few {
        let newest = sum.json["dumps"]
            .as_array()
            .into_iter()
            .flatten()
            .max_by_key(|d| d["timestamp_ms"].as_u64().unwrap_or(0));
        let top = newest.and_then(|d| {
            Some((
                d["top_holder"]["key"].as_str()?.to_owned(),
                d["evidence"].as_str()?.to_owned(),
            ))
        });
        if let Some((key, ev)) = top
            && let Some(owner) = owner_of(&key)
            && let Some(f) = add_owner_finding(fx, &owner, &ev)
        {
            let ok = fx
                .call(&[
                    "ledger",
                    "resolve",
                    &f,
                    "--as",
                    "suspected_static_only",
                    "--reason",
                    "not_exercised",
                    "--remedy",
                    MORE_DUMPS,
                    "--evidence",
                    &ev,
                ])
                .ok;
            fx.step(format!(
                "{f}: {owner} is the largest holder of the newest dump; {} dump(s) cannot tell a fill from a leak: {}",
                dumps.len(),
                if ok {
                    "suspected_static_only (not_exercised)"
                } else {
                    "resolve refused"
                }
            ));
        } else {
            fx.step("no holder to record from the dumps".to_owned());
        }
        return finish(fx);
    }
    for d in discriminators
        .iter()
        .filter(|d| d["favors"].as_str() == Some("non-leak"))
    {
        let (Some(key), Some(dev)) = (d["holder"].as_str(), d["evidence"].as_str()) else {
            continue;
        };
        let Some(owner) = owner_of(key) else { continue };
        let ev = growth.clone().unwrap_or_else(|| dev.to_owned());
        let Some(f) = add_owner_finding(fx, &owner, &ev) else {
            continue;
        };
        let r = fx.call(&["ledger", "refute", &f, "--evidence", dev]);
        fx.step(format!(
            "{f}: {owner} stopped growing ({dev} favors non-leak): {}",
            if r.ok { "ruled out" } else { "refute refused" }
        ));
    }
    finish(fx)
}

/// Ends an artifact or symptom run: the report.
fn finish(fx: &mut Fx) {
    fx.call(&["report"]);
}

/// The symptom path: no target, no stage, no claim.
pub(super) fn drive_symptom(fx: &mut Fx) {
    if !fx
        .call(&[
            "run",
            "init",
            "--budget",
            super::RUN_BUDGET,
            "--entry",
            "symptom",
        ])
        .ok
    {
        return;
    }
    let r = fx.call(&[
        "ledger",
        "resolve",
        "intake.target",
        "--status",
        "unresolved",
        "--reason",
        "env_unavailable",
        "--remedy",
        "name the repository, service or process to diagnose, or provide heap dumps of it",
    ]);
    fx.step(format!(
        "the symptom names no target: intake.target {}",
        if r.ok {
            "unresolved with a remedy"
        } else {
            "not recorded"
        }
    ));
    finish(fx);
}
