//! `dla sweep`: run every backend a language's detector table names and turn hits into
//! candidates.

use std::{collections::BTreeMap, fs, path::Path, time::Duration};

use dla_core::{
    EvidenceKind, Refusal,
    coverage::{BackendCoverage, CoverageManifest},
    detectors::{Detector, DetectorFile},
    error::{Error, codes},
    evidence::{CoverageStatus, EvidenceCoverage},
    runtime::Runtime,
    static_hit::StaticHit,
    toolctx::{Candidate, CandidateWriter, EvidenceSpec, StageOutcome, ToolCtx},
};
use serde_json::{Value, json};

use crate::{
    adapters::{self, Job, Outcome, Ran, dedupe},
    dla_rules,
    fsx::ToolEnv,
    registry,
};

/// Seconds a single backend run may take, overriding the registry default.
pub const ENV_TIMEOUT: &str = "DLA_BACKEND_TIMEOUT_SECS";

/// Loads and validates `<data>/detectors/<lang>.toml`. A missing file for a language
/// is an empty table (the DLA rules still run); a missing `detectors/` directory is a
/// refusal.
pub fn load_table(data: &Path, lang: Runtime) -> Result<DetectorFile, Refusal> {
    let dir = data.join("detectors");
    if !dir.is_dir() {
        return Err(Refusal::new(
            codes::INVALID_DATA_FILE,
            format!("{} does not exist", dla_core::paths::to_slash(&dir)),
            "the plugin's data/detectors directory",
            "dla --data <dir> sweep ... where <dir> holds detectors/<lang>.toml",
        ));
    }
    let name = format!("{}.toml", lang.as_str());
    let path = dir.join(&name);
    if !path.is_file() {
        return Ok(DetectorFile::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| Error::io(&path, e).to_refusal())?;
    DetectorFile::parse(&text, &format!("data/detectors/{name}"))
}

fn timeout_for(backend: &str, env: &ToolEnv) -> Duration {
    let secs = env
        .vars
        .get(ENV_TIMEOUT)
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|s| *s > 0)
        .or_else(|| registry::backend(backend).map(|b| b.timeout_secs))
        .unwrap_or(600);
    Duration::from_secs(secs)
}

/// File name of a backend's raw output under `sweep/`: `<lang>-<backend>.json`, so
/// backends shared by runtimes (`dla`, `codeql`) and concurrent workers never share a file.
pub fn raw_file(lang: Runtime, backend: &str) -> String {
    format!("{}-{backend}.json", lang.as_str())
}

/// The first line of `s`, cut at a word boundary with `...` when longer than 100 characters.
fn short(s: &str) -> String {
    let line = s.lines().next().unwrap_or("");
    if line.chars().count() <= 100 {
        return line.to_owned();
    }
    let cut: String = line.chars().take(97).collect();
    let cut = match cut.rfind(char::is_whitespace) {
        Some(i) if i > 0 => &cut[..i],
        _ => cut.as_str(),
    };
    format!("{} ...", cut.trim_end())
}

/// Runs the `dla` rule backend and wraps it as an adapter outcome.
fn run_dla(ctx: &ToolCtx, lang: Runtime) -> Outcome {
    match dla_rules::run(ctx, lang) {
        Ok(hits) => Outcome::Ran(Box::new(Ran {
            version: env!("CARGO_PKG_VERSION").to_owned(),
            command: vec!["dla".to_owned(), "(built-in rules)".to_owned()],
            exit: Some(0),
            hits,
            raw: json!({ "runs": [] }),
            ..Ran::default()
        })),
        Err(e) => Outcome::blocked(format!("dla rules failed: {}", e.message), vec![]),
    }
}

/// What one backend contributed.
struct Done {
    coverage: BackendCoverage,
    summary: Value,
    evidence: Option<String>,
    added: u64,
    skipped: u64,
    /// Hits dropped because their file lies outside the run's scope.
    dropped: u64,
}

fn bounded_result(
    lang: Runtime,
    rules: usize,
    hits: &[StaticHit],
    exit: Option<i32>,
    load_errors: u64,
) -> Value {
    let mut by_rule: BTreeMap<&str, u64> = BTreeMap::new();
    for h in hits {
        *by_rule.entry(h.rule.as_str()).or_default() += 1;
    }
    let mut v = json!({
        "lang": lang.as_str(),
        "rules_enabled": rules,
        "hits": hits.len(),
        "exit_code": exit,
        "load_errors": load_errors,
        "by_rule": by_rule,
    });
    if v.to_string().len() > dla_core::MAX_RESULT_BYTES - 200
        && let Some(o) = v.as_object_mut()
    {
        o.remove("by_rule");
    }
    v
}

/// Where a sweep writes its records.
struct Sink<'a> {
    ledger: &'a mut dla_core::Ledger,
    writer: &'a mut CandidateWriter,
    dir: &'a Path,
    scope: Option<&'a dla_core::state::Scope>,
}

fn record_ran(
    ctx: &ToolCtx,
    lang: Runtime,
    backend: &str,
    rules: &[&Detector],
    ran: Ran,
    sink: &mut Sink,
) -> Result<Done, Refusal> {
    let Ran {
        version,
        command,
        exit,
        hits,
        load_errors,
        mut limitations,
        note,
        raw,
    } = ran;
    let listed: Vec<&str> = rules.iter().map(|d| d.rule.as_str()).collect();
    let before = hits.len();
    let hits = dedupe(
        hits.into_iter()
            .filter(|h| listed.contains(&h.rule.as_str()))
            .collect(),
    );
    let unlisted = before.saturating_sub(hits.len());
    // Build logic (Gradle scripts, buildSrc) is not part of the measured process: a hit there
    // is dropped and named in a limitation, never a candidate.
    let with_logic = hits.len();
    let hits: Vec<StaticHit> = hits
        .into_iter()
        .filter(|h| !dla_core::paths::is_build_logic(&h.file))
        .collect();
    let build_logic_hits = with_logic - hits.len();
    let in_scope = hits.len();
    let hits: Vec<StaticHit> = hits
        .into_iter()
        .filter(|h| sink.scope.is_none_or(|s| s.includes(&h.file)))
        .collect();
    let dropped = u64::try_from(in_scope - hits.len()).unwrap_or(u64::MAX);
    let raw_name = raw_file(lang, backend);
    let doc = json!({
        "backend": backend,
        "lang": lang.as_str(),
        "status": "ran",
        "version": version,
        "rules": listed,
        "command": command,
        "exit_code": exit,
        "load_errors": load_errors,
        "limitations": limitations,
        "note": note,
        "build_logic_hits": build_logic_hits,
        "hits": hits,
        "capture": raw,
    });
    write_raw(&sink.dir.join(&raw_name), &doc)?;

    let artifact = format!("sweep/{raw_name}");
    // A run that analysed nothing (no sources of the language) is coverage, not evidence.
    if note.is_some() {
        let mut cov = BackendCoverage::ran(Some(version), 0);
        cov.reason.clone_from(&note);
        return Ok(Done {
            coverage: cov,
            summary: json!({"status": "ran", "enumerated": 0, "reason": note}),
            evidence: None,
            added: 0,
            skipped: 0,
            dropped,
        });
    }
    if unlisted > 0 {
        limitations.push(format!(
            "{unlisted} hit(s) for rules not listed in the detector table were dropped"
        ));
    }
    let mut result = bounded_result(lang, listed.len(), &hits, exit, load_errors);
    if build_logic_hits > 0 {
        result["build_logic_hits"] = json!(build_logic_hits);
        limitations.push(format!(
            "{build_logic_hits} hit(s) in build logic (Gradle scripts, buildSrc) were dropped: it is not part of the measured process"
        ));
    }
    if dropped > 0 {
        result["out_of_scope_hits"] = json!(dropped);
        limitations.push(format!(
            "{dropped} hit(s) in files outside the run's scope were dropped"
        ));
    }
    let mut spec = EvidenceSpec::new(EvidenceKind::StaticHit, backend, &version, result);
    spec.raw = Some(artifact);
    spec.coverage = EvidenceCoverage {
        status: if limitations.is_empty() && load_errors == 0 {
            CoverageStatus::Complete
        } else {
            CoverageStatus::Partial
        },
        examined: None,
        total: None,
    };
    spec.limitations.clone_from(&limitations);
    let ev = ctx.add_evidence(sink.ledger, spec)?;
    let (mut added, mut skipped) = (0u64, 0u64);
    for h in &hits {
        let arch = rules
            .iter()
            .find(|d| d.rule == h.rule)
            .map(|d| d.suggested_archetype.clone())
            .filter(|a| !a.is_empty());
        let c = Candidate {
            origin: h.origin(),
            evidence: vec![ev.clone()],
            site: h.site(),
            suggested_archetype: arch,
            ..Candidate::default()
        };
        match sink.writer.add(sink.ledger, c)? {
            Some(_) => added += 1,
            None => skipped += 1,
        }
    }
    let enumerated = added + skipped;
    let mut cov = BackendCoverage::ran(Some(version), enumerated);
    cov.duplicates = skipped;
    if !limitations.is_empty() {
        cov.reason = Some(short(&limitations.join("; ")));
    }
    Ok(Done {
        coverage: cov,
        summary: json!({
            "status": "ran",
            "enumerated": enumerated,
            "added": added,
            "limitations": limitations.len(),
        }),
        evidence: Some(ev.to_string()),
        added,
        skipped,
        dropped,
    })
}

fn write_raw(path: &Path, doc: &Value) -> Result<(), Refusal> {
    let text = serde_json::to_string_pretty(doc).map_err(|e| {
        Refusal::new(
            codes::MALFORMED,
            format!("cannot serialise the raw sweep output: {e}"),
            "serialisable output",
            "report this as a dla bug",
        )
    })?;
    fs::write(path, text).map_err(|e| Error::io(path, e).to_refusal())
}

/// Runs every backend named by the detector table of `lang` (plus the built-in `dla`
/// rules) and records evidence, candidates and coverage.
pub fn sweep(ctx: &ToolCtx, lang: Runtime) -> Result<StageOutcome, Refusal> {
    let env = ToolEnv::from_process(ctx.tools_dir());
    sweep_with(ctx, lang, &env)
}

/// [`sweep`] with an explicit tool environment (tests inject fake tools).
pub fn sweep_with(ctx: &ToolCtx, lang: Runtime, env: &ToolEnv) -> Result<StageOutcome, Refusal> {
    ctx.require_time("sweep")?;
    let data = ctx.data_files_dir()?;
    let scope = ctx.scope()?;
    let table = load_table(&data, lang)?;
    let tmp = ctx.tmp_dir()?;
    let sweep_dir = ctx.stage_dir("sweep")?;
    let mut ledger = ctx.ledger()?;
    let mut writer = CandidateWriter::for_ctx(ctx, &ledger)?;
    let mut dropped_hits = 0u64;

    let mut order: Vec<String> = table
        .backends()
        .into_iter()
        .filter(|b| *b != "dla")
        .map(str::to_owned)
        .collect();
    // Sonar reports last of the external backends: its reason names those that ran.
    order.sort_by_key(|b| b == "sonar");
    order.push("dla".to_owned());

    let mut coverage: Vec<(String, BackendCoverage)> = Vec::new();
    let mut summary = serde_json::Map::new();
    let mut evidence = Vec::new();
    let mut artifacts = Vec::new();
    let (mut added, mut skipped) = (0u64, 0u64);

    for backend in &order {
        let rules: Vec<&Detector> = table.for_backend(backend).collect();
        let outcome = if backend == "dla" {
            run_dla(ctx, lang)
        } else {
            let scratch = tmp.join(format!("{}-{backend}", lang.as_str()));
            fs::create_dir_all(&scratch).map_err(|e| Error::io(&scratch, e).to_refusal())?;
            let job = Job {
                ctx,
                lang,
                rules: rules.clone(),
                env,
                scratch,
                timeout: timeout_for(backend, env),
                scope: scope.clone(),
                ran: coverage
                    .iter()
                    // A backend that ran on nothing (not applicable) did not cover anything.
                    .filter(|(_, c)| {
                        c.status == dla_core::coverage::BackendStatus::Ran
                            && !c
                                .reason
                                .as_deref()
                                .is_some_and(|r| r.starts_with("not applicable"))
                    })
                    .map(|(b, _)| b.clone())
                    .collect(),
            };
            adapters::run(backend, &job)
        };
        let done = match outcome {
            Outcome::Missing { reason } => Done {
                coverage: BackendCoverage::missing(&reason, registry::install_hint(backend)),
                summary: json!({"status": "missing", "reason": short(&reason),
                                 "install_hint": short(&registry::install_hint(backend))}),
                evidence: None,
                added: 0,
                skipped: 0,
                dropped: 0,
            },
            Outcome::Blocked { reason, raw } => {
                let doc = json!({
                    "backend": backend, "lang": lang.as_str(), "status": "blocked",
                    "reason": reason, "capture": raw,
                });
                let raw_name = raw_file(lang, backend);
                write_raw(&sweep_dir.join(&raw_name), &doc)?;
                artifacts.push(ctx.pointer(&format!("sweep/{raw_name}")));
                Done {
                    coverage: BackendCoverage::blocked(&reason),
                    summary: json!({"status": "blocked", "reason": short(&reason)}),
                    evidence: None,
                    added: 0,
                    skipped: 0,
                    dropped: 0,
                }
            }
            Outcome::Ran(ran) => {
                let mut sink = Sink {
                    ledger: &mut ledger,
                    writer: &mut writer,
                    dir: &sweep_dir,
                    scope: scope.as_ref(),
                };
                let d = record_ran(ctx, lang, backend, &rules, *ran, &mut sink)?;
                artifacts.push(ctx.pointer(&format!("sweep/{}", raw_file(lang, backend))));
                d
            }
        };
        dropped_hits += done.dropped;
        added += done.added;
        skipped += done.skipped;
        if let Some(e) = done.evidence {
            evidence.push(e);
        }
        summary.insert(backend.clone(), done.summary);
        coverage.push((backend.clone(), done.coverage));
    }

    let inside = |rel: &str| scope.as_ref().is_none_or(|s| s.includes(rel));
    let out_of_scope = if scope.is_some() {
        dla_census::walk::count_out_of_scope(&ctx.project, lang, &inside)
    } else {
        0
    };
    dla_census::record_out_of_scope(ctx, lang, out_of_scope)?;
    let build_logic = dla_census::walk::count_build_logic(&ctx.project, lang);
    dla_census::record_build_logic(ctx, lang, build_logic)?;
    if build_logic > 0 {
        summary.insert("build_logic".into(), json!(build_logic));
    }
    if out_of_scope > 0 {
        summary.insert("out_of_scope".into(), json!(out_of_scope));
    }
    if dropped_hits > 0 {
        summary.insert("out_of_scope_hits".into(), json!(dropped_hits));
    }
    CoverageManifest::update(&ctx.run_dir, |m| {
        for (b, c) in coverage {
            m.set_backend(lang.as_str(), &b, c);
        }
    })
    .map_err(|e| e.to_refusal())?;

    Ok(StageOutcome {
        evidence,
        candidates: added,
        skipped_duplicates: skipped,
        artifacts,
        summary: Value::Object(summary),
    })
}
