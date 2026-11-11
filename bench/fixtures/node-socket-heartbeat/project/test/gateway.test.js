'use strict';

const test = require('node:test');
const assert = require('node:assert');
const { Gateway } = require('../src/gateway');
const { LoopbackSocket } = require('../src/loopback');
const { Decoder } = require('../src/frames');

function decodeAll(socket) {
  const decoder = new Decoder();
  return socket.written.flatMap((chunk) => decoder.push(chunk));
}

test('hello is answered with the session id', () => {
  const gateway = new Gateway();
  const socket = new LoopbackSocket();
  const session = gateway.accept(socket);
  socket.clientSend({ type: 'hello', user: 'unit-7' });
  assert.deepStrictEqual(decodeAll(socket), [{ type: 'welcome', id: session.id }]);
  socket.clientHangUp();
});

test('publish reaches other subscribers only', () => {
  const gateway = new Gateway();
  const a = new LoopbackSocket();
  const b = new LoopbackSocket();
  gateway.accept(a);
  gateway.accept(b);
  for (const s of [a, b]) {
    s.clientSend({ type: 'hello', user: s === a ? 'a' : 'b' });
    s.clientSend({ type: 'subscribe', topic: 'ops' });
  }
  a.clientSend({ type: 'publish', topic: 'ops', body: { ok: true } });
  const events = decodeAll(b).filter((m) => m.type === 'event');
  assert.strictEqual(events.length, 1);
  assert.deepStrictEqual(events[0].body, { ok: true });
  assert.strictEqual(decodeAll(a).filter((m) => m.type === 'event').length, 0);
  a.clientHangUp();
  b.clientHangUp();
});

test('hang up removes the session and its subscriptions', () => {
  const gateway = new Gateway();
  const socket = new LoopbackSocket();
  gateway.accept(socket);
  socket.clientSend({ type: 'subscribe', topic: 'ops' });
  assert.strictEqual(gateway.size, 1);
  assert.strictEqual(gateway.router.topics.size, 1);
  socket.clientHangUp();
  assert.strictEqual(gateway.size, 0);
  assert.strictEqual(gateway.router.topics.size, 0);
});

test('open sessions receive pings', (t) => {
  t.mock.timers.enable({ apis: ['setInterval', 'Date'] });
  const gateway = new Gateway({ heartbeatMs: 1000 });
  const socket = new LoopbackSocket();
  gateway.accept(socket);
  t.mock.timers.tick(1000);
  const pings = decodeAll(socket).filter((m) => m.type === 'ping');
  assert.strictEqual(pings.length, 1);
  socket.clientHangUp();
});

test('silent peers are dropped after three periods', (t) => {
  t.mock.timers.enable({ apis: ['setInterval', 'Date'] });
  const gateway = new Gateway({ heartbeatMs: 1000 });
  const socket = new LoopbackSocket();
  const session = gateway.accept(socket);
  t.mock.timers.tick(4000);
  assert.strictEqual(session.closed, true);
  assert.strictEqual(gateway.metrics.get('closed_idle'), 1);
});

test('a session the server ends is released once the socket has closed', async () => {
  const gateway = new Gateway();
  const socket = new LoopbackSocket();
  const session = gateway.accept(socket);
  socket.clientSend({ type: 'hello', user: 'unit-9' });
  session.close('idle');
  assert.strictEqual(gateway.size, 1);
  await new Promise((resolve) => setImmediate(resolve));
  assert.strictEqual(gateway.size, 0);
  assert.strictEqual(socket.destroyed, true);
});
