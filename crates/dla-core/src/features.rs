//! Rust: crate features that gate code of a target, and what a harness built without them
//! cannot run.
//!
//! `dla harness gen` reads what the features of the target's crate gate and writes it next to
//! the manifest as `harness/H<n>/features.json`: per target, the features that have gated
//! sites and that the harness build leaves off, with the counts, and the command that
//! generates the harness again with them on. The queue, `dla trend` and `dla ledger
//! close-rest` read it to say the same thing first when a target's code did not run.

use std::{
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{FindingId, harness::HarnessManifest};

/// `harness/H<n>/features.json`.
pub const FILE: &str = "features.json";

/// One feature of the target's crate that has gated sites and that the harness build leaves off.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureGap {
    /// The feature name.
    pub feature: String,
    /// The target's file (project-relative).
    pub file: String,
    /// Gated sites in the target's file.
    pub in_file: usize,
    /// Gated sites in the crate.
    pub in_crate: usize,
    /// Gated sites in the crate's entry files (`lib.rs`, `main.rs`).
    pub in_entry: usize,
    /// The crate gates the module of the target's file behind this feature (a `mod`
    /// declaration): the target block says so on its own line.
    #[serde(default)]
    pub by_module: bool,
    /// The `default` list of the crate turns the feature on (it is off here only because the
    /// build is made with `--no-default-features`).
    #[serde(default)]
    pub default_on: bool,
}

/// The gaps of one harness.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarnessFeatures {
    /// The command that generates this harness again with the features on.
    pub command: String,
    /// Per target (`F3`), its gaps.
    pub gaps: BTreeMap<String, Vec<FeatureGap>>,
}

impl HarnessFeatures {
    /// `<run_dir>/harness/<id>/features.json`.
    pub fn path_in(run_dir: &Path, id: &str) -> PathBuf {
        HarnessManifest::dir_in(run_dir, id).join(FILE)
    }

    /// The file of harness `id`; `None` when it has none or it does not read.
    pub fn load(run_dir: &Path, id: &str) -> Option<Self> {
        let text = fs::read_to_string(Self::path_in(run_dir, id)).ok()?;
        serde_json::from_str(&text).ok()
    }

    /// Writes the file into the harness directory `dir`.
    pub fn save(&self, dir: &Path) -> io::Result<()> {
        let text = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        fs::write(dir.join(FILE), text + "\n")
    }

    /// The gaps of target `id`.
    pub fn gaps_of(&self, id: &FindingId) -> &[FeatureGap] {
        self.gaps
            .get(&id.to_string())
            .map_or(&[][..], Vec::as_slice)
    }
}

fn sites(n: usize) -> String {
    format!("{n} gated site{}", if n == 1 { "" } else { "s" })
}

/// `enable (25 gated sites in src/span.rs, 43 in the crate)`, features joined with `, `.
pub fn describe(gaps: &[FeatureGap]) -> String {
    gaps.iter()
        .map(|g| {
            if g.by_module {
                format!(
                    "{} (gates the module of {}, {} in the crate)",
                    g.feature,
                    g.file,
                    sites(g.in_crate)
                )
            } else if g.in_file == 0 {
                format!(
                    "{} ({} in the crate, none in {})",
                    g.feature,
                    sites(g.in_crate),
                    g.file
                )
            } else {
                format!(
                    "{} ({} in {}, {} in the crate)",
                    g.feature,
                    sites(g.in_file),
                    g.file,
                    g.in_crate
                )
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// The sentence that names the features a harness was built without and the command that
/// builds one with them. It leads the remedy of a target whose code did not run, and it is
/// the line of the target block.
pub fn lead(gaps: &[FeatureGap], command: &str) -> String {
    let many = gaps.len() > 1;
    let state = if gaps.iter().all(|g| !g.default_on) {
        "off by default"
    } else {
        "off in this build"
    };
    format!(
        "{} {} {state} and the harness is built without {}, so the code behind {} does not run in it: regenerate it with {command}",
        if many { "features" } else { "feature" },
        describe(gaps),
        if many { "them" } else { "it" },
        if many { "them" } else { "it" },
    )
}

/// The features of `gaps`, sorted and without repeats.
pub fn names(gaps: &[FeatureGap]) -> Vec<String> {
    let mut v: Vec<String> = gaps.iter().map(|g| g.feature.clone()).collect();
    v.sort();
    v.dedup();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gap(feature: &str, in_file: usize, in_crate: usize) -> FeatureGap {
        FeatureGap {
            feature: feature.to_owned(),
            file: "src/span.rs".to_owned(),
            in_file,
            in_crate,
            in_entry: 0,
            by_module: false,
            default_on: false,
        }
    }

    #[test]
    fn the_lead_names_the_features_their_counts_and_the_command() {
        let one = lead(
            &[gap("enable", 25, 43)],
            "dla harness gen --features enable",
        );
        assert_eq!(
            one,
            "feature enable (25 gated sites in src/span.rs, 43 in the crate) off by default and the harness is built without it, so the code behind it does not run in it: regenerate it with dla harness gen --features enable"
        );
        let two = lead(&[gap("a", 1, 1), gap("b", 0, 2)], "C");
        assert!(
            two.starts_with(
                "features a (1 gated site in src/span.rs, 1 in the crate), b (2 gated sites in the crate, none in src/span.rs) off by default and the harness is built without them"
            ),
            "{two}"
        );
    }

    #[test]
    fn the_file_is_written_and_read_back_by_target() {
        let dir = tempfile::tempdir().unwrap();
        let mut f = HarnessFeatures {
            command: "C".into(),
            ..HarnessFeatures::default()
        };
        f.gaps.insert("F3".into(), vec![gap("enable", 1, 2)]);
        f.save(dir.path()).unwrap();
        let text = fs::read_to_string(dir.path().join(FILE)).unwrap();
        let back: HarnessFeatures = serde_json::from_str(&text).unwrap();
        assert_eq!(back, f);
        let id: FindingId = "F3".parse().unwrap();
        assert_eq!(back.gaps_of(&id).len(), 1);
        assert!(back.gaps_of(&"F4".parse().unwrap()).is_empty());
    }
}
