'use strict';

const test = require('node:test');
const assert = require('node:assert');
const { Relay } = require('../src/relay');

test('a connected device has its frames written to the store', () => {
  const lines = [];
  const relay = new Relay({ write: (device, line) => lines.push([device, line]) });
  relay.connect(4242);
  relay.deliver(4242, { kind: 1, device: 4242, value: 21 });
  assert.deepStrictEqual(lines, [[4242, 'temperature@4242=21']]);
});

test('frames for an unknown device are dropped', () => {
  const relay = new Relay({ write() {} });
  assert.strictEqual(relay.deliver(1, { kind: 1, device: 1, value: 1 }), 0);
});

test('a disconnected device writes nothing more', () => {
  const lines = [];
  const relay = new Relay({ write: (device, line) => lines.push(line) });
  relay.connect(9);
  assert.strictEqual(relay.disconnect(9), true);
  assert.strictEqual(relay.deliver(9, { kind: 2, device: 9, value: 3 }), 0);
  assert.deepStrictEqual(lines, []);
});
