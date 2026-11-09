//! The coverage row of memray, the Python harness's `--capture alloc` second report.
//!
//! The agent starts memray only when it imports in the target's interpreter. When it does
//! not, the agent lists an `alloc` capture with an error that names the missing package;
//! when it does, `py-<n>.memray.json` is among the capture files. Either way the run says
//! so in the coverage manifest, so the gap is a row and not only a line in the limitations.

use dla_core::{
    coverage::{BackendCoverage, CoverageManifest},
    harness::{CaptureFile, CaptureKind},
    toolctx::ToolCtx,
};

/// Worker key of the Python runtime in the coverage manifest.
const WORKER: &str = "python";
/// Backend name of the row.
const BACKEND: &str = "memray";
/// Start of the agent's error text when memray is not installed.
const MISSING: &str = "memray is not installed";

/// The memray row of one point: `ran` when a memray report was written, `missing` when the
/// agent said memray is absent from the interpreter (`errors` is `(kind, error)` as the
/// agent listed them), else nothing.
fn row(errors: &[(String, String)], files: &[CaptureFile]) -> Option<BackendCoverage> {
    let reports = files
        .iter()
        .filter(|f| f.kind == CaptureKind::Alloc && f.path.ends_with(".memray.json"))
        .count() as u64;
    if reports > 0 {
        return Some(BackendCoverage::ran(None, reports));
    }
    errors
        .iter()
        .any(|(kind, e)| kind == "alloc" && e.starts_with(MISSING))
        .then(|| {
            BackendCoverage::missing(
                "memray is not installed in the target's python interpreter, so --capture alloc has the tracemalloc report only",
                "pip install memray in the interpreter the workload runs under, then dla harness run <H> --capture alloc",
            )
        })
}

/// Notes the memray row for one point in the coverage manifest. Never fails the run.
pub fn note(ctx: &ToolCtx, errors: &[(String, String)], files: &[CaptureFile]) {
    if let Some(cov) = row(errors, files) {
        let _ = CoverageManifest::update(&ctx.run_dir, |m| m.note_backend(WORKER, BACKEND, cov));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dla_core::coverage::BackendStatus;

    fn file(path: &str) -> CaptureFile {
        CaptureFile {
            kind: CaptureKind::Alloc,
            path: path.to_owned(),
            label: "p1-native".to_owned(),
            seq: 1,
            x: None,
            role: None,
            bytes: 1,
            ms: None,
            captured_at: String::new(),
        }
    }

    fn absent() -> Vec<(String, String)> {
        vec![(
            "alloc".to_owned(),
            "memray is not installed in this interpreter (/usr/bin/python3); pip install memray"
                .to_owned(),
        )]
    }

    #[test]
    fn a_report_is_a_ran_row() {
        let files = [file("p/py-1.alloc.json"), file("p/py-1.memray.json")];
        let r = row(&[], &files).unwrap();
        assert_eq!((r.status, r.enumerated), (BackendStatus::Ran, 1));
    }

    #[test]
    fn an_absent_package_is_a_missing_row_with_the_fix() {
        let r = row(&absent(), &[file("p/py-1.alloc.json")]).unwrap();
        assert_eq!(r.status, BackendStatus::Missing);
        assert!(r.install_hint.unwrap().contains("pip install memray"));
    }

    #[test]
    fn a_run_without_memray_news_has_no_row() {
        assert!(row(&[], &[file("p/py-1.alloc.json")]).is_none());
        let other = vec![("alloc".to_owned(), "tracemalloc is not tracing".to_owned())];
        assert!(row(&other, &[]).is_none());
    }
}
