'use strict';

// Manual clock so schedules can be replayed deterministically.
class ManualClock {
  constructor(start = 0) {
    this.current = start;
  }

  now() {
    return this.current;
  }

  advance(ms) {
    this.current += ms;
    return this.current;
  }
}

module.exports = { ManualClock };
