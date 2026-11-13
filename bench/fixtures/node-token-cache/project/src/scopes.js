'use strict';

const REQUIRED = new Map([
  ['/statements', 'read:statements'],
  ['/transfers', 'write:transfers'],
  ['/profile', 'read:profile'],
]);

function allowed(claims, path) {
  const need = REQUIRED.get(path);
  return need === undefined || claims.scopes.includes(need);
}

module.exports = { allowed, REQUIRED };
