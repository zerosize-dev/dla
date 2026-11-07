'use strict';

const test = require('node:test');
const assert = require('node:assert');
const { encode, Decoder, MAX_FRAME_BYTES } = require('../src/frames');

test('round trip of a single frame', () => {
  const decoder = new Decoder();
  const out = decoder.push(encode({ type: 'hello', user: 'a' }));
  assert.deepStrictEqual(out, [{ type: 'hello', user: 'a' }]);
});

test('frames split across chunks are reassembled', () => {
  const decoder = new Decoder();
  const bytes = Buffer.concat([encode({ n: 1 }), encode({ n: 2 })]);
  assert.deepStrictEqual(decoder.push(bytes.subarray(0, 5)), []);
  assert.deepStrictEqual(decoder.push(bytes.subarray(5)), [{ n: 1 }, { n: 2 }]);
});

test('oversized frames are rejected', () => {
  assert.throws(() => encode({ blob: 'x'.repeat(MAX_FRAME_BYTES) }), RangeError);
  const decoder = new Decoder();
  const header = Buffer.alloc(4);
  header.writeUInt32BE(MAX_FRAME_BYTES + 1, 0);
  assert.throws(() => decoder.push(header), RangeError);
});
