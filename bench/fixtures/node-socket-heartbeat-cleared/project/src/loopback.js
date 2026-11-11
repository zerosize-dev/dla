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
    this.closeEmitted = false;
  }

  write(chunk) {
    if (this.destroyed) return false;
    this.written.push(chunk);
    return true;
  }

  // Like net.Socket.destroy(): the 'close' event follows asynchronously, once.
  destroy() {
    this.destroyed = true;
    process.nextTick(() => this.emitClose());
  }

  emitClose() {
    if (this.closeEmitted) return;
    this.closeEmitted = true;
    this.emit('close');
  }

  clientSend(message) {
    this.emit('data', encode(message));
  }

  clientHangUp() {
    this.destroyed = true;
    this.emitClose();
  }
}

module.exports = { LoopbackSocket };
