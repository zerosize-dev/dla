'use strict';

const test = require('node:test');
const assert = require('node:assert');
const { ManualClock } = require('../src/clock');
const { RetryScheduler, DEAD_LETTER_LIMIT } = require('../src/scheduler');

function scriptedEndpoint(outcomes) {
  const calls = [];
  return {
    calls,
    deliver(job, attempt) {
      calls.push([job.id, attempt]);
      const ok = outcomes[Math.min(attempt, outcomes.length) - 1];
      return ok ? { ok: true, status: 200 } : { ok: false, status: 503 };
    },
  };
}

const job = (id) => ({ id, target: 'https://hooks.example.test/t/inbox', payload: { order: id } });

test('a healthy target is delivered on the first tick', () => {
  const clock = new ManualClock();
  const endpoint = scriptedEndpoint([true]);
  const scheduler = new RetryScheduler({ clock, endpoint });
  scheduler.submit(job(1));
  scheduler.tick();
  assert.deepStrictEqual(endpoint.calls, [[1, 1]]);
  assert.strictEqual(scheduler.stats().delivered, 1);
  assert.strictEqual(scheduler.stats().pending, 0);
});

test('a failing attempt is retried after its delay', () => {
  const clock = new ManualClock();
  const endpoint = scriptedEndpoint([false, true]);
  const scheduler = new RetryScheduler({ clock, endpoint });
  scheduler.submit(job(2));
  scheduler.tick();
  assert.strictEqual(scheduler.stats().pending, 1);
  scheduler.tick();
  assert.strictEqual(endpoint.calls.length, 1);
  clock.advance(60000);
  scheduler.tick();
  assert.deepStrictEqual(endpoint.calls, [[2, 1], [2, 2]]);
  assert.strictEqual(scheduler.stats().delivered, 1);
  assert.strictEqual(scheduler.stats().inflight, 0);
});

test('jobs that exhaust their attempts land in the dead letters', () => {
  const clock = new ManualClock();
  const scheduler = new RetryScheduler({ clock, endpoint: scriptedEndpoint([false]), maxAttempts: 3 });
  for (let id = 0; id < DEAD_LETTER_LIMIT + 5; id += 1) scheduler.submit(job(id));
  for (let i = 0; i < 10; i += 1) {
    scheduler.tick();
    clock.advance(60000);
  }
  assert.strictEqual(scheduler.stats().dead, DEAD_LETTER_LIMIT + 5);
  assert.strictEqual(scheduler.deadLetters.length, DEAD_LETTER_LIMIT);
  assert.strictEqual(scheduler.stats().pending, 0);
});
