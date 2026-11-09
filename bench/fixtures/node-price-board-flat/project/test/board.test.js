'use strict';

const test = require('node:test');
const assert = require('node:assert');
const board = require('../src/board');
const { Feed } = require('../src/feed');
const { movingAverage, volumeWeighted } = require('../src/stats');
const { runWorkload } = require('../load');

test('moving average over a window', () => {
  const ticks = [1, 2, 3, 4].map((price, t) => ({ t, price, size: 1 }));
  assert.deepStrictEqual(movingAverage(ticks, 2), [1, 1.5, 2.5, 3.5]);
});

test('volume weighted price', () => {
  const ticks = [
    { t: 0, price: 10, size: 1 },
    { t: 1, price: 20, size: 3 },
  ];
  assert.strictEqual(volumeWeighted(ticks), 17.5);
});

test('publish makes the newest snapshot current', () => {
  board.reset();
  const feed = new Feed(1, 10);
  const first = feed.next();
  board.publish(first.symbol, first.ticks);
  const second = feed.next();
  const snapshot = board.publish(second.symbol, second.ticks);
  assert.strictEqual(board.current(), snapshot);
  assert.strictEqual(snapshot.id, 2);
});

test('verbose publish logs the change', () => {
  board.reset();
  const notes = [];
  board.publish('ALB', [{ t: 0, price: 1, size: 1 }], { verbose: true, log: (n) => notes.push(n) });
  board.publish('ALB', [{ t: 0, price: 1, size: 1 }, { t: 1, price: 2, size: 1 }], { verbose: true, log: (n) => notes.push(n) });
  assert.deepStrictEqual(notes, ['ALB: first snapshot', 'ALB: 1 -> 2 ticks']);
});

test('workload runs', () => {
  board.reset();
  const result = runWorkload(50, 10);
  assert.strictEqual(result.updates, 50);
});
