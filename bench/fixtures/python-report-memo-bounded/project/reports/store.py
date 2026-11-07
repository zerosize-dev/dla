import random

from .models import LedgerRow

CATEGORIES = (
    "groceries", "rent", "utilities", "transport", "dining",
    "insurance", "health", "travel", "subscriptions", "savings",
)

MEMO_WORDS = (
    "monthly", "invoice", "refund", "adjustment", "transfer", "statement",
    "recurring", "card", "settlement", "partial", "annual", "reminder",
)


class LedgerStore:
    """Produces the ledger rows of an account for one month.

    Rows are derived from the account number and the month so that every
    call returns the same data without any backing database.
    """

    def __init__(self, rows_per_month=28):
        self._rows_per_month = rows_per_month

    def fetch(self, account, month):
        rng = random.Random(account * 100003 + _month_index(month))
        rows = []
        for n in range(self._rows_per_month):
            category = rng.choice(CATEGORIES)
            amount = rng.randrange(-90_000, 250_000)
            memo = " ".join(rng.choice(MEMO_WORDS) for _ in range(4)) + f" #{n:03d}"
            rows.append(LedgerRow(account, month, category, amount, memo))
        return rows


def _month_index(month):
    year, mon = month.split("-")
    return int(year) * 12 + int(mon)
