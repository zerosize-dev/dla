use dla_core::redact::shape;

#[test]
fn pattern_examples() {
    let salt = b"salt";
    assert_eq!(shape("user_42@x.io", salt).pattern, "a+_9+@a.a+");
    assert_eq!(shape("Abc123", salt).pattern, "Aa+9+");
    assert_eq!(shape("", salt).pattern, "");
    assert_eq!(shape("x", salt).pattern, "a");
    assert_eq!(shape("AB-cd", salt).pattern, "A+-a+");
}

#[test]
fn length_counts_characters() {
    assert_eq!(shape("user_42@x.io", b"s").len, 12);
    assert_eq!(shape("héllo", b"s").len, 5);
}

#[test]
fn hash_is_stable_per_salt_and_value() {
    let a = shape("secret", b"salt-1");
    assert_eq!(a.hash.len(), 12);
    assert!(a.hash.bytes().all(|b| b.is_ascii_hexdigit()));
    assert_eq!(a, shape("secret", b"salt-1"));
    assert_ne!(a.hash, shape("secret", b"salt-2").hash);
    assert_ne!(a.hash, shape("secreT", b"salt-1").hash);
}

#[test]
fn hash_matches_known_hmac_vector() {
    // HMAC-SHA256(key="key", "The quick brown fox jumps over the lazy dog")
    let s = shape("The quick brown fox jumps over the lazy dog", b"key");
    assert_eq!(s.hash, "f7bc83f43053");
}
