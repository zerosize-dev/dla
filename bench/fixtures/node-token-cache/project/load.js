'use strict';

// Replays signed requests from a fixed set of clients. Every client keeps
// using its token until it expires, then asks the issuer for a new one.
// Usage: node load.js --iterations 20000 [--clients 24]

const { gateway, signer, clock } = require('./src/service');

const PATHS = ['/statements', '/transfers', '/profile'];
const TOKEN_LIFETIME = 120;

function parseArgs(argv) {
  const args = { iterations: 20000, clients: 24 };
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i].replace(/^--/, '');
    if (!(key in args)) throw new Error(`unknown option ${argv[i]}`);
    args[key] = Number.parseInt(argv[i + 1], 10);
  }
  return args;
}

function payload(i) {
  const line = `{"op":"transfer","seq":${i},"memo":"standing order ${i % 97}","amount":${(i * 37) % 5000}}`;
  return line.repeat(30);
}

// The gateway lives as long as the process, so every replay continues where the
// previous one stopped.
const clients = [];

function runWorkload(iterations, clientCount = 24) {
  while (clients.length < clientCount) clients.push({ id: `acct-${clients.length}`, token: null, expires: 0 });
  const counts = { 200: 0, 401: 0, 403: 0, 400: 0 };
  for (let i = 0; i < iterations; i += 1) {
    clock.now += 1;
    const client = clients[i % clientCount];
    if (clock.now >= client.expires) {
      const claims = { subject: client.id, scopes: ['read:statements', 'read:profile'], expiresAt: clock.now + TOKEN_LIFETIME };
      client.token = signer.sign(claims);
      client.expires = claims.expiresAt;
    }
    const result = gateway.authenticate({ token: client.token, path: PATHS[i % PATHS.length], body: payload(i) });
    counts[result.status] += 1;
  }
  return { counts, hits: gateway.cache.hits, misses: gateway.cache.misses, remembered: gateway.cache.size() };
}

if (require.main === module) {
  const args = parseArgs(process.argv.slice(2));
  const started = Date.now();
  const result = runWorkload(args.iterations, args.clients);
  const rss = Math.round(process.memoryUsage().rss / 1024);
  console.log(`ok=${result.counts[200]} denied=${result.counts[403]} hits=${result.hits} misses=${result.misses} rssKB=${rss} ms=${Date.now() - started}`);
}

module.exports = { runWorkload };
