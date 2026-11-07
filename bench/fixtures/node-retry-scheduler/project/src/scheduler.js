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
    this.latest = null;
    this.deadLetters = [];
    this.inflight = new Map();
    this.counts = { delivered: 0, retried: 0, dead: 0 };
  }

  submit(job) {
    this.inflight.set(job.id, job);
    this.enqueue(job, 1, this.clock.now());
  }

  // Create the record for one delivery attempt and put it on the pending list.
  // Each record points at the attempt scheduled before it, so a trail of recent
  // attempts can be read back from the newest one.
  enqueue(job, attempt, due) {
    const record = { job, attempt, due, previous: null, next: null };
    record.previous = this.latest;
    this.latest = record;
    return this.pending.append(record);
  }

  // The most recent attempts, newest first.
  recent(count = 10) {
    const trail = [];
    for (let record = this.latest; record && trail.length < count; record = record.previous) {
      trail.push({ id: record.job.id, attempt: record.attempt, due: record.due });
    }
    return trail;
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
        this.attempt(node.job, node.attempt);
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
    this.counts.retried += 1;
    this.enqueue(job, attempt + 1, this.clock.now() + delayFor(attempt, job.id));
  }

  stats() {
    return {
      pending: this.pending.length,
      inflight: this.inflight.size,
      ...this.counts,
    };
  }
}

// The dispatcher process uses one scheduler for its whole lifetime.
let shared = null;

function sharedScheduler(options) {
  if (!shared) shared = new RetryScheduler(options);
  return shared;
}

module.exports = { RetryScheduler, sharedScheduler, DEAD_LETTER_LIMIT };
