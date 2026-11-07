//! `dla-bench table`: renders `metrics.json` as Markdown.

use std::{fmt::Write as _, fs, path::Path};

use crate::{
    Res,
    model::{Group, MetricsFile, Ratio},
};

fn num(v: Option<f64>) -> String {
    v.map_or_else(|| "n/a".to_owned(), |x| format!("{x:.4}"))
}

fn ratio_row(name: &str, r: &Ratio) -> String {
    let spread = match (r.mean, r.stdev) {
        (Some(m), Some(s)) => format!("{m:.4} ± {s:.4}"),
        _ => "n/a".to_owned(),
    };
    format!(
        "| {name} | {} | {}/{} | {spread} |\n",
        num(r.value),
        r.num,
        r.den
    )
}

/// Renders one group.
pub fn render_group(g: &Group) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "## {} (archetypes {})\n", g.ablation, g.archetypes);
    s.push_str("| Metric | Value | Num/Den | Mean ± stdev over runs |\n|---|---|---|---|\n");
    let m = &g.metrics;
    let rows: [(&str, &Ratio); 12] = [
        ("leak_recall", &m.leak_recall),
        ("archetype_free_recall", &m.archetype_free_recall),
        ("flagged_recall", &m.flagged_recall),
        ("second_leak_recall", &m.second_leak_recall),
        ("grounded_attribution", &m.grounded_attribution),
        ("label_accuracy", &m.label_accuracy),
        ("forced_fit_rate", &m.forced_fit_rate),
        ("overclaim_rate", &m.overclaim_rate),
        ("false_confirmation_rate", &m.false_confirmation_rate),
        ("non_leak_precision", &m.non_leak_precision),
        ("trap_miss_rate", &m.trap_miss_rate),
        ("trap_false_alarm_rate", &m.trap_false_alarm_rate),
    ];
    for (n, r) in rows {
        s.push_str(&ratio_row(n, r));
    }
    let u = &g.unexplained_growth_share;
    let _ = writeln!(
        s,
        "| unexplained_growth_share | {} | over {} run(s) | |",
        num(u.value),
        u.runs
    );
    let us = &g.usage;
    let _ = writeln!(s, "\n### Usage\n");
    s.push_str("| Item | Sum | Mean per run |\n|---|---|---|\n");
    let _ = writeln!(s, "| runs | {} | |", us.runs);
    let _ = writeln!(s, "| errored runs | {} | |", us.errored);
    let _ = writeln!(
        s,
        "| findings source (ledger / findings.json / none) | {} / {} / {} | |",
        us.sources.ledger, us.sources.findings, us.sources.none
    );
    for (n, v) in [
        ("input tokens", &us.input_tokens),
        ("output tokens", &us.output_tokens),
        ("cache read tokens", &us.cache_read_input_tokens),
        ("cache creation tokens", &us.cache_creation_input_tokens),
    ] {
        let _ = writeln!(s, "| {n} | {} | {} |", v.sum, num(v.mean));
    }
    let _ = writeln!(
        s,
        "| wall seconds | {:.1} | {} (max {:.1}) |",
        us.wall_seconds.sum,
        us.wall_seconds
            .mean
            .map_or_else(|| "n/a".to_owned(), |m| format!("{m:.1}")),
        us.wall_seconds.max
    );
    let _ = writeln!(
        s,
        "| cost USD | {:.4} | {} |",
        us.cost_usd.sum,
        num(us.cost_usd.mean)
    );
    let _ = writeln!(s, "\n### Per fixture\n");
    s.push_str(
        "| Fixture | Category | Runs | Claim runs | Rule-out runs | Leaks (found/flagged/grounded) |\n|---|---|---|---|---|---|\n",
    );
    for (id, f) in &g.fixtures {
        let leaks = if f.leaks.is_empty() {
            "-".to_owned()
        } else {
            f.leaks
                .iter()
                .map(|(l, c)| format!("{l}: {}/{}/{}", c.found, c.flagged, c.grounded))
                .collect::<Vec<_>>()
                .join("; ")
        };
        let _ = writeln!(
            s,
            "| {id} | {} | {} | {} | {} | {leaks} |",
            f.category, f.runs, f.claim_runs, f.rule_out_runs
        );
    }
    s
}

/// Renders every group.
pub fn render(m: &MetricsFile) -> String {
    let mut s = String::from("# Benchmark results\n\n");
    if m.groups.is_empty() {
        s.push_str("No runs.\n");
    }
    for g in &m.groups {
        s.push_str(&render_group(g));
        s.push('\n');
    }
    s
}

/// Reads `<results>/metrics.json` and writes `<results>/table.md`.
pub fn write_table(results: &Path) -> Res<String> {
    let p = results.join("metrics.json");
    let text = fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    let m: MetricsFile =
        serde_json::from_str(&text).map_err(|e| format!("{}: {e}", p.display()))?;
    let out = results.join("table.md");
    fs::write(&out, render(&m)).map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(out.display().to_string())
}
