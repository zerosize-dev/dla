'use strict';

// Frames from the field gateways: a header byte, a device id and a reading.

const KIND = { temperature: 1, pressure: 2, battery: 3 };

function describe(frame) {
  const kind = Object.keys(KIND).find((name) => KIND[name] === frame.kind) || 'unknown';
  return `${kind}@${frame.device}=${frame.value}`;
}

function checksum(bytes) {
  let sum = 0;
  for (const b of bytes) sum = (sum * 31 + b) & 0xffff;
  return sum;
}

function pack(frame) {
  const bytes = Buffer.alloc(8);
  bytes.writeUInt8(frame.kind, 0);
  bytes.writeUInt32BE(frame.device, 1);
  bytes.writeUInt16BE(frame.value & 0xffff, 5);
  bytes.writeUInt8(checksum(bytes.subarray(0, 7)) & 0xff, 7);
  return bytes;
}

module.exports = { KIND, describe, checksum, pack };
