//! Statement lines waiting to be written to the downstream file.

use std::io::Write;

/// What one statement line says about an applied batch.
#[derive(Clone, Copy)]
pub struct Statement {
    pub batch: u64,
    pub postings: usize,
    pub net: i64,
    pub accounts: usize,
    pub first_ref: u32,
    pub last_ref: u32,
}

struct Line {
    text: String,
}

impl Statement {
    /// The line this statement is written as.
    fn open(&self) -> Line {
        let text = format!(
            "batch {:08} postings={:02} net={:+012} accounts={} first_ref={:06} last_ref={:06} channel=bank-feed",
            self.batch, self.postings, self.net, self.accounts, self.first_ref, self.last_ref
        );
        Line { text }
    }
}

/// Lines are handed to the writer as raw nodes so the writer can run without borrowing the outbox.
#[derive(Default)]
pub struct Outbox {
    pending: Vec<*mut Line>,
    written: usize,
}

impl Outbox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn queue(&mut self, statement: Statement) {
        let node = Box::into_raw(Box::new(statement.open()));
        self.pending.push(node);
    }

    /// Writes the queued lines in order and returns how many there were.
    pub fn flush(&mut self, sink: &mut impl Write) -> std::io::Result<usize> {
        let count = self.pending.len();
        for node in self.pending.drain(..) {
            let line = unsafe { &*node };
            sink.write_all(line.text.as_bytes())?;
            sink.write_all(b"\n")?;
            self.written += line.text.len() + 1;
        }
        Ok(count)
    }

    pub fn waiting(&self) -> usize {
        self.pending.len()
    }

    pub fn bytes_written(&self) -> usize {
        self.written
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn statement(batch: u64) -> Statement {
        Statement { batch, postings: 7, net: 1250, accounts: 40, first_ref: 11, last_ref: 99 }
    }

    #[test]
    fn lines_are_written_in_order() {
        let mut outbox = Outbox::new();
        outbox.queue(statement(1));
        outbox.queue(statement(2));
        assert_eq!(outbox.waiting(), 2);
        let mut sink = Vec::new();
        assert_eq!(outbox.flush(&mut sink).unwrap(), 2);
        let text = String::from_utf8(sink).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[0].starts_with("batch 00000001 postings=07 net=+00000001250"));
        assert!(lines[1].starts_with("batch 00000002 "));
        assert_eq!(outbox.waiting(), 0);
        assert_eq!(outbox.bytes_written(), text.len());
    }

    #[test]
    fn flushing_an_empty_outbox_writes_nothing() {
        let mut outbox = Outbox::new();
        let mut sink = Vec::new();
        assert_eq!(outbox.flush(&mut sink).unwrap(), 0);
        assert!(sink.is_empty());
    }
}
