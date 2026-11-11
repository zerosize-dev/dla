//! `dla ledger label`, `strong` and `explain` (schema 6).
//!
//! Each command loads the ledger the writes go to (the main one, or a worker's slice),
//! hands the request to `dla-core`, which builds the record with the proof it was accepted
//! on, and prints a short answer. The rules live in `dla_core::{labels, machine, explain}`.

use std::path::PathBuf;

use clap::Args;
use dla_core::{
    EvidenceId, FindingId, FindingState, Ledger, Record, RemovalClaim,
    archetypes::{self, Table},
    attribution,
    error::codes as core_codes,
    finding::ExplanationTag,
    labels::{self, Archetypes, SuggestRow},
    paths,
};
use serde_json::{Value, json};

use crate::{
    ctx::Ctx,
    fail::{Failure, Res},
    ledger_cmd::{open, pointer},
    parse::parse_enum,
};

/// Loads the archetype table: `None` (archetypes off) when `DLA_ARCHETYPES=off`, when no
/// plugin data directory can be found or when `data/archetypes.toml` is absent. A table
/// that exists and is invalid is a refusal that names the file.
pub fn archetype_table(data: Option<&PathBuf>) -> Res<Option<Table>> {
    let on = archetypes::archetypes_enabled(&|k| std::env::var(k).ok());
    if !on {
        return Ok(None);
    }
    let Ok(dir) = paths::data_files_dir(data.map(PathBuf::as_path)) else {
        return Ok(None);
    };
    Ok(archetypes::load(&dir)?)
}

/// Flags of `dla ledger label`.
#[derive(Args, Debug)]
pub struct LabelArgs {
    /// The finding (F3, F3.n1).
    #[arg(value_parser = parse_enum::<FindingId>)]
    pub finding: FindingId,
    /// The archetype ID (L-* on a leak-side finding, N-* on a non-leak sibling) or
    /// UNCLASSIFIED. DLA never picks a label.
    #[arg(long = "as", value_name = "ID", conflicts_with = "suggest")]
    pub label: Option<String>,
    /// Evidence IDs that meet the archetype's `requires`, comma separated (default: the
    /// evidence the finding cites or is supported by).
    #[arg(long, value_delimiter = ',', value_parser = parse_enum::<EvidenceId>)]
    pub evidence: Vec<EvidenceId>,
    /// Why nothing fits (required with `--as UNCLASSIFIED`).
    #[arg(long)]
    pub note: Option<String>,
    /// Read only: per archetype, the met alternative or the missing evidence kinds.
    #[arg(long)]
    pub suggest: bool,
}

/// Flags of `dla ledger strong`.
#[derive(Args, Debug)]
pub struct StrongArgs {
    /// The finding (F3).
    #[arg(value_parser = parse_enum::<FindingId>)]
    pub finding: FindingId,
    /// The second independent record: a retention record or an intervention, comma
    /// separated. A new `dla trend` record may be listed too when the one that confirmed
    /// the finding has no signal.
    #[arg(long, value_delimiter = ',', required = true, value_parser = parse_enum::<EvidenceId>)]
    pub evidence: Vec<EvidenceId>,
}

/// Flags of `dla ledger explain`.
#[derive(Args, Debug)]
pub struct ExplainArgs {
    /// The finding (F3).
    #[arg(value_parser = parse_enum::<FindingId>)]
    pub finding: FindingId,
    /// inferred | observed (observed needs --evidence).
    #[arg(long, value_parser = parse_enum::<ExplanationTag>)]
    pub tag: ExplanationTag,
    /// `file:line` of the sites that add to the holder, comma separated (default: the
    /// recorded writers).
    #[arg(long, value_delimiter = ',', value_name = "FILE:LINE")]
    pub adds: Vec<String>,
    /// never | not-found | conditional | wrong-key | unreachable
    #[arg(long, value_parser = parse_enum::<RemovalClaim>)]
    pub removal: RemovalClaim,
    /// `file:line` of the remover (required with conditional, wrong-key, unreachable).
    #[arg(long, value_name = "FILE:LINE")]
    pub remover: Option<String>,
    /// A retention, intervention or tied growth record that shows the pairing.
    #[arg(long, value_parser = parse_enum::<EvidenceId>)]
    pub evidence: Option<EvidenceId>,
    /// The explanation (40 to 600 characters, one line). It names the site, the holder, an
    /// adding site and how removal fails. DLA checks that these are stated and consistent
    /// with the recorded writers and removers; it cannot check that they are true.
    #[arg(long)]
    pub text: String,
}

/// `dla ledger label`.
pub fn label(ctx: &Ctx, data: Option<&PathBuf>, a: LabelArgs) -> Res<Value> {
    let mut ledger = open(ctx)?;
    let table = archetype_table(data)?;
    if a.suggest {
        return suggest(&ledger, &a, table.as_ref());
    }
    let Some(as_) = a.label.as_deref() else {
        return Err(Failure::refuse(
            core_codes::EMPTY_FIELD,
            "label needs --as <ID> or --suggest",
            "--as <archetype ID | UNCLASSIFIED> or --suggest",
            format!("dla ledger label {} --suggest", a.finding),
        ));
    };
    // The cheap checks first: archetypes off, the label's own state and polarity. Only then
    // the runtime's rows, so the refusal that comes first is the one that can be acted on.
    ledger.label_precheck(&a.finding, as_, Archetypes::from_table(table.as_ref()))?;
    if let Some(r) = runtime_refusal(ctx, data, &ledger, &a.finding, as_, a.note.as_deref()) {
        return Err(r);
    }
    let line = ledger.label(
        &a.finding,
        as_,
        &a.evidence,
        a.note.as_deref(),
        Archetypes::from_table(table.as_ref()),
    )?;
    let Record::FindingLabeled(rec) = &line.record else {
        unreachable!("label appends a finding_labeled record");
    };
    let mut out = json!({
        "ok": true,
        "finding": rec.finding.to_string(),
        "label": rec.label,
        "previous": rec.previous,
        "kinds": rec.satisfied.kinds,
        "evidence": rec.satisfied.evidence.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
    });
    if rec.region_mismatch {
        let f = ledger.finding(&rec.finding);
        let have = f.and_then(|f| f.region).map_or("none", |r| r.as_str());
        let fits: Vec<&str> = table
            .as_ref()
            .and_then(|t| t.get(&rec.label))
            .map(|e| e.regions.iter().map(|r| r.as_str()).collect())
            .unwrap_or_default();
        out["warning"] = json!(format!(
            "the finding's region {have} is not among the regions {} fits ({}); the label stands and the mismatch is recorded",
            rec.label,
            fits.join(", ")
        ));
    }
    Ok(out)
}

/// The runtime of the language a finding's site is written in (`native/src/lib.rs` is rust,
/// a C file of the jvm worker is native), when the reference has rows for it.
fn site_runtime(ledger: &Ledger, id: &FindingId) -> Option<(String, String)> {
    let known = |r: &str| archetypes::runtime_rows(r).is_some();
    let file = ledger
        .finding(&ledger.own_finding_id(id))?
        .site
        .as_deref()?
        .rsplit_once(':')?
        .0
        .to_owned();
    let r = archetypes::runtime_of_file(&file).filter(|r| known(r))?;
    Some((r.to_owned(), file))
}

/// The runtime of the worker a finding belongs to: its worker prefix, the worker the command
/// runs as, or the only runtime in the coverage manifest.
fn worker_runtime(ctx: &Ctx, id: &FindingId) -> Option<String> {
    let known = |r: &str| archetypes::runtime_rows(r).is_some();
    if let Some(p) = id.prefix().filter(|p| known(p)) {
        return Some(p.to_owned());
    }
    if let Some(w) = ctx.worker.as_deref().filter(|w| known(w)) {
        return Some(w.to_owned());
    }
    let cov = dla_core::coverage::CoverageManifest::load(&ctx.run_dir)
        .ok()
        .flatten()?;
    let mut it = cov.workers.keys().filter(|k| known(k));
    match (it.next(), it.next()) {
        (Some(only), None) => Some(only.clone()),
        _ => None,
    }
}

/// The refusal for an `L-*` label that has no row in the finding's runtime reference, unless
/// a note says why none of the runtime's rows fits. Nothing is written. No check when the
/// runtime cannot be determined. The runtime is the language of the finding's site first (a
/// C finding of the jvm worker is native), then the worker's.
fn runtime_refusal(
    ctx: &Ctx,
    data: Option<&PathBuf>,
    ledger: &Ledger,
    id: &FindingId,
    label: &str,
    note: Option<&str>,
) -> Option<Failure> {
    let note = note.map(str::trim).is_some_and(|n| !n.is_empty());
    let known_for = |rt: &str| known_leak_archetypes(data, rt);
    // A worker names its own findings bare (F1 is jvm/F1).
    let id = &ledger.own_finding_id(id);
    let worker = worker_runtime(ctx, id);
    match site_runtime(ledger, id) {
        Some((runtime, file)) => {
            let boundary = worker
                .as_deref()
                .filter(|w| *w != runtime)
                .map(|w| (w, file.as_str()));
            rows_refusal(&runtime, id, label, note, boundary, &known_for(&runtime))
        }
        None => {
            let w = worker?;
            rows_refusal(&w, id, label, note, None, &known_for(&w))
        }
    }
}

/// `boundary` is `(worker runtime, site file)` when the finding's site is in another
/// runtime's code than the worker's: a library the program calls (JNI, FFI).
fn rows_refusal(
    runtime: &str,
    id: &FindingId,
    label: &str,
    note: bool,
    boundary: Option<(&str, &str)>,
    known: &[String],
) -> Option<Failure> {
    if note || !label.starts_with("L-") || !dla_core::detectors::ARCHETYPE_IDS.contains(&label) {
        return None;
    }
    let rows = archetypes::runtime_rows(runtime)?;
    if rows.contains(&label) {
        return None;
    }
    // A shipped known-leaks row that names the archetype for this runtime is a row of the
    // runtime too (the Go ticker is L-REGISTER; Go has no L-REGISTER decoy row).
    if known.iter().any(|k| k == label) {
        return None;
    }
    let route = if known.is_empty() {
        format!("no shipped known-leaks row names an archetype for {runtime}")
    } else {
        format!(
            "archetypes a shipped known-leaks row names for {runtime}: {}",
            known.join(", ")
        )
    };
    if let Some((worker, file)) = boundary {
        // The label may be right: the leak is in code of another language that the worker's
        // program calls, and that language's reference lists other rows.
        return Some(Failure::refuse(
            dla_core::m6::codes::LABEL_RUNTIME_ROW,
            format!(
                "{id} is a {worker} worker finding whose site {file} is {runtime} code (a library the {worker} program calls through JNI or FFI), and {label} has no row in the {runtime} archetype reference (its rows: {}; {route}); {id} is not labelled",
                rows.join(", ")
            ),
            format!(
                "one of {} or UNCLASSIFIED, or --note naming the library boundary that keeps {label}",
                rows.join(", ")
            ),
            format!(
                "to keep {label} for code across the library boundary: dla ledger label {id} --as {label} --note \"library boundary: the {worker} program calls {runtime} code at {file}; <how the {runtime} code holds the resource and why none of the {runtime} rows fits>\"   or choose one of those rows or UNCLASSIFIED: dla ledger label {id} --suggest"
            ),
        ));
    }
    Some(Failure::refuse(
        dla_core::m6::codes::LABEL_RUNTIME_ROW,
        format!(
            "{label} has no row in the {runtime} archetype reference (its rows: {}; {route}); {id} is not labelled",
            rows.join(", ")
        ),
        format!(
            "one of {} or UNCLASSIFIED, or --note saying why none of the {runtime} rows fits",
            rows.join(", ")
        ),
        format!(
            "choose one of those rows or UNCLASSIFIED: dla ledger label {id} --suggest   or, to keep {label}: dla ledger label {id} --as {label} --note \"<why none of the {runtime} rows fits>\""
        ),
    ))
}

/// The archetypes the shipped `known-leaks.toml` names for `runtime` (sorted, without
/// repeats). Empty when the file is absent or invalid, or archetypes are off.
fn known_leak_archetypes(data: Option<&PathBuf>, runtime: &str) -> Vec<String> {
    let Ok(dir) = paths::data_files_dir(data.map(PathBuf::as_path)) else {
        return Vec::new();
    };
    let Ok(Some(file)) = dla_probe::known_leaks::load(&dir) else {
        return Vec::new();
    };
    let mut out: Vec<String> = file
        .leak
        .iter()
        .filter(|k| k.runtime.as_str() == runtime)
        .map(|k| k.archetype.clone())
        .collect();
    out.sort();
    out.dedup();
    out
}

/// The row that closes every `--suggest` answer.
fn unclassified_row(finding: &FindingId) -> String {
    format!(
        "{} none-fits: compare each summary above with the mechanism you stated; if none matches, choose UNCLASSIFIED: dla ledger label {finding} --as UNCLASSIFIED --note \"<holder and mechanism in words>\"",
        archetypes::UNCLASSIFIED_ROW
    )
}

fn row_line(r: &SuggestRow) -> String {
    let state = match r.status {
        "met" => format!("met [{}]", r.evidence.join(",")),
        "missing" => {
            // Only the closest alternative; the producers of the others are in `produce`.
            format!("missing {}", r.missing.join(","))
        }
        _ => format!("blocked, {}", r.blocked_by.join("; ")),
    };
    format!("{} {state}: {}", r.id, r.summary)
}

/// Adds `key` to the object when the answer stays within the output bound.
fn add_if_fits(out: &mut Value, key: &str, alternatives: Vec<Value>) {
    for v in alternatives {
        out[key] = v;
        if serde_json::to_string(out).is_ok_and(|s| s.len() < dla_core::MAX_OUTPUT_BYTES) {
            return;
        }
    }
    if let Some(o) = out.as_object_mut() {
        o.remove(key);
    }
}

fn suggest(ledger: &Ledger, a: &LabelArgs, table: Option<&Table>) -> Res<Value> {
    let Some(table) = table else {
        return Ok(json!({
            "ok": true,
            "finding": a.finding.to_string(),
            "archetypes": "off",
            "note": "archetypes are off (DLA_ARCHETYPES=off or no data/archetypes.toml): only `dla ledger label --as UNCLASSIFIED --note` applies",
            "rows": [unclassified_row(&a.finding)],
        }));
    };
    let found = ledger.suggest_labels(&a.finding, table)?;
    let mut rows: Vec<String> = found.iter().map(row_line).collect();
    rows.push(unclassified_row(&a.finding));
    let mut out = json!({
        "ok": true,
        "finding": a.finding.to_string(),
        "archetypes": "on",
        "rows": rows,
    });
    // The rows and their summaries stay whole; the commands around them shrink to fit.
    let mut full = serde_json::Map::new();
    let mut short = serde_json::Map::new();
    for k in found
        .iter()
        .filter(|r| r.status == "missing")
        .flat_map(|r| r.missing.iter().chain(r.or_missing.iter().flatten()))
    {
        if let Ok(kind) = k.parse::<dla_core::EvidenceKind>() {
            let cmd = labels::producer_command(kind, &a.finding);
            let bare = cmd.replace(&format!(" --finding {}", a.finding), "");
            full.insert(k.clone(), json!(cmd));
            short.insert(k.clone(), json!(bare));
        }
    }
    // The short form first, so `next` and the notes fit; the full form replaces it at the
    // end when everything still fits.
    let produce_full = (!full.is_empty()).then_some(Value::Object(full));
    if !short.is_empty() {
        add_if_fits(&mut out, "produce", vec![Value::Object(short)]);
    }
    add_if_fits(
        &mut out,
        "next",
        vec![
            json!(format!(
                "dla ledger label {} --as <ID> [--evidence E1,E2]   (met is a candidate, not an answer: DLA never picks)",
                a.finding
            )),
            json!(format!(
                "dla ledger label {} --as <ID> --evidence E..",
                a.finding
            )),
        ],
    );
    if found
        .iter()
        .any(|r| r.blocked_by.iter().any(|b| b.starts_with("holder ")))
    {
        let f = &a.finding;
        add_if_fits(
            &mut out,
            "holder_fix",
            vec![
                json!(format!(
                    "if the holder is wrong: dla ledger annotate {f} --holder <one the row lists> --evidence <E>"
                )),
                json!(format!(
                    "wrong holder: dla ledger annotate {f} --holder <H> --evidence <E>"
                )),
            ],
        );
    }
    let cited_only = found
        .iter()
        .filter(|r| r.status == "missing")
        .flat_map(|r| r.missing.iter().chain(r.or_missing.iter().flatten()))
        .filter_map(|k| k.parse::<dla_core::EvidenceKind>().ok())
        .any(labels::cited_only);
    if cited_only {
        let f = &a.finding;
        add_if_fits(
            &mut out,
            "cited_only",
            vec![
                json!(format!(
                    "a record of a kind that names no finding (non-heap, census-holder, static-hit, histogram-diff, should-be-dead, residual) counts only when {f} cites it: cite it when {f} moves (dla ledger test|confirm {f} --evidence E), so before confirmation; a confirmed finding uses the other alternatives"
                )),
                json!(format!(
                    "kinds that name no finding count only when {f} cites them (dla ledger test|confirm {f} --evidence E, before confirmation)"
                )),
            ],
        );
    }
    if let Some(full) = produce_full
        && out.get("produce").is_some()
    {
        let short = out["produce"].take();
        out["produce"] = full;
        if serde_json::to_string(&out).map_or(true, |s| s.len() >= dla_core::MAX_OUTPUT_BYTES) {
            out["produce"] = short;
        }
    }
    Ok(out)
}

/// Flags of `dla ledger duplicate`.
#[derive(Args, Debug)]
pub struct DuplicateArgs {
    /// The finding that repeats another one (F4).
    #[arg(value_parser = parse_enum::<FindingId>)]
    pub finding: FindingId,
    /// The finding of the same leak that stays (F1): the labelled one.
    #[arg(long = "of", value_name = "FINDING", value_parser = parse_enum::<FindingId>)]
    pub of: FindingId,
}

/// `dla ledger duplicate F --of G`.
pub fn duplicate(ctx: &Ctx, a: DuplicateArgs) -> Res<Value> {
    let mut ledger = open(ctx)?;
    let dup = ledger.own_finding_id(&a.finding);
    let canonical = ledger.own_finding_id(&a.of);
    let (line, by) = ledger.duplicate_finding(&dup, &canonical)?;
    Ok(json!({
        "ok": true,
        "duplicate": dup.to_string(),
        "canonical": canonical.to_string(),
        "by": by.describe(),
        "note": format!("{dup} is merged into {canonical}: its supporting evidence now counts for {canonical}, and the gates and the report count the leak once"),
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
    }))
}

/// `dla ledger strong`.
pub fn strong(ctx: &Ctx, a: StrongArgs) -> Res<Value> {
    let mut ledger = open(ctx)?;
    let finding = ledger.own_finding_id(&a.finding);
    let evidence: Vec<EvidenceId> = a
        .evidence
        .iter()
        .map(|e| ledger.own_evidence_id(e))
        .collect();
    let from = ledger.finding(&finding).map(|f| f.state);
    let line = ledger.transition(
        &finding,
        FindingState::Strong,
        None,
        None,
        evidence.clone(),
        "dla ledger strong",
    )?;
    Ok(json!({
        "ok": true,
        "finding": finding.to_string(),
        "from": from.map(|s| s.to_string()),
        "to": FindingState::Strong.as_str(),
        "evidence": evidence.iter().map(ToString::to_string).collect::<Vec<_>>(),
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
    }))
}

/// The writers and removers the census found for the finding's owner.
fn census_sites(ctx: &Ctx, ledger: &Ledger, finding: &FindingId) -> (Vec<String>, Vec<String>) {
    let Some(owner) = ledger.finding(finding).and_then(|f| f.owner.clone()) else {
        return (Vec::new(), Vec::new());
    };
    let Some(project) = ctx.run_dir.parent().and_then(|p| p.parent()) else {
        return (Vec::new(), Vec::new());
    };
    attribution::resolve(&ctx.run_dir, project, &owner)
        .map_or((Vec::new(), Vec::new()), |r| (r.writers, r.removers))
}

/// `dla ledger explain`.
pub fn explain(ctx: &Ctx, a: ExplainArgs) -> Res<Value> {
    let mut ledger = open(ctx)?;
    let finding = ledger.own_finding_id(&a.finding);
    let (cw, cr) = census_sites(ctx, &ledger, &finding);
    let line = ledger.explain(
        &finding,
        a.tag,
        &a.adds,
        a.removal,
        a.remover.as_deref(),
        a.evidence.as_ref(),
        &a.text,
        &cw,
        &cr,
    )?;
    let Record::FindingExplained(rec) = &line.record else {
        unreachable!("explain appends a finding_explained record");
    };
    Ok(json!({
        "ok": true,
        "finding": rec.finding.to_string(),
        "tag": rec.tag.as_str(),
        "adds": rec.adds,
        "removal": rec.removal.as_str(),
        "remover": rec.remover,
        "checked": "the text states the site, the holder, an adding site and the removal, consistently with the recorded writers and removers; DLA does not check that it is true",
        "seq": line.seq,
        "record": pointer(ctx, line.seq),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> FindingId {
        "F1".parse().unwrap()
    }

    #[test]
    fn a_label_without_a_row_in_the_runtime_reference_is_refused_and_names_the_rows() {
        let f = rows_refusal("native", &id(), "L-OWNERSHIP", false, None, &[]).unwrap();
        let r = &f.refusal;
        assert_eq!(r.code, "label-runtime-row");
        assert!(
            r.message.contains("native archetype reference") && r.message.contains("L-HANDLE"),
            "{}",
            r.message
        );
        assert!(
            r.fix.contains("--note") && r.fix.contains("UNCLASSIFIED"),
            "{}",
            r.fix
        );
        let f = rows_refusal("jvm", &id(), "L-WORKER", false, None, &[]).unwrap();
        assert!(
            f.refusal.message.contains("L-COLLECT, L-TLS, L-UNLOAD"),
            "{:?}",
            f.refusal
        );
    }

    #[test]
    fn code_in_another_runtimes_library_is_refused_with_the_library_boundary_note() {
        // jni-order-codec: a jvm worker finding at native/src/lib.rs:43 (rust code).
        let f = rows_refusal(
            "rust",
            &"jvm/F19".parse().unwrap(),
            "L-HANDLE",
            false,
            Some(("jvm", "native/src/lib.rs")),
            &[],
        )
        .unwrap();
        let r = &f.refusal;
        assert_eq!(r.code, "label-runtime-row");
        assert!(
            r.message
                .contains("library the jvm program calls through JNI or FFI")
                && r.message.contains("native/src/lib.rs")
                && r.message.contains("L-CYCLE, L-OWNERSHIP"),
            "{}",
            r.message
        );
        assert!(
            r.missing.contains("--note naming the library boundary"),
            "{}",
            r.missing
        );
        assert!(
            r.fix.contains("--as L-HANDLE --note \"library boundary:") && r.fix.contains("jvm/F19"),
            "{}",
            r.fix
        );
        // The note keeps the label.
        assert!(
            rows_refusal(
                "rust",
                &id(),
                "L-HANDLE",
                true,
                Some(("jvm", "native/src/lib.rs")),
                &[]
            )
            .is_none()
        );
        // The same runtime, or no worker, is not a boundary: the plain text.
        let plain = rows_refusal("rust", &id(), "L-HANDLE", false, None, &[]).unwrap();
        assert!(
            !plain.refusal.message.contains("JNI"),
            "{}",
            plain.refusal.message
        );
    }

    #[test]
    fn a_note_a_row_or_a_non_leak_label_is_not_refused() {
        assert!(rows_refusal("native", &id(), "L-OWNERSHIP", true, None, &[]).is_none());
        assert!(rows_refusal("jvm", &id(), "L-UNLOAD", false, None, &[]).is_none());
        assert!(rows_refusal("native", &id(), "L-HANDLE", false, None, &[]).is_none());
        assert!(rows_refusal("jvm", &id(), "N-WARM", false, None, &[]).is_none());
        assert!(rows_refusal("jvm", &id(), "UNCLASSIFIED", false, None, &[]).is_none());
        assert!(rows_refusal("cobol", &id(), "L-COLLECT", false, None, &[]).is_none());
    }

    #[test]
    fn a_source_file_names_its_runtime() {
        assert_eq!(archetypes::runtime_of_file("src/Main.java"), Some("jvm"));
        assert_eq!(archetypes::runtime_of_file("src/ring.c"), Some("native"));
        assert_eq!(archetypes::runtime_of_file("a.go"), Some("go"));
        assert_eq!(archetypes::runtime_of_file("README"), None);
    }
}
