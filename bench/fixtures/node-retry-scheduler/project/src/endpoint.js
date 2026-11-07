'use strict';

const { hash } = require('./backoff');

// Simulated receiving side. Some targets answer with a server error on the
// first tries, which is what pushes jobs onto the retry path.
class Endpoint {
  constructor(options = {}) {
    this.flakyShare = options.flakyShare ?? 35;
    this.received = 0;
  }

  deliver(job, attempt) {
    const roll = hash(`${job.target}/${job.id}`) % 100;
    const failsFirst = roll < this.flakyShare;
    const failsSecond = roll < this.flakyShare / 3;
    if ((attempt === 1 && failsFirst) || (attempt === 2 && failsSecond)) {
      return { ok: false, status: 503 };
    }
    this.received += 1;
    return { ok: true, status: 200 };
  }
}

module.exports = { Endpoint };
