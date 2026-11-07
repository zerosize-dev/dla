from dataclasses import dataclass


@dataclass(frozen=True)
class LedgerRow:
    account: int
    month: str
    category: str
    amount_cents: int
    memo: str


@dataclass(frozen=True)
class ReportRequest:
    account: int
    month: str
    style: str = "summary"


@dataclass(frozen=True)
class Response:
    account: int
    month: str
    body: str
    checksum: int
