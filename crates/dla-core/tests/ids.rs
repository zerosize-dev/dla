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

#[test]
fn worker_prefixes_follow_the_worker_name_rules() {
    use dla_core::ids::valid_worker_name;
    for ok in ["jvm", "go", "node-2", "a", "rust"] {
        assert!(valid_worker_name(ok), "{ok}");
        assert!(format!("{ok}/E1").parse::<EvidenceId>().is_ok());
        assert!(format!("{ok}/F2.n1").parse::<FindingId>().is_ok());
        assert!(format!("{ok}/O3").parse::<OpenItemId>().is_ok());
    }
    let long = "x".repeat(33);
    for bad in [
        "",
        "Jvm",
        "1jvm",
        "-jvm",
        "a_b",
        "a b",
        "a/b",
        long.as_str(),
    ] {
        assert!(!valid_worker_name(bad), "{bad:?}");
        assert!(
            format!("{bad}/E1").parse::<EvidenceId>().is_err(),
            "{bad:?}"
        );
        assert!(format!("{bad}/F1").parse::<FindingId>().is_err(), "{bad:?}");
        assert!(
            format!("{bad}/O1").parse::<OpenItemId>().is_err(),
            "{bad:?}"
        );
    }
    // A sibling keeps the worker and a parent of a prefixed sibling is prefixed.
    let f: FindingId = "jvm/F2.n1".parse().unwrap();
    assert_eq!(f.prefix(), Some("jvm"));
    assert_eq!(f.parent().unwrap().to_string(), "jvm/F2");
    assert_eq!(
        FindingId::with_prefix(Some("jvm"), 4)
            .sibling(2)
            .to_string(),
        "jvm/F4.n2"
    );
    assert_eq!(f.reprefixed(None).to_string(), "F2.n1");
    assert_eq!(
        EvidenceId::new(3).reprefixed(Some("go")).to_string(),
        "go/E3"
    );
    assert_eq!(OpenItemId::with_prefix(Some("go"), 1).to_string(), "go/O1");
}

#[test]
fn prefixed_ids_round_trip_through_serde_and_sort_after_plain_ones() {
    for s in ["jvm/E3", "jvm/F2.n1", "jvm/O2"] {
        let json = format!("\"{s}\"");
        if let Ok(e) = serde_json::from_str::<EvidenceId>(&json) {
            assert_eq!(serde_json::to_string(&e).unwrap(), json);
        }
        if let Ok(f) = serde_json::from_str::<FindingId>(&json) {
            assert_eq!(serde_json::to_string(&f).unwrap(), json);
        }
        if let Ok(o) = serde_json::from_str::<OpenItemId>(&json) {
            assert_eq!(serde_json::to_string(&o).unwrap(), json);
        }
    }
    assert!(serde_json::from_str::<FindingId>("\"Jvm/F1\"").is_err());
}
