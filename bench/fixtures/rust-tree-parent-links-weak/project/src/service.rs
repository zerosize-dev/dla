//! Request handling: parse a submitted document and answer with its contents listing.

use crate::parser::{self, ParseError};
use crate::toc;

#[derive(Debug)]
pub struct Response {
    pub headings: usize,
    pub body_lines: usize,
    pub toc_bytes: usize,
    pub deepest: usize,
}

#[derive(Default)]
pub struct Service {
    pub handled: u64,
    pub rejected: u64,
}

impl Service {
    pub fn handle(&mut self, text: &str) -> Result<Response, ParseError> {
        let parsed = match parser::parse(text) {
            Ok(parsed) => parsed,
            Err(err) => {
                self.rejected += 1;
                return Err(err);
            }
        };
        let listing = toc::render(&parsed.root);
        let deepest = listing
            .lines()
            .map(|l| l.split(' ').next().unwrap_or("").matches('.').count() + 1)
            .max()
            .unwrap_or(0);
        self.handled += 1;
        Ok(Response {
            headings: parsed.headings,
            body_lines: parsed.body_lines,
            toc_bytes: listing.len(),
            deepest,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_handled_and_rejected_requests() {
        let mut service = Service::default();
        assert!(service.handle("# A\n## B\n").is_ok());
        assert!(service.handle("nothing here\n").is_err());
        assert_eq!((service.handled, service.rejected), (1, 1));
    }

    #[test]
    fn reports_the_depth_of_the_outline() {
        let mut service = Service::default();
        let response = service.handle("# A\n## B\n### C\n").unwrap();
        assert_eq!(response.deepest, 3);
        assert_eq!(response.headings, 3);
    }
}
