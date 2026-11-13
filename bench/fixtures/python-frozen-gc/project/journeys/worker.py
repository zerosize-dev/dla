import gc

from . import planner
from .fares import FareTable


class BatchWorker:
    """Prices itineraries in batches, as one of several workers forked from a parent."""

    def __init__(self, table=None):
        self._table = table or FareTable()
        self.priced = 0
        self.batches = 0

    def run_batch(self, itineraries, band=0):
        quotes = [planner.quote(item, self._table, band) for item in itineraries]
        self.priced += len(quotes)
        self.batches += 1
        self._after_batch()
        return quotes

    def _after_batch(self):
        # Keep the pages of the fare table and the interned stop data shared
        # with the parent process: the collector must not touch them.
        gc.freeze()
