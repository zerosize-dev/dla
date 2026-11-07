import functools

from . import formatting


class ReportBuilder:
    """Renders ledger rows into text, one column buffer per output field."""

    def __init__(self, store, currency="USD"):
        self._store = store
        self._currency = currency
        self._columns = {"category": [], "amount": [], "memo": []}

    @functools.lru_cache(maxsize=128)
    def category_totals(self, account, month):
        totals = {}
        for row in self._store.fetch(account, month):
            totals[row.category] = totals.get(row.category, 0) + row.amount_cents
        return tuple(sorted(totals.items()))

    def render(self, account, month, style="summary"):
        for column in self._columns.values():
            column.clear()
        if style == "detail":
            self._fill_detail(account, month)
        else:
            self._fill_summary(account, month)
        lines = [formatting.title(account, month), "-" * 48]
        for category, amount, memo in zip(
            self._columns["category"], self._columns["amount"], self._columns["memo"]
        ):
            lines.append(f"{category} {amount} {memo}".rstrip())
        return "\n".join(lines)

    def _fill_summary(self, account, month):
        for category, cents in self.category_totals(account, month):
            self._columns["category"].append(formatting.pad(category, 14))
            self._columns["amount"].append(formatting.money(cents, self._currency).rjust(14))
            self._columns["memo"].append("")

    def _fill_detail(self, account, month):
        for row in self._store.fetch(account, month):
            self._columns["category"].append(formatting.pad(row.category, 14))
            self._columns["amount"].append(
                formatting.money(row.amount_cents, self._currency).rjust(14)
            )
            self._columns["memo"].append(row.memo)
