"""DLA loop driver. One process, one checkpoint record per round (x = units completed).

Fill the two DLA-HOOK bodies below and nothing else; keep the loop as it is.

Keys: DLA_KEY_MODE=fresh uses key = i (every unit sees a new key); DLA_KEY_MODE=repeated uses
key = i % repeat (a bounded key space). Rounds: DLA_WARMUP warm-up rounds, then DLA_ROUNDS
measured rounds of DLA_STEP units each (defaults are baked in at generation time).

Run from the scratch project root with the harness directory first on PYTHONPATH:

    PYTHONPATH=<harness dir>:. python <harness dir>/dla_harness.py
"""

import os

import sitecustomize as dla  # the DLA agent; writes the checkpoint records

STEP = int(os.environ.get("DLA_STEP") or "@@DLA_STEP@@")
WARMUP = int(os.environ.get("DLA_WARMUP") or "@@DLA_WARMUP@@")
ROUNDS = int(os.environ.get("DLA_ROUNDS") or "@@DLA_ROUNDS@@")
REPEAT = int(os.environ.get("DLA_REPEAT") or "@@DLA_REPEAT@@")
REPEATED = os.environ.get("DLA_KEY_MODE") == "repeated"

_state = {}


def setup():
    """Build the objects under test once, before the first round."""
    # DLA-HOOK setup: import the project modules and construct the services/caches/pools the workload needs; keep them in _state (or module globals) for unit()
    raise NotImplementedError("DLA-HOOK setup has not been filled in")


def unit(i, key):
    """One unit of work for `key` (i is the 1-based unit number)."""
    # DLA-HOOK unit: perform exactly one unit of the workload for this key (derive request ids/names/payloads from key so a repeated key reproduces an earlier request)
    raise NotImplementedError("DLA-HOOK unit has not been filled in")


def main():
    setup()
    total = WARMUP + ROUNDS
    done = 0
    for round_no in range(1, total + 1):
        for _ in range(STEP):
            done += 1
            unit(done, done % REPEAT if REPEATED else done)
        dla.checkpoint(done, last=round_no == total)


if __name__ == "__main__":
    main()
