use crate::job::{Job, Outcome, Receipt};

/// Fills one section with deterministic content derived from the job and section number.
fn render_section(job: u64, index: usize, size: usize) -> Vec<u8> {
    let mut state = job.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ (index as u64 + 1);
    let mut out = Vec::with_capacity(size);
    for _ in 0..size {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        out.push((state & 0xff) as u8);
    }
    out
}

fn checksum(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ *b as u64).wrapping_mul(0x100_0000_01b3))
}

/// Renders every section, joins them into one document and returns a receipt with its size and checksum.
pub fn render(job: &Job) -> Box<Receipt> {
    let parts: Vec<Vec<u8>> = job
        .sections
        .iter()
        .enumerate()
        .map(|(i, size)| render_section(job.id, i, *size))
        .collect();
    let mut document = Vec::with_capacity(job.total());
    for part in &parts {
        document.extend_from_slice(part);
    }
    let outcome = Outcome { id: job.id, bytes: document.len(), checksum: checksum(&document) };
    Box::new(Receipt { outcome, title: format!("export {} ({} sections)", job.id, job.sections.len()) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rendering_is_deterministic() {
        let job = Job { id: 9, sections: vec![300, 4000, 12] };
        assert_eq!(render(&job), render(&job));
        assert_eq!(render(&job).outcome.bytes, 4312);
    }

    #[test]
    fn different_jobs_differ() {
        let a = render(&Job { id: 1, sections: vec![500] });
        let b = render(&Job { id: 2, sections: vec![500] });
        assert_ne!(a.outcome.checksum, b.outcome.checksum);
    }
}
