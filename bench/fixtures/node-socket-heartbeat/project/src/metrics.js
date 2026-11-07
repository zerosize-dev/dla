'use strict';

const LATENCY_EDGES_MS = [1, 5, 20, 100, 500];

class Metrics {
  constructor() {
    this.counters = new Map();
    this.latency = new Array(LATENCY_EDGES_MS.length + 1).fill(0);
  }

  inc(name, by = 1) {
    this.counters.set(name, (this.counters.get(name) || 0) + by);
  }

  get(name) {
    return this.counters.get(name) || 0;
  }

  observe(ms) {
    let slot = LATENCY_EDGES_MS.findIndex((edge) => ms <= edge);
    if (slot === -1) slot = LATENCY_EDGES_MS.length;
    this.latency[slot] += 1;
  }

  snapshot() {
    return { counters: Object.fromEntries(this.counters), latency: [...this.latency] };
  }
}

module.exports = { Metrics, LATENCY_EDGES_MS };
