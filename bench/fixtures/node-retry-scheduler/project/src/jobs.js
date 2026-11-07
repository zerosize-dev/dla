'use strict';

// Builds the delivery jobs the scheduler works on.
function makeJob(id, random) {
  const rows = [];
  const count = 18 + Math.floor(random() * 8);
  for (let i = 0; i < count; i += 1) {
    rows.push({
      sku: `SKU-${Math.floor(random() * 90000) + 10000}`,
      qty: 1 + Math.floor(random() * 9),
      note: `line ${i} of order ${id}`,
    });
  }
  return {
    id,
    target: `https://hooks.example.test/tenant-${id % 17}/inbox`,
    payload: { order: id, rows },
  };
}

module.exports = { makeJob };
