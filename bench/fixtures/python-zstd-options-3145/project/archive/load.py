"""Replays synthetic read traffic against the segment reader.

    python3 -m archive.load --iterations 5000
"""

import argparse
import random
import time

from .reader import SegmentReader
from .segments import make_segment
from .tenants import make_tenants

SEGMENT_POOL = 48


def run(iterations, reader=None):
    reader = reader or SegmentReader()
    rng = random.Random(21)
    tenants = make_tenants(30)
    pool = [make_segment(rng) for _ in range(SEGMENT_POOL)]
    errors = 0
    for i in range(iterations):
        counts = reader.read(tenants[i % len(tenants)], pool[(i * 7) % len(pool)])
        errors += counts.get("ERROR", 0)
    return reader, errors


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iterations", type=int, default=2000)
    args = parser.parse_args()
    started = time.perf_counter()
    reader, errors = run(args.iterations)
    print(f"read {reader.segments} segments, {reader.lines} lines, {errors} errors, "
          f"{time.perf_counter() - started:.2f}s")


if __name__ == "__main__":
    main()
