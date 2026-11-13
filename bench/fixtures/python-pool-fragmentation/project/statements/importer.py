from collections import deque

from . import ledger
from .models import BatchSummary
from .parser import parse_batch

RECENT = 64


class Importer:
    """Imports statement batches and keeps the last few summaries for the audit screen."""

    def __init__(self, flag_limit_cents=500_000):
        self._flag_limit = flag_limit_cents
        self._recent = deque(maxlen=RECENT)
        self.batches = 0
        self.rows = 0
        self.flagged = 0

    def import_batch(self, batch_id, lines):
        rows = parse_batch(lines)
        accounts = ledger.totals_by_account(rows)
        flagged = ledger.flagged(rows, self._flag_limit)
        middle = rows[len(rows) // 2]
        summary = BatchSummary(
            batch_id,
            len(rows),
            sum(a.total_cents for a in accounts),
            middle["memo"] + f" [{middle['account']}]",
        )
        self._recent.append(summary)
        self.batches += 1
        self.rows += len(rows)
        self.flagged += len(flagged)
        return summary

    def recent(self):
        return list(self._recent)
