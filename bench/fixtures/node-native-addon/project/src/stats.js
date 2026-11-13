'use strict';

class Counters {
  constructor() {
    this.byKind = new Map();
    this.total = 0;
  }

  record(kind) {
    this.byKind.set(kind, (this.byKind.get(kind) || 0) + 1);
    this.total += 1;
  }

  snapshot() {
    return { total: this.total, kinds: Object.fromEntries(this.byKind) };
  }
}

module.exports = { Counters };
