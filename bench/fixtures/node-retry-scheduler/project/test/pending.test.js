'use strict';

const test = require('node:test');
const assert = require('node:assert');
const { PendingList } = require('../src/pending');
const { delayFor } = require('../src/backoff');

test('append keeps insertion order and unlink repairs the links', () => {
  const list = new PendingList();
  const [a, b, c] = ['a', 'b', 'c'].map((name) => list.append({ name }));
  assert.deepStrictEqual([...list].map((n) => n.name), ['a', 'b', 'c']);
  list.unlink(a, b);
  assert.deepStrictEqual([...list].map((n) => n.name), ['a', 'c']);
  list.unlink(null, a);
  list.unlink(null, c);
  assert.strictEqual(list.length, 0);
  assert.strictEqual(list.tail, null);
});

test('backoff doubles up to the cap', () => {
  const first = delayFor(1, 5);
  const third = delayFor(3, 5);
  assert.ok(third > first);
  assert.ok(delayFor(20, 5) <= 30000);
  assert.strictEqual(delayFor(2, 9), delayFor(2, 9));
});
