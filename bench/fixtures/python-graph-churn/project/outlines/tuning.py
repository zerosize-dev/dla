"""Collector settings for the conversion workers."""

import gc

# Young-generation collections were showing up as latency spikes in the p99
# of conversions, so collections are triggered less often than the default.
YOUNG_THRESHOLD = 40_000


def install():
    gc.set_threshold(YOUNG_THRESHOLD, 20, 20)
