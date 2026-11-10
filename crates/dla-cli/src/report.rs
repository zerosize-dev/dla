//! `dla report`: render, write `report.md`, store its hash in `state.json`.

use dla_core::{
    RunState, RunView, fsutil::write_atomic, integrity, state::REPORT_FILE, state::ReportRef,
    state::now_utc,
};
use sha2::{Digest, Sha256};

use crate::{ctx::Ctx, fail::Res};

/// What `dla report` produced.
pub struct Rendered {
    /// The markdown (also written to `report.md`).
    pub markdown: String,
    /// SHA-256 of the markdown bytes, lowercase hex.
    pub sha256: String,
    /// Project-relative pointer to `report.md`.
    pub path: String,
    /// Protected files that did not match their seal when the report was rendered. The
    /// report is INCOMPLETE and names them; the command exits non-zero.
    pub tamper: Vec<integrity::Tamper>,
}

/// Runs `dla report`. The report is written whatever the integrity check says, so the
/// state is never hidden; the caller turns a tamper into a non-zero exit.
pub fn run(ctx: &Ctx) -> Res<Rendered> {
    let view = RunView::load(&ctx.run_dir)?;
    let markdown = dla_report::render_markdown(&view);
    let sha256 = hex::encode(Sha256::digest(markdown.as_bytes()));
    let path = ctx.pointer(REPORT_FILE);
    let report = ReportRef {
        path: path.clone(),
        sha256: sha256.clone(),
        rendered_at: now_utc(),
    };
    // The report, its seal and the reference in `state.json` change together under the
    // run lock.
    RunState::update(&ctx.run_dir, |s| {
        write_atomic(&ctx.run_dir.join(REPORT_FILE), markdown.as_bytes())?;
        integrity::seal_file(&ctx.run_dir, REPORT_FILE)?;
        s.report = Some(report);
        Ok(())
    })?;
    Ok(Rendered {
        markdown,
        sha256,
        path,
        tamper: view.tamper,
    })
}
