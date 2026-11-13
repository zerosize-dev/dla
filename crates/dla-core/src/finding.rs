//! Findings: one per hypothesis.

use std::fmt;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use crate::{
    error::{Refusal, codes},
    ids::{EvidenceId, FindingId},
};

/// The archetype every finding carries until `dla ledger label` changes it.
pub const UNCLASSIFIED: &str = "UNCLASSIFIED";

string_enum! {
    /// Lifecycle state of a finding.
    pub enum FindingState {
        /// Raised, not yet tested.
        Candidate = "candidate",
        /// A test is in progress.
        UnderTest = "under_test",
        /// Growth, site and holder established.
        Confirmed = "confirmed",
        /// Independently corroborated.
        Strong = "strong",
        /// Ruled out by a discriminator.
        Refuted = "refuted",
        /// Seen statically, never exercised.
        SuspectedStaticOnly = "suspected_static_only",
        /// Left open, with a reason and a remedy.
        Unresolved = "unresolved",
    }
}

impl FindingState {
    /// True for states that count as a disposition in the report.
    pub fn is_dispositioned(self) -> bool {
        !matches!(self, Self::Candidate | Self::UnderTest)
    }
    /// True for states that require a reason and a remedy.
    pub fn needs_reason(self) -> bool {
        matches!(self, Self::Unresolved | Self::SuspectedStaticOnly)
    }
}

string_enum! {
    /// Why a finding could not be resolved.
    pub enum UnresolvedReason {
        /// The suspected path was never exercised.
        NotExercised = "not_exercised",
        /// No harness exists to exercise it.
        NoHarness = "no_harness",
        /// The needed environment is unavailable.
        EnvUnavailable = "env_unavailable",
        /// The project does not build.
        BuildFails = "build_fails",
        /// The user declined a privileged step.
        PrivilegeDeclined = "privilege_declined",
        /// A required tool is missing.
        ToolMissing = "tool_missing",
        /// The time budget ran out: the deadline is near, or a command was refused for
        /// time. Not a stand-in for any other reason.
        TimeBudget = "time_budget",
        /// Measured, and the measurement cannot decide: too few points, no plateau yet, a
        /// single snapshot, or a discriminator that came back inconclusive. Only
        /// `unresolved` takes it, and only with a measurement tied to the finding's family.
        Inconclusive = "inconclusive",
    }
}

string_enum! {
    /// What holds the leaked memory.
    pub enum Holder {
        /// A static or global variable.
        StaticGlobal = "static/global",
        /// A registration table (listeners, callbacks).
        RegistrationTable = "registration-table",
        /// A field of a container object.
        ContainerField = "container-field",
        /// Thread-local storage.
        ThreadLocal = "thread-local",
        /// A blocked stack.
        BlockedStack = "blocked-stack",
        /// A closure or continuation.
        ClosureContinuation = "closure/continuation",
        /// A native handle or global reference.
        NativeHandleGlobalRef = "native-handle/global-ref",
        /// A reference cycle only.
        CycleOnly = "cycle-only",
    }
}

string_enum! {
    /// Which memory region grows.
    pub enum Region {
        /// The managed heap.
        Heap = "heap",
        /// Native memory outside the managed heap.
        OffHeap = "off-heap",
        /// Runtime-internal non-heap memory.
        NonHeap = "non-heap",
    }
}

string_enum! {
    /// Shape of the growth curve.
    pub enum Shape {
        /// Steady growth.
        Steady = "steady",
        /// Step-wise growth.
        Stair = "stair",
        /// Transient spike.
        Spike = "spike",
        /// Growth that levels off.
        Plateau = "plateau",
    }
}

string_enum! {
    /// Where the leak lives.
    pub enum Locus {
        /// Application code.
        App = "app",
        /// A dependency.
        Dependency = "dependency",
        /// The language runtime.
        Runtime = "runtime",
    }
}

string_enum! {
    /// Class of repair.
    pub enum RepairClass {
        /// Release on the regular path.
        ReleaseRegular = "release-regular",
        /// Release on the exceptional path.
        ReleaseExceptional = "release-exceptional",
        /// Remove the entry.
        RemoveEntry = "remove-entry",
        /// Bound or evict.
        BoundEvict = "bound/evict",
        /// Deregister.
        Deregister = "deregister",
        /// Stop or cancel.
        StopCancel = "stop/cancel",
        /// Clear the reference.
        ClearReference = "clear-reference",
        /// Restructure synchronisation.
        RestructureSync = "restructure-sync",
        /// Return ownership.
        ReturnOwnership = "return-ownership",
    }
}

string_enum! {
    /// Whether an explanation was seen or reasoned.
    pub enum ExplanationTag {
        /// Directly observed.
        Observed = "observed",
        /// Inferred from observations.
        Inferred = "inferred",
    }
}

/// Free-text explanation with its epistemic tag.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Explanation {
    /// The explanation.
    pub text: String,
    /// Observed or inferred.
    pub tag: ExplanationTag,
}

/// Checks a `file:line` site: a non-empty path without control characters, a colon and
/// a positive decimal line number (no sign, no leading zeros).
pub fn check_site(site: &str) -> Result<(), String> {
    let bad = |why: &str| Err(format!("site {site:?} is not file:line ({why})"));
    let Some((file, line)) = site.rsplit_once(':') else {
        return bad("no colon");
    };
    if file.trim().is_empty() || file.chars().any(char::is_control) {
        return bad("empty or unprintable file");
    }
    if line.is_empty() || line.starts_with('0') || !line.bytes().all(|b| b.is_ascii_digit()) {
        return bad("the line must be a positive number");
    }
    if line.parse::<u32>().is_err() {
        return bad("line number too large");
    }
    Ok(())
}

/// Where a finding came from. Validated: `detector:<backend>/<rule>`, `census`,
/// `growth-series`, `archetype-row`, `known-leak:<id>`, `correlation`, `residual`,
/// `verifier`, `hypothesis`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct Origin(String);

/// The origin string is not one of the allowed forms.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error(
    "invalid origin {0:?}: expected detector:<backend>/<rule>, census, growth-series, archetype-row, known-leak:<id>, correlation, residual, verifier or hypothesis"
)]
pub struct OriginError(pub String);

fn token_ok(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| !c.is_whitespace() && !c.is_control())
}

impl Origin {
    /// Parses and validates an origin string.
    pub fn parse(s: &str) -> Result<Self, OriginError> {
        let ok = match s {
            "census" | "growth-series" | "archetype-row" | "correlation" | "residual"
            | "verifier" | "hypothesis" => true,
            _ => {
                if let Some(rest) = s.strip_prefix("detector:") {
                    rest.split_once('/')
                        .is_some_and(|(b, r)| token_ok(b) && token_ok(r) && !b.contains(':'))
                } else if let Some(id) = s.strip_prefix("known-leak:") {
                    token_ok(id)
                } else {
                    false
                }
            }
        };
        if ok {
            Ok(Self(s.to_owned()))
        } else {
            Err(OriginError(s.to_owned()))
        }
    }

    /// The origin as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Origin {
    type Error = OriginError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse(&s)
    }
}

impl From<Origin> for String {
    fn from(o: Origin) -> String {
        o.0
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<OriginError> for Refusal {
    fn from(e: OriginError) -> Self {
        Refusal::new(
            codes::INVALID_ORIGIN,
            e.to_string(),
            "a valid origin",
            "dla ledger add --origin census --evidence E1",
        )
    }
}

impl JsonSchema for Origin {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("Origin")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": "^(census|growth-series|archetype-row|correlation|residual|verifier|hypothesis|detector:[^\\s:/]+/\\S+|known-leak:\\S+)$"
        })
    }
}

/// The seven non-leak explanations a sibling hypothesis can name.
pub const NON_LEAK_IDS: [&str; 7] = [
    "N-ALLOC",
    "N-HEADROOM",
    "N-WARM",
    "N-HWM",
    "N-SCALE",
    "N-INTENT",
    "N-OBSERVER",
];

/// The `N-*` id a hypothesis begins with, when it is one of [`NON_LEAK_IDS`] followed by
/// the end of the text, a space or a colon.
pub fn nonleak_id(hypothesis: &str) -> Option<&'static str> {
    let text = hypothesis.trim_start();
    NON_LEAK_IDS.iter().copied().find(|id| {
        text.strip_prefix(id)
            .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', ':']))
    })
}

/// True when the hypothesis begins with `N-`, whether or not it names a valid id.
pub fn claims_nonleak(hypothesis: &str) -> bool {
    hypothesis.trim_start().starts_with("N-")
}

/// Which side of the leak question a finding argues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Polarity {
    /// The finding says memory is leaked.
    Leak,
    /// The finding says the growth has a non-leak explanation (an `N-*` sibling).
    NonLeak,
}

fn default_archetype() -> String {
    UNCLASSIFIED.to_owned()
}

/// One hypothesis about a leak and everything known about it.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct Finding {
    /// Identifier.
    pub id: FindingId,
    /// Lifecycle state.
    pub state: FindingState,
    /// Why unresolved or static-only; required exactly for those states.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<UnresolvedReason>,
    /// What would unblock it; non-empty whenever `reason` is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remedy: Option<String>,
    /// How the finding was raised.
    pub origin: Origin,
    /// A tag suggesting an archetype; never filters anything.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suggested_archetype: Option<String>,
    /// Assigned archetype; only `dla ledger label` changes it.
    #[serde(default = "default_archetype")]
    pub archetype: String,
    /// The cause this hypothesis proposes (on a sibling).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hypothesis: Option<String>,
    /// The result that would refute this hypothesis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refute_if: Option<String>,
    /// Parent finding of a sibling hypothesis.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent: Option<FindingId>,
    /// What holds the memory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holder: Option<Holder>,
    /// Owning field or container on the retention path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Memory region.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<Region>,
    /// Growth shape.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<Shape>,
    /// Where the leak lives.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locus: Option<Locus>,
    /// `file:line`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub site: Option<String>,
    /// Factors that worsen the leak.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub amplifiers: Vec<String>,
    /// Class of repair.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repair_class: Option<RepairClass>,
    /// Evidence supporting the finding.
    #[serde(default)]
    pub supports: Vec<EvidenceId>,
    /// Evidence contradicting the finding.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contradicts: Vec<EvidenceId>,
    /// Sibling hypotheses; maintained by the ledger.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub siblings: Vec<FindingId>,
    /// Explanation with its tag.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<Explanation>,
}

impl Finding {
    /// A new candidate with the mandatory fields and defaults elsewhere.
    pub fn candidate(id: FindingId, origin: Origin, supports: Vec<EvidenceId>) -> Self {
        Self {
            id,
            state: FindingState::Candidate,
            reason: None,
            remedy: None,
            origin,
            suggested_archetype: None,
            archetype: default_archetype(),
            hypothesis: None,
            refute_if: None,
            parent: None,
            holder: None,
            owner: None,
            region: None,
            shape: None,
            locus: None,
            site: None,
            amplifiers: Vec::new(),
            repair_class: None,
            supports,
            contradicts: Vec::new(),
            siblings: Vec::new(),
            explanation: None,
        }
    }

    /// Leak-side for a top-level finding and for a sibling whose hypothesis does not
    /// begin with an `N-*` id; non-leak for a sibling that does.
    pub fn polarity(&self) -> Polarity {
        if self.parent.is_some() && self.hypothesis.as_deref().and_then(nonleak_id).is_some() {
            Polarity::NonLeak
        } else {
            Polarity::Leak
        }
    }

    /// The finding that roots this finding's family: its parent, or itself.
    pub fn root_id(&self) -> FindingId {
        self.parent.clone().unwrap_or_else(|| self.id.clone())
    }

    /// The state rendered with its reason, such as `unresolved(time_budget)`.
    pub fn state_label(&self) -> String {
        match self.reason {
            Some(r) if self.state.needs_reason() => format!("{}({})", self.state, r),
            _ => self.state.to_string(),
        }
    }
}
