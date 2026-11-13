"""Replays synthetic device traffic against the gateway.

    python3 -m ingestd.load --iterations 5000
"""

import argparse
import random
import struct
import time

from .gateway import Gateway
from .models import Packet
from .sink import ListSink

DEVICES = tuple(f"meter-{n:03d}" for n in range(40))


def packets(count, seed=9):
    rng = random.Random(seed)
    for i in range(count):
        channels = rng.randrange(4, 12)
        raw = struct.pack(f"<HH{channels}f", i & 0xFFFF, channels,
                          *(rng.random() * 100 for _ in range(channels)))
        yield Packet(i, DEVICES[i % len(DEVICES)], raw)


def run(iterations, gateway=None):
    gateway = gateway or Gateway(ListSink())
    for packet in packets(iterations):
        gateway.handle(packet)
    return gateway


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--iterations", type=int, default=2000)
    args = parser.parse_args()
    started = time.perf_counter()
    gateway = run(args.iterations)
    print(f"handled {gateway.packets} packets, {gateway.readings} readings, "
          f"{time.perf_counter() - started:.2f}s")


if __name__ == "__main__":
    main()
