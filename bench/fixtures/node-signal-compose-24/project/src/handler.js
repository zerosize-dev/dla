'use strict';

const lifecycle = require('./lifecycle');
const { Deadline } = require('./deadline');
const { readStatement } = require('./upstream');
const { csvLine, summarize } = require('./format');

// Serves one statement export. The export stops early when the process is
// terminating or when the request has used up its time budget.
async function handle(request) {
  const deadline = new Deadline(request.budgetMs);
  const scope = AbortSignal.any([lifecycle.signal, deadline.signal]);
  try {
    const lines = await readStatement(request.account, scope);
    const totals = summarize(lines);
    const body = lines.map(csvLine).join('\n');
    return { status: 200, account: request.account, totals, bytes: body.length };
  } catch (err) {
    if (scope.aborted) {
      return { status: 503, account: request.account, reason: String(scope.reason && scope.reason.message) };
    }
    throw err;
  } finally {
    deadline.release();
  }
}

module.exports = { handle };
