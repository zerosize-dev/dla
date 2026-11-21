"""DLA loop driver. One process, one checkpoint record per round (x = units completed).

Fill the two DLA-HOOK bodies below and nothing else; keep the loop as it is.

Keys: DLA_KEY_MODE=fresh uses key = i (every unit sees a new key); DLA_KEY_MODE=repeated uses
key = i % repeat (a bounded key space). Rounds: DLA_WARMUP warm-up rounds, then DLA_ROUNDS
measured rounds of DLA_STEP units each (defaults are baked in at generation time).

Run from the scratch project root with the harness directory first on PYTHONPATH:

    PYTHONPATH=<harness dir>:. python <harness dir>/dla_harness.py
"""

import os
import sys
import time

import sitecustomize as dla  # the DLA agent; writes the checkpoint records

STEP = int(os.environ.get("DLA_STEP") or "@@DLA_STEP@@")
WARMUP = int(os.environ.get("DLA_WARMUP") or "@@DLA_WARMUP@@")
ROUNDS = int(os.environ.get("DLA_ROUNDS") or "@@DLA_ROUNDS@@")
REPEAT = int(os.environ.get("DLA_REPEAT") or "@@DLA_REPEAT@@")
REPEATED = os.environ.get("DLA_KEY_MODE") == "repeated"

# Time limit of this process (seconds; `dla harness run` sets DLA_TIME_LIMIT_SECS from the harness
# timeout, 0 or unset = no fitting). The loop fits its rounds into it: once MIN_STEP units have run
# (inside the first round, so one slow round cannot outlast the limit before the first look) and
# after every round it knows the time per unit, and when the units left would not fit (margin: 15
# percent and 5 seconds for the final checkpoint and the exit) it lowers the step of the round in
# progress and of the rounds left, never below MIN_STEP. 10 units: with fewer, the per-round costs
# (the forced collection and the checkpoint) and the heap noise swamp a leak of one entry per
# unit. The x axis stays the cumulative unit count.
LIMIT_SECS = float(os.environ.get("DLA_TIME_LIMIT_SECS") or "0")
MIN_STEP = 10
_T0 = time.monotonic()

_state = {}


def setup():
    """Build the objects under test once, before the first round."""
    # DLA-HOOK setup: import the project modules and construct the services/caches/pools the workload needs; keep them in _state (or module globals) for unit()
    raise NotImplementedError("DLA-HOOK setup has not been filled in")


# DLA-CHOICE-BEGIN
_CHOICE_N = [0, 0, 0, 0, 0, 0, 0, 0]  # the n of each slot seen so far: the radix of the later slots and the report


def dla_choice(key, slot, n):
    """Choice number `slot` of `n` options, from the digits of `key` in mixed radix.

    Deterministic, no allocation and no state beyond the n of each slot (8 slots): the
    combinations of the slots used in order 0, 1, 2, ... all occur within n0*n1*... keys and
    repeat evenly. Call it with the same n for a slot in every unit.
    """
    div = 1
    j = 0
    while j < slot and j < 8:
        div *= _CHOICE_N[j] or 1
        j += 1
    if slot < 8:
        _CHOICE_N[slot] = n
    return (key // div) % n


def _choices_note(done):
    slots = 0
    product = 1
    for n in _CHOICE_N:
        if n:
            slots += 1
            product *= n
    if slots == 0:
        return None
    took = min(product, done)
    more = f" (of {product} possible)" if product > took else ""
    return f"the units took {took} combinations of {slots} choices{more}"


# DLA-CHOICE-END


def unit(i, key):
    """One unit of work for `key` (i is the 1-based unit number)."""
    # Choices: `dla_choice(key, slot, n)` is choice number `slot` of `n` options, taken from the digits of `key` in mixed radix (all combinations of the slots occur within n1*n2*... keys).
    # Write the unit as choices (depth, which object gets the ending call, when intermediates are released, the order of releases, success or failure), not as one fixed sequence.
    # DLA-HOOK unit: perform exactly one unit of the workload for this key (derive request ids/names/payloads from key so a repeated key reproduces an earlier request)
    raise NotImplementedError("DLA-HOOK unit has not been filled in")


def main():
    dla.mark_phase("hook setup (the project imports and the objects under test)")
    setup()
    total = WARMUP + ROUNDS
    loop_start = time.monotonic()
    step = STEP
    done = 0
    warned = False

    def fit(round_no, in_round):
        """Lower `step` when the units left would not fit; `in_round` units of `round_no` have run."""
        nonlocal step, warned
        if not LIMIT_SECS > 0 or done == 0:
            return
        left = total - round_no
        per_unit = (time.monotonic() - loop_start) / done
        budget = LIMIT_SECS * 0.85 - 5 - (time.monotonic() - _T0)
        need = per_unit * (step - in_round + step * left)
        if need <= budget:
            return
        units = int(budget / per_unit)
        lowered = max(min(MIN_STEP, step), min(step, (units + in_round) // (left + 1)))
        if lowered < step:
            dla.note(
                f"the driver lowered its step from {step} to {lowered} units per round in round {round_no} "
                f"(after {in_round} units of it): one unit takes {per_unit * 1000:.1f} ms and the {LIMIT_SECS:g} s "
                "time limit would cut the process (the x axis is still the cumulative unit count)"
            )
            step = lowered
        need = per_unit * (step - in_round + step * left)
        if not warned and need > budget:
            warned = True
            dla.note(
                f"the unit is too slow for a series: even {step} units per round (the floor) needs {need:.0f} s "
                f"for the units left and {max(0.0, budget):.0f} s remain; make the unit cheaper (one call, no sleeps, "
                "smaller payloads) or raise timeout_secs in harness.json"
            )

    for round_no in range(1, total + 1):
        dla.mark_phase(f"hook unit (round {round_no} of {total}, units {done + 1} to {done + step})")
        n = 0
        while n < step:
            n += 1
            done += 1
            unit(done, done % REPEAT if REPEATED else done)
            if n == min(MIN_STEP, STEP):
                fit(round_no, n)
        if round_no == total and (line := _choices_note(done)):
            dla.note(line)
        dla.checkpoint(done, last=round_no == total)
        if round_no < total:
            fit(round_no + 1, 0)


if __name__ == "__main__":
    # The process ends when the loop is done and the last checkpoint is written: non-daemon threads,
    # servers and sockets of the units must not hold it until the kill (os._exit skips the atexit
    # handlers; the agent's own exit record is not wanted after a driver's last checkpoint, and the
    # coverage rides on the checkpoint records).
    code = 0
    try:
        main()
    except SystemExit as exc:
        code = exc.code if isinstance(exc.code, int) else 1
    except BaseException:
        import traceback

        traceback.print_exc()
        code = 1
    try:
        sys.stdout.flush()
        sys.stderr.flush()
    finally:
        os._exit(code)
