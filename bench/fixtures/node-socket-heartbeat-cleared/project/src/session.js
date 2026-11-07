'use strict';

const { encode, Decoder } = require('./frames');

const RECENT_LIMIT = 64;

// One client connection: decodes frames, remembers the latest messages for
// diagnostics and tracks its topic memberships.
class Session {
  constructor(id, socket, router, metrics) {
    this.id = id;
    this.socket = socket;
    this.router = router;
    this.metrics = metrics;
    this.decoder = new Decoder();
    this.openedAt = Date.now();
    this.lastSeen = this.openedAt;
    this.user = null;
    this.topics = new Set();
    this.recent = [];
    this.heartbeat = null;
    this.closed = false;
  }

  receive(chunk) {
    if (this.closed) return;
    this.lastSeen = Date.now();
    let messages;
    try {
      messages = this.decoder.push(chunk);
    } catch (err) {
      this.metrics.inc('protocol_errors');
      this.close('protocol');
      return;
    }
    for (const message of messages) {
      this.metrics.inc('frames_in');
      this.remember(message);
      this.router.dispatch(this, message);
    }
  }

  remember(message) {
    this.recent.push(message);
    if (this.recent.length > RECENT_LIMIT) this.recent.shift();
  }

  send(message) {
    this.sendRaw(encode(message));
  }

  sendRaw(frame) {
    if (this.closed) return;
    this.socket.write(frame);
    this.metrics.inc('frames_out');
  }

  idleFor(now) {
    return now - this.lastSeen;
  }

  close(reason) {
    if (this.closed) return;
    this.closed = true;
    clearInterval(this.heartbeat);
    for (const topic of [...this.topics]) this.router.unsubscribe(this, topic);
    this.socket.destroy();
    this.metrics.inc(`closed_${reason}`);
  }

  describe() {
    return {
      id: this.id,
      user: this.user,
      topics: [...this.topics],
      heartbeat: this.heartbeat !== null,
      recent: this.recent.length,
      closed: this.closed,
    };
  }
}

module.exports = { Session, RECENT_LIMIT };
