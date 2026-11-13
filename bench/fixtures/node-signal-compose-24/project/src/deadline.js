'use strict';

// A per-request time budget. `release()` must be called when the request ends so
// the timer does not outlive it.

class Deadline {
  constructor(ms) {
    this.ms = ms;
    this.controller = new AbortController();
    this.signal = this.controller.signal;
    this.timer = setTimeout(() => {
      this.controller.abort(new Error(`budget of ${ms} ms used up`));
    }, ms);
  }

  release() {
    clearTimeout(this.timer);
  }
}

module.exports = { Deadline };
