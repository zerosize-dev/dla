use dla_core::{EvidenceId, FindingId, OpenItemId};

#[test]
fn valid_ids_round_trip() {
    for s in ["E1", "E42", "jvm/E3", "node-2/E10"] {
        assert_eq!(s.parse::<EvidenceId>().unwrap().to_string(), s);
    }
    for s in ["F1", "F12", "F3.n1", "F3.n12", "jvm/F3", "py/F3.n2"] {
        assert_eq!(s.parse::<FindingId>().unwrap().to_string(), s);
    }
    for s in ["O1", "O9", "jvm/O2"] {
        assert_eq!(s.parse::<OpenItemId>().unwrap().to_string(), s);
    }
}

#[test]
fn invalid_ids_are_rejected() {
    for s in [
        "", "E", "E0", "E01", "e1", "F1", "E1.n1", "E-1", "E+1", "/E1", "JVM/E1", "a/b/E1", "E1 ",
    ] {
        assert!(s.parse::<EvidenceId>().is_err(), "{s:?} should fail");
    }
    for s in [
        "", "F", "F0", "F3.", "F3.n", "F3.n0", "F3.n01", "F3.m1", "F3.n1.n2", "E3", "f3", "F 3",
    ] {
        assert!(s.parse::<FindingId>().is_err(), "{s:?} should fail");
    }
    for s in ["", "O", "O0", "F1", "o1", "O1.n1"] {
        assert!(s.parse::<OpenItemId>().is_err(), "{s:?} should fail");
    }
}

#[test]
fn sibling_helpers() {
    let f: FindingId = "F3.n2".parse().unwrap();
    assert!(f.is_sibling());
    assert_eq!(f.sibling_number(), Some(2));
    assert_eq!(f.parent().unwrap().to_string(), "F3");
    assert_eq!(FindingId::new(3).sibling(1).to_string(), "F3.n1");
    assert_eq!(FindingId::new(3).parent(), None);
}

#[test]
fn ordering_is_numeric() {
    let mut v: Vec<FindingId> = ["F10", "F2", "F2.n1", "F1"]
        .iter()
        .map(|s| s.parse().unwrap())
        .collect();
    v.sort();
    let s: Vec<String> = v.iter().map(ToString::to_string).collect();
    assert_eq!(s, ["F1", "F2", "F2.n1", "F10"]);
}

#[test]
fn serde_uses_strings() {
    let id: EvidenceId = serde_json::from_str("\"jvm/E3\"").unwrap();
    assert_eq!(id.prefix(), Some("jvm"));
    assert_eq!(serde_json::to_string(&id).unwrap(), "\"jvm/E3\"");
    assert!(serde_json::from_str::<EvidenceId>("\"X3\"").is_err());
}
