//! `dla report`: render, write `report.md`, store its hash in `state.json`.

use dla_core::{
    RunState, RunView,
    fsutil::write_atomic,
    state::ReportRef,
    state::{REPORT_FILE, now_utc},
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
}

/// Runs `dla report`.
pub fn run(ctx: &Ctx) -> Res<Rendered> {
    let view = RunView::load(&ctx.run_dir)?;
    let markdown = dla_report::render_markdown(&view);
    let sha256 = hex::encode(Sha256::digest(markdown.as_bytes()));
    write_atomic(&ctx.run_dir.join(REPORT_FILE), markdown.as_bytes())?;
    let path = ctx.pointer(REPORT_FILE);
    let mut state: RunState = view.state;
    state.report = Some(ReportRef {
        path: path.clone(),
        sha256: sha256.clone(),
        rendered_at: now_utc(),
    });
    state.save(&ctx.run_dir)?;
    Ok(Rendered {
        markdown,
        sha256,
        path,
    })
}
