'use strict';

const { templateFor } = require('./cache');
const { GREETING, CLOSING } = require('./locales');

function money(cents) {
  return `${Math.floor(cents / 100)}.${String(cents % 100).padStart(2, '0')}`;
}

// Renders one notice for one customer in the customer's language.
function render(notice) {
  const template = templateFor(notice.kind, notice.locale);
  return template.render({
    greeting: GREETING[notice.locale],
    closing: CLOSING[notice.locale],
    customer: notice.customer,
    account: notice.account,
    amount: money(notice.cents),
    date: notice.date,
    effective: notice.effective,
  });
}

module.exports = { render, money };
