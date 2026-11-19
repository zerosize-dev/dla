import random
from compression import zstd

LEVELS = ("INFO", "WARN", "ERROR", "DEBUG")


def make_segment(rng, lines=60):
    text = "\n".join(
        f"2024-05-{rng.randrange(1, 29):02d}T{rng.randrange(24):02d}:{rng.randrange(60):02d} "
        f"{rng.choice(LEVELS)} request {rng.randrange(1_000_000)} took {rng.randrange(900)}ms"
        for _ in range(lines)
    )
    return zstd.compress(text.encode("utf-8"))


def split_lines(data):
    return data.decode("utf-8").split("\n")


def level_counts(lines):
    counts = {}
    for line in lines:
        level = line.split(" ")[1]
        counts[level] = counts.get(level, 0) + 1
    return counts
