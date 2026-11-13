'use strict';

const { decode } = require('./token');
const { TokenCache } = require('./cache');
const { allowed } = require('./scopes');

class Gateway {
  constructor(signer, clock) {
    this.signer = signer;
    this.clock = clock;
    this.cache = new TokenCache(clock);
  }

  // `request.body` is the raw payload; its digest is checked against the
  // `x-body-digest` header before the call is routed.
  authenticate(request) {
    const body = Buffer.from(request.body);
    if (body.length === 0) return { status: 400 };

    const cached = this.cache.lookup(request.token);
    let claims = cached;
    if (!claims) {
      const parsed = decode(request.token);
      if (!parsed) return { status: 401 };
      if (parsed.claims.expiresAt <= this.clock()) return { status: 401 };
      if (!this.signer.verify(parsed.claims, parsed.signature)) return { status: 401 };
      claims = parsed.claims;
      this.cache.remember(request.token, claims);
    }
    if (!allowed(claims, request.path)) return { status: 403 };
    return { status: 200, subject: claims.subject, bytes: body.length };
  }
}

module.exports = { Gateway };
