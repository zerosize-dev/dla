'use strict';

const { EventEmitter } = require('node:events');
const { encode } = require('./frames');

// In-process stand-in for a TCP socket: bytes written by the server are kept
// for inspection, bytes "sent" by the client are emitted as data events.
class LoopbackSocket extends EventEmitter {
  constructor() {
    super();
    this.written = [];
    this.destroyed = false;
  }

  write(chunk) {
    if (this.destroyed) return false;
    this.written.push(chunk);
    return true;
  }

  destroy() {
    this.destroyed = true;
  }

  clientSend(message) {
    this.emit('data', encode(message));
  }

  clientHangUp() {
    this.emit('close');
  }
}

module.exports = { LoopbackSocket };
