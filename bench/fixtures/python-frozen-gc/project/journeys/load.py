"""Replays synthetic booking traffic against the worker.

    python3 -m journeys.load --iterations 5000
"""

import argparse
import random
import time

from .models import Itinerary
from .worker import BatchWorker

BATCH = 25


def itineraries(count, seed=3):
    rng = random.Random(seed)
    for i in range(count):
        length = rng.randrange(3, 9)
        yield Itinerary(i, tuple(rng.randrange(0, 240) for _ in range(length)))


def run(iterations, worker=None, batch=BATCH):
    worker = worker or BatchWorker()
    total = 0
    pending = []
    for item in itineraries(iterations):
        pending.append(item)
        if len(pending) == batch:
            total += sum(q.fare for q in worker.run_batch(pending, band=item.reference))
            pending = []
    if pending:
        total += sum(q.fare for q in worker.run_batch(pending))
    return worker, total


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iterations", type=int, default=2000)
    args = parser.parse_args()
    started = time.perf_counter()
    worker, total = run(args.iterations)
    print(f"priced {worker.priced} itineraries in {worker.batches} batches, "
          f"total {total}, {time.perf_counter() - started:.2f}s")


if __name__ == "__main__":
    main()
