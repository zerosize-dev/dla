//! Resolving the project directory and the run directory.

use std::path::{Path, PathBuf};

use dla_core::state::{self, DLA_DIR, LEDGER_FILE, STATE_FILE};

use crate::{
    fail::{Failure, Res, codes},
    git,
};

/// The resolved project and run.
#[derive(Debug, Clone)]
pub struct Ctx {
    /// Run ID.
    pub run_id: String,
    /// `<project>/.dla/<run-id>`.
    pub run_dir: PathBuf,
}

fn env_nonempty(name: &str) -> Option<String> {
    std::env::var(name).ok().filter(|v| !v.trim().is_empty())
}

/// `--project`, else the git toplevel found by walking up from the cwd, else the cwd.
pub fn project_root(arg: Option<&Path>) -> Res<PathBuf> {
    let io = |e: std::io::Error| Failure::from(dla_core::Error::io(".", e));
    let root = match arg {
        Some(p) => std::path::absolute(p).map_err(io)?,
        None => {
            let cwd = std::env::current_dir().map_err(io)?;
            git::find_root(&cwd).unwrap_or(cwd)
        }
    };
    if !root.is_dir() {
        return Err(Failure::refuse(
            dla_core::error::codes::INVALID_VALUE,
            format!(
                "project directory {} does not exist",
                dla_core::paths::to_slash(&root)
            ),
            "an existing project directory",
            "dla --project <dir> run status",
        ));
    }
    Ok(root)
}

fn valid_run_id(id: &str) -> bool {
    !id.is_empty()
        && !id.starts_with('.')
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

impl Ctx {
    /// Resolves the run: `--run`, else `DLA_RUN`, else `.dla/current`.
    pub fn resolve(project: PathBuf, run_arg: Option<&str>) -> Res<Ctx> {
        let run_id = match run_arg
            .map(str::to_owned)
            .or_else(|| env_nonempty("DLA_RUN"))
        {
            Some(id) => id,
            None => state::read_current(&project)?.ok_or_else(|| {
                Failure::refuse(
                    codes::NO_RUN,
                    "no run selected and .dla/current does not exist",
                    "an active run",
                    "dla run init",
                )
            })?,
        };
        if !valid_run_id(&run_id) {
            return Err(Failure::refuse(
                dla_core::error::codes::INVALID_VALUE,
                format!("invalid run id {run_id:?}"),
                "a run id such as 20260101-120000-ab12",
                "dla run init",
            ));
        }
        let run_dir = state::run_dir(&project, &run_id);
        if !run_dir.join(STATE_FILE).is_file() {
            return Err(Failure::refuse(
                codes::UNKNOWN_RUN,
                format!("run {run_id} has no {STATE_FILE} under {DLA_DIR}/"),
                format!("{DLA_DIR}/{run_id}/{STATE_FILE}"),
                "dla run init",
            ));
        }
        Ok(Ctx { run_id, run_dir })
    }

    /// Project-relative pointer to a file in the run directory, with `/` separators.
    pub fn pointer(&self, file: &str) -> String {
        format!("{DLA_DIR}/{}/{file}", self.run_id)
    }

    /// Pointer to a ledger line.
    pub fn ledger_line(&self, seq: u64) -> String {
        format!("{}#L{seq}", self.pointer(LEDGER_FILE))
    }
}
