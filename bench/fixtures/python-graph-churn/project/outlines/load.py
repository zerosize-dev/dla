"""Replays synthetic conversion traffic against the service.

    python3 -m outlines.load --iterations 5000
"""

import argparse
import random
import time

from .models import OutlineRequest
from .service import OutlineService

TOPICS = ("storage", "network", "billing", "search", "identity", "queues")


def make_outline(rng, sections=6):
    lines = []
    for s in range(sections):
        lines.append(f"- {rng.choice(TOPICS)} {s}")
        for t in range(rng.randrange(2, 5)):
            lines.append(f"  - topic {s}.{t}")
            for u in range(rng.randrange(1, 4)):
                lines.append(f"    - detail {s}.{t}.{u}")
    return "\n".join(lines)


def requests(count, seed=11):
    rng = random.Random(seed)
    for i in range(count):
        yield OutlineRequest(f"doc-{i}", make_outline(rng))


def run(iterations, service=None):
    service = service or OutlineService()
    digest = 0
    for request in requests(iterations):
        digest ^= service.convert(request).checksum
    return service, digest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iterations", type=int, default=2000)
    args = parser.parse_args()
    started = time.perf_counter()
    service, digest = run(args.iterations)
    print(f"converted {service.converted} outlines, digest {digest:08x}, "
          f"{time.perf_counter() - started:.2f}s")


if __name__ == "__main__":
    main()
