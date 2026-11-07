'use strict';

const { PendingList } = require('./pending');
const { delayFor } = require('./backoff');

const DEAD_LETTER_LIMIT = 100;

// Runs delivery attempts when they fall due and re-arms failed ones.
class RetryScheduler {
  constructor({ clock, endpoint, maxAttempts = 4 }) {
    this.clock = clock;
    this.endpoint = endpoint;
    this.maxAttempts = maxAttempts;
    this.pending = new PendingList();
    this.issued = [];
    this.deadLetters = [];
    this.inflight = new Map();
    this.counts = { delivered: 0, retried: 0, dead: 0 };
  }

  submit(job) {
    this.inflight.set(job.id, job);
    this.pending.append({ due: this.clock.now(), attempt: 1, run: () => this.attempt(job, 1), next: null });
  }

  // Walk the pending list once and run everything that is due.
  tick() {
    const now = this.clock.now();
    let prev = null;
    let node = this.pending.head;
    while (node) {
      const next = node.next;
      if (node.due <= now) {
        this.pending.unlink(prev, node);
        node.run();
      } else {
        prev = node;
      }
      node = next;
    }
  }

  attempt(job, attempt) {
    const result = this.endpoint.deliver(job, attempt);
    if (result.ok) {
      this.counts.delivered += 1;
      this.inflight.delete(job.id);
      return;
    }
    if (attempt >= this.maxAttempts) {
      this.counts.dead += 1;
      this.inflight.delete(job.id);
      this.deadLetters.push({ id: job.id, status: result.status });
      if (this.deadLetters.length > DEAD_LETTER_LIMIT) this.deadLetters.shift();
      return;
    }
    const retry = () => this.attempt(job, attempt + 1);
    this.issued.push(retry);
    this.counts.retried += 1;
    const due = this.clock.now() + delayFor(attempt, job.id);
    this.pending.append({ due, attempt: attempt + 1, run: retry, next: null });
  }

  stats() {
    return {
      pending: this.pending.length,
      inflight: this.inflight.size,
      issued: this.issued.length,
      ...this.counts,
    };
  }
}

module.exports = { RetryScheduler, DEAD_LETTER_LIMIT };
