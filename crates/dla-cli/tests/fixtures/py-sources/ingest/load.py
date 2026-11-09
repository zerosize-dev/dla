"""Replays synthetic sensor traffic through the pipeline.

    python3 -m ingest.load --iterations 5000
"""

import argparse
import random
import time

from .pipeline import Pipeline

SENSORS = tuple(f"probe-{n:03d}" for n in range(64))
TAGS = ("indoor", "outdoor", "calibrated", "spare", "north", "south")
LINES_PER_BATCH = 2


def make_batches(count, seed=3):
    rng = random.Random(seed)
    clock = 1_700_000_000
    for _ in range(count):
        lines = []
        for _ in range(LINES_PER_BATCH):
            clock += rng.randrange(1, 5)
            tags = ";".join(rng.sample(TAGS, rng.randrange(1, 4)))
            lines.append(f"{clock},{rng.choice(SENSORS)},{rng.uniform(-20, 45):.2f},{tags}")
        yield lines


def run(iterations, pipeline=None):
    pipeline = pipeline or Pipeline()
    for lines in make_batches(iterations):
        pipeline.process_batch(lines)
    return pipeline


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iterations", type=int, default=2000)
    args = parser.parse_args()
    started = time.perf_counter()
    pipeline = run(args.iterations)
    print(f"ingested {pipeline.batches} batches, {pipeline.rows} rows, "
          f"{time.perf_counter() - started:.2f}s")


if __name__ == "__main__":
    main()
