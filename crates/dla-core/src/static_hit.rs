//! One hit of a static backend, as adapters report it before it becomes a candidate.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::finding::{Holder, Region};

/// A static analysis hit, normalised from a backend's SARIF or JSON output.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq)]
pub struct StaticHit {
    /// Backend name (one of the detector backends).
    pub backend: String,
    /// The backend's rule identifier.
    pub rule: String,
    /// Project-relative file with `/` separators.
    pub file: String,
    /// 1-based line.
    pub line: u32,
    /// The backend's message, trimmed.
    pub message: String,
    /// What holds the memory, when the backend knows (the DLA rules that name a container;
    /// none for external backends). The candidate takes it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub holder: Option<Holder>,
    /// The owner of the holder (`<module>.<Type>.<field>`), when the backend knows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// The region that grows, when the backend knows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region: Option<Region>,
}

impl StaticHit {
    /// The candidate origin: `detector:<backend>/<rule>`.
    pub fn origin(&self) -> String {
        format!("detector:{}/{}", self.backend, self.rule)
    }

    /// The candidate site: `file:line`.
    pub fn site(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn holder_owner_and_region_are_optional_in_both_directions() {
        // The raw sweep file of an external backend (and of an older run) names none of them.
        let old = r#"{"backend":"ruff","rule":"B019","file":"a.py","line":3,"message":"m"}"#;
        let h: StaticHit = serde_json::from_str(old).unwrap();
        assert_eq!((&h.holder, &h.owner, &h.region), (&None, &None, &None));
        // A hit without them is written as before.
        assert_eq!(serde_json::to_string(&h).unwrap(), old);
        let named = StaticHit {
            holder: Some(Holder::ContainerField),
            owner: Some("edge.index.Index.order".into()),
            region: Some(Region::Heap),
            ..h.clone()
        };
        let back: StaticHit =
            serde_json::from_str(&serde_json::to_string(&named).unwrap()).unwrap();
        assert_eq!(back, named);
    }
}
