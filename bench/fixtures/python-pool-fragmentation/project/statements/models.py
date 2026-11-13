from dataclasses import dataclass


@dataclass(frozen=True)
class BatchSummary:
    """What the audit screen shows for one finished batch."""

    batch_id: int
    rows: int
    total_cents: int
    sample: str


@dataclass(frozen=True)
class AccountTotal:
    account: int
    count: int
    total_cents: int
