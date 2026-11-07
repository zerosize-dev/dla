'use strict';

// Wire format: 4-byte big-endian length followed by a UTF-8 JSON body.

const HEADER_BYTES = 4;
const MAX_FRAME_BYTES = 64 * 1024;

function encode(message) {
  const body = Buffer.from(JSON.stringify(message), 'utf8');
  if (body.length > MAX_FRAME_BYTES) {
    throw new RangeError(`frame of ${body.length} bytes exceeds ${MAX_FRAME_BYTES}`);
  }
  const out = Buffer.allocUnsafe(HEADER_BYTES + body.length);
  out.writeUInt32BE(body.length, 0);
  body.copy(out, HEADER_BYTES);
  return out;
}

class Decoder {
  constructor() {
    this.pending = Buffer.alloc(0);
  }

  // Feed raw bytes, get back every complete message they finish.
  push(chunk) {
    this.pending = this.pending.length === 0 ? chunk : Buffer.concat([this.pending, chunk]);
    const messages = [];
    while (this.pending.length >= HEADER_BYTES) {
      const size = this.pending.readUInt32BE(0);
      if (size > MAX_FRAME_BYTES) {
        throw new RangeError(`peer announced a frame of ${size} bytes`);
      }
      if (this.pending.length < HEADER_BYTES + size) break;
      const body = this.pending.subarray(HEADER_BYTES, HEADER_BYTES + size);
      messages.push(JSON.parse(body.toString('utf8')));
      this.pending = this.pending.subarray(HEADER_BYTES + size);
    }
    return messages;
  }
}

module.exports = { encode, Decoder, MAX_FRAME_BYTES };
