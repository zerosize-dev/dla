'use strict';

const test = require('node:test');
const assert = require('node:assert');
const gateway = require('../src/gateway');
const audit = require('../src/audit');
const trail = require('../src/trail');
const routes = require('../src/routes');
const { runWorkload } = require('../load');

test('a tracking lookup returns the parcel checkpoints', () => {
  const response = gateway.handle({ path: '/parcels/track', parcel: 123456, client: 'shop-1' });
  assert.strictEqual(response.status, 200);
  assert.strictEqual(JSON.parse(response.body).checkpoints.length, 9);
});

test('an unknown path is a 404', () => {
  const response = gateway.handle({ path: '/parcels/missing', parcel: 1, client: 'shop-1' });
  assert.strictEqual(response.status, 404);
  assert.strictEqual(routes.lookup('/parcels/missing'), null);
});

test('the audit notes name the request and its timings', () => {
  const response = gateway.handle({ path: '/parcels/eta', parcel: 4242, client: 'shop-2' });
  const notes = audit.recent();
  assert.match(notes[notes.length - 1], new RegExp(`/parcels/eta #${response.id} 200`));
});

test('a response explains how it relates to the previous one', () => {
  gateway.handle({ path: '/parcels/eta', parcel: 77, client: 'shop-3' });
  const second = gateway.handle({ path: '/parcels/label', parcel: 77, client: 'shop-3' });
  assert.match(second.explain(), /same parcel as #\d+ \(200\)/);
  assert.strictEqual(trail.latest(), second);
});

test('the replay serves most of its requests', () => {
  const result = runWorkload(500);
  assert.strictEqual(result.requests, 500);
  assert.ok(result.ok > 300);
});
