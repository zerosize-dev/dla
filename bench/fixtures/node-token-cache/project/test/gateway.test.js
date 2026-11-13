'use strict';

const test = require('node:test');
const assert = require('node:assert');
const { Signer } = require('../src/signer');
const { Gateway } = require('../src/gateway');
const { decode } = require('../src/token');
const { allowed } = require('../src/scopes');
const { runWorkload } = require('../load');

function setup(now = 0) {
  const state = { now };
  const signer = new Signer('test-secret');
  return { state, signer, gateway: new Gateway(signer, () => state.now) };
}

function tokenFor(signer, scopes, expiresAt) {
  return signer.sign({ subject: 'acct-1', scopes, expiresAt });
}

test('a valid token is accepted and remembered', () => {
  const { signer, gateway } = setup();
  const token = tokenFor(signer, ['read:statements'], 100);
  const first = gateway.authenticate({ token, path: '/statements', body: 'x' });
  const second = gateway.authenticate({ token, path: '/statements', body: 'x' });
  assert.strictEqual(first.status, 200);
  assert.strictEqual(second.status, 200);
  assert.strictEqual(gateway.cache.hits, 1);
});

test('a token with a bad signature is refused', () => {
  const { signer, gateway } = setup();
  const token = tokenFor(signer, ['read:statements'], 100).replace(/.$/, '0');
  assert.strictEqual(gateway.authenticate({ token, path: '/statements', body: 'x' }).status, 401);
});

test('an expired token is refused', () => {
  const { signer, gateway, state } = setup();
  const token = tokenFor(signer, ['read:statements'], 10);
  state.now = 11;
  assert.strictEqual(gateway.authenticate({ token, path: '/statements', body: 'x' }).status, 401);
});

test('scopes decide the route', () => {
  const { signer, gateway } = setup();
  const token = tokenFor(signer, ['read:statements'], 100);
  assert.strictEqual(gateway.authenticate({ token, path: '/transfers', body: 'x' }).status, 403);
  assert.strictEqual(allowed(decode(token).claims, '/profile'), false);
});

test('the replay serves every request it can', () => {
  const result = runWorkload(600);
  assert.strictEqual(result.counts[200] + result.counts[403], 600);
  assert.ok(result.hits > result.misses);
});
