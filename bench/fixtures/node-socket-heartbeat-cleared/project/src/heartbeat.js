'use strict';

const IDLE_PERIODS = 3;

// Pings a session on a fixed period and drops peers that stopped answering.
function startHeartbeat(session, periodMs) {
  const beat = () => {
    if (session.closed) return;
    if (session.idleFor(Date.now()) > periodMs * IDLE_PERIODS) {
      session.close('idle');
      return;
    }
    session.send({ type: 'ping', at: Date.now() });
  };
  const timer = setInterval(beat, periodMs);
  timer.unref();
  return timer;
}

module.exports = { startHeartbeat, IDLE_PERIODS };
