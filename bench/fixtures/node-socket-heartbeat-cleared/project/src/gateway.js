'use strict';

const { Metrics } = require('./metrics');
const { Router } = require('./router');
const { Session } = require('./session');
const { startHeartbeat } = require('./heartbeat');

const DEFAULT_HEARTBEAT_MS = 30000;

class Gateway {
  constructor(options = {}) {
    this.heartbeatMs = options.heartbeatMs || DEFAULT_HEARTBEAT_MS;
    this.metrics = new Metrics();
    this.router = new Router(this.metrics);
    this.sessions = new Map();
    this.nextId = 1;
  }

  accept(socket) {
    const id = `c${this.nextId++}`;
    const session = new Session(id, socket, this.router, this.metrics);
    this.sessions.set(id, session);
    session.heartbeat = startHeartbeat(session, this.heartbeatMs);
    socket.on('data', (chunk) => session.receive(chunk));
    socket.once('close', () => this.release(session));
    this.metrics.inc('accepted');
    return session;
  }

  release(session) {
    session.close('socket');
    this.sessions.delete(session.id);
    this.metrics.inc('released');
  }

  get size() {
    return this.sessions.size;
  }

  snapshot() {
    return { sessions: this.sessions.size, topics: this.router.topics.size, ...this.metrics.snapshot() };
  }
}

module.exports = { Gateway };
