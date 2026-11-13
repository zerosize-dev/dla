'use strict';

// Bearer tokens look like <subject>.<scope,scope>.<expiresAt>.<signature>.

function encode(claims, signature) {
  return [claims.subject, claims.scopes.join(','), claims.expiresAt, signature].join('.');
}

function decode(text) {
  const parts = String(text).split('.');
  if (parts.length !== 4) return null;
  const expiresAt = Number.parseInt(parts[2], 10);
  if (!Number.isFinite(expiresAt)) return null;
  return {
    claims: { subject: parts[0], scopes: parts[1] ? parts[1].split(',') : [], expiresAt },
    signature: parts[3],
  };
}

function signingInput(claims) {
  return `${claims.subject}.${claims.scopes.join(',')}.${claims.expiresAt}`;
}

module.exports = { encode, decode, signingInput };
