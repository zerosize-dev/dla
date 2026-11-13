'use strict';

const test = require('node:test');
const assert = require('node:assert');
const { describe, checksum, pack, KIND } = require('../src/frames');
const { Counters } = require('../src/stats');

test('pack writes eight bytes with a trailing checksum', () => {
  const bytes = pack({ kind: KIND.pressure, device: 4242, value: 977 });
  assert.strictEqual(bytes.length, 8);
  assert.strictEqual(bytes[7], checksum(bytes.subarray(0, 7)) & 0xff);
});

test('describe names the reading', () => {
  assert.strictEqual(describe({ kind: KIND.battery, device: 7, value: 88 }), 'battery@7=88');
  assert.strictEqual(describe({ kind: 9, device: 7, value: 1 }), 'unknown@7=1');
});

test('counters add up per kind', () => {
  const counters = new Counters();
  counters.record(1);
  counters.record(1);
  counters.record(2);
  assert.deepStrictEqual(counters.snapshot(), { total: 3, kinds: { 1: 2, 2: 1 } });
});
