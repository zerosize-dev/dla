//! Deterministic sample documents for exercising the service.

const WORDS: [&str; 16] = [
    "Overview", "Install", "Network", "Storage", "Backup", "Restore", "Limits", "Billing",
    "Access", "Upgrade", "Metrics", "Alerts", "Quotas", "Regions", "Support", "Glossary",
];

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// Builds document number `seq`: 24 to 55 headings, three levels deep on average.
pub fn document(seq: u64) -> String {
    let mut rng = Lcg(seq.wrapping_mul(2654435761).wrapping_add(17));
    let count = 24 + rng.below(32);
    let mut level = 1;
    let mut text = String::new();
    for n in 0..count {
        match rng.below(4) {
            0 if level > 1 => level -= 1,
            1 | 2 if level < 4 => level += 1,
            _ => {}
        }
        let word = WORDS[rng.below(WORDS.len() as u64) as usize];
        text.push_str(&format!("{} {} {}-{}\n", "#".repeat(level), word, seq % 1000, n));
        for _ in 0..rng.below(3) {
            text.push_str("Body text for the section goes here.\n");
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documents_are_repeatable() {
        assert_eq!(document(7), document(7));
        assert_ne!(document(7), document(8));
    }

    #[test]
    fn documents_start_with_a_heading() {
        assert!(document(3).starts_with('#'));
    }
}
