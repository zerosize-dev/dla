'use strict';

// Process-wide stop switch. The termination hook trips it once; every request
// reads it so that in-flight exports wind down instead of being cut off.

const stop = new AbortController();

function onTerminate(reason) {
  if (!stop.signal.aborted) stop.abort(new Error(`terminating: ${reason}`));
}

function install(proc = process) {
  proc.once('SIGTERM', () => onTerminate('SIGTERM'));
  proc.once('SIGINT', () => onTerminate('SIGINT'));
}

module.exports = { signal: stop.signal, onTerminate, install };
