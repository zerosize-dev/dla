/// A report export request: how many sections and how big each one is.
#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    pub id: u64,
    pub sections: Vec<usize>,
}

/// What a finished job hands back.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    pub id: u64,
    pub bytes: usize,
    pub checksum: u64,
}

/// What a worker keeps of a finished job for the status page.
#[derive(Debug, Clone, PartialEq)]
pub struct Receipt {
    pub outcome: Outcome,
    pub title: String,
}

impl Job {
    pub fn total(&self) -> usize {
        self.sections.iter().sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_adds_the_sections() {
        let job = Job { id: 1, sections: vec![100, 250, 7] };
        assert_eq!(job.total(), 357);
    }
}
