"""Replays synthetic report traffic against the service.

    python3 -m reports.load --iterations 5000
"""

import argparse
import time

from .models import ReportRequest
from .service import ReportService
from .store import LedgerStore

MONTHS = tuple(f"2024-{m:02d}" for m in range(1, 13))
ACCOUNT_SPACE = 4000


def requests(count):
    for i in range(count):
        style = "detail" if i % 5 == 0 else "summary"
        yield ReportRequest((i * 7919) % ACCOUNT_SPACE + 1, MONTHS[i % len(MONTHS)], style)


def run(iterations, service=None):
    service = service or ReportService(LedgerStore())
    total = 0
    for request in requests(iterations):
        total ^= service.handle(request).checksum
    return service, total


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iterations", type=int, default=2000)
    args = parser.parse_args()
    started = time.perf_counter()
    service, total = run(args.iterations)
    elapsed = time.perf_counter() - started
    print(f"served {service.served} reports, digest {total:08x}, {elapsed:.2f}s")


if __name__ == "__main__":
    main()
