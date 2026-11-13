'use strict';

const crypto = require('node:crypto');

// Remembers tokens that have already passed signature verification so that a
// client that presents the same token again skips the HMAC.
class TokenCache {
  constructor(clock) {
    this.clock = clock;
    this.entries = new Map();
    this.hits = 0;
    this.misses = 0;
  }

  lookup(token) {
    const entry = this.entries.get(token);
    if (!entry) {
      this.misses += 1;
      return null;
    }
    if (entry.expiresAt <= this.clock()) {
      this.entries.delete(token);
      this.misses += 1;
      return null;
    }
    const presented = Buffer.from(token);
    if (presented.length !== entry.raw.length || !crypto.timingSafeEqual(presented, entry.raw)) {
      this.misses += 1;
      return null;
    }
    this.hits += 1;
    return entry.claims;
  }

  remember(token, claims) {
    this.entries.set(token, { raw: Buffer.from(token), claims, expiresAt: claims.expiresAt });
  }

  size() {
    return this.entries.size;
  }
}

module.exports = { TokenCache };
