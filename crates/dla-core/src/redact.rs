//! Value redaction: describe the shape of a string without revealing it.

use hmac::{Hmac, Mac};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

/// The shape of a redacted value.
#[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
pub struct ValueShape {
    /// Length in characters.
    pub len: usize,
    /// Character-class pattern, see [`shape`].
    pub pattern: String,
    /// First 12 hex characters of HMAC-SHA256(salt, value); stable within a run.
    pub hash: String,
}

fn class(c: char) -> char {
    if c.is_lowercase() {
        'a'
    } else if c.is_uppercase() {
        'A'
    } else if c.is_ascii_digit() {
        '9'
    } else {
        c
    }
}

/// Describes `value`: lowercase letters map to `a`, uppercase to `A`, ASCII digits
/// to `9`, every other character is kept, and a run of the same output character
/// collapses to that character followed by `+`
/// (`user_42@x.io` becomes `a+_9+@a.a+`).
pub fn shape(value: &str, salt: &[u8]) -> ValueShape {
    let mut pattern = String::new();
    let mut prev: Option<char> = None;
    let mut run = 0usize;
    let flush = |pattern: &mut String, prev: Option<char>, run: usize| {
        if let Some(p) = prev {
            pattern.push(p);
            if run > 1 {
                pattern.push('+');
            }
        }
    };
    for c in value.chars().map(class) {
        if Some(c) == prev {
            run += 1;
        } else {
            flush(&mut pattern, prev, run);
            prev = Some(c);
            run = 1;
        }
    }
    flush(&mut pattern, prev, run);

    // HMAC accepts keys of any length, so construction cannot fail.
    let mut mac = Hmac::<Sha256>::new_from_slice(salt)
        .unwrap_or_else(|_| Hmac::<Sha256>::new_from_slice(&[]).expect("empty key is valid"));
    mac.update(value.as_bytes());
    let digest = hex::encode(mac.finalize().into_bytes());
    ValueShape {
        len: value.chars().count(),
        pattern,
        hash: digest[..12].to_owned(),
    }
}
