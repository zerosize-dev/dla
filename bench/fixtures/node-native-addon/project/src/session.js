'use strict';

const codec = require('@meridian/frame-codec');
const { describe } = require('./frames');

// One field-gateway connection. The codec decodes raw bytes natively and
// calls back with every decoded frame addressed to the device.
class Session {
  constructor(device, store, counters) {
    this.device = device;
    this.store = store;
    this.counters = counters;
    this.open = true;
    this.unsubscribe = codec.subscribe(device, (frame) => {
      if (!this.open) return;
      this.counters.record(frame.kind);
      this.store.write(this.device, describe(frame));
    });
  }

  feed(bytes) {
    return codec.decode(this.device, bytes);
  }

  close() {
    this.open = false;
  }
}

module.exports = { Session };
