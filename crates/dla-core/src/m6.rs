//! Refusal codes of the schema 6 ledger rules (`label`, `strong`, `explain`).

/// Stable codes of the refusals raised by `dla ledger label`, `strong` and `explain`.
pub mod codes {
    /// `label`: the evidence an archetype requires is not recorded (or not tied).
    pub const LABEL_EVIDENCE_MISSING: &str = "label-evidence-missing";
    /// `label`: an `L-*` label on a non-leak finding or an `N-*` label on a leak-side one.
    pub const LABEL_POLARITY: &str = "label-polarity";
    /// `label`: an `N-X` label on a sibling whose hypothesis names another id.
    pub const LABEL_HYPOTHESIS_MISMATCH: &str = "label-hypothesis-mismatch";
    /// `label`: the finding is not confirmed or strong.
    pub const LABEL_STATE: &str = "label-state";
    /// `label`: the finding's holder is not among the archetype's holders.
    pub const LABEL_HOLDER: &str = "label-holder";
    /// `label`: the archetype has no row in the finding's runtime reference and no `--note`
    /// says why none of the runtime's rows fits.
    pub const LABEL_RUNTIME_ROW: &str = "label-runtime-row";
    /// `label`: a sibling the archetype must see refuted is still open.
    pub const LABEL_DISCONFIRM_OPEN: &str = "label-disconfirm-open";
    /// `label`: the label is neither `UNCLASSIFIED` nor an archetype ID.
    pub const UNKNOWN_ARCHETYPE: &str = "unknown-archetype";
    /// Archetypes are off (`DLA_ARCHETYPES=off` or no table): only `UNCLASSIFIED` is accepted.
    pub const ARCHETYPES_OFF: &str = "archetypes-off";
    /// `ledger add --explanation` at schema 6: use `dla ledger explain`.
    pub const EXPLANATION_LOCKED: &str = "explanation-locked";
    /// `strong`: only a leak-side finding can become strong.
    pub const STRONG_LEAK_ONLY: &str = "strong-leak-only";
    /// `strong`: the cited records are not a second class of evidence.
    pub const STRONG_SECOND_CLASS: &str = "strong-second-class-missing";
    /// `strong`: the growth record has no signal, so independence cannot be judged.
    pub const STRONG_GROWTH_UNSIGNED: &str = "strong-growth-unsigned";
    /// `strong`: the growth and the second record rest on the same observation.
    pub const STRONG_NOT_INDEPENDENT: &str = "strong-not-independent";
    /// `strong`: the second record observed another holder kind than the finding's.
    pub const STRONG_HOLDER_MISMATCH: &str = "strong-holder-mismatch";
    /// `duplicate`: a finding the command cannot merge (hypothesis, non-leak, sibling, itself,
    /// or one already merged).
    pub const DUPLICATE_STATE: &str = "duplicate-state";
    /// `duplicate`: the two findings share neither an owner nor a site.
    pub const DUPLICATE_NOT_SAME: &str = "duplicate-not-same";
    /// `explain`: the finding is not a confirmed or strong leak-side finding.
    pub const EXPLAIN_STATE: &str = "explain-state";
    /// `explain`: the text does not name the site (or owner).
    pub const EXPLAIN_MISSING_SITE: &str = "explain-missing-site";
    /// `explain`: the text does not name the holder.
    pub const EXPLAIN_MISSING_HOLDER: &str = "explain-missing-holder";
    /// `explain`: no writer is recorded for the finding.
    pub const EXPLAIN_NO_WRITER: &str = "explain-no-writer";
    /// `explain`: an adding site is not a recorded writer, or the text names none.
    pub const EXPLAIN_ADDS: &str = "explain-adds";
    /// `explain`: `never` or `not-found` although a remover is recorded.
    pub const EXPLAIN_REMOVER_RECORDED: &str = "explain-remover-recorded";
    /// `explain`: the remover is missing, not recorded, or not named in the text.
    pub const EXPLAIN_REMOVER: &str = "explain-remover";
    /// `explain`: the text does not state the chosen removal.
    pub const EXPLAIN_REMOVAL_TEXT: &str = "explain-removal-text";
    /// `explain`: `observed` without evidence that shows the pairing.
    pub const EXPLAIN_TAG: &str = "explain-tag";
    /// `explain`: the text is too short, too long or has control characters.
    pub const EXPLAIN_TEXT: &str = "explain-text";
}
