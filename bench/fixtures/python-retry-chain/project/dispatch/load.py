"""Replays synthetic webhook traffic against the service.

    python3 -m dispatch.load --iterations 5000
"""

import argparse
import time

from .models import Delivery
from .service import DeliveryService
from .transport import SimulatedTransport

ENDPOINTS = tuple(f"https://hooks.example.test/tenant-{n:02d}" for n in range(12))
EVENTS = ("invoice.paid", "invoice.failed", "plan.changed", "seat.added")


def deliveries(count):
    for i in range(count):
        yield Delivery(
            delivery_id=i,
            endpoint=ENDPOINTS[(i * 5) % len(ENDPOINTS)],
            event=EVENTS[i % len(EVENTS)],
            fields={"account": 1000 + (i * 31) % 977, "amount": (i * 17) % 4000,
                    "note": "x" * (200 + (i % 7) * 40)},
        )


def run(iterations, service=None):
    service = service or DeliveryService(SimulatedTransport())
    checksum = 0
    for delivery in deliveries(iterations):
        checksum ^= service.handle(delivery).size
    return service, checksum


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iterations", type=int, default=2000)
    args = parser.parse_args()
    started = time.perf_counter()
    service, checksum = run(args.iterations)
    print(f"delivered {service.delivered} events in {service.attempts} attempts, "
          f"checksum {checksum:08x}, {time.perf_counter() - started:.2f}s")


if __name__ == "__main__":
    main()
