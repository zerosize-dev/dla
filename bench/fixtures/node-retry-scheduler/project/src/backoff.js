'use strict';

// Exponential delay with deterministic jitter derived from the job id.
function delayFor(attempt, jobId, baseMs = 200, capMs = 30000) {
  const exp = Math.min(capMs, baseMs * 2 ** (attempt - 1));
  const jitter = (hash(`${jobId}:${attempt}`) % 100) / 100;
  return Math.round(exp * (0.5 + jitter / 2));
}

function hash(text) {
  let h = 2166136261;
  for (let i = 0; i < text.length; i += 1) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 16777619) >>> 0;
  }
  return h;
}

module.exports = { delayFor, hash };
