"""Replays synthetic statement batches against the importer.

    python3 -m statements.load --iterations 2000
"""

import argparse
import random
import time

from .importer import Importer

MEMO_WORDS = ("transfer", "standing order", "card payment", "refund", "interest",
              "fee", "salary", "direct debit", "reversal", "adjustment")


_rng = random.Random(4)
MEMOS = tuple(" ".join(_rng.choice(MEMO_WORDS) for _ in range(2000)) for _ in range(40))


def make_lines(rng, count):
    lines = []
    for i in range(count):
        memo = MEMOS[rng.randrange(len(MEMOS))]
        tags = ",".join(rng.choice(("a", "b", "c", "")) for _ in range(3))
        lines.append(f"{rng.randrange(1, 900)}|2024-{rng.randrange(1, 13):02d}-"
                     f"{rng.randrange(1, 29):02d}|{rng.randrange(-900000, 900000)}|{memo}|{tags}")
    return lines


def batches(count, seed=17):
    rng = random.Random(seed)
    for i in range(count):
        size = 60 + rng.randrange(0, 13) * 20
        yield i, make_lines(rng, size)


def run(iterations, importer=None):
    importer = importer or Importer()
    total = 0
    for batch_id, lines in batches(iterations):
        total ^= importer.import_batch(batch_id, lines).total_cents & 0xFFFFFF
    return importer, total


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iterations", type=int, default=300)
    args = parser.parse_args()
    started = time.perf_counter()
    importer, total = run(args.iterations)
    print(f"imported {importer.rows} rows in {importer.batches} batches, "
          f"checksum {total:06x}, {time.perf_counter() - started:.2f}s")


if __name__ == "__main__":
    main()
